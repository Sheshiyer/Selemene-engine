// Contract test for the new additive /assets/generate route.
// Verifies shape and that /witness/interpret response is untouched.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn assets_generate_is_additive_and_returns_expected_shape() {
    let router = common::get_router().await;

    // Minimal valid auth (jwt path in harness)
    let token = common::generate_test_token(3);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/assets/generate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "birth_data": {
                    "date": "1990-01-15",
                    "time": "14:30",
                    "latitude": 12.9716,
                    "longitude": 77.5946,
                    "timezone": "Asia/Kolkata",
                    "name": "Test"
                },
                "mode": "integrated-reading",
                "consciousness_level": 3
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = router.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Contract shape (additive only)
    assert!(json.get("mode").is_some());
    assert!(json.get("register").is_some());
    assert!(json.get("passes").is_some());
    assert!(json.get("assembled").is_some());
    assert!(json.get("engines_used").is_some());
    assert!(json.get("source_pack").is_some());

    // Requirement: passes.length > 0 (real engine context flowed)
    let passes = json["passes"].as_array().expect("passes must be array");
    assert!(passes.len() > 0, "passes must not be empty for valid mode");

    // Requirement: assembled present and non-trivial (pipeline-like assembly)
    let assembled = json["assembled"].as_str().expect("assembled must be string");
    assert!(!assembled.trim().is_empty(), "assembled must contain content");
    // Pipeline-style assembly: contains pass titles and engine seed references
    // (current wiring uses lowercase engine ids from orchestrator seeds)
    assert!(
        assembled.contains("Structural") || assembled.contains("Somatic") || assembled.contains("panchanga") || assembled.contains("numerology"),
        "assembled should contain pipeline pass titles or engine seeds"
    );

    // Requirement: source_pack looks like it came from factory/audit
    let sp = &json["source_pack"];
    assert!(sp.get("person_id").is_some());
    assert!(sp.get("mode").is_some());
    assert!(sp.get("register").is_some());
    assert!(sp.get("engines").is_some());
    assert!(sp.get("quality").is_some());
    let quality = &sp["quality"];
    assert!(quality.get("facts_count").is_some());
    assert!(quality.get("gate_status").is_some());

    // Requirement: register band correct for consciousness level 3 (<=3 -> l1_l3)
    assert_eq!(json["register"].as_str().unwrap(), "l1_l3");
}

#[tokio::test]
async fn assets_generate_register_band_l4_l5_for_high_consciousness() {
    let router = common::get_router().await;
    let token = common::generate_test_token(5); // enterprise level

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/assets/generate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "birth_data": {
                    "date": "1990-01-15",
                    "time": "14:30",
                    "latitude": 12.9716,
                    "longitude": 77.5946,
                    "timezone": "Asia/Kolkata",
                    "name": "HighLevel"
                },
                "mode": "integrated-reading",
                "consciousness_level": 5
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = router.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // High consciousness must map to l4_l5 register band
    assert_eq!(json["register"].as_str().unwrap(), "l4_l5");
    let passes = json["passes"].as_array().unwrap();
    assert!(passes.len() > 0);
}

