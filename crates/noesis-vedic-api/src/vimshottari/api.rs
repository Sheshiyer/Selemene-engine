//! Vimshottari Dasha API endpoint implementations
//!
//! FAPI-032: Implement POST /vimshottari-dasha endpoint
//! FAPI-033: Support Maha Dasha only level
//! FAPI-034: Support Antar Dasha level
//! FAPI-035: Support Pratyantar Dasha level
//! FAPI-036: Support Sookshma Dasha level

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};

use crate::error::{VedicApiError, VedicApiResult};
use crate::client::VedicApiClient;
use super::types::{DashaLevel, DashaLord, DashaPeriod, VimshottariTimeline, DashaBalance};

/// Request for Vimshottari Dasha calculation
#[derive(Debug, Clone, Serialize)]
pub struct VimshottariRequest {
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
    /// Level of detail to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    /// Ayanamsa to use
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl VimshottariRequest {
    /// Create a new Vimshottari request from birth details
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
            level: None,
            ayanamsa: Some("lahiri".to_string()),
        }
    }

    /// Set the dasha level detail
    pub fn with_level(mut self, level: DashaLevel) -> Self {
        self.level = Some(match level {
            DashaLevel::Mahadasha => "mahadasha".to_string(),
            DashaLevel::Antardasha => "antardasha".to_string(),
            DashaLevel::Pratyantardasha => "pratyantardasha".to_string(),
            DashaLevel::Sookshma => "sookshma".to_string(),
            DashaLevel::Prana => "prana".to_string(),
        });
        self
    }

    /// Set the ayanamsa
    pub fn with_ayanamsa(mut self, ayanamsa: &str) -> Self {
        self.ayanamsa = Some(ayanamsa.to_string());
        self
    }
}

/// Raw API response for Vimshottari Dasha
#[derive(Debug, Clone, Deserialize)]
pub struct VimshottariApiResponse {
    /// Moon nakshatra at birth
    pub moon_nakshatra: MoonNakshatraInfo,
    /// Balance of dasha at birth
    pub dasha_balance: DashaBalanceResponse,
    /// Mahadasha periods
    pub mahadashas: Vec<MahadashaResponse>,
}

/// Moon nakshatra information from API
#[derive(Debug, Clone, Deserialize)]
pub struct MoonNakshatraInfo {
    pub name: String,
    pub number: u8,
    pub lord: String,
    #[serde(default)]
    pub pada: Option<u8>,
    #[serde(default)]
    pub degree: Option<f64>,
}

/// Dasha balance at birth from API
#[derive(Debug, Clone, Deserialize)]
pub struct DashaBalanceResponse {
    pub lord: String,
    pub years: u32,
    pub months: u32,
    pub days: u32,
}

/// Single Mahadasha from API response
#[derive(Debug, Clone, Deserialize)]
pub struct MahadashaResponse {
    pub lord: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub antardashas: Option<Vec<AntardashaResponse>>,
}

/// Antardasha from API response
#[derive(Debug, Clone, Deserialize)]
pub struct AntardashaResponse {
    pub lord: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub pratyantardashas: Option<Vec<PratyantardashaResponse>>,
}

/// Pratyantardasha from API response
#[derive(Debug, Clone, Deserialize)]
pub struct PratyantardashaResponse {
    pub lord: String,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub sookshmas: Option<Vec<SookshmaResponse>>,
}

/// Sookshma dasha from API response
#[derive(Debug, Clone, Deserialize)]
pub struct SookshmaResponse {
    pub lord: String,
    pub start_date: String,
    pub end_date: String,
}

impl VedicApiClient {
    /// Get Vimshottari Dasha timeline
    ///
    /// FAPI-032: POST /vimshottari-dasha endpoint
    pub async fn get_vimshottari_dasha(&self, request: &VimshottariRequest) -> VedicApiResult<VimshottariApiResponse> {
        let response = self.post("/vimshottari-dasha", request).await?;
        serde_json::from_value(response)
            .map_err(|e| VedicApiError::ParseError(format!("Failed to parse vimshottari response: {}", e)))
    }

    /// Get Mahadasha level only
    ///
    /// FAPI-033: Support Maha Dasha only level
    pub async fn get_mahadasha_only(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<VimshottariApiResponse> {
        let request = VimshottariRequest::new(birth_datetime, latitude, longitude, timezone)
            .with_level(DashaLevel::Mahadasha);
        self.get_vimshottari_dasha(&request).await
    }

    /// Get Antardasha level
    ///
    /// FAPI-034: Support Antar Dasha level
    pub async fn get_antardasha_level(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<VimshottariApiResponse> {
        let request = VimshottariRequest::new(birth_datetime, latitude, longitude, timezone)
            .with_level(DashaLevel::Antardasha);
        self.get_vimshottari_dasha(&request).await
    }

    /// Get Pratyantardasha level
    ///
    /// FAPI-035: Support Pratyantar Dasha level
    pub async fn get_pratyantardasha_level(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<VimshottariApiResponse> {
        let request = VimshottariRequest::new(birth_datetime, latitude, longitude, timezone)
            .with_level(DashaLevel::Pratyantardasha);
        self.get_vimshottari_dasha(&request).await
    }

    /// Get Sookshma Dasha level
    ///
    /// FAPI-036: Support Sookshma Dasha level
    pub async fn get_sookshma_level(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<VimshottariApiResponse> {
        let request = VimshottariRequest::new(birth_datetime, latitude, longitude, timezone)
            .with_level(DashaLevel::Sookshma);
        self.get_vimshottari_dasha(&request).await
    }
}

/// Parse lord string to DashaLord enum
pub fn parse_dasha_lord(lord: &str) -> VedicApiResult<DashaLord> {
    match lord.to_lowercase().as_str() {
        "sun" | "surya" => Ok(DashaLord::Sun),
        "moon" | "chandra" => Ok(DashaLord::Moon),
        "mars" | "mangal" => Ok(DashaLord::Mars),
        "rahu" => Ok(DashaLord::Rahu),
        "jupiter" | "guru" => Ok(DashaLord::Jupiter),
        "saturn" | "shani" => Ok(DashaLord::Saturn),
        "mercury" | "budha" => Ok(DashaLord::Mercury),
        "ketu" => Ok(DashaLord::Ketu),
        "venus" | "shukra" => Ok(DashaLord::Venus),
        _ => Err(VedicApiError::ParseError(format!("Unknown dasha lord: {}", lord))),
    }
}

/// Parse date string from API
pub fn parse_date(date_str: &str) -> VedicApiResult<NaiveDate> {
    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%d-%m-%Y"))
        .or_else(|_| NaiveDate::parse_from_str(date_str, "%d/%m/%Y"))
        .map_err(|e| VedicApiError::ParseError(format!("Invalid date format '{}': {}", date_str, e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vimshottari_request_creation() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let request = VimshottariRequest::new(dt, 12.97, 77.59, 5.5);
        
        assert_eq!(request.birth_date, "1990-06-15");
        assert_eq!(request.birth_time, "10:30:00");
    }

    #[test]
    fn test_parse_dasha_lord() {
        assert_eq!(parse_dasha_lord("Sun").unwrap(), DashaLord::Sun);
        assert_eq!(parse_dasha_lord("surya").unwrap(), DashaLord::Sun);
        assert_eq!(parse_dasha_lord("RAHU").unwrap(), DashaLord::Rahu);
        assert!(parse_dasha_lord("Unknown").is_err());
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("2024-01-15").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }
}
