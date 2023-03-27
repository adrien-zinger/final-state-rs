//! Lempel Ziv, while_equal and some LZ variations implementations.
//!
//! This file is a part of `final_state_rs`.
//!
//! Documentation: doc/[language]/lempel_ziv.md
//! License: MIT or BSD
//! Author: Adrien Zinger <zinger.ad@gmail.com>
//!
//! ---
//!
//! This file contains multiple variations of the lempel-ziv algorithm. Some of
//! these algorithm may looks like lzss, lz77 or lz78 and any similarity is
//! absolutly non-fortuit because its totally inspired of. If you want to
//! contribute to the code or the documentation, correctly naming things would
//! be very appreciated.
//!
//! There is also some performance experimentation correlated with the
//! benchmarks. Because I talked about contributions, if you want to add
//! something, you're welcome. The LZ algorithms are known to be slow or greedy
//! in memory. Any amelioration, comment or new variation is correct.

use std::collections::HashMap;

/// La fonction suivante encodera une source en suivant une variation de
/// l'algorithme lempel_ziv. Pour le moment, nous chercherons des récurrences de
/// termes dans tout l'interval précédent l'index actuelle. Autrement dit, pour
/// une séquence de symboles situé dans l'intervalle [i, n] je chercherai les
/// séquences similaires dans l'intervalle [0, i - 1] de taille n, et je
/// selectionnerai la séquence commune avec la valeur de n la plus grande.
///
/// Un tel algorithme doit respecter certaines conditions pour être valide en
/// terme de compression de données. Premièrement, nous devons être en mesure de
/// décoder la sortie compréssée et forcement pouvoir retrouver la séquence
/// initiale sans modification. Deuxièmement, la donnée compréssée doit être de
/// taille inférieure ou égale à la source, ce point peut sembler évident mais
/// tout algorithme ne respecte pas cette condition.
///
/// ```
/// use final_state_rs::lempel_ziv::*;
/// use std::{fs::File, io::Read};
///
/// let mut book1 = [0; 4000];
/// File::open("./rsc/calgary_book1")
///     .expect("Cannot find calgary book1 ressource")
///     .read(&mut book1)
///     .expect("Unexpected fail to read calgary book1 ressource");
///
/// let encoded = encode_lz_no_windows_u8(&book1);
/// let decoded = decode_lz_u8(&encoded);
///
/// assert_eq!(book1.to_vec(), decoded);
/// assert!(encoded.len() <= decoded.len());
/// ```
///
/// Une source incompréssible, par exemple l'alphabet, devrait avoir une forme
/// identique une fois compressée. Et puisque nous en somme à prouver que le
/// résultat ne dépassera jamais en taille la source, prenons l'exemple précédent
/// avec l'alphabet latin.
///
/// ```
/// use final_state_rs::lempel_ziv::*;
///
/// let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZA".as_bytes();
/// let encoded = encode_lz_no_windows_u8(&alphabet);
/// let decoded = decode_lz_u8(&encoded);
/// assert_eq!(alphabet, encoded);
/// assert_eq!(decoded, encoded);
/// ```
pub fn encode_lz_no_windows_u8(src: &[u8]) -> Vec<u8> {
    internal_encode_lz_no_windows_u8::<Original>(src)
}

