//! Desktop sign-in (PKCE loopback flow) against a real Postgres.
//!
//! Requires Postgres at $DATABASE_URL with migrations through 042 applied
//! (`bash scripts/cloud/test-db.sh`). The browser identity is simulated with
//! the development bypass (`RUST_ENV=development` + `x-noesis-dev-auth`), which
//! the middleware treats as an interactive principal exactly like Cloudflare
//! Access.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use chrono::{Duration as ChronoDuration, Utc};
use http_body_util::BodyExt;
use noesis_api::desktop_auth::{hash_secret, new_secret, s256_challenge};
use noesis_api::{create_router, shared_metrics, ApiConfig, AppState};
use noesis_auth::AuthService;
use noesis_cache::CacheManager;
use noesis_data::repositories::desktop_session_repository::{
    DesktopGrant, DesktopSessionRepository,
};
use noesis_data::repositories::user_repository::UserRepository;
use noesis_orchestrator::WorkflowOrchestrator;
use serde_json::{json, Value};
use serial_test::serial;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "desktop-auth-jwt-secret-at-least-32-chars-long";
const DEV_TOKEN: &str = "desktop-auth-dev-bypass-token";
const REDIRECT: &str = "http://127.0.0.1:53682/callback";
const STATE: &str = "state-0123456789abcdef";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const DEV_USER: &str = "00000000-0000-0000-0000-000000000001";

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
    let state = AppState {
        orchestrator: Arc::new(WorkflowOrchestrator::new()),
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
        cf_access_validator: None,
        cf_dev_bypass_token: Some(DEV_TOKEN.into()),
        startup_time: Instant::now(),
        ephemeris_checksums: Arc::new(std::collections::HashMap::new()),
    };
    (create_router(state, &config), pool)
}

fn authorize_uri(challenge: &str, redirect: &str) -> String {
    format!(
        "/api/v1/desktop/authorize?code_challenge={challenge}&code_challenge_method=S256&redirect_uri={}&state={STATE}",
        urlencoding::encode(redirect)
    )
}

async fn send(router: &axum::Router, req: Request<Body>) -> (StatusCode, Option<String>, Value) {
    let res = router.clone().oneshot(req).await.expect("request");
    let status = res.status();
    let location = res
        .headers()
        .get(header::LOCATION)
        .map(|v| v.to_str().unwrap().to_string());
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, location, body)
}

fn browser_get(uri: &str) -> Request<Body> {
    Request::get(uri)
        .header("x-noesis-dev-auth", DEV_TOKEN)
        .body(Body::empty())
        .unwrap()
}

fn post_json(uri: &str, body: Value) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// Run authorize and return the issued code.
async fn obtain_code(router: &axum::Router) -> String {
    let (status, location, _) = send(
        router,
        browser_get(&authorize_uri(&s256_challenge(VERIFIER), REDIRECT)),
    )
    .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let location = location.expect("redirect location");
    let prefix = format!("{REDIRECT}?code=");
    assert!(
        location.starts_with(&prefix),
        "unexpected redirect {location}"
    );
    assert!(location.ends_with(&format!("&state={STATE}")));
    location[prefix.len()..location.len() - format!("&state={STATE}").len()].to_string()
}

async fn exchange(
    router: &axum::Router,
    code: &str,
    verifier: &str,
    redirect: &str,
) -> (StatusCode, Value) {
    let (status, _, body) = send(
        router,
        post_json(
            "/api/v1/desktop/token",
            json!({"grant_type": "authorization_code", "code": code, "code_verifier": verifier, "redirect_uri": redirect}),
        ),
    )
    .await;
    (status, body)
}

async fn refresh(router: &axum::Router, refresh_token: &str) -> (StatusCode, Value) {
    let (status, _, body) = send(
        router,
        post_json(
            "/api/v1/desktop/token",
            json!({"grant_type": "refresh_token", "refresh_token": refresh_token}),
        ),
    )
    .await;
    (status, body)
}

#[tokio::test]
#[serial]
async fn browser_sign_in_issues_a_scoped_desktop_session() {
    let (router, _) = app().await;
    let code = obtain_code(&router).await;
    let (status, body) = exchange(&router, &code, VERIFIER, REDIRECT).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["expires_in"], 3600);
    assert_eq!(body["refresh_expires_in"], 30 * 86_400);

    // The access token is a Selemene JWT for the signed-in user with basic access only.
    let access = body["access_token"].as_str().unwrap();
    let user = AuthService::new(JWT_SECRET.into())
        .validate_jwt_token(access)
        .await
        .unwrap();
    assert_eq!(user.user_id, DEV_USER);
    assert_eq!(user.permissions, vec!["basic:access".to_string()]);
    assert!(user.jti.is_some(), "desktop JWTs must be revocable");
}

