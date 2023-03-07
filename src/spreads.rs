//! Ce fichier contient l'implémentation de multiples fonctions de diffusion
//! utilisées pour la création d'une table de translation dans la version
//! de l'algorithme tANS, poussée par Yann Collet et Jarek Duda.
//!
//! Implémentation de final-state-rs, tenter d'implémenter FSE en Rust.
//! Author: Adrien Zinger, avec l'inspiration du travail de Jarek Duda,
//!         Yann Collet, Charles Bloom et bien d'autres.

/// Implémentation original dans fse.c par Yann Collet. Décrite par Charles
/// Bloom. Cette méthode à été mise à jour plus tard.
///
/// Cette implémentation est valable uniquement si l'histogramme est trié,
/// dans le cas contraire, il faut appeler la fonciton fse_spread_unsorted;
///
/// ```
///
/// fn is_sorted(hist: &[usize]) -> bool {
///     false // todo, I should provide something while the std is unstable
/// }
///
/// let mut hist = [0; 256];
/// hist['A' as usize] = 6;
/// hist['B' as usize] = 7;
/// hist['C' as usize] = 3;
/// let _ = if is_sorted(&hist) {
///     final_state_rs::spreads::fse_spread_sorted(&hist, 4)
/// } else {
///     final_state_rs::spreads::fse_spread_unsorted(&hist, 4)
/// };
/// ```
pub fn fse_spread_sorted(sorted_hist: &[usize], table_log: usize) -> Vec<u8> {
    let m = 1 << table_log;
    let mut ret = vec![0; m];
    let mut pos = 0;
    let step = (1 << (table_log - 1)) + (1 << (table_log - 3)) + 1;
    for (i, &count) in sorted_hist
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0)
    {
        for _ in 0..count {
            ret[pos] = i as u8;
            pos = (pos + step) % m;
        }
    }
    ret
}

/// Identique à fse_spread_sorted en tout point.
/// nextState = (currentState + (5/8) range + 3) % range
pub fn fast_compression_spread_sorted(sorted_hist: &[usize], table_log: usize) -> Vec<u8> {
    let range = 1 << table_log;
    let mut ret = vec![0; range];
    let mut pos = 0;
    let step = ((5 * range) >> 3) + 3;
    for (i, &count) in sorted_hist
        .iter()
        .enumerate()
        .filter(|(_, count)| **count > 0)
    {
        for _ in 0..count {
            ret[pos] = i as u8;
            // Il n'y a pas de différence de performance notable
            // entre un % et un masque en Rust
            pos = (pos + step) % range;
        }
    }
    ret
}

/// Même chose que fse_spread_sorted sauf qu'on trie l'histogramme en plus.
pub fn fse_spread_unsorted(hist: &[usize], table_log: usize) -> Vec<u8> {
    let m = 1 << table_log;
    let mut ret = vec![0; m];
    let mut pos = 0;
    let step = (1 << (table_log - 1)) + (1 << (table_log - 3)) + 1;
    let mut sorted_hist = hist
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, count)| *count > 0)
        .collect::<Vec<(usize, usize)>>();

    sorted_hist.sort_by(|(_, a), (_, b)| b.cmp(a));

    for (i, count) in sorted_hist {
        for _ in 0..count {
            ret[pos] = i as u8;
            pos = (pos + step) % m;
        }
    }
    ret
}

/// Proposition lu dans le blog de Charles Bloom à propos de tANS.
/// todo: l'histogramme devrait être trié, dans mon implémentation
///       actuelle ce n'est pas du tout le cas.
pub fn bit_reverse_spread(hist: &[usize], table_log: usize) -> Vec<u8> {
    let mut s = 0u32;
    let mut ret = vec![0; 1 << table_log];
    let t = u32::BITS - table_log as u32;
    for (i, &count) in hist.iter().enumerate().filter(|(_, count)| **count > 0) {
        for _ in 0..count {
            ret[(s.reverse_bits() >> t) as usize] = i as u8;
            s += 1;
        }
    }
    ret
}

// ****************************************************************************
// ****************************************************************************
// ****************************************************************************
// * Basic tests section

#[test]
fn fse_spread_test() {
    let mut sorted_hist = [0; 256];
    sorted_hist['A' as usize] = 7;
    sorted_hist['B' as usize] = 6;
    sorted_hist['C' as usize] = 3;
    assert_eq!(
        vec!['A', 'A', 'A', 'B', 'B', 'C', 'A', 'A', 'B', 'B', 'C', 'A', 'A', 'B', 'B', 'C'],
        fse_spread_sorted(&sorted_hist, 4)
            .iter()
            .map(|c| char::from(*c))
            .collect::<Vec<char>>()
    )
}

#[test]
fn fast_compression_spread_test() {
    let mut sorted_hist = [0; 256];
    sorted_hist['A' as usize] = 5;
    sorted_hist['B' as usize] = 5;
    sorted_hist['C' as usize] = 3;
    sorted_hist['D' as usize] = 3;
    assert_eq!(
        vec!['A', 'B', 'C', 'D', 'A', 'B', 'D', 'A', 'B', 'D', 'A', 'B', 'C', 'A', 'B', 'C'],
        fast_compression_spread_sorted(&sorted_hist, 4)
            .iter()
            .map(|c| char::from(*c))
            .collect::<Vec<char>>()
    )
}

#[test]
fn fse_spread_unsorted_test() {
    let mut sorted_hist = [0; 256];
    sorted_hist['A' as usize] = 7;
    sorted_hist['B' as usize] = 6;
    sorted_hist['C' as usize] = 3;
    assert_eq!(
        vec!['A', 'A', 'A', 'B', 'B', 'C', 'A', 'A', 'B', 'B', 'C', 'A', 'A', 'B', 'B', 'C'],
        fse_spread_unsorted(&sorted_hist, 4)
            .iter()
            .map(|c| char::from(*c))
            .collect::<Vec<char>>()
    );
    let mut sorted_hist = [0; 256];
    sorted_hist['A' as usize] = 6;
    sorted_hist['B' as usize] = 7;
    sorted_hist['C' as usize] = 3;
    assert_eq!(
        vec!['B', 'B', 'B', 'A', 'A', 'C', 'B', 'B', 'A', 'A', 'C', 'B', 'B', 'A', 'A', 'C'],
        fse_spread_unsorted(&sorted_hist, 4)
            .iter()
            .map(|c| char::from(*c))
            .collect::<Vec<char>>()
    );
}

#[test]
fn bitreverse_spread_test() {
    let mut hist = [0; 256];
    hist['A' as usize] = 7;
    hist['B' as usize] = 6;
    hist['C' as usize] = 3;
    assert_eq!(
        vec!['A', 'B', 'A', 'B', 'A', 'B', 'A', 'C', 'A', 'B', 'A', 'C', 'A', 'B', 'B', 'C'],
        bit_reverse_spread(&hist, 4)
            .iter()
            .map(|c| char::from(*c))
            .collect::<Vec<char>>()
    )
}
