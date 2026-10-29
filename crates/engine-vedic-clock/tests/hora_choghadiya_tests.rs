//! Hora and Choghadiya in the calculate output (#1523).
//!
//! Synthetic location and instants only. Sunrise/sunset for the hand-checked
//! case were cross-checked against Swiss Ephemeris `swe_rise_trans`
//! (Moshier, upper limb with refraction): ours differ by under 3 s.

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use engine_vedic_clock::{
    sun_event, ConsciousnessEngine, EngineError, EngineInput, SunEvent, VedicClockEngine,
};
use noesis_core::{Coordinates, Precision};
use serde_json::{json, Value};
use std::collections::HashMap;

const LAT: f64 = 12.9716;
const LON: f64 = 77.5946;
const IST: i32 = 330;

fn input(current_time: DateTime<Utc>, options: Value) -> EngineInput {
    let options: HashMap<String, Value> = serde_json::from_value(options).unwrap();
    EngineInput {
        birth_data: None,
        current_time,
        location: None,
        precision: Precision::Standard,
        options,
    }
}

fn located(current_time: DateTime<Utc>) -> EngineInput {
    input(
        current_time,
        json!({ "timezone_offset": IST, "latitude": LAT, "longitude": LON }),
    )
}

async fn result(input: EngineInput) -> Value {
    VedicClockEngine::new()
        .calculate(input)
        .await
        .expect("vedic-clock should calculate")
        .result
}

fn starts_with(value: &Value, prefix: &str) {
    let s = value
        .as_str()
        .unwrap_or_else(|| panic!("not a string: {value}"));
    assert!(s.starts_with(prefix), "{s} should start with {prefix}");
}

/// 2026-03-08 (Sunday) 12:13:16 IST at 12.9716 N, 77.5946 E.
///
/// Sunrise 06:31:37, sunset 18:29:23 IST: day = 43 066 s.
/// Hora = day / 12 = 3 588.8 s; 20 499 s after sunrise → index 5 → hora 6.
/// Sunday horas run Sun, Venus, Mercury, Moon, Saturn, Jupiter → Jupiter,
/// 11:30:41-12:30:30.
/// Choghadiya = day / 8 = 5 383.25 s → index 3 → 4th. Sunday day runs
/// Udveg, Char, Labh, Amrit → Amrit (Moon, good), 11:00:47-12:30:30.
#[tokio::test]
async fn hora_and_choghadiya_match_hand_checked_values() {
    let instant = Utc.with_ymd_and_hms(2026, 3, 8, 6, 43, 16).unwrap();
    let r = result(located(instant)).await;

    let day = &r["vedic_day"];
    assert_eq!(day["status"], "computed");
    assert_eq!(day["date"], "2026-03-08");
    assert_eq!(day["weekday"], "Sunday");
    starts_with(&day["sunrise"], "2026-03-08T06:31:3");
    starts_with(&day["sunset"], "2026-03-08T18:29:2");
    starts_with(&day["next_sunrise"], "2026-03-09T06:3");
    assert!(day["sunrise"].as_str().unwrap().ends_with("+05:30"));
    assert_eq!(day["location"]["source"], "options");
    assert_eq!(day["location"]["latitude"], LAT);
    assert_eq!(day["sun_event_method"], "noaa-solar-position");

    let hora = &r["hora"];
    assert_eq!(hora["number"], 6);
    assert_eq!(hora["ruler"], "Jupiter");
    assert_eq!(hora["period"], "day");
    assert_eq!(hora["is_favorable"], true);
    starts_with(&hora["start"], "2026-03-08T11:30:4");
    starts_with(&hora["end"], "2026-03-08T12:30:3");
    assert!(!hora["recommendations"].as_array().unwrap().is_empty());

    let chog = &r["choghadiya"];
    assert_eq!(chog["number"], 4);
    assert_eq!(chog["name"], "Amrit");
    assert_eq!(chog["nature"], "good");
    assert_eq!(chog["ruler"], "Moon");
    assert_eq!(chog["period"], "day");
    starts_with(&chog["start"], "2026-03-08T11:00:4");
    starts_with(&chog["end"], "2026-03-08T12:30:3");
    assert!(chog["suitable_for"]
        .as_array()
        .unwrap()
        .contains(&json!("General")));
}

