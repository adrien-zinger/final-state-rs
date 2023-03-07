use criterion::{criterion_group, criterion_main, Criterion};
use final_state_rs::count;

fn criterion_benchmark(c: &mut Criterion) {
    let src = (1..40000)
        .map(|_| rand::random::<u8>())
        .collect::<Vec<u8>>();
    c.bench_function("simple count", |b| {
        let mut ret = [0; 256];
        b.iter(|| count::simple_count_u8(&src, &mut ret))
    });
    c.bench_function("multi_bucket count", |b| {
        let mut ret = [0; 256];
        b.iter(|| count::multi_bucket_count_u8(&src, &mut ret))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
