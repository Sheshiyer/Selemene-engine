//! Error enhancement and enrichment agent
//!
//! Provides contextual information gathering and error transformation
//! to make errors more actionable and debuggable.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tracing::warn;

/// Enhanced error response with rich context
#[derive(Debug, Serialize, Deserialize)]
pub struct EnhancedError {
    /// Error type/category
    pub error: String,
    /// Human-readable error message
    pub message: String,
    /// Machine-readable error code
    pub code: String,
    /// HTTP status code
    pub status: u16,
    /// Additional context and metadata
    pub context: ErrorContext,
    /// Timestamp when error occurred
    pub timestamp: String,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Suggestions for resolving the error
    pub suggestions: Vec<String>,
}

/// Error context with debugging information
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Request method
    pub method: Option<String>,
    /// Request path
    pub path: Option<String>,
    /// User ID if authenticated
    pub user_id: Option<String>,
    /// Additional key-value metadata
    pub metadata: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            method: None,
            path: None,
            user_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_method(mut self, method: String) -> Self {
        self.method = Some(method);
        self
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Error enhancer agent
pub struct ErrorEnhancer;

impl ErrorEnhancer {
    /// Enhance a basic error with context and suggestions
    pub fn enhance(
        error_type: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
        status: StatusCode,
        context: ErrorContext,
    ) -> EnhancedError {
        let error = error_type.into();
        let code_str = code.into();
        let suggestions = Self::generate_suggestions(&error, &code_str, &context);

        EnhancedError {
            error: error.clone(),
            message: message.into(),
            code: code_str,
            status: status.as_u16(),
            context,
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id: None,
            suggestions,
        }
    }

    /// Generate helpful suggestions based on error type
    fn generate_suggestions(
        error_type: &str,
        code: &str,
        context: &ErrorContext,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        match error_type {
            "validation_error" => {
                suggestions.push("Check the request payload format and field types".to_string());
                suggestions.push("Ensure all required fields are present".to_string());
                if let Some(path) = &context.path {
                    suggestions.push(format!("Review the API documentation for endpoint: {}", path));
                }
            }
            "authentication_error" => {
                suggestions.push("Verify your API key or JWT token is valid".to_string());
                suggestions.push("Check if the token has expired".to_string());
                suggestions.push("Ensure the Authorization header is properly formatted".to_string());
            }
            "rate_limit_exceeded" => {
                suggestions.push("Wait before making additional requests".to_string());
                suggestions.push("Consider implementing exponential backoff".to_string());
                suggestions.push("Review your rate limit tier and upgrade if needed".to_string());
            }
            "calculation_error" => {
                suggestions.push("Verify input parameters are within valid ranges".to_string());
                suggestions.push("Check if date and coordinate values are correct".to_string());
                suggestions.push("Try reducing precision level if using extreme precision".to_string());
            }
            "engine_not_found" => {
                suggestions.push("Verify the engine ID is correct".to_string());
                suggestions.push("Check available engines via /api/v1/engines endpoint".to_string());
            }
            "workflow_not_found" => {
                suggestions.push("Verify the workflow ID is correct".to_string());
                suggestions.push("List available workflows via /api/v1/workflows endpoint".to_string());
            }
            "phase_access_denied" => {
                suggestions.push("Your user phase level is insufficient for this operation".to_string());
                suggestions.push("Contact support to upgrade your access level".to_string());
            }
            "internal_error" => {
                suggestions.push("This is likely a server-side issue".to_string());
                suggestions.push("Please retry the request".to_string());
                suggestions.push("Contact support if the issue persists".to_string());
            }
            _ => {
                if code.contains("TIMEOUT") {
                    suggestions.push("The request took too long to process".to_string());
                    suggestions.push("Try reducing the complexity of your request".to_string());
                }
            }
        }

        suggestions
    }

    /// Extract context from request
    pub fn extract_context_from_request(req: &Request) -> ErrorContext {
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let user_id = req
            .headers()
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let mut context = ErrorContext::new()
            .with_method(method)
            .with_path(path);

        if let Some(uid) = user_id {
            context = context.with_user_id(uid);
        }

        // Add query parameters if present
        if let Some(query) = req.uri().query() {
            context = context.add_metadata("query", query);
        }

        context
    }
}

/// Middleware for automatic error enhancement
pub async fn error_enhancement_middleware(
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let context = ErrorEnhancer::extract_context_from_request(&req);
    
    // Store context in extensions for handlers to use
    // Note: This would require adding to request extensions properly
    
    let response = next.run(req).await;
    
    // Log errors for monitoring
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        let duration = start.elapsed();
        warn!(
            status = status.as_u16(),
            method = ?context.method,
            path = ?context.path,
            duration_ms = duration.as_millis(),
            "Request failed"
        );
    }
    
    response
}

