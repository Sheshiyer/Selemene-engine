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
    repositories::{
        biofield_repository::BiofieldRepository, readings_repository::ReadingsRepository,
        user_repository::UserRepository,
    },
};
use noesis_orchestrator::WorkflowOrchestrator;
use serde_json::{json, Value};
use serial_test::serial;
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
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

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

struct EnvGuard {
    saved: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn set(vars: &[(&'static str, String)]) -> Self {
        let mut saved = Vec::with_capacity(vars.len());
        for (key, value) in vars {
            saved.push((*key, std::env::var(key).ok()));
            std::env::set_var(key, value);
        }
        Self { saved }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, prior) in self.saved.drain(..).rev() {
            if let Some(value) = prior {
                std::env::set_var(key, value);
            } else {
                std::env::remove_var(key);
            }
        }
    }
}

struct MultipartField<'a> {
    name: &'a str,
    body: Vec<u8>,
    file_name: Option<&'a str>,
    content_type: Option<&'a str>,
}

fn build_multipart_body(fields: Vec<MultipartField<'_>>) -> (Vec<u8>, String) {
    let boundary = format!("----bf1-{}", Uuid::new_v4().simple());
    let mut body = Vec::new();

    for field in fields {
        body.extend(format!("--{}\r\n", boundary).as_bytes());
        if let Some(file_name) = field.file_name {
            body.extend(
                format!(
                    "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                    field.name, file_name
                )
                .as_bytes(),
            );
            if let Some(content_type) = field.content_type {
                body.extend(format!("Content-Type: {}\r\n", content_type).as_bytes());
            }
            body.extend(b"\r\n");
            body.extend(&field.body);
            body.extend(b"\r\n");
        } else {
            body.extend(
                format!(
                    "Content-Disposition: form-data; name=\"{}\"\r\n\r\n",
                    field.name
                )
                .as_bytes(),
            );
            body.extend(&field.body);
            body.extend(b"\r\n");
        }
    }

    body.extend(format!("--{}--\r\n", boundary).as_bytes());
    (body, boundary)
}

struct BiofieldCaptureHarness {
    router: Router,
    pool: PgPool,
    user_repository: Arc<UserRepository>,
    biofield_repository: Arc<BiofieldRepository>,
    readings_repository: Arc<ReadingsRepository>,
}

impl BiofieldCaptureHarness {
    async fn new() -> Option<Self> {
        let database_url =
            match std::env::var("DATABASE_URL").or_else(|_| std::env::var("TEST_DATABASE_URL")) {
                Ok(url) => url,
                Err(_) => {
                    eprintln!("Skipping biofield capture integration test: DATABASE_URL not set");
                    return None;
                }
            };

        let connect_options = match database_url.parse::<PgConnectOptions>() {
            Ok(opts) => opts.statement_cache_capacity(0),
            Err(err) => {
                eprintln!(
                    "Skipping biofield capture integration test: invalid DATABASE_URL: {err}"
                );
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
                eprintln!("Skipping biofield capture integration test: could not connect: {err}");
                return None;
            }
        };

        ensure_biofield_schema(&pool).await;

        let config = api_config(database_url);
        let user_repository = Arc::new(UserRepository::new(pool.clone()));
        let biofield_repository = Arc::new(BiofieldRepository::new(pool.clone()));
        let readings_repository = Arc::new(ReadingsRepository::new(pool.clone()));

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
            biofield_repository: Some(biofield_repository.clone()),
            readings_repository: Some(readings_repository.clone()),
            usage_repository: None,
            oauth_repository: None,
            startup_time: Instant::now(),
            db_available: true,
            discord_client_id: None,
            discord_client_secret: None,
            discord_redirect_uri: None,
        };

        let router = create_router(state, &config);

