//! Integration tests for rate limiting middleware

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use noesis_api::create_router;
use noesis_auth::{ApiKey, AuthService};
use noesis_cache::CacheManager;
use noesis_orchestrator::WorkflowOrchestrator;
use tower::ServiceExt;
use chrono::{Utc, Duration as ChronoDuration};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::Once;

static INIT_METRICS: Once = Once::new();

/// Build app state for testing (with proper metrics initialization)
fn build_test_app_state() -> noesis_api::AppState {
    // -- Orchestrator with engines --
    let mut orchestrator = WorkflowOrchestrator::new();
    orchestrator.register_engine(Arc::new(engine_panchanga::PanchangaEngine::new()));
    orchestrator.register_engine(Arc::new(engine_numerology::NumerologyEngine::new()));
    orchestrator.register_engine(Arc::new(engine_biorhythm::BiorhythmEngine::new()));

    // -- Cache --
    let cache = CacheManager::new(
        String::new(),        // no Redis URL in tests
        100,                  // L1: 100 MB
        Duration::from_secs(3600), // L2 TTL: 1 hour
        false,                // L3 disabled
    );

    // -- Auth --
    let jwt_secret = "test-secret-for-integration-tests".to_string();
    let auth = AuthService::new(jwt_secret);

    // -- Metrics -- initialize only once globally
    static mut METRICS: Option<Arc<noesis_metrics::NoesisMetrics>> = None;
    let metrics = unsafe {
        INIT_METRICS.call_once(|| {
            let m = noesis_metrics::NoesisMetrics::new()
                .expect("Failed to initialize metrics");
            METRICS = Some(Arc::new(m));
        });
        METRICS.as_ref().unwrap().clone()
    };

    noesis_api::AppState {
        orchestrator: Arc::new(orchestrator),
        cache: Arc::new(cache),
        auth: Arc::new(auth),
        metrics,
        startup_time: Instant::now(),
    }
}

/// Test helper to create a test API key with specific rate limit
async fn create_test_api_key(auth: &Arc<AuthService>, user_id: &str, rate_limit: u32) -> String {
    let api_key_value = format!("test-key-{}", user_id);
    
    let api_key = ApiKey {
        key: api_key_value.clone(),
        user_id: user_id.to_string(),
        tier: "test".to_string(),
        permissions: vec!["basic:access".to_string()],
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + ChronoDuration::hours(1)),
        last_used: None,
        rate_limit,
        consciousness_level: 0,
    };
    
    auth.add_api_key(api_key).await.expect("Failed to add API key");
    api_key_value
}

#[tokio::test]
async fn test_rate_limit_allows_requests_under_limit() {
    let state = build_test_app_state();
    let api_key = create_test_api_key(&state.auth, "user1", 5).await;
    let app = create_router(state);
    
    // Make 5 requests (all should succeed)
    for i in 0..5 {
        let request = Request::builder()
            .uri("/api/v1/status")
            .header("X-API-Key", &api_key)
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Request {} should succeed",
            i + 1
        );
        
        // Check rate limit headers are present
        assert!(response.headers().contains_key("X-RateLimit-Limit"));
        assert!(response.headers().contains_key("X-RateLimit-Remaining"));
        assert!(response.headers().contains_key("X-RateLimit-Reset"));
        
        let remaining = response
            .headers()
            .get("X-RateLimit-Remaining")
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<u32>()
            .unwrap();
        
        assert_eq!(remaining, 5 - (i + 1), "Remaining count should decrease");
    }
}

#[tokio::test]
async fn test_rate_limit_blocks_requests_over_limit() {
    let state = build_test_app_state();
    let api_key = create_test_api_key(&state.auth, "user2", 3).await;
    let app = create_router(state);
    
    // Make 3 requests (should all succeed)
    for _ in 0..3 {
        let request = Request::builder()
            .uri("/api/v1/status")
            .header("X-API-Key", &api_key)
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    // 4th request should be rate limited
    let request = Request::builder()
        .uri("/api/v1/status")
        .header("X-API-Key", &api_key)
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    
    assert_eq!(
        response.status(),
        StatusCode::TOO_MANY_REQUESTS,
        "Request over limit should return 429"
    );
    
    // Check rate limit headers are still present
    let headers = response.headers();
    assert!(headers.contains_key("X-RateLimit-Limit"));
    assert!(headers.contains_key("X-RateLimit-Remaining"));
    assert!(headers.contains_key("X-RateLimit-Reset"));
    
    let remaining = headers
        .get("X-RateLimit-Remaining")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(remaining, "0", "Remaining should be 0 when rate limited");
}

#[tokio::test]
async fn test_rate_limit_per_user_isolation() {
    let state = build_test_app_state();
    let api_key1 = create_test_api_key(&state.auth, "user3", 2).await;
    let api_key2 = create_test_api_key(&state.auth, "user4", 2).await;
    let app = create_router(state);
    
    // User1 makes 2 requests (reaches limit)
    for _ in 0..2 {
        let request = Request::builder()
            .uri("/api/v1/status")
            .header("X-API-Key", &api_key1)
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
    
    // User1's 3rd request should fail
    let request = Request::builder()
        .uri("/api/v1/status")
        .header("X-API-Key", &api_key1)
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    
    // User2 should still be able to make requests (independent rate limit)
    let request = Request::builder()
        .uri("/api/v1/status")
        .header("X-API-Key", &api_key2)
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "User2 should have independent rate limit"
    );
}

#[tokio::test]
async fn test_rate_limit_skips_public_routes() {
    let state = build_test_app_state();
    let app = create_router(state);
    
    // Make multiple requests to /health without authentication
    for _ in 0..10 {
        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        
        let response = app.clone().oneshot(request).await.unwrap();
        
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "Public route should not be rate limited"
        );
        
        // Public routes should not have rate limit headers
        assert!(!response.headers().contains_key("X-RateLimit-Limit"));
    }
}

#[tokio::test]
async fn test_rate_limit_response_format() {
    let state = build_test_app_state();
    let api_key = create_test_api_key(&state.auth, "user5", 1).await;
    let app = create_router(state);
    
    // First request succeeds
    let request = Request::builder()
        .uri("/api/v1/status")
        .header("X-API-Key", &api_key)
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Second request should be rate limited
    let request = Request::builder()
        .uri("/api/v1/status")
        .header("X-API-Key", &api_key)
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    
    // Parse response body to check error format
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();
    
    // Check error response structure
    assert_eq!(json["error_code"], "RATE_LIMIT_EXCEEDED");
    assert!(json["error"].as_str().unwrap().contains("Rate limit exceeded"));
    assert!(json["details"].is_object());
    assert!(json["details"]["limit"].is_number());
    assert!(json["details"]["window_seconds"].is_number());
    assert!(json["details"]["reset_at"].is_number());
}

#[tokio::test]
async fn test_rate_limit_default_100_per_minute() {
    let state = build_test_app_state();
    // Create API key with rate_limit = 0 (should use default 100)
    let api_key = create_test_api_key(&state.auth, "user6", 0).await;
    let app = create_router(state);
    
    let request = Request::builder()
        .uri("/api/v1/status")
        .header("X-API-Key", &api_key)
        .body(Body::empty())
        .unwrap();
    
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // Check that the rate limit header shows 100
    let limit = response
        .headers()
        .get("X-RateLimit-Limit")
        .unwrap()
        .to_str()
        .unwrap();
    
    assert_eq!(limit, "100", "Default rate limit should be 100");
}
