//! Gene Keys data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A single Gene Key with its three frequency levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneKey {
    /// Gene Key number (1-64)
    pub number: u8,

    /// Name of the Gene Key (e.g., "The Creative")
    pub name: String,

    /// Shadow frequency - reactive unconscious pattern
    pub shadow: String,

    /// Gift frequency - constructive conscious expression
    pub gift: String,

    /// Siddhi frequency - transcendent realization
    pub siddhi: String,

    /// Full shadow description (preserved archetypal depth)
    pub shadow_description: String,

    /// Full gift description (preserved archetypal depth)
    pub gift_description: String,

    /// Full siddhi description (preserved archetypal depth)
    pub siddhi_description: String,

    /// Programming partner gate (opposite in wheel)
    pub programming_partner: Option<u8>,

    /// Codon sequence
    pub codon: Option<String>,

    /// Amino acid
    pub amino_acid: Option<String>,

    /// Physiology reference
    pub physiology: Option<String>,

    /// Keywords for this Gene Key
    pub keywords: Vec<String>,

    /// Life theme statement
    pub life_theme: Option<String>,
}

/// Gene Key activation from HD gate mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneKeyActivation {
    /// Gene Key number (1-64, maps 1:1 to HD gate)
    pub key_number: u8,
    /// HD line (1-6), or internal sentinel 0 when gate-only input has no line.
    /// EngineOutput exposes an unavailable line as JSON null.
    pub line: u8,
    /// Source of activation (Personality Sun, Design Earth, etc.)
    pub source: ActivationSource,
    /// Bundled framework record; presence does not establish verified completeness.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gene_key_data: Option<GeneKey>,
}

/// Source of Gene Key activation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivationSource {
    PersonalitySun,
    PersonalityEarth,
    PersonalityMoon,
    PersonalityNorthNode,
    PersonalitySouthNode,
    PersonalityMercury,
    PersonalityVenus,
    PersonalityMars,
    PersonalityJupiter,
    PersonalitySaturn,
    PersonalityUranus,
    PersonalityNeptune,
    PersonalityPluto,
    DesignSun,
    DesignEarth,
    DesignMoon,
    DesignNorthNode,
    DesignSouthNode,
    DesignMercury,
    DesignVenus,
    DesignMars,
    DesignJupiter,
    DesignSaturn,
    DesignUranus,
    DesignNeptune,
    DesignPluto,
    Other(String),
}

impl ActivationSource {
    /// Returns true if this is a personality activation
    pub fn is_personality(&self) -> bool {
        matches!(
            self,
            Self::PersonalitySun
                | Self::PersonalityEarth
                | Self::PersonalityNorthNode
                | Self::PersonalitySouthNode
                | Self::PersonalityMoon
                | Self::PersonalityMercury
                | Self::PersonalityVenus
                | Self::PersonalityMars
                | Self::PersonalityJupiter
                | Self::PersonalitySaturn
                | Self::PersonalityUranus
                | Self::PersonalityNeptune
                | Self::PersonalityPluto
        )
    }

    /// Returns true if this is a design activation
    pub fn is_design(&self) -> bool {
        matches!(
            self,
            Self::DesignSun
                | Self::DesignEarth
                | Self::DesignNorthNode
                | Self::DesignSouthNode
                | Self::DesignMoon
                | Self::DesignMercury
                | Self::DesignVenus
                | Self::DesignMars
                | Self::DesignJupiter
                | Self::DesignSaturn
                | Self::DesignUranus
                | Self::DesignNeptune
                | Self::DesignPluto
        )
    }
}

/// Legacy gate-pair relationships retained for API compatibility.
///
/// These pairs are not the canonical four single-sphere Activation Sequence.
/// Use `GeneKeysChart::activation_spheres` for canonical source bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivationSequence {
    /// Legacy pair: Personality Sun + Personality Earth.
    pub lifes_work: (u8, u8),
    /// Legacy pair: Design Sun + Design Earth.
    pub evolution: (u8, u8),
    /// Legacy pair: Personality Sun + Design Sun.
    pub radiance: (u8, u8),
    /// Legacy pair: Personality Earth + Design Earth.
    pub purpose: (u8, u8),
}

impl ActivationSequence {
    /// Create from HD gate activations
    pub fn from_activations(
        personality_sun: u8,
        personality_earth: u8,
        design_sun: u8,
        design_earth: u8,
    ) -> Self {
        Self {
            lifes_work: (personality_sun, personality_earth),
            evolution: (design_sun, design_earth),
            radiance: (personality_sun, design_sun),
            purpose: (personality_earth, design_earth),
        }
    }
}

