use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use noesis_vedic_api::{Config, VedicApiClient, VedicApiError};

/// A real vendor response captured in `tests/fixtures/captures`.
fn planets_envelope() -> serde_json::Value {
    let path = format!(
        "{}/tests/fixtures/captures/planets_bangalore_1991-08-13.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    serde_json::from_slice(&bytes).unwrap_or_else(|e| panic!("parse {path}: {e}"))
}

// Panchang has been computed natively since PR2 and makes no HTTP call, so
// the API-key header and the HTTP error mapping are exercised through
// `/planets` (the birth chart), which still goes to the vendor.

#[tokio::test]
async fn test_get_panchang_is_native_and_makes_no_request() {
    let server = MockServer::start().await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let client = VedicApiClient::new(config);

    let result = client
        .get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5)
        .await
        .expect("native panchang must succeed");

    assert_eq!(
        (result.date.year, result.date.month, result.date.day),
        (2024, 1, 1)
    );
    assert!((1..=30).contains(&result.tithi.number));
    assert!((1..=27).contains(&result.nakshatra.number));
    let received = server.received_requests().await.expect("requests");
    assert!(received.is_empty(), "panchang must not call the vendor");
}

#[tokio::test]
async fn test_birth_chart_uses_x_api_key_header_and_planets_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/planets"))
        .and(header("x-api-key", "test_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(planets_envelope()))
        .expect(1)
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let client = VedicApiClient::new(config);

    let chart = client
        .get_birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
        .await
        .expect("Expected birth chart response");

    assert_eq!(chart.ascendant.sign.as_str(), "Scorpio");
}

#[tokio::test]
async fn test_unauthorized_maps_to_configuration_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/planets"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid api key"))
        .mount(&server)
        .await;

    let config = Config::new("bad_key").with_base_url(server.uri());
    let client = VedicApiClient::new(config);

    let err = client
        .get_birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
        .await
        .expect_err("Expected configuration error");

    match err {
        VedicApiError::Configuration { field, .. } => assert_eq!(field, "api_key"),
        _ => panic!("Unexpected error: {err:?}"),
    }
}

#[tokio::test]
async fn test_rate_limit_maps_to_rate_limit_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/planets"))
        .respond_with(ResponseTemplate::new(429).set_body_string("rate limit"))
        .mount(&server)
        .await;

    let config = Config::new("test_key").with_base_url(server.uri());
    let client = VedicApiClient::new(config);

    let err = client
        .get_birth_chart(1991, 8, 13, 13, 31, 0, 12.9716, 77.5946, 5.5)
        .await
        .expect_err("Expected rate limit error");

    match err {
        VedicApiError::RateLimit { .. } => {}
        _ => panic!("Unexpected error: {err:?}"),
    }
}
