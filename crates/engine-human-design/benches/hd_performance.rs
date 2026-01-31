//! Human Design Engine Performance Benchmarks
//!
//! W1-S7-06: Criterion benchmarks for HD engine subsystems.
//! Targets: Full chart <100ms, 26 activations <5ms, Type/Authority <1ms each.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine_human_design::{
    HumanDesignEngine, ConsciousnessEngine, EngineInput,
    EphemerisCalculator, HDPlanet,
    longitude_to_gate_and_line,
    calculate_all_activations, calculate_personality_activations,
    analyze_centers, analyze_channels, determine_type, determine_authority,
    calculate_profile,
    generate_hd_chart, initialize_ephemeris,
    models::{Activation, Planet, Center, CenterState, Channel},
};
use chrono::{TimeZone, Utc};
use noesis_core::{BirthData, Precision};
use std::collections::HashMap;

/// Helper: create a standard EngineInput for benchmarking
fn create_bench_input() -> EngineInput {
    EngineInput {
        birth_data: Some(BirthData {
            name: Some("Benchmark".to_string()),
            date: "1990-01-15".to_string(),
            time: Some("14:30".to_string()),
            latitude: 40.7128,
            longitude: -74.0060,
            timezone: "America/New_York".to_string(),
        }),
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options: HashMap::new(),
    }
}

/// Benchmark: Full HD chart via ConsciousnessEngine::calculate
fn bench_full_hd_chart(c: &mut Criterion) {
    let engine = HumanDesignEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("hd_full_chart", |b| {
        b.iter(|| {
            let input = create_bench_input();
            let result = rt.block_on(engine.calculate(black_box(input)));
            black_box(result)
        })
    });
}

/// Benchmark: All 26 planetary activations (13 personality + 13 design)
fn bench_26_activations(c: &mut Criterion) {
    initialize_ephemeris("");
    let birth_time = Utc.with_ymd_and_hms(1990, 1, 15, 19, 30, 0).unwrap(); // UTC equivalent

    c.bench_function("hd_26_activations", |b| {
        b.iter(|| {
            black_box(calculate_all_activations(black_box(birth_time), ""))
        })
    });
}

/// Benchmark: Personality-only 13 activations
fn bench_personality_activations(c: &mut Criterion) {
    initialize_ephemeris("");
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(1990, 1, 15, 19, 30, 0).unwrap();

    c.bench_function("hd_13_personality_activations", |b| {
        b.iter(|| {
            black_box(calculate_personality_activations(black_box(&birth_time), &calculator))
        })
    });
}

/// Benchmark: Type determination from centers and channels
fn bench_type_determination(c: &mut Criterion) {
    // Pre-build centers with Sacral defined (Generator scenario)
    let mut centers = HashMap::new();
    centers.insert(Center::Head, CenterState { defined: false, gates: vec![] });
    centers.insert(Center::Ajna, CenterState { defined: false, gates: vec![] });
    centers.insert(Center::Throat, CenterState { defined: true, gates: vec![62, 23] });
    centers.insert(Center::G, CenterState { defined: true, gates: vec![1, 8] });
    centers.insert(Center::Heart, CenterState { defined: false, gates: vec![] });
    centers.insert(Center::Spleen, CenterState { defined: false, gates: vec![] });
    centers.insert(Center::SolarPlexus, CenterState { defined: true, gates: vec![6, 59] });
    centers.insert(Center::Sacral, CenterState { defined: true, gates: vec![34, 20] });
    centers.insert(Center::Root, CenterState { defined: false, gates: vec![] });

    let channels = vec![
        Channel { gate1: 1, gate2: 8, name: "Inspiration".to_string(), circuitry: "Individual".to_string() },
        Channel { gate1: 6, gate2: 59, name: "Mating".to_string(), circuitry: "Tribal".to_string() },
        Channel { gate1: 34, gate2: 20, name: "Charisma".to_string(), circuitry: "Individual".to_string() },
    ];

    c.bench_function("hd_type_determination", |b| {
        b.iter(|| {
            black_box(determine_type(black_box(&centers), black_box(&channels)))
        })
    });
}

/// Benchmark: Authority determination from centers and channels
fn bench_authority_determination(c: &mut Criterion) {
    let mut centers = HashMap::new();
    centers.insert(Center::Head, CenterState { defined: false, gates: vec![] });
    centers.insert(Center::Ajna, CenterState { defined: false, gates: vec![] });
    centers.insert(Center::Throat, CenterState { defined: true, gates: vec![62, 23] });
    centers.insert(Center::G, CenterState { defined: true, gates: vec![1, 8] });
    centers.insert(Center::Heart, CenterState { defined: false, gates: vec![] });
    centers.insert(Center::Spleen, CenterState { defined: true, gates: vec![57, 34] });
    centers.insert(Center::SolarPlexus, CenterState { defined: true, gates: vec![6, 59] });
    centers.insert(Center::Sacral, CenterState { defined: true, gates: vec![34, 20] });
    centers.insert(Center::Root, CenterState { defined: false, gates: vec![] });

    let channels = vec![
        Channel { gate1: 1, gate2: 8, name: "Inspiration".to_string(), circuitry: "Individual".to_string() },
        Channel { gate1: 6, gate2: 59, name: "Mating".to_string(), circuitry: "Tribal".to_string() },
    ];

    c.bench_function("hd_authority_determination", |b| {
        b.iter(|| {
            black_box(determine_authority(black_box(&centers), black_box(&channels)))
        })
    });
}