/// Just before sunrise the instant still belongs to Saturday's night: last
/// hora (24, Mars) and last night choghadiya (8, Labh). From sunrise on it is
/// Sunday's first hora (Sun) and first choghadiya (Udveg).
#[tokio::test]
async fn sunrise_boundary_switches_vedic_day() {
    let date = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();
    let sunrise = sun_event(date, LAT, LON, IST, SunEvent::Rise).unwrap();

    let before = result(located(sunrise - Duration::seconds(1))).await;
    assert_eq!(before["vedic_day"]["date"], "2026-03-07");
    assert_eq!(before["vedic_day"]["weekday"], "Saturday");
    assert_eq!(before["hora"]["number"], 24);
    assert_eq!(before["hora"]["ruler"], "Mars");
    assert_eq!(before["hora"]["period"], "night");
    assert_eq!(before["choghadiya"]["number"], 8);
    assert_eq!(before["choghadiya"]["name"], "Labh");
    assert_eq!(before["choghadiya"]["period"], "night");
    assert_eq!(
        before["vedic_day"]["next_sunrise"], before["hora"]["end"],
        "last hora ends at the next sunrise"
    );

    for instant in [sunrise, sunrise + Duration::seconds(1)] {
        let after = result(located(instant)).await;
        assert_eq!(after["vedic_day"]["date"], "2026-03-08");
        assert_eq!(after["vedic_day"]["weekday"], "Sunday");
        assert_eq!(after["hora"]["number"], 1);
        assert_eq!(after["hora"]["ruler"], "Sun");
        assert_eq!(after["choghadiya"]["number"], 1);
        assert_eq!(after["choghadiya"]["name"], "Udveg");
        assert_eq!(after["hora"]["start"], after["vedic_day"]["sunrise"]);
        assert_eq!(
            after["vedic_day"]["sunrise"],
            before["vedic_day"]["next_sunrise"]
        );
    }
}

#[tokio::test]
async fn missing_location_omits_hora_and_choghadiya() {
    let instant = Utc.with_ymd_and_hms(2026, 3, 8, 6, 43, 16).unwrap();
    let r = result(input(instant, json!({ "timezone_offset": IST }))).await;

    assert!(r.get("hora").is_none(), "{r}");
    assert!(r.get("choghadiya").is_none(), "{r}");
    assert_eq!(r["vedic_day"]["status"], "not_computed");
    assert_eq!(r["vedic_day"]["not_computed_reason"], "location_missing");
    assert_eq!(r["vedic_day"]["omitted"], json!(["hora", "choghadiya"]));
    assert!(r["vedic_day"].get("sunrise").is_none());
    // The organ clock is unaffected.
    assert_eq!(r["current_organ"]["organ"], "Heart");
}

#[tokio::test]
async fn polar_night_reports_no_sunrise() {
    let instant = Utc.with_ymd_and_hms(2026, 12, 21, 12, 0, 0).unwrap();
    let r = result(input(
        instant,
        json!({ "timezone_offset": 60, "latitude": 78.22, "longitude": 15.65 }),
    ))
    .await;

    assert!(r.get("hora").is_none());
    assert!(r.get("choghadiya").is_none());
    assert_eq!(
        r["vedic_day"]["not_computed_reason"],
        "no_sunrise_or_sunset"
    );
    assert_eq!(r["vedic_day"]["location"]["latitude"], 78.22);
}

#[tokio::test]
async fn location_sources_follow_priority() {
    let instant = Utc.with_ymd_and_hms(2026, 3, 8, 6, 43, 16).unwrap();
    let birth = noesis_core::BirthData {
        name: None,
        date: "1991-08-13".to_string(),
        time: Some("13:31".to_string()),
        latitude: 28.6139,
        longitude: 77.2090,
        timezone: "Asia/Kolkata".to_string(),
    };

    let mut from_birth = input(instant, json!({}));
    from_birth.birth_data = Some(birth.clone());
    let r = result(from_birth).await;
    assert_eq!(r["vedic_day"]["location"]["source"], "birth_data");
    starts_with(&r["vedic_day"]["sunrise"], "2026-03-08T06:39");

    let mut from_location = input(instant, json!({}));
    from_location.birth_data = Some(birth.clone());
    from_location.location = Some(Coordinates {
        latitude: LAT,
        longitude: LON,
        altitude: None,
    });
    let r = result(from_location).await;
    assert_eq!(r["vedic_day"]["location"]["source"], "location");
    starts_with(&r["vedic_day"]["sunrise"], "2026-03-08T06:31");

    let mut from_options = located(instant);
    from_options.birth_data = Some(birth);
    let r = result(from_options).await;
    assert_eq!(r["vedic_day"]["location"]["source"], "options");
}

#[tokio::test]
async fn invalid_location_is_a_validation_error() {
    let instant = Utc.with_ymd_and_hms(2026, 3, 8, 6, 43, 16).unwrap();
    let engine = VedicClockEngine::new();

    for options in [
        json!({ "latitude": 95.0, "longitude": LON }),
        json!({ "latitude": LAT, "longitude": -181.0 }),
        json!({ "latitude": LAT }),
        json!({ "latitude": "north", "longitude": LON }),
    ] {
        let err = engine
            .calculate(input(instant, options.clone()))
            .await
            .expect_err(&options.to_string());
        assert!(
            matches!(err, EngineError::ValidationError(_)),
            "{options}: {err}"
        );
    }
}

#[tokio::test]
async fn located_cache_key_tracks_instant_and_place() {
    let engine = VedicClockEngine::new();
    let instant = Utc.with_ymd_and_hms(2026, 3, 8, 6, 43, 16).unwrap();
    let key = engine.cache_key(&located(instant));

    assert_eq!(key, engine.cache_key(&located(instant)));
    assert_ne!(
        key,
        engine.cache_key(&located(instant + Duration::seconds(1)))
    );
    assert_ne!(
        key,
        engine.cache_key(&input(
            instant,
            json!({ "timezone_offset": IST, "latitude": LAT, "longitude": 0.0 })
        ))
    );
}
