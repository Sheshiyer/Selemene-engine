//! Gene Keys Engine Performance Benchmarks
//!
//! W1-S7-07: Criterion benchmarks for Gene Keys engine subsystems.
//! Targets: Full chart <50ms, Activation sequences <5ms, Frequency assessment <5ms.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use engine_gene_keys::{
    GeneKeysEngine, ConsciousnessEngine, EngineInput,
    ActivationSequence, GeneKeysChart, GeneKeyActivation, ActivationSource,
    assess_frequencies, generate_transformation_pathways, generate_complete_pathways,
    generate_witness_prompt, get_gene_key,
};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

/// Helper: create EngineInput with hd_gates (Mode 2)
fn create_engine_input(ps: u8, pe: u8, ds: u8, de: u8) -> EngineInput {
    let mut options = HashMap::new();
    options.insert("hd_gates".to_string(), json!({
        "personality_sun": ps,
        "personality_earth": pe,
        "design_sun": ds,
        "design_earth": de
    }));

    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: noesis_core::Precision::Standard,
        options,
    }
}

/// Helper: create a GeneKeysChart for benchmarking
fn create_test_chart(ps: u8, pe: u8, ds: u8, de: u8) -> GeneKeysChart {
    GeneKeysChart {
        activation_sequence: ActivationSequence {
            lifes_work: (ps, pe),
            evolution: (ds, de),
            radiance: (ps, ds),
            purpose: (pe, de),
        },
        active_keys: vec![
            GeneKeyActivation {
                key_number: ps,
                line: 3,
                source: ActivationSource::PersonalitySun,
                gene_key_data: get_gene_key(ps).cloned(),
            },
            GeneKeyActivation {
                key_number: pe,
                line: 3,
                source: ActivationSource::PersonalityEarth,
                gene_key_data: get_gene_key(pe).cloned(),
            },
            GeneKeyActivation {
                key_number: ds,
                line: 3,
                source: ActivationSource::DesignSun,
                gene_key_data: get_gene_key(ds).cloned(),
            },
            GeneKeyActivation {
                key_number: de,
                line: 3,
                source: ActivationSource::DesignEarth,
                gene_key_data: get_gene_key(de).cloned(),
            },
        ],
    }
}

/// Benchmark: Full Gene Keys engine calculation (Mode 2: hd_gates)
fn bench_full_gene_keys_chart(c: &mut Criterion) {
    let engine = GeneKeysEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("gk_full_chart", |b| {
        b.iter(|| {
            let input = create_engine_input(17, 18, 45, 26);
            let result = rt.block_on(engine.calculate(black_box(input)));
            black_box(result)
        })
    });
}

/// Benchmark: Activation sequence calculation
fn bench_activation_sequences(c: &mut Criterion) {
    c.bench_function("gk_activation_sequences", |b| {
        b.iter(|| {
            let seq = ActivationSequence {
                lifes_work: (black_box(17), black_box(18)),
                evolution: (black_box(45), black_box(26)),
                radiance: (black_box(17), black_box(45)),
                purpose: (black_box(18), black_box(26)),
            };
            black_box(seq)
        })
    });
}

/// Benchmark: Frequency assessment (4 keys with consciousness level)
fn bench_frequency_assessment(c: &mut Criterion) {
    let chart = create_test_chart(17, 18, 45, 26);

    c.bench_function("gk_frequency_assessment", |b| {
        b.iter(|| {
            let assessments = assess_frequencies(black_box(&chart), Some(3));
            black_box(assessments)
        })
    });
}

/// Benchmark: Transformation pathway generation
fn bench_transformation_pathways(c: &mut Criterion) {
    let chart = create_test_chart(36, 6, 55, 49);
    let assessments = assess_frequencies(&chart, Some(2));

    c.bench_function("gk_transformation_pathways", |b| {
        b.iter(|| {
            let pathways = generate_transformation_pathways(black_box(&assessments));
            black_box(pathways)
        })
    });
}

/// Benchmark: Complete pathways (both Shadow->Gift and Gift->Siddhi)
fn bench_complete_pathways(c: &mut Criterion) {
    let chart = create_test_chart(1, 2, 13, 7);
    let assessments = assess_frequencies(&chart, Some(3));

    c.bench_function("gk_complete_pathways", |b| {
        b.iter(|| {
            let pathways = generate_complete_pathways(black_box(&assessments));
            black_box(pathways)
        })
    });
}

/// Benchmark: Witness prompt generation
fn bench_witness_prompt_generation(c: &mut Criterion) {
    let chart = create_test_chart(1, 2, 13, 7);

    c.bench_function("gk_witness_prompt", |b| {
        b.iter(|| {
            let prompt = generate_witness_prompt(black_box(&chart), black_box(3));
            black_box(prompt)
        })
    });
}

/// Benchmark: Gene Key lookup (all 64)
fn bench_gene_key_lookup_all(c: &mut Criterion) {
    c.bench_function("gk_lookup_all_64", |b| {
        b.iter(|| {
            for i in 1..=64 {
                black_box(get_gene_key(black_box(i)));
            }
        })
    });
}

/// Benchmark: Individual Gene Key lookup (parameterized)
fn bench_gene_key_lookup_individual(c: &mut Criterion) {
    let mut group = c.benchmark_group("gk_individual_lookup");

    for key_num in [1u8, 16, 32, 48, 64] {
        group.bench_with_input(
            BenchmarkId::from_parameter(key_num),
            &key_num,
            |b, &num| {
                b.iter(|| black_box(get_gene_key(black_box(num))))
            },
        );
    }
    group.finish();
}

/// Benchmark: Frequency assessment with different consciousness levels
fn bench_frequency_levels(c: &mut Criterion) {
    let chart = create_test_chart(17, 18, 45, 26);
    let mut group = c.benchmark_group("gk_frequency_by_level");

    for level in [1u8, 3, 5] {
        group.bench_with_input(
            BenchmarkId::from_parameter(level),
            &level,
            |b, &lvl| {
                b.iter(|| {
                    black_box(assess_frequencies(black_box(&chart), Some(lvl)))
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    gk_benches,
    bench_full_gene_keys_chart,
    bench_activation_sequences,
    bench_frequency_assessment,
    bench_transformation_pathways,
    bench_complete_pathways,
    bench_witness_prompt_generation,
    bench_gene_key_lookup_all,
    bench_gene_key_lookup_individual,
    bench_frequency_levels,
);
criterion_main!(gk_benches);
