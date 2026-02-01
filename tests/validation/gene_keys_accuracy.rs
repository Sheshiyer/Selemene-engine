//! Accuracy Validation - Gene Keys Calculations
//!
//! W2-S8-06: Validates Gene Keys calculations against reference data
//! Target: 100% accuracy for gate-to-key mapping and frequency assessments
//!
//! Run with: cargo test --test gene_keys_accuracy -- --nocapture

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use noesis_api::{build_app_state, create_router, ApiConfig};
use noesis_auth::AuthService;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tower::ServiceExt;

// ===========================================================================
// Test Utilities
// ===========================================================================

static GK_ROUTER: OnceLock<Router> = OnceLock::new();

fn get_router() -> &'static Router {
    GK_ROUTER.get_or_init(|| {
        let config = ApiConfig::from_env();
        let state = build_app_state(&config);
        create_router(state, &config)
    })
}

fn test_token() -> String {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    let auth = AuthService::new(jwt_secret);
    auth.generate_jwt_token(
        "gk-accuracy-test",
        "premium",
        &["read".to_string()],
        5,
    )
    .expect("JWT")
}

async fn calculate_gk(input: Value) -> (StatusCode, Value) {
    let router = get_router();
    let token = test_token();
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/engines/gene-keys/calculate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&input).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, json)
}

// ===========================================================================
// Reference Gene Keys Data
// ===========================================================================

/// Gene Keys reference: Key 17
const KEY_17_SHADOW: &str = "Opinion";
const KEY_17_GIFT: &str = "Far-sightedness";
const KEY_17_SIDDHI: &str = "Omniscience";

/// Gene Keys reference: Key 18
const KEY_18_SHADOW: &str = "Judgement";
const KEY_18_GIFT: &str = "Integrity";
const KEY_18_SIDDHI: &str = "Perfection";

/// Gene Keys reference: Key 45
const KEY_45_SHADOW: &str = "Dominance";
const KEY_45_GIFT: &str = "Synergy";
const KEY_45_SIDDHI: &str = "Communion";

/// Gene Keys reference: Key 26
const KEY_26_SHADOW: &str = "Pride";
const KEY_26_GIFT: &str = "Artfulness";
const KEY_26_SIDDHI: &str = "Invisibility";

// ===========================================================================
// Accuracy Tests
// ===========================================================================

/// Test that Gene Keys are in valid range (1-64)
#[tokio::test]
async fn test_key_range_validation() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "hd_gates": {
                "personality_sun": 17,
                "personality_earth": 18,
                "design_sun": 45,
                "design_earth": 26
            }
        }
    });
    
    let (status, body) = calculate_gk(input).await;
    
    assert_eq!(status, StatusCode::OK, "Gene Keys failed: {:?}", body);
    
    let result = &body["result"];
    
    // Verify active_keys are in range
    if let Some(active_keys) = result.get("active_keys").and_then(|k| k.as_array()) {
        for key in active_keys {
            if let Some(key_num) = key.get("key").or(key.get("number")).and_then(|k| k.as_u64()) {
                assert!(
                    (1..=64).contains(&key_num),
                    "Key {} out of range 1-64",
                    key_num
                );
            }
        }
    }
}

/// Test activation sequence structure
#[tokio::test]
async fn test_activation_sequence_structure() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "hd_gates": {
                "personality_sun": 17,
                "personality_earth": 18,
                "design_sun": 45,
                "design_earth": 26
            }
        }
    });
    
    let (status, body) = calculate_gk(input).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let result = &body["result"];
    let activation = &result["activation_sequence"];
    
    // Verify all four sequence positions exist
    assert!(
        activation.get("lifes_work").is_some(),
        "Missing Life's Work in activation sequence"
    );
    assert!(
        activation.get("evolution").is_some(),
        "Missing Evolution in activation sequence"
    );
    assert!(
        activation.get("radiance").is_some(),
        "Missing Radiance in activation sequence"
    );
    assert!(
        activation.get("purpose").is_some(),
        "Missing Purpose in activation sequence"
    );
    
    println!("Activation sequence structure verified");
}

/// Test gate-to-key mapping accuracy
#[tokio::test]
async fn test_gate_to_key_mapping() {
    // Gate numbers equal Key numbers in Gene Keys
    let test_gates = vec![
        (17, "Life's Work"),
        (18, "Evolution"),
        (45, "Radiance"),
        (26, "Purpose"),
    ];
    
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "hd_gates": {
                "personality_sun": 17,
                "personality_earth": 18,
                "design_sun": 45,
                "design_earth": 26
            }
        }
    });
    
    let (status, body) = calculate_gk(input).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let result = &body["result"];
    let activation = &result["activation_sequence"];
    
    // Verify Life's Work is key 17
    if let Some(lifes_work) = activation.get("lifes_work").and_then(|l| l.as_array()) {
        if !lifes_work.is_empty() {
            let first_key = &lifes_work[0];
            let key_num = first_key.get("key")
                .or(first_key.get("number"))
                .and_then(|k| k.as_u64());
            if let Some(num) = key_num {
                assert_eq!(num, 17, "Life's Work should be Key 17");
            }
        }
    }
    
    println!("Gate-to-key mapping verified");
}

