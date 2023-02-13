use std::collections::HashMap;

use crate::fse16::build_cumulative_symbol_frequency;

#[test]
fn build_ecoding_table() {
    let mut hist = vec![24, 20, 49, 18];
    let mut symbol_index = HashMap::new();

    symbol_index.insert(65534, 2);
    symbol_index.insert(65532, 1);
    symbol_index.insert(55, 0);
    symbol_index.insert(1, 3);

    let symbols = vec![55, 65532, 65534, 1];

    let table_log = 3;

    let src = vec![
        65532, 65532, 55, 65534, 65534, 1, 65534, 65534, 65534, 65534, 65534, 65534, 65534,
    ];
    let r = crate::fse16::encode(&mut hist, &symbol_index, table_log, &src);
    println!("starting state {}", r.0);

    println!("\n{:?}\n", r);

    let (state, nb_bits, flac) = r;
    let decoded = crate::fse16::decode(state, nb_bits, flac, &hist, &symbols, table_log);

    assert_eq!(src, decoded);
    println!("{:?}", decoded);
}
#[ignore = "wip"]
#[test]
fn try_to_break_norm() {
    let alphabet_size = 100;
    let mut alphabet = vec![];
    let mut symbol_index = HashMap::new();
    for i in 0..alphabet_size {
        let symbol = rand::random::<u16>();
        alphabet.push(symbol);
        symbol_index.insert(symbol, i);
    }

    let mut src = vec![];
    let mut hist = vec![1; alphabet_size as usize];
    for _ in 0..100 {
        let symbol_index = rand::random::<usize>() % alphabet_size;
        src.push(*alphabet.get(symbol_index).unwrap());
        hist[symbol_index as usize] += 1;
    }
    let mut cs = build_cumulative_symbol_frequency(&hist);
    let table_log = 7;
    println!("{:?}", hist);
    println!("{:?}", cs);
    crate::fse16::simple_normalization(&mut hist, &mut cs, table_log);
    println!("{:?}", hist);
    println!("{:?}", cs);
    assert_eq!(hist.iter().sum::<usize>(), 1 << table_log)
}

#[test]
fn tmp() {
    let mut hist = vec![1000, 500, 125, 125];
    let mut symbol_index = HashMap::new();

    symbol_index.insert(1, 0);
    symbol_index.insert(2, 1);
    symbol_index.insert(3, 2);
    symbol_index.insert(4, 3);

    let symbols = vec![1, 2, 3, 4];
    //                 A  B  C  D
    let table_log = 3;

    let src = vec![2, 1, 1, 4, 1, 2, 1, 3];

    let r = crate::fse16::encode(&mut hist, &symbol_index, table_log, &src);

    println!("\n{:?} {:0b} + {:0b}\n", r, r.0, r.2[0]);
    println!("\n{:?}\n", hist);

    let (state, nb_bits, flac) = r;
    let decoded = crate::fse16::decode(state, nb_bits, flac, &hist, &symbols, table_log);

    assert_eq!(src, decoded);
    println!("{:?}", decoded);
}

#[test]
fn fuzzingly() {
    let alphabet_size = 100;
    let mut alphabet = vec![];
    let mut symbol_index = HashMap::new();
    let table_log = 12; // should be enough!
    for i in 0..alphabet_size {
        let symbol = rand::random::<u16>();
        alphabet.push(symbol);
        symbol_index.insert(symbol, i);
    }

    let mut src = vec![];
    let mut hist = vec![1; alphabet_size as usize];
    for _ in 0..100 {
        let symbol_index = rand::random::<usize>() % alphabet_size;
        src.push(*alphabet.get(symbol_index).unwrap());
        hist[symbol_index as usize] += 1;
    }
    let src_size = src.len() * 16;
    let r = crate::fse16::encode(&mut hist, &symbol_index, table_log, &src);
    println!("starting state {}", r.0);

    println!("\n{:?}\n", r);

    let (state, nb_bits, flac) = r;
    let encoded_size = nb_bits.len() * 8 + flac.len() * 8;
    let decoded = crate::fse16::decode(state, nb_bits, flac, &hist, &alphabet, table_log);

    assert_eq!(src, decoded);
    println!("{:?}", decoded);

    println!("encoded size + pop information: {}", encoded_size);
    println!("src size: {}", src_size);
}

#[test]
fn fuzzingly_u8() {
    let table_log = 13; // should be enough!

    let mut src = vec![];
    let mut hist = vec![1; 256];
    for _ in 0..100 {
        let symbol = rand::random::<u8>();
        src.push(symbol);
        hist[symbol as usize] += 1;
    }
    let src_size = src.len() * 16;
    let r = crate::fse16::encode_u8(&mut hist, table_log, &src);
    println!("starting state {}", r.0);

    println!("\n{:?}\n", r);

    let (state, nb_bits, flac) = r;
    let encoded_size = nb_bits.len() * 8 + flac.len() * 8;
    let decoded = crate::fse16::decode_u8(state, nb_bits, flac, &hist, table_log);

    assert_eq!(src, decoded);
    println!("{:?}", decoded);

    println!("encoded size + pop information: {}", encoded_size);
    println!("src size: {}", src_size);
}

#[test]
fn simple_normalization() {
    let mut hist = vec![2, 3, 6, 2];
    let mut cs = build_cumulative_symbol_frequency(&hist);
    let table_log = 4; // should pass with 3 after
    println!("{:?}", hist);
    println!("{:?}", cs);
    crate::fse16::simple_normalization(&mut hist, &mut cs, table_log);
    println!("{:?}", hist);
    println!("{:?}", cs);
    assert_eq!(hist.iter().sum::<usize>(), 1 << table_log)
}
