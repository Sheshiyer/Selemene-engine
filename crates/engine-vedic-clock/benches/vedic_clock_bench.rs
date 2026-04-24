//! VedicClock Engine Performance Benchmarks
//!
//! Criterion benchmark suites covering:
//!   Group 1 – core calculation path (organ lookup, dosha mapping, temporal recommendations)
//!   Group 2 – batch operations (all-hours sweep, multiple timezones, engine calculate)

use chrono::{Duration, TimeZone, Utc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use engine_vedic_clock::{
    dosha_times, get_current_organ, get_dosha_for_hour, get_local_hour,
    get_temporal_recommendation, generate_witness_prompt, minutes_until_next_transition,
    organ_clock, synthesize_organ_dosha,
    ConsciousnessEngine, EngineInput, VedicClockEngine,
};
use noesis_core::Precision;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Group 1: core calculation path
// ---------------------------------------------------------------------------

/// Benchmark: get_current_organ for a single datetime + timezone offset
fn bench_get_current_organ(c: &mut Criterion) {
    let dt = Utc.with_ymd_and_hms(2024, 6, 15, 8, 30, 0).unwrap();
    let mut group = c.benchmark_group("vedic_clock_core");

    // UTC
    group.bench_function("get_current_organ_utc", |b| {
        b.iter(|| black_box(get_current_organ(black_box(dt), black_box(0))))
    });

    // IST (+5:30 = +330 minutes)
    group.bench_function("get_current_organ_ist", |b| {
        b.iter(|| black_box(get_current_organ(black_box(dt), black_box(330))))
    });

    // PST (−8:00 = −480 minutes)
    group.bench_function("get_current_organ_pst", |b| {
        b.iter(|| black_box(get_current_organ(black_box(dt), black_box(-480))))
    });

    // Parameterised over hours of the day
    let base_dt = Utc.with_ymd_and_hms(2024, 6, 15, 0, 0, 0).unwrap();
    for h in [0u32, 3, 7, 11, 15, 19, 23] {
        let dt_h = base_dt + Duration::hours(h as i64);
        group.bench_with_input(
            BenchmarkId::new("get_current_organ_hour", h),
            &dt_h,
            |b, &dt| b.iter(|| black_box(get_current_organ(dt, 0))),
        );
    }

    group.finish();
}

/// Benchmark: dosha_times() and get_dosha_for_hour across the day
fn bench_dosha_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vedic_clock_core");

    group.bench_function("dosha_times_all", |b| {
        b.iter(|| black_box(dosha_times()))
    });

    group.bench_function("get_dosha_for_hour_noon", |b| {
        b.iter(|| black_box(get_dosha_for_hour(black_box(12))))
    });

    // All 24 hours
    group.bench_function("get_dosha_all_24h", |b| {
        b.iter(|| {
            let results: Vec<_> = (0u8..24)
                .map(|h| black_box(get_dosha_for_hour(black_box(h))))
                .collect();
            black_box(results)
        })
    });

    group.finish();
}

/// Benchmark: organ_clock() – loads all 12 organ windows
fn bench_organ_clock_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("vedic_clock_core");

    group.bench_function("organ_clock_all_12", |b| {
        b.iter(|| black_box(organ_clock()))
    });

    group.finish();
}

/// Benchmark: get_temporal_recommendation with and without panchanga
fn bench_temporal_recommendation(c: &mut Criterion) {
    let dt = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
    let mut group = c.benchmark_group("vedic_clock_core");

    // Without panchanga overlay
    group.bench_function("temporal_recommendation_bare", |b| {
        b.iter(|| {
            black_box(get_temporal_recommendation(
                black_box(dt),
                black_box(330),
                black_box(None),
                black_box(None),
            ))
        })
    });

    // With tithi + nakshatra overlay
    group.bench_function("temporal_recommendation_with_panchanga", |b| {
        b.iter(|| {
            black_box(get_temporal_recommendation(
                black_box(dt),
                black_box(330),
                black_box(Some(5)),
                black_box(Some(12)),
            ))
        })
    });

    group.finish();
}

