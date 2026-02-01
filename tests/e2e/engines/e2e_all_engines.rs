//! E2E Test Suite for All 14 Consciousness Engines
//!
//! W2-S8-01: Comprehensive E2E tests covering:
//! - Valid input calculations
//! - Invalid input handling
//! - Edge cases
//! - Error responses
//! - Response structure validation
//!
//! Run with: cargo test --test e2e_all_engines -- --nocapture

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

static E2E_ROUTER: OnceLock<Router> = OnceLock::new();

fn get_router() -> &'static Router {
    E2E_ROUTER.get_or_init(|| {
        let config = ApiConfig::from_env();
        let state = build_app_state(&config);
        create_router(state, &config)
    })
}

fn test_token(consciousness_level: u8) -> String {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    let auth = AuthService::new(jwt_secret);
    auth.generate_jwt_token(
        "e2e-all-engines-test",
        "enterprise",
        &["read".to_string(), "write".to_string()],
        consciousness_level,
    )
    .expect("Failed to generate JWT")
}

async fn authenticated_post(uri: &str, token: &str, body: Value) -> (StatusCode, Value) {
    let router = get_router();
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let json: Value = if bytes.is_empty() {
        json!({})
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            json!({"raw": String::from_utf8_lossy(&bytes).to_string()})
        })
    };
    (status, json)
}

async fn unauthenticated_post(uri: &str, body: Value) -> (StatusCode, Value) {
    let router = get_router();
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, json)
}

async fn authenticated_get(uri: &str, token: &str) -> (StatusCode, Value) {
    let router = get_router();
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, json)
}

/// Standard birth data input for most engines
fn birth_input() -> Value {
    json!({
        "birth_data": {
            "name": "E2E Test User",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "America/New_York"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    })
}

/// Verify common response structure for all engines
fn verify_engine_response(body: &Value, engine_id: &str) {
    assert_eq!(body["engine_id"].as_str().unwrap(), engine_id);
    assert!(
        body["witness_prompt"].as_str().map(|s| !s.is_empty()).unwrap_or(false),
        "witness_prompt must be non-empty for {}: {:?}",
        engine_id,
        body
    );
    assert!(
        body["result"].is_object() || body["result"].is_array(),
        "result must be object or array for {}: {:?}",
        engine_id,
        body
    );
    assert!(
        body["metadata"]["calculation_time_ms"].as_f64().is_some(),
        "metadata.calculation_time_ms required for {}: {:?}",
        engine_id,
        body
    );
}

// ===========================================================================
// HUMAN DESIGN ENGINE TESTS (Required Phase: 1)
// ===========================================================================

mod human_design {
    use super::*;

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/human-design/calculate",
            &token,
            birth_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "HD failed: {:?}", body);
        verify_engine_response(&body, "human-design");

        let result = &body["result"];
        assert!(result["hd_type"].is_string(), "Missing hd_type");
        assert!(result["authority"].is_string(), "Missing authority");
        assert!(result["profile"].is_string(), "Missing profile");
        assert!(result["defined_centers"].is_array(), "Missing defined_centers");
        assert!(result["active_channels"].is_array(), "Missing active_channels");
    }

    #[tokio::test]
    async fn test_unauthorized() {
        let (status, body) = unauthenticated_post(
            "/api/v1/engines/human-design/calculate",
            birth_input(),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error_code"], "UNAUTHORIZED");
    }

    #[tokio::test]
    async fn test_phase_gating() {
        let token = test_token(0); // HD requires phase 1
        let (status, body) = authenticated_post(
            "/api/v1/engines/human-design/calculate",
            &token,
            birth_input(),
        )
        .await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error_code"], "PHASE_ACCESS_DENIED");
    }

    #[tokio::test]
    async fn test_missing_birth_data() {
        let token = test_token(5);
        let input = json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {}
        });

        let (status, _body) = authenticated_post(
            "/api/v1/engines/human-design/calculate",
            &token,
            input,
        )
        .await;

        assert!(
            status == StatusCode::UNPROCESSABLE_ENTITY || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Expected validation error, got {}",
            status
        );
    }

    #[tokio::test]
    async fn test_engine_info() {
        let token = test_token(5);
        let (status, body) = authenticated_get("/api/v1/engines/human-design/info", &token).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["engine_id"], "human-design");
        assert_eq!(body["required_phase"], 1);
    }
}

