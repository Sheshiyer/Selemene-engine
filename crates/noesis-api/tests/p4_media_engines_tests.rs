//! Route-level contract tests for the four P4 media-engine API surfaces.

mod common;

use axum::http::StatusCode;
use common::test_harness::RoutingHarness;
use noesis_orchestrator::WorkflowOrchestrator;
use serde_json::{json, Value};
use std::collections::BTreeSet;

const FOCUS_ENGINES: [&str; 4] = ["biofield", "face-reading", "raaga", "sigil-forge"];

fn fixture(name: &str) -> Value {
    let raw = match name {
        "biofield" => include_str!("fixtures/p4/biofield-request.json"),
        "face-reading" => include_str!("fixtures/p4/face-reading-request.json"),
        "raaga" => include_str!("fixtures/p4/raaga-request.json"),
        "sigil-forge" => include_str!("fixtures/p4/sigil-forge-request.json"),
        "legacy" => include_str!("fixtures/p4/legacy-options-request.json"),
        _ => panic!("unknown fixture: {name}"),
    };
    serde_json::from_str(raw).expect("P4 fixture must contain valid JSON")
}

#[tokio::test]
async fn sdk_fixtures_cross_http_auth_normalization_and_orchestrator_dispatch() {
    let harness = RoutingHarness::with_probe_engines(&FOCUS_ENGINES).await;

    for engine_id in FOCUS_ENGINES {
        let (status, response) = harness
            .send_authenticated_json(
                "POST",
                &format!("/api/v1/engines/{engine_id}/calculate"),
                Some(fixture(engine_id)),
            )
            .await;

        assert_eq!(status, StatusCode::OK, "{engine_id}: {response}");
        assert_eq!(response["engine_id"], engine_id);
        assert_eq!(
            response["result"]["route_marker"],
            format!("probe::{engine_id}")
        );
        assert_eq!(harness.log.count(engine_id), 1);

        let input = harness
            .log
            .last_input(engine_id)
            .expect("probe must receive normalized input");
        assert_eq!(input.options["consciousness_level"], 5);
        assert_eq!(
            input.options["consent"]["granted"], true,
            "{engine_id} consent must survive the HTTP boundary"
        );
    }

    assert!(harness.log.last_input("biofield").unwrap().options["image_data"].is_object());
    assert!(harness.log.last_input("face-reading").unwrap().options["image_data"].is_object());
    assert!(harness.log.last_input("raaga").unwrap().options["audio_ref"].is_object());
    assert_eq!(
        harness.log.last_input("sigil-forge").unwrap().options["generate_image"],
        true
    );
}

#[tokio::test]
async fn generated_media_is_available_at_sdk_and_legacy_paths() {
    let harness = RoutingHarness::with_probe_engines(&["raaga", "sigil-forge"]).await;

    for (engine_id, field) in [
        ("raaga", "generated_audio"),
        ("sigil-forge", "generated_image"),
    ] {
        let (status, response) = harness
            .send_authenticated_json(
                "POST",
                &format!("/api/v1/engines/{engine_id}/calculate"),
                Some(fixture(engine_id)),
            )
            .await;

        assert_eq!(status, StatusCode::OK, "{engine_id}: {response}");
        assert_eq!(response[field], response["result"][field]);
        assert!(response[field].is_object());
    }
}

#[tokio::test]
async fn legacy_options_payload_crosses_the_same_http_route() {
    let harness = RoutingHarness::with_probe_engines(&["raaga", "biofield"]).await;

    let (status, response) = harness
        .send_authenticated_json(
            "POST",
            "/api/v1/engines/raaga/calculate",
            Some(fixture("legacy")),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{response}");
    assert_eq!(
        harness.log.last_input("raaga").unwrap().options["melakarta"],
        1
    );

    let legacy_biofield = json!({
        "options": {
            "image_data": {"b64": "AAECf4D/", "mime_type": "image/png"},
            "consent": {
                "granted": true,
                "scopes": ["biofield-capture"],
                "timestamp": "2026-07-18T00:00:00Z"
            }
        }
    });
    let (status, response) = harness
        .send_authenticated_json(
            "POST",
            "/api/v1/engines/biofield/calculate",
            Some(legacy_biofield),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{response}");
    assert_eq!(
        harness.log.last_input("biofield").unwrap().options["image_data"]["mime_type"],
        "image/png"
    );
}

#[tokio::test]
async fn missing_or_wrong_consent_fails_before_engine_dispatch() {
    let harness = RoutingHarness::with_probe_engines(&FOCUS_ENGINES).await;

    for engine_id in FOCUS_ENGINES {
        let mut wrong = fixture(engine_id);
        wrong["consent"]["scopes"] = json!(["wrong-scope"]);
        let (status, response) = harness
            .send_authenticated_json(
                "POST",
                &format!("/api/v1/engines/{engine_id}/calculate"),
                Some(wrong),
            )
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response["error_code"], "CONSENT_REQUIRED");

        let mut missing = fixture(engine_id);
        missing.as_object_mut().unwrap().remove("consent");
        if let Some(media) = missing.get_mut("image_data").and_then(Value::as_object_mut) {
            media.remove("consent");
        }
        if let Some(media) = missing.get_mut("audio_ref").and_then(Value::as_object_mut) {
            media.remove("consent");
        }
        let (status, response) = harness
            .send_authenticated_json(
                "POST",
                &format!("/api/v1/engines/{engine_id}/calculate"),
                Some(missing),
            )
            .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response["error_code"], "CONSENT_REQUIRED");
        assert_eq!(harness.log.count(engine_id), 0);
    }
}

#[tokio::test]
async fn unknown_engine_is_rejected_and_runtime_boundaries_are_explicit() {
    let harness = RoutingHarness::with_probe_engines(&FOCUS_ENGINES).await;
    let (status, response) = harness
        .send_authenticated_json(
            "POST",
            "/api/v1/engines/not-a-real-engine/calculate",
            Some(json!({"parameters": {}})),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{response}");

    assert_eq!(
        noesis_bridge::P4_TS_FOCUS_ENGINE_IDS,
        ["raaga", "sigil-forge"]
    );
    let mut native_runtime = WorkflowOrchestrator::new();
    native_runtime.register_native_runtime_engines();
    let native_ids = native_runtime
        .list_engines()
        .into_iter()
        .collect::<BTreeSet<_>>();
    assert!(native_ids.contains("biofield"));
    assert!(native_ids.contains("face-reading"));
    assert!(!native_ids.contains("raaga"));
    assert!(!native_ids.contains("sigil-forge"));
}
