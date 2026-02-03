//! Yogas API endpoint implementations
//!
//! FAPI-064: Implement GET /yogas endpoint

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::error::{VedicApiError, VedicApiResult};
use crate::client::VedicApiClient;
use super::types::{YogaAnalysis, DetectedYoga, YogaCategory, YogaStrength};

/// Request for yoga detection
#[derive(Debug, Clone, Serialize)]
pub struct YogaDetectionRequest {
    pub birth_date: String,
    pub birth_time: String,
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
}

impl YogaDetectionRequest {
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
            categories: None,
        }
    }

    /// Filter to specific yoga categories
    pub fn with_categories(mut self, categories: Vec<YogaCategory>) -> Self {
        self.categories = Some(categories.iter().map(|c| c.to_string()).collect());
        self
    }
}

/// API response for yoga detection
#[derive(Debug, Clone, Deserialize)]
pub struct YogaApiResponse {
    pub yogas: Vec<YogaApiItem>,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YogaApiItem {
    pub name: String,
    pub category: String,
    #[serde(default)]
    pub strength: Option<String>,
    pub planets: Vec<String>,
    #[serde(default)]
    pub houses: Option<Vec<u8>>,
    pub description: String,
    #[serde(default)]
    pub results: Option<String>,
}

impl VedicApiClient {
    /// Get yoga analysis
    ///
    /// FAPI-064: GET /yogas endpoint
    pub async fn get_yogas(&self, request: &YogaDetectionRequest) -> VedicApiResult<YogaApiResponse> {
        let response = self.post("/yogas", request).await?;
        serde_json::from_value(response)
            .map_err(|e| VedicApiError::ParseError(format!("Failed to parse yoga response: {}", e)))
    }
}

/// Map API response to internal analysis
pub fn map_yoga_response(response: YogaApiResponse) -> YogaAnalysis {
    let mut analysis = YogaAnalysis::empty();
    
    for item in response.yogas {
        let category = parse_yoga_category(&item.category);
        let strength = item.strength.as_ref()
            .map(|s| parse_yoga_strength(s))
            .unwrap_or(YogaStrength::Partial);
        
        let yoga = DetectedYoga {
            name: item.name,
            category,
            strength,
            planets_involved: item.planets,
            houses_involved: item.houses.unwrap_or_default(),
            description: item.description,
            results: item.results.unwrap_or_default(),
            activation_periods: vec![],
        };
        
        analysis.add_yoga(yoga);
    }
    
    analysis.calculate_score();
    analysis.generate_summary();
    
    analysis
}

fn parse_yoga_category(category: &str) -> YogaCategory {
    match category.to_lowercase().as_str() {
        "raj" | "raja" | "raj yoga" => YogaCategory::RajYoga,
        "dhana" | "wealth" => YogaCategory::DhanaYoga,
        "mahapurusha" => YogaCategory::MahapurushaYoga,
        "arishta" | "affliction" => YogaCategory::ArishtaYoga,
        "nabhasa" => YogaCategory::NabhasaYoga,
        "chandra" | "moon" => YogaCategory::ChandraYoga,
        "surya" | "sun" => YogaCategory::SuryaYoga,
        _ => YogaCategory::ShubhaYoga,
    }
}

fn parse_yoga_strength(strength: &str) -> YogaStrength {
    match strength.to_lowercase().as_str() {
        "full" | "strong" | "complete" => YogaStrength::Full,
        "partial" | "moderate" => YogaStrength::Partial,
        "weak" | "mild" => YogaStrength::Weak,
        "cancelled" | "none" => YogaStrength::Cancelled,
        _ => YogaStrength::Partial,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_yoga_request_creation() {
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1990, 6, 15).unwrap(),
            NaiveTime::from_hms_opt(10, 30, 0).unwrap(),
        );
        let request = YogaDetectionRequest::new(dt, 12.97, 77.59, 5.5);
        
        assert_eq!(request.birth_date, "1990-06-15");
    }

    #[test]
    fn test_parse_yoga_category() {
        assert_eq!(parse_yoga_category("Raj Yoga"), YogaCategory::RajYoga);
        assert_eq!(parse_yoga_category("dhana"), YogaCategory::DhanaYoga);
    }

    #[test]
    fn test_parse_yoga_strength() {
        assert_eq!(parse_yoga_strength("Full"), YogaStrength::Full);
        assert_eq!(parse_yoga_strength("weak"), YogaStrength::Weak);
    }
}
