//! FAPI-094: Integration tests for noesis-vedic-api
//!
//! Tests end-to-end flows including:
//! - Cache hits and misses
//! - Service wrapper methods against mock API responses
//! - Error handling and fallback behavior
//! - Rate limiter interaction
//! - Parallel-safe isolated test execution

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use noesis_vedic_api::dasha::DashaLevel;
use noesis_vedic_api::{mocks, CachedVedicClient, Config, VedicApiError, VedicApiService};

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

// PR2 of 3: Panchang is now a pure-native facade (no HTTP). The previous
// wiremock-based tests asserted HTTP request counts and HTTP-status error
// propagation against a `/panchang` endpoint that no longer gets called.
// They've been replaced with native-call tests that exercise the same
// public methods (`service.panchang(...)`, `client.get_panchang(...)`) and
// assert shape + cache behavior against deterministic inputs.
mod panchang_tests {
    use super::*;

    /// A `MockServer` is no longer involved — but we still pass a dummy
    /// base_url through `mock_config` so the wider service plumbing stays
    /// identical.
    fn native_service() -> VedicApiService {
        test_service("http://localhost:0")
    }
    fn native_client() -> CachedVedicClient {
        test_client("http://localhost:0")
    }

    #[tokio::test]
    async fn test_panchang_native_returns_valid_shape() {
        let service = native_service();
        let panchang = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await
            .expect("native panchang must succeed");
        assert_eq!(panchang.date.year, 2024);
        assert_eq!(panchang.date.month, 1);
        assert_eq!(panchang.date.day, 15);
        assert!(panchang.tithi.number >= 1 && panchang.tithi.number <= 30);
        assert!(panchang.nakshatra.number >= 1 && panchang.nakshatra.number <= 27);
    }

