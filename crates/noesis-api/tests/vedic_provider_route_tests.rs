use noesis_core::{BirthData, EngineInput, Precision};
use serde_json::Value;
use serial_test::serial;
use std::collections::HashMap;
use tokio::sync::OnceCell;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;

static TS_HEALTH_SERVER: OnceCell<MockServer> = OnceCell::const_new();

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

fn canonical_birth_input() -> EngineInput {
    EngineInput {
        birth_data: Some(BirthData {
            name: Some("Canonical".to_string()),
            date: "1991-08-13".to_string(),
            time: Some("13:31".to_string()),
            latitude: 12.9340,
            longitude: 77.6214,
            timezone: "Asia/Kolkata".to_string(),
        }),
        current_time: chrono::Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

fn auth_token() -> String {
    common::generate_test_token(5)
}

async fn authed_request(uri: &str, body: Value) -> (axum::http::StatusCode, Value) {
    let token = auth_token();
    common::make_authenticated_request("POST", uri, &token, Some(body)).await
}

fn provider_env(provider_base_url: String, ts_base_url: String) -> EnvGuard {
    EnvGuard::set(&[
        ("FREE_ASTROLOGY_API_KEY", "test_key".to_string()),
        ("FREE_ASTROLOGY_API_BASE_URL", provider_base_url),
        ("VEDIC_ENGINE_PROVIDER", "api".to_string()),
        ("VEDIC_ENGINE_FALLBACK_ENABLED", "false".to_string()),
        ("TS_ENGINES_URL", ts_base_url),
    ])
}

#[tokio::test]
#[serial]
async fn test_panchanga_route_stays_native_even_when_provider_env_is_set() {
    let provider_server = MockServer::start().await;
    let ts_base_url = ts_health_server_uri().await;
    let _env = provider_env(provider_server.uri(), ts_base_url);
    let input = canonical_birth_input();

    let (status, body) = authed_request(
        "/api/v1/engines/panchanga/calculate",
        serde_json::to_value(input).unwrap(),
    )
    .await;

    assert_eq!(status, axum::http::StatusCode::OK, "body={body:?}");
    assert_eq!(body["metadata"]["backend"], "native-rust");
    assert!(body["result"].get("provider").is_none());
    assert!(body["result"].get("_selemene_execution").is_none());

    let requests = provider_server.received_requests().await.unwrap();
    assert!(
        requests.is_empty(),
        "provider should not be called for panchanga runtime"
    );
}

#[tokio::test]
#[serial]
async fn test_status_and_readiness_no_longer_report_provider_fields() {
    let provider_server = MockServer::start().await;
    let ts_base_url = ts_health_server_uri().await;
    let _env = provider_env(provider_server.uri(), ts_base_url);
    let token = auth_token();

    let (status_status, body_status) =
        common::make_authenticated_request("GET", "/api/v1/status", &token, None).await;
    assert_eq!(
        status_status,
        axum::http::StatusCode::OK,
        "body={body_status:?}"
    );
    assert!(body_status.get("configured_vedic_provider").is_none());
    assert!(body_status.get("effective_vedic_provider").is_none());
    assert!(body_status.get("vedic_engine_modes").is_none());

    let (status_ready, body_ready) =
        common::make_unauthenticated_request("GET", "/ready", None).await;
    assert_eq!(
        status_ready,
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        "body={body_ready:?}"
    );
    assert!(body_ready.get("vedic_api").is_none());
    assert!(body_ready.get("configured_vedic_provider").is_none());
    assert!(body_ready.get("effective_vedic_provider").is_none());
}
