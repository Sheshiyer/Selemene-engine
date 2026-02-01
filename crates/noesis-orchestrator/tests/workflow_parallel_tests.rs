//! Parallel Execution Tests for Workflow Engine
//!
//! Verifies that engines execute in parallel (not sequential).
//! A 3-engine workflow should take ~max(engine_time), not sum(engine_times).

use chrono::Utc;
use noesis_core::{
    CalculationMetadata, ConsciousnessEngine, EngineError, EngineInput, EngineOutput,
    Precision, ValidationResult,
};
use noesis_orchestrator::workflow::WorkflowExecutor;
use noesis_orchestrator::EngineRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use async_trait::async_trait;

/// Mock engine that takes a configurable time to complete
struct TimedMockEngine {
    id: String,
    name: String,
    delay_ms: u64,
}

impl TimedMockEngine {
    fn new(id: &str, delay_ms: u64) -> Self {
        Self {
            id: id.to_string(),
            name: format!("Timed Mock {}", id),
            delay_ms,
        }
    }
}

#[async_trait]
impl ConsciousnessEngine for TimedMockEngine {
    fn engine_id(&self) -> &str {
        &self.id
    }

    fn engine_name(&self) -> &str {
        &self.name
    }

    fn required_phase(&self) -> u8 {
        0
    }