    #[tokio::test]
    async fn test_panchang_cache_hit_avoids_recompute() {
        let client = native_client();

        // First call: cache miss
        let r1 = client
            .get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(r1.is_ok());

        // Second call (same date + location, different time): cache hit
        let r2 = client
            .get_panchang(2024, 1, 15, 14, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(r2.is_ok());

        let stats = client.cache_stats().await;
        assert_eq!(stats.hits, 1, "Should have 1 cache hit");
        assert_eq!(stats.misses, 1, "Should have 1 cache miss");
        assert_eq!(stats.panchang_entries, 1, "Should have 1 panchang cached");
    }

    #[tokio::test]
    async fn test_panchang_different_dates_produce_separate_cache_entries() {
        let client = native_client();

        let _ = client
            .get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        let _ = client
            .get_panchang(2024, 1, 16, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        let stats = client.cache_stats().await;
        assert_eq!(stats.panchang_entries, 2);
        assert_eq!(stats.misses, 2);
    }

    #[tokio::test]
    async fn test_panchang_invalid_date_returns_input_error() {
        let service = native_service();
        // Month 13 is invalid -> InvalidInput from compute_panchang_native.
        let result = service
            .panchang(2024, 13, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_err());
    }
}

// ===========================================================================
// Module: Vimshottari Dasha endpoint tests
// ===========================================================================

// PR2 of 3: Vimshottari Dasha is now a pure-native facade (no HTTP). The
// previous wiremock-based tests have been replaced with native-call tests
// that exercise the same public methods against deterministic inputs.
mod dasha_tests {
    use super::*;

    fn native_service() -> VedicApiService {
        test_service("http://localhost:0")
    }
    fn native_client() -> CachedVedicClient {
        test_client("http://localhost:0")
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_dasha_native_returns_valid_shape() {
        let service = native_service();
        let dasha = service
            .vimshottari_dasha(
                1991,
                8,
                13,
                13,
                31,
                0,
                12.9716,
                77.5946,
                5.5,
                DashaLevel::Mahadasha,
            )
            .await
            .expect("native vimshottari must succeed");
        assert!(!dasha.moon_nakshatra.is_empty());
        assert!(!dasha.mahadashas.is_empty());
        assert!(dasha.balance.total_period_years > 0.0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_dasha_cache_hit_avoids_recompute() {
        let client = native_client();

        let r1 = client
            .get_vimshottari_dasha(
                1991,
                8,
                13,
                13,
                31,
                0,
                12.9716,
                77.5946,
                5.5,
                DashaLevel::Mahadasha,
            )
            .await;
        assert!(r1.is_ok());

        let r2 = client
            .get_vimshottari_dasha(
                1991,
                8,
                13,
                13,
                31,
                0,
                12.9716,
                77.5946,
                5.5,
                DashaLevel::Mahadasha,
            )
            .await;
        assert!(r2.is_ok());

        let stats = client.cache_stats().await;
        assert_eq!(stats.dasha_entries, 1);
        assert_eq!(stats.hits, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_dasha_different_levels_produce_separate_cache_entries() {
        let client = native_client();

        let _ = client
            .get_vimshottari_dasha(
                1991,
                8,
                13,
                13,
                31,
                0,
                12.9716,
                77.5946,
                5.5,
                DashaLevel::Mahadasha,
            )
            .await;
        let _ = client
            .get_vimshottari_dasha(
                1991,
                8,
                13,
                13,
                31,
                0,
                12.9716,
                77.5946,
                5.5,
                DashaLevel::Antardasha,
            )
            .await;

        let stats = client.cache_stats().await;
        assert_eq!(stats.dasha_entries, 2);
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
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_birth_chart()))
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(
            result.is_ok(),
            "Birth chart fetch should succeed: {:?}",
            result.err()
        );
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
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_birth_chart()))
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
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_navamsa_chart()))
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .navamsa_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(
            result.is_ok(),
            "Navamsa chart should succeed: {:?}",
            result.err()
        );
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
    async fn test_separate_clients_each_have_isolated_caches() {
        // PR2: panchang is native; the original test asserted two separate
        // wiremock hits. Here we verify each client maintains an isolated
        // cache (one entry each after the same call).
        let client1 = test_client("http://localhost:0");
        let r1 = client1
            .get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(r1.is_ok());

        let client2 = test_client("http://localhost:0");
        let r2 = client2
            .get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(r2.is_ok());

        let stats1 = client1.cache_stats().await;
        let stats2 = client2.cache_stats().await;
        assert_eq!(stats1.panchang_entries, 1);
        assert_eq!(stats2.panchang_entries, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_cache_stats_accumulate_correctly() {
        // PR2: panchang + dasha are native (no HTTP). birth_chart still
        // hits HTTP, but the test fails before reaching it because birth
        // chart code is unchanged from PR1. We exercise miss-then-hit on
        // each of the three cached methods using a localhost dummy URL —
        // panchang and dasha will succeed; birth_chart will fail (caught
        // by the leading `let _ =` patterns).
        let client = test_client("http://localhost:0");

        // Miss x2 (panchang + dasha) — birth_chart still hits HTTP and is
        // not exercised here in the native test variant.
        let _ = client
            .get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        let _ = client
            .get_vimshottari_dasha(
                1991,
                8,
                13,
                13,
                31,
                0,
                12.9716,
                77.5946,
                5.5,
                DashaLevel::Mahadasha,
            )
            .await;

        // Hit x2
        let _ = client
            .get_panchang(2024, 1, 15, 14, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        let _ = client
            .get_vimshottari_dasha(
                1991,
                8,
                13,
                13,
                31,
                0,
                12.9716,
                77.5946,
                5.5,
                DashaLevel::Mahadasha,
            )
            .await;

        let stats = client.cache_stats().await;
        assert_eq!(stats.misses, 2, "Should have 2 misses");
        assert_eq!(stats.hits, 2, "Should have 2 hits");
        assert_eq!(stats.total, 4);
        assert!(
            (stats.hit_rate - 50.0).abs() < 0.1,
            "Hit rate should be 50%"
        );
        assert_eq!(stats.panchang_entries, 1);
        assert_eq!(stats.dasha_entries, 1);
    }

    #[tokio::test]
    async fn test_cache_isolation_between_clients() {
        // PR2: panchang is native — no wiremock required.
        let client1 = test_client("http://localhost:0");
        let client2 = test_client("http://localhost:0");

        let _ = client1
            .get_panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        // Client2 should not see Client1's cache
        let stats2 = client2.cache_stats().await;
        assert_eq!(
            stats2.panchang_entries, 0,
            "Client2 should have empty cache"
        );
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
            rate_limit: noesis_vedic_api::rate_limit::RateLimitConfig::default(),
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
        assert!(VedicApiError::Network {
            message: "timeout".to_string()
        }
        .is_retryable());
        assert!(VedicApiError::RateLimit { retry_after: None }.is_retryable());
        assert!(VedicApiError::Api {
            status_code: 502,
            message: "bad gw".to_string()
        }
        .is_retryable());
        assert!(!VedicApiError::Api {
            status_code: 400,
            message: "bad req".to_string()
        }
        .is_retryable());
        assert!(!VedicApiError::Configuration {
            field: "k".to_string(),
            message: "m".to_string()
        }
        .is_retryable());
        assert!(!VedicApiError::Parse {
            message: "bad json".to_string()
        }
        .is_retryable());
    }

    #[test]
    fn test_error_fallback_classification() {
        assert!(VedicApiError::Network {
            message: "timeout".to_string()
        }
        .should_fallback());
        assert!(VedicApiError::CircuitBreakerOpen.should_fallback());
        assert!(VedicApiError::RateLimit {
            retry_after: Some(60)
        }
        .should_fallback());
        assert!(VedicApiError::Api {
            status_code: 503,
            message: "down".to_string()
        }
        .should_fallback());
        assert!(!VedicApiError::Api {
            status_code: 400,
            message: "bad".to_string()
        }
        .should_fallback());
        assert!(!VedicApiError::Configuration {
            field: "x".to_string(),
            message: "y".to_string()
        }
        .should_fallback());
        assert!(!VedicApiError::InvalidInput {
            field: "lat".to_string(),
            message: "bad".to_string()
        }
        .should_fallback());
    }

    #[test]
    fn test_error_status_code() {
        assert_eq!(
            VedicApiError::Api {
                status_code: 404,
                message: "not found".to_string()
            }
            .status_code(),
            Some(404)
        );
        assert_eq!(
            VedicApiError::RateLimit { retry_after: None }.status_code(),
            Some(429)
        );
        assert_eq!(
            VedicApiError::Network {
                message: "timeout".to_string()
            }
            .status_code(),
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
            api_error: Box::new(VedicApiError::RateLimit {
                retry_after: Some(3600),
            }),
            native_error: "Not implemented".to_string(),
        };
        let display = format!("{}", err);
        assert!(display.contains("Fallback"));
        assert!(display.contains("Not implemented"));
        assert!(display.contains("Rate limit"));
    }

    #[tokio::test]
    async fn test_panchang_invalid_input_returns_input_error() {
        // PR2: native panchang — no JSON parse path. Invalid input
        // (impossible date) surfaces as a structural error from
        // `compute_panchang_native`.
        let service = test_service("http://localhost:0");
        let result = service
            .panchang(2024, 13, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_err());
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
        // PR2: native facade — no HTTP, no wiremock.
        let service = test_service("http://localhost:0");
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
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_birth_chart()))
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
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_navamsa_chart()))
            .expect(1)
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let result = service
            .navamsa_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_service_dasha_delegates() {
        // PR2: native facade — no HTTP, no wiremock.
        let service = test_service("http://localhost:0");
        let result = service
            .vimshottari_dasha(
                1991,
                8,
                13,
                13,
                31,
                0,
                12.9716,
                77.5946,
                5.5,
                DashaLevel::Mahadasha,
            )
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

// ===========================================================================
// Module: Circuit breaker integration tests
// ===========================================================================

mod circuit_breaker_tests {
    use noesis_vedic_api::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
    use std::time::Duration;

    #[test]
    fn test_circuit_starts_closed() {
        let cb = CircuitBreaker::with_defaults();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_opens_after_threshold_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        // Record 3 failures to trip the breaker
        for _ in 0..3 {
            cb.record_failure();
        }
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request(), "Open breaker should reject requests");
    }

    #[test]
    fn test_circuit_success_resets_failure_count() {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        cb.record_failure();
        // Success should reset the failure counter
        cb.record_success();
        // Two more failures should NOT open (count was reset)
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_manual_reset_from_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_secs(300),
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_half_open_after_recovery_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(1), // Instant recovery for testing
            success_threshold: 1,
            failure_window: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Wait for recovery timeout
        std::thread::sleep(Duration::from_millis(10));

        // allow_request should transition to HalfOpen
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_half_open_success_closes() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(1),
            success_threshold: 1,
            failure_window: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        std::thread::sleep(Duration::from_millis(10));
        cb.allow_request(); // transitions to HalfOpen
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_half_open_failure_reopens() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(1),
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        std::thread::sleep(Duration::from_millis(10));
        cb.allow_request(); // HalfOpen

        cb.record_failure(); // Should reopen
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_metrics_tracking() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_secs(300),
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.allow_request();
        cb.allow_request();
        cb.record_failure();
        cb.record_failure(); // Opens circuit
        cb.allow_request(); // Rejected

        let metrics = cb.metrics();
        assert_eq!(metrics.total_calls, 3); // 2 allowed + 1 rejected
        assert_eq!(metrics.total_failures, 2);
        assert!(metrics.total_rejections >= 1);
        assert_eq!(metrics.state, CircuitState::Open);
    }
}

// ===========================================================================
// Module: Retry logic integration tests
// ===========================================================================

mod retry_logic_tests {
    use noesis_vedic_api::retry::{is_retryable, with_retry, RetryConfig};
    use noesis_vedic_api::VedicApiError;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.multiplier, 2.0);
    }

    #[test]
    fn test_retry_config_quick() {
        let config = RetryConfig::quick();
        assert_eq!(config.initial_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(1));
    }

    #[test]
    fn test_retry_config_patient() {
        let config = RetryConfig::patient();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.max_delay, Duration::from_secs(30));
    }

