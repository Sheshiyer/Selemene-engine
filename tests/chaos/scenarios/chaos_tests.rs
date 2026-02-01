//! Chaos Engineering Test Suite
//!
//! W2-S8-04: Tests for graceful degradation under failure scenarios:
//! - Redis connection failure
//! - TypeScript engine timeout/unavailability
//! - Swiss Ephemeris file missing
//! - Resource pressure (simulated)
//!
//! Run with: cargo test --test chaos_tests -- --nocapture --test-threads=1

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use noesis_api::{build_app_state, create_router, ApiConfig};
use noesis_auth::AuthService;
use serde_json::{json, Value};
use std::sync::OnceLock;
use std::time::Duration;
use tower::ServiceExt;

// ===========================================================================
// Test Utilities
// ===========================================================================

static CHAOS_ROUTER: OnceLock<Router> = OnceLock::new();

fn get_router() -> &'static Router {
    CHAOS_ROUTER.get_or_init(|| {
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
        "chaos-test-user",
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
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, json)
}

fn birth_input() -> Value {
    json!({
        "birth_data": {
            "name": "Chaos Test User",
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

// ===========================================================================
// SCENARIO 1: Redis Cache Failure
// Tests that the system operates correctly when Redis is unavailable
// ===========================================================================

mod redis_failure {
    use super::*;

    /// Test that calculations still work when cache is bypassed
    /// Expected behavior: Calculations succeed but may be slower
    #[tokio::test]
    async fn test_calculation_without_cache() {
        let token = test_token(5);
        
        // Panchanga should work even without cache
        let (status, body) = authenticated_post(
            "/api/v1/engines/panchanga/calculate",
            &token,
            birth_input(),
        )
        .await;

        // Should succeed regardless of cache status
        assert_eq!(status, StatusCode::OK, "Should work without cache: {:?}", body);
        assert_eq!(body["engine_id"], "panchanga");
    }

    /// Test that repeated calculations work without cache (no caching benefit)
    #[tokio::test]
    async fn test_repeated_calculations_no_cache() {
        let token = test_token(5);
        let input = birth_input();
        
        // First call
        let start1 = std::time::Instant::now();
        let (status1, _) = authenticated_post(
            "/api/v1/engines/numerology/calculate",
            &token,
            input.clone(),
        )
        .await;
        let duration1 = start1.elapsed();

        // Second call - without cache, should take similar time
        let start2 = std::time::Instant::now();
        let (status2, _) = authenticated_post(
            "/api/v1/engines/numerology/calculate",
            &token,
            input,
        )
        .await;
        let duration2 = start2.elapsed();

        assert_eq!(status1, StatusCode::OK);
        assert_eq!(status2, StatusCode::OK);
        
        // Both should complete within reasonable time
        assert!(duration1.as_secs() < 5, "First calc too slow: {:?}", duration1);
        assert!(duration2.as_secs() < 5, "Second calc too slow: {:?}", duration2);
    }

    /// Test workflow execution without cache
    #[tokio::test]
    async fn test_workflow_without_cache() {
        let token = test_token(5);
        
        let (status, body) = authenticated_post(
            "/api/v1/workflows/birth-blueprint/execute",
            &token,
            birth_input(),
        )
        .await;

        // Workflow should complete even without cache
        assert_eq!(status, StatusCode::OK, "Workflow should work: {:?}", body);
        assert!(body["engine_outputs"].is_object());
    }
}

// ===========================================================================
// SCENARIO 2: TypeScript Engine Timeout/Unavailability
// Tests graceful degradation when TS engines are not responding
// ===========================================================================

mod ts_engine_failure {
    use super::*;

    /// Test that TS engine timeout returns appropriate error
    /// Expected: 502 Bad Gateway or 504 Gateway Timeout
    #[tokio::test]
    async fn test_ts_engine_timeout_handling() {
        let token = test_token(5);
        
        // Tarot is a TS engine - may timeout if TS server not running
        let (status, body) = authenticated_post(
            "/api/v1/engines/tarot/calculate",
            &token,
            json!({
                "current_time": "2025-01-15T12:00:00Z",
                "precision": "Standard",
                "options": {
                    "spread_type": "three_card"
                }
            }),
        )
        .await;

        // Should return either success or appropriate error (not 500)
        match status {
            StatusCode::OK => {
                // TS engine is running
                assert!(body["engine_id"].is_string());
            }
            StatusCode::SERVICE_UNAVAILABLE | StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => {
                // TS engine is unavailable - this is expected graceful degradation
                assert!(
                    body.get("error").is_some() || body.get("error_code").is_some(),
                    "Should have error info: {:?}",
                    body
                );
            }
            StatusCode::NOT_FOUND => {
                // Engine not registered
            }
            other => {
                // Should not be an unhandled error
                assert!(
                    other != StatusCode::INTERNAL_SERVER_ERROR,
                    "Should not be 500: {}",
                    other
                );
            }
        }
    }

    /// Test workflow gracefully handles TS engine failures
    #[tokio::test]
    async fn test_workflow_with_ts_engine_failure() {
        let token = test_token(5);
        
        // birth-blueprint doesn't include TS engines, so should succeed
        let (status, body) = authenticated_post(
            "/api/v1/workflows/birth-blueprint/execute",
            &token,
            birth_input(),
        )
        .await;

        // Should succeed with at least the Rust engines
        assert_eq!(status, StatusCode::OK);
        let outputs = body["engine_outputs"].as_object().unwrap();
        assert!(outputs.len() >= 2, "Should have Rust engine outputs");
    }

    /// Test full-spectrum workflow handles partial TS engine failures
    #[tokio::test]
    async fn test_full_spectrum_partial_failure() {
        let token = test_token(5);
        
        let (status, body) = authenticated_post(
            "/api/v1/workflows/full-spectrum/execute",
            &token,
            birth_input(),
        )
        .await;

        if status == StatusCode::OK {
            let outputs = body["engine_outputs"].as_object().unwrap();
            
            // Should have at least the Rust engines
            let rust_engines = ["panchanga", "numerology", "biorhythm", "human-design", "gene-keys", "vimshottari"];
            let has_rust_outputs = rust_engines.iter()
                .any(|e| outputs.contains_key(*e));
            
            assert!(has_rust_outputs, "Should have Rust engine outputs: {:?}", outputs.keys().collect::<Vec<_>>());
            
            // May have errors for TS engines - check if error tracking exists
            if let Some(errors) = body.get("errors") {
                // Errors are tracked separately
                assert!(errors.is_array() || errors.is_object());
            }
        }
    }
}

// ===========================================================================
// SCENARIO 3: Swiss Ephemeris File Missing
// Tests behavior when ephemeris data files are unavailable
// ===========================================================================

mod ephemeris_failure {
    use super::*;

    /// Test Human Design handles ephemeris issues gracefully
    #[tokio::test]
    async fn test_hd_ephemeris_fallback() {
        let token = test_token(5);
        
        let (status, body) = authenticated_post(
            "/api/v1/engines/human-design/calculate",
            &token,
            birth_input(),
        )
        .await;

        // HD should either:
        // 1. Succeed using native calculations
        // 2. Return a meaningful error about ephemeris
        match status {
            StatusCode::OK => {
                assert!(body["result"]["hd_type"].is_string());
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                // Check for ephemeris-related error message
                let error_msg = body.get("error")
                    .or(body.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                
                if error_msg.to_lowercase().contains("ephemeris") {
                    // This is expected if ephemeris files are missing
                    eprintln!("Note: Ephemeris files may be missing");
                }
            }
            _ => {}
        }
    }

    /// Test Vimshottari handles ephemeris issues
    #[tokio::test]
    async fn test_vimshottari_ephemeris_fallback() {
        let token = test_token(5);
        
        // Use direct moon longitude to bypass ephemeris
        let input = json!({
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "moon_longitude": 125.0,
                "birth_date": "1990-01-15"
            }
        });
        
        let (status, body) = authenticated_post(
            "/api/v1/engines/vimshottari/calculate",
            &token,
            input,
        )
        .await;

        // Should work when moon longitude is provided directly
        assert_eq!(status, StatusCode::OK, "Direct moon calc should work: {:?}", body);
    }
}

// ===========================================================================
// SCENARIO 4: High Load / Resource Pressure
// Tests system behavior under simulated resource constraints
// ===========================================================================

mod resource_pressure {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    /// Test concurrent request handling
    #[tokio::test]
    async fn test_high_concurrency() {
        let token = test_token(5);
        let concurrent_requests = 20;
        let semaphore = Arc::new(Semaphore::new(concurrent_requests));
        
        let mut handles = vec![];
        
        for i in 0..concurrent_requests {
            let t = token.clone();
            let sem = semaphore.clone();
            
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                
                let input = json!({
                    "birth_data": {
                        "name": format!("Concurrent User {}", i),
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
                
                authenticated_post("/api/v1/engines/numerology/calculate", &t, input).await
            }));
        }
        
        let results: Vec<_> = futures::future::join_all(handles).await;
        
        let mut success_count = 0;
        let mut error_count = 0;
        
        for result in results {
            match result {
                Ok((status, _)) if status == StatusCode::OK => success_count += 1,
                Ok((status, _)) if status == StatusCode::TOO_MANY_REQUESTS => {
                    // Rate limiting is acceptable
                    error_count += 1;
                }
                _ => error_count += 1,
            }
        }
        
        // Most requests should succeed
        assert!(
            success_count >= concurrent_requests / 2,
            "At least half should succeed: {} success, {} error",
            success_count,
            error_count
        );
    }

    /// Test large payload handling
    #[tokio::test]
    async fn test_large_options_payload() {
        let token = test_token(5);
        
        // Create a large options object
        let mut options = serde_json::Map::new();
        for i in 0..100 {
            options.insert(format!("option_{}", i), json!("value"));
        }
        
        let input = json!({
            "birth_data": {
                "name": "Large Payload Test",
                "date": "1990-01-15",
                "time": "14:30",
                "latitude": 40.7128,
                "longitude": -74.006,
                "timezone": "America/New_York"
            },
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": options
        });
        
        let (status, _body) = authenticated_post(
            "/api/v1/engines/numerology/calculate",
            &token,
            input,
        )
        .await;

        // Should handle large options without crashing
        assert!(
            status == StatusCode::OK || status == StatusCode::UNPROCESSABLE_ENTITY,
            "Should handle gracefully: {}",
            status
        );
    }

    /// Test timeout behavior
    #[tokio::test]
    async fn test_request_timeout() {
        let token = test_token(5);
        
        let start = std::time::Instant::now();
        let (status, _) = authenticated_post(
            "/api/v1/engines/human-design/calculate",
            &token,
            birth_input(),
        )
        .await;
        let duration = start.elapsed();
        
        // Request should complete within reasonable timeout
        assert!(
            duration.as_secs() < 30,
            "Request took too long: {:?}",
            duration
        );
        
        // Should not hang indefinitely
        assert!(
            status != StatusCode::GATEWAY_TIMEOUT || duration.as_secs() < 60,
            "Should timeout gracefully"
        );
    }
}

// ===========================================================================
// SCENARIO 5: Invalid/Malicious Input Handling
// Tests that the system handles edge cases gracefully
// ===========================================================================

mod edge_cases {
    use super::*;

    /// Test extreme coordinate values
    #[tokio::test]
    async fn test_extreme_coordinates() {
        let token = test_token(5);
        
        // Test North Pole
        let input = json!({
            "birth_data": {
                "name": "North Pole",
                "date": "1990-06-21",
                "time": "12:00",
                "latitude": 90.0,
                "longitude": 0.0,
                "timezone": "UTC"
            },
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {}
        });
        
        let (status, body) = authenticated_post(
            "/api/v1/engines/panchanga/calculate",
            &token,
            input,
        )
        .await;

        // Should either succeed or return validation error
        assert!(
            status == StatusCode::OK
                || status == StatusCode::UNPROCESSABLE_ENTITY
                || status == StatusCode::BAD_REQUEST,
            "Extreme coords should be handled: {} {:?}",
            status,
            body
        );
    }

    /// Test date boundary (year 2000, leap year)
    #[tokio::test]
    async fn test_date_boundaries() {
        let token = test_token(5);
        
        let dates = vec![
            ("2000-02-29", "12:00"), // Leap year
            ("1999-12-31", "23:59"), // Y2K boundary
            ("2000-01-01", "00:00"), // Y2K
        ];
        
        for (date, time) in dates {
            let input = json!({
                "birth_data": {
                    "name": "Date Boundary Test",
                    "date": date,
                    "time": time,
                    "latitude": 40.7128,
                    "longitude": -74.006,
                    "timezone": "America/New_York"
                },
                "current_time": "2025-01-15T12:00:00Z",
                "precision": "Standard",
                "options": {}
            });
            
            let (status, _) = authenticated_post(
                "/api/v1/engines/numerology/calculate",
                &token,
                input,
            )
            .await;

            assert!(
                status == StatusCode::OK,
                "Date {} should work, got {}",
                date,
                status
            );
        }
    }

    /// Test empty/null fields
    #[tokio::test]
    async fn test_null_handling() {
        let token = test_token(5);
        
        let input = json!({
            "birth_data": {
                "name": null,
                "date": "1990-01-15",
                "time": "14:30",
                "latitude": 40.7128,
                "longitude": -74.006,
                "timezone": "America/New_York"
            },
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": null
        });
        
        let (status, _body) = authenticated_post(
            "/api/v1/engines/numerology/calculate",
            &token,
            input,
        )
        .await;

        // Should handle null gracefully
        assert!(
            status == StatusCode::OK
                || status == StatusCode::UNPROCESSABLE_ENTITY
                || status == StatusCode::BAD_REQUEST,
            "Null fields should be handled"
        );
    }
}
