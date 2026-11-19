//! Separate test process initializes the production OnceLock exactly once with
//! distinct configured audiences, before any request can cache missing config.
use axum::{
    body::Body,
    http::{Request, StatusCode},
};

use http_body_util::BodyExt;
use noesis_api::desktop_auth::s256_challenge;
use noesis_api::{create_router, shared_metrics, ApiConfig, AppState};
use noesis_auth::AuthService;
use noesis_cache::CacheManager;
use noesis_data::repositories::user_repository::UserRepository;
use noesis_orchestrator::WorkflowOrchestrator;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "desktop-auth-jwt-secret-at-least-32-chars-long";
const DEV_TOKEN: &str = "desktop-auth-dev-bypass-token";
const STATE: &str = "state-0123456789abcdef";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres@localhost/noesis_test".to_string())
}

async fn app() -> (axum::Router, PgPool) {
    std::env::set_var("RUST_ENV", "development");
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url())
        .await
        .expect("connect to local Postgres (run scripts/cloud/test-db.sh)");

    let fixture_pool = pool.clone();
    let pool = if let Ok(url) = std::env::var("TEST_SHARED_RUNTIME_URL") {
        PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .unwrap()
    } else {
        pool
    };
    let config = ApiConfig {
        host: "127.0.0.1".into(),
        port: 0,
        jwt_secret: JWT_SECRET.into(),
        database_url: Some(database_url()),
        redis_url: None,
        allowed_origins: vec![],
        rate_limit_requests: 1000,
        rate_limit_window_secs: 60,
        request_timeout_secs: 30,
        log_level: "info".into(),
        log_format: "pretty".into(),
        cf_access_issuer: None,
        cf_access_audience: None,
        cf_dev_bypass_token: Some(DEV_TOKEN.into()),
        dodo_payments_api_key: None,
        dodo_payments_webhook_key: None,
        dodo_payments_env: None,
        python_biofield_url: "http://localhost:8002".into(),
        python_biofield_timeout_ms: 10_000,
        gateway_url: None,
        gateway_token: None,
    };
    let mut orchestrator = WorkflowOrchestrator::new();
    orchestrator.register_engine(Arc::new(engine_biorhythm::BiorhythmEngine::new()));
    let state = AppState {
        orchestrator: Arc::new(orchestrator),
        bridge_manager: Arc::new(noesis_bridge::BridgeManager::from_env()),
        workflow_registry: None,
        cache: Arc::new(CacheManager::new(
            String::new(),
            100,
            Duration::from_secs(60),
            false,
        )),
        auth: Arc::new(AuthService::with_pool(
            JWT_SECRET.into(),
            Some(pool.clone()),
        )),
        metrics: shared_metrics(),
        user_repository: Arc::new(UserRepository::new(pool.clone())),
        admin_repository: None,
        billing_repository: None,
        biofield_repository: None,
        readings_repository: None,
        usage_repository: None,
        db_available: true,
        cf_access_validator: std::env::var("TEST_SHARED_CF_ISSUER").ok().map(|issuer| {
            Arc::new(noesis_api::cf_access::CfAccessValidator::new(
                issuer,
                "admin-test-audience".into(),
            ))
        }),
        cf_dev_bypass_token: Some(DEV_TOKEN.into()),
        startup_time: Instant::now(),
        ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
    };
    (create_router(state, &config), fixture_pool)
}

#[tokio::test]
async fn nested_router_keeps_end_user_and_admin_audiences_separate() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };
    let dir = tempfile::tempdir().unwrap();
    let key = dir.path().join("synthetic.pem");
    assert!(std::process::Command::new("openssl")
        .args(["genrsa", "-out"])
        .arg(&key)
        .arg("2048")
        .output()
        .unwrap()
        .status
        .success());
    let modulus = std::process::Command::new("openssl")
        .args(["rsa", "-in"])
        .arg(&key)
        .args(["-modulus", "-noout"])
        .output()
        .unwrap();
    assert!(modulus.status.success());
    let text = String::from_utf8(modulus.stdout).unwrap();
    let hex = text.trim().strip_prefix("Modulus=").unwrap();
    let n: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    let server = MockServer::start().await;
    Mock::given(method("GET")).and(path("/cdn-cgi/access/certs")).respond_with(ResponseTemplate::new(200).set_body_json(json!({"keys":[{"kid":"synthetic","kty":"RSA","n":URL_SAFE_NO_PAD.encode(n),"e":"AQAB"}]}))).mount(&server).await;
    std::env::set_var("TEST_SHARED_CF_ISSUER", server.uri());
    std::env::set_var("CF_END_USER_ISSUER", server.uri());
    std::env::set_var("CF_END_USER_AUDIENCE", "end-user-test-audience");
    let (app, pool) = app().await;
    std::env::remove_var("TEST_SHARED_CF_ISSUER");
    let key = jsonwebtoken::EncodingKey::from_rsa_pem(&std::fs::read(key).unwrap()).unwrap();
    let proof = |email: &str, audience: &str| {
        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some("synthetic".into());
        jsonwebtoken::encode(&header,&json!({"iss":server.uri(),"sub":format!("sub-{email}"),"email":email,"aud":[audience],"iat":chrono::Utc::now().timestamp(),"exp":chrono::Utc::now().timestamp()+300}),&key).unwrap()
    };

    let email = format!("audience-{}@example.test", Uuid::new_v4());
    let authorize = format!("/api/v1/sessions/authorize?client_id=antahkarana&redirect_uri=http%3A%2F%2F127.0.0.1%3A53682%2Fcallback&state={STATE}&code_challenge={}&code_challenge_method=S256", s256_challenge(VERIFIER));
    for (path, audience, expected) in [
        (
            authorize.as_str(),
            "end-user-test-audience",
            StatusCode::SEE_OTHER,
        ),
        (
            authorize.as_str(),
            "admin-test-audience",
            StatusCode::UNAUTHORIZED,
        ),
        (
            authorize.as_str(),
            "wrong-audience",
            StatusCode::UNAUTHORIZED,
        ),
        (
            "/api/v1/admin/session",
            "end-user-test-audience",
            StatusCode::UNAUTHORIZED,
        ),
        (
            "/api/v1/admin/session",
            "wrong-audience",
            StatusCode::UNAUTHORIZED,
        ),
        (
            "/api/v1/admin/session",
            "admin-test-audience",
            StatusCode::OK,
        ),
        (
            "/api/v1/users/me/context",
            "end-user-test-audience",
            StatusCode::OK,
        ),
        (
            "/api/v1/users/me/context",
            "admin-test-audience",
            StatusCode::UNAUTHORIZED,
        ),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::get(path)
                    .header("cf-access-jwt-assertion", proof(&email, audience))
                    .header("x-original-uri", "/api/v1/sessions/authorize")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let location = response.headers().get("location").cloned();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(
            status,
            expected,
            "{path} {audience}: {}",
            String::from_utf8_lossy(&body)
        );
        if status == StatusCode::SEE_OTHER {
            let redirect = reqwest::Url::parse(location.unwrap().to_str().unwrap()).unwrap();
            let code = redirect
                .query_pairs()
                .find(|(key, _)| key == "code")
                .unwrap()
                .1
                .into_owned();
            let persisted: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM registered_auth_codes WHERE code_hash=$1)",
            )
            .bind(noesis_api::desktop_auth::hash_secret(&code))
            .fetch_one(&pool)
            .await
            .unwrap();
            assert!(persisted);
        }
    }
}