    #[test]
    fn test_retryable_error_classification() {
        // Retryable errors
        assert!(is_retryable(&VedicApiError::NetworkError(
            "timeout".to_string()
        )));
        assert!(is_retryable(&VedicApiError::Timeout("slow".to_string())));
        assert!(is_retryable(&VedicApiError::RateLimited {
            retry_after_seconds: Some(60)
        }));
        assert!(is_retryable(&VedicApiError::ServiceUnavailable(
            "503".to_string()
        )));

        // Non-retryable errors
        assert!(!is_retryable(&VedicApiError::InvalidInput {
            field: "lat".to_string(),
            message: "bad".to_string(),
        }));
        assert!(!is_retryable(&VedicApiError::Configuration {
            field: "key".to_string(),
            message: "missing".to_string(),
        }));
        assert!(!is_retryable(&VedicApiError::Parse {
            message: "bad json".to_string(),
        }));
    }

    #[tokio::test]
    async fn test_retry_succeeds_on_first_try() {
        let config = RetryConfig::quick();
        let result = with_retry(&config, || async { Ok::<_, VedicApiError>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_succeeds_after_transient_failure() {
        let attempts = Arc::new(AtomicU32::new(0));
        let config = RetryConfig::new(3, Duration::from_millis(10));

        let attempts_clone = attempts.clone();
        let result = with_retry(&config, || {
            let a = attempts_clone.clone();
            async move {
                let n = a.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    // Use the struct variant Network{} which is matched by is_retryable()
                    Err(VedicApiError::Network {
                        message: "transient".to_string(),
                    })
                } else {
                    Ok(99)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 99);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_gives_up_after_max_attempts() {
        let attempts = Arc::new(AtomicU32::new(0));
        let config = RetryConfig::new(2, Duration::from_millis(10));

        let attempts_clone = attempts.clone();
        let result: Result<i32, VedicApiError> = with_retry(&config, || {
            let a = attempts_clone.clone();
            async move {
                a.fetch_add(1, Ordering::SeqCst);
                // Use the struct variant Network{} which is retryable
                Err(VedicApiError::Network {
                    message: "persistent".to_string(),
                })
            }
        })
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_retry_does_not_retry_non_retryable_error() {
        let attempts = Arc::new(AtomicU32::new(0));
        let config = RetryConfig::new(5, Duration::from_millis(10));

        let attempts_clone = attempts.clone();
        let result: Result<i32, VedicApiError> = with_retry(&config, || {
            let a = attempts_clone.clone();
            async move {
                a.fetch_add(1, Ordering::SeqCst);
                Err(VedicApiError::InvalidInput {
                    field: "bad".to_string(),
                    message: "not retryable".to_string(),
                })
            }
        })
        .await;

        assert!(result.is_err());
        // Should have tried exactly once (non-retryable = no retries)
        assert_eq!(attempts.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_backoff_config_conversion() {
        let config = RetryConfig::default();
        let backoff = config.to_backoff_config();
        assert_eq!(backoff.initial_delay_ms, 500);
        assert_eq!(backoff.max_delay_ms, 10000);
        assert_eq!(backoff.max_retries, 3);
        assert_eq!(backoff.multiplier, 2.0);
        assert!(backoff.jitter);
    }
}

// ===========================================================================
// Module: Fallback-aware service method tests
// ===========================================================================

mod fallback_service_tests {
    use super::*;

    /// Create a VedicApiService with fallback enabled, pointing at a wiremock server
    fn test_service_with_fallback(base_url: &str) -> VedicApiService {
        let config = mocks::mock_config_with_fallback(base_url);
        let client = CachedVedicClient::new(config);
        VedicApiService::with_fallback(client, true)
    }

    #[tokio::test]
    async fn test_panchang_with_fallback_success_native() {
        // PR2: panchang is native; "API path" never fails over HTTP.
        let service = test_service_with_fallback("http://localhost:0");
        let result = service
            .panchang_with_fallback(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(
            result.is_ok(),
            "Native panchang must succeed: {:?}",
            result.err()
        );
        let panchang = result.unwrap();
        assert_eq!(panchang.date.year, 2024);
        assert_eq!(panchang.date.month, 1);
        assert_eq!(panchang.date.day, 15);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_vimshottari_with_fallback_success_native() {
        // PR2: vimshottari is native; "API path" never fails over HTTP.
        let service = test_service_with_fallback("http://localhost:0");
        let result = service
            .vimshottari_dasha_with_fallback(
                1991,
                9,
                14,
                9,
                30,
                0,
                12.9716,
                77.5946,
                5.5,
                DashaLevel::Mahadasha,
            )
            .await;
        assert!(
            result.is_ok(),
            "Native vimshottari must succeed: {:?}",
            result.err()
        );
        let dasha = result.unwrap();
        assert!(!dasha.mahadashas.is_empty());
    }

    #[tokio::test]
    async fn test_birth_chart_with_fallback_api_failure_uses_native() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/horoscope-chart"))
            .respond_with(ResponseTemplate::new(502).set_body_string("Bad gateway"))
            .mount(&server)
            .await;

        let service = test_service_with_fallback(&server.uri());
        let result = service
            .birth_chart_with_fallback(1991, 9, 14, 9, 30, 0, 12.9716, 77.5946, 5.5)
            .await;

        assert!(
            result.is_ok(),
            "Birth chart fallback should succeed: {:?}",
            result.err()
        );
        let chart = result.unwrap();
        assert_eq!(chart.houses.len(), 12);
        assert!(!chart.planets.is_empty());
    }

    #[tokio::test]
    async fn test_panchang_invalid_date_in_fallback_path() {
        // PR2: replaces the old 401-propagation test. With native panchang,
        // the only path that errors is structural input validation.
        let service = test_service_with_fallback("http://localhost:0");
        let result = service
            .panchang_with_fallback(2024, 13, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_err(), "Month 13 must surface an error");
    }
}

// ===========================================================================
// Module: Metrics integration tests
// ===========================================================================

mod metrics_integration_tests {
    use super::*;
    use noesis_vedic_api::metrics::NoesisMetrics;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_service_records_api_call_metrics() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()))
            .mount(&server)
            .await;

        let metrics = Arc::new(NoesisMetrics::new());
        let config = mocks::mock_config(&server.uri());
        let client = CachedVedicClient::new(config);
        let service = VedicApiService::with_metrics(client, metrics.clone());

        let result = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_ok());

        // Verify Prometheus export has our endpoint
        let export = metrics.export_prometheus().await;
        assert!(
            export.contains("panchang"),
            "Prometheus export should contain panchang metric"
        );
    }

    #[tokio::test]
    async fn test_service_records_error_metrics_on_failure() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal error"))
            .mount(&server)
            .await;

        let metrics = Arc::new(NoesisMetrics::new());
        let config = mocks::mock_config(&server.uri());
        let client = CachedVedicClient::new(config);
        let service = VedicApiService::with_metrics(client, metrics.clone());

        let _ = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        let export = metrics.export_prometheus().await;
        assert!(
            export.contains("noesis_errors_total"),
            "Should record error metrics"
        );
    }

    #[tokio::test]
    async fn test_service_metrics_json_export() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()))
            .mount(&server)
            .await;

        let metrics = Arc::new(NoesisMetrics::new());
        let config = mocks::mock_config(&server.uri());
        let client = CachedVedicClient::new(config);
        let service = VedicApiService::with_metrics(client, metrics.clone());

        let _ = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        let json = service.export_metrics_json().await;
        assert!(json.is_object(), "Metrics JSON should be an object");
    }

    #[tokio::test]
    async fn test_service_reset_metrics() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()))
            .mount(&server)
            .await;

