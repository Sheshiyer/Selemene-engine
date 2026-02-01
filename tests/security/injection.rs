//! Security Tests - Injection Attacks
//!
//! W2-S8-05: Tests for injection vulnerabilities:
//! - SQL injection (if applicable)
//! - Command injection
//! - JSON injection
//! - Path traversal
//! - SSRF attempts
//!
//! Run with: cargo test --test injection -- --nocapture

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

static INJECTION_ROUTER: OnceLock<Router> = OnceLock::new();

fn get_router() -> &'static Router {
    INJECTION_ROUTER.get_or_init(|| {
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
        "injection-test-user",
        "premium",
        &["read".to_string(), "write".to_string()],
        5,
    )
    .expect("Failed to generate JWT")
}

async fn authenticated_post(uri: &str, body: Value) -> (StatusCode, Value) {
    let router = get_router();
    let token = test_token();
    let request = Request::builder()
        .method("POST")
        .uri(uri)
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

// ===========================================================================
// SQL Injection Tests (if applicable)
// ===========================================================================

#[tokio::test]
async fn test_sql_injection_in_name() {
    let payloads = vec![
        "'; DROP TABLE users; --",
        "1' OR '1'='1",
        "1; SELECT * FROM users",
        "' UNION SELECT * FROM admin --",
        "Robert'); DROP TABLE users;--",
    ];
    
    for payload in payloads {
        let input = json!({
            "birth_data": {
                "name": payload,
                "date": "1990-01-15",
                "time": "14:30",
                "latitude": 40.7128,
                "longitude": -74.006,
                "timezone": "America/New_York"
            },
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {}
        });
        
        let (status, body) = authenticated_post(
            "/api/v1/engines/numerology/calculate",
            input,
        ).await;

        // Should not error in a way that reveals SQL
        assert!(
            status != StatusCode::INTERNAL_SERVER_ERROR
                || !body.to_string().to_lowercase().contains("sql"),
            "SQL injection payload should be handled safely: {}",
            payload
        );
    }
}

// ===========================================================================
// Command Injection Tests
// ===========================================================================

#[tokio::test]
async fn test_command_injection_in_timezone() {
    let payloads = vec![
        "America/New_York; cat /etc/passwd",
        "`cat /etc/passwd`",
        "$(cat /etc/passwd)",
        "America/New_York | ls -la",
        "America/New_York && rm -rf /",
    ];
    
    for payload in payloads {
        let input = json!({
            "birth_data": {
                "name": "Command Test",
                "date": "1990-01-15",
                "time": "14:30",
                "latitude": 40.7128,
                "longitude": -74.006,
                "timezone": payload
            },
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {}
        });
        
        let (status, body) = authenticated_post(
            "/api/v1/engines/panchanga/calculate",
            input,
        ).await;

        // Should return validation error or handle safely
        assert!(
            status == StatusCode::UNPROCESSABLE_ENTITY
                || status == StatusCode::BAD_REQUEST
                || status == StatusCode::OK
                || (status == StatusCode::INTERNAL_SERVER_ERROR
                    && !body.to_string().contains("root:")), // No /etc/passwd content
            "Command injection should be handled: {} - {}",
            payload,
            status
        );
    }
}

// ===========================================================================
// JSON Injection Tests
// ===========================================================================

#[tokio::test]
async fn test_json_prototype_pollution() {
    let input = json!({
        "birth_data": {
            "name": "Prototype Test",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "America/New_York",
            "__proto__": {"admin": true},
            "constructor": {"prototype": {"admin": true}}
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {
            "__proto__": {"isAdmin": true}
        }
    });
    
    let (status, _body) = authenticated_post(
        "/api/v1/engines/numerology/calculate",
        input,
    ).await;

    // Should handle prototype pollution attempts safely
    // Rust's serde doesn't have prototype pollution like JS, but good to test
    assert!(
        status == StatusCode::OK
            || status == StatusCode::UNPROCESSABLE_ENTITY
            || status == StatusCode::BAD_REQUEST,
        "Should handle prototype pollution attempt: {}",
        status
    );
}

#[tokio::test]
async fn test_deeply_nested_json() {
    // Create deeply nested JSON to test stack overflow protection
    let mut deep = json!({});
    let mut current = &mut deep;
    
    for i in 0..100 {
        current["nested"] = json!({
            "level": i,
            "data": format!("Level {}", i)
        });
        current = current.get_mut("nested").unwrap();
    }
    
    let input = json!({
        "birth_data": {
            "name": "Deep Nest Test",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "America/New_York"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": deep
    });
    
    let (status, _body) = authenticated_post(
        "/api/v1/engines/numerology/calculate",
        input,
    ).await;

    // Should handle deep nesting without crashing
    assert!(
        status == StatusCode::OK
            || status == StatusCode::UNPROCESSABLE_ENTITY
            || status == StatusCode::BAD_REQUEST,
        "Deep nesting should be handled safely"
    );
}

// ===========================================================================
// Path Traversal Tests
// ===========================================================================

#[tokio::test]
async fn test_path_traversal_in_engine_id() {
    let router = get_router();
    let token = test_token();
    
    let payloads = vec![
        "../../../etc/passwd",
        "..%2F..%2F..%2Fetc%2Fpasswd",
        "panchanga/../../../etc/passwd",
        "panchanga%00.txt",
        "..\\..\\..\\windows\\system32\\config\\sam",
    ];
    
    for payload in payloads {
        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/v1/engines/{}/calculate", payload))
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"current_time":"2025-01-15T12:00:00Z"}"#))
            .unwrap();

        let response = router.clone().oneshot(request).await.unwrap();
        
        // Should return 404 or 400, not expose file system
        assert!(
            response.status() == StatusCode::NOT_FOUND
                || response.status() == StatusCode::BAD_REQUEST,
            "Path traversal should be blocked: {}",
            payload
        );
    }
}

// ===========================================================================
// SSRF Tests
// ===========================================================================

#[tokio::test]
async fn test_ssrf_in_options() {
    let payloads = vec![
        "http://169.254.169.254/latest/meta-data/",  // AWS metadata
        "http://localhost:22/",
        "http://127.0.0.1:3306/",
        "file:///etc/passwd",
        "gopher://localhost:25/",
    ];
    
    for payload in payloads {
        let input = json!({
            "birth_data": {
                "name": "SSRF Test",
                "date": "1990-01-15",
                "time": "14:30",
                "latitude": 40.7128,
                "longitude": -74.006,
                "timezone": "America/New_York"
            },
            "current_time": "2025-01-15T12:00:00Z",
            "precision": "Standard",
            "options": {
                "callback_url": payload,
                "external_service": payload
            }
        });
        
        let (status, body) = authenticated_post(
            "/api/v1/engines/numerology/calculate",
            input,
        ).await;

        // Should not make outbound requests to arbitrary URLs
        // Response should not contain AWS metadata or local service info
        assert!(
            !body.to_string().contains("ami-")
                && !body.to_string().contains("instance-id"),
            "SSRF should be blocked: {}",
            payload
        );
    }
}

// ===========================================================================
// Unicode/Encoding Attack Tests
// ===========================================================================

#[tokio::test]
async fn test_unicode_null_byte_injection() {
    let input = json!({
        "birth_data": {
            "name": "Test\u{0000}Admin",
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "America/New_York"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(
        "/api/v1/engines/numerology/calculate",
        input,
    ).await;

    // Should handle null bytes safely
    assert!(
        status == StatusCode::OK
            || status == StatusCode::UNPROCESSABLE_ENTITY
            || status == StatusCode::BAD_REQUEST,
        "Null byte should be handled safely"
    );
}

#[tokio::test]
async fn test_unicode_overflow_characters() {
    // Very long unicode string
    let long_string: String = "漢".repeat(10000);
    
    let input = json!({
        "birth_data": {
            "name": long_string,
            "date": "1990-01-15",
            "time": "14:30",
            "latitude": 40.7128,
            "longitude": -74.006,
            "timezone": "America/New_York"
        },
        "current_time": "2025-01-15T12:00:00Z",
        "precision": "Standard",
        "options": {}
    });
    
    let (status, _body) = authenticated_post(
        "/api/v1/engines/numerology/calculate",
        input,
    ).await;

    // Should handle long unicode strings without crashing
    assert!(
        status == StatusCode::OK
            || status == StatusCode::UNPROCESSABLE_ENTITY
            || status == StatusCode::BAD_REQUEST
            || status == StatusCode::REQUEST_ENTITY_TOO_LARGE,
        "Long unicode should be handled safely"
    );
}