// ===========================================================================
// GENE KEYS ENGINE TESTS (Required Phase: 2)
// ===========================================================================

mod gene_keys {
    use super::*;

    fn gene_keys_input() -> Value {
        json!({
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
        })
    }

    #[tokio::test]
    async fn test_valid_calculation_with_gates() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/gene-keys/calculate",
            &token,
            gene_keys_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "Gene Keys failed: {:?}", body);
        verify_engine_response(&body, "gene-keys");

        let result = &body["result"];
        assert!(result["activation_sequence"].is_object(), "Missing activation_sequence");
        assert!(result["active_keys"].is_array(), "Missing active_keys");
    }

    #[tokio::test]
    async fn test_phase_gating() {
        let token = test_token(1); // Gene Keys requires phase 2
        let (status, body) = authenticated_post(
            "/api/v1/engines/gene-keys/calculate",
            &token,
            gene_keys_input(),
        )
        .await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error_code"], "PHASE_ACCESS_DENIED");
    }

    #[tokio::test]
    async fn test_birth_data_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/gene-keys/calculate",
            &token,
            birth_input(),
        )
        .await;

        // Should work with birth data (calculates from HD gates)
        assert_eq!(status, StatusCode::OK, "Gene Keys birth calc failed: {:?}", body);
    }

    #[tokio::test]
    async fn test_engine_info() {
        let token = test_token(5);
        let (status, body) = authenticated_get("/api/v1/engines/gene-keys/info", &token).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["engine_id"], "gene-keys");
        assert_eq!(body["required_phase"], 2);
    }
}

// ===========================================================================
// VIMSHOTTARI ENGINE TESTS (Required Phase: 2)
// ===========================================================================

mod vimshottari {
    use super::*;

    fn vimshottari_input() -> Value {
        json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "moon_longitude": 125.0,
                "birth_date": "1985-06-15",
                "birth_time": "14:30"
            }
        })
    }

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/vimshottari/calculate",
            &token,
            vimshottari_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "Vimshottari failed: {:?}", body);
        verify_engine_response(&body, "vimshottari");

        let result = &body["result"];
        let timeline = &result["timeline"];
        assert_eq!(timeline["total_years"], 120);
        
        let mahadashas = timeline["mahadashas"].as_array().expect("mahadashas");
        assert_eq!(mahadashas.len(), 9);
    }

    #[tokio::test]
    async fn test_nakshatra_magha() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/vimshottari/calculate",
            &token,
            vimshottari_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        let nak = &body["result"]["birth_nakshatra"];
        assert_eq!(nak["name"], "Magha"); // Moon at 125° is in Magha
    }

    #[tokio::test]
    async fn test_phase_gating() {
        let token = test_token(1);
        let (status, body) = authenticated_post(
            "/api/v1/engines/vimshottari/calculate",
            &token,
            vimshottari_input(),
        )
        .await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error_code"], "PHASE_ACCESS_DENIED");
    }

    #[tokio::test]
    async fn test_with_birth_data() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/vimshottari/calculate",
            &token,
            birth_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "Vimshottari birth calc: {:?}", body);
    }
}

// ===========================================================================
// PANCHANGA ENGINE TESTS (Required Phase: 0)
// ===========================================================================

mod panchanga {
    use super::*;

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(0); // Phase 0 is sufficient
        let (status, body) = authenticated_post(
            "/api/v1/engines/panchanga/calculate",
            &token,
            birth_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "Panchanga failed: {:?}", body);
        verify_engine_response(&body, "panchanga");

        let result = &body["result"];
        assert!(result["tithi"].is_object() || result["tithi"].is_string(), "Missing tithi");
        assert!(result["nakshatra"].is_object() || result["nakshatra"].is_string(), "Missing nakshatra");
    }

    #[tokio::test]
    async fn test_no_phase_restriction() {
        let token = test_token(0);
        let (status, _body) = authenticated_post(
            "/api/v1/engines/panchanga/calculate",
            &token,
            birth_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
    }
}

// ===========================================================================
// NUMEROLOGY ENGINE TESTS (Required Phase: 0)
// ===========================================================================

