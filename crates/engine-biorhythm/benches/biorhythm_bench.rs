//! Biorhythm Engine Performance Benchmarks
//!
//! P4-W1-S1-03: Criterion benchmark suite for the biorhythm engine.
//! Three groups:
//!   biorhythm_single_day     — single-day cycle calculation
//!   biorhythm_30_day_range   — 30-day forecast window
//!   biorhythm_cycle_analysis — cycle-analysis across diverse birth dates

use chrono::{TimeZone, Utc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use engine_biorhythm::BiorhythmEngine;
use noesis_core::{BirthData, ConsciousnessEngine, EngineInput, Precision};
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_input(birth_date: &str, forecast_days: i64) -> EngineInput {
    EngineInput {
        birth_data: Some(BirthData {
            name: Some("Benchmark".to_string()),
            date: birth_date.to_string(),
            time: Some("12:00".to_string()),
            latitude: 0.0,
            longitude: 0.0,
            timezone: "UTC".to_string(),
        }),
        current_time: Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap(),
        location: None,
        options: {
            let mut m = std::collections::HashMap::new();
            m.insert("forecast_days".to_string(), json!(forecast_days));
            m
        },
        precision: Precision::Standard,
    }
}

// Shared runtime used by all benchmark functions to avoid per-function setup overhead.
fn shared_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// Benchmark: single-day calculation (no forecast)
// ---------------------------------------------------------------------------

fn benchmark_single_day(c: &mut Criterion) {
    let engine = BiorhythmEngine::new();
    let rt = shared_runtime();

    let mut group = c.benchmark_group("biorhythm_single_day");

    let birth_dates = ["1985-06-15", "1990-01-15", "1978-11-03", "2000-05-20"];

    for date in &birth_dates {
        let input = make_input(date, 0);
        group.bench_with_input(
            BenchmarkId::from_parameter(date),
            &input,
            |b, input| {
                b.iter(|| {
                    rt.block_on(engine.calculate(black_box(input.clone())))
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: 30-day forecast range
// ---------------------------------------------------------------------------

fn benchmark_30_day_range(c: &mut Criterion) {
    let engine = BiorhythmEngine::new();
    let rt = shared_runtime();

    let mut group = c.benchmark_group("biorhythm_30_day_range");

    let birth_dates = ["1985-06-15", "1990-01-15", "1978-11-03"];

    for date in &birth_dates {
        let input = make_input(date, 30);
        group.bench_with_input(
            BenchmarkId::from_parameter(date),
            &input,
            |b, input| {
                b.iter(|| {
                    rt.block_on(engine.calculate(black_box(input.clone())))
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: cycle analysis across many birth dates
// ---------------------------------------------------------------------------

fn benchmark_cycle_analysis(c: &mut Criterion) {
    let engine = BiorhythmEngine::new();
    let rt = shared_runtime();

    // 10 diverse birth dates to exercise all cycle phases
    let birth_dates: Vec<String> = (0..10)
        .map(|i| format!("{}-{:02}-15", 1975 + i, (i % 12) + 1))
        .collect();

    let inputs: Vec<EngineInput> = birth_dates.iter().map(|d| make_input(d, 7)).collect();

    c.bench_function("biorhythm_cycle_analysis_10_births", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut out = Vec::with_capacity(inputs.len());
                for input in &inputs {
                    out.push(engine.calculate(black_box(input.clone())).await.unwrap());
                }
                black_box(out)
            })
        })
    });
}

// ---------------------------------------------------------------------------
// criterion wiring
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    benchmark_single_day,
    benchmark_30_day_range,
    benchmark_cycle_analysis,
);
criterion_main!(benches);
