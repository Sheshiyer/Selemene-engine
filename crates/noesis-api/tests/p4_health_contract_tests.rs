//! Route-level regressions for the P4 dependency-health boundary.

mod common;

use axum::http::StatusCode;
use serial_test::serial;
use std::collections::BTreeSet;

fn force_unreachable_dependency_endpoints() {
    // Port 9 is deliberately unused in the test environment. This forces the
    // error/degraded branch without exposing or calling any real dependency.
    std::env::set_var("PYTHON_BIOFIELD_URL", "http://127.0.0.1:9");
    std::env::set_var("SELEMENE_FACE_CV_URL", "http://127.0.0.1:9");
    std::env::set_var("TS_ENGINES_URL", "http://127.0.0.1:9");
}

#[tokio::test]
#[serial]
async fn legacy_liveness_path_status_and_body_remain_unchanged() {
    force_unreachable_dependency_endpoints();
    let (status, body) = common::make_unauthenticated_request("GET", "/health/live", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(
        body.as_object()
            .expect("liveness body must be an object")
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "engines_loaded",
            "status",
            "uptime_seconds",
            "version",
            "workflows_loaded",
        ])
    );
}

#[tokio::test]
#[serial]
async fn dependency_health_route_is_allowlisted_and_non_generative() {
    force_unreachable_dependency_endpoints();
    let (status, body) =
        common::make_unauthenticated_request("GET", "/health/dependencies", None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body.as_object()
            .expect("dependency body must be an object")
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["engines", "providers", "sidecars", "status"])
    );
    assert_eq!(body["status"], "unavailable");

    for collection in ["engines", "sidecars", "providers"] {
        for entry in body[collection]
            .as_array()
            .expect("dependency collections must be arrays")
        {
            assert_eq!(
                entry
                    .as_object()
                    .expect("dependency entry must be an object")
                    .keys()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from(["id", "reason", "status"])
            );
        }
    }

    let engine_ids = body["engines"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry["id"].as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        engine_ids,
        BTreeSet::from(["biofield", "face-reading", "raaga", "sigil-forge"])
    );

    let sidecar_ids = body["sidecars"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|entry| entry["id"].as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        sidecar_ids,
        BTreeSet::from(["biofield-cv", "mediapipe-face-mesh"])
    );

    let encoded = body.to_string().to_ascii_lowercase();
    for forbidden in [
        "token", "api_key", "http://", "https://", "version", "stack", "trace",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "dependency health leaked forbidden field or value: {forbidden}"
        );
    }
}
