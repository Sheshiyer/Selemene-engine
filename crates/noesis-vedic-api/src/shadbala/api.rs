//! Shadbala API endpoint implementations
//!
//! FAPI-068: Implement GET /shadbala endpoint

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::error::{VedicApiError, VedicApiResult};
use crate::client::VedicApiClient;
use super::types::{ShadbalaAnalysis, PlanetShadbala, ShadbalaValue, ShadbalaComponent, ChartStrength, required_shadbala};

/// Request for Shadbala calculation
#[derive(Debug, Clone, Serialize)]
pub struct ShadbalaRequest {
    pub birth_date: String,
    pub birth_time: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl ShadbalaRequest {
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

/// API response for Shadbala
#[derive(Debug, Clone, Deserialize)]
pub struct ShadbalaApiResponse {
    pub planets: Vec<ShadbalaPlanetResponse>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShadbalaPlanetResponse {
    pub name: String,
    pub sthana_bala: f64,
    pub dig_bala: f64,
    pub kala_bala: f64,
    pub chesta_bala: f64,
    pub naisargika_bala: f64,
    pub drik_bala: f64,
    pub total: f64,
}

impl VedicApiClient {
    /// Get Shadbala analysis
    ///
    /// FAPI-068: GET /shadbala endpoint
    pub async fn get_shadbala(&self, request: &ShadbalaRequest) -> VedicApiResult<ShadbalaApiResponse> {
        let response = self.post("/shadbala", request).await?;
        serde_json::from_value(response)
            .map_err(|e| VedicApiError::ParseError(format!("Failed to parse shadbala response: {}", e)))
    }
}

/// Map API response to internal analysis
pub fn map_shadbala_response(response: ShadbalaApiResponse) -> ShadbalaAnalysis {
    let planets: Vec<PlanetShadbala> = response.planets.iter().map(|p| {
        let required = required_shadbala(&p.name);
        let ratio = p.total / required;
        
        PlanetShadbala {
            planet: p.name.clone(),
            components: vec![
                ShadbalaValue { component: ShadbalaComponent::SthanaBala, rupas: p.sthana_bala, shashtiamsas: p.sthana_bala * 60.0 },
                ShadbalaValue { component: ShadbalaComponent::DigBala, rupas: p.dig_bala, shashtiamsas: p.dig_bala * 60.0 },
                ShadbalaValue { component: ShadbalaComponent::KalaBala, rupas: p.kala_bala, shashtiamsas: p.kala_bala * 60.0 },
                ShadbalaValue { component: ShadbalaComponent::ChestaBala, rupas: p.chesta_bala, shashtiamsas: p.chesta_bala * 60.0 },
                ShadbalaValue { component: ShadbalaComponent::NaisargikaBala, rupas: p.naisargika_bala, shashtiamsas: p.naisargika_bala * 60.0 },
                ShadbalaValue { component: ShadbalaComponent::DrikBala, rupas: p.drik_bala, shashtiamsas: p.drik_bala * 60.0 },
            ],
            total_rupas: p.total,
            total_shashtiamsas: p.total * 60.0,
            required_minimum: required,
            strength_ratio: ratio,
            is_strong: ratio >= 1.0,
        }
    }).collect();
    
    let strongest = planets.iter()
        .max_by(|a, b| a.strength_ratio.partial_cmp(&b.strength_ratio).unwrap())
        .map(|p| p.planet.clone())
        .unwrap_or_default();
    
    let weakest = planets.iter()
        .min_by(|a, b| a.strength_ratio.partial_cmp(&b.strength_ratio).unwrap())
        .map(|p| p.planet.clone())
        .unwrap_or_default();
    
    let avg_ratio: f64 = if !planets.is_empty() {
        planets.iter().map(|p| p.strength_ratio).sum::<f64>() / planets.len() as f64
    } else {
        0.0
    };
    
    ShadbalaAnalysis {
        planets,
        strongest_planet: strongest,
        weakest_planet: weakest,
        chart_strength: ChartStrength::from_average_ratio(avg_ratio),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_shadbala_request_creation() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let request = ShadbalaRequest::new(dt, 12.97, 77.59, 5.5);
        
        assert_eq!(request.birth_date, "1990-06-15");
    }
}
