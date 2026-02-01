//! Birth Blueprint Synthesis — Cross-reference natal patterns
//!
//! Finds correlations between:
//! - Life Path number ↔ HD Type
//! - Expression number ↔ HD Profile
//! - Current Dasha lord ↔ Active centers
//! - Identifies aligned themes and tensions

use super::Synthesizer;
use crate::workflow::birth_blueprint::{HumanDesignData, NumerologyData, VimshottariData};
use crate::workflow::models::{
    Alignment, SynthesisResult as ExtSynthesisResult, Tension, Theme,
};

// Re-export for trait impl
pub type SynthesisResult = ExtSynthesisResult;
use noesis_core::{EngineInput, EngineOutput};
use std::collections::HashMap;

/// Synthesizer for Birth Blueprint workflow
pub struct BirthBlueprintSynthesizer;

impl Synthesizer for BirthBlueprintSynthesizer {
    fn synthesize(
        results: &HashMap<String, EngineOutput>,
        _input: &EngineInput,
    ) -> SynthesisResult {
        // Extract data from each engine
        let numerology = results
            .get("numerology")
            .and_then(|o| NumerologyData::from_json(&o.result));
        
        let human_design = results
            .get("human-design")
            .and_then(|o| HumanDesignData::from_json(&o.result));
        
        let vimshottari = results
            .get("vimshottari")
            .and_then(|o| VimshottariData::from_json(&o.result));

        let mut alignments = Vec::new();
        let mut tensions = Vec::new();

        // Collect themes from each system
        let mut theme_map: HashMap<String, Theme> = HashMap::new();

        if let Some(ref num) = numerology {
            for (theme_name, description) in num.themes() {
                let theme = theme_map.entry(theme_name.clone()).or_insert_with(|| {
                    Theme::new(theme_name.clone(), description.clone())
                });
                theme.add_source("numerology");
            }
        }

        if let Some(ref hd) = human_design {
            for (theme_name, description) in hd.themes() {
                let theme = theme_map.entry(theme_name.clone()).or_insert_with(|| {
                    Theme::new(theme_name.clone(), description.clone())
                });
                theme.add_source("human-design");
            }
        }

        if let Some(ref vim) = vimshottari {
            for (theme_name, description) in vim.themes() {
                let theme = theme_map.entry(theme_name.clone()).or_insert_with(|| {
                    Theme::new(theme_name.clone(), description.clone())
                });
                theme.add_source("vimshottari");
            }
        }

        // Identify strong themes (appearing in multiple systems)
        let mut themes: Vec<Theme> = theme_map.into_values().collect();
        themes.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap_or(std::cmp::Ordering::Equal));

        // Find specific correlations
        if let (Some(ref num), Some(ref hd)) = (&numerology, &human_design) {
            // Life Path ↔ HD Type correlation
            if let Some(alignment) = correlate_life_path_and_type(num.life_path, &hd.hd_type) {
                alignments.push(alignment);
            }

            // Expression ↔ Profile correlation
            if let Some(alignment) = correlate_expression_and_profile(num.expression, &hd.profile) {
                alignments.push(alignment);
            }

            // Check for tensions
            if let Some(tension) = find_visibility_tension(num, hd) {
                tensions.push(tension);
            }
        }

        // Dasha ↔ Active centers correlation
        if let (Some(ref hd), Some(ref vim)) = (&human_design, &vimshottari) {
            if let Some(alignment) = correlate_dasha_and_centers(vim, hd) {
                alignments.push(alignment);
            }
        }

        // Generate summary
        let summary = generate_summary(&themes, &alignments, &tensions, &numerology, &human_design, &vimshottari);

        SynthesisResult {
            themes,
            alignments,
            tensions,
            summary,
        }
    }
}

/// Correlate Life Path number with HD Type
fn correlate_life_path_and_type(life_path: u8, hd_type: &str) -> Option<Alignment> {
    // Life Path 1, 8 → Manifestor (leadership, authority)
    // Life Path 2, 6 → Projector (guiding, nurturing)
    // Life Path 3, 5 → MG (creative, adaptable)
    // Life Path 4, 7 → Generator (building, depth)
    
    let correlation = match (life_path, hd_type) {
        (1, "Manifestor") | (8, "Manifestor") => {
            Some(("Leadership alignment", "Both systems emphasize initiating and authority"))
        }
        (2, "Projector") | (6, "Projector") => {
            Some(("Guiding alignment", "Both systems emphasize guiding and supporting others"))
        }
        (3, "Manifesting Generator") | (5, "Manifesting Generator") => {
            Some(("Dynamic creativity", "Both systems emphasize creative adaptability"))
        }
        (4, "Generator") | (7, "Generator") => {
            Some(("Deep work alignment", "Both systems emphasize sustained focus and mastery"))
        }
        (9, "Projector") => {
            Some(("Humanitarian guidance", "Both systems point to service through wisdom"))
        }
        _ => None,
    };

    correlation.map(|(aspect, desc)| {
        Alignment::new(aspect, desc)
            .with_engines(vec!["numerology".to_string(), "human-design".to_string()])
            .with_confidence(0.8)
    })
}

