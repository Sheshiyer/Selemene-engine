//! Self Inquiry Synthesis (W2-S6-02)
//!
//! Synthesizes Gene Keys and Enneagram outputs to map shadow patterns
//! and support deep self-inquiry.
//!
//! # Synthesis Approach
//! Map Gene Keys shadows to Enneagram patterns:
//! - GK Shadow frequency ↔ Enneagram core fear
//! - GK Gift frequency ↔ Enneagram healthy traits
//! - GK Siddhi ↔ Enneagram integration point

use super::Synthesizer;
use crate::workflow::models::{
    Alignment, SynthesisResult, Tension, Theme, WitnessPrompt, InquiryType,
};
use noesis_core::{EngineInput, EngineOutput};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Self Inquiry synthesis implementation
pub struct SelfInquirySynthesis;

impl SelfInquirySynthesis {
    /// Synthesize results from all engines (convenience method without input)
    pub fn synthesize_results(results: &HashMap<String, EngineOutput>) -> SynthesisResult {
        let dummy_input = EngineInput {
            birth_data: None,
            current_time: chrono::Utc::now(),
            location: None,
            precision: noesis_core::Precision::Standard,
            options: HashMap::new(),
        };
        <Self as Synthesizer>::synthesize(results, &dummy_input)
    }
}

impl Synthesizer for SelfInquirySynthesis {
    fn synthesize(
        results: &HashMap<String, EngineOutput>,
        _input: &EngineInput,
    ) -> SynthesisResult {
        let mut themes = Vec::new();
        let mut alignments = Vec::new();
        let mut tensions = Vec::new();

        let gk_data = Self::extract_gene_keys_data(results.get("gene-keys"));
        let enn_data = Self::extract_enneagram_data(results.get("enneagram"));

        themes.extend(Self::extract_shadow_themes(&gk_data));
        themes.extend(Self::extract_enneagram_themes(&enn_data));

        alignments.extend(Self::find_shadow_fear_alignment(&gk_data, &enn_data));
        alignments.extend(Self::find_gift_strength_alignment(&gk_data, &enn_data));
        alignments.extend(Self::find_siddhi_integration_alignment(&gk_data, &enn_data));

        tensions.extend(Self::find_growth_edge_tensions(&gk_data, &enn_data));

        let summary = Self::create_summary(&gk_data, &enn_data, &alignments);

        SynthesisResult {
            themes,
            alignments,
            tensions,
            summary,
        }
    }
}

impl SelfInquirySynthesis {
    /// Generate shadow-work focused witness prompts
    pub fn generate_witness_prompts(gk_data: &Value, enn_data: &Value) -> Vec<WitnessPrompt> {
        let mut prompts = Vec::new();

        if let Some(shadow) = gk_data.get("primary_shadow").and_then(|v| v.as_str()) {
            prompts.push(WitnessPrompt::new(
                format!("Where do you notice '{}' arising in daily life?", shadow.to_lowercase()),
                InquiryType::PatternNoticing,
            ).with_context("Gene Keys Shadow"));
        }

        if let Some(t) = enn_data.get("type").and_then(|v| v.as_i64()) {
            let (fear, _, _) = Self::get_core_pattern(Some(t));
            prompts.push(WitnessPrompt::new(
                format!("What happens in your body when '{}' is triggered?", fear),
                InquiryType::PatternNoticing,
            ).with_context("Enneagram Core Fear"));
        }

        if let Some(weakness) = enn_data.get("core_weakness").and_then(|v| v.as_str()) {
            prompts.push(WitnessPrompt::new(
                format!("Can you observe '{}' without trying to fix it?", weakness.to_lowercase()),
                InquiryType::PerspectiveShift,
            ));
        }

        if let Some(gift) = gk_data.get("primary_gift").and_then(|v| v.as_str()) {
            prompts.push(WitnessPrompt::new(
                format!("When does '{}' naturally emerge without effort?", gift),
                InquiryType::Understanding,
            ).with_context("Gene Keys Gift"));
        }

        prompts.push(WitnessPrompt::new(
            "What part of you is observing these patterns right now?",
            InquiryType::PerspectiveShift,
        ));

        prompts
    }

