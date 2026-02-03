//! Resilience layer: exponential backoff, fallback chain, and metrics
//!
//! FAPI-098: API Fallback to Native Calculation
//! FAPI-105: Rate Limit Handling with Exponential Backoff

use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tracing::{debug, info, warn, error};

use crate::{
    Config, VedicApiClient, VedicApiError,
    cache::{ApiCache, panchang_key},
    panchang::Panchang,
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
        let delay_ms = self.config.initial_delay_ms as f64
            * self.config.multiplier.powi(attempt as i32);
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

/// Orchestrates the fallback chain: API -> Cache -> Native Calculation
#[derive(Debug)]
pub struct FallbackChain {
    client: VedicApiClient,
    cache: ApiCache,
    backoff: ExponentialBackoff,
    metrics: ResilienceMetrics,
    config: Config,
}

impl FallbackChain {
    /// Create a new fallback chain with the given API configuration
    pub fn new(config: Config) -> Self {
        let client = VedicApiClient::new(config.clone());
        let cache = ApiCache::new();
        let backoff = ExponentialBackoff::new(BackoffConfig {
            initial_delay_ms: 100,
            max_delay_ms: 5000,
            max_retries: 2,
            multiplier: 2.0,
            jitter: false,
        });
        let metrics = ResilienceMetrics::new();

        Self {
            client,
            cache,
            backoff,
            metrics,
            config,
        }
    }

    /// Create with custom backoff configuration
    pub fn with_backoff(mut self, config: BackoffConfig) -> Self {
        self.backoff = ExponentialBackoff::new(config);
        self
    }

    /// Get a reference to the resilience metrics
    pub fn metrics(&self) -> &ResilienceMetrics {
        &self.metrics
    }

    /// Execute the full fallback chain for a Panchang request:
    /// 1. Try cache first (fast path)
    /// 2. Try API with backoff
    /// 3. Fall back to native calculation
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
        let mut attempts: u32 = 0;

        // ---- Step 1: Check cache ----
        attempts += 1;
        if let Some(cached) = self.cache.get_panchang(&cache_key).await {
            debug!("Fallback chain: cache hit for {}", cache_key);
            self.metrics.record_cache_hit();
            return Ok(FallbackResult {
                value: cached,
                source: FallbackSource::Cache,
                attempts,
                total_duration: start.elapsed(),
            });
        }

        // ---- Step 2: Try API with backoff ----
        attempts += 1;
        let client = self.client.clone();
        let api_result = self
            .backoff
            .execute(|| {
                let c = client.clone();
                async move {
                    c.get_panchang(year, month, day, hour, minute, second, lat, lng, tzone)
                        .await
                }
            })
            .await;

        match api_result {
            Ok(panchang) => {
                info!("Fallback chain: API success for {}", cache_key);
                self.metrics.record_api_success();
                // Populate cache for next time
                self.cache.set_panchang(&cache_key, panchang.clone()).await;
                return Ok(FallbackResult {
                    value: panchang,
                    source: FallbackSource::Api,
                    attempts,
                    total_duration: start.elapsed(),
                });
            }
            Err(api_err) => {
                warn!("Fallback chain: API failed ({}), trying native", api_err);
                self.metrics.record_api_failure();

                // ---- Step 3: Native calculation fallback ----
                if self.config.fallback_enabled {
                    attempts += 1;
                    match self.native_panchang(year, month, day, hour, minute, second, lat, lng, tzone) {
                        Ok(panchang) => {
                            info!("Fallback chain: native calculation success");
                            self.metrics.record_native_fallback();
                            // Cache the native result too
                            self.cache.set_panchang(&cache_key, panchang.clone()).await;
                            return Ok(FallbackResult {
                                value: panchang,
                                source: FallbackSource::NativeCalculation,
                                attempts,
                                total_duration: start.elapsed(),
                            });
                        }
                        Err(native_err) => {
                            error!(
                                "Fallback chain: all sources failed. API: {}, Native: {}",
                                api_err, native_err
                            );
                            return Err(VedicApiError::FallbackFailed {
                                api_error: Box::new(api_err),
                                native_error: native_err,
                            });
                        }
                    }
                } else {
                    return Err(api_err);
                }
            }
        }
    }

    /// Native Panchang calculation fallback.
    /// Returns a basic Panchang with approximate values computed locally.
    fn native_panchang(
        &self,
        year: i32,
        month: u32,
        day: u32,
        _hour: u32,
        _minute: u32,
        _second: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> std::result::Result<Panchang, String> {
        use crate::panchang::*;

        // Compute Julian Day Number for basic astronomical reference
        let jdn = julian_day_number(year, month, day);

        // Approximate tithi from lunar phase (synodic month ~ 29.53 days)
        let lunar_age = (jdn - 2451550.1) % 29.530588; // Reference new moon
        let tithi_num = ((lunar_age / 29.530588) * 30.0).floor() as u32 + 1;
        let tithi_num = tithi_num.min(30).max(1);
        let tithi_name = TithiName::from_number(tithi_num);
        let paksha = if tithi_num <= 15 {
            Paksha::Shukla
        } else {
            Paksha::Krishna
        };

        // Approximate nakshatra from lunar longitude
        // Moon moves ~13.2 degrees/day, 27 nakshatras span 360 degrees
        let moon_lng = (lunar_age * 13.176) % 360.0;
        let nakshatra_num = ((moon_lng / 13.333).floor() as u32 + 1).min(27);
        let nakshatra_name = NakshatraName::from_number(nakshatra_num);
        let pada = ((moon_lng % 13.333) / 3.333).floor() as u32 + 1;

        // Yoga: sum of sun and moon longitudes / 13.333
        let sun_lng = approximate_sun_longitude(jdn);
        let yoga_value = (sun_lng + moon_lng) % 360.0;
        let yoga_num = ((yoga_value / 13.333).floor() as u32 + 1).min(27);
        let yoga_name = YogaName::from_number(yoga_num);

        // Karana: half of tithi
        let karana_num = ((tithi_num - 1) * 2 + 1).min(60);
        let karana_name = KaranaName::from_number(karana_num);

        // Vara (day of week) from JDN
        let vara_num = (((jdn as i64 + 1) % 7) as u8).max(1);
        let vara = Vara::from_number(vara_num).unwrap_or(Vara::Monday);

        // Approximate sunrise/sunset (simplified for latitude)
        let sunrise = approximate_sunrise(lat, jdn);
        let sunset = approximate_sunset(lat, jdn);

        Ok(Panchang {
            date: DateInfo {
                year,
                month,
                day,
                day_of_week: vara_num,
                julian_day: jdn,
                hindu_date: None,
            },
            location: Location {
                latitude: lat,
                longitude: lng,
                timezone: tzone,
                name: None,
            },
            tithi: Tithi {
                number: tithi_num as u8,
                name_tithi: tithi_name,
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
                is_complete: true,
            },
            nakshatra: Nakshatra {
                number: nakshatra_num as u8,
                name_nakshatra: nakshatra_name,
                pada: pada as u8,
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
                longitude: moon_lng,
            },
            yoga: Yoga {
                number: yoga_num as u8,
                name_yoga: yoga_name,
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
            },
            karana: Karana {
                name_karana: karana_name,
                karana_type: KaranaType::Movable,
                start_time: "00:00".to_string(),
                end_time: "23:59".to_string(),
            },
            vara,
            paksha,
            planets: PlanetaryPositions {
                sun: PlanetPosition {
                    name: "Sun".to_string(),
                    longitude: sun_lng,
                    latitude: 0.0,
                    speed: 1.0,
                    sign: sign_from_longitude(sun_lng).to_string(),
                    nakshatra: "Native".to_string(),
                    pada: 1,
                    is_retrograde: false,
                },
                moon: PlanetPosition {
                    name: "Moon".to_string(),
                    longitude: moon_lng,
                    latitude: 0.0,
                    speed: 13.2,
                    sign: sign_from_longitude(moon_lng).to_string(),
                    nakshatra: "Native".to_string(),
                    pada: pada as u8,
                    is_retrograde: false,
                },
                mars: None,
                mercury: None,
                jupiter: None,
                venus: None,
                saturn: None,
                rahu: None,
                ketu: None,
            },
            day_boundaries: DayBoundaries {
                sunrise: sunrise.clone(),
                sunset: sunset.clone(),
                next_sunrise: sunrise,
                day_duration: "12:00".to_string(),
                night_duration: "12:00".to_string(),
            },
            ayanamsa: 24.17, // Approximate Lahiri ayanamsa for modern era
        })
    }
}

// ====================== ASTRONOMICAL HELPERS ======================

/// Compute Julian Day Number from calendar date
fn julian_day_number(year: i32, month: u32, day: u32) -> f64 {
    let y = year as f64;
    let m = month as f64;
    let d = day as f64;

    let a = ((14.0 - m) / 12.0).floor();
    let yy = y + 4800.0 - a;
    let mm = m + 12.0 * a - 3.0;

    d + ((153.0 * mm + 2.0) / 5.0).floor() + 365.0 * yy
        + (yy / 4.0).floor()
        - (yy / 100.0).floor()
        + (yy / 400.0).floor()
        - 32045.0
}

/// Approximate sun longitude (degrees) for a given JDN
fn approximate_sun_longitude(jdn: f64) -> f64 {
    let t = (jdn - 2451545.0) / 36525.0; // Julian centuries from J2000
    let mean_anomaly = (357.5291 + 35999.0503 * t) % 360.0;
    let mean_lng = (280.4664 + 36000.7698 * t) % 360.0;
    let rad = mean_anomaly.to_radians();
    let equation_center = 1.9146 * rad.sin() + 0.02 * (2.0 * rad).sin();
    ((mean_lng + equation_center) % 360.0 + 360.0) % 360.0
}

/// Get zodiac sign name from ecliptic longitude
fn sign_from_longitude(lng: f64) -> &'static str {
    let index = ((lng % 360.0) / 30.0).floor() as usize;
    const SIGNS: [&str; 12] = [
        "Aries", "Taurus", "Gemini", "Cancer", "Leo", "Virgo",
        "Libra", "Scorpio", "Sagittarius", "Capricorn", "Aquarius", "Pisces",
    ];
    SIGNS[index.min(11)]
}

