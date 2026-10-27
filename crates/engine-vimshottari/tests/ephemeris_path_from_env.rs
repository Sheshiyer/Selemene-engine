//! Process-level check (own test binary, so the process-global Swiss
//! Ephemeris path starts unset): `SWISS_EPHEMERIS_PATH`, as set by the
//! production image, drives the ephemeris even when the working directory has
//! no `data/ephemeris`, and vimshottari then reports real Swiss files.
use chrono::{DateTime, Utc};
use engine_human_design::{active_ephemeris_path, EphemerisCalculator, HDPlanet};
use engine_vimshottari::{ConsciousnessEngine, EngineInput, VimshottariEngine};
use noesis_core::{BirthData, Precision};
use std::collections::HashMap;
use std::path::Path;

#[tokio::test]
async fn swiss_ephemeris_path_env_selects_swiss_files_from_any_cwd() {
    let data = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/ephemeris")
        .canonicalize()
        .expect("repository data/ephemeris");
    std::env::remove_var("SWISS_EPHE_PATH");
    std::env::remove_var("EPHEMERIS_DIR");
    std::env::set_var("SWISS_EPHEMERIS_PATH", &data);
    // A working directory with no data/ephemeris above it, like a container
    // whose cwd is not the app root.
    std::env::set_current_dir(std::env::temp_dir()).unwrap();

    let output = VimshottariEngine::new()
        .calculate(EngineInput {
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
        })
        .await
        .unwrap();

    assert_eq!(active_ephemeris_path(), Some(data.to_str().unwrap()));
    assert_eq!(
        output.metadata.backend,
        "swiss-ephemeris-lahiri-full-v1:swiss-files"
    );
    assert!(!output.metadata.fallback_used);

    // The same pinned path serves the other Swiss Ephemeris engines.
    let calc = EphemerisCalculator::new("");
    assert_eq!(calc.data_path(), data.to_str().unwrap());
    let sun = calc
        .get_planet_position(HDPlanet::Sun, &Utc::now())
        .expect("sun position");
    assert!((0.0..360.0).contains(&sun.longitude));
}
