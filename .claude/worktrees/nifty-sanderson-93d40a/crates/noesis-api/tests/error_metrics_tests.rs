use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use tower::ServiceExt;

mod common;

fn parse_error_metric(metrics_text: &str, error_code: &str) -> f64 {
    let needle = format!("error_code=\"{}\"", error_code);
    metrics_text
        .lines()
        .find(|line| line.starts_with("noesis_api_errors_total{") && line.contains(&needle))
        .and_then(|line| line.split_whitespace().last())
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(0.0)
}

async fn fetch_metrics_text() -> String {
    let router = common::get_router().await;
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[serial_test::serial(api_error_metrics)]
#[tokio::test]
async fn metrics_endpoint_tracks_api_errors_by_error_code() {
    let before = fetch_metrics_text().await;
    let before_not_found = parse_error_metric(&before, "ENGINE_NOT_FOUND");
    let before_unauthorized = parse_error_metric(&before, "UNAUTHORIZED");

    let token = common::generate_test_token(5);
    let router = common::get_router().await;

    let not_found_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/engines/does-not-exist/calculate")
                .header(header::AUTHORIZATION, format!("Bearer {}", token))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(
                        &serde_json::to_value(common::create_test_birth_input()).unwrap(),
                    )
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(not_found_response.status(), StatusCode::NOT_FOUND);

    let unauthorized_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(unauthorized_response.status(), StatusCode::UNAUTHORIZED);

    let after = fetch_metrics_text().await;
    let after_not_found = parse_error_metric(&after, "ENGINE_NOT_FOUND");
    let after_unauthorized = parse_error_metric(&after, "UNAUTHORIZED");

    assert!(
        after.contains("noesis_api_errors_total"),
        "missing noesis_api_errors_total from metrics"
    );
    assert_eq!(after_not_found, before_not_found + 1.0);
    assert_eq!(after_unauthorized, before_unauthorized + 1.0);
}
