//! Workflow Registry — Pre-defined workflow definitions
//!
//! Manages the 6 canonical workflows plus any custom-registered workflows.

use super::{ExtendedWorkflowDefinition, SynthesisType};
use std::collections::HashMap;

/// Registry of all available workflow definitions
pub struct WorkflowRegistry {
    workflows: HashMap<String, ExtendedWorkflowDefinition>,
}

impl WorkflowRegistry {
    /// Create a new registry pre-populated with the 6 canonical workflows
    pub fn new() -> Self {
        let mut registry = Self {
            workflows: HashMap::new(),
        };
        registry.register_default_workflows();
        registry
    }

    /// Register the 6 canonical workflows
    fn register_default_workflows(&mut self) {
        // Birth Blueprint: Core identity mapping through birth data
        self.register(ExtendedWorkflowDefinition {
            id: "birth-blueprint".into(),
            name: "Birth Blueprint".into(),
            description: "Core identity mapping through birth data analysis".into(),
            required_phase: 0,
            engine_ids: vec![
                "numerology".into(),
                "human-design".into(),
                "vimshottari".into(),
            ],
            synthesis_type: SynthesisType::BirthBlueprint,
            default_options: HashMap::new(),
        });

        // Daily Practice: Temporal optimization for daily activities
        self.register(ExtendedWorkflowDefinition {
            id: "daily-practice".into(),
            name: "Daily Practice".into(),
            description: "Daily rhythm optimization through temporal analysis".into(),
            required_phase: 0,
            engine_ids: vec![
                "panchanga".into(),
                "vedic-clock".into(),
                "biorhythm".into(),
            ],
            synthesis_type: SynthesisType::DailyPractice,
            default_options: HashMap::new(),
        });

        // Decision Support: Multi-perspective decision guidance
        self.register(ExtendedWorkflowDefinition {
            id: "decision-support".into(),
            name: "Decision Support".into(),
            description: "Multi-system decision mirrors for clarity".into(),
            required_phase: 1,
            engine_ids: vec![
                "tarot".into(),
                "i-ching".into(),
                "human-design".into(),
            ],
            synthesis_type: SynthesisType::DecisionSupport,
            default_options: HashMap::new(),
        });

        // Self-Inquiry: Deep shadow work and type exploration
        self.register(ExtendedWorkflowDefinition {
            id: "self-inquiry".into(),
            name: "Self-Inquiry".into(),
            description: "Deep self-consciousness exploration and shadow work".into(),
            required_phase: 2,
            engine_ids: vec![
                "gene-keys".into(),
                "enneagram".into(),
            ],
            synthesis_type: SynthesisType::SelfInquiry,
            default_options: HashMap::new(),
        });

        // Creative Expression: Generative and aesthetic guidance
        self.register(ExtendedWorkflowDefinition {
            id: "creative-expression".into(),
            name: "Creative Expression".into(),
            description: "Creative and aesthetic exploration through symbols".into(),
            required_phase: 1,
            engine_ids: vec![
                "sigil-forge".into(),
                "sacred-geometry".into(),
            ],
            synthesis_type: SynthesisType::CreativeExpression,
            default_options: HashMap::new(),
        });

        // Full Spectrum: All 11+ engines integrated
        self.register(ExtendedWorkflowDefinition {
            id: "full-spectrum".into(),
            name: "Full Spectrum".into(),
            description: "Complete integration of all consciousness engines".into(),
            required_phase: 3,
            engine_ids: vec![
                // Rust engines
                "numerology".into(),
                "human-design".into(),
                "vimshottari".into(),
                "panchanga".into(),
                "vedic-clock".into(),
                "biorhythm".into(),
                "gene-keys".into(),
                "biofield".into(),
                "face-reading".into(),
                // TS engines via bridge
                "tarot".into(),
                "i-ching".into(),
                "enneagram".into(),
                "sacred-geometry".into(),
                "sigil-forge".into(),
            ],
            synthesis_type: SynthesisType::FullSpectrum,
            default_options: HashMap::new(),
        });
    }

    /// Register a workflow definition
    pub fn register(&mut self, workflow: ExtendedWorkflowDefinition) {
        self.workflows.insert(workflow.id.clone(), workflow);
    }

    /// Get a workflow by ID
    pub fn get(&self, id: &str) -> Option<&ExtendedWorkflowDefinition> {
        self.workflows.get(id)
    }

    /// List all registered workflows
    pub fn list(&self) -> Vec<&ExtendedWorkflowDefinition> {
        let mut workflows: Vec<_> = self.workflows.values().collect();
        workflows.sort_by_key(|w| &w.id);
        workflows
    }

    /// List workflows accessible at a given phase
    pub fn list_for_phase(&self, phase: u8) -> Vec<&ExtendedWorkflowDefinition> {
        let mut workflows: Vec<_> = self
            .workflows
            .values()
            .filter(|w| w.required_phase <= phase)
            .collect();
        workflows.sort_by_key(|w| &w.id);
        workflows
    }

    /// Check if a workflow exists
    pub fn contains(&self, id: &str) -> bool {
        self.workflows.contains_key(id)
    }

    /// Number of registered workflows
    pub fn len(&self) -> usize {
        self.workflows.len()
    }

    /// Whether the registry is empty
    pub fn is_empty(&self) -> bool {
        self.workflows.is_empty()
    }
}

impl Default for WorkflowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_registry_has_six_workflows() {
        let registry = WorkflowRegistry::new();
        assert_eq!(registry.len(), 6);
    }

    #[test]
    fn get_birth_blueprint() {
        let registry = WorkflowRegistry::new();
        let wf = registry.get("birth-blueprint").unwrap();
        
        assert_eq!(wf.name, "Birth Blueprint");
        assert_eq!(wf.engine_ids, vec!["numerology", "human-design", "vimshottari"]);
        assert_eq!(wf.synthesis_type, SynthesisType::BirthBlueprint);
    }

    #[test]
    fn get_daily_practice() {
        let registry = WorkflowRegistry::new();
        let wf = registry.get("daily-practice").unwrap();
        
        assert_eq!(wf.engine_ids, vec!["panchanga", "vedic-clock", "biorhythm"]);
        assert_eq!(wf.synthesis_type, SynthesisType::DailyPractice);
    }

    #[test]
    fn list_for_phase_filters() {
        let registry = WorkflowRegistry::new();
        
        // Phase 0 should get birth-blueprint and daily-practice
        let phase0 = registry.list_for_phase(0);
        assert_eq!(phase0.len(), 2);
        
        // Phase 3 should get all
        let phase3 = registry.list_for_phase(3);
        assert_eq!(phase3.len(), 6);
    }

    #[test]
    fn full_spectrum_has_all_engines() {
        let registry = WorkflowRegistry::new();
        let wf = registry.get("full-spectrum").unwrap();
        
        assert!(wf.engine_ids.len() >= 11);
        assert!(wf.engine_ids.contains(&"numerology".to_string()));
        assert!(wf.engine_ids.contains(&"tarot".to_string()));
    }

    #[test]
    fn custom_workflow_registration() {
        let mut registry = WorkflowRegistry::new();
        
        registry.register(ExtendedWorkflowDefinition {
            id: "custom".into(),
            name: "Custom Workflow".into(),
            description: "A custom test workflow".into(),
            required_phase: 1,
            engine_ids: vec!["numerology".into()],
            synthesis_type: SynthesisType::BirthBlueprint,
            default_options: HashMap::new(),
        });

        assert_eq!(registry.len(), 7);
        assert!(registry.contains("custom"));
    }
}
