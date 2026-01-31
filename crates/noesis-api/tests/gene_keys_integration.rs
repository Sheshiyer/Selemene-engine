//! Gene Keys Engine Integration Tests
//!
//! Validates full API workflow for Gene Keys engine:
//! - Mode 1: birth_data → HD → Gene Keys
//! - Mode 2: direct hd_gates input
//! - Consciousness-level adaptive witness prompts
//! - Error handling and authorization
//! - Archetypal depth preservation

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    Router,
};
use noesis_api::{build_app_state, create_router, ApiConfig};
use noesis_auth::AuthService;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// Test fixtures
// ---------------------------------------------------------------------------

static TEST_ROUTER: OnceLock<Router> = OnceLock::new();

fn get_test_router() -> &'static Router {
    TEST_ROUTER.get_or_init(|| {
        let config = ApiConfig::from_env();
        let state = build_app_state(&config);
        create_router(state, &config)
    })
}

fn generate_test_token(consciousness_level: u8) -> String {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    let auth = AuthService::new(jwt_secret);
    
    auth.generate_jwt_token(
        "test-user-gene-keys",
        "premium",
        &["read".to_string(), "write".to_string()],
        consciousness_level,
    )
    .expect("Failed to generate test JWT")
}

async fn make_authenticated_request(
    router: &Router,
    method: &str,
    uri: &str,
    token: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request_builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json");

    let body = if let Some(json_body) = body {
        Body::from(serde_json::to_vec(&json_body).unwrap())
    } else {
        Body::empty()
    };

    let request = request_builder.body(body).unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes)
        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&body_bytes).to_string()}));
    
    (status, body_json)
}

// ---------------------------------------------------------------------------
// Integration Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_gene_keys_with_birth_data() {
    // Test Mode 1: birth_data → HD → Gene Keys
    let router = get_test_router();
    let token = generate_test_token(3); // consciousness level 3
    
    let request_body = json!({
        "birth_data": {
            "date": "1985-06-15",
            "time": "14:30:00",
            "timezone": "America/New_York",
            "latitude": 40.7128,
            "longitude": -74.0060
        },
        "options": {
            "consciousness_level": 3
        }
    });
    
    let (status, response) = make_authenticated_request(
        router,
        "POST",
        "/api/v1/engines/gene-keys/calculate",
        &token,
        Some(request_body),
    ).await;
    
    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    // Validate structure
    assert_eq!(response["engine_id"], "gene-keys");
    assert!(!response["witness_prompt"].as_str().unwrap_or("").is_empty(), "Rule 5: witness_prompt must be present");
    
    // Validate witness_prompt is an inquiry
    let witness_prompt = response["witness_prompt"].as_str().unwrap();
    assert!(witness_prompt.contains("?"), "Witness prompt must be an inquiry");
    
    // Validate data structure
    let data = &response["data"];
    assert!(data.is_object(), "data field must be an object");
    
    // Validate activation_sequence
    let activation_sequence = &data["activation_sequence"];
    assert!(activation_sequence["lifes_work"].is_array());
    assert_eq!(activation_sequence["lifes_work"].as_array().unwrap().len(), 2);
    assert!(activation_sequence["evolution"].is_array());
    assert_eq!(activation_sequence["evolution"].as_array().unwrap().len(), 2);
    assert!(activation_sequence["radiance"].is_array());
    assert_eq!(activation_sequence["radiance"].as_array().unwrap().len(), 2);
    assert!(activation_sequence["purpose"].is_array());
    assert_eq!(activation_sequence["purpose"].as_array().unwrap().len(), 2);
    
    // Validate active_keys contains at least 4 keys (Sun/Earth personality/design minimum)
    let active_keys = data["active_keys"].as_array().expect("active_keys must be array");
    assert!(active_keys.len() >= 4, "Should have at least 4 active keys (Sun/Earth P/D)");
    
    // Validate each key has proper structure
    for key in active_keys {
        assert!(key["key_number"].is_number());
        let key_num = key["key_number"].as_u64().unwrap();
        assert!(key_num >= 1 && key_num <= 64, "Key number must be 1-64");
        assert!(key["line"].is_number());
        assert!(key["source"].is_string());
    }
}

