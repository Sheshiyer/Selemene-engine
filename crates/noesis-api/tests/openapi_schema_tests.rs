//! OpenAPI schema regression tests for per-engine result documentation.

use axum::{body::Body, http::Request, http::StatusCode};
use serde_json::Value;
use std::collections::BTreeSet;
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

async fn openapi_spec() -> Value {
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
    serde_json::from_slice(&bytes).expect("valid openapi json")
}

fn schema_properties(schema: &Value, schemas: &serde_json::Map<String, Value>) -> BTreeSet<String> {
    let mut properties = BTreeSet::new();
    if let Some(object) = schema.get("properties").and_then(Value::as_object) {
        properties.extend(object.keys().cloned());
    }
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        if let Some(name) = reference.rsplit('/').next() {
            if let Some(target) = schemas.get(name) {
                properties.extend(schema_properties(target, schemas));
            }
        }
    }
    if let Some(parts) = schema.get("allOf").and_then(Value::as_array) {
        for part in parts {
            properties.extend(schema_properties(part, schemas));
        }
    }
    properties
}

#[tokio::test]
async fn contract_v1_openapi_surfaces_canonical_compatibility_fields() {
    let spec = openapi_spec().await;
    let schemas = spec["components"]["schemas"]
        .as_object()
        .expect("components.schemas object");

    let cases = [
        (
            "ApiEngineInput",
            [
                "contract_version",
                "consciousness_level",
                "parameters",
                "birth_data",
                "current_time",
                "location",
                "precision",
                "options",
                "image_data",
                "audio_ref",
                "consent",
                "quality",
            ]
            .as_slice(),
        ),
        (
            "ApiEngineOutputResponse",
            [
                "contract_version",
                "envelope_version",
                "engine_id",
                "result",
                "witness_prompt",
                "witness_prompts",
                "consciousness_level",
                "calculated_at",
                "processing_time_ms",
                "provenance",
                "generated_image",
                "generated_audio",
            ]
            .as_slice(),
        ),
        (
            "ErrorResponse",
            [
                "contract_version",
                "status",
                "error_code",
                "message",
                "error",
                "details",
                "trace_id",
            ]
            .as_slice(),
        ),
    ];

    for (schema_name, expected_fields) in cases {
        let properties = schema_properties(&schemas[schema_name], schemas);
        for &field in expected_fields {
            assert!(
                properties.contains(field),
                "{schema_name} must expose canonical v1 field {field}"
            );
        }
    }
}

#[tokio::test]
async fn contract_v1_result_schema_accepts_the_additive_api_envelope() {
    let canonical: Value = serde_json::from_str(include_str!(
        "../../../contracts/v1/schemas/engine-result.schema.json"
    ))
    .expect("canonical result schema JSON");
    let canonical_properties: BTreeSet<String> = canonical["properties"]
        .as_object()
        .expect("canonical result properties")
        .keys()
        .cloned()
        .collect();
    let canonical_required: BTreeSet<String> = canonical["required"]
        .as_array()
        .expect("canonical result required")
        .iter()
        .map(|value| value.as_str().expect("required field string").to_string())
        .collect();

    let spec = openapi_spec().await;
    let schemas = spec["components"]["schemas"]
        .as_object()
        .expect("components.schemas object");
    let api_properties = schema_properties(&schemas["ApiEngineOutputResponse"], schemas);

    for field in &api_properties {
        assert!(
            canonical_properties.contains(field),
            "canonical result schema must permit additive API field {field}"
        );
    }
    for field in &canonical_required {
        assert!(
            api_properties.contains(field),
            "API result envelope must represent canonical required field {field}"
        );
    }
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
