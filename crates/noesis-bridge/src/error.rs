//! Bridge-specific error types
//!
//! These errors are converted into `EngineError::BridgeError` when crossing
//! the trait boundary.

use thiserror::Error;

/// Errors that can occur during bridge operations.
#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("HTTP request failed: {0}")]
    HttpError(String),

    #[error("Request timeout after {timeout_secs}s")]
    Timeout { timeout_secs: u64 },

    #[error("Connection refused to {url}")]
    ConnectionRefused { url: String },

    #[error("TS engine returned {status}: {body}")]
    EngineResponse { status: u16, body: String },

    #[error("Failed to deserialize response: {0}")]
    DeserializationError(String),

    #[error("Server unavailable: {0}")]
    ServerUnavailable(String),
}

impl BridgeError {
    /// Convert into a string suitable for `EngineError::BridgeError`.
    pub fn to_engine_error_message(&self) -> String {
        self.to_string()
    }
}
