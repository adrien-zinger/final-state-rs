//! Implémentation de l'algorithme LZSS dans `final_state_rs`. Cette partie
//! du programme est implémentée avec un style literate programming.
//!
//! Autheur: Adrien Zinger <zinger.ad@gmail.com>

Dans le cadre de l'implémentation d'une méthode de compréssion dérivée de
LZW, nous nous interresserons particulièrement à LZSS. Ou en tout cas, dans
ce fichier.

Commençons par une simple implémentation de l'algorithme. Nous connaissons les grandes étapes de cette méthode de compression. Lesquels sont:

Pour chaque symbole à la position `x`: rechercher dans l'intervalle `[0, x[` la plus grande séquence commune avec la suite de symboles `s(x) + s(x+1) + s(x+2) + ... + s(x+n)`. Remplacer la suite par une paire taille-index indiquant la sous-séquence commune.

De plus, nous souhaitons que cette séquence respecte deux critères. Premièrement, sa longueur doit être supérieure à 4, qui est le nombre d'octets minimum nécessaire pour écrire la paire taille-index substituant la sous-séquence originale. Si nous échangions toutes les séquences inférieures à ce nombre, nous nous retrouverions avec un texte compressé plus grand que le texte original. En effet, imaginez que nous remplacions chaque lettre déjà présente dans une phrase par 4 autres lettres, si nous récitions l'alphabet, cela ne poserait pas de problème. Mais si nous répétions une lettre quelconque, nous nous trouverions alors avec 24 symboles, additionnés à 4 autres, si nous récitons l'alphabet français, alors que 25 auraient suffi. Deuxièmement, la séquence doit avoir une taille inférieure à 2¹⁵, car nous réserverons le premier bit pour y mettre un signal de compression. Lors du décodage, si le premier octet est supérieur à 2¹⁵, nous savons que nous allons lire une paire taille-index. Ceci implique également que chaque symbole du texte original est également inférieur à 2⁷.

Vous remarquerez qu'un tel algorithme doit avoir une complexité quadratique. En effet, pour une entrée de taille n, nous exécuterons pour chaque symbole un nombre d'opérations égal à sa position. Comme vous pouvez le constater dans la figure ci-dessous, le nombre d'opérations risque d'augmenter infiniment dans des proportions que nous ne pourrions accepter. Nous verrons comment résoudre ce problème plus tard, pour le moment nous l'ignorerons.

```
f(2) = 1
f(3) = 2 + 1 = 3
f(4) = 3 + 2 + 1 = 5
f(5) = 4 + 3 + 2 + 1 = 9
f(6) = 5 + 4 + 3 + 2 + 1 = 14
```

La fonction suivante encodera une source en suivant une variation de l'algorithme lzss. Pour le moment, nous chercherons des récurrences de termes dans tout l'intervalle précédant l'index actuel. Autrement dit, pour une séquence de symboles situés dans l'intervalle `[i, n]` je chercherai les séquences similaires dans l'intervalle `[0, i[` de taille n, et je sélectionnerai la séquence commune avec la valeur de n la plus grande.

Un tel algorithme doit respecter certaines conditions pour être valide en matière de compression de données. Premièrement, nous devons être en mesure de décoder la sortie compressée et forcement pouvoir retrouver la séquence initiale sans modification. Deuxièmement, la donnée compressée doit être de taille inférieure ou égale à la source, ce point peut sembler évident mais tout algorithme ne respecte pas cette condition.

```rust
use final_state_rs::lzss::*;
use std::{fs::File, io::Read};

let mut book1 = [0; 4000];
File::open("./rsc/calgary_book1")
    .expect("Cannot find calgary book1 ressource")
    .read(&mut book1)
    .expect("Unexpected fail to read calgary book1 ressource");

let encoded = encode_lzw_no_windows_u8(&book1);
let decoded = decode_lzw_u8(&encoded);

assert_eq!(book1.to_vec(), decoded);
assert!(encoded.len() <= decoded.len());
```

Une source incompressible, par exemple l'alphabet, devrait avoir une forme identique une fois compressée. Et puisque nous en sommes à prouver que le résultat ne dépassera jamais en taille la source, prenons l'exemple précédent avec l'alphabet latin.

```rust
use final_state_rs::lzss::*;

let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZA".as_bytes();
let encoded = encode_lzw_no_windows_u8(&alphabet);
let decoded = decode_lzw_u8(&encoded);
assert_eq!(alphabet, encoded);
assert_eq!(decoded, encoded);

pub fn encode_lzw_no_windows_u8(src: &[u8]) -> Vec<u8> {
    internal_encode_lzw_no_windows_u8::<Original>(src)
}
```

