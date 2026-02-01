//! Decision Support Synthesis (W2-S5-08)
//!
//! Synthesizes Tarot, I-Ching, and Human Design outputs to find
//! alignments and tensions across archetypal perspectives.
//!
//! # Synthesis Approach
//! 1. Extract Tarot card themes (Major Arcana = significant, positions = temporal)
//! 2. Extract I-Ching hexagram meaning and changing lines
//! 3. Extract HD Authority (Sacral, Emotional, Splenic, Ego, Self-Projected, Mental, Lunar)
//!
//! # Alignments
//! - Tarot archetypes echoing I-Ching imagery
//! - Both systems pointing same direction (wait/act)
//! - HD Authority alignment with gut-level or mind-level guidance
//!
//! # Tensions
//! - Framed as "multiple perspectives," not contradictions

use super::Synthesizer;
use crate::workflow::models::{
    Alignment, SynthesisResult, Tension, Theme, WitnessPrompt, InquiryType,
};
use noesis_core::{EngineInput, EngineOutput};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Decision Support synthesis implementation
pub struct DecisionSupportSynthesis;

impl DecisionSupportSynthesis {
    /// Synthesize results from all engines (convenience method without input)
    pub fn synthesize_results(results: &HashMap<String, EngineOutput>) -> SynthesisResult {
        // Create a dummy input for the trait method
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

impl Synthesizer for DecisionSupportSynthesis {
    fn synthesize(
        results: &HashMap<String, EngineOutput>,
        _input: &EngineInput,
    ) -> SynthesisResult {
        let mut themes = Vec::new();
        let mut alignments = Vec::new();
        let mut tensions = Vec::new();

        // Extract data from each engine
        let tarot_data = Self::extract_tarot_data(results.get("tarot"));
        let iching_data = Self::extract_iching_data(results.get("i-ching"));
        let hd_data = Self::extract_hd_authority(results.get("human-design"));

        // Extract themes from each system
        themes.extend(Self::extract_tarot_themes(&tarot_data));
        themes.extend(Self::extract_iching_themes(&iching_data));
        themes.extend(Self::extract_authority_themes(&hd_data));

        // Find alignments
        alignments.extend(Self::find_directional_alignment(&tarot_data, &iching_data));
        alignments.extend(Self::find_archetypal_echoes(&tarot_data, &iching_data));
        alignments.extend(Self::find_authority_alignment(&hd_data, &tarot_data, &iching_data));

        // Find tensions (framed as multiple perspectives)
        tensions.extend(Self::find_temporal_tensions(&tarot_data, &iching_data));
        tensions.extend(Self::find_body_mind_tensions(&hd_data, &tarot_data));

        // Create summary
        let summary = Self::create_summary(&themes, &alignments, &tensions);

        SynthesisResult {
            themes,
            alignments,
            tensions,
            summary,
        }
    }
}

impl DecisionSupportSynthesis {
    /// Generate witness prompts for decision support
    pub fn generate_witness_prompts(
        hd_data: &Value,
        tarot_data: &Value,
        iching_data: &Value,
    ) -> Vec<WitnessPrompt> {
        let mut prompts = Vec::new();

        // Authority-based prompt
        if let Some(auth) = hd_data.get("authority").and_then(|v| v.as_str()) {
            prompts.push(WitnessPrompt::new(
                format!("What does your {} sense about this decision?", auth.replace("_", " ").replace("-", " ")),
                InquiryType::PatternNoticing,
            ).with_context("HD Authority"));
        }

        // Card and hexagram contemplation
        if let (Some(cards), Some(hex_name)) = (
            tarot_data.get("cards").and_then(|c| c.as_array()).and_then(|a| a.first()),
            iching_data.get("name").and_then(|n| n.as_str())
        ) {
            let card_name = cards.get("name").and_then(|n| n.as_str()).unwrap_or("the card");
            prompts.push(WitnessPrompt::new(
                format!("How do the images of {} and {} sit together in you?", card_name, hex_name),
                InquiryType::TensionExploration,
            ));
        }

        // Perspective tension prompt
        prompts.push(WitnessPrompt::new(
            "Where do you feel pulled between these different perspectives?",
            InquiryType::TensionExploration,
        ));

        // Body awareness prompt
        prompts.push(WitnessPrompt::new(
            "As you hold this decision, where do you notice sensation in your body?",
            InquiryType::PatternNoticing,
        ));

        prompts
    }

    /// Extract relevant data from Tarot output
    fn extract_tarot_data(output: Option<&EngineOutput>) -> Value {
        let Some(output) = output else {
            return json!({ "available": false });
        };

        let result = &output.result;
        let cards = result.get("cards").cloned().unwrap_or(json!([]));
        let spread_type = result.get("spread").cloned().unwrap_or(json!("unknown"));
        
        let major_arcana: Vec<&Value> = if let Some(cards_arr) = cards.as_array() {
            cards_arr.iter()
                .filter(|c| c.get("arcana").and_then(|a| a.as_str()) == Some("major"))
                .collect()
        } else {
            vec![]
        };

        let direction = Self::infer_tarot_direction(result);

        json!({
            "available": true,
            "cards": cards,
            "spread": spread_type,
            "major_arcana_count": major_arcana.len(),
            "major_arcana": major_arcana,
            "direction": direction,
            "positions": result.get("positions").cloned().unwrap_or(json!({}))
        })
    }

    fn infer_tarot_direction(result: &Value) -> String {
        let action_cards = ["The Chariot", "Strength", "The Magician", "Wheel of Fortune", "The Sun"];
        let wait_cards = ["The Hermit", "The Hanged Man", "The High Priestess", "The Moon", "Four of Swords"];
        
        let cards_text = result.to_string().to_lowercase();
        
        let action_score: i32 = action_cards.iter()
            .filter(|c| cards_text.contains(&c.to_lowercase()))
            .count() as i32;
        let wait_score: i32 = wait_cards.iter()
            .filter(|c| cards_text.contains(&c.to_lowercase()))
            .count() as i32;

        match action_score.cmp(&wait_score) {
            std::cmp::Ordering::Greater => "action".to_string(),
            std::cmp::Ordering::Less => "wait".to_string(),
            std::cmp::Ordering::Equal => "reflect".to_string(),
        }
    }

    fn extract_iching_data(output: Option<&EngineOutput>) -> Value {
        let Some(output) = output else {
            return json!({ "available": false });
        };

        let result = &output.result;
        let hexagram = result.get("hexagram").cloned().unwrap_or(json!({}));
        let number = hexagram.get("number").cloned().unwrap_or(json!(0));
        let name = hexagram.get("name").cloned().unwrap_or(json!("unknown"));
        let changing_lines = result.get("changing_lines").cloned().unwrap_or(json!([]));
        let relating = result.get("relating_hexagram").cloned();
        let direction = Self::infer_iching_direction(&hexagram, &changing_lines);

        json!({
            "available": true,
            "number": number,
            "name": name,
            "hexagram": hexagram,
            "changing_lines": changing_lines,
            "relating_hexagram": relating,
            "direction": direction,
            "judgment": hexagram.get("judgment").cloned().unwrap_or(json!("")),
            "image": hexagram.get("image").cloned().unwrap_or(json!(""))
        })
    }

    fn infer_iching_direction(hexagram: &Value, _changing_lines: &Value) -> String {
        let action_hexagrams = [1, 3, 14, 25, 34, 42, 43, 49];
        let wait_hexagrams = [2, 5, 23, 27, 33, 52, 53, 62];
        
        let number = hexagram.get("number")
            .and_then(|n| n.as_i64())
            .unwrap_or(0) as i32;

        if action_hexagrams.contains(&number) {
            "action".to_string()
        } else if wait_hexagrams.contains(&number) {
            "wait".to_string()
        } else {
            "observe".to_string()
        }
    }

    fn extract_hd_authority(output: Option<&EngineOutput>) -> Value {
        let Some(output) = output else {
            return json!({ "available": false });
        };

        let result = &output.result;
        let authority = result.get("authority")
            .or_else(|| result.get("chart").and_then(|c| c.get("authority")))
            .cloned()
            .unwrap_or(json!("unknown"));
        
        let hd_type = result.get("type")
            .or_else(|| result.get("chart").and_then(|c| c.get("type")))
            .cloned()
            .unwrap_or(json!("unknown"));

        let authority_str = authority.as_str().unwrap_or("unknown").to_lowercase();
        let decision_style = Self::categorize_authority(&authority_str);

        json!({
            "available": true,
            "authority": authority,
            "type": hd_type,
            "decision_style": decision_style,
            "authority_guidance": Self::authority_guidance(&authority_str)
        })
    }

    fn categorize_authority(authority: &str) -> &'static str {
        match authority {
            s if s.contains("sacral") => "body",
            s if s.contains("emotional") || s.contains("solar plexus") => "emotional-body",
            s if s.contains("splenic") => "body-instinct",
            s if s.contains("ego") || s.contains("heart") => "willpower",
            s if s.contains("self-projected") || s.contains("g center") => "identity",
            s if s.contains("mental") || s.contains("outer") => "mind-reflective",
            s if s.contains("lunar") || s.contains("moon") => "time-based",
            _ => "unknown",
        }
    }

    fn authority_guidance(authority: &str) -> &'static str {
        match authority {
            s if s.contains("sacral") => "Trust your gut response - the immediate yes/no that arises",
            s if s.contains("emotional") || s.contains("solar plexus") => "Wait for emotional clarity - decisions improve over time",
            s if s.contains("splenic") => "Trust instant knowing - the body's survival wisdom",
            s if s.contains("ego") || s.contains("heart") => "Check if you truly have the will/desire for this",
            s if s.contains("self-projected") => "Talk it through - hear yourself speak about it",
            s if s.contains("mental") || s.contains("outer") => "Discuss with trusted others - reflect on their mirrors",
            s if s.contains("lunar") || s.contains("moon") => "Allow a full lunar cycle - patterns reveal over time",
            _ => "Notice what arises without forcing a decision",
        }
    }

