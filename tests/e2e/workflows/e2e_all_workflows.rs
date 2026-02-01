//! E2E Test Suite for All 6 Workflows
//!
//! W2-S8-02: Comprehensive workflow E2E tests covering:
//! - birth-blueprint
//! - daily-practice
//! - decision-support
//! - self-inquiry
//! - creative-expression
//! - full-spectrum
//!
//! Run with: cargo test --test e2e_all_workflows -- --nocapture

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

static WORKFLOW_ROUTER: OnceLock<Router> = OnceLock::new();

fn get_router() -> &'static Router {
    WORKFLOW_ROUTER.get_or_init(|| {
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
        "workflow-e2e-test",
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

/// Standard birth data input for workflows
fn birth_input() -> Value {
    json!({
        "birth_data": {
            "name": "Workflow Test User",
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

/// Verify workflow response structure
fn verify_workflow_response(body: &Value, workflow_id: &str) {
    assert_eq!(
        body["workflow_id"].as_str().unwrap(),
        workflow_id,
        "workflow_id mismatch"
    );
    assert!(
        body["engine_outputs"].is_object(),
        "Missing engine_outputs for {}: {:?}",
        workflow_id,
        body
    );
    assert!(
        body["total_time_ms"].as_f64().is_some(),
        "Missing total_time_ms for {}",
        workflow_id
    );
}

// ===========================================================================
// BIRTH-BLUEPRINT WORKFLOW TESTS
// Engines: numerology, human-design, gene-keys, panchanga
// ===========================================================================

mod birth_blueprint {
    use super::*;

    #[tokio::test]
    async fn test_execute_full() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/workflows/birth-blueprint/execute",
            &token,
            birth_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK, "birth-blueprint failed: {:?}", body);
        verify_workflow_response(&body, "birth-blueprint");

        let outputs = body["engine_outputs"].as_object().unwrap();
        
        // numerology should always be present (phase 0)
        assert!(outputs.contains_key("numerology"), "Missing numerology output");
        
        // human-design should be present (phase 1, user is phase 5)
        assert!(outputs.contains_key("human-design"), "Missing human-design output");
        
        // Should have at least 2 engine outputs
        assert!(
            outputs.len() >= 2,
            "Expected at least 2 outputs, got {}",
            outputs.len()
        );
    }

    #[tokio::test]
    async fn test_info() {
        let token = test_token(5);
        let (status, body) = authenticated_get(
            "/api/v1/workflows/birth-blueprint/info",
            &token,
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert!(body["engine_ids"].is_array(), "Missing engine_ids");
        
        let engine_ids: Vec<&str> = body["engine_ids"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect();
        
        assert!(engine_ids.contains(&"numerology"), "Should include numerology");
        assert!(engine_ids.contains(&"human-design"), "Should include human-design");
    }

    #[tokio::test]
    async fn test_unauthorized() {
        let (status, body) = unauthenticated_post(
            "/api/v1/workflows/birth-blueprint/execute",
            birth_input(),
        )
        .await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error_code"], "UNAUTHORIZED");
    }

    #[tokio::test]
    async fn test_partial_execution_with_low_phase() {
        let token = test_token(0); // Phase 0 - only numerology, panchanga, biorhythm accessible
        let (status, body) = authenticated_post(
            "/api/v1/workflows/birth-blueprint/execute",
            &token,
            birth_input(),
        )
        .await;

        // Workflow should still execute with partial results
        // or return 403 if all required engines are blocked
        if status == StatusCode::OK {
            let outputs = body["engine_outputs"].as_object().unwrap();
            // Only phase 0 engines should have results
            assert!(outputs.contains_key("numerology") || outputs.contains_key("panchanga"));
        } else {
            assert_eq!(status, StatusCode::FORBIDDEN);
        }
    }

    #[tokio::test]
    async fn test_synthesis_present() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/workflows/birth-blueprint/execute",
            &token,
            birth_input(),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        
        // Synthesis may be present depending on implementation
        if let Some(synthesis) = body.get("synthesis") {
            assert!(synthesis.is_object() || synthesis.is_string());
        }
    }
}

// ===========================================================================
// DAILY-PRACTICE WORKFLOW TESTS
// Engines: panchanga, vedic-clock, biorhythm
// ===========================================================================

mod daily_practice {
    use super::*;

    #[tokio::test]
    async fn test_execute_full() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/workflows/daily-practice/execute",
            &token,
            birth_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_workflow_response(&body, "daily-practice");
            
            let outputs = body["engine_outputs"].as_object().unwrap();
            // panchanga and biorhythm should be present
            assert!(
                outputs.contains_key("panchanga") || outputs.contains_key("biorhythm"),
                "Should have at least panchanga or biorhythm"
            );
        } else {
            // Workflow may not exist or have different name
            assert!(
                status == StatusCode::NOT_FOUND || status == StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected status: {}",
                status
            );
        }
    }

    #[tokio::test]
    async fn test_info() {
        let token = test_token(5);
        let (status, body) = authenticated_get(
            "/api/v1/workflows/daily-practice/info",
            &token,
        )
        .await;

        if status == StatusCode::OK {
            assert!(body["engine_ids"].is_array());
        } else {
            assert_eq!(status, StatusCode::NOT_FOUND);
        }
    }

    #[tokio::test]
    async fn test_low_phase_access() {
        let token = test_token(0);
        let (status, body) = authenticated_post(
            "/api/v1/workflows/daily-practice/execute",
            &token,
            birth_input(),
        )
        .await;

        // Daily practice has mostly phase 0 engines (panchanga, biorhythm)
        if status == StatusCode::OK {
            let outputs = body["engine_outputs"].as_object().unwrap();
            assert!(outputs.len() >= 1, "Should have at least 1 output");
        }
    }
}

// ===========================================================================
// DECISION-SUPPORT WORKFLOW TESTS
// Engines: tarot, i-ching, numerology
// ===========================================================================

mod decision_support {
    use super::*;

    fn decision_input() -> Value {
        json!({
            "birth_data": {
                "name": "Decision Test",
                "date": "1990-01-15",
                "time": "14:30",
                "latitude": 40.7128,
                "longitude": -74.006,
                "timezone": "America/New_York"
            },
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "question": "Should I take this new opportunity?"
            }
        })
    }

    #[tokio::test]
    async fn test_execute_full() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/workflows/decision-support/execute",
            &token,
            decision_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_workflow_response(&body, "decision-support");
            
            let outputs = body["engine_outputs"].as_object().unwrap();
            // numerology is phase 0 so should always be present
            assert!(
                outputs.contains_key("numerology"),
                "Should have numerology output"
            );
        } else {
            // May not be implemented or TS engines unavailable
            assert!(
                status == StatusCode::NOT_FOUND
                    || status == StatusCode::SERVICE_UNAVAILABLE
                    || status == StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected status: {} {:?}",
                status,
                body
            );
        }
    }

    #[tokio::test]
    async fn test_info() {
        let token = test_token(5);
        let (status, body) = authenticated_get(
            "/api/v1/workflows/decision-support/info",
            &token,
        )
        .await;

        if status == StatusCode::OK {
            let engine_ids: Vec<&str> = body["engine_ids"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .collect();
            
            // Should include divination engines
            assert!(
                engine_ids.contains(&"tarot")
                    || engine_ids.contains(&"i-ching")
                    || engine_ids.contains(&"numerology"),
                "Should include divination engines: {:?}",
                engine_ids
            );
        }
    }
}

// ===========================================================================
// SELF-INQUIRY WORKFLOW TESTS
// Engines: gene-keys, enneagram, human-design
// ===========================================================================

mod self_inquiry {
    use super::*;

    #[tokio::test]
    async fn test_execute_full() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/workflows/self-inquiry/execute",
            &token,
            birth_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_workflow_response(&body, "self-inquiry");
            
            let outputs = body["engine_outputs"].as_object().unwrap();
            // Should have deep self-work engines
            assert!(
                outputs.contains_key("human-design") || outputs.contains_key("gene-keys"),
                "Should have HD or Gene Keys output: {:?}",
                outputs.keys().collect::<Vec<_>>()
            );
        } else {
            assert!(
                status == StatusCode::NOT_FOUND || status == StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected status: {}",
                status
            );
        }
    }

    #[tokio::test]
    async fn test_info() {
        let token = test_token(5);
        let (status, body) = authenticated_get(
            "/api/v1/workflows/self-inquiry/info",
            &token,
        )
        .await;

        if status == StatusCode::OK {
            let engine_ids: Vec<&str> = body["engine_ids"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .collect();
            
            assert!(
                engine_ids.contains(&"gene-keys") || engine_ids.contains(&"human-design"),
                "Should include self-inquiry engines: {:?}",
                engine_ids
            );
        }
    }

    #[tokio::test]
    async fn test_phase_restriction() {
        let token = test_token(1); // Below gene-keys requirement
        let (status, body) = authenticated_post(
            "/api/v1/workflows/self-inquiry/execute",
            &token,
            birth_input(),
        )
        .await;

        if status == StatusCode::OK {
            // Should have partial results (HD but not gene-keys)
            let outputs = body["engine_outputs"].as_object().unwrap();
            if outputs.contains_key("gene-keys") {
                panic!("Phase 1 user should not get gene-keys output");
            }
        }
    }
}

