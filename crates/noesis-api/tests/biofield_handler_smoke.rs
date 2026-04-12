//! Smoke tests for the placeholder biofield route namespace and OpenAPI exposure.

mod common;

use axum::{body::Body, http::Request, http::StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

#[tokio::test]
async fn biofield_handler_smoke_routes_exist_under_api_v1() {
    let unauthorized_cases = [
        ("POST", "/api/v1/biofield/sessions", Some(json!({}))),
        (
            "POST",
            "/api/v1/biofield/sessions/00000000-0000-0000-0000-000000000001/close",
            Some(json!({})),
        ),
        (
            "GET",
            "/api/v1/biofield/sessions/00000000-0000-0000-0000-000000000001",
            None,
        ),
        (
            "POST",
            "/api/v1/biofield/sessions/00000000-0000-0000-0000-000000000001/captures",
            Some(json!({})),
        ),
        ("GET", "/api/v1/biofield/readings", None),
        (
            "GET",
            "/api/v1/biofield/readings/00000000-0000-0000-0000-000000000002",
            None,
        ),
        (
            "POST",
            "/api/v1/biofield/readings/00000000-0000-0000-0000-000000000002/reprocess",
            Some(json!({})),
        ),
        ("GET", "/api/v1/biofield/baselines", None),
        ("POST", "/api/v1/biofield/baselines", Some(json!({}))),
    ];

    for (method, uri, body) in unauthorized_cases {
        let (status, _) = common::make_unauthenticated_request(method, uri, body).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "route should exist: {uri}"
        );
    }
}

#[tokio::test]
async fn biofield_handler_smoke_openapi_contains_biofield_paths() {
    let router = common::get_router().await;

    let request = Request::builder()
        .method("GET")
        .uri("/api/openapi.json")
        .body(Body::empty())
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let spec: Value = serde_json::from_slice(&bytes).expect("valid openapi json");

    let paths = spec
        .get("paths")
        .and_then(|v| v.as_object())
        .expect("paths object");

    let expected = [
        "/api/v1/biofield/sessions",
        "/api/v1/biofield/sessions/{session_id}/close",
        "/api/v1/biofield/sessions/{session_id}",
        "/api/v1/biofield/sessions/{session_id}/captures",
        "/api/v1/biofield/readings",
        "/api/v1/biofield/readings/{reading_id}",
        "/api/v1/biofield/readings/{reading_id}/reprocess",
        "/api/v1/biofield/baselines",
    ];

    for path in expected {
        assert!(
            paths.contains_key(path),
            "missing biofield path in openapi: {path}"
        );
    }
}
