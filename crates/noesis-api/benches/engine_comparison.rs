//! Engine Comparison Benchmarks
//!
//! Side-by-side comparison of all 3 Wave 1 Phase 2 engines.
//! Uses Mode 2 (pre-computed inputs) for fair comparison.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine_human_design::HumanDesignEngine;
use engine_gene_keys::GeneKeysEngine;
use engine_vimshottari::VimshottariEngine;
use noesis_core::{ConsciousnessEngine, EngineInput, BirthData, Precision};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

fn create_hd_input() -> EngineInput {
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

fn create_gk_input() -> EngineInput {
    let mut options = HashMap::new();
    options.insert("hd_gates".to_string(), json!({
        "personality_sun": 17,
        "personality_earth": 18,
        "design_sun": 45,
        "design_earth": 26
    }));

    EngineInput {
        birth_data: None,
        current_time: Utc::now(),
        location: None,
        precision: Precision::Standard,
        options,
    }
}

fn create_vim_input() -> EngineInput {
    let mut options = HashMap::new();
    options.insert("moon_longitude".to_string(), json!(125.0));
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

fn compare_engines(c: &mut Criterion) {
    let hd_engine = HumanDesignEngine::new();
    let gk_engine = GeneKeysEngine::new();
    let vim_engine = VimshottariEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("engine_comparison");

    group.bench_function("human_design", |b| {
        b.iter(|| {
            let input = create_hd_input();
            let result = rt.block_on(hd_engine.calculate(black_box(input)));
            black_box(result)
        })
    });

    group.bench_function("gene_keys", |b| {
        b.iter(|| {
            let input = create_gk_input();
            let result = rt.block_on(gk_engine.calculate(black_box(input)));
            black_box(result)
        })
    });

    group.bench_function("vimshottari", |b| {
        b.iter(|| {
            let input = create_vim_input();
            let result = rt.block_on(vim_engine.calculate(black_box(input)));
            black_box(result)
        })
    });

    group.finish();
}

fn compare_validation(c: &mut Criterion) {
    let hd_engine = HumanDesignEngine::new();
    let gk_engine = GeneKeysEngine::new();
    let vim_engine = VimshottariEngine::new();
    let rt = tokio::runtime::Runtime::new().unwrap();

    let hd_output = rt.block_on(hd_engine.calculate(create_hd_input())).unwrap();
    let gk_output = rt.block_on(gk_engine.calculate(create_gk_input())).unwrap();
    let vim_output = rt.block_on(vim_engine.calculate(create_vim_input())).unwrap();

    let mut group = c.benchmark_group("validation_comparison");

    group.bench_function("human_design_validate", |b| {
        b.iter(|| {
            let result = rt.block_on(hd_engine.validate(black_box(&hd_output)));
            black_box(result)
        })
    });

    group.bench_function("gene_keys_validate", |b| {
        b.iter(|| {
            let result = rt.block_on(gk_engine.validate(black_box(&gk_output)));
            black_box(result)
        })
    });

    group.bench_function("vimshottari_validate", |b| {
        b.iter(|| {
            let result = rt.block_on(vim_engine.validate(black_box(&vim_output)));
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(
    comparison_benches,
    compare_engines,
    compare_validation,
);
criterion_main!(comparison_benches);
