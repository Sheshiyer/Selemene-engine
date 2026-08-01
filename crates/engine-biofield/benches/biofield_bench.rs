//! Biofield Engine Performance Benchmarks
//!
//! Criterion benchmark suites covering:
//!   Group 1 – core calculation path (mock metrics generation, vitality index, chakra readings)
//!   Group 2 – batch operations (multiple users, seeded reproducible runs)

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use engine_biofield::{
    generate_metrics_for_user, generate_mock_metrics, generate_witness_prompts, get_chakra_wisdom,
    get_metric_interpretation, models::Chakra, BiofieldEngine, ConsciousnessEngine, EngineInput,
};
use noesis_core::Precision;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Group 1: core calculation path
// ---------------------------------------------------------------------------

/// Benchmark: generate mock metrics with a fixed seed (pure calculation path)
fn bench_generate_mock_metrics_seeded(c: &mut Criterion) {
    let mut group = c.benchmark_group("biofield_core");

    group.bench_function("mock_metrics_seeded", |b| {
        b.iter(|| {
            let metrics = generate_mock_metrics(black_box(Some(42)));
            black_box(metrics)
        })
    });

    group.bench_function("mock_metrics_no_seed", |b| {
        b.iter(|| {
            let metrics = generate_mock_metrics(black_box(None));
            black_box(metrics)
        })
    });

    // Parameterised over several seeds to avoid constant-folding
    for seed in [0u64, 1, 42, 1337, u64::MAX / 2] {
        group.bench_with_input(
            BenchmarkId::new("mock_metrics_seed", seed),
            &seed,
            |b, &s| b.iter(|| black_box(generate_mock_metrics(Some(s)))),
        );
    }

    group.finish();
}

/// Benchmark: chakra wisdom lookup for each of the 7 chakras
fn bench_chakra_wisdom_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("biofield_core");

    let chakras = [
        Chakra::Root,
        Chakra::Sacral,
        Chakra::SolarPlexus,
        Chakra::Heart,
        Chakra::Throat,
        Chakra::ThirdEye,
        Chakra::Crown,
    ];

    group.bench_function("chakra_wisdom_all_7", |b| {
        b.iter(|| {
            for chakra in &chakras {
                black_box(get_chakra_wisdom(black_box(*chakra)));
            }
        })
    });

    group.finish();
}

/// Benchmark: metric interpretation lookup
fn bench_metric_interpretation(c: &mut Criterion) {
    let mut group = c.benchmark_group("biofield_core");

    // The four core BiofieldMetrics fields (see engine_biofield::models::BiofieldMetrics)
    let metrics = ["coherence", "entropy", "fractal_dimension", "symmetry"];
    group.bench_function("metric_interpretation_all", |b| {
        b.iter(|| {
            for m in &metrics {
                black_box(get_metric_interpretation(black_box(m)));
            }
        })
    });

    group.finish();
}

/// Benchmark: witness prompt generation from a BiofieldAnalysis
fn bench_witness_prompt(c: &mut Criterion) {
    use engine_biofield::models::BiofieldAnalysis;

    let analysis = BiofieldAnalysis {
        metrics: generate_mock_metrics(Some(7)),
        interpretation: "Vitality is in a moderate range".to_string(),
        areas_of_attention: vec!["coherence".to_string()],
        is_mock_data: true,
        // Capture-path fields (T-026). This bench measures prompt generation
        // from a plain analysis, so no capture payload is attached.
        consent: None,
        quality: None,
    };
    let mut group = c.benchmark_group("biofield_core");

    group.bench_function("witness_prompts", |b| {
        b.iter(|| black_box(generate_witness_prompts(black_box(&analysis))))
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 2: batch operations
// ---------------------------------------------------------------------------

/// Benchmark: generate metrics for a list of user IDs (API-load simulation)
fn bench_batch_user_metrics(c: &mut Criterion) {
    let user_ids: Vec<String> = (0..10).map(|i| format!("user_{:04}", i)).collect();

    let mut group = c.benchmark_group("biofield_batch");

    group.bench_function("batch_10_users", |b| {
        b.iter(|| {
            let results: Vec<_> = user_ids
                .iter()
                .map(|id| generate_metrics_for_user(black_box(id.as_str())))
                .collect();
            black_box(results)
        })
    });

    // Parameterised batch sizes
    for size in [1usize, 5, 20, 50] {
        let ids: Vec<String> = (0..size).map(|i| format!("bench_user_{}", i)).collect();
        group.bench_with_input(BenchmarkId::new("batch_users", size), &ids, |b, ids| {
            b.iter(|| {
                let results: Vec<_> = ids
                    .iter()
                    .map(|id| generate_metrics_for_user(black_box(id.as_str())))
                    .collect();
                black_box(results)
            })
        });
    }

    group.finish();
}

/// Benchmark: full engine calculate() via ConsciousnessEngine trait (async)
fn bench_engine_calculate(c: &mut Criterion) {
    let engine = BiofieldEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("biofield_batch");

    // Without birth data (mock path)
    group.bench_function("engine_calculate_mock_path", |b| {
        b.iter(|| {
            let input = EngineInput {
                birth_data: None,
                current_time: Utc::now(),
                location: None,
                precision: Precision::Standard,
                options: HashMap::new(),
            };
            let result = rt.block_on(engine.calculate(black_box(input)));
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(
    biofield_core_benches,
    bench_generate_mock_metrics_seeded,
    bench_chakra_wisdom_lookup,
    bench_metric_interpretation,
    bench_witness_prompt,
);

criterion_group!(
    biofield_batch_benches,
    bench_batch_user_metrics,
    bench_engine_calculate,
);

criterion_main!(biofield_core_benches, biofield_batch_benches);
