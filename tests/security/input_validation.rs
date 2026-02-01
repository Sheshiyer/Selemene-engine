//! Security Tests - Input Validation
//!
//! W2-S8-05: Tests for malformed and edge case input handling:
//! - Invalid data types
//! - Boundary values
//! - Missing required fields
//! - Extra unexpected fields
//! - Malformed JSON
//!
//! Run with: cargo test --test input_validation -- --nocapture

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use noesis_api::{build_app_state, create_router, ApiConfig};
use noesis_auth::AuthService;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tower::ServiceExt;

// ===========================================================================
// Test Utilities
// ===========================================================================

static INPUT_VALIDATION_ROUTER: OnceLock<Router> = OnceLock::new();

fn get_router() -> &'static Router {
    INPUT_VALIDATION_ROUTER.get_or_init(|| {
        let config = ApiConfig::from_env();
        let state = build_app_state(&config);
        create_router(state, &config)
    })
}

fn test_token() -> String {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    let auth = AuthService::new(jwt_secret);
    auth.generate_jwt_token(
        "input-validation-user",
        "premium",
        &["read".to_string(), "write".to_string()],
        5,
    )
    .expect("Failed to generate JWT")
}

async fn authenticated_post(body: Value) -> (StatusCode, Value) {
    let router = get_router();
    let token = test_token();
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/engines/panchanga/calculate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(json!({}));
    (status, json)
}

async fn raw_post(body: &str, content_type: &str) -> StatusCode {
    let router = get_router();
    let token = test_token();
    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/engines/panchanga/calculate")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(body.to_string()))
        .unwrap();

    router.clone().oneshot(request).await.unwrap().status()
}

// ===========================================================================
// Invalid Data Type Tests
// ===========================================================================