#[tokio::test]
#[serial]
async fn codes_are_single_use() {
    let (router, _) = app().await;
    let code = obtain_code(&router).await;
    assert_eq!(
        exchange(&router, &code, VERIFIER, REDIRECT).await.0,
        StatusCode::OK
    );
    let (status, body) = exchange(&router, &code, VERIFIER, REDIRECT).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error_code"], "INVALID_GRANT", "{body}");
}

#[tokio::test]
#[serial]
async fn wrong_verifier_or_redirect_is_rejected() {
    let (router, _) = app().await;
    let code = obtain_code(&router).await;
    let wrong = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXx";
    assert_eq!(
        exchange(&router, &code, wrong, REDIRECT).await.0,
        StatusCode::BAD_REQUEST
    );

    let code = obtain_code(&router).await;
    let other = "http://127.0.0.1:53683/callback";
    assert_eq!(
        exchange(&router, &code, VERIFIER, other).await.0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
#[serial]
async fn expired_codes_are_rejected() {
    let (router, pool) = app().await;
    let code = new_secret();
    DesktopSessionRepository::new(pool)
        .insert_auth_code(
            &hash_secret(&code),
            &DesktopGrant {
                user_id: Uuid::parse_str(DEV_USER).unwrap(),
                tier: "free".into(),
                consciousness_level: 0,
            },
            &s256_challenge(VERIFIER),
            REDIRECT,
            Utc::now() - ChronoDuration::seconds(1),
        )
        .await
        .unwrap();
    assert_eq!(
        exchange(&router, &code, VERIFIER, REDIRECT).await.0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
#[serial]
async fn authorize_requires_a_browser_identity() {
    let (router, _) = app().await;
    let uri = authorize_uri(&s256_challenge(VERIFIER), REDIRECT);

    // No identity at all.
    let (status, _, _) = send(&router, Request::get(&uri).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // A desktop access token must not be able to mint further sessions.
    let code = obtain_code(&router).await;
    let access = exchange(&router, &code, VERIFIER, REDIRECT).await.1["access_token"]
        .as_str()
        .unwrap()
        .to_string();
    let (status, _, body) = send(
        &router,
        Request::get(&uri)
            .header(header::AUTHORIZATION, format!("Bearer {access}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(
        body["error_code"], "DESKTOP_AUTH_REQUIRES_BROWSER",
        "{body}"
    );
}

#[tokio::test]
#[serial]
async fn authorize_validates_redirect_and_pkce_parameters() {
    let (router, _) = app().await;
    let challenge = s256_challenge(VERIFIER);
    for uri in [
        authorize_uri(&challenge, "https://evil.example/callback"),
        authorize_uri(&challenge, "http://127.0.0.1:53682/other"),
        authorize_uri("not-a-challenge", REDIRECT),
        authorize_uri(&challenge, REDIRECT).replace("S256", "plain"),
        authorize_uri(&challenge, REDIRECT).replace(STATE, "x"),
    ] {
        let (status, location, body) = send(&router, browser_get(&uri)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri} -> {body}");
        assert!(location.is_none(), "must never redirect on a bad request");
    }
}

#[tokio::test]
#[serial]
async fn refresh_rotates_and_reuse_revokes_the_family() {
    let (router, _) = app().await;
    let code = obtain_code(&router).await;
    let first = exchange(&router, &code, VERIFIER, REDIRECT).await.1;
    let r1 = first["refresh_token"].as_str().unwrap().to_string();

    let (status, second) = refresh(&router, &r1).await;
    assert_eq!(status, StatusCode::OK, "{second}");
    let r2 = second["refresh_token"].as_str().unwrap().to_string();
    assert_ne!(r1, r2);

    // Replaying the rotated token is treated as theft: it fails and kills r2 too.
    assert_eq!(refresh(&router, &r1).await.0, StatusCode::BAD_REQUEST);
    assert_eq!(refresh(&router, &r2).await.0, StatusCode::BAD_REQUEST);
}

#[tokio::test]
#[serial]
async fn revoke_ends_the_session_without_leaking_token_validity() {
    let (router, _) = app().await;
    let code = obtain_code(&router).await;
    let r1 = exchange(&router, &code, VERIFIER, REDIRECT).await.1["refresh_token"]
        .as_str()
        .unwrap()
        .to_string();

    let (status, _, _) = send(
        &router,
        post_json("/api/v1/desktop/revoke", json!({"refresh_token": r1})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(refresh(&router, &r1).await.0, StatusCode::BAD_REQUEST);

    // Unknown tokens get the same answer.
    let (status, _, _) = send(
        &router,
        post_json(
            "/api/v1/desktop/revoke",
            json!({"refresh_token": "unknown"}),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
#[serial]
async fn unsupported_grant_type_is_rejected() {
    let (router, _) = app().await;
    let (status, _, body) = send(
        &router,
        post_json("/api/v1/desktop/token", json!({"grant_type": "password"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error_code"], "UNSUPPORTED_GRANT_TYPE", "{body}");
}