        let metrics = Arc::new(NoesisMetrics::new());
        let config = mocks::mock_config(&server.uri());
        let client = CachedVedicClient::new(config);
        let service = VedicApiService::with_metrics(client, metrics.clone());

        let _ = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        service.reset_metrics().await;

        // After reset, the Prometheus export should reflect zero counters
        let export = metrics.export_prometheus().await;
        // It should still have the metric names, just zeroed
        assert!(export.contains("noesis_"));
    }

    #[tokio::test]
    async fn test_resilience_metrics_snapshot() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()))
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let _ = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;

        let snapshot = service.resilience_snapshot();
        assert!(snapshot.total_requests > 0 || snapshot.api_successes > 0);
    }
}

// ===========================================================================
// Module: Fallback calculator unit integration tests
// ===========================================================================

mod fallback_calculator_tests {
    use noesis_vedic_api::dasha::DashaLevel;
    use noesis_vedic_api::fallback::FallbackCalculator;

    #[test]
    fn test_fallback_panchang_happy_path() {
        let calc = FallbackCalculator::new(true);
        let result = calc.calculate_panchang(2024, 3, 20, 12, 0, 0, 12.97, 77.59, 5.5);
        assert!(result.is_ok());
        let p = result.unwrap();
        assert_eq!(p.date.year, 2024);
        assert!(p.tithi.number >= 1 && p.tithi.number <= 30);
        assert!(p.nakshatra.number >= 1 && p.nakshatra.number <= 27);
    }

