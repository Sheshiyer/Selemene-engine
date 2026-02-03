//! Transit API endpoint implementations
//!
//! FAPI-074: Implement GET /transits endpoint

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::error::{VedicApiError, VedicApiResult};
use crate::client::VedicApiClient;
use super::types::{TransitEvent, TransitAnalysis};

/// Request for transit calculation
#[derive(Debug, Clone, Serialize)]
pub struct TransitRequest {
    /// Birth date
    pub birth_date: String,
    /// Birth time
    pub birth_time: String,
    /// Birth latitude
    pub latitude: f64,
    /// Birth longitude
    pub longitude: f64,
    /// Timezone
    pub timezone: f64,
    /// Date to calculate transits for
    pub transit_date: String,
    /// Ayanamsa
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl TransitRequest {
    pub fn new(
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
        transit_date: NaiveDate,
    ) -> Self {
        Self {
            birth_date: birth_datetime.date().format("%Y-%m-%d").to_string(),
            birth_time: birth_datetime.time().format("%H:%M:%S").to_string(),
            latitude,
            longitude,
            timezone,
            transit_date: transit_date.format("%Y-%m-%d").to_string(),
            ayanamsa: Some("lahiri".to_string()),
        }
    }
}

/// API response for transits
#[derive(Debug, Clone, Deserialize)]
pub struct TransitApiResponse {
    pub transits: Vec<TransitPlanetResponse>,
    #[serde(default)]
    pub sade_sati: Option<SadeSatiResponse>,
    #[serde(default)]
    pub jupiter_transit: Option<JupiterTransitResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransitPlanetResponse {
    pub planet: String,
    pub sign: String,
    pub degree: f64,
    #[serde(default)]
    pub is_retrograde: Option<bool>,
    #[serde(default)]
    pub natal_aspects: Option<Vec<NatalAspectResponse>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatalAspectResponse {
    pub natal_planet: String,
    pub aspect_type: String,
    pub orb: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SadeSatiResponse {
    pub is_active: bool,
    #[serde(default)]
    pub phase: Option<String>,
    pub saturn_sign: String,
    pub moon_sign: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterTransitResponse {
    pub sign: String,
    pub from_ascendant: u8,
    pub from_moon: u8,
    #[serde(default)]
    pub quality: Option<String>,
}

impl VedicApiClient {
    /// Get transit analysis
    ///
    /// FAPI-074: GET /transits endpoint
    pub async fn get_transits(&self, request: &TransitRequest) -> VedicApiResult<TransitApiResponse> {
        let response = self.post("/transits", request).await?;
        serde_json::from_value(response)
            .map_err(|e| VedicApiError::ParseError(format!("Failed to parse transit response: {}", e)))
    }

    /// Get current transits for a birth chart
    pub async fn get_current_transits(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<TransitApiResponse> {
        let today = chrono::Utc::now().date_naive();
        let request = TransitRequest::new(birth_datetime, latitude, longitude, timezone, today);
        self.get_transits(&request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    #[test]
    fn test_transit_request() {
        let birth = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let transit = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        
        let request = TransitRequest::new(birth, 12.97, 77.59, 5.5, transit);
        
        assert_eq!(request.birth_date, "1990-06-15");
        assert_eq!(request.transit_date, "2024-01-15");
    }
}
