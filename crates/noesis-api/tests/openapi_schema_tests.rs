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

/// #1191: the Panchanga result schema documents which path produced the
/// longitudes and whether the birth time was assumed.
#[tokio::test]
async fn panchanga_result_schema_documents_provenance_fields() {
    let spec = openapi_spec().await;
    let schemas = spec["components"]["schemas"]
        .as_object()
        .expect("components.schemas object");
    let panchanga = &schemas["PanchangaResultSchema"];
    let properties = schema_properties(panchanga, schemas);
    assert!(properties.contains("position_source"));
    assert!(properties.contains("birth_time_assumed"));
    assert!(properties.contains("vara_basis"));
    assert!(properties.contains("ayanamsa_degrees"));
    assert_eq!(
        panchanga["properties"]["birth_time_assumed"]["type"],
        "boolean"
    );

    let source = &schemas["PanchangaPositionSourceSchema"];
    let values: BTreeSet<&str> = source["enum"]
        .as_array()
        .expect("position source enum")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert_eq!(
        values,
        BTreeSet::from([
            "swiss_ephemeris_files",
            "swiss_ephemeris_moshier",
            "mean_longitude"
        ])
    );
    assert_eq!(
        schemas["PanchangaVaraBasisSchema"]["enum"],
        serde_json::json!(["local_date"])
    );
    let number_type = &panchanga["properties"]["ayanamsa_degrees"]["type"];
    assert!(
        number_type == "number"
            || number_type
                .as_array()
                .is_some_and(|types| types.contains(&serde_json::json!("number")))
    );
}

/// The documented Panchanga schema must describe exactly what the engine
/// serializes: same property names, same JSON types, and only the fields old
/// results may lack are optional. Fails when either side drifts.
#[tokio::test]
async fn panchanga_result_schema_matches_engine_output() {
    let result = engine_panchanga::compute_panchanga("2000-01-01", "12:00", 0.0);
    let actual = serde_json::to_value(&result).expect("serialize PanchangaResult");
    let actual = actual.as_object().expect("PanchangaResult is an object");

    let spec = openapi_spec().await;
    let schemas = spec["components"]["schemas"]
        .as_object()
        .expect("components.schemas object");
    let panchanga = &schemas["PanchangaResultSchema"];
    let documented = schema_properties(panchanga, schemas);
    let emitted: BTreeSet<String> = actual.keys().cloned().collect();
    assert_eq!(
        documented, emitted,
        "schema properties vs engine output keys"
    );

    let optional = BTreeSet::from(["position_source", "vara_basis", "ayanamsa_degrees"]);
    let required: BTreeSet<&str> = panchanga["required"]
        .as_array()
        .expect("required list")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let expected_required: BTreeSet<&str> = emitted
        .iter()
        .map(String::as_str)
        .filter(|key| !optional.contains(key))
        .collect();
    assert_eq!(required, expected_required);

    // Each emitted JSON value has the type the schema declares. Nullable
    // fields declare `["<type>", "null"]` or a `$ref`/`oneOf` to an enum.
    for (key, value) in actual {
        let property = &panchanga["properties"][key];
        let declared: Vec<&str> = match &property["type"] {
            Value::String(t) => vec![t.as_str()],
            Value::Array(ts) => ts.iter().filter_map(Value::as_str).collect(),
            _ => continue, // enum reference; covered by the provenance test
        };
        let emitted_type = match value {
            Value::Bool(_) => "boolean",
            Value::String(_) => "string",
            Value::Number(n) if n.is_u64() || n.is_i64() => "integer",
            Value::Number(_) => "number",
            Value::Null => "null",
            _ => "object",
        };
        let ok = declared.contains(&emitted_type)
            || (emitted_type == "integer" && declared.contains(&"number"));
        assert!(
            ok,
            "{key}: emitted {emitted_type}, schema declares {declared:?}"
        );
    }
}

/// Resolve a property schema to the object/array/scalar schema it describes,
/// following a `$ref` and the nullable `oneOf`/`allOf` wrappers utoipa emits.
fn resolve<'a>(schema: &'a Value, schemas: &'a serde_json::Map<String, Value>) -> &'a Value {
    if let Some(name) = schema
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|r| r.rsplit('/').next())
    {
        return resolve(&schemas[name], schemas);
    }
    for wrapper in ["oneOf", "allOf", "anyOf"] {
        if let Some(parts) = schema.get(wrapper).and_then(Value::as_array) {
            if let Some(part) = parts
                .iter()
                .find(|p| p.get("type").and_then(Value::as_str) != Some("null"))
            {
                return resolve(part, schemas);
            }
        }
    }
    schema
}

fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Bool(_) => "boolean",
        Value::String(_) => "string",
        Value::Number(n) if n.is_u64() || n.is_i64() => "integer",
        Value::Number(_) => "number",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
        Value::Null => "null",
    }
}

/// Walk an emitted value against its schema: every object key is documented,
/// every required property is present, scalar types and enum values match.
/// Records each documented object path whose properties were all observed.
fn assert_value_matches(
    path: &str,
    value: &Value,
    schema: &Value,
    schemas: &serde_json::Map<String, Value>,
    seen: &mut std::collections::BTreeMap<String, BTreeSet<String>>,
) {
    let schema = resolve(schema, schemas);
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        assert!(
            values.contains(value),
            "{path}: {value} not in enum {values:?}"
        );
        return;
    }
    let declared: Vec<&str> = match &schema["type"] {
        Value::String(t) => vec![t.as_str()],
        Value::Array(ts) => ts.iter().filter_map(Value::as_str).collect(),
        _ => panic!("{path}: schema declares no type: {schema}"),
    };
    let emitted = json_type(value);
    assert!(
        declared.contains(&emitted) || (emitted == "integer" && declared.contains(&"number")),
        "{path}: emitted {emitted}, schema declares {declared:?}"
    );
    match value {
        Value::Object(object) => {
            let properties = schema["properties"]
                .as_object()
                .unwrap_or_else(|| panic!("{path}: object schema has no properties"));
            for key in object.keys() {
                assert!(
                    properties.contains_key(key),
                    "{path}.{key} is emitted but not documented"
                );
            }
            for required in schema["required"].as_array().into_iter().flatten() {
                let required = required.as_str().unwrap();
                assert!(
                    object.contains_key(required),
                    "{path}.{required} is required by the schema but not emitted"
                );
            }
            seen.entry(path.to_string())
                .or_default()
                .extend(object.keys().cloned());
            for (key, child) in object {
                assert_value_matches(
                    &format!("{path}.{key}"),
                    child,
                    &properties[key],
                    schemas,
                    seen,
                );
            }
        }
        Value::Array(items) => {
            for item in items {
                assert_value_matches(&format!("{path}[]"), item, &schema["items"], schemas, seen);
            }
        }
        _ => {}
    }
}

/// #1311: `EnneagramResultSchema` must describe what the TypeScript engine
/// actually emits in every mode. The fixture is generated from the live engine
/// and `ts-engines/tests/enneagram_provenance.test.ts` fails if it goes stale,
/// so this fails whenever the schema and the engine drift apart.
#[tokio::test]
async fn enneagram_result_schema_matches_engine_output() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../ts-engines/tests/fixtures/enneagram/result-modes.json"
    ))
    .expect("enneagram fixture JSON");
    let spec = openapi_spec().await;
    let schemas = spec["components"]["schemas"]
        .as_object()
        .expect("components.schemas object");
    let schema = &schemas["EnneagramResultSchema"];

    let mut seen = std::collections::BTreeMap::new();
    let cases = fixture["cases"].as_array().expect("fixture cases");
    assert_eq!(cases.len(), 7);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        assert_value_matches(name, &case["result"], schema, schemas, &mut seen);
    }

    // Nothing documented that the engine never emits: across all modes, every
    // documented property of the top-level result and of `assessment` is seen.
    let observed = |suffix: &str| -> BTreeSet<String> {
        seen.iter()
            .filter(|(path, _)| {
                let rest = path.split_once('.').map(|(_, r)| r).unwrap_or("");
                rest == suffix
            })
            .flat_map(|(_, keys)| keys.iter().cloned())
            .collect()
    };
    assert_eq!(schema_properties(schema, schemas), observed(""));
    let assessment = resolve(&schema["properties"]["assessment"], schemas);
    assert_eq!(
        schema_properties(assessment, schemas),
        observed("assessment")
    );

    // The two discriminators are closed enums.
    let enum_of = |property: &str| -> BTreeSet<String> {
        resolve(&schema["properties"][property], schemas)["enum"]
            .as_array()
            .unwrap_or_else(|| panic!("{property} is not an enum"))
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    };
    assert_eq!(
        enum_of("mode"),
        BTreeSet::from(["assessment".into(), "lookup".into(), "questions".into()])
    );
    assert_eq!(
        enum_of("typeSource"),
        BTreeSet::from(["assessment".into(), "user-supplied".into(), "none".into()])
    );
}