Implémentation générique de lzw sans fenêtre. Utilisé par `encode_lzw_no_windows_u8` et `encode_lzw_no_windows_u8_fast` décrit plus loin. Élimine de la duplication de code par pur principe.

Nous reviendrons rapidement sur les raisons de cette généricité, pour le moment vous pouvez faire abstraction du template.

```rust
fn internal_encode_lzw_no_windows_u8<T: WhileEqual>(src: &[u8]) -> Vec<u8> {
    let mut index = 4;
    let mut ret: Vec<u8> = vec![];
    ret.append(&mut src[..4].to_vec());

    while index < src.len() - 4 {
        let mut s = 0;
        let mut repetition = Pair::default();

        Recherche de la plus longue séquence dans l'interval
        [s; index - 4]. Puisque nous savons qu'un charactère identique
        en `index - 4` donnera une taille maximum de 4, nous pouvons
        dors et déjà éviter un encodage superflue.
        while s < index - 4 {
            if src[s] == src[index] {
                Si src[s] == src[index], nous pouvons commencer à rechercher
                la taille de la séquence commune à partir des deux indexes.
                let len = T::while_equal(src, s, index);
                if (5..32768).contains(&len) && repetition.len < len {
                    repetition.len = len;
                    repetition.index = s;
                }
            }
            s += 1;
        }
        if repetition.len == 0 {
            Je n'ai trouvé aucune répétition,
            donc j'écris le symbole et j'avance de 1.
            ret.push(src[index]);
            index += 1;
        } else {
            J'ai trouvé une répétition, j'avance de la
            taille de celle-ci

            Construit la paire taille-index sur 32 bits
            const FLAG_MASK: u32 = 1 << 15;
            let bits: u32 = ((repetition.len | FLAG_MASK) << 16) + repetition.index as u32;
            ret.append(&mut bits.to_be_bytes().to_vec());
            index += repetition.len as usize;
        }
    }
    Ecrit les dernier bits restants dans le cas où index est
    dans l'interval [len - 4; len[
    if index < src.len() {
        let diff = src.len() - index;
        ret.append(&mut src[src.len() - diff..].to_vec());
    }
    ret
}
```

Voici un exemple de compression de donnés sans perte avec une approche grammaticale. Nous supposons que le texte original contient de multiples répétitions de séquence, chose courante dans un langage naturel. Moins, cependant, dans un fichier contenant un binaire, bien que cette assertion soit discutable.

Nous avons ci-dessus décrit l'algorithme dans son implémentation la plus simple, chaque étape est décrite avec suffisamment de précision, et nous avons démontré son fonctionnement par de multiples tests. Lorsque je dis que cette implémentation fonctionne, je suis sûr, à quelques pourcents proches de 100, qu'elle fonctionne. Dans un tel contexte, et au vu du temps que j'ai à y consacrer, je chercherai divers moyens d'accélérer l'exécution, en gardant l'implementation originale intacte. Vous pourriez vous demander pourquoi j'entre ainsi dans ces détails. Le fait est que je cherche à justifier que ce qui suivra n'ait pas une optimisation prématurée. À mon sens, l'accumulation des faits étant: la stabilité de l'algorithme, diverses preuves du fonctionnement de l'implémentation, la validation des auteurs c'est-à-dire moi-même est le temps que je souhaite y consacrer; nous éloignons d'un contexte prématuré. Chaque élément listé précédemment était indispensable à cette condition.

Concentrons-nous à présent sur les possibles améliorations en nous intéressant au hardware. Nous savons que la plupart des processeurs que nous possédons ont la faculté d'exécuter plusieurs lectures ou écritures simultanément, tant que ces opérations n'opèrent pas dans des régions trop proches. Nous ne nous a tardé pas sur ce fait car il a déjà été abordé dans une précédente étude.

Les premiers éléments à optimiser sont les boucles. En effet, compter peut-être réalisé parallèlement par le processeur. J'entends par la que le parrallèlisme est différent du multithreadé. Nous profitons, au lieu de celà, des capacités des processeurs à faire du `out-of-order`. La fonction sur laquelle nous nous pencherons compte le nombre de caractères identique à partir de deux indexes dans une source.

Pour éviter la duplication de code entre une version optimisée et une version originale de l'algorithme, je définirai le trait suivant dont je préciserais les implémentations dans des structures dédiées uniquement à la fonction `while_equal`.

Des accès publiques sont définis comme suit.

