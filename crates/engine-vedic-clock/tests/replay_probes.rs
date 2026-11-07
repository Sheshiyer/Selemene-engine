//! Named replay probes for #1250: the result and witness prompt are bound to
//! timezone, sunrise/sunset, location and temporal-boundary inputs.
//!
//! Synthetic location and instants only.

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use engine_vedic_clock::{
    sun_event, ConsciousnessEngine, EngineError, EngineInput, EngineOutput, SunEvent,
    VedicClockEngine,
};
use noesis_core::{BirthData, Precision};
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

fn with_birth_timezone(current_time: DateTime<Utc>, timezone: &str) -> EngineInput {
    let mut request = input(current_time, json!({}));
    request.birth_data = Some(BirthData {
        name: None,
        date: "1990-01-15".to_string(),
        time: Some("12:00".to_string()),
        latitude: 40.7128,
        longitude: -74.006,
        timezone: timezone.to_string(),
    });
    request
}

async fn calculate(request: EngineInput) -> Result<EngineOutput, EngineError> {
    VedicClockEngine::new().calculate(request).await
}

fn instant() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 3, 8, 6, 43, 16).unwrap()
}

/// probe:happy-path — the same request twice gives the same result and
/// witness prompt, and the result echoes every input it depends on.
#[tokio::test]
async fn probe_happy_path_replays_identically() {
    for level in 0..=6 {
        let mut request = located(instant());
        request
            .options
            .insert("consciousness_level".into(), json!(level));
        request.options.insert("activity".into(), json!("work"));

        let first = calculate(request.clone()).await.unwrap();
        let second = calculate(request).await.unwrap();
        assert_eq!(first.result, second.result, "level {level}: result");
        assert_eq!(
            first.witness_prompt, second.witness_prompt,
            "level {level}: witness_prompt"
        );
    }

    let output = calculate(located(instant())).await.unwrap();
    let replay = &output.result["replay"];
    assert_eq!(replay["deterministic"], true);
    assert!(replay["fingerprint"].as_str().unwrap().len() == 16);
    let inputs = &replay["inputs"];
    assert_eq!(inputs["current_time"], "2026-03-08T06:43:16Z");
    assert_eq!(inputs["timezone"]["offset_minutes"], IST);
    assert_eq!(inputs["timezone"]["source"], "options.timezone_offset");
    assert_eq!(inputs["location"]["latitude"], LAT);
    assert_eq!(inputs["location"]["source"], "options");
    assert_eq!(inputs["consciousness_level"], 2);

    let detail = &output.result["provenance_detail"];
    assert_eq!(detail["current_organ"]["origin"], "table_lookup");
    assert_eq!(detail["sunrise_sunset"]["origin"], "calculated");
    assert_eq!(detail["hora"]["origin"], "calculated");
    assert_eq!(detail["choghadiya"]["origin"], "calculated");
    assert_eq!(detail["recommendation"]["origin"], "derived_interpretation");
    assert_eq!(detail["witness_prompt"]["origin"], "templated_narrative");
    assert_eq!(detail["panchanga_quality"]["origin"], "not_supplied");
    assert_eq!(detail["activity_timing"]["origin"], "not_requested");
    assert_eq!(detail["fallback"]["used"], false);
    assert_eq!(detail["mock"]["used"], false);
    assert_eq!(detail["generated_artifacts"]["status"], "not_applicable");
}

/// probe:boundary — one second either side of sunrise changes the hora and
/// the replay fingerprint, and the cache keys differ, so a cached result
/// from one side is never served for the other.
#[tokio::test]
async fn probe_boundary_sunrise_is_bound_into_replay_and_cache_key() {
    let date = NaiveDate::from_ymd_opt(2026, 3, 8).unwrap();
    let sunrise = sun_event(date, LAT, LON, IST, SunEvent::Rise).unwrap();
    let engine = VedicClockEngine::new();

    let before = calculate(located(sunrise - Duration::seconds(1)))
        .await
        .unwrap();
    let after = calculate(located(sunrise + Duration::seconds(1)))
        .await
        .unwrap();
    assert_eq!(before.result["hora"]["number"], 24);
    assert_eq!(after.result["hora"]["number"], 1);
    assert_ne!(
        before.result["replay"]["fingerprint"],
        after.result["replay"]["fingerprint"]
    );
    assert_ne!(
        engine.cache_key(&located(sunrise - Duration::seconds(1))),
        engine.cache_key(&located(sunrise + Duration::seconds(1)))
    );
}

/// probe:boundary — organ windows change on odd local hours. Requests
/// either side of 01:00 must not share a cache entry.
#[tokio::test]
async fn probe_boundary_organ_window_change_is_not_cached_across() {
    let engine = VedicClockEngine::new();
    let at = |h, m, s| {
        input(
            Utc.with_ymd_and_hms(2026, 3, 8, h, m, s).unwrap(),
            json!({}),
        )
    };

    let before = calculate(at(0, 59, 59)).await.unwrap();
    let after = calculate(at(1, 0, 0)).await.unwrap();
    assert_ne!(
        before.result["current_organ"]["organ"],
        after.result["current_organ"]["organ"]
    );
    assert_ne!(
        engine.cache_key(&at(0, 59, 59)),
        engine.cache_key(&at(1, 0, 0))
    );
}

