//! FAPI-110: Test fixtures and helpers for noesis-vedic-api integration tests
//!
//! This module provides:
//! - Shared test constants (Bangalore location, standard dates)
//! - Wiremock server setup helpers
//! - Assertion helpers for common validation patterns
//! - Pre-built fixture data for parallel-safe test execution

use noesis_vedic_api::{
    Config, CachedVedicClient, VedicApiService,
    mocks,
};

// ===========================================================================
// Standard test constants
// ===========================================================================

/// Standard test location: Bangalore, India
pub const BANGALORE_LAT: f64 = 12.9716;
pub const BANGALORE_LNG: f64 = 77.5946;
pub const IST_TIMEZONE: f64 = 5.5;

/// Standard test location: New Delhi, India
pub const DELHI_LAT: f64 = 28.6139;
pub const DELHI_LNG: f64 = 77.2090;

/// Standard test location: London, UK
pub const LONDON_LAT: f64 = 51.5074;
pub const LONDON_LNG: f64 = -0.1278;
pub const GMT_TIMEZONE: f64 = 0.0;

/// Standard test date: 2024-01-15 (Monday, Shukla Panchami)
pub const TEST_YEAR: i32 = 2024;
pub const TEST_MONTH: u32 = 1;
pub const TEST_DAY: u32 = 15;
pub const TEST_HOUR: u32 = 12;
pub const TEST_MINUTE: u32 = 0;
pub const TEST_SECOND: u32 = 0;

/// Standard birth data: 1991-08-13 13:31 Bangalore
pub const BIRTH_YEAR: i32 = 1991;
pub const BIRTH_MONTH: u32 = 8;
pub const BIRTH_DAY: u32 = 13;
pub const BIRTH_HOUR: u32 = 13;
pub const BIRTH_MINUTE: u32 = 31;
pub const BIRTH_SECOND: u32 = 0;

// ===========================================================================
// Client factory helpers
// ===========================================================================

/// Create a test CachedVedicClient pointing at the given base URL
pub fn create_test_client(base_url: &str) -> CachedVedicClient {
    let config = mocks::mock_config(base_url);
    CachedVedicClient::new(config)
}

/// Create a test VedicApiService pointing at the given base URL
pub fn create_test_service(base_url: &str) -> VedicApiService {
    VedicApiService::new(create_test_client(base_url))
}

/// Create a test client with fallback enabled
pub fn create_test_client_with_fallback(base_url: &str) -> CachedVedicClient {
    let config = mocks::mock_config_with_fallback(base_url);
    CachedVedicClient::new(config)
}

// ===========================================================================
// Assertion helpers
// ===========================================================================

/// Assert that a Panchang result has valid basic structure
pub fn assert_panchang_valid(panchang: &noesis_vedic_api::Panchang) {
    assert!(panchang.date.year > 1900, "Year should be reasonable");
    assert!((1..=12).contains(&panchang.date.month), "Month should be 1-12");
    assert!((1..=31).contains(&panchang.date.day), "Day should be 1-31");
    assert!(!panchang.tithi.name().is_empty(), "Tithi name should not be empty");
    assert!(!panchang.nakshatra.name().is_empty(), "Nakshatra name should not be empty");
    assert!(!panchang.day_boundaries.sunrise.is_empty(), "Sunrise should be set");
    assert!(!panchang.day_boundaries.sunset.is_empty(), "Sunset should be set");
}

/// Assert that a VimshottariDasha result has valid basic structure
pub fn assert_dasha_valid(dasha: &noesis_vedic_api::VimshottariDasha) {
    assert!(!dasha.birth_date.is_empty(), "Birth date should be set");
    assert!(!dasha.moon_nakshatra.is_empty(), "Moon nakshatra should be set");
    assert!(!dasha.mahadashas.is_empty(), "Should have at least one mahadasha");
    assert!(
        dasha.balance.total_period_years > 0.0,
        "Balance period should be positive"
    );
}

/// Assert that a BirthChart result has valid basic structure
pub fn assert_birth_chart_valid(chart: &noesis_vedic_api::BirthChart) {
    assert!(!chart.planets.is_empty(), "Should have planet positions");
    assert_eq!(chart.houses.len(), 12, "Should have exactly 12 houses");
    assert!(
        chart.get_planet("Sun").is_some(),
        "Should contain Sun position"
    );
    assert!(
        chart.get_planet("Moon").is_some(),
        "Should contain Moon position"
    );
}

/// Assert cache stats match expected hit/miss counts
pub fn assert_cache_stats(
    stats: &noesis_vedic_api::cache::CacheStats,
    expected_hits: u64,
    expected_misses: u64,
) {
    assert_eq!(
        stats.hits, expected_hits,
        "Expected {} hits, got {}",
        expected_hits, stats.hits
    );
    assert_eq!(
        stats.misses, expected_misses,
        "Expected {} misses, got {}",
        expected_misses, stats.misses
    );
}