/// Implémentation générique de lz sans fenêtre. Utilisé par
/// `encode_lz_no_windows_u8` et `encode_lz_no_windows_u8_fast` décrit plus
/// loin. Élimine de la duplication de code par pur principe.
//
// Nous reviendrons rapidement sur les raisons de cette généricité, pour le
// moment vous pouvez faire abstraction du template.
fn internal_encode_lz_no_windows_u8<T: WhileEqual>(src: &[u8]) -> Vec<u8> {
    let mut index = 4;
    let mut ret: Vec<u8> = vec![];
    ret.append(&mut src[..4].to_vec());

    while index < src.len() - 4 {
        let mut s = 0;
        let mut repetition = Pair::default();

        // Recherche de la plus longue séquence dans l'interval
        // [s; index - 4]. Puisque nous savons qu'un charactère identique
        // en `index - 4` donnera une taille maximum de 4, nous pouvons
        // dors et déjà éviter un encodage superflue.
        while s < index - 4 {
            if src[s] == src[index] {
                // Si src[s] == src[index], nous pouvons commencer à rechercher
                // la taille de la séquence commune à partir des deux indexes.
                let len = T::while_equal(src, s, index);
                if (5..32768).contains(&len) && repetition.len < len {
                    repetition.len = len;
                    repetition.index = s;
                }
            }
            s += 1;
        }
        if repetition.len == 0 {
            // Je n'ai trouvé aucune répétition,
            // donc j'écris le symbole et j'avance de 1.
            ret.push(src[index]);
            index += 1;
        } else {
            // J'ai trouvé une répétition, j'avance de la
            // taille de celle-ci

            // Construit la paire taille-index sur 32 bits
            const FLAG_MASK: u32 = 1 << 15;
            let bits: u32 = ((repetition.len | FLAG_MASK) << 16) + repetition.index as u32;
            ret.append(&mut bits.to_be_bytes().to_vec());
            index += repetition.len as usize;
        }
    }
    // Ecrit les dernier bits restants dans le cas où index est
    // dans l'interval [len - 4; len[
    if index < src.len() {
        let diff = src.len() - index;
        ret.append(&mut src[src.len() - diff..].to_vec());
    }
    ret
}

/// Pour éviter trop de duplication de code entre une version optimisée et une version
/// originale des algorithmes, je définirai le trait suivant dont je préciserais les implémentations
/// pour des structures dédiées uniquement à cette fonction `while_equal`.
///
/// Des accès publiques sont définis comme suit.
///
/// ```
/// use final_state_rs::lempel_ziv::*;
///
/// let src = "ABCDFGHABCDEFGHI".as_bytes();
/// println!("src: {:?}", src);
/// let len1 = while_equal_fast(src, 0, 7);
/// let len2 = while_equal(src, 0, 7);
/// assert_eq!(len1, len2);
///
/// let src = "ABCDABCDEFGHI".as_bytes();
/// let len1 = while_equal_fast(src, 0, 4);
/// let len2 = while_equal(src, 0, 4);
/// assert_eq!(len1, len2);
/// ```
pub trait WhileEqual {
    /// La fonction `while_equal` prend comme arguments une source et deux indexes.
    /// Elle calculera le nombre de carractères identiques à partir de ces deux
    /// indexes dans la limite suivante min(index - from, src.len - index).
    fn while_equal(src: &[u8], from: usize, index: usize) -> u32;
}

impl WhileEqual for Original {
    /// Naive while_equal implementation.
    fn while_equal(src: &[u8], from: usize, index: usize) -> u32 {
        assert!(from < index);
        assert!(index < src.len());
        assert_eq!(src[from], src[index]);

        let mut s = from + 1;
        let mut i = index + 1;

        while s < index && i < src.len() && src[s] == src[i] {
            s += 1;
            i += 1;
        }
        (s - from) as u32
    }
}

impl WhileEqual for Fast {
    /// Symetrical implementation of Original::while_equal optimized for out-of-order processors.
    ///
    /// Note: performance are slightly better most of the time but lack of stability led us to develop
    /// Faster::while_equal which is always faster.
    fn while_equal(src: &[u8], from: usize, index: usize) -> u32 {
        assert!(from < index);
        assert!(index < src.len());
        assert_eq!(src[from], src[index]);

        let mut s = from + 1;
        let mut i = index + 1;

        // Split in 4 the tests, each block will be done in parrallel by an OoO
        // processor.
        while s + 4 < index && i + 4 < src.len() {
            let mut b1 = false;
            if src[s] == src[i] {
                b1 = true;
            }
            let mut b2 = false;
            if src[s + 1] == src[i + 1] {
                b2 = true;
            }
            let mut b3 = false;
            if src[s + 2] == src[i + 2] {
                b3 = true;
            }
            let mut b4 = false;
            if src[s + 3] == src[i + 3] {
                b4 = true;
            }
            if b1 && b2 && b3 && b4 {
                s += 4;
                i += 4;
            } else {
                break;
            }
        }

        // Fix the last bytes unchecked
        while s < index && i < src.len() && src[s] == src[i] {
            s += 1;
            i += 1;
        }

        (s - from) as u32
    }
}

