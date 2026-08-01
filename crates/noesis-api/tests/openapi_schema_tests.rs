//! OpenAPI schema regression tests for per-engine result documentation.

use axum::{body::Body, http::Request, http::StatusCode};
use serde_json::Value;
use tower::ServiceExt;

mod common;

fn engine_schema_names() -> Vec<&'static str> {
    vec![
        "PanchangaResultSchema",
        "NumerologyResultSchema",
        "BiorhythmResultSchema",
        "HumanDesignResultSchema",
        "GeneKeysResultSchema",
        "VimshottariResultSchema",
        "BiofieldResultSchema",
        "VedicClockResultSchema",
        "FaceReadingResultSchema",
        "NadabrahmanResultSchema",
        "TransitsResultSchema",
        "EnneagramResultSchema",
        "TarotResultSchema",
        "IChingResultSchema",
        "SacredGeometryResultSchema",
        "SigilForgeResultSchema",
        "FinancialBiosensorResultSchema",
    ]
}

#[tokio::test]
async fn test_openapi_contains_per_engine_result_schemas() {
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

    let mut found = 0;
    for name in engine_schema_names() {
        let schema = schemas
            .get(name)
            .unwrap_or_else(|| panic!("missing schema: {name}"));

        let properties_len = schema
            .get("properties")
            .and_then(|p| p.as_object())
            .map(|props| props.len())
            .unwrap_or(0);

        assert!(
            properties_len >= 3,
            "schema {name} should have at least 3 documented fields"
        );
        found += 1;
    }

    assert_eq!(found, 17, "should include 17 per-engine schemas");
}

#[tokio::test]
async fn test_engine_output_result_references_engine_union_schema() {
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

    let engine_output = spec
        .get("components")
        .and_then(|c| c.get("schemas"))
        .and_then(|s| s.get("EngineOutput"))
        .expect("EngineOutput schema");

    let result = engine_output
        .get("properties")
        .and_then(|p| p.get("result"))
        .expect("EngineOutput.result property");

    let refs_engine_union = result
        .get("$ref")
        .and_then(|v| v.as_str())
        .map(|s| s.ends_with("/EngineResultData"))
        .unwrap_or(false)
        || result
            .get("allOf")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().any(|entry| {
                    entry
                        .get("$ref")
                        .and_then(|v| v.as_str())
                        .map(|s| s.ends_with("/EngineResultData"))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

    assert!(
        refs_engine_union,
        "EngineOutput.result should reference EngineResultData schema"
    );
}
