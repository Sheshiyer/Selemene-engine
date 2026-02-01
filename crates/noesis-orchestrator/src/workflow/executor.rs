//! Workflow Executor — Parallel engine execution and synthesis coordination
//!
//! Executes workflows by running engines in parallel and coordinating synthesis.

use super::models::{SynthesisResult, WorkflowOutput};
use super::registry::WorkflowRegistry;
use super::synthesis::{BirthBlueprintSynthesizer, DailyPracticeSynthesizer, Synthesizer};
use super::witness::generate_workflow_witness_prompts;
use super::{ExtendedWorkflowDefinition, SynthesisType};
use crate::EngineRegistry;
use chrono::Utc;
use futures::future::join_all;
use noesis_core::{EngineError, EngineInput, EngineOutput};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn, instrument};

/// Executes workflows with parallel engine execution and synthesis
pub struct WorkflowExecutor {
    engine_registry: Arc<EngineRegistry>,
    workflow_registry: WorkflowRegistry,
}

impl WorkflowExecutor {
    /// Create a new executor with the given engine registry
    pub fn new(engine_registry: Arc<EngineRegistry>) -> Self {
        Self {
            engine_registry,
            workflow_registry: WorkflowRegistry::new(),
        }
    }

    /// Create with custom workflow registry
    pub fn with_workflow_registry(
        engine_registry: Arc<EngineRegistry>,
        workflow_registry: WorkflowRegistry,
    ) -> Self {
        Self {
            engine_registry,
            workflow_registry,
        }
    }

    /// Execute a workflow by ID
    #[instrument(skip(self, input), fields(workflow_id = %workflow_id, user_phase))]
    pub async fn execute(
        &self,
        workflow_id: &str,
        input: EngineInput,
        user_phase: u8,
    ) -> Result<WorkflowOutput, EngineError> {
        let workflow = self
            .workflow_registry
            .get(workflow_id)
            .ok_or_else(|| EngineError::WorkflowNotFound(workflow_id.to_string()))?;

        // Check phase access
        if workflow.required_phase > user_phase {
            return Err(EngineError::PhaseAccessDenied {
                required: workflow.required_phase,
                current: user_phase,
            });
        }

        self.execute_workflow(workflow, input, user_phase).await
    }

    /// Execute a workflow definition
    #[instrument(skip(self, workflow, input), fields(workflow_id = %workflow.id))]
    pub async fn execute_workflow(
        &self,
        workflow: &ExtendedWorkflowDefinition,
        input: EngineInput,
        user_phase: u8,
    ) -> Result<WorkflowOutput, EngineError> {
        let start = Instant::now();

        info!(
            workflow_id = %workflow.id,
            engine_count = workflow.engine_ids.len(),
            "Starting parallel workflow execution"
        );

        // Execute all engines in parallel
        let engine_results = self
            .execute_engines_parallel(&workflow.engine_ids, input.clone(), user_phase)
            .await;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        info!(
            workflow_id = %workflow.id,
            engines_completed = engine_results.len(),
            execution_time_ms,
            "Engine execution complete, starting synthesis"
        );

        // Synthesize results based on workflow type
        let synthesis = self.synthesize(&workflow.synthesis_type, &engine_results, &input);

        // Generate witness prompts from synthesis
        let witness_prompts = generate_workflow_witness_prompts(&synthesis, user_phase);

        Ok(WorkflowOutput {
            workflow_id: workflow.id.clone(),
            engine_results,
            synthesis,
            witness_prompts,
            execution_time_ms,
            timestamp: Utc::now(),
        })
    }

    /// Execute multiple engines in parallel
    async fn execute_engines_parallel(
        &self,
        engine_ids: &[String],
        input: EngineInput,
        user_phase: u8,
    ) -> HashMap<String, EngineOutput> {
        let futures: Vec<_> = engine_ids
            .iter()
            .map(|engine_id| {
                let engine_opt = self.engine_registry.get(engine_id);
                let input_clone = input.clone();
                let engine_id_owned = engine_id.clone();

                async move {
                    let engine = match engine_opt {
                        Some(e) => e,
                        None => {
                            warn!(engine_id = %engine_id_owned, "Engine not found, skipping");
                            return (engine_id_owned, None);
                        }
                    };

                    // Phase gate
                    let required = engine.required_phase();
                    if required > user_phase {
                        warn!(
                            engine_id = %engine_id_owned,
                            required_phase = required,
                            user_phase,
                            "Phase access denied, skipping"
                        );
                        return (engine_id_owned, None);
                    }

                    info!(engine_id = %engine_id_owned, "Executing engine");
                    match engine.calculate(input_clone).await {
                        Ok(output) => (engine_id_owned, Some(output)),
                        Err(e) => {
                            warn!(engine_id = %engine_id_owned, error = %e, "Engine failed");
                            (engine_id_owned, None)
                        }
                    }
                }
            })
            .collect();

        // Run all engines concurrently
        let results = join_all(futures).await;

        // Collect successful results
        results
            .into_iter()
            .filter_map(|(id, opt)| opt.map(|output| (id, output)))
            .collect()
    }

