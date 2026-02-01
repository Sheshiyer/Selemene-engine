//! Creative Expression Synthesis (W2-S6-04)
//!
//! Synthesizes Sigil Forge and Sacred Geometry outputs to provide
//! combined creative direction and inspiration.

use super::Synthesizer;
use crate::workflow::models::{
    Alignment, SynthesisResult, Tension, Theme, WitnessPrompt, InquiryType,
};
use noesis_core::{EngineInput, EngineOutput};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Creative Expression synthesis implementation
pub struct CreativeExpressionSynthesis;

impl CreativeExpressionSynthesis {
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

impl Synthesizer for CreativeExpressionSynthesis {
    fn synthesize(
        results: &HashMap<String, EngineOutput>,
        _input: &EngineInput,
    ) -> SynthesisResult {
        let mut themes = Vec::new();
        let mut alignments = Vec::new();
        let mut tensions = Vec::new();

        let sigil_data = Self::extract_sigil_data(results.get("sigil-forge"));
        let geo_data = Self::extract_geometry_data(results.get("sacred-geometry"));

        themes.extend(Self::extract_sigil_themes(&sigil_data));
        themes.extend(Self::extract_geometry_themes(&geo_data));

        alignments.extend(Self::find_intention_form_alignment(&sigil_data, &geo_data));
        tensions.extend(Self::find_creative_tensions(&sigil_data, &geo_data));

        let summary = Self::create_summary(&sigil_data, &geo_data, &themes);

        SynthesisResult {
            themes,
            alignments,
            tensions,
            summary,
        }
    }
}

impl CreativeExpressionSynthesis {
    /// Generate witness prompts for creative expression
    pub fn generate_witness_prompts(sigil_data: &Value, geo_data: &Value) -> Vec<WitnessPrompt> {
        let mut prompts = Vec::new();

        let intention = sigil_data.get("distilled")
            .or_else(|| sigil_data.get("intention"))
            .and_then(|v| v.as_str())
            .unwrap_or("your intention");
        
        let form = geo_data.get("form").and_then(|v| v.as_str()).unwrap_or("the sacred form");

        prompts.push(WitnessPrompt::new(
            format!("What emerges when you hold '{}' while contemplating {}?", intention, form),
            InquiryType::Integration,
        ));

        prompts.push(WitnessPrompt::new(
            "Where does this creative impulse want to flow?",
            InquiryType::Understanding,
        ));

        prompts.push(WitnessPrompt::new(
            "What would it look like to express this combined theme in your medium?",
            InquiryType::Integration,
        ));

        prompts.push(WitnessPrompt::new(
            "Where do you feel creative energy stirring in your body?",
            InquiryType::PatternNoticing,
        ));

        prompts.push(WitnessPrompt::new(
            "What wants to be created through you, rather than by you?",
            InquiryType::PerspectiveShift,
        ));

        prompts
    }

    /// Generate creative direction from combined data
    pub fn generate_creative_direction(sigil_data: &Value, geo_data: &Value) -> Value {
        let intention = sigil_data.get("distilled")
            .or_else(|| sigil_data.get("intention"))
            .and_then(|v| v.as_str())
            .unwrap_or("your intention");
        
        let form = geo_data.get("form").and_then(|v| v.as_str()).unwrap_or("sacred form");
        let description = geo_data.get("visual_description").and_then(|v| v.as_str()).unwrap_or("");
        
        json!({
            "core_direction": format!("Channel '{}' through the lens of the {}", intention, form),
            "visual_anchor": description,
            "creative_prompt": format!("If {} were a visual expression contained within {}, what would emerge?", intention, form),
            "suggested_approach": Self::suggest_creative_approach(sigil_data, geo_data)
        })
    }

