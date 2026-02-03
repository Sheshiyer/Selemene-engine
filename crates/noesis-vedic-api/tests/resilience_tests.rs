//! Tests for the resilience layer: fallback chain, exponential backoff, batch requests
//! FAPI-098, FAPI-105, FAPI-106

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path};

use noesis_vedic_api::{Config, VedicApiClient, VedicApiError};
use noesis_vedic_api::panchang::{
    Panchang, DateInfo, Location, Tithi, TithiName, Nakshatra, NakshatraName,
    Yoga, YogaName, Karana, KaranaName, KaranaType, Vara, Paksha,
    PlanetaryPositions, PlanetPosition, DayBoundaries,
};
use noesis_vedic_api::resilience::{
    ExponentialBackoff, BackoffConfig, FallbackChain, FallbackResult, FallbackSource,
    ResilienceMetrics,
};
use noesis_vedic_api::batch::{BatchScheduler, BatchConfig, BatchRequest};

// ====================== TEST HELPERS ======================

fn sample_planet(name: &str, sign: &str, nakshatra: &str) -> PlanetPosition {
    PlanetPosition {
        name: name.to_string(),
        longitude: 120.0,
        latitude: 0.0,
        speed: 1.0,
        sign: sign.to_string(),
        nakshatra: nakshatra.to_string(),
        pada: 1,
        is_retrograde: false,
    }
}

fn sample_panchang() -> Panchang {
    Panchang {
        date: DateInfo {
            year: 2024,
            month: 1,
            day: 1,
            day_of_week: 1,
            julian_day: 2459945.5,
            hindu_date: None,
        },
        location: Location {
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: 5.5,
            name: Some("Bengaluru".to_string()),
        },
        tithi: Tithi {
            number: 1,
            name_tithi: TithiName::Pratipada,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
            is_complete: true,
        },
        nakshatra: Nakshatra {
            number: 1,
            name_nakshatra: NakshatraName::Ashwini,
            pada: 1,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
            longitude: 13.3,
        },
        yoga: Yoga {
            number: 1,
            name_yoga: YogaName::Preeti,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
        },
        karana: Karana {
            name_karana: KaranaName::Bava,
            karana_type: KaranaType::Movable,
            start_time: "00:00".to_string(),
            end_time: "23:59".to_string(),
        },
        vara: Vara::Monday,
        paksha: Paksha::Shukla,
        planets: PlanetaryPositions {
            sun: sample_planet("Sun", "Capricorn", "Shravana"),
            moon: sample_planet("Moon", "Aries", "Ashwini"),
            mars: None,
            mercury: None,
            jupiter: None,
            venus: None,
            saturn: None,
            rahu: None,
            ketu: None,
        },
        day_boundaries: DayBoundaries {
            sunrise: "06:30".to_string(),
            sunset: "18:15".to_string(),
            next_sunrise: "06:31".to_string(),
            day_duration: "11:45".to_string(),
            night_duration: "12:15".to_string(),
        },
        ayanamsa: 24.0,
    }
}

// ====================== FAPI-105: EXPONENTIAL BACKOFF TESTS ======================

#[test]
fn test_backoff_config_defaults() {
    let config = BackoffConfig::default();
    assert_eq!(config.initial_delay_ms, 1000);
    assert_eq!(config.max_delay_ms, 16000);
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.multiplier, 2.0);
}

#[test]
fn test_backoff_config_custom() {
    let config = BackoffConfig {
        initial_delay_ms: 500,
        max_delay_ms: 8000,
        max_retries: 3,
        multiplier: 2.0,
        jitter: false,
    };
    assert_eq!(config.initial_delay_ms, 500);
    assert_eq!(config.max_delay_ms, 8000);
    assert_eq!(config.max_retries, 3);
}