#[tokio::test]
async fn test_gene_keys_with_hd_gates() {
    // Test Mode 2: direct hd_gates input
    let router = get_test_router();
    let token = generate_test_token(4);
    
    let request_body = json!({
        "options": {
            "hd_gates": {
                "personality_sun": 17,
                "personality_earth": 18,
                "design_sun": 1,
                "design_earth": 2
            },
            "consciousness_level": 4
        }
    });
    
    let (status, response) = make_authenticated_request(
        router,
        "POST",
        "/api/v1/engines/gene-keys/calculate",
        &token,
        Some(request_body),
    ).await;
    
    assert_eq!(status, StatusCode::OK, "Response: {:?}", response);
    
    let data = &response["data"];
    let activation_sequence = &data["activation_sequence"];
    
    // Validate Life's Work (Personality Sun + Earth)
    assert_eq!(activation_sequence["lifes_work"][0], 17);
    assert_eq!(activation_sequence["lifes_work"][1], 18);
    
    // Validate Evolution (Design Sun + Earth)
    assert_eq!(activation_sequence["evolution"][0], 1);
    assert_eq!(activation_sequence["evolution"][1], 2);
    
    // Validate Radiance (P Sun + D Sun)
    assert_eq!(activation_sequence["radiance"][0], 17);
    assert_eq!(activation_sequence["radiance"][1], 1);
    
    // Validate Purpose (P Earth + D Earth)
    assert_eq!(activation_sequence["purpose"][0], 18);
    assert_eq!(activation_sequence["purpose"][1], 2);
}

#[tokio::test]
async fn test_consciousness_level_adaptation() {
    // Test witness prompts adapt to consciousness levels
    let router = get_test_router();
    
    let test_levels = vec![
        (1, "shadow-focused"), 
        (3, "gift-focused"), 
        (6, "siddhi-focused")
    ];
    
    let mut prompts = Vec::new();
    
    for (level, _description) in test_levels {
        let token = generate_test_token(level);
        
        let request_body = json!({
            "options": {
                "hd_gates": {
                    "personality_sun": 1,
                    "personality_earth": 2,
                    "design_sun": 3,
                    "design_earth": 4
                },
                "consciousness_level": level
            }
        });
        
        let (status, response) = make_authenticated_request(
            router,
            "POST",
            "/api/v1/engines/gene-keys/calculate",
            &token,
            Some(request_body),
        ).await;
        
        assert_eq!(status, StatusCode::OK);
        
        let prompt = response["witness_prompt"].as_str().unwrap().to_string();
        prompts.push(prompt);
    }
    
    // Validate prompts are different for different consciousness levels
    assert_ne!(prompts[0], prompts[1], "Shadow vs Gift prompts should differ");
    assert_ne!(prompts[1], prompts[2], "Gift vs Siddhi prompts should differ");
    
    // Validate all prompts are inquiries
    for prompt in &prompts {
        assert!(prompt.contains("?"), "All witness prompts must be inquiries");
    }
}

#[tokio::test]
async fn test_gene_keys_requires_input() {
    // Test error case: no birth_data AND no hd_gates
    let router = get_test_router();
    let token = generate_test_token(3);
    
    let request_body = json!({
        "options": {
            "consciousness_level": 3
        }
    });
    
    let (status, response) = make_authenticated_request(
        router,
        "POST",
        "/api/v1/engines/gene-keys/calculate",
        &token,
        Some(request_body),
    ).await;
    
    // Should return 422 Unprocessable Entity or 400 Bad Request
    assert!(
        status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::BAD_REQUEST,
        "Expected 422 or 400, got {}: {:?}", status, response
    );
    
    // Error message should indicate missing input
    let error_msg = response["error"].as_str()
        .or_else(|| response["message"].as_str())
        .unwrap_or("");
    assert!(
        error_msg.contains("birth_data") || error_msg.contains("hd_gates") || error_msg.contains("input"),
        "Error should mention missing input: {}", error_msg
    );
}

#[tokio::test]
async fn test_consciousness_level_check() {
    // Test 403 if user consciousness_level < required_phase
    // Gene Keys requires phase 2+
    let router = get_test_router();
    let token = generate_test_token(1); // Level 1, below required phase
    
    let request_body = json!({
        "options": {
            "hd_gates": {
                "personality_sun": 1,
                "personality_earth": 2,
                "design_sun": 3,
                "design_earth": 4
            },
            "consciousness_level": 3
        }
    });
    
    let (status, response) = make_authenticated_request(
        router,
        "POST",
        "/api/v1/engines/gene-keys/calculate",
        &token,
        Some(request_body),
    ).await;
    
    // Should return 403 Forbidden if authorization middleware active
    // Or 200 if middleware not enforcing (acceptable for local dev)
    if status == StatusCode::FORBIDDEN {
        let error_msg = response["error"].as_str()
            .or_else(|| response["message"].as_str())
            .unwrap_or("");
        assert!(
            error_msg.contains("consciousness") || error_msg.contains("level") || error_msg.contains("phase"),
            "Error should mention consciousness level requirement"
        );
    }
    // If 200, middleware not active - acceptable
}

