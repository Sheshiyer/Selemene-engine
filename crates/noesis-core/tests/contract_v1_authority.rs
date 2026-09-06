use noesis_core::contract::{
    CapabilityAvailability, ContractEngineRequest, ContractEngineResult, ContractError,
    EngineCapability, RuntimeKind,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

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

fn engine_registry() -> Value {
    fixture(include_str!(
        "../../../contracts/v1/registries/engines.json"
    ))
}

fn engine_issue_index() -> Value {
    fixture(include_str!(
        "../../../docs/plans/selemene-engine/ENGINE-ISSUE-INDEX.json"
    ))
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
    assert_eq!(result.witness_prompts.as_ref().unwrap().len(), 1);
    assert_eq!(
        result.provenance.as_ref().unwrap().runtime_kind,
        RuntimeKind::Native
    );
    assert!(!result.provenance.as_ref().unwrap().fallback_used);
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

#[test]
fn canonical_engine_registry_preserves_counts_owners_and_evidence_axes() {
    let registry = engine_registry();
    let rows = registry["engines"]
        .as_array()
        .expect("canonical engine registry must contain rows");
    let expected_axes = BTreeSet::from([
        "declared",
        "implemented",
        "executable",
        "integrated",
        "deployed",
        "operational",
    ]);

    let ids = rows
        .iter()
        .map(|row| {
            row["id"]
                .as_str()
                .expect("each registry row must have a runtime ID")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(rows.len(), 19);
    assert_eq!(ids.len(), 19);

    let class_counts = rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        let runtime_class = row["runtime_class"]
            .as_str()
            .expect("each registry row must have a runtime class");
        *counts.entry(runtime_class).or_insert(0usize) += 1;
        counts
    });
    assert_eq!(class_counts["native"], 12);
    assert_eq!(class_counts["database-conditional"], 1);
    assert_eq!(class_counts["typescript"], 6);

    let public_groups = rows
        .iter()
        .filter_map(|row| row["public_mirror_group"].as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(public_groups.len(), 17);

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for row in rows {
        let engine_id = row["id"].as_str().expect("runtime ID");
        let owner = row["owner"].as_str().expect("owner");
        assert!(
            repo_root.join(owner).exists(),
            "{engine_id} owner path does not exist: {owner}"
        );

        let evidence = row["evidence"]
            .as_object()
            .expect("evidence must be an object");
        assert_eq!(
            evidence.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            expected_axes,
            "{engine_id} must retain all six evidence axes"
        );
        for axis in ["deployed", "operational"] {
            assert_ne!(
                evidence[axis]["status"].as_str(),
                Some("evidenced"),
                "{engine_id} cannot claim complete {axis} evidence from registry declaration"
            );
        }
    }
}

#[test]
fn canonical_engine_registry_reuses_existing_issue_ids() {
    let registry = engine_registry();
    let issue_index = engine_issue_index();
    let rows = registry["engines"].as_array().expect("registry rows");
    let indexed_engines = issue_index["engines"]
        .as_object()
        .expect("issue index engine groups");
    let roles = [
        ("authority_baseline", 0usize),
        ("runtime_registration", 10usize),
        ("golden_fixtures", 26usize),
        ("release_gate", 28usize),
        ("deployment_recovery", 29usize),
    ];

    for row in rows {
        let engine_id = row["id"].as_str().expect("runtime ID");
        let indexed = indexed_engines[engine_id]
            .as_array()
            .expect("engine issue group");
        for (role, index) in roles {
            assert_eq!(
                row["issue_ids"][role].as_u64(),
                indexed[index]["number"].as_u64(),
                "{engine_id} must reuse the existing {role} issue ID"
            );
        }
    }
}

#[test]
fn database_conditional_registry_view_is_explicit_with_and_without_configuration() {
    let registry = engine_registry();
    let rows = registry["engines"].as_array().expect("registry rows");
    let without_database = rows
        .iter()
        .filter(|row| row["runtime_class"] != "database-conditional")
        .map(|row| row["id"].as_str().expect("runtime ID"))
        .collect::<BTreeSet<_>>();
    assert_eq!(without_database.len(), 18);
    assert!(!without_database.contains("biofield-capture"));

    let with_database = rows
        .iter()
        .map(|row| row["id"].as_str().expect("runtime ID"))
        .collect::<BTreeSet<_>>();
    assert_eq!(with_database.len(), 19);
    assert!(with_database.contains("biofield-capture"));
}

#[test]
fn canonical_result_accepts_singular_prompt_without_provenance() {
    let result: ContractEngineResult = serde_json::from_value(serde_json::json!({
        "contract_version": "v1",
        "engine_id": "numerology",
        "result": {},
        "consciousness_level": 2,
        "witness_prompt": "What is witnessed?",
        "calculated_at": "2026-08-26T06:30:00Z",
        "processing_time_ms": 1.0
    }))
    .expect("every canonical-compatible result must deserialize");

    assert_eq!(result.witness_prompt.as_deref(), Some("What is witnessed?"));
    assert!(result.witness_prompts.is_none());
    assert!(result.provenance.is_none());
}
