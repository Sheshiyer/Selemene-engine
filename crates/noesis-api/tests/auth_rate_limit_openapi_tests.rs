//! OpenAPI auth and rate-limit documentation tests.

use axum::{body::Body, http::Request, http::StatusCode};
use serde_json::Value;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn test_openapi_security_schemes_include_bearer_and_api_key() {
    let router = common::get_router().await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let spec: Value = serde_json::from_slice(&bytes).expect("valid openapi json");

    let schemes = spec
        .get("components")
        .and_then(|c| c.get("securitySchemes"))
        .and_then(|s| s.as_object())
        .expect("components.securitySchemes object");

    assert!(schemes.contains_key("bearer_auth"));
    assert!(schemes.contains_key("api_key"));
}

#[tokio::test]
async fn test_all_secured_operations_document_429_and_rate_headers() {
    let router = common::get_router().await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let spec: Value = serde_json::from_slice(&bytes).expect("valid openapi json");

    let paths = spec
        .get("paths")
        .and_then(|v| v.as_object())
        .expect("paths object");

    for (path, item) in paths {
        let Some(obj) = item.as_object() else {
            continue;
        };

        for method in ["get", "post", "put", "patch", "delete", "options", "head"] {
            let Some(operation) = obj.get(method) else {
                continue;
            };

            let is_secured = operation
                .get("security")
                .and_then(|v| v.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(false);

            if !is_secured {
                continue;
            }

            let responses = operation
                .get("responses")
                .and_then(|r| r.as_object())
                .unwrap_or_else(|| panic!("missing responses for {method} {path}"));

            let r429 = responses
                .get("429")
                .unwrap_or_else(|| panic!("missing 429 response for {method} {path}"));

            let headers = r429
                .get("headers")
                .and_then(|h| h.as_object())
                .unwrap_or_else(|| panic!("missing rate-limit headers for {method} {path}"));

            for expected in [
                "X-RateLimit-Limit",
                "X-RateLimit-Remaining",
                "X-RateLimit-Reset",
                "X-RateLimit-Daily-Remaining",
                "X-RateLimit-Daily-Reset",
            ] {
                assert!(
                    headers.contains_key(expected),
                    "missing header {} for {} {}",
                    expected,
                    method,
                    path
                );
            }
        }
    }
}
