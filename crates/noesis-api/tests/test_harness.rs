mod common;

use common::test_harness::RoutingHarness;

#[tokio::test]
async fn test_harness_can_assert_single_engine_route_delegation() {
    let harness = RoutingHarness::with_probe_engines(&["panchanga"]).await;

    harness.assert_engine_calculate_routed("panchanga").await;
}

#[tokio::test]
async fn test_harness_can_assert_workflow_route_delegation() {
    let harness =
        RoutingHarness::with_probe_engines(&["numerology", "human-design", "gene-keys"]).await;

    harness
        .assert_workflow_execute_routed(
            "birth-blueprint",
            &["numerology", "human-design", "gene-keys"],
        )
        .await;
}