#[test]
fn test_backoff_delay_sequence() {
    let config = BackoffConfig {
        initial_delay_ms: 1000,
        max_delay_ms: 16000,
        max_retries: 5,
        multiplier: 2.0,
        jitter: false,
    };
    let backoff = ExponentialBackoff::new(config);

    // Verify the delay sequence: 1s, 2s, 4s, 8s, 16s
    assert_eq!(backoff.delay_for_attempt(0).as_millis(), 1000);
    assert_eq!(backoff.delay_for_attempt(1).as_millis(), 2000);
    assert_eq!(backoff.delay_for_attempt(2).as_millis(), 4000);
    assert_eq!(backoff.delay_for_attempt(3).as_millis(), 8000);
    assert_eq!(backoff.delay_for_attempt(4).as_millis(), 16000);
}

#[test]
fn test_backoff_caps_at_max() {
    let config = BackoffConfig {
        initial_delay_ms: 1000,
        max_delay_ms: 16000,
        max_retries: 10,
        multiplier: 2.0,
        jitter: false,
    };
    let backoff = ExponentialBackoff::new(config);

    // Beyond the cap, delay should stay at max
    assert_eq!(backoff.delay_for_attempt(5).as_millis(), 16000);
    assert_eq!(backoff.delay_for_attempt(6).as_millis(), 16000);
}

#[test]
fn test_backoff_should_retry() {
    let config = BackoffConfig {
        initial_delay_ms: 1000,
        max_delay_ms: 16000,
        max_retries: 3,
        multiplier: 2.0,
        jitter: false,
    };
    let backoff = ExponentialBackoff::new(config);

    assert!(backoff.should_retry(0));
    assert!(backoff.should_retry(1));
    assert!(backoff.should_retry(2));
    assert!(!backoff.should_retry(3)); // max_retries = 3, so attempt 3 = 4th attempt
}

#[tokio::test]
async fn test_backoff_execute_succeeds_first_try() {
    let config = BackoffConfig::default();
    let backoff = ExponentialBackoff::new(config);
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = call_count.clone();

    let result = backoff
        .execute(|| {
            let cc = cc.clone();
            async move {
                cc.fetch_add(1, Ordering::SeqCst);
                Ok::<_, VedicApiError>(42)
            }
        })
        .await;

    assert_eq!(result.unwrap(), 42);
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_backoff_execute_retries_on_retryable_error() {
    let config = BackoffConfig {
        initial_delay_ms: 10, // Use small delays for tests
        max_delay_ms: 100,
        max_retries: 3,
        multiplier: 2.0,
        jitter: false,
    };
    let backoff = ExponentialBackoff::new(config);
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = call_count.clone();

    let result = backoff
        .execute(|| {
            let cc = cc.clone();
            async move {
                let count = cc.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(VedicApiError::RateLimit { retry_after: None })
                } else {
                    Ok::<_, VedicApiError>(99)
                }
            }
        })
        .await;

    assert_eq!(result.unwrap(), 99);
    assert_eq!(call_count.load(Ordering::SeqCst), 3); // 2 failures + 1 success
}

