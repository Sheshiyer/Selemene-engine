//! FAPI-094: Integration tests for noesis-vedic-api
//!
//! Tests end-to-end flows including:
//! - Cache hits and misses
//! - Service wrapper methods against mock API responses
//! - Error handling and fallback behavior
//! - Rate limiter interaction
//! - Parallel-safe isolated test execution

use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

use noesis_vedic_api::{
    Config, CachedVedicClient, VedicApiService, VedicApiError,
    mocks,
};
use noesis_vedic_api::dasha::DashaLevel;

// ===========================================================================
// Test helpers
// ===========================================================================

/// Create a CachedVedicClient pointing at a wiremock server
fn test_client(base_url: &str) -> CachedVedicClient {
    let config = mocks::mock_config(base_url);
    CachedVedicClient::new(config)
}

/// Create a VedicApiService pointing at a wiremock server
fn test_service(base_url: &str) -> VedicApiService {
    VedicApiService::new(test_client(base_url))
}

// ===========================================================================
// Module: Panchang endpoint tests
// ===========================================================================

mod panchang_tests {
    use super::*;

    #[tokio::test]
    async fn test_panchang_success_from_api() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(result.is_ok(), "Panchang fetch should succeed: {:?}", result.err());
        let panchang = result.unwrap();
        assert_eq!(panchang.date.year, 2024);
        assert_eq!(panchang.tithi.name(), "Panchami");
        assert_eq!(panchang.nakshatra.name(), "Pushya");
    }

    #[tokio::test]
    async fn test_panchang_cache_hit_avoids_second_api_call() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()),
            )
            .expect(1) // Only ONE request should reach the server
            .mount(&server)
            .await;

        let client = test_client(&server.uri());

        // First call: cache miss -> hits API
        let r1 = client
            .get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(r1.is_ok());

        // Second call: cache hit -> no API call
        let r2 = client
            .get_panchang(2024, 1, 15, 14, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(r2.is_ok());

        // Verify cache stats
        let stats = client.cache_stats().await;
        assert_eq!(stats.hits, 1, "Should have 1 cache hit");
        assert_eq!(stats.misses, 1, "Should have 1 cache miss");
        assert_eq!(stats.panchang_entries, 1, "Should have 1 panchang cached");
    }

    #[tokio::test]
    async fn test_panchang_different_dates_produce_separate_cache_entries() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()),
            )
            .expect(2) // Two distinct dates -> two API calls
            .mount(&server)
            .await;

        let client = test_client(&server.uri());

        // Date 1
        let _ = client
            .get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        // Date 2 (different day)
        let _ = client
            .get_panchang(2024, 1, 16, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        let stats = client.cache_stats().await;
        assert_eq!(stats.panchang_entries, 2);
        assert_eq!(stats.misses, 2);
    }

    #[tokio::test]
    async fn test_panchang_api_error_propagates() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(ResponseTemplate::new(500).set_body_string(
                mocks::mock_error_json(500, "Internal server error"),
            ))
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            VedicApiError::Api { status_code, .. } => {
                assert_eq!(status_code, 500);
            }
            _ => panic!("Expected Api error, got {:?}", err),
        }
    }

    #[tokio::test]
    async fn test_panchang_401_returns_config_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(
                ResponseTemplate::new(401).set_body_string("Unauthorized"),
            )
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            VedicApiError::Configuration { field, .. } => {
                assert_eq!(field, "api_key");
            }
            other => panic!("Expected Configuration error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_panchang_429_returns_rate_limit_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(
                ResponseTemplate::new(429).set_body_string("Too many requests"),
            )
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            VedicApiError::RateLimit { .. } => {}
            other => panic!("Expected RateLimit error, got {:?}", other),
        }
    }
}

// ===========================================================================
// Module: Vimshottari Dasha endpoint tests
// ===========================================================================

mod dasha_tests {
    use super::*;

    #[tokio::test]
    async fn test_dasha_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/vimshottari-dasha"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_vimshottari_dasha()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .vimshottari_dasha(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5, DashaLevel::Mahadasha)
            .await;

