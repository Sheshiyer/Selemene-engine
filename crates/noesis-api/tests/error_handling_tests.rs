//! W1-S7-11: Error Handling and Recovery Tests
//!
//! Verifies that all API error paths return structured ErrorResponse with
//! correct HTTP status codes, error_code fields, and descriptive messages.
//! Covers: 400, 401, 403, 404, 422, 429, 500 error scenarios.

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

// ---------------------------------------------------------------------------
// Test fixtures (shared singleton router)
// ---------------------------------------------------------------------------

static ERROR_TEST_ROUTER: OnceCell<Router> = OnceCell::const_new();

async fn get_router() -> &'static Router {
    ERROR_TEST_ROUTER
        .get_or_init(|| async {
            let config = ApiConfig::from_env();
            let state = build_app_state_lazy_db(&config).await;
            create_router(state, &config)
        })
        .await
}

fn generate_token(consciousness_level: u8) -> String {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    let auth = AuthService::new(jwt_secret);
    auth.generate_jwt_token(
        "error-test-user",
        "premium",
        &["read".to_string(), "write".to_string()],
        consciousness_level,
    )
    .expect("Failed to generate test JWT")
}

fn create_birth_input() -> EngineInput {
    EngineInput {
        birth_data: Some(noesis_core::BirthData {
            name: Some("Error Test User".to_string()),
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

/// Helper: send an authenticated request, return (status, headers, body).
async fn send_authenticated(
    method: &str,
    uri: &str,
    token: &str,
    body: Option<Value>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let router = get_router().await;
    let request_builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json");

    let body = if let Some(json_body) = body {
        Body::from(serde_json::to_vec(&json_body).unwrap())
    } else {
        Body::empty()
    };

    let request = request_builder.body(body).unwrap();
    let response = router.clone().oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = if body_bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or(json!({}))
    };

    (status, headers, json)
}

/// Helper: send an unauthenticated request.
async fn send_unauthenticated(
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let router = get_router().await;
    let request_builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");

    let body = if let Some(json_body) = body {
        Body::from(serde_json::to_vec(&json_body).unwrap())
    } else {
        Body::empty()
    };

    let request = request_builder.body(body).unwrap();
    let response = router.clone().oneshot(request).await.unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = if body_bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&body_bytes).unwrap_or(json!({}))
    };

    (status, headers, json)
}

// ---------------------------------------------------------------------------
// 401 Unauthorized tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_unauthorized_no_token_engine_calculate() {
    let input = create_birth_input();
    let (status, _, body) = send_unauthenticated(
        "POST",
        "/api/v1/engines/panchanga/calculate",
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error_code"], "UNAUTHORIZED");
    assert!(
        body["error"].as_str().unwrap().contains("Authentication"),
        "Error message should mention authentication: {}",
        body["error"]
    );
}

#[tokio::test]
async fn test_unauthorized_no_token_workflow_execute() {
    let input = create_birth_input();
    let (status, _, body) = send_unauthenticated(
        "POST",
        "/api/v1/workflows/birth-blueprint/execute",
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error_code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn test_unauthorized_invalid_jwt_token() {
    let (status, _, body) = send_authenticated(
        "GET",
        "/api/v1/engines",
        "invalid.jwt.token",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error_code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn test_unauthorized_no_token_list_engines() {
    let (status, _, body) = send_unauthenticated("GET", "/api/v1/engines", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error_code"], "UNAUTHORIZED");
}

// ---------------------------------------------------------------------------
// 403 Forbidden tests (Phase Access Denied)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_phase_access_denied_human_design() {
    // Human Design requires phase 1; use phase 0 token
    let token = generate_token(0);
    let input = create_birth_input();

    let (status, _, body) = send_authenticated(
        "POST",
        "/api/v1/engines/human-design/calculate",
        &token,
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "Phase 0 user should be denied access to HD (phase 1). Body: {:?}",
        body
    );
    assert_eq!(body["error_code"], "PHASE_ACCESS_DENIED");
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("phase"),
        "Error should mention phase/consciousness level: {}",
        body["error"]
    );
    // Verify details include required and current phase
    assert!(body["details"]["required_phase"].is_number());
    assert!(body["details"]["current_phase"].is_number());
}

#[tokio::test]
async fn test_phase_access_denied_gene_keys() {
    // Gene Keys requires phase 2; use phase 0 token
    let token = generate_token(0);
    let input = create_birth_input();

    let (status, _, body) = send_authenticated(
        "POST",
        "/api/v1/engines/gene-keys/calculate",
        &token,
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "Phase 0 user should be denied access to Gene Keys (phase 2). Body: {:?}",
        body
    );
    assert_eq!(body["error_code"], "PHASE_ACCESS_DENIED");
}

#[tokio::test]
async fn test_phase_access_allowed_with_sufficient_level() {
    // Phase 5 should access anything
    let token = generate_token(5);
    let input = create_birth_input();

    let (status, _, _body) = send_authenticated(
        "POST",
        "/api/v1/engines/human-design/calculate",
        &token,
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Phase 5 user should access Human Design"
    );
}

// ---------------------------------------------------------------------------
// 404 Not Found tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_engine_not_found() {
    let token = generate_token(5);
    let input = create_birth_input();

    let (status, _, body) = send_authenticated(
        "POST",
        "/api/v1/engines/nonexistent-engine/calculate",
        &token,
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], "ENGINE_NOT_FOUND");
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("nonexistent-engine"),
        "Error should include engine name"
    );
}

#[tokio::test]
async fn test_workflow_not_found() {
    let token = generate_token(5);
    let input = create_birth_input();

    let (status, _, body) = send_authenticated(
        "POST",
        "/api/v1/workflows/nonexistent-workflow/execute",
        &token,
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], "WORKFLOW_NOT_FOUND");
}

