pub fn encode_lzss_no_windows_u8(src: &[u8]) -> Vec<u8> {
    let mut index = 4;
    let mut ret: Vec<u8> = vec![];
    ret.append(&mut src[..4].to_vec());
    const LEN_MASK: u32 = 1 << 15;
    while index < src.len() {
        let mut s = 0;
        let tmp = index;
        while s + 4 < index {
            if src[s] == src[index] {
                let len = while_equal(src, s, index);
                if (4..32768).contains(&len) {
                    //println!("push len {len} at {s}");
                    let bits: u32 = ((len | LEN_MASK) << 16) + s as u32;
                    ret.append(&mut bits.to_be_bytes().to_vec());
                    index += len as usize;
                    if index == src.len() {
                        return ret;
                    }
                    break;
                }
            }

            if src[s + 1] == src[index] {
                let len = while_equal(src, s + 1, index);
                if (4..32768).contains(&len) {
                    //println!("push len {len} at {}", s + 1);
                    let bits: u32 = ((len | LEN_MASK) << 16) + s as u32 + 1;
                    ret.append(&mut bits.to_be_bytes().to_vec());
                    index += len as usize;
                    if index == src.len() {
                        return ret;
                    }
                    break;
                }
            }

            if src[s + 2] == src[index] {
                let len = while_equal(src, s + 2, index);

                if (4..32768).contains(&len) {
                    //println!("push len {len} at {}", s + 2);
                    let bits: u32 = ((len | LEN_MASK) << 16) + s as u32 + 2;
                    ret.append(&mut bits.to_be_bytes().to_vec());
                    index += len as usize;
                    if index == src.len() {
                        return ret;
                    }
                    break;
                }
            }

            if src[s + 3] == src[index] {
                let len = while_equal(src, s + 3, index);
                if (4..32768).contains(&len) {
                    //println!("push len {len} at {}", s + 3);
                    let bits: u32 = ((len | LEN_MASK) << 16) + s as u32 + 3;
                    ret.append(&mut bits.to_be_bytes().to_vec());
                    index += len as usize;
                    if index == src.len() {
                        return ret;
                    }
                    break;
                }
            }

            s += 4;
        }

        if tmp == index {
            ret.push(src[index]);
            index += 1
        }
    }
    ret
}

pub fn encode_lzss_no_windows_u8_simple(src: &[u8]) -> Vec<u8> {
    let mut index = 4;
    let mut ret: Vec<u8> = vec![];
    ret.append(&mut src[..4].to_vec());
    while index < src.len() {
        let mut s = 0;
        let tmp = index;
        while s < index {
            if src[s] == src[index] {
                let len = while_equal_simple(src, s, index);
                if (4..32768).contains(&len) {
                    let bits: u32 = ((len << 16) | 1 << 15) + s as u32;
                    ret.append(&mut bits.to_be_bytes().to_vec());
                    index += len as usize;
                    if index == src.len() {
                        return ret;
                    }
                    break;
                }
            }
            s += 1;
        }
        if tmp == index {
            ret.push(src[index]);
            index += 1
        }
    }
    ret
}

// La fonction ci-dessus est un exemple d'application de compression suivant une
// méthode dérivée de LZW. Un algorithme LZSS, doit avoir une fenêtre de taille
// fixe dans laquelle rechercher des séries répétées pour pouvoir être O(n). Nous
// verrons une implémentation de la sorte plus tard. Pour l'instant concentrons
// nous sur les possibles améliorations, en nous intéressant au hardware.

// Prenons le code suivant:
// while s + 1 < index && tmp_index + 1 < src.len()
//       && src[s + 1] == src[tmp_index + 1]
// {
//     s += 1;
//     tmp_index += 1;
// }
// Nous savons qu'un processeur OoO comme celui que je possède actuellement
// peut parralléliser quelques instructions de lecture et d'écriture.

pub fn while_equal(src: &[u8], from: usize, index: usize) -> u32 {
    let mut s = from + 1;
    let mut i = index + 1;
    while s + 4 < index && i + 4 < src.len() {
        if src[s] != src[i] {
            break;
        }
        if src[s + 1] != src[i + 1] {
            s += 1;
            break;
        }
        if src[s + 2] != src[i + 2] {
            s += 2;
            break;
        }
        if src[s + 3] != src[i + 3] {
            s += 3;
            break;
        }
        s += 4;
        i += 4;
    }

    if s + 4 >= index || i + 4 >= src.len() {
        while s < index && i < src.len() && src[s] == src[i] {
            s += 1;
            i += 1;
        }
    }

    (s - from) as u32
}