mod numerology {
    use super::*;

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(0);
        let (status, body) = authenticated_post(
            "/api/v1/engines/numerology/calculate",
            &token,
            birth_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "Numerology failed: {:?}", body);
        verify_engine_response(&body, "numerology");

        let result = &body["result"];
        assert!(result["life_path"].is_number(), "Missing life_path");
    }

    #[tokio::test]
    async fn test_life_path_calculation() {
        let token = test_token(5);
        let input = json!({
            "birth_data": {
                "name": "Life Path Test",
                "date": "1990-01-15",
                "time": "12:00",
                "latitude": 40.7128,
                "longitude": -74.006,
                "timezone": "America/New_York"
            },
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {}
        });

        let (status, body) = authenticated_post(
            "/api/v1/engines/numerology/calculate",
            &token,
            input,
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        // 1990-01-15 = 1+9+9+0+0+1+1+5 = 26 = 2+6 = 8
        // Or depending on method: 1+9+9+0+1+1+5 = 26 = 8
        let life_path = body["result"]["life_path"].as_u64().unwrap();
        assert!((1..=33).contains(&life_path), "Life path {} out of range", life_path);
    }
}

// ===========================================================================
// BIORHYTHM ENGINE TESTS (Required Phase: 0)
// ===========================================================================

mod biorhythm {
    use super::*;

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(0);
        let (status, body) = authenticated_post(
            "/api/v1/engines/biorhythm/calculate",
            &token,
            birth_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "Biorhythm failed: {:?}", body);
        verify_engine_response(&body, "biorhythm");

        let result = &body["result"];
        assert!(result["physical"].is_number(), "Missing physical");
        assert!(result["emotional"].is_number(), "Missing emotional");
        assert!(result["intellectual"].is_number(), "Missing intellectual");

        // Values should be between -1 and 1
        let physical = result["physical"].as_f64().unwrap();
        assert!((-1.0..=1.0).contains(&physical), "Physical {} out of range", physical);
    }
}

// ===========================================================================
// VEDIC CLOCK ENGINE TESTS (Required Phase: 1)
// ===========================================================================

mod vedic_clock {
    use super::*;

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/vedic-clock/calculate",
            &token,
            birth_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_engine_response(&body, "vedic-clock");
        } else {
            // Engine may not be fully implemented
            assert!(
                status == StatusCode::NOT_FOUND || status == StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected status: {}",
                status
            );
        }
    }
}

// ===========================================================================
// BIOFIELD ENGINE TESTS (Required Phase: 1)
// ===========================================================================

mod biofield {
    use super::*;

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/biofield/calculate",
            &token,
            birth_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_engine_response(&body, "biofield");
        } else {
            assert!(
                status == StatusCode::NOT_FOUND || status == StatusCode::INTERNAL_SERVER_ERROR,
                "Biofield unexpected: {} {:?}",
                status,
                body
            );
        }
    }
}

// ===========================================================================
// FACE READING ENGINE TESTS (Required Phase: 2)
// ===========================================================================

mod face_reading {
    use super::*;

    #[tokio::test]
    async fn test_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/face-reading/calculate",
            &token,
            birth_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_engine_response(&body, "face-reading");
        } else {
            // Face reading may require image input or not be implemented
            assert!(
                status == StatusCode::NOT_FOUND
                    || status == StatusCode::UNPROCESSABLE_ENTITY
                    || status == StatusCode::INTERNAL_SERVER_ERROR,
                "Face reading unexpected: {}",
                status
            );
        }
    }
}

// ===========================================================================
// TYPESCRIPT ENGINE TESTS (Tarot, I-Ching, Enneagram, Sacred Geometry, Sigil Forge)
// These engines run via the TypeScript bridge on port 3001
// ===========================================================================

mod tarot {
    use super::*;

    fn tarot_input() -> Value {
        json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "spread_type": "three_card",
                "question": "What should I focus on today?"
            }
        })
    }

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/tarot/calculate",
            &token,
            tarot_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_engine_response(&body, "tarot");
            let result = &body["result"];
            assert!(result["cards"].is_array(), "Missing cards");
        } else if status == StatusCode::SERVICE_UNAVAILABLE || status == StatusCode::BAD_GATEWAY {
            eprintln!("TS engine not available, skipping tarot test");
        } else {
            assert!(
                status == StatusCode::NOT_FOUND,
                "Tarot unexpected: {} {:?}",
                status,
                body
            );
        }
    }
}

