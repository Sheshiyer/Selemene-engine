use axum::{http::StatusCode, Json};
use noesis_core::EngineError;
use noesis_metrics::record_api_error;
use serde::Serialize;
use std::future::Future;
use utoipa::ToSchema;

tokio::task_local! {
    static REQUEST_TRACE_ID: String;
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub contract_version: String,
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
    pub async fn with_request_trace_id<T>(trace_id: String, future: impl Future<Output = T>) -> T {
        REQUEST_TRACE_ID.scope(trace_id, future).await
    }

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
            EngineError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE".to_string(),
                msg.clone(),
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
        record_api_error(&error_code);
        Self::record_sentry_context(
            status,
            &error_code,
            &message,
            &trace_id,
            sentry_message.as_deref().unwrap_or(&message),
        );

        (
            status,
            Json(ErrorResponse {
                contract_version: noesis_core::contract::CONTRACT_VERSION.to_string(),
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
        REQUEST_TRACE_ID
            .try_with(Clone::clone)
            .ok()
            .or_else(|| tracing::Span::current().id().map(|id| format!("{id:?}")))
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
    }

    fn record_sentry_context(
        status: StatusCode,
        error_code: &str,
        message: &str,
        trace_id: &str,
        sentry_message: &str,
    ) {
        use sentry::protocol::{Breadcrumb, Map, Value};

        let mut data = Map::new();
        data.insert("status".to_string(), Value::from(status.as_u16() as i64));
        data.insert(
            "error_code".to_string(),
            Value::from(error_code.to_string()),
        );
        data.insert("trace_id".to_string(), Value::from(trace_id.to_string()));

        sentry::add_breadcrumb(Breadcrumb {
            category: Some("api.error".to_string()),
            ty: "error".to_string(),
            level: if status.is_server_error() {
                sentry::Level::Error
            } else {
                sentry::Level::Warning
            },
            message: Some(format!("{}: {}", error_code, message)),
            data,
            ..Default::default()
        });

        if status.is_server_error() {
            sentry::configure_scope(|scope| {
                scope.set_tag("error_code", error_code);
                scope.set_tag("trace_id", trace_id);
            });
            sentry::capture_message(sentry_message, sentry::Level::Error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorMapper;
    use axum::http::StatusCode;
    use noesis_core::EngineError;
    use sentry::{test::with_captured_events_options, ClientOptions, Level};
    use serde_json::json;

    fn run_in_runtime<T>(future: impl std::future::Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("test runtime should build")
            .block_on(future)
    }

    fn assert_error_mapper_exhaustive(err: &EngineError) {
        match err {
            EngineError::EngineNotFound(_) => {}
            EngineError::WorkflowNotFound(_) => {}
            EngineError::PhaseAccessDenied { .. } => {}
            EngineError::AuthError(_) => {}
            EngineError::RateLimitExceeded => {}
            EngineError::ValidationError(_) => {}
            EngineError::CalculationError(_) => {}
            EngineError::CacheError(_) => {}
            EngineError::ConfigError(_) => {}
            EngineError::BridgeError(_) => {}
            EngineError::SwissEphemerisError(_) => {}
            EngineError::InternalError(_) => {}
            EngineError::ServiceUnavailable(_) => {}
        }
    }

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

    #[test]
    fn error_exhaustiveness_maps_all_engine_error_variants() {
        let cases = vec![
            (
                EngineError::EngineNotFound("missing".to_string()),
                StatusCode::NOT_FOUND,
                "ENGINE_NOT_FOUND",
            ),
            (
                EngineError::WorkflowNotFound("wf".to_string()),
                StatusCode::NOT_FOUND,
                "WORKFLOW_NOT_FOUND",
            ),
            (
                EngineError::PhaseAccessDenied {
                    required: 2,
                    current: 1,
                },
                StatusCode::FORBIDDEN,
                "PHASE_ACCESS_DENIED",
            ),
            (
                EngineError::AuthError("bad token".to_string()),
                StatusCode::UNAUTHORIZED,
                "AUTH_ERROR",
            ),
            (
                EngineError::RateLimitExceeded,
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
            ),
            (
                EngineError::ValidationError("invalid".to_string()),
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
            ),
            (
                EngineError::CalculationError("calc".to_string()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "CALCULATION_ERROR",
            ),
            (
                EngineError::CacheError("cache".to_string()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "CACHE_ERROR",
            ),
            (
                EngineError::ConfigError("config".to_string()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIG_ERROR",
            ),
            (
                EngineError::BridgeError("bridge".to_string()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "BRIDGE_ERROR",
            ),
            (
                EngineError::SwissEphemerisError("swiss".to_string()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "SWISS_EPHEMERIS_ERROR",
            ),
            (
                EngineError::InternalError("internal".to_string()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
            ),
            (
                EngineError::ServiceUnavailable("unavailable".to_string()),
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
            ),
        ];

        assert_eq!(cases.len(), 13);

        for (err, expected_status, expected_code) in cases {
            assert_error_mapper_exhaustive(&err);
            let (status, body) = ErrorMapper::map(err);
            assert_eq!(status, expected_status);
            assert_eq!(body.0.error_code, expected_code);
        }
    }

    #[tokio::test]
    async fn request_trace_id_scope_is_reused_in_error_response() {
        let (_status, body) = ErrorMapper::with_request_trace_id("trace-123".to_string(), async {
            ErrorMapper::response(
                StatusCode::BAD_REQUEST,
                "BAD_INPUT",
                "bad input",
                Some(json!({"field": "date"})),
            )
        })
        .await;

        assert_eq!(body.0.trace_id, "trace-123");
    }

    #[test]
    fn sentry_captures_event_for_5xx_errors() {
        let events = with_captured_events_options(
            || {
                run_in_runtime(async {
                    let (_status, body) =
                        ErrorMapper::with_request_trace_id("trace-500".to_string(), async {
                            ErrorMapper::map(EngineError::BridgeError(
                                "sidecar timeout".to_string(),
                            ))
                        })
                        .await;
                    assert_eq!(body.0.trace_id, "trace-500");
                });
            },
            ClientOptions::default(),
        );

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.level, Level::Error);
        assert_eq!(
            event.tags.get("error_code").map(String::as_str),
            Some("BRIDGE_ERROR")
        );
        assert_eq!(
            event.tags.get("trace_id").map(String::as_str),
            Some("trace-500")
        );
        assert!(
            event
                .message
                .as_deref()
                .unwrap_or("")
                .contains("Bridge error"),
            "expected bridge error message, got {:?}",
            event.message
        );
        assert!(
            event.breadcrumbs.iter().any(|crumb| {
                crumb.category.as_deref() == Some("api.error")
                    && crumb
                        .message
                        .as_deref()
                        .unwrap_or("")
                        .contains("BRIDGE_ERROR")
            }),
            "expected api.error breadcrumb on captured 5xx event"
        );
    }

    #[test]
    fn sentry_records_breadcrumb_only_for_4xx_errors() {
        let events = with_captured_events_options(
            || {
                run_in_runtime(async {
                    let (_status, body) =
                        ErrorMapper::with_request_trace_id("trace-404".to_string(), async {
                            ErrorMapper::map(EngineError::EngineNotFound("missing".to_string()))
                        })
                        .await;
                    assert_eq!(body.0.trace_id, "trace-404");
                });

                sentry::capture_message("flush breadcrumbs", Level::Info);
            },
            ClientOptions::default(),
        );

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.message.as_deref(), Some("flush breadcrumbs"));
        assert!(
            !event.tags.contains_key("error_code"),
            "4xx breadcrumb flow should not promote error_code tag into a captured event"
        );
        assert!(
            event.breadcrumbs.iter().any(|crumb| {
                crumb.category.as_deref() == Some("api.error")
                    && crumb
                        .message
                        .as_deref()
                        .unwrap_or("")
                        .contains("ENGINE_NOT_FOUND")
                    && crumb.data.get("trace_id").and_then(|value| value.as_str())
                        == Some("trace-404")
            }),
            "expected 4xx breadcrumb with trace_id on the captured event"
        );
    }
}
