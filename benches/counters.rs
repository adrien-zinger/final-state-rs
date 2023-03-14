use std::{fs::File, io::Read};

use criterion::{criterion_group, criterion_main, Criterion};
use final_state_rs::count;

fn criterion_benchmark(c: &mut Criterion) {
    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");

    c.bench_function("simple count", |b| {
        let mut ret = [0; 256];
        b.iter(|| count::simple_count_u8_inplace(&book1, &mut ret))
    });
    c.bench_function("multi_bucket count", |b| {
        let mut ret = [0; 256];
        b.iter(|| count::multi_bucket_count_u8(&book1, &mut ret))
    });
    #[cfg(feature = "rayon")]
    c.bench_function("rayon count", |b| {
        b.iter(|| {
            count::divide_and_conquer_count(
                &book1,
                std::thread::available_parallelism().unwrap().get(),
            )
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
