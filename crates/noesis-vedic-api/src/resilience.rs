//! Resilience layer: exponential backoff, fallback chain, and metrics
//!
//! FAPI-098: API Fallback to Native Calculation
//! FAPI-105: Rate Limit Handling with Exponential Backoff

use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tracing::{debug, warn};

use crate::{
    cache::{panchang_key, ApiCache},
    panchang::Panchang,
    Config, VedicApiError,
};

// ====================== EXPONENTIAL BACKOFF (FAPI-105) ======================

/// Configuration for exponential backoff behavior
#[derive(Debug, Clone)]
pub struct BackoffConfig {
    /// Initial delay in milliseconds (default: 1000)
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds (default: 16000)
    pub max_delay_ms: u64,
    /// Maximum number of retries (default: 5)
    pub max_retries: u32,
    /// Multiplier for each successive delay (default: 2.0)
    pub multiplier: f64,
    /// Whether to add jitter to delays (default: true)
    pub jitter: bool,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 1000,
            max_delay_ms: 16000,
            max_retries: 5,
            multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Exponential backoff executor for retrying failed API calls
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    config: BackoffConfig,
}

impl ExponentialBackoff {
    /// Create a new backoff executor with the given configuration
    pub fn new(config: BackoffConfig) -> Self {
        Self { config }
    }

    /// Calculate the delay for a given attempt number (0-indexed)
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms =
            self.config.initial_delay_ms as f64 * self.config.multiplier.powi(attempt as i32);
        let capped_ms = delay_ms.min(self.config.max_delay_ms as f64) as u64;

