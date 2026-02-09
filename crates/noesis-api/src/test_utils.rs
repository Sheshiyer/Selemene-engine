//! Test utilities and fixtures for API testing
//!
//! Provides reusable test helpers, mock data, and assertion utilities
//! for comprehensive API testing.

use axum::{
    body::Body,
    http::{Request, Response, StatusCode, header},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower::ServiceExt;

/// Test client for making API requests
pub struct TestClient {
    router: Router,
}

impl TestClient {
    /// Create a new test client with the given router
    pub fn new(router: Router) -> Self {
        Self { router }
    }

    /// Make a GET request
    pub async fn get(&self, uri: &str) -> TestResponse {
        let request = Request::builder()
            .uri(uri)
            .method("GET")
            .body(Body::empty())
            .unwrap();

        self.request(request).await
    }

    /// Make a POST request with JSON body
    pub async fn post(&self, uri: &str, body: impl Serialize) -> TestResponse {
        let json = serde_json::to_string(&body).unwrap();
        let request = Request::builder()
            .uri(uri)
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json))
            .unwrap();

        self.request(request).await
    }

    /// Make a POST request with authentication
    pub async fn post_with_auth(
        &self,
        uri: &str,
        body: impl Serialize,
        token: &str,
    ) -> TestResponse {
        let json = serde_json::to_string(&body).unwrap();
        let request = Request::builder()
            .uri(uri)
            .method("POST")
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::from(json))
            .unwrap();

        self.request(request).await
    }

    /// Make a request with API key
    pub async fn request_with_api_key(
        &self,
        request: Request<Body>,
        api_key: &str,
    ) -> TestResponse {
        let (mut parts, body) = request.into_parts();
        parts.headers.insert("x-api-key", api_key.parse().unwrap());
        let request = Request::from_parts(parts, body);
        self.request(request).await
    }

    /// Make a raw request
    pub async fn request(&self, request: Request<Body>) -> TestResponse {
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("Failed to execute request");

        TestResponse::new(response)
    }
}

/// Test response wrapper with convenience methods
pub struct TestResponse {
    response: Response<Body>,
}

impl TestResponse {
    fn new(response: Response<Body>) -> Self {
        Self { response }
    }

    /// Get response status code
    pub fn status(&self) -> StatusCode {
        self.response.status()
    }