#[tokio::test]
async fn assets_generate_supports_integrated_kundali_l0_mode() {
    let router = common::get_router().await;
    let token = common::generate_test_token(4);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/assets/generate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "birth_data": {
                    "date": "1990-01-15",
                    "time": "14:30",
                    "latitude": 12.9716,
                    "longitude": 77.5946,
                    "timezone": "Asia/Kolkata",
                    "name": "KundaliMode"
                },
                "mode": "integrated-kundali-l0",
                "consciousness_level": 4
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = router.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["mode"].as_str().unwrap(), "integrated-kundali-l0");
    assert_eq!(json["register"].as_str().unwrap(), "l4_l5");
    let passes = json["passes"].as_array().unwrap();
    assert_eq!(passes.len(), 12);
    assert_eq!(passes[0]["id"].as_str().unwrap(), "opening");
    assert_eq!(passes[11]["id"].as_str().unwrap(), "final-synthesis");

    let assembled = json["assembled"].as_str().unwrap();
    assert!(assembled.contains("Do not guarantee financial outcomes"));
    assert!(assembled.contains("Do not predict marriage inevitability"));
    assert!(assembled.contains("Do not diagnose"));
    assert!(assembled.contains("Do not predict childbirth"));

    let sp = &json["source_pack"];
    let sections = sp["quality"]["sections"]
        .as_array()
        .expect("sections rubric matrix");
    assert_eq!(sections.len(), 12);
    for section in sections {
        assert!(section.get("target_words").is_some());
        assert!(section.get("actual_words").is_some());
        assert!(section.get("model_used").is_some());
        assert!(section.get("latency_ms").is_some());
    }
}

#[tokio::test]
async fn witness_interpret_contract_unchanged() {
    let router = common::get_router().await;
    let token = common::generate_test_token(2);

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/witness/interpret")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::to_vec(&json!({
                "live_scores": {
                    "energy": 0.65,
                    "coherence": 0.70,
                    "symmetry": 0.60,
                    "complexity": 0.55,
                    "regulation": 0.58,
                    "color_balance": 0.62
                },
                "consciousness_level": 2
            }))
            .unwrap(),
        ))
        .unwrap();

    let response = router.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Existing frozen contract — EXACT shape, byte-for-byte compatible.
    // Must have exactly these 6 fields, no more, no less.
    let obj = json.as_object().expect("response must be object");
    let keys: Vec<&String> = obj.keys().collect();
    assert_eq!(keys.len(), 6, "WitnessInterpretResponse must have exactly 6 fields, got: {:?}", keys);

    // Explicit field presence (frozen public contract)
    assert!(json.get("aletheios").is_some());
    assert!(json.get("pichet").is_some());
    assert!(json.get("synthesis").is_some());
    assert!(json.get("witness_question").is_some());
    assert!(json.get("engines_used").is_some());
    assert!(json.get("llm_powered").is_some());

    // Type/shape assertions (must remain stable)
    assert!(json["aletheios"].is_string());
    assert!(json["pichet"].is_string());
    assert!(json["synthesis"].is_string());
    assert!(json["witness_question"].is_string());
    assert!(json["engines_used"].is_array());
    assert!(json["llm_powered"].is_boolean());

    // Regression: engines_used must be an array of strings
    for e in json["engines_used"].as_array().unwrap() {
        assert!(e.is_string(), "engines_used entries must be strings");
    }
}

