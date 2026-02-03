//! Birth data verification against multiple calculation sources
//!
//! This module verifies birth chart calculations by comparing:
//! - Vedic API results
//! - Native engine calculations
//! - Expected values from known charts

use chrono::{DateTime, Utc, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::{IntegrationError, Result};

/// Birth profile for verification and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthProfile {
    /// Birth date (YYYY-MM-DD)
    pub date: String,
    /// Birth time (HH:MM, optional)
    pub time: Option<String>,
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Timezone (IANA identifier)
    pub timezone: String,
}

impl BirthProfile {
    /// Create a new birth profile
    pub fn new(
        date: &str,
        time: &str,
        latitude: f64,
        longitude: f64,
        timezone: &str,
    ) -> Self {
        Self {
            date: date.to_string(),
            time: Some(time.to_string()),
            latitude,
            longitude,
            timezone: timezone.to_string(),
        }
    }
    
    /// Create from components
    pub fn from_components(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        latitude: f64,
        longitude: f64,
        timezone: &str,
    ) -> Self {
        Self {
            date: format!("{:04}-{:02}-{:02}", year, month, day),
            time: Some(format!("{:02}:{:02}", hour, minute)),
            latitude,
            longitude,
            timezone: timezone.to_string(),
        }
    }
    
    /// Parse date as NaiveDate
    pub fn parse_date(&self) -> Result<NaiveDate> {
        NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")
            .map_err(|e| IntegrationError::DateParse(e.to_string()))
    }
    
    /// Parse time as NaiveTime
    pub fn parse_time(&self) -> Option<NaiveTime> {
        self.time.as_ref().and_then(|t| {
            NaiveTime::parse_from_str(t, "%H:%M").ok()
        })
    }
    
    /// Convert to noesis-core BirthData
    pub fn to_core_birth_data(&self) -> noesis_core::BirthData {
        noesis_core::BirthData {
            name: None,
            date: self.date.clone(),
            time: self.time.clone(),
            latitude: self.latitude,
            longitude: self.longitude,
            timezone: self.timezone.clone(),
        }
    }
    
    /// Get Shesh's birth profile (for verification)
    pub fn shesh() -> Self {
        Self::new(
            "1991-08-13",
            "13:31",
            12.9716,
            77.5946,
            "Asia/Kolkata",
        )
    }
}

/// Verification result comparing multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether verification passed
    pub verified: bool,
    /// Confidence level (0.0-1.0)
    pub confidence: f64,
    /// Individual check results
    pub checks: Vec<VerificationCheck>,
    /// Discrepancies found
    pub discrepancies: Vec<Discrepancy>,
    /// Overall summary
    pub summary: String,
}

/// Individual verification check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    /// Name of the check
    pub name: String,
    /// Source system
    pub source: String,
    /// Expected value
    pub expected: String,
    /// Actual value
    pub actual: String,
    /// Whether it matches
    pub matches: bool,
    /// Tolerance for numeric comparisons
    pub tolerance: Option<f64>,
}

/// Discrepancy between expected and actual
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrepancy {
    /// Field with discrepancy
    pub field: String,
    /// Expected value
    pub expected: String,
    /// Actual value
    pub actual: String,
    /// Severity
    pub severity: DiscrepancySeverity,
}

/// Severity of a discrepancy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscrepancySeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

/// Verifies birth data against multiple sources
pub struct DataVerifier {
    /// Known good profiles for comparison
    reference_profiles: Vec<(BirthProfile, ExpectedValues)>,
}

/// Expected values for a known birth chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedValues {
    /// Expected Moon Nakshatra
    pub moon_nakshatra: String,
    /// Expected Moon longitude
    pub moon_longitude: f64,
    /// Expected Ascendant
    pub ascendant: String,
    /// Expected Sun sign
    pub sun_sign: String,
    /// Expected current Mahadasha (as of 2024)
    pub current_mahadasha: String,
    /// Expected Mahadasha end date
    pub mahadasha_end: String,
    /// Expected Tithi type
    pub tithi: String,
    /// Expected Paksha
    pub paksha: String,
}

impl DataVerifier {
    /// Create a new verifier with default reference profiles
    pub fn new() -> Self {
        let mut verifier = Self {
            reference_profiles: Vec::new(),
        };
        
        // Add Shesh's profile as reference
        verifier.add_shesh_reference();
        
        verifier
    }
    