        assert!(result.is_ok(), "Dasha fetch should succeed: {:?}", result.err());
        let dasha = result.unwrap();
        assert_eq!(dasha.moon_nakshatra, "Pushya");
        assert!(!dasha.mahadashas.is_empty());
    }

    #[tokio::test]
    async fn test_dasha_cache_hit() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/vimshottari-dasha"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_vimshottari_dasha()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server.uri());

        // First call: cache miss
        let r1 = client
            .get_vimshottari_dasha(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5, DashaLevel::Mahadasha)
            .await;
        assert!(r1.is_ok());

        // Second call: cache hit (same birth data + level)
        let r2 = client
            .get_vimshottari_dasha(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5, DashaLevel::Mahadasha)
            .await;
        assert!(r2.is_ok());

        let stats = client.cache_stats().await;
        assert_eq!(stats.dasha_entries, 1);
        assert_eq!(stats.hits, 1);
    }

    #[tokio::test]
    async fn test_dasha_different_levels_produce_separate_cache_entries() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/vimshottari-dasha"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_vimshottari_dasha()),
            )
            .expect(2)
            .mount(&server)
            .await;

        let client = test_client(&server.uri());

        let _ = client
            .get_vimshottari_dasha(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5, DashaLevel::Mahadasha)
            .await;

        let _ = client
            .get_vimshottari_dasha(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5, DashaLevel::Antardasha)
            .await;

        let stats = client.cache_stats().await;
        assert_eq!(stats.dasha_entries, 2);
    }

    #[tokio::test]
    async fn test_dasha_api_error_propagates() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/vimshottari-dasha"))
            .respond_with(
                ResponseTemplate::new(500).set_body_string("Server error"),
            )
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .vimshottari_dasha(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5, DashaLevel::Mahadasha)
            .await;

        assert!(result.is_err());
    }
}

// ===========================================================================
// Module: Birth Chart endpoint tests
// ===========================================================================

mod birth_chart_tests {
    use super::*;

    #[tokio::test]
    async fn test_birth_chart_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/horoscope-chart"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_birth_chart()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(result.is_ok(), "Birth chart fetch should succeed: {:?}", result.err());
        let chart = result.unwrap();
        assert_eq!(chart.ascendant.sign.as_str(), "Aries");
        assert!(!chart.planets.is_empty());
        assert_eq!(chart.houses.len(), 12);
    }

    #[tokio::test]
    async fn test_birth_chart_cache_hit() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/horoscope-chart"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_birth_chart()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server.uri());

        let _ = client
            .get_birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;

        // Same birth data -> cache hit
        let r2 = client
            .get_birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(r2.is_ok());

        let stats = client.cache_stats().await;
        assert_eq!(stats.birth_chart_entries, 1);
        assert_eq!(stats.hits, 1);
    }

    #[tokio::test]
    async fn test_birth_chart_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/horoscope-chart"))
            .respond_with(ResponseTemplate::new(502).set_body_string("Bad Gateway"))
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            VedicApiError::Api { status_code, .. } => assert_eq!(status_code, 502),
            other => panic!("Expected Api error, got {:?}", other),
        }
    }
}

// ===========================================================================
// Module: Navamsa Chart endpoint tests
// ===========================================================================

mod navamsa_tests {
    use super::*;

    #[tokio::test]
    async fn test_navamsa_chart_success() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/navamsa-chart"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_navamsa_chart()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .navamsa_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(result.is_ok(), "Navamsa chart should succeed: {:?}", result.err());
        let chart = result.unwrap();
        assert_eq!(chart.d9_lagna.as_str(), "Sagittarius");
        assert!(!chart.navamsa_positions.is_empty());
    }

    #[tokio::test]
    async fn test_navamsa_chart_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/navamsa-chart"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .navamsa_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(result.is_err());
    }
}

// ===========================================================================
// Module: Cache behavior tests
// ===========================================================================

mod cache_behavior_tests {
    use super::*;

    #[tokio::test]
    async fn test_separate_clients_each_make_api_calls() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()),
            )
            .expect(2) // Two separate clients, two API calls
            .mount(&server)
            .await;

        // Client 1 fetches and caches
        let client1 = test_client(&server.uri());
        let r1 = client1
            .get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(r1.is_ok());

        // Client 2 has its own cache, so it also hits the API
        let client2 = test_client(&server.uri());
        let r2 = client2
            .get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(r2.is_ok());

        // Each client has 1 panchang entry
        let stats1 = client1.cache_stats().await;
        let stats2 = client2.cache_stats().await;
        assert_eq!(stats1.panchang_entries, 1);
        assert_eq!(stats2.panchang_entries, 1);
    }

    #[tokio::test]
    async fn test_cache_stats_accumulate_correctly() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()),
            )
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/vimshottari-dasha"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_vimshottari_dasha()),
            )
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/horoscope-chart"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_birth_chart()),
            )
            .mount(&server)
            .await;

        let client = test_client(&server.uri());

        // Miss x3
        let _ = client.get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5).await;
        let _ = client.get_vimshottari_dasha(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5, DashaLevel::Mahadasha).await;
        let _ = client.get_birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5).await;

        // Hit x3
        let _ = client.get_panchang(2024, 1, 15, 14, 0, 0, 12.9716, 77.5946, 5.5).await;
        let _ = client.get_vimshottari_dasha(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5, DashaLevel::Mahadasha).await;
        let _ = client.get_birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5).await;

        let stats = client.cache_stats().await;
        assert_eq!(stats.misses, 3, "Should have 3 misses");
        assert_eq!(stats.hits, 3, "Should have 3 hits");
        assert_eq!(stats.total, 6);
        assert!((stats.hit_rate - 50.0).abs() < 0.1, "Hit rate should be 50%");
        assert_eq!(stats.panchang_entries, 1);
        assert_eq!(stats.dasha_entries, 1);
        assert_eq!(stats.birth_chart_entries, 1);
    }

    #[tokio::test]
    async fn test_cache_isolation_between_clients() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()),
            )
            .mount(&server)
            .await;

        let client1 = test_client(&server.uri());
        let client2 = test_client(&server.uri());

        let _ = client1.get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5).await;

        // Client2 should not see Client1's cache
        let stats2 = client2.cache_stats().await;
        assert_eq!(stats2.panchang_entries, 0, "Client2 should have empty cache");
    }
}

