//! Birth Chart API endpoint implementations
//!
//! FAPI-046: Implement GET /horoscope-chart endpoint

use chrono::{NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::error::{VedicApiError, VedicApiResult};
use crate::client::VedicApiClient;
use super::types::{Planet, ZodiacSign, BirthChart, PlanetPosition, HouseCusp, Dignity};

/// Request for birth chart calculation
#[derive(Debug, Clone, Serialize)]
pub struct BirthChartRequest {
    /// Birth date (YYYY-MM-DD)
    pub birth_date: String,
    /// Birth time (HH:MM:SS)
    pub birth_time: String,
    /// Latitude of birth location
    pub latitude: f64,
    /// Longitude of birth location
    pub longitude: f64,
    /// Timezone offset in hours
    pub timezone: f64,
    /// Ayanamsa to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
    /// House system (placidus, whole-sign, equal, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub house_system: Option<String>,
}

impl BirthChartRequest {
    /// Create a new birth chart request
    pub fn new(
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> Self {
        Self {
            birth_date: birth_datetime.date().format("%Y-%m-%d").to_string(),
            birth_time: birth_datetime.time().format("%H:%M:%S").to_string(),
            latitude,
            longitude,
            timezone,
            ayanamsa: Some("lahiri".to_string()),
            house_system: Some("whole-sign".to_string()),
        }
    }

    /// Set ayanamsa
    pub fn with_ayanamsa(mut self, ayanamsa: &str) -> Self {
        self.ayanamsa = Some(ayanamsa.to_string());
        self
    }

    /// Set house system
    pub fn with_house_system(mut self, system: &str) -> Self {
        self.house_system = Some(system.to_string());
        self
    }
}

/// Raw API response for birth chart
#[derive(Debug, Clone, Deserialize)]
pub struct BirthChartApiResponse {
    pub planets: Vec<PlanetApiResponse>,
    pub houses: Vec<HouseApiResponse>,
    pub ascendant: AscendantApiResponse,
    #[serde(default)]
    pub ayanamsa: Option<AyanamsaApiResponse>,
}

/// Planet from API response
#[derive(Debug, Clone, Deserialize)]
pub struct PlanetApiResponse {
    pub name: String,
    pub sign: String,
    pub sign_num: u8,
    pub degree: f64,
    pub longitude: f64,
    pub house: u8,
    #[serde(default)]
    pub nakshatra: Option<String>,
    #[serde(default)]
    pub nakshatra_pada: Option<u8>,
    #[serde(default)]
    pub is_retrograde: Option<bool>,
    #[serde(default)]
    pub speed: Option<f64>,
}

/// House from API response
#[derive(Debug, Clone, Deserialize)]
pub struct HouseApiResponse {
    pub house: u8,
    pub sign: String,
    pub sign_num: u8,
    pub degree: f64,
}

/// Ascendant from API response
#[derive(Debug, Clone, Deserialize)]
pub struct AscendantApiResponse {
    pub sign: String,
    pub sign_num: u8,
    pub degree: f64,
    pub longitude: f64,
}

/// Ayanamsa from API response
#[derive(Debug, Clone, Deserialize)]
pub struct AyanamsaApiResponse {
    pub name: String,
    pub value: f64,
}

impl VedicApiClient {
    /// Get birth chart from API with request object
    ///
    /// FAPI-046: Implement GET /horoscope-chart endpoint
    pub async fn fetch_birth_chart(&self, request: &BirthChartRequest) -> VedicApiResult<BirthChartApiResponse> {
        let response = self.post("/horoscope-chart", request).await?;
        serde_json::from_value(response)
            .map_err(|e| VedicApiError::ParseError(format!("Failed to parse birth chart response: {}", e)))
    }

    /// Convenience method for getting a birth chart with common defaults
    pub async fn fetch_birth_chart_simple(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<BirthChartApiResponse> {
        let request = BirthChartRequest::new(birth_datetime, latitude, longitude, timezone);
        self.fetch_birth_chart(&request).await
    }
}

/// Parse planet name to enum
pub fn parse_planet(name: &str) -> VedicApiResult<Planet> {
    match name.to_lowercase().as_str() {
        "sun" | "surya" => Ok(Planet::Sun),
        "moon" | "chandra" => Ok(Planet::Moon),
        "mars" | "mangal" => Ok(Planet::Mars),
        "mercury" | "budha" => Ok(Planet::Mercury),
        "jupiter" | "guru" => Ok(Planet::Jupiter),
        "venus" | "shukra" => Ok(Planet::Venus),
        "saturn" | "shani" => Ok(Planet::Saturn),
        "rahu" => Ok(Planet::Rahu),
        "ketu" => Ok(Planet::Ketu),
        "ascendant" | "lagna" => Ok(Planet::Ascendant),
        _ => Err(VedicApiError::ParseError(format!("Unknown planet: {}", name))),
    }
}

/// Parse sign name to enum
pub fn parse_sign(name: &str) -> VedicApiResult<ZodiacSign> {
    match name.to_lowercase().as_str() {
        "aries" | "mesha" => Ok(ZodiacSign::Aries),
        "taurus" | "vrishabha" => Ok(ZodiacSign::Taurus),
        "gemini" | "mithuna" => Ok(ZodiacSign::Gemini),
        "cancer" | "karka" => Ok(ZodiacSign::Cancer),
        "leo" | "simha" => Ok(ZodiacSign::Leo),
        "virgo" | "kanya" => Ok(ZodiacSign::Virgo),
        "libra" | "tula" => Ok(ZodiacSign::Libra),
        "scorpio" | "vrishchika" => Ok(ZodiacSign::Scorpio),
        "sagittarius" | "dhanu" => Ok(ZodiacSign::Sagittarius),
        "capricorn" | "makara" => Ok(ZodiacSign::Capricorn),
        "aquarius" | "kumbha" => Ok(ZodiacSign::Aquarius),
        "pisces" | "meena" => Ok(ZodiacSign::Pisces),
        _ => Err(VedicApiError::ParseError(format!("Unknown sign: {}", name))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_birth_chart_request_creation() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let request = BirthChartRequest::new(dt, 12.97, 77.59, 5.5);
        
        assert_eq!(request.birth_date, "1990-06-15");
        assert_eq!(request.birth_time, "10:30:00");
        assert_eq!(request.ayanamsa, Some("lahiri".to_string()));
    }

    #[test]
    fn test_parse_planet() {
        assert_eq!(parse_planet("Sun").unwrap(), Planet::Sun);
        assert_eq!(parse_planet("surya").unwrap(), Planet::Sun);
        assert_eq!(parse_planet("JUPITER").unwrap(), Planet::Jupiter);
        assert!(parse_planet("Unknown").is_err());
    }

    #[test]
    fn test_parse_sign() {
        assert_eq!(parse_sign("Aries").unwrap(), ZodiacSign::Aries);
        assert_eq!(parse_sign("mesha").unwrap(), ZodiacSign::Aries);
        assert!(parse_sign("Unknown").is_err());
    }
}
