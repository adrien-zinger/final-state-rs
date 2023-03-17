use criterion::{criterion_group, criterion_main, Criterion};
use final_state_rs::lzss::*;

fn criterion_benchmark(c: &mut Criterion) {
    use std::fs::File;
    use std::io::Read;

    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");

    let book1 = &book1[0..4000];

    let mut inputs_rand: Vec<u8> = (0..2000).map(|_| rand::random::<u8>()).collect();
    inputs_rand.append(&mut inputs_rand.clone());
    assert_eq!(while_equal(&inputs_rand, 0, 2000), 2000);
    assert_eq!(while_equal_fast(&inputs_rand, 0, 2000), 2000);

    c.bench_function("compare before optimizations", |b| {
        b.iter(|| while_equal(&inputs_rand, 0, 2000))
    });

    c.bench_function("compare after optimizations", |b| {
        b.iter(|| while_equal_fast(&inputs_rand, 0, 2000))
    });

    c.bench_function("lzss encoding before optimizations", |b| {
        b.iter(|| encode_lzw_no_windows_u8(book1))
    });

    c.bench_function("lzss encoding after optimizations", |b| {
        b.iter(|| encode_lzw_no_windows_u8_fast(book1))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