    fn extract_gene_keys_data(output: Option<&EngineOutput>) -> Value {
        let Some(output) = output else {
            return json!({ "available": false });
        };

        let result = &output.result;
        let spheres = result.get("spheres").cloned().unwrap_or(json!({}));
        let life_work = Self::extract_sphere(&spheres, &["life_work", "lifes_work", "lifeWork"]);
        let evolution = Self::extract_sphere(&spheres, &["evolution"]);
        let radiance = Self::extract_sphere(&spheres, &["radiance"]);
        let purpose = Self::extract_sphere(&spheres, &["purpose"]);

        let shadows: Vec<Value> = [&life_work, &evolution, &radiance, &purpose]
            .iter()
            .filter_map(|s| s.get("shadow").cloned())
            .collect();

        let gifts: Vec<Value> = [&life_work, &evolution, &radiance, &purpose]
            .iter()
            .filter_map(|s| s.get("gift").cloned())
            .collect();

        json!({
            "available": true,
            "spheres": { "life_work": life_work, "evolution": evolution, "radiance": radiance, "purpose": purpose },
            "shadows": shadows,
            "gifts": gifts,
            "primary_shadow": life_work.get("shadow").cloned().unwrap_or(json!(null)),
            "primary_gift": life_work.get("gift").cloned().unwrap_or(json!(null)),
            "primary_siddhi": life_work.get("siddhi").cloned().unwrap_or(json!(null))
        })
    }

    fn extract_sphere(spheres: &Value, keys: &[&str]) -> Value {
        for key in keys {
            if let Some(sphere) = spheres.get(*key) {
                return sphere.clone();
            }
        }
        json!({})
    }

    fn extract_enneagram_data(output: Option<&EngineOutput>) -> Value {
        let Some(output) = output else {
            return json!({ "available": false });
        };

        let result = &output.result;
        let etype = result.get("type").or_else(|| result.get("enneagram_type")).cloned().unwrap_or(json!(null));
        let core_fear = result.get("core_fear").or_else(|| result.get("fear")).cloned().unwrap_or(json!(null));
        let core_desire = result.get("core_desire").or_else(|| result.get("desire")).cloned().unwrap_or(json!(null));
        let core_weakness = result.get("core_weakness").or_else(|| result.get("weakness")).or_else(|| result.get("passion")).cloned().unwrap_or(json!(null));
        let healthy_traits = result.get("healthy_traits").or_else(|| result.get("growth_traits")).cloned().unwrap_or(json!([]));
        let integration = result.get("integration").or_else(|| result.get("integration_point")).cloned();

        let type_num = etype.as_i64().or_else(|| etype.as_str().and_then(|s| s.parse().ok()));

        json!({
            "available": type_num.is_some(),
            "type": type_num,
            "type_name": Self::get_type_name(type_num),
            "core_fear": core_fear,
            "core_desire": core_desire,
            "core_weakness": core_weakness,
            "healthy_traits": healthy_traits,
            "integration": integration
        })
    }

