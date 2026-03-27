use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neopilot_models_core::Engine;
use std::path::Path;

fn bench_serialization(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut engine = Engine::new();
    rt.block_on(engine.load_from_directory(Path::new("providers"))).unwrap();
    
    let mut group = c.benchmark_group("serialization");
    
    // Benchmark JSON serialization of all providers
    group.bench_function("serialize_all_providers", |b| {
        b.iter(|| {
            black_box(engine.get_providers_json().unwrap())
        })
    });
    
    // Benchmark JSON serialization of a single provider
    group.bench_function("serialize_anthropic_provider", |b| {
        b.iter(|| {
            black_box(engine.get_provider_json("anthropic").unwrap())
        })
    });
    
    // Benchmark JSON serialization of a single model
    group.bench_function("serialize_gpt4_model", |b| {
        b.iter(|| {
            black_box(engine.get_model_json("anthropic", "claude-3-5-sonnet-20241022").unwrap())
        })
    });
    
    group.finish();
}

fn bench_deserialization(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut engine = Engine::new();
    rt.block_on(engine.load_from_directory(Path::new("providers"))).unwrap();
    
    let json_data = engine.get_providers_json().unwrap();
    let json_string = serde_json::to_string(&json_data).unwrap();
    
    let mut group = c.benchmark_group("deserialization");
    
    group.bench_function("deserialize_all_providers", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<serde_json::Value>(black_box(&json_string)))
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_serialization, bench_deserialization);
criterion_main!(benches);
