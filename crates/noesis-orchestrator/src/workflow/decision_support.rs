//! Decision Support Workflow (W2-S5-07)
//!
//! Helps users approach decisions from multiple archetypal perspectives
//! by combining Tarot, I-Ching, and Human Design Authority.
//!
//! # Purpose
//! Provide multi-system mirrors for decision-making without prescribing outcomes.
//!
//! # Engines
//! - **Tarot**: Archetypal imagery and temporal positioning
//! - **I-Ching**: Hexagram wisdom and changing lines
//! - **Human Design**: Authority type for decision-making style

use super::ExtendedWorkflowDefinition;
use super::SynthesisType;
use noesis_core::{EngineOutput, WorkflowDefinition};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Input parameters for Decision Support workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionSupportInput {
    /// The decision or situation to explore (required)
    pub question: String,
    /// Tarot spread type (default: THREE_CARD)
    #[serde(default = "default_spread")]
    pub spread: TarotSpread,
}

fn default_spread() -> TarotSpread {
    TarotSpread::ThreeCard
}

/// Available Tarot spread types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TarotSpread {
    #[serde(rename = "SINGLE", alias = "single", alias = "single_card")]
    Single,
    #[serde(rename = "THREE_CARD", alias = "three_card", alias = "three")]
    ThreeCard,
    #[serde(rename = "CELTIC_CROSS", alias = "celtic_cross", alias = "celtic")]
    CelticCross,
    #[serde(rename = "HORSESHOE", alias = "horseshoe")]
    Horseshoe,
    #[serde(rename = "RELATIONSHIP", alias = "relationship")]
    Relationship,
    #[serde(rename = "CAREER", alias = "career")]
    Career,
    #[serde(rename = "YES_NO", alias = "yes_no", alias = "yes_or_no", alias = "yesno")]
    YesNo,
}

#[allow(clippy::derivable_impls)]
impl Default for TarotSpread {
    fn default() -> Self {
        TarotSpread::ThreeCard
    }
}

impl TarotSpread {
    pub fn as_str(&self) -> &'static str {
        match self {
            TarotSpread::Single => "SINGLE",
            TarotSpread::ThreeCard => "THREE_CARD",
            TarotSpread::CelticCross => "CELTIC_CROSS",
            TarotSpread::Horseshoe => "HORSESHOE",
            TarotSpread::Relationship => "RELATIONSHIP",
            TarotSpread::Career => "CAREER",
            TarotSpread::YesNo => "YES_NO",
        }
    }

    pub fn as_engine_param(&self) -> &'static str {
        match self {
            TarotSpread::Single => "single_card",
            TarotSpread::ThreeCard => "three_card",
            TarotSpread::CelticCross => "celtic_cross",
            TarotSpread::Horseshoe => "horseshoe",
            TarotSpread::Relationship => "relationship",
            TarotSpread::Career => "career",
            TarotSpread::YesNo => "yes_no",
        }
    }
}

/// Decision Support Workflow implementation
pub struct DecisionSupportWorkflow;

impl DecisionSupportWorkflow {
    /// Workflow identifier
    pub const ID: &'static str = "decision-support";

    /// Required consciousness phase
    pub const REQUIRED_PHASE: u8 = 1;

    /// Returns the extended workflow definition with synthesis support
    pub fn definition() -> ExtendedWorkflowDefinition {
        ExtendedWorkflowDefinition {
            id: Self::ID.to_string(),
            name: "Decision Support".to_string(),
            description: "Multi-system decision mirrors combining Tarot archetypes, \
                         I-Ching hexagrams, and Human Design Authority for \
                         exploring decisions from multiple perspectives"
                .to_string(),
            engine_ids: vec![
                "tarot".to_string(),
                "i-ching".to_string(),
                "human-design".to_string(),
                "enneagram".to_string(),
                "gene-keys".to_string(),
            ],
            synthesis_type: SynthesisType::DecisionSupport,
            required_phase: Self::REQUIRED_PHASE,
            default_options: Self::default_options(),
        }
    }

    /// Returns the base workflow definition
    pub fn base_definition() -> WorkflowDefinition {
        Self::definition().to_base()
    }

    /// Default options for the workflow
    pub fn default_options() -> HashMap<String, Value> {
        let mut opts = HashMap::new();
        opts.insert(
            "spread".to_string(),
            json!(TarotSpread::ThreeCard.as_engine_param()),
        );
        opts
    }

