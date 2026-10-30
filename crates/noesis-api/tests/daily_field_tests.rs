//! Daily field and the capture-optional Witness Dyad.
//!
//! - `POST /witness/interpret` without `live_scores` succeeds and never claims biofield.
//! - `current_time` dates the reading.
//! - `/daily` and `/daily/refresh` are authenticated, validate the date before touching
//!   storage, and report a missing readings store instead of failing silently.

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use chrono::{Duration as ChronoDuration, Utc};
use noesis_api::{build_app_state_lazy_db, create_router, ApiConfig};
use noesis_auth::{ApiKey, AuthService};
use serde_json::Value;
use serial_test::serial;
use std::sync::Arc;
use tower::ServiceExt;

const BIRTH: &str = r#"{"date": "1991-08-13", "time": "13:31", "latitude": 12.9716, "longitude": 77.5946, "timezone": "Asia/Kolkata"}"#;

async fn router_with_key(permissions: &[&str]) -> (Router, String) {
    let mut config = ApiConfig::from_env().expect("test config");
    config.database_url = None;
    let state = build_app_state_lazy_db(&config).await;
    let key = add_key(&state.auth, permissions).await;
    (create_router(state, &config), key)
}

async fn add_key(auth: &Arc<AuthService>, permissions: &[&str]) -> String {
    let value = format!("daily-test-key-{}", uuid::Uuid::new_v4());
    auth.add_api_key(ApiKey {
        key: value.clone(),
        user_id: uuid::Uuid::new_v4().to_string(),
        tier: "enterprise".to_string(),
        permissions: permissions.iter().map(|p| p.to_string()).collect(),
        created_at: Utc::now(),
        expires_at: Some(Utc::now() + ChronoDuration::hours(1)),
        last_used: None,
        rate_limit: 1000,
        consciousness_level: 5,
    })
    .await
    .expect("add api key");
    value
}

async fn call(router: &Router, method: &str, uri: &str, key: Option<&str>, body: Option<&str>) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(k) = key {
        builder = builder.header("X-API-Key", k);
    }
    let request = match body {
        Some(b) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(b.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    };
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
}

#[tokio::test]
#[serial]
async fn witness_without_capture_succeeds_and_never_claims_biofield() {
    let (router, key) = router_with_key(&["engines:*"]).await;
    let body = format!(r#"{{"birth_data": {BIRTH}, "current_time": "2026-10-30T00:30:00Z"}}"#);

    let (status, resp) = call(&router, "POST", "/api/v1/witness/interpret", Some(&key), Some(&body)).await;

    assert_eq!(status, StatusCode::OK, "{resp}");
    for field in ["aletheios", "pichet", "synthesis", "witness_question"] {
        assert!(resp[field].is_string(), "{field} missing: {resp}");
    }
    let used: Vec<String> = serde_json::from_value(resp["engines_used"].clone()).unwrap();
    assert!(!used.iter().any(|e| e == "biofield"), "biofield claimed without a capture: {used:?}");
    if resp["llm_powered"] == Value::Bool(false) {
        for field in ["aletheios", "pichet", "synthesis"] {
            let text = resp[field].as_str().unwrap().to_lowercase();
            assert!(!text.contains("coherence") && !text.contains("regulation"), "{field}: {text}");
        }
    }
}

#[tokio::test]
#[serial]
async fn witness_with_capture_still_accepts_the_original_body() {
    let (router, key) = router_with_key(&["engines:*"]).await;
    let body = format!(
        r#"{{"birth_data": {BIRTH}, "live_scores": {{"energy": 0.8, "coherence": 0.5, "symmetry": 0.6, "complexity": 0.5, "regulation": 0.5, "color_balance": 0.5}}}}"#
    );

    let (status, resp) = call(&router, "POST", "/api/v1/witness/interpret", Some(&key), Some(&body)).await;

    assert_eq!(status, StatusCode::OK, "{resp}");
    assert!(resp["witness_question"].is_string(), "{resp}");
}

#[tokio::test]
#[serial]
async fn daily_routes_require_authentication() {
    let (router, _key) = router_with_key(&["engines:*"]).await;
    let (get_status, _) = call(&router, "GET", "/api/v1/daily", None, None).await;
    let (post_status, _) = call(&router, "POST", "/api/v1/daily/refresh", None, Some("{}")).await;
    assert_eq!(get_status, StatusCode::UNAUTHORIZED);
    assert_eq!(post_status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn daily_routes_validate_the_date_before_storage() {
    let (router, key) = router_with_key(&["engines:*"]).await;
    let (bad_format, resp) = call(&router, "GET", "/api/v1/daily?date=30-10-2026", Some(&key), None).await;
    assert_eq!(bad_format, StatusCode::BAD_REQUEST, "{resp}");
    assert_eq!(resp["error_code"], "INVALID_DATE");

    let (too_far, resp) = call(&router, "POST", "/api/v1/daily/refresh", Some(&key), Some(r#"{"date": "2020-01-01"}"#)).await;
    assert_eq!(too_far, StatusCode::BAD_REQUEST, "{resp}");
    assert_eq!(resp["error_code"], "DATE_OUT_OF_RANGE");

    // A malformed body is refused, never read as "today".
    for body in [r#"{"date": 20261029}"#, "not json"] {
        let (status, resp) = call(&router, "POST", "/api/v1/daily/refresh", Some(&key), Some(body)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}: {resp}");
        assert_eq!(resp["error_code"], "INVALID_BODY", "{body}: {resp}");
    }
}

#[tokio::test]
#[serial]
async fn daily_routes_report_a_missing_store_or_profile_instead_of_inventing_a_day() {
    // No database in this harness: the readings store is absent, so the routes must say so
    // (or that there is no profile) rather than compute an unstored, unowned day.
    let (router, key) = router_with_key(&["engines:*"]).await;
    let (get_status, get_resp) = call(&router, "GET", "/api/v1/daily", Some(&key), None).await;
    let (post_status, post_resp) = call(&router, "POST", "/api/v1/daily/refresh", Some(&key), Some("{}")).await;
    for (status, resp) in [(get_status, get_resp), (post_status, post_resp)] {
        assert!(
            matches!(status, StatusCode::SERVICE_UNAVAILABLE | StatusCode::UNPROCESSABLE_ENTITY | StatusCode::INTERNAL_SERVER_ERROR),
            "{status} {resp}"
        );
        let code = resp["error_code"].as_str().unwrap_or_default();
        assert!(
            ["READINGS_UNAVAILABLE", "PROFILE_INCOMPLETE", "PROFILE_READ_FAILED"].contains(&code),
            "unexpected error code {code}: {resp}"
        );
    }
}
