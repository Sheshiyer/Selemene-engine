//! Varga API endpoint implementations
//!
//! FAPI-057: Implement GET /horoscope-chart with vargas parameter

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::error::{VedicApiError, VedicApiResult};
use crate::client::VedicApiClient;
use super::types::{VargaType, VargaChart, VargaPosition};

/// Request for divisional chart calculation
#[derive(Debug, Clone, Serialize)]
pub struct VargaChartRequest {
    /// Birth date (YYYY-MM-DD)
    pub birth_date: String,
    /// Birth time (HH:MM:SS)
    pub birth_time: String,
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Timezone offset
    pub timezone: f64,
    /// Which varga to calculate
    pub varga: String,
    /// Ayanamsa
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl VargaChartRequest {
    /// Create a new varga chart request
    pub fn new(
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
        varga: VargaType,
    ) -> Self {
        Self {
            birth_date: birth_datetime.date().format("%Y-%m-%d").to_string(),
            birth_time: birth_datetime.time().format("%H:%M:%S").to_string(),
            latitude,
            longitude,
            timezone,
            varga: format!("D{}", varga.divisor()),
            ayanamsa: Some("lahiri".to_string()),
        }
    }
}

/// API response for varga chart
#[derive(Debug, Clone, Deserialize)]
pub struct VargaChartApiResponse {
    pub varga: String,
    pub planets: Vec<VargaPlanetResponse>,
    pub ascendant: VargaAscendantResponse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VargaPlanetResponse {
    pub name: String,
    pub sign: String,
    pub sign_num: u8,
    #[serde(default)]
    pub degree: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VargaAscendantResponse {
    pub sign: String,
    pub sign_num: u8,
}

impl VedicApiClient {
    /// Get a divisional chart
    ///
    /// FAPI-057: GET /horoscope-chart with vargas parameter
    pub async fn get_varga_chart(&self, request: &VargaChartRequest) -> VedicApiResult<VargaChartApiResponse> {
        let response = self.post("/horoscope-chart/varga", request).await?;
        serde_json::from_value(response)
            .map_err(|e| VedicApiError::ParseError(format!("Failed to parse varga chart: {}", e)))
    }

    /// Get Navamsa chart
    pub async fn get_navamsa_chart(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<VargaChartApiResponse> {
        let request = VargaChartRequest::new(birth_datetime, latitude, longitude, timezone, VargaType::Navamsa);
        self.get_varga_chart(&request).await
    }

    /// Get Dasamsa chart (career)
    pub async fn get_dasamsa_chart(
        &self,
        birth_datetime: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<VargaChartApiResponse> {
        let request = VargaChartRequest::new(birth_datetime, latitude, longitude, timezone, VargaType::Dasamsa);
        self.get_varga_chart(&request).await
    }
}

/// Map API response to internal VargaChart
pub fn map_varga_response(response: VargaChartApiResponse, varga_type: VargaType) -> VargaChart {
    let positions = response.planets.iter().map(|p| {
        VargaPosition {
            varga: varga_type,
            planet: p.name.clone(),
            sign: p.sign.clone(),
            sign_number: p.sign_num,
            degree: p.degree,
        }
    }).collect();

    VargaChart {
        varga_type,
        positions,
        ascendant_sign: response.ascendant.sign,
        ascendant_sign_number: response.ascendant.sign_num,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_varga_request_creation() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let request = VargaChartRequest::new(dt, 12.97, 77.59, 5.5, VargaType::Navamsa);
        
        assert_eq!(request.varga, "D9");
    }
}
