//! Bridge-specific error types
//!
//! These errors are converted into `EngineError::BridgeError` when crossing
//! the trait boundary.

use noesis_core::EngineError;
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

    /// Convert a bridge error produced while calculating an engine result.
    pub fn into_calculate_engine_error(self, engine_id: &str) -> EngineError {
        self.into_engine_error(engine_id, "calculate")
    }

    /// Convert a bridge error produced while validating an engine result.
    pub fn into_validate_engine_error(self, engine_id: &str) -> EngineError {
        self.into_engine_error(engine_id, "validate")
    }

    /// Convert a bridge-management error (health/readiness checks) into an engine error.
    pub fn into_manager_engine_error(self) -> EngineError {
        EngineError::BridgeError(self.to_string())
    }

    fn into_engine_error(self, engine_id: &str, operation: &str) -> EngineError {
        match self {
            BridgeError::EngineResponse { status, body } if (400..500).contains(&status) => {
                let message = if operation == "validate" {
                    format!("Engine {} validate rejected input ({}): {}", engine_id, status, body)
                } else {
                    format!("Engine {} rejected input ({}): {}", engine_id, status, body)
                };
                EngineError::ValidationError(message)
            }
            BridgeError::EngineResponse { status, body } => {
                let message = if operation == "validate" {
                    format!("Engine {} validate returned {}: {}", engine_id, status, body)
                } else {
                    format!("Engine {} returned {}: {}", engine_id, status, body)
                };
                EngineError::BridgeError(message)
            }
            BridgeError::Timeout { timeout_secs } => EngineError::BridgeError(format!(
                "Request to {} timed out after {}s",
                engine_id, timeout_secs
            )),
            BridgeError::ConnectionRefused { url } => EngineError::BridgeError(format!(
                "Connection refused while calling {} at {}",
                engine_id, url
            )),
            BridgeError::DeserializationError(message) => EngineError::BridgeError(format!(
                "Failed to deserialize {} response: {}",
                engine_id, message
            )),
            BridgeError::ServerUnavailable(message) => EngineError::BridgeError(format!(
                "Server unavailable while calling {}: {}",
                engine_id, message
            )),
            BridgeError::HttpError(message) => EngineError::BridgeError(format!(
                "HTTP request failed while calling {}: {}",
                engine_id, message
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BridgeError;
    use noesis_core::EngineError;

    #[test]
    fn timeout_preserves_timeout_seconds() {
        let err = BridgeError::Timeout { timeout_secs: 5 }.into_calculate_engine_error("tarot");
        match err {
            EngineError::BridgeError(message) => {
                assert!(message.contains("tarot"));
                assert!(message.contains("5s"));
            }
            other => panic!("expected BridgeError, got {:?}", other),
        }
    }

    #[test]
    fn connection_refused_preserves_url() {
        let err = BridgeError::ConnectionRefused {
            url: "http://localhost:3001".to_string(),
        }
        .into_calculate_engine_error("tarot");

        match err {
            EngineError::BridgeError(message) => {
                assert!(message.contains("tarot"));
                assert!(message.contains("http://localhost:3001"));
            }
            other => panic!("expected BridgeError, got {:?}", other),
        }
    }

    #[test]
    fn http_error_preserves_context() {
        let err = BridgeError::HttpError("socket hang up".to_string())
            .into_calculate_engine_error("tarot");

        match err {
            EngineError::BridgeError(message) => {
                assert!(message.contains("tarot"));
                assert!(message.contains("socket hang up"));
            }
            other => panic!("expected BridgeError, got {:?}", other),
        }
    }

    #[test]
    fn engine_response_client_errors_become_validation_errors() {
        let err = BridgeError::EngineResponse {
            status: 422,
            body: "{\"error\":\"bad input\"}".to_string(),
        }
        .into_calculate_engine_error("tarot");

        match err {
            EngineError::ValidationError(message) => {
                assert!(message.contains("tarot"));
                assert!(message.contains("422"));
                assert!(message.contains("bad input"));
            }
            other => panic!("expected ValidationError, got {:?}", other),
        }
    }

    #[test]
    fn engine_response_server_errors_stay_bridge_errors() {
        let err = BridgeError::EngineResponse {
            status: 503,
            body: "upstream unavailable".to_string(),
        }
        .into_calculate_engine_error("tarot");

        match err {
            EngineError::BridgeError(message) => {
                assert!(message.contains("tarot"));
                assert!(message.contains("503"));
                assert!(message.contains("upstream unavailable"));
            }
            other => panic!("expected BridgeError, got {:?}", other),
        }
    }

    #[test]
    fn deserialization_error_preserves_payload_context() {
        let err = BridgeError::DeserializationError("missing result field".to_string())
            .into_calculate_engine_error("tarot");

        match err {
            EngineError::BridgeError(message) => {
                assert!(message.contains("tarot"));
                assert!(message.contains("missing result field"));
            }
            other => panic!("expected BridgeError, got {:?}", other),
        }
    }

    #[test]
    fn server_unavailable_preserves_reason() {
        let err = BridgeError::ServerUnavailable("health endpoint down".to_string())
            .into_manager_engine_error();

        match err {
            EngineError::BridgeError(message) => {
                assert!(message.contains("health endpoint down"));
            }
            other => panic!("expected BridgeError, got {:?}", other),
        }
    }
}