    fn suggest_creative_approach(sigil: &Value, geo: &Value) -> Vec<String> {
        let mut approaches = Vec::new();
        let energy = sigil.get("energy").and_then(|v| v.as_str()).unwrap_or("");
        let form = geo.get("form").and_then(|v| v.as_str()).unwrap_or("");

        match energy {
            "expansive" => approaches.push("Work outward from center, allowing forms to multiply and grow".to_string()),
            "concentrating" => approaches.push("Distill and refine, reducing to essential elements".to_string()),
            "flowing" => approaches.push("Allow continuous movement, embrace impermanence in form".to_string()),
            "protective" => approaches.push("Create boundaries and containers, define sacred space".to_string()),
            "transformative" => approaches.push("Work with before/after, thresholds, and liminal spaces".to_string()),
            "restorative" => approaches.push("Focus on harmony, balance, and natural proportions".to_string()),
            _ => approaches.push("Begin with emptiness, allow the form to emerge".to_string()),
        }

        if form.to_lowercase().contains("spiral") || form.to_lowercase().contains("fibonacci") {
            approaches.push("Work with the golden ratio: 1:1.618 proportions".to_string());
        }
        if form.to_lowercase().contains("flower") || form.to_lowercase().contains("seed") {
            approaches.push("Use overlapping circles as your foundational structure".to_string());
        }

        approaches
    }

    fn extract_sigil_data(output: Option<&EngineOutput>) -> Value {
        let Some(output) = output else {
            return json!({ "available": false });
        };

        let result = &output.result;
        let intention = result.get("intention").or_else(|| result.get("original_intention")).cloned().unwrap_or(json!(""));
        let distilled = result.get("distilled").or_else(|| result.get("distilled_intention")).or_else(|| result.get("essence")).cloned().unwrap_or(json!(""));
        let method = result.get("method").cloned().unwrap_or(json!("unknown"));
        
        let keywords = Self::extract_keywords(&intention, &distilled);
        let energy = Self::determine_energy_quality(&intention);

        json!({
            "available": true,
            "intention": intention,
            "distilled": distilled,
            "method": method,
            "keywords": keywords,
            "energy": energy
        })
    }

    fn extract_keywords(intention: &Value, distilled: &Value) -> Vec<String> {
        let mut keywords = Vec::new();
        let text = format!("{} {}", intention.as_str().unwrap_or(""), distilled.as_str().unwrap_or("")).to_lowercase();

        let creative_words = ["create", "manifest", "express", "flow", "inspire", "transform", "birth", "grow", "expand", "connect", "illuminate", "reveal", "awaken", "heal", "balance"];
        for word in creative_words {
            if text.contains(word) {
                keywords.push(word.to_string());
            }
        }
        keywords
    }

    fn determine_energy_quality(intention: &Value) -> &'static str {
        let text = intention.as_str().unwrap_or("").to_lowercase();
        
        if text.contains("expand") || text.contains("grow") || text.contains("increase") { "expansive" }
        else if text.contains("focus") || text.contains("concentrate") || text.contains("distill") { "concentrating" }
        else if text.contains("flow") || text.contains("release") || text.contains("let go") { "flowing" }
        else if text.contains("protect") || text.contains("shield") || text.contains("boundary") { "protective" }
        else if text.contains("transform") || text.contains("change") || text.contains("shift") { "transformative" }
        else if text.contains("heal") || text.contains("restore") || text.contains("balance") { "restorative" }
        else { "generative" }
    }

    fn extract_geometry_data(output: Option<&EngineOutput>) -> Value {
        let Some(output) = output else {
            return json!({ "available": false });
        };

        let result = &output.result;
        let form = result.get("form").or_else(|| result.get("selected_form")).or_else(|| result.get("geometry")).cloned().unwrap_or(json!("unknown"));
        let qualities = result.get("qualities").or_else(|| result.get("properties")).cloned().unwrap_or(json!([]));
        let meditation = result.get("meditation").or_else(|| result.get("contemplation")).cloned();

        json!({
            "available": true,
            "form": form,
            "qualities": qualities,
            "meditation": meditation,
            "visual_description": Self::get_form_description(&form)
        })
    }

    fn get_form_description(form: &Value) -> &'static str {
        let form_str = form.as_str().unwrap_or("").to_lowercase();
        
