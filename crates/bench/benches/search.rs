use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neopilot_models_core::Engine;
use std::path::Path;

fn bench_search_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut engine = Engine::new();
    rt.block_on(engine.load_from_directory(Path::new("providers"))).unwrap();
    
    let mut group = c.benchmark_group("search");
    
    // Benchmark text search
    group.bench_function("search_text_gpt", |b| {
        b.iter(|| {
            black_box(engine.search_models(black_box("gpt")))
        })
    });
    
    group.bench_function("search_text_claude", |b| {
        b.iter(|| {
            black_box(engine.search_models(black_box("claude")))
        })
    });
    
    // Benchmark family-based search
    group.bench_function("search_family_gpt", |b| {
        b.iter(|| {
            black_box(engine.get_models_by_family(black_box("gpt")))
        })
    });
    
    group.bench_function("search_family_claude", |b| {
        b.iter(|| {
            black_box(engine.get_models_by_family(black_box("claude")))
        })
    });
    
    // Benchmark capability-based search
    group.bench_function("search_capability_reasoning", |b| {
        b.iter(|| {
            black_box(engine.get_models_by_capability(black_box("reasoning")))
        })
    });
    
    group.bench_function("search_capability_tool_call", |b| {
        b.iter(|| {
            black_box(engine.get_models_by_capability(black_box("tool_call")))
        })
    });
    
    // Benchmark modality-based search
    group.bench_function("search_modality_image", |b| {
        b.iter(|| {
            black_box(engine.get_models_by_modality(black_box("image")))
        })
    });
    
    group.bench_function("search_modality_audio", |b| {
        b.iter(|| {
            black_box(engine.get_models_by_modality(black_box("audio")))
        })
    });
    
    group.finish();
}

fn bench_cache_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut engine = Engine::new();
    rt.block_on(engine.load_from_directory(Path::new("providers"))).unwrap();
    
    let mut group = c.benchmark_group("cache");
    
    // Benchmark cache hit
    group.bench_function("cache_hit_providers", |b| {
        // Warm up cache
        engine.get_providers_json().unwrap();
        
        b.iter(|| {
            black_box(engine.get_providers_json().unwrap())
        })
    });
    
    // Benchmark cache miss
    group.bench_function("cache_miss_clear", |b| {
        b.iter(|| {
            engine.clear_cache();
            black_box(engine.get_providers_json().unwrap())
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_search_operations, bench_cache_operations);
criterion_main!(benches);
