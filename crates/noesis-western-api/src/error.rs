use thiserror::Error;

#[derive(Error, Debug)]
pub enum WesternApiError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
}

pub type Result<T> = std::result::Result<T, WesternApiError>;
