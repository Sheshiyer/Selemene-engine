//! W1-S7-05: Parallel Workflow Execution Tests
//!
//! Verifies that the WorkflowOrchestrator executes engines in parallel
//! (via futures::future::join_all) rather than sequentially, and that
//! workflow results are correctly synthesized from engine outputs.

use async_trait::async_trait;
use chrono::Utc;
use futures::future::join_all;
use noesis_core::{
    BirthData, CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    Precision, ValidationResult,
};
use noesis_orchestrator::WorkflowOrchestrator;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Delay-aware mock engine for parallelism testing
// ---------------------------------------------------------------------------

/// Mock engine that sleeps for a configurable duration to measure parallelism.
struct DelayMockEngine {
    id: String,
    name: String,
    phase: u8,
    delay: Duration,
    should_fail: bool,
}

impl DelayMockEngine {
    fn new(id: &str, phase: u8, delay: Duration) -> Self {
        Self {
            id: id.to_string(),
            name: format!("Delay Mock {}", id),
            phase,
            delay,
            should_fail: false,
        }
    }

    fn failing(id: &str, phase: u8, delay: Duration) -> Self {
        Self {
            id: id.to_string(),
            name: format!("Failing Delay Mock {}", id),
            phase,
            delay,
            should_fail: true,
        }
    }
}

#[async_trait]
impl ConsciousnessEngine for DelayMockEngine {
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
        // Simulate computation time
        tokio::time::sleep(self.delay).await;

        if self.should_fail {
            return Err(EngineError::CalculationError(format!(
                "{} intentionally failed",
                self.id
            )));
        }