    #[test]
    fn test_fallback_vimshottari_happy_path() {
        let calc = FallbackCalculator::new(true);
        let result = calc.calculate_vimshottari(
            1991,
            9,
            14,
            9,
            30,
            0,
            12.97,
            77.59,
            5.5,
            DashaLevel::Mahadasha,
        );
        assert!(result.is_ok());
        let d = result.unwrap();
        assert_eq!(d.mahadashas.len(), 9);
        assert!(!d.moon_nakshatra.is_empty());
    }

    #[test]
    fn test_fallback_birth_chart_happy_path() {
        let calc = FallbackCalculator::new(true);
        let result = calc.calculate_birth_chart(1991, 9, 14, 9, 30, 0, 12.97, 77.59, 5.5);
        assert!(result.is_ok());
        let c = result.unwrap();
        assert_eq!(c.houses.len(), 12);
        assert!(!c.planets.is_empty());
    }

    #[test]
    fn test_fallback_disabled_returns_error() {
        let calc = FallbackCalculator::new(false);
        assert!(calc
            .calculate_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
            .is_err());
        assert!(calc
            .calculate_vimshottari(1990, 1, 1, 12, 0, 0, 28.0, 77.0, 5.5, DashaLevel::Mahadasha)
            .is_err());
        assert!(calc
            .calculate_birth_chart(1990, 1, 1, 12, 0, 0, 28.0, 77.0, 5.5)
            .is_err());
    }

