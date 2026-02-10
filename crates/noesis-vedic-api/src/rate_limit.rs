//! HTTP 429 rate limit handling with exponential backoff
//!
//! FAPI-105: Handle 429 rate limit responses with exponential backoff
//!
//! This module provides a `RateLimitHandler` that tracks 429 responses from
//! the FreeAstrologyAPI and calculates appropriate backoff delays. It respects
//! the `Retry-After` header when present, and falls back to exponential
//! backoff otherwise.
//!
//! This is distinct from `rate_limiter.rs` which tracks daily quota (50/day).
//! This module handles *server-side* rate limiting (429 Too Many Requests).

use chrono::{DateTime, Utc};
use std::time::Duration;
use tracing::{debug, warn};

/// Configuration for 429 rate limit backoff behavior.
///
/// Loaded from environment variables or constructed with defaults.
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of retry attempts on 429 (default: 3)
    pub max_retries: u32,
    /// Base delay in milliseconds for exponential backoff (default: 1000)
    pub base_delay_ms: u64,
    /// Maximum delay in milliseconds (cap) (default: 60000)
    pub max_delay_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 60_000,
        }
    }
}

impl RateLimitConfig {
    /// Load from environment variables with fallback to defaults.
    ///
    /// Environment variables:
    /// - `FREE_ASTROLOGY_API_RATE_LIMIT_MAX_RETRIES` (default: 3)
    /// - `FREE_ASTROLOGY_API_RATE_LIMIT_BASE_DELAY` (default: 1000 ms)
    /// - `FREE_ASTROLOGY_API_RATE_LIMIT_MAX_DELAY` (default: 60000 ms)
    pub fn from_env() -> Self {
        let max_retries = std::env::var("FREE_ASTROLOGY_API_RATE_LIMIT_MAX_RETRIES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);

        let base_delay_ms = std::env::var("FREE_ASTROLOGY_API_RATE_LIMIT_BASE_DELAY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1000);

        let max_delay_ms = std::env::var("FREE_ASTROLOGY_API_RATE_LIMIT_MAX_DELAY")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60_000);

        Self {
            max_retries,
            base_delay_ms,
            max_delay_ms,
        }
    }
}

/// Tracks 429 rate limit state and computes exponential backoff delays.
///
/// Usage:
/// 1. On 429 response, call `record_429()` with optional Retry-After value
/// 2. Check `should_retry()` to see if retries remain
/// 3. Call `next_backoff_delay()` to get the sleep duration
/// 4. On success, call `reset()` to clear retry state
#[derive(Debug, Clone)]
pub struct RateLimitHandler {
    config: RateLimitConfig,
    retry_count: u32,
    last_429_at: Option<DateTime<Utc>>,
    last_retry_after: Option<u64>,
}

impl RateLimitHandler {
    /// Create a new handler with the given configuration.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            retry_count: 0,
            last_429_at: None,
            last_retry_after: None,
        }
    }

    /// Create a handler with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(RateLimitConfig::default())
    }

    /// Create a handler with a custom max_retries (convenience).
    pub fn with_max_retries(max_retries: u32) -> Self {
        Self::new(RateLimitConfig {
            max_retries,
            ..RateLimitConfig::default()
        })
    }

    /// Whether we have retries remaining.
    pub fn should_retry(&self) -> bool {
        self.retry_count < self.config.max_retries
    }

    /// Current retry count.
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }

    /// Compute the next backoff delay.
    ///
    /// If a `Retry-After` header was recorded from the last 429 response,
    /// that value takes precedence (capped at max_delay).
    /// Otherwise, exponential backoff is used: base_delay * 2^retry_count,
    /// capped at max_delay.
    pub fn next_backoff_delay(&self) -> Duration {
        // Retry-After header takes precedence
        if let Some(retry_after_secs) = self.last_retry_after {
            let capped = retry_after_secs.min(self.config.max_delay_ms / 1000);
            debug!(
                "Using Retry-After header: {}s (capped to {}s)",
                retry_after_secs, capped
            );
            return Duration::from_secs(capped);
        }

        // Exponential backoff: base_delay * 2^retry_count
        let delay_ms = self.config.base_delay_ms as f64 * 2.0_f64.powi(self.retry_count as i32);
        let capped_ms = (delay_ms as u64).min(self.config.max_delay_ms);

        debug!(
            "Exponential backoff: attempt {}, delay {}ms (cap {}ms)",
            self.retry_count, capped_ms, self.config.max_delay_ms
        );

        Duration::from_millis(capped_ms)
    }

    /// Record that a 429 response was received.
    ///
    /// `retry_after` is the parsed value of the `Retry-After` header, if present.
    pub fn record_429(&mut self, retry_after: Option<u64>) {
        self.retry_count += 1;
        self.last_429_at = Some(Utc::now());
        self.last_retry_after = retry_after;

        warn!(
            "429 rate limited (attempt {}/{}). Retry-After: {:?}",
            self.retry_count, self.config.max_retries, retry_after
        );
    }

    /// Reset the handler after a successful request.
    pub fn reset(&mut self) {
        if self.retry_count > 0 {
            debug!(
                "Rate limit handler reset after {} retries",
                self.retry_count
            );
        }
        self.retry_count = 0;
        self.last_429_at = None;
        self.last_retry_after = None;
    }

    /// Get the timestamp of the last 429 response, if any.
    pub fn last_429_at(&self) -> Option<DateTime<Utc>> {
        self.last_429_at
    }

    /// Get the configuration.
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RateLimitConfig Tests ====================

    #[test]
    fn test_default_config() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 60_000);
    }

    #[test]
    fn test_config_from_env() {
        // Set environment variables
        std::env::set_var("FREE_ASTROLOGY_API_RATE_LIMIT_MAX_RETRIES", "5");
        std::env::set_var("FREE_ASTROLOGY_API_RATE_LIMIT_BASE_DELAY", "2000");
        std::env::set_var("FREE_ASTROLOGY_API_RATE_LIMIT_MAX_DELAY", "30000");

        let config = RateLimitConfig::from_env();
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay_ms, 2000);
        assert_eq!(config.max_delay_ms, 30_000);

        // Clean up
        std::env::remove_var("FREE_ASTROLOGY_API_RATE_LIMIT_MAX_RETRIES");
        std::env::remove_var("FREE_ASTROLOGY_API_RATE_LIMIT_BASE_DELAY");
        std::env::remove_var("FREE_ASTROLOGY_API_RATE_LIMIT_MAX_DELAY");
    }

    // ==================== RateLimitHandler Tests ====================

    #[test]
    fn test_handler_creation() {
        let handler = RateLimitHandler::with_defaults();
        assert_eq!(handler.retry_count(), 0);
        assert!(handler.should_retry());
        assert!(handler.last_429_at().is_none());
    }

    #[test]
    fn test_exponential_backoff_sequence() {
        // base_delay=1000ms, max_delay=60000ms
        // Expected: 1s, 2s, 4s, 8s, 16s, 32s, 60s (capped)
        let config = RateLimitConfig {
            max_retries: 7,
            base_delay_ms: 1000,
            max_delay_ms: 60_000,
        };
        let mut handler = RateLimitHandler::new(config);

        // Attempt 0 (before any 429): delay = 1000 * 2^0 = 1000ms
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(1000));

        // Record first 429, retry_count becomes 1: delay = 1000 * 2^1 = 2000ms
        handler.record_429(None);
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(2000));

        // Record second 429, retry_count becomes 2: delay = 1000 * 2^2 = 4000ms
        handler.record_429(None);
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(4000));

        // Record third 429, retry_count becomes 3: delay = 1000 * 2^3 = 8000ms
        handler.record_429(None);
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(8000));

        // Record fourth 429, retry_count becomes 4: delay = 1000 * 2^4 = 16000ms
        handler.record_429(None);
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(16000));

        // Record fifth 429, retry_count becomes 5: delay = 1000 * 2^5 = 32000ms
        handler.record_429(None);
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(32000));

        // Record sixth 429, retry_count becomes 6: delay = 1000 * 2^6 = 64000 -> capped to 60000
        handler.record_429(None);
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(60000));
    }

    #[test]
    fn test_retry_exhaustion() {
        let mut handler = RateLimitHandler::with_max_retries(3);

        assert!(handler.should_retry()); // 0 < 3
        handler.record_429(None);

        assert!(handler.should_retry()); // 1 < 3
        handler.record_429(None);

        assert!(handler.should_retry()); // 2 < 3
        handler.record_429(None);

        assert!(!handler.should_retry()); // 3 == 3, exhausted
    }

    #[test]
    fn test_retry_after_header_override() {
        let mut handler = RateLimitHandler::with_defaults();

        // Without Retry-After, get exponential backoff
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(1000));

        // Record 429 with Retry-After: 5 seconds
        handler.record_429(Some(5));
        assert_eq!(handler.next_backoff_delay(), Duration::from_secs(5));

        // Record another 429 without Retry-After, back to exponential
        handler.record_429(None);
        // retry_count is now 2, so delay = 1000 * 2^2 = 4000ms
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(4000));
    }

    #[test]
    fn test_retry_after_capped_at_max_delay() {
        let config = RateLimitConfig {
            max_retries: 3,
            base_delay_ms: 1000,
            max_delay_ms: 10_000, // 10s max
        };
        let mut handler = RateLimitHandler::new(config);

        // Retry-After says 120s, but max_delay is 10s -> capped at 10s
        handler.record_429(Some(120));
        assert_eq!(handler.next_backoff_delay(), Duration::from_secs(10));
    }

    #[test]
    fn test_reset_after_success() {
        let mut handler = RateLimitHandler::with_defaults();

        handler.record_429(None);
        handler.record_429(None);
        assert_eq!(handler.retry_count(), 2);
        assert!(handler.last_429_at().is_some());

        handler.reset();
        assert_eq!(handler.retry_count(), 0);
        assert!(handler.last_429_at().is_none());
        assert!(handler.should_retry());
        // After reset, delay should be back to base
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(1000));
    }

    #[test]
    fn test_last_429_timestamp_recorded() {
        let mut handler = RateLimitHandler::with_defaults();
        assert!(handler.last_429_at().is_none());

        let before = Utc::now();
        handler.record_429(None);
        let after = Utc::now();

        let ts = handler.last_429_at().unwrap();
        assert!(ts >= before && ts <= after);
    }

    #[test]
    fn test_zero_max_retries() {
        let handler = RateLimitHandler::with_max_retries(0);
        // With 0 max retries, should never retry
        assert!(!handler.should_retry());
    }

    #[test]
    fn test_custom_base_delay() {
        let config = RateLimitConfig {
            max_retries: 3,
            base_delay_ms: 500,
            max_delay_ms: 30_000,
        };
        let handler = RateLimitHandler::new(config);

        // 500 * 2^0 = 500ms
        assert_eq!(handler.next_backoff_delay(), Duration::from_millis(500));
    }
}
