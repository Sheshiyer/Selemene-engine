//! Transits Engine Performance Benchmarks
//!
//! P4-W1-S1-07: Criterion benchmarks for Transits engine subsystems.
//! Two benchmark groups:
//!   transits_calculation — full engine calculate + planetary position computation
//!   transits_analysis    — aspect detection, Sade Sati, and period quality scoring

use chrono::{TimeZone, Utc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use engine_human_design::EphemerisCalculator;
use engine_transits::{
    aspects::{calculate_aspects, calculate_aspects_with_orbs, significant_aspects},
    ephemeris::calculate_all_positions,
    sade_sati::detect_sade_sati,
    TransitsEngine,
};
use noesis_core::{BirthData, ConsciousnessEngine, EngineInput, Precision};
use std::collections::HashMap;

fn make_ephemeris_calculator() -> EphemerisCalculator {
    EphemerisCalculator::new("")
}

/// Canonical birth input used throughout (1991-08-13 13:31 IST)
fn canonical_input() -> EngineInput {
    EngineInput {
        birth_data: Some(BirthData {
            name: None,
            date: "1991-08-13".to_string(),
            time: Some("13:31".to_string()),
            latitude: 12.934,
            longitude: 77.6214,
            timezone: "Asia/Kolkata".to_string(),
        }),
        current_time: Utc.with_ymd_and_hms(2026, 3, 10, 1, 0, 0).unwrap(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

// ── Group 1: transits_calculation ────────────────────────────────────────────

fn bench_engine_calculate(c: &mut Criterion) {
    let engine = TransitsEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("transits_calculation");

    // Full end-to-end engine calculate
    group.bench_function("engine_calculate_canonical", |b| {
        b.iter(|| {
            let result = rt.block_on(engine.calculate(black_box(canonical_input())));
            black_box(result)
        })
    });

    // Planetary position calculation for multiple dates
    let dates = [
        Utc.with_ymd_and_hms(1991, 8, 13, 8, 1, 0).unwrap(),
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2024, 6, 21, 12, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 3, 10, 1, 0, 0).unwrap(),
    ];

    let calc = make_ephemeris_calculator();
    for (i, &date) in dates.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("planetary_positions", i),
            &date,
            |b, &dt| {
                b.iter(|| {
                    let positions = calculate_all_positions(&calc, black_box(&dt));
                    black_box(positions)
                })
            },
        );
    }

    group.finish();
}

// ── Group 2: transits_analysis ───────────────────────────────────────────────

fn bench_analysis(c: &mut Criterion) {
    let calc = make_ephemeris_calculator();

    let birth_dt = Utc.with_ymd_and_hms(1991, 8, 13, 8, 1, 0).unwrap();
    let transit_dt = Utc.with_ymd_and_hms(2026, 3, 10, 1, 0, 0).unwrap();

    // Pre-compute positions once for analysis benchmarks
    let natal_positions = calculate_all_positions(&calc, &birth_dt)
        .expect("setup: natal position calculation failed");
    let transit_positions = calculate_all_positions(&calc, &transit_dt)
        .expect("setup: transit position calculation failed");

    let mut group = c.benchmark_group("transits_analysis");

    // Aspect calculation (default orbs)
    group.bench_function("calculate_aspects_default_orbs", |b| {
        b.iter(|| {
            let aspects =
                calculate_aspects(black_box(&transit_positions), black_box(&natal_positions));
            black_box(aspects)
        })
    });

    // Aspect calculation (tight 0.5× orbs)
    group.bench_function("calculate_aspects_tight_orbs", |b| {
        b.iter(|| {
            let aspects = calculate_aspects_with_orbs(
                black_box(&transit_positions),
                black_box(&natal_positions),
                Some(0.5),
            );
            black_box(aspects)
        })
    });

    // Significant aspects filter (from pre-calculated full list)
    let all_aspects = calculate_aspects(&transit_positions, &natal_positions);
    group.bench_function("significant_aspects_filter", |b| {
        b.iter(|| {
            let sig = significant_aspects(black_box(&all_aspects));
            black_box(sig)
        })
    });

    // Sade Sati detection
    group.bench_function("detect_sade_sati", |b| {
        b.iter(|| {
            let status =
                detect_sade_sati(black_box(&transit_positions), black_box(&natal_positions));
            black_box(status)
        })
    });

    // Combined analysis for multiple transit dates
    let transit_dates = [
        Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2026, 3, 10, 1, 0, 0).unwrap(),
        Utc.with_ymd_and_hms(2027, 12, 31, 23, 59, 0).unwrap(),
    ];
    for (i, &tdt) in transit_dates.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("aspects_and_sade_sati", i),
            &tdt,
            |b, &tdt| {
                b.iter(|| {
                    let tpos =
                        calculate_all_positions(&calc, black_box(&tdt)).expect("transit positions");
                    let aspects = calculate_aspects(&tpos, &natal_positions);
                    let sade_sati = detect_sade_sati(&tpos, &natal_positions);
                    black_box((aspects, sade_sati))
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_engine_calculate, bench_analysis);
criterion_main!(benches);
