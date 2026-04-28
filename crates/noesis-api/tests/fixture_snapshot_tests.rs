//! Snapshot tests for golden output fixture files.
//!
//! Validates all 30 fixture files (10 users × 3 engines) in
//!   tests/fixtures/expected_outputs/{numerology,human-design,gene-keys}/
//! against the EngineOutput JSON schema and engine-specific constraints.
//!
//! Run with: cargo test --test fixture_snapshot_tests

use noesis_core::EngineOutput;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns the absolute path to the repo root (works from any cwd).
fn repo_root() -> PathBuf {
    // Cargo sets CARGO_MANIFEST_DIR to the crate's directory.
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    // Go up two levels: crates/noesis-api → crates → repo root
    PathBuf::from(manifest)
        .parent() // crates/
        .unwrap()
        .parent() // repo root
        .unwrap()
        .to_path_buf()
}

fn fixture_dir(engine: &str) -> PathBuf {
    repo_root()
        .join("tests/fixtures/expected_outputs")
        .join(engine)
}

fn load_fixture(engine: &str, user_id: &str) -> EngineOutput {
    let path = fixture_dir(engine).join(format!("{}.json", user_id));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", path.display(), e));
    serde_json::from_str::<EngineOutput>(&content)
        .unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", path.display(), e))
}

const REFERENCE_USERS: &[&str] = &[
    "user_nyc_1990",
    "user_london_1985",
    "user_tokyo_1995",
    "user_sydney_1988",
    "user_mumbai_1992",
    "user_berlin_1979",
    "user_saopaulo_2000",
    "user_cairo_1975",
    "user_la_1998",
    "user_delhi_1983",
];

// ---------------------------------------------------------------------------
// Schema validation helpers
// ---------------------------------------------------------------------------

/// Validate the common EngineOutput wrapper fields.
fn validate_common(output: &EngineOutput, expected_engine_id: &str, user_id: &str) {
    assert_eq!(
        output.engine_id, expected_engine_id,
        "[{}] engine_id mismatch",
        user_id
    );
    assert!(
        !output.witness_prompt.is_empty(),
        "[{}] witness_prompt must not be empty",
        user_id
    );
    // result must be a non-null JSON object
    assert!(
        output.result.is_object(),
        "[{}] result must be a JSON object, got: {}",
        user_id,
        output.result
    );
    // metadata must have a non-negative calculation_time_ms
    assert!(
        output.metadata.calculation_time_ms >= 0.0,
        "[{}] calculation_time_ms must be >= 0",
        user_id
    );
    assert!(
        !output.metadata.backend.is_empty(),
        "[{}] backend must not be empty",
        user_id
    );
}

/// Validate a numerology result object.
fn validate_numerology_result(result: &serde_json::Value, user_id: &str) {
    for field in &["life_path", "expression", "soul_urge", "personality", "birthday", "chaldean_name"] {
        let obj = result.get(field).unwrap_or_else(|| {
            panic!("[{}] numerology result missing required field '{}'", user_id, field)
        });
        let value = obj
            .get("value")
            .and_then(|v| v.as_u64())
            .unwrap_or_else(|| panic!("[{}] '{}' missing numeric 'value'", user_id, field));
        assert!(
            (1..=33).contains(&value) || value == 0,
            "[{}] '{}' value {} is out of expected range [1,33]",
            user_id,
            field,
            value
        );
        assert!(
            obj.get("meaning").and_then(|v| v.as_str()).is_some(),
            "[{}] '{}' missing 'meaning' string",
            user_id,
            field
        );
        assert!(
            obj.get("reduction_chain").and_then(|v| v.as_array()).is_some(),
            "[{}] '{}' missing 'reduction_chain' array",
            user_id,
            field
        );
    }
}