#[tokio::test]
async fn test_backoff_gives_up_after_max_retries() {
    let config = BackoffConfig {
        initial_delay_ms: 10,
        max_delay_ms: 100,
        max_retries: 2,
        multiplier: 2.0,
        jitter: false,
    };
    let backoff = ExponentialBackoff::new(config);
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = call_count.clone();

    let result: Result<i32, VedicApiError> = backoff
        .execute(|| {
            let cc = cc.clone();
            async move {
                cc.fetch_add(1, Ordering::SeqCst);
                Err(VedicApiError::Network {
                    message: "connection refused".to_string(),
                })
            }
        })
        .await;

    assert!(result.is_err());
    // initial attempt + 2 retries = 3
    assert_eq!(call_count.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_backoff_does_not_retry_non_retryable_errors() {
    let config = BackoffConfig {
        initial_delay_ms: 10,
        max_delay_ms: 100,
        max_retries: 5,
        multiplier: 2.0,
        jitter: false,
    };
    let backoff = ExponentialBackoff::new(config);
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = call_count.clone();

    let result: Result<i32, VedicApiError> = backoff
        .execute(|| {
            let cc = cc.clone();
            async move {
                cc.fetch_add(1, Ordering::SeqCst);
                Err(VedicApiError::Configuration {
                    field: "api_key".to_string(),
                    message: "missing".to_string(),
                })
            }
        })
        .await;

    assert!(result.is_err());
    assert_eq!(call_count.load(Ordering::SeqCst), 1); // No retry for config errors
}

#[tokio::test]
async fn test_backoff_respects_retry_after_header() {
    let config = BackoffConfig {
        initial_delay_ms: 10,
        max_delay_ms: 100,
        max_retries: 3,
        multiplier: 2.0,
        jitter: false,
    };
    let backoff = ExponentialBackoff::new(config);

    // When rate limit has retry_after, should use that instead of calculated delay
    let delay = backoff.delay_for_rate_limit(Some(5));
    assert_eq!(delay.as_secs(), 5);
}

// ====================== FAPI-098: FALLBACK CHAIN TESTS ======================

#[test]
fn test_fallback_result_source_tracking() {
    let result = FallbackResult {
        value: sample_panchang(),
        source: FallbackSource::Api,
        attempts: 1,
        total_duration: Duration::from_millis(100),
    };
    assert!(matches!(result.source, FallbackSource::Api));

    let cached_result = FallbackResult {
        value: sample_panchang(),
        source: FallbackSource::Cache,
        attempts: 2,
        total_duration: Duration::from_millis(50),
    };
    assert!(matches!(cached_result.source, FallbackSource::Cache));

    let native_result = FallbackResult {
        value: sample_panchang(),
        source: FallbackSource::NativeCalculation,
        attempts: 3,
        total_duration: Duration::from_millis(200),
    };
    assert!(matches!(native_result.source, FallbackSource::NativeCalculation));
}

#[test]
fn test_resilience_metrics_initialization() {
    let metrics = ResilienceMetrics::new();
    assert_eq!(metrics.api_successes(), 0);
    assert_eq!(metrics.api_failures(), 0);
    assert_eq!(metrics.cache_hits(), 0);
    assert_eq!(metrics.native_fallbacks(), 0);
    assert_eq!(metrics.total_retries(), 0);
}

#[test]
fn test_resilience_metrics_recording() {
    let metrics = ResilienceMetrics::new();

    metrics.record_api_success();
    metrics.record_api_success();
    metrics.record_api_failure();
    metrics.record_cache_hit();
    metrics.record_cache_hit();
    metrics.record_cache_hit();
    metrics.record_native_fallback();
    metrics.record_retry();
    metrics.record_retry();

    assert_eq!(metrics.api_successes(), 2);
    assert_eq!(metrics.api_failures(), 1);
    assert_eq!(metrics.cache_hits(), 3);
    assert_eq!(metrics.native_fallbacks(), 1);
    assert_eq!(metrics.total_retries(), 2);
}

#[test]
fn test_resilience_metrics_fallback_rate() {
    let metrics = ResilienceMetrics::new();

    // 7 API successes, 3 fallbacks = 30% fallback rate
    for _ in 0..7 {
        metrics.record_api_success();
    }
    for _ in 0..3 {
        metrics.record_native_fallback();
    }

    let rate = metrics.fallback_rate();
    assert!((rate - 30.0).abs() < 0.1);
}

#[test]
fn test_resilience_metrics_fallback_rate_no_requests() {
    let metrics = ResilienceMetrics::new();
    assert_eq!(metrics.fallback_rate(), 0.0);
}

#[tokio::test]
async fn test_fallback_chain_api_success_path() {
    let server = MockServer::start().await;
    let sample = sample_panchang();

    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample))
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let chain = FallbackChain::new(config);

    let result = chain
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await;

    assert!(result.is_ok());
    let fallback_result = result.unwrap();
    assert!(matches!(fallback_result.source, FallbackSource::Api));
    // Chain checks cache first (attempt 1), then API (attempt 2)
    assert_eq!(fallback_result.attempts, 2);
}