    /// Get response as JSON
    pub async fn json<T: for<'de> Deserialize<'de>>(self) -> T {
        let body = axum::body::to_bytes(self.response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        serde_json::from_slice(&body).expect("Failed to parse JSON")
    }

    /// Get response as Value
    pub async fn json_value(self) -> Value {
        self.json().await
    }

    /// Get response as text
    pub async fn text(self) -> String {
        let body = axum::body::to_bytes(self.response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");
        String::from_utf8(body.to_vec()).expect("Failed to parse as UTF-8")
    }

    /// Assert status code
    pub fn assert_status(&self, expected: StatusCode) {
        assert_eq!(
            self.status(),
            expected,
            "Expected status {}, got {}",
            expected,
            self.status()
        );
    }

    /// Assert success status (2xx)
    pub fn assert_success(&self) {
        assert!(
            self.status().is_success(),
            "Expected success status, got {}",
            self.status()
        );
    }

    /// Assert client error (4xx)
    pub fn assert_client_error(&self) {
        assert!(
            self.status().is_client_error(),
            "Expected client error status, got {}",
            self.status()
        );
    }

    /// Assert server error (5xx)
    pub fn assert_server_error(&self) {
        assert!(
            self.status().is_server_error(),
            "Expected server error status, got {}",
            self.status()
        );
    }
}

/// Test fixtures for common test data
pub mod fixtures {
    use serde_json::json;

    /// Valid Panchanga calculation request
    pub fn panchanga_request() -> serde_json::Value {
        json!({
            "date": "2025-01-27",
            "latitude": 19.0760,
            "longitude": 72.8777,
            "timezone": "Asia/Kolkata",
            "precision": "high",
            "include_details": true
        })
    }

    /// Invalid Panchanga request (missing required fields)
    pub fn invalid_panchanga_request() -> serde_json::Value {
        json!({
            "date": "2025-01-27"
        })
    }

    /// Invalid date format request
    pub fn invalid_date_request() -> serde_json::Value {
        json!({
            "date": "27/01/2025",
            "latitude": 19.0760,
            "longitude": 72.8777
        })
    }

    /// Invalid coordinates request
    pub fn invalid_coordinates_request() -> serde_json::Value {
        json!({
            "date": "2025-01-27",
            "latitude": 100.0,  // Invalid: > 90
            "longitude": 200.0  // Invalid: > 180
        })
    }

    /// Batch calculation request
    pub fn batch_request() -> serde_json::Value {
        json!({
            "requests": [
                panchanga_request(),
                {
                    "date": "2025-01-28",
                    "latitude": 28.6139,
                    "longitude": 77.2090,
                    "timezone": "Asia/Kolkata"
                }
            ],
            "parallel": true,
            "max_concurrent": 5
        })
    }

    /// User registration request
    pub fn register_request() -> serde_json::Value {
        json!({
            "email": "test@example.com",
            "password": "secure_password_123",
            "full_name": "Test User"
        })
    }

    /// User login request
    pub fn login_request() -> serde_json::Value {
        json!({
            "email": "test@example.com",
            "password": "secure_password_123"
        })
    }

    /// Mock JWT token for testing
    pub fn mock_jwt_token() -> String {
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IlRlc3QgVXNlciIsImlhdCI6MTUxNjIzOTAyMn0.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".to_string()
    }

    /// Mock API key for testing
    pub fn mock_api_key() -> String {
        "test_api_key_1234567890abcdef".to_string()
    }
}

/// Assertion helpers for testing
pub mod assertions {
    use serde_json::Value;

    /// Assert JSON contains key
    pub fn assert_json_contains(json: &Value, key: &str) {
        assert!(
            json.get(key).is_some(),
            "Expected JSON to contain key '{}'",
            key
        );
    }

    /// Assert JSON value equals
    pub fn assert_json_eq(json: &Value, key: &str, expected: &Value) {
        let actual = json.get(key).expect(&format!("Key '{}' not found", key));
        assert_eq!(actual, expected, "JSON value mismatch for key '{}'", key);
    }

    /// Assert JSON string value equals
    pub fn assert_json_str(json: &Value, key: &str, expected: &str) {
        let actual = json
            .get(key)
            .and_then(|v| v.as_str())
            .expect(&format!("Key '{}' not found or not a string", key));
        assert_eq!(actual, expected, "JSON string mismatch for key '{}'", key);
    }

    /// Assert JSON number value equals
    pub fn assert_json_num(json: &Value, key: &str, expected: f64) {
        let actual = json
            .get(key)
            .and_then(|v| v.as_f64())
            .expect(&format!("Key '{}' not found or not a number", key));
        assert_eq!(actual, expected, "JSON number mismatch for key '{}'", key);
    }

    /// Assert JSON boolean value equals
    pub fn assert_json_bool(json: &Value, key: &str, expected: bool) {
        let actual = json
            .get(key)
            .and_then(|v| v.as_bool())
            .expect(&format!("Key '{}' not found or not a boolean", key));
        assert_eq!(actual, expected, "JSON boolean mismatch for key '{}'", key);
    }

    /// Assert JSON array length
    pub fn assert_json_array_len(json: &Value, key: &str, expected_len: usize) {
        let array = json
            .get(key)
            .and_then(|v| v.as_array())
            .expect(&format!("Key '{}' not found or not an array", key));
        assert_eq!(
            array.len(),
            expected_len,
            "JSON array length mismatch for key '{}'",
            key
        );
    }

    /// Assert error response format
    pub fn assert_error_response(json: &Value) {
        assert_json_contains(json, "error");
        assert_json_contains(json, "message");
        assert_json_contains(json, "code");
    }

    /// Assert success response format
    pub fn assert_success_response(json: &Value) {
        assert_json_contains(json, "success");
        let success = json.get("success").unwrap().as_bool().unwrap();
        assert!(success, "Expected success to be true");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::fixtures::*;
    use super::assertions::*;

    #[test]
    fn test_fixtures() {
        let request = panchanga_request();
        assert_json_str(&request, "date", "2025-01-27");
        assert_json_num(&request, "latitude", 19.0760);
        assert_json_num(&request, "longitude", 72.8777);
    }

    #[test]
    fn test_assertions() {
        let json = serde_json::json!({
            "success": true,
            "count": 5,
            "items": [1, 2, 3]
        });

        assert_json_contains(&json, "success");
        assert_json_bool(&json, "success", true);
        assert_json_num(&json, "count", 5.0);
        assert_json_array_len(&json, "items", 3);
    }

    #[test]
    #[should_panic(expected = "Expected JSON to contain key")]
    fn test_assertion_missing_key() {
        let json = serde_json::json!({"key": "value"});
        assert_json_contains(&json, "missing");
    }
}
