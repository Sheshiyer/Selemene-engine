//! Compatibility probes for vedic-clock replay (#1250) through the real
//! `/api/v1/engines/vedic-clock/calculate` route: the engine-specific fields
//! survive the API envelope, a replayed request returns the same result, and
//! invalid replay inputs fail with the canonical validation error.

use axum::http::StatusCode;
use serde_json::{json, Value};

mod common;

const ROUTE: &str = "/api/v1/engines/vedic-clock/calculate";

fn located_request() -> Value {
    json!({
        "current_time": "2026-03-08T06:43:16Z",
        "precision": "Standard",
        "options": {
            "timezone_offset": 330,
            "latitude": 12.9716,
            "longitude": 77.5946,
        },
    })
}

async fn post(body: Value) -> (StatusCode, Value) {
    let token = common::generate_test_token(5);
    common::make_authenticated_request("POST", ROUTE, &token, Some(body)).await
}

/// probe:compatibility — hora, choghadiya, the replay block and the
/// provenance detail reach API clients unchanged, and a replayed request
/// returns the same result and witness prompt.
#[tokio::test]
async fn vedic_clock_route_keeps_replay_fields_and_replays_identically() {
    let (status, first) = post(located_request()).await;
    assert_eq!(status, StatusCode::OK, "{first}");
    let (status, second) = post(located_request()).await;
    assert_eq!(status, StatusCode::OK, "{second}");

    let result = &first["result"];
    for key in [
        "current_organ",
        "current_dosha",
        "vedic_day",
        "hora",
        "choghadiya",
        "day_periods",
        "replay",
        "provenance_detail",
    ] {
        assert!(result.get(key).is_some(), "missing result.{key}: {result}");
    }
    assert_eq!(result["hora"]["number"], 6);
    assert_eq!(result["hora"]["ruler"], "Jupiter");
    assert_eq!(
        result["replay"]["inputs"]["current_time"],
        "2026-03-08T06:43:16Z"
    );
    // The route sets consciousness_level from the caller's token (level 5
    // here), and replay records the level the engine actually used.
    assert_eq!(result["replay"]["inputs"]["consciousness_level"], 5);
    assert_eq!(result["provenance_detail"]["fallback"]["used"], false);

    assert_eq!(first["result"], second["result"]);
    assert_eq!(first["witness_prompt"], second["witness_prompt"]);
}

/// probe:invalid-input — an unknown timezone is a 422 VALIDATION_ERROR, not
/// a result silently computed in UTC.
#[tokio::test]
async fn vedic_clock_route_rejects_unknown_timezone() {
    let body = json!({
        "birth_data": {
            "date": "1990-01-15",
            "time": "12:00",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "Mars/Olympus_Mons",
        },
        "current_time": "2026-03-08T06:43:16Z",
        "precision": "Standard",
        "options": {},
    });
    let (status, response) = post(body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{response}");
    assert_eq!(response["error_code"], "VALIDATION_ERROR", "{response}");
}
