use criterion::{criterion_group, criterion_main, Criterion};
use final_state_rs::count::simple_count_u8_inplace;
use final_state_rs::normalization::*;

/// Ce benchmark n'est pas très représentatif si l'on souhaite connaitre
/// la vitesse dans un cas réel des algorithmes présents. Il s'agit ici de
/// faire un peu de lumière sur les différences de performances dans une
/// même situation très courte d'une petite partie de FSE.
fn criterion_benchmark(c: &mut Criterion) {
    let src = vec![
        37, 65, 32, 65, 98, 100, 111, 117, 44, 32, 73, 46, 69, 46, 10, 37, 65, 32, 87, 111, 110,
        103, 44, 32, 75, 46, 89, 46, 10, 37, 68, 32, 49, 57, 56, 50, 10, 37, 84, 32, 65, 110, 97,
        108, 121, 115, 105, 115, 32, 111,
    ];
    let mut histogram = [0; 256];
    let max_symbol = simple_count_u8_inplace(&src, &mut histogram);

    c.bench_function("slow normalization", |b| {
        b.iter(|| slow_normalization(&histogram, 10))
    });
    c.bench_function("fast normalization", |b| {
        b.iter(|| fast_normalization_1(&histogram, 10))
    });

    let hist3 = histogram.to_vec();
    c.bench_function("normalization with compensation (binary_heap)", |b| {
        b.iter(|| normalization_with_compensation_binary_heap(&hist3, 8, max_symbol))
    });

    c.bench_function("fast normalization with compensation", |b| {
        b.iter(|| normalization_with_compensation_binary_heap(&hist3, 8, max_symbol))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
