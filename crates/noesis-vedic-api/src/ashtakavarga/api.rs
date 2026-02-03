//! Ashtakavarga API endpoint implementations
//!
//! FAPI-071: Implement GET /ashtakavarga endpoint

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::error::{VedicApiError, VedicApiResult};
use crate::client::VedicApiClient;
use super::types::{SarvaAshtakavarga, PlanetAshtakavarga};

/// Request for Ashtakavarga calculation
#[derive(Debug, Clone, Serialize)]
pub struct AshtakavargaRequest {
    pub birth_date: String,
    pub birth_time: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl AshtakavargaRequest {
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
        }
    }
}

/// API response for Ashtakavarga
#[derive(Debug, Clone, Deserialize)]
pub struct AshtakavargaApiResponse {
    pub planets: Vec<AshtakavargaPlanetResponse>,
    pub sarva: SarvaResponse,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AshtakavargaPlanetResponse {
    pub name: String,
    pub points: Vec<u8>,
    pub total: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SarvaResponse {
    pub points: Vec<u8>,
    pub total: u16,
}

impl VedicApiClient {
    /// Get Ashtakavarga analysis
    ///
    /// FAPI-071: GET /ashtakavarga endpoint
    pub async fn get_ashtakavarga(&self, request: &AshtakavargaRequest) -> VedicApiResult<AshtakavargaApiResponse> {
        let response = self.post("/ashtakavarga", request).await?;
        serde_json::from_value(response)
            .map_err(|e| VedicApiError::ParseError(format!("Failed to parse ashtakavarga response: {}", e)))
    }
}

/// Map API response to internal Sarva Ashtakavarga
pub fn map_ashtakavarga_response(response: AshtakavargaApiResponse) -> SarvaAshtakavarga {
    let mut sarva = SarvaAshtakavarga::empty();
    
    for planet in response.planets {
        let mut av = PlanetAshtakavarga::empty(&planet.name);
        
        for (i, points) in planet.points.iter().enumerate() {
            if i < 12 {
                av.sign_points[i] = *points;
            }
        }
        av.total_points = planet.total;
        
        sarva.planets.push(av);
    }
    
    for (i, points) in response.sarva.points.iter().enumerate() {
        if i < 12 {
            sarva.sarva_points[i] = *points;
        }
    }
    sarva.grand_total = response.sarva.total;
    
    sarva
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_ashtakavarga_request() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let request = AshtakavargaRequest::new(dt, 12.97, 77.59, 5.5);
        
        assert_eq!(request.birth_date, "1990-06-15");
    }
}