#[tokio::test]
async fn assets_generate_accepts_report_level_and_subjects_rich_path() {
    let router = common::get_router().await;
    let token = common::generate_test_token(3);

    let req_body = json!({
        "mode": "integrated-reading",
        "consciousness_level": 3,
        "report_level": "L2",
        "subjects": [
            {
                "role": "primary",
                "name": "TestSubject",
                "birth_date": "1990-01-15",
                "birth_time": "14:30",
                "normalized_location": {
                    "display_name": "Bengaluru, India",
                    "latitude": 12.9716,
                    "longitude": 77.5946,
                    "timezone": "Asia/Kolkata",
                    "provider": "manual",
                    "confidence": "exact"
                }
            }
        ]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/assets/generate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // The handler should prefer report_level and subjects when provided (Task 6).
    // Assert they are reflected inside source_pack so callers see the contract was accepted.
    let sp = &json["source_pack"];
    assert_eq!(
        sp["report_level"].as_str().unwrap_or("MISSING"),
        "L2",
        "report_level should be captured from request into source_pack"
    );
    let subjects = sp.get("subjects").and_then(|v| v.as_array());
    assert!(subjects.is_some(), "subjects array should be present in source_pack");
    let subs = subjects.unwrap();
    assert_eq!(subs.len(), 1, "should have one subject");
    assert_eq!(subs[0]["name"].as_str().unwrap_or(""), "TestSubject");
    assert_eq!(subs[0]["role"].as_str().unwrap_or(""), "primary");

    // Task 8: small assertion that report_level + subject info (count + first normalized summary) flow into source_pack
    assert_eq!(
        sp.get("subject_count").and_then(|v| v.as_u64()).unwrap_or(0),
        1,
        "subject_count should be emitted in source_pack for rich contract"
    );
    let first_loc = sp.get("first_normalized_location");
    assert!(first_loc.is_some(), "first_normalized_location summary should be present");
    assert_eq!(
        first_loc.unwrap().get("display_name").and_then(|v| v.as_str()).unwrap_or("MISSING"),
        "Bengaluru, India",
        "first normalized location summary should reflect the subject's location"
    );
}

#[tokio::test]
async fn assets_generate_rejects_incomplete_subjects_missing_normalized_location() {
    let router = common::get_router().await;
    let token = common::generate_test_token(3);

    // Rich subjects form but one subject lacks normalized_location → should 422 (is_complete gate)
    let req_body = json!({
        "mode": "integrated-reading",
        "consciousness_level": 3,
        "report_level": "L1",
        "subjects": [
            {
                "role": "primary",
                "name": "IncompleteSubject",
                "birth_date": "1990-01-15",
                "birth_time": "14:30"
                // deliberately no normalized_location
            }
        ]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/assets/generate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(req).await.unwrap();
    let status = response.status();

    // Per plan: assert 422 or clear error when subject lacks normalized_location
    assert_eq!(
        status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "expected 422 for rich subjects missing normalized_location, got {}: {:?}",
        status,
        axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap()
    );

    // Body should be a structured error (ErrorResponse shape)
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        json.get("error_code").is_some() || json.get("message").is_some() || json.get("error").is_some(),
        "response should carry clear error signal, got: {}",
        json
    );
}

// Task 9: explicit L0 using rich subjects shape (no legacy birth_data)
#[tokio::test]
async fn assets_generate_accepts_l0_with_rich_subjects_shape() {
    let router = common::get_router().await;
    let token = common::generate_test_token(5); // l4_l5 for full L0

    let req_body = json!({
        "mode": "integrated-kundali-l0",
        "consciousness_level": 5,
        "report_level": "L0",
        "subjects": [
            {
                "role": "primary",
                "name": "L0Subject",
                "birth_date": "1990-01-15",
                "birth_time": "14:30",
                "normalized_location": {
                    "display_name": "Bengaluru, India",
                    "latitude": 12.9716,
                    "longitude": 77.5946,
                    "timezone": "Asia/Kolkata",
                    "provider": "manual",
                    "confidence": "exact"
                }
            }
        ]
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/assets/generate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Assert level echoed in response and source_pack (Task 9 requirement)
    assert_eq!(json["mode"].as_str().unwrap(), "integrated-kundali-l0");
    assert_eq!(json["register"].as_str().unwrap(), "l4_l5");

    let sp = &json["source_pack"];
    assert_eq!(
        sp["report_level"].as_str().unwrap_or("MISSING"),
        "L0",
        "report_level L0 should be captured from rich subjects request into source_pack"
    );

    // subjects reflected in source_pack
    let subs = sp["subjects"].as_array().expect("subjects array in source_pack for L0 rich");
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0]["role"].as_str().unwrap_or(""), "primary");
    assert_eq!(subs[0]["name"].as_str().unwrap_or(""), "L0Subject");

    // L0 contract: exactly 12 passes with opening/final-synthesis (same as legacy L0 test)
    let passes = json["passes"].as_array().unwrap();
    assert_eq!(passes.len(), 12, "L0 via rich subjects must still yield 12 passes");
    assert_eq!(passes[0]["id"].as_str().unwrap(), "opening");
    assert_eq!(passes[11]["id"].as_str().unwrap(), "final-synthesis");
}