    fn extract_tarot_themes(data: &Value) -> Vec<Theme> {
        let mut themes = Vec::new();
        
        if data.get("available").and_then(|v| v.as_bool()) != Some(true) {
            return themes;
        }

        if let Some(count) = data.get("major_arcana_count").and_then(|v| v.as_i64()) {
            if count > 0 {
                let mut theme = Theme::new("Archetypal Forces", "Significant archetypal energies at play in this decision");
                theme.add_source("tarot");
                themes.push(theme);
            }
        }

        if let Some(dir) = data.get("direction").and_then(|v| v.as_str()) {
            let mut theme = Theme::new(
                format!("Tarot Direction: {}", dir),
                format!("Cards suggest a '{}' orientation", dir)
            );
            theme.add_source("tarot");
            themes.push(theme);
        }

        themes
    }

    fn extract_iching_themes(data: &Value) -> Vec<Theme> {
        let mut themes = Vec::new();
        
        if data.get("available").and_then(|v| v.as_bool()) != Some(true) {
            return themes;
        }

        if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
            let mut theme = Theme::new(format!("Hexagram: {}", name), format!("I-Ching reveals {}", name));
            theme.add_source("i-ching");
            themes.push(theme);
        }

        if let Some(lines) = data.get("changing_lines").and_then(|v| v.as_array()) {
            if !lines.is_empty() {
                let mut theme = Theme::new("Transition", "Situation is in a state of change");
                theme.add_source("i-ching");
                themes.push(theme);
            }
        }