impl WhileEqual for Faster {
    /// Use an unsafe conversion of *const u8 into *const usize. Which
    /// allow us to test 4 or 8 bytes once. Panic if src.len() > BYTES_LEN
    /// where BYTES_LEN is 4 or 8 depending of the target.
    fn while_equal(src: &[u8], from: usize, index: usize) -> u32 {
        assert!(from < index);
        assert!(index < src.len());
        assert!(src.len() > BYTES_LEN + 1);
        assert_eq!(src[from], src[index]);

        let mut s = from + 1;
        let mut i = index + 1;

        // Nous récupérons le nombre d'octets pour chaque étape.
        const BYTES_LEN: usize = usize::BITS as usize / 8;

        let mut ps = unsafe { src.as_ptr().add(s) as *const usize };
        let mut is = unsafe { src.as_ptr().add(i) as *const usize };

        // s + BYTES_LEN < index && i + BYTES_LEN < src.len(): verification en
        // premier lieu que nous n'empiétons pas sur la partie droite de la
        // source. Puis en second lieu que nos déréferencements ce font bien sur
        // un interval où nous avons notre source.
        while s + BYTES_LEN < index && i + BYTES_LEN < src.len() && unsafe { *ps == *is } {
            unsafe {
                ps = ps.add(BYTES_LEN);
                is = is.add(BYTES_LEN);
            }
            s += BYTES_LEN;
            i += BYTES_LEN;
        }

        // Fix the last bytes unchecked
        while s < index && i < src.len() && src[s] == src[i] {
            s += 1;
            i += 1;
        }

        (s - from) as u32
    }
}

// Avant de passer à la suite, deffinissons des accès à nos fonction et remarquons
// les différences de performance. Il semble que `encode_lz_no_windows_u8_faster`
// est 25% plus rapide sur ma machine.

/// Do the same thing as `encode_lz_no_windows_u8` but use `while_equal_fast`
/// which is optimized for OoO processor.
pub fn encode_lz_no_windows_u8_fast(src: &[u8]) -> Vec<u8> {
    internal_encode_lz_no_windows_u8::<Fast>(src)
}

/// Do the same thing as `encode_lz_no_windows_u8` but use `while_equal_faster`
/// which has a better optimization.
pub fn encode_lz_no_windows_u8_faster(src: &[u8]) -> Vec<u8> {
    internal_encode_lz_no_windows_u8::<Faster>(src)
}

/// Checks that theorically lz is more performant to compress than its
/// approximation.
#[test]
fn compare_lz() {
    use std::{fs::File, io::Read};
    let mut book1 = [0; 10000];
    let _ = File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");
    let encoded1 = encode_lz_no_windows_u8_fast(&book1);
    let encoded2 = encode_lempel_ziv_u8(&book1, 2000);
    assert!(encoded1.len() <= encoded2.len());
    // Dans ce cas précisement, lz à de meilleures performances.
    assert!(encoded1.len() < encoded2.len());
}

/// lempel_ziv variation of lz algorithm with a windows size.
pub fn encode_lempel_ziv_u8(src: &[u8], windows_size: usize) -> Vec<u8> {
    internal_encode_lempel_ziv_u8::<Original>(src, windows_size)
}

