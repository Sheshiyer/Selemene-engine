//! Per-engine permission scopes on engine routes (W3E slot 20 / #913).
//!
//! Reproduces the 2026-10-25 production finding: an API key whose permissions
//! were only `basic:access` + `face-reading:read` executed panchanga. These
//! tests pin the fixed behaviour for API-key principals in each
//! `ENGINE_SCOPE_MODE`, and prove JWT principals stay tier-governed.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use chrono::{Duration as ChronoDuration, Utc};
use noesis_api::{build_app_state_lazy_db, create_router, ApiConfig};
use noesis_auth::{ApiKey, AuthService};
use serde_json::Value;
use serial_test::serial;
use std::sync::Arc;
use tower::ServiceExt;

mod common;

const PANCHANGA_BODY: &str = r#"{
  "contract_version": "v1",
  "consciousness_level": 1,
  "parameters": {},
  "birth_data": {"date": "2026-10-25", "time": "06:00", "latitude": 12.97, "longitude": 77.59, "timezone": "Asia/Kolkata"}
}"#;

async fn router_with_key(permissions: &[&str]) -> (Router, String) {
    let mut config = ApiConfig::from_env().expect("test config");
    config.database_url = None;
    let state = build_app_state_lazy_db(&config).await;
    let key = add_key(&state.auth, permissions).await;
    (create_router(state, &config), key)
}

async fn add_key(auth: &Arc<AuthService>, permissions: &[&str]) -> String {
    let value = format!("scope-test-key-{}", uuid::Uuid::new_v4());
    auth.add_api_key(ApiKey {
        key: value.clone(),
        user_id: uuid::Uuid::new_v4().to_string(),
        tier: "premium".to_string(),
        permissions: permissions.iter().map(|p| p.to_string()).collect(),
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + ChronoDuration::hours(1)),
        last_used: None,
        rate_limit: 1000,
        consciousness_level: 2,
    })
    .await
    .expect("add api key");
    value
}

async fn post(router: &Router, uri: &str, auth: (&str, &str), body: &str) -> (StatusCode, Value) {
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header(auth.0, auth.1)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

struct ModeGuard;
impl ModeGuard {
    fn set(mode: &str) -> Self {
        std::env::set_var("ENGINE_SCOPE_MODE", mode);
        Self
    }
}
impl Drop for ModeGuard {
    fn drop(&mut self) {
        std::env::remove_var("ENGINE_SCOPE_MODE");
    }
}

#[tokio::test]
#[serial]
async fn enforce_mode_rejects_key_without_engine_scope() {
    let _mode = ModeGuard::set("enforce");
    let (router, key) = router_with_key(&["basic:access", "face-reading:read"]).await;

    let (status, body) = post(
        &router,
        "/api/v1/engines/panchanga/calculate",
        ("X-API-Key", &key),
        PANCHANGA_BODY,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error_code"], "ENGINE_SCOPE_FORBIDDEN");
    assert_eq!(body["details"]["engine_id"], "panchanga");
    assert_eq!(body["details"]["required_permission"], "panchanga:read");
}

#[tokio::test]
#[serial]
async fn enforce_mode_allows_key_with_engine_scope() {
    let _mode = ModeGuard::set("enforce");
    let (router, key) = router_with_key(&["basic:access", "panchanga:read"]).await;

    let (status, body) = post(
        &router,
        "/api/v1/engines/panchanga/calculate",
        ("X-API-Key", &key),
        PANCHANGA_BODY,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["engine_id"], "panchanga");
}

#[tokio::test]
#[serial]
async fn enforce_mode_allows_engines_wildcard() {
    let _mode = ModeGuard::set("enforce");
    let (router, key) = router_with_key(&["basic:access", "engines:*"]).await;

    let (status, body) = post(
        &router,
        "/api/v1/engines/panchanga/calculate",
        ("X-API-Key", &key),
        PANCHANGA_BODY,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
#[serial]
async fn enforce_mode_checks_every_engine_in_a_workflow() {
    let _mode = ModeGuard::set("enforce");
    // birth-blueprint composes several engines; panchanga alone is not enough.
    let (router, key) = router_with_key(&["basic:access", "panchanga:read"]).await;

    let (status, body) = post(
        &router,
        "/api/v1/workflows/birth-blueprint/execute",
        ("X-API-Key", &key),
        PANCHANGA_BODY,
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "{body}");
    assert_eq!(body["error_code"], "ENGINE_SCOPE_FORBIDDEN");
    assert!(body["details"]["required_permission"]
        .as_str()
        .unwrap()
        .ends_with(":read"));
}

#[tokio::test]
#[serial]
async fn audit_mode_logs_but_does_not_block() {
    let _mode = ModeGuard::set("audit");
    let (router, key) = router_with_key(&["basic:access", "face-reading:read"]).await;

    let (status, body) = post(
        &router,
        "/api/v1/engines/panchanga/calculate",
        ("X-API-Key", &key),
        PANCHANGA_BODY,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
#[serial]
async fn jwt_principal_is_tier_governed_not_scope_checked() {
    let _mode = ModeGuard::set("enforce");
    let (router, _key) = router_with_key(&["basic:access"]).await;
    let token = common::generate_test_token(2);

    let (status, body) = post(
        &router,
        "/api/v1/engines/panchanga/calculate",
        ("Authorization", &format!("Bearer {token}")),
        PANCHANGA_BODY,
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
#[serial]
async fn unknown_engine_with_scope_still_returns_not_found_shape() {
    let _mode = ModeGuard::set("enforce");
    let (router, key) = router_with_key(&["basic:access", "engines:*"]).await;

    let (status, _body) = post(
        &router,
        "/api/v1/engines/does-not-exist/calculate",
        ("X-API-Key", &key),
        PANCHANGA_BODY,
    )
    .await;

    assert_ne!(status, StatusCode::FORBIDDEN);
    assert_ne!(status, StatusCode::OK);
}
