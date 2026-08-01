mod common;

use common::test_harness::RoutingHarness;
use noesis_api::{build_app_state_lazy_db, ApiConfig};

#[tokio::test]
async fn test_harness_can_assert_single_engine_route_delegation() {
    let harness = RoutingHarness::with_probe_engines(&["panchanga"]).await;

    harness.assert_engine_calculate_routed("panchanga").await;
}

#[tokio::test]
async fn test_harness_can_assert_workflow_route_delegation() {
    // The canonical birth-blueprint engine set, per WorkflowRegistry and
    // docs/baseline/workflow-parity.json. A second, divergent definition used
    // to live in WorkflowOrchestrator::default_workflows() and listed
    // gene-keys instead; consolidating onto the registry left this assertion
    // behind.
    let engine_ids = [
        "numerology",
        "human-design",
        "vimshottari",
        "biofield",
        "face-reading",
    ];

    let harness = RoutingHarness::with_probe_engines(&engine_ids).await;

    harness
        .assert_workflow_execute_routed("birth-blueprint", &engine_ids)
        .await;
}

#[tokio::test]
async fn test_harness_builds_lazy_app_state_with_biofield_repository_when_database_url_is_configured(
) {
    let config = ApiConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        jwt_secret: common::TEST_JWT_SECRET.to_string(),
        database_url: Some(
            "postgresql://noesis_user:noesis_password@localhost:5432/noesis".to_string(),
        ),
        redis_url: None,
        allowed_origins: vec![],
        rate_limit_requests: 100,
        rate_limit_window_secs: 60,
        request_timeout_secs: 30,
        log_level: "info".to_string(),
        log_format: "pretty".to_string(),
        cf_access_issuer: None,
        cf_access_audience: None,
        cf_dev_bypass_token: None,
        dodo_payments_api_key: None,
        dodo_payments_webhook_key: None,
        dodo_payments_env: None,
        python_biofield_url: "http://localhost:8002".to_string(),
        python_biofield_timeout_ms: 10_000,
        gateway_url: None,
        gateway_token: None,
    };

    let state = build_app_state_lazy_db(&config).await;

    assert!(state.biofield_repository.is_some());
}