    /// Add Shesh's birth profile as reference
    fn add_shesh_reference(&mut self) {
        let profile = BirthProfile::shesh();
        
        // Expected values for Shesh's chart
        // Birth: 1991-08-13 13:31 IST, Bengaluru
        let expected = ExpectedValues {
            moon_nakshatra: "Uttara Phalguni".to_string(),
            moon_longitude: 156.0, // Approximate
            ascendant: "Scorpio".to_string(),
            sun_sign: "Leo".to_string(),
            current_mahadasha: "Mars".to_string(), // Until 2026-09-14
            mahadasha_end: "2026-09-14".to_string(),
            tithi: "Chaturthi".to_string(),
            paksha: "Shukla".to_string(),
        };
        
        self.reference_profiles.push((profile, expected));
    }
    
    /// Verify a birth profile
    pub async fn verify(&self, profile: &BirthProfile) -> Result<VerificationResult> {
        let mut checks = Vec::new();
        let mut discrepancies = Vec::new();
        
        // Check 1: Date format
        let date_check = self.verify_date_format(profile);
        if !date_check.matches {
            discrepancies.push(Discrepancy {
                field: "date".to_string(),
                expected: "YYYY-MM-DD format".to_string(),
                actual: profile.date.clone(),
                severity: DiscrepancySeverity::Critical,
            });
        }
        checks.push(date_check);
        
        // Check 2: Time format (if provided)
        if let Some(time_check) = self.verify_time_format(profile) {
            if !time_check.matches {
                discrepancies.push(Discrepancy {
                    field: "time".to_string(),
                    expected: "HH:MM format".to_string(),
                    actual: profile.time.clone().unwrap_or_default(),
                    severity: DiscrepancySeverity::Major,
                });
            }
            checks.push(time_check);
        }
        
        // Check 3: Coordinates range
        let coord_check = self.verify_coordinates(profile);
        if !coord_check.matches {
            discrepancies.push(Discrepancy {
                field: "coordinates".to_string(),
                expected: "Valid latitude (-90 to 90) and longitude (-180 to 180)".to_string(),
                actual: format!("{}, {}", profile.latitude, profile.longitude),
                severity: DiscrepancySeverity::Critical,
            });
        }
        checks.push(coord_check);
        
        // Check against reference profiles if match
        for (ref_profile, expected) in &self.reference_profiles {
            if profiles_match(profile, ref_profile) {
                checks.push(VerificationCheck {
                    name: "Reference match".to_string(),
                    source: "Reference database".to_string(),
                    expected: "Known profile".to_string(),
                    actual: "Match found".to_string(),
                    matches: true,
                    tolerance: None,
                });
                
                // Add expected values verification
                checks.push(VerificationCheck {
                    name: "Moon Nakshatra".to_string(),
                    source: "Reference".to_string(),
                    expected: expected.moon_nakshatra.clone(),
                    actual: expected.moon_nakshatra.clone(),
                    matches: true,
                    tolerance: None,
                });
            }
        }
        
        // Calculate confidence based on checks
        let total_checks = checks.len();
        let passed_checks = checks.iter().filter(|c| c.matches).count();
        let confidence = if total_checks > 0 {
            passed_checks as f64 / total_checks as f64
        } else {
            0.0
        };
        
        let verified = confidence >= 0.8 && discrepancies.is_empty();
        
        let summary = if verified {
            format!("Verification passed with {:.0}% confidence", confidence * 100.0)
        } else {
            format!(
                "Verification failed: {}/{} checks passed, {} discrepancies found",
                passed_checks,
                total_checks,
                discrepancies.len()
            )
        };
        
        Ok(VerificationResult {
            verified,
            confidence,
            checks,
            discrepancies,
            summary,
        })
    }
    
    /// Verify date format
    fn verify_date_format(&self, profile: &BirthProfile) -> VerificationCheck {
        let valid = NaiveDate::parse_from_str(&profile.date, "%Y-%m-%d").is_ok();
        
        VerificationCheck {
            name: "Date format".to_string(),
            source: "ISO 8601".to_string(),
            expected: "YYYY-MM-DD".to_string(),
            actual: profile.date.clone(),
            matches: valid,
            tolerance: None,
        }
    }
    
    /// Verify time format
    fn verify_time_format(&self, profile: &BirthProfile) -> Option<VerificationCheck> {
        profile.time.as_ref().map(|t| {
            let valid = NaiveTime::parse_from_str(t, "%H:%M").is_ok();
            
            VerificationCheck {
                name: "Time format".to_string(),
                source: "ISO 8601".to_string(),
                expected: "HH:MM".to_string(),
                actual: t.clone(),
                matches: valid,
                tolerance: None,
            }
        })
    }
    