/// Internal implementation of the lempel_ziv algorithm.
pub fn internal_encode_lempel_ziv_u8<T: WhileEqual>(src: &[u8], windows_size: usize) -> Vec<u8> {
    assert!(windows_size < src.len());

    // On peut découper le calcule de la sortie en 2 algorithmes. La première
    // partie pour les indexes <= à windows_size, et la deuxième pour les
    // indexes >=. Ce découpage nous permet d'éviter les branchements de
    // vérification si windows_size < index.

    // TODO: use a bitstream instead of a vec
    let mut ret = internal_encode_lz_no_windows_u8::<T>(&src[..=windows_size]);

    let mut index = windows_size + 1;
    while index < src.len() - 4 {
        let mut s = index - windows_size;
        let mut repetition = Pair::default();

        // Recherche de la plus longue séquence.
        while s < index - 4 {
            if src[s] == src[index] {
                let len = T::while_equal(src, s, index);
                if (5..32768).contains(&len) && repetition.len < len {
                    repetition.len = len;
                    repetition.index = s;
                }
            }
            s += 1;
        }
        if repetition.len == 0 {
            // Je n'ai trouvé aucune répétition,
            // donc j'écris le symbole et j'avance de 1.
            ret.push(src[index]);
            index += 1;
        } else {
            // J'ai trouvé une répétition, j'avance de la
            // taille de celle-ci

            // Construit la paire taille-index sur 32 bits
            const FLAG_MASK: u32 = 1 << 15;
            let bits: u32 = ((repetition.len | FLAG_MASK) << 16) + repetition.index as u32;
            ret.append(&mut bits.to_be_bytes().to_vec());
            index += repetition.len as usize;
        }
    }
    // Ecrit les dernier bits restants dans le cas où index est
    // dans l'interval [len - 4; len[
    if index < src.len() {
        let diff = src.len() - index;
        ret.append(&mut src[src.len() - diff..].to_vec());
    }
    ret
}

/// Internal implementation of the lempel-ziv algorithm.
pub fn internal_encode_lz_with_hashmap_u8<T: WhileEqual>(src: &[u8]) -> Vec<u8> {
    use std::collections::hash_map::Entry::*;

    // On peut découper le calcule de la sortie en 2 algorithmes. La première
    // partie pour les indexes <= à windows_size, et la deuxième pour les
    // indexes >=. Ce découpage nous permet d'éviter les branchements de
    // vérification si windows_size < index.

    // TODO: use a bitstream instead of a vec
    // let mut ret = internal_encode_lz_no_windows_u8::<T>(&src[..=windows_size]);

    let mut ret = vec![];
    let mut hmap = HashMap::<u32, Vec<usize>>::default();

    let mut index = 0;
    while index < src.len() - 4 {
        let mut repetition = Pair::default();

        // Recherche de la plus longue séquence.

        // TODO: an error is hidden in that code. When I try with more
        //       than 100k, we have got a problem.
        let key = unsafe { *(src.as_ptr().add(index) as *const u32) };
        match hmap.entry(key) {
            Occupied(mut entry) => {
                let prev = entry.get_mut();
                for s in prev.iter() {
                    let len = T::while_equal(src, *s, index);
                    if (5..32768).contains(&len) && repetition.len < len {
                        repetition.len = len;
                        repetition.index = *s;
                    }
                }
                prev.push(index);
            }
            Vacant(e) => {
                e.insert(vec![index]);
            }
        };
        if repetition.len == 0 {
            // Je n'ai trouvé aucune répétition,
            // donc j'écris le symbole et j'avance de 1.
            ret.push(src[index]);
            index += 1;
        } else {
            // J'ai trouvé une répétition, j'avance de la
            // taille de celle-ci

            // Construit la paire taille-index sur 32 bits
            const FLAG_MASK: u32 = 1 << 15;
            let bits: u32 = ((repetition.len | FLAG_MASK) << 16) + repetition.index as u32;
            ret.append(&mut bits.to_be_bytes().to_vec());
            for i in index + 1..index + repetition.len as usize {
                let key = unsafe { *(src.as_ptr().add(i) as *const u32) };
                match hmap.entry(key) {
                    Occupied(mut entry) => {
                        entry.get_mut().push(i);
                    }
                    Vacant(e) => {
                        e.insert(vec![i]);
                    }
                };
            }
            index += repetition.len as usize;
        }
    }
    // Ecrit les dernier bits restants dans le cas où index est
    // dans l'interval [len - 4; len[
    if index < src.len() {
        let diff = src.len() - index;
        ret.append(&mut src[src.len() - diff..].to_vec());
    }
    ret
}

