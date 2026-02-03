//! Muhurta API endpoint implementations
//!
//! FAPI-082: Implement GET /muhurta endpoint

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::{VedicApiError, VedicApiResult};
use crate::client::VedicApiClient;
use super::types::{MuhurtaActivity, MuhurtaResults, SelectedMuhurta, MuhurtaQuality};

/// Request for Muhurta calculation
#[derive(Debug, Clone, Serialize)]
pub struct MuhurtaRequest {
    /// Activity type
    pub activity: String,
    /// Start date of search
    pub from_date: String,
    /// End date of search
    pub to_date: String,
    /// Location latitude
    pub latitude: f64,
    /// Location longitude
    pub longitude: f64,
    /// Timezone
    pub timezone: f64,
    /// Ayanamsa
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ayanamsa: Option<String>,
}

impl MuhurtaRequest {
    pub fn new(
        activity: MuhurtaActivity,
        from_date: NaiveDate,
        to_date: NaiveDate,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> Self {
        Self {
            activity: activity.to_string().to_lowercase(),
            from_date: from_date.format("%Y-%m-%d").to_string(),
            to_date: to_date.format("%Y-%m-%d").to_string(),
            latitude,
            longitude,
            timezone,
            ayanamsa: Some("lahiri".to_string()),
        }
    }
}

/// API response for Muhurta
#[derive(Debug, Clone, Deserialize)]
pub struct MuhurtaApiResponse {
    pub muhurtas: Vec<MuhurtaItemResponse>,
    #[serde(default)]
    pub advice: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MuhurtaItemResponse {
    pub start_time: String,
    pub end_time: String,
    pub date: String,
    pub quality: String,
    pub score: u8,
    pub tithi: String,
    pub nakshatra: String,
    #[serde(default)]
    pub yoga: Option<String>,
    #[serde(default)]
    pub karana: Option<String>,
    pub vara: String,
    #[serde(default)]
    pub favorable: Option<Vec<String>>,
    #[serde(default)]
    pub unfavorable: Option<Vec<String>>,
}

impl VedicApiClient {
    /// Get Muhurta for an activity
    ///
    /// FAPI-082: GET /muhurta endpoint
    pub async fn get_muhurta(&self, request: &MuhurtaRequest) -> VedicApiResult<MuhurtaApiResponse> {
        let response = self.post("/muhurta", request).await?;
        serde_json::from_value(response)
            .map_err(|e| VedicApiError::ParseError(format!("Failed to parse muhurta response: {}", e)))
    }

    /// Find marriage muhurtas
    pub async fn find_marriage_muhurta(
        &self,
        from_date: NaiveDate,
        to_date: NaiveDate,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<MuhurtaApiResponse> {
        let request = MuhurtaRequest::new(
            MuhurtaActivity::Marriage,
            from_date,
            to_date,
            latitude,
            longitude,
            timezone,
        );
        self.get_muhurta(&request).await
    }

    /// Find business muhurtas
    pub async fn find_business_muhurta(
        &self,
        from_date: NaiveDate,
        to_date: NaiveDate,
        latitude: f64,
        longitude: f64,
        timezone: f64,
    ) -> VedicApiResult<MuhurtaApiResponse> {
        let request = MuhurtaRequest::new(
            MuhurtaActivity::Business,
            from_date,
            to_date,
            latitude,
            longitude,
            timezone,
        );
        self.get_muhurta(&request).await
    }
}

/// Map API response to internal results
pub fn map_muhurta_response(response: MuhurtaApiResponse, activity: MuhurtaActivity) -> MuhurtaResults {
    let muhurtas: Vec<SelectedMuhurta> = response.muhurtas.iter()
        .filter_map(|m| {
            let date = chrono::NaiveDate::parse_from_str(&m.date, "%Y-%m-%d").ok()?;
            let start_time = chrono::NaiveTime::parse_from_str(&m.start_time, "%H:%M").ok()?;
            let end_time = chrono::NaiveTime::parse_from_str(&m.end_time, "%H:%M").ok()?;
            
            Some(SelectedMuhurta {
                start_time: chrono::NaiveDateTime::new(date, start_time),
                end_time: chrono::NaiveDateTime::new(date, end_time),
                quality: parse_quality(&m.quality),
                tithi: m.tithi.clone(),
                nakshatra: m.nakshatra.clone(),
                yoga: m.yoga.clone().unwrap_or_default(),
                karana: m.karana.clone().unwrap_or_default(),
                vara: m.vara.clone(),
                score: m.score,
                favorable_factors: m.favorable.clone().unwrap_or_default(),
                unfavorable_factors: m.unfavorable.clone().unwrap_or_default(),
                recommendation: String::new(),
            })
        })
        .collect();
    
    let excellent_count = muhurtas.iter()
        .filter(|m| m.quality == MuhurtaQuality::Excellent)
        .count();
    let good_count = muhurtas.iter()
        .filter(|m| m.quality == MuhurtaQuality::Good)
        .count();
    
    MuhurtaResults {
        activity,
        from_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        to_date: chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        muhurtas,
        excellent_count,
        good_count,
        advice: response.advice.unwrap_or_default(),
    }
}

fn parse_quality(quality: &str) -> MuhurtaQuality {
    match quality.to_lowercase().as_str() {
        "excellent" => MuhurtaQuality::Excellent,
        "good" => MuhurtaQuality::Good,
        "average" => MuhurtaQuality::Average,
        "not recommended" => MuhurtaQuality::NotRecommended,
        "avoid" => MuhurtaQuality::Avoid,
        _ => MuhurtaQuality::Average,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_muhurta_request() {
        let from = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        
        let request = MuhurtaRequest::new(
            MuhurtaActivity::Marriage,
            from,
            to,
            12.97,
            77.59,
            5.5,
        );
        
        assert_eq!(request.activity, "marriage");
    }
}
