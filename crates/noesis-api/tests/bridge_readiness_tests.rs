use axum::http::StatusCode;
use serial_test::serial;
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

async fn ts_health_server_uri(readiness_status: u16, readiness_body: serde_json::Value) -> String {
    let server = TS_HEALTH_SERVER
        .get_or_init(|| async { MockServer::start().await })
        .await;

    server.reset().await;

    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "healthy",
            "engines": ["tarot", "i-ching", "enneagram", "sacred-geometry", "sigil-forge"],
            "uptime_ms": 123,
            "version": "1.0.0"
        })))
        .mount(server)
        .await;

    Mock::given(method("GET"))
        .and(path("/health/ready"))
        .respond_with(ResponseTemplate::new(readiness_status).set_body_json(readiness_body))
        .mount(server)
        .await;

    server.uri()
}

#[tokio::test]
#[serial]
async fn test_ready_reports_bridge_available_when_sidecar_is_ready() {
    let ts_base_url = ts_health_server_uri(
        200,
        serde_json::json!({
            "status": "ready",
            "engines": [
                {"engine_id":"tarot","healthy":true,"detail":"ok","latency_ms":1},
                {"engine_id":"i-ching","healthy":true,"detail":"ok","latency_ms":1}
            ],
            "failed_engines": []
        }),
    )
    .await;
    let _env = EnvGuard::set(&[("TS_ENGINES_URL", ts_base_url)]);

    let (status, body) = common::make_unauthenticated_request("GET", "/ready", None).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "body={body:?}");
    assert_eq!(body["bridge_status"], "available");
    assert!(body["bridge_engines"].is_array());
    assert_eq!(body["bridge_engines"].as_array().unwrap().len(), 2);
}

#[tokio::test]
#[serial]
async fn test_ready_reports_bridge_degraded_with_failed_engine_details() {
    let ts_base_url = ts_health_server_uri(
        503,
        serde_json::json!({
            "status": "degraded",
            "engines": [
                {"engine_id":"tarot","healthy":true,"detail":"ok","latency_ms":1},
                {"engine_id":"sigil-forge","healthy":false,"detail":"engine unhealthy","latency_ms":2}
            ],
            "failed_engines": ["sigil-forge"]
        }),
    )
    .await;
    let _env = EnvGuard::set(&[("TS_ENGINES_URL", ts_base_url)]);

    let (status, body) = common::make_unauthenticated_request("GET", "/ready", None).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE, "body={body:?}");
    assert_eq!(body["bridge_status"], "degraded");
    assert_eq!(body["bridge_failed_engines"], serde_json::json!(["sigil-forge"]));
    assert_eq!(
        body["bridge_engines"][1]["detail"],
        serde_json::json!("engine unhealthy")
    );
}
