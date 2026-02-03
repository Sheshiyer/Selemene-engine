//! Common types for Vedic API integration

use serde::{Deserialize, Serialize};

/// Geographic coordinates
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinates {
    pub fn new(lat: f64, lng: f64) -> Self {
        Self {
            latitude: lat,
            longitude: lng,
        }
    }
    
    pub fn validate(&self) -> Result<(), String> {
        if self.latitude < -90.0 || self.latitude > 90.0 {
            return Err(format!("Invalid latitude: {}", self.latitude));
        }
        if self.longitude < -180.0 || self.longitude > 180.0 {
            return Err(format!("Invalid longitude: {}", self.longitude));
        }
        Ok(())
    }
}

/// Birth data for Vedic calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthData {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub coordinates: Coordinates,
    pub timezone_offset: f64,
}

impl BirthData {
    pub fn new(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        lat: f64,
        lng: f64,
        tz: f64,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            coordinates: Coordinates::new(lat, lng),
            timezone_offset: tz,
        }
    }
    
    /// Convert to chrono::NaiveDateTime
    pub fn to_datetime(&self) -> Option<chrono::NaiveDateTime> {
        chrono::NaiveDate::from_ymd_opt(self.year, self.month, self.day)?
            .and_hms_opt(self.hour, self.minute, self.second)
    }
}

/// API request configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub observation_point: String, // "topocentric" or "geocentric"
    pub ayanamsha: String,         // "lahiri", "raman", "krishnamurti"
    pub house_system: String,      // "placidus", "whole_sign", "equal"
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            observation_point: "topocentric".to_string(),
            ayanamsha: "lahiri".to_string(),
            house_system: "placidus".to_string(),
        }
    }
}

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Planet enum for Vedic astrology
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Planet {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
    Rahu,
    Ketu,
}

impl Planet {
    pub fn as_str(&self) -> &'static str {
        match self {
            Planet::Sun => "Sun",
            Planet::Moon => "Moon",
            Planet::Mars => "Mars",
            Planet::Mercury => "Mercury",
            Planet::Jupiter => "Jupiter",
            Planet::Venus => "Venus",
            Planet::Saturn => "Saturn",
            Planet::Rahu => "Rahu",
            Planet::Ketu => "Ketu",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sun" | "surya" => Some(Planet::Sun),
            "moon" | "chandra" => Some(Planet::Moon),
            "mars" | "mangal" => Some(Planet::Mars),
            "mercury" | "buddha" => Some(Planet::Mercury),
            "jupiter" | "guru" | "brahaspati" => Some(Planet::Jupiter),
            "venus" | "shukra" => Some(Planet::Venus),
            "saturn" | "shani" => Some(Planet::Saturn),
            "rahu" => Some(Planet::Rahu),
            "ketu" => Some(Planet::Ketu),
            _ => None,
        }
    }
}

/// Zodiac sign enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ZodiacSign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl ZodiacSign {
    pub fn as_str(&self) -> &'static str {
        match self {
            ZodiacSign::Aries => "Aries",
            ZodiacSign::Taurus => "Taurus",
            ZodiacSign::Gemini => "Gemini",
            ZodiacSign::Cancer => "Cancer",
            ZodiacSign::Leo => "Leo",
            ZodiacSign::Virgo => "Virgo",
            ZodiacSign::Libra => "Libra",
            ZodiacSign::Scorpio => "Scorpio",
            ZodiacSign::Sagittarius => "Sagittarius",
            ZodiacSign::Capricorn => "Capricorn",
            ZodiacSign::Aquarius => "Aquarius",
            ZodiacSign::Pisces => "Pisces",
        }
    }
    
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(ZodiacSign::Aries),
            1 => Some(ZodiacSign::Taurus),
            2 => Some(ZodiacSign::Gemini),
            3 => Some(ZodiacSign::Cancer),
            4 => Some(ZodiacSign::Leo),
            5 => Some(ZodiacSign::Virgo),
            6 => Some(ZodiacSign::Libra),
            7 => Some(ZodiacSign::Scorpio),
            8 => Some(ZodiacSign::Sagittarius),
            9 => Some(ZodiacSign::Capricorn),
            10 => Some(ZodiacSign::Aquarius),
            11 => Some(ZodiacSign::Pisces),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinates_validation() {
        let valid = Coordinates::new(12.9716, 77.5946);
        assert!(valid.validate().is_ok());
        
        let invalid_lat = Coordinates::new(95.0, 77.5946);
        assert!(invalid_lat.validate().is_err());
        
        let invalid_lng = Coordinates::new(12.9716, 200.0);
        assert!(invalid_lng.validate().is_err());
    }

    #[test]
    fn test_planet_from_str() {
        assert_eq!(Planet::from_str("Sun"), Some(Planet::Sun));
        assert_eq!(Planet::from_str("moon"), Some(Planet::Moon));
        assert_eq!(Planet::from_str("surya"), Some(Planet::Sun));
        assert_eq!(Planet::from_str("invalid"), None);
    }

    #[test]
    fn test_zodiac_from_index() {
        assert_eq!(ZodiacSign::from_index(0), Some(ZodiacSign::Aries));
        assert_eq!(ZodiacSign::from_index(11), Some(ZodiacSign::Pisces));
        assert_eq!(ZodiacSign::from_index(12), None);
    }
}