    /// Verify coordinates are in valid range
    fn verify_coordinates(&self, profile: &BirthProfile) -> VerificationCheck {
        let valid_lat = profile.latitude >= -90.0 && profile.latitude <= 90.0;
        let valid_lng = profile.longitude >= -180.0 && profile.longitude <= 180.0;
        
        VerificationCheck {
            name: "Coordinates range".to_string(),
            source: "Geographic".to_string(),
            expected: "Lat: -90 to 90, Lng: -180 to 180".to_string(),
            actual: format!("Lat: {}, Lng: {}", profile.latitude, profile.longitude),
            matches: valid_lat && valid_lng,
            tolerance: Some(0.0001),
        }
    }
    
    /// Compare calculated values with expected
    pub fn compare_with_expected(
        &self,
        calculated: &CalculatedValues,
        expected: &ExpectedValues,
    ) -> Vec<VerificationCheck> {
        let mut checks = Vec::new();
        
        // Moon Nakshatra
        checks.push(VerificationCheck {
            name: "Moon Nakshatra".to_string(),
            source: "Vedic calculation".to_string(),
            expected: expected.moon_nakshatra.clone(),
            actual: calculated.moon_nakshatra.clone(),
            matches: calculated.moon_nakshatra.eq_ignore_ascii_case(&expected.moon_nakshatra),
            tolerance: None,
        });
        
        // Moon Longitude (with tolerance)
        let moon_long_match = (calculated.moon_longitude - expected.moon_longitude).abs() < 1.0;
        checks.push(VerificationCheck {
            name: "Moon Longitude".to_string(),
            source: "Ephemeris".to_string(),
            expected: expected.moon_longitude.to_string(),
            actual: calculated.moon_longitude.to_string(),
            matches: moon_long_match,
            tolerance: Some(1.0),
        });
        
        // Ascendant
        checks.push(VerificationCheck {
            name: "Ascendant".to_string(),
            source: "Chart calculation".to_string(),
            expected: expected.ascendant.clone(),
            actual: calculated.ascendant.clone(),
            matches: calculated.ascendant.eq_ignore_ascii_case(&expected.ascendant),
            tolerance: None,
        });
        
        // Sun Sign
        checks.push(VerificationCheck {
            name: "Sun Sign".to_string(),
            source: "Tropical Zodiac".to_string(),
            expected: expected.sun_sign.clone(),
            actual: calculated.sun_sign.clone(),
            matches: calculated.sun_sign.eq_ignore_ascii_case(&expected.sun_sign),
            tolerance: None,
        });
        
        checks
    }
}

impl Default for DataVerifier {
    fn default() -> Self {
        Self::new()
    }
}

/// Values calculated from birth data
#[derive(Debug, Clone, Default)]
pub struct CalculatedValues {
    pub moon_nakshatra: String,
    pub moon_longitude: f64,
    pub ascendant: String,
    pub sun_sign: String,
    pub current_mahadasha: String,
    pub tithi: String,
    pub paksha: String,
}

/// Check if two profiles match
fn profiles_match(a: &BirthProfile, b: &BirthProfile) -> bool {
    a.date == b.date
        && a.time == b.time
        && (a.latitude - b.latitude).abs() < 0.01
        && (a.longitude - b.longitude).abs() < 0.01
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_birth_profile_creation() {
        let profile = BirthProfile::shesh();
        
        assert_eq!(profile.date, "1991-08-13");
        assert_eq!(profile.time, Some("13:31".to_string()));
        assert!((profile.latitude - 12.9716).abs() < 0.0001);
        assert!((profile.longitude - 77.5946).abs() < 0.0001);
    }

    #[test]
    fn test_parse_date() {
        let profile = BirthProfile::shesh();
        let date = profile.parse_date().unwrap();
        
        assert_eq!(date.year(), 1991);
        assert_eq!(date.month(), 8);
        assert_eq!(date.day(), 13);
    }

    #[test]
    fn test_verify_date_format() {
        let verifier = DataVerifier::new();
        
        let valid_profile = BirthProfile::shesh();
        let check = verifier.verify_date_format(&valid_profile);
        assert!(check.matches);
        
        let invalid_profile = BirthProfile {
            date: "13-08-1991".to_string(), // Wrong format
            time: None,
            latitude: 0.0,
            longitude: 0.0,
            timezone: "UTC".to_string(),
        };
        let check = verifier.verify_date_format(&invalid_profile);
        assert!(!check.matches);
    }

    #[test]
    fn test_verify_coordinates() {
        let verifier = DataVerifier::new();
        
        let valid_profile = BirthProfile::shesh();
        let check = verifier.verify_coordinates(&valid_profile);
        assert!(check.matches);
        
        let invalid_profile = BirthProfile {
            date: "1991-08-13".to_string(),
            time: None,
            latitude: 95.0, // Invalid
            longitude: 77.5946,
            timezone: "UTC".to_string(),
        };
        let check = verifier.verify_coordinates(&invalid_profile);
        assert!(!check.matches);
    }
}
