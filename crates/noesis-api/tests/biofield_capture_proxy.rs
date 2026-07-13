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
    models::{
        biofield::{
            NewBiofieldBaseline, NewBiofieldCaptureArtifact, NewBiofieldSession,
            BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE,
        },
        reading::NewReading,
    },
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
        ) AND EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_name = 'biofield_baselines'
        ) AND EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_name = 'biofield_baseline_readings'
        ) AND EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_name = 'biofield_exports'
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

    let migration_017 =
        fs::read_to_string(repo_root().join("migrations/017_biofield_sessions.sql"))
            .expect("root migration 017");
    let migration_018 =
        fs::read_to_string(repo_root().join("migrations/018_biofield_baselines.sql"))
            .expect("root migration 018");
    let migration_019 = fs::read_to_string(repo_root().join("migrations/019_biofield_exports.sql"))
        .expect("root migration 019");

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
            ) AND EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'biofield_baselines'
            ) AND EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'biofield_baseline_readings'
            ) AND EXISTS (
                SELECT 1
                FROM information_schema.tables
                WHERE table_name = 'biofield_exports'
            )
            "#,
        )
        .fetch_one(pool)
        .await
        .expect("schema probe after lock should succeed");

        if schema_exists_after_lock {
            return;
        }

        sqlx::raw_sql(&migration_017)
            .execute(pool)
            .await
            .expect("biofield migration 017 should apply to test database");
        sqlx::raw_sql(&migration_018)
            .execute(pool)
            .await
            .expect("biofield migration 018 should apply to test database");
        sqlx::raw_sql(&migration_019)
            .execute(pool)
            .await
            .expect("biofield migration 019 should apply to test database");
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
        cf_access_issuer: None,
        cf_access_audience: None,
        cf_dev_bypass_token: None,
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

fn json_body(value: Value) -> Body {
    Body::from(serde_json::to_vec(&value).expect("json body should serialize"))
}

