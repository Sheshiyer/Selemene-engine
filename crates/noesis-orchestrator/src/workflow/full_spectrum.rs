//! Full Spectrum Workflow — Complete self-portrait across all consciousness systems
//!
//! Executes all 11+ engines in parallel and synthesizes a comprehensive view
//! of the user's consciousness landscape.

use crate::{ConsciousnessEngine, EngineError, EngineInput, EngineOutput};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{info, warn};

/// Categories for organizing engine outputs in synthesis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineCategory {
    /// Fixed natal patterns: HD, Gene Keys, Numerology, Enneagram
    Natal,
    /// Time-based cycles: Panchanga, VedicClock, Biorhythm, Vimshottari
    Temporal,
    /// Archetypal guidance: Tarot, I-Ching
    Archetypal,
    /// Body-based: Biofield, Face Reading
    Somatic,
    /// Generative: Sacred Geometry, Sigil Forge
    Creative,
}

impl EngineCategory {
    /// Get category for an engine ID
    pub fn from_engine_id(engine_id: &str) -> Self {
        match engine_id {
            "human-design" | "gene-keys" | "numerology" | "enneagram" => Self::Natal,
            "panchanga" | "vedic-clock" | "biorhythm" | "vimshottari" => Self::Temporal,
            "tarot" | "i-ching" => Self::Archetypal,
            "biofield" | "face-reading" => Self::Somatic,
            "sacred-geometry" | "sigil-forge" => Self::Creative,
            _ => Self::Natal, // Default for unknown engines
        }
    }

    /// Get all engine IDs belonging to this category
    pub fn engine_ids(&self) -> &'static [&'static str] {
        match self {
            Self::Natal => &["human-design", "gene-keys", "numerology", "enneagram"],
            Self::Temporal => &["panchanga", "vedic-clock", "biorhythm", "vimshottari"],
            Self::Archetypal => &["tarot", "i-ching"],
            Self::Somatic => &["biofield", "face-reading"],
            Self::Creative => &["sacred-geometry", "sigil-forge"],
        }
    }
}

/// Result from a single engine execution with timing and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineExecutionResult {
    pub engine_id: String,
    pub category: EngineCategory,
    pub output: Option<EngineOutput>,
    pub error: Option<String>,
    pub execution_time_ms: f64,
    pub is_stub: bool,
}

/// Complete result from full spectrum workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSpectrumResult {
    /// Unique execution ID
    pub execution_id: String,
    /// Results organized by category
    pub by_category: HashMap<EngineCategory, Vec<EngineExecutionResult>>,
    /// All successful engine outputs
    pub successful_outputs: HashMap<String, EngineOutput>,
    /// Failed engines with error messages
    pub failed_engines: HashMap<String, String>,
    /// Total execution time (wall clock)
    pub total_time_ms: f64,
    /// Number of engines attempted
    pub engines_attempted: usize,
    /// Number of engines succeeded
    pub engines_succeeded: usize,
    /// Timestamp of execution
    pub timestamp: DateTime<Utc>,
}

/// Configuration for full spectrum execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullSpectrumConfig {
    /// Timeout for individual engine execution
    pub engine_timeout: Duration,
    /// Whether to include stub/placeholder engines
    pub include_stubs: bool,
    /// User's consciousness phase level
    pub user_phase: u8,
    /// Optional question/intention to guide synthesis
    pub question: Option<String>,
}

impl Default for FullSpectrumConfig {
    fn default() -> Self {
        Self {
            engine_timeout: Duration::from_secs(5),
            include_stubs: true,
            user_phase: 5, // Maximum access by default
            question: None,
        }
    }
}

/// Full spectrum workflow executor
pub struct FullSpectrumWorkflow {
    engines: HashMap<String, Arc<dyn ConsciousnessEngine>>,
    config: FullSpectrumConfig,
}

