use noesis_core::contract::{
    CapabilityAvailability, ContractEngineRequest, ContractEngineResult, ContractError,
    EngineCapability, RuntimeKind,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

fn fixture<T: DeserializeOwned>(source: &str) -> T {
    serde_json::from_str(source).expect("canonical fixture must deserialize")
}

fn assert_round_trip<T>(source: &str)
where
    T: DeserializeOwned + Serialize,
{
    let expected: Value = serde_json::from_str(source).expect("fixture must be JSON");
    let parsed: T = fixture(source);
    let actual = serde_json::to_value(parsed).expect("contract type must serialize");
    assert_eq!(actual, expected);
}

#[test]
fn canonical_request_deserializes_typed_consent_and_quality() {
    let request: ContractEngineRequest = fixture(include_str!(
        "../../../contracts/v1/fixtures/engine-request.json"
    ));

    assert_eq!(request.contract_version.as_str(), "v1");
    assert_eq!(request.consciousness_level, 2);
    assert_eq!(
        request.image_data.unwrap().consent.unwrap().scopes,
        ["face-image"]
    );
    assert_eq!(request.quality.unwrap().score, Some(0.92));
}

#[test]
fn canonical_legacy_request_round_trips_without_field_drift() {
    assert_round_trip::<ContractEngineRequest>(include_str!(
        "../../../contracts/v1/fixtures/engine-request-legacy.json"
    ));
}

#[test]
fn canonical_result_deserializes_typed_provenance_and_prompts() {
    let result: ContractEngineResult = fixture(include_str!(
        "../../../contracts/v1/fixtures/engine-result.json"
    ));

    assert_eq!(result.contract_version.as_str(), "v1");
    assert_eq!(result.engine_id, "numerology");
    assert_eq!(result.witness_prompts.len(), 1);
    assert_eq!(result.provenance.runtime_kind, RuntimeKind::Native);
    assert!(!result.provenance.fallback_used);
}

#[test]
fn canonical_error_deserializes_and_round_trips() {
    let source = include_str!("../../../contracts/v1/fixtures/error.json");
    let error: ContractError = fixture(source);

    assert_eq!(error.status, 422);
    assert_eq!(error.error_code, "VALIDATION_ERROR");
    assert_round_trip::<ContractError>(source);
}

#[test]
fn canonical_capability_deserializes_runtime_truth() {
    let capability: EngineCapability = fixture(include_str!(
        "../../../contracts/v1/fixtures/engine-capability.json"
    ));

    assert_eq!(capability.contract_version.as_str(), "v1");
    assert_eq!(capability.availability, CapabilityAvailability::Available);
    assert_eq!(capability.runtime_kind, RuntimeKind::Native);
    assert!(capability.dependencies.is_empty());
}