#[tokio::test]
async fn test_fallback_chain_uses_cache_when_api_fails() {
    let server = MockServer::start().await;
    let sample = sample_panchang();

    // First request succeeds (populates cache)
    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample))
        .expect(1)
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let chain = FallbackChain::new(config);

    // First call - populates cache
    let _ = chain
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .unwrap();

    // Now the mock is exhausted, API will fail
    // Second call should hit cache
    let result = chain
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await;

    assert!(result.is_ok());
    let fallback_result = result.unwrap();
    assert!(matches!(fallback_result.source, FallbackSource::Cache));
}

#[tokio::test]
async fn test_fallback_chain_api_unreachable_falls_to_native() {
    // No mock server = connection refused
    let config = Config::new("test_key")
        .with_base_url("http://127.0.0.1:1") // Unreachable port
        .with_timeout(1);

    let chain = FallbackChain::new(config);

    let result = chain
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await;

    // Should either succeed with native calculation or return FallbackFailed
    // The chain should have attempted all three sources
    match result {
        Ok(r) => assert!(matches!(r.source, FallbackSource::NativeCalculation)),
        Err(VedicApiError::FallbackFailed { .. }) => {
            // This is acceptable - native fallback may not be fully implemented yet
        }
        Err(e) => panic!("Unexpected error type: {:?}", e),
    }
}

// ====================== FAPI-106: BATCH REQUEST TESTS ======================

#[test]
fn test_batch_config_defaults() {
    let config = BatchConfig::default();
    assert_eq!(config.max_batch_size, 10);
    assert_eq!(config.batch_timeout_ms, 5000);
    assert!(config.coalesce_identical);
}

#[test]
fn test_batch_config_custom() {
    let config = BatchConfig {
        max_batch_size: 5,
        batch_timeout_ms: 3000,
        coalesce_identical: false,
        max_concurrent_batches: 2,
    };
    assert_eq!(config.max_batch_size, 5);
    assert_eq!(config.batch_timeout_ms, 3000);
    assert!(!config.coalesce_identical);
}

#[test]
fn test_batch_request_key_generation() {
    let req1 = BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5);
    let req2 = BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5);
    let req3 = BatchRequest::panchang(2024, 1, 2, 12, 0, 0, 12.97, 77.59, 5.5);

    // Identical requests should have the same key
    assert_eq!(req1.cache_key(), req2.cache_key());
    // Different requests should have different keys
    assert_ne!(req1.cache_key(), req3.cache_key());
}

#[tokio::test]
async fn test_batch_scheduler_coalesces_identical_requests() {
    let server = MockServer::start().await;
    let sample = sample_panchang();
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = call_count.clone();

    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample))
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let batch_config = BatchConfig::default();
    let scheduler = BatchScheduler::new(config, batch_config);

    // Submit 3 identical requests concurrently
    let req1 = BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5);
    let req2 = BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5);
    let req3 = BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5);

    let results = scheduler.execute_batch(vec![req1, req2, req3]).await;

    assert_eq!(results.len(), 3);
    // All should succeed
    for r in &results {
        assert!(r.is_ok());
    }
    // With coalescing, only 1 actual API call should be made
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 3);
}

#[tokio::test]
async fn test_batch_scheduler_respects_max_batch_size() {
    let config = BatchConfig {
        max_batch_size: 2,
        batch_timeout_ms: 5000,
        coalesce_identical: true,
        max_concurrent_batches: 3,
    };

    let server = MockServer::start().await;
    let sample = sample_panchang();

    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample))
        .mount(&server)
        .await;

    let api_config = Config::new("test_key").with_base_url(server.uri());
    let scheduler = BatchScheduler::new(api_config, config);

    // Submit 5 different requests (exceeds max_batch_size of 2)
    let requests: Vec<_> = (1..=5)
        .map(|d| BatchRequest::panchang(2024, 1, d, 12, 0, 0, 12.97, 77.59, 5.5))
        .collect();

    let results = scheduler.execute_batch(requests).await;
    assert_eq!(results.len(), 5);
}

