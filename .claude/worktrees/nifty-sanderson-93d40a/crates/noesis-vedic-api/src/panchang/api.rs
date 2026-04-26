//! Panchang API endpoint implementations
//!
//! FAPI-012: Implement GET /panchang endpoint call
//! FAPI-017: Implement GET /sunrise-sunset endpoint

use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::error::{VedicApiError, VedicApiResult};
use crate::client::VedicApiClient;

/// Request parameters for Panchang API call
#[derive(Debug, Clone, Serialize)]
pub struct PanchangApiRequest {
    /// Date for panchang calculation (YYYY-MM-DD)
    pub date: String,
    /// Latitude of location
    pub latitude: f64,
    /// Longitude of location
    pub longitude: f64,
    /// Time for calculation (HH:MM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    /// Timezone offset in hours (e.g., 5.5 for IST)
    pub timezone: f64,
    /// Ayanamsa type (lahiri, raman, krishnamurti, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl PanchangApiRequest {
    /// Create a new Panchang API request
    pub fn new(date: NaiveDate, latitude: f64, longitude: f64, timezone: f64) -> Self {
        Self {
            date: date.format("%Y-%m-%d").to_string(),
            latitude,
            longitude,
            time: None,
            timezone,
            ayanamsa: Some("lahiri".to_string()),
        }
    }

    /// Add specific time to the request
    pub fn with_time(mut self, time: NaiveTime) -> Self {
        self.time = Some(time.format("%H:%M").to_string());
        self
    }

    /// Set ayanamsa type
    pub fn with_ayanamsa(mut self, ayanamsa: &str) -> Self {
        self.ayanamsa = Some(ayanamsa.to_string());
        self
    }
}

/// Raw API response for Panchang
#[derive(Debug, Clone, Deserialize)]
pub struct PanchangApiResponse {
    pub tithi: TithiApiResponse,
    pub nakshatra: NakshatraApiResponse,
    pub yoga: YogaApiResponse,
    pub karana: KaranaApiResponse,
    pub vara: VaraApiResponse,
    #[serde(default)]
    pub sunrise: Option<String>,
    #[serde(default)]
    pub sunset: Option<String>,
    #[serde(default)]
    pub moonrise: Option<String>,
    #[serde(default)]
    pub moonset: Option<String>,
}

/// Tithi from API
#[derive(Debug, Clone, Deserialize)]
pub struct TithiApiResponse {
    pub number: u8,
    pub name: String,
    pub paksha: String,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub deity: Option<String>,
}

/// Nakshatra from API
#[derive(Debug, Clone, Deserialize)]
pub struct NakshatraApiResponse {
    pub number: u8,
    pub name: String,
    #[serde(default)]
    pub pada: Option<u8>,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub lord: Option<String>,
    #[serde(default)]
    pub deity: Option<String>,
}

/// Yoga from API
#[derive(Debug, Clone, Deserialize)]
pub struct YogaApiResponse {
    pub number: u8,
    pub name: String,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub meaning: Option<String>,
}

/// Karana from API
#[derive(Debug, Clone, Deserialize)]
pub struct KaranaApiResponse {
    pub number: u8,
    pub name: String,
    #[serde(default)]
    pub end_time: Option<String>,
}

/// Vara (weekday) from API
#[derive(Debug, Clone, Deserialize)]
pub struct VaraApiResponse {
    pub number: u8,
    pub name: String,
    #[serde(default)]
    pub lord: Option<String>,
}

/// Request for sunrise/sunset calculation
#[derive(Debug, Clone, Serialize)]
pub struct SunriseSunsetRequest {
    pub date: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
}

/// Response from sunrise/sunset API
#[derive(Debug, Clone, Deserialize)]
pub struct SunriseSunsetResponse {
    pub sunrise: String,
    pub sunset: String,
    #[serde(default)]
    pub dawn: Option<String>,
    #[serde(default)]
    pub dusk: Option<String>,
    #[serde(default)]
    pub solar_noon: Option<String>,
    #[serde(default)]
    pub day_duration: Option<String>,
}

impl VedicApiClient {
    /// Call the Panchang API endpoint
    ///
    /// FAPI-012: GET /panchang endpoint
    pub async fn get_panchang_raw(&self, request: &PanchangApiRequest) -> VedicApiResult<PanchangApiResponse> {
        let response = self.get("/panchang", Some(request)).await?;
        serde_json::from_value(response)
            .map_err(|e| VedicApiError::ParseError(format!("Failed to parse panchang response: {}", e)))
    }

    /// Get sunrise and sunset times
    ///
    /// FAPI-017: GET /sunrise-sunset endpoint
    pub async fn get_sunrise_sunset(&self, request: &SunriseSunsetRequest) -> VedicApiResult<SunriseSunsetResponse> {
        let response = self.get("/sunrise-sunset", Some(request)).await?;
        serde_json::from_value(response)
            .map_err(|e| VedicApiError::ParseError(format!("Failed to parse sunrise/sunset response: {}", e)))
    }

    /// Get Panchang for a specific date and location
    pub async fn get_panchang_for_date(
        &self,
        date: NaiveDate,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<PanchangApiResponse> {
        let request = PanchangApiRequest::new(date, latitude, longitude, timezone);
        self.get_panchang_raw(&request).await
    }

    /// Get Panchang for a specific datetime and location
    pub async fn get_panchang_for_datetime(
        &self,
        datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<PanchangApiResponse> {
        let request = PanchangApiRequest::new(datetime.date(), latitude, longitude, timezone)
            .with_time(datetime.time());
        self.get_panchang_raw(&request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panchang_request_creation() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let request = PanchangApiRequest::new(date, 12.97, 77.59, 5.5);
        
        assert_eq!(request.date, "2024-01-15");
        assert_eq!(request.latitude, 12.97);
        assert_eq!(request.longitude, 77.59);
        assert_eq!(request.timezone, 5.5);
    }

    #[test]
    fn test_panchang_request_with_time() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let time = NaiveTime::from_hms_opt(10, 30, 0).unwrap();
        let request = PanchangApiRequest::new(date, 12.97, 77.59, 5.5)
            .with_time(time);
        
        assert_eq!(request.time, Some("10:30".to_string()));
    }
}
