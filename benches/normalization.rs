use criterion::{criterion_group, criterion_main, Criterion};
use final_state_rs::normalization::*;

fn criterion_benchmark(c: &mut Criterion) {
    let mut hist = vec![1; 256];
    for _ in 0..5000 {
        hist[rand::random::<u8>() as usize] += 1;
    }
    c.bench_function("slow normalization", |b| {
        b.iter(|| slow_normalization(&hist, 10))
    });
    c.bench_function("fast normalization", |b| {
        b.iter(|| fast_normalization_1(&hist, 10))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