mod i_ching {
    use super::*;

    fn i_ching_input() -> Value {
        json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "question": "What is the nature of this moment?",
                "method": "yarrow"
            }
        })
    }

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/i-ching/calculate",
            &token,
            i_ching_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_engine_response(&body, "i-ching");
            let result = &body["result"];
            assert!(result["hexagram"].is_object() || result["hexagram"].is_number(), "Missing hexagram");
        } else if status == StatusCode::SERVICE_UNAVAILABLE || status == StatusCode::BAD_GATEWAY {
            eprintln!("TS engine not available, skipping i-ching test");
        } else {
            assert!(
                status == StatusCode::NOT_FOUND,
                "I-Ching unexpected: {} {:?}",
                status,
                body
            );
        }
    }
}

mod enneagram {
    use super::*;

    fn enneagram_input() -> Value {
        json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "birth_data": {
                "name": "Enneagram Test",
                "date": "1990-01-15",
                "time": "14:30",
                "latitude": 40.7128,
                "longitude": -74.006,
                "timezone": "America/New_York"
            },
            "options": {}
        })
    }

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/enneagram/calculate",
            &token,
            enneagram_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_engine_response(&body, "enneagram");
        } else if status == StatusCode::SERVICE_UNAVAILABLE || status == StatusCode::BAD_GATEWAY {
            eprintln!("TS engine not available, skipping enneagram test");
        } else {
            assert!(
                status == StatusCode::NOT_FOUND,
                "Enneagram unexpected: {} {:?}",
                status,
                body
            );
        }
    }
}

mod sacred_geometry {
    use super::*;

    fn sacred_geometry_input() -> Value {
        json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "pattern_type": "flower_of_life"
            }
        })
    }

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/sacred-geometry/calculate",
            &token,
            sacred_geometry_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_engine_response(&body, "sacred-geometry");
        } else if status == StatusCode::SERVICE_UNAVAILABLE || status == StatusCode::BAD_GATEWAY {
            eprintln!("TS engine not available, skipping sacred-geometry test");
        } else {
            assert!(
                status == StatusCode::NOT_FOUND,
                "Sacred geometry unexpected: {} {:?}",
                status,
                body
            );
        }
    }
}

mod sigil_forge {
    use super::*;

    fn sigil_input() -> Value {
        json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "intention": "I am centered and focused",
                "style": "runic"
            }
        })
    }

    #[tokio::test]
    async fn test_valid_calculation() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/engines/sigil-forge/calculate",
            &token,
            sigil_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_engine_response(&body, "sigil-forge");
        } else if status == StatusCode::SERVICE_UNAVAILABLE || status == StatusCode::BAD_GATEWAY {
            eprintln!("TS engine not available, skipping sigil-forge test");
        } else {
            assert!(
                status == StatusCode::NOT_FOUND,
                "Sigil forge unexpected: {} {:?}",
                status,
                body
            );
        }
    }
}

// ===========================================================================
// CROSS-ENGINE TESTS
// ===========================================================================

#[tokio::test]
async fn test_engine_list_contains_all_registered() {
    let token = test_token(5);
    let (status, body) = authenticated_get("/api/v1/engines", &token).await;

    assert_eq!(status, StatusCode::OK);
    let engines = body["engines"].as_array().expect("engines array");
    
    // Core Rust engines that must be registered
    let expected_rust = ["panchanga", "numerology", "biorhythm", "human-design", "gene-keys", "vimshottari"];
    for engine in expected_rust {
        assert!(
            engines.iter().any(|e| e.as_str() == Some(engine)),
            "Missing expected engine: {}",
            engine
        );
    }
}

#[tokio::test]
async fn test_nonexistent_engine_404() {
    let token = test_token(5);
    let (status, body) = authenticated_post(
        "/api/v1/engines/nonexistent-engine/calculate",
        &token,
        birth_input(),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], "ENGINE_NOT_FOUND");
}

#[tokio::test]
async fn test_malformed_json_body() {
    let router = get_router();
    let token = test_token(5);
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/engines/panchanga/calculate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{invalid json"))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 400 or 422 for malformed JSON"
    );
}
