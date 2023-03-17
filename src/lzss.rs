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
/// avec l'alphabet latin.
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
pub fn encode_lzw_no_windows_u8(src: &[u8]) -> Vec<u8> {
    let mut index = 4;
    let mut ret: Vec<u8> = vec![];
    ret.append(&mut src[..4].to_vec());

    while index < src.len() {
        let mut s = 0;
        let mut repetition = Pair::default();

        // Recherche de la plus longue séquence commune.
        while s < index {
            if src[s] == src[index] {
                let len = while_equal(src, s, index);

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

            // Construit la paire taille-index sur 32 bits
            const FLAG_MASK: u32 = 1 << 15;
            let bits: u32 = ((repetition.len | FLAG_MASK) << 16) + repetition.index as u32;
            ret.append(&mut bits.to_be_bytes().to_vec());
            index += repetition.len as usize;
        }
    }
    ret
}

// Voici un exemple de compression de donnés sans perte avec une approche
// grammaticale. Nous supposons que le text original contient de multiple
// répétitions de séquence, chose courante dans un langage naturel. Moins,
// cependant, dans un fichier contenant un binaire, bien que cette assertion
// soit discutable.
//
// Nous avons ci-dessus décrit l'algorithme dans son implémentation la plus
// simple, chaque étape est décrite avec suffisement de précision, et nous avons
// démontré son fonctionnement par de multiple test. Lorsque je dis que cet
// implémentation fonctionne, je suis sûr, à quelques pourcents proches de 100,
// qu'elle fonctionne. Dans un tel context, et au vu du temps que j'ai à y
// consacrer, je chercherai divers moyen d'accelerer l'execution, en gardant
// l'implementation original intacte. Vous pourriez vous demander pourquoi
// j'entre ainsi dans ces détails. Le fait est que je cherche à justifier que ce
// qui suivra n'est pas une optimisation prématurée. À mon sens, l'accumulation
// des faits étant: la stabilité de l'algorithme, diverses preuves du
// fonctionnement de l'implémentation, la validation des auteurs c'est à dire
// moi même et le temps que je souhaite y consacrer; nous éloignent d'un
// contexte prématuré. Chaque élément listé précédemment étant indispensable à
// cette condition.
//
// Concentrons nous à présent sur les possibles améliorations en nous
// intéressant au hardware. Nous savons que la plupart des processeurs que nous
// possédons ont la faculté d'executer plusieurs lécture ou écriture
// simultanément, tant que ces opérations n'opèrent pas dans des régions trop
// proches. Nous ne nous atarderons pas sur ce fait car il a déjà été abordé
// dans une précédente étude.

// Les premiers éléments à optimiser sont les boucles. En effet, compter
// peut être réalisé de façon parrallèle par le processeur. La première
// fonction sur laquelle nous nous pencherons compte le nombre de carractères
// identique à partir de deux indexes dans une source.

/// La fonction `while_equal` prend comme arguments une source et deux indexes.
/// Elle calculera le nombre de carractères identiques à partir de ces deux
/// indexes dans la limite suivante min(index - from, src.len - index).
///  
/// Dans le context présent, il est impératif que le premier index soit
/// inférieur au second, et ces deux index doivent être inférieur à la taille de
/// la source. De plus, j'ai choisi arbitrairement d'appeler cette fonction
/// uniquement lorsque je constate que deux éléments dans la source, à la
/// position `from` et `index`, sont égaux. Il convient donc de vérifier s'il
/// sont bien égaux avant de poursuivre la procédure.
///
/// La fonction est triviale et je doute qu'il faille s'attarder plus longtemps
/// dessus.
pub fn while_equal(src: &[u8], from: usize, index: usize) -> u32 {
    assert!(from < index);
    assert!(index < src.len());
    assert_eq!(src[from], src[index]);

    let mut s = from + 1;
    let mut i = index + 1;

    // Loop while equals
    while s < index && i < src.len() && src[s] == src[i] {
        s += 1;
        i += 1;
    }
    (s - from) as u32
}

// Dans le code précédant, la boucle de test peut tout à fait être divisé en 4.
// Ce que nous allons faire, en prenant soint de vérifier la consistance entre
// l'implémentation triviale et rapide à l'aide d'un jeu de tests. Nous pourrons
// vérifier si nous obtenons oui ou non de meilleures performances par la suite.

/// Cette implémentation simétrique à `while_equal`.
///
// Pour tenter de prouver une telle simétrie, il est important de définir des
// tests tels que ceux présents dans la figure suivante. Cependant nous
// pourrions nous en convaincre en parcourant le code attentivement.
// Premièrement, nous avons modifié le pas de la boucle principale initiallement
// de 1 à 4. Chaque test de i à i + 3 sont réalisés en utilisant de nouvelles
// variables locales. Un processeur OoO peut ainsi procéder parrallèlement
// chaque test. L'execution parrallèle s'arrête au moment de l'écriture de s,
// car cette opération doit respecter un ordre définis par le processeur
// lorsqu'il est sur un seul thread. De plus, l'union peut aussi se faire lors
// du break, car le branchement respecte les mêmes conditions que la variable s
// dans ce contexte.
//
// Dans un deuxieme temps, nous comptons les derniers caractères oublié dans les
// intervals [s - 4, index]  [i - 4, src.len()]. Ces derniers caractère ne
// pouvant pas être divisé en 4. Ensuite, ce serait une erreur de tenter de
// diviser en 3, ou 2 ces tests, l'ajout de branchement serait trop couteux par
// rapport au gain.
///
/// ```
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
pub fn while_equal_fast(src: &[u8], from: usize, index: usize) -> u32 {
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

// Le changement ce dessus peut être réitéré sur la recherche de symbole
// similaire. Toutefois, je considère important de préciser que de tels
// améliorations sont extrèmement dépendante du système dans lequel elles sont
// appliqué. Par exemple, si mon processeur est faible, très demandé par
// d'autres procéssus, il est tout à fait probable que les deux algorithmes
// aient des performances plus ou moins identiques. Sur des petites longueures,
// l'algorithme que nous considérions comme rapide pourrait même devenir lent.
// C'est à ce moment qu'interviennent quelques heuristiques égocentriques que
// je n'incluerai pas dans directement dans cette bibliothèque.

// Les résultats sont acceptables, relativement au

pub fn encode_lzw_no_windows_u8_fast(src: &[u8]) -> Vec<u8> {
    let mut index = 4;
    let mut ret: Vec<u8> = vec![];
    ret.append(&mut src[..4].to_vec());

    while index < src.len() {
        let mut s = 0;
        let mut repetition = Pair::default();

        // Recherche de la plus longue séquence commune.
        while s < index {
            if src[s] == src[index] {
                let len = while_equal_fast(src, s, index);
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

            // Construit la paire taille-index sur 32 bits
            const FLAG_MASK: u32 = 1 << 15;
            let bits: u32 = ((repetition.len | FLAG_MASK) << 16) + repetition.index as u32;
            ret.append(&mut bits.to_be_bytes().to_vec());
            index += repetition.len as usize;
        }
    }
    ret
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

    let encoded1 = encode_lzw_no_windows_u8_fast(src);
    let encoded2 = encode_lzw_no_windows_u8(src);

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
    let encoded = encode_lzw_no_windows_u8(src.as_bytes());
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
    let encoded = encode_lzw_no_windows_u8(book1);
    let decoded = decode_lzw_u8(&encoded);
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
    let len1 = while_equal_fast(src, 2046, 3991);
    let len2 = while_equal(src, 2046, 3991);
    assert_eq!(len1, len2);
}

#[test]
fn while_equal_consitency_doctest_enhanced() {
    let src = "ABCDFGHABCDEFGHI".as_bytes();
    let len1 = while_equal_fast(src, 0, 7);
    let len2 = while_equal(src, 0, 7);
    assert_eq!(len1, len2);

    let src = "ABCDABCDEFGHI".as_bytes();
    let len1 = while_equal_fast(src, 0, 4);
    let len2 = while_equal(src, 0, 4);
    assert_eq!(len1, len2);

    let src = "AA".as_bytes();
    let len1 = while_equal_fast(src, 0, 1);
    let len2 = while_equal(src, 0, 1);
    assert_eq!(len1, len2);

    let src = "ABAB".as_bytes();
    let len1 = while_equal_fast(src, 0, 2);
    let len2 = while_equal(src, 0, 2);
    assert_eq!(len1, len2);

    let src = "ABCABC".as_bytes();
    let len1 = while_equal_fast(src, 0, 3);
    let len2 = while_equal(src, 0, 3);
    assert_eq!(len1, len2);

    let src = "ABCDABC".as_bytes();
    let len1 = while_equal_fast(src, 0, 4);
    let len2 = while_equal(src, 0, 4);
    assert_eq!(len1, len2);

    let src = "ABCDABCD".as_bytes();
    let len1 = while_equal_fast(src, 0, 4);
    let len2 = while_equal(src, 0, 4);
    assert_eq!(len1, len2);
}
