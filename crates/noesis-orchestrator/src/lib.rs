//! Noesis Orchestrator -- Multi-engine workflow execution
//!
//! Coordinates parallel execution of multiple consciousness engines
//! and synthesizes their outputs into unified workflow results.
//!
//! # Architecture
//!
//! The orchestrator has two main components:
//!
//! - **`EngineRegistry`** -- stores `Arc<dyn ConsciousnessEngine>` trait objects
//!   keyed by engine ID and supports phase-gated lookups.
//!
//! - **`WorkflowOrchestrator`** -- owns a registry and a set of predefined
//!   `WorkflowDefinition`s. It executes single engines or entire workflows
//!   (running all constituent engines in parallel via `futures::future::join_all`).
//!
//! Phase gating is enforced at execution time: an engine will only run if
//! `engine.required_phase() <= user_phase`. Otherwise `EngineError::PhaseAccessDenied`
//! is returned.

pub use noesis_core::{
    ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    WorkflowDefinition, WorkflowResult,
};

use chrono::Utc;
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn, instrument};

// ---------------------------------------------------------------------------
// EngineRegistry
// ---------------------------------------------------------------------------

/// Thread-safe registry of consciousness engine trait objects.
///
/// Engines are stored behind `Arc` so they can be shared across
/// concurrent workflow executions without cloning.
pub struct EngineRegistry {
    engines: HashMap<String, Arc<dyn ConsciousnessEngine>>,
}

impl EngineRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            engines: HashMap::new(),
        }
    }

    /// Register an engine. The engine's `engine_id()` is used as the key.
    /// If an engine with the same ID already exists it is replaced.
    pub fn register(&mut self, engine: Arc<dyn ConsciousnessEngine>) {
        let id = engine.engine_id().to_string();
        info!(engine_id = %id, "Registering engine");
        self.engines.insert(id, engine);
    }

    /// Retrieve an engine by ID.
    pub fn get(&self, engine_id: &str) -> Option<Arc<dyn ConsciousnessEngine>> {
        self.engines.get(engine_id).cloned()
    }

    /// List all registered engine IDs (sorted for deterministic output).
    pub fn list(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.engines.keys().map(|s| s.as_str()).collect();
        ids.sort();
        ids
    }

    /// List engine IDs that are accessible at the given consciousness phase.
    pub fn list_for_phase(&self, phase: u8) -> Vec<&str> {
        let mut ids: Vec<&str> = self
            .engines
            .iter()
            .filter(|(_, engine)| engine.required_phase() <= phase)
            .map(|(id, _)| id.as_str())
            .collect();
        ids.sort();
        ids
    }

    /// Total number of registered engines.
    pub fn len(&self) -> usize {
        self.engines.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.engines.is_empty()
    }
}

impl Default for EngineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// WorkflowOrchestrator
// ---------------------------------------------------------------------------

/// Top-level orchestrator that coordinates multi-engine workflow execution.
///
/// Holds a registry of engines and a map of predefined workflow definitions.
/// Workflows execute all their constituent engines concurrently using
/// `futures::future::join_all`.
pub struct WorkflowOrchestrator {
    registry: EngineRegistry,
    workflows: HashMap<String, WorkflowDefinition>,
}

impl WorkflowOrchestrator {
    /// Create a new orchestrator pre-loaded with the 6 canonical workflows.
    pub fn new() -> Self {
        let workflows = Self::default_workflows();
        Self {
            registry: EngineRegistry::new(),
            workflows,
        }
    }

    /// Register a consciousness engine with the internal registry.
    pub fn register_engine(&mut self, engine: Arc<dyn ConsciousnessEngine>) {
        self.registry.register(engine);
    }

    /// Register a custom workflow definition.
    pub fn register_workflow(&mut self, workflow: WorkflowDefinition) {
        info!(workflow_id = %workflow.id, "Registering workflow");
        self.workflows.insert(workflow.id.clone(), workflow);
    }

    // -- Single-engine execution -------------------------------------------