/// Benchmark: Channel detection from activations
fn bench_channel_detection(c: &mut Criterion) {
    // Create a set of activations with some complete channels
    let activations = vec![
        Activation { planet: Planet::Sun, gate: 1, line: 3, longitude: 13.875 },
        Activation { planet: Planet::Earth, gate: 8, line: 1, longitude: 193.875 },
        Activation { planet: Planet::Moon, gate: 6, line: 2, longitude: 100.0 },
        Activation { planet: Planet::Mercury, gate: 59, line: 4, longitude: 200.0 },
        Activation { planet: Planet::Venus, gate: 34, line: 1, longitude: 50.0 },
        Activation { planet: Planet::Mars, gate: 20, line: 6, longitude: 150.0 },
        Activation { planet: Planet::Jupiter, gate: 17, line: 2, longitude: 250.0 },
        Activation { planet: Planet::Saturn, gate: 18, line: 5, longitude: 310.0 },
        Activation { planet: Planet::NorthNode, gate: 25, line: 3, longitude: 75.0 },
        Activation { planet: Planet::SouthNode, gate: 51, line: 1, longitude: 255.0 },
        Activation { planet: Planet::Uranus, gate: 36, line: 4, longitude: 120.0 },
        Activation { planet: Planet::Neptune, gate: 35, line: 2, longitude: 340.0 },
        Activation { planet: Planet::Pluto, gate: 45, line: 6, longitude: 290.0 },
    ];

    c.bench_function("hd_channel_detection", |b| {
        b.iter(|| {
            black_box(analyze_channels(black_box(&activations)))
        })
    });
}

/// Benchmark: Center definition from activations
fn bench_center_definition(c: &mut Criterion) {
    let activations = vec![
        Activation { planet: Planet::Sun, gate: 1, line: 3, longitude: 13.875 },
        Activation { planet: Planet::Earth, gate: 8, line: 1, longitude: 193.875 },
        Activation { planet: Planet::Moon, gate: 6, line: 2, longitude: 100.0 },
        Activation { planet: Planet::Mercury, gate: 59, line: 4, longitude: 200.0 },
        Activation { planet: Planet::Venus, gate: 34, line: 1, longitude: 50.0 },
        Activation { planet: Planet::Mars, gate: 20, line: 6, longitude: 150.0 },
        Activation { planet: Planet::Jupiter, gate: 17, line: 2, longitude: 250.0 },
        Activation { planet: Planet::Saturn, gate: 18, line: 5, longitude: 310.0 },
    ];

    c.bench_function("hd_center_definition", |b| {
        b.iter(|| {
            black_box(analyze_centers(black_box(&activations)))
        })
    });
}

/// Benchmark: Swiss Ephemeris Moon position lookup
fn bench_ephemeris_moon_lookup(c: &mut Criterion) {
    initialize_ephemeris("");
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(1990, 1, 15, 19, 30, 0).unwrap();

    c.bench_function("hd_ephemeris_moon_lookup", |b| {
        b.iter(|| {
            black_box(calculator.get_planet_position(HDPlanet::Moon, black_box(&birth_time)))
        })
    });
}

/// Benchmark: Swiss Ephemeris Sun position lookup
fn bench_ephemeris_sun_lookup(c: &mut Criterion) {
    initialize_ephemeris("");
    let calculator = EphemerisCalculator::new("");
    let birth_time = Utc.with_ymd_and_hms(1990, 1, 15, 19, 30, 0).unwrap();

    c.bench_function("hd_ephemeris_sun_lookup", |b| {
        b.iter(|| {
            black_box(calculator.get_planet_position(HDPlanet::Sun, black_box(&birth_time)))
        })
    });
}

/// Benchmark: Longitude to gate conversion (pure computation)
fn bench_gate_conversion(c: &mut Criterion) {
    c.bench_function("hd_longitude_to_gate", |b| {
        b.iter(|| {
            for lng in (0..360).map(|i| i as f64 + 0.5) {
                black_box(longitude_to_gate_and_line(black_box(lng)));
            }
        })
    });
}

/// Benchmark: Profile calculation
fn bench_profile_calculation(c: &mut Criterion) {
    let personality = vec![
        Activation { planet: Planet::Sun, gate: 1, line: 6, longitude: 13.875 },
        Activation { planet: Planet::Earth, gate: 2, line: 2, longitude: 193.875 },
    ];
    let design = vec![
        Activation { planet: Planet::Sun, gate: 25, line: 3, longitude: 75.0 },
        Activation { planet: Planet::Earth, gate: 46, line: 1, longitude: 255.0 },
    ];

    c.bench_function("hd_profile_calculation", |b| {
        b.iter(|| {
            black_box(calculate_profile(black_box(&personality), black_box(&design)))
        })
    });
}

/// Benchmark: Full chart generation (generate_hd_chart)
fn bench_chart_generation(c: &mut Criterion) {
    initialize_ephemeris("");
    let birth_utc = Utc.with_ymd_and_hms(1990, 1, 15, 19, 30, 0).unwrap();

    c.bench_function("hd_chart_generation", |b| {
        b.iter(|| {
            black_box(generate_hd_chart(black_box(birth_utc), ""))
        })
    });
}

criterion_group!(
    hd_benches,
    bench_full_hd_chart,
    bench_26_activations,
    bench_personality_activations,
    bench_type_determination,
    bench_authority_determination,
    bench_channel_detection,
    bench_center_definition,
    bench_ephemeris_moon_lookup,
    bench_ephemeris_sun_lookup,
    bench_gate_conversion,
    bench_profile_calculation,
    bench_chart_generation,
);
criterion_main!(hd_benches);
