mod common;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use noesis_api::{create_router, shared_metrics, ApiConfig, AppState};
use noesis_auth::AuthService;
use noesis_cache::CacheManager;
use noesis_data::{
    models::biofield::NewBiofieldSession,
    repositories::{biofield_repository::BiofieldRepository, user_repository::UserRepository},
};
use noesis_orchestrator::WorkflowOrchestrator;
use serde_json::{json, Value};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use std::{
    fs,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
use tower::ServiceExt;
use uuid::Uuid;

const BIOFIELD_SCHEMA_LOCK_ID: i64 = 20_260_405_017;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crate dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

async fn ensure_biofield_schema(pool: &PgPool) {
    let schema_exists: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_name = 'biofield_sessions'
        ) AND EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_name = 'biofield_capture_artifacts'
        )
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("schema probe should succeed");

    if schema_exists {
        return;
    }

    sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(BIOFIELD_SCHEMA_LOCK_ID)
        .execute(pool)
        .await
        .expect("schema lock should succeed");

    let migration_sql =
        fs::read_to_string(repo_root().join("migrations/017_biofield_sessions.sql"))
            .expect("root migration 017");

    let apply_result = async {
        let schema_exists_after_lock: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'biofield_sessions'
            ) AND EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'biofield_capture_artifacts'
            )
            "#,
        )
        .fetch_one(pool)
        .await
        .expect("schema probe after lock should succeed");

        if schema_exists_after_lock {
            return;
        }

        sqlx::raw_sql(&migration_sql)
            .execute(pool)
            .await
            .expect("biofield migration should apply to test database");
    }
    .await;

    sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(BIOFIELD_SCHEMA_LOCK_ID)
        .execute(pool)
        .await
        .expect("schema unlock should succeed");

    apply_result
}

fn api_config(database_url: String) -> ApiConfig {
    ApiConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        jwt_secret: common::TEST_JWT_SECRET.to_string(),
        database_url: Some(database_url),
        redis_url: None,
        allowed_origins: vec![],
        rate_limit_requests: 100,
        rate_limit_window_secs: 60,
        request_timeout_secs: 30,
        log_level: "info".to_string(),
        log_format: "pretty".to_string(),
        discord_client_id: None,
        discord_client_secret: None,
        discord_redirect_uri: None,
        dodo_payments_api_key: None,
        dodo_payments_webhook_key: None,
        dodo_payments_env: None,
        python_biofield_url: "http://localhost:8002".to_string(),
        python_biofield_timeout_ms: 10_000,
        gateway_url: None,
        gateway_token: None,
    }
}

struct BiofieldSessionHarness {
    router: Router,
    pool: PgPool,
    user_repository: Arc<UserRepository>,
    biofield_repository: Arc<BiofieldRepository>,
}

impl BiofieldSessionHarness {
    async fn new() -> Option<Self> {
        let database_url =
            match std::env::var("DATABASE_URL").or_else(|_| std::env::var("TEST_DATABASE_URL")) {
                Ok(url) => url,
                Err(_) => {
                    eprintln!("Skipping biofield API integration test: DATABASE_URL not set");
                    return None;
                }
            };

        let connect_options = match database_url.parse::<PgConnectOptions>() {
            Ok(opts) => opts.statement_cache_capacity(0),
            Err(err) => {
                eprintln!("Skipping biofield API integration test: invalid DATABASE_URL: {err}");
                return None;
            }
        };

        let pool = match PgPoolOptions::new()
            .max_connections(2)
            .connect_with(connect_options)
            .await
        {
            Ok(pool) => pool,
            Err(err) => {
                eprintln!("Skipping biofield API integration test: could not connect: {err}");
                return None;
            }
        };

        ensure_biofield_schema(&pool).await;

        let config = api_config(database_url);
        let user_repository = Arc::new(UserRepository::new(pool.clone()));
        let biofield_repository = Arc::new(BiofieldRepository::new(pool.clone()));

        let state = AppState {
            orchestrator: Arc::new(WorkflowOrchestrator::new()),
            bridge_manager: Arc::new(noesis_bridge::BridgeManager::from_env()),
            cache: Arc::new(CacheManager::new(
                String::new(),
                100,
                Duration::from_secs(3600),
                false,
            )),
            auth: Arc::new(AuthService::new(common::TEST_JWT_SECRET.to_string())),
            metrics: shared_metrics(),
            user_repository: user_repository.clone(),
            admin_repository: None,
            billing_repository: None,
            biofield_repository: Some(biofield_repository.clone()),
            readings_repository: None,
            usage_repository: None,
            oauth_repository: None,
            startup_time: Instant::now(),
            db_available: true,
            discord_client_id: None,
            discord_client_secret: None,
            discord_redirect_uri: None,
            ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
        };

        let router = create_router(state, &config);

        Some(Self {
            router,
            pool,
            user_repository,
            biofield_repository,
        })
    }

    async fn create_user_id(&self, label: &str) -> Uuid {
        let email = format!("biofield-api-{label}-{}@example.com", Uuid::new_v4());
        self.user_repository
            .create_user(&email, "test_password_hash", "Biofield API Test User")
            .await
            .expect("test user should be created")
            .id
    }

    fn token_for_user(&self, user_id: Uuid) -> String {
        AuthService::new(common::TEST_JWT_SECRET.to_string())
            .generate_jwt_token(
                &user_id.to_string(),
                "premium",
                &["read".to_string(), "write".to_string()],
                5,
            )
            .expect("test jwt should generate")
    }

