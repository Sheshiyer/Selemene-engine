//! Integration tests for the Rust/API contract-v1 capability discovery route.
//!
//! This mirrors the TypeScript `/engines/capabilities` endpoint's contract-v1
//! shape (`contracts/v1` `EngineCapability`) on two Rust/API surfaces: the
//! admin-gated `/admin/engines/capabilities` and the API-key / bearer
//! `/engines/capabilities` (same auth group as `GET /engines`). Both delegate
//! to `noesis_api::capabilities::collect_capabilities`, which lists native
//! engines plus bridge-proxied TypeScript engines and derives bridge
//! availability from the same readiness self-check `/admin/bridge/health`
//! uses (no provider, database, or additional remote calls).

use axum::http::StatusCode;
use noesis_auth::AuthService;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

mod common;

// The shared Router owns a PgPool. Keep its Tokio runtime alive across tests,
// including when CI runs each test sequentially with --test-threads=1.
static TEST_RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn test_runtime() -> &'static Runtime {
    TEST_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to build shared test runtime")
    })
}

const KNOWN_TS_ENGINES: [&str; 6] = [
    "tarot",
    "i-ching",
    "enneagram",
    "sacred-geometry",
    "sigil-forge",
    "raaga",
];

fn generate_admin_token() -> String {
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| common::TEST_JWT_SECRET.to_string());
    let auth = AuthService::new(jwt_secret);

    auth.generate_jwt_token(
        "550e8400-e29b-41d4-a716-446655440099",
        "pro",
        &["admin:system:read".to_string()],
        7,
    )
    .expect("Failed to generate admin token")
}

fn generate_user_token() -> String {
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| common::TEST_JWT_SECRET.to_string());
    let auth = AuthService::new(jwt_secret);

    auth.generate_jwt_token(
        "550e8400-e29b-41d4-a716-446655440098",
        "free",
        &["read".to_string()],
        5,
    )
    .expect("Failed to generate user token")
}

#[test]
fn test_capability_route_requires_auth() {
    test_runtime().block_on(async {
        let (status, _body) =
            common::make_unauthenticated_request("GET", "/api/v1/admin/engines/capabilities", None)
                .await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_capability_route_requires_admin_permission() {
    test_runtime().block_on(async {
        let token = generate_user_token();
        let (status, _body) = common::make_authenticated_request(
            "GET",
            "/api/v1/admin/engines/capabilities",
            &token,
            None,
        )
        .await;

        assert_eq!(status, StatusCode::FORBIDDEN);
    });
}

#[test]
fn test_capability_route_returns_contract_v1_shape_for_all_ts_engines() {
    test_runtime().block_on(async {
        let token = generate_admin_token();
        let (status, body) = common::make_authenticated_request(
            "GET",
            "/api/v1/admin/engines/capabilities",
            &token,
            None,
        )
        .await;

        assert_eq!(status, StatusCode::OK, "body={body:?}");

        let capabilities = body
            .as_array()
            .unwrap_or_else(|| panic!("response body should be a JSON array, got body={body:?}"));

        // The admin surface now lists native engines too; the TS subset must
        // still be exactly the bridge registry.
        let ts_capabilities: Vec<&serde_json::Value> = capabilities
            .iter()
            .filter(|capability| capability["runtime_kind"] == "typescript")
            .collect();
        assert_eq!(
            ts_capabilities.len(),
            KNOWN_TS_ENGINES.len(),
            "expected {} TS engine capability rows, got body={body:?}",
            KNOWN_TS_ENGINES.len()
        );

        let mut seen_ids: Vec<String> = Vec::new();
        for capability in capabilities {
            assert_eq!(
                capability["contract_version"], "v1",
                "capability={capability:?}"
            );
            assert!(
                ["native", "typescript"]
                    .contains(&capability["runtime_kind"].as_str().unwrap_or("")),
                "capability={capability:?}"
            );
            assert!(
                capability["availability"].is_string(),
                "capability={capability:?}"
            );
            let availability = capability["availability"].as_str().unwrap();
            assert!(
                ["declared", "available", "degraded", "unavailable"].contains(&availability),
                "unexpected availability value: {availability} in capability={capability:?}"
            );
            assert!(
                capability["display_name"].is_string(),
                "capability={capability:?}"
            );
            assert!(
                capability["dependencies"].is_array(),
                "capability={capability:?}"
            );

            seen_ids.push(
                capability["engine_id"]
                    .as_str()
                    .expect("engine_id should be a string")
                    .to_string(),
            );
        }

        for expected in KNOWN_TS_ENGINES {
            assert!(
                seen_ids.iter().any(|id| id == expected),
                "expected engine_id {expected} in {seen_ids:?}"
            );
        }
    });
}

#[test]
fn test_public_capabilities_route_is_reachable_with_user_token() {
    test_runtime().block_on(async {
        let token = generate_user_token();
        let (status, body) =
            common::make_authenticated_request("GET", "/api/v1/engines/capabilities", &token, None)
                .await;
        assert_eq!(status, StatusCode::OK, "body={body:?}");
        let caps = body["capabilities"].as_array().expect("capabilities array");
        assert_eq!(body["count"].as_u64().unwrap() as usize, caps.len());
        assert!(
            !caps.is_empty(),
            "must never be empty even if bridge readiness fails"
        );
        let native = caps
            .iter()
            .find(|c| c["engine_id"] == "panchanga")
            .expect("native engine listed");
        assert_eq!(native["contract_version"], "v1");
        assert_eq!(native["runtime_kind"], "native");
        assert_eq!(native["availability"], "available");
        for id in KNOWN_TS_ENGINES {
            let row = caps
                .iter()
                .find(|c| c["engine_id"] == id)
                .unwrap_or_else(|| panic!("{id} missing in {caps:?}"));
            assert_eq!(row["contract_version"], "v1");
            assert_eq!(row["runtime_kind"], "typescript");
            assert!(matches!(
                row["availability"].as_str(),
                Some("available" | "unavailable")
            ));
        }
    });
}

#[test]
fn test_public_capabilities_route_requires_auth() {
    test_runtime().block_on(async {
        let (status, _) =
            common::make_unauthenticated_request("GET", "/api/v1/engines/capabilities", None)
                .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    });
}
