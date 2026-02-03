//! Hora alarm types

use serde::{Deserialize, Serialize};

/// Planet for hora calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HoraPlanet {
    Sun,
    Moon,
    Mars,
    Mercury,
    Jupiter,
    Venus,
    Saturn,
}

impl HoraPlanet {
    /// Get planet name as string
    pub fn name(&self) -> &'static str {
        match self {
            HoraPlanet::Sun => "Sun",
            HoraPlanet::Moon => "Moon",
            HoraPlanet::Mars => "Mars",
            HoraPlanet::Mercury => "Mercury",
            HoraPlanet::Jupiter => "Jupiter",
            HoraPlanet::Venus => "Venus",
            HoraPlanet::Saturn => "Saturn",
        }
    }
    
    /// Get Chaldean order sequence
    pub fn chaldean_sequence() -> &'static [HoraPlanet] {
        &[
            HoraPlanet::Saturn,
            HoraPlanet::Jupiter,
            HoraPlanet::Mars,
            HoraPlanet::Sun,
            HoraPlanet::Venus,
            HoraPlanet::Mercury,
            HoraPlanet::Moon,
        ]
    }
    
    /// Get day ruler for weekday (0 = Sunday)
    pub fn day_ruler(weekday: u8) -> HoraPlanet {
        match weekday % 7 {
            0 => HoraPlanet::Sun,
            1 => HoraPlanet::Moon,
            2 => HoraPlanet::Mars,
            3 => HoraPlanet::Mercury,
            4 => HoraPlanet::Jupiter,
            5 => HoraPlanet::Venus,
            6 => HoraPlanet::Saturn,
            _ => HoraPlanet::Sun,
        }
    }
    
    /// Get favorable activities for this hora
    pub fn favorable_activities(&self) -> Vec<&'static str> {
        match self {
            HoraPlanet::Sun => vec!["Authority matters", "Government work", "Leadership"],
            HoraPlanet::Moon => vec!["Travel", "Public relations", "Water activities"],
            HoraPlanet::Mars => vec!["Physical activity", "Competition", "Courage"],
            HoraPlanet::Mercury => vec!["Communication", "Commerce", "Learning"],
            HoraPlanet::Jupiter => vec!["Spiritual practices", "Teaching", "Finance"],
            HoraPlanet::Venus => vec!["Arts", "Romance", "Luxury purchases"],
            HoraPlanet::Saturn => vec!["Discipline", "Long-term planning", "Property"],
        }
    }
}

impl std::fmt::Display for HoraPlanet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
