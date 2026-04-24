mod common;

use common::route_inventory::{documented_route_inventory, source_route_inventory};

#[test]
fn route_inventory_matches_source_router() {
    let documented = documented_route_inventory();
    let source = source_route_inventory();

    assert_eq!(documented.inventory_scope, "/api/v1");
    assert_eq!(documented.source, "crates/noesis-api/src/lib.rs");
    assert_eq!(documented.path_count, documented.routes.len());
    assert_eq!(
        documented.route_count,
        documented
            .routes
            .iter()
            .map(|route| route.methods.len())
            .sum::<usize>()
    );
    assert_eq!(documented, source);
}