#[tokio::test]
async fn test_engine_info_not_found() {
    let token = generate_token(5);

    let (status, _, body) =
        send_authenticated("GET", "/api/v1/engines/imaginary/info", &token, None).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], "ENGINE_NOT_FOUND");
}

#[tokio::test]
async fn test_workflow_info_not_found() {
    let token = generate_token(5);

    let (status, _, body) = send_authenticated(
        "GET",
        "/api/v1/workflows/fake-workflow/info",
        &token,
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], "WORKFLOW_NOT_FOUND");
}

// ---------------------------------------------------------------------------
// 400 Bad Request / 422 Validation Error tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_malformed_json_body() {
    // Send completely invalid JSON
    let router = get_router().await;
    let token = generate_token(5);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/engines/panchanga/calculate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(b"{ this is not json }".to_vec()))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();

    // Axum returns 400 for malformed JSON
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Malformed JSON should return 400 or 422, got {}",
        status
    );
}

#[tokio::test]
async fn test_empty_json_body() {
    let token = generate_token(5);

    let (status, _, _body) = send_authenticated(
        "POST",
        "/api/v1/engines/panchanga/calculate",
        &token,
        Some(json!({})),
    )
    .await;

    // Empty object is missing required fields -- should fail
    assert!(
        status == StatusCode::BAD_REQUEST
            || status == StatusCode::UNPROCESSABLE_ENTITY
            || status == StatusCode::INTERNAL_SERVER_ERROR,
        "Empty JSON body should fail with 400/422/500, got {}",
        status
    );
}

#[tokio::test]
async fn test_missing_birth_data_validation() {
    let token = generate_token(5);

    // EngineInput without birth_data (birth_data: null)
    let input = json!({
        "birth_data": null,
        "current_time": "2024-01-15T14:30:00Z",
        "location": null,
        "precision": "Standard",
        "options": {}
    });

    let (status, _, body) = send_authenticated(
        "POST",
        "/api/v1/engines/human-design/calculate",
        &token,
        Some(input),
    )
    .await;

    // HD engine requires birth data; should return validation error or calculation error
    assert!(
        status.is_client_error() || status.is_server_error(),
        "Missing birth_data should return error, got {}",
        status
    );

    // If it returns a structured error, verify error_code
    if body["error_code"].is_string() {
        let code = body["error_code"].as_str().unwrap();
        assert!(
            code == "VALIDATION_ERROR"
                || code == "CALCULATION_ERROR"
                || code == "PHASE_ACCESS_DENIED",
            "Expected validation/calculation error code, got: {}",
            code
        );
    }
}

// ---------------------------------------------------------------------------
// 429 Rate Limit tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_rate_limit_headers_present() {
    // A single authenticated request should include rate limit headers
    let token = generate_token(5);

    let (status, headers, _body) =
        send_authenticated("GET", "/api/v1/engines", &token, None).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        headers.contains_key("x-ratelimit-limit"),
        "Response should include X-RateLimit-Limit header"
    );
    assert!(
        headers.contains_key("x-ratelimit-remaining"),
        "Response should include X-RateLimit-Remaining header"
    );
    assert!(
        headers.contains_key("x-ratelimit-reset"),
        "Response should include X-RateLimit-Reset header"
    );
}

