//! Birth Blueprint Workflow — Core identity mapping
//!
//! Executes: numerology, human-design, vimshottari
//! Synthesizes natal patterns for identity understanding.

use noesis_core::{BirthData, EngineInput};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Birth Blueprint specific data extracted from engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthBlueprintData {
    /// From Numerology
    pub numerology: Option<NumerologyData>,
    /// From Human Design
    pub human_design: Option<HumanDesignData>,
    /// From Vimshottari
    pub vimshottari: Option<VimshottariData>,
}

/// Numerology engine data relevant to synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumerologyData {
    pub life_path: u8,
    pub life_path_name: String,
    pub expression: u8,
    pub expression_name: String,
    pub soul_urge: u8,
    pub soul_urge_name: String,
}

impl NumerologyData {
    /// Extract from engine output JSON
    pub fn from_json(value: &Value) -> Option<Self> {
        let life_path = value.get("life_path")?.as_u64()? as u8;
        let expression = value.get("expression_number")
            .or_else(|| value.get("expression"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;
        let soul_urge = value.get("soul_urge_number")
            .or_else(|| value.get("soul_urge"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;

        Some(Self {
            life_path,
            life_path_name: number_meaning(life_path),
            expression,
            expression_name: number_meaning(expression),
            soul_urge,
            soul_urge_name: number_meaning(soul_urge),
        })
    }

    /// Get key themes from numerology
    pub fn themes(&self) -> Vec<(String, String)> {
        vec![
            (self.life_path_name.clone(), format!("Life Path {}: {}", self.life_path, self.life_path_name)),
            (self.expression_name.clone(), format!("Expression {}: {}", self.expression, self.expression_name)),
            (self.soul_urge_name.clone(), format!("Soul Urge {}: {}", self.soul_urge, self.soul_urge_name)),
        ]
    }
}

/// Human Design engine data relevant to synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanDesignData {
    pub hd_type: String,
    pub authority: String,
    pub profile: String,
    pub defined_centers: Vec<String>,
    pub undefined_centers: Vec<String>,
}

impl HumanDesignData {
    /// Extract from engine output JSON
    pub fn from_json(value: &Value) -> Option<Self> {
        let hd_type = value.get("type")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        
        let authority = value.get("authority")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let profile = value.get("profile")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let defined_centers = value.get("defined_centers")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let undefined_centers = value.get("undefined_centers")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Some(Self {
            hd_type,
            authority,
            profile,
            defined_centers,
            undefined_centers,
        })
    }

    /// Get key themes from Human Design
    pub fn themes(&self) -> Vec<(String, String)> {
        let mut themes = vec![];
        
        // Type-based themes
        match self.hd_type.as_str() {
            "Manifestor" => themes.push(("Leadership".to_string(), "Manifestor: Initiates and impacts".to_string())),
            "Generator" | "Manifesting Generator" => themes.push(("Response".to_string(), "Generator: Life force through response".to_string())),
            "Projector" => themes.push(("Recognition".to_string(), "Projector: Guides through recognition".to_string())),
            "Reflector" => themes.push(("Reflection".to_string(), "Reflector: Mirrors the community".to_string())),
            _ => {}
        }

        // Authority-based themes
        if self.authority.contains("Emotional") || self.authority.contains("Solar Plexus") {
            themes.push(("Emotional Clarity".to_string(), "Emotional Authority: Wait for clarity over time".to_string()));
        } else if self.authority.contains("Sacral") {
            themes.push(("Gut Response".to_string(), "Sacral Authority: Trust gut responses".to_string()));
        } else if self.authority.contains("Splenic") {
            themes.push(("Intuition".to_string(), "Splenic Authority: Instant intuitive knowing".to_string()));
        }

        themes
    }
}

/// Vimshottari Dasha engine data relevant to synthesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VimshottariData {
    pub current_mahadasha: String,
    pub current_mahadasha_lord: String,
    pub current_antardasha: String,
    pub years_remaining: f64,
    pub upcoming_transitions: Vec<String>,
}

impl VimshottariData {
    /// Extract from engine output JSON
    pub fn from_json(value: &Value) -> Option<Self> {
        let current = value.get("current_dasha")?;
        
        let mahadasha = current.get("mahadasha")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let antardasha = current.get("antardasha")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let years_remaining = current.get("years_remaining")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let lord = dasha_lord(&mahadasha);

        let transitions = value.get("upcoming_transitions")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        Some(Self {
            current_mahadasha: mahadasha,
            current_mahadasha_lord: lord,
            current_antardasha: antardasha,
            years_remaining,
            upcoming_transitions: transitions,
        })
    }

    /// Get key themes from current dasha
    pub fn themes(&self) -> Vec<(String, String)> {
        let mut themes = vec![];
        
        // Dasha lord themes
        match self.current_mahadasha_lord.as_str() {
            "Sun" => themes.push(("Leadership".to_string(), "Sun Dasha: Period of authority and visibility".to_string())),
            "Moon" => themes.push(("Emotional Growth".to_string(), "Moon Dasha: Period of emotional development".to_string())),
            "Mars" => themes.push(("Action".to_string(), "Mars Dasha: Period of energy and initiative".to_string())),
            "Mercury" => themes.push(("Communication".to_string(), "Mercury Dasha: Period of learning and expression".to_string())),
            "Jupiter" => themes.push(("Expansion".to_string(), "Jupiter Dasha: Period of growth and wisdom".to_string())),
            "Venus" => themes.push(("Harmony".to_string(), "Venus Dasha: Period of relationships and beauty".to_string())),
            "Saturn" => themes.push(("Discipline".to_string(), "Saturn Dasha: Period of structure and responsibility".to_string())),
            "Rahu" => themes.push(("Transformation".to_string(), "Rahu Dasha: Period of unconventional growth".to_string())),
            "Ketu" => themes.push(("Release".to_string(), "Ketu Dasha: Period of spiritual liberation".to_string())),
            _ => {}
        }

        themes
    }
}

/// Map numerology numbers to meanings
fn number_meaning(n: u8) -> String {
    match n {
        1 => "Leadership".to_string(),
        2 => "Partnership".to_string(),
        3 => "Creativity".to_string(),
        4 => "Foundation".to_string(),
        5 => "Freedom".to_string(),
        6 => "Nurturing".to_string(),
        7 => "Introspection".to_string(),
        8 => "Power".to_string(),
        9 => "Humanitarianism".to_string(),
        11 => "Intuition".to_string(),
        22 => "Master Builder".to_string(),
        33 => "Master Teacher".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Map dasha names to planetary lords
fn dasha_lord(dasha: &str) -> String {
    match dasha.to_lowercase().as_str() {
        "ketu" => "Ketu".to_string(),
        "venus" | "shukra" => "Venus".to_string(),
        "sun" | "surya" => "Sun".to_string(),
        "moon" | "chandra" => "Moon".to_string(),
        "mars" | "mangal" => "Mars".to_string(),
        "rahu" => "Rahu".to_string(),
        "jupiter" | "guru" => "Jupiter".to_string(),
        "saturn" | "shani" => "Saturn".to_string(),
        "mercury" | "budha" => "Mercury".to_string(),
        _ => dasha.to_string(),
    }
}

/// Create input specifically for Birth Blueprint workflow
pub fn create_birth_blueprint_input(birth_data: BirthData) -> EngineInput {
    EngineInput {
        birth_data: Some(birth_data),
        current_time: chrono::Utc::now(),
        location: None,
        precision: noesis_core::Precision::Standard,
        options: std::collections::HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numerology_from_json() {
        let json = json!({
            "life_path": 1,
            "expression_number": 3,
            "soul_urge": 7
        });

        let data = NumerologyData::from_json(&json).unwrap();
        assert_eq!(data.life_path, 1);
        assert_eq!(data.life_path_name, "Leadership");
        assert_eq!(data.expression, 3);
        assert_eq!(data.soul_urge, 7);
    }

    #[test]
    fn human_design_from_json() {
        let json = json!({
            "type": "Generator",
            "authority": "Sacral",
            "profile": "1/3",
            "defined_centers": ["Root", "Sacral"],
            "undefined_centers": ["Head", "Ajna"]
        });

        let data = HumanDesignData::from_json(&json).unwrap();
        assert_eq!(data.hd_type, "Generator");
        assert_eq!(data.authority, "Sacral");
        assert_eq!(data.defined_centers.len(), 2);
    }

    #[test]
    fn vimshottari_from_json() {
        let json = json!({
            "current_dasha": {
                "mahadasha": "Jupiter",
                "antardasha": "Saturn",
                "years_remaining": 5.5
            },
            "upcoming_transitions": ["Saturn Mahadasha in 5.5 years"]
        });

        let data = VimshottariData::from_json(&json).unwrap();
        assert_eq!(data.current_mahadasha, "Jupiter");
        assert_eq!(data.current_mahadasha_lord, "Jupiter");
    }

    #[test]
    fn numerology_themes() {
        let data = NumerologyData {
            life_path: 1,
            life_path_name: "Leadership".to_string(),
            expression: 3,
            expression_name: "Creativity".to_string(),
            soul_urge: 7,
            soul_urge_name: "Introspection".to_string(),
        };

        let themes = data.themes();
        assert_eq!(themes.len(), 3);
    }
}
