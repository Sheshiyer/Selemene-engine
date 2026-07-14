//! FAPI-110: Full test suite runner for noesis-vedic-api
//!
//! This file exercises representative tests from every test category,
//! verifying that the full test suite remains healthy. Run with:
//!
//!   cargo test -p noesis-vedic-api --test full_suite
//!
//! For the complete test suite across all test files:
//!
//!   cargo test -p noesis-vedic-api --lib --tests
//!
//! For real API tests (requires FREE_ASTROLOGY_API_KEY):
//!
//!   cargo test -p noesis-vedic-api --features real-api --lib --tests

// ===========================================================================
// Category 1: Unit test smoke checks (core types and error handling)
// ===========================================================================

mod unit_smoke_tests {
    use noesis_vedic_api::config::ProviderType;
    use noesis_vedic_api::{Config, VedicApiError, VERSION};

    #[test]
    fn test_version_is_set() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_config_creation_and_defaults() {
        let config = Config::new("test-key");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.retry_count, 3);
        assert!(config.fallback_enabled);
        assert!(config.is_api_enabled());
    }

    #[test]
    fn test_provider_type_roundtrip() {
        assert_eq!("api".parse::<ProviderType>().unwrap(), ProviderType::Api);
        assert_eq!(
            "native".parse::<ProviderType>().unwrap(),
            ProviderType::Native
        );
        assert_eq!(format!("{}", ProviderType::Api), "api");
    }

    #[test]
    fn test_error_display_and_classification() {
        let network_err = VedicApiError::Network {
            message: "timeout".to_string(),
        };
        assert!(network_err.is_retryable());
        assert!(network_err.should_fallback());
        assert!(network_err.status_code().is_none());
        assert!(format!("{}", network_err).contains("timeout"));

        let rate_err = VedicApiError::RateLimit {
            retry_after: Some(60),
        };
        assert!(rate_err.is_retryable());
        assert!(rate_err.should_fallback());
        assert_eq!(rate_err.status_code(), Some(429));

        let api_err = VedicApiError::Api {
            status_code: 500,
            message: "down".to_string(),
        };
        assert!(api_err.is_retryable());
        assert!(api_err.should_fallback());

        let config_err = VedicApiError::Configuration {
            field: "key".to_string(),
            message: "missing".to_string(),
        };
        assert!(!config_err.is_retryable());
        assert!(!config_err.should_fallback());

        let parse_err = VedicApiError::Parse {
            message: "bad".to_string(),
        };
        assert!(!parse_err.is_retryable());
        assert!(!parse_err.should_fallback());

        let cb_err = VedicApiError::CircuitBreakerOpen;
        assert!(cb_err.should_fallback());
    }
}

// ===========================================================================
// Category 2: Mock data integrity checks
// ===========================================================================

mod mock_data_integrity {
    use noesis_vedic_api::mocks;
    use noesis_vedic_api::panchang::Vara;

    #[test]
    fn test_mock_panchang_valid() {
        let p = mocks::mock_panchang();
        assert_eq!(p.date.year, 2024);
        assert_eq!(p.vara, Vara::Monday);
        assert_eq!(p.tithi.name(), "Panchami");
        assert_eq!(p.nakshatra.name(), "Pushya");
    }

    #[test]
    fn test_mock_dasha_valid() {
        let d = mocks::mock_vimshottari_dasha();
        assert_eq!(d.moon_nakshatra, "Pushya");
        assert!(!d.mahadashas.is_empty());
    }

    #[test]
    fn test_mock_birth_chart_valid() {
        let c = mocks::mock_birth_chart();
        assert_eq!(c.houses.len(), 12);
        assert!(!c.planets.is_empty());
    }

    #[test]
    fn test_mock_navamsa_valid() {
        let n = mocks::mock_navamsa_chart();
        assert!(!n.navamsa_positions.is_empty());
    }

    #[test]
    fn test_mock_complete_panchang_valid() {
        let cp = mocks::mock_complete_panchang();
        assert!(cp.muhurtas.abhijit.is_some());
        assert!(!cp.hora_timings.day_horas.is_empty());
        assert!(!cp.choghadiya.day.is_empty());
    }