        Some(Self {
            router,
            pool,
            user_repository,
            biofield_repository,
            readings_repository,
        })
    }

    async fn create_user_id(&self, label: &str) -> Uuid {
        let email = format!("biofield-capture-{label}-{}@example.com", Uuid::new_v4());
        self.user_repository
            .create_user(&email, "test_password_hash", "Biofield Capture Test User")
            .await
            .expect("test user should be created")
            .id
    }

    async fn create_session_for_user(&self, user_id: Uuid) -> Uuid {
        self.biofield_repository
            .create_session(&NewBiofieldSession::new(user_id))
            .await
            .expect("session should be created")
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

    async fn send_authenticated_multipart(
        &self,
        uri: &str,
        token: &str,
        body: Vec<u8>,
        boundary: &str,
    ) -> (StatusCode, Value) {
        let request = Request::builder()
            .method("POST")
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header(
                header::CONTENT_TYPE,
                format!("multipart/form-data; boundary={boundary}"),
            )
            .body(Body::from(body))
            .expect("request should build");

        let response = self
            .router
            .clone()
            .oneshot(request)
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
#[serial]
async fn biofield_capture_rejects_missing_image_field() {
    let Some(harness) = BiofieldCaptureHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("missing-image").await;
    let session_id = harness.create_session_for_user(user_id).await;
    let token = harness.token_for_user(user_id);

    let (body, boundary) = build_multipart_body(vec![MultipartField {
        name: "options",
        body: br#"{"mode":"capture"}"#.to_vec(),
        file_name: None,
        content_type: None,
    }]);

    let (status, payload) = harness
        .send_authenticated_multipart(
            &format!("/api/v1/biofield/sessions/{session_id}/captures"),
            &token,
            body,
            &boundary,
        )
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(payload["error_code"], "BIOFIELD_CAPTURE_INVALID");

    harness.delete_user(user_id).await;
}

#[tokio::test]
#[serial]
async fn biofield_capture_rejects_missing_or_inactive_sessions() {
    let Some(harness) = BiofieldCaptureHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("session-errors").await;
    let token = harness.token_for_user(user_id);

    let (body, boundary) = build_multipart_body(vec![MultipartField {
        name: "image",
        body: b"\xff\xd8\xff\xe0fake-jpeg".to_vec(),
        file_name: Some("capture.jpg"),
        content_type: Some("image/jpeg"),
    }]);

    let missing_id = Uuid::new_v4();
    let (missing_status, missing_payload) = harness
        .send_authenticated_multipart(
            &format!("/api/v1/biofield/sessions/{missing_id}/captures"),
            &token,
            body.clone(),
            &boundary,
        )
        .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_eq!(missing_payload["error_code"], "BIOFIELD_SESSION_NOT_FOUND");

    let active_session_id = harness.create_session_for_user(user_id).await;
    harness
        .biofield_repository
        .close_session(active_session_id, user_id)
        .await
        .expect("session close should succeed");

    let (inactive_status, inactive_payload) = harness
        .send_authenticated_multipart(
            &format!("/api/v1/biofield/sessions/{active_session_id}/captures"),
            &token,
            body,
            &boundary,
        )
        .await;
    assert_eq!(inactive_status, StatusCode::CONFLICT);
    assert_eq!(
        inactive_payload["error_code"],
        "BIOFIELD_SESSION_NOT_ACTIVE"
    );

    harness.delete_user(user_id).await;
}

#[tokio::test]
#[serial]
async fn biofield_capture_proxies_to_sidecar_and_persists_reading() {
    let Some(harness) = BiofieldCaptureHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("success").await;
    let session_id = harness.create_session_for_user(user_id).await;
    let token = harness.token_for_user(user_id);

    let sidecar = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/analyze"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "contract_version": "biofield-cv/v1",
            "analysis_version": "stub-metrics/v1",
            "metrics": {
                "light_quanta_density": 42.5,
                "normalized_area": 0.67,
                "average_intensity": 0.51,
                "inner_noise": 0.11,
                "energy_analysis": {
                    "low": 0.2,
                    "medium": 0.5,
                    "high": 0.3,
                    "total": 1.0
                },
                "entropy_form_coefficient": 2.7,
                "fractal_dimension": 1.42,
                "correlation_dimension": 1.95,
                "body_symmetry": 0.33,
                "contour_complexity": 1.8,
                "pattern_regularity": 0.76
            },
            "quality_assessment": {
                "sharpness": 0.88,
                "contrast": 0.82,
                "noise_level": 0.08,
                "exposure": 0.54,
                "sufficient_quality": true
            },
            "algorithms_run": ["fractal_dimension"],
            "processing_time_ms": 12.5
        })))
        .mount(&sidecar)
        .await;

    let _env = EnvGuard::set(&[
        ("PYTHON_BIOFIELD_URL", sidecar.uri()),
        ("PYTHON_BIOFIELD_TIMEOUT_MS", "1000".to_string()),
    ]);

    let (body, boundary) = build_multipart_body(vec![
        MultipartField {
            name: "image",
            body: b"\xff\xd8\xff\xe0fake-jpeg".to_vec(),
            file_name: Some("capture.jpg"),
            content_type: Some("image/jpeg"),
        },
        MultipartField {
            name: "algorithms",
            body: br#"["fractal_dimension"]"#.to_vec(),
            file_name: None,
            content_type: None,
        },
        MultipartField {
            name: "options",
            body: br#"{"mode":"capture"}"#.to_vec(),
            file_name: None,
            content_type: None,
        },
        MultipartField {
            name: "capture_metadata",
            body: br#"{"platform":"web","viewport":{"width":1440,"height":900}}"#.to_vec(),
            file_name: None,
            content_type: None,
        },
    ]);

    let (status, payload) = harness
        .send_authenticated_multipart(
            &format!("/api/v1/biofield/sessions/{session_id}/captures"),
            &token,
            body,
            &boundary,
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(payload["session_id"], session_id.to_string());
    assert_eq!(payload["analysis_version"], "stub-metrics/v1");
    assert_eq!(payload["quality_assessment"]["sufficient_quality"], true);
    assert_eq!(payload["artifacts"][0]["kind"], "source-image");
    assert_eq!(payload["artifacts"][0]["mime_type"], "image/jpeg");

    let reading_id = Uuid::parse_str(payload["reading_id"].as_str().expect("reading id string"))
        .expect("valid reading uuid");
    let reading = harness
        .readings_repository
        .get_reading(reading_id, user_id)
        .await
        .expect("reading fetch should succeed")
        .expect("reading should exist");

    assert_eq!(reading.engine_id, "biofield-capture");
    assert_eq!(reading.input_data["session_id"], session_id.to_string());
    assert_eq!(reading.result_data["analysis_version"], "stub-metrics/v1");

    harness.delete_user(user_id).await;
}

