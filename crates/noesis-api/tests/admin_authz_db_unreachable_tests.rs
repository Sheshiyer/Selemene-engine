//! Admin authorization must be decided before (and independently of) the
//! database.
//!
//! The router here is built with a lazy pool pointed at a port nothing listens
//! on, so every database call fails. A caller whose JWT lacks the permission
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
use tower::ServiceExt;

const JWT_SECRET: &str = "admin-authz-db-unreachable-test-secret";
/// Port 1 on loopback: connections are refused, so every query fails fast.
const UNREACHABLE_DATABASE_URL: &str =
    "postgres://noesis_unreachable_role@127.0.0.1:1/noesis_unreachable";
/// A UUID subject, so the permission resolver would consult the database.
const USER_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

async fn unreachable_db_router() -> Router {
    let mut config = ApiConfig::from_env().expect("test config");
    config.jwt_secret = JWT_SECRET.to_string();
    config.database_url = Some(UNREACHABLE_DATABASE_URL.to_string());
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
        ("GET", "/api/v1/admin/api-key-tiers".into(), "admin:keys:list"),
        ("GET", "/api/v1/admin/audit-events".into(), "admin:audit:list"),
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