/// Core Gene Keys activation chart; this does not certify wisdom completeness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneKeysChart {
    /// Legacy gate-pair relationships; use activation_spheres() for canonical spheres.
    pub activation_sequence: ActivationSequence,

    /// All active Gene Keys from HD gates
    pub active_keys: Vec<GeneKeyActivation>,
}

/// Root JSON structure for archetypes.json
#[derive(Debug, Clone, Deserialize)]
pub struct GeneKeysData {
    pub gene_keys_info: GeneKeysInfo,
    pub gene_keys: HashMap<String, GeneKey>,
}

/// Metadata about the Gene Keys system
#[derive(Debug, Clone, Deserialize)]
pub struct GeneKeysInfo {
    pub name: String,
    pub description: String,
    pub total_keys: u8,
    pub source: String,
    pub sequences: Vec<String>,
}

impl ActivationSource {
    /// Create source from planet and is_design flag
    pub fn from_planet(planet: engine_human_design::Planet, is_design: bool) -> Self {
        use engine_human_design::Planet;
        match (planet, is_design) {
            (Planet::Sun, false) => Self::PersonalitySun,
            (Planet::Earth, false) => Self::PersonalityEarth,
            (Planet::Moon, false) => Self::PersonalityMoon,
            (Planet::NorthNode, false) => Self::PersonalityNorthNode,
            (Planet::SouthNode, false) => Self::PersonalitySouthNode,
            (Planet::Mercury, false) => Self::PersonalityMercury,
            (Planet::Venus, false) => Self::PersonalityVenus,
            (Planet::Mars, false) => Self::PersonalityMars,
            (Planet::Jupiter, false) => Self::PersonalityJupiter,
            (Planet::Saturn, false) => Self::PersonalitySaturn,
            (Planet::Uranus, false) => Self::PersonalityUranus,
            (Planet::Neptune, false) => Self::PersonalityNeptune,
            (Planet::Pluto, false) => Self::PersonalityPluto,
            (Planet::Sun, true) => Self::DesignSun,
            (Planet::Earth, true) => Self::DesignEarth,
            (Planet::Moon, true) => Self::DesignMoon,
            (Planet::NorthNode, true) => Self::DesignNorthNode,
            (Planet::SouthNode, true) => Self::DesignSouthNode,
            (Planet::Mercury, true) => Self::DesignMercury,
            (Planet::Venus, true) => Self::DesignVenus,
            (Planet::Mars, true) => Self::DesignMars,
            (Planet::Jupiter, true) => Self::DesignJupiter,
            (Planet::Saturn, true) => Self::DesignSaturn,
            (Planet::Uranus, true) => Self::DesignUranus,
            (Planet::Neptune, true) => Self::DesignNeptune,
            (Planet::Pluto, true) => Self::DesignPluto,
        }
    }
}

/// A canonical sphere bound to one Human Design activation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalActivationSphere {
    pub key_number: u8,
    /// Preserved HD line, or null when no line was supplied/calculated.
    pub line: Option<u8>,
    pub source: ActivationSource,
}

/// The four single spheres; unavailable source activations remain null.
///
/// Source mapping follows the public Gene Keys planet/sphere contract:
/// https://genekeys.com/docs/what-planets-does-each-sphere-of-the-golden-path-profile-correlate-to/
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalActivationSpheres {
    pub lifes_work: Option<CanonicalActivationSphere>,
    pub evolution: Option<CanonicalActivationSphere>,
    pub radiance: Option<CanonicalActivationSphere>,
    pub purpose: Option<CanonicalActivationSphere>,
}

impl GeneKeysChart {
    /// Bind spheres by activation source, never by legacy pair positions.
    pub fn activation_spheres(&self) -> CanonicalActivationSpheres {
        let sphere = |source: ActivationSource| {
            self.active_keys
                .iter()
                .find(|activation| activation.source == source)
                .map(|activation| CanonicalActivationSphere {
                    key_number: activation.key_number,
                    line: (1..=6)
                        .contains(&activation.line)
                        .then_some(activation.line),
                    source: activation.source.clone(),
                })
        };
        CanonicalActivationSpheres {
            lifes_work: sphere(ActivationSource::PersonalitySun),
            evolution: sphere(ActivationSource::PersonalityEarth),
            radiance: sphere(ActivationSource::DesignSun),
            purpose: sphere(ActivationSource::DesignEarth),
        }
    }
}
