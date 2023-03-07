use criterion::{criterion_group, criterion_main, Criterion};
use final_state_rs::spreads::fast_compression_spread_sorted;

fn criterion_benchmark(c: &mut Criterion) {
    let mut sorted_hist = [0; 256];
    sorted_hist['A' as usize] = 5;
    sorted_hist['B' as usize] = 5;
    sorted_hist['C' as usize] = 3;
    sorted_hist['D' as usize] = 3;
    c.bench_function("fast compression spread", |b| {
        b.iter(|| fast_compression_spread_sorted(&sorted_hist, 10))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
