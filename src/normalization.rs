//! Ce fichier contient multiple implémentation de normalisation. Il est
//! utilisé par la bibliothèque en interne, bien qu'acessible en soit par
//! un utilisateur externe.
//!
//! Implémentation de final-state-rs, tenter d'implémenter FSE en Rust.
//! Author: Adrien Zinger, avec l'inspiration du travail de Jarek Duda,
//!         Yann Collet, Charles Bloom et bien d'autres.

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
) -> Result<(), Box<NormError>> {
    let len = hist.len();
    const HIGH_NUM: usize = (usize::BITS - 2) as usize;

    let scale: usize = HIGH_NUM - table_log;
    let step: usize = (1usize << HIGH_NUM) / hist.iter().sum::<usize>();
    let mut max = 0;
    let mut max_norm = &mut 0;
    let mut still_to_distribute: isize = 1 << table_log;
    for s in hist.iter_mut() {
        if *s == len {
            return Err(Box::new(NormError::RunLengthEncoding(
                "An rle compression should be more accurate",
            )));
        } else if *s > 0 {
            let proba = ((*s) * step) >> scale;
            *s = proba;
            if proba > max {
                max_norm = s;
                max = proba;
            }
            still_to_distribute -= proba as isize;
        }
    }
    if -still_to_distribute >= (max >> 1) as isize {
        // todo: erreur
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

/// Normalisation utilisant une interpolation linéaire de la somme cumulative
/// de l'histogramme. On normalise la fonction cumulative et on en déduis
/// l'histogramme en calculant la dérivée de la fonction.
///
/// On pourrait surement améliorer cette méthode en la rendant plus robuste.
/// Par exemple on pourrait tenter de normaliser avec une table log < total de
/// l'histogramme. Mais cette méthode reste un peu plus lente que l'original,
/// de plus je ne peux pas affirmer qu'elle soit performante pour la
/// compression. À tester.
///
/// # Return
/// The cumulative function in a Ok, or a normalization error in an Err.
/// The input `histogram` is modified in a side effect.
pub fn derivative_normalization(
    histogram: &mut [usize],
    table_log: usize,
) -> Result<Vec<usize>, NormError> {
    // linear interpolation naïve sur une fonction de cumulation
    let mut previous = 0;
    let mut cumul = build_cumulative_function(histogram);
    let max_cumul = *cumul.last().unwrap();
    let target_range = 1 << table_log; // D - C
    let actual_range = max_cumul; // B - A

    cumul.iter_mut().enumerate().skip(1).for_each(|(i, c)| {
        *c = (target_range * (*c)) / actual_range;
        if *c <= previous {
            panic!("table log too low");
            // todo: we expect to never force value actually...
            // we need to increase table_log instead

            // note: we could force to previous + 1 and accumulate a dept that
            //       we substract to the nexts values. If at the end we keep
            //       a dept > 0 we should panic. If not just inform user that
            //       we got to force the normalized counter to fit.

            // D'autres idées:
            // 1. Correction à posteriorie, si j'ai une dette, après avoir
            // calculé ma cdf je verifie si je peut pas supprimer quelques
            // truc pour forcer a faire entrer dans mon table_log.
            // 2. Panic je double
            // 3. Lorsque je tombe sur un pépin, j'invertie les deux dernières
            // valeurs.
        }

        histogram[i - 1] = *c - previous;
        previous = *c;
    });
    Ok(cumul)
}

/// Pareil en somme à la normalisation dérivative. Excepté qu'on augmente le
/// numérateur avec un nombre important (2^62 ou 2^30 selon l'architecture).
/// Cette méthode peut ne pas être adapté avec des fréquence d'aparitions trop
/// grandes.
pub fn derivative_normalization_fast(
    histogram: &mut [usize],
    table_log: usize,
) -> Result<Vec<usize>, NormError> {
    let mut previous = 0;
    let mut cumul = build_cumulative_function(histogram);
    let max_cumul = *cumul.last().unwrap();
    const HIGH_NUM: usize = usize::BITS as usize - 2;
    let scale: usize = HIGH_NUM - table_log;
    let step = (1 << HIGH_NUM) / max_cumul;
    let mut still_to_distribute = 1 << table_log;
    for (i, c) in cumul.iter_mut().enumerate().skip(1) {
        *c = (*c)
            .checked_mul(step)
            .ok_or(NormError::MultiplicationOverflow)?
            >> scale;
        if *c <= previous {
            panic!("table log too low");
        }
        histogram[i - 1] = *c - previous;
        still_to_distribute -= histogram[i - 1];
        previous = *c;
    }
    if still_to_distribute > 0 {
        *cumul.last_mut().unwrap() += still_to_distribute;
        *histogram.last_mut().unwrap() += still_to_distribute;
    }
    Ok(cumul)
}