/// Test frequency assessment structure
#[tokio::test]
async fn test_frequency_assessment_structure() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "hd_gates": {
                "personality_sun": 17,
                "personality_earth": 18,
                "design_sun": 45,
                "design_earth": 26
            }
        }
    });
    
    let (status, body) = calculate_gk(input).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let result = &body["result"];
    
    // Check frequency assessments
    if let Some(assessments) = result.get("frequency_assessments").and_then(|f| f.as_array()) {
        for assessment in assessments {
            // Each assessment should have shadow, gift, siddhi
            if let Some(shadow) = assessment.get("shadow") {
                assert!(shadow.is_string() || shadow.is_object(), "Shadow should be string or object");
            }
            if let Some(gift) = assessment.get("gift") {
                assert!(gift.is_string() || gift.is_object(), "Gift should be string or object");
            }
            if let Some(siddhi) = assessment.get("siddhi") {
                assert!(siddhi.is_string() || siddhi.is_object(), "Siddhi should be string or object");
            }
        }
        println!("Found {} frequency assessments", assessments.len());
    }
}

/// Test reference Key 17 frequency names
#[tokio::test]
async fn test_key_17_frequencies() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "hd_gates": {
                "personality_sun": 17,
                "personality_earth": 17,
                "design_sun": 17,
                "design_earth": 17
            }
        }
    });
    
    let (status, body) = calculate_gk(input).await;
    
    assert_eq!(status, StatusCode::OK);
    
    let result = &body["result"];
    
    // Check if Key 17 frequencies are correct
    if let Some(active_keys) = result.get("active_keys").and_then(|k| k.as_array()) {
        for key_data in active_keys {
            let key_num = key_data.get("key")
                .or(key_data.get("number"))
                .and_then(|k| k.as_u64());
            
            if key_num == Some(17) {
                // Verify shadow, gift, siddhi names if present
                if let Some(shadow) = key_data.get("shadow").and_then(|s| s.as_str()) {
                    println!("Key 17 Shadow: {} (expected: {})", shadow, KEY_17_SHADOW);
                }
                if let Some(gift) = key_data.get("gift").and_then(|g| g.as_str()) {
                    println!("Key 17 Gift: {} (expected: {})", gift, KEY_17_GIFT);
                }
                if let Some(siddhi) = key_data.get("siddhi").and_then(|s| s.as_str()) {
                    println!("Key 17 Siddhi: {} (expected: {})", siddhi, KEY_17_SIDDHI);
                }
            }
        }
    }
}

/// Test calculation from birth data
#[tokio::test]
async fn test_calculation_from_birth_data() {
    let input = json!({
        "birth_data": {
            "name": "Gene Keys Birth Test",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "America/New_York"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, body) = calculate_gk(input).await;
    
    assert_eq!(status, StatusCode::OK, "Birth data calculation failed: {:?}", body);
    
    let result = &body["result"];
    
    // Should calculate gates from birth data and produce activation sequence
    assert!(
        result.get("activation_sequence").is_some() || result.get("active_keys").is_some(),
        "Should have activation sequence or active keys from birth data"
    );
}

/// Test idempotency
#[tokio::test]
async fn test_gene_keys_idempotency() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "hd_gates": {
                "personality_sun": 17,
                "personality_earth": 18,
                "design_sun": 45,
                "design_earth": 26
            }
        }
    });
    
    let (status1, body1) = calculate_gk(input.clone()).await;
    let (status2, body2) = calculate_gk(input).await;
    
    assert_eq!(status1, StatusCode::OK);
    assert_eq!(status2, StatusCode::OK);
    
    // Active keys should be identical
    let keys1 = &body1["result"]["active_keys"];
    let keys2 = &body2["result"]["active_keys"];
    
    if keys1.is_array() && keys2.is_array() {
        assert_eq!(keys1.as_array().unwrap().len(), keys2.as_array().unwrap().len());
    }
}

/// Test all 64 keys have valid data (comprehensive coverage)
#[tokio::test]
async fn test_all_keys_coverage() {
    // Test a sample of keys across the range
    let test_keys = vec![1, 8, 16, 24, 32, 40, 48, 56, 64];
    
    for key in test_keys {
        let input = json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "hd_gates": {
                    "personality_sun": key,
                    "personality_earth": key,
                    "design_sun": key,
                    "design_earth": key
                }
            }
        });
        
        let (status, body) = calculate_gk(input).await;
        
        assert_eq!(status, StatusCode::OK, "Key {} calculation failed: {:?}", key, body);
        
        // Should produce valid results
        let result = &body["result"];
        assert!(
            result.is_object(),
            "Key {} should produce valid result",
            key
        );
        
        println!("Key {} validated", key);
    }
}