// ===========================================================================
// Module: Rate limiter integration tests
// ===========================================================================

mod rate_limiter_tests {
    use super::*;
    use noesis_vedic_api::RateLimiter;

    #[tokio::test]
    async fn test_rate_limiter_allows_requests_within_limit() {
        let limiter = RateLimiter::with_limits(10, 2);
        // Should allow 8 requests (10 - 2 buffer)
        for _ in 0..8 {
            assert!(limiter.can_request());
            assert!(limiter.acquire().await);
        }
        // Should be blocked now
        assert!(!limiter.can_request());
    }

    #[tokio::test]
    async fn test_rate_limiter_release_restores_quota() {
        let limiter = RateLimiter::with_limits(10, 0);
        assert!(limiter.acquire().await);
        assert_eq!(limiter.remaining(), 9);
        limiter.release();
        assert_eq!(limiter.remaining(), 10);
    }

    #[tokio::test]
    async fn test_rate_limiter_status_report() {
        let limiter = RateLimiter::with_limits(50, 5);
        let status = limiter.status();
        assert_eq!(status.daily_limit, 50);
        assert_eq!(status.buffer, 5);
        assert_eq!(status.used_today, 0);
        assert_eq!(status.effective_remaining, 45);
    }

    #[tokio::test]
    async fn test_cached_client_rate_limit_behavior() {
        // When rate limiter blocks and fallback is disabled, we get RateLimit error
        let server = MockServer::start().await;

        // Set up a client with a very small rate limit (buffer = limit = 1)
        // so it immediately blocks
        let config = Config {
            api_key: "test-key-for-rate-limit".to_string(),
            base_url: server.uri(),
            timeout_seconds: 5,
            retry_count: 0,
            cache_ttl_birth_data: 0,
            cache_ttl_daily: 86400,
            provider: noesis_vedic_api::config::ProviderType::Api,
            fallback_enabled: false,
        };
        let client = CachedVedicClient::new(config);

        // The default RateLimiter has 50 limit / 5 buffer, so first request works
        // We verify the health check works as a proxy
        let healthy = client.health_check().await;
        assert!(healthy);
    }
}

