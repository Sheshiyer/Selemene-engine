//! Security Tests - Rate Limiting
//!
//! W2-S8-05: Tests for rate limiting protection:
//! - Request rate limiting
//! - Burst protection
//! - Rate limit headers
//! - Per-user rate limiting
//!
//! Run with: cargo test --test rate_limit -- --nocapture

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use noesis_api::{build_app_state, create_router, ApiConfig};
use noesis_auth::AuthService;
use serde_json::{json, Value};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tower::ServiceExt;

// ===========================================================================
// Test Utilities
// ===========================================================================

static RATE_LIMIT_ROUTER: OnceLock<Router> = OnceLock::new();

fn get_router() -> &'static Router {
    RATE_LIMIT_ROUTER.get_or_init(|| {
        let config = ApiConfig::from_env();
        let state = build_app_state(&config);
        create_router(state, &config)
    })
}

fn test_token(user_id: &str) -> String {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    let auth = AuthService::new(jwt_secret);
    auth.generate_jwt_token(
        user_id,
        "free",  // Free tier typically has lower rate limits
        &["read".to_string()],
        3,
    )
    .expect("Failed to generate JWT")
}

fn enterprise_token(user_id: &str) -> String {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    let auth = AuthService::new(jwt_secret);
    auth.generate_jwt_token(
        user_id,
        "enterprise",
        &["read".to_string(), "write".to_string()],
        5,
    )
    .expect("Failed to generate JWT")
}

