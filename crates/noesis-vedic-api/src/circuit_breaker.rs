//! Circuit breaker pattern for FreeAstrologyAPI
//!
//! FAPI-008: Implement circuit breaker pattern
//!
//! The circuit breaker prevents cascading failures by:
//! - Opening after consecutive failures exceed threshold
//! - Staying open for a recovery period
//! - Half-opening to test recovery
//! - Closing when successful calls resume

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected immediately
    Open,
    /// Circuit is testing if service has recovered
    HalfOpen,
}

/// Configuration for the circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening (default: 5)
    pub failure_threshold: u32,
    /// Duration to stay open before testing recovery (default: 30s)
    pub recovery_timeout: Duration,
    /// Number of successful calls in half-open to close (default: 2)
    pub success_threshold: u32,
    /// Window for counting failures (default: 60s)
    pub failure_window: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 2,
            failure_window: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker for protecting against cascading failures
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: RwLock<CircuitState>,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: RwLock<Option<Instant>>,
    opened_at: RwLock<Option<Instant>>,
    
    // Metrics
    total_calls: AtomicU64,
    total_failures: AtomicU64,
    total_rejections: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            last_failure_time: RwLock::new(None),
            opened_at: RwLock::new(None),
            total_calls: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            total_rejections: AtomicU64::new(0),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }

    /// Get the current circuit state
    pub fn state(&self) -> CircuitState {
        *self.state.read().unwrap()
    }

    /// Check if the circuit allows a request to proceed
    pub fn allow_request(&self) -> bool {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
        
        let current_state = *self.state.read().unwrap();
        
        match current_state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if recovery timeout has passed
                if let Some(opened_at) = *self.opened_at.read().unwrap() {
                    if opened_at.elapsed() >= self.config.recovery_timeout {
                        // Transition to half-open
                        self.transition_to(CircuitState::HalfOpen);
                        info!("Circuit breaker transitioning to half-open for testing");
                        true
                    } else {
                        self.total_rejections.fetch_add(1, Ordering::Relaxed);
                        debug!("Circuit breaker rejecting request (open)");
                        false
                    }
                } else {
                    self.total_rejections.fetch_add(1, Ordering::Relaxed);
                    false
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                true
            }
        }
    }

    /// Record a successful call
    pub fn record_success(&self) {
        let current_state = *self.state.read().unwrap();
        
        match current_state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::HalfOpen => {
                let successes = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if successes >= self.config.success_threshold {
                    // Close the circuit
                    self.transition_to(CircuitState::Closed);
                    info!("Circuit breaker closed after successful recovery");
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                warn!("Success recorded while circuit is open");
            }
        }
    }

    /// Record a failed call
    pub fn record_failure(&self) {
        self.total_failures.fetch_add(1, Ordering::Relaxed);
        
        let current_state = *self.state.read().unwrap();
        let now = Instant::now();
        
        // Update last failure time
        *self.last_failure_time.write().unwrap() = Some(now);
        
        match current_state {
            CircuitState::Closed => {
                // Check if we should reset the failure window
                if let Some(last_failure) = *self.last_failure_time.read().unwrap() {
                    if last_failure.elapsed() > self.config.failure_window {
                        self.failure_count.store(1, Ordering::Relaxed);
                        return;
                    }
                }
                
                let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if failures >= self.config.failure_threshold {
                    // Open the circuit
                    self.transition_to(CircuitState::Open);
                    warn!(
                        "Circuit breaker opened after {} consecutive failures",
                        failures
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open reopens the circuit
                self.transition_to(CircuitState::Open);
                warn!("Circuit breaker reopened after failure during recovery test");
            }
            CircuitState::Open => {
                // Already open, just log
                debug!("Failure recorded while circuit is open");
            }
        }
    }

    /// Manually reset the circuit breaker to closed state
    pub fn reset(&self) {
        self.transition_to(CircuitState::Closed);
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        *self.opened_at.write().unwrap() = None;
        info!("Circuit breaker manually reset");
    }

    /// Get circuit breaker metrics
    pub fn metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            state: self.state(),
            total_calls: self.total_calls.load(Ordering::Relaxed),
            total_failures: self.total_failures.load(Ordering::Relaxed),
            total_rejections: self.total_rejections.load(Ordering::Relaxed),
            current_failure_count: self.failure_count.load(Ordering::Relaxed),
        }
    }

    fn transition_to(&self, new_state: CircuitState) {
        let mut state = self.state.write().unwrap();
        
        match new_state {
            CircuitState::Open => {
                *self.opened_at.write().unwrap() = Some(Instant::now());
                self.success_count.store(0, Ordering::Relaxed);
            }
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::Relaxed);
                self.success_count.store(0, Ordering::Relaxed);
                *self.opened_at.write().unwrap() = None;
            }
            CircuitState::HalfOpen => {
                self.success_count.store(0, Ordering::Relaxed);
            }
        }
        
        *state = new_state;
    }
}

/// Metrics from the circuit breaker
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub state: CircuitState,
    pub total_calls: u64,
    pub total_failures: u64,
    pub total_rejections: u64,
    pub current_failure_count: u32,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "CLOSED"),
            CircuitState::Open => write!(f, "OPEN"),
            CircuitState::HalfOpen => write!(f, "HALF_OPEN"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_closed() {
        let cb = CircuitBreaker::with_defaults();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_allows_requests_when_closed() {
        let cb = CircuitBreaker::with_defaults();
        assert!(cb.allow_request());
    }

    #[test]
    fn test_opens_after_threshold_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        for _ in 0..3 {
            cb.record_failure();
        }

        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_rejects_when_open() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_secs(60),
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_success_resets_failure_count() {
        let cb = CircuitBreaker::with_defaults();
        
        cb.record_failure();
        cb.record_failure();
        cb.record_success();
        
        assert_eq!(cb.failure_count.load(Ordering::Relaxed), 0);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_manual_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 1,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_metrics() {
        let cb = CircuitBreaker::with_defaults();
        cb.allow_request();
        cb.record_failure();

        let metrics = cb.metrics();
        assert_eq!(metrics.total_calls, 1);
        assert_eq!(metrics.total_failures, 1);
    }
}
