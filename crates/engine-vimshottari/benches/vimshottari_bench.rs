//! Vimshottari Dasha Benchmark Suite
//!
//! Criterion groups covering three dasha-level granularities:
//!   1. `mahadasha_calculation` — nakshatra resolution, dasha balance, 9-period generation
//!   2. `antardasha_subdivision` — 81 antardasha sub-period calculations
//!   3. `full_120year_timeline`  — complete 3-level nested 120-year timeline (729 pratyantardashas)

use chrono::{TimeZone, Utc};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use engine_vimshottari::{
    calculate_antardashas, calculate_complete_timeline, calculate_dasha_balance,
    calculate_mahadashas, calculate_pratyantardashas, get_nakshatra_from_longitude, VedicPlanet,
};

// Midpoint balance (≈ half of a 7-year Ketu Mahadasha) used as a
// representative fixed input when varying the starting planet.
const BENCH_BALANCE_YEARS: f64 = 4.375;

/// Returns the birth time used across all benchmark groups.
fn birth_time() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap()
}

/// Pre-built flat mahadasha list (no antardasha/pratyantardasha) for reuse.
fn flat_mahadashas() -> Vec<engine_vimshottari::models::Mahadasha> {
    let nak = get_nakshatra_from_longitude(125.0);
    let balance = calculate_dasha_balance(125.0, nak);
    calculate_mahadashas(birth_time(), nak.ruling_planet, balance)
}

// ---------------------------------------------------------------------------
// Group 1: mahadasha_calculation
// ---------------------------------------------------------------------------

fn bench_nakshatra_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("mahadasha_calculation");

    // Nakshatra resolution from Moon longitude
    let longitudes = [5.0_f64, 45.0, 125.0, 200.0, 315.0];
    for lon in longitudes {
        group.bench_with_input(
            BenchmarkId::new("nakshatra_from_longitude", format!("{lon:.0}deg")),
            &lon,
            |b, &l| b.iter(|| black_box(get_nakshatra_from_longitude(black_box(l)))),
        );
    }

    // Dasha balance calculation
    let nak = get_nakshatra_from_longitude(125.0);
    group.bench_function("dasha_balance", |b| {
        b.iter(|| black_box(calculate_dasha_balance(black_box(125.0), black_box(nak))))
    });

    // Generate 9 Mahadasha periods
    let balance = calculate_dasha_balance(125.0, nak);
    let ruling_planet = nak.ruling_planet;
    group.bench_function("generate_9_mahadashas", |b| {
        b.iter(|| {
            black_box(calculate_mahadashas(
                black_box(birth_time()),
                black_box(ruling_planet),
                black_box(balance),
            ))
        })
    });

    // Mahadasha generation across different starting planets
    for planet in [
        VedicPlanet::Sun,
        VedicPlanet::Moon,
        VedicPlanet::Mars,
        VedicPlanet::Jupiter,
        VedicPlanet::Saturn,
    ] {
        group.bench_with_input(
            BenchmarkId::new("mahadashas_starting_planet", format!("{planet:?}")),
            &planet,
            |b, &p| {
                b.iter(|| {
                    black_box(calculate_mahadashas(
                        black_box(birth_time()),
                        black_box(p),
                        BENCH_BALANCE_YEARS,
                    ))
                })
            },
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 2: antardasha_subdivision
// ---------------------------------------------------------------------------

fn bench_antardasha_subdivision(c: &mut Criterion) {
    let mut group = c.benchmark_group("antardasha_subdivision");

    let mahadashas = flat_mahadashas();

    // Antardasha calculation for a single Mahadasha
    let first_maha = &mahadashas[0];
    group.bench_function("antardashas_single_mahadasha", |b| {
        b.iter(|| black_box(calculate_antardashas(black_box(first_maha))))
    });

    // All 81 antardasha periods (9 mahadashas × 9 antardashas each)
    group.bench_function("antardashas_all_81", |b| {
        b.iter(|| {
            let mut all = Vec::with_capacity(81);
            for maha in black_box(&mahadashas) {
                all.extend(calculate_antardashas(maha));
            }
            black_box(all)
        })
    });

    // Pratyantardasha calculation per antardasha (9 per antardasha)
    let antardashas = calculate_antardashas(first_maha);
    let first_antar = &antardashas[0];
    group.bench_function("pratyantardashas_single_antardasha", |b| {
        b.iter(|| black_box(calculate_pratyantardashas(black_box(first_antar))))
    });

    // All 81 antardasha periods per mahadasha, measured individually
    for (idx, maha) in mahadashas.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("antardashas_per_mahadasha", idx),
            maha,
            |b, m| b.iter(|| black_box(calculate_antardashas(black_box(m)))),
        );
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 3: full_120year_timeline
// ---------------------------------------------------------------------------

fn bench_full_120year_timeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_120year_timeline");

    let mahadashas = flat_mahadashas();

    // End-to-end 120-year timeline (729 pratyantardashas)
    group.bench_function("complete_timeline_729_pratyantardashas", |b| {
        b.iter(|| black_box(calculate_complete_timeline(black_box(mahadashas.clone()))))
    });

    // Timeline generation from different nakshatras to verify consistent cost
    for (name, longitude) in [("Ashwini", 5.0_f64), ("Magha", 125.0), ("Mula", 245.0)] {
        group.bench_with_input(
            BenchmarkId::new("full_timeline_by_nakshatra", name),
            &longitude,
            |b, &lng| {
                let nak = get_nakshatra_from_longitude(lng);
                let balance = calculate_dasha_balance(lng, nak);
                let mds = calculate_mahadashas(birth_time(), nak.ruling_planet, balance);
                b.iter(|| black_box(calculate_complete_timeline(black_box(mds.clone()))))
            },
        );
    }

    // Batch: generate 5 independent full timelines (simulating concurrent API calls)
    let batch_inputs: Vec<_> = [
        (VedicPlanet::Ketu, BENCH_BALANCE_YEARS),
        (VedicPlanet::Venus, 12.0),
        (VedicPlanet::Sun, 3.0),
        (VedicPlanet::Moon, 7.5),
        (VedicPlanet::Mars, 5.0),
    ]
    .iter()
    .map(|&(planet, balance)| calculate_mahadashas(birth_time(), planet, balance))
    .collect();

    group.bench_function("batch_5_full_timelines", |b| {
        b.iter(|| {
            let results: Vec<_> = batch_inputs
                .iter()
                .map(|mds| calculate_complete_timeline(black_box(mds.clone())))
                .collect();
            black_box(results)
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion entry-points
// ---------------------------------------------------------------------------

criterion_group!(mahadasha_calculation, bench_nakshatra_lookup,);

criterion_group!(antardasha_subdivision, bench_antardasha_subdivision,);

criterion_group!(full_120year_timeline, bench_full_120year_timeline,);

criterion_main!(
    mahadasha_calculation,
    antardasha_subdivision,
    full_120year_timeline,
);