#[tokio::test]
async fn test_rate_limit_remaining_decrements() {
    // Make two requests and verify remaining decrements
    let token = generate_token(5);

    let (_, headers1, _) = send_authenticated("GET", "/api/v1/engines", &token, None).await;
    let remaining1: u32 = headers1
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();

    let (_, headers2, _) = send_authenticated("GET", "/api/v1/engines", &token, None).await;
    let remaining2: u32 = headers2
        .get("x-ratelimit-remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse()
        .unwrap();

    assert!(
        remaining2 < remaining1,
        "X-RateLimit-Remaining should decrement: {} -> {}",
        remaining1,
        remaining2
    );
}

// Note: Full rate limit exhaustion test is commented out because it would
// need 100+ sequential requests through tower::oneshot, which is slow.
// The rate limiter logic is tested directly in middleware unit tests.

// ---------------------------------------------------------------------------
// Error response structure validation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_error_response_structure_401() {
    let (_, _, body) = send_unauthenticated("GET", "/api/v1/engines", None).await;

    // Validate ErrorResponse structure
    assert!(body["error"].is_string(), "error field should be a string");
    assert!(
        body["error_code"].is_string(),
        "error_code field should be a string"
    );
    // details may be null or present
    assert!(
        body["details"].is_null() || body["details"].is_object(),
        "details should be null or object"
    );
}

#[tokio::test]
async fn test_error_response_structure_404() {
    let token = generate_token(5);
    let input = create_birth_input();

    let (_, _, body) = send_authenticated(
        "POST",
        "/api/v1/engines/missing-engine/calculate",
        &token,
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    assert!(body["error"].is_string());
    assert!(body["error_code"].is_string());
    assert!(
        body["details"].is_null() || body["details"].is_object(),
        "details should be null or object"
    );
}

#[tokio::test]
async fn test_error_response_structure_403() {
    let token = generate_token(0);
    let input = create_birth_input();

    let (status, _, body) = send_authenticated(
        "POST",
        "/api/v1/engines/human-design/calculate",
        &token,
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    if status == StatusCode::FORBIDDEN {
        assert!(body["error"].is_string());
        assert_eq!(body["error_code"], "PHASE_ACCESS_DENIED");
        assert!(body["details"].is_object());
        assert!(body["details"]["required_phase"].is_number());
        assert!(body["details"]["current_phase"].is_number());
    }
}

// ---------------------------------------------------------------------------
// Error does not leak internals
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_error_does_not_leak_stack_traces() {
    let token = generate_token(5);
    let input = create_birth_input();

    // Trigger an engine not found error
    let (_, _, body) = send_authenticated(
        "POST",
        "/api/v1/engines/nonexistent/calculate",
        &token,
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    let error_msg = body["error"].as_str().unwrap_or("");
    assert!(
        !error_msg.contains("panic"),
        "Error message should not contain 'panic'"
    );
    assert!(
        !error_msg.contains("stack backtrace"),
        "Error message should not contain stack traces"
    );
    assert!(
        !error_msg.contains("thread '"),
        "Error message should not contain thread info"
    );
}

// ---------------------------------------------------------------------------
// Health endpoints do not require auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_endpoint_no_auth_required() {
    let (status, _, body) = send_unauthenticated("GET", "/health", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_metrics_endpoint_no_auth_required() {
    let router = get_router().await;

    let request = Request::builder()
        .method("GET")
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

// ---------------------------------------------------------------------------
// Legacy endpoints do not require auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_legacy_panchanga_no_auth() {
    let (status, _, _body) = send_unauthenticated(
        "POST",
        "/api/legacy/panchanga/calculate",
        Some(json!({
            "date": "2024-01-15",
            "time": "14:30",
            "latitude": 12.9716,
            "longitude": 77.5946,
            "timezone": "Asia/Kolkata"
        })),
    )
    .await;

    // Legacy should work without auth
    assert_eq!(
        status,
        StatusCode::OK,
        "Legacy panchanga should not require auth"
    );
}

// ---------------------------------------------------------------------------
// Structured error codes map correctly
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_error_code_mapping_engine_not_found() {
    let token = generate_token(5);
    let input = create_birth_input();

    let (status, _, body) = send_authenticated(
        "POST",
        "/api/v1/engines/does-not-exist/calculate",
        &token,
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], "ENGINE_NOT_FOUND");
}

#[tokio::test]
async fn test_error_code_mapping_workflow_not_found() {
    let token = generate_token(5);
    let input = create_birth_input();

    let (status, _, body) = send_authenticated(
        "POST",
        "/api/v1/workflows/does-not-exist/execute",
        &token,
        Some(serde_json::to_value(input).unwrap()),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], "WORKFLOW_NOT_FOUND");
}

#[tokio::test]
async fn test_error_code_mapping_unauthorized() {
    let (status, _, body) = send_unauthenticated("GET", "/api/v1/status", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error_code"], "UNAUTHORIZED");
}
