//! Snapshot tests for the Panchanga engine
//!
//! Compares live engine output against golden fixture files in
//! `tests/fixtures/expected_outputs/panchanga/` for all 10 reference
//! users defined in `tests/fixtures/birth_data.json`.
//!
//! Non-deterministic metadata fields (`calculation_time_ms` and `timestamp`)
//! are excluded from comparison so the test remains stable across runs.

use engine_panchanga::PanchangaEngine;
use noesis_core::{BirthData, ConsciousnessEngine, EngineInput, EngineOutput, Precision};
use serde_json::Value;
use std::{collections::HashMap, fs, path::Path};

// ---------------------------------------------------------------------------
// Reference user data (mirrors birth_data.json reference_users array)
// ---------------------------------------------------------------------------

struct RefUser {
    id: &'static str,
    date: &'static str,
    time: &'static str,
    latitude: f64,
    longitude: f64,
    timezone: &'static str,
}

const REFERENCE_USERS: &[RefUser] = &[
    RefUser {
        id: "user_nyc_1990",
        date: "1990-01-15",
        time: "14:30",
        latitude: 40.7128,
        longitude: -74.006,
        timezone: "America/New_York",
    },
    RefUser {
        id: "user_london_1985",
        date: "1985-06-20",
        time: "09:15",
        latitude: 51.5074,
        longitude: -0.1278,
        timezone: "Europe/London",
    },
    RefUser {
        id: "user_tokyo_1995",
        date: "1995-12-03",
        time: "18:45",
        latitude: 35.6762,
        longitude: 139.6503,
        timezone: "Asia/Tokyo",
    },
    RefUser {
        id: "user_sydney_1988",
        date: "1988-03-21",
        time: "06:00",
        latitude: -33.8688,
        longitude: 151.2093,
        timezone: "Australia/Sydney",
    },
    RefUser {
        id: "user_mumbai_1992",
        date: "1992-07-10",
        time: "22:30",
        latitude: 19.076,
        longitude: 72.8777,
        timezone: "Asia/Kolkata",
    },
    RefUser {
        id: "user_berlin_1979",
        date: "1979-11-25",
        time: "11:45",
        latitude: 52.52,
        longitude: 13.405,
        timezone: "Europe/Berlin",
    },
    RefUser {
        id: "user_saopaulo_2000",
        date: "2000-02-14",
        time: "03:15",
        latitude: -23.5505,
        longitude: -46.6333,
        timezone: "America/Sao_Paulo",
    },
    RefUser {
        id: "user_cairo_1975",
        date: "1975-09-08",
        time: "16:00",
        latitude: 30.0444,
        longitude: 31.2357,
        timezone: "Africa/Cairo",
    },
    RefUser {
        id: "user_la_1998",
        date: "1998-04-30",
        time: "20:15",
        latitude: 34.0522,
        longitude: -118.2437,
        timezone: "America/Los_Angeles",
    },
    RefUser {
        id: "user_delhi_1983",
        date: "1983-08-17",
        time: "07:45",
        latitude: 28.6139,
        longitude: 77.209,
        timezone: "Asia/Kolkata",
    },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Path to the panchanga fixture directory, resolved relative to this
/// crate's manifest directory (two levels up from the workspace root).
fn fixture_path(user_id: &str) -> std::path::PathBuf {
    let manifest = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest)
        .join("../../tests/fixtures/expected_outputs/panchanga")
        .join(format!("{user_id}.json"))
}

/// Strip non-deterministic metadata before comparing.
fn normalize(output: &EngineOutput) -> Value {
    let mut v = serde_json::to_value(output).expect("serialize EngineOutput");
    if let Some(meta) = v.get_mut("metadata") {
        let m = meta.as_object_mut().expect("metadata is object");
        m.remove("calculation_time_ms");
        m.remove("timestamp");
    }
    v
}

/// Load a fixture file and strip the same non-deterministic fields.
fn load_fixture(path: &Path) -> Value {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read fixture {}: {e}", path.display()));
    let mut v: Value = serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("invalid JSON in fixture {}: {e}", path.display()));
    if let Some(meta) = v.get_mut("metadata") {
        let m = meta.as_object_mut().expect("metadata is object");
        m.remove("calculation_time_ms");
        m.remove("timestamp");
    }
    v
}

/// Floating-point fields in `result` that are compared approximately.
/// All other fields (names, indices, engine_id, witness_prompt, metadata) are
/// compared exactly.
const F64_RESULT_FIELDS: &[&str] = &[
    "solar_longitude",
    "lunar_longitude",
    "tithi_value",
    "nakshatra_value",
    "yoga_value",
    "karana_value",
    "julian_day",
];

