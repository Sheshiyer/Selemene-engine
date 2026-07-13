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
    assert!(
        !passes.is_empty(),
        "passes must not be empty for valid mode"
    );

    // Requirement: assembled present and non-trivial (pipeline-like assembly)
    let assembled = json["assembled"]
        .as_str()
        .expect("assembled must be string");
    assert!(
        !assembled.trim().is_empty(),
        "assembled must contain content"
    );
    // Pipeline-style assembly: contains pass titles and engine seed references
    // (current wiring uses lowercase engine ids from orchestrator seeds)
    assert!(
        assembled.contains("Structural")
            || assembled.contains("Somatic")
            || assembled.contains("panchanga")
            || assembled.contains("numerology"),
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // High consciousness must map to l4_l5 register band
    assert_eq!(json["register"].as_str().unwrap(), "l4_l5");
    let passes = json["passes"].as_array().unwrap();
    assert!(!passes.is_empty());
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
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
    assert_eq!(
        keys.len(),
        6,
        "WitnessInterpretResponse must have exactly 6 fields, got: {:?}",
        keys
    );

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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
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
    assert!(
        subjects.is_some(),
        "subjects array should be present in source_pack"
    );
    let subs = subjects.unwrap();
    assert_eq!(subs.len(), 1, "should have one subject");
    assert_eq!(subs[0]["name"].as_str().unwrap_or(""), "TestSubject");
    assert_eq!(subs[0]["role"].as_str().unwrap_or(""), "primary");

    // Task 8: small assertion that report_level + subject info (count + first normalized summary) flow into source_pack
    assert_eq!(
        sp.get("subject_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        1,
        "subject_count should be emitted in source_pack for rich contract"
    );
    let first_loc = sp.get("first_normalized_location");
    assert!(
        first_loc.is_some(),
        "first_normalized_location summary should be present"
    );
    assert_eq!(
        first_loc
            .unwrap()
            .get("display_name")
            .and_then(|v| v.as_str())
            .unwrap_or("MISSING"),
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
        axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
    );

    // Body should be a structured error (ErrorResponse shape)
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert!(
        json.get("error_code").is_some()
            || json.get("message").is_some()
            || json.get("error").is_some(),
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
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
    let subs = sp["subjects"]
        .as_array()
        .expect("subjects array in source_pack for L0 rich");
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0]["role"].as_str().unwrap_or(""), "primary");
    assert_eq!(subs[0]["name"].as_str().unwrap_or(""), "L0Subject");

    // L0 contract: exactly 12 passes with opening/final-synthesis (same as legacy L0 test)
    let passes = json["passes"].as_array().unwrap();
    assert_eq!(
        passes.len(),
        12,
        "L0 via rich subjects must still yield 12 passes"
    );
    assert_eq!(passes[0]["id"].as_str().unwrap(), "opening");
    assert_eq!(passes[11]["id"].as_str().unwrap(), "final-synthesis");
}

