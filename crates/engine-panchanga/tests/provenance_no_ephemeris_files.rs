//! #1191 unavailable-dependency probe: no Swiss Ephemeris `.se1` files.
//!
//! The ephemeris path is chosen once per process by the first
//! `EphemerisCalculator::new`, so this probe lives in its own test binary and
//! pins a directory without `.se1` files before Panchanga runs. Swiss
//! Ephemeris then substitutes its built-in Moshier ephemeris, which Panchanga
//! must label distinctly from both the file-backed and mean-longitude paths.

use chrono::Utc;
use engine_human_design::ephemeris::EphemerisCalculator;
use engine_panchanga::{ConsciousnessEngine, EngineInput, PanchangaEngine, PositionSource};
use noesis_core::{BirthData, Precision};
use std::collections::HashMap;

#[tokio::test]
async fn probe_unavailable_ephemeris_files_reports_moshier() {
    // Must be the first ephemeris use in this process; keep this file to one test.
    EphemerisCalculator::new("/nonexistent/selemene-probe-no-se1-files");

    let input = EngineInput {
        birth_data: Some(BirthData {
            name: None,
            date: "1991-08-13".to_string(),
            time: Some("13:31".to_string()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: "Asia/Kolkata".to_string(),
        }),
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    };
    let output = PanchangaEngine::new()
        .calculate(input)
        .await
        .expect("Moshier still produces a result");

    assert!(output.metadata.fallback_used);
    assert_eq!(output.metadata.backend, "native-rust");
    assert_eq!(output.result["position_source"], "swiss_ephemeris_moshier");
    let source: PositionSource =
        serde_json::from_value(output.result["position_source"].clone()).unwrap();
    assert_eq!(source, PositionSource::SwissEphemerisMoshier);
    // Moshier is a full ephemeris, not the mean-longitude approximation: the
    // canonical synthetic fixture still lands in Uttara Phalguni.
    assert_eq!(output.result["nakshatra_name"], "Uttara Phalguni");
}
