/// Compute an histogram with a very basic method.
pub fn simple_count_u8(src: &[u8], ret: &mut [usize; 256]) {
    src.iter().for_each(|&c| ret[c as usize] += 1)
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