fn artifact_dir(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("biofield-artifacts-{label}-{}", Uuid::new_v4()))
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

        let mut orchestrator = WorkflowOrchestrator::new();
        orchestrator.register_engine(Arc::new(noesis_orchestrator::BiofieldCaptureEngine::new(
            pool.clone(),
        )));

        let state = AppState {
            orchestrator: Arc::new(orchestrator),
            bridge_manager: Arc::new(noesis_bridge::BridgeManager::from_env()),
            workflow_registry: None,
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
            readings_repository: Some(readings_repository.clone()),
            usage_repository: None,
            startup_time: Instant::now(),
            db_available: true,
            cf_access_validator: None,
            cf_dev_bypass_token: None,
            ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
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

        self.send_request(request).await
    }

    async fn send_authenticated_request(
        &self,
        method: &str,
        uri: &str,
        token: &str,
    ) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .expect("request should build");

        self.send_request(request).await
    }

    async fn send_authenticated_json(
        &self,
        method: &str,
        uri: &str,
        token: &str,
        payload: Value,
    ) -> (StatusCode, Value) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(json_body(payload))
            .expect("request should build");

        self.send_request(request).await
    }

    async fn send_request(&self, request: Request<Body>) -> (StatusCode, Value) {
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
        sqlx::query("DELETE FROM biofield_exports WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("cleanup biofield_exports should succeed");

        sqlx::query(
            r#"
            DELETE FROM biofield_baseline_readings
            WHERE baseline_id IN (
                SELECT id FROM biofield_baselines WHERE user_id = $1
            )
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .expect("cleanup biofield_baseline_readings should succeed");

        sqlx::query("DELETE FROM biofield_baselines WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("cleanup biofield_baselines should succeed");

        sqlx::query(
            r#"
            DELETE FROM biofield_capture_artifacts
            WHERE session_id IN (
                SELECT id FROM biofield_sessions WHERE user_id = $1
            ) OR reading_id IN (
                SELECT id FROM readings WHERE user_id = $1
            )
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .expect("cleanup biofield_capture_artifacts should succeed");

        sqlx::query("DELETE FROM biofield_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("cleanup biofield_sessions should succeed");

        sqlx::query("DELETE FROM progression_logs WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("cleanup progression_logs should succeed");

        sqlx::query("DELETE FROM history_sync_state WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("cleanup history_sync_state should succeed");

        sqlx::query("DELETE FROM readings WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("cleanup readings should succeed");

        sqlx::query("DELETE FROM user_devices WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("cleanup user_devices should succeed");

        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("test user cleanup should succeed");
    }
}

async fn create_persisted_biofield_reading(
    harness: &BiofieldCaptureHarness,
    user_id: Uuid,
    session_id: Uuid,
    label: &str,
    metrics: Value,
) -> Uuid {
    let reading_id = harness
        .readings_repository
        .save_reading(&NewReading {
            user_id,
            engine_id: "biofield-capture".to_string(),
            workflow_id: None,
            input_hash: format!("bf-{:.8}-{}", label, Uuid::new_v4().simple()),
            input_data: json!({
                "session_id": session_id,
                "content_type": "image/jpeg",
                "file_name": format!("{label}.jpg"),
                "capture_metadata": {
                    "platform": "test"
                }
            }),
            result_data: json!({
                "analysis_version": "stub-metrics/v1",
                "metrics": metrics,
                "quality_assessment": {
                    "sharpness": 0.88,
                    "contrast": 0.82,
                    "noise_level": 0.08,
                    "exposure": 0.54,
                    "sufficient_quality": true
                }
            }),
            witness_prompt: None,
            consciousness_level: 0,
            calculation_time_ms: Some(12.5),
            client_event_id: None,
            client_device_id: Some("test-device".to_string()),
            device_platform: Some("test".to_string()),
            device_app_version: Some("biofield-web/test".to_string()),
        })
        .await
        .expect("reading should be created");

    harness
        .biofield_repository
        .create_artifact(&NewBiofieldCaptureArtifact {
            session_id,
            reading_id: Some(reading_id),
            artifact_kind: BIOFIELD_CAPTURE_ARTIFACT_SOURCE_IMAGE.to_string(),
            storage_path: format!("biofield/{session_id}/{label}.jpg"),
            mime_type: "image/jpeg".to_string(),
            byte_size: 4_096,
            capture_metadata: json!({
                "file_name": format!("{label}.jpg"),
                "capture_metadata": {
                    "platform": "test"
                }
            }),
        })
        .await
        .expect("artifact should be created");

    reading_id
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

    let artifacts_dir = artifact_dir("success");
    let _env = EnvGuard::set(&[
        ("PYTHON_BIOFIELD_URL", sidecar.uri()),
        ("PYTHON_BIOFIELD_TIMEOUT_MS", "1000".to_string()),
        (
            "BIOFIELD_ARTIFACTS_DIR",
            artifacts_dir.display().to_string(),
        ),
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
    assert!(payload["artifacts"][0]["id"].as_str().is_some());
    assert!(payload["artifacts"][0]["storage_path"]
        .as_str()
        .expect("storage path should exist")
        .contains(&session_id.to_string()));

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

    let artifacts = harness
        .biofield_repository
        .list_reading_artifacts(reading_id, user_id)
        .await
        .expect("reading artifacts should load");
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].reading_id, Some(reading_id));
    assert_eq!(artifacts[0].session_id, session_id);
    assert_eq!(artifacts[0].artifact_kind, "source-image");
    assert!(artifacts[0].storage_path.contains(&session_id.to_string()));

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

    let artifacts_dir = artifact_dir("quality");
    let _env = EnvGuard::set(&[
        ("PYTHON_BIOFIELD_URL", sidecar.uri()),
        ("PYTHON_BIOFIELD_TIMEOUT_MS", "1000".to_string()),
        (
            "BIOFIELD_ARTIFACTS_DIR",
            artifacts_dir.display().to_string(),
        ),
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

    let artifacts = harness
        .biofield_repository
        .list_session_artifacts(session_id, user_id)
        .await
        .expect("session artifacts should load");
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].reading_id, None);
    assert_eq!(artifacts[0].artifact_kind, "source-image");

    harness.delete_user(user_id).await;
}

#[tokio::test]
#[serial]
async fn biofield_readings_history_and_detail_are_user_scoped() {
    let Some(harness) = BiofieldCaptureHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("history-owner").await;
    let other_user_id = harness.create_user_id("history-other").await;
    let session_id = harness.create_session_for_user(user_id).await;
    let other_session_id = harness.create_session_for_user(other_user_id).await;
    let token = harness.token_for_user(user_id);
    let other_token = harness.token_for_user(other_user_id);

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

    let artifacts_dir = artifact_dir("history");
    let _env = EnvGuard::set(&[
        ("PYTHON_BIOFIELD_URL", sidecar.uri()),
        ("PYTHON_BIOFIELD_TIMEOUT_MS", "1000".to_string()),
        (
            "BIOFIELD_ARTIFACTS_DIR",
            artifacts_dir.display().to_string(),
        ),
    ]);

    let (body, boundary) = build_multipart_body(vec![MultipartField {
        name: "image",
        body: b"\xff\xd8\xff\xe0history-jpeg".to_vec(),
        file_name: Some("history.jpg"),
        content_type: Some("image/jpeg"),
    }]);

    let (owner_status, owner_payload) = harness
        .send_authenticated_multipart(
            &format!("/api/v1/biofield/sessions/{session_id}/captures"),
            &token,
            body.clone(),
            &boundary,
        )
        .await;
    assert_eq!(owner_status, StatusCode::CREATED);

    let owner_reading_id = owner_payload["reading_id"]
        .as_str()
        .expect("owner reading id")
        .to_string();

    let (other_status, other_payload) = harness
        .send_authenticated_multipart(
            &format!("/api/v1/biofield/sessions/{other_session_id}/captures"),
            &other_token,
            body,
            &boundary,
        )
        .await;
    assert_eq!(other_status, StatusCode::CREATED);

    let other_reading_id = other_payload["reading_id"]
        .as_str()
        .expect("other reading id")
        .to_string();

    let (list_status, list_payload) = harness
        .send_authenticated_request("GET", "/api/v1/biofield/readings", &token)
        .await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(
        list_payload["items"].as_array().expect("items array").len(),
        1
    );
    assert_eq!(list_payload["items"][0]["reading_id"], owner_reading_id);
    assert_eq!(
        list_payload["items"][0]["session_id"],
        session_id.to_string()
    );
    assert_eq!(list_payload["items"][0]["artifact"]["kind"], "source-image");
    assert!(list_payload["items"][0]["artifact"]["storage_path"]
        .as_str()
        .is_some());

    let (detail_status, detail_payload) = harness
        .send_authenticated_request(
            "GET",
            &format!("/api/v1/biofield/readings/{owner_reading_id}"),
            &token,
        )
        .await;
    assert_eq!(detail_status, StatusCode::OK);
    assert_eq!(detail_payload["reading_id"], owner_reading_id);
    assert_eq!(detail_payload["session_id"], session_id.to_string());
    assert_eq!(detail_payload["quality"]["sufficient_quality"], true);
    assert_eq!(
        detail_payload["result"]["analysis_version"],
        "stub-metrics/v1"
    );
    assert_eq!(
        detail_payload["result"]["metrics"]["light_quanta_density"],
        42.5
    );
    assert!(detail_payload["artifacts"][0]["storage_path"]
        .as_str()
        .is_some());

    let (foreign_status, foreign_payload) = harness
        .send_authenticated_request(
            "GET",
            &format!("/api/v1/biofield/readings/{other_reading_id}"),
            &token,
        )
        .await;
    assert_eq!(foreign_status, StatusCode::NOT_FOUND);
    assert_eq!(foreign_payload["error_code"], "BIOFIELD_READING_NOT_FOUND");

    harness.delete_user(other_user_id).await;
    harness.delete_user(user_id).await;
}

#[tokio::test]
#[serial]
async fn biofield_reprocess_uses_stored_source_artifact_and_creates_new_reading() {
    let Some(harness) = BiofieldCaptureHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("reprocess").await;
    let session_id = harness.create_session_for_user(user_id).await;
    let token = harness.token_for_user(user_id);

    let sidecar = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/analyze"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "contract_version": "biofield-cv/v1",
            "analysis_version": "stub-metrics/v1",
            "metrics": {
                "light_quanta_density": 51.5,
                "normalized_area": 0.74,
                "average_intensity": 0.62,
                "inner_noise": 0.07,
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

    let artifacts_dir = artifact_dir("reprocess");
    let _env = EnvGuard::set(&[
        ("PYTHON_BIOFIELD_URL", sidecar.uri()),
        ("PYTHON_BIOFIELD_TIMEOUT_MS", "1000".to_string()),
        (
            "BIOFIELD_ARTIFACTS_DIR",
            artifacts_dir.display().to_string(),
        ),
    ]);

    let (body, boundary) = build_multipart_body(vec![MultipartField {
        name: "image",
        body: b"\xff\xd8\xff\xe0reprocess-jpeg".to_vec(),
        file_name: Some("reprocess.jpg"),
        content_type: Some("image/jpeg"),
    }]);

    let (capture_status, capture_payload) = harness
        .send_authenticated_multipart(
            &format!("/api/v1/biofield/sessions/{session_id}/captures"),
            &token,
            body,
            &boundary,
        )
        .await;
    assert_eq!(capture_status, StatusCode::CREATED);

    let original_reading_id = capture_payload["reading_id"]
        .as_str()
        .expect("original reading id")
        .to_string();
    let original_storage_path = capture_payload["artifacts"][0]["storage_path"]
        .as_str()
        .expect("original storage path")
        .to_string();
    assert!(artifacts_dir.join(&original_storage_path).exists());

    let (reprocess_status, reprocess_payload) = harness
        .send_authenticated_json(
            "POST",
            &format!("/api/v1/biofield/readings/{original_reading_id}/reprocess"),
            &token,
            json!({}),
        )
        .await;
    assert_eq!(reprocess_status, StatusCode::CREATED);
    assert_eq!(reprocess_payload["source_reading_id"], original_reading_id);

    let reprocessed_reading_id = reprocess_payload["reading_id"]
        .as_str()
        .expect("reprocessed reading id")
        .to_string();
    assert_ne!(reprocessed_reading_id, original_reading_id);
    let reprocessed_storage_path = reprocess_payload["artifacts"][0]["storage_path"]
        .as_str()
        .expect("reprocessed storage path")
        .to_string();
    assert!(artifacts_dir.join(&reprocessed_storage_path).exists());

    let reprocessed_reading_uuid = Uuid::parse_str(&reprocessed_reading_id).expect("uuid");
    let reprocessed_reading = harness
        .readings_repository
        .get_reading(reprocessed_reading_uuid, user_id)
        .await
        .expect("reading fetch should succeed")
        .expect("reprocessed reading should exist");
    assert_eq!(
        reprocessed_reading.input_data["reprocessed_from_reading_id"],
        original_reading_id
    );

    let artifacts = harness
        .biofield_repository
        .list_reading_artifacts(reprocessed_reading_uuid, user_id)
        .await
        .expect("reprocessed artifacts should load");
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].session_id, session_id);
    assert_eq!(artifacts[0].reading_id, Some(reprocessed_reading_uuid));

    harness.delete_user(user_id).await;
    let _ = fs::remove_dir_all(artifacts_dir);
}

#[tokio::test]
#[serial]
async fn biofield_baselines_create_and_list_are_user_scoped() {
    let Some(harness) = BiofieldCaptureHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("baseline-owner").await;
    let other_user_id = harness.create_user_id("baseline-other").await;
    let session_id = harness.create_session_for_user(user_id).await;
    let other_session_id = harness.create_session_for_user(other_user_id).await;
    let token = harness.token_for_user(user_id);
    let other_token = harness.token_for_user(other_user_id);

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

    let artifacts_dir = artifact_dir("baseline");
    let _env = EnvGuard::set(&[
        ("PYTHON_BIOFIELD_URL", sidecar.uri()),
        ("PYTHON_BIOFIELD_TIMEOUT_MS", "1000".to_string()),
        (
            "BIOFIELD_ARTIFACTS_DIR",
            artifacts_dir.display().to_string(),
        ),
    ]);

    let (body, boundary) = build_multipart_body(vec![MultipartField {
        name: "image",
        body: b"\xff\xd8\xff\xe0baseline-jpeg".to_vec(),
        file_name: Some("baseline.jpg"),
        content_type: Some("image/jpeg"),
    }]);

    let mut owner_reading_ids = Vec::new();
    for _ in 0..2 {
        let (status, payload) = harness
            .send_authenticated_multipart(
                &format!("/api/v1/biofield/sessions/{session_id}/captures"),
                &token,
                body.clone(),
                &boundary,
            )
            .await;
        assert_eq!(status, StatusCode::CREATED);
        owner_reading_ids.push(
            payload["reading_id"]
                .as_str()
                .expect("owner reading id")
                .to_string(),
        );
    }

    let (other_status, other_payload) = harness
        .send_authenticated_multipart(
            &format!("/api/v1/biofield/sessions/{other_session_id}/captures"),
            &other_token,
            body,
            &boundary,
        )
        .await;
    assert_eq!(other_status, StatusCode::CREATED);
    let other_reading_id = other_payload["reading_id"]
        .as_str()
        .expect("other reading id")
        .to_string();

    let (create_status, create_payload) = harness
        .send_authenticated_json(
            "POST",
            "/api/v1/biofield/baselines",
            &token,
            json!({
                "name": "Morning baseline",
                "notes": "Two accepted captures",
                "reading_ids": owner_reading_ids,
            }),
        )
        .await;
    assert_eq!(create_status, StatusCode::CREATED);
    assert_eq!(create_payload["name"], "Morning baseline");
    assert_eq!(create_payload["reading_count"], 2);

    let (list_status, list_payload) = harness
        .send_authenticated_request("GET", "/api/v1/biofield/baselines", &token)
        .await;
    assert_eq!(list_status, StatusCode::OK);
    assert_eq!(list_payload["items"].as_array().expect("items").len(), 1);
    assert_eq!(list_payload["items"][0]["name"], "Morning baseline");
    assert_eq!(list_payload["items"][0]["reading_count"], 2);

    let (other_list_status, other_list_payload) = harness
        .send_authenticated_request("GET", "/api/v1/biofield/baselines", &other_token)
        .await;
    assert_eq!(other_list_status, StatusCode::OK);
    assert_eq!(
        other_list_payload["items"].as_array().expect("items").len(),
        0
    );

    let (cross_user_status, cross_user_payload) = harness
        .send_authenticated_json(
            "POST",
            "/api/v1/biofield/baselines",
            &token,
            json!({
                "name": "Bad baseline",
                "reading_ids": [other_reading_id],
            }),
        )
        .await;
    assert_eq!(cross_user_status, StatusCode::NOT_FOUND);
    assert_eq!(
        cross_user_payload["error_code"],
        "BIOFIELD_READING_NOT_FOUND"
    );

    harness.delete_user(other_user_id).await;
    harness.delete_user(user_id).await;
    let _ = fs::remove_dir_all(artifacts_dir);
}

#[tokio::test]
#[serial]
async fn biofield_reading_detail_supports_baseline_comparison() {
    let Some(harness) = BiofieldCaptureHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("comparison-owner").await;
    let session_id = harness.create_session_for_user(user_id).await;
    let token = harness.token_for_user(user_id);

    let primary_reading_id = create_persisted_biofield_reading(
        &harness,
        user_id,
        session_id,
        "comparison-primary",
        json!({
            "light_quanta_density": 50.0,
            "normalized_area": 0.8,
            "average_intensity": 0.55,
            "energy_analysis": {
                "low": 0.2,
                "medium": 0.5,
                "high": 0.3,
                "total": 1.0
            }
        }),
    )
    .await;
    let baseline_reading_id = create_persisted_biofield_reading(
        &harness,
        user_id,
        session_id,
        "comparison-baseline",
        json!({
            "light_quanta_density": 30.0,
            "normalized_area": 0.5,
            "average_intensity": 0.4,
            "energy_analysis": {
                "low": 0.1,
                "medium": 0.6,
                "high": 0.3,
                "total": 1.0
            }
        }),
    )
    .await;

    let baseline = harness
        .biofield_repository
        .create_baseline(
            &NewBiofieldBaseline {
                user_id,
                name: "Reference baseline".to_string(),
                notes: Some("Built for BF3 comparison".to_string()),
            },
            &[baseline_reading_id],
        )
        .await
        .expect("baseline should be created");

    let (status, payload) = harness
        .send_authenticated_request(
            "GET",
            &format!(
                "/api/v1/biofield/readings/{}?baseline_id={}",
                primary_reading_id, baseline.id
            ),
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        payload["comparison"]["comparison_version"],
        "biofield-baseline-delta/v1"
    );
    assert_eq!(
        payload["comparison"]["baseline"]["baseline_id"],
        baseline.id.to_string()
    );
    let delta = payload["comparison"]["deltas"]
        .as_array()
        .expect("comparison deltas array")
        .iter()
        .find(|item| item["key"] == "light_quanta_density")
        .expect("light_quanta_density delta should exist");
    assert_eq!(delta["reading_value"], 50.0);
    assert_eq!(delta["baseline_value"], 30.0);
    assert_eq!(delta["absolute_delta"], 20.0);

    harness.delete_user(user_id).await;
}

#[tokio::test]
#[serial]
async fn biofield_capture_engine_route_resolves_owner_reading() {
    let Some(harness) = BiofieldCaptureHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("engine-route").await;
    let session_id = harness.create_session_for_user(user_id).await;
    let reading_id = create_persisted_biofield_reading(
        &harness,
        user_id,
        session_id,
        "engine-route",
        json!({
            "light_quanta_density": 48.0,
            "normalized_area": 0.62,
            "average_intensity": 172.0,
            "fractal_dimension": 1.41,
            "body_symmetry": 0.77,
            "pattern_regularity": 0.58
        }),
    )
    .await;
    let token = harness.token_for_user(user_id);
    let reading_count_before = harness
        .readings_repository
        .count_readings(user_id, Some("biofield-capture"))
        .await
        .expect("reading count before route call should succeed");

    let (status, payload) = harness
        .send_authenticated_json(
            "POST",
            "/api/v1/engines/biofield-capture/calculate",
            &token,
            json!({
                "options": {
                    "biofield_capture": {
                        "reading_id": reading_id
                    }
                }
            }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(payload["engine_id"], "biofield-capture");
    assert_eq!(payload["result"]["reading_id"], reading_id.to_string());
    assert_eq!(
        payload["result"]["analysis"]["analysis_version"],
        "stub-metrics/v1"
    );
    assert_eq!(
        payload["result"]["quality_assessment"]["sufficient_quality"],
        true
    );
    assert!(payload["witness_prompt"].as_str().unwrap_or_default().len() > 8);

    let reading_count_after = harness
        .readings_repository
        .count_readings(user_id, Some("biofield-capture"))
        .await
        .expect("reading count after route call should succeed");
    assert_eq!(reading_count_before, 1);
    assert_eq!(reading_count_after, reading_count_before);

    sqlx::query("DELETE FROM biofield_capture_artifacts WHERE session_id = $1")
        .bind(session_id)
        .execute(&harness.pool)
        .await
        .expect("cleanup artifacts");
    sqlx::query("DELETE FROM biofield_sessions WHERE id = $1")
        .bind(session_id)
        .execute(&harness.pool)
        .await
        .expect("cleanup session");
    sqlx::query("DELETE FROM readings WHERE id = $1")
        .bind(reading_id)
        .execute(&harness.pool)
        .await
        .expect("cleanup reading");
    harness.delete_user(user_id).await;
}

#[tokio::test]
#[serial]
async fn biofield_exports_persist_bundle_and_enforce_user_scope() {
    let Some(harness) = BiofieldCaptureHarness::new().await else {
        return;
    };

    let user_id = harness.create_user_id("export-owner").await;
    let other_user_id = harness.create_user_id("export-other").await;
    let session_id = harness.create_session_for_user(user_id).await;
    let other_session_id = harness.create_session_for_user(other_user_id).await;
    let token = harness.token_for_user(user_id);
    let artifacts_dir = artifact_dir("export");
    let _env = EnvGuard::set(&[(
        "BIOFIELD_ARTIFACTS_DIR",
        artifacts_dir.display().to_string(),
    )]);

    let reading_id = create_persisted_biofield_reading(
        &harness,
        user_id,
        session_id,
        "export-reading",
        json!({
            "light_quanta_density": 42.5,
            "normalized_area": 0.67,
            "average_intensity": 0.51,
            "energy_analysis": {
                "low": 0.2,
                "medium": 0.5,
                "high": 0.3,
                "total": 1.0
            }
        }),
    )
    .await;
    let baseline_reading_id = create_persisted_biofield_reading(
        &harness,
        user_id,
        session_id,
        "export-baseline",
        json!({
            "light_quanta_density": 40.0,
            "normalized_area": 0.6,
            "average_intensity": 0.49,
            "energy_analysis": {
                "low": 0.25,
                "medium": 0.45,
                "high": 0.3,
                "total": 1.0
            }
        }),
    )
    .await;
    let baseline = harness
        .biofield_repository
        .create_baseline(
            &NewBiofieldBaseline {
                user_id,
                name: "Export baseline".to_string(),
                notes: Some("Used for BF3 export proof".to_string()),
            },
            &[baseline_reading_id],
        )
        .await
        .expect("baseline should be created");

    let other_reading_id = create_persisted_biofield_reading(
        &harness,
        other_user_id,
        other_session_id,
        "export-foreign",
        json!({
            "light_quanta_density": 10.0,
            "normalized_area": 0.2,
            "average_intensity": 0.1,
            "energy_analysis": {
                "low": 0.4,
                "medium": 0.4,
                "high": 0.2,
                "total": 1.0
            }
        }),
    )
    .await;

    let (status, payload) = harness
        .send_authenticated_json(
            "POST",
            "/api/v1/biofield/exports",
            &token,
            json!({
                "reading_id": reading_id,
                "baseline_id": baseline.id,
                "format": "json",
            }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(payload["reading_id"], reading_id.to_string());
    assert_eq!(payload["baseline_id"], baseline.id.to_string());
    assert_eq!(payload["format"], "json");
    assert_eq!(payload["bundle"]["contract_version"], "biofield-export/v1");
    assert_eq!(
        payload["bundle"]["reading"]["comparison"]["baseline"]["baseline_id"],
        baseline.id.to_string()
    );

    let export_id = Uuid::parse_str(payload["export_id"].as_str().expect("export id string"))
        .expect("valid export uuid");
    let storage_path = payload["storage_path"]
        .as_str()
        .expect("storage path should exist")
        .to_string();
    assert!(artifacts_dir.join(&storage_path).exists());

    let export_record = harness
        .biofield_repository
        .get_export(export_id, user_id)
        .await
        .expect("export lookup should succeed")
        .expect("export should exist for owner");
    assert_eq!(export_record.reading_id, reading_id);
    assert_eq!(export_record.baseline_id, Some(baseline.id));
    assert_eq!(export_record.export_format, "json");

    let foreign_export = harness
        .biofield_repository
        .get_export(export_id, other_user_id)
        .await
        .expect("foreign export lookup should not error");
    assert!(foreign_export.is_none());

    let (cross_status, cross_payload) = harness
        .send_authenticated_json(
            "POST",
            "/api/v1/biofield/exports",
            &token,
            json!({
                "reading_id": other_reading_id,
                "format": "json",
            }),
        )
        .await;
    assert_eq!(cross_status, StatusCode::NOT_FOUND);
    assert_eq!(cross_payload["error_code"], "BIOFIELD_READING_NOT_FOUND");

    harness.delete_user(other_user_id).await;
    harness.delete_user(user_id).await;
    let _ = fs::remove_dir_all(artifacts_dir);
}
