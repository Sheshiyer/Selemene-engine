//! Process-level probe (own test binary): with no ephemeris directory
//! reachable from the environment, working directory or executable, Swiss
//! Ephemeris runs on built-in Moshier and the biofield vedic path declares the
//! fallback while staying distinguishable from the mock path.
use chrono::{DateTime, Utc};
use engine_biofield::{BiofieldEngine, ConsciousnessEngine, EngineInput};
use engine_human_design::{active_ephemeris_path, EPHEMERIS_PATH_ENV_VARS};
use noesis_core::{BirthData, Precision};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn missing_ephemeris_files_report_moshier_fallback() {
    for name in EPHEMERIS_PATH_ENV_VARS {
        std::env::remove_var(name);
    }
    std::env::set_current_dir(std::env::temp_dir()).unwrap();

    // Synthetic birth data.
    let input = EngineInput {
        birth_data: Some(BirthData {
            name: None,
            date: "1991-08-13".into(),
            time: Some("13:31".into()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: "Asia/Kolkata".into(),
        }),
        current_time: DateTime::parse_from_rfc3339("2026-10-24T04:43:21Z")
            .unwrap()
            .with_timezone(&Utc),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    };
    let engine = BiofieldEngine::new();
    let output = engine.calculate(input).await.unwrap();

    assert_eq!(active_ephemeris_path(), Some(""));
    assert_eq!(output.metadata.backend, "vedic-ephemeris");
    assert!(output.metadata.fallback_used);

    let prov = &output.result["provenance"];
    assert_eq!(prov["backend"], json!("vedic-ephemeris"));
    assert_eq!(prov["fallback_used"], json!(true));
    let reason = prov["fallback_reason"].as_str().unwrap();
    assert!(reason.contains("Moshier"), "reason was {reason}");

    // Still a calculated (degraded) result, never mock.
    assert_eq!(output.result["is_mock_data"], json!(false));
    assert_eq!(output.result["computation_mode"], json!("vedic"));
    assert!(output.result.get("mock_reason").is_none());
    let validation = engine.validate(&output).await.unwrap();
    assert!(validation.valid, "{:?}", validation.messages);
}
