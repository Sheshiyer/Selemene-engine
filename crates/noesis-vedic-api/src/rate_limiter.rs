//! Rate limiter for FreeAstrologyAPI.com free plan
//! 
//! Free plan limits:
//! - 50 requests per day
//! - 1 request per second

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Rate limiter for API requests
#[derive(Debug)]
pub struct RateLimiter {
    daily_limit: u32,
    remaining_today: AtomicU32,
    last_request: Mutex<Option<Instant>>,
    buffer: u32,
}

impl RateLimiter {
    /// Create new rate limiter with free plan defaults
    pub fn new() -> Self {
        Self::with_limits(50, 5) // 50/day, 5 buffer
    }
    
    /// Create with custom limits
    pub fn with_limits(daily_limit: u32, buffer: u32) -> Self {
        info!(
            "RateLimiter initialized: {} requests/day, {} buffer",
            daily_limit, buffer
        );
        
        Self {
            daily_limit,
            remaining_today: AtomicU32::new(daily_limit),
            last_request: Mutex::new(None),
            buffer,
        }
    }
    
    /// Check if a request can be made
    pub fn can_request(&self) -> bool {
        let remaining = self.remaining_today.load(Ordering::SeqCst);
        remaining > self.buffer
    }
    
    /// Get remaining requests (excluding buffer)
    pub fn remaining(&self) -> u32 {
        let remaining = self.remaining_today.load(Ordering::SeqCst);
        remaining.saturating_sub(self.buffer)
    }
    
    /// Get total used requests today
    pub fn used_today(&self) -> u32 {
        self.daily_limit - self.remaining_today.load(Ordering::SeqCst)
    }
    
    /// Wait for permission to make a request
    /// Returns true if allowed, false if daily limit exceeded
    pub async fn acquire(&self) -> bool {
        // Check daily limit (with buffer)
        let remaining = self.remaining_today.load(Ordering::SeqCst);
        if remaining <= self.buffer {
            warn!(
                "Daily rate limit reached: {}/{} used, {} buffer",
                self.used_today(),
                self.daily_limit,
                self.buffer
            );
            return false;
        }
        
        // Check 1 request per second limit
        let should_wait = {
            let last_request = self.last_request.lock().await;
            if let Some(last) = *last_request {
                let elapsed = last.elapsed();
                elapsed < Duration::from_secs(1)
            } else {
                false
            }
        };
        
        if should_wait {
            // Re-acquire lock and wait
            let last_request = self.last_request.lock().await;
            if let Some(last) = *last_request {
                let elapsed = last.elapsed();
                if elapsed < Duration::from_secs(1) {
                    let wait = Duration::from_secs(1) - elapsed;
                    drop(last_request); // Release lock before sleep
                    debug!("Rate limiting: waiting {:?} for 1/sec limit", wait);
                    tokio::time::sleep(wait).await;
                }
            }
        }
        
        // Update last request time and decrement
        let mut last_request = self.last_request.lock().await;
        *last_request = Some(Instant::now());
        drop(last_request);
        
        let new_remaining = self.remaining_today.fetch_sub(1, Ordering::SeqCst) - 1;
        
        debug!(
            "Request allowed: {} remaining ({} used today)",
            new_remaining,
            self.used_today()
        );
        
        true
    }
    
    /// Release a request (if it failed, give back the quota)
    pub fn release(&self) {
        let remaining = self.remaining_today.load(Ordering::SeqCst);
        if remaining < self.daily_limit {
            self.remaining_today.fetch_add(1, Ordering::SeqCst);
            debug!("Rate limit released");
        }
    }
    
    /// Get status summary
    pub fn status(&self) -> RateLimitStatus {
        RateLimitStatus {
            daily_limit: self.daily_limit,
            remaining_today: self.remaining_today.load(Ordering::SeqCst),
            buffer: self.buffer,
            effective_remaining: self.remaining(),
            used_today: self.used_today(),
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        // Create new with same limits, reset state
        Self::with_limits(self.daily_limit, self.buffer)
    }
}

/// Rate limit status for monitoring
#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub daily_limit: u32,
    pub remaining_today: u32,
    pub buffer: u32,
    pub effective_remaining: u32,
    pub used_today: u32,
}

impl std::fmt::Display for RateLimitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rate Limit: {}/{} used today, {} effective remaining",
            self.used_today,
            self.daily_limit,
            self.effective_remaining
        )
    }
}

/// Request queue with rate limiting
#[derive(Debug, Clone)]
pub struct RequestQueue {
    limiter: RateLimiter,
}

impl RequestQueue {
    pub fn new() -> Self {
        Self {
            limiter: RateLimiter::new(),
        }
    }
    
    /// Execute a request with rate limiting
    pub async fn execute<F, Fut, T>(&self, f: F) -> crate::Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = crate::Result<T>>,
    {
        // Acquire rate limit
        if !self.limiter.acquire().await {
            return Err(crate::VedicApiError::RateLimit {
                retry_after: Some(3600), // Try again in 1 hour
            });
        }
        
        // Execute request
        match f().await {
            Ok(result) => Ok(result),
            Err(e) => {
                // If request failed, release the quota
                self.limiter.release();
                Err(e)
            }
        }
    }
    
    /// Get rate limiter status
    pub fn status(&self) -> RateLimitStatus {
        self.limiter.status()
    }
}

impl Default for RequestQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::with_limits(10, 2); // 10/day, 2 buffer
        
        // Should allow 8 requests (10 - 2 buffer)
        for i in 0..8 {
            assert!(limiter.acquire().await, "Request {} should be allowed", i);
        }
        
        // Should block (only 2 left, which is buffer)
        assert!(!limiter.acquire().await, "Should be rate limited");
        
        // Check status
        let status = limiter.status();
        assert_eq!(status.used_today, 8);
        assert_eq!(status.effective_remaining, 0);
    }

    #[tokio::test]
    async fn test_rate_limit_release() {
        let limiter = RateLimiter::with_limits(10, 0);
        
        assert!(limiter.acquire().await);
        assert_eq!(limiter.remaining(), 9);
        
        limiter.release();
        assert_eq!(limiter.remaining(), 10);
    }
}
