//! Face Reading Engine Performance Benchmarks
//!
//! P4-W1-S1-07: Criterion benchmarks for Face Reading engine subsystems.
//! Two benchmark groups:
//!   face_reading_calculation — full engine calculate + mock analysis generation
//!   face_reading_components  — witness prompt generation and zone wisdom lookups

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use engine_face_reading::{
    all_zone_wisdom, generate_mock_analysis, generate_single_witness_prompt, get_zone_wisdom,
    ConsciousnessEngine, EngineInput, FaceReadingEngine, FaceZone,
};
use noesis_core::Precision;
use serde_json::json;
use std::collections::HashMap;

fn create_engine_input(seed: Option<u64>) -> EngineInput {
    let mut options = HashMap::new();
    if let Some(s) = seed {
        options.insert("seed".to_string(), json!(s));
    }
    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options,
    }
}

// ── Group 1: face_reading_calculation ────────────────────────────────────────

fn bench_full_engine_calculate(c: &mut Criterion) {
    let engine = FaceReadingEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("face_reading_calculation");

    // Seeded (deterministic) — typical API call with reproducible seed
    group.bench_function("engine_calculate_seeded", |b| {
        b.iter(|| {
            let input = create_engine_input(Some(42));
            let result = rt.block_on(engine.calculate(black_box(input)));
            black_box(result)
        })
    });

    // Unseeded — entropy-based generation
    group.bench_function("engine_calculate_unseeded", |b| {
        b.iter(|| {
            let input = create_engine_input(None);
            let result = rt.block_on(engine.calculate(black_box(input)));
            black_box(result)
        })
    });

    // Mock analysis generation only (no async overhead)
    for seed in [0u64, 1, 42, 12345, u64::MAX / 2] {
        group.bench_with_input(
            BenchmarkId::new("mock_analysis_seeded", seed),
            &seed,
            |b, &seed| b.iter(|| black_box(generate_mock_analysis(Some(black_box(seed))))),
        );
    }

    group.bench_function("mock_analysis_unseeded", |b| {
        b.iter(|| black_box(generate_mock_analysis(None)))
    });

    group.finish();
}

// ── Group 2: face_reading_components ─────────────────────────────────────────

fn bench_components(c: &mut Criterion) {
    let mut group = c.benchmark_group("face_reading_components");

    // Pre-compute an analysis to reuse across witness / wisdom benchmarks
    let analysis = generate_mock_analysis(Some(7));

    // Witness prompt generation at each consciousness level
    for level in 0u8..=5 {
        group.bench_with_input(
            BenchmarkId::new("witness_prompt_level", level),
            &level,
            |b, &level| {
                b.iter(|| {
                    black_box(generate_single_witness_prompt(
                        black_box(&analysis),
                        black_box(level),
                    ))
                })
            },
        );
    }

    // Zone wisdom lookups for all canonical face zones
    let zones = [
        FaceZone::Forehead,
        FaceZone::Eyes,
        FaceZone::Nose,
        FaceZone::Cheeks,
        FaceZone::Mouth,
        FaceZone::Chin,
        FaceZone::Ears,
    ];
    for zone in zones {
        group.bench_with_input(
            BenchmarkId::new("zone_wisdom_lookup", format!("{:?}", zone)),
            &zone,
            |b, &zone| b.iter(|| black_box(get_zone_wisdom(black_box(zone)))),
        );
    }

    // Full wisdom iterator (all zones)
    group.bench_function("all_zone_wisdom_iter", |b| {
        b.iter(|| {
            let count = all_zone_wisdom().count();
            black_box(count)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_full_engine_calculate, bench_components);
criterion_main!(benches);
