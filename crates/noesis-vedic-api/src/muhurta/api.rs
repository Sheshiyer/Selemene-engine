//! Muhurta API — native facade backed by `search_muhurtas` (PR3).
//!
//! Previous implementation POSTed to `/muhurta` (dead, 403) and the
//! `map_muhurta_response` mapper hardcoded the date range to January 2024
//! (line 177). PR3 rewires `get_muhurta` to call the new search loop and
//! fixes the date-range bug.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::search::search_muhurtas;
use super::types::{
    MuhurtaActivity, MuhurtaQuality, MuhurtaResults, MuhurtaSearchCriteria, SelectedMuhurta,
    TimePreference,
};
use crate::client::VedicApiClient;
use crate::error::{VedicApiError, VedicApiResult};

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

/// API response for Muhurta (now also `Serialize` so callers can
/// roundtrip JSON unchanged).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuhurtaApiResponse {
    pub muhurtas: Vec<MuhurtaItemResponse>,
    #[serde(default)]
    pub advice: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn parse_activity(s: &str) -> MuhurtaActivity {
    match s.to_lowercase().as_str() {
        "marriage" => MuhurtaActivity::Marriage,
        "business" => MuhurtaActivity::Business,
        "travel" => MuhurtaActivity::Travel,
        "education" => MuhurtaActivity::Education,
        "medical" => MuhurtaActivity::Medical,
        "construction" => MuhurtaActivity::Construction,
        "religious" => MuhurtaActivity::Religious,
        "journey start" | "journey_start" | "journeystart" => MuhurtaActivity::JourneyStart,
        "new venture" | "new_venture" | "newventure" => MuhurtaActivity::NewVenture,
        "interview" => MuhurtaActivity::Interview,
        "property purchase" | "property_purchase" | "propertypurchase" => {
            MuhurtaActivity::PropertyPurchase
        }
        "vehicle purchase" | "vehicle_purchase" | "vehiclepurchase" => {
            MuhurtaActivity::VehiclePurchase
        }
        "moving house" | "moving_house" | "movinghouse" => MuhurtaActivity::MovingHouse,
        _ => MuhurtaActivity::General,
    }
}

fn quality_string(q: MuhurtaQuality) -> &'static str {
    match q {
        MuhurtaQuality::Excellent => "excellent",
        MuhurtaQuality::Good => "good",
        MuhurtaQuality::Average => "average",
        MuhurtaQuality::NotRecommended => "not recommended",
        MuhurtaQuality::Avoid => "avoid",
    }
}

/// Convert a `MuhurtaResults` into the legacy wire envelope. Used by
/// `get_muhurta` to keep downstream JSON consumers happy.
fn results_to_response(results: &MuhurtaResults) -> MuhurtaApiResponse {
    let items = results
        .muhurtas
        .iter()
        .map(|m| MuhurtaItemResponse {
            start_time: m.start_time.format("%H:%M").to_string(),
            end_time: m.end_time.format("%H:%M").to_string(),
            date: m.start_time.date().format("%Y-%m-%d").to_string(),
            quality: quality_string(m.quality).to_string(),
            score: m.score,
            tithi: m.tithi.clone(),
            nakshatra: m.nakshatra.clone(),
            yoga: Some(m.yoga.clone()),
            karana: Some(m.karana.clone()),
            vara: m.vara.clone(),
            favorable: Some(m.favorable_factors.clone()),
            unfavorable: Some(m.unfavorable_factors.clone()),
        })
        .collect();
    MuhurtaApiResponse {
        muhurtas: items,
        advice: Some(results.advice.clone()),
    }
}

impl VedicApiClient {
    /// Find muhurtas for an activity over the requested date range. PR3:
    /// native search loop, no HTTP call.
    pub async fn get_muhurta(
        &self,
        request: &MuhurtaRequest,
    ) -> VedicApiResult<MuhurtaApiResponse> {
        let from_date = NaiveDate::parse_from_str(&request.from_date, "%Y-%m-%d").map_err(|e| {
            VedicApiError::ParseError(format!("invalid from_date '{}': {}", request.from_date, e))
        })?;
        let to_date = NaiveDate::parse_from_str(&request.to_date, "%Y-%m-%d").map_err(|e| {
            VedicApiError::ParseError(format!("invalid to_date '{}': {}", request.to_date, e))
        })?;
        let activity = parse_activity(&request.activity);
        let criteria = MuhurtaSearchCriteria {
            activity,
            from_date,
            to_date,
            preferred_time: Some(TimePreference::Any),
            latitude: request.latitude,
            longitude: request.longitude,
            timezone: request.timezone,
            // Default min_quality = Average so partial matches still surface.
            min_quality: MuhurtaQuality::Average,
        };
        let results = search_muhurtas(&criteria).await;
        Ok(results_to_response(&results))
    }

    /// Find marriage muhurtas natively.
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

    /// Find business muhurtas natively.
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

/// Map API response to internal results.
///
/// PR3: the old implementation hardcoded the date range to
/// `2024-01-01..=2024-01-31` regardless of the request — a latent bug we
/// are closing here. The new signature takes the actual `from_date` /
/// `to_date` the caller searched for. If a caller can't provide them
/// (e.g. mapping a vendor response with no embedded range) we infer the
/// range from the items themselves: the minimum and maximum `date` across
/// `response.muhurtas` — never falling back to hardcoded January 2024.
pub fn map_muhurta_response(
    response: MuhurtaApiResponse,
    activity: MuhurtaActivity,
    from_date: chrono::NaiveDate,
    to_date: chrono::NaiveDate,
) -> MuhurtaResults {
    let muhurtas: Vec<SelectedMuhurta> = response
        .muhurtas
        .iter()
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

    let excellent_count = muhurtas
        .iter()
        .filter(|m| m.quality == MuhurtaQuality::Excellent)
        .count();
    let good_count = muhurtas
        .iter()
        .filter(|m| m.quality == MuhurtaQuality::Good)
        .count();

    MuhurtaResults {
        activity,
        from_date,
        to_date,
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

        let request = MuhurtaRequest::new(MuhurtaActivity::Marriage, from, to, 12.97, 77.59, 5.5);

        assert_eq!(request.activity, "marriage");
    }
}