pub fn encode_lz_with_hashmap_u8(src: &[u8]) -> Vec<u8> {
    internal_encode_lz_with_hashmap_u8::<Faster>(src)
}

/// Decode any output from encode_lempel_ziv* and encode_lz*.
pub fn decode_lz_u8(src: &[u8]) -> Vec<u8> {
    let mut ret: Vec<u8> = vec![];
    let mut it = src.iter();
    const FLAG_BIT: u8 = 1 << 7;
    const FLAG_MASK: u8 = FLAG_BIT - 1;
    while let Some(symbol) = it.next() {
        if *symbol >= FLAG_BIT {
            let hi_bits_len = ((*symbol & FLAG_MASK) as u16) << 8;
            let lo_bits_len = *it.next().unwrap();
            let len = (hi_bits_len + lo_bits_len as u16) as usize;
            let hi_bits_index = (*it.next().unwrap() as u16) << 8;
            let lo_bits_index = *it.next().unwrap() as u16;
            let index = (hi_bits_index + lo_bits_index) as usize;
            ret.append(&mut ret[index..index + len].to_vec());
        } else {
            ret.push(*symbol);
        }
    }
    ret
}

/* *************************************************************************
_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-

    Annexe contenant quelques tests suplémentaires ainsi que des déclarations
    pratique pour la présentation de ce fichier.

_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-
 ************************************************************************  */

/// Public access to Original::while_equal
pub fn while_equal(src: &[u8], from: usize, index: usize) -> u32 {
    Original::while_equal(src, from, index)
}

/// Public access to Fast::while_equal
pub fn while_equal_fast(src: &[u8], from: usize, index: usize) -> u32 {
    Fast::while_equal(src, from, index)
}

/// Public access to Faster::while_equal
pub fn while_equal_faster(src: &[u8], from: usize, index: usize) -> u32 {
    Faster::while_equal(src, from, index)
}

/// lempel_ziv variation of lz algorithm with a windows size. With the optimization
/// for OoO processors.
pub fn encode_lempel_ziv_u8_fast(src: &[u8], windows_size: usize) -> Vec<u8> {
    internal_encode_lempel_ziv_u8::<Fast>(src, windows_size)
}

/// lempel_ziv variation of lz algorithm with a windows size. With the usize optimization.
pub fn encode_lz_u8_faster(src: &[u8], windows_size: usize) -> Vec<u8> {
    internal_encode_lempel_ziv_u8::<Faster>(src, windows_size)
}

/// Representation of a size-index pair, we could have done without it and used
/// a simple tuple. Only adding this structure increases the clarity of the
/// code. Moreover, it does not impact the performance.
///
/// That pair is written in place of a copy of an already printed sequence in
/// the encoded vector output.
#[derive(Default)]
struct Pair {
    /// Index of the latest occurence of a similar sequence in the buffer.
    index: usize,
    /// Size of the sequence
    len: u32,
}

// The empties structures Original, Fast, Faster and X86_64 are used to dispatch
// statically the lempel_ziv and lz algorithm which uses the while_equal functions.
// Since the while_equal function has multiple implementation, you can choose
// which one to use.
//
// i.e.: `internal_encode_lempel_ziv_u8::<Faster>(src, windows_size)`

/// Namespace for the original while_equal algorithm.
pub struct Original;
/// Namespace for the fast (OoO) while_equal algorithm.
pub struct Fast;
/// Namespace for the faster (usize) while_equal algorithm.
pub struct Faster;

#[cfg(all(feature = "portable_simd", feature = "target_x86_64"))]
pub struct X86_64;