/// Convert noesis_core::EngineError to EnhancedError
pub fn enhance_engine_error(
    err: noesis_core::EngineError,
    context: ErrorContext,
) -> EnhancedError {
    use noesis_core::EngineError;

    match err {
        EngineError::ValidationError(msg) => {
            ErrorEnhancer::enhance(
                "validation_error",
                msg,
                "VALIDATION_FAILED",
                StatusCode::BAD_REQUEST,
                context,
            )
        }
        EngineError::AuthError(msg) => {
            ErrorEnhancer::enhance(
                "authentication_error",
                msg,
                "AUTH_FAILED",
                StatusCode::UNAUTHORIZED,
                context,
            )
        }
        EngineError::RateLimitExceeded => {
            ErrorEnhancer::enhance(
                "rate_limit_exceeded",
                "Rate limit exceeded. Please try again later.",
                "RATE_LIMIT_EXCEEDED",
                StatusCode::TOO_MANY_REQUESTS,
                context,
            )
        }
        EngineError::EngineNotFound(engine) => {
            ErrorEnhancer::enhance(
                "engine_not_found",
                format!("Engine '{}' not found", engine),
                "ENGINE_NOT_FOUND",
                StatusCode::NOT_FOUND,
                context,
            )
        }
        EngineError::WorkflowNotFound(workflow) => {
            ErrorEnhancer::enhance(
                "workflow_not_found",
                format!("Workflow '{}' not found", workflow),
                "WORKFLOW_NOT_FOUND",
                StatusCode::NOT_FOUND,
                context,
            )
        }
        EngineError::PhaseAccessDenied { required, current } => {
            ErrorEnhancer::enhance(
                "phase_access_denied",
                format!("Access denied. Required phase: {}, current phase: {}", required, current),
                "PHASE_ACCESS_DENIED",
                StatusCode::FORBIDDEN,
                context,
            )
        }
        EngineError::CalculationError(msg) => {
            ErrorEnhancer::enhance(
                "calculation_error",
                msg,
                "CALCULATION_FAILED",
                StatusCode::INTERNAL_SERVER_ERROR,
                context,
            )
        }
        EngineError::CacheError(msg) => {
            ErrorEnhancer::enhance(
                "cache_error",
                msg,
                "CACHE_ERROR",
                StatusCode::INTERNAL_SERVER_ERROR,
                context,
            )
        }
        EngineError::ConfigError(msg) => {
            ErrorEnhancer::enhance(
                "configuration_error",
                msg,
                "CONFIG_ERROR",
                StatusCode::INTERNAL_SERVER_ERROR,
                context,
            )
        }
        EngineError::BridgeError(msg) => {
            ErrorEnhancer::enhance(
                "bridge_error",
                msg,
                "BRIDGE_ERROR",
                StatusCode::INTERNAL_SERVER_ERROR,
                context,
            )
        }
        EngineError::SwissEphemerisError(msg) => {
            ErrorEnhancer::enhance(
                "ephemeris_error",
                msg,
                "EPHEMERIS_ERROR",
                StatusCode::INTERNAL_SERVER_ERROR,
                context,
            )
        }
        EngineError::InternalError(msg) => {
            ErrorEnhancer::enhance(
                "internal_error",
                msg,
                "INTERNAL_ERROR",
                StatusCode::INTERNAL_SERVER_ERROR,
                context,
            )
        }
    }
}

impl IntoResponse for EnhancedError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_builder() {
        let context = ErrorContext::new()
            .with_method("POST".to_string())
            .with_path("/api/v1/calculate".to_string())
            .with_user_id("user123".to_string())
            .add_metadata("engine", "panchanga");

        assert_eq!(context.method, Some("POST".to_string()));
        assert_eq!(context.path, Some("/api/v1/calculate".to_string()));
        assert_eq!(context.user_id, Some("user123".to_string()));
        assert_eq!(context.metadata.get("engine"), Some(&"panchanga".to_string()));
    }

    #[test]
    fn test_error_enhancement() {
        let context = ErrorContext::new()
            .with_method("POST".to_string())
            .with_path("/api/v1/engines/panchanga/calculate".to_string());

        let enhanced = ErrorEnhancer::enhance(
            "validation_error",
            "Invalid date format",
            "INVALID_DATE",
            StatusCode::BAD_REQUEST,
            context,
        );

        assert_eq!(enhanced.error, "validation_error");
        assert_eq!(enhanced.code, "INVALID_DATE");
        assert_eq!(enhanced.status, 400);
        assert!(!enhanced.suggestions.is_empty());
    }

    #[test]
    fn test_suggestion_generation() {
        let context = ErrorContext::new();

        let suggestions = ErrorEnhancer::generate_suggestions(
            "authentication_error",
            "AUTH_FAILED",
            &context,
        );

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("API key") || s.contains("token")));
    }

    #[test]
    fn test_rate_limit_suggestions() {
        let context = ErrorContext::new();
        let suggestions = ErrorEnhancer::generate_suggestions(
            "rate_limit_exceeded",
            "RATE_LIMIT",
            &context,
        );

        assert!(suggestions.iter().any(|s| s.contains("Wait") || s.contains("backoff")));
    }
}
