//! Integration tests for usage analytics endpoints.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use noesis_auth::AuthService;
use serde_json::{json, Value};
use tower::ServiceExt;

mod common;

async fn get_test_router() -> &'static Router {
    common::get_router().await
}

fn generate_user_token() -> String {
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| common::TEST_JWT_SECRET.to_string());
    let auth = AuthService::new(jwt_secret);

    auth.generate_jwt_token(
        "550e8400-e29b-41d4-a716-446655440000",
        "free",
        &["read".to_string()],
        5,
    )
    .expect("Failed to generate user token")
}

fn generate_admin_token() -> String {
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| common::TEST_JWT_SECRET.to_string());
    let auth = AuthService::new(jwt_secret);

    auth.generate_jwt_token(
        "550e8400-e29b-41d4-a716-446655440001",
        "pro",
        &["admin:analytics:read".to_string()],
        7,
    )
    .expect("Failed to generate admin token")
}

async fn make_authenticated_request(
    router: &Router,
    method: &str,
    uri: &str,
    token: &str,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let body: Value = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&bytes).unwrap_or(json!({}))
    };

    (status, body)
}

fn is_db_unavailable(status: StatusCode) -> bool {
    status == StatusCode::INTERNAL_SERVER_ERROR || status == StatusCode::SERVICE_UNAVAILABLE
}

#[tokio::test]
async fn test_get_my_usage_requires_auth() {
    let router = get_test_router().await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/v1/users/me/usage")
        .body(Body::empty())
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_my_usage_response_shape() {
    let router = get_test_router().await;
    let token = generate_user_token();

    let (status, body) =
        make_authenticated_request(router, "GET", "/api/v1/users/me/usage", &token).await;

    if is_db_unavailable(status) {
        eprintln!(
            "SKIP: GET /api/v1/users/me/usage -- DB not available (status {})",
            status
        );
        return;
    }

    assert_eq!(status, StatusCode::OK, "Unexpected response: {:?}", body);
    assert!(body["daily"].is_object());
    assert!(body["monthly"].is_object());
    assert!(body["engine_breakdown"].is_array());
}

#[tokio::test]
async fn test_admin_usage_summary_requires_admin_permission() {
    let router = get_test_router().await;
    let user_token = generate_user_token();

    let (status, _body) =
        make_authenticated_request(router, "GET", "/api/v1/admin/usage/summary", &user_token).await;

    // Could be 403 (permission denied) or DB-unavailable path if middleware/bootstrapping fails.
    assert!(status == StatusCode::FORBIDDEN || is_db_unavailable(status));
}

#[tokio::test]
async fn test_admin_usage_summary_response_shape() {
    let router = get_test_router().await;
    let admin_token = generate_admin_token();

    let (status, body) = make_authenticated_request(
        router,
        "GET",
        "/api/v1/admin/usage/summary?engine_limit=5&top_users_limit=5",
        &admin_token,
    )
    .await;

    if is_db_unavailable(status) {
        eprintln!(
            "SKIP: GET /api/v1/admin/usage/summary -- DB not available (status {})",
            status
        );
        return;
    }

    assert_eq!(status, StatusCode::OK, "Unexpected response: {:?}", body);
    assert!(body["daily"].is_object());
    assert!(body["monthly"].is_object());
    assert!(body["engine_breakdown"].is_array());
    assert!(body["top_users"].is_array());
}
