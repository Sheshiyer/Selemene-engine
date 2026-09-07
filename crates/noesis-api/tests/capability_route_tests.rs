//! Integration tests for the Rust/API contract-v1 capability discovery route.
//!
//! This mirrors the TypeScript `/engines/capabilities` endpoint's contract-v1
//! shape (`contracts/v1` `EngineCapability`), but on the admin-gated Rust/API
//! surface, deriving availability from the same bridge readiness self-check
//! data source that `/admin/bridge/health` already uses. Permission resolution
//! still reads the configured admin database; no provider generation is performed.

use axum::http::StatusCode;
use noesis_auth::AuthService;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

mod common;

// The shared Router owns a PgPool. Keep its Tokio runtime alive across tests,
// including when CI runs each #[test] sequentially with --test-threads=1.
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

const CANONICAL_ENGINE_IDS: [&str; 19] = [
    "biofield",
    "biofield-capture",
    "biorhythm",
    "enneagram",
    "face-reading",
    "financial-biosensor",
    "gene-keys",
    "human-design",
    "i-ching",
    "nadabrahman",
    "numerology",
    "panchanga",
    "raaga",
    "sacred-geometry",
    "sigil-forge",
    "tarot",
    "transits",
    "vedic-clock",
    "vimshottari",
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
fn test_public_capability_route_requires_auth() {
    test_runtime().block_on(async {
        let (status, _body) = common::make_unauthenticated_request(
            "GET",
            "/api/v1/engines/capabilities",
            None,
        )
        .await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_public_capability_route_returns_canonical_19_row_envelope() {
    test_runtime().block_on(async {
        let token = generate_user_token();
        let (status, body) = common::make_authenticated_request(
            "GET",
            "/api/v1/engines/capabilities",
            &token,
            None,
        )
        .await;

        assert_eq!(status, StatusCode::OK, "body={body:?}");
        assert_eq!(body["contract_version"], "v1");
        assert_eq!(body["count"], 19);
        assert_eq!(body["public_mirror_count"], 17);

        let capabilities = body["capabilities"]
            .as_array()
            .unwrap_or_else(|| panic!("capabilities should be an array: {body:?}"));
        assert_eq!(capabilities.len(), CANONICAL_ENGINE_IDS.len());

        let ids = capabilities
            .iter()
            .map(|capability| capability["engine_id"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids, CANONICAL_ENGINE_IDS);
        for capability in capabilities {
            assert_eq!(capability["contract_version"], "v1");
            assert!(capability["reason_code"].is_string());
            assert!(capability["dependency_observations"].is_array());
            assert!(capability["operations"].is_object());
            assert!(capability.get("admin_analytics").is_none());
            assert!(capability.get("internal_url").is_none());
        }
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

        assert_eq!(
            capabilities.len(),
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
            assert_eq!(
                capability["runtime_kind"], "typescript",
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