    #[test]
    fn test_fallback_invocation_counter() {
        let calc = FallbackCalculator::new(true);
        assert_eq!(calc.invocation_count(), 0);

        let _ = calc.calculate_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5);
        assert_eq!(calc.invocation_count(), 1);

        let _ = calc.calculate_vimshottari(
            1990,
            1,
            1,
            12,
            0,
            0,
            28.0,
            77.0,
            5.5,
            DashaLevel::Mahadasha,
        );
        assert_eq!(calc.invocation_count(), 2);

        calc.reset_count();
        assert_eq!(calc.invocation_count(), 0);
    }

    #[test]
    fn test_fallback_panchang_boundary_dates() {
        let calc = FallbackCalculator::new(true);

        // January 1st
        let r1 = calc.calculate_panchang(2024, 1, 1, 0, 0, 0, 0.0, 0.0, 0.0);
        assert!(r1.is_ok());

        // December 31st
        let r2 = calc.calculate_panchang(2024, 12, 31, 23, 59, 59, 0.0, 0.0, 0.0);
        assert!(r2.is_ok());

        // Negative timezone (Americas)
        let r3 = calc.calculate_panchang(2024, 6, 15, 12, 0, 0, 40.71, -74.01, -5.0);
        assert!(r3.is_ok());

        // Southern hemisphere
        let r4 = calc.calculate_panchang(2024, 6, 15, 12, 0, 0, -33.87, 151.21, 10.0);
        assert!(r4.is_ok());
    }
}

