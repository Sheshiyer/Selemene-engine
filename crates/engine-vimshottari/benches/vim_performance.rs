//! Vimshottari Dasha Engine Performance Benchmarks
//!
//! W1-S7-08: Criterion benchmarks for Vimshottari engine subsystems.
//! Targets: Full 120y timeline <200ms, 729 pratyantardashas <100ms, current period <5ms.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use engine_vimshottari::{
    VimshottariEngine, ConsciousnessEngine, EngineInput,
    calculate_birth_nakshatra,
    calculate_dasha_balance,
    calculate_mahadashas,
    calculate_antardashas,
    calculate_complete_timeline,
    get_nakshatra_from_longitude,
    get_nakshatra,
    enrich_period_with_qualities,
    VedicPlanet, Mahadasha,
};
use engine_vimshottari::calculator::{find_current_period, calculate_upcoming_transitions};
use chrono::{Duration, TimeZone, Utc};
use noesis_core::Precision;
use serde_json::json;
use std::collections::HashMap;

fn create_moon_input(longitude: f64) -> EngineInput {
    let mut options = HashMap::new();
    options.insert("moon_longitude".to_string(), json!(longitude));
    options.insert("birth_date".to_string(), json!("1985-06-15"));
    options.insert("birth_time".to_string(), json!("14:30"));

    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options,
    }
}

fn create_test_timeline() -> Vec<Mahadasha> {
    let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    let nakshatra = get_nakshatra_from_longitude(125.0);
    let balance = calculate_dasha_balance(125.0, nakshatra);
    let mahadashas = calculate_mahadashas(birth, nakshatra.ruling_planet, balance);
    calculate_complete_timeline(mahadashas)
}

fn bench_full_120year_timeline(c: &mut Criterion) {
    let engine = VimshottariEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("vim_full_120y_timeline", |b| {
        b.iter(|| {
            let input = create_moon_input(125.0);
            let result = rt.block_on(engine.calculate(black_box(input)));
            black_box(result)
        })
    });
}

fn bench_nakshatra_from_longitude(c: &mut Criterion) {
    c.bench_function("vim_nakshatra_from_longitude", |b| {
        b.iter(|| {
            black_box(get_nakshatra_from_longitude(black_box(125.0)))
        })
    });
}

fn bench_nakshatra_calculation_ephe(c: &mut Criterion) {
    let birth_time = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();

    c.bench_function("vim_nakshatra_from_ephemeris", |b| {
        b.iter(|| {
            black_box(calculate_birth_nakshatra(black_box(birth_time), ""))
        })
    });
}

fn bench_dasha_balance(c: &mut Criterion) {
    let nakshatra = get_nakshatra_from_longitude(125.0);

    c.bench_function("vim_dasha_balance", |b| {
        b.iter(|| {
            black_box(calculate_dasha_balance(black_box(125.0), black_box(nakshatra)))
        })
    });
}

fn bench_9_mahadashas(c: &mut Criterion) {
    let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    let balance = 4.375;

    c.bench_function("vim_9_mahadashas", |b| {
        b.iter(|| {
            black_box(calculate_mahadashas(
                black_box(birth),
                black_box(VedicPlanet::Ketu),
                black_box(balance),
            ))
        })
    });
}

fn bench_81_antardashas(c: &mut Criterion) {
    let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    let mahadashas = calculate_mahadashas(birth, VedicPlanet::Ketu, 4.375);

    c.bench_function("vim_81_antardashas", |b| {
        b.iter(|| {
            let mut total = Vec::new();
            for maha in black_box(&mahadashas) {
                total.extend(calculate_antardashas(maha));
            }
            black_box(total)
        })
    });
}

fn bench_729_pratyantardashas(c: &mut Criterion) {
    let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    let mahadashas = calculate_mahadashas(birth, VedicPlanet::Ketu, 4.375);

    c.bench_function("vim_729_pratyantardashas", |b| {
        b.iter(|| {
            black_box(calculate_complete_timeline(black_box(mahadashas.clone())))
        })
    });
}

fn bench_current_period_detection(c: &mut Criterion) {
    let complete = create_test_timeline();
    let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    let query_time = birth + Duration::days(10000);

    c.bench_function("vim_current_period_search", |b| {
        b.iter(|| {
            black_box(find_current_period(black_box(&complete), black_box(query_time)))
        })
    });
}

fn bench_current_period_positions(c: &mut Criterion) {
    let complete = create_test_timeline();
    let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();

    let mut group = c.benchmark_group("vim_period_search_positions");

    for years_offset in [1, 10, 30, 60, 100] {
        let query_time = birth + Duration::days((years_offset as i64) * 365);
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}y", years_offset)),
            &query_time,
            |b, &qt| {
                b.iter(|| {
                    black_box(find_current_period(black_box(&complete), black_box(qt)))
                })
            },
        );
    }
    group.finish();
}

fn bench_upcoming_transitions(c: &mut Criterion) {
    let complete = create_test_timeline();
    let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    let query_time = birth + Duration::days(10000);

    c.bench_function("vim_upcoming_transitions_5", |b| {
        b.iter(|| {
            black_box(calculate_upcoming_transitions(
                black_box(&complete),
                black_box(query_time),
                black_box(5),
            ))
        })
    });
}

fn bench_period_enrichment(c: &mut Criterion) {
    c.bench_function("vim_period_enrichment", |b| {
        b.iter(|| {
            black_box(enrich_period_with_qualities(
                black_box(&VedicPlanet::Jupiter),
                black_box(&VedicPlanet::Saturn),
                black_box(&VedicPlanet::Mercury),
            ))
        })
    });
}

fn bench_nakshatra_lookup_all(c: &mut Criterion) {
    c.bench_function("vim_nakshatra_lookup_all_27", |b| {
        b.iter(|| {
            for i in 1..=27 {
                black_box(get_nakshatra(black_box(i)));
            }
        })
    });
}

fn bench_different_nakshatras(c: &mut Criterion) {
    let mut group = c.benchmark_group("vim_timeline_by_nakshatra");
    let birth = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();

    for (name, longitude) in [("Ashwini", 5.0), ("Magha", 125.0), ("Mula", 245.0)] {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &longitude,
            |b, &lng| {
                let nak = get_nakshatra_from_longitude(lng);
                let balance = calculate_dasha_balance(lng, nak);
                let mahadashas = calculate_mahadashas(birth, nak.ruling_planet, balance);
                b.iter(|| {
                    black_box(calculate_complete_timeline(black_box(mahadashas.clone())))
                })
            },
        );
    }
    group.finish();
}

criterion_group!(
    vim_benches,
    bench_full_120year_timeline,
    bench_nakshatra_from_longitude,
    bench_nakshatra_calculation_ephe,
    bench_dasha_balance,
    bench_9_mahadashas,
    bench_81_antardashas,
    bench_729_pratyantardashas,
    bench_current_period_detection,
    bench_current_period_positions,
    bench_upcoming_transitions,
    bench_period_enrichment,
    bench_nakshatra_lookup_all,
    bench_different_nakshatras,
);
criterion_main!(vim_benches);