pub fn while_equal_simple(src: &[u8], from: usize, index: usize) -> u32 {
    let mut s = from + 1;
    let mut i = index + 1;
    while s < index && i < src.len() && src[s] == src[i] {
        s += 1;
        i += 1;
    }
    (s - from) as u32
}

#[test]
fn while_equal_functions_consistency() {
    let src = "ABCDFGHABCDEFGHI".as_bytes();
    println!("src: {:?}", src);
    let len1 = while_equal(src, 0, 7);
    let len2 = while_equal_simple(src, 0, 7);
    assert_eq!(len1, len2);

    let src = "ABCDABCDEFGHI".as_bytes();
    let len1 = while_equal(src, 0, 4);
    let len2 = while_equal_simple(src, 0, 4);
    assert_eq!(len1, len2);

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
    let len1 = while_equal(src, 2046, 3991);
    let len2 = while_equal_simple(src, 2046, 3991);
    assert_eq!(len1, len2);
}

#[test]
fn lzss_optimizations_functions_consistency() {
    use std::fs::File;
    use std::io::Read;

    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");

    let src = &book1[0..4000];

    let encoded1 = encode_lzss_no_windows_u8(src);
    let encoded2 = encode_lzss_no_windows_u8_simple(src);

    assert_eq!(encoded1.len(), encoded2.len());
}

pub fn encode_lzss_with_windows_u8(src: &[u8], windows_size: usize) -> Vec<u8> {
    // assert de la taille de la fenetre < 32768
    let mut index = 4;
    let mut ret: Vec<u8> = vec![];
    ret.append(&mut src[..4].to_vec());
    while index < src.len() {
        let mut s = index.saturating_sub(windows_size);
        while s < index {
            if src[s] == src[index] {
                let from = s;
                let mut tmp_index = index + 1;
                s += 1;
                while s + 1 < index && tmp_index + 1 < src.len() && src[s + 1] == src[tmp_index + 1]
                {
                    s += 1;
                    tmp_index += 1;
                }
                let len = (s - from + 1) as u32;
                if (4..32768).contains(&len) {
                    // ajuster ce test avec la taille de la fenetre.
                    let bits: u32 = ((len << 16) | 1 << 15) + from as u32;
                    ret.append(&mut bits.to_be_bytes().to_vec());
                    index = tmp_index + 1;
                    if index == src.len() {
                        return ret;
                    }
                    break;
                }
            } else {
                s += 1;
            }
        }
        ret.push(src[index]);
        index += 1
    }
    ret
}