/// Approximate sunrise time string for a latitude and JDN
fn approximate_sunrise(lat: f64, jdn: f64) -> String {
    // Simplified sunrise calculation
    let day_of_year = ((jdn - julian_day_number(2024, 1, 1)) % 365.25) as f64;
    let declination = 23.45 * ((360.0 / 365.0 * (day_of_year + 284.0)).to_radians().sin());
    let lat_rad = lat.to_radians();
    let decl_rad = declination.to_radians();

    let cos_ha = -(lat_rad.tan() * decl_rad.tan());
    let ha = if cos_ha.abs() > 1.0 {
        90.0 // Polar edge case
    } else {
        cos_ha.acos().to_degrees()
    };

    let sunrise_hours = 12.0 - ha / 15.0;
    let h = sunrise_hours.floor() as u32;
    let m = ((sunrise_hours - h as f64) * 60.0).round() as u32;
    format!("{:02}:{:02}", h.min(23), m.min(59))
}

/// Approximate sunset time string for a latitude and JDN
fn approximate_sunset(lat: f64, jdn: f64) -> String {
    let day_of_year = ((jdn - julian_day_number(2024, 1, 1)) % 365.25) as f64;
    let declination = 23.45 * ((360.0 / 365.0 * (day_of_year + 284.0)).to_radians().sin());
    let lat_rad = lat.to_radians();
    let decl_rad = declination.to_radians();

    let cos_ha = -(lat_rad.tan() * decl_rad.tan());
    let ha = if cos_ha.abs() > 1.0 {
        90.0
    } else {
        cos_ha.acos().to_degrees()
    };

    let sunset_hours = 12.0 + ha / 15.0;
    let h = sunset_hours.floor() as u32;
    let m = ((sunset_hours - h as f64) * 60.0).round() as u32;
    format!("{:02}:{:02}", h.min(23), m.min(59))
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
        let total = self.api_successes()
            + self.api_failures()
            + self.native_fallbacks();
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
    fn test_approximate_sunrise_reasonable() {
        // Bangalore at equinox-ish
        let sunrise = approximate_sunrise(12.97, julian_day_number(2024, 3, 20));
        // Should be roughly 6:00-6:30
        assert!(sunrise.starts_with("06:") || sunrise.starts_with("05:"),
            "Sunrise at lat 12.97 should be around 6am, got {}", sunrise);
    }
}
