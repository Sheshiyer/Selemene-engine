//! Shared test infrastructure for noesis-api integration tests.
//!
//! Provides a single `OnceCell<Router>` that is lazily initialized per test
//! binary. Each test file that `mod common;` references this module gets the
//! same Router (and therefore the same AppState, NoesisMetrics, engine
//! instances, etc.) within that binary.
//!
//! **Why this matters for SIGTRAP prevention:**
//! - A single `NoesisMetrics` instance avoids duplicate Prometheus registration panics.
//! - A single Swiss Ephemeris initialization avoids concurrent global C state corruption.
//! - A single PgPool avoids multiple lazy pools racing during cleanup.

#![allow(dead_code)]

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use noesis_api::{build_app_state_lazy_db, create_router, ApiConfig};
use noesis_auth::AuthService;
use noesis_core::EngineInput;
use serde_json::{json, Value};
use tokio::sync::OnceCell;
use tower::ServiceExt;

/// Test-only JWT secret. Never use in production.
pub const TEST_JWT_SECRET: &str = "noesis-dev-secret-change-in-production";

// ---------------------------------------------------------------------------
// Singleton router
// ---------------------------------------------------------------------------

/// Global test router -- created once per test binary, shared across all tests.
static SHARED_ROUTER: OnceCell<Router> = OnceCell::const_new();

/// Get or create the singleton test router.
///
/// All test files within the same binary share this router, which ensures:
/// - Single `NoesisMetrics` instance (no duplicate Prometheus registration)
/// - Single Swiss Ephemeris initialization (no concurrent global C state)
/// - Single PgPool (no multiple lazy pools racing during cleanup)
pub async fn get_router() -> &'static Router {
    SHARED_ROUTER
        .get_or_init(|| async {
            let config = ApiConfig::from_env().expect("failed to load test config");
            let state = build_app_state_lazy_db(&config).await;
            create_router(state, &config)
        })
        .await
}

// ---------------------------------------------------------------------------
// JWT helpers
// ---------------------------------------------------------------------------

/// Generate a valid JWT token for testing with a specific consciousness level.
pub fn generate_test_token(consciousness_level: u8) -> String {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| TEST_JWT_SECRET.to_string());
    let auth = AuthService::new(jwt_secret);

    auth.generate_jwt_token(
        "shared-test-user",
        "premium",
        &["read".to_string(), "write".to_string()],
        consciousness_level,
    )
    .expect("Failed to generate test JWT")
}

// ---------------------------------------------------------------------------
// Request helpers
// ---------------------------------------------------------------------------

/// Make an authenticated HTTP request through the shared Axum router.
pub async fn make_authenticated_request(
    method: &str,
    uri: &str,
    token: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let router = get_router().await;
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json");

    let body = match body {
        Some(v) => Body::from(serde_json::to_vec(&v).unwrap()),
        None => Body::empty(),
    };

    let response = router
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&bytes).unwrap_or(json!({}))
    };
    (status, json)
}

/// Make an authenticated request and also return response headers.
pub async fn make_authenticated_request_with_headers(
    method: &str,
    uri: &str,
    token: &str,
    body: Option<Value>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let router = get_router().await;
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json");

    let body = match body {
        Some(v) => Body::from(serde_json::to_vec(&v).unwrap()),
        None => Body::empty(),
    };

    let response = router
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&bytes).unwrap_or(json!({}))
    };
    (status, headers, json)
}

/// Make an unauthenticated HTTP request through the shared Axum router.
pub async fn make_unauthenticated_request(
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let router = get_router().await;
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");

    let body = match body {
        Some(v) => Body::from(serde_json::to_vec(&v).unwrap()),
        None => Body::empty(),
    };

    let response = router
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&bytes).unwrap_or(json!({}))
    };
    (status, json)
}

/// Make an unauthenticated request and also return response headers.
pub async fn make_unauthenticated_request_with_headers(
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let router = get_router().await;
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");

    let body = match body {
        Some(v) => Body::from(serde_json::to_vec(&v).unwrap()),
        None => Body::empty(),
    };

    let response = router
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&bytes).unwrap_or(json!({}))
    };
    (status, headers, json)
}

// ---------------------------------------------------------------------------
// Input helpers
// ---------------------------------------------------------------------------

/// Standard test birth input used across multiple test files.
pub fn create_test_birth_input() -> EngineInput {
    EngineInput {
        birth_data: Some(noesis_core::BirthData {
            name: Some("Test User".to_string()),
            date: "1990-01-15".to_string(),
            time: Some("14:30".to_string()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: "Asia/Kolkata".to_string(),
        }),
        current_time: chrono::Utc::now(),
        location: Some(noesis_core::Coordinates {
            latitude: 12.9716,
            longitude: 77.5946,
            altitude: None,
        }),
        precision: noesis_core::Precision::Standard,
        options: std::collections::HashMap::new(),
    }
}

/// Reference birth input for E2E tests (New York coordinates).
pub fn reference_birth_input() -> EngineInput {
    EngineInput {
        birth_data: Some(noesis_core::BirthData {
            name: Some("E2E Reference".to_string()),
            date: "1990-01-15".to_string(),
            time: Some("14:30".to_string()),
            latitude: 40.7128,
            longitude: -74.0060,
            timezone: "America/New_York".to_string(),
        }),
        current_time: chrono::Utc::now(),
        location: Some(noesis_core::Coordinates {
            latitude: 40.7128,
            longitude: -74.0060,
            altitude: None,
        }),
        precision: noesis_core::Precision::Standard,
        options: std::collections::HashMap::new(),
    }
}