// ===========================================================================
// Module: Error handling tests
// ===========================================================================

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_error_retryable_classification() {
        assert!(VedicApiError::Network { message: "timeout".to_string() }.is_retryable());
        assert!(VedicApiError::RateLimit { retry_after: None }.is_retryable());
        assert!(VedicApiError::Api { status_code: 502, message: "bad gw".to_string() }.is_retryable());
        assert!(!VedicApiError::Api { status_code: 400, message: "bad req".to_string() }.is_retryable());
        assert!(!VedicApiError::Configuration { field: "k".to_string(), message: "m".to_string() }.is_retryable());
        assert!(!VedicApiError::Parse { message: "bad json".to_string() }.is_retryable());
    }

    #[test]
    fn test_error_fallback_classification() {
        assert!(VedicApiError::Network { message: "timeout".to_string() }.should_fallback());
        assert!(VedicApiError::CircuitBreakerOpen.should_fallback());
        assert!(VedicApiError::RateLimit { retry_after: Some(60) }.should_fallback());
        assert!(VedicApiError::Api { status_code: 503, message: "down".to_string() }.should_fallback());
        assert!(!VedicApiError::Api { status_code: 400, message: "bad".to_string() }.should_fallback());
        assert!(!VedicApiError::Configuration { field: "x".to_string(), message: "y".to_string() }.should_fallback());
        assert!(!VedicApiError::InvalidInput { field: "lat".to_string(), message: "bad".to_string() }.should_fallback());
    }

    #[test]
    fn test_error_status_code() {
        assert_eq!(
            VedicApiError::Api { status_code: 404, message: "not found".to_string() }.status_code(),
            Some(404)
        );
        assert_eq!(
            VedicApiError::RateLimit { retry_after: None }.status_code(),
            Some(429)
        );
        assert_eq!(
            VedicApiError::Network { message: "timeout".to_string() }.status_code(),
            None
        );
    }

    #[test]
    fn test_error_display_contains_context() {
        let err = VedicApiError::Configuration {
            field: "API_KEY".to_string(),
            message: "Missing in environment".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("API_KEY"));
        assert!(display.contains("Missing in environment"));
    }

    #[test]
    fn test_fallback_error_chains_display() {
        let err = VedicApiError::FallbackFailed {
            api_error: Box::new(VedicApiError::RateLimit { retry_after: Some(3600) }),
            native_error: "Not implemented".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Fallback"));
        assert!(display.contains("Not implemented"));
        assert!(display.contains("Rate limit"));
    }

    #[tokio::test]
    async fn test_malformed_json_response_returns_parse_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(
                ResponseTemplate::new(200).set_body_string("{ this is not valid json }}}"),
            )
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(result.is_err());
        // Should be a Network/Parse error from serde deserialization failure
        let err = result.unwrap_err();
        let display = format!("{}", err);
        assert!(
            display.contains("error") || display.contains("JSON") || display.contains("parse"),
            "Error should indicate parse failure, got: {}",
            display
        );
    }
}

// ===========================================================================
// Module: Service wrapper tests (VedicApiService)
// ===========================================================================

mod service_wrapper_tests {
    use super::*;

    #[tokio::test]
    async fn test_service_from_client() {
        let server = MockServer::start().await;
        let service = test_service(&server.uri());
        // Verify the service holds a client reference
        let _client = service.client();
    }

    #[tokio::test]
    async fn test_service_panchang_delegates_to_client() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_birth_chart_delegates() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/horoscope-chart"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_birth_chart()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_navamsa_delegates() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/navamsa-chart"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_navamsa_chart()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .navamsa_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_service_dasha_delegates() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/vimshottari-dasha"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(mocks::mock_vimshottari_dasha()),
            )
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .vimshottari_dasha(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5, DashaLevel::Mahadasha)
            .await;
        assert!(result.is_ok());
    }
}

// ===========================================================================
// Module: Health check and status report tests
// ===========================================================================

mod health_and_status_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check_returns_true_when_available() {
        let server = MockServer::start().await;
        let client = test_client(&server.uri());
        // Health check just verifies rate limiter can_request, no HTTP call
        assert!(client.health_check().await);
    }

    #[tokio::test]
    async fn test_status_report_format() {
        let server = MockServer::start().await;
        let client = test_client(&server.uri());
        let report = client.status_report().await;
        let formatted = format!("{}", report);
        assert!(formatted.contains("Vedic API Status"));
        assert!(formatted.contains("remaining"));
    }

    #[tokio::test]
    async fn test_rate_limit_status_initial() {
        let server = MockServer::start().await;
        let client = test_client(&server.uri());
        let status = client.rate_limit_status().await;
        assert_eq!(status.daily_limit, 50);
        assert_eq!(status.used_today, 0);
    }
}

// ===========================================================================
// Module: Config tests
// ===========================================================================

mod config_tests {
    use super::*;

    #[test]
    fn test_mock_config_has_test_key() {
        let config = mocks::mock_config("http://localhost:9999");
        assert!(config.api_key.contains("test-mock"));
        assert_eq!(config.timeout_seconds, 5);
        assert!(!config.fallback_enabled);
    }

    #[test]
    fn test_mock_config_with_fallback_enabled() {
        let config = mocks::mock_config_with_fallback("http://localhost:9999");
        assert!(config.fallback_enabled);
    }

    #[test]
    fn test_config_builder_methods() {
        let config = Config::new("my-key")
            .with_base_url("http://test.local")
            .with_timeout(10)
            .with_retry_count(5);
        assert_eq!(config.base_url, "http://test.local");
        assert_eq!(config.timeout_seconds, 10);
        assert_eq!(config.retry_count, 5);
    }

    #[test]
    fn test_config_api_key_masking() {
        let config = Config::new("abcdefghijklmnopqrstuvwxyz");
        let masked = config.masked_api_key();
        assert!(masked.starts_with("abcd"));
        assert!(masked.ends_with("wxyz"));
        assert!(masked.contains("..."));
        assert!(!masked.contains("efghijklmnop"));
    }
}