        if self.config.jitter {
            // Add up to 25% jitter
            let jitter = (capped_ms as f64 * 0.25 * rand_simple()) as u64;
            Duration::from_millis(capped_ms + jitter)
        } else {
            Duration::from_millis(capped_ms)
        }
    }

    /// Calculate delay honoring a Retry-After value from rate limit response
    pub fn delay_for_rate_limit(&self, retry_after: Option<u64>) -> Duration {
        match retry_after {
            Some(seconds) => Duration::from_secs(seconds),
            None => self.delay_for_attempt(0),
        }
    }

    /// Whether we should retry at the given attempt number
    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.config.max_retries
    }

    /// Execute an async operation with exponential backoff on retryable errors.
    ///
    /// The closure is called repeatedly until it succeeds, returns a
    /// non-retryable error, or the maximum number of retries is exhausted.
    pub async fn execute<F, Fut, T>(&self, mut f: F) -> Result<T, VedicApiError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, VedicApiError>>,
    {
        let mut attempt: u32 = 0;

        loop {
            match f().await {
                Ok(value) => {
                    if attempt > 0 {
                        debug!("Succeeded after {} retries", attempt);
                    }
                    return Ok(value);
                }
                Err(err) => {
                    if !err.is_retryable() {
                        debug!("Non-retryable error, not retrying: {}", err);
                        return Err(err);
                    }

                    if !self.should_retry(attempt) {
                        warn!(
                            "Max retries ({}) exhausted, giving up. Last error: {}",
                            self.config.max_retries, err
                        );
                        return Err(err);
                    }

                    let delay = match &err {
                        VedicApiError::RateLimit { retry_after } => {
                            self.delay_for_rate_limit(*retry_after)
                        }
                        _ => self.delay_for_attempt(attempt),
                    };

                    debug!(
                        "Attempt {} failed ({}), retrying in {:?}",
                        attempt, err, delay
                    );

                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }
}

/// Simple pseudo-random for jitter (avoids pulling in rand crate)
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

// ====================== FALLBACK CHAIN (FAPI-098) ======================

/// The source that ultimately provided the data
#[derive(Debug, Clone, PartialEq)]
pub enum FallbackSource {
    /// Data came from the remote API
    Api,
    /// Data came from the local cache
    Cache,
    /// Data came from native (local) calculation
    NativeCalculation,
}

/// Result from the fallback chain, including provenance metadata
#[derive(Debug, Clone)]
pub struct FallbackResult<T> {
    /// The actual value
    pub value: T,
    /// Where the value came from
    pub source: FallbackSource,
    /// How many sources were attempted before success
    pub attempts: u32,
    /// Total wall-clock time across all attempts
    pub total_duration: Duration,
}

/// Cache-first Panchang lookup with provenance metadata.
///
/// Panchang is computed natively (`panchang::api::compute_panchang_native`)
/// and never calls the vendor API, so there is no API step to fall back
/// from: a request is served from the cache, or computed locally and
/// labelled [`FallbackSource::NativeCalculation`].
#[derive(Debug)]
pub struct FallbackChain {
    cache: ApiCache,
    metrics: ResilienceMetrics,
}

impl FallbackChain {
    /// Create a new chain. `config` is accepted for compatibility; Panchang
    /// needs no API configuration.
    pub fn new(_config: Config) -> Self {
        Self {
            cache: ApiCache::new(),
            metrics: ResilienceMetrics::new(),
        }
    }

    /// Get a reference to the resilience metrics
    pub fn metrics(&self) -> &ResilienceMetrics {
        &self.metrics
    }

    /// Get a Panchang:
    /// 1. from the cache, labelled [`FallbackSource::Cache`];
    /// 2. otherwise computed natively and cached, labelled
    ///    [`FallbackSource::NativeCalculation`].
    ///
    /// Invalid input (e.g. month 13) is returned as the native calculator's
    /// error.
    pub async fn get_panchang(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> Result<FallbackResult<Panchang>, VedicApiError> {
        let start = Instant::now();
        let cache_key = panchang_key(year, month, day, lat, lng);

        // ---- Step 1: Check cache ----
        if let Some(cached) = self.cache.get_panchang(&cache_key).await {
            debug!("Fallback chain: cache hit for {}", cache_key);
            self.metrics.record_cache_hit();
            return Ok(FallbackResult {
                value: cached,
                source: FallbackSource::Cache,
                attempts: 1,
                total_duration: start.elapsed(),
            });
        }

        // ---- Step 2: Compute natively ----
        let panchang = crate::panchang::api::compute_panchang_native(
            year, month, day, hour, minute, second, lat, lng, tzone,
        )?;
        debug!("Fallback chain: computed natively for {}", cache_key);
        self.cache.set_panchang(&cache_key, panchang.clone()).await;
        Ok(FallbackResult {
            value: panchang,
            source: FallbackSource::NativeCalculation,
            attempts: 2,
            total_duration: start.elapsed(),
        })
    }
}

// ====================== ASTRONOMICAL HELPERS ======================

/// Compute Julian Day Number from calendar date
pub fn julian_day_number(year: i32, month: u32, day: u32) -> f64 {
    let y = year as f64;
    let m = month as f64;
    let d = day as f64;

    let a = ((14.0 - m) / 12.0).floor();
    let yy = y + 4800.0 - a;
    let mm = m + 12.0 * a - 3.0;

    d + ((153.0 * mm + 2.0) / 5.0).floor() + 365.0 * yy + (yy / 4.0).floor() - (yy / 100.0).floor()
        + (yy / 400.0).floor()
        - 32045.0
}

/// Approximate sun longitude (degrees) for a given JDN
pub fn approximate_sun_longitude(jdn: f64) -> f64 {
    let t = (jdn - 2451545.0) / 36525.0; // Julian centuries from J2000
    let mean_anomaly = (357.5291 + 35999.0503 * t) % 360.0;
    let mean_lng = (280.4664 + 36000.7698 * t) % 360.0;
    let rad = mean_anomaly.to_radians();
    let equation_center = 1.9146 * rad.sin() + 0.02 * (2.0 * rad).sin();
    ((mean_lng + equation_center) % 360.0 + 360.0) % 360.0
}

/// Get zodiac sign name from ecliptic longitude
pub fn sign_from_longitude(lng: f64) -> &'static str {
    let index = ((lng % 360.0) / 30.0).floor() as usize;
    const SIGNS: [&str; 12] = [
        "Aries",
        "Taurus",
        "Gemini",
        "Cancer",
        "Leo",
        "Virgo",
        "Libra",
        "Scorpio",
        "Sagittarius",
        "Capricorn",
        "Aquarius",
        "Pisces",
    ];
    SIGNS[index.min(11)]
}

/// Local sunrise/sunset/next-sunrise (`HH:MM` at `tz_hours`, or a
/// `polar_day` / `polar_night` marker) for the civil date at the location.
pub fn day_boundary_times(
    year: i32,
    month: u32,
    day: u32,
    lat: f64,
    lng: f64,
    tz_hours: f64,
) -> (String, String, String) {
    use crate::sun::{local_event_hhmm, SunEvent};
    let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month, day) else {
        return (String::new(), String::new(), String::new());
    };
    let next = date.succ_opt().unwrap_or(date);
    (
        local_event_hhmm(date, lat, lng, tz_hours, SunEvent::Rise),
        local_event_hhmm(date, lat, lng, tz_hours, SunEvent::Set),
        local_event_hhmm(next, lat, lng, tz_hours, SunEvent::Rise),
    )
}

// ====================== RESILIENCE METRICS ======================

/// Thread-safe metrics for monitoring resilience behavior
#[derive(Debug)]
pub struct ResilienceMetrics {
    api_successes: AtomicU64,
    api_failures: AtomicU64,
    cache_hits: AtomicU64,
    native_fallbacks: AtomicU64,
    total_retries: AtomicU64,
}

impl ResilienceMetrics {
    /// Create a new zeroed metrics instance
    pub fn new() -> Self {
        Self {
            api_successes: AtomicU64::new(0),
            api_failures: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            native_fallbacks: AtomicU64::new(0),
            total_retries: AtomicU64::new(0),
        }
    }

    pub fn record_api_success(&self) {
        self.api_successes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_api_failure(&self) {
        self.api_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_native_fallback(&self) {
        self.native_fallbacks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_retry(&self) {
        self.total_retries.fetch_add(1, Ordering::Relaxed);
    }

    pub fn api_successes(&self) -> u64 {
        self.api_successes.load(Ordering::Relaxed)
    }

    pub fn api_failures(&self) -> u64 {
        self.api_failures.load(Ordering::Relaxed)
    }

    pub fn cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    pub fn native_fallbacks(&self) -> u64 {
        self.native_fallbacks.load(Ordering::Relaxed)
    }

    pub fn total_retries(&self) -> u64 {
        self.total_retries.load(Ordering::Relaxed)
    }

    /// Calculate fallback rate as a percentage of total handled requests
    pub fn fallback_rate(&self) -> f64 {
        let total = self.api_successes() + self.api_failures() + self.native_fallbacks();
        if total == 0 {
            return 0.0;
        }
        (self.native_fallbacks() as f64 / total as f64) * 100.0
    }

    /// Get a point-in-time snapshot of all metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        let api_successes = self.api_successes();
        let api_failures = self.api_failures();
        let cache_hits = self.cache_hits();
        let native_fallbacks = self.native_fallbacks();
        let total_retries = self.total_retries();
        let total_requests = api_successes + api_failures + cache_hits + native_fallbacks;
        let fallback_rate = if total_requests == 0 {
            0.0
        } else {
            (native_fallbacks as f64 / total_requests as f64) * 100.0
        };

        MetricsSnapshot {
            api_successes,
            api_failures,
            cache_hits,
            native_fallbacks,
            total_retries,
            total_requests,
            fallback_rate,
        }
    }
}

impl Default for ResilienceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Point-in-time snapshot of resilience metrics
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub api_successes: u64,
    pub api_failures: u64,
    pub cache_hits: u64,
    pub native_fallbacks: u64,
    pub total_retries: u64,
    pub total_requests: u64,
    pub fallback_rate: f64,
}

impl std::fmt::Display for MetricsSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Resilience: API={}/{} Cache={} Native={} Retries={} Fallback={:.1}%",
            self.api_successes,
            self.api_successes + self.api_failures,
            self.cache_hits,
            self.native_fallbacks,
            self.total_retries,
            self.fallback_rate,
        )
    }
}

