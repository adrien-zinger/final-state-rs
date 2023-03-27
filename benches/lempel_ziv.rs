use criterion::{criterion_group, criterion_main, Criterion};
use final_state_rs::lempel_ziv::*;

fn criterion_benchmark(c: &mut Criterion) {
    use std::fs::File;
    use std::io::Read;

    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");

    let book1_extract = &book1[0..4000];

    let mut inputs_rand: Vec<u8> = (0..2000).map(|_| rand::random::<u8>()).collect();
    inputs_rand.append(&mut inputs_rand.clone());

    assert_eq!(while_equal(&inputs_rand, 0, 2000), 2000);
    assert_eq!(while_equal_fast(&inputs_rand, 0, 2000), 2000);

    c.bench_function("while equal simple", |b| {
        b.iter(|| while_equal(&inputs_rand, 0, 2000))
    });

    c.bench_function("while equal OoO", |b| {
        b.iter(|| while_equal_fast(&inputs_rand, 0, 2000))
    });

    c.bench_function("while equal on usize len", |b| {
        b.iter(|| while_equal_faster(&inputs_rand, 0, 2000))
    });

    #[cfg(all(feature = "portable_simd", feature = "target_x86_64"))]
    c.bench_function("while equal target x86_64 specific", |b| {
        b.iter(|| while_equal_target_x86_64(&inputs_rand, 0, 2000))
    });

    c.bench_function("lz simple", |b| {
        b.iter(|| encode_lz_no_windows_u8(book1_extract))
    });

    c.bench_function("lzw OoO optimizations", |b| {
        b.iter(|| encode_lz_no_windows_u8_fast(book1_extract))
    });

    c.bench_function("lzss on usize len", |b| {
        b.iter(|| encode_lz_u8_faster(book1_extract, 100))
    });

    c.bench_function("lzss with a dict ", |b| {
        b.iter(|| encode_lz_with_hashmap_u8(book1_extract))
    });

    let book1_10k = &book1[0..10000];
    c.bench_function("lzss with a dict 10k", |b| {
        b.iter(|| encode_lz_with_hashmap_u8(book1_10k))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
