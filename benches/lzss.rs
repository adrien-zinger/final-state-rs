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

    let book1 = &book1[0..8000];

    c.bench_function("lzss encoding after optimizations", |b| {
        b.iter(|| encode_lzss_no_windows_u8(book1))
    });

    c.bench_function("lzss encoding before optimizations", |b| {
        b.iter(|| encode_lzw_no_windows_u8_simple(book1))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
