//! Self Inquiry Workflow (W2-S6-01)
//!
//! Supports shadow work and pattern recognition by combining
//! Gene Keys and Enneagram perspectives.
//!
//! # Purpose
//! Facilitate deep self-inquiry through the lens of Gene Keys frequency
//! bands (Shadow → Gift → Siddhi) and Enneagram core patterns.
//!
//! # Engines
//! - **Gene Keys**: Profile calculation with Life's Work, Evolution, Radiance, Purpose
//! - **Enneagram**: Type analysis with core fear/desire/weakness patterns

use super::ExtendedWorkflowDefinition;
use super::SynthesisType;
use noesis_core::{EngineOutput, WorkflowDefinition};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Input parameters for Self Inquiry workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfInquiryInput {
    /// Birth datetime in ISO 8601 format (required for Gene Keys)
    pub birth_datetime: String,
    /// Birth location with latitude and longitude (required for Gene Keys)
    pub birth_location: BirthLocation,
    /// Enneagram type if known (1-9), otherwise assessment questions are returned
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enneagram_type: Option<u8>,
    /// Focus areas for inquiry
    #[serde(default)]
    pub focus_areas: Vec<FocusArea>,
}

/// Birth location for Gene Keys calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthLocation {
    pub latitude: f64,
    pub longitude: f64,
    /// Optional timezone (IANA format)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

/// Focus areas for self-inquiry
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FocusArea {
    /// Life purpose and core identity
    LifePurpose,
    /// Relationships and connection
    Relationships,
    /// Shadow patterns and triggers
    ShadowWork,
    /// Growth edges and evolution
    Evolution,
    /// Gifts and natural strengths
    Gifts,
}

/// Gene Keys sphere identifiers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GeneKeysSphere {
    /// Life's Work (conscious Sun)
    LifesWork,
    /// Evolution (conscious Earth)
    Evolution,
    /// Radiance (conscious Node)
    Radiance,
    /// Purpose (conscious North Node)
    Purpose,
    /// Attraction (unconscious Node)
    Attraction,
    /// IQ (conscious Mercury)
    IQ,
    /// EQ (conscious Venus)
    EQ,
    /// SQ (unconscious Jupiter)
    SQ,
    /// Vocation (unconscious Mars)
    Vocation,
    /// Culture (unconscious Saturn)
    Culture,
    /// Brand (conscious Moon)
    Brand,
}

/// Self Inquiry Workflow implementation
pub struct SelfInquiryWorkflow;

impl SelfInquiryWorkflow {
    /// Workflow identifier
    pub const ID: &'static str = "self-inquiry";
    
    /// Required consciousness phase
    pub const REQUIRED_PHASE: u8 = 2;