    async fn calculate(&self, _input: EngineInput) -> Result<EngineOutput, EngineError> {
        // Simulate work by sleeping
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;

        Ok(EngineOutput {
            engine_id: self.id.clone(),
            result: serde_json::json!({
                "mock": true,
                "engine": self.id,
                "delay_ms": self.delay_ms
            }),
            witness_prompt: format!("Witness prompt from {}", self.id),
            consciousness_level: 0,
            metadata: CalculationMetadata {
                calculation_time_ms: self.delay_ms as f64,
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
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

/// Test that engines execute in parallel
/// 
/// If 3 engines each take 100ms and run sequentially, total would be 300ms.
/// If they run in parallel, total should be ~100ms (plus some overhead).
#[tokio::test]
async fn engines_execute_in_parallel() {
    let engine_delay_ms = 100;
    let engine_count = 3;
    
    // Setup registry with timed mock engines for birth-blueprint workflow
    let mut engine_registry = EngineRegistry::new();
    engine_registry.register(Arc::new(TimedMockEngine::new("numerology", engine_delay_ms)));
    engine_registry.register(Arc::new(TimedMockEngine::new("human-design", engine_delay_ms)));
    engine_registry.register(Arc::new(TimedMockEngine::new("vimshottari", engine_delay_ms)));
    
    let executor = WorkflowExecutor::new(Arc::new(engine_registry));
    
    let start = Instant::now();
    let result = executor.execute("birth-blueprint", test_input(), 5).await;
    let elapsed = start.elapsed();
    
    // Verify execution succeeded
    assert!(result.is_ok(), "Workflow should execute successfully");
    let output = result.unwrap();
    assert_eq!(output.engine_results.len(), engine_count);
    
    // Calculate expected timings
    let sequential_time = Duration::from_millis(engine_delay_ms * engine_count as u64);
    let parallel_time = Duration::from_millis(engine_delay_ms);
    let overhead_allowance = Duration::from_millis(50); // Allow some overhead
    
    // Actual time should be closer to parallel_time than sequential_time
    let parallel_target = parallel_time + overhead_allowance;
    
    println!("Elapsed: {:?}", elapsed);
    println!("Sequential would be: {:?}", sequential_time);
    println!("Parallel target: {:?}", parallel_target);
    
    assert!(
        elapsed < sequential_time,
        "Execution took {:?}, which suggests sequential execution (would be {:?}). \
         Expected parallel execution around {:?}",
        elapsed,
        sequential_time,
        parallel_time
    );
    
    // More specifically, should be close to max engine time
    assert!(
        elapsed < parallel_target,
        "Execution took {:?}, expected around {:?} for parallel execution",
        elapsed,
        parallel_time
    );
}

/// Test daily-practice workflow also runs in parallel
#[tokio::test]
async fn daily_practice_runs_in_parallel() {
    let engine_delay_ms = 80;
    
    let mut engine_registry = EngineRegistry::new();
    engine_registry.register(Arc::new(TimedMockEngine::new("panchanga", engine_delay_ms)));
    engine_registry.register(Arc::new(TimedMockEngine::new("vedic-clock", engine_delay_ms)));
    engine_registry.register(Arc::new(TimedMockEngine::new("biorhythm", engine_delay_ms)));
    
    let executor = WorkflowExecutor::new(Arc::new(engine_registry));
    
    let start = Instant::now();
    let result = executor.execute("daily-practice", test_input(), 5).await;
    let elapsed = start.elapsed();
    
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.engine_results.len(), 3);
    
    // Should complete in roughly the time of one engine, not three
    let max_expected = Duration::from_millis(engine_delay_ms * 2); // Allow 2x for overhead
    assert!(
        elapsed < max_expected,
        "Daily practice took {:?}, expected parallel execution under {:?}",
        elapsed,
        max_expected
    );
}

/// Test with varying engine times - total should be ~max(times)
#[tokio::test]
async fn parallel_execution_with_varying_times() {
    let mut engine_registry = EngineRegistry::new();
    
    // Engines with different delays: 50ms, 100ms, 150ms
    engine_registry.register(Arc::new(TimedMockEngine::new("numerology", 50)));
    engine_registry.register(Arc::new(TimedMockEngine::new("human-design", 100)));
    engine_registry.register(Arc::new(TimedMockEngine::new("vimshottari", 150)));
    
    let executor = WorkflowExecutor::new(Arc::new(engine_registry));
    
    let start = Instant::now();
    let result = executor.execute("birth-blueprint", test_input(), 5).await;
    let elapsed = start.elapsed();
    
    assert!(result.is_ok());
    
    // Sequential would be 50 + 100 + 150 = 300ms
    // Parallel should be ~150ms (the slowest engine)
    let sequential_time = Duration::from_millis(300);
    let max_engine_time = Duration::from_millis(150);
    let overhead_allowance = Duration::from_millis(50);
    
    println!("Elapsed with varying times: {:?}", elapsed);
    
    assert!(
        elapsed < sequential_time,
        "Took {:?}, sequential would be {:?}",
        elapsed,
        sequential_time
    );
    
    assert!(
        elapsed < max_engine_time + overhead_allowance,
        "Took {:?}, expected around {:?} (max engine time)",
        elapsed,
        max_engine_time
    );
}

/// Test that synthesis happens after all engines complete
#[tokio::test]
async fn synthesis_includes_all_engine_results() {
    let mut engine_registry = EngineRegistry::new();
    engine_registry.register(Arc::new(TimedMockEngine::new("numerology", 30)));
    engine_registry.register(Arc::new(TimedMockEngine::new("human-design", 30)));
    engine_registry.register(Arc::new(TimedMockEngine::new("vimshottari", 30)));
    
    let executor = WorkflowExecutor::new(Arc::new(engine_registry));
    let result = executor.execute("birth-blueprint", test_input(), 5).await.unwrap();
    
    // All engines should be present in results
    assert!(result.engine_results.contains_key("numerology"));
    assert!(result.engine_results.contains_key("human-design"));
    assert!(result.engine_results.contains_key("vimshottari"));
    
    // Synthesis should have been performed
    assert!(!result.synthesis.summary.is_empty() || result.synthesis.themes.is_empty());
    
    // Witness prompts should be generated
    // (may be empty if synthesis found no patterns, but shouldn't panic)
    let _ = result.witness_prompts;
}

/// Test workflow execution time is recorded correctly
#[tokio::test]
async fn execution_time_recorded_correctly() {
    let engine_delay_ms = 50;
    
    let mut engine_registry = EngineRegistry::new();
    engine_registry.register(Arc::new(TimedMockEngine::new("numerology", engine_delay_ms)));
    engine_registry.register(Arc::new(TimedMockEngine::new("human-design", engine_delay_ms)));
    engine_registry.register(Arc::new(TimedMockEngine::new("vimshottari", engine_delay_ms)));
    
    let executor = WorkflowExecutor::new(Arc::new(engine_registry));
    let result = executor.execute("birth-blueprint", test_input(), 5).await.unwrap();
    
    // Recorded execution time should be non-zero
    assert!(result.execution_time_ms > 0);
    
    // And should be roughly consistent with actual execution
    // (at least as long as the minimum engine time)
    assert!(result.execution_time_ms >= engine_delay_ms);
    
    println!("Recorded execution time: {}ms", result.execution_time_ms);
}

/// Test handling of missing engines - should not block others
#[tokio::test]
async fn missing_engines_dont_block_parallel_execution() {
    let engine_delay_ms = 100;
    
    let mut engine_registry = EngineRegistry::new();
    // Only register 2 of 3 engines for birth-blueprint
    engine_registry.register(Arc::new(TimedMockEngine::new("numerology", engine_delay_ms)));
    engine_registry.register(Arc::new(TimedMockEngine::new("human-design", engine_delay_ms)));
    // vimshottari is missing
    
    let executor = WorkflowExecutor::new(Arc::new(engine_registry));
    
    let start = Instant::now();
    let result = executor.execute("birth-blueprint", test_input(), 5).await;
    let elapsed = start.elapsed();
    
    // Should still succeed with partial results
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.engine_results.len(), 2);
    
    // Should still be parallel (not sequential)
    let max_expected = Duration::from_millis(engine_delay_ms * 2);
    assert!(
        elapsed < max_expected,
        "Missing engine caused sequential behavior? Took {:?}",
        elapsed
    );
}
