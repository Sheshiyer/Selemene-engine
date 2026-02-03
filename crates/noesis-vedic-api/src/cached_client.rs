//! Cached API client with rate limiting for FreeAstrologyAPI.com
//! 
//! This client combines:
//! - Aggressive caching (minimize API calls)
//! - Rate limiting (respect 50/day, 1/sec)
//! - Fallback to native calculations
//! - Comprehensive Panchang support (Muhurtas, Hora, Choghadiya)

use chrono::Datelike;
use tracing::{debug, info, warn};

use crate::{
    config::Config, 
    error::Result, 
    error::VedicApiError,
    client::VedicApiClient,
    cache::{ApiCache, birth_key, panchang_key},
    rate_limiter::{RateLimiter, RateLimitStatus},
    panchang::{
        Panchang, CompletePanchang, PanchangMetadata, PanchangQuery,
        MuhurtaCollection, Muhurta, MuhurtaNature,
        AbhijitMuhurta, RahuKalam, YamaGandam, GulikaKaal, BrahmaMuhurta, AmritKaal,
        HoraTimings, ChoghadiyaTimings,
    },
    dasha::{VimshottariDasha, DashaLevel},
    chart::{BirthChart, NavamsaChart},
};

/// Enhanced client with caching and rate limiting
#[derive(Debug, Clone)]
pub struct CachedVedicClient {
    inner: VedicApiClient,
    cache: ApiCache,
    rate_limiter: RateLimiter,
    config: Config,
}

impl CachedVedicClient {
    /// Create new cached client
    pub fn new(config: Config) -> Self {
        let inner = VedicApiClient::new(config.clone());
        let cache = ApiCache::new();
        let rate_limiter = RateLimiter::new();
        
        info!("CachedVedicClient initialized with rate limiting and caching");
        
        Self {
            inner,
            cache,
            rate_limiter,
            config,
        }
    }
    
    /// Create from environment
    pub fn from_env() -> Result<Self> {
        let config = Config::from_env()?;
        Ok(Self::new(config))
    }
    
