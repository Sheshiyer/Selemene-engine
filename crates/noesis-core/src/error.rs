//! Unified error types for the Noesis platform

/// Main error type for all Noesis engines and services
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Engine not found: {0}")]
    EngineNotFound(String),

    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),

    #[error("Phase access denied: engine requires phase {required}, user is at phase {current}")]
    PhaseAccessDenied { required: u8, current: u8 },

    #[error("Bridge error: {0}")]
    BridgeError(String),

    #[error("Swiss Ephemeris error: {0}")]
    SwissEphemerisError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}
