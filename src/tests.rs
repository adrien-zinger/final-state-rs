use std::collections::HashMap;

use tiny_bitstream::{BitDstream, BitEstream, BitReader};

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
    let r = crate::r_ans::encode(&mut hist, &symbol_index, table_log, &src);
    println!("starting state {}", r.0);

    println!("\n{:?}\n", r);

    let (state, nb_bits, flac) = r;
    let decoded = crate::r_ans::decode(state, nb_bits, flac, &hist, &symbols, table_log);

    assert_eq!(src, decoded);
    println!("{:?}", decoded);
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

    let r = crate::r_ans::encode(&mut hist, &symbol_index, table_log, &src);

    println!("\n{:?} {:0b} + {:0b}\n", r, r.0, r.2[0]);
    println!("\n{:?}\n", hist);

    let (state, nb_bits, flac) = r;
    let decoded = crate::r_ans::decode(state, nb_bits, flac, &hist, &symbols, table_log);

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
    let r = crate::r_ans::encode(&mut hist, &symbol_index, table_log, &src);
    println!("starting state {}", r.0);

    println!("\n{:?}\n", r);

    let (state, nb_bits, flac) = r;
    let encoded_size = nb_bits.len() * 8 + flac.len() * 8;
    let decoded = crate::r_ans::decode(state, nb_bits, flac, &hist, &alphabet, table_log);

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
    for _ in 0..1000 {
        let symbol = rand::random::<u8>();
        src.push(symbol);
        hist[symbol as usize] += 1;
    }
    let src_size = src.len() * 16;
    println!("hist: {:?}", hist);
    let r = crate::r_ans::encode_u8(&mut hist, table_log, &src);
    println!("starting state {}", r.0);

    println!("\n{:?}\n", r);

    let (state, nb_bits, flac) = r;
    let encoded_size = nb_bits.len() * 8 + flac.len() * 8;
    let decoded = crate::r_ans::decode_u8(state, nb_bits, flac, &hist, table_log);

    assert_eq!(src, decoded);
    println!("{:?}", decoded);

    println!("encoded size + pop information: {}", encoded_size);
    println!("src size: {}", src_size);
}

#[test]
fn test_derivative_normalization() {
    use crate::normalization::derivative_normalization;

    let mut hist = vec![2, 3, 6, 2];
    let table_log = 4;
    let cs = derivative_normalization(&mut hist, table_log);
    println!("{:?}, {:?}", hist, cs);
    assert_eq!(hist.iter().sum::<usize>(), 1 << table_log)
}

#[test]
fn test_derivative_normalization_slow_vs_fast() {
    use crate::normalization::derivative_normalization;
    use crate::normalization::derivative_normalization_fast;

    let mut hist = vec![2, 3, 6, 2];
    let mut hist2 = vec![2, 3, 6, 2];
    let table_log = 4;
    // Ce test passe mais hélas est fortuit, les arrondis dans la méthode
    // rapide peuvent amener à avoir une somme total < 2^table_log et le
    // problème est palier en ajoutant au dernier élément le reste.
    derivative_normalization(&mut hist, table_log).unwrap();
    derivative_normalization_fast(&mut hist2, table_log).unwrap();
    assert_eq!(hist, hist2)
}

#[test]
fn test_fast_normalization_1() {
    use crate::normalization::fast_normalization_1;

    let hist = vec![2, 3, 6, 2];
    let table_log = 4;
    let normalized = fast_normalization_1(&hist, table_log).expect("expect to succeed");
    println!("{:?}", normalized);
    assert_eq!(normalized.iter().sum::<usize>(), 1 << table_log)
}

#[test]
fn test_fast_normalization_1_inplace() {
    use crate::normalization::{fast_normalization_1, zstd_normalization_1_inplace};

    let mut hist = vec![2, 3, 6, 2];
    let table_log = 4;
    let normalized = fast_normalization_1(&hist, table_log).expect("expect to succeed");
    println!("{:?}", normalized);
    zstd_normalization_1_inplace(&mut hist, table_log).expect("expect to succeed");
    assert_eq!(normalized, hist)
}

#[test]
fn test_slow_normalization() {
    use crate::normalization::slow_normalization;

    let hist = vec![2, 3, 6, 2];
    let table_log = 4;
    let normalized = slow_normalization(&hist, table_log).expect("expect to succeed");
    println!("{:?}", normalized);
    assert_eq!(normalized.iter().sum::<usize>(), 1 << table_log)
}

#[test]
fn test_slow_normalization_vs_fast() {
    use crate::normalization::fast_normalization_1;
    use crate::normalization::slow_normalization;

    let mut hist = vec![1; 256];
    for _ in 0..5000 {
        hist[rand::random::<u8>() as usize] += 1;
    }
    let table_log = 4;
    let normalized = slow_normalization(&hist, table_log).expect("expect to succeed");
    let normalized2 = fast_normalization_1(&hist, table_log).expect("expect to succeed");
    assert_eq!(normalized, normalized2)
}

#[test]
fn test_build_table() {
    use crate::{
        spreads::bit_reverse_spread,
        t_ans::{build_decode_table, build_encode_table, decode_symbol, encode_symbol},
    };

    let mut hist = [0; 256];
    hist['A' as usize] = 3;
    hist['B' as usize] = 3;
    hist['C' as usize] = 2;
    let data = "ABBCBACA";
    let spread = bit_reverse_spread(&hist, 3);

    let (table, delta_nb_bits, starts) = build_encode_table(&hist, 3, &spread);

    let mut state = 8;
    let mut stream = BitEstream::new();

    data.chars().for_each(|symbol| {
        state = encode_symbol(
            &delta_nb_bits,
            &starts,
            &table,
            state,
            symbol as usize,
            &mut stream,
        );
    });

    let encoded_data: Vec<u8> = stream.try_into().unwrap();
    for i in encoded_data.iter() {
        print!("{:08b}", i);
    }
    println!();
    let mut dstream: BitDstream = encoded_data.try_into().unwrap();
    dstream.read(1).unwrap(); // read mark

    state -= 1 << 3;
    let (nb_bits, new_states) = build_decode_table(3, &spread, &hist);

    let mut decoded_data = vec![];
    for _ in 0..8 {
        let (new_state, symbol) =
            decode_symbol(&mut dstream, &nb_bits, &new_states, state, &spread);
        decoded_data.push(symbol);
        state = new_state;
    }
    decoded_data.reverse();
    assert_eq!(
        data.chars().map(|c| c as u8).collect::<Vec<u8>>(),
        decoded_data
    );
}
