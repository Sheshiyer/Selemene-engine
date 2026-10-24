//! Full-sidereal reference values from Swiss Ephemeris 2.10.03 / pyswisseph
//! 2.10.3.2, SIDM_LAHIRI with SWIEPH|SPEED|SIDEREAL and repository
//! data/ephemeris files (actual returned flags 65858). Separate processes
//! with an empty SWISS_EPHE_PATH test independent Moshier references (65860).
//! 1e-5 degrees / 180 seconds allow small ephemeris-file differences without
//! accepting either fixed-ayanamsa or tropical-minus-ayanamsa approximations.
use chrono::{DateTime, Utc};
use engine_human_design::{EphemerisCalculator, HDPlanet};
use engine_vimshottari::{
    calculate_birth_nakshatra, ConsciousnessEngine, EngineInput, VimshottariEngine,
};
use noesis_core::{BirthData, Precision};
use serde_json::json;
use std::collections::HashMap;

fn initialize_reference_ephemeris() {
    let path = std::env::var("SWISS_EPHE_PATH")
        .unwrap_or_else(|_| concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/ephemeris").into());
    EphemerisCalculator::new(path);
}

fn reference_moon(backend: &str, swiss: f64, moshier: f64) -> f64 {
    match backend {
        "swiss-ephemeris-lahiri-full-v1:swiss-files" => swiss,
        "swiss-ephemeris-lahiri-full-v1:moshier" => moshier,
        other => panic!("unexpected backend for independent reference: {other}"),
    }
}

fn input(date: &str, time: &str, timezone: &str) -> EngineInput {
    initialize_reference_ephemeris();
    EngineInput {
        birth_data: Some(BirthData {
            name: None,
            date: date.into(),
            time: Some(time.into()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: timezone.into(),
        }),
        current_time: DateTime::parse_from_rfc3339("2026-10-24T04:43:21Z")
            .unwrap()
            .with_timezone(&Utc),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

#[tokio::test]
async fn full_lahiri_moon_and_jupiter_boundary_match_reference() {
    let output = VimshottariEngine::new()
        .calculate(input("1991-08-13", "13:31", "Asia/Kolkata"))
        .await
        .unwrap();
    let moon = output.result["birth_nakshatra"]["moon_longitude"]
        .as_f64()
        .unwrap();
    let reference = reference_moon(
        &output.metadata.backend,
        159.80574124985853,
        159.805939126006,
    );
    assert!(
        (moon - reference).abs() < 1e-5,
        "full Lahiri Moon expected, got {moon}"
    );
    assert_eq!(output.result["birth_nakshatra"]["name"], "Uttara Phalguni");
    assert_eq!(
        output.result["current_period"]["mahadasha"]["planet"],
        "Jupiter"
    );
    let start = DateTime::parse_from_rfc3339(
        output.result["current_period"]["mahadasha"]["start"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let expected = DateTime::parse_from_rfc3339(if output.metadata.backend.ends_with(":moshier") {
        "2026-09-13T23:31:43Z"
    } else {
        "2026-09-14T00:18:33Z"
    })
    .unwrap();
    assert!(
        (start - expected).num_seconds().abs() <= 180,
        "wrong Jupiter boundary: {start}"
    );
}

#[tokio::test]
async fn full_lahiri_uses_birth_epoch_not_constant_offset() {
    for (date, time, swiss, moshier) in [
        (
            "2000-01-01",
            "12:00",
            199.47052873289056,
            199.47055295238044,
        ),
        (
            "2024-04-08",
            "18:00",
            354.98856835520945,
            354.98803450931035,
        ),
    ] {
        let output = VimshottariEngine::new()
            .calculate(input(date, time, "UTC"))
            .await
            .unwrap();
        let actual = output.result["birth_nakshatra"]["moon_longitude"]
            .as_f64()
            .unwrap();
        let moon = reference_moon(&output.metadata.backend, swiss, moshier);
        assert!((actual - moon).abs() < 1e-5, "{date}: {actual} != {moon}");
    }
}

#[tokio::test]
async fn timezone_equivalent_births_have_identical_timelines() {
    let engine = VimshottariEngine::new();
    let local = engine
        .calculate(input("1991-08-13", "13:31", "Asia/Kolkata"))
        .await
        .unwrap();
    let utc = engine
        .calculate(input("1991-08-13", "08:01", "UTC"))
        .await
        .unwrap();
    assert_eq!(local.result, utc.result);
}

#[test]
fn exported_birth_nakshatra_uses_sidereal_moon() {
    initialize_reference_ephemeris();
    let birth = DateTime::parse_from_rfc3339("1991-08-13T08:01:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let nakshatra = calculate_birth_nakshatra(birth, "").unwrap();
    assert_eq!(nakshatra.name, "Uttara Phalguni");
    assert_eq!(nakshatra.ruling_planet.as_str(), "Sun");
}

#[tokio::test]
async fn explicit_sidereal_longitude_is_not_converted_again() {
    let mut request = input("1991-08-13", "13:31", "Asia/Kolkata");
    request.birth_data = None;
    request
        .options
        .insert("moon_longitude".into(), json!(159.80574124985853));
    request
        .options
        .insert("birth_date".into(), json!("1991-08-13"));
    request.options.insert("birth_time".into(), json!("13:31"));
    request
        .options
        .insert("timezone".into(), json!("Asia/Kolkata"));
    let output = VimshottariEngine::new().calculate(request).await.unwrap();
    assert_eq!(
        output.result["birth_nakshatra"]["moon_longitude"],
        json!(159.80574124985853)
    );
    assert_eq!(output.metadata.backend, "moon-longitude");
}

#[test]
fn corrected_method_has_new_cache_namespace() {
    let key = VimshottariEngine::new().cache_key(&input("1991-08-13", "13:31", "Asia/Kolkata"));
    assert!(
        key.starts_with("vim:lahiri-full-v1:"),
        "old approximate cache must not be reused: {key}"
    );
    let utc = VimshottariEngine::new().cache_key(&input("1991-08-13", "13:31", "UTC"));
    assert_ne!(
        key, utc,
        "timezone must distinguish different birth instants"
    );
}

#[test]
fn sidereal_calculations_do_not_contaminate_tropical_moon_across_threads() {
    initialize_reference_ephemeris();
    let birth = DateTime::parse_from_rfc3339("1991-08-13T08:01:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let calculator = EphemerisCalculator::new("");
    let tropical = calculator
        .get_planet_position(HDPlanet::Moon, &birth)
        .unwrap()
        .longitude;
    let threads: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();
                let calc = EphemerisCalculator::new("");
                for _ in 0..8 {
                    {
                        let _guard = engine_human_design::ephemeris::EPHE_MUTEX.lock().unwrap();
                        // SAFETY: shared guard owns global sidereal state. Deliberately
                        // choose a different mode to verify each calculation sets Lahiri.
                        unsafe {
                            libswisseph_sys::tuple_result::simple::swe_set_sid_mode(0, 0.0, 0.0);
                        }
                    }
                    let output = rt
                        .block_on(VimshottariEngine::new().calculate(input(
                            "1991-08-13",
                            "08:01",
                            "UTC",
                        )))
                        .unwrap();
                    let moon = output.result["birth_nakshatra"]["moon_longitude"]
                        .as_f64()
                        .unwrap();
                    let reference = reference_moon(
                        &output.metadata.backend,
                        159.80574124985853,
                        159.805939126006,
                    );
                    assert!((moon - reference).abs() < 1e-5);
                    let after = calc
                        .get_planet_position(HDPlanet::Moon, &birth)
                        .unwrap()
                        .longitude;
                    assert!(
                        (after - tropical).abs() < 1e-10,
                        "tropical coordinates contaminated"
                    );
                }
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
}

#[test]
fn cache_distinguishes_temporal_targets() {
    let engine = VimshottariEngine::new();
    let earlier = input("1991-08-13", "13:31", "Asia/Kolkata");
    let mut later = earlier.clone();
    later.current_time += chrono::Duration::seconds(1);
    assert_ne!(engine.cache_key(&earlier), engine.cache_key(&later));
}
