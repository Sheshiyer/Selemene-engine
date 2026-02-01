//! Creative Expression Workflow (W2-S6-03)
//!
//! Provides generative and creative guidance through the combination
//! of Sigil Forge and Sacred Geometry.
//!
//! # Purpose
//! Support creative intention-setting and inspiration by combining
//! symbolic systems for creative direction.
//!
//! # Engines
//! - **Sigil Forge**: Process intention into sigil guidance
//! - **Sacred Geometry**: Select/provide appropriate sacred form

use super::ExtendedWorkflowDefinition;
use super::SynthesisType;
use noesis_core::{EngineOutput, WorkflowDefinition};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Input parameters for Creative Expression workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeExpressionInput {
    /// Creative intention or project focus (required)
    pub intention: String,
    /// Preferred sacred geometry form (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<SacredForm>,
    /// Sigil creation method (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<SigilMethod>,
    /// Creative medium or context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<String>,
}

/// Available sacred geometry forms
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SacredForm {
    /// Perfect circle - unity, wholeness, cycles
    Circle,
    /// Vesica Piscis - intersection, birth, duality
    VesicaPiscis,
    /// Seed of Life - creation, potential, genesis
    SeedOfLife,
    /// Flower of Life - interconnection, infinite patterns
    FlowerOfLife,
    /// Metatron's Cube - all Platonic solids, universal truth
    MetatronsCube,
    /// Sri Yantra - supreme energy, manifestation
    SriYantra,
    /// Torus - flow, self-sustaining energy
    Torus,
    /// Fibonacci Spiral - natural growth, golden ratio
    FibonacciSpiral,
    /// Merkaba - light spirit body, interdimensional
    Merkaba,
    /// Platonic Solids - elemental forms
    PlatonicSolids,
}

impl SacredForm {
    pub fn as_str(&self) -> &'static str {
        match self {
            SacredForm::Circle => "circle",
            SacredForm::VesicaPiscis => "vesica_piscis",
            SacredForm::SeedOfLife => "seed_of_life",
            SacredForm::FlowerOfLife => "flower_of_life",
            SacredForm::MetatronsCube => "metatrons_cube",
            SacredForm::SriYantra => "sri_yantra",
            SacredForm::Torus => "torus",
            SacredForm::FibonacciSpiral => "fibonacci_spiral",
            SacredForm::Merkaba => "merkaba",
            SacredForm::PlatonicSolids => "platonic_solids",
        }
    }

    /// Get the qualities associated with this form
    pub fn qualities(&self) -> &[&'static str] {
        match self {
            SacredForm::Circle => &["unity", "wholeness", "cycles", "eternity", "completion"],
            SacredForm::VesicaPiscis => &["intersection", "birth", "duality", "feminine", "portal"],
            SacredForm::SeedOfLife => &["creation", "potential", "genesis", "beginning", "fertility"],
            SacredForm::FlowerOfLife => &["interconnection", "infinite", "patterns", "life", "growth"],
            SacredForm::MetatronsCube => &["universal truth", "balance", "harmony", "divine", "protection"],
            SacredForm::SriYantra => &["supreme energy", "manifestation", "cosmos", "enlightenment", "abundance"],
            SacredForm::Torus => &["flow", "energy", "self-sustaining", "dynamic", "renewal"],
            SacredForm::FibonacciSpiral => &["natural growth", "golden ratio", "expansion", "evolution", "beauty"],
            SacredForm::Merkaba => &["light body", "interdimensional", "ascension", "protection", "transport"],
            SacredForm::PlatonicSolids => &["elements", "structure", "foundation", "primal", "mathematical"],
        }
    }
}

/// Sigil creation methods
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SigilMethod {
    /// Traditional letter elimination and combination
    LetterElimination,
    /// Word-based symbolic construction
    WordSquare,
    /// Planetary magic square method
    PlanetarySquare,
    /// Rose cross method
    RoseCross,
    /// Chaos magic style
    ChaosSigil,
    /// Geometric abstraction
    Geometric,
    /// Automatic drawing / channeled
    Automatic,
}