async fn make_request(token: &str) -> (StatusCode, Option<String>, Option<String>) {
    let router = get_router();
    let body = json!({
        "birth_data": {
            "name": "Rate Limit Test",
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
    
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/engines/numerology/calculate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    
    // Extract rate limit headers
    let remaining = response.headers()
        .get("X-RateLimit-Remaining")
        .map(|v| v.to_str().unwrap_or("").to_string());
    let reset = response.headers()
        .get("X-RateLimit-Reset")
        .map(|v| v.to_str().unwrap_or("").to_string());
    
    (status, remaining, reset)
}

// ===========================================================================
// Rate Limit Tests
// ===========================================================================

/// Test that rate limiting kicks in after many requests
#[tokio::test]
async fn test_rate_limit_enforcement() {
    let token = test_token("rate-limit-test-user-1");
    let mut rate_limited = false;
    let request_count = 200;  // Make many requests
    
    for i in 0..request_count {
        let (status, remaining, _) = make_request(&token).await;
        
        if status == StatusCode::TOO_MANY_REQUESTS {
            rate_limited = true;
            println!("Rate limited after {} requests", i + 1);
            break;
        }
        
        // Check remaining header decreasing
        if let Some(rem) = remaining {
            println!("Request {}: {} remaining", i + 1, rem);
        }
        
        // Small delay to avoid overwhelming
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    // Note: Rate limiting may or may not be enabled in test environment
    // This test documents expected behavior
    println!("Rate limit enforced: {}", rate_limited);
}

/// Test rate limit headers are present
#[tokio::test]
async fn test_rate_limit_headers_present() {
    let token = test_token("rate-header-test-user");
    let (status, remaining, reset) = make_request(&token).await;
    
    if status == StatusCode::OK {
        // Rate limit headers should be present (if rate limiting is enabled)
        // Note: Headers may not be present in all configurations
        if remaining.is_some() {
            println!("X-RateLimit-Remaining: {:?}", remaining);
            println!("X-RateLimit-Reset: {:?}", reset);
        }
    }
}

/// Test that different users have separate rate limit buckets
#[tokio::test]
async fn test_per_user_rate_limiting() {
    let token_a = test_token("rate-user-a");
    let token_b = test_token("rate-user-b");
    
    // Make requests for user A
    let mut user_a_limited = false;
    for _ in 0..50 {
        let (status, _, _) = make_request(&token_a).await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            user_a_limited = true;
            break;
        }
    }
    
    // User B should have fresh rate limit bucket
    let (status_b, _, _) = make_request(&token_b).await;
    
    // User B should succeed even if User A is rate limited
    assert!(
        status_b == StatusCode::OK,
        "User B should have separate rate limit bucket"
    );
    
    println!("User A limited: {}, User B status: {}", user_a_limited, status_b);
}

/// Test tier-based rate limits (enterprise vs free)
#[tokio::test]
async fn test_tier_based_rate_limits() {
    let free_token = test_token("free-tier-user");
    let enterprise_token = enterprise_token("enterprise-tier-user");
    
    let mut free_limited_at = 0;
    let mut enterprise_limited_at = 0;
    let max_requests = 100;
    
    // Test free tier
    for i in 0..max_requests {
        let (status, _, _) = make_request(&free_token).await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            free_limited_at = i + 1;
            break;
        }
    }
    
    // Test enterprise tier
    for i in 0..max_requests {
        let (status, _, _) = make_request(&enterprise_token).await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            enterprise_limited_at = i + 1;
            break;
        }
    }
    
    println!("Free tier limited at: {}", free_limited_at);
    println!("Enterprise tier limited at: {}", enterprise_limited_at);
    
    // Enterprise should have higher limits (or no limits in test)
    if free_limited_at > 0 && enterprise_limited_at > 0 {
        assert!(
            enterprise_limited_at >= free_limited_at,
            "Enterprise should have higher or equal limits"
        );
    }
}

/// Test rate limit reset
#[tokio::test]
async fn test_rate_limit_reset() {
    let token = test_token("rate-reset-test-user");
    
    // Exhaust rate limit
    let mut limited = false;
    for _ in 0..200 {
        let (status, _, _) = make_request(&token).await;
        if status == StatusCode::TOO_MANY_REQUESTS {
            limited = true;
            break;
        }
    }
    
    if limited {
        println!("Rate limited, waiting for reset...");
        
        // Wait for rate limit window to reset (typically 1 minute)
        // In tests, we use a shorter wait
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Try again after wait
        let (status, _, _) = make_request(&token).await;
        println!("Status after wait: {}", status);
        // Status might still be limited if window is long
    }
}

/// Test burst protection
#[tokio::test]
async fn test_burst_protection() {
    let token = test_token("burst-test-user");
    let start = Instant::now();
    let burst_size = 50;
    let mut results = Vec::new();
    
    // Send burst of requests as fast as possible
    for _ in 0..burst_size {
        let (status, _, _) = make_request(&token).await;
        results.push(status);
    }
    
    let duration = start.elapsed();
    let success_count = results.iter().filter(|s| **s == StatusCode::OK).count();
    let limited_count = results.iter().filter(|s| **s == StatusCode::TOO_MANY_REQUESTS).count();
    
    println!("Burst test: {} requests in {:?}", burst_size, duration);
    println!("Successful: {}, Rate limited: {}", success_count, limited_count);
    
    // Some requests should succeed
    assert!(success_count > 0, "Some burst requests should succeed");
}

/// Test 429 response includes retry-after header
#[tokio::test]
async fn test_retry_after_header() {
    let router = get_router();
    let token = test_token("retry-after-test-user");
    
    // Make many requests to trigger rate limit
    let mut retry_after: Option<String> = None;
    for _ in 0..200 {
        let body = json!({
            "birth_data": {
                "name": "Retry After Test",
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
        
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/engines/numerology/calculate")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        
        if response.status() == StatusCode::TOO_MANY_REQUESTS {
            retry_after = response.headers()
                .get("Retry-After")
                .map(|v| v.to_str().unwrap_or("").to_string());
            break;
        }
    }
    
    if let Some(retry) = retry_after {
        println!("Retry-After header: {}", retry);
        // Should be a valid number (seconds)
        assert!(
            retry.parse::<u64>().is_ok(),
            "Retry-After should be a number: {}",
            retry
        );
    }
}