/// Tolerance for continuous astronomical values (arc-seconds — well within
/// the precision margin of the underlying Swiss Ephemeris calls).
const F64_TOLERANCE: f64 = 1e-4;

/// Assert `actual` matches `expected`, using approximate equality for
/// known floating-point result fields and exact equality elsewhere.
fn assert_output_matches(actual: &Value, expected: &Value, user_id: &str) {
    // Top-level scalar fields
    assert_eq!(
        actual["engine_id"], expected["engine_id"],
        "engine_id mismatch for {user_id}"
    );
    assert_eq!(
        actual["witness_prompt"], expected["witness_prompt"],
        "witness_prompt mismatch for {user_id}"
    );
    assert_eq!(
        actual["consciousness_level"], expected["consciousness_level"],
        "consciousness_level mismatch for {user_id}"
    );

    // Metadata (non-f64 fields)
    assert_eq!(
        actual["metadata"]["backend"], expected["metadata"]["backend"],
        "metadata.backend mismatch for {user_id}"
    );
    assert_eq!(
        actual["metadata"]["precision_achieved"], expected["metadata"]["precision_achieved"],
        "metadata.precision_achieved mismatch for {user_id}"
    );
    assert_eq!(
        actual["metadata"]["cached"], expected["metadata"]["cached"],
        "metadata.cached mismatch for {user_id}"
    );
    assert_eq!(
        actual["metadata"]["engine_version"], expected["metadata"]["engine_version"],
        "metadata.engine_version mismatch for {user_id}"
    );

    // Result: exact fields
    let result_exact_fields = [
        "tithi_index",
        "tithi_name",
        "nakshatra_index",
        "nakshatra_name",
        "yoga_index",
        "yoga_name",
        "karana_index",
        "karana_name",
        "vara_index",
        "vara_name",
    ];
    for field in &result_exact_fields {
        assert_eq!(
            actual["result"][field], expected["result"][field],
            "result.{field} mismatch for {user_id}"
        );
    }

    // Result: approximate float fields
    for field in F64_RESULT_FIELDS {
        let a = actual["result"][field]
            .as_f64()
            .unwrap_or_else(|| panic!("result.{field} not a float in actual for {user_id}"));
        let e = expected["result"][field]
            .as_f64()
            .unwrap_or_else(|| panic!("result.{field} not a float in fixture for {user_id}"));
        assert!(
            (a - e).abs() <= F64_TOLERANCE,
            "result.{field} mismatch for {user_id}: actual={a:.15}, expected={e:.15}, diff={:.2e}",
            (a - e).abs()
        );
    }
}

// ---------------------------------------------------------------------------
// Snapshot tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_panchanga_snapshot_all_reference_users() {
    let engine = PanchangaEngine::new();

    for user in REFERENCE_USERS {
        let input = EngineInput {
            birth_data: Some(BirthData {
                name: None,
                date: user.date.to_string(),
                time: Some(user.time.to_string()),
                latitude: user.latitude,
                longitude: user.longitude,
                timezone: user.timezone.to_string(),
            }),
            current_time: chrono::Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        };

        let output = engine
            .calculate(input)
            .await
            .unwrap_or_else(|e| panic!("engine failed for {}: {e}", user.id));

        let path = fixture_path(user.id);
        let expected = load_fixture(&path);
        let actual = normalize(&output);

        assert_output_matches(&actual, &expected, user.id);
    }
}

/// Each reference user fixture must have a non-empty `result` block and the
/// expected `engine_id`.
#[test]
fn test_fixtures_are_well_formed() {
    for user in REFERENCE_USERS {
        let path = fixture_path(user.id);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read fixture {}: {e}", path.display()));
        let v: Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("invalid JSON in {}: {e}", path.display()));

        assert_eq!(
            v["engine_id"].as_str(),
            Some("panchanga"),
            "wrong engine_id in {}",
            path.display()
        );
        assert!(
            v["result"].is_object(),
            "result must be an object in {}",
            path.display()
        );
        assert!(
            !v["witness_prompt"].as_str().unwrap_or("").is_empty(),
            "witness_prompt must not be empty in {}",
            path.display()
        );
        assert!(
            v["result"]["tithi_name"].as_str().is_some(),
            "tithi_name missing in {}",
            path.display()
        );
        assert!(
            v["result"]["nakshatra_name"].as_str().is_some(),
            "nakshatra_name missing in {}",
            path.display()
        );
    }
}
