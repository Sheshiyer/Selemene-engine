//! Vimshottari Dasha data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Complete Vimshottari chart timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimshottariChart {
    pub birth_date: DateTime<Utc>,
    pub mahadashas: Vec<Mahadasha>,
    pub current_period: CurrentPeriod,
    pub upcoming_transitions: Vec<Transition>,
}

/// Major planetary period (Mahadasha)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mahadasha {
    pub planet: VedicPlanet,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration_years: f64,
    pub antardashas: Vec<Antardasha>,
    pub qualities: PlanetaryQualities,
}

/// Sub-period within a Mahadasha (Antardasha)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Antardasha {
    pub planet: VedicPlanet,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration_years: f64,
    pub pratyantardashas: Vec<Pratyantardasha>,
}

/// Sub-sub-period within an Antardasha (Pratyantardasha)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pratyantardasha {
    pub planet: VedicPlanet,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub duration_days: f64,
}

/// Vedic planetary system (9 planets)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VedicPlanet {
    Sun,
    Moon,
    Mars,
    Rahu,    // North Node
    Jupiter,
    Saturn,
    Mercury,
    Ketu,    // South Node
    Venus,
}

impl VedicPlanet {
    /// Get the Mahadasha period duration in years
    pub fn period_years(&self) -> u8 {
        match self {
            VedicPlanet::Sun => 6,
            VedicPlanet::Moon => 10,
            VedicPlanet::Mars => 7,
            VedicPlanet::Rahu => 18,
            VedicPlanet::Jupiter => 16,
            VedicPlanet::Saturn => 19,
            VedicPlanet::Mercury => 17,
            VedicPlanet::Ketu => 7,
            VedicPlanet::Venus => 20,
        }
    }

    /// Get the next planet in the Vimshottari cycle
    pub fn next_planet(&self) -> VedicPlanet {
        match self {
            VedicPlanet::Sun => VedicPlanet::Moon,
            VedicPlanet::Moon => VedicPlanet::Mars,
            VedicPlanet::Mars => VedicPlanet::Rahu,
            VedicPlanet::Rahu => VedicPlanet::Jupiter,
            VedicPlanet::Jupiter => VedicPlanet::Saturn,
            VedicPlanet::Saturn => VedicPlanet::Mercury,
            VedicPlanet::Mercury => VedicPlanet::Ketu,
            VedicPlanet::Ketu => VedicPlanet::Venus,
            VedicPlanet::Venus => VedicPlanet::Sun,
        }
    }

    /// Get planet name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            VedicPlanet::Sun => "Sun",
            VedicPlanet::Moon => "Moon",
            VedicPlanet::Mars => "Mars",
            VedicPlanet::Rahu => "Rahu",
            VedicPlanet::Jupiter => "Jupiter",
            VedicPlanet::Saturn => "Saturn",
            VedicPlanet::Mercury => "Mercury",
            VedicPlanet::Ketu => "Ketu",
            VedicPlanet::Venus => "Venus",
        }
    }

    /// Parse planet from string
    pub fn from_str(s: &str) -> Option<VedicPlanet> {
        match s {
            "Sun" => Some(VedicPlanet::Sun),
            "Moon" => Some(VedicPlanet::Moon),
            "Mars" => Some(VedicPlanet::Mars),
            "Rahu" => Some(VedicPlanet::Rahu),
            "Jupiter" => Some(VedicPlanet::Jupiter),
            "Saturn" => Some(VedicPlanet::Saturn),
            "Mercury" => Some(VedicPlanet::Mercury),
            "Ketu" => Some(VedicPlanet::Ketu),
            "Venus" => Some(VedicPlanet::Venus),
            _ => None,
        }
    }
}

/// Nakshatra (lunar mansion)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nakshatra {
    pub number: u8,
    pub name: String,
    pub ruling_planet: VedicPlanet,
    pub start_degree: f64,
    pub end_degree: f64,
    pub deity: String,
    pub symbol: String,
    pub qualities: Vec<String>,
    pub description: String,
}

/// Currently active periods with full details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentPeriod {
    pub mahadasha: CurrentMahadasha,
    pub antardasha: CurrentAntardasha,
    pub pratyantardasha: CurrentPratyantardasha,
    pub current_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentMahadasha {
    pub planet: VedicPlanet,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub years: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentAntardasha {
    pub planet: VedicPlanet,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub years: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentPratyantardasha {
    pub planet: VedicPlanet,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub days: f64,
}

/// Planetary qualities and themes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryQualities {
    pub themes: Vec<String>,
    pub qualities: Vec<String>,
    pub element: String,
    pub description: String,
    pub consciousness_lessons: Vec<String>,
    pub optimal_practices: Vec<String>,
    pub challenges: Vec<String>,
}

/// Period transition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub from_planet: VedicPlanet,
    pub to_planet: VedicPlanet,
    pub date: DateTime<Utc>,
    pub level: TransitionLevel,
}

/// Level of period transition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionLevel {
    Mahadasha,
    Antardasha,
    Pratyantardasha,
}

/// Transition type alias for backwards compatibility
pub type TransitionType = TransitionLevel;

/// Upcoming transition with time-until information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingTransition {
    pub transition_type: TransitionType,
    pub from_planet: VedicPlanet,
    pub to_planet: VedicPlanet,
    pub transition_date: DateTime<Utc>,
    pub days_until: i64,
}

/// Planetary period qualities for enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetaryPeriodQualities {
    pub planet: VedicPlanet,
    pub themes: Vec<String>,
    pub life_areas: Vec<String>,
    pub challenges: Vec<String>,
    pub opportunities: Vec<String>,
    pub description: String,
}

/// Enriched period information combining all three levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodEnrichment {
    pub mahadasha_themes: Vec<String>,
    pub antardasha_themes: Vec<String>,
    pub pratyantardasha_themes: Vec<String>,
    pub combined_description: String,
    pub life_areas: Vec<String>,
    pub opportunities: Vec<String>,
    pub challenges: Vec<String>,
}
