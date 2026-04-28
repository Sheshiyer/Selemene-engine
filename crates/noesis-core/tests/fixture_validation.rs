//! Fixture validation tests for face-reading, nadabrahman, and transits engines.
//!
//! Validates all 15 golden output JSON files against the EngineOutput schema,
//! ensuring required fields are present and well-formed.

use noesis_core::EngineOutput;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Load fixture JSON and deserialise into EngineOutput
fn load_fixture(engine: &str, user: &str) -> EngineOutput {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/expected_outputs")
        .join(engine)
        .join(format!("{}.json", user));

    let contents = fs::read_to_string(&root)
        .unwrap_or_else(|e| panic!("Cannot read fixture {}/{}.json: {}", engine, user, e));

    serde_json::from_str::<EngineOutput>(&contents)
        .unwrap_or_else(|e| panic!("Cannot parse fixture {}/{}.json: {}", engine, user, e))
}

/// Validate EngineOutput common schema fields
fn assert_common_schema(output: &EngineOutput, engine: &str, user: &str) {
    assert_eq!(
        output.engine_id, engine,
        "engine_id mismatch in {}/{}",
        engine, user
    );
    assert!(
        !output.witness_prompt.is_empty(),
        "witness_prompt is empty in {}/{}",
        engine,
        user
    );
    assert!(
        output.consciousness_level <= 5,
        "consciousness_level out of range in {}/{}",
        engine,
        user
    );
    assert!(
        output.metadata.calculation_time_ms >= 0.0,
        "negative calculation_time_ms in {}/{}",
        engine,
        user
    );
    assert!(
        !output.metadata.backend.is_empty(),
        "backend is empty in {}/{}",
        engine,
        user
    );
    assert!(
        !output.metadata.precision_achieved.is_empty(),
        "precision_achieved is empty in {}/{}",
        engine,
        user
    );
}

/// Validate face-reading result schema
fn assert_face_reading_schema(result: &Value, user: &str) {
    let analysis = result
        .get("analysis")
        .unwrap_or_else(|| panic!("Missing 'analysis' in face-reading/{}", user));

    let constitution = analysis
        .get("constitution")
        .unwrap_or_else(|| panic!("Missing 'constitution' in face-reading/{}", user));
    assert!(
        constitution.get("primary_dosha").is_some(),
        "Missing primary_dosha in face-reading/{}",
        user
    );
    assert!(
        constitution.get("tcm_element").is_some(),
        "Missing tcm_element in face-reading/{}",
        user
    );
    assert!(
        constitution.get("body_type").is_some(),
        "Missing body_type in face-reading/{}",
        user
    );

    let balance = analysis
        .get("elemental_balance")
        .unwrap_or_else(|| panic!("Missing 'elemental_balance' in face-reading/{}", user));
    for field in &["wood", "fire", "earth", "metal", "water", "dominant"] {
        assert!(
            balance.get(field).is_some(),
            "Missing elemental_balance.{} in face-reading/{}",
            field,
            user
        );
    }

    assert!(
        result.get("traditions").is_some(),
        "Missing 'traditions' in face-reading/{}",
        user
    );
    assert!(
        result.get("disclaimer").is_some(),
        "Missing 'disclaimer' in face-reading/{}",
        user
    );
}

/// Validate nadabrahman result schema
fn assert_nadabrahman_schema(result: &Value, user: &str) {
    let time_rec = result
        .get("time_recommendation")
        .unwrap_or_else(|| panic!("Missing 'time_recommendation' in nadabrahman/{}", user));

    for field in &[
        "prahar_name",
        "prahar_number",
        "time_range",
        "primary_raga",
        "dosha_dominance",
        "energy_quality",
    ] {
        assert!(
            time_rec.get(field).is_some(),
            "Missing time_recommendation.{} in nadabrahman/{}",
            field,
            user
        );
    }

    let primary_raga = time_rec.get("primary_raga").unwrap();
    assert!(
        primary_raga.get("raga_number").is_some(),
        "Missing primary_raga.raga_number in nadabrahman/{}",
        user
    );
    assert!(
        primary_raga.get("raga_name").is_some(),
        "Missing primary_raga.raga_name in nadabrahman/{}",
        user
    );

    let recommendations = result
        .get("recommendations")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("Missing or invalid 'recommendations' in nadabrahman/{}", user));

    assert!(
        !recommendations.is_empty(),
        "recommendations is empty in nadabrahman/{}",
        user
    );

    for rec in recommendations {
        assert!(
            rec.get("raga_number").is_some(),
            "Missing raga_number in recommendation in nadabrahman/{}",
            user
        );
        assert!(
            rec.get("raga_name").is_some(),
            "Missing raga_name in recommendation in nadabrahman/{}",
            user
        );
        assert!(
            rec.get("score").is_some(),
            "Missing score in recommendation in nadabrahman/{}",
            user
        );
    }
}