    /// Execute a single engine by ID.
    ///
    /// Phase gating is enforced: if the engine requires a higher phase than
    /// `user_phase`, `EngineError::PhaseAccessDenied` is returned.
    #[instrument(skip(self, input), fields(engine_id = %engine_id, user_phase))]
    pub async fn execute_engine(
        &self,
        engine_id: &str,
        input: EngineInput,
        user_phase: u8,
    ) -> Result<EngineOutput, EngineError> {
        let engine = self
            .registry
            .get(engine_id)
            .ok_or_else(|| EngineError::EngineNotFound(engine_id.to_string()))?;

        // Phase gate
        let required = engine.required_phase();
        if required > user_phase {
            warn!(
                engine_id,
                required_phase = required,
                user_phase,
                "Phase access denied"
            );
            return Err(EngineError::PhaseAccessDenied {
                required,
                current: user_phase,
            });
        }

        info!(engine_id, "Executing engine");
        engine.calculate(input).await
    }

    // -- Workflow execution ------------------------------------------------

    /// Execute a predefined workflow (all engines in parallel).
    ///
    /// Each engine in the workflow runs concurrently. If an individual engine
    /// fails or is phase-gated, its error is logged but the overall workflow
    /// still succeeds -- the failed engine is simply omitted from the results.
    #[instrument(skip(self, input), fields(workflow_id = %workflow_id, user_phase))]
    pub async fn execute_workflow(
        &self,
        workflow_id: &str,
        input: EngineInput,
        user_phase: u8,
    ) -> Result<WorkflowResult, EngineError> {
        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| EngineError::WorkflowNotFound(workflow_id.to_string()))?;

        info!(
            workflow_id,
            engine_count = workflow.engine_ids.len(),
            "Starting workflow execution"
        );

        let start = Instant::now();

        // Build futures for all engines in the workflow.
        let futures: Vec<_> = workflow
            .engine_ids
            .iter()
            .map(|eid| {
                let engine_opt = self.registry.get(eid);
                let input_clone = input.clone();
                let eid_owned = eid.clone();

                async move {
                    let engine = match engine_opt {
                        Some(e) => e,
                        None => {
                            warn!(engine_id = %eid_owned, "Engine not found in registry, skipping");
                            let err = EngineError::EngineNotFound(eid_owned.clone());
                            return (
                                eid_owned,
                                Err(err),
                            );
                        }
                    };

                    // Phase gate
                    let required = engine.required_phase();
                    if required > user_phase {
                        warn!(
                            engine_id = %eid_owned,
                            required_phase = required,
                            user_phase,
                            "Phase access denied, skipping engine"
                        );
                        return (
                            eid_owned,
                            Err(EngineError::PhaseAccessDenied {
                                required,
                                current: user_phase,
                            }),
                        );
                    }

                    info!(engine_id = %eid_owned, "Executing engine in workflow");
                    let result = engine.calculate(input_clone).await;
                    (eid_owned, result)
                }
            })
            .collect();

        // Run all engines concurrently.
        let results = join_all(futures).await;

        // Collect successful outputs; log failures.
        let mut engine_outputs = HashMap::new();
        for (eid, result) in results {
            match result {
                Ok(output) => {
                    info!(engine_id = %eid, "Engine completed successfully");
                    engine_outputs.insert(eid, output);
                }
                Err(e) => {
                    warn!(engine_id = %eid, error = %e, "Engine failed, omitting from results");
                }
            }
        }

        let total_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        info!(
            workflow_id,
            engines_succeeded = engine_outputs.len(),
            total_time_ms,
            "Workflow execution complete"
        );