```rust
use final_state_rs::lzss::*;

let src = "ABCDFGHABCDEFGHI".as_bytes();
println!("src: {:?}", src);
let len1 = while_equal_fast(src, 0, 7);
let len2 = while_equal(src, 0, 7);
assert_eq!(len1, len2);

let src = "ABCDABCDEFGHI".as_bytes();
let len1 = while_equal_fast(src, 0, 4);
let len2 = while_equal(src, 0, 4);
assert_eq!(len1, len2);
```


La fonction `while_equal` prend comme arguments une source et deux indexes. Elle calculera le nombre de carractères identiques à partir de ces deux indexes dans la limite suivante min(index - from, src.len - index).

```rust
fn while_equal(src: &[u8], from: usize, index: usize) -> u32;
```

Dans notre context présent, il est impératif que le premier indexe soit inférieur au second, et ces deux index doivent être inférieur à la taille de la source. De plus, j'ai choisi arbitrairement d'appeler cette fonction uniquement lorsque je constate que deux éléments dans la source, à la position `from` et `index`, sont égaux. Il convient donc de vérifier s'il sont bien égaux avant de poursuivre la procédure.

La fonction est triviale et je doute qu'il faille s'attarder plus longtemps dessus.

```rust
fn while_equal(src: &[u8], from: usize, index: usize) -> u32 {
    assert!(from < index);
    assert!(index < src.len());
    assert_eq!(src[from], src[index]);

    let mut s = from + 1;
    let mut i = index + 1;

    Loop while equals
    while s < index && i < src.len() && src[s] == src[i] {
        s += 1;
        i += 1;
    }
    (s - from) as u32
}
```

Dans le code précédant, la boucle de test peut tout à fait être divisé en 4. Ce que nous allons faire, en prenant soint de vérifier la consistance entre l'implémentation triviale et rapide à l'aide d'un jeu de tests. Nous pourrons vérifier si nous obtenons oui ou non de meilleures performances par la suite.


Cette implémentation est simétrique à sa version original et optimisée pour les processeurs OoO.

Pour tenter de prouver une telle simétrie, il est important de définir des tests tels que ceux présents dans la figure suivante. Cependant nous pourrions nous en convaincre en parcourant le code attentivement. Premièrement, nous avons modifié le pas de la boucle principale initiallement de 1 à 4. Chaque test de i à i + 3 sont réalisés en utilisant de nouvelles variables locales. Un processeur OoO peut ainsi procéder parrallèlement chaque test. L'execution parrallèle s'arrête au moment de l'écriture de s, car cette opération doit respecter un ordre définis par le processeur lorsqu'il est sur un seul thread. De plus, l'union peut aussi se faire lors du break, car le branchement respecte les mêmes conditions que la variable s dans ce contexte.

Dans un deuxieme temps, nous comptons les derniers caractères oublié dans les intervals [s - 4, index]  [i - 4, src.len()]. Ces derniers caractère ne pouvant pas être divisé en 4. Ensuite, ce serait une erreur de tenter de diviser en 3, ou 2 ces tests, l'ajout de branchement serait trop couteux par rapport au gain.

```rust
fn while_equal(src: &[u8], from: usize, index: usize) -> u32 {
    assert!(from < index);
    assert!(index < src.len());
    assert_eq!(src[from], src[index]);

    let mut s = from + 1;
    let mut i = index + 1;

    Split in 4 the tests, each block will be done in parrallel by an OoO
    processor.
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

    Fix the last bytes unchecked
    while s < index && i < src.len() && src[s] == src[i] {
        s += 1;
        i += 1;
    }

    (s - from) as u32
}
```

À ce stade de notre progression, je considère important de préciser que de tels améliorations sont extrèmement dépendante du système dans lequel elles sont appliqué. Par exemple, si mon processeur est faible, très demandé par d'autres procéssus, il est tout à fait probable que les deux algorithmes aient des performances plus ou moins identiques. Sur des petites longueures, l'algorithme que nous considérions comme rapide pourrait même devenir lent. C'est à ce moment qu'interviennent quelques heuristiques égocentriques que je n'incluerai pas dans directement dans cette bibliothèque.

Les résultats sont acceptables et concorde avec nos déductions. Nous pouvons à présent réaliser une nouvelle implémentation en tout point symétrique utilisant cette nouvelle méthode. En quelques tests de vitesses nous montreront un gain. Il semble également qu'essayer d'obtimiser plus encore avec cette strategie serait contre productif. Je fermerai donc là le sujet OoO.

Cependant, l'instabilité de ce gain me laisse encore dans le doute. De plus, les performances restent très proche l'une de l'autre. Pour aller plus loin, je souhaiterai proposer une nouvelle approche plus ou moins similaire.