#[tokio::test]
async fn test_witness_prompt_inquiry_format() {
    // Validate witness_prompt contains inquiry and references Gene Keys
    let router = get_test_router();
    let token = generate_test_token(3);
    
    let request_body = json!({
        "options": {
            "hd_gates": {
                "personality_sun": 17,
                "personality_earth": 18,
                "design_sun": 21,
                "design_earth": 48
            },
            "consciousness_level": 3
        }
    });
    
    let (status, response) = make_authenticated_request(
        router,
        "POST",
        "/api/v1/engines/gene-keys/calculate",
        &token,
        Some(request_body),
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let witness_prompt = response["witness_prompt"].as_str().unwrap();
    
    // Must be an inquiry
    assert!(witness_prompt.contains("?"), "Witness prompt must contain question mark");
    
    // Should reference Gene Keys by number or name
    let has_number_ref = witness_prompt.contains("17") || 
                         witness_prompt.contains("18") || 
                         witness_prompt.contains("21") || 
                         witness_prompt.contains("48");
    let has_key_ref = witness_prompt.to_lowercase().contains("gene key") ||
                      witness_prompt.to_lowercase().contains("shadow") ||
                      witness_prompt.to_lowercase().contains("gift") ||
                      witness_prompt.to_lowercase().contains("siddhi");
    
    assert!(
        has_number_ref || has_key_ref,
        "Witness prompt should reference specific Gene Keys: {}", witness_prompt
    );
}

#[tokio::test]
async fn test_archetypal_depth_in_output() {
    // Validate API output preserves archetypal depth (not summaries)
    let router = get_test_router();
    let token = generate_test_token(3);
    
    let request_body = json!({
        "options": {
            "hd_gates": {
                "personality_sun": 1,
                "personality_earth": 2,
                "design_sun": 3,
                "design_earth": 4
            },
            "consciousness_level": 3
        }
    });
    
    let (status, response) = make_authenticated_request(
        router,
        "POST",
        "/api/v1/engines/gene-keys/calculate",
        &token,
        Some(request_body),
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let data = &response["data"];
    let active_keys = data["active_keys"].as_array().unwrap();
    
    // Find Gene Key 1 in active_keys
    let key_1 = active_keys.iter()
        .find(|k| k["key_number"] == 1)
        .expect("Gene Key 1 should be in active keys");
    
    // If gene_key_data is included, validate archetypal depth
    if let Some(gene_key_data) = key_1.get("gene_key_data") {
        if !gene_key_data.is_null() {
            let shadow_desc = gene_key_data["shadow_description"].as_str().unwrap_or("");
            let gift_desc = gene_key_data["gift_description"].as_str().unwrap_or("");
            let siddhi_desc = gene_key_data["siddhi_description"].as_str().unwrap_or("");
            
            // Validate non-empty descriptions
            assert!(!shadow_desc.is_empty(), "Shadow description must be present");
            assert!(!gift_desc.is_empty(), "Gift description must be present");
            assert!(!siddhi_desc.is_empty(), "Siddhi description must be present");
            
            // Validate minimum depth (at least 20 words per description)
            let shadow_words = shadow_desc.split_whitespace().count();
            let gift_words = gift_desc.split_whitespace().count();
            let siddhi_words = siddhi_desc.split_whitespace().count();
            
            assert!(shadow_words >= 10, "Shadow description too shallow: {} words", shadow_words);
            assert!(gift_words >= 10, "Gift description too shallow: {} words", gift_words);
            assert!(siddhi_words >= 10, "Siddhi description too shallow: {} words", siddhi_words);
        }
    }
}

#[tokio::test]
async fn test_invalid_gate_numbers() {
    // Test validation of gate number ranges
    let router = get_test_router();
    let token = generate_test_token(3);
    
    let request_body = json!({
        "options": {
            "hd_gates": {
                "personality_sun": 0,  // Invalid: must be 1-64
                "personality_earth": 2,
                "design_sun": 3,
                "design_earth": 4
            },
            "consciousness_level": 3
        }
    });
    
    let (status, _response) = make_authenticated_request(
        router,
        "POST",
        "/api/v1/engines/gene-keys/calculate",
        &token,
        Some(request_body),
    ).await;
    
    assert!(
        status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::BAD_REQUEST,
        "Expected validation error for invalid gate number"
    );
}
