//! Performance benchmarks for Noesis Orchestrator workflows
//!
//! Benchmarks full spectrum and individual workflow execution times.
//! Targets:
//! - Full spectrum (14 engines): <2 seconds
//! - Single workflow: <500ms

use async_trait::async_trait;
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use noesis_core::{
    CalculationMetadata, EngineError, EngineInput, EngineOutput, Precision, ValidationResult,
};
use noesis_orchestrator::{
    ConsciousnessEngine, FullSpectrumWorkflow, WorkflowOrchestrator,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

// ---------------------------------------------------------------------------
// Mock Engine for Benchmarks
// ---------------------------------------------------------------------------

struct BenchMockEngine {
    id: String,
    delay_us: u64,
}

impl BenchMockEngine {
    fn new(id: &str, delay_us: u64) -> Self {
        Self {
            id: id.to_string(),
            delay_us,
        }
    }
}

#[async_trait]
impl ConsciousnessEngine for BenchMockEngine {
    fn engine_id(&self) -> &str {
        &self.id
    }

    fn engine_name(&self) -> &str {
        &self.id
    }

    fn required_phase(&self) -> u8 {
        0
    }

    async fn calculate(&self, _input: EngineInput) -> Result<EngineOutput, EngineError> {
        if self.delay_us > 0 {
            tokio::time::sleep(Duration::from_micros(self.delay_us)).await;
        }

        Ok(EngineOutput {
            engine_id: self.id.clone(),
            result: serde_json::json!({ "bench": true }),
            witness_prompt: "Benchmark prompt".to_string(),
            consciousness_level: 0,
            metadata: CalculationMetadata {
                calculation_time_ms: self.delay_us as f64 / 1000.0,
                backend: "bench".to_string(),
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
        format!("bench-{}", self.id)
    }
}

// ---------------------------------------------------------------------------
// Test Input
// ---------------------------------------------------------------------------

fn bench_input() -> EngineInput {
    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

// ---------------------------------------------------------------------------
// Benchmark: Full Spectrum Workflow
// ---------------------------------------------------------------------------

fn bench_full_spectrum(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    // Create engines with varying simulated delays (0-10ms each)
    let engine_ids = [
        "numerology",
        "human-design",
        "gene-keys",
        "panchanga",
        "vedic-clock",
        "biorhythm",
        "vimshottari",
        "tarot",
        "i-ching",
        "enneagram",
        "sigil-forge",
        "sacred-geometry",
        "biofield",
        "face-reading",
    ];

    let mut group = c.benchmark_group("full_spectrum");
    group.measurement_time(Duration::from_secs(10));

    // Benchmark with zero delay (pure overhead measurement)
    {
        let mut engines: HashMap<String, Arc<dyn ConsciousnessEngine>> = HashMap::new();
        for id in &engine_ids {
            engines.insert(id.to_string(), Arc::new(BenchMockEngine::new(id, 0)));
        }
        let workflow = FullSpectrumWorkflow::new(engines);
        let input = bench_input();

        group.bench_function("14_engines_zero_delay", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    black_box(workflow.execute(input.clone()).await.unwrap())
                })
            })
        });
    }

    // Benchmark with realistic delays (1ms per engine)
    {
        let mut engines: HashMap<String, Arc<dyn ConsciousnessEngine>> = HashMap::new();
        for id in &engine_ids {
            engines.insert(id.to_string(), Arc::new(BenchMockEngine::new(id, 1000))); // 1ms
        }
        let workflow = FullSpectrumWorkflow::new(engines);
        let input = bench_input();

        group.bench_function("14_engines_1ms_each", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    black_box(workflow.execute(input.clone()).await.unwrap())
                })
            })
        });
    }

    // Benchmark with 10ms delays (parallel should still complete fast)
    {
        let mut engines: HashMap<String, Arc<dyn ConsciousnessEngine>> = HashMap::new();
        for id in &engine_ids {
            engines.insert(id.to_string(), Arc::new(BenchMockEngine::new(id, 10000))); // 10ms
        }
        let workflow = FullSpectrumWorkflow::new(engines);
        let input = bench_input();

        group.bench_function("14_engines_10ms_each", |b| {
            b.iter(|| {
                runtime.block_on(async {
                    black_box(workflow.execute(input.clone()).await.unwrap())
                })
            })
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Individual Workflows
// ---------------------------------------------------------------------------

fn bench_individual_workflows(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut orchestrator = WorkflowOrchestrator::new();

    // Register all engines with 1ms simulated delay
    let all_engines = [
        "numerology", "human-design", "gene-keys", "panchanga", "vedic-clock",
        "biorhythm", "vimshottari", "tarot", "i-ching", "enneagram",
        "sigil-forge", "sacred-geometry", "biofield", "face-reading",
    ];

    for id in all_engines {
        orchestrator.register_engine(Arc::new(BenchMockEngine::new(id, 1000)));
    }

    let mut group = c.benchmark_group("workflows");
    group.measurement_time(Duration::from_secs(10));

    let workflows = [
        ("birth-blueprint", 3),
        ("daily-practice", 3),
        ("decision-support", 3),
        ("self-inquiry", 2),
        ("creative-expression", 2),
        ("full-spectrum", 14),
    ];

    for (workflow_id, engine_count) in workflows {
        let input = bench_input();
        group.bench_with_input(
            BenchmarkId::new(workflow_id, engine_count),
            &input,
            |b, input| {
                b.iter(|| {
                    runtime.block_on(async {
                        black_box(
                            orchestrator
                                .execute_workflow(workflow_id, input.clone(), 5)
                                .await
                                .unwrap()
                        )
                    })
                })
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Scaling
// ---------------------------------------------------------------------------

fn bench_scaling(c: &mut Criterion) {
    let runtime = Runtime::new().unwrap();

    let mut group = c.benchmark_group("scaling");
    group.measurement_time(Duration::from_secs(10));

    // Test with different numbers of engines
    for engine_count in [1, 5, 10, 20, 50] {
        let mut engines: HashMap<String, Arc<dyn ConsciousnessEngine>> = HashMap::new();
        for i in 0..engine_count {
            let id = format!("engine-{}", i);
            engines.insert(id.clone(), Arc::new(BenchMockEngine::new(&id, 1000))); // 1ms each
        }

        let workflow = FullSpectrumWorkflow::new(engines);
        let input = bench_input();

        group.bench_with_input(
            BenchmarkId::new("engines", engine_count),
            &input,
            |b, input| {
                b.iter(|| {
                    runtime.block_on(async {
                        black_box(workflow.execute(input.clone()).await.unwrap())
                    })
                })
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: Cache Performance
// ---------------------------------------------------------------------------

fn bench_cache(c: &mut Criterion) {
    use noesis_orchestrator::{WorkflowCache, WorkflowCacheKey};

    let runtime = Runtime::new().unwrap();
    let cache = WorkflowCache::new(10000);

    // Pre-populate cache
    runtime.block_on(async {
        for i in 0..1000 {
            let key = WorkflowCacheKey::new("test", i, "v1");
            let result = noesis_core::WorkflowResult {
                workflow_id: "test".to_string(),
                engine_outputs: HashMap::new(),
                synthesis: None,
                total_time_ms: 100.0,
                timestamp: Utc::now(),
            };
            cache.set(key, result, Duration::from_secs(3600)).await;
        }
    });

    let mut group = c.benchmark_group("cache");

    group.bench_function("cache_hit", |b| {
        let key = WorkflowCacheKey::new("test", 500, "v1");
        b.iter(|| {
            runtime.block_on(async {
                black_box(cache.get(&key).await)
            })
        })
    });

    group.bench_function("cache_miss", |b| {
        let key = WorkflowCacheKey::new("test", 99999, "v1");
        b.iter(|| {
            runtime.block_on(async {
                black_box(cache.get(&key).await)
            })
        })
    });

    group.bench_function("cache_set", |b| {
        let mut counter = 10000u64;
        b.iter(|| {
            counter += 1;
            let key = WorkflowCacheKey::new("bench", counter, "v1");
            let result = noesis_core::WorkflowResult {
                workflow_id: "bench".to_string(),
                engine_outputs: HashMap::new(),
                synthesis: None,
                total_time_ms: 100.0,
                timestamp: Utc::now(),
            };
            runtime.block_on(async {
                black_box(cache.set(key, result, Duration::from_secs(60)).await)
            })
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_full_spectrum,
    bench_individual_workflows,
    bench_scaling,
    bench_cache,
);
criterion_main!(benches);
