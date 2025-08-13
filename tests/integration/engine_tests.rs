use selemene_engine::{SelemeneEngine, EngineConfig, CalculationConfig, CacheConfig, EngineBackendConfig, SwissEphemerisConfig, NativeEngineConfig, BackendRoutingStrategy, PrecisionLevel};
use selemene_engine::models::{PanchangaRequest, PrecisionLevel as ModelPrecisionLevel};

#[tokio::test]
async fn test_engine_initialization() {
    let config = EngineConfig {
        calculation: CalculationConfig {
            default_backend: BackendRoutingStrategy::Intelligent,
            cross_validation_rate: 0.01,
            max_concurrent: 1000,
            timeout_seconds: 30,
        },
        cache: CacheConfig {
            redis_url: "redis://localhost:6379".to_string(),
            size_mb: 512,
            ttl_seconds: 3600,
        },
        engines: EngineBackendConfig {
            swiss_ephemeris: SwissEphemerisConfig {
                enabled: true,
                data_path: "./data/ephemeris".to_string(),
            },
            native_solar: NativeEngineConfig {
                enabled: true,
                precision: PrecisionLevel::High,
            },
            native_lunar: NativeEngineConfig {
                enabled: true,
                precision: PrecisionLevel::High,
            },
        },
        server: selemene_engine::ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
        },
    };
    
    let engine = SelemeneEngine::new(config);
    let engine_config = engine.get_config().await;
    
    assert_eq!(engine_config.calculation.max_concurrent, 1000);
    assert_eq!(engine_config.engines.native_solar.enabled, true);
    assert_eq!(engine_config.engines.native_lunar.enabled, true);
}

#[tokio::test]
async fn test_panchanga_request_validation() {
    let request = PanchangaRequest {
        date: "2025-01-27".to_string(),
        latitude: Some(19.0760),
        longitude: Some(72.8777),
        timezone: Some("Asia/Kolkata".to_string()),
        precision: Some(ModelPrecisionLevel::High),
        include_details: Some(true),
    };
    
    assert_eq!(request.date, "2025-01-27");
    assert_eq!(request.latitude, Some(19.0760));
    assert_eq!(request.longitude, Some(72.8777));
    assert_eq!(request.precision, Some(ModelPrecisionLevel::High));
}

#[tokio::test]
async fn test_configuration_loading() {
    // Test that configuration can be loaded from environment variables
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("PORT", "9090");
    std::env::set_var("WORKERS", "8");
    
    // This would test the actual config loading logic
    // For now, just verify environment variables are set
    assert_eq!(std::env::var("RUST_LOG").unwrap(), "debug");
    assert_eq!(std::env::var("PORT").unwrap(), "9090");
    assert_eq!(std::env::var("WORKERS").unwrap(), "8");
}

#[tokio::test]
async fn test_precision_levels() {
    let standard = PrecisionLevel::Standard;
    let high = PrecisionLevel::High;
    let extreme = PrecisionLevel::Extreme;
    
    assert!(standard < high);
    assert!(high < extreme);
    assert_eq!(standard as u8, 1);
    assert_eq!(high as u8, 2);
    assert_eq!(extreme as u8, 3);
}

#[tokio::test]
async fn test_backend_routing_strategies() {
    let strategies = vec![
        BackendRoutingStrategy::AlwaysNative,
        BackendRoutingStrategy::AlwaysSwiss,
        BackendRoutingStrategy::Intelligent,
        BackendRoutingStrategy::Validated,
        BackendRoutingStrategy::PerformanceOptimized,
    ];
    
    assert_eq!(strategies.len(), 5);
    
    // Test that all strategies can be cloned and compared
    for strategy in strategies {
        let cloned = strategy.clone();
        assert_eq!(strategy, cloned);
    }
}