#[tokio::test]
#[serial]
async fn biofield_capture_normalizes_quality_rejection() {
    let Some(harness) = BiofieldCaptureHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("quality").await;
    let session_id = harness.create_session_for_user(user_id).await;
    let token = harness.token_for_user(user_id);

    let sidecar = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/analyze"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "contract_version": "biofield-cv/v1",
            "analysis_version": "stub-metrics/v1",
            "metrics": {
                "light_quanta_density": 12.0,
                "normalized_area": 0.2,
                "average_intensity": 0.18,
                "inner_noise": 0.25,
                "energy_analysis": {
                    "low": 0.5,
                    "medium": 0.35,
                    "high": 0.15,
                    "total": 1.0
                },
                "entropy_form_coefficient": 1.8,
                "fractal_dimension": 1.1,
                "correlation_dimension": 0.9,
                "body_symmetry": 0.1,
                "contour_complexity": 0.8,
                "pattern_regularity": 0.2
            },
            "quality_assessment": {
                "sharpness": 0.2,
                "contrast": 0.25,
                "noise_level": 0.3,
                "exposure": 0.1,
                "sufficient_quality": false
            },
            "algorithms_run": ["fractal_dimension"],
            "processing_time_ms": 7.5
        })))
        .mount(&sidecar)
        .await;

    let _env = EnvGuard::set(&[
        ("PYTHON_BIOFIELD_URL", sidecar.uri()),
        ("PYTHON_BIOFIELD_TIMEOUT_MS", "1000".to_string()),
    ]);

    let (body, boundary) = build_multipart_body(vec![MultipartField {
        name: "image",
        body: b"\xff\xd8\xff\xe0tiny-jpeg".to_vec(),
        file_name: Some("tiny.jpg"),
        content_type: Some("image/jpeg"),
    }]);

    let (status, payload) = harness
        .send_authenticated_multipart(
            &format!("/api/v1/biofield/sessions/{session_id}/captures"),
            &token,
            body,
            &boundary,
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(payload["error_code"], "BIOFIELD_CAPTURE_REJECTED_QUALITY");
    assert_eq!(
        payload["details"]["quality_assessment"]["sufficient_quality"],
        false
    );

    harness.delete_user(user_id).await;
}
