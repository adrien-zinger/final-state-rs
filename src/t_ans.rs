//! Ce fichier contient une implémentation en Rust de l'algorithme tANS
//! poussé en particulier par Jarek Duda et Yann Collet.
//!
//! Implémentation de final-state-rs, tenter d'implémenter FSE en Rust.
//! Author: Adrien Zinger, avec l'inspiration du travail de Jarek Duda,
//!         Yann Collet, Charles Bloom et bien d'autres.

use tiny_bitstream::{BitDstream, BitEstream, BitReader, BitWriter};

/// Preparation for tANS of the encoding table.
///
/// # Algorithme
/// start[s] = -Ls + somme (Ls', s'<s)
/// next[s] = Ls
///
/// --- Formula to know the number of bits to add to the stream while encoding.
///     Basically log_ceil(Symbol's frequency) or log_floor(Symbol's frequency)
///     depending of the current state.
///
/// for state in L..2L {
///     symbol = spread[state - L]
///     table[start[s] + next[s]++] = state
/// }
pub fn build_encode_table(
    hist: &[usize],
    table_log: usize,
    spread: &[u8],
) -> (Vec<usize>, Vec<usize>, Vec<i32>) {
    let mut delta_nb_bits = vec![0; hist.len()];
    let mut starts = vec![0i32; hist.len()];
    let mut total = 0i32;
    let table_size = 1 << table_log;
    for (s, c) in hist.iter().enumerate() {
        /* On peut considérer qu'un charactère non présent dans l'histograme
        soit à lire. Dans un contexte de streaming par exemple. Dans ce cas,
        il faut ajouter les lignes suivantes. Un test de performance peut aussi
        nous décider à laisser on non cette condition dans tout les cas.
        if *c == 0 {
            delta_nb_bits[s] = ((table_log + 1) << 16) - table_size;
        } else */
        if *c == 1 {
            // Si le symbole n'apparait qu'une fois, il faudra pouvoir lire un
            // nombre de bit suffisant pour avoir un delta qui fasse toute la
            // table. Cette valeure est constante :
            // `(table_log << 16) - table_size`
            delta_nb_bits[s] = (table_log << 16) - table_size;
            starts[s] = total - *c as i32;
            total += 1;
        } else if *c > 0 {
            let hb = 31 - ((*c - 1) as u32).leading_zeros();
            let max_bits_out = table_log - hb as usize;
            delta_nb_bits[s] = (max_bits_out << 16) - (*c << max_bits_out);
            starts[s] = total - *c as i32;
            total += *c as i32;
        }
    }

    let mut table = vec![0; table_size + 2];
    let mut nexts = hist.to_vec();
    for x in table_size..2 * table_size {
        let s = spread[x - table_size] as usize;
        table[(starts[s] + nexts[s] as i32) as usize] = x;
        nexts[s] += 1;
    }
    (table, delta_nb_bits, starts)
}

/// Encode one given symbol reguarding the current state, the encoding table,
/// the table of bits to put in the stream...
///
/// Return the new state after encoding the symbol and modifying the stream.
#[inline] // I want to be sure that will be inlined
pub fn encode_symbol(
    delta_nb_bits: &[usize],
    starts: &[i32],
    table: &[usize],
    state: usize,
    symbol: usize,
    stream: &mut BitEstream,
) -> usize {
    let nb_bits_out = (state + delta_nb_bits[symbol]) >> 16;
    stream.unchecked_write(state, nb_bits_out as u8);
    table[((state >> nb_bits_out) as i32 + starts[symbol]) as usize]
}

#[inline] // I want to be sure that will be inlined
pub fn decode_symbol(
    dstream: &mut BitDstream,
    nb_bits: &[usize],
    new_states: &[usize],
    state: usize,
    spread: &[u8],
) -> (usize, u8) {
    // Panic if we try to look further than the length of the stream
    let bits = dstream
        .read(nb_bits[state] as u8)
        .unwrap_or_else(|_| panic!("Expected to be able to read {} bytes", nb_bits[state]));
    let ret = new_states[state] + bits;
    (ret, spread[state])
}

