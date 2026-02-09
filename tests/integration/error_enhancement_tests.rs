//! Tests for error enhancement and enrichment functionality

#[cfg(test)]
mod error_enhancer_tests {
    use noesis_api::error_enhancer::{ErrorContext, ErrorEnhancer, EnhancedError};
    use axum::http::StatusCode;

    #[test]
    fn test_error_enhancement_with_context() {
        let context = ErrorContext::new()
            .with_method("POST".to_string())
            .with_path("/api/v1/engines/panchanga/calculate".to_string())
            .with_user_id("user_123".to_string())
            .add_metadata("engine", "panchanga");

        let enhanced = ErrorEnhancer::enhance(
            "validation_error",
            "Invalid date format",
            "INVALID_DATE_FORMAT",
            StatusCode::BAD_REQUEST,
            context,
        );

        assert_eq!(enhanced.error, "validation_error");
        assert_eq!(enhanced.code, "INVALID_DATE_FORMAT");
        assert_eq!(enhanced.status, 400);
        assert!(enhanced.context.method.is_some());
        assert!(enhanced.context.path.is_some());
        assert!(enhanced.context.user_id.is_some());
        assert!(!enhanced.suggestions.is_empty());
    }

    #[test]
    fn test_validation_error_suggestions() {
        let context = ErrorContext::new()
            .with_path("/api/v1/engines/panchanga/calculate".to_string());

        let enhanced = ErrorEnhancer::enhance(
            "validation_error",
            "Invalid request",
            "VALIDATION_FAILED",
            StatusCode::BAD_REQUEST,
            context,
        );

        // Should include suggestions for validation errors
        assert!(enhanced.suggestions.iter().any(|s| 
            s.contains("format") || s.contains("field") || s.contains("documentation")
        ));
    }

    #[test]
    fn test_authentication_error_suggestions() {
        let context = ErrorContext::new();

        let enhanced = ErrorEnhancer::enhance(
            "authentication_error",
            "Invalid token",
            "AUTH_FAILED",
            StatusCode::UNAUTHORIZED,
            context,
        );

        // Should include auth-specific suggestions
        assert!(enhanced.suggestions.iter().any(|s| 
            s.contains("API key") || s.contains("token") || s.contains("expired")
        ));
    }

    #[test]
    fn test_rate_limit_error_suggestions() {
        let context = ErrorContext::new();

        let enhanced = ErrorEnhancer::enhance(
            "rate_limit_exceeded",
            "Too many requests",
            "RATE_LIMIT_EXCEEDED",
            StatusCode::TOO_MANY_REQUESTS,
            context,
        );

        // Should include rate limiting suggestions
        assert!(enhanced.suggestions.iter().any(|s| 
            s.contains("Wait") || s.contains("backoff") || s.contains("rate limit")
        ));
    }

    #[test]
    fn test_calculation_error_suggestions() {
        let context = ErrorContext::new();

        let enhanced = ErrorEnhancer::enhance(
            "calculation_error",
            "Calculation failed",
            "CALCULATION_FAILED",
            StatusCode::INTERNAL_SERVER_ERROR,
            context,
        );

        // Should include calculation-specific suggestions
        assert!(enhanced.suggestions.iter().any(|s| 
            s.contains("parameter") || s.contains("range") || s.contains("precision")
        ));
    }

    #[test]
    fn test_engine_not_found_suggestions() {
        let context = ErrorContext::new();

        let enhanced = ErrorEnhancer::enhance(
            "engine_not_found",
            "Engine not found",
            "ENGINE_NOT_FOUND",
            StatusCode::NOT_FOUND,
            context,
        );

        // Should suggest checking available engines
        assert!(enhanced.suggestions.iter().any(|s| 
            s.contains("engine") && s.contains("available")
        ));
    }

    #[test]
    fn test_workflow_not_found_suggestions() {
        let context = ErrorContext::new();

        let enhanced = ErrorEnhancer::enhance(
            "workflow_not_found",
            "Workflow not found",
            "WORKFLOW_NOT_FOUND",
            StatusCode::NOT_FOUND,
            context,
        );

        // Should suggest checking available workflows
        assert!(enhanced.suggestions.iter().any(|s| 
            s.contains("workflow") && s.contains("available")
        ));
    }

    #[test]
    fn test_phase_access_denied_suggestions() {
        let context = ErrorContext::new();

        let enhanced = ErrorEnhancer::enhance(
            "phase_access_denied",
            "Insufficient phase level",
            "PHASE_ACCESS_DENIED",
            StatusCode::FORBIDDEN,
            context,
        );

        // Should suggest upgrading access level
        assert!(enhanced.suggestions.iter().any(|s| 
            s.contains("phase") || s.contains("access") || s.contains("upgrade")
        ));
    }

    #[test]
    fn test_internal_error_suggestions() {
        let context = ErrorContext::new();

        let enhanced = ErrorEnhancer::enhance(
            "internal_error",
            "Server error",
            "INTERNAL_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
            context,
        );

        // Should suggest retry and support contact
        assert!(enhanced.suggestions.iter().any(|s| 
            s.contains("retry") || s.contains("support") || s.contains("server")
        ));
    }

    #[test]
    fn test_error_context_metadata() {
        let context = ErrorContext::new()
            .add_metadata("engine_id", "panchanga")
            .add_metadata("calculation_type", "daily")
            .add_metadata("precision", "high");

        assert_eq!(context.metadata.get("engine_id"), Some(&"panchanga".to_string()));
        assert_eq!(context.metadata.get("calculation_type"), Some(&"daily".to_string()));
        assert_eq!(context.metadata.get("precision"), Some(&"high".to_string()));
    }

    #[test]
    fn test_error_timestamp_format() {
        let context = ErrorContext::new();
        let enhanced = ErrorEnhancer::enhance(
            "test_error",
            "Test message",
            "TEST_CODE",
            StatusCode::BAD_REQUEST,
            context,
        );

        // Should be valid RFC3339 timestamp
        assert!(chrono::DateTime::parse_from_rfc3339(&enhanced.timestamp).is_ok());
    }

    #[test]
    fn test_multiple_metadata_entries() {
        let mut context = ErrorContext::new();
        
        for i in 0..10 {
            context = context.add_metadata(
                format!("key_{}", i),
                format!("value_{}", i),
            );
        }

        assert_eq!(context.metadata.len(), 10);
        assert_eq!(context.metadata.get("key_0"), Some(&"value_0".to_string()));
        assert_eq!(context.metadata.get("key_9"), Some(&"value_9".to_string()));
    }
}