        Ok(WorkflowResult {
            workflow_id: workflow_id.to_string(),
            engine_outputs,
            synthesis: None, // Synthesis is a future enhancement
            total_time_ms,
            timestamp: Utc::now(),
        })
    }

    // -- Query methods -----------------------------------------------------

    /// List all predefined workflow definitions.
    pub fn list_workflows(&self) -> Vec<&WorkflowDefinition> {
        let mut wfs: Vec<&WorkflowDefinition> = self.workflows.values().collect();
        wfs.sort_by_key(|w| &w.id);
        wfs
    }

    /// List all registered engine IDs.
    pub fn list_engines(&self) -> Vec<String> {
        self.registry.list().iter().map(|s| s.to_string()).collect()
    }

    /// Get a specific workflow definition by ID.
    pub fn get_workflow(&self, workflow_id: &str) -> Option<&WorkflowDefinition> {
        self.workflows.get(workflow_id)
    }

    /// Get access to the underlying engine registry.
    pub fn registry(&self) -> &EngineRegistry {
        &self.registry
    }

    // -- Default workflows ------------------------------------------------

    fn default_workflows() -> HashMap<String, WorkflowDefinition> {
        let definitions = vec![
            WorkflowDefinition {
                id: "birth-blueprint".into(),
                name: "Birth Blueprint".into(),
                description: "Core identity mapping through birth data".into(),
                engine_ids: vec![
                    "numerology".into(),
                    "human-design".into(),
                    "gene-keys".into(),
                ],
            },
            WorkflowDefinition {
                id: "daily-practice".into(),
                name: "Daily Practice".into(),
                description: "Daily rhythm and awareness tools".into(),
                engine_ids: vec![
                    "panchanga".into(),
                    "vedic-clock".into(),
                    "biorhythm".into(),
                ],
            },
            WorkflowDefinition {
                id: "decision-support".into(),
                name: "Decision Support".into(),
                description: "Multi-system decision mirrors".into(),
                engine_ids: vec![
                    "tarot".into(),
                    "i-ching".into(),
                    "human-design".into(),
                ],
            },
            WorkflowDefinition {
                id: "self-inquiry".into(),
                name: "Self-Inquiry".into(),
                description: "Deep self-consciousness exploration".into(),
                engine_ids: vec![
                    "gene-keys".into(),
                    "enneagram".into(),
                ],
            },
            WorkflowDefinition {
                id: "creative-expression".into(),
                name: "Creative Expression".into(),
                description: "Creative and aesthetic exploration".into(),
                engine_ids: vec![
                    "sigil-forge".into(),
                    "sacred-geometry".into(),
                ],
            },
            WorkflowDefinition {
                id: "full-spectrum".into(),
                name: "Full Spectrum".into(),
                description: "All available engines".into(),
                engine_ids: vec![
                    "numerology".into(),
                    "human-design".into(),
                    "biorhythm".into(),
                    "panchanga".into(),
                    "vimshottari".into(),
                    "gene-keys".into(),
                    "vedic-clock".into(),
                    "biofield".into(),
                    "face-reading".into(),
                    "tarot".into(),
                    "i-ching".into(),
                    "enneagram".into(),
                    "sacred-geometry".into(),
                    "sigil-forge".into(),
                ],
            },
        ];

        definitions
            .into_iter()
            .map(|w| (w.id.clone(), w))
            .collect()
    }
}