/// probe:boundary — the witness prompt depends on consciousness_level, so
/// the level is part of the cache key.
#[test]
fn probe_boundary_consciousness_level_is_in_the_cache_key() {
    let engine = VedicClockEngine::new();
    let level = |l: u8| input(instant(), json!({ "consciousness_level": l }));
    assert_ne!(engine.cache_key(&level(1)), engine.cache_key(&level(5)));
    assert_eq!(engine.cache_key(&level(5)), engine.cache_key(&level(5)));
}

/// probe:boundary — an IANA birth timezone resolves with daylight saving at
/// the requested instant (New York: UTC−5 in January, UTC−4 in July).
#[tokio::test]
async fn probe_boundary_iana_timezone_follows_daylight_saving() {
    let winter = Utc.with_ymd_and_hms(2026, 1, 15, 17, 0, 0).unwrap();
    let summer = Utc.with_ymd_and_hms(2026, 7, 15, 17, 0, 0).unwrap();
    for (instant, offset, local_hour) in [(winter, -300, 12), (summer, -240, 13)] {
        let output = calculate(with_birth_timezone(instant, "America/New_York"))
            .await
            .unwrap();
        let timezone = &output.result["timezone"];
        assert_eq!(timezone["offset_minutes"], offset, "{instant}");
        assert_eq!(timezone["source"], "birth_data.timezone");
        assert_eq!(timezone["local_hour"], local_hour);
        assert_eq!(
            output.result["replay"]["inputs"]["timezone"]["zone"],
            "America/New_York"
        );
    }
}

/// probe:unavailable-dependency — the only input hora, choghadiya and the
/// day periods depend on is a location; without one they are omitted with
/// a reason, and the organ clock and dosha are still computed.
#[tokio::test]
async fn probe_degraded_missing_location_keeps_organ_clock() {
    let output = calculate(input(instant(), json!({ "timezone_offset": IST })))
        .await
        .unwrap();
    let result = &output.result;
    assert_eq!(result["current_organ"]["organ"], "Heart");
    assert!(result["current_dosha"]["dosha"].is_string());
    assert!(result.get("hora").is_none());
    assert!(result.get("choghadiya").is_none());
    assert_eq!(
        result["vedic_day"]["not_computed_reason"],
        "location_missing"
    );

    let detail = &result["provenance_detail"];
    for part in ["sunrise_sunset", "hora", "choghadiya", "day_periods"] {
        assert_eq!(detail[part]["origin"], "not_computed", "{part}");
        assert_eq!(detail[part]["reason"], "location_missing", "{part}");
    }
    assert_eq!(detail["current_organ"]["origin"], "table_lookup");
    assert_eq!(detail["fallback"]["used"], false);
    assert!(result["replay"]["inputs"]["location"].is_null());
}

/// probe:invalid-input — malformed or out-of-range inputs fail with
/// `ValidationError` instead of silently becoming UTC or another index.
#[tokio::test]
async fn probe_invalid_input_is_a_validation_error() {
    let cases: Vec<(&str, EngineInput)> = vec![
        (
            "unknown IANA zone",
            with_birth_timezone(instant(), "Mars/Olympus_Mons"),
        ),
        (
            "malformed offset string",
            with_birth_timezone(instant(), "+5:3"),
        ),
        (
            "timezone_offset as a string",
            input(instant(), json!({ "timezone_offset": "330" })),
        ),
        (
            "timezone_offset out of range",
            input(instant(), json!({ "timezone_offset": 2000 })),
        ),
        (
            "tithi_index out of range",
            input(instant(), json!({ "tithi_index": 30 })),
        ),
        (
            "nakshatra_index out of range",
            input(instant(), json!({ "nakshatra_index": 27 })),
        ),
        (
            "tithi_index not an integer",
            input(instant(), json!({ "tithi_index": "Panchami" })),
        ),
    ];
    for (label, request) in cases {
        let err = calculate(request).await.expect_err(label);
        assert!(
            matches!(err, EngineError::ValidationError(_)),
            "{label}: {err}"
        );
    }
}

/// Krishna-paksha tithi indices (15–29), as the panchanga engine emits them,
/// are accepted and recorded as caller-supplied.
#[tokio::test]
async fn caller_supplied_panchanga_indices_are_labelled() {
    let output = calculate(input(
        instant(),
        json!({ "tithi_index": 25, "nakshatra_index": 14 }),
    ))
    .await
    .unwrap();
    assert!(output.result["recommendation"]["panchanga_quality"].is_string());
    let detail = &output.result["provenance_detail"]["panchanga_quality"];
    assert_eq!(detail["origin"], "table_lookup");
    assert_eq!(detail["inputs"], "caller_supplied");
    assert_eq!(output.result["replay"]["inputs"]["tithi_index"], 25);
}

/// probe:serialization — `EngineOutput` survives a JSON round trip with the
/// engine-specific fields intact, as the API, SDK and persistence layers
/// carry it.
#[tokio::test]
async fn probe_serialization_round_trip_keeps_engine_fields() {
    let output = calculate(located(instant())).await.unwrap();
    let wire = serde_json::to_string(&output).unwrap();
    let back: EngineOutput = serde_json::from_str(&wire).unwrap();
    assert_eq!(back.result, output.result);
    assert_eq!(back.witness_prompt, output.witness_prompt);
    for key in [
        "hora",
        "choghadiya",
        "vedic_day",
        "day_periods",
        "current_organ",
        "replay",
        "provenance_detail",
    ] {
        assert!(back.result.get(key).is_some(), "{key}");
    }
}