    #[test]
    fn test_mock_error_variants() {
        let errors = [
            mocks::mock_rate_limit_error(),
            mocks::mock_network_error(),
            mocks::mock_api_server_error(),
            mocks::mock_config_error(),
            mocks::mock_invalid_input_error(),
            mocks::mock_parse_error(),
            mocks::mock_fallback_error(),
            mocks::mock_circuit_breaker_error(),
        ];
        // Each should have a non-empty display string
        for (i, err) in errors.iter().enumerate() {
            let s = format!("{}", err);
            assert!(!s.is_empty(), "Error {} should have display text", i);
        }
    }

    #[test]
    fn test_mock_json_serialization() {
        let pj = mocks::mock_panchang_json();
        assert!(pj.contains("Pushya"));

        let dj = mocks::mock_vimshottari_dasha_json();
        assert!(dj.contains("saturn"));

        let cj = mocks::mock_birth_chart_json();
        assert!(cj.contains("aries"));
    }
}

// ===========================================================================
// Category 3: Test mocks (Shesh profile) verification
// ===========================================================================

#[cfg(any(test, feature = "test-mocks"))]
mod shesh_mock_verification {
    use noesis_vedic_api::chart::ZodiacSign;
    use noesis_vedic_api::dasha::DashaPlanet;
    use noesis_vedic_api::panchang::{Paksha, Vara};
    use noesis_vedic_api::test_mocks::{
        shesh_birth_chart, shesh_navamsa_chart, shesh_panchang, shesh_vimshottari_dasha,
        MockApiClient,
    };

    #[test]
    fn test_shesh_panchang_identity() {
        let p = shesh_panchang();
        assert_eq!(p.date.year, 1991);
        assert_eq!(p.date.month, 9);
        assert_eq!(p.date.day, 14);
        assert_eq!(p.vara, Vara::Saturday);
        assert_eq!(p.paksha, Paksha::Shukla);
        assert_eq!(p.nakshatra.name(), "Uttara Phalguni");
    }

    #[test]
    fn test_shesh_dasha_identity() {
        let d = shesh_vimshottari_dasha();
        assert_eq!(d.moon_nakshatra, "Uttara Phalguni");
        assert_eq!(d.balance.planet, DashaPlanet::Sun);
        assert_eq!(d.mahadashas.len(), 9);
        assert_eq!(d.mahadashas[0].planet, DashaPlanet::Sun);
    }

    #[test]
    fn test_shesh_chart_identity() {
        let c = shesh_birth_chart();
        assert_eq!(c.ascendant.sign, ZodiacSign::Scorpio);
        assert_eq!(c.moon.sign, ZodiacSign::Virgo);
        assert_eq!(c.planets.len(), 9);
    }

    #[test]
    fn test_shesh_navamsa_identity() {
        let n = shesh_navamsa_chart();
        assert_eq!(n.d9_lagna, ZodiacSign::Cancer);
        assert_eq!(n.navamsa_positions.len(), 9);
    }

    #[test]
    fn test_shesh_cross_consistency() {
        let p = shesh_panchang();
        let d = shesh_vimshottari_dasha();
        let c = shesh_birth_chart();

        // Moon nakshatra must match across all three.
        //
        // FIXED IN PR2: `chart_mapping::map_planets_envelope_to_birth_chart`
        // now derives Moon nakshatra/pada from sidereal longitude via
        // `engine_vimshottari::get_nakshatra_from_longitude`, so the live
        // mapper path produces the same nakshatra string as the dasha engine.
        // `shesh_birth_chart()` is still a hand-crafted mock for this test, so
        // the assertion is mock-consistent; for the live-path correctness
        // proof see `chart_mapping::tests::test_maps_planets_envelope_fixture`
        // which asserts `chart.moon.nakshatra == "Hasta"` from the captured
        // Bangalore fixture.
        assert_eq!(p.nakshatra.name(), d.moon_nakshatra);
        assert_eq!(d.moon_nakshatra, c.moon.nakshatra);
    }

    #[test]
    fn test_mock_client_all_methods() {
        let client = MockApiClient::new();
        let _ = client.get_panchang();
        let _ = client.get_vimshottari_dasha();
        let _ = client.get_birth_chart();
        let _ = client.get_navamsa_chart();
        assert_eq!(client.call_count(), 4);
    }
}

// ===========================================================================
// Category 4: Fallback calculator verification
// ===========================================================================

mod fallback_verification {
    use noesis_vedic_api::dasha::DashaLevel;
    use noesis_vedic_api::fallback::FallbackCalculator;