    async fn send_authenticated_json(
        &self,
        method_name: &str,
        uri: &str,
        token: &str,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let request_builder = Request::builder()
            .method(method_name)
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header(header::CONTENT_TYPE, "application/json");

        let body = if let Some(json_body) = body {
            Body::from(serde_json::to_vec(&json_body).expect("json body should serialize"))
        } else {
            Body::empty()
        };

        let response = self
            .router
            .clone()
            .oneshot(request_builder.body(body).expect("request should build"))
            .await
            .expect("router request should succeed");

        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body should read");
        let json = if body_bytes.is_empty() {
            json!({})
        } else {
            serde_json::from_slice(&body_bytes).expect("response should be valid json")
        };

        (status, json)
    }

    async fn delete_user(&self, user_id: Uuid) {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("test user cleanup should succeed");
    }
}

#[tokio::test]
async fn biofield_session_create_returns_created_resource() {
    let Some(harness) = BiofieldSessionHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("create").await;
    let token = harness.token_for_user(user_id);

    let (status, body) = harness
        .send_authenticated_json(
            "POST",
            "/api/v1/biofield/sessions",
            &token,
            Some(json!({
                "client_device_id": "desktop-browser-1",
                "viewer_version": "web-0.1.0",
                "context": {"platform": "macos"}
            })),
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["status"], "active");
    assert_eq!(body["client_device_id"], "desktop-browser-1");
    assert_eq!(body["viewer_version"], "web-0.1.0");
    assert!(body["closed_at"].is_null());

    let session_id = Uuid::parse_str(body["id"].as_str().expect("session id string"))
        .expect("valid session uuid");
    let persisted = harness
        .biofield_repository
        .get_session(session_id, user_id)
        .await
        .expect("session fetch should succeed")
        .expect("session should exist");

    assert_eq!(persisted.id, session_id);
    assert_eq!(
        persisted.client_device_id.as_deref(),
        Some("desktop-browser-1")
    );
    assert_eq!(persisted.viewer_version.as_deref(), Some("web-0.1.0"));
    assert!(persisted.closed_at.is_none());

    harness.delete_user(user_id).await;
}

#[tokio::test]
async fn biofield_session_get_is_user_scoped() {
    let Some(harness) = BiofieldSessionHarness::new().await else {
        return;
    };

    let owner_id = harness.create_user_id("owner").await;
    let other_id = harness.create_user_id("other").await;
    let owner_token = harness.token_for_user(owner_id);
    let other_token = harness.token_for_user(other_id);

    let session = harness
        .biofield_repository
        .create_session(&NewBiofieldSession::new(owner_id))
        .await
        .expect("session should be created");

    let (owner_status, owner_body) = harness
        .send_authenticated_json(
            "GET",
            &format!("/api/v1/biofield/sessions/{}", session.id),
            &owner_token,
            None,
        )
        .await;
    assert_eq!(owner_status, StatusCode::OK);
    assert_eq!(owner_body["id"], session.id.to_string());
    assert_eq!(owner_body["status"], "active");

    let (other_status, other_body) = harness
        .send_authenticated_json(
            "GET",
            &format!("/api/v1/biofield/sessions/{}", session.id),
            &other_token,
            None,
        )
        .await;
    assert_eq!(other_status, StatusCode::NOT_FOUND);
    assert_eq!(other_body["error_code"], "BIOFIELD_SESSION_NOT_FOUND");

    harness.delete_user(owner_id).await;
    harness.delete_user(other_id).await;
}

#[tokio::test]
async fn biofield_session_close_updates_status_and_timestamp() {
    let Some(harness) = BiofieldSessionHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("close").await;
    let token = harness.token_for_user(user_id);
    let session = harness
        .biofield_repository
        .create_session(&NewBiofieldSession::new(user_id))
        .await
        .expect("session should be created");

    let (status, body) = harness
        .send_authenticated_json(
            "POST",
            &format!("/api/v1/biofield/sessions/{}/close", session.id),
            &token,
            Some(json!({ "reason": "user-left" })),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], session.id.to_string());
    assert_eq!(body["status"], "closed");
    assert!(body["closed_at"].as_str().is_some());

    let persisted = harness
        .biofield_repository
        .get_session(session.id, user_id)
        .await
        .expect("session fetch should succeed")
        .expect("session should exist");
    assert_eq!(persisted.status, "closed");
    assert!(persisted.closed_at.is_some());

    harness.delete_user(user_id).await;
}

#[tokio::test]
async fn biofield_session_close_rejects_missing_and_non_active_sessions() {
    let Some(harness) = BiofieldSessionHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("rejects").await;
    let token = harness.token_for_user(user_id);

    let missing_id = Uuid::new_v4();
    let (missing_status, missing_body) = harness
        .send_authenticated_json(
            "POST",
            &format!("/api/v1/biofield/sessions/{missing_id}/close"),
            &token,
            Some(json!({})),
        )
        .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_body["error_code"], "BIOFIELD_SESSION_NOT_FOUND");

    let session = harness
        .biofield_repository
        .create_session(&NewBiofieldSession::new(user_id))
        .await
        .expect("session should be created");
    harness
        .biofield_repository
        .close_session(session.id, user_id)
        .await
        .expect("direct close should succeed");

    let (closed_status, closed_body) = harness
        .send_authenticated_json(
            "POST",
            &format!("/api/v1/biofield/sessions/{}/close", session.id),
            &token,
            Some(json!({ "reason": "already-closed" })),
        )
        .await;
    assert_eq!(closed_status, StatusCode::CONFLICT);
    assert_eq!(closed_body["error_code"], "BIOFIELD_SESSION_NOT_ACTIVE");

    harness.delete_user(user_id).await;
}
