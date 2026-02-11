//! NadaBrahman data structures
//!
//! Models for raga-based sound therapy analysis.

use serde::{Deserialize, Serialize};

/// A Melakarta raga from the 72-raga system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Raga {
    pub number: u32,
    pub name: String,
    pub chakra: u32,
    pub arohanam: Vec<String>,
    pub avarohanam: Vec<String>,
    pub madhyama_type: String,
    pub mood: String,
    pub time_of_day: String,
    pub therapeutic_qualities: Vec<String>,
    pub dosha_affinity: Vec<String>,
    pub rasa: String,
    pub consciousness_level: u8,
}

/// A recommendation of a specific raga with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagaRecommendation {
    pub raga_number: u32,
    pub raga_name: String,
    pub reason: String,
    pub score: f64,
}

/// Chakra-frequency mapping for sound therapy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChakraFrequency {
    pub chakra_name: String,
    pub solfeggio_hz: f64,
    pub binaural_target_hz: f64,
}

/// Time period (prahar) recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PraharRecommendation {
    pub prahar_name: String,
    pub prahar_number: u32,
    pub time_range: String,
    pub primary_raga: RagaRecommendation,
    pub secondary_ragas: Vec<RagaRecommendation>,
    pub dosha_dominance: String,
    pub energy_quality: String,
}

/// Complete NadaBrahman analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NadaBrahmanAnalysis {
    /// Time-of-day raga recommendation
    pub time_recommendation: PraharRecommendation,
    /// Top raga recommendations (combined score from time + mood + dosha)
    pub recommendations: Vec<RagaRecommendation>,
    /// Active chakra frequency for sound therapy
    pub chakra_frequency: Option<ChakraFrequency>,
    /// Dosha-specific recommendation if dosha was provided
    pub dosha_recommendation: Option<String>,
    /// Current mood/rasa mapping if mood was provided
    pub rasa_mapping: Option<String>,
}

/// The eight prahars (3-hour time blocks) of the Indian day
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prahar {
    /// 06:00-09:00
    First,
    /// 09:00-12:00
    Second,
    /// 12:00-15:00
    Third,
    /// 15:00-18:00
    Fourth,
    /// 18:00-21:00
    Fifth,
    /// 21:00-00:00
    Sixth,
    /// 00:00-03:00
    Seventh,
    /// 03:00-06:00
    Eighth,
}

impl Prahar {
    /// Determine prahar from hour of day (0-23)
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            6..=8 => Prahar::First,
            9..=11 => Prahar::Second,
            12..=14 => Prahar::Third,
            15..=17 => Prahar::Fourth,
            18..=20 => Prahar::Fifth,
            21..=23 => Prahar::Sixth,
            0..=2 => Prahar::Seventh,
            3..=5 => Prahar::Eighth,
            _ => Prahar::First, // fallback
        }
    }

    /// Get the prahar number (1-8)
    pub fn number(&self) -> u32 {
        match self {
            Prahar::First => 1,
            Prahar::Second => 2,
            Prahar::Third => 3,
            Prahar::Fourth => 4,
            Prahar::Fifth => 5,
            Prahar::Sixth => 6,
            Prahar::Seventh => 7,
            Prahar::Eighth => 8,
        }
    }

    /// Get the prahar name
    pub fn name(&self) -> &'static str {
        match self {
            Prahar::First => "Pratah (Morning)",
            Prahar::Second => "Sangava (Forenoon)",
            Prahar::Third => "Madhyahna (Midday)",
            Prahar::Fourth => "Aparahna (Afternoon)",
            Prahar::Fifth => "Sayahna (Evening)",
            Prahar::Sixth => "Pradosha (Early Night)",
            Prahar::Seventh => "Nisha (Midnight)",
            Prahar::Eighth => "Brahma Muhurta (Pre-dawn)",
        }
    }

    /// Get the time range string
    pub fn time_range(&self) -> &'static str {
        match self {
            Prahar::First => "06:00-09:00",
            Prahar::Second => "09:00-12:00",
            Prahar::Third => "12:00-15:00",
            Prahar::Fourth => "15:00-18:00",
            Prahar::Fifth => "18:00-21:00",
            Prahar::Sixth => "21:00-00:00",
            Prahar::Seventh => "00:00-03:00",
            Prahar::Eighth => "03:00-06:00",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prahar_from_hour() {
        assert_eq!(Prahar::from_hour(7), Prahar::First);
        assert_eq!(Prahar::from_hour(10), Prahar::Second);
        assert_eq!(Prahar::from_hour(13), Prahar::Third);
        assert_eq!(Prahar::from_hour(16), Prahar::Fourth);
        assert_eq!(Prahar::from_hour(19), Prahar::Fifth);
        assert_eq!(Prahar::from_hour(22), Prahar::Sixth);
        assert_eq!(Prahar::from_hour(1), Prahar::Seventh);
        assert_eq!(Prahar::from_hour(4), Prahar::Eighth);
    }

    #[test]
    fn test_prahar_number() {
        assert_eq!(Prahar::First.number(), 1);
        assert_eq!(Prahar::Eighth.number(), 8);
    }

    #[test]
    fn test_prahar_name() {
        assert!(Prahar::First.name().contains("Morning"));
        assert!(Prahar::Eighth.name().contains("Pre-dawn"));
    }
}