impl FullSpectrumWorkflow {
    /// Create a new full spectrum workflow with the given engines
    pub fn new(engines: HashMap<String, Arc<dyn ConsciousnessEngine>>) -> Self {
        Self {
            engines,
            config: FullSpectrumConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(
        engines: HashMap<String, Arc<dyn ConsciousnessEngine>>,
        config: FullSpectrumConfig,
    ) -> Self {
        Self { engines, config }
    }

    /// Register an additional engine
    pub fn register_engine(&mut self, engine: Arc<dyn ConsciousnessEngine>) {
        let id = engine.engine_id().to_string();
        self.engines.insert(id, engine);
    }

    /// Get list of all available engine IDs
    pub fn available_engines(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.engines.keys().map(|s| s.as_str()).collect();
        ids.sort();
        ids
    }

    /// Execute a single engine with timeout and error handling
    async fn execute_with_timeout(
        &self,
        engine: Arc<dyn ConsciousnessEngine>,
        input: EngineInput,
    ) -> EngineExecutionResult {
        let engine_id = engine.engine_id().to_string();
        let category = EngineCategory::from_engine_id(&engine_id);
        let start = Instant::now();

        // Check phase access
        let required_phase = engine.required_phase();
        if required_phase > self.config.user_phase {
            return EngineExecutionResult {
                engine_id,
                category,
                output: None,
                error: Some(format!(
                    "Phase access denied: requires {}, user at {}",
                    required_phase, self.config.user_phase
                )),
                execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                is_stub: false,
            };
        }

        // Execute with timeout
        let result = timeout(self.config.engine_timeout, engine.calculate(input)).await;

        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            Ok(Ok(output)) => {
                let is_stub = output.metadata.backend.contains("stub")
                    || output.metadata.backend.contains("placeholder");
                EngineExecutionResult {
                    engine_id,
                    category,
                    output: Some(output),
                    error: None,
                    execution_time_ms,
                    is_stub,
                }
            }
            Ok(Err(e)) => EngineExecutionResult {
                engine_id,
                category,
                output: None,
                error: Some(e.to_string()),
                execution_time_ms,
                is_stub: false,
            },
            Err(_) => EngineExecutionResult {
                engine_id,
                category,
                output: None,
                error: Some(format!(
                    "Timeout after {:?}",
                    self.config.engine_timeout
                )),
                execution_time_ms,
                is_stub: false,
            },
        }
    }

    /// Execute all engines in parallel
    pub async fn execute(&self, input: EngineInput) -> Result<FullSpectrumResult, EngineError> {
        let execution_id = format!("fs-{}", Utc::now().timestamp_millis());
        let start = Instant::now();

        info!(
            execution_id = %execution_id,
            engine_count = self.engines.len(),
            "Starting full spectrum execution"
        );

        // Build futures for all engines
        let futures: Vec<_> = self
            .engines
            .values()
            .map(|engine| {
                let engine_clone = Arc::clone(engine);
                let input_clone = input.clone();
                self.execute_with_timeout(engine_clone, input_clone)
            })
            .collect();

        // Execute all in parallel
        let results = join_all(futures).await;

        // Organize results by category
        let mut by_category: HashMap<EngineCategory, Vec<EngineExecutionResult>> = HashMap::new();
        let mut successful_outputs: HashMap<String, EngineOutput> = HashMap::new();
        let mut failed_engines: HashMap<String, String> = HashMap::new();

        for result in results {
            let category = result.category;
            let engine_id = result.engine_id.clone();

            // Filter out stubs if configured
            if result.is_stub && !self.config.include_stubs {
                info!(engine_id = %engine_id, "Skipping stub engine");
                continue;
            }

            if let Some(ref output) = result.output {
                successful_outputs.insert(engine_id.clone(), output.clone());
            } else if let Some(ref error) = result.error {
                warn!(engine_id = %engine_id, error = %error, "Engine failed");
                failed_engines.insert(engine_id.clone(), error.clone());
            }

            by_category.entry(category).or_default().push(result);
        }

        let engines_attempted = self.engines.len();
        let engines_succeeded = successful_outputs.len();
        let total_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        info!(
            execution_id = %execution_id,
            engines_attempted,
            engines_succeeded,
            total_time_ms,
            "Full spectrum execution complete"
        );

        Ok(FullSpectrumResult {
            execution_id,
            by_category,
            successful_outputs,
            failed_engines,
            total_time_ms,
            engines_attempted,
            engines_succeeded,
            timestamp: Utc::now(),
        })
    }

    /// Execute only engines in specific categories
    pub async fn execute_categories(
        &self,
        input: EngineInput,
        categories: &[EngineCategory],
    ) -> Result<FullSpectrumResult, EngineError> {
        let execution_id = format!("fs-cat-{}", Utc::now().timestamp_millis());
        let start = Instant::now();

        // Filter engines by category
        let category_engine_ids: Vec<&str> = categories
            .iter()
            .flat_map(|cat| cat.engine_ids().iter().copied())
            .collect();

        let filtered_engines: Vec<_> = self
            .engines
            .iter()
            .filter(|(id, _)| category_engine_ids.contains(&id.as_str()))
            .map(|(_, engine)| Arc::clone(engine))
            .collect();

        info!(
            execution_id = %execution_id,
            engine_count = filtered_engines.len(),
            categories = ?categories,
            "Starting category-filtered full spectrum execution"
        );

        // Build and execute futures
        let futures: Vec<_> = filtered_engines
            .into_iter()
            .map(|engine| {
                let input_clone = input.clone();
                self.execute_with_timeout(engine, input_clone)
            })
            .collect();

        let results = join_all(futures).await;

        // Organize results
        let mut by_category: HashMap<EngineCategory, Vec<EngineExecutionResult>> = HashMap::new();
        let mut successful_outputs: HashMap<String, EngineOutput> = HashMap::new();
        let mut failed_engines: HashMap<String, String> = HashMap::new();
        let mut engines_attempted = 0;

        for result in results {
            engines_attempted += 1;
            let category = result.category;
            let engine_id = result.engine_id.clone();

            if let Some(ref output) = result.output {
                successful_outputs.insert(engine_id.clone(), output.clone());
            } else if let Some(ref error) = result.error {
                failed_engines.insert(engine_id.clone(), error.clone());
            }

            by_category.entry(category).or_default().push(result);
        }

        let engines_succeeded = successful_outputs.len();
        let total_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(FullSpectrumResult {
            execution_id,
            by_category,
            successful_outputs,
            failed_engines,
            total_time_ms,
            engines_attempted,
            engines_succeeded,
            timestamp: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use noesis_core::{CalculationMetadata, ValidationResult};

    struct MockEngine {
        id: String,
        phase: u8,
        delay_ms: u64,
        should_fail: bool,
    }

    impl MockEngine {
        fn new(id: &str, phase: u8) -> Self {
            Self {
                id: id.to_string(),
                phase,
                delay_ms: 0,
                should_fail: false,
            }
        }

        fn slow(id: &str, delay_ms: u64) -> Self {
            Self {
                id: id.to_string(),
                phase: 0,
                delay_ms,
                should_fail: false,
            }
        }

        fn failing(id: &str) -> Self {
            Self {
                id: id.to_string(),
                phase: 0,
                delay_ms: 0,
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
            &self.id
        }

        fn required_phase(&self) -> u8 {
            self.phase
        }

        async fn calculate(&self, _input: EngineInput) -> Result<EngineOutput, EngineError> {
            if self.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
            }

            if self.should_fail {
                return Err(EngineError::CalculationError("Mock failure".into()));
            }

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

    fn test_input() -> EngineInput {
        EngineInput {
            birth_data: None,
            current_time: Utc::now(),
            location: None,
            precision: noesis_core::Precision::Standard,
            options: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_full_spectrum_parallel_execution() {
        let mut engines: HashMap<String, Arc<dyn ConsciousnessEngine>> = HashMap::new();
        engines.insert("numerology".into(), Arc::new(MockEngine::new("numerology", 0)));
        engines.insert("human-design".into(), Arc::new(MockEngine::new("human-design", 0)));
        engines.insert("tarot".into(), Arc::new(MockEngine::new("tarot", 0)));

        let workflow = FullSpectrumWorkflow::new(engines);
        let result = workflow.execute(test_input()).await.unwrap();

        assert_eq!(result.engines_attempted, 3);
        assert_eq!(result.engines_succeeded, 3);
        assert!(result.successful_outputs.contains_key("numerology"));
        assert!(result.successful_outputs.contains_key("human-design"));
        assert!(result.successful_outputs.contains_key("tarot"));
    }

    #[tokio::test]
    async fn test_handles_engine_failure() {
        let mut engines: HashMap<String, Arc<dyn ConsciousnessEngine>> = HashMap::new();
        engines.insert("numerology".into(), Arc::new(MockEngine::new("numerology", 0)));
        engines.insert("failing".into(), Arc::new(MockEngine::failing("failing")));

        let workflow = FullSpectrumWorkflow::new(engines);
        let result = workflow.execute(test_input()).await.unwrap();

        assert_eq!(result.engines_attempted, 2);
        assert_eq!(result.engines_succeeded, 1);
        assert!(result.failed_engines.contains_key("failing"));
    }

    #[tokio::test]
    async fn test_timeout_handling() {
        let mut engines: HashMap<String, Arc<dyn ConsciousnessEngine>> = HashMap::new();
        engines.insert("fast".into(), Arc::new(MockEngine::new("fast", 0)));
        engines.insert("slow".into(), Arc::new(MockEngine::slow("slow", 200)));

        let config = FullSpectrumConfig {
            engine_timeout: Duration::from_millis(50),
            ..Default::default()
        };

        let workflow = FullSpectrumWorkflow::with_config(engines, config);
        let result = workflow.execute(test_input()).await.unwrap();

        assert_eq!(result.engines_succeeded, 1);
        assert!(result.successful_outputs.contains_key("fast"));
        assert!(result.failed_engines.contains_key("slow"));
        assert!(result.failed_engines["slow"].contains("Timeout"));
    }

    #[tokio::test]
    async fn test_phase_access_control() {
        let mut engines: HashMap<String, Arc<dyn ConsciousnessEngine>> = HashMap::new();
        engines.insert("basic".into(), Arc::new(MockEngine::new("basic", 0)));
        engines.insert("advanced".into(), Arc::new(MockEngine::new("advanced", 3)));

        let config = FullSpectrumConfig {
            user_phase: 1,
            ..Default::default()
        };

        let workflow = FullSpectrumWorkflow::with_config(engines, config);
        let result = workflow.execute(test_input()).await.unwrap();

        assert_eq!(result.engines_succeeded, 1);
        assert!(result.successful_outputs.contains_key("basic"));
        assert!(result.failed_engines.contains_key("advanced"));
        assert!(result.failed_engines["advanced"].contains("Phase access denied"));
    }

    #[tokio::test]
    async fn test_category_filtering() {
        let mut engines: HashMap<String, Arc<dyn ConsciousnessEngine>> = HashMap::new();
        engines.insert("numerology".into(), Arc::new(MockEngine::new("numerology", 0)));
        engines.insert("panchanga".into(), Arc::new(MockEngine::new("panchanga", 0)));
        engines.insert("tarot".into(), Arc::new(MockEngine::new("tarot", 0)));

        let workflow = FullSpectrumWorkflow::new(engines);
        let result = workflow
            .execute_categories(test_input(), &[EngineCategory::Natal])
            .await
            .unwrap();

        // Only numerology is in Natal category
        assert_eq!(result.engines_succeeded, 1);
        assert!(result.successful_outputs.contains_key("numerology"));
    }

    #[tokio::test]
    async fn test_parallel_timing() {
        // Verify that parallel execution takes ~max(engine_time), not sum
        let mut engines: HashMap<String, Arc<dyn ConsciousnessEngine>> = HashMap::new();
        engines.insert("a".into(), Arc::new(MockEngine::slow("a", 50)));
        engines.insert("b".into(), Arc::new(MockEngine::slow("b", 50)));
        engines.insert("c".into(), Arc::new(MockEngine::slow("c", 50)));

        let config = FullSpectrumConfig {
            engine_timeout: Duration::from_secs(1),
            ..Default::default()
        };

        let workflow = FullSpectrumWorkflow::with_config(engines, config);
        let start = Instant::now();
        let result = workflow.execute(test_input()).await.unwrap();
        let elapsed = start.elapsed();

        // Should be ~50ms (parallel), not 150ms (sequential)
        // Allow for some overhead
        assert!(elapsed.as_millis() < 150, "Execution should be parallel");
        assert_eq!(result.engines_succeeded, 3);
    }
}