impl SigilMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            SigilMethod::LetterElimination => "letter_elimination",
            SigilMethod::WordSquare => "word_square",
            SigilMethod::PlanetarySquare => "planetary_square",
            SigilMethod::RoseCross => "rose_cross",
            SigilMethod::ChaosSigil => "chaos_sigil",
            SigilMethod::Geometric => "geometric",
            SigilMethod::Automatic => "automatic",
        }
    }
}

/// Creative Expression Workflow implementation
pub struct CreativeExpressionWorkflow;

impl CreativeExpressionWorkflow {
    /// Workflow identifier
    pub const ID: &'static str = "creative-expression";
    
    /// Required consciousness phase
    pub const REQUIRED_PHASE: u8 = 1;

    /// Returns the extended workflow definition with synthesis support
    pub fn definition() -> ExtendedWorkflowDefinition {
        ExtendedWorkflowDefinition {
            id: Self::ID.to_string(),
            name: "Creative Expression".to_string(),
            description: "Generative creative guidance combining Sigil Forge \
                         intention processing with Sacred Geometry forms for \
                         creative inspiration and direction".to_string(),
            engine_ids: vec![
                "sigil-forge".to_string(),
                "sacred-geometry".to_string(),
            ],
            synthesis_type: SynthesisType::CreativeExpression,
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
        opts.insert("method".to_string(), json!("letter_elimination"));
        opts.insert("include_guidance".to_string(), json!(true));
        opts
    }

    /// Prepare engine-specific options from workflow input
    pub fn prepare_engine_options(input: &CreativeExpressionInput) -> HashMap<String, HashMap<String, Value>> {
        let mut engine_opts = HashMap::new();

        // Sigil Forge options
        let mut sigil_opts = HashMap::new();
        sigil_opts.insert("intention".to_string(), json!(input.intention.clone()));
        if let Some(method) = &input.method {
            sigil_opts.insert("method".to_string(), json!(method.as_str()));
        }
        if let Some(medium) = &input.medium {
            sigil_opts.insert("medium".to_string(), json!(medium));
        }
        engine_opts.insert("sigil-forge".to_string(), sigil_opts);

        // Sacred Geometry options
        let mut geo_opts = HashMap::new();
        if let Some(form) = &input.form {
            geo_opts.insert("form".to_string(), json!(form.as_str()));
        } else {
            // Let the engine choose based on intention keywords
            geo_opts.insert("intention".to_string(), json!(input.intention.clone()));
            geo_opts.insert("auto_select".to_string(), json!(true));
        }
        geo_opts.insert("include_qualities".to_string(), json!(true));
        geo_opts.insert("include_meditation".to_string(), json!(true));
        engine_opts.insert("sacred-geometry".to_string(), geo_opts);

        engine_opts
    }

    /// Validate that required engines are present in results
    pub fn validate_results(results: &HashMap<String, EngineOutput>) -> Result<(), Vec<String>> {
        // At least one engine should succeed
        let available: Vec<&str> = ["sigil-forge", "sacred-geometry"]
            .iter()
            .filter(|e| results.contains_key(&e.to_string()))
            .copied()
            .collect();

        if available.is_empty() {
            Err(vec!["sigil-forge".to_string(), "sacred-geometry".to_string()])
        } else {
            Ok(())
        }
    }

    /// Suggest a sacred form based on intention keywords
    pub fn suggest_form(intention: &str) -> SacredForm {
        let lower = intention.to_lowercase();
        
        if lower.contains("begin") || lower.contains("start") || lower.contains("new") {
            SacredForm::SeedOfLife
        } else if lower.contains("grow") || lower.contains("evolve") || lower.contains("expand") {
            SacredForm::FibonacciSpiral
        } else if lower.contains("connect") || lower.contains("relation") || lower.contains("community") {
            SacredForm::FlowerOfLife
        } else if lower.contains("balance") || lower.contains("harmony") || lower.contains("truth") {
            SacredForm::MetatronsCube
        } else if lower.contains("manifest") || lower.contains("create") || lower.contains("abundance") {
            SacredForm::SriYantra
        } else if lower.contains("flow") || lower.contains("energy") || lower.contains("renew") {
            SacredForm::Torus
        } else if lower.contains("transform") || lower.contains("transcend") || lower.contains("ascend") {
            SacredForm::Merkaba
        } else if lower.contains("complete") || lower.contains("whole") || lower.contains("unity") {
            SacredForm::Circle
        } else if lower.contains("birth") || lower.contains("portal") || lower.contains("dual") {
            SacredForm::VesicaPiscis
        } else {
            // Default to Flower of Life for general creativity
            SacredForm::FlowerOfLife
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition() {
        let def = CreativeExpressionWorkflow::definition();
        assert_eq!(def.id, "creative-expression");
        assert_eq!(def.engine_ids.len(), 2);
        assert!(def.engine_ids.contains(&"sigil-forge".to_string()));
        assert!(def.engine_ids.contains(&"sacred-geometry".to_string()));
        assert_eq!(def.synthesis_type, SynthesisType::CreativeExpression);
        assert_eq!(def.required_phase, 1);
    }

    #[test]
    fn test_base_definition() {
        let base = CreativeExpressionWorkflow::base_definition();
        assert_eq!(base.id, "creative-expression");
        assert_eq!(base.engine_ids.len(), 2);
    }

    #[test]
    fn test_prepare_engine_options_with_all_params() {
        let input = CreativeExpressionInput {
            intention: "Manifest creative abundance".to_string(),
            form: Some(SacredForm::SriYantra),
            method: Some(SigilMethod::Geometric),
            medium: Some("visual art".to_string()),
        };

        let opts = CreativeExpressionWorkflow::prepare_engine_options(&input);
        
        assert!(opts.contains_key("sigil-forge"));
        assert_eq!(opts["sigil-forge"]["intention"], json!("Manifest creative abundance"));
        assert_eq!(opts["sigil-forge"]["method"], json!("geometric"));
        
        assert!(opts.contains_key("sacred-geometry"));
        assert_eq!(opts["sacred-geometry"]["form"], json!("sri_yantra"));
    }

    #[test]
    fn test_prepare_engine_options_minimal() {
        let input = CreativeExpressionInput {
            intention: "Express my truth".to_string(),
            form: None,
            method: None,
            medium: None,
        };

        let opts = CreativeExpressionWorkflow::prepare_engine_options(&input);
        
        assert!(opts.contains_key("sigil-forge"));
        assert!(opts.contains_key("sacred-geometry"));
        assert_eq!(opts["sacred-geometry"]["auto_select"], json!(true));
    }

    #[test]
    fn test_suggest_form() {
        assert_eq!(
            CreativeExpressionWorkflow::suggest_form("new beginning"),
            SacredForm::SeedOfLife
        );
        assert_eq!(
            CreativeExpressionWorkflow::suggest_form("grow and expand"),
            SacredForm::FibonacciSpiral
        );
        assert_eq!(
            CreativeExpressionWorkflow::suggest_form("manifest abundance"),
            SacredForm::SriYantra
        );
        assert_eq!(
            CreativeExpressionWorkflow::suggest_form("random intention"),
            SacredForm::FlowerOfLife // default
        );
    }

    #[test]
    fn test_sacred_form_qualities() {
        let form = SacredForm::SeedOfLife;
        let qualities = form.qualities();
        assert!(qualities.contains(&"creation"));
        assert!(qualities.contains(&"potential"));
    }

    #[test]
    fn test_sigil_method_str() {
        assert_eq!(SigilMethod::LetterElimination.as_str(), "letter_elimination");
        assert_eq!(SigilMethod::Geometric.as_str(), "geometric");
        assert_eq!(SigilMethod::Automatic.as_str(), "automatic");
    }
}