impl Default for WorkflowOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use noesis_core::{CalculationMetadata, ValidationResult};

    // -- Mock engine for testing ------------------------------------------

    /// A simple mock engine that returns a predictable output.
    struct MockEngine {
        id: String,
        name: String,
        phase: u8,
        /// If true, `calculate` will return an error.
        should_fail: bool,
    }

    impl MockEngine {
        fn new(id: &str, phase: u8) -> Self {
            Self {
                id: id.to_string(),
                name: format!("Mock {}", id),
                phase,
                should_fail: false,
            }
        }

        fn failing(id: &str, phase: u8) -> Self {
            Self {
                id: id.to_string(),
                name: format!("Failing Mock {}", id),
                phase,
                should_fail: true,
            }
        }
    }

    #[async_trait]
    impl ConsciousnessEngine for MockEngine {
        fn engine_id(&self) -> &str {
            &self.id
        }

        fn engine_name(&self) -> &str {
            &self.name
        }

        fn required_phase(&self) -> u8 {
            self.phase
        }

        async fn calculate(&self, _input: EngineInput) -> Result<EngineOutput, EngineError> {
            if self.should_fail {
                return Err(EngineError::CalculationError(format!(
                    "{} intentionally failed",
                    self.id
                )));
            }

            Ok(EngineOutput {
                engine_id: self.id.clone(),
                result: serde_json::json!({ "mock": true, "engine": self.id }),
                witness_prompt: format!("Witness prompt from {}", self.id),
                consciousness_level: self.phase,
                metadata: CalculationMetadata {
                    calculation_time_ms: 1.0,
                    backend: "mock".to_string(),
                    precision_achieved: "standard".to_string(),
                    cached: false,
                    timestamp: Utc::now(),
                },
            })
        }

        async fn validate(&self, _output: &EngineOutput) -> Result<ValidationResult, EngineError> {
            Ok(ValidationResult {
                valid: true,
                confidence: 1.0,
                messages: vec![],
            })
        }

        fn cache_key(&self, _input: &EngineInput) -> String {
            format!("mock-{}", self.id)
        }
    }

    /// Helper to build a standard test input.
    fn test_input() -> EngineInput {
        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: noesis_core::Precision::Standard,
            options: HashMap::new(),
        }
    }

    // -- EngineRegistry tests ---------------------------------------------

    #[test]
    fn registry_starts_empty() {
        let registry = EngineRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.list().is_empty());
    }

    #[test]
    fn registry_register_and_get() {
        let mut registry = EngineRegistry::new();
        let engine: Arc<dyn ConsciousnessEngine> = Arc::new(MockEngine::new("numerology", 0));
        registry.register(engine);

        assert_eq!(registry.len(), 1);
        assert!(registry.get("numerology").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn registry_list_engines() {
        let mut registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("beta", 0)));
        registry.register(Arc::new(MockEngine::new("alpha", 1)));
        registry.register(Arc::new(MockEngine::new("gamma", 2)));

        let ids = registry.list();
        assert_eq!(ids, vec!["alpha", "beta", "gamma"]); // sorted
    }

    #[test]
    fn registry_list_for_phase() {
        let mut registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("phase0", 0)));
        registry.register(Arc::new(MockEngine::new("phase1", 1)));
        registry.register(Arc::new(MockEngine::new("phase2", 2)));
        registry.register(Arc::new(MockEngine::new("phase3", 3)));

        let phase1_engines = registry.list_for_phase(1);
        assert_eq!(phase1_engines, vec!["phase0", "phase1"]);

        let phase3_engines = registry.list_for_phase(3);
        assert_eq!(phase3_engines, vec!["phase0", "phase1", "phase2", "phase3"]);

        let phase0_engines = registry.list_for_phase(0);
        assert_eq!(phase0_engines, vec!["phase0"]);
    }

    #[test]
    fn registry_replace_engine() {
        let mut registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("numerology", 0)));
        registry.register(Arc::new(MockEngine::new("numerology", 2)));

        assert_eq!(registry.len(), 1);
        let engine = registry.get("numerology").unwrap();
        assert_eq!(engine.required_phase(), 2);
    }

    // -- WorkflowOrchestrator tests ----------------------------------------

    #[test]
    fn orchestrator_has_default_workflows() {
        let orchestrator = WorkflowOrchestrator::new();
        let workflows = orchestrator.list_workflows();
        assert_eq!(workflows.len(), 6);

        let ids: Vec<&str> = workflows.iter().map(|w| w.id.as_str()).collect();
        assert!(ids.contains(&"birth-blueprint"));
        assert!(ids.contains(&"daily-practice"));
        assert!(ids.contains(&"decision-support"));
        assert!(ids.contains(&"self-inquiry"));
        assert!(ids.contains(&"creative-expression"));
        assert!(ids.contains(&"full-spectrum"));
    }

    #[test]
    fn orchestrator_get_workflow() {
        let orchestrator = WorkflowOrchestrator::new();
        let wf = orchestrator.get_workflow("birth-blueprint").unwrap();
        assert_eq!(wf.name, "Birth Blueprint");
        assert_eq!(wf.engine_ids, vec!["numerology", "human-design", "gene-keys"]);
    }

    #[test]
    fn orchestrator_register_engine() {
        let mut orchestrator = WorkflowOrchestrator::new();
        orchestrator.register_engine(Arc::new(MockEngine::new("numerology", 0)));
        let engines = orchestrator.list_engines();
        assert_eq!(engines, vec!["numerology"]);
    }

    #[tokio::test]
    async fn execute_single_engine_success() {
        let mut orchestrator = WorkflowOrchestrator::new();
        orchestrator.register_engine(Arc::new(MockEngine::new("numerology", 0)));

        let output = orchestrator
            .execute_engine("numerology", test_input(), 0)
            .await
            .unwrap();

        assert_eq!(output.engine_id, "numerology");
    }

    #[tokio::test]
    async fn execute_single_engine_not_found() {
        let orchestrator = WorkflowOrchestrator::new();
        let result = orchestrator
            .execute_engine("nonexistent", test_input(), 5)
            .await;

        assert!(matches!(result, Err(EngineError::EngineNotFound(_))));
    }

    #[tokio::test]
    async fn execute_single_engine_phase_denied() {
        let mut orchestrator = WorkflowOrchestrator::new();
        orchestrator.register_engine(Arc::new(MockEngine::new("advanced", 3)));

        let result = orchestrator
            .execute_engine("advanced", test_input(), 1)
            .await;

        assert!(matches!(
            result,
            Err(EngineError::PhaseAccessDenied {
                required: 3,
                current: 1
            })
        ));
    }

    #[tokio::test]
    async fn execute_workflow_success() {
        let mut orchestrator = WorkflowOrchestrator::new();
        orchestrator.register_engine(Arc::new(MockEngine::new("numerology", 0)));
        orchestrator.register_engine(Arc::new(MockEngine::new("human-design", 0)));
        orchestrator.register_engine(Arc::new(MockEngine::new("gene-keys", 1)));

        let result = orchestrator
            .execute_workflow("birth-blueprint", test_input(), 1)
            .await
            .unwrap();

        assert_eq!(result.workflow_id, "birth-blueprint");
        assert_eq!(result.engine_outputs.len(), 3);
        assert!(result.engine_outputs.contains_key("numerology"));
        assert!(result.engine_outputs.contains_key("human-design"));
        assert!(result.engine_outputs.contains_key("gene-keys"));
        assert!(result.total_time_ms >= 0.0);
    }

    #[tokio::test]
    async fn execute_workflow_not_found() {
        let orchestrator = WorkflowOrchestrator::new();
        let result = orchestrator
            .execute_workflow("nonexistent", test_input(), 5)
            .await;

        assert!(matches!(result, Err(EngineError::WorkflowNotFound(_))));
    }

    #[tokio::test]
    async fn execute_workflow_phase_gates_individual_engines() {
        let mut orchestrator = WorkflowOrchestrator::new();
        orchestrator.register_engine(Arc::new(MockEngine::new("numerology", 0)));
        orchestrator.register_engine(Arc::new(MockEngine::new("human-design", 0)));
        orchestrator.register_engine(Arc::new(MockEngine::new("gene-keys", 3))); // requires phase 3

        // User is phase 1 -- gene-keys should be omitted
        let result = orchestrator
            .execute_workflow("birth-blueprint", test_input(), 1)
            .await
            .unwrap();

        assert_eq!(result.engine_outputs.len(), 2);
        assert!(result.engine_outputs.contains_key("numerology"));
        assert!(result.engine_outputs.contains_key("human-design"));
        assert!(!result.engine_outputs.contains_key("gene-keys"));
    }

    #[tokio::test]
    async fn execute_workflow_handles_engine_failure() {
        let mut orchestrator = WorkflowOrchestrator::new();
        orchestrator.register_engine(Arc::new(MockEngine::new("numerology", 0)));
        orchestrator.register_engine(Arc::new(MockEngine::failing("human-design", 0)));
        orchestrator.register_engine(Arc::new(MockEngine::new("gene-keys", 0)));

        let result = orchestrator
            .execute_workflow("birth-blueprint", test_input(), 5)
            .await
            .unwrap();

        // human-design failed but workflow still succeeds
        assert_eq!(result.engine_outputs.len(), 2);
        assert!(result.engine_outputs.contains_key("numerology"));
        assert!(!result.engine_outputs.contains_key("human-design"));
        assert!(result.engine_outputs.contains_key("gene-keys"));
    }

    #[tokio::test]
    async fn execute_workflow_missing_engine_skipped() {
        let mut orchestrator = WorkflowOrchestrator::new();
        // Only register numerology; human-design and gene-keys are missing
        orchestrator.register_engine(Arc::new(MockEngine::new("numerology", 0)));

        let result = orchestrator
            .execute_workflow("birth-blueprint", test_input(), 5)
            .await
            .unwrap();

        assert_eq!(result.engine_outputs.len(), 1);
        assert!(result.engine_outputs.contains_key("numerology"));
    }

    #[test]
    fn full_spectrum_has_all_engines() {
        let orchestrator = WorkflowOrchestrator::new();
        let wf = orchestrator.get_workflow("full-spectrum").unwrap();
        assert_eq!(wf.engine_ids.len(), 14);
    }

    #[test]
    fn register_custom_workflow() {
        let mut orchestrator = WorkflowOrchestrator::new();
        orchestrator.register_workflow(WorkflowDefinition {
            id: "custom".into(),
            name: "Custom".into(),
            description: "A custom workflow".into(),
            engine_ids: vec!["numerology".into()],
        });

        assert!(orchestrator.get_workflow("custom").is_some());
        assert_eq!(orchestrator.list_workflows().len(), 7);
    }
}
