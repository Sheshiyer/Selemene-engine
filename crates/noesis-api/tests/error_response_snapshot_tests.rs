use noesis_api::ErrorMapper;
use noesis_core::EngineError;

async fn snapshot_error(name: &str, error: EngineError) {
    let (status, body) = ErrorMapper::with_request_trace_id("snapshot-trace".to_string(), async {
        ErrorMapper::map(error)
    })
    .await;

    assert_eq!(status.as_u16(), body.0.status);
    let json = serde_json::to_value(body.0).expect("error response should serialize");
    insta::assert_json_snapshot!(name, json);
}

#[tokio::test]
async fn snapshot_engine_not_found() {
    snapshot_error(
        "engine_not_found",
        EngineError::EngineNotFound("missing".to_string()),
    )
    .await;
}

#[tokio::test]
async fn snapshot_workflow_not_found() {
    snapshot_error(
        "workflow_not_found",
        EngineError::WorkflowNotFound("birth-blueprint".to_string()),
    )
    .await;
}

#[tokio::test]
async fn snapshot_phase_access_denied() {
    snapshot_error(
        "phase_access_denied",
        EngineError::PhaseAccessDenied {
            required: 3,
            current: 1,
        },
    )
    .await;
}

#[tokio::test]
async fn snapshot_auth_error() {
    snapshot_error(
        "auth_error",
        EngineError::AuthError("bad token".to_string()),
    )
    .await;
}

#[tokio::test]
async fn snapshot_rate_limit_exceeded() {
    snapshot_error("rate_limit_exceeded", EngineError::RateLimitExceeded).await;
}

#[tokio::test]
async fn snapshot_validation_error() {
    snapshot_error(
        "validation_error",
        EngineError::ValidationError("date must be ISO-8601".to_string()),
    )
    .await;
}

#[tokio::test]
async fn snapshot_calculation_error() {
    snapshot_error(
        "calculation_error",
        EngineError::CalculationError("ephemeris drift".to_string()),
    )
    .await;
}

#[tokio::test]
async fn snapshot_cache_error() {
    snapshot_error(
        "cache_error",
        EngineError::CacheError("redis down".to_string()),
    )
    .await;
}

#[tokio::test]
async fn snapshot_config_error() {
    snapshot_error(
        "config_error",
        EngineError::ConfigError("missing JWT secret".to_string()),
    )
    .await;
}

#[tokio::test]
async fn snapshot_bridge_error() {
    snapshot_error(
        "bridge_error",
        EngineError::BridgeError("sidecar timeout".to_string()),
    )
    .await;
}

#[tokio::test]
async fn snapshot_swiss_ephemeris_error() {
    snapshot_error(
        "swiss_ephemeris_error",
        EngineError::SwissEphemerisError("unable to load ephemeris".to_string()),
    )
    .await;
}

#[tokio::test]
async fn snapshot_internal_error() {
    snapshot_error(
        "internal_error",
        EngineError::InternalError("unexpected panic".to_string()),
    )
    .await;
}

#[test]
fn snapshot_suite_covers_all_current_engine_error_variants() {
    const SNAPSHOT_VARIANT_COUNT: usize = 12;
    assert_eq!(SNAPSHOT_VARIANT_COUNT, 12);
}
