use criterion::{black_box, criterion_group, criterion_main, Criterion};
use selemene_engine::models::{PanchangaRequest, PrecisionLevel};
use std::time::Instant;

// Benchmark for Panchanga request creation
fn benchmark_panchanga_request_creation(c: &mut Criterion) {
    c.bench_function("create_panchanga_request", |b| {
        b.iter(|| {
            black_box(PanchangaRequest {
                date: "2025-01-27".to_string(),
                latitude: Some(19.0760),
                longitude: Some(72.8777),
                timezone: Some("Asia/Kolkata".to_string()),
                precision: Some(PrecisionLevel::High),
                include_details: Some(true),
            });
        });
    });
}

// Benchmark for precision level comparisons
fn benchmark_precision_level_comparisons(c: &mut Criterion) {
    c.bench_function("precision_level_comparisons", |b| {
        b.iter(|| {
            let standard = PrecisionLevel::Standard;
            let high = PrecisionLevel::High;
            let extreme = PrecisionLevel::Extreme;
            
            black_box(standard < high);
            black_box(high < extreme);
            black_box(standard as u8);
            black_box(high as u8);
            black_box(extreme as u8);
        });
    });
}

// Benchmark for string operations
fn benchmark_string_operations(c: &mut Criterion) {
    c.bench_function("string_operations", |b| {
        b.iter(|| {
            let date = "2025-01-27";
            let latitude = "19.0760";
            let longitude = "72.8777";
            
            let combined = format!("{}_{}_{}", date, latitude, longitude);
            black_box(combined);
            
            let parsed_date = date.parse::<String>().unwrap();
            black_box(parsed_date);
        });
    });
}

// Benchmark for vector operations
fn benchmark_vector_operations(c: &mut Criterion) {
    c.bench_function("vector_operations", |b| {
        b.iter(|| {
            let mut vec = Vec::new();
            for i in 0..1000 {
                vec.push(i);
            }
            
            let sum: i32 = vec.iter().sum();
            black_box(sum);
            
            vec.sort();
            black_box(vec);
        });
    });
}

// Benchmark for hash map operations
fn benchmark_hashmap_operations(c: &mut Criterion) {
    use std::collections::HashMap;
    
    c.bench_function("hashmap_operations", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..100 {
                map.insert(i.to_string(), i);
            }
            
            let value = map.get("50");
            black_box(value);
            
            map.remove("25");
            black_box(map.len());
        });
    });
}

// Benchmark for async operations simulation
fn benchmark_async_operations(c: &mut Criterion) {
    c.bench_function("async_operations", |b| {
        b.iter(|| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let start = Instant::now();
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                let duration = start.elapsed();
                black_box(duration);
            });
        });
    });
}

// Benchmark for JSON serialization/deserialization
fn benchmark_json_operations(c: &mut Criterion) {
    use serde_json;
    
    c.bench_function("json_serialization", |b| {
        let request = PanchangaRequest {
            date: "2025-01-27".to_string(),
            latitude: Some(19.0760),
            longitude: Some(72.8777),
            timezone: Some("Asia/Kolkata".to_string()),
            precision: Some(PrecisionLevel::High),
            include_details: Some(true),
        };
        
        b.iter(|| {
            let json = serde_json::to_string(&request).unwrap();
            black_box(json);
        });
    });
    
    c.bench_function("json_deserialization", |b| {
        let json = r#"{"date":"2025-01-27","latitude":19.076,"longitude":72.8777,"timezone":"Asia/Kolkata","precision":2,"include_details":true}"#;
        
        b.iter(|| {
            let request: PanchangaRequest = serde_json::from_str(json).unwrap();
            black_box(request);
        });
    });
}

// Benchmark for memory allocation
fn benchmark_memory_allocation(c: &mut Criterion) {
    c.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(1000);
            for i in 0..1000 {
                vec.push(i.to_string());
            }
            black_box(vec);
        });
    });
}

// Benchmark for mathematical operations
fn benchmark_math_operations(c: &mut Criterion) {
    c.bench_function("math_operations", |b| {
        b.iter(|| {
            let mut result = 0.0;
            for i in 0..1000 {
                result += (i as f64).sin() * (i as f64).cos();
            }
            black_box(result);
        });
    });
}

criterion_group!(
    benches,
    benchmark_panchanga_request_creation,
    benchmark_precision_level_comparisons,
    benchmark_string_operations,
    benchmark_vector_operations,
    benchmark_hashmap_operations,
    benchmark_async_operations,
    benchmark_json_operations,
    benchmark_memory_allocation,
    benchmark_math_operations,
);

criterion_main!(benches);
