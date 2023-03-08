use tiny_bitstream::{BitDstream, BitEstream, BitReader};

use crate::normalization::normalization_with_compensation_binary_heap;

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
    println!("hist: {:?}", hist);
    let hist = normalization_with_compensation_binary_heap(&hist, table_log, 256).unwrap();
    let r = crate::r_ans::encode_rans(&hist, table_log, &src);
    println!("starting state {}", r.0);

    println!("\n{:?}\n", r);

    let (state, nb_bits, flac) = r;
    let encoded_size = nb_bits.len() * 8 + flac.len() * 8;
    let decoded = crate::r_ans::decode_rans(state, nb_bits, flac, &hist, table_log, src.len());

    assert_eq!(src, decoded);
    println!("{:?}", decoded);

    println!("encoded size + pop information: {}", encoded_size);
    println!("src size: {}", src_size);
}

#[test]
fn fuzzingly_zero_u8() {
    // Test rANS avec une source random, comme dans `fuzzingly_zero_u8` excepté
    // qu'on force quelques elements à 0.
    let table_log = 13; // should be enough!

    let mut src = vec![];
    let mut hist = vec![0; 256];
    let mut zeroed = vec![0; 256];
    for _ in 0..10 {
        let symbol = rand::random::<u8>();
        zeroed[symbol as usize] = 1;
    }
    for _ in 0..100 {
        let symbol = rand::random::<u8>();
        if zeroed[symbol as usize] == 1 {
            continue;
        }
        src.push(symbol);
        hist[symbol as usize] += 1;
    }
    let src_size = src.len() * 16;
    println!("hist: {:?}", hist);
    let hist = normalization_with_compensation_binary_heap(&hist, table_log, 256).unwrap();

    let r = crate::r_ans::encode_rans(&hist, table_log, &src);
    println!("starting state {}", r.0);

    println!("\n{:?}\n", r);

    let (state, nb_bits, flac) = r;
    let encoded_size = nb_bits.len() * 8 + flac.len() * 8;
    let decoded = crate::r_ans::decode_rans(state, nb_bits, flac, &hist, table_log, src.len());

    assert_eq!(src, decoded);
    println!("{:?}", decoded);

    println!("encoded size + pop information: {}", encoded_size);
    println!("src size: {}", src_size);
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

#[test]
fn test_rans_homemade_1() {
    use crate::{
        normalization::normalization_with_fast_compensation,
        r_ans::{decode_rans, encode_rans},
    };

    let table_log = 8;
    let ((histogram, _), src) = get_calgary_extract_histogram_1();
    let normalized_histogram = normalization_with_fast_compensation(&histogram, table_log).unwrap();

    assert_eq!(normalized_histogram.iter().sum::<usize>(), 1 << table_log);

    let (state, bits, stream) = encode_rans(&normalized_histogram, table_log, &src);
    let res = decode_rans(
        state,
        bits,
        stream,
        &normalized_histogram,
        table_log,
        src.len(),
    );

    assert_eq!(src.to_vec(), res);
}

const fn get_calgary_extract_histogram_1() -> (([usize; 256], usize), [u8; 50]) {
    use crate::count::simple_count_u8;
    const SRC: [u8; 50] = [
        37, 65, 32, 65, 98, 100, 111, 117, 44, 32, 73, 46, 69, 46, 10, 37, 65, 32, 87, 111, 110,
        103, 44, 32, 75, 46, 89, 46, 10, 37, 68, 32, 49, 57, 56, 50, 10, 37, 84, 32, 65, 110, 97,
        108, 121, 115, 105, 115, 32, 111,
    ];
    (simple_count_u8(&SRC), SRC)
}

#[test]
fn normalization_with_compensation_binary_heap_test() {
    use crate::normalization::normalization_with_compensation_binary_heap;
    let table_log = 8;
    let ((histogram, max_symbol), _) = get_calgary_extract_histogram_1();

    let normalized_histogram =
        normalization_with_compensation_binary_heap(&histogram, table_log, max_symbol).unwrap();

    for i in 0..max_symbol {
        if histogram[i] > 0 {
            assert!(normalized_histogram[i] > 0);
        }
    }
}
