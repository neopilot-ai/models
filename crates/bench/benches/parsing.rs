use criterion::{black_box, criterion_group, criterion_main, Criterion};
use neopilot_models_core::{Engine, Parser};
use std::path::Path;

fn bench_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    
    // Benchmark parsing the entire providers directory
    group.bench_function("parse_all_providers", |b| {
        b.iter(|| {
            let parser = Parser::new();
            let providers_path = Path::new("providers");
            black_box(parser.parse_directory(black_box(providers_path)))
        })
    });
    
    // Benchmark parsing a single provider
    group.bench_function("parse_anthropic_provider", |b| {
        b.iter(|| {
            let parser = Parser::new();
            let provider_path = Path::new("providers/anthropic");
            black_box(parser.parse_provider(black_box(provider_path)))
        })
    });
    
    group.finish();
}

fn bench_engine_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine_loading");
    
    group.bench_function("load_from_directory", |b| {
        b.iter(|| {
            let mut engine = Engine::new();
            let providers_path = Path::new("providers");
            black_box(tokio::runtime::Runtime::new().unwrap().block_on(
                engine.load_from_directory(black_box(providers_path))
            ))
        })
    });
    
    group.finish();
}

criterion_group!(benches, bench_parsing, bench_engine_loading);
criterion_main!(benches);
