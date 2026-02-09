//! Payload validation middleware and utilities
//!
//! Provides comprehensive request validation including:
//! - JSON schema validation
//! - Request body size limits
//! - Content-type checking
//! - Field-level validation rules

use axum::{
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Maximum request body size in bytes (10MB by default)
pub const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

/// Validation result with detailed field errors
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
        }
    }

    pub fn add_error(&mut self, field: impl Into<String>, message: impl Into<String>, code: impl Into<String>) {
        self.valid = false;
        self.errors.push(ValidationError {
            field: field.into(),
            message: message.into(),
            code: code.into(),
        });
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Payload validator trait for custom validation logic
pub trait PayloadValidator {
    fn validate(&self, payload: &Value) -> ValidationResult;
}

/// Request size validator
pub struct SizeValidator {
    max_size: usize,
}

impl SizeValidator {
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }

    pub fn validate_size(&self, size: usize) -> Result<(), ValidationError> {
        if size > self.max_size {
            return Err(ValidationError {
                field: "_request".to_string(),
                message: format!("Request body too large. Maximum size is {} bytes", self.max_size),
                code: "REQUEST_TOO_LARGE".to_string(),
            });
        }
        Ok(())
    }
}

/// Content-type validator
pub struct ContentTypeValidator {
    allowed_types: Vec<String>,
}

impl ContentTypeValidator {
    pub fn new(allowed_types: Vec<String>) -> Self {
        Self { allowed_types }
    }

    pub fn json() -> Self {
        Self {
            allowed_types: vec!["application/json".to_string()],
        }
    }

