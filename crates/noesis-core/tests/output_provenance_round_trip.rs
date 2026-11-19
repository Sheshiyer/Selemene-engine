use noesis_core::EngineOutput;
use serde_json::{json, Value};

const RESERVED: &str = "__contract_provenance";

fn legacy_output() -> Value {
    json!({
        "engine_id": "synthetic-probe",
        "result": {"value": 42, "provenance": {"source": "engine-owned"}},
        "witness_prompt": "What do you notice?",
        "consciousness_level": 0,
        "metadata": {
            "calculation_time_ms": 0.0, "backend": "typescript",
            "precision_achieved": "exact", "cached": false,
            "timestamp": "2026-11-04T00:00:00Z", "engine_version": "fixture",
            "fallback_used": false
        }
    })
}

fn provenance() -> Value {
    json!({"runtime_kind": "typescript", "implementation_version": "1.0.0",
           "cached": false, "fallback_used": false, "confidence": 0.75})
}

#[test]
fn api_contract_provenance_survives_a_shared_output_round_trip() {
    let mut wire = legacy_output();
    wire["provenance"] = provenance();
    // The API also supplies envelope fields not present in the legacy runtime DTO.
    wire["envelope_version"] = json!("1");
    let output: EngineOutput = serde_json::from_value(wire.clone()).unwrap();
    let saved = serde_json::to_value(&output).unwrap();
    assert_eq!(saved["provenance"], wire["provenance"]);
    assert_eq!(saved["result"], wire["result"]);
    assert!(saved["result"].get(RESERVED).is_none());
}

#[test]
fn legacy_outputs_do_not_gain_invented_contract_provenance() {
    let wire = legacy_output();
    let output: EngineOutput = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(output).unwrap(), wire);
}

#[test]
fn bridge_contract_scaffolding_is_lifted_without_mutating_the_output() {
    let mut wire = legacy_output();
    wire["result"][RESERVED] = provenance();
    let output: EngineOutput = serde_json::from_value(wire.clone()).unwrap();
    let saved = serde_json::to_value(&output).unwrap();
    assert_eq!(saved["provenance"], provenance());
    assert_eq!(saved["result"]["provenance"], wire["result"]["provenance"]);
    assert!(saved["result"].get(RESERVED).is_none());
    assert_eq!(output.result[RESERVED], provenance());
}

#[test]
fn malformed_envelope_provenance_is_rejected_instead_of_silently_dropped() {
    for invalid in [
        json!({"runtime_kind": "bogus"}),
        json!({"extra": 1}),
        json!("claim"),
    ] {
        let mut wire = legacy_output();
        wire["provenance"] = invalid;
        assert!(serde_json::from_value::<EngineOutput>(wire).is_err());
    }
}

#[test]
fn conflicting_contract_blocks_are_rejected() {
    let mut wire = legacy_output();
    wire["provenance"] = provenance();
    wire["result"][RESERVED] = provenance();
    wire["result"][RESERVED]["fallback_used"] = json!(true);
    assert!(serde_json::from_value::<EngineOutput>(wire).is_err());
}

#[test]
fn a_non_object_result_cannot_silently_discard_contract_provenance() {
    let mut wire = legacy_output();
    wire["result"] = json!(42);
    let legacy: EngineOutput = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(legacy).unwrap(), wire);
    wire["provenance"] = provenance();
    assert!(serde_json::from_value::<EngineOutput>(wire).is_err());
}