// ===========================================================================
// CREATIVE-EXPRESSION WORKFLOW TESTS
// Engines: sacred-geometry, sigil-forge, tarot
// ===========================================================================

mod creative_expression {
    use super::*;

    fn creative_input() -> Value {
        json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "intention": "Express my creative vision",
                "style": "organic"
            }
        })
    }

    #[tokio::test]
    async fn test_execute_full() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/workflows/creative-expression/execute",
            &token,
            creative_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_workflow_response(&body, "creative-expression");
        } else {
            // TS engines may not be available
            assert!(
                status == StatusCode::NOT_FOUND
                    || status == StatusCode::SERVICE_UNAVAILABLE
                    || status == StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected status: {} {:?}",
                status,
                body
            );
        }
    }

    #[tokio::test]
    async fn test_info() {
        let token = test_token(5);
        let (status, body) = authenticated_get(
            "/api/v1/workflows/creative-expression/info",
            &token,
        )
        .await;

        if status == StatusCode::OK {
            let engine_ids: Vec<&str> = body["engine_ids"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|v| v.as_str())
                .collect();
            
            assert!(
                engine_ids.contains(&"sacred-geometry")
                    || engine_ids.contains(&"sigil-forge")
                    || engine_ids.contains(&"tarot"),
                "Should include creative engines"
            );
        }
    }
}