        match form_str.as_str() {
            s if s.contains("circle") => "A single, perfect circle representing wholeness and unity",
            s if s.contains("vesica") => "Two overlapping circles creating an almond-shaped intersection",
            s if s.contains("seed") => "Seven circles arranged in perfect hexagonal symmetry",
            s if s.contains("flower") => "Nineteen overlapping circles in perfect rotational harmony",
            s if s.contains("metatron") => "Thirteen circles with all interconnecting lines revealed",
            s if s.contains("sri") || s.contains("yantra") => "Nine interlocking triangles forming 43 smaller triangles",
            s if s.contains("torus") => "A donut-shaped field of continuous energy flow",
            s if s.contains("fibonacci") || s.contains("spiral") => "A spiral expanding in golden ratio proportions",
            s if s.contains("merkaba") => "Two interlocking tetrahedra forming a three-dimensional star",
            _ => "A sacred geometric form with mathematical precision and symbolic meaning"
        }
    }

    fn extract_sigil_themes(data: &Value) -> Vec<Theme> {
        let mut themes = Vec::new();
        if data.get("available").and_then(|v| v.as_bool()) != Some(true) {
            return themes;
        }

        if let Some(distilled) = data.get("distilled").and_then(|v| v.as_str()) {
            if !distilled.is_empty() {
                let mut theme = Theme::new(format!("Distilled Intention: {}", distilled), "Core intention condensed to essence");
                theme.add_source("sigil-forge");
                themes.push(theme);
            }
        }

        if let Some(energy) = data.get("energy").and_then(|v| v.as_str()) {
            let mut theme = Theme::new(format!("Energy Quality: {}", energy), format!("The {} nature of this creative impulse", energy));
            theme.add_source("sigil-forge");
            themes.push(theme);
        }
        themes
    }

    fn extract_geometry_themes(data: &Value) -> Vec<Theme> {
        let mut themes = Vec::new();
        if data.get("available").and_then(|v| v.as_bool()) != Some(true) {
            return themes;
        }

        if let Some(form) = data.get("form").and_then(|v| v.as_str()) {
            let mut theme = Theme::new(format!("Sacred Form: {}", form), Self::get_form_description(&json!(form)).to_string());
            theme.add_source("sacred-geometry");
            themes.push(theme);
        }

        if let Some(qualities) = data.get("qualities").and_then(|v| v.as_array()) {
            let quality_strs: Vec<&str> = qualities.iter().filter_map(|q| q.as_str()).take(3).collect();
            if !quality_strs.is_empty() {
                let mut theme = Theme::new("Form Qualities", format!("Qualities: {}", quality_strs.join(", ")));
                theme.add_source("sacred-geometry");
                themes.push(theme);
            }
        }
        themes
    }

    fn find_intention_form_alignment(sigil: &Value, geo: &Value) -> Vec<Alignment> {
        let mut alignments = Vec::new();
        let sigil_energy = sigil.get("energy").and_then(|v| v.as_str()).unwrap_or("");
        let form = geo.get("form").and_then(|v| v.as_str()).unwrap_or("");
        let qualities = geo.get("qualities").and_then(|v| v.as_array());

        let form_energy_map: HashMap<&str, &str> = [
            ("expansive", "fibonacci"), ("expansive", "spiral"), ("expansive", "flower"),
            ("concentrating", "circle"), ("concentrating", "sri"),
            ("flowing", "torus"), ("protective", "metatron"), ("protective", "merkaba"),
            ("transformative", "vesica"), ("transformative", "merkaba"),
            ("restorative", "seed"), ("restorative", "flower"), ("generative", "seed"),
        ].iter().cloned().collect();

        let form_lower = form.to_lowercase();
        for (energy, form_key) in &form_energy_map {
            if sigil_energy == *energy && form_lower.contains(form_key) {
                alignments.push(
                    Alignment::new(
                        "Energy-Form Alignment",
                        format!("The {} energy aligns naturally with the {} form", sigil_energy, form)
                    )
                    .with_engines(vec!["sigil-forge".into(), "sacred-geometry".into()])
                    .with_confidence(0.8)
                );
                break;
            }
        }

        if let (Some(keywords), Some(quals)) = (sigil.get("keywords").and_then(|v| v.as_array()), qualities) {
            for keyword in keywords.iter().filter_map(|k| k.as_str()) {
                for quality in quals.iter().filter_map(|q| q.as_str()) {
                    if keyword.to_lowercase().contains(&quality.to_lowercase()) || quality.to_lowercase().contains(&keyword.to_lowercase()) {
                        alignments.push(
                            Alignment::new(
                                format!("Shared Resonance: {}", keyword),
                                format!("'{}' appears in both intention and form qualities", keyword)
                            )
                            .with_engines(vec!["sigil-forge".into(), "sacred-geometry".into()])
                            .with_confidence(0.7)
                        );
                    }
                }
            }
        }
        alignments
    }

    fn find_creative_tensions(sigil: &Value, geo: &Value) -> Vec<Tension> {
        let mut tensions = Vec::new();
        let energy = sigil.get("energy").and_then(|v| v.as_str()).unwrap_or("");
        let form = geo.get("form").and_then(|v| v.as_str()).unwrap_or("");

        tensions.push(
            Tension::new(
                "Formless to Form",
                "The dance between intention (formless) and geometry (precise form)"
            )
            .with_perspectives("sigil-forge", "intention", "sacred-geometry", "form")
            .with_integration_hint(format!(
                "Your intention seeks expression, while {} provides structure. \
                 What emerges in the creative tension between chaos and order?", form
            ))
        );

        if energy == "expansive" && form.to_lowercase().contains("circle") {
            tensions.push(
                Tension::new(
                    "Expansion vs Containment",
                    "Expansive energy meeting containing form"
                )
                .with_perspectives("sigil-forge", "expansive", "sacred-geometry", "containing")
                .with_integration_hint("How does infinite expansion express through finite boundary?")
            );
        }
        tensions
    }

    fn create_summary(sigil: &Value, geo: &Value, themes: &[Theme]) -> String {
        let sigil_available = sigil.get("available").and_then(|v| v.as_bool()) == Some(true);
        let geo_available = geo.get("available").and_then(|v| v.as_bool()) == Some(true);

        if !sigil_available && !geo_available {
            return "Awaiting engine outputs for creative synthesis".to_string();
        }

        let mut parts = Vec::new();

        if let Some(distilled) = sigil.get("distilled").and_then(|v| v.as_str()) {
            if !distilled.is_empty() {
                parts.push(format!("Intention distilled to '{}'", distilled));
            }
        }

        if let Some(form) = geo.get("form").and_then(|v| v.as_str()) {
            parts.push(format!("{} provides structural foundation", form));
        }

        if !themes.is_empty() {
            parts.push(format!("{} creative themes identified", themes.len()));
        }

        if parts.is_empty() {
            "Partial synthesis available—awaiting additional engine outputs".to_string()
        } else {
            format!("{}. The creative direction emerges from their combination—allow it to guide rather than dictate.", parts.join(". "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use noesis_core::CalculationMetadata;

    fn mock_sigil_output() -> EngineOutput {
        EngineOutput {
            engine_id: "sigil-forge".to_string(),
            result: json!({
                "intention": "Manifest creative abundance in my life",
                "distilled": "CREATE ABUNDANCE",
                "method": "letter_elimination"
            }),
            witness_prompt: "What does this sigil evoke?".to_string(),
            consciousness_level: 1,
            metadata: CalculationMetadata {
                calculation_time_ms: 5.0, backend: "bridge".to_string(),
                precision_achieved: "standard".to_string(), cached: false, timestamp: Utc::now(),
            },
        }
    }

    fn mock_geometry_output() -> EngineOutput {
        EngineOutput {
            engine_id: "sacred-geometry".to_string(),
            result: json!({
                "form": "Seed of Life",
                "qualities": ["creation", "potential", "genesis"],
                "meditation": "Contemplate the seven circles"
            }),
            witness_prompt: "What do you see in this form?".to_string(),
            consciousness_level: 1,
            metadata: CalculationMetadata {
                calculation_time_ms: 8.0, backend: "bridge".to_string(),
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
        results.insert("sigil-forge".to_string(), mock_sigil_output());
        results.insert("sacred-geometry".to_string(), mock_geometry_output());

        let synthesis = CreativeExpressionSynthesis::synthesize(&results, &test_input());
        assert!(!synthesis.themes.is_empty());
        assert!(!synthesis.summary.is_empty());
    }

    #[test]
    fn test_energy_determination() {
        assert_eq!(CreativeExpressionSynthesis::determine_energy_quality(&json!("expand my vision")), "expansive");
        assert_eq!(CreativeExpressionSynthesis::determine_energy_quality(&json!("protect my space")), "protective");
    }
}
