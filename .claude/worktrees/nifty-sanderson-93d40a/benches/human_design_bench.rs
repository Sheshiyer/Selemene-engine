use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use chrono::{TimeZone, Utc};
use engine_human_design::{
    generate_hd_chart,
    calculate_all_activations,
    calculate_personality_activations,
    calculate_design_activations,
    calculate_design_time,
    analyze_hd_chart,
    initialize_ephemeris,
    longitude_to_gate_and_line,
};

// Initialize ephemeris once for all benchmarks
fn setup_ephemeris() -> String {
    // Use standard ephemeris path - adjust if needed
    let ephemeris_path = std::env::var("EPHEMERIS_PATH")
        .unwrap_or_else(|_| "./data/ephemeris".to_string());
    
    initialize_ephemeris(&ephemeris_path)
        .expect("Failed to initialize ephemeris for benchmarks");
    
    ephemeris_path
}

/// Benchmark full HD chart calculation (end-to-end)
fn benchmark_full_chart(c: &mut Criterion) {
    let ephemeris_path = setup_ephemeris();
    
    // Test with multiple birth dates to avoid cache effects
    let test_dates = vec![
        Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap(),
        Utc.with_ymd_and_hms(1990, 1, 15, 9, 45, 0).unwrap(),
        Utc.with_ymd_and_hms(1978, 11, 3, 22, 15, 0).unwrap(),
        Utc.with_ymd_and_hms(2000, 5, 20, 6, 0, 0).unwrap(),
    ];
    
    let mut group = c.benchmark_group("hd_full_chart");
    
    for (idx, birth_time) in test_dates.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("date_{}", idx)),
            birth_time,
            |b, birth_time| {
                b.iter(|| {
                    let chart = generate_hd_chart(
                        black_box(*birth_time),
                        black_box(&ephemeris_path)
                    ).unwrap();
                    black_box(chart);
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark 26 planetary activations calculation only
fn benchmark_activations_only(c: &mut Criterion) {
    let ephemeris_path = setup_ephemeris();
    let birth_time = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    
    c.bench_function("hd_26_activations", |b| {
        b.iter(|| {
            let (personality, design) = calculate_all_activations(
                black_box(birth_time),
                black_box(&ephemeris_path)
            ).unwrap();
            black_box((personality, design));
        })
    });
}

/// Benchmark personality activations only (13 planets at birth)
fn benchmark_personality_activations(c: &mut Criterion) {
    let ephemeris_path = setup_ephemeris();
    let birth_time = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    
    c.bench_function("hd_personality_13_planets", |b| {
        b.iter(|| {
            let personality = calculate_personality_activations(
                black_box(birth_time),
                black_box(&ephemeris_path)
            ).unwrap();
            black_box(personality);
        })
    });
}

/// Benchmark design time calculation (88-day solar arc)
fn benchmark_design_time(c: &mut Criterion) {
    let ephemeris_path = setup_ephemeris();
    let birth_time = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    
    c.bench_function("hd_design_time_88day", |b| {
        b.iter(|| {
            let design_time = calculate_design_time(
                black_box(birth_time),
                black_box(&ephemeris_path)
            ).unwrap();
            black_box(design_time);
        })
    });
}

/// Benchmark design activations only (13 planets at design time)
fn benchmark_design_activations(c: &mut Criterion) {
    let ephemeris_path = setup_ephemeris();
    let birth_time = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    
    // Pre-calculate design time to isolate activation calculation
    let design_time = calculate_design_time(birth_time, &ephemeris_path).unwrap();
    
    c.bench_function("hd_design_13_planets", |b| {
        b.iter(|| {
            let design = calculate_design_activations(
                black_box(design_time),
                black_box(&ephemeris_path)
            ).unwrap();
            black_box(design);
        })
    });
}

/// Benchmark analysis phase only (Type/Authority/Profile determination)
fn benchmark_analysis_only(c: &mut Criterion) {
    let ephemeris_path = setup_ephemeris();
    let birth_time = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    
    // Pre-calculate activations to isolate analysis
    let (personality, design) = calculate_all_activations(birth_time, &ephemeris_path).unwrap();
    
    c.bench_function("hd_analysis_type_authority_profile", |b| {
        b.iter(|| {
            let analysis = analyze_hd_chart(
                black_box(&personality),
                black_box(&design)
            );
            black_box(analysis);
        })
    });
}

/// Benchmark gate mapping (longitude to gate/line conversion)
fn benchmark_gate_mapping(c: &mut Criterion) {
    let test_longitudes = vec![0.0, 15.5, 45.3, 90.7, 180.0, 270.2, 359.9];
    
    let mut group = c.benchmark_group("hd_gate_mapping");
    
    for lon in test_longitudes {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("lon_{:.1}", lon)),
            &lon,
            |b, lon| {
                b.iter(|| {
                    let (gate, line) = longitude_to_gate_and_line(black_box(*lon));
                    black_box((gate, line));
                })
            },
        );
    }
    
    group.finish();
}

/// Benchmark batch chart generation (simulating API load)
fn benchmark_batch_charts(c: &mut Criterion) {
    let ephemeris_path = setup_ephemeris();
    
    // Simulate 10 concurrent requests
    let batch_dates: Vec<_> = (0..10)
        .map(|i| Utc.with_ymd_and_hms(1980 + i, 6, 15, 14, 30, 0).unwrap())
        .collect();
    
    c.bench_function("hd_batch_10_charts", |b| {
        b.iter(|| {
            let charts: Vec<_> = batch_dates
                .iter()
                .map(|birth_time| {
                    generate_hd_chart(*birth_time, &ephemeris_path).unwrap()
                })
                .collect();
            black_box(charts);
        })
    });
}

/// Comprehensive benchmark group comparing phases
fn benchmark_phases(c: &mut Criterion) {
    let ephemeris_path = setup_ephemeris();
    let birth_time = Utc.with_ymd_and_hms(1985, 6, 15, 14, 30, 0).unwrap();
    
    let mut group = c.benchmark_group("hd_phase_breakdown");
    
    // Phase 1: Design time calculation
    group.bench_function("1_design_time", |b| {
        b.iter(|| {
            calculate_design_time(black_box(birth_time), black_box(&ephemeris_path)).unwrap()
        })
    });
    
    // Phase 2: Personality activations
    group.bench_function("2_personality_planets", |b| {
        b.iter(|| {
            calculate_personality_activations(black_box(birth_time), black_box(&ephemeris_path)).unwrap()
        })
    });
    
    // Phase 3: Design activations
    let design_time = calculate_design_time(birth_time, &ephemeris_path).unwrap();
    group.bench_function("3_design_planets", |b| {
        b.iter(|| {
            calculate_design_activations(black_box(design_time), black_box(&ephemeris_path)).unwrap()
        })
    });
    
    // Phase 4: Analysis
    let (personality, design) = calculate_all_activations(birth_time, &ephemeris_path).unwrap();
    group.bench_function("4_analysis", |b| {
        b.iter(|| {
            analyze_hd_chart(black_box(&personality), black_box(&design))
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_full_chart,
    benchmark_activations_only,
    benchmark_personality_activations,
    benchmark_design_time,
    benchmark_design_activations,
    benchmark_analysis_only,
    benchmark_gate_mapping,
    benchmark_batch_charts,
    benchmark_phases,
);

criterion_main!(benches);