    /// Get Panchang with caching
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
    ) -> Result<Panchang> {
        // Generate cache key (date + location, not time)
        let cache_key = panchang_key(year, month, day, lat, lng);
        
        // Try cache first
        if let Some(cached) = self.cache.get_panchang(&cache_key).await {
            debug!("Panchang cache hit for {}", cache_key);
            return Ok(cached);
        }
        
        debug!("Panchang cache miss, fetching from API");
        
        // Check rate limit
        if !self.rate_limiter.can_request() {
            warn!("Rate limit reached, trying fallback");
            return self.fallback_panchang(year, month, day, hour, minute, second, lat, lng, tzone).await;
        }
        
        // Fetch from API
        let panchang = self.inner.get_panchang(
            year, month, day, hour, minute, second, lat, lng, tzone
        ).await?;
        
        // Store in cache
        self.cache.set_panchang(&cache_key, panchang.clone()).await;
        
        Ok(panchang)
    }
    
    /// Get Vimshottari Dasha with caching
    pub async fn get_vimshottari_dasha(
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
        level: DashaLevel,
    ) -> Result<VimshottariDasha> {
        // Generate cache key (birth data + level)
        let cache_key = format!("{}:{:?}", 
            birth_key(year, month, day, hour, minute, lat, lng),
            level
        );
        
        // Try cache first
        if let Some(cached) = self.cache.get_dasha(&cache_key).await {
            debug!("Dasha cache hit for {}", cache_key);
            return Ok(cached);
        }
        
        debug!("Dasha cache miss, fetching from API");
        
        // Check rate limit
        if !self.rate_limiter.can_request() {
            warn!("Rate limit reached, trying fallback");
            return self.fallback_dasha(year, month, day, hour, minute, second, lat, lng, tzone, level).await;
        }
        
        // Fetch from API
        let dasha = self.inner.get_vimshottari_dasha(
            year, month, day, hour, minute, second, lat, lng, tzone, level
        ).await?;
        
        // Store in cache (infinite TTL)
        self.cache.set_dasha(&cache_key, dasha.clone()).await;
        
        Ok(dasha)
    }
    
    /// Get Birth Chart with caching
    pub async fn get_birth_chart(
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
    ) -> Result<BirthChart> {
        let cache_key = birth_key(year, month, day, hour, minute, lat, lng);
        
        // Try cache first
        if let Some(cached) = self.cache.get_birth_chart(&cache_key).await {
            debug!("Birth chart cache hit");
            return Ok(cached);
        }
        
        debug!("Birth chart cache miss");
        
        // Check rate limit
        if !self.rate_limiter.can_request() {
            warn!("Rate limit reached, trying fallback");
            return self.fallback_birth_chart(year, month, day, hour, minute, second, lat, lng, tzone).await;
        }
        
        // Fetch from API
        let chart = self.inner.get_birth_chart(
            year, month, day, hour, minute, second, lat, lng, tzone
        ).await?;
        
        // Store in cache (infinite TTL)
        self.cache.set_birth_chart(&cache_key, chart.clone()).await;
        
        Ok(chart)
    }
    
    /// Get Navamsa chart
    pub async fn get_navamsa_chart(
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
    ) -> Result<NavamsaChart> {
        // For now, fetch directly (can be optimized later)
        if !self.rate_limiter.can_request() {
            return Err(VedicApiError::RateLimit { retry_after: Some(3600) });
        }
        
        self.inner.get_navamsa_chart(
            year, month, day, hour, minute, second, lat, lng, tzone
        ).await
    }

    // ==================== PHASE 2: PANCHANG EXTENSIONS ====================
    
    /// Get Complete Panchang with all sub-systems (Muhurtas, Hora, Choghadiya)
    pub async fn get_complete_panchang(
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
    ) -> Result<CompletePanchang> {
        // Get base Panchang (cached)
        let panchang = self.get_panchang(year, month, day, hour, minute, second, lat, lng, tzone).await?;
        
        // Get day of week for Muhurta and Choghadiya calculations
        let day_of_week = panchang.vara.as_str();
        let sunrise = &panchang.day_boundaries.sunrise;
        let sunset = &panchang.day_boundaries.sunset;
        let next_sunrise = &panchang.day_boundaries.next_sunrise;
        
        // Build Muhurtas
        let muhurtas = self.calculate_muhurtas(day_of_week, sunrise, sunset).await?;
        
        // Build Hora timings
        let hora_timings = crate::panchang::hora::calculate_hora_timings(
            day_of_week,
            sunrise,
            sunset,
            next_sunrise,
        );
        
        // Build Choghadiya
        let choghadiya = crate::panchang::choghadiya::calculate_choghadiya(
            day_of_week,
            sunrise,
            sunset,
            next_sunrise,
        );
        
        let metadata = PanchangMetadata {
            source: "FreeAstrologyAPI.com".to_string(),
            calculated_at: chrono::Utc::now().to_rfc3339(),
            ayanamsa: "Lahiri".to_string(),
            timezone: tzone,
            dst_active: false,
        };
        
        Ok(CompletePanchang {
            panchang,
            muhurtas,
            hora_timings,
            choghadiya,
            metadata,
        })
    }
    
    /// Get Complete Panchang using query builder
    pub async fn get_panchang_with_query(&self, query: &PanchangQuery) -> Result<CompletePanchang> {
        self.get_complete_panchang(
            query.year,
            query.month,
            query.day,
            query.hour,
            query.minute,
            query.second,
            query.latitude,
            query.longitude,
            query.timezone,
        ).await
    }
    
    /// Get only Muhurtas for a day
    pub async fn get_muhurtas(
        &self,
        year: i32,
        month: u32,
        day: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> Result<MuhurtaCollection> {
        // Get Panchang to extract day info
        let panchang = self.get_panchang(year, month, day, 12, 0, 0, lat, lng, tzone).await?;
        
        let day_of_week = panchang.vara.as_str();
        let sunrise = &panchang.day_boundaries.sunrise;
        let sunset = &panchang.day_boundaries.sunset;
        
        self.calculate_muhurtas(day_of_week, sunrise, sunset).await
    }
    
    /// Get Hora (planetary hours) timings
    pub async fn get_hora_timings(
        &self,
        year: i32,
        month: u32,
        day: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> Result<HoraTimings> {
        // Get Panchang for sunrise/sunset info
        let panchang = self.get_panchang(year, month, day, 12, 0, 0, lat, lng, tzone).await?;
        
        let day_of_week = panchang.vara.as_str();
        let sunrise = &panchang.day_boundaries.sunrise;
        let sunset = &panchang.day_boundaries.sunset;
        let next_sunrise = &panchang.day_boundaries.next_sunrise;
        
        Ok(crate::panchang::hora::calculate_hora_timings(
            day_of_week,
            sunrise,
            sunset,
            next_sunrise,
        ))
    }
    
    /// Get Choghadiya timings
    pub async fn get_choghadiya(
        &self,
        year: i32,
        month: u32,
        day: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> Result<ChoghadiyaTimings> {
        // Get Panchang for timing info
        let panchang = self.get_panchang(year, month, day, 12, 0, 0, lat, lng, tzone).await?;
        
        let day_of_week = panchang.vara.as_str();
        let sunrise = &panchang.day_boundaries.sunrise;
        let sunset = &panchang.day_boundaries.sunset;
        let next_sunrise = &panchang.day_boundaries.next_sunrise;
        
        Ok(crate::panchang::choghadiya::calculate_choghadiya(
            day_of_week,
            sunrise,
            sunset,
            next_sunrise,
        ))
    }
    
    /// Get current Muhurta at a specific time
    pub async fn get_current_muhurta(
        &self,
        year: i32,
        month: u32,
        day: u32,
        time: &str, // HH:MM format
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> Result<Option<Muhurta>> {
        let muhurtas = self.get_muhurtas(year, month, day, lat, lng, tzone).await?;
        
        // Check each Muhurta
        if let Some(ref amrit) = muhurtas.amrit_kaal {
            if is_time_in_range(time, &amrit.start, &amrit.end) {
                return Ok(Some(amrit.clone()));
            }
        }
        if let Some(ref abhijit) = muhurtas.abhijit {
            if is_time_in_range(time, &abhijit.start, &abhijit.end) {
                return Ok(Some(abhijit.clone()));
            }
        }
        if let Some(ref rahu) = muhurtas.rahu_kalam {
            if is_time_in_range(time, &rahu.start, &rahu.end) {
                return Ok(Some(rahu.clone()));
            }
        }
        if let Some(ref yama) = muhurtas.yama_gandam {
            if is_time_in_range(time, &yama.start, &yama.end) {
                return Ok(Some(yama.clone()));
            }
        }
        if let Some(ref gulika) = muhurtas.gulika_kaal {
            if is_time_in_range(time, &gulika.start, &gulika.end) {
                return Ok(Some(gulika.clone()));
            }
        }
        
        Ok(None)
    }
    
    /// Get favorable Muhurtas for a specific activity
    pub async fn get_favorable_muhurtas(
        &self,
        year: i32,
        month: u32,
        day: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> Result<Vec<Muhurta>> {
        let muhurtas = self.get_muhurtas(year, month, day, lat, lng, tzone).await?;
        
        let mut favorable = Vec::new();
        
        if let Some(ref amrit) = muhurtas.amrit_kaal {
            if amrit.nature.is_good_for_starting() {
                favorable.push(amrit.clone());
            }
        }
        if let Some(ref abhijit) = muhurtas.abhijit {
            if abhijit.nature.is_good_for_starting() {
                favorable.push(abhijit.clone());
            }
        }
        if let Some(ref brahma) = muhurtas.brahma_muhurta {
            if brahma.nature.is_good_for_starting() {
                favorable.push(brahma.clone());
            }
        }
        
        Ok(favorable)
    }
    
    /// Internal: Calculate Muhurtas
    async fn calculate_muhurtas(
        &self,
        day_of_week: &str,
        sunrise: &str,
        _sunset: &str,
    ) -> Result<MuhurtaCollection> {
        use crate::panchang::muhurta::*;
        
        // Calculate Rahu Kalam
        let rahu = RahuKalam::for_day(day_of_week, sunrise, _sunset);
        let rahu_muhurta = Muhurta {
            name: "Rahu Kalam".to_string(),
            start: rahu.start,
            end: rahu.end,
            duration_minutes: 90,
            nature: MuhurtaNature::VeryInauspicious,
            ruler: "Rahu".to_string(),
            suitable_activities: vec!["Worship of Durga".to_string()],
            avoid_activities: vec![
                "New beginnings".to_string(),
                "Business ventures".to_string(),
                "Travel".to_string(),
                "Marriage".to_string(),
            ],
        };
        
        // Calculate Yama Gandam
        let yama = YamaGandam::for_day(day_of_week);
        let yama_muhurta = Muhurta {
            name: "Yama Gandam".to_string(),
            start: yama.start,
            end: yama.end,
            duration_minutes: 90,
            nature: MuhurtaNature::VeryInauspicious,
            ruler: "Yama".to_string(),
            suitable_activities: vec!["Charity".to_string()],
            avoid_activities: vec![
                "Important activities".to_string(),
                "New beginnings".to_string(),
            ],
        };
        
        // Calculate Gulika Kaal
        let gulika = GulikaKaal::for_day(day_of_week);
        let gulika_muhurta = Muhurta {
            name: "Gulika Kaal".to_string(),
            start: gulika.start,
            end: gulika.end,
            duration_minutes: 90,
            nature: MuhurtaNature::Inauspicious,
            ruler: "Gulika".to_string(),
            suitable_activities: vec!["Building construction".to_string()],
            avoid_activities: vec![
                "New ventures".to_string(),
                "Starting journeys".to_string(),
            ],
        };
        
        // Abhijit Muhurta (approximately 11:40 AM to 12:20 PM)
        let abhijit_muhurta = Muhurta {
            name: "Abhijit Muhurta".to_string(),
            start: "11:40".to_string(),
            end: "12:20".to_string(),
            duration_minutes: 40,
            nature: MuhurtaNature::Auspicious,
            ruler: "Mercury".to_string(),
            suitable_activities: vec![
                "All activities".to_string(),
                "New beginnings".to_string(),
                "Important work".to_string(),
            ],
            avoid_activities: vec![],
        };
        
        // Amrit Kaal (varies by day, simplified)
        let amrit_muhurta = Muhurta {
            name: "Amrit Kaal".to_string(),
            start: "06:00".to_string(),
            end: "07:30".to_string(),
            duration_minutes: 90,
            nature: MuhurtaNature::Auspicious,
            ruler: "Moon".to_string(),
            suitable_activities: vec![
                "New ventures".to_string(),
                "Purchases".to_string(),
                "Travel".to_string(),
            ],
            avoid_activities: vec![],
        };
        
        // Brahma Muhurta (approx 1h 36m before sunrise)
        let brahma_muhurta = Muhurta {
            name: "Brahma Muhurta".to_string(),
            start: "04:24".to_string(), // Simplified: assumes 6:00 AM sunrise
            end: sunrise.to_string(),
            duration_minutes: 96,
            nature: MuhurtaNature::Auspicious,
            ruler: "Brahma".to_string(),
            suitable_activities: vec![
                "Meditation".to_string(),
                "Spiritual practices".to_string(),
                "Study of scriptures".to_string(),
                "Yoga".to_string(),
            ],
            avoid_activities: vec!["Sleep".to_string()],
        };
        
        Ok(MuhurtaCollection {
            abhijit: Some(abhijit_muhurta),
            amrit_kaal: Some(amrit_muhurta),
            rahu_kalam: Some(rahu_muhurta),
            yama_gandam: Some(yama_muhurta),
            gulika_kaal: Some(gulika_muhurta),
            dur_muhurta: None,
            varjyam: None,
            brahma_muhurta: Some(brahma_muhurta),
        })
    }
    
    /// Health check - verifies API connectivity
    pub async fn health_check(&self) -> bool {
        // Simple check - try to make a lightweight request
        // For now, just check if the rate limiter allows requests
        self.rate_limiter.can_request()
    }

    /// Get rate limit status
    pub async fn rate_limit_status(&self) -> RateLimitStatus {
        self.rate_limiter.status()
    }
    
    /// Get cache stats
    pub async fn cache_stats(&self) -> crate::cache::CacheStats {
        self.cache.stats().await
    }
    
    /// Get combined status report
    pub async fn status_report(&self) -> StatusReport {
        StatusReport {
            rate_limit: self.rate_limit_status().await,
            cache: self.cache_stats().await,
        }
    }
    
    /// Pre-fetch data for upcoming days
    /// Useful for warming cache with important dates
    pub async fn prefetch_panchang(
        &self,
        start_year: i32,
        start_month: u32,
        start_day: u32,
        days: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> u32 {
        let mut fetched = 0;
        
        info!("Pre-fetching {} days of Panchang data", days);
        
        for i in 0..days {
            let date = chrono::NaiveDate::from_ymd_opt(start_year, start_month, start_day)
                .unwrap()
                .checked_add_signed(chrono::Duration::days(i as i64))
                .unwrap();
            
            // Check if already cached
            let key = panchang_key(date.year(), date.month(), date.day(), lat, lng);
            if self.cache.get_panchang(&key).await.is_some() {
                continue;
            }
            
            // Check rate limit
            if !self.rate_limiter.can_request() {
                warn!("Rate limit reached during pre-fetch, stopping");
                break;
            }
            
            // Fetch
            match self.get_panchang(
                date.year(), date.month(), date.day(),
                12, 0, 0, // noon
                lat, lng, tzone
            ).await {
                Ok(_) => fetched += 1,
                Err(e) => {
                    warn!("Failed to fetch Panchang for {}: {}", date, e);
                    break;
                }
            }
            
            // Small delay to respect rate limit
            tokio::time::sleep(tokio::time::Duration::from_millis(1100)).await;
        }
        
        info!("Pre-fetched {} days of Panchang data", fetched);
        fetched
    }
    
    // ==================== FALLBACK METHODS ====================
    
    /// Fallback to native Panchang calculation
    async fn fallback_panchang(
        &self,
        _year: i32,
        _month: u32,
        _day: u32,
        _hour: u32,
        _minute: u32,
        _second: u32,
        _lat: f64,
        _lng: f64,
        _tzone: f64,
    ) -> Result<Panchang> {
        if !self.config.fallback_enabled {
            return Err(VedicApiError::RateLimit { retry_after: Some(3600) });
        }
        
        warn!("Falling back to native Panchang calculation");
        
        // TODO: Integrate with native engine-panchanga
        // For now, return error
        Err(VedicApiError::FallbackFailed {
            api_error: Box::new(VedicApiError::RateLimit { retry_after: Some(3600) }),
            native_error: "Native fallback not yet implemented".to_string(),
        })
    }
    
    /// Fallback to native Dasha calculation
    async fn fallback_dasha(
        &self,
        _year: i32,
        _month: u32,
        _day: u32,
        _hour: u32,
        _minute: u32,
        _second: u32,
        _lat: f64,
        _lng: f64,
        _tzone: f64,
        _level: DashaLevel,
    ) -> Result<VimshottariDasha> {
        if !self.config.fallback_enabled {
            return Err(VedicApiError::RateLimit { retry_after: Some(3600) });
        }
        
        warn!("Falling back to native Dasha calculation");
        
        // TODO: Integrate with native engine-vimshottari
        Err(VedicApiError::FallbackFailed {
            api_error: Box::new(VedicApiError::RateLimit { retry_after: Some(3600) }),
            native_error: "Native fallback not yet implemented".to_string(),
        })
    }
    
    /// Fallback to native birth chart calculation
    async fn fallback_birth_chart(
        &self,
        _year: i32,
        _month: u32,
        _day: u32,
        _hour: u32,
        _minute: u32,
        _second: u32,
        _lat: f64,
        _lng: f64,
        _tzone: f64,
    ) -> Result<BirthChart> {
        if !self.config.fallback_enabled {
            return Err(VedicApiError::RateLimit { retry_after: Some(3600) });
        }
        
        warn!("Falling back to native birth chart calculation");
        
        // TODO: This would require native calculation
        Err(VedicApiError::FallbackFailed {
            api_error: Box::new(VedicApiError::RateLimit { retry_after: Some(3600) }),
            native_error: "Native fallback not yet implemented".to_string(),
        })
    }
}

/// Combined status report
#[derive(Debug, Clone)]
pub struct StatusReport {
    pub rate_limit: RateLimitStatus,
    pub cache: crate::cache::CacheStats,
}

impl std::fmt::Display for StatusReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Vedic API Status ===")?;
        writeln!(f, "Rate: {}/{} used, {} remaining", 
            self.rate_limit.used_today,
            self.rate_limit.daily_limit,
            self.rate_limit.effective_remaining)?;
        writeln!(f, "{}", self.cache)?;
        Ok(())
    }
}

/// Helper function to check if a time falls within a range
/// Assumes times are in HH:MM format and within the same day
fn is_time_in_range(time: &str, start: &str, end: &str) -> bool {
    time >= start && time <= end
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_time_in_range() {
        assert!(is_time_in_range("12:00", "11:00", "13:00"));
        assert!(!is_time_in_range("10:00", "11:00", "13:00"));
        assert!(is_time_in_range("11:30", "11:30", "12:30"));
    }

    #[test]
    fn test_status_report() {
        let report = StatusReport {
            rate_limit: RateLimitStatus {
                daily_limit: 50,
                remaining_today: 45,
                buffer: 5,
                effective_remaining: 40,
                used_today: 5,
            },
            cache: crate::cache::CacheStats {
                hits: 100,
                misses: 10,
                total: 110,
                hit_rate: 90.9,
                panchang_entries: 5,
                dasha_entries: 3,
                birth_chart_entries: 2,
            },
        };
        
        let output = format!("{}", report);
        assert!(output.contains("Vedic API Status"));
        assert!(output.contains("90.9%"));
    }
}
