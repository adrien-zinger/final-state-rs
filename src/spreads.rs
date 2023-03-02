//! Ce fichier contient l'implémentation de multiple fonction de diffusion
//! utilisées pour la création d'une table de translation dans la version
//! de l'algorithme tANS, poussée par Yann Collet et Jarek Duda.
//!
//! Implémentation de final-state-rs, tenter d'implémenter FSE en Rust.
//! Author: Adrien Zinger, avec l'inspiration du travail de Jarek Duda,
//!         Yann Collet, Charles Bloom et bien d'autres.

/// Implémentation original dans fse.c par Yann Collet.
/// todo: l'histogramme devrait être trié, dans mon implémentation
///       actuelle ce n'est pas du tout le cas.
pub fn fse_spread(hist: &[u8], table_log: usize) -> Vec<u8> {
    let m = 1 << table_log;
    let mut ret = vec![0; m];
    let mut pos = 0;
    let step = (1 << (table_log - 1)) + (1 << (table_log - 3)) + 1;
    for (i, &count) in hist.iter().enumerate().filter(|(_, count)| **count > 0) {
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
pub fn bit_reverse_spread(hist: &[u8], table_log: usize) -> Vec<u8> {
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
    let mut hist = [0; 256];
    hist['A' as usize] = 7;
    hist['B' as usize] = 6;
    hist['C' as usize] = 3;
    let res = fse_spread(&hist, 4)
        .iter()
        .map(|c| char::from(*c))
        .collect::<Vec<char>>();
    let expect = vec![
        'A', 'A', 'A', 'B', 'B', 'C', 'A', 'A', 'B', 'B', 'C', 'A', 'A', 'B', 'B', 'C',
    ];
    assert_eq!(expect, res)
}

#[test]
fn bitreverse_spread_test() {
    let mut hist = [0; 256];
    hist['A' as usize] = 7;
    hist['B' as usize] = 6;
    hist['C' as usize] = 3;
    let res = bit_reverse_spread(&hist, 4)
        .iter()
        .map(|c| char::from(*c))
        .collect::<Vec<char>>();
    let expect = vec![
        'A', 'B', 'A', 'B', 'A', 'B', 'A', 'C', 'A', 'B', 'A', 'C', 'A', 'B', 'B', 'C',
    ];
    assert_eq!(expect, res)
}