#[cfg(all(feature = "portable_simd", feature = "target_x86_64"))]
pub fn while_equal_target_x86_64(src: &[u8], from: usize, index: usize) -> u32 {
    X86_64::while_equal(src, from, index)
}

#[cfg(all(feature = "portable_simd", feature = "target_x86_64"))]
impl WhileEqual for X86_64 {
    fn while_equal(src: &[u8], from: usize, index: usize) -> u32 {
        assert!(from < index);
        assert!(index < src.len());
        assert!(src.len() > I64X2_BYTES_LEN + 1);
        assert_eq!(src[from], src[index]);

        let mut s = from + 1;
        let mut i = index + 1;

        use std::arch::x86_64::_mm_cmpistrc;
        use std::arch::x86_64::_mm_loadu_si128;
        use std::arch::x86_64::_SIDD_CMP_EQUAL_ORDERED;

        const I64X2_BYTES_LEN: usize = 16;
        // s + I64X2_BYTES_LEN < index && i + I64X2_BYTES_LEN < src.len(): verification en
        // premier lieu que nous n'empiétons pas sur la partie droite de la
        // source. Puis en second lieu que nos déréferencements ce font bien sur
        // un interval où nous avons notre source.
        while s + I64X2_BYTES_LEN < index && i + I64X2_BYTES_LEN < src.len() {
            let ps = unsafe { _mm_loadu_si128(src[s..].as_ptr() as *const _) };
            let pi = unsafe { _mm_loadu_si128(src[i..].as_ptr() as *const _) };
            if unsafe { _mm_cmpistrc::<_SIDD_CMP_EQUAL_ORDERED>(ps, pi) } != 0 {
                break;
            }
            s += I64X2_BYTES_LEN;
            i += I64X2_BYTES_LEN;
        }

        // Fix the last bytes unchecked
        while s < index && i < src.len() && src[s] == src[i] {
            s += 1;
            i += 1;
        }

        (s - from) as u32
    }
}

#[test]
fn no_windows_test() {
    let src = "ABCABCABCBADABCABCABCABCABCDBA";
    println!("source: {:?}", src.as_bytes());
    let encoded = encode_lz_no_windows_u8(src.as_bytes());
    println!("encoded {:?}", encoded);
    for e in encoded.iter() {
        println!("{:8b}", *e);
    }
    let decoded = decode_lz_u8(&encoded);
    assert_eq!(src.as_bytes(), decoded);
}

#[test]
fn consistency_with_hashmap_test() {
    let src = "ABCABCABCBADABCABCABCABCABCDBA".as_bytes();
    let encoded1 = encode_lz_no_windows_u8(src);
    let encoded2 = encode_lz_with_hashmap_u8(src);
    assert_eq!(encoded1, encoded2);
}

#[test]
fn no_windows_calgary_book1_compression_test() {
    use std::{fs::File, io::Read};
    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");
    let book1 = &book1[3000..4000];
    let encoded = encode_lz_no_windows_u8(book1);
    let decoded = decode_lz_u8(&encoded);
    assert_eq!(book1, decoded)
}

#[test]
fn lempel_ziv_calgary_book1_compression_test() {
    use std::{fs::File, io::Read};
    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");
    let book1 = &book1[..4000];
    let encoded = encode_lempel_ziv_u8(book1, 1000);

    // Dans ce cas précisément on s'attend déjà voir une modification
    // de la taille.
    assert!(encoded.len() < book1.len());
    let decoded = decode_lz_u8(&encoded);
    assert_eq!(book1, decoded)
}

#[test]
fn while_equal_functions_consistency() {
    use std::fs::File;
    use std::io::Read;
    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");

    let src = &book1[0..4000];
    println!(
        "{:?}; {:?}",
        src[2046..2054].to_vec(),
        src[3991..3999].to_vec()
    );
    let len1 = Fast::while_equal(src, 2046, 3991);
    let len2 = Original::while_equal(src, 2046, 3991);
    assert_eq!(len1, len2);
}

