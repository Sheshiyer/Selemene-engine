//! NadaBrahman Engine Performance Benchmarks
//!
//! P4-W1-S1-07: Criterion benchmarks for NadaBrahman engine subsystems.
//! Two benchmark groups:
//!   nadabrahman_calculation — full engine calculate with various option combinations
//!   nadabrahman_data_lookup — individual data-layer lookups (raga DB, dosha, rasa, chakra)

use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use engine_nadabrahman::{
    data::{get_chakra_frequency, get_prahar_ragas, get_raga, get_ragas_for_dosha,
           get_ragas_for_rasa, raga_db},
    ConsciousnessEngine, EngineInput, NadaBrahmanEngine,
};
use noesis_core::Precision;
use serde_json::json;
use std::collections::HashMap;

fn create_engine_input(opts: &[(&str, serde_json::Value)]) -> EngineInput {
    let mut options = HashMap::new();
    for (k, v) in opts {
        options.insert(k.to_string(), v.clone());
    }
    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options,
    }
}

// ── Group 1: nadabrahman_calculation ─────────────────────────────────────────

fn bench_engine_calculate(c: &mut Criterion) {
    let engine = NadaBrahmanEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("nadabrahman_calculation");

    // Minimal — time-of-day only
    group.bench_function("calculate_time_only", |b| {
        b.iter(|| {
            let input = create_engine_input(&[]);
            let result = rt.block_on(engine.calculate(black_box(input)));
            black_box(result)
        })
    });

    // With dosha — common API pattern
    for dosha in ["vata", "pitta", "kapha"] {
        group.bench_with_input(
            BenchmarkId::new("calculate_with_dosha", dosha),
            dosha,
            |b, dosha| {
                b.iter(|| {
                    let input = create_engine_input(&[("dosha", json!(*dosha))]);
                    let result = rt.block_on(engine.calculate(black_box(input)));
                    black_box(result)
                })
            },
        );
    }

    // Full options — dosha + rasa + chakra
    group.bench_function("calculate_full_options", |b| {
        b.iter(|| {
            let input = create_engine_input(&[
                ("dosha", json!("pitta")),
                ("rasa", json!("shanta")),
                ("chakra", json!("heart")),
            ]);
            let result = rt.block_on(engine.calculate(black_box(input)));
            black_box(result)
        })
    });

    group.finish();
}

// ── Group 2: nadabrahman_data_lookup ─────────────────────────────────────────

fn bench_data_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("nadabrahman_data_lookup");

    // Raga database initialization / access (lazy_static, first call may differ)
    group.bench_function("raga_db_access", |b| {
        b.iter(|| {
            let db = raga_db();
            black_box(db.ragas.len())
        })
    });

    // Single raga lookup by number
    for number in [1u32, 29, 36, 64, 72] {
        group.bench_with_input(
            BenchmarkId::new("get_raga", number),
            &number,
            |b, &n| b.iter(|| black_box(get_raga(black_box(n)))),
        );
    }

    // Prahar (time-of-day) raga recommendations for all 8 prahars
    for prahar_number in 1u32..=8 {
        group.bench_with_input(
            BenchmarkId::new("prahar_ragas", prahar_number),
            &prahar_number,
            |b, &p| b.iter(|| black_box(get_prahar_ragas(black_box(p)))),
        );
    }

    // Dosha-based recommendations
    for dosha in ["vata", "pitta", "kapha"] {
        group.bench_with_input(
            BenchmarkId::new("ragas_for_dosha", dosha),
            dosha,
            |b, dosha| b.iter(|| black_box(get_ragas_for_dosha(dosha))),
        );
    }

    // Rasa-based recommendations for common rasas
    for rasa in ["shanta", "shringara", "karuna", "vira"] {
        group.bench_with_input(
            BenchmarkId::new("ragas_for_rasa", rasa),
            rasa,
            |b, rasa| b.iter(|| black_box(get_ragas_for_rasa(rasa))),
        );
    }

    // Chakra frequency lookup for all 7 chakras
    for chakra in ["root", "sacral", "solar_plexus", "heart", "throat", "third_eye", "crown"] {
        group.bench_with_input(
            BenchmarkId::new("chakra_frequency", chakra),
            chakra,
            |b, chakra| b.iter(|| black_box(get_chakra_frequency(chakra))),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_engine_calculate, bench_data_lookups);
criterion_main!(benches);