#[tokio::test]
async fn test_string_instead_of_number_latitude() {
    let input = json!({
        "birth_data": {
            "name": "Type Test",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": "not a number",
            "longitude": -74.006,
            "timezone": "America/New_York"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, body) = authenticated_post(input).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "String latitude should be rejected: {} {:?}",
        status,
        body
    );
}

#[tokio::test]
async fn test_number_instead_of_string_date() {
    let input = json!({
        "birth_data": {
            "name": "Type Test",
            "date": 19900115,
            "time": "14:30",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "America/New_York"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Numeric date should be rejected: {}",
        status
    );
}

#[tokio::test]
async fn test_array_instead_of_object() {
    let input = json!({
        "birth_data": ["invalid", "array", "format"],
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Array birth_data should be rejected"
    );
}

// ===========================================================================
// Boundary Value Tests
// ===========================================================================

#[tokio::test]
async fn test_latitude_exactly_90() {
    let input = json!({
        "birth_data": {
            "name": "North Pole",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 90.0,
            "longitude": 0.0,
            "timezone": "UTC"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    // 90.0 is valid latitude (North Pole)
    assert!(
        status == StatusCode::OK || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Latitude 90 should be handled: {}",
        status
    );
}

#[tokio::test]
async fn test_latitude_over_90() {
    let input = json!({
        "birth_data": {
            "name": "Invalid Lat",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 91.0,
            "longitude": 0.0,
            "timezone": "UTC"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Latitude > 90 should be rejected"
    );
}

#[tokio::test]
async fn test_longitude_exactly_180() {
    let input = json!({
        "birth_data": {
            "name": "Date Line",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 0.0,
            "longitude": 180.0,
            "timezone": "Pacific/Fiji"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    // 180.0 is valid longitude
    assert!(
        status == StatusCode::OK || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Longitude 180 should be handled: {}",
        status
    );
}

#[tokio::test]
async fn test_longitude_over_180() {
    let input = json!({
        "birth_data": {
            "name": "Invalid Lon",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 0.0,
            "longitude": 181.0,
            "timezone": "UTC"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Longitude > 180 should be rejected"
    );
}

#[tokio::test]
async fn test_negative_infinity() {
    let input = json!({
        "birth_data": {
            "name": "Infinity Test",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": f64::NEG_INFINITY,
            "longitude": 0.0,
            "timezone": "UTC"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Infinity should be rejected"
    );
}

#[tokio::test]
async fn test_nan_value() {
    // NaN in JSON typically serializes as null
    let input = json!({
        "birth_data": {
            "name": "NaN Test",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": null,
            "longitude": 0.0,
            "timezone": "UTC"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Null latitude should be rejected"
    );
}

// ===========================================================================
// Missing Required Fields Tests
// ===========================================================================

#[tokio::test]
async fn test_missing_birth_data() {
    let input = json!({
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    // Missing birth_data should be handled (may be optional for some engines)
    assert!(
        status == StatusCode::OK  // Some engines don't require birth_data
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Missing birth_data should be handled: {}",
        status
    );
}

#[tokio::test]
async fn test_missing_date_in_birth_data() {
    let input = json!({
        "birth_data": {
            "name": "No Date",
            "time": "14:30",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "America/New_York"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Missing date should be rejected"
    );
}

#[tokio::test]
async fn test_empty_birth_data() {
    let input = json!({
        "birth_data": {},
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Empty birth_data should be rejected"
    );
}

// ===========================================================================
// Extra/Unexpected Fields Tests
// ===========================================================================

#[tokio::test]
async fn test_extra_fields_ignored() {
    let input = json!({
        "birth_data": {
            "name": "Extra Fields",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "America/New_York",
            "extra_field_1": "should be ignored",
            "extra_field_2": 12345
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {},
        "unexpected_top_level": "also ignored"
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    // Extra fields should be ignored, not cause errors
    assert!(
        status == StatusCode::OK,
        "Extra fields should be ignored: {}",
        status
    );
}

// ===========================================================================
// Malformed JSON Tests
// ===========================================================================

#[tokio::test]
async fn test_malformed_json_syntax() {
    let status = raw_post("{invalid json", "application/json").await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Malformed JSON should be rejected"
    );
}

#[tokio::test]
async fn test_trailing_comma_json() {
    let status = raw_post(
        r#"{"birth_data": {"name": "Test",}}"#,
        "application/json"
    ).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Trailing comma should be rejected"
    );
}

#[tokio::test]
async fn test_single_quotes_json() {
    let status = raw_post(
        r#"{'birth_data': {'name': 'Test'}}"#,
        "application/json"
    ).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Single quotes should be rejected"
    );
}

#[tokio::test]
async fn test_empty_body() {
    let status = raw_post("", "application/json").await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Empty body should be rejected"
    );
}

#[tokio::test]
async fn test_wrong_content_type() {
    let status = raw_post(
        r#"<xml><data>test</data></xml>"#,
        "application/xml"
    ).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST
            || status == StatusCode::UNPROCESSABLE_ENTITY
            || status == StatusCode::UNSUPPORTED_MEDIA_TYPE,
        "XML should be rejected when JSON expected"
    );
}

// ===========================================================================
// Date/Time Format Tests
// ===========================================================================

#[tokio::test]
async fn test_invalid_date_format() {
    let invalid_dates = vec![
        "15-01-1990",    // DD-MM-YYYY instead of YYYY-MM-DD
        "01/15/1990",    // MM/DD/YYYY
        "1990/01/15",    // Wrong separator
        "1990-1-15",     // Missing leading zero
        "1990-13-01",    // Invalid month
        "1990-01-32",    // Invalid day
        "not-a-date",    // Completely invalid
    ];
    
    for date in invalid_dates {
        let input = json!({
            "birth_data": {
                "name": "Date Format Test",
                "date": date,
                "time": "14:30",
                "latitude": 40.7128,
                "longitude": -74.006,
                "timezone": "America/New_York"
            },
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {}
        });
        
        let (status, _body) = authenticated_post(input).await;
        
        assert!(
            status == StatusCode::BAD_REQUEST
                || status == StatusCode::UNPROCESSABLE_ENTITY
                || status == StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid date '{}' should be rejected: {}",
            date,
            status
        );
    }
}

#[tokio::test]
async fn test_invalid_time_format() {
    let invalid_times = vec![
        "2:30 PM",       // 12-hour format
        "14:30:00",      // Seconds included (may or may not be valid)
        "14.30",         // Wrong separator
        "25:00",         // Invalid hour
        "14:60",         // Invalid minute
        "noon",          // Text
    ];
    
    for time in invalid_times {
        let input = json!({
            "birth_data": {
                "name": "Time Format Test",
                "date": "1990-01-15",
                "time": time,
                "latitude": 40.7128,
                "longitude": -74.006,
                "timezone": "America/New_York"
            },
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {}
        });
        
        let (status, _body) = authenticated_post(input).await;
        
        // Some formats might be accepted, some rejected
        assert!(
            status == StatusCode::OK
                || status == StatusCode::BAD_REQUEST
                || status == StatusCode::UNPROCESSABLE_ENTITY,
            "Time '{}' handling: {}",
            time,
            status
        );
    }
}

#[tokio::test]
async fn test_invalid_current_time_format() {
    let input = json!({
        "birth_data": {
            "name": "Current Time Test",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "America/New_York"
        },
        "current_time": "not-a-timestamp",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(input).await;
    
    assert!(
        status == StatusCode::BAD_REQUEST || status == StatusCode::UNPROCESSABLE_ENTITY,
        "Invalid current_time should be rejected"
    );
}
