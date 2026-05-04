//! Gene Keys Engine Benchmark Suite
//!
//! [P4-W1-S1-04] Criterion benchmark groups for:
//! - Single Gene Key lookup
//! - Full 64-key profile generation
//! - Shadow-Gift-Siddhi resolution

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use engine_gene_keys::{
    assess_frequencies, gene_keys, generate_complete_pathways, generate_transformation_pathways,
    get_gene_key, ActivationSequence, ActivationSource, GeneKeyActivation, GeneKeysChart,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal GeneKeysChart from four Sun/Earth gate numbers.
fn make_chart(ps: u8, pe: u8, ds: u8, de: u8) -> GeneKeysChart {
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

/// Build a full 64-key GeneKeysChart (all keys present once).
fn make_full_chart() -> GeneKeysChart {
    let active_keys: Vec<GeneKeyActivation> = (1u8..=64)
        .map(|n| GeneKeyActivation {
            key_number: n,
            line: ((n % 6) + 1),
            source: ActivationSource::PersonalitySun,
            gene_key_data: get_gene_key(n).cloned(),
        })
        .collect();

    GeneKeysChart {
        activation_sequence: ActivationSequence {
            lifes_work: (1, 2),
            evolution: (63, 64),
            radiance: (1, 63),
            purpose: (2, 64),
        },
        active_keys,
    }
}

// ---------------------------------------------------------------------------
// Group 1: single-key lookup
// ---------------------------------------------------------------------------

/// Parameterised single-key lookup across representative keys.
fn bench_single_key_lookup(c: &mut Criterion) {
    // Pre-warm static store (initialised lazily via OnceLock)
    let _ = gene_keys();

    let mut group = c.benchmark_group("single_key_lookup");

    for &key in &[1u8, 16, 32, 48, 64] {
        group.bench_with_input(BenchmarkId::from_parameter(key), &key, |b, &k| {
            b.iter(|| black_box(get_gene_key(black_box(k))))
        });
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 2: full 64-key profile
// ---------------------------------------------------------------------------

/// Build the full 64-key profile chart.
fn bench_full_profile_build(c: &mut Criterion) {
    let _ = gene_keys();

    let mut group = c.benchmark_group("full_profile_64_keys");

    group.bench_function("build_chart", |b| b.iter(|| black_box(make_full_chart())));

    group.bench_function("iterate_all_keys", |b| {
        b.iter(|| {
            for i in 1u8..=64 {
                black_box(get_gene_key(black_box(i)));
            }
        })
    });

    group.bench_function("assess_frequencies_64", |b| {
        let chart = make_full_chart();
        b.iter(|| black_box(assess_frequencies(black_box(&chart), Some(3))))
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Group 3: shadow-gift-siddhi resolution
// ---------------------------------------------------------------------------

/// Resolution of Shadow→Gift→Siddhi pathways.
fn bench_shadow_gift_siddhi(c: &mut Criterion) {
    let _ = gene_keys();

    let mut group = c.benchmark_group("shadow_gift_siddhi");

    // Shadow-level assessments (consciousness_level = 1)
    group.bench_function("assess_shadow_level", |b| {
        let chart = make_chart(17, 18, 45, 26);
        b.iter(|| black_box(assess_frequencies(black_box(&chart), Some(1))))
    });

    // Gift-level assessments (consciousness_level = 4)
    group.bench_function("assess_gift_level", |b| {
        let chart = make_chart(17, 18, 45, 26);
        b.iter(|| black_box(assess_frequencies(black_box(&chart), Some(4))))
    });

    // Siddhi-level assessments (consciousness_level = 6)
    group.bench_function("assess_siddhi_level", |b| {
        let chart = make_chart(17, 18, 45, 26);
        b.iter(|| black_box(assess_frequencies(black_box(&chart), Some(6))))
    });

    // Transformation pathway generation (Shadow→Gift)
    group.bench_function("transformation_pathways", |b| {
        let chart = make_chart(36, 6, 55, 49);
        let assessments = assess_frequencies(&chart, Some(2));
        b.iter(|| black_box(generate_transformation_pathways(black_box(&assessments))))
    });

    // Complete pathways (both arcs)
    group.bench_function("complete_pathways", |b| {
        let chart = make_chart(1, 2, 13, 7);
        let assessments = assess_frequencies(&chart, Some(3));
        b.iter(|| black_box(generate_complete_pathways(black_box(&assessments))))
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion wiring
// ---------------------------------------------------------------------------

criterion_group!(
    gene_keys_bench,
    bench_single_key_lookup,
    bench_full_profile_build,
    bench_shadow_gift_siddhi,
);
criterion_main!(gene_keys_bench);
