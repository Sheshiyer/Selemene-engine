//! Public calculate-path regressions. Fixtures are synthetic; these assert
//! preserved HD values and source bindings, not independent ephemeris accuracy.
use chrono::Utc;
use engine_gene_keys::{ConsciousnessEngine, EngineInput, GeneKeysEngine};
use noesis_core::{BirthData, Precision};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};

fn synthetic_birth_input() -> EngineInput {
    EngineInput {
        birth_data: Some(BirthData {
            name: Some("Test Reader".into()),
            date: "1990-01-01".into(),
            time: Some("12:00".into()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: "Asia/Kolkata".into(),
        }),
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

#[tokio::test]
async fn birth_calculate_preserves_each_hd_gate_line_and_canonical_binding() {
    let hd = Arc::new(engine_human_design::HumanDesignEngine::new());
    let engine = GeneKeysEngine::with_hd_engine(hd.clone());
    let input = synthetic_birth_input();
    let hd_output = hd.calculate(input.clone()).await.unwrap();
    let output = engine.calculate(input).await.unwrap();
    let mut saw_non_three_line = false;
    for (sphere, index, block, planet, source) in [
        (
            "lifes_work",
            0,
            "personality_activations",
            "sun",
            "PersonalitySun",
        ),
        (
            "evolution",
            1,
            "personality_activations",
            "earth",
            "PersonalityEarth",
        ),
        ("radiance", 2, "design_activations", "sun", "DesignSun"),
        ("purpose", 3, "design_activations", "earth", "DesignEarth"),
    ] {
        let activation = &hd_output.result[block][planet];
        saw_non_three_line |= activation["line"] != json!(3);
        assert_eq!(
            output.result["active_keys"][index]["key_number"],
            activation["gate"]
        );
        assert_eq!(
            output.result["active_keys"][index]["line"],
            activation["line"]
        );
        assert_eq!(output.result["active_keys"][index]["source"], source);
        assert_eq!(
            output.result["activation_spheres"][sphere],
            json!({"key_number":activation["gate"], "line":activation["line"], "source":source})
        );
    }
    assert!(
        saw_non_three_line,
        "fixture must detect the old constant-line bug"
    );
    assert_eq!(
        output.result["line_provenance"],
        "preserved_human_design_activations"
    );
    assert_eq!(
        output.result["meaning_source_quality"]["status"],
        "unverified_incomplete"
    );
    assert_eq!(
        output.result["frequency_assessment_context"]["measurement_status"],
        "not_measured"
    );
    assert!(output
        .witness_prompt
        .contains("not a measurement of user consciousness or frequency"));
}

#[tokio::test]
async fn gate_only_calculate_preserves_legacy_arrays_with_unavailable_lines() {
    let engine = GeneKeysEngine::new();
    let mut input = synthetic_birth_input();
    input.birth_data = None;
    input.options.insert(
        "hd_gates".into(),
        json!({"personality_sun":17,"personality_earth":18,"design_sun":45,"design_earth":26}),
    );
    let output = engine.calculate(input).await.unwrap();
    assert_eq!(
        output.result["activation_sequence"],
        json!({"lifes_work":[17,18],"evolution":[45,26],"radiance":[17,45],"purpose":[18,26]})
    );
    for key in output.result["active_keys"].as_array().unwrap() {
        assert!(key["line"].is_null());
        assert_eq!(key["meaning_source_quality"], "unverified_incomplete");
    }
    for sphere in ["lifes_work", "evolution", "radiance", "purpose"] {
        assert!(output.result["activation_spheres"][sphere]["line"].is_null());
    }
    assert_eq!(
        output.result["line_provenance"],
        "unavailable_gate_only_input"
    );
    assert_eq!(output.witness_prompt.matches("line unavailable").count(), 4);
}