pub fn dencode_lzss_u8(src: &[u8]) -> Vec<u8> {
    let mut ret: Vec<u8> = vec![];
    let mut it = src.iter();
    while let Some(symbol) = it.next() {
        if *symbol >> 7 == 1 {
            let hi_bits_len = ((*symbol & 0b01111111) as u16) << 8;
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

#[test]
fn no_windows_test() {
    let src = "ABCABCABCBADABCABCABCABCABCDBA";
    println!("source: {:?}", src.as_bytes());
    let encoded = encode_lzss_no_windows_u8(src.as_bytes());
    println!("encoded {:?}", encoded);
    for e in encoded.iter() {
        println!("{:8b}", *e);
    }
    let decoded = dencode_lzss_u8(&encoded);
    assert_eq!(src.as_bytes(), decoded);
}

#[test]
fn no_windows_calgary_book1_compression_test() {
    use std::{fs::File, io::Read};
    /*

    src: [101, 32, 98, 111, 121, 46, 10, 72, 105, 115, 32, 104, 101, 105, 103, 104, 116, 32, 97, 110, 100, 32, 98, 114, 101, 97, 100, 116, 104, 32, 119, 111, 117, 108, 100, 32, 104, 97, 118, 101, 32, 98, 101, 101, 110, 32, 115, 117, 102, 102, 105, 99, 105, 101, 110, 116, 32, 116, 111, 10, 109, 97, 107, 101, 32, 104, 105, 115, 32, 112, 114, 101, 115, 101, 110, 99, 101, 32, 105, 109, 112, 111, 115, 105, 110, 103, 44, 32, 104, 97, 100, 32, 116, 104, 101, 121, 32, 98, 101, 101, 110, 32, 101, 120, 104, 105, 98, 105, 116, 101, 100, 10, 119, 105, 116, 104, 32, 100, 117, 101, 32, 99, 111, 110, 115, 105, 100, 101, 114, 97, 116, 105, 111, 110, 46, 32, 66, 117, 116, 32, 116, 104, 101, 114, 101, 32, 105, 115, 32, 97, 32, 119, 97, 121, 32, 115, 111, 109, 101, 32, 109, 101, 110, 10, 104, 97, 118, 101, 44, 32, 114, 117, 114, 97, 108, 32, 97, 110, 100, 32, 117, 114, 98, 97, 110, 32, 97, 108, 105, 107, 101, 44, 32, 102, 111, 114, 32, 119, 104, 105, 99, 104, 32, 116, 104, 101, 32, 109, 105, 110, 100, 32, 105, 115, 32, 109, 111, 114, 101, 10, 114, 101, 115, 112, 111, 110, 115, 105, 98, 108, 101, 32, 116, 104, 97, 110, 32, 102, 108, 101, 115, 104, 32, 97, 110, 100, 32, 115, 105, 110, 101, 119, 32, 58, 32, 105, 116, 32, 105, 115, 32, 97, 32, 119, 97, 121, 32, 111, 102, 32, 99, 117, 114, 116, 97, 105, 108, 43, 10, 105, 110, 103, 32, 116, 104, 101, 105, 114, 32, 100, 105, 109, 101, 110, 115, 105, 111, 110, 115, 32]
    enc: [101, 32, 98, 111, 121, 46, 10, 72, 105, 115, 32, 104, 101, 105, 103, 104, 116, 32, 97, 110, 100, 32, 98, 114, 101, 97, 100, 116, 104, 32, 119, 111, 117, 108, 100, 32, 104, 97, 118, 101, 32, 98, 101, 101, 110, 32, 115, 117, 102, 102, 105, 99, 105, 101, 110, 116, 32, 116, 111, 10, 109, 97, 107, 101, 32, 104, 105, 115, 32, 112, 114, 101, 115, 101, 110, 99, 101, 32, 105, 109, 112, 111, 115, 105, 110, 103, 44, 32, 104, 97, 100, 32, 116, 104, 101, 121, 128, 6, 0, 40, 101, 120, 104, 105, 98, 105, 116, 101, 100, 10, 119, 105, 116, 104, 32, 100, 117, 101, 32, 99, 111, 110, 115, 105, 100, 101, 114, 97, 116, 105, 111, 110, 46, 32, 66, 117, 116, 128, 4, 0, 91, 114, 101, 32, 105, 115, 32, 97, 32, 119, 97, 121, 32, 115, 111, 109, 101, 32, 109, 101, 110, 10, 128, 4, 0, 36, 44, 32, 114, 117, 114, 97, 108, 128, 5, 0, 17, 117, 114, 98, 97, 110, 32, 97, 108, 105, 107, 101, 44, 32, 102, 111, 114, 32, 119, 104, 105, 99, 104, 128, 4, 0, 91, 32, 109, 105, 110, 100, 128, 4, 0, 145, 109, 111, 114, 101, 10, 114, 101, 115, 112, 128, 4, 0, 122, 98, 108, 101, 32, 116, 104, 97, 110, 32, 102, 108, 101, 115, 104, 128, 5, 0, 17, 115, 105, 110, 101, 119, 32, 58, 32, 105, 116, 128, 10, 0, 145, 111, 102, 32, 99, 117, 114, 116, 97, 105, 108, 43, 10, 105, 110, 103, 128, 4, 0, 91, 105, 114, 32, 100, 105, 109, 101, 110, 115, 105, 111, 110, 115, 32]


     */
    let mut book1 = vec![];
    File::open("./rsc/calgary_book1")
        .expect("Cannot find calgary book1 ressource")
        .read_to_end(&mut book1)
        .expect("Unexpected fail to read calgary book1 ressource");
    let book1 = &book1[3000..4000];

    println!("source len: {}", book1.len());

    let encoded = encode_lzss_no_windows_u8(book1);

    println!("encoded len {:?}", encoded.len());

    let decoded = dencode_lzss_u8(&encoded);
    assert_eq!(book1, decoded)
}