/// Benchmark: witness prompt generation
fn bench_witness_prompt(c: &mut Criterion) {
    let dt = Utc.with_ymd_and_hms(2024, 6, 15, 8, 0, 0).unwrap();
    let organ = get_current_organ(dt, 0);
    let local_hour = get_local_hour(dt, 0);
    let dosha = get_dosha_for_hour(local_hour);

    let mut group = c.benchmark_group("vedic_clock_core");

    group.bench_function("witness_prompt_level_1", |b| {
        b.iter(|| {
            black_box(generate_witness_prompt(
                black_box(&organ.organ),
                black_box(&dosha.dosha),
                black_box(1),
            ))
        })
    });

    group.bench_function("witness_prompt_level_4", |b| {
        b.iter(|| {
            black_box(generate_witness_prompt(
                black_box(&organ.organ),
                black_box(&dosha.dosha),
                black_box(4),
            ))
        })
    });

    group.bench_function("witness_prompt_level_6", |b| {
        b.iter(|| {
            black_box(generate_witness_prompt(
                black_box(&organ.organ),
                black_box(&dosha.dosha),
                black_box(6),
            ))
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 2: batch operations
// ---------------------------------------------------------------------------

/// Benchmark: sweep all 24 hours to get organ windows (full-day simulation)
fn bench_all_hours_sweep(c: &mut Criterion) {
    let base_dt = Utc.with_ymd_and_hms(2024, 6, 15, 0, 0, 0).unwrap();
    let mut group = c.benchmark_group("vedic_clock_batch");

    group.bench_function("organ_sweep_24h_utc", |b| {
        b.iter(|| {
            let windows: Vec<_> = (0u32..24)
                .map(|h| {
                    let dt = base_dt + Duration::hours(h as i64);
                    black_box(get_current_organ(dt, 0))
                })
                .collect();
            black_box(windows)
        })
    });

    group.bench_function("synthesis_sweep_24h", |b| {
        b.iter(|| {
            let results: Vec<_> = (0u32..24)
                .map(|h| {
                    let dt = base_dt + Duration::hours(h as i64);
                    black_box(synthesize_organ_dosha(dt, 0))
                })
                .collect();
            black_box(results)
        })
    });

    group.finish();
}

/// Benchmark: batch temporal recommendations across multiple timezones
fn bench_batch_timezone_recommendations(c: &mut Criterion) {
    let dt = Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    // Major timezone offsets in minutes
    let offsets: Vec<i32> = vec![-720, -480, -300, 0, 60, 330, 540, 600];

    let mut group = c.benchmark_group("vedic_clock_batch");

    group.bench_function("batch_8_timezones", |b| {
        b.iter(|| {
            let results: Vec<_> = offsets
                .iter()
                .map(|&tz| get_temporal_recommendation(black_box(dt), black_box(tz), None, None))
                .collect();
            black_box(results)
        })
    });

    // Parameterised by number of timezones
    for n in [4usize, 8, 16] {
        let offsets_n: Vec<i32> = (0..n as i32).map(|i| i * 60 - (n as i32 / 2) * 60).collect();
        group.bench_with_input(
            BenchmarkId::new("timezone_recommendations", n),
            &offsets_n,
            |b, offsets| {
                b.iter(|| {
                    let results: Vec<_> = offsets
                        .iter()
                        .map(|&tz| {
                            get_temporal_recommendation(
                                black_box(dt),
                                black_box(tz),
                                None,
                                None,
                            )
                        })
                        .collect();
                    black_box(results)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark: full engine calculate() via ConsciousnessEngine trait (async)
fn bench_engine_calculate(c: &mut Criterion) {
    let engine = VedicClockEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("vedic_clock_batch");

    // Minimal input (uses current time, UTC)
    group.bench_function("engine_calculate_utc", |b| {
        b.iter(|| {
            let mut opts = HashMap::new();
            opts.insert("timezone".to_string(), serde_json::json!("UTC"));
            let input = EngineInput {
                birth_data: None,
                current_time: Utc::now(),
                location: None,
                precision: Precision::Standard,
                options: opts,
            };
            let result = rt.block_on(engine.calculate(black_box(input)));
            black_box(result)
        })
    });

    // With IST timezone string
    group.bench_function("engine_calculate_ist", |b| {
        b.iter(|| {
            let mut opts = HashMap::new();
            opts.insert("timezone".to_string(), serde_json::json!("Asia/Kolkata"));
            let input = EngineInput {
                birth_data: None,
                current_time: Utc::now(),
                location: None,
                precision: Precision::Standard,
                options: opts,
            };
            let result = rt.block_on(engine.calculate(black_box(input)));
            black_box(result)
        })
    });

    group.finish();
}

/// Benchmark: minutes_until_next_transition across all 24 hours
fn bench_transition_countdown(c: &mut Criterion) {
    let base_dt = Utc.with_ymd_and_hms(2024, 6, 15, 0, 0, 0).unwrap();
    let mut group = c.benchmark_group("vedic_clock_batch");

    group.bench_function("transition_countdown_24h", |b| {
        b.iter(|| {
            let results: Vec<_> = (0u32..24)
                .map(|h| {
                    let dt = base_dt + Duration::hours(h as i64);
                    black_box(minutes_until_next_transition(dt, 0))
                })
                .collect();
            black_box(results)
        })
    });

    group.finish();
}

criterion_group!(
    vedic_clock_core_benches,
    bench_get_current_organ,
    bench_dosha_calculations,
    bench_organ_clock_data,
    bench_temporal_recommendation,
    bench_witness_prompt,
);

criterion_group!(
    vedic_clock_batch_benches,
    bench_all_hours_sweep,
    bench_batch_timezone_recommendations,
    bench_engine_calculate,
    bench_transition_countdown,
);

criterion_main!(vedic_clock_core_benches, vedic_clock_batch_benches);
