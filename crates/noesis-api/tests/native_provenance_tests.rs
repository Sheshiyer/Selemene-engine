//! Lane N1: contract-v1 envelope `provenance` for native engines.
//!
//! Native engines (registered with the orchestrator, not served by the
//! TypeScript bridge) get a block derived from the metadata each engine fills
//! in. Bridge engines keep their sidecar-lifted block or none.

mod common;

use axum::http::StatusCode;
use common::test_harness::RoutingHarness;
use noesis_core::contract::{Provenance, RuntimeKind};
use serde_json::{json, Value};
use std::collections::BTreeSet;

const SEVEN_FIELDS: [&str; 7] = [
    "runtime_kind",
    "implementation_version",
    "cached",
    "fallback_used",
    "backend_id",
    "provider_id",
    "confidence",
];

const BRIDGE_ENGINE_IDS: [&str; 6] = [
    "tarot",
    "i-ching",
    "enneagram",
    "sacred-geometry",
    "sigil-forge",
    "raaga",
];

/// The block uses only contract keys, carries the required ones, and parses
/// under `deny_unknown_fields` back to the identical JSON.
fn assert_contract_block(block: &Value) -> Provenance {
    let keys: BTreeSet<&str> = block
        .as_object()
        .expect("provenance must be an object")
        .keys()
        .map(String::as_str)
        .collect();
    let allowed: BTreeSet<&str> = SEVEN_FIELDS.into_iter().collect();
    assert!(keys.is_subset(&allowed), "unexpected keys in {block}");
    for required in ["runtime_kind", "implementation_version", "cached", "fallback_used"] {
        assert!(keys.contains(required), "missing {required} in {block}");
    }
    let parsed: Provenance =
        serde_json::from_value(block.clone()).expect("block parses as contract Provenance");
    assert_eq!(&serde_json::to_value(&parsed).unwrap(), block);
    parsed
}

async fn calculate(engine_id: &str, body: Value) -> (StatusCode, Value) {
    common::make_authenticated_request(
        "POST",
        &format!("/api/v1/engines/{engine_id}/calculate"),
        &common::generate_test_token(5),
        Some(body),
    )
    .await
}

#[tokio::test]
async fn native_success_carries_native_block_without_fallback() {
    let body = serde_json::to_value(common::reference_birth_input()).unwrap();
    let (status, response) = calculate("numerology", body).await;
    assert_eq!(status, StatusCode::OK, "{response}");

    let block = &response["provenance"];
    let prov = assert_contract_block(block);
    assert_eq!(prov.runtime_kind, RuntimeKind::Native);
    assert!(!prov.fallback_used);
    assert!(!prov.cached);
    assert_eq!(prov.backend_id.as_deref(), Some("native-rust"));
    assert_eq!(
        prov.implementation_version,
        response["metadata"]["engine_version"].as_str().unwrap()
    );
    assert!(prov.provider_id.is_none());
    assert!(prov.confidence.is_none());
    // Exactly the keys this block populates; nothing reserved leaks into result.
    let keys: BTreeSet<&str> = block.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        BTreeSet::from(["runtime_kind", "implementation_version", "cached", "fallback_used", "backend_id"])
    );
    assert!(response["result"].get("__contract_provenance").is_none());
}

#[tokio::test]
async fn face_reading_without_image_declares_fallback() {
    let body = serde_json::to_value(common::reference_birth_input()).unwrap();
    let (status, response) = calculate("face-reading", body).await;
    assert_eq!(status, StatusCode::OK, "{response}");

    let prov = assert_contract_block(&response["provenance"]);
    assert_eq!(prov.runtime_kind, RuntimeKind::Native);
    assert!(prov.fallback_used);
    assert_eq!(prov.backend_id.as_deref(), Some("birth-physiognomy-fallback"));
    assert_eq!(response["metadata"]["fallback_used"], json!(true));
    // The engine-owned block (kept for compatibility) agrees with the envelope.
    assert_eq!(response["result"]["provenance"]["fallback_used"], json!(true));
    assert!(response["result"]["provenance"]["fallback_reason"].is_string());
}

#[tokio::test]
async fn native_probe_gets_derived_block_and_bridge_probe_gets_none() {
    let harness = RoutingHarness::with_probe_engines(&["numerology", "sigil-forge"]).await;

    let native = harness.assert_engine_calculate_routed("numerology").await;
    let prov = assert_contract_block(&native["provenance"]);
    assert_eq!(prov.runtime_kind, RuntimeKind::Native);
    assert_eq!(prov.backend_id.as_deref(), Some("test-harness"));
    assert_eq!(prov.implementation_version, "test");

    // sigil-forge is served by the bridge: no sidecar block was stashed by the
    // probe, and a bridge engine never gets a derived substitute.
    let bridge = harness.assert_engine_calculate_routed("sigil-forge").await;
    assert!(bridge.get("provenance").is_none(), "{bridge}");
}

#[tokio::test]
async fn workflow_engine_results_carry_per_engine_provenance() {
    let engine_ids: Vec<&str> = [
        "panchanga",
        "numerology",
        "biorhythm",
        "human-design",
        "gene-keys",
        "vimshottari",
        "vedic-clock",
        "nadabrahman",
        "transits",
        "biofield",
        "face-reading",
    ]
    .into_iter()
    .chain(BRIDGE_ENGINE_IDS)
    .collect();
    let harness = RoutingHarness::with_probe_engines(&engine_ids).await;

    let (status, response) = harness
        .send_authenticated_json(
            "POST",
            "/api/v1/workflows/birth-blueprint/execute",
            Some(serde_json::to_value(common::reference_birth_input()).unwrap()),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{response}");

    let results = response["engine_results"]
        .as_object()
        .expect("engine_results object");
    assert!(!results.is_empty());
    let mut native_seen = 0;
    for (engine_id, output) in results {
        if BRIDGE_ENGINE_IDS.contains(&engine_id.as_str()) {
            assert!(output.get("provenance").is_none(), "{engine_id}: {output}");
        } else {
            let prov = assert_contract_block(&output["provenance"]);
            assert_eq!(prov.runtime_kind, RuntimeKind::Native, "{engine_id}");
            native_seen += 1;
        }
        // The persisted-shape map never carries an envelope block.
        assert!(response["engine_outputs"][engine_id].get("provenance").is_none());
        assert!(output["result"].get("__contract_provenance").is_none());
    }
    assert!(native_seen > 0, "birth-blueprint should run native engines");
}

#[test]
fn stored_reading_without_new_metadata_fields_still_deserializes() {
    // A reading persisted before lane N1: metadata has neither `fallback_used`
    // nor `provider_id`.
    let stored = json!({
        "engine_id": "numerology",
        "result": {"life_path": 7},
        "witness_prompt": "What do you notice?",
        "consciousness_level": 0,
        "metadata": {
            "calculation_time_ms": 1.2,
            "backend": "native-rust",
            "precision_achieved": "Standard",
            "cached": false,
            "timestamp": "2026-01-01T00:00:00Z",
            "engine_version": "3.0.0"
        }
    });
    let output: noesis_core::EngineOutput =
        serde_json::from_value(stored).expect("pre-N1 reading must still load");
    assert!(!output.metadata.fallback_used);
    assert!(output.metadata.provider_id.is_none());

    // And it serializes back with `provider_id` omitted when absent.
    let wire = serde_json::to_value(&output).unwrap();
    assert_eq!(wire["metadata"]["fallback_used"], json!(false));
    assert!(wire["metadata"].get("provider_id").is_none());
}
