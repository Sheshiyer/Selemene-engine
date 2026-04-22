mod common;

use common::route_inventory::{documented_route_inventory, source_route_inventory};

#[test]
fn route_inventory_source_includes_vedic_chart_route() {
    let source = source_route_inventory();

    let chart_route = source
        .routes
        .iter()
        .find(|route| route.path == "/charts/vedic")
        .expect("source route inventory should include /charts/vedic");

    assert_eq!(chart_route.auth_requirement, "bearer_or_api_key");
    assert!(
        chart_route
            .methods
            .iter()
            .any(|method| method.method == "POST" && method.handler == "vedic_chart_handler"),
        "source chart route should publish POST via vedic_chart_handler"
    );
}

#[test]
fn route_inventory_documented_surface_includes_vedic_chart_route() {
    let documented = documented_route_inventory();

    let chart_route = documented
        .routes
        .iter()
        .find(|route| route.path == "/charts/vedic")
        .expect("documented route inventory should include /charts/vedic");

    assert_eq!(chart_route.auth_requirement, "bearer_or_api_key");
    assert!(
        chart_route
            .methods
            .iter()
            .any(|method| method.method == "POST" && method.handler == "vedic_chart_handler"),
        "documented chart route should publish POST via vedic_chart_handler"
    );
}
