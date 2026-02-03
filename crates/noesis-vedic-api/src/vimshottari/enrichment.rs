//! Vimshottari enrichment helpers backed by wisdom data

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::dasha::{DashaPeriod, DashaPlanet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodInfo {
    pub years: u8,
    pub element: String,
    pub qualities: Vec<String>,
    pub themes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryQualities {
    pub consciousness_lessons: Vec<String>,
    pub optimal_practices: Vec<String>,
    pub challenges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimshottariWisdomData {
    pub periods: HashMap<String, PeriodInfo>,
    pub planetary_qualities: HashMap<String, PlanetaryQualities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashaEnrichment {
    pub planet: DashaPlanet,
    pub element: Option<String>,
    pub qualities: Vec<String>,
    pub themes: Vec<String>,
    pub consciousness_lessons: Vec<String>,
    pub optimal_practices: Vec<String>,
    pub challenges: Vec<String>,
}

static WISDOM_DATA: OnceLock<VimshottariWisdomData> = OnceLock::new();

fn load_wisdom() -> &'static VimshottariWisdomData {
    WISDOM_DATA.get_or_init(|| {
        let raw = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/vimshottari/vimshottari_periods.json"));
        serde_json::from_str(raw).expect("Invalid vimshottari_periods.json")
    })
}

fn planet_key(planet: DashaPlanet) -> &'static str {
    match planet {
        DashaPlanet::Ketu => "Ketu",
        DashaPlanet::Venus => "Venus",
        DashaPlanet::Sun => "Sun",
        DashaPlanet::Moon => "Moon",
        DashaPlanet::Mars => "Mars",
        DashaPlanet::Rahu => "Rahu",
        DashaPlanet::Jupiter => "Jupiter",
        DashaPlanet::Saturn => "Saturn",
        DashaPlanet::Mercury => "Mercury",
    }
}

/// Retrieve base period info for a given planet.
pub fn period_info(planet: DashaPlanet) -> Option<PeriodInfo> {
    let data = load_wisdom();
    data.periods.get(planet_key(planet)).cloned()
}

/// Retrieve planetary qualities for a given planet.
pub fn planetary_qualities(planet: DashaPlanet) -> Option<PlanetaryQualities> {
    let data = load_wisdom();
    data.planetary_qualities.get(planet_key(planet)).cloned()
}

/// Enrich a dasha period with themes, qualities, and practices.
pub fn enrich_period(period: &DashaPeriod) -> DashaEnrichment {
    let info = period_info(period.planet);
    let qualities = planetary_qualities(period.planet);

    DashaEnrichment {
        planet: period.planet,
        element: info.as_ref().map(|p| p.element.clone()),
        qualities: info.as_ref().map(|p| p.qualities.clone()).unwrap_or_default(),
        themes: info.as_ref().map(|p| p.themes.clone()).unwrap_or_default(),
        consciousness_lessons: qualities.as_ref().map(|q| q.consciousness_lessons.clone()).unwrap_or_default(),
        optimal_practices: qualities.as_ref().map(|q| q.optimal_practices.clone()).unwrap_or_default(),
        challenges: qualities.as_ref().map(|q| q.challenges.clone()).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dasha::{DashaLevel, DashaPlanet, DashaPeriod};

    #[test]
    fn test_period_info_loads() {
        let info = period_info(DashaPlanet::Mars).expect("Mars info");
        assert!(info.qualities.iter().any(|q| q.to_lowercase().contains("courage")));
    }

    #[test]
    fn test_enrich_period() {
        let period = DashaPeriod {
            planet: DashaPlanet::Jupiter,
            level: DashaLevel::Mahadasha,
            start_date: "2020-01-01".to_string(),
            end_date: "2036-01-01".to_string(),
            duration_years: 16.0,
            duration_days: 5844,
            sub_periods: None,
        };

        let enriched = enrich_period(&period);
        assert_eq!(enriched.planet, DashaPlanet::Jupiter);
        assert!(!enriched.themes.is_empty());
    }
}
