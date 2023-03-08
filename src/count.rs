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
/// If there is any update in a stable version of Rust, we would like to
/// fix that.
pub const fn simple_count_u8(src: &[u8]) -> ([usize; 256], usize) {
    let mut max_symbol = 0;
    let mut ret = [0usize; 256];
    let mut i = 0;
    loop {
        if i == src.len() - 1 {
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
/// on obtient des résultat un peu plus lent que le conteur simple.
/// Plus d'investigation autour des conteurs sera nécessaire.
pub fn multi_bucket_count_u8(src: &[u8], ret: &mut [usize; 256]) {
    for i in (0..src.len() - 4).step_by(4) {
        let s1 = src[i] as usize;
        ret[s1] += 1;
        let s2 = src[i + 1] as usize;
        ret[s2] += 1;
        let s3 = src[i + 2] as usize;
        ret[s3] += 1;
        let s4 = src[i + 3] as usize;
        ret[s4] += 1;
    }
}