    pub fn validate(&self, content_type: Option<&str>) -> Result<(), ValidationError> {
        match content_type {
            None => Err(ValidationError {
                field: "_request".to_string(),
                message: "Content-Type header is missing".to_string(),
                code: "MISSING_CONTENT_TYPE".to_string(),
            }),
            Some(ct) => {
                let ct_lower = ct.to_lowercase();
                if !self.allowed_types.iter().any(|allowed| ct_lower.starts_with(allowed)) {
                    Err(ValidationError {
                        field: "_request".to_string(),
                        message: format!("Invalid Content-Type. Expected one of: {}", self.allowed_types.join(", ")),
                        code: "INVALID_CONTENT_TYPE".to_string(),
                    })
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Field validators for common validation patterns
pub struct FieldValidators;

impl FieldValidators {
    /// Validate required field exists
    pub fn required(value: &Value, field: &str) -> Result<(), ValidationError> {
        if value.get(field).is_none() || value[field].is_null() {
            return Err(ValidationError {
                field: field.to_string(),
                message: format!("Field '{}' is required", field),
                code: "REQUIRED_FIELD".to_string(),
            });
        }
        Ok(())
    }

    /// Validate string length
    pub fn string_length(
        value: &str,
        field: &str,
        min: Option<usize>,
        max: Option<usize>,
    ) -> Result<(), ValidationError> {
        let len = value.len();
        if let Some(min_len) = min {
            if len < min_len {
                return Err(ValidationError {
                    field: field.to_string(),
                    message: format!("Field '{}' must be at least {} characters", field, min_len),
                    code: "MIN_LENGTH".to_string(),
                });
            }
        }
        if let Some(max_len) = max {
            if len > max_len {
                return Err(ValidationError {
                    field: field.to_string(),
                    message: format!("Field '{}' must be at most {} characters", field, max_len),
                    code: "MAX_LENGTH".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Validate numeric range
    pub fn numeric_range(
        value: f64,
        field: &str,
        min: Option<f64>,
        max: Option<f64>,
    ) -> Result<(), ValidationError> {
        if let Some(min_val) = min {
            if value < min_val {
                return Err(ValidationError {
                    field: field.to_string(),
                    message: format!("Field '{}' must be at least {}", field, min_val),
                    code: "MIN_VALUE".to_string(),
                });
            }
        }
        if let Some(max_val) = max {
            if value > max_val {
                return Err(ValidationError {
                    field: field.to_string(),
                    message: format!("Field '{}' must be at most {}", field, max_val),
                    code: "MAX_VALUE".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Validate email format
    pub fn email(value: &str, field: &str) -> Result<(), ValidationError> {
        if !value.contains('@') || !value.contains('.') {
            return Err(ValidationError {
                field: field.to_string(),
                message: format!("Field '{}' must be a valid email address", field),
                code: "INVALID_EMAIL".to_string(),
            });
        }
        Ok(())
    }

    /// Validate date format (ISO 8601)
    pub fn date_format(value: &str, field: &str) -> Result<(), ValidationError> {
        if chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").is_err() {
            return Err(ValidationError {
                field: field.to_string(),
                message: format!("Field '{}' must be a valid date in YYYY-MM-DD format", field),
                code: "INVALID_DATE_FORMAT".to_string(),
            });
        }
        Ok(())
    }

    /// Validate coordinate values (latitude/longitude)
    pub fn latitude(value: f64, field: &str) -> Result<(), ValidationError> {
        Self::numeric_range(value, field, Some(-90.0), Some(90.0))
    }

    pub fn longitude(value: f64, field: &str) -> Result<(), ValidationError> {
        Self::numeric_range(value, field, Some(-180.0), Some(180.0))
    }
}

/// Middleware for request body size validation
pub async fn validate_request_size(
    req: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let content_length = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok());

    if let Some(size) = content_length {
        let validator = SizeValidator::new(MAX_BODY_SIZE);
        if let Err(err) = validator.validate_size(size) {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(serde_json::json!({
                    "error": "payload_too_large",
                    "message": err.message,
                    "code": err.code,
                })),
            ));
        }
    }

    Ok(next.run(req).await)
}

/// Middleware for content-type validation
pub async fn validate_content_type(
    req: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Skip validation for GET requests
    if req.method() == axum::http::Method::GET {
        return Ok(next.run(req).await);
    }

    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    let validator = ContentTypeValidator::json();
    if let Err(err) = validator.validate(content_type) {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(serde_json::json!({
                "error": "unsupported_media_type",
                "message": err.message,
                "code": err.code,
            })),
        ));
    }

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());
        assert_eq!(result.errors.len(), 0);

        result.add_error("email", "Invalid email format", "INVALID_EMAIL");
        assert!(!result.is_valid());
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].field, "email");
    }

    #[test]
    fn test_size_validator() {
        let validator = SizeValidator::new(1000);
        assert!(validator.validate_size(500).is_ok());
        assert!(validator.validate_size(1500).is_err());
    }

    #[test]
    fn test_content_type_validator() {
        let validator = ContentTypeValidator::json();
        assert!(validator.validate(Some("application/json")).is_ok());
        assert!(validator.validate(Some("application/json; charset=utf-8")).is_ok());
        assert!(validator.validate(Some("text/html")).is_err());
        assert!(validator.validate(None).is_err());
    }

    #[test]
    fn test_field_validators() {
        // Test string length
        assert!(FieldValidators::string_length("hello", "test", Some(3), Some(10)).is_ok());
        assert!(FieldValidators::string_length("hi", "test", Some(3), None).is_err());
        assert!(FieldValidators::string_length("hello world!", "test", None, Some(10)).is_err());

        // Test numeric range
        assert!(FieldValidators::numeric_range(5.0, "test", Some(0.0), Some(10.0)).is_ok());
        assert!(FieldValidators::numeric_range(-1.0, "test", Some(0.0), None).is_err());
        assert!(FieldValidators::numeric_range(11.0, "test", None, Some(10.0)).is_err());

        // Test email
        assert!(FieldValidators::email("test@example.com", "email").is_ok());
        assert!(FieldValidators::email("invalid", "email").is_err());

        // Test date format
        assert!(FieldValidators::date_format("2025-01-27", "date").is_ok());
        assert!(FieldValidators::date_format("2025/01/27", "date").is_err());
        assert!(FieldValidators::date_format("invalid", "date").is_err());

        // Test coordinates
        assert!(FieldValidators::latitude(45.0, "lat").is_ok());
        assert!(FieldValidators::latitude(100.0, "lat").is_err());
        assert!(FieldValidators::longitude(120.0, "lon").is_ok());
        assert!(FieldValidators::longitude(200.0, "lon").is_err());
    }
}