/// Correlate Expression number with HD Profile
fn correlate_expression_and_profile(expression: u8, profile: &str) -> Option<Alignment> {
    // Expression 1, 8 → Profile 1/x (investigator, leadership)
    // Expression 3, 5 → Profile x/3 (experimenter)
    // Expression 7 → Profile 2/x (hermit)
    // Expression 6 → Profile 4/x (network, community)

    let profile_lines: Vec<&str> = profile.split('/').collect();
    if profile_lines.len() < 2 {
        return None;
    }

    let correlation = match (expression, profile_lines[0], profile_lines[1]) {
        (1, "1", _) | (8, "1", _) => {
            Some(("Investigative leadership", "Both emphasize research and authority"))
        }
        (3, _, "3") | (5, _, "3") => {
            Some(("Experiential learning", "Both emphasize learning through experience"))
        }
        (7, "2", _) => {
            Some(("Natural wisdom", "Both emphasize inner knowing and retreat"))
        }
        (6, "4", _) => {
            Some(("Community orientation", "Both emphasize networks and relationships"))
        }
        _ => None,
    };

    correlation.map(|(aspect, desc)| {
        Alignment::new(aspect, desc)
            .with_engines(vec!["numerology".to_string(), "human-design".to_string()])
            .with_confidence(0.7)
    })
}

/// Find visibility tensions between introversion/extraversion
fn find_visibility_tension(num: &NumerologyData, hd: &HumanDesignData) -> Option<Tension> {
    // Soul Urge 7 (introspection) vs Manifestor or MG (visibility)
    let needs_visibility = hd.hd_type == "Manifestor" || hd.hd_type == "Manifesting Generator";
    let seeks_introspection = num.soul_urge == 7 || num.life_path == 7;

    if needs_visibility && seeks_introspection {
        Some(
            Tension::new(
                "Visibility vs Introspection",
                "Inner need for solitude meets design for public impact"
            )
            .with_perspectives(
                "numerology",
                "Soul Urge 7 seeks depth, privacy, inner wisdom",
                "human-design",
                format!("{} is designed to initiate and inform", hd.hd_type)
            )
            .with_integration_hint(
                "Consider periods of retreat followed by strategic emergence. \
                Your insights from solitude fuel your public impact."
            )
        )
    } else {
        None
    }
}

/// Correlate current Dasha with active HD centers
fn correlate_dasha_and_centers(vim: &VimshottariData, hd: &HumanDesignData) -> Option<Alignment> {
    // Sun Dasha + Defined G Center → Strong identity period
    // Moon Dasha + Defined Solar Plexus → Emotional development
    // Mercury Dasha + Defined Throat → Communication mastery
    // Jupiter Dasha + Defined Ajna → Wisdom integration

    let correlation = match vim.current_mahadasha_lord.as_str() {
        "Sun" if hd.defined_centers.iter().any(|c| c.contains("G") || c.contains("Identity")) => {
            Some(("Identity activation", "Sun period amplifies your defined sense of self"))
        }
        "Moon" if hd.defined_centers.iter().any(|c| c.contains("Solar") || c.contains("Emotional")) => {
            Some(("Emotional mastery", "Moon period deepens emotional intelligence"))
        }
        "Mercury" if hd.defined_centers.iter().any(|c| c.contains("Throat")) => {
            Some(("Communication power", "Mercury period enhances your natural voice"))
        }
        "Jupiter" if hd.defined_centers.iter().any(|c| c.contains("Ajna") || c.contains("Mind")) => {
            Some(("Wisdom expansion", "Jupiter period expands your mental gifts"))
        }
        "Saturn" if hd.defined_centers.iter().any(|c| c.contains("Root")) => {
            Some(("Grounded discipline", "Saturn period strengthens your foundation"))
        }
        _ => None,
    };

    correlation.map(|(aspect, desc)| {
        Alignment::new(aspect, desc)
            .with_engines(vec!["vimshottari".to_string(), "human-design".to_string()])
            .with_confidence(0.75)
    })
}