        themes
    }

    fn extract_authority_themes(data: &Value) -> Vec<Theme> {
        let mut themes = Vec::new();
        
        if data.get("available").and_then(|v| v.as_bool()) != Some(true) {
            return themes;
        }

        if let Some(auth) = data.get("authority").and_then(|v| v.as_str()) {
            let mut theme = Theme::new(
                format!("Authority: {}", auth),
                format!("Decision-making through {} authority", auth)
            );
            theme.add_source("human-design");
            themes.push(theme);
        }

        themes
    }

    fn find_directional_alignment(tarot: &Value, iching: &Value) -> Vec<Alignment> {
        let mut alignments = Vec::new();

        let tarot_dir = tarot.get("direction").and_then(|v| v.as_str());
        let iching_dir = iching.get("direction").and_then(|v| v.as_str());

        if let (Some(t), Some(i)) = (tarot_dir, iching_dir) {
            if t == i {
                alignments.push(
                    Alignment::new(
                        "Directional Alignment",
                        format!("Both systems suggest a '{}' orientation", t)
                    )
                    .with_engines(vec!["tarot".into(), "i-ching".into()])
                    .with_confidence(0.8)
                );
            }
        }

        alignments
    }

    fn find_archetypal_echoes(tarot: &Value, iching: &Value) -> Vec<Alignment> {
        let mut alignments = Vec::new();

        let archetypes = [
            ("hermit", "mountain", "stillness/contemplation"),
            ("emperor", "creative", "authority/structure"),
            ("empress", "receptive", "nurturing/receiving"),
            ("tower", "shock", "sudden change/breakthrough"),
            ("death", "splitting apart", "transformation/ending"),
            ("star", "peace", "hope/renewal"),
            ("chariot", "progress", "movement/determination"),
        ];

        let tarot_text = tarot.to_string().to_lowercase();
        let iching_text = iching.to_string().to_lowercase();

        for (tarot_key, iching_key, theme) in archetypes {
            if tarot_text.contains(tarot_key) && iching_text.contains(iching_key) {
                alignments.push(
                    Alignment::new(
                        format!("Archetypal Echo: {}", theme),
                        format!("Shared theme of {} across both systems", theme)
                    )
                    .with_engines(vec!["tarot".into(), "i-ching".into()])
                    .with_confidence(0.7)
                );
            }
        }

        alignments
    }

    fn find_authority_alignment(hd: &Value, tarot: &Value, iching: &Value) -> Vec<Alignment> {
        let mut alignments = Vec::new();

        let decision_style = hd.get("decision_style").and_then(|v| v.as_str()).unwrap_or("unknown");
        let tarot_dir = tarot.get("direction").and_then(|v| v.as_str()).unwrap_or("unknown");
        let iching_dir = iching.get("direction").and_then(|v| v.as_str()).unwrap_or("unknown");

        if (decision_style == "body" || decision_style == "body-instinct") && tarot_dir == "action" {
            alignments.push(
                Alignment::new(
                    "Body-Action Alignment",
                    "Body-based authority aligns with action-oriented cards"
                )
                .with_engines(vec!["human-design".into(), "tarot".into()])
                .with_confidence(0.75)
            );
        }

        if (decision_style == "emotional-body" || decision_style == "time-based") 
            && (tarot_dir == "wait" || iching_dir == "wait") {
            alignments.push(
                Alignment::new(
                    "Patience Alignment",
                    "Time-based authority aligns with patience guidance"
                )
                .with_engines(vec!["human-design".into(), "i-ching".into()])
                .with_confidence(0.75)
            );
        }

        alignments
    }

    fn find_temporal_tensions(tarot: &Value, iching: &Value) -> Vec<Tension> {
        let mut tensions = Vec::new();

        let tarot_dir = tarot.get("direction").and_then(|v| v.as_str());
        let iching_dir = iching.get("direction").and_then(|v| v.as_str());

        if let (Some(t), Some(i)) = (tarot_dir, iching_dir) {
            if (t == "action" && i == "wait") || (t == "wait" && i == "action") {
                tensions.push(
                    Tension::new(
                        "Timing Perspectives",
                        "Different perspectives on timing"
                    )
                    .with_perspectives("tarot", t, "i-ching", i)
                    .with_integration_hint(format!(
                        "Tarot points toward '{}' while I-Ching suggests '{}'. \
                         What does each perspective illuminate about your situation?",
                        t, i
                    ))
                );
            }
        }

        tensions
    }

    fn find_body_mind_tensions(hd: &Value, tarot: &Value) -> Vec<Tension> {
        let mut tensions = Vec::new();

        let decision_style = hd.get("decision_style").and_then(|v| v.as_str()).unwrap_or("unknown");
        let tarot_dir = tarot.get("direction").and_then(|v| v.as_str()).unwrap_or("unknown");

        if decision_style == "mind-reflective" && tarot_dir == "action" {
            tensions.push(
                Tension::new(
                    "Reflection vs Action",
                    "Reflective authority meets action-oriented imagery"
                )
                .with_perspectives("human-design", "mind-reflective", "tarot", "action")
                .with_integration_hint(
                    "Your design suggests processing through reflection with others, \
                     while the cards show action energy. How might you honor both?"
                )
            );
        }

        if decision_style == "body" && tarot_dir == "wait" {
            tensions.push(
                Tension::new(
                    "Body vs Stillness",
                    "Body-based authority meets contemplative imagery"
                )
                .with_perspectives("human-design", "body-based", "tarot", "wait")
                .with_integration_hint(
                    "Your sacral responds in the moment, yet the cards suggest stillness. \
                     What does your body know that your mind hasn't caught up with?"
                )
            );
        }

        tensions
    }

    fn create_summary(themes: &[Theme], alignments: &[Alignment], tensions: &[Tension]) -> String {
        let mut parts = Vec::new();

        if !themes.is_empty() {
            parts.push(format!("{} themes identified", themes.len()));
        }

        if !alignments.is_empty() {
            parts.push(format!("{} alignments found", alignments.len()));
        }

        if !tensions.is_empty() {
            parts.push(format!("{} contrasting perspectives noted", tensions.len()));
        }

        if parts.is_empty() {
            "Synthesis awaiting complete engine outputs".to_string()
        } else {
            format!(
                "Decision mirrors reflect: {}. These perspectives \
                 offer different vantage points for your inquiry.",
                parts.join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use noesis_core::CalculationMetadata;

    fn mock_tarot_output() -> EngineOutput {
        EngineOutput {
            engine_id: "tarot".to_string(),
            result: json!({
                "spread": "THREE_CARD",
                "cards": [
                    { "name": "The Hermit", "position": "past", "arcana": "major" },
                    { "name": "The Chariot", "position": "present", "arcana": "major" },
                    { "name": "Three of Cups", "position": "future", "arcana": "minor" }
                ]
            }),
            witness_prompt: "What do these images evoke?".to_string(),
            consciousness_level: 1,
            metadata: CalculationMetadata {
                calculation_time_ms: 10.0,
                backend: "bridge".to_string(),
                precision_achieved: "standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
            },
        }
    }

    fn mock_iching_output() -> EngineOutput {
        EngineOutput {
            engine_id: "i-ching".to_string(),
            result: json!({
                "hexagram": { "number": 5, "name": "Waiting" },
                "changing_lines": [2, 5]
            }),
            witness_prompt: "What does this hexagram show you?".to_string(),
            consciousness_level: 1,
            metadata: CalculationMetadata {
                calculation_time_ms: 8.0,
                backend: "bridge".to_string(),
                precision_achieved: "standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
            },
        }
    }

    fn mock_hd_output() -> EngineOutput {
        EngineOutput {
            engine_id: "human-design".to_string(),
            result: json!({ "authority": "sacral", "type": "Generator" }),
            witness_prompt: "How does your sacral respond?".to_string(),
            consciousness_level: 1,
            metadata: CalculationMetadata {
                calculation_time_ms: 15.0,
                backend: "native".to_string(),
                precision_achieved: "standard".to_string(),
                cached: false,
                timestamp: Utc::now(),
            },
        }
    }

    fn test_input() -> EngineInput {
        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: noesis_core::Precision::Standard,
            options: HashMap::new(),
        }
    }

    #[test]
    fn test_synthesis_with_all_engines() {
        let mut results = HashMap::new();
        results.insert("tarot".to_string(), mock_tarot_output());
        results.insert("i-ching".to_string(), mock_iching_output());
        results.insert("human-design".to_string(), mock_hd_output());

        let synthesis = DecisionSupportSynthesis::synthesize(&results, &test_input());

        assert!(!synthesis.themes.is_empty());
        assert!(!synthesis.summary.is_empty());
    }

    #[test]
    fn test_synthesis_handles_missing_engines() {
        let mut results = HashMap::new();
        results.insert("tarot".to_string(), mock_tarot_output());

        let synthesis = DecisionSupportSynthesis::synthesize(&results, &test_input());
        // Should still work with partial data
        assert!(!synthesis.summary.is_empty());
    }

    #[test]
    fn test_authority_categorization() {
        assert_eq!(DecisionSupportSynthesis::categorize_authority("sacral"), "body");
        assert_eq!(DecisionSupportSynthesis::categorize_authority("emotional"), "emotional-body");
        assert_eq!(DecisionSupportSynthesis::categorize_authority("splenic"), "body-instinct");
    }
}
