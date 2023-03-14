//! Ce fichier contient multiple implémentation de normalisation. Il est
//! utilisé par la bibliothèque en interne, bien qu'acessible en soit par
//! un utilisateur externe.
//!
//! Implémentation de final-state-rs, tenter d'implémenter FSE en Rust.
//! Author: Adrien Zinger, avec l'inspiration du travail de Jarek Duda,
//!         Yann Collet, Charles Bloom et bien d'autres.

use std::collections::BinaryHeap;

#[derive(Debug)]
pub enum NormError {
    RunLengthEncoding(&'static str),
    MultiplicationOverflow,
    NormalizationError,
}

/// Normalisation de la bibliothèque FSE écrite par Yann Collet.
///
/// Notes : Il manque rtbTable et quelques optimisations. Mon objectif
/// principale étant d'écrire ce que je comprend et uniquement ce que je
/// comprend. Une PR avec une amélioration serait la bienvenue avec une
/// excellente description des tenants et des aboutissants ! Sinon je continue
/// à étudier donc les améliorations viendront toute seule.
pub fn fast_normalization_1(
    hist: &[usize],
    table_log: usize,
) -> Result<Vec<usize>, Box<NormError>> {
    let mut norm = vec![0usize; hist.len()];
    let len = hist.len();

    const HIGH_NUM: usize = (usize::BITS - 2) as usize;

    // L'échelle nous permet de travailler sans utiliser des nombres réels,
    // tout en conservant une certaine précision. Les types tels que float,
    // double, etc. sont souvent difficiles à optimiser pour un programme.
    // On cherche un nombre suffisement grand, mais pas trop pour éviter les
    // difficulté de multiplications.
    let scale: usize = HIGH_NUM - table_log;
    let step: usize = (1usize << HIGH_NUM) / hist.iter().sum::<usize>();
    let mut max = 0;
    let mut max_norm = &mut 0;
    let mut still_to_distribute: isize = 1 << table_log;
    for (s, n) in hist.iter().copied().zip(norm.iter_mut()) {
        if s == len {
            // Lorsque la probabilité de trouver un symbole est égale au nombre
            // total de symboles, la méthode de compression la plus simple
            // consiste à compresser en indiquant une plage de ce symbole.
            //
            // C: [Header, Symbol, Len] = [ "rle", "s", 32 ]
            //
            // Il est probable que pour certaines autres caractéristiques, une
            // compression par plage soit préférable. Cependant, cette question
            // devrait être analysée en dehors de la bibliothèque.
            return Err(Box::new(NormError::RunLengthEncoding(
                "An rle compression should be more accurate",
            )));
        } else if s > 0 {
            // La mise à l'échelle a pour biais le fait qu'une grande
            // statistique d'apparition peut potentiellement dépasser
            // la limite d'un nombre sur 32 ou 64 bits (selon l'architecture).
            // D'où le test de multiplication.
            let proba = s
                .checked_mul(step)
                .ok_or(NormError::MultiplicationOverflow)?
                >> scale;
            *n = proba;
            if proba > max {
                max_norm = n;
                max = proba;
            }
            still_to_distribute -= proba as isize;
        }
    }
    if -still_to_distribute >= (max >> 1) as isize {
        return Err(Box::new(NormError::NormalizationError));
    }
    *max_norm += still_to_distribute as usize;
    Ok(norm)
}

/// Fonction de normalisation assez rapide
pub fn normalization_with_fast_compensation(
    hist: &[usize],
    table_log: usize,
) -> Result<Vec<usize>, Box<NormError>> {
    let mut norm = vec![0usize; hist.len()];
    let len = hist.len();

    const HIGH_NUM: usize = (usize::BITS - 2) as usize;

    let scale: usize = HIGH_NUM - table_log;
    let step: usize = (1usize << HIGH_NUM) / hist.iter().sum::<usize>();
    let mut max = 0;
    let mut max_norm = &mut 0;
    let mut total: usize = 0;
    for (s, n) in hist.iter().copied().zip(norm.iter_mut()) {
        if s == len {
            return Err(Box::new(NormError::RunLengthEncoding(
                "An rle compression should be more accurate",
            )));
        } else if s > 0 {
            let proba = std::cmp::max(
                1,
                s.checked_mul(step)
                    .ok_or(NormError::MultiplicationOverflow)?
                    >> scale,
            );
            *n = proba;
            if proba > max {
                max_norm = n;
                max = proba;
            }
            total += proba;
        }
    }
    let table_size = 1 << table_log;
    if total < table_size {
        *max_norm += table_size - total;
        assert_eq!(norm.iter().sum::<usize>(), table_size);
        return Ok(norm);
    }
    while total > table_size {
        for n in norm.iter_mut().rev() {
            if total == table_size {
                break;
            }
            if *n > 1 {
                *n -= 1;
                total -= 1;
            }
        }
    }
    assert_eq!(total, table_size);
    assert_eq!(norm.iter().sum::<usize>(), table_size);

    #[cfg(test)]
    for (real_counter, normalized) in hist.iter().zip(norm.iter()) {
        if *real_counter > 0 {
            assert!(
                *normalized != 0,
                "if Fs > 0 then Normalize should be at least 1"
            );
        }
    }

    Ok(norm)
}

pub fn normalization_with_compensation_binary_heap(
    histogram: &[usize],
    table_log: usize,
    max_symbol: usize,
) -> Result<Vec<usize>, Box<NormError>> {
    use std::cmp::max;
    use NormError::MultiplicationOverflow as Overflow;

    let mut normalized = vec![0usize; max_symbol + 1];
    let len = histogram.len();

    const HIGH_NUM: usize = (usize::BITS - 2) as usize;

    let scale: usize = HIGH_NUM - table_log;
    let step: usize = (1usize << HIGH_NUM) / histogram.iter().sum::<usize>();
    let mut total: usize = 0;

    for (index, &count) in histogram.iter().enumerate().take(max_symbol + 1) {
        if count == len {
            return Err(Box::new(NormError::RunLengthEncoding(
                "An rle compression should be more accurate",
            )));
        } else if count > 0 {
            let proba = max(count.checked_mul(step).ok_or(Overflow)? >> scale, 1);
            normalized[index] = proba;
            total += proba;
        }
    }

    let table_size = 1 << table_log;
    if total == table_size {
        assert_eq!(normalized.iter().sum::<usize>(), table_size);
        return Ok(normalized);
    }

    #[derive(PartialEq)]
    struct SortedProba {
        index: usize,
        change: f32,
    }

    impl Ord for SortedProba {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            match self.change > other.change {
                true => std::cmp::Ordering::Greater,
                false => std::cmp::Ordering::Less,
            }
        }
    }