    /// Prepare engine-specific options from workflow input
    pub fn prepare_engine_options(
        input: &DecisionSupportInput,
    ) -> HashMap<String, HashMap<String, Value>> {
        let mut engine_opts = HashMap::new();

        // Tarot options
        let mut tarot_opts = HashMap::new();
        tarot_opts.insert("spread".to_string(), json!(input.spread.as_engine_param()));
        tarot_opts.insert("question".to_string(), json!(input.question.clone()));
        engine_opts.insert("tarot".to_string(), tarot_opts);

        // I-Ching options
        let mut iching_opts = HashMap::new();
        iching_opts.insert("question".to_string(), json!(input.question.clone()));
        engine_opts.insert("i-ching".to_string(), iching_opts);

        // Human Design options (extract authority only)
        let mut hd_opts = HashMap::new();
        hd_opts.insert("extract_fields".to_string(), json!(["authority", "type"]));
        engine_opts.insert("human-design".to_string(), hd_opts);

        // Enneagram and Gene Keys options add pattern-depth context
        let mut enneagram_opts = HashMap::new();
        enneagram_opts.insert("question".to_string(), json!(input.question.clone()));
        enneagram_opts.insert("includeMovementPrompts".to_string(), json!(true));
        engine_opts.insert("enneagram".to_string(), enneagram_opts);

        let mut gk_opts = HashMap::new();
        gk_opts.insert("question".to_string(), json!(input.question.clone()));
        gk_opts.insert("include_shadow_gift_siddhi".to_string(), json!(true));
        engine_opts.insert("gene-keys".to_string(), gk_opts);

        engine_opts
    }

    /// Validate that required engines are present in results
    pub fn validate_results(results: &HashMap<String, EngineOutput>) -> Result<(), Vec<String>> {
        let required = ["tarot", "i-ching", "human-design", "enneagram", "gene-keys"];
        let missing: Vec<String> = required
            .iter()
            .filter(|e| !results.contains_key(&e.to_string()))
            .map(|e| e.to_string())
            .collect();

        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition() {
        let def = DecisionSupportWorkflow::definition();
        assert_eq!(def.id, "decision-support");
        assert_eq!(def.engine_ids.len(), 5);
        assert!(def.engine_ids.contains(&"tarot".to_string()));
        assert!(def.engine_ids.contains(&"i-ching".to_string()));
        assert!(def.engine_ids.contains(&"human-design".to_string()));
        assert!(def.engine_ids.contains(&"enneagram".to_string()));
        assert!(def.engine_ids.contains(&"gene-keys".to_string()));
        assert_eq!(def.synthesis_type, SynthesisType::DecisionSupport);
    }

    #[test]
    fn test_base_definition() {
        let base = DecisionSupportWorkflow::base_definition();
        assert_eq!(base.id, "decision-support");
        assert_eq!(base.engine_ids.len(), 5);
    }

    #[test]
    fn test_prepare_engine_options() {
        let input = DecisionSupportInput {
            question: "Should I change careers?".to_string(),
            spread: TarotSpread::CelticCross,
        };

        let opts = DecisionSupportWorkflow::prepare_engine_options(&input);

        assert!(opts.contains_key("tarot"));
        assert_eq!(opts["tarot"]["spread"], json!("celtic_cross"));
        assert_eq!(opts["tarot"]["question"], json!("Should I change careers?"));

        assert!(opts.contains_key("i-ching"));
        assert_eq!(
            opts["i-ching"]["question"],
            json!("Should I change careers?")
        );
    }

    #[test]
    fn test_spread_types() {
        assert_eq!(TarotSpread::Single.as_str(), "SINGLE");
        assert_eq!(TarotSpread::ThreeCard.as_str(), "THREE_CARD");
        assert_eq!(TarotSpread::CelticCross.as_str(), "CELTIC_CROSS");
        assert_eq!(TarotSpread::Horseshoe.as_str(), "HORSESHOE");
        assert_eq!(TarotSpread::Relationship.as_str(), "RELATIONSHIP");
        assert_eq!(TarotSpread::Career.as_str(), "CAREER");
        assert_eq!(TarotSpread::YesNo.as_str(), "YES_NO");
        assert_eq!(TarotSpread::YesNo.as_engine_param(), "yes_no");
    }

    #[test]
    fn test_spread_deserialization_aliases() {
        #[derive(Deserialize)]
        struct Wrapper {
            spread: TarotSpread,
        }

        let upper: Wrapper = serde_json::from_value(json!({ "spread": "YES_NO" }))
            .expect("uppercase enum variant should deserialize");
        assert_eq!(upper.spread, TarotSpread::YesNo);

        let canonical: Wrapper = serde_json::from_value(json!({ "spread": "CAREER" }))
            .expect("career enum variant should deserialize");
        assert_eq!(canonical.spread, TarotSpread::Career);

        let lower_yes_no: Wrapper = serde_json::from_value(json!({ "spread": "yes_no" }))
            .expect("lowercase yes_no alias should deserialize");
        assert_eq!(lower_yes_no.spread, TarotSpread::YesNo);

        let lower_relationship: Wrapper =
            serde_json::from_value(json!({ "spread": "relationship" }))
                .expect("lowercase relationship alias should deserialize");
        assert_eq!(lower_relationship.spread, TarotSpread::Relationship);
    }
}
