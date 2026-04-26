use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use noesis_api::{build_app_state_lazy_db, create_router, ApiConfig};
use noesis_core::{BirthData, Coordinates, EngineInput, Precision};
use serde_json::Value;
use serial_test::serial;
use std::collections::HashMap;
use tokio::sync::OnceCell;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

static TS_HEALTH_SERVER: OnceCell<MockServer> = OnceCell::const_new();

async fn ts_health_server_uri() -> String {
    let server = TS_HEALTH_SERVER
        .get_or_init(|| async {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/health"))
                .respond_with(ResponseTemplate::new(200))
                .mount(&server)
                .await;
            server
        })
        .await;
    server.uri()
}

struct EnvGuard {
    saved: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn set(vars: &[(&'static str, String)]) -> Self {
        let mut saved = Vec::with_capacity(vars.len());
        for (key, value) in vars {
            saved.push((*key, std::env::var(key).ok()));
            std::env::set_var(key, value);
        }
        Self { saved }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, prior) in self.saved.drain(..).rev() {
            if let Some(value) = prior {
                std::env::set_var(key, value);
            } else {
                std::env::remove_var(key);
            }
        }
    }
}

fn canonical_birth_input() -> EngineInput {
    EngineInput {
        birth_data: Some(BirthData {
            name: Some("Chart Test".to_string()),
            date: "1991-09-14".to_string(),
            time: Some("09:30".to_string()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: "Asia/Kolkata".to_string(),
        }),
        current_time: chrono::Utc::now(),
        location: Some(Coordinates {
            latitude: 12.9716,
            longitude: 77.5946,
            altitude: None,
        }),
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

fn api_config() -> ApiConfig {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/noesis_test".to_string());

    ApiConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        jwt_secret: common::TEST_JWT_SECRET.to_string(),
        database_url: Some(database_url),
        redis_url: None,
        allowed_origins: vec![],
        rate_limit_requests: 100,
        rate_limit_window_secs: 60,
        request_timeout_secs: 30,
        log_level: "info".to_string(),
        log_format: "pretty".to_string(),
        discord_client_id: None,
        discord_client_secret: None,
        discord_redirect_uri: None,
        dodo_payments_api_key: None,
        dodo_payments_webhook_key: None,
        dodo_payments_env: None,
        python_biofield_url: "http://localhost:8002".to_string(),
        python_biofield_timeout_ms: 10_000,
        gateway_url: None,
        gateway_token: None,
    }
}

async fn build_router() -> Router {
    let config = api_config();
    let state = build_app_state_lazy_db(&config).await;
    create_router(state, &config)
}

async fn make_authenticated_request(
    router: &Router,
    method_name: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let token = common::generate_test_token(5);
    let request_builder = Request::builder()
        .method(method_name)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json");

    let body = if let Some(json_body) = body {
        Body::from(serde_json::to_vec(&json_body).expect("json body should serialize"))
    } else {
        Body::empty()
    };

    let response = router
        .clone()
        .oneshot(request_builder.body(body).expect("request should build"))
        .await
        .expect("router request should succeed");

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should read");
    let json = if body_bytes.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_slice(&body_bytes).expect("response should be valid json")
    };

    (status, json)
}

const VALID_SIGNS: [&str; 12] = [
    "aries",
    "taurus",
    "gemini",
    "cancer",
    "leo",
    "virgo",
    "libra",
    "scorpio",
    "sagittarius",
    "capricorn",
    "aquarius",
    "pisces",
];

#[tokio::test]
#[serial]
async fn test_vedic_chart_endpoint_returns_d1_and_d9_bundle() {
    let ts_base_url = ts_health_server_uri().await;
    let _env = EnvGuard::set(&[
        ("TS_ENGINES_URL", ts_base_url),
        ("JWT_SECRET", common::TEST_JWT_SECRET.to_string()),
    ]);
    let router = build_router().await;

    let (status, body) = make_authenticated_request(
        &router,
        "POST",
        "/api/v1/charts/vedic",
        Some(serde_json::to_value(canonical_birth_input()).expect("input should serialize")),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "body={body:?}");

    // D1 structure checks
    let d1_asc_sign = body["d1"]["ascendant"]["sign"].as_str().unwrap_or("");
    assert!(
        VALID_SIGNS.contains(&d1_asc_sign),
        "d1.ascendant.sign '{d1_asc_sign}' not a valid sign"
    );

    let d1_planets = body["d1"]["planets"]
        .as_array()
        .expect("d1.planets should be array");
    assert!(!d1_planets.is_empty(), "d1.planets should not be empty");
    assert_eq!(d1_planets.len(), 9, "d1 should have 9 vedic planets");

    let planet_names: Vec<&str> = d1_planets
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();
    for expected in [
        "Sun", "Moon", "Mars", "Mercury", "Jupiter", "Venus", "Saturn", "Rahu", "Ketu",
    ] {
        assert!(
            planet_names.contains(&expected),
            "missing planet {expected}"
        );
    }

    let d1_moon_nak = body["d1"]["moon"]["nakshatra"].as_str().unwrap_or("");
    assert!(!d1_moon_nak.is_empty(), "d1.moon.nakshatra should be set");

    let d1_houses = body["d1"]["houses"]
        .as_array()
        .expect("d1.houses should be array");
    assert_eq!(d1_houses.len(), 12, "d1 should have 12 houses");

    assert!(
        body["d1"]["ayanamsa"].as_f64().is_some(),
        "d1.ayanamsa should be a number"
    );
    assert_eq!(body["d1"]["house_system"], "whole_sign");

    // D9 structure checks
    let d9_positions = body["d9"]["navamsa_positions"]
        .as_array()
        .expect("d9.navamsa_positions should be array");
    assert!(
        !d9_positions.is_empty(),
        "d9.navamsa_positions should not be empty"
    );

    let d9_lagna = body["d9"]["d9_lagna"].as_str().unwrap_or("");
    assert!(
        VALID_SIGNS.contains(&d9_lagna),
        "d9.d9_lagna '{d9_lagna}' not a valid sign"
    );
}

#[tokio::test]
#[serial]
async fn test_vedic_chart_endpoint_rejects_missing_birth_data() {
    let router = build_router().await;
    let input = EngineInput {
        birth_data: None,
        current_time: chrono::Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    };

    let (status, body) = make_authenticated_request(
        &router,
        "POST",
        "/api/v1/charts/vedic",
        Some(serde_json::to_value(input).expect("input should serialize")),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "body={body:?}");
    assert_eq!(body["error_code"], "VALIDATION_ERROR");
}

#[tokio::test]
#[serial]
async fn test_openapi_includes_vedic_chart_endpoint() {
    let router = build_router().await;

    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/openapi.json")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("router request should succeed");

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("openapi body should read");
    let spec: Value = serde_json::from_slice(&bytes).expect("valid openapi json");
    let paths = spec
        .get("paths")
        .and_then(|value| value.as_object())
        .expect("paths object");

    assert!(
        paths.contains_key("/api/v1/charts/vedic"),
        "missing chart path in openapi spec"
    );
}