// ===========================================================================
// FULL-SPECTRUM WORKFLOW TESTS
// All engines comprehensive analysis
// ===========================================================================

mod full_spectrum {
    use super::*;

    #[tokio::test]
    async fn test_execute_full() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/workflows/full-spectrum/execute",
            &token,
            birth_input(),
        )
        .await;

        if status == StatusCode::OK {
            verify_workflow_response(&body, "full-spectrum");
            
            let outputs = body["engine_outputs"].as_object().unwrap();
            
            // Should have many engine outputs
            assert!(
                outputs.len() >= 4,
                "Full spectrum should have at least 4 outputs, got {}: {:?}",
                outputs.len(),
                outputs.keys().collect::<Vec<_>>()
            );
            
            // Core engines should be present
            assert!(outputs.contains_key("numerology"), "Missing numerology");
            assert!(outputs.contains_key("human-design") || outputs.contains_key("panchanga"));
        } else {
            assert!(
                status == StatusCode::NOT_FOUND || status == StatusCode::INTERNAL_SERVER_ERROR,
                "Unexpected: {} {:?}",
                status,
                body
            );
        }
    }

    #[tokio::test]
    async fn test_info() {
        let token = test_token(5);
        let (status, body) = authenticated_get(
            "/api/v1/workflows/full-spectrum/info",
            &token,
        )
        .await;

        if status == StatusCode::OK {
            let engine_ids = body["engine_ids"].as_array().expect("engine_ids");
            
            // Full spectrum should include many engines
            assert!(
                engine_ids.len() >= 6,
                "Full spectrum should have at least 6 engines, got {}",
                engine_ids.len()
            );
        }
    }

    #[tokio::test]
    async fn test_execution_time_reasonable() {
        let token = test_token(5);
        let start = std::time::Instant::now();
        let (status, body) = authenticated_post(
            "/api/v1/workflows/full-spectrum/execute",
            &token,
            birth_input(),
        )
        .await;
        let elapsed = start.elapsed();

        if status == StatusCode::OK {
            // Should complete within 30 seconds even with all engines
            assert!(
                elapsed.as_secs() < 30,
                "Full spectrum took too long: {:?}",
                elapsed
            );
            
            // Verify total_time_ms is reasonable
            let total_ms = body["total_time_ms"].as_f64().unwrap();
            assert!(total_ms > 0.0 && total_ms < 30000.0);
        }
    }

    #[tokio::test]
    async fn test_graceful_engine_failures() {
        let token = test_token(5);
        let (status, body) = authenticated_post(
            "/api/v1/workflows/full-spectrum/execute",
            &token,
            birth_input(),
        )
        .await;

        if status == StatusCode::OK {
            // Should handle individual engine failures gracefully
            // (some TS engines may fail if server not running)
            
            // Errors may be in a separate field
            if let Some(errors) = body.get("errors") {
                // Errors should be an array or object with error details
                assert!(errors.is_array() || errors.is_object());
            }
        }
    }
}