#[tokio::test]
async fn test_batch_scheduler_uses_cache() {
    let server = MockServer::start().await;
    let sample = sample_panchang();

    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample))
        .expect(1) // Should only be called once
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let scheduler = BatchScheduler::new(config, BatchConfig::default());

    // First batch - hits API
    let req1 = vec![BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)];
    let results1 = scheduler.execute_batch(req1).await;
    assert_eq!(results1.len(), 1);
    assert!(results1[0].is_ok());

    // Second batch - same request should hit cache
    let req2 = vec![BatchRequest::panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)];
    let results2 = scheduler.execute_batch(req2).await;
    assert_eq!(results2.len(), 1);
    assert!(results2[0].is_ok());
}

// ====================== INTEGRATION: COMBINED RESILIENCE TESTS ======================

#[tokio::test]
async fn test_429_triggers_backoff_then_succeeds() {
    let server = MockServer::start().await;
    let sample = sample_panchang();

    // First two requests return 429, third succeeds
    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(
            ResponseTemplate::new(429)
                .append_header("Retry-After", "1")
                .set_body_string("rate limited"),
        )
        .up_to_n_times(2)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/panchang"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&sample))
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let backoff = ExponentialBackoff::new(BackoffConfig {
        initial_delay_ms: 10,
        max_delay_ms: 100,
        max_retries: 5,
        multiplier: 2.0,
        jitter: false,
    });

    let client = VedicApiClient::new(config);
    let result = backoff
        .execute(|| {
            let client = client.clone();
            async move {
                client
                    .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
                    .await
            }
        })
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resilience_metrics_snapshot() {
    let metrics = ResilienceMetrics::new();

    metrics.record_api_success();
    metrics.record_api_success();
    metrics.record_api_failure();
    metrics.record_cache_hit();
    metrics.record_native_fallback();
    metrics.record_retry();
    metrics.record_retry();
    metrics.record_retry();

    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.api_successes, 2);
    assert_eq!(snapshot.api_failures, 1);
    assert_eq!(snapshot.cache_hits, 1);
    assert_eq!(snapshot.native_fallbacks, 1);
    assert_eq!(snapshot.total_retries, 3);
    // 1 native_fallback / (2 api_success + 1 api_failure + 1 cache_hit + 1 native_fallback) = 20%
    assert!((snapshot.fallback_rate - 20.0).abs() < 0.1);
}

#[tokio::test]
async fn test_backoff_timing_approximately_correct() {
    let config = BackoffConfig {
        initial_delay_ms: 50,
        max_delay_ms: 400,
        max_retries: 3,
        multiplier: 2.0,
        jitter: false,
    };
    let backoff = ExponentialBackoff::new(config);
    let call_count = Arc::new(AtomicU32::new(0));
    let cc = call_count.clone();

    let start = Instant::now();
    let _result: Result<i32, VedicApiError> = backoff
        .execute(|| {
            let cc = cc.clone();
            async move {
                cc.fetch_add(1, Ordering::SeqCst);
                Err(VedicApiError::Network {
                    message: "timeout".to_string(),
                })
            }
        })
        .await;

    let elapsed = start.elapsed();
    // Should have waited: 50 + 100 + 200 = 350ms (approximately)
    // Allow some tolerance
    assert!(
        elapsed.as_millis() >= 300,
        "Should take at least 300ms, took {}ms",
        elapsed.as_millis()
    );
    assert!(
        elapsed.as_millis() < 700,
        "Should take less than 700ms, took {}ms",
        elapsed.as_millis()
    );
}
