//! Biofield data structures
//!
//! Models for biofield analysis from PIP (Polycontrast Interference Photography)
//! devices. Currently returns mock data - full implementation requires hardware.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Core biofield metrics from PIP analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiofieldMetrics {
    /// Fractal dimension of biofield pattern (1.0-2.0 range)
    /// Higher values indicate more complex, healthy patterns
    pub fractal_dimension: f64,

    /// Shannon entropy of color distribution (0.0-1.0)
    /// Balanced entropy suggests stable energy patterns
    pub entropy: f64,

    /// Coherence measure from interference patterns (0.0-1.0)
    /// Higher coherence indicates aligned energy flow
    pub coherence: f64,

    /// Left-right symmetry of biofield (0.0-1.0)
    /// Higher values indicate balanced energy distribution
    pub symmetry: f64,

    /// Composite vitality index (calculated from other metrics)
    pub vitality_index: f64,

    /// Individual chakra energy readings
    pub chakra_readings: Vec<ChakraReading>,

    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
}

/// Individual chakra energy reading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChakraReading {
    /// Which chakra this reading is for
    pub chakra: Chakra,

    /// Activity level (0.0-1.0)
    /// Higher values indicate more energy flow
    pub activity_level: f64,

    /// Left-right balance (-1.0 to 1.0)
    /// Negative = left dominant, Positive = right dominant
    pub balance: f64,

    /// Dominant color intensity observed
    pub color_intensity: String,
}

/// The seven primary chakras
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chakra {
    Root,
    Sacral,
    SolarPlexus,
    Heart,
    Throat,
    ThirdEye,
    Crown,
}

impl Chakra {
    /// Get all chakras in order from Root to Crown
    pub fn all() -> Vec<Chakra> {
        vec![
            Chakra::Root,
            Chakra::Sacral,
            Chakra::SolarPlexus,
            Chakra::Heart,
            Chakra::Throat,
            Chakra::ThirdEye,
            Chakra::Crown,
        ]
    }

    /// Get the chakra name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Chakra::Root => "Root",
            Chakra::Sacral => "Sacral",
            Chakra::SolarPlexus => "Solar Plexus",
            Chakra::Heart => "Heart",
            Chakra::Throat => "Throat",
            Chakra::ThirdEye => "Third Eye",
            Chakra::Crown => "Crown",
        }
    }
}

/// Complete biofield analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiofieldAnalysis {
    /// Core biofield metrics
    pub metrics: BiofieldMetrics,

    /// Human-readable interpretation of the metrics
    pub interpretation: String,

    /// Areas that may benefit from attention
    pub areas_of_attention: Vec<String>,

    /// Always true for stub implementation
    /// Full implementation with PIP hardware will set this to false
    pub is_mock_data: bool,

    // T-026 P2 start: capture result mapping to frozen contracts (11 metrics + consent)
    // refs: p1-w1-worker-bootstrap-packet.md + resources-and-assets.md + gaps-and-improvements.md + goal-understanding.md + P1W1-CONTRACTS-FROZEN.md (11 metrics from py sidecar + sankalpa) + detailed-task-list T-026 + EXECUTION-STATUS + noesis-core frozen media types. Keep is_mock for now.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chakra_all() {
        let chakras = Chakra::all();
        assert_eq!(chakras.len(), 7);
        assert_eq!(chakras[0], Chakra::Root);
        assert_eq!(chakras[6], Chakra::Crown);
    }

    #[test]
    fn test_chakra_names() {
        assert_eq!(Chakra::Root.name(), "Root");
        assert_eq!(Chakra::SolarPlexus.name(), "Solar Plexus");
        assert_eq!(Chakra::ThirdEye.name(), "Third Eye");
    }
}