Ici, nous utiliserons l'arithmetique des pointeurs pour nous déplacer sur la source. Cette opération est considérée à raison comme étant `unsafe` par Rust. Mais un développeur aguéris constatera que toute lecture de la mémoire sera faite après des tests qui validerons si la zone est occupée par un élément que nous recherchons.

Nous transformerons un pointeur sur octet en un pointeur sur 32 ou 64 bits selon l'architecture dont dispose l'utilisateur. Nous pourrons dont tester, non pas 4 bytes simultanément, mais 8, dans le meilleur des cas. Et ceci sans parraléliser astucieusement notre code.

```rust
fn while_equal(src: &[u8], from: usize, index: usize) -> u32 {
    assert!(from < index);
    assert!(index < src.len());
    assert!(src.len() > BYTES_LEN + 1);
    assert_eq!(src[from], src[index]);

    let mut s = from + 1;
    let mut i = index + 1;

    Nous récupérons le nombre d'octets pour chaque étape.
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

    Fix the last bytes unchecked
    while s < index && i < src.len() && src[s] == src[i] {
        s += 1;
        i += 1;
    }

    (s - from) as u32
}
```

Avant de passer à la suite, deffinissons des accès à nos fonction et remarquons les différences de performance. Il semble que `encode_lzw_no_windows_u8_faster` est 25% plus rapide sur ma machine.

Do the same thing as `encode_lzw_no_windows_u8` but use `while_equal_fast` which is optimized for OoO processor.

```rust
pub fn encode_lzw_no_windows_u8_fast(src: &[u8]) -> Vec<u8> {
    internal_encode_lzw_no_windows_u8::<Fast>(src)
}

/// Do the same thing as `encode_lzw_no_windows_u8` but use `while_equal_faster`
/// which has a better optimization.
pub fn encode_lzw_no_windows_u8_faster(src: &[u8]) -> Vec<u8> {
    internal_encode_lzw_no_windows_u8::<Faster>(src)
}
```

D'autre optimisations, plus spécifiques à nos architectures peuvent être possible et je me réserve un temps pour les étudier plus tard.

Nous pouvons maintenant passer à la suite de notre chapitre qui est celle de l'implémentation de lzss. Je vous prie de pardonner mon approximation de lzw précédemment, car ce n'est pas exactement l'algorithme qui peut être décrit dans d'autres oeuvres. En effet, certaines caractéristiques telles que la comparaison des tailles de la sous-séquence et de du pair index-taille, ainsi que l'écriture de cette paire avec un masque d'un bit sur le bit le plus élevé, sont déjà les différences notables que l'on peut trouver entre lz77 et lzss. En réalité, il ne nous manque plus qu'implémenter le concept de fenêtre glissante.

Les différents algorithmes dérivant de lzw ont en commun qu'ils cherchent à réduire le temps de calcul en diminuant la complexité temporelle de son parent. Pour cela, ils usent de plusieurs techniques étant soit coûteuses en mémoire, soit coûteuse en tant que résultat final. Dans le cas de lzss, c'est en approximant le résultat que nous réussissons à rendre la complexité quadratique linéaire. L'approximation dégradant le résultat final, la sortie compressée de lzss sera nécessairement de taille supérieure ou égale à celle de lzw.

```rust
/// LZSS variation of LZW algorithm with a windows size.
pub fn encode_lzss_u8(src: &[u8], windows_size: usize) -> Vec<u8> {
    internal_encode_lzss_u8::<Original>(src, windows_size)
}

fn internal_encode_lzss_u8<T: WhileEqual>(src: &[u8], windows_size: usize) -> Vec<u8> {
    assert!(windows_size < src.len());

    // On peut découper le calcule de la sortie en 2 algorithmes. La première
    // partie pour les indexes <= à windows_size, et la deuxième pour les
    // indexes >=. Ce découpage nous permet d'éviter les branchements de
    // vérification quand windows_size < index.
    let mut ret = internal_encode_lzw_no_windows_u8::<T>(&src[..=windows_size]);

    let mut index = windows_size + 1;
    while index < src.len() - 4 {
        let mut s = index - windows_size;
        let mut repetition = Pair::default();

        Recherche de la plus longue séquence.
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
            Je n'ai trouvé aucune répétition,
            donc j'écris le symbole et j'avance de 1.
            ret.push(src[index]);
            index += 1;
        } else {
            J'ai trouvé une répétition, j'avance de la
            taille de celle-ci

            Construit la paire taille-index sur 32 bits
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
```

```rust
pub fn decode_lzw_u8(src: &[u8]) -> Vec<u8> {
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
```