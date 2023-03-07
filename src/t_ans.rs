use tiny_bitstream::{BitDstream, BitEstream, BitReader, BitWriter};

use crate::{
    count::simple_count_u8, normalization::fast_normalization_1, spreads::bit_reverse_spread,
};

/// Preparation de la table d'encodage pour tANS.
///
/// # Algorithme
/// start[s] = -Ls + somme (Ls', s'<s)
/// next[s] = Ls
///
/// --- Formule qui permet de connaitre le nombre de bits à ajouter au stream
///     pendant l'encodage.
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
        } else
        */
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

    let l = 1 << table_log;
    let mut table = vec![0; l + 2];
    let mut nexts = hist.to_vec();
    for x in l..2 * l {
        let s = spread[x - l] as usize;
        table[(starts[s] + nexts[s] as i32) as usize] = x;
        nexts[s] += 1;
    }
    (table, delta_nb_bits, starts)
}

#[inline]
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
    println!(
        "encode symbol: {}, actual state: {} ({:b}), nb_bits_out: {}, next state {}(shifted) + {}(start) = {};",
        char::from(symbol as u8),
        state,
        state,
        nb_bits_out,
        (state >> nb_bits_out),
        starts[symbol],
        table[((state >> nb_bits_out) as i32 + starts[symbol]) as usize]
    );
    table[((state >> nb_bits_out) as i32 + starts[symbol]) as usize]
}

pub fn decode_symbol(
    dstream: &mut BitDstream,
    nb_bits: &[usize],
    new_states: &[usize],
    state: usize,
    spread: &[u8],
) -> (usize, u8) {
    let bits = dstream.read(nb_bits[state] as u8).unwrap();
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

pub fn encode_1(src: &[u8], table_log: usize) -> (Vec<u8>, usize, usize) {
    let mut histogram = [0; 256];
    simple_count_u8(src, &mut histogram);
    let histogram = fast_normalization_1(&histogram, table_log).unwrap();
    let spread = bit_reverse_spread(&histogram, table_log);
    // Récupère le matériel pour encoder une source
    let (table, delta_nb_bits, starts) = build_encode_table(&histogram, table_log, &spread);
    let mut estream = BitEstream::new();
    let mut state = 1 << table_log;
    src.iter().for_each(|&symbol| {
        state = encode_symbol(
            &delta_nb_bits,
            &starts,
            &table,
            state,
            symbol as usize,
            &mut estream,
        )
    });
    (
        estream.try_into().unwrap(),
        state - (1 << table_log),
        src.len(),
    )
}

pub fn decode_1(src: Vec<u8>, table_log: usize, mut state: usize, buffer: &mut [u8]) {
    // La première étape est la construction de l'histogramme, il s'agit d'une
    // structure de la taille du nombre de symboles (255 si le symbole est sur 1 octet).
    // où sont répertoriés le nombre d'apparition du symbole dans la source donnée.
    //
    // Dans le cas de tANS, cet histogramme permet de construire un table optimal en
    // fonction du nombre de fois où le symbole apparait. Par exemple, si un symbole
    // apparait cinq fois sur mille, le nombre de bit nécessaire pour l'encoder de la
    // façon la plus petite possible est de `-log(5/1000)` (formule de Shannon),
    // soit `7.643`.
    // En pratique, un demi bit n'existe pas, lors de la construction de la table, on
    // deffinit un seuil à partir duquel on ecrira 8 bit au lieu de 7.
    let mut histogram = [0; 256];
    simple_count_u8(&src, &mut histogram);
    // Normalisation, la somme de l'histogramme doit être égale à 2^table_log.
    let histogram = fast_normalization_1(&histogram, table_log).unwrap();

    // Le spread doit être identique pour la compression et la décompression.
    // Sinon la table constuite plus tard sera différente.
    let spread = bit_reverse_spread(&histogram, table_log);

    let (nb_bits, new_states) = build_decode_table(table_log, &spread, &histogram);
    let mut dstream = BitDstream::try_from(src).unwrap();
    dstream.read(1).unwrap(); // Read mark
    buffer.iter_mut().for_each(|byte| {
        let (new_state, symbol) =
            decode_symbol(&mut dstream, &nb_bits, &new_states, state, &spread);
        *byte = symbol;
        state = new_state;
    });
}
