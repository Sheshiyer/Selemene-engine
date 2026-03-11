use axum::response::{IntoResponse, Response};
use noesis_core::EngineError;
use crate::ErrorMapper;

// Wrapper for EngineError to implement IntoResponse
pub struct ApiError(pub EngineError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = ErrorMapper::map(self.0);
        (status, body).into_response()
    }
}

impl From<EngineError> for ApiError {
    fn from(err: EngineError) -> Self {
        ApiError(err)
    }
}
