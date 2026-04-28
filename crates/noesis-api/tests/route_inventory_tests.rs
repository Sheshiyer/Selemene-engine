mod common;

use common::route_inventory::{documented_route_inventory, source_route_inventory, workspace_root};

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

/// Generate (or regenerate) `docs/baseline/api-route-inventory.json` from the
/// current router source.  Run with:
///
///   cargo test --test route_inventory_tests generate_route_inventory -- --ignored
#[test]
#[ignore]
fn generate_route_inventory() {
    let inventory = source_route_inventory();
    let json =
        serde_json::to_string_pretty(&inventory).expect("inventory should serialise to JSON");
    let path = workspace_root().join("docs/baseline/api-route-inventory.json");
    std::fs::create_dir_all(path.parent().unwrap()).expect("docs/baseline dir should be creatable");
    std::fs::write(&path, json).expect("inventory file should be writable");
    println!("Wrote {}", path.display());
}