    impl PartialOrd for SortedProba {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            match self.index.partial_cmp(&other.index) {
                Some(core::cmp::Ordering::Equal) => {}
                ord => return ord,
            }
            self.change.partial_cmp(&other.change)
        }
    }

    impl Eq for SortedProba {}

    // Creation of a binary heap that will sort the probabilities.
    let mut sorted_probas = BinaryHeap::with_capacity(max_symbol);
    for index in
        (0..max_symbol).filter(|&i| histogram[i] != 0 && (normalized[i] > 1 || table_size > total))
    {
        // (double) to[i] / (to[i] + correction_sign) ) * from[i];
        let normalized_plus = if table_size > total {
            normalized[index] + 1
        } else {
            normalized[index] - 1
        };
        let change =
            ((normalized[index] as f32) / normalized_plus as f32).log2() * histogram[index] as f32;
        sorted_probas.push(SortedProba { change, index });
    }

    while total != table_size {
        let mut proba = sorted_probas.pop().unwrap();
        if table_size > total {
            normalized[proba.index] += 1;
            total += 1;
        } else {
            normalized[proba.index] -= 1;
            total -= 1;
        }
        if normalized[proba.index] > 1 || table_size > total {
            let normalized_plus = if table_size > total {
                normalized[proba.index] + 1
            } else {
                normalized[proba.index] - 1
            };
            proba.change = ((normalized[proba.index] as f32) / normalized_plus as f32).log2()
                * histogram[proba.index] as f32;
            sorted_probas.push(proba);
        }
    }

    assert_eq!(total, table_size);
    assert_eq!(normalized.iter().sum::<usize>(), table_size);
    Ok(normalized)
}

