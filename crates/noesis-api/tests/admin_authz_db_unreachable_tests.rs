//! Admin authorization must be decided before (and independently of) the
//! database.
//!
//! The router here is built with a lazy pool pointed at a local socket that
//! does not speak the Postgres protocol, so every database call fails. A caller whose JWT lacks the permission
//! must still get 403 `FORBIDDEN` (never 500), and a caller whose JWT already
//! carries the permission must not need the database to pass the gate.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use noesis_api::{build_app_state_lazy_db, create_router, ApiConfig};
use noesis_auth::AuthService;
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tower::ServiceExt;

const JWT_SECRET: &str = "admin-authz-db-unreachable-test-secret";
/// A UUID subject, so the permission resolver would consult the database.
const USER_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

/// Start a TCP endpoint that answers every connection with non-Postgres bytes
/// and hangs up. Unlike a refused port (which sqlx retries until its acquire
/// timeout), this fails each connection attempt immediately.
async fn broken_database_url() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fake database");
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let _ = socket.write_all(b"not a postgres server\n").await;
        }
    });
    format!("postgres://noesis_unreachable_role@127.0.0.1:{port}/noesis_unreachable")
}

/// A database URL whose port refuses connections. sqlx treats a refused
/// connect as transient and retries it until the pool's acquire timeout, so
/// this is the case that used to hold a request for sqlx's 30 s default.
async fn refused_database_url() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind probe port");
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    format!("postgres://noesis_unreachable_role@127.0.0.1:{port}/noesis_unreachable")
}

/// A database URL that accepts TCP connections and never answers, like a
/// blackholed or wedged server: only the acquire timeout ends the wait.
async fn silent_database_url() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind silent database");
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        let mut held = Vec::new();
        while let Ok((socket, _)) = listener.accept().await {
            held.push(socket);
        }
    });
    format!("postgres://noesis_unreachable_role@127.0.0.1:{port}/noesis_unreachable")
}

async fn unreachable_db_router() -> Router {
    router_for(broken_database_url().await).await
}

async fn router_for(database_url: String) -> Router {
    let mut config = ApiConfig::from_env().expect("test config");
    config.jwt_secret = JWT_SECRET.to_string();
    config.database_url = Some(database_url);
    let state = build_app_state_lazy_db(&config).await;
    assert!(
        state.admin_repository.is_some(),
        "the admin repository must exist so the database path is really exercised"
    );
    create_router(state, &config)
}

fn token(permissions: &[&str]) -> String {
    AuthService::new(JWT_SECRET.to_string())
        .generate_jwt_token(
            USER_ID,
            "free",
            &permissions
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>(),
            1,
        )
        .expect("token")
}

async fn send(router: &Router, method: &str, uri: &str, token: &str) -> (StatusCode, Value) {
    let body = if method == "GET" {
        Body::empty()
    } else {
        Body::from("{}")
    };
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(body)
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

const READING: &str = "75c78363-a307-4b4c-a26c-e9cc96470ce7";

#[tokio::test]
async fn viewer_gets_403_not_500_when_database_is_unreachable() {
    let router = unreachable_db_router().await;
    let viewer = token(&["basic:access"]);

    let cases: Vec<(&str, String, &str)> = vec![
        (
            "GET",
            "/api/v1/admin/living-readings".into(),
            "admin:analytics:read",
        ),
        (
            "GET",
            format!("/api/v1/admin/living-readings/{READING}"),
            "admin:analytics:read",
        ),
        (
            "GET",
            format!("/api/v1/admin/living-readings/{READING}/invitations"),
            "admin:analytics:read",
        ),
        (
            "POST",
            format!("/api/v1/admin/living-readings/{READING}/invitations"),
            "admin:analytics:write",
        ),
        ("GET", "/api/v1/admin/users".into(), "admin:users:list"),
        ("GET", "/api/v1/admin/api-keys".into(), "admin:keys:list"),
        (
            "GET",
            "/api/v1/admin/api-key-tiers".into(),
            "admin:keys:list",
        ),
        (
            "GET",
            "/api/v1/admin/audit-events".into(),
            "admin:audit:list",
        ),
        (
            "GET",
            format!("/api/v1/admin/audit-events/{READING}"),
            "admin:audit:read",
        ),
        (
            "GET",
            "/api/v1/admin/billing/overview".into(),
            "admin:billing:read",
        ),
    ];

    for (method, uri, required) in cases {
        let (status, body) = send(&router, method, &uri, &viewer).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {uri}: {body}");
        assert_eq!(body["error_code"], "FORBIDDEN", "{method} {uri}: {body}");
        assert_eq!(
            body["details"]["required_permission"], required,
            "{method} {uri}: {body}"
        );
    }
}

#[tokio::test]
async fn jwt_holding_the_permission_does_not_need_the_database() {
    let router = unreachable_db_router().await;

    // The key-tier catalog is static: once the permission gate passes, the
    // handler never touches the database. With an unreachable pool it can
    // therefore only answer 200 if the gate itself skipped the lookup.
    for permission in ["admin:keys:list", "admin:keys:create", "admin:*"] {
        let (status, body) = send(
            &router,
            "GET",
            "/api/v1/admin/api-key-tiers",
            &token(&[permission]),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{permission}: {body}");
        assert!(body.as_array().is_some_and(|tiers| !tiers.is_empty()));
    }
}

/// Generous ceiling for one request against an unreachable database. The
/// default acquire timeout (`DATABASE_ACQUIRE_TIMEOUT_MS`, 3 s) sits well
/// below it; sqlx's previous 30 s default sat well above it.
const FAIL_FAST_CEILING: std::time::Duration = std::time::Duration::from_secs(15);

/// A viewer request that consults the database must still be denied with 403
/// when the database is unreachable, and must be denied promptly rather than
/// holding the handler for the pool's old 30 s acquire timeout.
async fn assert_viewer_denied_promptly(router: &Router, kind: &str) {
    let viewer = token(&["basic:access"]);
    for uri in [
        "/api/v1/admin/living-readings".to_string(),
        format!("/api/v1/admin/living-readings/{READING}"),
    ] {
        let started = std::time::Instant::now();
        let (status, body) = send(router, "GET", &uri, &viewer).await;
        let elapsed = started.elapsed();
        assert_eq!(status, StatusCode::FORBIDDEN, "{kind} {uri}: {body}");
        assert_eq!(body["error_code"], "FORBIDDEN", "{kind} {uri}: {body}");
        assert_eq!(
            body["details"]["required_permission"], "admin:analytics:read",
            "{kind} {uri}: {body}"
        );
        assert!(
            elapsed < FAIL_FAST_CEILING,
            "{kind} {uri}: took {elapsed:?}, want < {FAIL_FAST_CEILING:?}"
        );
    }
}

#[tokio::test]
async fn refused_database_denies_viewer_fast() {
    let router = router_for(refused_database_url().await).await;
    assert_viewer_denied_promptly(&router, "refused").await;
}

#[tokio::test]
async fn silent_database_denies_viewer_fast() {
    let router = router_for(silent_database_url().await).await;
    assert_viewer_denied_promptly(&router, "silent").await;
}
