//! GH #1411: the contract-v1 `witness_prompts` list (prompt, context, themes)
//! must survive the shared `EngineOutput` the bridge, API, SDK and TUI use,
//! instead of collapsing to the single `witness_prompt` string.
use noesis_core::{EngineOutput, CONTRACT_WITNESS_PROMPTS_KEY};
use serde_json::{json, Value};

fn legacy_output() -> Value {
    json!({
        "engine_id": "synthetic-probe",
        "result": {"value": 42},
        "witness_prompt": "What do you notice?",
        "consciousness_level": 0,
        "metadata": {
            "calculation_time_ms": 0.0, "backend": "typescript",
            "precision_achieved": "exact", "cached": false,
            "timestamp": "2026-11-07T00:00:00Z", "engine_version": "fixture",
            "fallback_used": false
        }
    })
}

fn prompts() -> Value {
    json!([
        {"prompt": "What do you notice?", "context": "Contemplating", "themes": ["pattern"]},
        {"prompt": "Before you begin: rest.", "context": "Meditation guidance", "themes": []}
    ])
}

#[test]
fn api_witness_prompts_survive_a_shared_output_round_trip() {
    let mut wire = legacy_output();
    wire["witness_prompts"] = prompts();
    let output: EngineOutput = serde_json::from_value(wire.clone()).unwrap();

    let parsed = output.contract_witness_prompts().expect("prompts retained");
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[1].context.as_deref(), Some("Meditation guidance"));
    assert_eq!(output.witness_prompt, "What do you notice?");

    let saved = serde_json::to_value(&output).unwrap();
    assert_eq!(saved["witness_prompts"], wire["witness_prompts"]);
    assert_eq!(saved["result"], wire["result"]);
    assert!(saved["result"].get(CONTRACT_WITNESS_PROMPTS_KEY).is_none());
}

#[test]
fn legacy_outputs_do_not_gain_invented_witness_prompts() {
    let wire = legacy_output();
    let output: EngineOutput = serde_json::from_value(wire.clone()).unwrap();
    assert!(output.contract_witness_prompts().is_none());
    assert_eq!(serde_json::to_value(output).unwrap(), wire);
}

#[test]
fn bridge_scaffolding_is_lifted_to_the_envelope_field() {
    let mut wire = legacy_output();
    wire["result"][CONTRACT_WITNESS_PROMPTS_KEY] = prompts();
    let output: EngineOutput = serde_json::from_value(wire).unwrap();
    let saved = serde_json::to_value(&output).unwrap();
    assert_eq!(saved["witness_prompts"], prompts());
    assert!(saved["result"].get(CONTRACT_WITNESS_PROMPTS_KEY).is_none());
}

#[test]
fn malformed_or_conflicting_witness_prompts_are_rejected() {
    for invalid in [
        json!("a single string"),
        json!([{"prompt": "x", "extra": 1}]),
        json!([{"context": "missing prompt"}]),
    ] {
        let mut wire = legacy_output();
        wire["witness_prompts"] = invalid;
        assert!(serde_json::from_value::<EngineOutput>(wire).is_err());
    }

    let mut wire = legacy_output();
    wire["witness_prompts"] = prompts();
    wire["result"][CONTRACT_WITNESS_PROMPTS_KEY] = json!([{"prompt": "different"}]);
    assert!(serde_json::from_value::<EngineOutput>(wire).is_err());
}

#[test]
fn a_non_object_result_keeps_the_single_prompt_instead_of_failing() {
    // Every API envelope carries witness_prompts, so rejecting here (as
    // provenance does) would break clients on responses that work today.
    let mut wire = legacy_output();
    wire["result"] = json!(42);
    wire["witness_prompts"] = prompts();
    let output: EngineOutput = serde_json::from_value(wire).unwrap();
    assert!(output.contract_witness_prompts().is_none());
    assert_eq!(output.witness_prompt, "What do you notice?");
}