    fn get_type_name(type_num: Option<i64>) -> &'static str {
        match type_num {
            Some(1) => "The Reformer", Some(2) => "The Helper", Some(3) => "The Achiever",
            Some(4) => "The Individualist", Some(5) => "The Investigator", Some(6) => "The Loyalist",
            Some(7) => "The Enthusiast", Some(8) => "The Challenger", Some(9) => "The Peacemaker",
            _ => "Unknown",
        }
    }

    fn get_core_pattern(type_num: Option<i64>) -> (&'static str, &'static str, &'static str) {
        match type_num {
            Some(1) => ("being corrupt/defective", "goodness/integrity", "resentment"),
            Some(2) => ("being unwanted/unloved", "being loved", "pride"),
            Some(3) => ("being worthless", "being valuable", "deceit"),
            Some(4) => ("having no identity", "being unique", "envy"),
            Some(5) => ("being useless/incapable", "being competent", "avarice"),
            Some(6) => ("being without support", "security", "fear/anxiety"),
            Some(7) => ("being trapped in pain", "satisfaction/freedom", "gluttony"),
            Some(8) => ("being controlled/harmed", "self-protection", "lust"),
            Some(9) => ("loss of connection", "inner stability", "sloth"),
            _ => ("unknown", "unknown", "unknown"),
        }
    }

    fn get_shadow_fear_mapping() -> HashMap<&'static str, Vec<i64>> {
        let mut map = HashMap::new();
        map.insert("control", vec![1, 8]);
        map.insert("perfectionism", vec![1, 3]);
        map.insert("rejection", vec![2, 4]);
        map.insert("worthlessness", vec![3, 4]);
        map.insert("melancholy", vec![4]);
        map.insert("envy", vec![4]);
        map.insert("isolation", vec![4, 5]);
        map.insert("fear", vec![5, 6]);
        map.insert("anxiety", vec![6]);
        map.insert("superficiality", vec![7, 3]);
        map.insert("gluttony", vec![7]);
        map.insert("avoidance", vec![7, 9]);
        map.insert("domination", vec![8]);
        map.insert("sloth", vec![9]);
        map.insert("inertia", vec![9]);
        map
    }

    fn extract_shadow_themes(data: &Value) -> Vec<Theme> {
        let mut themes = Vec::new();
        if data.get("available").and_then(|v| v.as_bool()) != Some(true) {
            return themes;
        }

        if let Some(shadow) = data.get("primary_shadow").and_then(|v| v.as_str()) {
            let mut theme = Theme::new(format!("Shadow: {}", shadow), format!("Primary shadow pattern: {}", shadow));
            theme.add_source("gene-keys");
            themes.push(theme);
        }

        if let Some(shadows) = data.get("shadows").and_then(|v| v.as_array()) {
            if shadows.len() > 1 {
                let mut theme = Theme::new("Multiple Shadows", format!("{} shadow frequencies identified", shadows.len()));
                theme.add_source("gene-keys");
                themes.push(theme);
            }
        }
        themes
    }

    fn extract_enneagram_themes(data: &Value) -> Vec<Theme> {
        let mut themes = Vec::new();
        if data.get("available").and_then(|v| v.as_bool()) != Some(true) {
            return themes;
        }

        if let Some(name) = data.get("type_name").and_then(|v| v.as_str()) {
            if name != "Unknown" {
                let mut theme = Theme::new(format!("Enneagram: {}", name), format!("Core type pattern: {}", name));
                theme.add_source("enneagram");
                themes.push(theme);
            }
        }

        if let Some(weakness) = data.get("core_weakness").and_then(|v| v.as_str()) {
            let mut theme = Theme::new(format!("Core Passion: {}", weakness), format!("The passion of {}", weakness));
            theme.add_source("enneagram");
            themes.push(theme);
        }
        themes
    }

    fn find_shadow_fear_alignment(gk: &Value, enn: &Value) -> Vec<Alignment> {
        let mut alignments = Vec::new();
        let type_num = enn.get("type").and_then(|v| v.as_i64());
        let shadows_text = gk.to_string().to_lowercase();
        
        if let Some(t) = type_num {
            let mapping = Self::get_shadow_fear_mapping();
            for (shadow_key, types) in &mapping {
                if shadows_text.contains(shadow_key) && types.contains(&t) {
                    let (fear, _, _) = Self::get_core_pattern(Some(t));
                    alignments.push(
                        Alignment::new(
                            "Shadow-Fear Resonance",
                            format!("Gene Keys shadow of '{}' resonates with Type {} fear of {}", shadow_key, t, fear)
                        )
                        .with_engines(vec!["gene-keys".into(), "enneagram".into()])
                        .with_confidence(0.75)
                    );
                }
            }
        }
        alignments
    }

    fn find_gift_strength_alignment(gk: &Value, enn: &Value) -> Vec<Alignment> {
        let mut alignments = Vec::new();
        let gift = gk.get("primary_gift").and_then(|v| v.as_str());
        let healthy = enn.get("healthy_traits");

        if let (Some(g), Some(h)) = (gift, healthy) {
            let gift_lower = g.to_lowercase();
            let healthy_text = h.to_string().to_lowercase();
            let common_themes = ["compassion", "wisdom", "presence", "authenticity", "courage", "peace", "clarity", "love", "truth", "acceptance"];

            for theme in common_themes {
                if gift_lower.contains(theme) && healthy_text.contains(theme) {
                    alignments.push(
                        Alignment::new(
                            format!("Shared Quality: {}", theme),
                            format!("Both systems point to {} as a growth quality", theme)
                        )
                        .with_engines(vec!["gene-keys".into(), "enneagram".into()])
                        .with_confidence(0.7)
                    );
                }
            }
        }
        alignments
    }

    fn find_siddhi_integration_alignment(gk: &Value, enn: &Value) -> Vec<Alignment> {
        let mut alignments = Vec::new();
        let siddhi = gk.get("primary_siddhi").and_then(|v| v.as_str());
        let integration = enn.get("integration");

        if let (Some(s), Some(_i)) = (siddhi, integration) {
            alignments.push(
                Alignment::new(
                    "Highest Potential",
                    format!("GK Siddhi '{}' represents highest frequency, aligns with Enneagram integration direction", s)
                )
                .with_engines(vec!["gene-keys".into(), "enneagram".into()])
                .with_confidence(0.65)
            );
        }
        alignments
    }

    fn find_growth_edge_tensions(gk: &Value, enn: &Value) -> Vec<Tension> {
        let mut tensions = Vec::new();
        let shadow = gk.get("primary_shadow").and_then(|v| v.as_str());
        let gift = gk.get("primary_gift").and_then(|v| v.as_str());
        let type_num = enn.get("type").and_then(|v| v.as_i64());

        if let (Some(s), Some(g)) = (shadow, gift) {
            tensions.push(
                Tension::new(
                    "Shadow-Gift Journey",
                    format!("The transformation from {} to {}", s, g)
                )
                .with_perspectives("gene-keys", s, "gene-keys", g)
                .with_integration_hint(format!(
                    "The shadow of '{}' and the gift of '{}' are not opposites—they are \
                     frequency expressions of the same energy. What might it mean to fully embrace both?", s, g
                ))
            );
        }

        if let Some(t) = type_num {
            let (fear, desire, _) = Self::get_core_pattern(Some(t));
            tensions.push(
                Tension::new(
                    "Core Type Tension",
                    format!("Type {} fundamental polarity", t)
                )
                .with_perspectives("enneagram", fear, "enneagram", desire)
                .with_integration_hint(format!(
                    "The fear of '{}' and desire for '{}' create a fundamental tension. \
                     What happens when you simply witness both without trying to resolve them?", fear, desire
                ))
            );
        }
        tensions
    }

    fn create_summary(gk: &Value, enn: &Value, alignments: &[Alignment]) -> String {
        let gk_available = gk.get("available").and_then(|v| v.as_bool()) == Some(true);
        let enn_available = enn.get("available").and_then(|v| v.as_bool()) == Some(true);

        if !gk_available && !enn_available {
            return "Awaiting engine outputs for synthesis".to_string();
        }

        let mut parts = Vec::new();

        if let Some(shadow) = gk.get("primary_shadow").and_then(|v| v.as_str()) {
            if let Some(gift) = gk.get("primary_gift").and_then(|v| v.as_str()) {
                parts.push(format!("Gene Keys reveals the {} → {} frequency journey", shadow, gift));
            }
        }

        if let Some(name) = enn.get("type_name").and_then(|v| v.as_str()) {
            if name != "Unknown" {
                parts.push(format!("Enneagram {} patterns offer complementary insight", name));
            }
        }

        if !alignments.is_empty() {
            parts.push(format!("{} alignments found between systems", alignments.len()));
        }

        if parts.is_empty() {
            "Partial synthesis available—additional engine outputs will enrich the inquiry".to_string()
        } else {
            format!("{}. These maps are invitations to witness, not prescriptions to follow.", parts.join(". "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use noesis_core::CalculationMetadata;

    fn mock_gene_keys_output() -> EngineOutput {
        EngineOutput {
            engine_id: "gene-keys".to_string(),
            result: json!({
                "spheres": {
                    "life_work": { "gene_key": 55, "shadow": "Victimization", "gift": "Freedom", "siddhi": "Freedom" },
                    "evolution": { "gene_key": 59, "shadow": "Dishonesty", "gift": "Intimacy", "siddhi": "Transparency" }
                }
            }),
            witness_prompt: "What frequency are you operating from?".to_string(),
            consciousness_level: 2,
            metadata: CalculationMetadata {
                calculation_time_ms: 25.0, backend: "native".to_string(),
                precision_achieved: "standard".to_string(), cached: false, timestamp: Utc::now(),
            },
        }
    }

    fn mock_enneagram_output() -> EngineOutput {
        EngineOutput {
            engine_id: "enneagram".to_string(),
            result: json!({
                "type": 4, "core_fear": "having no identity",
                "core_weakness": "envy", "healthy_traits": ["creative", "authentic"],
                "integration": 1
            }),
            witness_prompt: "What is your true identity?".to_string(),
            consciousness_level: 1,
            metadata: CalculationMetadata {
                calculation_time_ms: 12.0, backend: "bridge".to_string(),
                precision_achieved: "standard".to_string(), cached: false, timestamp: Utc::now(),
            },
        }
    }

    fn test_input() -> EngineInput {
        EngineInput {
            birth_data: None, current_time: Utc::now(), location: None,
            precision: noesis_core::Precision::Standard, options: HashMap::new(),
        }
    }

    #[test]
    fn test_synthesis_with_both_engines() {
        let mut results = HashMap::new();
        results.insert("gene-keys".to_string(), mock_gene_keys_output());
        results.insert("enneagram".to_string(), mock_enneagram_output());

        let synthesis = SelfInquirySynthesis::synthesize(&results, &test_input());
        assert!(!synthesis.themes.is_empty());
        assert!(!synthesis.summary.is_empty());
    }

    #[test]
    fn test_type_names() {
        assert_eq!(SelfInquirySynthesis::get_type_name(Some(1)), "The Reformer");
        assert_eq!(SelfInquirySynthesis::get_type_name(Some(4)), "The Individualist");
    }
}
