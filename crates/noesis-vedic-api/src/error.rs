//! Error types for FreeAstrologyAPI.com integration

use std::fmt;

/// Result type alias for Vedic API operations
pub type Result<T> = std::result::Result<T, VedicApiError>;

/// Alias for VedicApiResult for consistency
pub type VedicApiResult<T> = std::result::Result<T, VedicApiError>;

/// Errors that can occur in the Vedic API integration
#[derive(Debug, Clone, PartialEq)]
pub enum VedicApiError {
    /// Configuration error (missing API key, etc.)
    Configuration {
        field: String,
        message: String,
    },
    
    /// Network error (connection failed, timeout, etc.)
    Network {
        message: String,
    },
    
    /// API error (invalid response, rate limit, etc.)
    Api {
        status_code: u16,
        message: String,
    },
    
    /// Rate limit exceeded
    RateLimit {
        retry_after: Option<u64>,
    },
    
    /// Invalid input parameters
    InvalidInput {
        field: String,
        message: String,
    },
    
    /// Parse error (invalid JSON, missing fields, etc.)
    Parse {
        message: String,
    },
    
    /// Circuit breaker is open
    CircuitBreakerOpen,
    
    /// Cache error
    Cache {
        message: String,
    },
    
    /// Fallback to native calculation failed
    FallbackFailed {
        api_error: Box<VedicApiError>,
        native_error: String,
    },
    
    /// Simple parse error variant (for compatibility)
    ParseError(String),
    
    /// Simple network error variant (for compatibility)
    NetworkError(String),
    
    /// Timeout error
    Timeout(String),
    
    /// Rate limited (simple variant)
    RateLimited { retry_after_seconds: Option<u64> },
    
    /// Service unavailable
    ServiceUnavailable(String),
}

impl fmt::Display for VedicApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VedicApiError::Configuration { field, message } => {
                write!(f, "Configuration error for '{}': {}", field, message)
            }
            VedicApiError::Network { message } => {
                write!(f, "Network error: {}", message)
            }
            VedicApiError::Api { status_code, message } => {
                write!(f, "API error (HTTP {}): {}", status_code, message)
            }
            VedicApiError::RateLimit { retry_after } => {
                match retry_after {
                    Some(seconds) => write!(f, "Rate limit exceeded. Retry after {} seconds", seconds),
                    None => write!(f, "Rate limit exceeded"),
                }
            }
            VedicApiError::InvalidInput { field, message } => {
                write!(f, "Invalid input for '{}': {}", field, message)
            }
            VedicApiError::Parse { message } => {
                write!(f, "Parse error: {}", message)
            }
            VedicApiError::CircuitBreakerOpen => {
                write!(f, "Circuit breaker is open. API temporarily unavailable.")
            }
            VedicApiError::Cache { message } => {
                write!(f, "Cache error: {}", message)
            }
            VedicApiError::FallbackFailed { api_error, native_error } => {
                write!(f, "Fallback failed. API error: {}, Native error: {}", api_error, native_error)
            }
            VedicApiError::ParseError(msg) => {
                write!(f, "Parse error: {}", msg)
            }
            VedicApiError::NetworkError(msg) => {
                write!(f, "Network error: {}", msg)
            }
            VedicApiError::Timeout(msg) => {
                write!(f, "Timeout: {}", msg)
            }
            VedicApiError::RateLimited { retry_after_seconds } => {
                match retry_after_seconds {
                    Some(secs) => write!(f, "Rate limited. Retry after {} seconds", secs),
                    None => write!(f, "Rate limited"),
                }
            }
            VedicApiError::ServiceUnavailable(msg) => {
                write!(f, "Service unavailable: {}", msg)
            }
        }
    }
}

impl std::error::Error for VedicApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<reqwest::Error> for VedicApiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            VedicApiError::Network {
                message: format!("Request timeout: {}", err),
            }
        } else if err.is_connect() {
            VedicApiError::Network {
                message: format!("Connection error: {}", err),
            }
        } else {
            VedicApiError::Network {
                message: format!("HTTP client error: {}", err),
            }
        }
    }
}

impl From<serde_json::Error> for VedicApiError {
    fn from(err: serde_json::Error) -> Self {
        VedicApiError::Parse {
            message: format!("JSON parse error: {}", err),
        }
    }
}

impl VedicApiError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            VedicApiError::Network { .. } => true,
            VedicApiError::RateLimit { .. } => true,
            VedicApiError::Api { status_code, .. } => *status_code >= 500,
            _ => false,
        }
    }
    
    /// Check if we should fallback to native calculation
    pub fn should_fallback(&self) -> bool {
        matches!(self,
            VedicApiError::Network { .. } |
            VedicApiError::CircuitBreakerOpen |
            VedicApiError::RateLimit { .. } |
            VedicApiError::Api { status_code: 500..=599, .. }
        )
    }
    
    /// Get the HTTP status code if applicable
    pub fn status_code(&self) -> Option<u16> {
        match self {
            VedicApiError::Api { status_code, .. } => Some(*status_code),
            VedicApiError::RateLimit { .. } => Some(429),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = VedicApiError::Configuration {
            field: "API_KEY".to_string(),
            message: "Not found".to_string(),
        };
        assert!(err.to_string().contains("API_KEY"));
    }

    #[test]
    fn test_is_retryable() {
        let network_err = VedicApiError::Network { message: "timeout".to_string() };
        assert!(network_err.is_retryable());
        
        let config_err = VedicApiError::Configuration {
            field: "test".to_string(),
            message: "test".to_string(),
        };
        assert!(!config_err.is_retryable());
    }

    #[test]
    fn test_should_fallback() {
        let circuit_err = VedicApiError::CircuitBreakerOpen;
        assert!(circuit_err.should_fallback());
        
        let parse_err = VedicApiError::Parse { message: "test".to_string() };
        assert!(!parse_err.should_fallback());
    }
}
