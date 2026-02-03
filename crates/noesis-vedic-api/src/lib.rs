//! Noesis Vedic API Client
//!
//! A comprehensive async client for FreeAstrologyAPI.com with:
//! - Built-in rate limiting (50 req/day with 1 req/sec throttle)
//! - Intelligent caching (birth data: infinite TTL, daily data: 24h TTL)
//! - Automatic fallback to native calculations
//! - Full Panchang support (Tithi, Nakshatra, Yoga, Karana, Vara)
//! - Muhurta calculations (Abhijit, Rahu Kalam, Yama Gandam, etc.)
//! - Hora (planetary hours)
//! - Choghadiya timings
//!
//! # Quick Start
//!
//! ```no_run
//! use noesis_vedic_api::CachedVedicClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize client from environment
//!     let client = CachedVedicClient::from_env()?;
//!
//!     // Get Panchang for a date
//!     let panchang = client.get_panchang(2024, 1, 1, 12, 0, 0, 12.97, 77.59, 5.5).await?;
//!     println!("Tithi: {}", panchang.tithi.name());
//!
//!     // Get Vimshottari Dasha
//!     use noesis_vedic_api::dasha::DashaLevel;
//!     let dasha = client.get_vimshottari_dasha(1990, 1, 1, 12, 0, 0, 28.61, 77.23, 5.5, DashaLevel::Mahadasha).await?;
//!     println!("Birth Moon Nakshatra: {}", dasha.moon_nakshatra);
//!
//!     // Check rate limit status
//!     let status = client.status_report().await;
//!     println!("Remaining API calls today: {}", status.rate_limit.remaining_today);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Rate Limiting
//!
//! The free tier allows 50 requests per day with 1 request per second rate limit.
//! This client implements a 5-request safety buffer, giving you 45 usable calls per day.
//! Rate limit status is automatically tracked and reported.
//!
//! # Caching Strategy
//!
//! - **Birth Data** (Dasha, Birth Chart): Cached indefinitely (birth data never changes)
//! - **Daily Data** (Panchang, Muhurtas): Cached for 24 hours
//! - **Predicted hit rate**: 95%+ for typical usage patterns (~30 API calls/day)
//!
//! # Environment Variables
//!
//! Required:
//! - `FREE_ASTROLOGY_API_KEY` - Your API key from freeastrologyapi.com
//!
//! Optional:
//! - `FREE_ASTROLOGY_API_BASE_URL` - Default: `https://json.freeastrologyapi.com`
//! - `FREE_ASTROLOGY_API_TIMEOUT` - Default: 30 seconds
//! - `FREE_ASTROLOGY_API_RETRY_COUNT` - Default: 3
//! - `VEDIC_ENGINE_FALLBACK_ENABLED` - Default: true

// Configuration
pub mod config;

/// Default API base URL for FreeAstrologyAPI.com
pub const API_BASE_URL: &str = "https://json.freeastrologyapi.com";

// Error handling
pub mod error;

// HTTP client
pub mod client;

// Logging helpers
pub mod logging;

// Cache layer
pub mod cache;

// Rate limiting
pub mod rate_limiter;

// Cached client (main interface)
pub mod cached_client;

// Unified service
pub mod service;

// Resilience layer: fallback chain + exponential backoff (FAPI-098, FAPI-105)
pub mod resilience;

// Batch request optimization (FAPI-106)
pub mod batch;

// Metrics and monitoring (FAPI-099)
pub mod metrics;

// API versioning (FAPI-107)
pub mod versioning;

// Dasha types
pub mod dasha;

// Vimshottari helpers
pub mod vimshottari;

// Chart types
pub mod chart;

// Birth chart enrichments
pub mod birth_chart;

// Vargas (divisional charts)
pub mod vargas;

// Progressions
pub mod progressions;

// Panchang modules
pub mod panchang;

// Retry logic with exponential backoff (FAPI-006)
pub mod retry;

// Circuit breaker pattern (FAPI-008)
pub mod circuit_breaker;

// Transits module (FAPI-073 to FAPI-078)
pub mod transits;

// Yogas module (FAPI-063 to FAPI-066)
pub mod yogas;

// Shadbala module (FAPI-067 to FAPI-069)
pub mod shadbala;