    #[test]
    fn test_fallback_enabled_panchang() {
        let calc = FallbackCalculator::new(true);
        let result = calc.calculate_panchang(2024, 6, 15, 12, 0, 0, 28.61, 77.23, 5.5);
        assert!(result.is_ok());
        let p = result.unwrap();
        assert!(p.tithi.number >= 1 && p.tithi.number <= 30);
    }

    #[test]
    fn test_fallback_enabled_dasha() {
        let calc = FallbackCalculator::new(true);
        let result = calc.calculate_vimshottari(
            1990,
            6,
            15,
            10,
            30,
            0,
            28.61,
            77.23,
            5.5,
            DashaLevel::Mahadasha,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().mahadashas.len(), 9);
    }

    #[test]
    fn test_fallback_enabled_birth_chart() {
        let calc = FallbackCalculator::new(true);
        let result = calc.calculate_birth_chart(1990, 6, 15, 10, 30, 0, 28.61, 77.23, 5.5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().houses.len(), 12);
    }

    #[test]
    fn test_fallback_disabled_all_fail() {
        let calc = FallbackCalculator::new(false);
        assert!(calc
            .calculate_panchang(2024, 1, 1, 12, 0, 0, 0.0, 0.0, 0.0)
            .is_err());
        assert!(calc
            .calculate_vimshottari(1990, 1, 1, 12, 0, 0, 0.0, 0.0, 0.0, DashaLevel::Mahadasha)
            .is_err());
        assert!(calc
            .calculate_birth_chart(1990, 1, 1, 12, 0, 0, 0.0, 0.0, 0.0)
            .is_err());
    }
}

// ===========================================================================
// Category 5: Circuit breaker verification
// ===========================================================================

mod circuit_breaker_verification {
    use noesis_vedic_api::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
    use std::time::Duration;

    #[test]
    fn test_lifecycle_closed_to_open_to_half_open_to_closed() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(1),
            success_threshold: 1,
            failure_window: Duration::from_secs(60),
        };
        let cb = CircuitBreaker::new(config);

        // Closed -> Open
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Open -> HalfOpen (after recovery timeout)
        std::thread::sleep(Duration::from_millis(10));
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // HalfOpen -> Closed (after success)
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}

// ===========================================================================
// Category 6: Retry logic verification
// ===========================================================================

mod retry_verification {
    use noesis_vedic_api::retry::RetryConfig;

    #[test]
    fn test_all_config_presets_are_valid() {
        let default = RetryConfig::default();
        assert!(default.max_attempts > 0);
        assert!(default.initial_delay.as_millis() > 0);

        let quick = RetryConfig::quick();
        assert!(quick.initial_delay < default.initial_delay);

        let patient = RetryConfig::patient();
        assert!(patient.max_attempts > default.max_attempts);
    }

    #[test]
    fn test_backoff_config_conversion_roundtrip() {
        let config = RetryConfig::new(4, std::time::Duration::from_millis(250));
        let backoff = config.to_backoff_config();
        assert_eq!(backoff.initial_delay_ms, 250);
        assert_eq!(backoff.max_retries, 4);
    }
}

// ===========================================================================
// Category 7: Integration endpoint smoke checks (wiremock-based)
// ===========================================================================

mod integration_smoke_tests {
    use noesis_vedic_api::dasha::DashaLevel;
    use noesis_vedic_api::{mocks, CachedVedicClient, VedicApiService};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn test_service(base_url: &str) -> VedicApiService {
        let config = mocks::mock_config(base_url);
        VedicApiService::new(CachedVedicClient::new(config))
    }

    #[tokio::test]
    async fn test_panchang_endpoint_roundtrip() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/complete-panchang"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_panchang()))
            .mount(&server)
            .await;

        let result = test_service(&server.uri())
            .panchang(2024, 1, 15, 12, 0, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dasha_endpoint_roundtrip() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/vimshottari-dasha"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_vimshottari_dasha()))
            .mount(&server)
            .await;

        let result = test_service(&server.uri())
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

    #[tokio::test]
    async fn test_birth_chart_endpoint_roundtrip() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/horoscope-chart"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_birth_chart()))
            .mount(&server)
            .await;

        let result = test_service(&server.uri())
            .birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_navamsa_endpoint_roundtrip() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/navamsa-chart"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mocks::mock_navamsa_chart()))
            .mount(&server)
            .await;

        let result = test_service(&server.uri())
            .navamsa_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
            .await;
        assert!(result.is_ok());
    }
}