// ====================== TESTS ======================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_julian_day_number() {
        // J2000.0 = 2451545.0 for 2000-01-01 12:00
        let jdn = julian_day_number(2000, 1, 1);
        assert!((jdn - 2451545.0).abs() < 1.0);
    }

    #[test]
    fn test_sign_from_longitude() {
        assert_eq!(sign_from_longitude(0.0), "Aries");
        assert_eq!(sign_from_longitude(30.0), "Taurus");
        assert_eq!(sign_from_longitude(120.0), "Leo");
        assert_eq!(sign_from_longitude(270.0), "Capricorn");
        assert_eq!(sign_from_longitude(359.9), "Pisces");
    }

    #[test]
    fn test_backoff_delay_no_jitter() {
        let config = BackoffConfig {
            initial_delay_ms: 1000,
            max_delay_ms: 16000,
            max_retries: 5,
            multiplier: 2.0,
            jitter: false,
        };
        let backoff = ExponentialBackoff::new(config);

        assert_eq!(backoff.delay_for_attempt(0).as_millis(), 1000);
        assert_eq!(backoff.delay_for_attempt(1).as_millis(), 2000);
        assert_eq!(backoff.delay_for_attempt(2).as_millis(), 4000);
        assert_eq!(backoff.delay_for_attempt(3).as_millis(), 8000);
        assert_eq!(backoff.delay_for_attempt(4).as_millis(), 16000);
        assert_eq!(backoff.delay_for_attempt(5).as_millis(), 16000); // capped
    }

    #[test]
    fn test_metrics_thread_safe() {
        let metrics = ResilienceMetrics::new();
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let m = &metrics as *const ResilienceMetrics as usize;
                std::thread::spawn(move || {
                    let metrics = unsafe { &*(m as *const ResilienceMetrics) };
                    metrics.record_api_success();
                })
            })
            .collect();

        for h in handles {
            h.join().unwrap();
        }

        assert_eq!(metrics.api_successes(), 10);
    }

    #[test]
    fn test_day_boundary_times_bengaluru() {
        // Bengaluru 2026-01-15, IST: Swiss Ephemeris 06:45 / 18:12 (#1537).
        // Times are rounded to the nearest minute, so allow one minute.
        let (rise, set, next) = day_boundary_times(2026, 1, 15, 12.9716, 77.5946, 5.5);
        assert!(
            ["06:45", "06:46"].contains(&rise.as_str()),
            "sunrise {rise}"
        );
        assert!(
            ["18:11", "18:12", "18:13"].contains(&set.as_str()),
            "sunset {set}"
        );
        assert!(
            ["06:45", "06:46"].contains(&next.as_str()),
            "next sunrise {next}"
        );
    }

    #[test]
    fn test_day_boundary_times_polar() {
        // Longyearbyen: polar night in December, midnight sun in June.
        let (rise, set, next) = day_boundary_times(2026, 12, 21, 78.22, 15.65, 1.0);
        assert_eq!(
            (rise.as_str(), set.as_str()),
            ("polar_night", "polar_night")
        );
        assert_eq!(next, "polar_night");
        let (rise, set, _) = day_boundary_times(2026, 6, 21, 78.22, 15.65, 2.0);
        assert_eq!((rise.as_str(), set.as_str()), ("polar_day", "polar_day"));
    }
}
