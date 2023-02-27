use std::{collections::HashMap, convert::TryInto};
use tiny_bitstream::{BitDstream, BitEstream, BitReader, BitWriter};

use crate::normalization::{build_cumulative_function, derivative_normalization};

pub fn compress(state: usize, table_log: usize, frequency: usize, cumul: usize) -> usize {
    #[cfg(feature = "checks")]
    // todo: add some natural checks behind a compilation feature; in some case that test
    // doesn't have any reasons to be true.
    if frequency == 0 {
        panic!("attemp division by zero because of an unexpected null frequency")
    }

    // usize div by usize naturally give a rounded floor usize in rust
    //(((state as f64 / frequency as f64).floor() as usize) << table_log)
    //    + (state % frequency)
    //    + cumul
    ((state / frequency) << table_log) + (state % frequency) + cumul
}

/// Meme chose que encode_u8 mais avec un tbleau de u16 comme source. Generalement
/// l'histogramme est plus coûteux à réaliser sur cette taille là.
pub fn encode(
    hist: &mut [usize],
    symbol_index: &HashMap<u16, usize>,
    table_log: usize, // R
    src: &[u16],
) -> (usize, Vec<u32>, Vec<u8>) {
    let cs = derivative_normalization(hist, table_log).unwrap();

    let mut state = 0;

    let d = 32 - table_log;
    let msk = 2usize.pow(16) - 1;

    let mut estream = BitEstream::new();

    let mut nb_bits_table = vec![];

    src.iter().for_each(|symbol| {
        let index = *symbol_index.get(symbol).unwrap();
        let fs = *hist.get(index).unwrap();
        if state >= (fs << d) {
            let bits = state & msk;
            let nb_bits = u64::BITS - bits.leading_zeros();
            estream.unchecked_write(bits, nb_bits.try_into().unwrap());
            nb_bits_table.push(nb_bits);
            state >>= 16;
        };

        state = compress(state, table_log, fs, *cs.get(index).unwrap());
    });
    (state, nb_bits_table, estream.try_into().unwrap())
}

/// Compresse une source de u8, on a besoin d'un histogramme ainsi que d'une
/// table des symbole ("a" est à la position i dans l'histogramme)
pub fn encode_u8(
    hist: &mut [usize],
    table_log: usize, // R
    src: &[u8],
) -> (usize, Vec<u32>, Vec<u8>) {
    let cs = derivative_normalization(hist, table_log).unwrap();

    let mut state = 0;

    // Une table superieure a 32 fera crasher ce programne,
    // mais en general, il est deconseille d'utiliser
    // une table superieure a 13. Pour des questions de
    // performances, pas par superstition...
    let d = 32 - table_log;
    let msk = 2usize.pow(16) - 1;

    let mut estream = BitEstream::new();

    let mut nb_bits_table = vec![];

    src.iter().for_each(|symbol| {
        let index = *symbol as usize;
        let fs = hist[index];
        // On fait attention de ne le faire que si
        // l'etat est plus grand que la probabilitee << d.
        //
        // Ca nous permet de tenir un etat entre 2^16 et 2^32 une
        // fois 2^16 depasse. Et de laisser l'etat tranquil si
        // on est encore en dessous de 2^16.
        //
        // Ce shift nous permet surtout de ne pas avoir un etat qui
        // tend vers l'infini, et ne nous empeche pas de trouver le
        // prochain etat de notre state machine.
        //
        // A cause de la normalisation, le max des probabilites
        // devrait tenir sur table_log bits.
        // Comme d = 32 - table_log, max(fs << d) = 2^32.
        if state >= (fs << d) {
            // On recupere les 16 premier bits
            // de l'etat actuelle et ont la stoque dans un
            // stream. On shift l'etat de 16 pour guarder
            // seulement les 16 bit plus grands.
            let bits = state & msk;
            let nb_bits = u64::BITS - bits.leading_zeros();
            estream.unchecked_write(bits, nb_bits.try_into().unwrap());
            nb_bits_table.push(nb_bits);
            state >>= 16;
        };

        state = compress(state, table_log, fs, cs[index]);
    });
    //println!("state {state}");
    (state, nb_bits_table, estream.try_into().unwrap())
}

/// Todo: trouver le symbole par dychotomie. ( et explorer d'autres méthodes plus
/// couteuses en mémoire)
pub fn find_s(state: usize, cs: &[usize]) -> usize {
    for (i, &c) in cs.iter().enumerate() {
        if c == state {
            return i;
        }
        if c > state {
            return i - 1;
        }
    }
    0
}

pub fn decompress(state: usize, frequency: usize, table_log: usize, cumul: usize) -> usize {
    let mask = 2usize.pow(table_log as u32) - 1;
    (frequency * (state >> table_log)) + (state & mask) - cumul
}

/// Décompression de la source u16, pareil que u8
pub fn decode(
    mut state: usize,
    mut bits: Vec<u32>,
    str: Vec<u8>,
    normalized_counter: &[usize],
    symbols: &[u16],
    table_log: usize,
) -> Vec<u16> {
    let mask = 2usize.pow(table_log as u32) - 1;

    let mut dstream: BitDstream = str.try_into().unwrap();
    dstream.read(1).unwrap(); // read mark

    let cs = build_cumulative_function(normalized_counter);
    let mut ret = vec![];
    while state > 0 {
        //println!("reverse state {state}");
        // todo add a security timing to auto kill loop
        let symbol_index = find_s(state & mask, &cs);
        ret.push(*symbols.get(symbol_index).expect("symbol not found"));
        state = decompress(
            state,
            *normalized_counter
                .get(symbol_index)
                .expect("symbol frequency not found"),
            table_log,
            *cs.get(symbol_index).expect("symbol cumul not found"),
        );
        if state < 2usize.pow(16) {
            if let Some(nb_bits) = bits.pop() {
                state = (state << 16) + dstream.read(nb_bits as u8).unwrap() as usize;
            }
        }
    }
    ret.reverse();
    ret
}

pub fn decode_u8(
    mut state: usize,
    mut bits: Vec<u32>,
    str: Vec<u8>,
    normalized_counter: &[usize],
    table_log: usize,
) -> Vec<u8> {
    let mask = 2usize.pow(table_log as u32) - 1;

    let mut dstream: BitDstream = str.try_into().unwrap();
    dstream.read(1).unwrap(); // read mark

    let cs = build_cumulative_function(normalized_counter);
    let mut ret = vec![];
    while state > 0 {
        // todo add a security timing to auto kill loop
        let symbol_index = find_s(state & mask, &cs);
        ret.push(symbol_index.try_into().expect("symbol overflow"));
        state = decompress(
            state,
            *normalized_counter
                .get(symbol_index)
                .expect("symbol frequency not found"),
            table_log,
            *cs.get(symbol_index).expect("symbol cumul not found"),
        );
        if state < 2usize.pow(16) {
            // Si on a un etat < 16, on essaye de lire le stream.
            // Dans le cas ou on avait shifte, le stream contient
            // forcement des bits. Si on ne trouve pas de bits,
            // ca veut dire qu'on arrive a la fin de la decompression
            // et que l'etat a une valeur attendue.
            if let Some(nb_bits) = bits.pop() {
                state = (state << 16) + dstream.read(nb_bits as u8).unwrap() as usize;
            }
        }
    }
    ret.reverse();
    ret
}