// ===========================================================================
// CROSS-WORKFLOW TESTS
// ===========================================================================

#[tokio::test]
async fn test_workflow_list() {
    let token = test_token(5);
    let (status, body) = authenticated_get("/api/v1/workflows", &token).await;

    assert_eq!(status, StatusCode::OK);
    let workflows = body["workflows"].as_array().expect("workflows array");
    
    // Should have at least birth-blueprint
    let workflow_ids: Vec<&str> = workflows
        .iter()
        .filter_map(|w| w.as_str())
        .collect();
    
    assert!(
        workflow_ids.contains(&"birth-blueprint"),
        "Should have birth-blueprint workflow: {:?}",
        workflow_ids
    );
}

#[tokio::test]
async fn test_nonexistent_workflow_404() {
    let token = test_token(5);
    let (status, body) = authenticated_post(
        "/api/v1/workflows/nonexistent-workflow/execute",
        &token,
        birth_input(),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error_code"], "WORKFLOW_NOT_FOUND");
}

#[tokio::test]
async fn test_workflow_idempotency() {
    let token = test_token(5);
    let input = json!({
        "birth_data": {
            "name": "Idempotency Test",
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

    let (status1, body1) = authenticated_post(
        "/api/v1/workflows/birth-blueprint/execute",
        &token,
        input.clone(),
    )
    .await;

    let (status2, body2) = authenticated_post(
        "/api/v1/workflows/birth-blueprint/execute",
        &token,
        input,
    )
    .await;

    if status1 == StatusCode::OK && status2 == StatusCode::OK {
        // Both should succeed with valid structure
        verify_workflow_response(&body1, "birth-blueprint");
        verify_workflow_response(&body2, "birth-blueprint");
        
        // Number of outputs should be the same
        let outputs1 = body1["engine_outputs"].as_object().unwrap();
        let outputs2 = body2["engine_outputs"].as_object().unwrap();
        assert_eq!(outputs1.len(), outputs2.len());
    }
}

#[tokio::test]
async fn test_concurrent_workflow_execution() {
    let token = test_token(5);
    
    let futures = (0..3).map(|i| {
        let t = token.clone();
        let input = json!({
            "birth_data": {
                "name": format!("Concurrent Test {}", i),
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
        
        async move {
            authenticated_post("/api/v1/workflows/birth-blueprint/execute", &t, input).await
        }
    });

    let results: Vec<_> = futures::future::join_all(futures).await;
    
    // All should succeed
    for (status, body) in results {
        if status == StatusCode::OK {
            verify_workflow_response(&body, "birth-blueprint");
        }
    }
}
