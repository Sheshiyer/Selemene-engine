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

    // Existing frozen contract
    assert!(json.get("aletheios").is_some());
    assert!(json.get("pichet").is_some());
    assert!(json.get("synthesis").is_some());
    assert!(json.get("witness_question").is_some());
    assert!(json.get("engines_used").is_some());
    assert!(json.get("llm_powered").is_some());
}
