//! Process-level check (own test binary): with no ephemeris directory
//! reachable from the environment, working directory or executable, Swiss
//! Ephemeris runs on built-in Moshier and vimshottari declares the fallback.
use chrono::{DateTime, Utc};
use engine_human_design::{active_ephemeris_path, HumanDesignEngine, EPHEMERIS_PATH_ENV_VARS};
use engine_vimshottari::{ConsciousnessEngine, EngineInput, VimshottariEngine};
use noesis_core::{BirthData, Precision};
use std::collections::HashMap;

#[tokio::test]
async fn missing_ephemeris_files_report_moshier_fallback() {
    for name in EPHEMERIS_PATH_ENV_VARS {
        std::env::remove_var(name);
    }
    std::env::set_current_dir(std::env::temp_dir()).unwrap();

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
    let output = VimshottariEngine::new()
        .calculate(input.clone())
        .await
        .unwrap();

    assert_eq!(active_ephemeris_path(), Some(""));
    assert_eq!(
        output.metadata.backend,
        "swiss-ephemeris-lahiri-full-v1:moshier"
    );
    assert!(output.metadata.fallback_used);

    // Human Design now surfaces the same Moshier substitution.
    let hd = HumanDesignEngine::new().calculate(input).await.unwrap();
    assert_eq!(hd.metadata.backend, "swiss-ephemeris");
    assert!(hd.metadata.fallback_used);
}
