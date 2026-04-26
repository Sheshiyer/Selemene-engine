//! OpenAPI workflow schema tests for workflow-specific endpoint definitions.

use axum::{body::Body, http::Request, http::StatusCode};
use serde_json::Value;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn test_openapi_contains_six_workflow_execute_paths() {
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
        "/api/v1/workflows/birth-blueprint/execute",
        "/api/v1/workflows/daily-practice/execute",
        "/api/v1/workflows/decision-support/execute",
        "/api/v1/workflows/self-inquiry/execute",
        "/api/v1/workflows/creative-expression/execute",
        "/api/v1/workflows/full-spectrum/execute",
    ];

    for path in expected {
        assert!(paths.contains_key(path), "missing workflow path: {path}");
    }
}

#[tokio::test]
async fn test_workflow_synthesis_schemas_are_typed() {
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

    let schemas = spec
        .get("components")
        .and_then(|c| c.get("schemas"))
        .and_then(|s| s.as_object())
        .expect("components.schemas object");

    let expected_response_schemas = [
        "BirthBlueprintWorkflowResultSchema",
        "DailyPracticeWorkflowResultSchema",
        "DecisionSupportWorkflowResultSchema",
        "SelfInquiryWorkflowResultSchema",
        "CreativeExpressionWorkflowResultSchema",
        "FullSpectrumWorkflowResultSchema",
    ];

    for schema_name in expected_response_schemas {
        let schema = schemas
            .get(schema_name)
            .unwrap_or_else(|| panic!("missing response schema: {schema_name}"));

        let synthesis = schema
            .get("properties")
            .and_then(|p| p.get("synthesis"))
            .expect("synthesis field in workflow result schema");

        let has_typed_ref = synthesis
            .get("allOf")
            .and_then(|v| v.as_array())
            .map(|items| {
                items.iter().any(|item| {
                    item.get("$ref")
                        .and_then(|r| r.as_str())
                        .map(|r| r.contains("SynthesisSchema"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
            || synthesis
                .get("$ref")
                .and_then(|r| r.as_str())
                .map(|r| r.contains("SynthesisSchema"))
                .unwrap_or(false);

        assert!(
            has_typed_ref,
            "workflow schema {schema_name} should reference a typed synthesis schema"
        );
    }
}
