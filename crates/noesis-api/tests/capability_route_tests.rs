use axum::http::StatusCode;
use serde_json::Value;

mod common;

fn capability_by_id<'a>(body: &'a Value, engine_id: &str) -> &'a Value {
    body["capabilities"]
        .as_array()
        .expect("capabilities must be an array")
        .iter()
        .find(|capability| capability["engine_id"] == engine_id)
        .unwrap_or_else(|| panic!("missing capability for {engine_id}"))
}

#[tokio::test]
async fn engine_capabilities_route_returns_contract_v1_runtime_rows() {
    let token = common::generate_test_token(5);

    let (status, body) =
        common::make_authenticated_request("GET", "/api/v1/engines/capabilities", &token, None)
            .await;

    assert_eq!(status, StatusCode::OK);
    let capabilities = body["capabilities"]
        .as_array()
        .expect("capabilities must be an array");
    assert_eq!(body["count"].as_u64(), Some(capabilities.len() as u64));
    assert!(capabilities.len() >= 18);
    assert!(capabilities
        .iter()
        .all(|capability| capability["contract_version"] == "v1"));

    let panchanga = capability_by_id(&body, "panchanga");
    assert_eq!(panchanga["runtime_kind"], "native");
    assert_eq!(panchanga["availability"], "available");
    assert_eq!(panchanga["dependencies"], Value::Array(vec![]));

    let tarot = capability_by_id(&body, "tarot");
    assert_eq!(tarot["runtime_kind"], "typescript");
    assert_eq!(tarot["availability"], "declared");
    assert_eq!(tarot["dependencies"], serde_json::json!(["ts-engines"]));
}
