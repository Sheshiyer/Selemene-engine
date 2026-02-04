use axum::response::{IntoResponse, Response};
use noesis_core::EngineError;

// Wrapper for EngineError to implement IntoResponse
pub struct ApiError(pub EngineError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = crate::engine_error_to_response(self.0);
        (status, body).into_response()
    }
}

impl From<EngineError> for ApiError {
    fn from(err: EngineError) -> Self {
        ApiError(err)
    }
}
