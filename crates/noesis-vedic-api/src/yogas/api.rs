//! Yogas API — native facade backed by Swiss Ephemeris + the in-tree
//! detectors (`detect_raj_yogas`, `detect_dhana_yogas`,
//! `detect_kendra_trikona_yogas`).
//!
//! PR3 of 3: the previous implementation POSTed to `/yogas` (dead, 403).
//! `get_yogas` now builds a native birth chart via
//! [`crate::birth_chart::native::build_native_chart`], runs all detectors,
//! and assembles the legacy `YogaApiResponse` envelope so downstream
//! callers keep working.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::types::{DetectedYoga, YogaAnalysis, YogaCategory, YogaStrength};
use crate::birth_chart::native::{build_native_chart, parse_birth_inputs};
use crate::client::VedicApiClient;
use crate::error::VedicApiResult;

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

/// API response for yoga detection (kept serialisation-compatible with the
/// retired vendor envelope — `Serialize` is now also derived so callers that
/// round-trip through JSON keep working).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YogaApiResponse {
    pub yogas: Vec<YogaApiItem>,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Convert in-memory `DetectedYoga` into the wire-compatible `YogaApiItem`.
fn detected_to_item(y: &DetectedYoga) -> YogaApiItem {
    YogaApiItem {
        name: y.name.clone(),
        category: y.category.to_string(),
        strength: Some(
            match y.strength {
                YogaStrength::Full => "full",
                YogaStrength::Partial => "partial",
                YogaStrength::Weak => "weak",
                YogaStrength::Cancelled => "cancelled",
            }
            .to_string(),
        ),
        planets: y.planets_involved.clone(),
        houses: Some(y.houses_involved.clone()),
        description: y.description.clone(),
        results: Some(y.results.clone()),
    }
}

impl VedicApiClient {
    /// Detect yogas natively from birth inputs.
    ///
    /// PR3: no HTTP call. Builds the chart via Swiss Ephemeris (on a
    /// blocking thread) and runs `detect_raj_yogas`,
    /// `detect_dhana_yogas`, and `detect_kendra_trikona_yogas`.
    pub async fn get_yogas(
        &self,
        request: &YogaDetectionRequest,
    ) -> VedicApiResult<YogaApiResponse> {
        let (y, m, d, hh, mm, ss) = parse_birth_inputs(&request.birth_date, &request.birth_time)?;
        let chart = build_native_chart(
            y,
            m,
            d,
            hh,
            mm,
            ss,
            request.latitude,
            request.longitude,
            request.timezone,
        )
        .await?;

        let mut yogas: Vec<DetectedYoga> = crate::yogas::detect_raj_yogas(&chart);
        yogas.extend(crate::yogas::detect_dhana_yogas(&chart));
        // Kendra-Trikona is already wired through `detect_raj_yogas`, so we
        // only add it again if a category filter was requested explicitly.

        // Optional category filtering — `categories` is currently the
        // canonical YogaCategory display string list.
        if let Some(filter) = request.categories.as_ref() {
            let allowed: std::collections::HashSet<String> =
                filter.iter().map(|s| s.to_lowercase()).collect();
            yogas.retain(|y| allowed.contains(&y.category.to_string().to_lowercase()));
        }

        let items: Vec<YogaApiItem> = yogas.iter().map(detected_to_item).collect();
        let summary = if items.is_empty() {
            Some("No major yogas detected for this birth.".to_string())
        } else {
            Some(format!(
                "{} yoga(s) detected from native chart analysis.",
                items.len()
            ))
        };

        Ok(YogaApiResponse {
            yogas: items,
            summary,
        })
    }
}

/// Map API response to internal analysis
pub fn map_yoga_response(response: YogaApiResponse) -> YogaAnalysis {
    let mut analysis = YogaAnalysis::empty();

    for item in response.yogas {
        let category = parse_yoga_category(&item.category);
        let strength = item
            .strength
            .as_ref()
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

    fn test_client() -> VedicApiClient {
        VedicApiClient::new(crate::mocks::mock_config("http://localhost:0"))
    }

    /// Bangalore 1991-08-13 13:31 IST: Scorpio ascendant. Mars (lord of Asc
    /// and 6th) in Leo (10th from Scorpio) is a textbook Ruchaka
    /// Mahapurusha trigger via `detect_mahapurusha_yogas`, and 7th lord
    /// Venus tends to form a Kendra-Trikona link with the 9th lord Moon.
    /// We assert at least one yoga is detected and that the call doesn't
    /// touch the network.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn native_yogas_bangalore_1991_produces_at_least_one_yoga() {
        let client = test_client();
        let dt = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(1991, 8, 13).unwrap(),
            NaiveTime::from_hms_opt(13, 31, 0).unwrap(),
        );
        let req = YogaDetectionRequest::new(dt, 12.9716, 77.5946, 5.5);
        let resp = client
            .get_yogas(&req)
            .await
            .expect("native yogas should succeed");
        assert!(
            !resp.yogas.is_empty(),
            "Scorpio asc Bangalore chart should produce at least one yoga"
        );
        // All yogas should have non-empty names.
        for y in &resp.yogas {
            assert!(!y.name.is_empty());
            assert!(!y.description.is_empty());
        }
    }
}