/// Validate a human-design result object.
fn validate_human_design_result(result: &serde_json::Value, user_id: &str) {
    for field in &[
        "hd_type",
        "authority",
        "profile",
        "definition",
        "defined_centers",
        "active_channels",
        "personality_activations",
        "design_activations",
    ] {
        assert!(
            result.get(field).is_some(),
            "[{}] human-design result missing required field '{}'",
            user_id,
            field
        );
    }

    let valid_types = [
        "Generator",
        "ManifestingGenerator",
        "Manifestor",
        "Projector",
        "Reflector",
    ];
    let hd_type = result["hd_type"].as_str().unwrap_or("");
    assert!(
        valid_types.contains(&hd_type),
        "[{}] invalid hd_type '{}', expected one of {:?}",
        user_id,
        hd_type,
        valid_types
    );

    let valid_authorities = [
        "Emotional",
        "Sacral",
        "Splenic",
        "Ego",
        "Self",
        "Environmental",
        "Lunar",
        "None",
    ];
    let authority = result["authority"].as_str().unwrap_or("");
    assert!(
        valid_authorities.iter().any(|&a| authority.contains(a)),
        "[{}] invalid authority '{}', expected one of {:?}",
        user_id,
        authority,
        valid_authorities
    );

    // defined_centers must be an array
    assert!(
        result["defined_centers"].is_array(),
        "[{}] defined_centers must be an array",
        user_id
    );
    // active_channels must be an array
    assert!(
        result["active_channels"].is_array(),
        "[{}] active_channels must be an array",
        user_id
    );
    // personality_activations must be an object
    assert!(
        result["personality_activations"].is_object(),
        "[{}] personality_activations must be an object",
        user_id
    );
}

/// Validate a gene-keys result object.
fn validate_gene_keys_result(result: &serde_json::Value, user_id: &str) {
    for field in &["activation_sequence", "active_keys", "frequency_assessments"] {
        assert!(
            result.get(field).is_some(),
            "[{}] gene-keys result missing required field '{}'",
            user_id,
            field
        );
    }

    let seq = &result["activation_sequence"];
    for sub in &["lifes_work", "evolution", "radiance", "purpose"] {
        assert!(
            seq.get(sub).is_some(),
            "[{}] activation_sequence missing '{}'",
            user_id,
            sub
        );
    }

    let keys = result["active_keys"].as_array().unwrap_or_else(|| {
        panic!("[{}] active_keys must be an array", user_id)
    });
    assert!(
        !keys.is_empty(),
        "[{}] active_keys must not be empty",
        user_id
    );
    for key in keys {
        let key_number = key.get("key_number").and_then(|v| v.as_u64()).unwrap_or(0);
        assert!(
            (1..=64).contains(&key_number),
            "[{}] key_number {} is out of range [1,64]",
            user_id,
            key_number
        );
        assert!(
            key.get("shadow").and_then(|v| v.as_str()).is_some(),
            "[{}] active key missing 'shadow'",
            user_id
        );
        assert!(
            key.get("gift").and_then(|v| v.as_str()).is_some(),
            "[{}] active key missing 'gift'",
            user_id
        );
        assert!(
            key.get("siddhi").and_then(|v| v.as_str()).is_some(),
            "[{}] active key missing 'siddhi'",
            user_id
        );
    }
}

// ---------------------------------------------------------------------------
// Numerology snapshot tests (10 users)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_numerology_user_nyc_1990() {
    let output = load_fixture("numerology", "user_nyc_1990");
    validate_common(&output, "numerology", "user_nyc_1990");
    validate_numerology_result(&output.result, "user_nyc_1990");
}

#[test]
fn snapshot_numerology_user_london_1985() {
    let output = load_fixture("numerology", "user_london_1985");
    validate_common(&output, "numerology", "user_london_1985");
    validate_numerology_result(&output.result, "user_london_1985");
}

#[test]
fn snapshot_numerology_user_tokyo_1995() {
    let output = load_fixture("numerology", "user_tokyo_1995");
    validate_common(&output, "numerology", "user_tokyo_1995");
    validate_numerology_result(&output.result, "user_tokyo_1995");
}

#[test]
fn snapshot_numerology_user_sydney_1988() {
    let output = load_fixture("numerology", "user_sydney_1988");
    validate_common(&output, "numerology", "user_sydney_1988");
    validate_numerology_result(&output.result, "user_sydney_1988");
}