/// Preparation de la table de décodage tANS.
///
/// # Algorithme
/// L=2^R
/// R=table_log
/// next[s] = histogram <-- nombre de prochaines apparition d'un symbole
/// for state in 0..L {
///     let symbol = spread[state]
///     let x = next[symbol]++
///     nb_bits = R - logceil(x)
///     new_state = (x << nb_bits) /* vraiment shifter l'état, pas une puissance de 2 */ - L;
///     table[state] = (nb_bits, new_state)
/// }
///
/// # Return
/// Cette fonction construit la table de décodage qui est constituée de deux
/// vecteurs de taille 2^table_log.
///
/// 1. Nombre de bits à lire à un état depuis un stream
/// 2. Prochain point de départ pour le prochain état (ce point de départ sera
///    additioné avec la valeur lue dans le stream)
pub fn build_decode_table(
    table_log: usize,
    spread: &[u8],
    histogram: &[usize],
) -> (Vec<usize>, Vec<usize>) {
    let mut symbol_next = histogram.to_vec();
    let table_size = 1 << table_log;
    let mut nb_bits = vec![0; table_size];
    let mut new_state = vec![0; table_size];
    for state in 0..table_size {
        let symbol = spread[state];
        let x = symbol_next[symbol as usize];
        symbol_next[symbol as usize] += 1;
        // Cette opération est équivalente un un ceil(log2())
        let hb = usize::BITS - 1 - x.leading_zeros();
        nb_bits[state] = table_log - hb as usize;
        new_state[state] = (x << nb_bits[state]) - table_size;
    }
    (nb_bits, new_state)
}

/// Encode with the t_ans algorithm. Prerequisites are a histogram (basically a
/// table where histogram[symbole] = number of occurrences in the sources). That
/// histogram has to be normalized previously in order to have
/// `histogram.iter().sum() == 2^table_log`.
///
/// A spread table that is a base of the state machine. A table_log to build the
/// internal state table. And the current state that has to be `>= 2^table_log`.
/// Initially, the state should be equal to `2^table_log`.
///
/// Return the final state after compressing the source and a vector containing
/// the compressed output.
/// ```
/// use std::{fs::File, io::Read};
///
/// use final_state_rs::count::*;
/// use final_state_rs::normalization::*;
/// use final_state_rs::spreads::*;
/// use final_state_rs::t_ans::*;
///
/// const TABLE_LOG: usize = 11;
/// let mut book1 = vec![];
/// File::open("./rsc/calgary_book1")
///     .expect("Cannot find calgary book1 ressource")
///     .read_to_end(&mut book1)
///     .expect("Unexpected fail to read calgary book1 ressource");
/// let mut hist = [0; 256];
///
/// let max_symbol = multi_bucket_count_u8(&book1, &mut hist);
/// let hist = normalization_with_compensation_binary_heap(&hist, TABLE_LOG, max_symbol).unwrap();
/// let spread = &fse_spread_unsorted(&hist, TABLE_LOG);
/// let mut state = 1 << TABLE_LOG;
/// let (book1_encoded, state) = encode_tans(&book1, &hist, spread, TABLE_LOG, &mut state);
/// ```
pub fn encode_tans(
    src: &[u8],
    histogram: &[usize],
    spread: &[u8],
    table_log: usize,
    state: &mut usize,
) -> (Vec<u8>, usize) {
    assert!(
        *state >= (1 << table_log),
        "The state has to be in [1^table_log..2 x 1^table_log - 1]"
    );
    // Récupère le matériel pour encoder une source
    let (table, delta_nb_bits, starts) = build_encode_table(histogram, table_log, spread);
    let mut estream = BitEstream::new();

    src.iter().for_each(|&symbol| {
        *state = encode_symbol(
            &delta_nb_bits,
            &starts,
            &table,
            *state,
            symbol as usize,
            &mut estream,
        )
    });
    (estream.try_into().unwrap(), *state - (1 << table_log))
}

/// Decode any source encoded with `encode_tans` if we know the histogram, the
/// spread table and the table_log used for it. The state should be the latest
/// state that encode_symbol gave, which is also returned by the `encode_tans`
/// function.
pub fn decode_tans(
    src: Vec<u8>,
    histogram: &[usize],
    spread: &[u8],
    table_log: usize,
    mut state: usize,
    dst_buffer: &mut [u8],
) {
    let (nb_bits, new_states) = build_decode_table(table_log, spread, histogram);
    let mut dstream = BitDstream::try_from(src).unwrap();
    dstream.read(1).unwrap(); // Read mark
    dst_buffer.iter_mut().rev().for_each(|byte| {
        let (new_state, symbol) = decode_symbol(&mut dstream, &nb_bits, &new_states, state, spread);
        *byte = symbol;
        state = new_state;
    });
}