// ===========================================================================
// Module: Batch panchang integration tests
// ===========================================================================

mod batch_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_batch_panchang_returns_results_for_each_request() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/panchang"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()))
            .mount(&server)
            .await;

        let service = test_service(&server.uri());
        let requests = vec![
            (2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5),
            (2024, 1, 16, 12, 0, 0, 12.9716, 77.5946, 5.5),
        ];

        let results = service.batch_panchang(requests).await;
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }
}

// ===========================================================================
// Module: Mock client validation tests
// ===========================================================================

#[cfg(any(test, feature = "test-mocks"))]
mod mock_client_integration_tests {
    use noesis_vedic_api::chart::ZodiacSign;
    use noesis_vedic_api::test_mocks::{
        MockApiClient, MockResponses, SHESH_DAY, SHESH_MONTH, SHESH_YEAR,
    };

    #[test]
    fn test_mock_api_client_panchang() {
        let client = MockApiClient::new();
        let p = client.get_panchang();
        assert_eq!(p.date.year, SHESH_YEAR);
        assert_eq!(p.date.month, SHESH_MONTH);
        assert_eq!(p.date.day, SHESH_DAY);
        assert_eq!(client.call_count(), 1);
    }

    #[test]
    fn test_mock_api_client_dasha() {
        let client = MockApiClient::new();
        let d = client.get_vimshottari_dasha();
        assert_eq!(d.moon_nakshatra, "Uttara Phalguni");
        assert_eq!(d.mahadashas.len(), 9);
    }

    #[test]
    fn test_mock_api_client_birth_chart() {
        let client = MockApiClient::new();
        let c = client.get_birth_chart();
        assert_eq!(c.ascendant.sign, ZodiacSign::Scorpio);
        assert_eq!(c.planets.len(), 9);
    }

    #[test]
    fn test_mock_api_client_navamsa() {
        let client = MockApiClient::new();
        let n = client.get_navamsa_chart();
        assert_eq!(n.d9_lagna, ZodiacSign::Cancer);
    }

    #[test]
    fn test_mock_responses_json_roundtrip() {
        // Panchang
        let panchang_val = MockResponses::panchang_response();
        assert!(panchang_val.is_object());
        let panchang_str = MockResponses::panchang_json();
        assert!(panchang_str.contains("1991"));

        // Dasha
        let dasha_val = MockResponses::vimshottari_response();
        assert!(dasha_val.is_object());

        // Birth chart
        let chart_val = MockResponses::birth_chart_response();
        assert!(chart_val.is_object());

        // Navamsa
        let navamsa_val = MockResponses::navamsa_response();
        assert!(navamsa_val.is_object());
    }

    #[test]
    fn test_mock_api_client_call_count_reset() {
        let client = MockApiClient::new();
        let _ = client.get_panchang();
        let _ = client.get_birth_chart();
        let _ = client.get_vimshottari_dasha();
        let _ = client.get_navamsa_chart();
        assert_eq!(client.call_count(), 4);

        client.reset_count();
        assert_eq!(client.call_count(), 0);
    }
}

// ===========================================================================
// Module: Resilience chain integration tests
// ===========================================================================

mod resilience_chain_tests {
    use noesis_vedic_api::resilience::{BackoffConfig, ExponentialBackoff};
    use noesis_vedic_api::VedicApiError;
    use std::time::Duration;

    #[test]
    fn test_backoff_delay_increases_exponentially() {
        let config = BackoffConfig {
            initial_delay_ms: 100,
            max_delay_ms: 10000,
            max_retries: 5,
            multiplier: 2.0,
            jitter: false,
        };
        let backoff = ExponentialBackoff::new(config);

        let d0 = backoff.delay_for_attempt(0);
        let d1 = backoff.delay_for_attempt(1);
        let d2 = backoff.delay_for_attempt(2);

        assert_eq!(d0, Duration::from_millis(100));
        assert_eq!(d1, Duration::from_millis(200));
        assert_eq!(d2, Duration::from_millis(400));
    }

