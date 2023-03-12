/// Compute an histogram with a very basic method.
pub fn simple_count_u8_inplace(src: &[u8], ret: &mut [usize; 256]) -> usize {
    let mut max_symbol = 0;
    src.iter().for_each(|&c| {
        ret[c as usize] += 1;
        max_symbol = std::cmp::max(max_symbol, c as usize)
    });
    max_symbol
}

/// Constant implementation of simple_count_u8_inplace. The implementation
/// may looks like the method bellow but current Rust version miss std::cmp::max
/// and for loops in constant functions.
///
/// That function might be used for testing, it will save a lot of execution time if
/// we need to read the same input over all the test cases.
///
/// If there is any update in a stable version of Rust, we would like to
/// fix that.
pub const fn simple_count_u8(src: &[u8]) -> ([usize; 256], usize) {
    let mut max_symbol = 0;
    let mut ret = [0usize; 256];
    let mut i = 0;
    loop {
        if i == src.len() {
            break;
        }
        let c = src[i] as usize;
        ret[c as usize] += 1;
        if max_symbol < c {
            max_symbol = c;
        }
        i += 1;
    }
    (ret, max_symbol)
}

/// Un test pour vérifier si l'OoO en rust est possible, de cette manière
/// on obtient des résultat plus performant que le conteur simple.
///
/// Dans le cas présent, un processeur capable de paralleliser des opérations
/// sera dans la possibilité d'effectuer les compteurs des buckets distincts en même temps.
/// Du fait que les buckets occupent chacuns des espaces different en cache, il
/// est peu probable qu'il y ait une invalidation de celui-ci. De plus, les
/// lignes 1.1, et 2.1 pourront être parrallèlisé par un processeur considérant
/// l'instruction `mov` comme une instruction pouvant être réordonné dans son
/// context. Il en va de même pour chacune des lignes numérotées.
///
/// ```ignore
/// let s1 = src[i] as usize;     // 1.1
/// bucket1[s1] += 1;             // 1.2
/// let s2 = src[i + 1] as usize; // 2.1
/// bucket2[s2] += 1;             // 2.2
/// let s3 = src[i + 2] as usize; // 3.1
/// bucket3[s3] += 1;             // 3.2
/// let s4 = src[i + 3] as usize; // 4.1
/// bucket4[s4] += 1;             // 4.2
/// ```
pub fn multi_bucket_count_u8(src: &[u8], ret: &mut [usize; 256]) -> usize {
    assert!(
        src.len() >= 4,
        "Length of src too small for a multibucket count"
    );
    let mut bucket1 = [0usize; 256];
    let mut bucket2 = [0usize; 256];
    let mut bucket3 = [0usize; 256];
    let mut bucket4 = [0usize; 256];
    let mut index = 0;
    for i in (0..src.len() - 4).step_by(4) {
        let s1 = src[i] as usize; // 1
        bucket1[s1] += 1;
        let s2 = src[i + 1] as usize; // 2
        bucket2[s2] += 1;
        let s3 = src[i + 2] as usize; // 3
        bucket3[s3] += 1;
        let s4 = src[i + 3] as usize; // 4
        bucket4[s4] += 1;
        index = i + 4;
    }
    (index..src.len()).for_each(|i| {
        let s = src[i] as usize; // 4
        bucket1[s] += 1;
    });
    let mut max_symbol = 0;
    for i in 0..ret.len() {
        ret[i] = bucket1[i] + bucket2[i] + bucket3[i] + bucket4[i];
        if ret[i] > 0 {
            max_symbol = i;
        }
    }
    max_symbol
}

/// Vu que précédemment nous avons consacré du temps à écrire un algorithm
/// de parallélisation du compteur de symboles bas niveau, nous passerons également
/// un peu de temps à écrire un "divisé pour mieux rêgner" beaucoup plus classique.
#[cfg(feature = "rayon")]
pub fn divide_and_conquer_count(src: &[u8], split: usize) -> ([usize; 256], usize) {
    use rayon::prelude::{ParallelIterator, ParallelSlice};

    let mut ret = [0; 256];
    let chunk_size = src.len() / split;
    let buckets = src
        .par_chunks(chunk_size)
        .map(|b| {
            let mut ret = [0usize; 256];
            b.iter().for_each(|&c| ret[c as usize] += 1);
            ret
        })
        .collect::<Vec<[usize; 256]>>();

    for bucket in buckets {
        for (&b, r) in bucket.iter().zip(ret.iter_mut()) {
            *r += b;
        }
    }
    let mut max_symbol = 0;
    (0..ret.len()).for_each(|i| {
        if ret[i] > 0 {
            max_symbol = i;
        }
    });
    (ret, max_symbol)
}
