use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::hint::black_box;
use serde_json::json;

#[path = "../src/toon/mod.rs"]
mod toon;

#[path = "../src/converter.rs"]
mod converter;

fn generate_json_data(size: usize) -> String {
    let users: Vec<_> = (0..size)
        .map(|i| {
            json!({
                "id": i,
                "name": format!("User{}", i),
                "email": format!("user{}@example.com", i),
                "age": 20 + (i % 50),
                "active": i % 2 == 0,
            })
        })
        .collect();
    
    json!({
        "users": users,
        "metadata": {
            "total": size,
            "timestamp": "2024-11-14T00:00:00Z",
            "version": "1.0.0"
        }
    })
    .to_string()
}

fn json_to_toon_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_to_toon");
    
    for size in [10, 100, 1000].iter() {
        let json_data = generate_json_data(*size);
        let bytes = json_data.len();
        
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &json_data, |b, data| {
            b.iter(|| {
                converter::json_to_toon(black_box(data))
                    .expect("Conversion should succeed")
            });
        });
    }
    
    group.finish();
}

fn toon_to_json_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("toon_to_json");
    
    for size in [10, 100, 1000].iter() {
        let json_data = generate_json_data(*size);
        let toon_data = converter::json_to_toon(&json_data)
            .expect("Initial conversion should succeed");
        let bytes = toon_data.len();
        
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &toon_data, |b, data| {
            b.iter(|| {
                converter::toon_to_json(black_box(data))
                    .expect("Conversion should succeed")
            });
        });
    }
    
    group.finish();
}

fn roundtrip_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip");
    
    for size in [10, 100, 1000].iter() {
        let json_data = generate_json_data(*size);
        let bytes = json_data.len();
        
        group.throughput(Throughput::Bytes(bytes as u64));
        group.bench_with_input(BenchmarkId::new("size", size), &json_data, |b, data| {
            b.iter(|| {
                let toon = converter::json_to_toon(black_box(data))
                    .expect("JSON to TOON should succeed");
                let json = converter::toon_to_json(black_box(&toon))
                    .expect("TOON to JSON should succeed");
                black_box(json)
            });
        });
    }
    
    group.finish();
}

fn small_payload_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_payload");
    
    let small_json = r#"{"status":"ok","count":42}"#;
    
    group.bench_function("json_to_toon", |b| {
        b.iter(|| {
            converter::json_to_toon(black_box(small_json))
                .expect("Conversion should succeed")
        });
    });
    
    let toon_data = converter::json_to_toon(small_json).unwrap();
    group.bench_function("toon_to_json", |b| {
        b.iter(|| {
            converter::toon_to_json(black_box(&toon_data))
                .expect("Conversion should succeed")
        });
    });
    
    group.finish();
}

fn complex_nested_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_nested");
    
    let complex_json = json!({
        "products": (0..50).map(|i| json!({
            "id": i,
            "name": format!("Product {}", i),
            "price": 10.99 + (i as f64),
            "inStock": i % 3 == 0,
            "tags": ["tag1", "tag2", "tag3"],
            "metadata": {
                "created": "2024-01-01T00:00:00Z",
                "updated": "2024-11-14T00:00:00Z"
            }
        })).collect::<Vec<_>>(),
        "pagination": {
            "page": 1,
            "perPage": 50,
            "total": 1000
        }
    }).to_string();
    
    group.bench_function("json_to_toon", |b| {
        b.iter(|| {
            converter::json_to_toon(black_box(&complex_json))
                .expect("Conversion should succeed")
        });
    });
    
    let toon_data = converter::json_to_toon(&complex_json).unwrap();
    group.bench_function("toon_to_json", |b| {
        b.iter(|| {
            converter::toon_to_json(black_box(&toon_data))
                .expect("Conversion should succeed")
        });
    });
    
    group.finish();
}

fn token_efficiency_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_efficiency");
    
    let json_data = generate_json_data(100);
    let toon_data = converter::json_to_toon(&json_data).unwrap();
    
    let json_tokens = json_data.split_whitespace().count();
    let toon_tokens = toon_data.split_whitespace().count();
    let savings = ((json_tokens - toon_tokens) as f64 / json_tokens as f64) * 100.0;
    
    println!("\n=== Token Efficiency Comparison ===");
    println!("JSON size: {} bytes, ~{} tokens", json_data.len(), json_tokens);
    println!("TOON size: {} bytes, ~{} tokens", toon_data.len(), toon_tokens);
    println!("Token savings: {:.1}%\n", savings);
    
    group.bench_function("measure_savings", |b| {
        b.iter(|| {
            let toon = converter::json_to_toon(black_box(&json_data)).unwrap();
            (json_data.len(), toon.len())
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    json_to_toon_benchmark,
    toon_to_json_benchmark,
    roundtrip_benchmark,
    small_payload_benchmark,
    complex_nested_benchmark,
    token_efficiency_benchmark
);
criterion_main!(benches);