#[test]
fn snapshot_numerology_user_mumbai_1992() {
    let output = load_fixture("numerology", "user_mumbai_1992");
    validate_common(&output, "numerology", "user_mumbai_1992");
    validate_numerology_result(&output.result, "user_mumbai_1992");
}

#[test]
fn snapshot_numerology_user_berlin_1979() {
    let output = load_fixture("numerology", "user_berlin_1979");
    validate_common(&output, "numerology", "user_berlin_1979");
    validate_numerology_result(&output.result, "user_berlin_1979");
}

#[test]
fn snapshot_numerology_user_saopaulo_2000() {
    let output = load_fixture("numerology", "user_saopaulo_2000");
    validate_common(&output, "numerology", "user_saopaulo_2000");
    validate_numerology_result(&output.result, "user_saopaulo_2000");
}

#[test]
fn snapshot_numerology_user_cairo_1975() {
    let output = load_fixture("numerology", "user_cairo_1975");
    validate_common(&output, "numerology", "user_cairo_1975");
    validate_numerology_result(&output.result, "user_cairo_1975");
}

#[test]
fn snapshot_numerology_user_la_1998() {
    let output = load_fixture("numerology", "user_la_1998");
    validate_common(&output, "numerology", "user_la_1998");
    validate_numerology_result(&output.result, "user_la_1998");
}

#[test]
fn snapshot_numerology_user_delhi_1983() {
    let output = load_fixture("numerology", "user_delhi_1983");
    validate_common(&output, "numerology", "user_delhi_1983");
    validate_numerology_result(&output.result, "user_delhi_1983");
}

// ---------------------------------------------------------------------------
// Human Design snapshot tests (10 users)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_human_design_user_nyc_1990() {
    let output = load_fixture("human-design", "user_nyc_1990");
    validate_common(&output, "human-design", "user_nyc_1990");
    validate_human_design_result(&output.result, "user_nyc_1990");
}

#[test]
fn snapshot_human_design_user_london_1985() {
    let output = load_fixture("human-design", "user_london_1985");
    validate_common(&output, "human-design", "user_london_1985");
    validate_human_design_result(&output.result, "user_london_1985");
}

#[test]
fn snapshot_human_design_user_tokyo_1995() {
    let output = load_fixture("human-design", "user_tokyo_1995");
    validate_common(&output, "human-design", "user_tokyo_1995");
    validate_human_design_result(&output.result, "user_tokyo_1995");
}

#[test]
fn snapshot_human_design_user_sydney_1988() {
    let output = load_fixture("human-design", "user_sydney_1988");
    validate_common(&output, "human-design", "user_sydney_1988");
    validate_human_design_result(&output.result, "user_sydney_1988");
}

#[test]
fn snapshot_human_design_user_mumbai_1992() {
    let output = load_fixture("human-design", "user_mumbai_1992");
    validate_common(&output, "human-design", "user_mumbai_1992");
    validate_human_design_result(&output.result, "user_mumbai_1992");
}

#[test]
fn snapshot_human_design_user_berlin_1979() {
    let output = load_fixture("human-design", "user_berlin_1979");
    validate_common(&output, "human-design", "user_berlin_1979");
    validate_human_design_result(&output.result, "user_berlin_1979");
}

#[test]
fn snapshot_human_design_user_saopaulo_2000() {
    let output = load_fixture("human-design", "user_saopaulo_2000");
    validate_common(&output, "human-design", "user_saopaulo_2000");
    validate_human_design_result(&output.result, "user_saopaulo_2000");
}

#[test]
fn snapshot_human_design_user_cairo_1975() {
    let output = load_fixture("human-design", "user_cairo_1975");
    validate_common(&output, "human-design", "user_cairo_1975");
    validate_human_design_result(&output.result, "user_cairo_1975");
}

#[test]
fn snapshot_human_design_user_la_1998() {
    let output = load_fixture("human-design", "user_la_1998");
    validate_common(&output, "human-design", "user_la_1998");
    validate_human_design_result(&output.result, "user_la_1998");
}

