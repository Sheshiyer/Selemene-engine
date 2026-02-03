//! Retry logic with exponential backoff for FreeAstrologyAPI
//!
//! FAPI-006: Implement retry logic with exponential backoff
//!
//! This module re-exports the retry functionality from the resilience module
//! and provides additional convenience methods for common retry patterns.

pub use crate::resilience::{BackoffConfig, ExponentialBackoff};

use std::future::Future;
use std::time::Duration;
use crate::error::VedicApiError;

/// Retry configuration with sensible defaults for astrology API
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential growth
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_attempts: u32, initial_delay: Duration) -> Self {
        Self {
            max_attempts,
            initial_delay,
            ..Default::default()
        }
    }

    /// Create a configuration for quick retries (short delays)
    pub fn quick() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            multiplier: 2.0,
        }
    }

    /// Create a configuration for patient retries (longer delays)
    pub fn patient() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }

    /// Convert to BackoffConfig for use with ExponentialBackoff
    pub fn to_backoff_config(&self) -> BackoffConfig {
        BackoffConfig {
            initial_delay_ms: self.initial_delay.as_millis() as u64,
            max_delay_ms: self.max_delay.as_millis() as u64,
            max_retries: self.max_attempts,
            multiplier: self.multiplier,
            jitter: true,
        }
    }
}

/// Determines if an error is retryable
pub fn is_retryable(error: &VedicApiError) -> bool {
    matches!(
        error,
        VedicApiError::NetworkError(_)
            | VedicApiError::Timeout(_)
            | VedicApiError::RateLimited { .. }
            | VedicApiError::ServiceUnavailable(_)
    )
}

/// Execute an async operation with retry logic
pub async fn with_retry<F, Fut, T>(
    config: &RetryConfig,
    operation: F,
) -> Result<T, VedicApiError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, VedicApiError>>,
{
    let backoff = ExponentialBackoff::new(config.to_backoff_config());
    backoff.execute(operation).await
}

/// Execute with default retry configuration
pub async fn with_default_retry<F, Fut, T>(operation: F) -> Result<T, VedicApiError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, VedicApiError>>,
{
    with_retry(&RetryConfig::default(), operation).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(500));
    }

    #[test]
    fn test_quick_config() {
        let config = RetryConfig::quick();
        assert_eq!(config.initial_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_patient_config() {
        let config = RetryConfig::patient();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_secs(1));
    }

    #[test]
    fn test_is_retryable() {
        assert!(is_retryable(&VedicApiError::Timeout("test".to_string())));
        assert!(is_retryable(&VedicApiError::NetworkError("connection reset".to_string())));
        assert!(!is_retryable(&VedicApiError::InvalidInput { 
            field: "date".to_string(), 
            message: "bad date".to_string() 
        }));
    }
}
