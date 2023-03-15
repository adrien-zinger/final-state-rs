// Dans le cadre de l'implémentation d'une méthode de compréssion dérivée de
// LZW, nous nous interresserons particulièrement à LZSS. Ou en tout cas, dans
// ce fichier.

// Commençons par une implémentation facile de l'algorithme. Nous connaissons
// les grandes étape de cette méthode de compression. Lesquels sont:

// Pour chaque symbole à une position `x`: rechercher dans l'interval [0, x[ la
// plus grande séquence commune avec la suite de symboles s(x) + s(x+1) + s(x+2)
// + ... + s(x+n). Remplacer la suite par une paire taille-index indiquant la
// sous-sequence commune.

// De plus, nous souhaitons que cette séquence réspecte deux critères.
// Premièrement, sa longueur doit être supérieur à 4, qui est le nombre d'octet
// minimum nécessaire pour écrire la paire taille-index substituant la
// sous-séquence original. Si nous échangions toutes les séquences inférieures à
// ce nombre, nous nous retrouverions avec un text compressé plus grand que le
// text original. En effet, imaginez que nous remplaçions chaque lettre déjà
// présente dans une phrase par 4 autre lettres, si nous récitions l'alphabet,
// celà ne poserai pas de problème. Mais si nous répetions une lettre
// quelconque, nous nous trouverions alors avec 24 symboles, additionnés à 4
// autres, si nous récitons l'alphabet français, alors que 25 auraient suffis.
// Deuxièmement, la séquence doit avoir une taille inférieure à 2¹⁵, car nous
// réserverons le premier bit pour y mettre une signal de compression. Lors du
// décodage, si le premier octet est supérieur à 2¹⁵, nous savons que nous
// allons lire une paire taille-index. Ceci implique également que chaque
// symbole du text original est également inférieur à 2⁷.
//
// Vous remarquerez qu'un tel algorithme doit avoir une complexité quadratique.
// En effet, pour une entré de taille n, nous executerons pour chaque symbole un
// nombre d'opération égal à sa position. Comme vous pouvez le constater dans la
// figure ci dessous, le nombre d'opérations risque d'augmenter infiniment dans
// des proportions que nous ne pourrions accepter. Nous verrons comment résoudre
// ce problème plus tard, pour le moment nous l'ignorerons.
//
// f(2) = 1
// f(3) = 2 + 1 = 3
// f(4) = 3 + 2 + 1 = 5
// f(5) = 4 + 3 + 2 + 1 = 9
// f(6) = 5 + 4 + 3 + 2 + 1 = 14
//
//

/// La fonction suivante encodera une source en suivant une variation de
/// l'algorithme lzss. Pour le moment, nous chercherons des récurrences de
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
/// use final_state_rs::lzss::encode_lzw_no_windows_u8_simple;
/// use final_state_rs::lzss::decode_lzw_u8;
/// use std::{fs::File, io::Read};
///
/// let mut book1 = vec![];
/// File::open("./rsc/calgary_book1")
///     .expect("Cannot find calgary book1 ressource")
///     .read_to_end(&mut book1)
///     .expect("Unexpected fail to read calgary book1 ressource");
/// book1 = book1[0..4000].to_vec();
///
/// let encoded = encode_lzw_no_windows_u8_simple(&book1);
/// let decoded = decode_lzw_u8(&encoded);
///
/// assert_eq!(book1, decoded);
/// assert!(encoded.len() <= decoded.len());
/// ```
///
/// Une source incompréssible, par exemple l'alphabet, devrait avoir une forme
/// identique une fois compressée. Et puisque nous en somme à prouver que le
/// résultat ne dépassera jamais en taille la source, prenons l'exemple précédent
/// avec l'alphabet latin utilisé en France.
///
/// ```
/// use final_state_rs::lzss::encode_lzw_no_windows_u8_simple;
/// use final_state_rs::lzss::decode_lzw_u8;
///
/// let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZA".as_bytes();
/// let encoded = encode_lzw_no_windows_u8_simple(&alphabet);
/// let decoded = decode_lzw_u8(&encoded);
/// assert_eq!(alphabet, encoded);
/// assert_eq!(decoded, encoded);
/// ```
pub fn encode_lzw_no_windows_u8_simple(src: &[u8]) -> Vec<u8> {
    let mut index = 4;
    let mut ret: Vec<u8> = vec![];
    ret.append(&mut src[..4].to_vec());

    while index < src.len() {
        let mut s = 0;
        let mut repetition = Pair::default();
        while s < index {
            if src[s] == src[index] {
                let len = while_equal_simple(src, s, index);
                if (5..32768).contains(&len) && repetition.len < len {
                    repetition.len = len;
                    repetition.index = s;
                }
            }
            s += 1;
        }
        if repetition.len == 0 {
            // Je n'ai pas trouvé une répétition,
            // donc j'écris le symbole et j'avance de 1.
            ret.push(src[index]);
            index += 1;
        } else {
            // J'ai trouvé une répétition, j'avance de la
            // taille de celle-ci

            const FLAG_MASK: u32 = 1 << 15;

            // Construit la paire taille-index sur 32 bits
            let bits: u32 = ((repetition.len | FLAG_MASK) << 16) + repetition.index as u32;
            ret.append(&mut bits.to_be_bytes().to_vec());
            index += repetition.len as usize;
        }
    }
    ret
}