/// Validate transits result schema
fn assert_transits_schema(result: &Value, user: &str) {
    for field in &[
        "natal_positions",
        "transit_positions",
        "aspects",
        "sade_sati",
        "period_quality",
        "retrograde_planets",
    ] {
        assert!(
            result.get(field).is_some(),
            "Missing '{}' in transits/{}",
            field,
            user
        );
    }

    // Validate natal_positions array is non-empty
    let natal = result["natal_positions"]
        .as_array()
        .unwrap_or_else(|| panic!("natal_positions not array in transits/{}", user));
    assert!(
        !natal.is_empty(),
        "natal_positions is empty in transits/{}",
        user
    );
    for pos in natal {
        for field in &["planet", "longitude", "sign", "degree_in_sign", "is_retrograde"] {
            assert!(
                pos.get(field).is_some(),
                "Missing {field} in natal_positions entry in transits/{user}"
            );
        }
    }

    // Validate transit_positions array is non-empty
    let transits_arr = result["transit_positions"]
        .as_array()
        .unwrap_or_else(|| panic!("transit_positions not array in transits/{}", user));
    assert!(
        !transits_arr.is_empty(),
        "transit_positions is empty in transits/{}",
        user
    );

    // Validate sade_sati
    let sade = &result["sade_sati"];
    assert!(
        sade.get("is_active").is_some(),
        "Missing sade_sati.is_active in transits/{}",
        user
    );
    assert!(
        sade.get("saturn_sign").is_some(),
        "Missing sade_sati.saturn_sign in transits/{}",
        user
    );
    assert!(
        sade.get("moon_sign").is_some(),
        "Missing sade_sati.moon_sign in transits/{}",
        user
    );
}

// ---------------------------------------------------------------------------
// Face-reading fixture tests
// ---------------------------------------------------------------------------

const FACE_READING_USERS: &[&str] = &[
    "user_nyc_1990",
    "user_london_1985",
    "user_tokyo_1995",
    "user_sydney_1988",
    "user_mumbai_1992",
];

#[test]
fn face_reading_fixtures_schema_valid() {
    for user in FACE_READING_USERS {
        let output = load_fixture("face-reading", user);
        assert_common_schema(&output, "face-reading", user);
        assert_face_reading_schema(&output.result, user);
    }
}

// ---------------------------------------------------------------------------
// Nadabrahman fixture tests
// ---------------------------------------------------------------------------

const NADABRAHMAN_USERS: &[&str] = &[
    "user_nyc_1990",
    "user_london_1985",
    "user_tokyo_1995",
    "user_sydney_1988",
    "user_mumbai_1992",
];

#[test]
fn nadabrahman_fixtures_schema_valid() {
    for user in NADABRAHMAN_USERS {
        let output = load_fixture("nadabrahman", user);
        assert_common_schema(&output, "nadabrahman", user);
        assert_nadabrahman_schema(&output.result, user);
    }
}

// ---------------------------------------------------------------------------
// Transits fixture tests
// ---------------------------------------------------------------------------

const TRANSITS_USERS: &[&str] = &[
    "user_nyc_1990",
    "user_london_1985",
    "user_tokyo_1995",
    "user_sydney_1988",
    "user_mumbai_1992",
];

#[test]
fn transits_fixtures_schema_valid() {
    for user in TRANSITS_USERS {
        let output = load_fixture("transits", user);
        assert_common_schema(&output, "transits", user);
        assert_transits_schema(&output.result, user);
    }
}

// ---------------------------------------------------------------------------
// Coverage: confirm all 15 files are present and parseable
// ---------------------------------------------------------------------------

#[test]
fn all_fifteen_fixtures_present() {
    let engines_and_users = [
        ("face-reading", FACE_READING_USERS),
        ("nadabrahman", NADABRAHMAN_USERS),
        ("transits", TRANSITS_USERS),
    ];

    let mut count = 0;
    for (engine, users) in &engines_and_users {
        for user in *users {
            let _ = load_fixture(engine, user);
            count += 1;
        }
    }
    assert_eq!(count, 15, "Expected exactly 15 fixture files");
}