    /// Returns the extended workflow definition with synthesis support
    pub fn definition() -> ExtendedWorkflowDefinition {
        ExtendedWorkflowDefinition {
            id: Self::ID.to_string(),
            name: "Self Inquiry".to_string(),
            description: "Deep self-inquiry combining Gene Keys frequency bands \
                         with Enneagram core patterns for shadow work and \
                         pattern recognition".to_string(),
            engine_ids: vec![
                "gene-keys".to_string(),
                "enneagram".to_string(),
            ],
            synthesis_type: SynthesisType::SelfInquiry,
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
        opts.insert("spheres".to_string(), json!([
            "life_work", "evolution", "radiance", "purpose"
        ]));
        opts.insert("include_shadow_gift_siddhi".to_string(), json!(true));
        opts
    }

    /// Prepare engine-specific options from workflow input
    pub fn prepare_engine_options(input: &SelfInquiryInput) -> HashMap<String, HashMap<String, Value>> {
        let mut engine_opts = HashMap::new();

        // Gene Keys options
        let mut gk_opts = HashMap::new();
        gk_opts.insert("birth_datetime".to_string(), json!(input.birth_datetime.clone()));
        gk_opts.insert("latitude".to_string(), json!(input.birth_location.latitude));
        gk_opts.insert("longitude".to_string(), json!(input.birth_location.longitude));
        if let Some(ref tz) = input.birth_location.timezone {
            gk_opts.insert("timezone".to_string(), json!(tz));
        }
        gk_opts.insert("spheres".to_string(), json!([
            "life_work", "evolution", "radiance", "purpose"
        ]));
        engine_opts.insert("gene-keys".to_string(), gk_opts);

        // Enneagram options
        let mut enn_opts = HashMap::new();
        if let Some(etype) = input.enneagram_type {
            enn_opts.insert("type".to_string(), json!(etype));
            enn_opts.insert("mode".to_string(), json!("analysis"));
        } else {
            enn_opts.insert("mode".to_string(), json!("assessment"));
        }
        enn_opts.insert("include_wings".to_string(), json!(true));
        enn_opts.insert("include_integration_disintegration".to_string(), json!(true));
        engine_opts.insert("enneagram".to_string(), enn_opts);

        engine_opts
    }

    /// Validate that required engines are present in results
    pub fn validate_results(results: &HashMap<String, EngineOutput>) -> Result<(), Vec<String>> {
        // At least one engine should succeed
        let available: Vec<&str> = ["gene-keys", "enneagram"]
            .iter()
            .filter(|e| results.contains_key(&e.to_string()))
            .copied()
            .collect();

        if available.is_empty() {
            Err(vec!["gene-keys".to_string(), "enneagram".to_string()])
        } else {
            Ok(())
        }
    }

    /// Get assessment questions if Enneagram type is unknown
    pub fn get_enneagram_assessment_questions() -> Vec<AssessmentQuestion> {
        vec![
            AssessmentQuestion {
                id: "q1".to_string(),
                text: "When stressed, I tend to:".to_string(),
                options: vec![
                    ("a", "Become more critical and perfectionist"),
                    ("b", "Focus on helping others while ignoring my needs"),
                    ("c", "Work harder to achieve and impress"),
                    ("d", "Withdraw and feel misunderstood"),
                    ("e", "Retreat into research and analysis"),
                    ("f", "Become more anxious and seek reassurance"),
                    ("g", "Distract myself with new activities"),
                    ("h", "Become more controlling and confrontational"),
                    ("i", "Go along to keep the peace"),
                ].into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            },
            AssessmentQuestion {
                id: "q2".to_string(),
                text: "My core motivation is:".to_string(),
                options: vec![
                    ("a", "To be good and have integrity"),
                    ("b", "To be loved and needed"),
                    ("c", "To be successful and admired"),
                    ("d", "To understand myself and be unique"),
                    ("e", "To be capable and competent"),
                    ("f", "To feel secure and supported"),
                    ("g", "To be happy and avoid pain"),
                    ("h", "To protect myself and be in control"),
                    ("i", "To have inner peace and harmony"),
                ].into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            },
        ]
    }
}

/// Assessment question for determining Enneagram type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentQuestion {
    pub id: String,
    pub text: String,
    pub options: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition() {
        let def = SelfInquiryWorkflow::definition();
        assert_eq!(def.id, "self-inquiry");
        assert_eq!(def.engine_ids.len(), 2);
        assert!(def.engine_ids.contains(&"gene-keys".to_string()));
        assert!(def.engine_ids.contains(&"enneagram".to_string()));
        assert_eq!(def.synthesis_type, SynthesisType::SelfInquiry);
        assert_eq!(def.required_phase, 2);
    }

    #[test]
    fn test_base_definition() {
        let base = SelfInquiryWorkflow::base_definition();
        assert_eq!(base.id, "self-inquiry");
        assert_eq!(base.engine_ids.len(), 2);
    }

    #[test]
    fn test_prepare_engine_options_with_known_type() {
        let input = SelfInquiryInput {
            birth_datetime: "1990-05-15T14:30:00Z".to_string(),
            birth_location: BirthLocation {
                latitude: 12.9716,
                longitude: 77.5946,
                timezone: Some("Asia/Kolkata".to_string()),
            },
            enneagram_type: Some(4),
            focus_areas: vec![FocusArea::ShadowWork],
        };

        let opts = SelfInquiryWorkflow::prepare_engine_options(&input);
        
        assert!(opts.contains_key("gene-keys"));
        assert_eq!(opts["gene-keys"]["latitude"], json!(12.9716));
        
        assert!(opts.contains_key("enneagram"));
        assert_eq!(opts["enneagram"]["type"], json!(4));
        assert_eq!(opts["enneagram"]["mode"], json!("analysis"));
    }

    #[test]
    fn test_prepare_engine_options_without_known_type() {
        let input = SelfInquiryInput {
            birth_datetime: "1990-05-15T14:30:00Z".to_string(),
            birth_location: BirthLocation {
                latitude: 12.9716,
                longitude: 77.5946,
                timezone: None,
            },
            enneagram_type: None,
            focus_areas: vec![],
        };

        let opts = SelfInquiryWorkflow::prepare_engine_options(&input);
        
        assert!(opts.contains_key("enneagram"));
        assert_eq!(opts["enneagram"]["mode"], json!("assessment"));
        assert!(!opts["enneagram"].contains_key("type"));
    }

    #[test]
    fn test_assessment_questions() {
        let questions = SelfInquiryWorkflow::get_enneagram_assessment_questions();
        assert!(!questions.is_empty());
        
        for q in questions {
            assert!(!q.text.is_empty());
            assert_eq!(q.options.len(), 9); // 9 Enneagram types
        }
    }
}