// La fonction ci-dessus est un exemple d'application de compression suivant une
// méthode dérivée de LZW.
// Concentrons nous sur les possibles améliorations en nous intéressant au hardware. Nous
// savons que pour la plupart des processeurs que nous possédons, la faculter d'executer
// plusieurs lécture ou écriture simultanément est possible, tant que ces opérations
// n'opèrent pas dans des régions trop proches. Nous ne nous atarderons pas sur ce fait
// car il a déjà été abordé dans une précédente étude.
//

pub fn while_equal_simple(src: &[u8], from: usize, index: usize) -> u32 {
    let mut s = from + 1;
    let mut i = index + 1;
    while s < index && i < src.len() && src[s] == src[i] {
        s += 1;
        i += 1;
    }
    (s - from) as u32
}

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

pub fn encode_lzss_no_windows_u8(src: &[u8]) -> Vec<u8> {
    let mut index = 4;
    let mut ret: Vec<u8> = vec![];
    ret.append(&mut src[..4].to_vec());
    while index < src.len() {
        let mut s = 0;
        let mut repetition = Pair::default();

        while s + 4 < index {
            if src[s] == src[index] {
                let len = while_equal(src, s, index);
                if (5..32768).contains(&len) && repetition.len < len {
                    repetition.len = len;
                    repetition.index = s;
                }
            }

            if src[s + 1] == src[index] {
                let len = while_equal(src, s + 1, index);
                if (5..32768).contains(&len) && repetition.len < len {
                    repetition.len = len;
                    repetition.index = s + 1;
                }
            }

            if src[s + 2] == src[index] {
                let len = while_equal(src, s + 2, index);
                if (5..32768).contains(&len) && repetition.len < len {
                    repetition.len = len;
                    repetition.index = s + 2;
                }
            }

            if src[s + 3] == src[index] {
                let len = while_equal(src, s + 3, index);
                if (5..32768).contains(&len) && repetition.len < len {
                    repetition.len = len;
                    repetition.index = s + 3;
                }
            }

            s += 4;
        }

        if repetition.len == 0 {
            ret.push(src[index]);
            index += 1;
        } else {
            const FLAG_MASK: u32 = 1 << 15;
            let bits: u32 = ((repetition.len | FLAG_MASK) << 16) + repetition.index as u32;
            ret.append(&mut bits.to_be_bytes().to_vec());
            index += repetition.len as usize;
        }
    }
    ret
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
    let encoded2 = encode_lzw_no_windows_u8_simple(src);

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

pub fn decode_lzw_u8(src: &[u8]) -> Vec<u8> {
    let mut ret: Vec<u8> = vec![];
    let mut it = src.iter();
    const FLAG_BIT: u8 = 1 << 7;
    while let Some(symbol) = it.next() {
        if *symbol >= FLAG_BIT {
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

/* *************************************************************************
_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-

    Annexe contenant quelques tests suplémentaires ainsi que des déclarations
    pratique pour notre présentation.

_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-_-
 ************************************************************************  */

/// Représentation d'une paire taille-index, nous aurions pu nous
/// en passer et utiliser un simple tuple. Seulement ajouter cette
/// structure augmente la clarté du code. De plus, elle n'impacte en
/// rien les performances.
#[derive(Default)]
struct Pair {
    index: usize,
    len: u32,
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
    let decoded = decode_lzw_u8(&encoded);
    assert_eq!(src.as_bytes(), decoded);
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

    println!("source len: {}", book1.len());

    let encoded = encode_lzss_no_windows_u8(book1);

    println!("encoded len {:?}", encoded.len());

    let decoded = decode_lzw_u8(&encoded);
    assert_eq!(book1, decoded)
}