// Ashtakavarga module (FAPI-070 to FAPI-072)
pub mod ashtakavarga;

// Muhurta module (FAPI-081 to FAPI-086)
pub mod muhurta;

// Report generator (FAPI-111)
pub mod report_generator;

// Daily panchang service (FAPI-112)
pub mod daily_panchang;

// Hora alarms (FAPI-113)
pub mod hora_alarms;

// Dasha alerts (FAPI-114)
pub mod dasha_alerts;

// Festivals (FAPI-115)
pub mod festivals;

// Eclipses (FAPI-116)
pub mod eclipses;

// Fasting / Vrat (FAPI-117)
pub mod fasting;

// Naming / Namkaran (FAPI-118)
pub mod naming;

// Remedies (FAPI-119, FAPI-120)
pub mod remedies;

// Common types
pub mod types;

// Mock data factories for testing
#[cfg(any(test, feature = "mocks"))]
pub mod mocks;

// Re-export main types
pub use config::{Config, ProviderType};
pub use error::{VedicApiError, VedicApiResult, Result};
pub use client::VedicApiClient;
pub use cache::ApiCache;
pub use rate_limiter::{RateLimiter, RateLimitStatus};
pub use cached_client::CachedVedicClient;
pub use service::VedicApiService;

// Re-export Dasha types
pub use dasha::{
    VimshottariDasha,
    DashaPeriod,
    DashaLevel,
    DashaPlanet,
    DashaTree,
};

// Re-export Chart types
pub use chart::{
    BirthChart,
    NavamsaChart,
    PlanetPosition,
    HousePosition,
    ZodiacSign,
};

// Re-export Panchang types
pub use panchang::{
    // Core data
    Panchang,
    CompletePanchang,
    PanchangMetadata,
    PanchangQuery,
    DateInfo,
    Location,
    DayBoundaries,
    PlanetaryPositions,
    HinduDate,
    HinduMonth,
    
    // Tithi
    Tithi,
    TithiName,
    
    // Nakshatra
    Nakshatra,
    NakshatraName,
    
    // Yoga
    Yoga,
    YogaName,
    
    // Karana
    Karana,
    KaranaName,
    KaranaType,
    
    // Vara
    Vara,
    
    // Paksha
    Paksha,
    
    // Muhurtas
    MuhurtaCollection,
    Muhurta,
    MuhurtaNature,
    AbhijitMuhurta,
    RahuKalam,
    YamaGandam,
    GulikaKaal,
    BrahmaMuhurta,
    AmritKaal,
    
    // Hora
    HoraTimings,
    Hora,
    Planet as HoraPlanet,
    ActivityType as HoraActivity,
    
    // Choghadiya
    ChoghadiyaTimings,
    Choghadiya,
    ChoghadiyaName,
    ChoghadiyaNature,
    ActivityCategory as ChoghadiyaActivity,
    Recommendation as ChoghadiyaRecommendation,
};

/// Version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a new cached client from environment variables
///
/// # Example
/// ```no_run
/// use noesis_vedic_api::create_client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = create_client()?;
///     Ok(())
/// }
/// ```
pub fn create_client() -> VedicApiResult<CachedVedicClient> {
    CachedVedicClient::from_env()
}

/// Convenience function to quickly check if the API is accessible
///
/// Returns true if a simple health check succeeds
pub async fn is_api_available() -> bool {
    match CachedVedicClient::from_env() {
        Ok(client) => client.health_check().await,
        Err(_) => false,
    }
}

/// Get a formatted status report of the API client
pub async fn get_status_report() -> String {
    match CachedVedicClient::from_env() {
        Ok(client) => {
            let status = client.status_report().await;
            format!(
                "Vedic API Client Status:\n\
                 - API Calls Remaining Today: {}\n\
                 - Effective Remaining (after buffer): {}\n\
                 - Used Today: {}",
                status.rate_limit.remaining_today,
                status.rate_limit.effective_remaining,
                status.rate_limit.used_today
            )
        }
        Err(e) => format!("Failed to initialize client: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_error_display() {
        let err = VedicApiError::Configuration {
            field: "API_KEY".to_string(),
            message: "test".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("test"));
    }
}
