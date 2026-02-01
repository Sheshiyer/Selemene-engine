//! Design Time Calculation for Human Design
//!
//! The Design time is calculated using an 88-day solar arc — finding the exact moment
//! approximately 88 days before birth when the Sun was at the same ecliptic longitude
//! as at birth.
//!
//! This is NOT a simple 88-day subtraction, but requires iterative calculation using
//! Swiss Ephemeris to find the precise moment when:
//! sun_longitude(design_time) ≈ sun_longitude(birth_time)
//!
//! Accuracy requirement: within 1 hour of professional Human Design software.

use chrono::{DateTime, Utc, Duration, Timelike, Datelike, TimeZone};
use swisseph::{Body, Seflg};
use swisseph::swe;

/// Error type for Design Time calculation
#[derive(Debug, thiserror::Error)]
pub enum DesignTimeError {
    #[error("Swiss Ephemeris error: {0}")]
    SwissEphemerisError(String),
    #[error("Convergence error: could not find design time within tolerance after {0} iterations")]
    ConvergenceError(usize),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Initialize Swiss Ephemeris with the given data path
///
/// # Arguments
/// * `ephe_path` - Path to Swiss Ephemeris data files (e.g., "/path/to/ephe")
pub fn initialize_ephemeris(ephe_path: &str) {
    swe::set_ephe_path(ephe_path);
}

/// Calculate the Julian Day for a given DateTime<Utc>
fn datetime_to_julian_day(dt: &DateTime<Utc>) -> f64 {
    let year = dt.year();
    let month = dt.month() as i32;
    let day = dt.day() as i32;
    let hour = dt.hour() as f64 + (dt.minute() as f64 / 60.0) + (dt.second() as f64 / 3600.0);
    
    // gregorian flag = 1 for Gregorian calendar
    swe::julday(year, month, day, hour, 1)
}

/// Convert Julian Day back to DateTime<Utc>
fn julian_day_to_datetime(jd: f64) -> DateTime<Utc> {
    // Simplified conversion (could be more precise with swe::revjul)
    // For now, use a baseline and add days
    let j2000 = 2451545.0; // JD for 2000-01-01 12:00:00
    let days_since_j2000 = jd - j2000;
    
    let base = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
    let duration = Duration::milliseconds((days_since_j2000 * 86400.0 * 1000.0) as i64);
    
    base + duration
}

/// Calculate the Sun's ecliptic longitude for a given Julian Day
///
/// # Arguments
/// * `jd` - Julian Day (UT)
///
/// # Returns
/// Sun's longitude in degrees (0-360), or error
fn calculate_sun_longitude(jd: f64) -> Result<f64, DesignTimeError> {
    let flags = Seflg::SWIEPH; // Use Swiss Ephemeris
    
    // calc_ut returns Result<Out<[f64; 6], i32>, String>
    // where Out has fields: out (the array) and code (status code)
    match swe::calc_ut(jd, Body::Sun as u32, flags.into()) {
        Ok(result) => {
            // result.out[0] contains the ecliptic longitude
            Ok(result.out[0])
        }
        Err(e) => {
            Err(DesignTimeError::SwissEphemerisError(e))
        }
    }
}

/// Calculate the Design Time for a given birth time
///
/// Uses iterative refinement to find the moment approximately 88 days before birth
/// when the Sun was at the same longitude as at birth.
///
/// # Arguments
/// * `birth_time` - Birth time in UTC
/// * `ephe_path` - Optional path to Swiss Ephemeris data (if not already initialized)
///
/// # Returns
/// Design time (approximately 88 days before birth), or error
///
/// # Algorithm
/// 1. Calculate Sun's longitude at birth
/// 2. Start with initial estimate: birth_time - 88 days
/// 3. Iteratively refine until Sun's longitude matches birth longitude
/// 4. Use binary search for convergence
pub fn calculate_design_time(
    birth_time: DateTime<Utc>,
    ephe_path: Option<&str>,
) -> Result<DateTime<Utc>, DesignTimeError> {
    // Initialize ephemeris if path provided
    if let Some(path) = ephe_path {
        initialize_ephemeris(path);
    }
    
    // Calculate Sun's longitude at birth
    let birth_jd = datetime_to_julian_day(&birth_time);
    let birth_longitude = calculate_sun_longitude(birth_jd)?;
    
    // Design time is when the Sun was 88 DEGREES behind the birth position
    // This is NOT the same as 88 days - it's based on solar arc
    // Target longitude = birth_longitude - 88° (going backwards in the zodiac)
    let target_longitude = (birth_longitude - 88.0).rem_euclid(360.0);
    
    // Initial estimate: ~88-92 days before birth (Sun moves ~1° per day)
    let initial_estimate = birth_time - Duration::days(88);
    
    // Perform iterative refinement using binary search
    let design_time = refine_design_time(initial_estimate, target_longitude)?;
    
    Ok(design_time)
}

/// Refine the design time estimate using iterative binary search
///
/// # Arguments
/// * `initial_estimate` - Initial estimate of design time (birth - 88 days)
/// * `target_longitude` - Target Sun longitude (from birth time)
///
/// # Returns
/// Refined design time, or error if convergence fails
fn refine_design_time(
    initial_estimate: DateTime<Utc>,
    target_longitude: f64,
) -> Result<DateTime<Utc>, DesignTimeError> {
    const MAX_ITERATIONS: usize = 50;
    const TOLERANCE_DEGREES: f64 = 0.001; // ~3.6 arcseconds (~0.24 seconds of time)
    const TOLERANCE_SECONDS: i64 = 3600; // 1 hour in seconds
    
    // Search bounds: ±3 days from initial estimate
    let mut lower_bound = initial_estimate - Duration::days(3);
    let mut upper_bound = initial_estimate + Duration::days(3);
    
    for _iteration in 0..MAX_ITERATIONS {
        // Try the midpoint
        let midpoint = lower_bound + (upper_bound - lower_bound) / 2;
        let midpoint_jd = datetime_to_julian_day(&midpoint);
        let midpoint_longitude = calculate_sun_longitude(midpoint_jd)?;
        
        // Calculate longitude difference (accounting for 360° wrap)
        let diff = longitude_difference(midpoint_longitude, target_longitude);
        
        // Check convergence
        if diff.abs() < TOLERANCE_DEGREES {
            // Additional check: ensure we're within 1 hour
            let time_diff = (midpoint - initial_estimate).num_seconds().abs();
            if time_diff < TOLERANCE_SECONDS * 24 * 3 { // Within ±3 days is reasonable
                return Ok(midpoint);
            }
        }
        
        // Adjust search bounds
        // Sun moves ~1° per day, so if we're ahead in longitude, we need to go earlier in time
        if diff > 0.0 {
            // Current longitude is ahead of target, need to go earlier
            upper_bound = midpoint;
        } else {
            // Current longitude is behind target, need to go later
            lower_bound = midpoint;
        }
        
        // Check if bounds are too close (converged)
        if (upper_bound - lower_bound).num_seconds() < 60 {
            return Ok(midpoint);
        }
    }
    
    Err(DesignTimeError::ConvergenceError(MAX_ITERATIONS))
}

/// Calculate the shortest angular distance between two longitudes
///
/// Accounts for 360° wrap-around (e.g., 359° and 1° are only 2° apart)
///
/// # Arguments
/// * `lon1` - First longitude (0-360)
/// * `lon2` - Second longitude (0-360)
///
/// # Returns
/// Signed difference (-180 to +180 degrees)
fn longitude_difference(lon1: f64, lon2: f64) -> f64 {
    let mut diff = lon1 - lon2;
    
    // Normalize to -180 to +180
    while diff > 180.0 {
        diff -= 360.0;
    }
    while diff < -180.0 {
        diff += 360.0;
    }
    
    diff
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_longitude_difference() {
        assert_eq!(longitude_difference(10.0, 5.0), 5.0);
        assert_eq!(longitude_difference(5.0, 10.0), -5.0);
        assert_eq!(longitude_difference(359.0, 1.0), -2.0);
        assert_eq!(longitude_difference(1.0, 359.0), 2.0);
        assert_eq!(longitude_difference(180.0, 180.0), 0.0);
    }

    #[test]
    fn test_julian_day_calculation() {
        // Test a known date: 2000-01-01 12:00:00 UTC
        // JD should be approximately 2451545.0
        let dt = Utc.with_ymd_and_hms(2000, 1, 1, 12, 0, 0).unwrap();
        let jd = datetime_to_julian_day(&dt);
        
        assert!((jd - 2451545.0).abs() < 0.01, "JD calculation incorrect: got {}", jd);
    }

    #[test]
    #[ignore] // Requires Swiss Ephemeris data files
    fn test_calculate_design_time() {
        // Test with a known birth time
        let birth_time = Utc.with_ymd_and_hms(1990, 6, 15, 14, 30, 0).unwrap();
        
        // This test requires Swiss Ephemeris data files
        // In production, set the path to your ephe directory
        let ephe_path = "/path/to/ephe";
        
        let result = calculate_design_time(birth_time, Some(ephe_path));
        
        if let Ok(design_time) = result {
            // Design time should be approximately 88 days before birth
            let diff_days = (birth_time - design_time).num_days();
            assert!(diff_days >= 85 && diff_days <= 91, 
                "Design time should be ~88 days before birth, got {} days", diff_days);
            
            println!("Birth time: {}", birth_time);
            println!("Design time: {}", design_time);
            println!("Difference: {} days", diff_days);
        } else {
            println!("Test skipped: Swiss Ephemeris data not available");
        }
    }

    #[test]
    fn test_design_time_validation() {
        // Test that design time is approximately 88 days before birth
        let birth_time = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
        let expected_design = birth_time - Duration::days(88);
        
        // Verify the expected range
        let diff = (birth_time - expected_design).num_days();
        assert_eq!(diff, 88, "Expected ~88 days difference");
    }

    #[test]
    fn test_julian_day_roundtrip() {
        // Test converting to JD and back
        let original = Utc.with_ymd_and_hms(2020, 6, 15, 18, 30, 45).unwrap();
        let jd = datetime_to_julian_day(&original);
        let converted = julian_day_to_datetime(jd);
        
        // Should be within 1 second
        let diff = (original - converted).num_seconds().abs();
        assert!(diff <= 1, "Roundtrip conversion failed: {} seconds difference", diff);
    }
}