/// Generate human-readable summary
fn generate_summary(
    themes: &[Theme],
    alignments: &[Alignment],
    tensions: &[Tension],
    numerology: &Option<NumerologyData>,
    human_design: &Option<HumanDesignData>,
    vimshottari: &Option<VimshottariData>,
) -> String {
    let mut parts = Vec::new();

    // Core identity summary
    if let Some(ref num) = numerology {
        parts.push(format!(
            "Your Life Path {} ({}) guides your journey",
            num.life_path, num.life_path_name
        ));
    }

    if let Some(ref hd) = human_design {
        parts.push(format!(
            "as a {} with {} Authority",
            hd.hd_type, hd.authority
        ));
    }

    if let Some(ref vim) = vimshottari {
        parts.push(format!(
            "currently in {} Dasha ({:.1} years remaining)",
            vim.current_mahadasha, vim.years_remaining
        ));
    }

    // Theme summary
    let strong_themes: Vec<&str> = themes
        .iter()
        .filter(|t| t.strength >= 0.4)
        .take(3)
        .map(|t| t.name.as_str())
        .collect();
    
    if !strong_themes.is_empty() {
        parts.push(format!(
            "Key themes across systems: {}",
            strong_themes.join(", ")
        ));
    }

    // Alignment summary
    if !alignments.is_empty() {
        let alignment_names: Vec<&str> = alignments.iter().map(|a| a.aspect.as_str()).collect();
        parts.push(format!(
            "Systems align on: {}",
            alignment_names.join(", ")
        ));
    }

    // Tension summary
    if !tensions.is_empty() {
        parts.push(format!(
            "Creative tensions to explore: {}",
            tensions.iter().map(|t| t.aspect.as_str()).collect::<Vec<_>>().join(", ")
        ));
    }

    parts.join(". ") + "."
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mock_output(engine_id: &str, result: serde_json::Value) -> EngineOutput {
        EngineOutput {
            engine_id: engine_id.to_string(),
            result,
            witness_prompt: String::new(),
            consciousness_level: 0,
            metadata: noesis_core::CalculationMetadata {
                calculation_time_ms: 1.0,
                backend: "mock".to_string(),
                precision_achieved: "standard".to_string(),
                cached: false,
                timestamp: chrono::Utc::now(),
            },
        }
    }

    #[test]
    fn synthesize_with_all_engines() {
        let mut results = HashMap::new();
        
        results.insert("numerology".to_string(), mock_output("numerology", json!({
            "life_path": 1,
            "expression_number": 8,
            "soul_urge": 7
        })));
        
        results.insert("human-design".to_string(), mock_output("human-design", json!({
            "type": "Manifestor",
            "authority": "Emotional",
            "profile": "1/3",
            "defined_centers": ["G Center", "Throat"],
            "undefined_centers": ["Head", "Ajna"]
        })));
        
        results.insert("vimshottari".to_string(), mock_output("vimshottari", json!({
            "current_dasha": {
                "mahadasha": "Sun",
                "antardasha": "Moon",
                "years_remaining": 4.5
            }
        })));

        let input = EngineInput {
            birth_data: None,
            current_time: chrono::Utc::now(),
            location: None,
            precision: noesis_core::Precision::Standard,
            options: HashMap::new(),
        };

        let synthesis = BirthBlueprintSynthesizer::synthesize(&results, &input);

        // Should have found Leadership theme from both numerology (1) and HD (Manifestor)
        assert!(!synthesis.themes.is_empty());
        let leadership_theme = synthesis.themes.iter().find(|t| t.name == "Leadership");
        assert!(leadership_theme.is_some());
        
        // Should have leadership alignment
        assert!(!synthesis.alignments.is_empty());
        
        // Should have visibility tension (Soul Urge 7 + Manifestor)
        assert!(!synthesis.tensions.is_empty());
        
        // Summary should not be empty
        assert!(!synthesis.summary.is_empty());
    }

    #[test]
    fn synthesize_partial_results() {
        let mut results = HashMap::new();
        
        results.insert("numerology".to_string(), mock_output("numerology", json!({
            "life_path": 3,
            "expression_number": 5,
            "soul_urge": 2
        })));

        let input = EngineInput {
            birth_data: None,
            current_time: chrono::Utc::now(),
            location: None,
            precision: noesis_core::Precision::Standard,
            options: HashMap::new(),
        };

        let synthesis = BirthBlueprintSynthesizer::synthesize(&results, &input);

        // Should still produce themes from numerology alone
        assert!(!synthesis.themes.is_empty());
        assert!(!synthesis.summary.is_empty());
    }
}