// Akshay (humdes-extracted) solo L0 end-to-end smoke.
// Uses the auto-extracted fields from tests/fixtures/humdes/readings/personal/...Akshay...
#[tokio::test]
async fn assets_generate_akshay_humdes_solo_l0() {
    let router = common::get_router().await;
    let token = common::generate_test_token(5); // l4_l5 for full L0

    let req_body = json!({
        "mode": "integrated-kundali-l0",
        "consciousness_level": 5,
        "report_level": "L0",
        "subjects": [
            {
                "role": "primary",
                "name": "Akshay",
                "gender": "male",
                "sex_for_external_chart_source": "M",
                "birth_date": "1990-10-05",
                "birth_time": "13:13:00",
                "birth_time_confidence": "exact",
                "birth_location_query": "Bengaluru, India",
                "normalized_location": {
                    "display_name": "Bengaluru, Bangalore North, Bengaluru Urban, Karnataka, India",
                    "latitude": 12.9767936,
                    "longitude": 77.590082,
                    "timezone": "Asia/Kolkata",
                    "provider": "nominatim",
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
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Akshay humdes solo L0 must succeed"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["mode"].as_str().unwrap(), "integrated-kundali-l0");
    assert_eq!(json["register"].as_str().unwrap(), "l4_l5");

    let passes = json["passes"].as_array().unwrap();
    assert_eq!(passes.len(), 12, "Akshay L0 must yield 12 passes");
    assert_eq!(passes[0]["id"].as_str().unwrap(), "opening");
    assert_eq!(passes[11]["id"].as_str().unwrap(), "final-synthesis");

    let assembled = json["assembled"].as_str().unwrap();
    assert!(
        !assembled.trim().is_empty(),
        "assembled report must be non-empty"
    );

    let engines = json["engines_used"]
        .as_array()
        .expect("engines_used must be array");
    assert!(
        !engines.is_empty(),
        "engines_used must be populated for Akshay solo L0"
    );

    let sp = &json["source_pack"];
    assert_eq!(sp["report_level"].as_str().unwrap_or("MISSING"), "L0");
    assert_eq!(sp["subject_count"].as_u64().unwrap_or(0), 1);

    let subs = sp["subjects"]
        .as_array()
        .expect("subjects array in source_pack");
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0]["name"].as_str().unwrap_or(""), "Akshay");
    assert_eq!(subs[0]["role"].as_str().unwrap_or(""), "primary");
    // Note: source_pack currently echoes only role/name/birth fields + normalized_location,
    // so sex_for_external_chart_source is accepted by intake but not mirrored here.

    let first_loc = sp["first_normalized_location"]
        .as_object()
        .expect("first_normalized_location");
    assert_eq!(
        first_loc["display_name"].as_str().unwrap_or(""),
        "Bengaluru, Bangalore North, Bengaluru Urban, Karnataka, India"
    );
    assert_eq!(first_loc["latitude"].as_f64().unwrap(), 12.9767936);
    assert_eq!(first_loc["longitude"].as_f64().unwrap(), 77.590082);

    let quality = sp["quality"].as_object().expect("quality object");
    assert_eq!(quality["gate_status"].as_str().unwrap_or(""), "ready");
    let sections = quality["sections"]
        .as_array()
        .expect("sections rubric matrix");
    assert_eq!(
        sections.len(),
        12,
        "source pack rubric matrix must cover all 12 passes"
    );
}

// Same Akshay L0 run, but persists the full API response JSONs to disk
// so we have a minimum artifact store for every reading.
#[tokio::test]
async fn assets_generate_akshay_humdes_solo_l0_persist() {
    use std::fs;
    use std::path::PathBuf;

    let router = common::get_router().await;
    let token = common::generate_test_token(5);

    let req_body = json!({
        "mode": "integrated-kundali-l0",
        "consciousness_level": 5,
        "report_level": "L0",
        "subjects": [
            {
                "role": "primary",
                "name": "Akshay",
                "gender": "male",
                "sex_for_external_chart_source": "M",
                "birth_date": "1990-10-05",
                "birth_time": "13:13:00",
                "birth_time_confidence": "exact",
                "birth_location_query": "Bengaluru, India",
                "normalized_location": {
                    "display_name": "Bengaluru, Bangalore North, Bengaluru Urban, Karnataka, India",
                    "latitude": 12.9767936,
                    "longitude": 77.590082,
                    "timezone": "Asia/Kolkata",
                    "provider": "nominatim",
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Persist minimum JSON artifacts.
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(".local/akshay-l0-api-run");
    fs::create_dir_all(&out_dir).expect("create output dir");

    let request_path = out_dir.join("request.json");
    let response_path = out_dir.join("response.json");
    let source_pack_path = out_dir.join("source-pack.json");

    fs::write(
        &request_path,
        serde_json::to_string_pretty(&req_body).unwrap(),
    )
    .expect("write request.json");
    fs::write(&response_path, serde_json::to_string_pretty(&json).unwrap())
        .expect("write response.json");
    if let Some(sp) = json.get("source_pack") {
        fs::write(&source_pack_path, serde_json::to_string_pretty(sp).unwrap())
            .expect("write source-pack.json");
    }

    println!("Persisted Akshay L0 API run:");
    println!(
        "  request.json   -> {}",
        request_path
            .canonicalize()
            .unwrap_or(request_path)
            .display()
    );
    println!(
        "  response.json  -> {}",
        response_path
            .canonicalize()
            .unwrap_or_else(|_| response_path.clone())
            .display()
    );
    println!(
        "  source-pack.json -> {}",
        source_pack_path
            .canonicalize()
            .unwrap_or_else(|_| source_pack_path.clone())
            .display()
    );

    assert!(response_path.exists());
    assert!(source_pack_path.exists());
}

// Task 9 + relationship_context parity: two-subject rich shape with family context.
// Asserts relationship_context flows into source_pack and subjects carry roles (and optionally relationship_label).
#[tokio::test]
async fn assets_generate_accepts_two_subjects_synastry_rich_shape() {
    let router = common::get_router().await;
    let token = common::generate_test_token(4);

    let req_body = json!({
        "mode": "composite-dyad",
        "consciousness_level": 4,
        "report_level": "L3",
        "subjects": [
            {
                "role": "mother",
                "name": "MotherPerson",
                "birth_date": "1965-03-12",
                "birth_time": "08:00",
                "normalized_location": {
                    "display_name": "Bengaluru, India",
                    "latitude": 12.9716,
                    "longitude": 77.5946,
                    "timezone": "Asia/Kolkata",
                    "provider": "manual",
                    "confidence": "exact"
                },
                "relationship_label": "mother"
            },
            {
                "role": "son",
                "name": "SonPerson",
                "birth_date": "1992-06-20",
                "birth_time": "09:15",
                "normalized_location": {
                    "display_name": "Mumbai, India",
                    "latitude": 19.0760,
                    "longitude": 72.8777,
                    "timezone": "Asia/Kolkata",
                    "provider": "manual",
                    "confidence": "exact"
                },
                "relationship_label": "son"
            }
        ],
        "relationship_context": {
            "type": "family",
            "mapping_goal": "mother-son lineage and dharma",
            "sensitivity_level": "high"
        }
    });

    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/assets/generate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&req_body).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(req).await.unwrap();
    // Task 9 requirement: no crash (must be 200)
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "two-subject synastry rich shape must not crash"
    );

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // engines_used must be populated (non-empty array of strings)
    let engines = json["engines_used"]
        .as_array()
        .expect("engines_used must be array");
    assert!(
        !engines.is_empty(),
        "engines_used must be populated for two-subject synastry request"
    );
    for e in engines {
        assert!(e.is_string(), "engines_used entries must be strings");
    }

    // report_level and both subjects must flow to source_pack
    let sp = &json["source_pack"];
    assert_eq!(
        sp["report_level"].as_str().unwrap_or("MISSING"),
        "L3",
        "report_level should be captured for synastry rich request"
    );
    let subs = sp["subjects"]
        .as_array()
        .expect("subjects array in source_pack for synastry");
    assert_eq!(subs.len(), 2, "synastry request must carry two subjects");
    assert_eq!(subs[0]["role"].as_str().unwrap_or(""), "mother");
    assert_eq!(subs[1]["role"].as_str().unwrap_or(""), "son");
    assert_eq!(
        subs[0]["relationship_label"].as_str().unwrap_or(""),
        "mother"
    );
    assert_eq!(subs[1]["relationship_label"].as_str().unwrap_or(""), "son");

    // subject_count should reflect 2
    assert_eq!(
        sp.get("subject_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        2,
        "subject_count should be 2 for synastry"
    );

    // Core requirement: relationship_context from rich request must appear in source_pack
    let rc = sp
        .get("relationship_context")
        .expect("relationship_context must be present in source_pack for rich request");
    assert_eq!(
        rc.get("type").and_then(|v| v.as_str()).unwrap_or("MISSING"),
        "family",
        "relationship_context.type must be echoed from request"
    );
    assert!(
        rc.get("mapping_goal").is_some(),
        "relationship_context.mapping_goal should be present"
    );

    // Mother-son style assertion (relationship_label + family framing)
    assert_eq!(
        subs[0]["relationship_label"].as_str().unwrap_or(""),
        "mother"
    );
    assert_eq!(subs[1]["relationship_label"].as_str().unwrap_or(""), "son");
    assert_eq!(
        rc.get("type").and_then(|v| v.as_str()).unwrap_or(""),
        "family"
    );
}

// TDD: failing test first — expects language + relationship_context to round-trip into source_pack.
// This will fail until language is added to AssetGenerateRequest and wired to build_source_pack_with_audit.
#[tokio::test]
async fn assets_generate_roundtrips_language_with_relationship_context() {
    let router = common::get_router().await;
    let token = common::generate_test_token(3);

    let req_body = json!({
        "mode": "composite-dyad",
        "consciousness_level": 3,
        "report_level": "L2",
        "subjects": [
            {
                "role": "mother",
                "name": "Mother",
                "birth_date": "1965-03-12",
                "birth_time": "08:00",
                "normalized_location": {
                    "display_name": "Bengaluru, India",
                    "latitude": 12.9716,
                    "longitude": 77.5946,
                    "timezone": "Asia/Kolkata",
                    "provider": "manual",
                    "confidence": "exact"
                }
            },
            {
                "role": "son",
                "name": "Son",
                "birth_date": "1992-06-20",
                "birth_time": "09:15",
                "normalized_location": {
                    "display_name": "Mumbai, India",
                    "latitude": 19.0760,
                    "longitude": 72.8777,
                    "timezone": "Asia/Kolkata",
                    "provider": "manual",
                    "confidence": "exact"
                }
            }
        ],
        "relationship_context": {
            "type": "family",
            "mapping_goal": "mother-son lineage",
            "sensitivity_level": "high"
        },
        "language": "hi"
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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let sp = &json["source_pack"];

    // language must round-trip into source_pack (orchestrator/prompt metadata for TS side)
    assert_eq!(
        sp.get("language").and_then(|v| v.as_str()),
        Some("hi"),
        "language must be echoed in source_pack for orchestrator/prompt use"
    );

    // relationship_context present and not mutated
    let rc = sp
        .get("relationship_context")
        .expect("relationship_context must be present in source_pack");
    assert_eq!(
        rc.get("type").and_then(|v| v.as_str()),
        Some("family"),
        "relationship_context.type must be preserved"
    );
    assert_eq!(
        rc.get("mapping_goal").and_then(|v| v.as_str()),
        Some("mother-son lineage"),
        "relationship_context.mapping_goal must be preserved"
    );
    assert_eq!(
        rc.get("sensitivity_level").and_then(|v| v.as_str()),
        Some("high"),
        "relationship_context.sensitivity_level must be preserved"
    );

    // subjects also present (not mutated)
    let subs = sp
        .get("subjects")
        .and_then(|v| v.as_array())
        .expect("subjects array in source_pack");
    assert_eq!(subs.len(), 2);
    assert_eq!(subs[0]["role"].as_str().unwrap_or(""), "mother");
    assert_eq!(subs[1]["role"].as_str().unwrap_or(""), "son");
}