#[cfg(feature = "target_x86_64")]
#[test]
fn while_equal_functions_consistency_target_x86_64() {
    use std::fs::File;
    use std::io::Read;
    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");

    let src = &book1[0..4000];
    println!(
        "{:?}; {:?}",
        src[2046..2054].to_vec(),
        src[3991..3999].to_vec()
    );
    let len1 = Fast::while_equal(src, 2046, 3991);
    let len2 = X86_64::while_equal(src, 2046, 3991);
    assert_eq!(len1, len2);
}

#[test]
fn while_equal_consitency_doctest_enhanced() {
    let src = "ABCDFGHABCDEFGHI".as_bytes();
    let len1 = Fast::while_equal(src, 0, 7);
    let len2 = Original::while_equal(src, 0, 7);
    let len3 = Faster::while_equal(src, 0, 7);
    assert_eq!(len1, 4);
    assert_eq!(len1, len2);
    assert_eq!(len1, len3);

    let src = "ABCDABCDEFGHI".as_bytes();
    let len1 = Fast::while_equal(src, 0, 4);
    let len2 = Original::while_equal(src, 0, 4);
    let len3 = Faster::while_equal(src, 0, 4);
    assert_eq!(len1, len2);
    assert_eq!(len1, len3);

    let src = "AA".as_bytes();
    let len1 = Fast::while_equal(src, 0, 1);
    let len2 = Original::while_equal(src, 0, 1);
    assert_eq!(len1, len2);

    let src = "ABAB".as_bytes();
    let len1 = Fast::while_equal(src, 0, 2);
    let len2 = Original::while_equal(src, 0, 2);
    assert_eq!(len1, len2);

    let src = "ABCABC".as_bytes();
    let len1 = Fast::while_equal(src, 0, 3);
    let len2 = Original::while_equal(src, 0, 3);
    assert_eq!(len1, len2);

    let src = "ABCDABC".as_bytes();
    let len1 = Fast::while_equal(src, 0, 4);
    let len2 = Original::while_equal(src, 0, 4);
    assert_eq!(len1, len2);

    let src = "ABCDABCD".as_bytes();
    let len1 = Fast::while_equal(src, 0, 4);
    let len2 = Original::while_equal(src, 0, 4);
    assert_eq!(len1, len2);
}

/// Check result consistency with multiple dispatch of lz algorithm.
#[test]
fn lempel_ziv_optimizations_functions_consistency() {
    use std::fs::File;
    use std::io::Read;

    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");
    /*
    let src = &book1[0..4000];

    let encoded1 = encode_lz_no_windows_u8_fast(src);
    let encoded2 = encode_lz_no_windows_u8(src);
    let encoded3 = encode_lz_no_windows_u8_faster(src);

    assert_eq!(encoded1.len(), encoded2.len());
    assert_eq!(encoded1.len(), encoded3.len());

    */
    let src = &book1[120..380];

    println!("len {}", src.len());
    println!("{:?} {:?}", &src[216..221], &src[252..257]);
    println!("{}", unsafe { *(src.as_ptr().add(216) as *const u32) });

    println!("encode no windows");
    let encoded1 = encode_lz_no_windows_u8_fast(src);
    let decoded1 = decode_lz_u8(&encoded1);
    assert_eq!(decoded1, src, "error consistency no windows u8 fast");

    println!("encode with hashmap");
    let encoded2 = encode_lz_with_hashmap_u8(src);
    let decoded2 = decode_lz_u8(&encoded2);
    assert_eq!(decoded2, src, "error consistency with hashmap");

    assert!(encoded2.len() < 4000);
    assert_eq!(encoded1, encoded2);
}

#[test]
fn lempel_ziv_with_dict() {
    use std::fs::File;
    use std::io::Read;

    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");

    let src = &book1[40000..100000];
    let encoded = encode_lz_with_hashmap_u8(src);
    assert!(encoded.len() < src.len());
    println!("{} < {}", encoded.len(), src.len());
    assert_eq!(src, decode_lz_u8(&encoded));
}
