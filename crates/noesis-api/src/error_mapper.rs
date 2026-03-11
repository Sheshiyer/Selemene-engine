use axum::{http::StatusCode, Json};
use noesis_core::EngineError;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub status: u16,
    pub error_code: String,
    pub message: String,
    // Preserve the legacy field while the API migrates to `message`.
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub trace_id: String,
}

pub struct ErrorMapper;

impl ErrorMapper {
    pub fn map(err: EngineError) -> (StatusCode, Json<ErrorResponse>) {
        let err_display = err.to_string();

        let (status, error_code, message, details) = match &err {
            EngineError::EngineNotFound(id) => (
                StatusCode::NOT_FOUND,
                "ENGINE_NOT_FOUND".to_string(),
                err.to_string(),
                Some(serde_json::json!({ "engine_id": id })),
            ),
            EngineError::WorkflowNotFound(id) => (
                StatusCode::NOT_FOUND,
                "WORKFLOW_NOT_FOUND".to_string(),
                err.to_string(),
                Some(serde_json::json!({ "workflow_id": id })),
            ),
            EngineError::PhaseAccessDenied { required, current } => (
                StatusCode::FORBIDDEN,
                "PHASE_ACCESS_DENIED".to_string(),
                err.to_string(),
                Some(serde_json::json!({
                    "required_phase": required,
                    "current_phase": current
                })),
            ),
            EngineError::AuthError(msg) => (
                StatusCode::UNAUTHORIZED,
                "AUTH_ERROR".to_string(),
                err.to_string(),
                Some(serde_json::json!({ "reason": msg })),
            ),
            EngineError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED".to_string(),
                err.to_string(),
                None,
            ),
            EngineError::ValidationError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR".to_string(),
                err.to_string(),
                Some(serde_json::json!({ "validation_message": msg })),
            ),
            EngineError::CalculationError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CALCULATION_ERROR".to_string(),
                "An internal calculation error occurred".to_string(),
                None,
            ),
            EngineError::CacheError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CACHE_ERROR".to_string(),
                "An internal cache error occurred".to_string(),
                None,
            ),
            EngineError::ConfigError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIG_ERROR".to_string(),
                "An internal configuration error occurred".to_string(),
                None,
            ),
            EngineError::BridgeError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "BRIDGE_ERROR".to_string(),
                "An internal bridge error occurred".to_string(),
                None,
            ),
            EngineError::SwissEphemerisError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SWISS_EPHEMERIS_ERROR".to_string(),
                "An internal ephemeris error occurred".to_string(),
                None,
            ),
            EngineError::InternalError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR".to_string(),
                "An internal error occurred".to_string(),
                None,
            ),
        };

        Self::response_internal(status, error_code, message, details, Some(err_display))
    }

    pub fn response(
        status: StatusCode,
        error_code: impl Into<String>,
        message: impl Into<String>,
        details: Option<serde_json::Value>,
    ) -> (StatusCode, Json<ErrorResponse>) {
        Self::response_internal(status, error_code.into(), message.into(), details, None)
    }

    fn response_internal(
        status: StatusCode,
        error_code: String,
        message: String,
        details: Option<serde_json::Value>,
        sentry_message: Option<String>,
    ) -> (StatusCode, Json<ErrorResponse>) {
        let trace_id = Self::current_trace_id();

        if status.is_server_error() {
            sentry::configure_scope(|scope| {
                scope.set_tag("error_code", &error_code);
                scope.set_tag("trace_id", &trace_id);
            });
            sentry::capture_message(
                sentry_message.as_deref().unwrap_or(&message),
                sentry::Level::Error,
            );
        }

        (
            status,
            Json(ErrorResponse {
                status: status.as_u16(),
                error_code,
                message: message.clone(),
                error: message,
                details,
                trace_id,
            }),
        )
    }

    fn current_trace_id() -> String {
        tracing::Span::current()
            .id()
            .map(|id| format!("{id:?}"))
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorMapper;
    use axum::http::StatusCode;
    use noesis_core::EngineError;

    #[test]
    fn map_includes_expanded_schema_fields() {
        let (_status, body) = ErrorMapper::map(EngineError::EngineNotFound("missing".to_string()));
        let response = body.0;

        assert_eq!(response.status, 404);
        assert_eq!(response.error_code, "ENGINE_NOT_FOUND");
        assert_eq!(response.message, response.error);
        assert!(!response.trace_id.is_empty());
    }

    #[test]
    fn response_helper_includes_trace_id() {
        let (_status, body) =
            ErrorMapper::response(StatusCode::BAD_REQUEST, "BAD_INPUT", "bad input", None);
        assert_eq!(body.0.status, 400);
        assert_eq!(body.0.message, "bad input");
        assert!(!body.0.trace_id.is_empty());
    }
}