        Ok(EngineOutput {
            engine_id: self.id.clone(),
            result: serde_json::json!({
                "engine": self.id,
                "type": "mock_result",
                "delay_ms": self.delay.as_millis(),
            }),
            witness_prompt: format!("What does {} reveal about your journey?", self.id),
            consciousness_level: self.phase,
            metadata: CalculationMetadata {
                calculation_time_ms: self.delay.as_millis() as f64,
                backend: "delay_mock".to_string(),
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
        format!("delay-mock-{}", self.id)
    }
}

/// Helper to build a standard test input with birth data.
fn test_input() -> EngineInput {
    EngineInput {
        birth_data: Some(BirthData {
            name: Some("Parallel Test".to_string()),
            date: "1990-01-15".to_string(),
            time: Some("14:30".to_string()),
            latitude: 12.9716,
            longitude: 77.5946,
            timezone: "Asia/Kolkata".to_string(),
        }),
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

/// Build an orchestrator with 3 delay engines registered for birth-blueprint.
fn build_parallel_orchestrator(delay: Duration) -> WorkflowOrchestrator {
    let mut orchestrator = WorkflowOrchestrator::new();
    orchestrator
        .register_engine(Arc::new(DelayMockEngine::new("numerology", 0, delay)));
    orchestrator
        .register_engine(Arc::new(DelayMockEngine::new("human-design", 0, delay)));
    orchestrator
        .register_engine(Arc::new(DelayMockEngine::new("gene-keys", 0, delay)));
    orchestrator
}

// ---------------------------------------------------------------------------
// W1-S7-05: Parallel execution tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_birth_blueprint_workflow_parallelism() {
    // birth-blueprint workflow includes: numerology, human-design, gene-keys
    // Each engine delays ~50ms
    // Sequential would be ~150ms
    // Parallel should be ~50ms (max of the three)

    let delay = Duration::from_millis(50);
    let orchestrator = build_parallel_orchestrator(delay);

    let start = Instant::now();
    let result = orchestrator
        .execute_workflow("birth-blueprint", test_input(), 5)
        .await
        .expect("Workflow execution should succeed");
    let elapsed = start.elapsed();

    // All three engines should have produced results
    assert!(
        result.engine_outputs.contains_key("numerology"),
        "numerology engine output missing"
    );
    assert!(
        result.engine_outputs.contains_key("human-design"),
        "human-design engine output missing"
    );
    assert!(
        result.engine_outputs.contains_key("gene-keys"),
        "gene-keys engine output missing"
    );
    assert_eq!(result.engine_outputs.len(), 3);

    // Verify parallel execution: total time approximately max(engine_times), not sum
    // With parallel: ~50ms, sequential would be ~150ms
    // Allow generous margin for CI/test environment variance
    assert!(
        elapsed.as_millis() < 130,
        "Expected parallel execution (<130ms) but took {}ms (sequential would be ~150ms)",
        elapsed.as_millis()
    );
    assert!(
        elapsed.as_millis() >= 40,
        "Execution was suspiciously fast ({}ms) - engines should take at least ~50ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_full_spectrum_workflow_parallelism() {
    // full-spectrum has 14 engines; we register 6 that exist as delay mocks
    // All should execute in parallel

    let delay = Duration::from_millis(50);
    let mut orchestrator = WorkflowOrchestrator::new();

    // Register 6 engines (the ones that would be registered in a real deployment)
    let engine_ids = [
        "panchanga",
        "numerology",
        "biorhythm",
        "human-design",
        "gene-keys",
        "vimshottari",
    ];
    for id in &engine_ids {
        orchestrator.register_engine(Arc::new(DelayMockEngine::new(id, 0, delay)));
    }

    let start = Instant::now();
    let result = orchestrator
        .execute_workflow("full-spectrum", test_input(), 5)
        .await
        .expect("Workflow execution should succeed");
    let elapsed = start.elapsed();

    // Should have outputs for all 6 registered engines
    assert_eq!(
        result.engine_outputs.len(),
        6,
        "Expected 6 engine outputs, got {}. Keys: {:?}",
        result.engine_outputs.len(),
        result.engine_outputs.keys().collect::<Vec<_>>()
    );

    // Even with 6 engines, parallel execution should be ~50ms, not 6*50=300ms
    assert!(
        elapsed.as_millis() < 200,
        "Expected parallel execution (<200ms) but took {}ms (6x sequential would be ~300ms)",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_workflow_result_synthesis() {
    // Verify WorkflowResult contains all engine results with correct structure

    let delay = Duration::from_millis(10);
    let orchestrator = build_parallel_orchestrator(delay);

    let result = orchestrator
        .execute_workflow("birth-blueprint", test_input(), 5)
        .await
        .expect("Workflow should succeed");

    // Verify workflow_id
    assert_eq!(result.workflow_id, "birth-blueprint");

    // Verify each engine result is a complete EngineOutput
    for engine_id in ["numerology", "human-design", "gene-keys"] {
        let output = result
            .engine_outputs
            .get(engine_id)
            .unwrap_or_else(|| panic!("Missing output for engine: {}", engine_id));

        assert_eq!(output.engine_id, engine_id);
        assert!(
            !output.witness_prompt.is_empty(),
            "Witness prompt should not be empty for {}",
            engine_id
        );
        assert!(
            output.result.get("type").is_some(),
            "Result should contain 'type' field for {}",
            engine_id
        );
        assert!(
            output.result.get("engine").is_some(),
            "Result should contain 'engine' field for {}",
            engine_id
        );
    }

    // Verify timing metadata
    assert!(
        result.total_time_ms >= 0.0,
        "total_time_ms should be non-negative"
    );
    assert!(
        result.timestamp <= Utc::now(),
        "Timestamp should be in the past or present"
    );
}

#[tokio::test]
async fn test_workflow_partial_failure_graceful_degradation() {
    // If one engine fails, workflow should still return results from successful engines

    let delay = Duration::from_millis(10);
    let mut orchestrator = WorkflowOrchestrator::new();

    // Register numerology and human-design as working, gene-keys as failing
    orchestrator
        .register_engine(Arc::new(DelayMockEngine::new("numerology", 0, delay)));
    orchestrator
        .register_engine(Arc::new(DelayMockEngine::new("human-design", 0, delay)));
    orchestrator.register_engine(Arc::new(DelayMockEngine::failing(
        "gene-keys",
        0,
        delay,
    )));

    let result = orchestrator
        .execute_workflow("birth-blueprint", test_input(), 5)
        .await
        .expect("Workflow should succeed even with partial failures");

    // Should have results from successful engines
    assert!(
        result.engine_outputs.contains_key("numerology"),
        "numerology should succeed"
    );
    assert!(
        result.engine_outputs.contains_key("human-design"),
        "human-design should succeed"
    );

    // Failed engine should be omitted (not in engine_outputs)
    assert!(
        !result.engine_outputs.contains_key("gene-keys"),
        "gene-keys should be omitted due to failure"
    );

    // Workflow itself should succeed with 2 out of 3 results
    assert_eq!(result.engine_outputs.len(), 2);
    assert_eq!(result.workflow_id, "birth-blueprint");
}

#[tokio::test]
async fn test_workflow_all_engines_fail_still_succeeds() {
    // Even if ALL engines fail, the workflow itself should return Ok with empty results

    let delay = Duration::from_millis(10);
    let mut orchestrator = WorkflowOrchestrator::new();

    orchestrator.register_engine(Arc::new(DelayMockEngine::failing(
        "numerology",
        0,
        delay,
    )));
    orchestrator.register_engine(Arc::new(DelayMockEngine::failing(
        "human-design",
        0,
        delay,
    )));
    orchestrator.register_engine(Arc::new(DelayMockEngine::failing(
        "gene-keys",
        0,
        delay,
    )));

    let result = orchestrator
        .execute_workflow("birth-blueprint", test_input(), 5)
        .await
        .expect("Workflow should still succeed with all failures");

    assert_eq!(result.engine_outputs.len(), 0);
    assert_eq!(result.workflow_id, "birth-blueprint");
}

#[tokio::test]
async fn test_workflow_phase_gated_engines_skipped() {
    // Engines above user's phase should be skipped gracefully

    let delay = Duration::from_millis(10);
    let mut orchestrator = WorkflowOrchestrator::new();

    orchestrator
        .register_engine(Arc::new(DelayMockEngine::new("numerology", 0, delay)));
    orchestrator
        .register_engine(Arc::new(DelayMockEngine::new("human-design", 1, delay)));
    orchestrator
        .register_engine(Arc::new(DelayMockEngine::new("gene-keys", 3, delay)));

    // User at phase 1 -- gene-keys (phase 3) should be skipped
    let result = orchestrator
        .execute_workflow("birth-blueprint", test_input(), 1)
        .await
        .expect("Workflow should succeed");

    assert_eq!(result.engine_outputs.len(), 2);
    assert!(result.engine_outputs.contains_key("numerology"));
    assert!(result.engine_outputs.contains_key("human-design"));
    assert!(
        !result.engine_outputs.contains_key("gene-keys"),
        "gene-keys should be phase-gated (requires 3, user has 1)"
    );
}

#[tokio::test]
async fn test_workflow_concurrent_execution() {
    // Multiple workflows should execute concurrently without blocking each other

    let delay = Duration::from_millis(50);
    let orchestrator = Arc::new(build_parallel_orchestrator(delay));

    let mut workflows: Vec<
        std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<noesis_core::WorkflowResult, EngineError>>>,
        >,
    > = Vec::new();

    for _ in 0..3 {
        let orch = orchestrator.clone();
        let input = test_input();
        workflows.push(Box::pin(async move {
            orch.execute_workflow("birth-blueprint", input, 5).await
        }));
    }

    let start = Instant::now();
    let results = join_all(workflows).await;
    let elapsed = start.elapsed();

    // All 3 workflows should succeed
    assert_eq!(results.len(), 3);
    for (i, result) in results.iter().enumerate() {
        let wf_result = result
            .as_ref()
            .unwrap_or_else(|e| panic!("Workflow {} failed: {}", i, e));
        assert_eq!(wf_result.engine_outputs.len(), 3);
    }

    // 3 workflows in parallel should not take 3x the single workflow time
    // Single workflow: ~50ms, 3 sequential: ~150ms
    // Parallel: should still be ~50ms (shared tokio runtime)
    assert!(
        elapsed.as_millis() < 200,
        "3 concurrent workflows took {}ms, expected <200ms (not 3x sequential)",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn test_workflow_not_found_error() {
    let orchestrator = WorkflowOrchestrator::new();

    let result = orchestrator
        .execute_workflow("nonexistent-workflow", test_input(), 5)
        .await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        EngineError::WorkflowNotFound(_)
    ));
}

#[tokio::test]
async fn test_workflow_missing_engines_skipped() {
    // Workflow references engines that are not registered -- they should be skipped

    let delay = Duration::from_millis(10);
    let mut orchestrator = WorkflowOrchestrator::new();

    // Only register numerology; human-design and gene-keys are missing
    orchestrator
        .register_engine(Arc::new(DelayMockEngine::new("numerology", 0, delay)));

    let result = orchestrator
        .execute_workflow("birth-blueprint", test_input(), 5)
        .await
        .expect("Workflow should succeed with missing engines");

    assert_eq!(result.engine_outputs.len(), 1);
    assert!(result.engine_outputs.contains_key("numerology"));
}

#[tokio::test]
async fn test_workflow_timing_metadata_accurate() {
    // Verify total_time_ms is roughly correct

    let delay = Duration::from_millis(50);
    let orchestrator = build_parallel_orchestrator(delay);

    let result = orchestrator
        .execute_workflow("birth-blueprint", test_input(), 5)
        .await
        .expect("Workflow should succeed");

    // total_time_ms should be at least the delay time (parallel execution)
    assert!(
        result.total_time_ms >= 40.0,
        "total_time_ms ({}) should be at least ~50ms (engine delay)",
        result.total_time_ms
    );

    // Should not be sequential time (3 * 50ms = 150ms)
    assert!(
        result.total_time_ms < 130.0,
        "total_time_ms ({}) suggests sequential execution (expected <130ms for parallel)",
        result.total_time_ms
    );
}