    #[test]
    fn test_backoff_delay_capped_at_max() {
        let config = BackoffConfig {
            initial_delay_ms: 100,
            max_delay_ms: 300,
            max_retries: 10,
            multiplier: 2.0,
            jitter: false,
        };
        let backoff = ExponentialBackoff::new(config);

        let d3 = backoff.delay_for_attempt(5);
        assert_eq!(d3, Duration::from_millis(300));
    }

    #[test]
    fn test_backoff_respects_retry_after_header() {
        let config = BackoffConfig::default();
        let backoff = ExponentialBackoff::new(config);

        let delay = backoff.delay_for_rate_limit(Some(60));
        assert_eq!(delay, Duration::from_secs(60));

        // Without retry-after, falls back to attempt 0 delay
        let delay_no_header = backoff.delay_for_rate_limit(None);
        assert!(delay_no_header >= Duration::from_millis(1000));
    }

    #[test]
    fn test_backoff_should_retry_respects_max() {
        let config = BackoffConfig {
            max_retries: 3,
            ..Default::default()
        };
        let backoff = ExponentialBackoff::new(config);

        assert!(backoff.should_retry(0));
        assert!(backoff.should_retry(2));
        assert!(!backoff.should_retry(3));
        assert!(!backoff.should_retry(10));
    }

    #[tokio::test]
    async fn test_backoff_execute_non_retryable_fails_immediately() {
        let config = BackoffConfig {
            max_retries: 5,
            initial_delay_ms: 10,
            ..Default::default()
        };
        let backoff = ExponentialBackoff::new(config);

        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result: Result<(), VedicApiError> = backoff
            .execute(|| {
                let a = attempts_clone.clone();
                async move {
                    a.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Err(VedicApiError::Configuration {
                        field: "key".to_string(),
                        message: "missing".to_string(),
                    })
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(
            attempts.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "Non-retryable error should not trigger retries"
        );
    }
}

// ===========================================================================
// Module: Real API tests (require FREE_ASTROLOGY_API_KEY in environment)
// ===========================================================================

/// These tests only compile and run when the `real-api` feature is enabled.
/// They require a valid API key in the `FREE_ASTROLOGY_API_KEY` environment variable.
///
/// Run with: `cargo test --features real-api -p noesis-vedic-api --test integration_tests real_api_tests`
#[cfg(feature = "real-api")]
mod real_api_tests {
    use noesis_vedic_api::dasha::DashaLevel;
    use noesis_vedic_api::{CachedVedicClient, Config, VedicApiService};

    fn create_real_service() -> VedicApiService {
        let config =
            Config::from_env().expect("FREE_ASTROLOGY_API_KEY must be set for real-api tests");
        VedicApiService::new(CachedVedicClient::new(config))
    }

    #[tokio::test]
    async fn test_real_api_panchang() {
        let service = create_real_service();
        let result = service
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(
            result.is_ok(),
            "Real API panchang should succeed: {:?}",
            result.err()
        );
        let p = result.unwrap();
        assert_eq!(p.date.year, 2024);
    }

    #[tokio::test]
    async fn test_real_api_vimshottari_dasha() {
        let service = create_real_service();
        let result = service
            .vimshottari_dasha(
                1991,
                9,
                14,
                9,
                30,
                0,
                12.9716,
                77.5946,
                5.5,
                DashaLevel::Mahadasha,
            )
            .await;
        assert!(
            result.is_ok(),
            "Real API dasha should succeed: {:?}",
            result.err()
        );
        let d = result.unwrap();
        assert!(!d.mahadashas.is_empty());
    }

    #[tokio::test]
    async fn test_real_api_birth_chart() {
        let service = create_real_service();
        let result = service
            .birth_chart(1991, 9, 14, 9, 30, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(
            result.is_ok(),
            "Real API birth chart should succeed: {:?}",
            result.err()
        );
        let c = result.unwrap();
        assert!(!c.planets.is_empty());
        assert_eq!(c.houses.len(), 12);
    }
}