#[test]
fn snapshot_human_design_user_delhi_1983() {
    let output = load_fixture("human-design", "user_delhi_1983");
    validate_common(&output, "human-design", "user_delhi_1983");
    validate_human_design_result(&output.result, "user_delhi_1983");
}

// ---------------------------------------------------------------------------
// Gene Keys snapshot tests (10 users)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_gene_keys_user_nyc_1990() {
    let output = load_fixture("gene-keys", "user_nyc_1990");
    validate_common(&output, "gene-keys", "user_nyc_1990");
    validate_gene_keys_result(&output.result, "user_nyc_1990");
}

#[test]
fn snapshot_gene_keys_user_london_1985() {
    let output = load_fixture("gene-keys", "user_london_1985");
    validate_common(&output, "gene-keys", "user_london_1985");
    validate_gene_keys_result(&output.result, "user_london_1985");
}

#[test]
fn snapshot_gene_keys_user_tokyo_1995() {
    let output = load_fixture("gene-keys", "user_tokyo_1995");
    validate_common(&output, "gene-keys", "user_tokyo_1995");
    validate_gene_keys_result(&output.result, "user_tokyo_1995");
}

#[test]
fn snapshot_gene_keys_user_sydney_1988() {
    let output = load_fixture("gene-keys", "user_sydney_1988");
    validate_common(&output, "gene-keys", "user_sydney_1988");
    validate_gene_keys_result(&output.result, "user_sydney_1988");
}

#[test]
fn snapshot_gene_keys_user_mumbai_1992() {
    let output = load_fixture("gene-keys", "user_mumbai_1992");
    validate_common(&output, "gene-keys", "user_mumbai_1992");
    validate_gene_keys_result(&output.result, "user_mumbai_1992");
}

#[test]
fn snapshot_gene_keys_user_berlin_1979() {
    let output = load_fixture("gene-keys", "user_berlin_1979");
    validate_common(&output, "gene-keys", "user_berlin_1979");
    validate_gene_keys_result(&output.result, "user_berlin_1979");
}

#[test]
fn snapshot_gene_keys_user_saopaulo_2000() {
    let output = load_fixture("gene-keys", "user_saopaulo_2000");
    validate_common(&output, "gene-keys", "user_saopaulo_2000");
    validate_gene_keys_result(&output.result, "user_saopaulo_2000");
}

#[test]
fn snapshot_gene_keys_user_cairo_1975() {
    let output = load_fixture("gene-keys", "user_cairo_1975");
    validate_common(&output, "gene-keys", "user_cairo_1975");
    validate_gene_keys_result(&output.result, "user_cairo_1975");
}

#[test]
fn snapshot_gene_keys_user_la_1998() {
    let output = load_fixture("gene-keys", "user_la_1998");
    validate_common(&output, "gene-keys", "user_la_1998");
    validate_gene_keys_result(&output.result, "user_la_1998");
}

#[test]
fn snapshot_gene_keys_user_delhi_1983() {
    let output = load_fixture("gene-keys", "user_delhi_1983");
    validate_common(&output, "gene-keys", "user_delhi_1983");
    validate_gene_keys_result(&output.result, "user_delhi_1983");
}

// ---------------------------------------------------------------------------
// Meta-test: verify all 30 fixture files exist and are non-empty
// ---------------------------------------------------------------------------

#[test]
fn all_fixture_files_exist() {
    for engine in &["numerology", "human-design", "gene-keys"] {
        let dir = fixture_dir(engine);
        assert!(
            dir.exists(),
            "Fixture directory does not exist: {}",
            dir.display()
        );
        for user_id in REFERENCE_USERS {
            let path = dir.join(format!("{}.json", user_id));
            assert!(
                path.exists(),
                "Missing fixture file: {}",
                path.display()
            );
            let metadata = std::fs::metadata(&path).unwrap();
            assert!(
                metadata.len() > 0,
                "Empty fixture file: {}",
                path.display()
            );
        }
    }
}