    /// Synthesize engine results based on workflow type
    fn synthesize(
        &self,
        synthesis_type: &SynthesisType,
        results: &HashMap<String, EngineOutput>,
        input: &EngineInput,
    ) -> SynthesisResult {
        match synthesis_type {
            SynthesisType::BirthBlueprint => {
                BirthBlueprintSynthesizer::synthesize(results, input)
            }
            SynthesisType::DailyPractice => {
                DailyPracticeSynthesizer::synthesize(results, input)
            }
            // TODO: Implement other synthesizers
            _ => self.generic_synthesis(results),
        }
    }

    /// Generic synthesis for unimplemented types
    fn generic_synthesis(&self, results: &HashMap<String, EngineOutput>) -> SynthesisResult {
        let engine_names: Vec<String> = results.keys().cloned().collect();
        
        SynthesisResult {
            themes: Vec::new(),
            alignments: Vec::new(),
            tensions: Vec::new(),
            summary: format!(
                "Synthesis from {} engines: {}. Full synthesis implementation pending.",
                results.len(),
                engine_names.join(", ")
            ),
        }
    }

    /// Get the workflow registry
    pub fn workflow_registry(&self) -> &WorkflowRegistry {
        &self.workflow_registry
    }

    /// Get mutable workflow registry for registration
    pub fn workflow_registry_mut(&mut self) -> &mut WorkflowRegistry {
        &mut self.workflow_registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use noesis_core::{CalculationMetadata, Precision, ValidationResult};

    /// Local mock engine for executor tests
    struct MockEngine {
        id: String,
        phase: u8,
    }

    impl MockEngine {
        fn new(id: &str, phase: u8) -> Self {
            Self {
                id: id.to_string(),
                phase,
            }
        }
    }

    #[async_trait]
    impl noesis_core::ConsciousnessEngine for MockEngine {
        fn engine_id(&self) -> &str {
            &self.id
        }

        fn engine_name(&self) -> &str {
            &self.id
        }

        fn required_phase(&self) -> u8 {
            self.phase
        }

        async fn calculate(&self, _input: EngineInput) -> Result<EngineOutput, noesis_core::EngineError> {
            Ok(EngineOutput {
                engine_id: self.id.clone(),
                result: serde_json::json!({ "mock": true }),
                witness_prompt: format!("Witness from {}", self.id),
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

        async fn validate(&self, _output: &EngineOutput) -> Result<ValidationResult, noesis_core::EngineError> {
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

    fn test_input() -> EngineInput {
        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: Precision::Standard,
            options: HashMap::new(),
        }
    }

    fn setup_executor_with_mocks() -> WorkflowExecutor {
        let mut registry = EngineRegistry::new();
        registry.register(Arc::new(MockEngine::new("numerology", 0)));
        registry.register(Arc::new(MockEngine::new("human-design", 0)));
        registry.register(Arc::new(MockEngine::new("vimshottari", 0)));
        registry.register(Arc::new(MockEngine::new("panchanga", 0)));
        registry.register(Arc::new(MockEngine::new("vedic-clock", 0)));
        registry.register(Arc::new(MockEngine::new("biorhythm", 0)));

        WorkflowExecutor::new(Arc::new(registry))
    }

    #[tokio::test]
    async fn execute_birth_blueprint() {
        let executor = setup_executor_with_mocks();
        let result = executor.execute("birth-blueprint", test_input(), 5).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.workflow_id, "birth-blueprint");
        assert_eq!(output.engine_results.len(), 3);
    }

    #[tokio::test]
    async fn execute_daily_practice() {
        let executor = setup_executor_with_mocks();
        let result = executor.execute("daily-practice", test_input(), 5).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.workflow_id, "daily-practice");
        assert_eq!(output.engine_results.len(), 3);
    }

    #[tokio::test]
    async fn execute_nonexistent_workflow() {
        let executor = setup_executor_with_mocks();
        let result = executor.execute("nonexistent", test_input(), 5).await;

        assert!(matches!(result, Err(EngineError::WorkflowNotFound(_))));
    }

    #[tokio::test]
    async fn execute_phase_denied() {
        let executor = setup_executor_with_mocks();
        // full-spectrum requires phase 3
        let result = executor.execute("full-spectrum", test_input(), 1).await;

        assert!(matches!(
            result,
            Err(EngineError::PhaseAccessDenied { required: 3, current: 1 })
        ));
    }
}