/// Même fonction que `fast_normalisation_1` à l'exception qu'on n'augmente pas
/// artificiellement les variables avec une grande valeur. Le fait de
/// travailler avec des nombres rationnels ralentit énormément le calcul.
/// (utiliser la commande `cargo test` pour voir les différences)
pub fn slow_normalization(hist: &[usize], table_log: usize) -> Result<Vec<usize>, Box<NormError>> {
    let mut norm = vec![0usize; hist.len()];
    let step = (1usize << table_log) as isize / hist.iter().sum::<usize>() as isize;
    let mut max = 0;
    let mut max_norm = &mut 0;
    let mut still_to_distribute: isize = 1 << table_log;
    for (s, n) in hist.iter().copied().zip(norm.iter_mut()) {
        if s > 0 {
            let proba = s as isize * step;
            *n = proba as usize;
            if proba > max {
                max_norm = n;
                max = proba;
            }
            still_to_distribute -= proba as isize;
        }
    }
    if -still_to_distribute >= (max >> 1) as isize {
        return Err(Box::new(NormError::NormalizationError));
    }
    *max_norm += still_to_distribute as usize;
    Ok(norm)
}

pub fn zstd_normalization_1_inplace(
    hist: &mut [usize],
    table_log: usize,
    max_symbol: usize,
) -> Result<(), Box<NormError>> {
    let len = hist.len();
    const HIGH_NUM: usize = (usize::BITS - 2) as usize;

    let scale: usize = HIGH_NUM - table_log;
    let total = hist.iter().sum::<usize>();
    let step: usize = (1usize << HIGH_NUM) / total;
    const RTB_TABLE: [usize; 8] = [0, 473195, 504333, 520860, 550000, 700000, 750000, 830000];
    let v_step = 1 << (scale - 20);
    let mut max = 0;
    let mut max_norm = &mut 0;
    let mut still_to_distribute: isize = 1 << table_log;
    let low_threshold = total >> table_log;
    for s in hist.iter_mut().take(max_symbol) {
        if *s <= low_threshold {
            *s = 1;
            still_to_distribute -= 1;
        } else if *s == len {
            return Err(Box::new(NormError::RunLengthEncoding(
                "An rle compression should be more accurate",
            )));
        } else if *s > 0 {
            let mut proba = std::cmp::max(1, ((*s) * step) >> scale);
            if proba < 8 && (*s) * step - (proba << scale) > v_step * RTB_TABLE[proba] {
                proba += 1;
            }
            *s = proba;
            if proba > max {
                max_norm = s;
                max = proba;
            }
            still_to_distribute -= proba as isize;
        }
    }
    if -still_to_distribute >= (max >> 1) as isize {
        panic!("fail to normalize")
    }
    *max_norm += still_to_distribute as usize;
    Ok(())
}

/// Build cs = f0 + f1 + ... + fs-1
///
/// # hist
///
/// hist[symbol_index] is symbol frequency
/// hist.len() is number of symbols
pub fn build_cumulative_function(hist: &[usize]) -> Vec<usize> {
    let mut cs = Vec::with_capacity(hist.len() + 1);

    let cumul_fn = |acc, frequency| {
        cs.push(acc);
        acc + frequency
    };
    let sum = hist.iter().fold(0, cumul_fn);
    cs.push(sum);
    cs
}
