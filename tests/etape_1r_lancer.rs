//! Étape 1, partie 1R : `lancer` avec un réseau (directives `nourriture`, `debit`, `loi`, `pas`) : l'empreinte
//! (`PHYSAR02`), les mesures, l'aperçu, les erreurs de directives, le calcul qui s'arrête, le binaire, et les anciens
//! scénarios qui ne bougent pas. Valeurs attendues : `references/etape-1r-valeurs.md`, identifiants `V-LRES-xx`.
//!
//! L'empreinte est vérifiée d'abord **à la main** : l'écriture canonique de deux petits cas écrite octet par octet
//! d'après la doc de `lancer` et de `graphe`, son SHA-256 calculé par le modèle Python `ref_reseaux.py` (écrit d'après
//! la seule doc, sans lien avec le code Rust : mêmes opérations `f64` dans le même ordre, donc mêmes bits) ; puis sur
//! d'autres scénarios dont l'état a été simulé par ce modèle. Seul `regression_reference_reseau_graine_1` compare à un
//! fichier produit par notre code.

mod common;

use std::fs;
use std::process::Command;

use physarum_lab::alea::Alea;
use physarum_lab::empreinte::sha256_hex;
use physarum_lab::graphe::{Grille, Voisinage};
use physarum_lab::lancer::{ErreurLancer, Sortie, lancer};
use physarum_lab::mesures::tubes_vifs;
use physarum_lab::reseaux::{Loi, NOURRITURE, Reseau, SEUIL_VIF};
use physarum_lab::scenario::lire_scenario;

const RACINE: &str = env!("CARGO_MANIFEST_DIR");

/// Un cas à murs : un mur vertical qui laisse passer par le bas, une cellule de poche (la cellule `(8, 6)`, enfermée
/// par les murs `(7, 6)` et `(8, 5)`) qui disparaît, quatre nourritures.
const MURS: &str = "grille 9 7\nvoisinage 4\nmur 4 0 4 4 0\nmur 7 6 7 6 0\nmur 8 5 8 5 0\n\
nourriture 1 1 0\nnourriture 7 1 0\nnourriture 1 6 0\nnourriture 6 3 0\ndebit 1.5\nloi sigmoide\npas 60 0.2\n";

/// Un cas à deux étages : un lien vertical de longueur 0,5 et de coût 2, l'étage 1 deux fois moins cher, un mur sur
/// l'étage 1, trois nourritures sur les deux étages.
const ETAGES: &str = "grille 6 4\nvoisinage 8\netages 2 0.5 2.0\ncout_etage 1 0.5\nmur 2 1 3 2 1\nnourriture 0 0 0\n\
nourriture 5 3 1\nnourriture 5 0 0\ndebit 2\nloi sigmoide\npas 30 0.15\n";

/// Un cas à seize nourritures (le maximum), loi linéaire.
fn seize() -> String {
    let positions = [
        (0, 0),
        (2, 0),
        (4, 0),
        (6, 0),
        (1, 2),
        (3, 2),
        (5, 2),
        (7, 2),
        (0, 4),
        (2, 4),
        (4, 4),
        (6, 4),
        (1, 6),
        (3, 6),
        (5, 6),
        (7, 6),
    ];
    let mut texte = String::from("grille 8 8\n");
    for (x, y) in positions {
        texte.push_str(&format!("nourriture {x} {y} 0\n"));
    }
    texte.push_str("debit 0.5\nloi lineaire\npas 25 0.3\n");
    texte
}

fn lancer_texte(texte: &str, graine: u64) -> Sortie {
    let scenario = lire_scenario(texte).expect("scénario valide");
    lancer(&scenario, graine).unwrap_or_else(|e| panic!("lancer {texte:?} : {e:?}"))
}

fn erreur_de(texte: &str) -> Option<ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario lisible"), 1).err()
}

fn fichier(nom: &str) -> String {
    fs::read_to_string(format!("{RACINE}/scenarios/{nom}")).unwrap_or_else(|e| panic!("scenarios/{nom} : {e}"))
}

/// Un `f64` en huit octets petit-boutistes.
fn f(x: f64) -> [u8; 8] {
    x.to_bits().to_le_bytes()
}

/// L'écriture canonique du graphe d'une grille de `largeur` colonnes sur une ligne, à un étage, voisinage 4, arêtes de
/// longueur et de coût 1 (doc de `graphe`), écrite ici octet par octet.
fn graphe_ligne(largeur: u32) -> Vec<u8> {
    let mut o = largeur.to_le_bytes().to_vec();
    for x in 0..largeur as i32 {
        o.extend_from_slice(&x.to_le_bytes());
        o.extend_from_slice(&0_i32.to_le_bytes());
        o.extend_from_slice(&0_u32.to_le_bytes());
    }
    o.extend_from_slice(&(largeur - 1).to_le_bytes());
    for a in 0..largeur - 1 {
        o.extend_from_slice(&a.to_le_bytes());
        o.extend_from_slice(&(a + 1).to_le_bytes());
        o.extend_from_slice(&f(1.0));
        o.extend_from_slice(&f(1.0));
    }
    o
}

/// L'état de `PHYSAR02` : l'étiquette, le graphe, `k` et les nœuds de nourriture par rang, les bits du débit, l'octet
/// de la loi, le nombre de pas, les bits de `h`, les bits de chaque conductivité finale, l'état du générateur.
#[allow(clippy::too_many_arguments)]
fn etat(
    graphe: Vec<u8>,
    foods: &[u32],
    debit: f64,
    loi: u8,
    nombre: u32,
    h: f64,
    d_bits: &[u64],
    alea: [u8; 16],
) -> Vec<u8> {
    let mut o = b"PHYSAR02".to_vec();
    o.extend_from_slice(&graphe);
    o.extend_from_slice(&(foods.len() as u32).to_le_bytes());
    for v in foods {
        o.extend_from_slice(&v.to_le_bytes());
    }
    o.extend_from_slice(&f(debit));
    o.push(loi);
    o.extend_from_slice(&nombre.to_le_bytes());
    o.extend_from_slice(&f(h));
    for d in d_bits {
        o.extend_from_slice(&d.to_le_bytes());
    }
    o.extend_from_slice(&alea);
    o
}

fn hex_vers_octets(hex: &str) -> [u8; 16] {
    let mut o = [0_u8; 16];
    for (i, octet) in o.iter_mut().enumerate() {
        *octet = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap();
    }
    o
}

const PETIT_A: &str = "grille 3 1\nnourriture 0 0 0\nnourriture 2 0 0\ndebit 1.0\nloi lineaire\npas 1 0.5\n";
const PETIT_B: &str =
    "grille 3 1\nnourriture 2 0 0\nnourriture 0 0 0\nnourriture 1 0 0\ndebit 2.0\nloi sigmoide\npas 2 0.25\n";

/// juste : l'empreinte d'un petit cas **linéaire**, écrite octet par octet (V-LRES-01). `grille 3 1`, nourritures
/// `(0,0)` et `(2,0)`, `debit 1.0`, `loi lineaire`, `pas 1 0.5`, graine 7. Le graphe a trois nœuds et deux arêtes ; les
/// nourritures sont les nœuds 0 et 2 (rangs 0 et 1) ; les quatre premiers tirages donnent les deux conductivités de
/// départ, les deux suivants la paire (`(0, 1)` : la source est la nourriture de rang 0) ; chaque tube porte le débit
/// 1, donc `D <- D + 0,5 (1 - D)`. Les deux conductivités finales, du modèle Python, sont `3FED22A5CE1B6194` et
/// `3FE8CC5E27EDBB33` ; l'état du générateur après six tirages est `db230361d5324820 6d00000000000000` ; l'état fait `L
/// + 49 + 4 k + 8 m = 92 + 49 + 8 + 16 = 165` octets ; le SHA-256 (Python `hashlib`) est `1a05ce67…61cd`.
///
/// Ne couvre pas : le contenu de `mesures` et de `apercu` (voir `mesures_en_csv` et `l_apercu_d_un_reseau`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn empreinte_d_un_petit_cas_lineaire_a_la_main() {
    let d = [0x3FED_22A5_CE1B_6194, 0x3FE8_CC5E_27ED_BB33];
    let alea = hex_vers_octets("db230361d53248206d00000000000000");
    let octets = etat(graphe_ligne(3), &[0, 2], 1.0, 0, 1, 0.5, &d, alea);
    assert_eq!(octets.len(), 165);
    assert_eq!(sha256_hex(&octets), "1a05ce67b1f882c5001649a42ee2ec545e80917c5cf856d85f2b98c7d19a61cd");
    assert_eq!(lancer_texte(PETIT_A, 7).empreinte, sha256_hex(&octets));
    // les directives dans un autre ordre, des commentaires : le même état
    let autre =
        "pas 1 0.5\n# un commentaire\nloi lineaire\nnourriture 0 0 0\ndebit 1.0\ngrille 3 1\nnourriture 2 0 0\n";
    assert_eq!(lancer_texte(autre, 7).empreinte, sha256_hex(&octets));
    // l'étiquette PHYSAR01 donnerait une autre empreinte ; la loi (un octet) compte, le débit, le rang aussi
    let mut faux = octets.clone();
    faux[7] = b'1';
    assert_ne!(sha256_hex(&faux), sha256_hex(&octets));
    for (texte, graine) in [
        (PETIT_A.replace("loi lineaire", "loi sigmoide"), 7),
        (PETIT_A.replace("debit 1.0", "debit 1.5"), 7),
        (PETIT_A.replace("pas 1 0.5", "pas 1 0.25"), 7),
        (PETIT_A.replace("pas 1 0.5", "pas 2 0.5"), 7),
        (PETIT_A.to_string(), 8),
        (PETIT_A.replace("nourriture 0 0 0\nnourriture 2 0 0", "nourriture 2 0 0\nnourriture 0 0 0"), 7),
    ] {
        assert_ne!(lancer_texte(&texte, graine).empreinte, sha256_hex(&octets), "{texte:?}");
    }
}

/// juste : l'empreinte d'un petit cas **sigmoïde à trois nourritures**, écrite octet par octet (V-LRES-02). `grille 3
/// 1`, nourritures `(2,0)`, `(0,0)`, `(1,0)` (les **rangs** 0, 1, 2 sont les nœuds 2, 0, 1 : l'état écrit les nœuds par
/// rang), `debit 2.0`, `loi sigmoide`, `pas 2 0.25`, graine 3 : deux pas tirent deux paires parmi trois (`borne(3)`
/// puis `borne(2)`) ; les deux conductivités finales, du modèle Python, sont `3FE80F280BCBFFB4` et `3FE535D5F1BF8F69` ;
/// l'état du générateur `b550126546a0931b 6d00000000000000` ; l'état fait `92 + 49 + 12 + 16 = 169` octets ; le SHA-256
/// est `d8973396…48dd`. L'octet de la loi vaut 1.
///
/// Ne couvre pas : les murs et les étages (`empreintes_de_scenarios_de_reseau`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn empreinte_d_un_petit_cas_sigmoide_a_la_main() {
    let d = [0x3FE8_0F28_0BCB_FFB4, 0x3FE5_35D5_F1BF_8F69];
    let alea = hex_vers_octets("b550126546a0931b6d00000000000000");
    let octets = etat(graphe_ligne(3), &[2, 0, 1], 2.0, 1, 2, 0.25, &d, alea);
    assert_eq!(octets.len(), 169);
    assert_eq!(sha256_hex(&octets), "d897339650400f87c60d963a57a9d342db38f25e411f547cc5be903e955dd148");
    assert_eq!(lancer_texte(PETIT_B, 3).empreinte, sha256_hex(&octets));
    // les nœuds par indice de nœud trié au lieu de par rang : une autre empreinte
    let trie =
        etat(graphe_ligne(3), &[0, 1, 2], 2.0, 1, 2, 0.25, &d, hex_vers_octets("b550126546a0931b6d00000000000000"));
    assert_ne!(sha256_hex(&trie), sha256_hex(&octets));
}

/// juste : l'empreinte de scénarios de réseau simulés par le modèle Python `ref_reseaux.py` (V-LRES-03) :
/// `scenarios/reseau.txt` (grille 12 × 12 à huit voisins, six nourritures, `debit 3`, sigmoïde, 400 pas à `h = 0,1`)
/// pour les graines 1 (`70eee2fd…713a`) et 2 (`c444a911…660b`) ; le même à 20 pas (`6b8c38a0…295d`), à 100 pas
/// (`6c821086… 923b`) et à 0 pas (`4f3a6be5…dd9e` : les conductivités de départ seules) ; avec `loi lineaire`
/// (`072eb53b…0946`), `debit 1` (`e209f6e9…7447`), `h = 0,11` (`d8af07ce…3ef0`), les deux premières nourritures
/// échangées (`ac8c6eed…5f28`) ; un cas à murs (`MURS`, graine 6, `d62d813c…74b0` : 55 nœuds, la poche et les murs
/// retirés) ; un cas à deux étages avec un coût d'étage et un mur (`ETAGES`, graine 4, `fb01ce16…c83b`) ; seize
/// nourritures en loi linéaire (graine 5, `27039c92…6581`) ; les débits extrêmes `20` (`c8581427…a1d3`) et `0.05`
/// (`494bd97c…054e`) sur deux nourritures. L'ordre des directives, les commentaires et un nom de fichier différent ne
/// changent pas l'empreinte ; chaque paramètre la change.
///
/// Ne couvre pas : Linux contre Windows (c'est la CI qui rejoue l'empreinte de référence).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn empreintes_de_scenarios_de_reseau() {
    let reseau = fichier("reseau.txt");
    let cas: Vec<(String, u64, &str)> = vec![
        (reseau.clone(), 1, "70eee2fd1df3835e54ebd2fde05600c3a478d1286ed8766957c9d35cef47713a"),
        (reseau.clone(), 2, "c444a911d943a0ee2108ecce81f0f072a927607d93c28e47384833b3d5fa660b"),
        (
            reseau.replace("pas 400 0.1", "pas 20 0.1"),
            1,
            "6b8c38a0a070c44b24fd422d57b1413200e05860cf4fbc9c42be81654a29295d",
        ),
        (
            reseau.replace("pas 400 0.1", "pas 100 0.1"),
            1,
            "6c8210862c5d2f25c2befb6ebe7967eea82032274319755110a8efc18b12923b",
        ),
        (
            reseau.replace("pas 400 0.1", "pas 0 0.1"),
            1,
            "4f3a6be5e4e054d5ab273dff1c40094f597e8d17708f7b7b36d8198b03a5dd9e",
        ),
        (
            reseau.replace("loi sigmoide", "loi lineaire"),
            1,
            "072eb53b54be4e11342039c07b8d6ef1eb90503409d0802161ccb26268a80946",
        ),
        (reseau.replace("debit 3", "debit 1"), 1, "e209f6e905686fa6a192486fd4ab32c8fdeed0f2f068e581765dc872eb397447"),
        (
            reseau.replace("pas 400 0.1", "pas 400 0.11"),
            1,
            "d8af07cec06f65d55870a874db0cb777179e845f10e905daa0ef24563bd13ef0",
        ),
        (
            reseau.replace("nourriture 1 1 0\nnourriture 10 2 0", "nourriture 10 2 0\nnourriture 1 1 0"),
            1,
            "ac8c6eed3a167ce2aab0a7a42aa0ef9120a5df06f19a3bf99acc0eff86b55f28",
        ),
        (MURS.to_string(), 6, "d62d813c5da5be071ce7651f3707d6e39923139fbb8379eee1dc430295d274b0"),
        (ETAGES.to_string(), 4, "fb01ce16b5479b9b2795da4419ad960822e73afeaffd4241ca41312c7d52c83b"),
        (seize(), 5, "27039c92164545e69f695e230c7599c70059220a437fc48d5315aa96c2006581"),
        (
            "grille 4 4\nnourriture 0 0 0\nnourriture 3 3 0\ndebit 20\nloi sigmoide\npas 10 0.5\n".to_string(),
            2,
            "c8581427a4f0b968adebf6b93767410512b5b6437c97b6b89c90a280846fd1a3",
        ),
        (
            "grille 4 4\nnourriture 0 0 0\nnourriture 3 3 0\ndebit 0.05\nloi sigmoide\npas 10 0.5\n".to_string(),
            2,
            "494bd97c858f5bbb1aa4c7e47b1a1862301a2ca7e6054303eed33fb6caec054e",
        ),
    ];
    for (texte, graine, attendu) in &cas {
        assert_eq!(
            &lancer_texte(texte, *graine).empreinte,
            attendu,
            "graine {graine}, {}",
            &texte[..40.min(texte.len())]
        );
    }
    // l'ordre des directives, des commentaires, des lignes vides : le même état que `MURS` (les nourritures gardent
    // leur ordre : c'est le rang)
    let autres: Vec<&str> = MURS.lines().filter(|l| !l.starts_with("nourriture")).collect();
    let foods: Vec<&str> = MURS.lines().filter(|l| l.starts_with("nourriture")).collect();
    let mut lignes: Vec<&str> = autres.iter().rev().take(4).copied().collect();
    lignes.extend(&foods);
    lignes.extend(autres.iter().rev().skip(4));
    let desordre = lignes.join("\n\n# un commentaire\n") + "\n";
    assert_eq!(lancer_texte(&desordre, 6).empreinte, cas[9].2);
    // un mur donné en deux rectangles, un mur en double : le même état
    let coupe = MURS.replace("mur 4 0 4 4 0", "mur 4 0 4 2 0\nmur 4 2 4 4 0\nmur 4 1 4 3 0");
    assert_eq!(lancer_texte(&coupe, 6).empreinte, cas[9].2);
    // chaque changement change l'empreinte
    for (texte, graine) in [
        (MURS.replace("mur 7 6 7 6 0\n", ""), 6),
        (MURS.replace("debit 1.5", "debit 1.6"), 6),
        (MURS.replace("nourriture 6 3 0", "nourriture 6 4 0"), 6),
        (MURS.to_string(), 7),
        (ETAGES.replace("cout_etage 1 0.5", "cout_etage 1 0.51"), 4),
    ] {
        let attendu = if texte.contains("etages") { cas[10].2 } else { cas[9].2 };
        assert_ne!(lancer_texte(&texte, graine).empreinte, attendu, "{texte:?}");
    }
}

/// régression : pour `scenarios/reseau.txt` et la graine 1, l'empreinte rendue par `lancer` est celle du fichier
/// `references/reseau-graine-1.empreinte` (l'empreinte puis un saut de ligne), que la CI rejoue sous Linux et sous
/// Windows ; le fichier est produit par le binaire, et sa valeur est celle du modèle Python (V-LRES-04).
///
/// Ne couvre pas : que l'empreinte soit juste (`empreintes_de_scenarios_de_reseau` la compare au modèle Python).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn regression_reference_reseau_graine_1() {
    let reference = fs::read_to_string(format!("{RACINE}/references/reseau-graine-1.empreinte")).unwrap();
    assert_eq!(reference.len(), 65);
    assert!(reference.ends_with('\n'));
    assert_eq!(lancer_texte(&fichier("reseau.txt"), 1).empreinte, reference.trim_end());
}

/// juste : les mesures en CSV d'un réseau (V-LRES-05) : `temps,source,puits,pression,vifs,longueur`, une ligne par pas,
/// les nombres du modèle Python. Les deux petits cas en entier : `PETIT_A` (`0,0,1,3.036618473,2,2.000000` : au pas 0,
/// la paire est (rang 0, rang 1), la pression de la source est `1/D1 + 1/D2` pour `I0 = 1`, deux tubes vifs de longueur
/// 2 au total) ; `PETIT_B` (`0,2,0,3.596297331,2,2.000000` puis `1,0,1,5.959117801,2,2.000000` : les rangs tirés, la
/// pression de la source à chaque pas) ; zéro pas : l'en-tête seul. Pour les autres : le nombre de lignes, la première,
/// la dernière et le SHA-256 du texte : `reseau.txt` (400 lignes, la première `0,3,0,3.447662552,506,606.239682` : les
/// 506 arêtes de la grille sont vives au départ, de longueur totale 606,239682 ; la dernière `399,4,0,129.723993117,46,
/// 50.556349` : 46 tubes vifs), `MURS` (le départ à 89 arêtes, de longueur 89 : le graphe du plan), `ETAGES` (130 tubes
/// vifs pour 139,053824 de longueur), `seize`. Dans chaque ligne, source et puits sont deux rangs différents de `0..k`
/// ; le seuil des tubes vifs est `SEUIL_VIF = 0,001`.
///
/// Ne couvre pas : le seuil à sa limite (`etape_1r_mesures`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn mesures_en_csv() {
    let entete = "temps,source,puits,pression,vifs,longueur\n";
    assert_eq!(lancer_texte(PETIT_A, 7).mesures, format!("{entete}0,0,1,3.036618473,2,2.000000\n"));
    assert_eq!(
        lancer_texte(PETIT_B, 3).mesures,
        format!("{entete}0,2,0,3.596297331,2,2.000000\n1,0,1,5.959117801,2,2.000000\n")
    );
    assert_eq!(lancer_texte(&PETIT_A.replace("pas 1 0.5", "pas 0 0.5"), 7).mesures, entete);
    assert_eq!(SEUIL_VIF, 0.001);
    #[allow(clippy::type_complexity)]
    let cas: [(String, u64, usize, usize, &str, &str, &str); 4] = [
        (
            fichier("reseau.txt"),
            1,
            400,
            6,
            "0,3,0,3.447662552,506,606.239682",
            "399,4,0,129.723993117,46,50.556349",
            "2db91f181d435473db07b0e45ffc1119e6b2f533733a51a5c39734cbed8aa975",
        ),
        (
            MURS.to_string(),
            6,
            60,
            4,
            "0,0,3,8.479861188,89,89.000000",
            "59,2,0,17.449073725,16,16.000000",
            "564d140c94a38d188ebeb1247904f956cfe262ea9fe2bfe71cd4d8367fdd7cc1",
        ),
        (
            ETAGES.to_string(),
            4,
            30,
            3,
            "0,1,2,1.859387935,130,139.053824",
            "29,1,2,20.305575684,130,139.053824",
            "a16d36a47174bae3ec02277d1f1463edd67e5f5a5806abdca24ff21de4177ec9",
        ),
        (
            seize(),
            5,
            25,
            16,
            "0,2,13,0.978467153,112,112.000000",
            "24,5,1,12.905477176,112,112.000000",
            "1ea39f6a725b9025939d75b15c4b3e19918d65dec6c4c5624546fa5d42fbec8a",
        ),
    ];
    for (texte, graine, nb, k, premiere, derniere, sha) in &cas {
        let mesures = lancer_texte(texte, *graine).mesures;
        let lignes: Vec<&str> = mesures.lines().collect();
        assert_eq!(lignes.len(), nb + 1, "{graine}");
        assert!(mesures.ends_with('\n') && !mesures.contains('\r'));
        assert_eq!(lignes[0], entete.trim_end());
        assert_eq!(&lignes[1], premiere);
        assert_eq!(&lignes[*nb], derniere);
        assert_eq!(&sha256_hex(mesures.as_bytes()), sha);
        for (t, ligne) in lignes[1..].iter().enumerate() {
            let champs: Vec<&str> = ligne.split(',').collect();
            assert_eq!(champs.len(), 6, "{ligne}");
            assert_eq!(champs[0], t.to_string());
            let (i, j): (usize, usize) = (champs[1].parse().unwrap(), champs[2].parse().unwrap());
            assert!(i < *k && j < *k && i != j, "{ligne}");
        }
    }
}

/// juste : `lancer` est « `Reseau::au_hasard` puis `nombre` fois `Reseau::avancer`, avec le même générateur »
/// (V-LRES-06) : pour `scenarios/reseau.txt` et la graine 1, le test refait la simulation avec l'API (`Alea::new(1)`,
/// `Reseau::au_hasard`, 400 `avancer`), écrit les lignes de mesures d'après la doc de `lancer` et l'état `PHYSAR02`
/// avec `Graphe::octets` : le texte des mesures et l'empreinte sont ceux de `lancer`. Les conductivités finales ne sont
/// pas celles de départ.
///
/// Ne couvre pas : les murs (le graphe du plan, `etape_1l`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn lancer_est_au_hasard_puis_avancer() {
    let grille = Grille {
        largeur: 12,
        hauteur: 12,
        etages: 1,
        voisinage: Voisinage::Huit,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let g = grille.graphe().unwrap();
    let positions = [(1, 1), (10, 2), (5, 5), (2, 10), (10, 10), (6, 10)];
    let foods: Vec<usize> = positions.iter().map(|&(x, y)| grille.indice(x, y, 0)).collect();
    let mut alea = Alea::new(1);
    let mut reseau = Reseau::au_hasard(&g, &mut alea, foods.clone(), 3.0, Loi::Sigmoide).unwrap();
    let depart = reseau.conductivites().to_vec();
    let mut csv = String::from("temps,source,puits,pression,vifs,longueur\n");
    for t in 0..400 {
        let (vifs, longueur) = tubes_vifs(&g, reseau.conductivites(), 0.001);
        let pas = reseau.avancer(&g, &mut alea, 0.1).unwrap();
        let rang = |v: usize| foods.iter().position(|&x| x == v).unwrap();
        csv.push_str(&format!(
            "{t},{},{},{:.9},{vifs},{longueur:.6}\n",
            rang(pas.source),
            rang(pas.puits),
            pas.etat.pressions[pas.source]
        ));
    }
    assert_ne!(reseau.conductivites(), &depart[..]);
    let noeuds: Vec<u32> = foods.iter().map(|&v| v as u32).collect();
    let bits: Vec<u64> = reseau.conductivites().iter().map(|d| d.to_bits()).collect();
    let octets = etat(g.octets(), &noeuds, 3.0, 1, 400, 0.1, &bits, alea.octets());
    assert_eq!(octets.len(), g.octets().len() + 49 + 4 * 6 + 8 * g.nb_aretes());
    let sortie = lancer_texte(&fichier("reseau.txt"), 1);
    assert_eq!(sortie.mesures, csv);
    assert_eq!(sortie.empreinte, sha256_hex(&octets));
}

/// Les points du treillis d'une ligne, chacun un carré de `echelle` pixels.
fn ligne_de_points(points: &[[u8; 3]], echelle: usize) -> Vec<u8> {
    let mut rvb = Vec::new();
    for _ in 0..echelle {
        for p in points {
            for _ in 0..echelle {
                rvb.extend_from_slice(p);
            }
        }
    }
    rvb
}

/// juste : l'aperçu d'un réseau (V-LRES-07) : le PNG de `reseaux::apercu(graphe, conductivités, nourritures, f(I0),
/// echelle)` avec `f(I0) = loi.valeur(debit)` pour référence (1 pour `PETIT_A` linéaire, `0,8` pour `PETIT_B` sigmoïde
/// : `4 / 5`) et l'échelle `512 / TL` entre 1 et 8. (1) `PETIT_A` : `grille 3 1`, treillis de 5 × 1 points, échelle 8,
/// 40 × 8 pixels : `N`, le milieu `(56, 202, 234)`, le nœud `(56, 202, 234)`, le milieu `(49, 174, 203)`, `N` (`N` =
/// `NOURRITURE`) ; calculé à la main : `D = 0,9087` et `0,7749` donnent les niveaux 232 et 198 (`melange(FOND, VIF,
/// 232) = (56, 202, 234)`), le nœud 1 prend le plus fort de ses deux tubes (232). (2) `PETIT_B` : `N`, `(57, 208,
/// 241)`, `N`, `(52, 185, 215)`, `N` (le nœud 1 est une nourriture aussi). (3) `reseau.txt` (graine 1) : 184 × 184
/// pixels (TL = 23, échelle 8), SHA-256 des octets RVB `c3856831…cfbd9`, et les six carrés de 8 × 8 pixels des
/// nourritures sont `NOURRITURE` ; `MURS` : 136 × 104, SHA-256 `29d62fc8…a897` ; `ETAGES` : 192 × 56, `dc7a3622…f266`
/// (le cadre est celui des nœuds du plan, deux étages côte à côte).
///
/// Ne couvre pas : le dessin des tubes seul (`etape_1_apercu`, `etape_1r_apercu`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn l_apercu_d_un_reseau() {
    let a = common::relire_png(&lancer_texte(PETIT_A, 7).apercu).expect("PNG valide");
    assert_eq!((a.largeur, a.hauteur), (40, 8));
    let points_a = [NOURRITURE, [56, 202, 234], [56, 202, 234], [49, 174, 203], NOURRITURE];
    assert_eq!(a.rvb, ligne_de_points(&points_a, 8));
    let b = common::relire_png(&lancer_texte(PETIT_B, 3).apercu).expect("PNG valide");
    let points_b = [NOURRITURE, [57, 208, 241], NOURRITURE, [52, 185, 215], NOURRITURE];
    assert_eq!(b.rvb, ligne_de_points(&points_b, 8));
    let r = common::relire_png(&lancer_texte(&fichier("reseau.txt"), 1).apercu).expect("PNG valide");
    assert_eq!((r.largeur, r.hauteur), (184, 184));
    assert_eq!(sha256_hex(&r.rvb), "c38568312352918514d604540e7aca1d15e40151ac448c665a3f22381b8cfbd9");
    for (x, y) in [(1, 1), (10, 2), (5, 5), (2, 10), (10, 10), (6, 10)] {
        for py in 8 * 2 * y..8 * (2 * y + 1) {
            for px in 8 * 2 * x..8 * (2 * x + 1) {
                let i = 3 * (py * 184 + px);
                assert_eq!(&r.rvb[i..i + 3], &NOURRITURE, "nourriture ({x}, {y}), pixel ({px}, {py})");
            }
        }
    }
    let nb = r.rvb.chunks(3).filter(|p| *p == NOURRITURE).count();
    assert_eq!(nb, 6 * 64, "il n'y a pas d'autre pixel de cette couleur");
    let m = common::relire_png(&lancer_texte(MURS, 6).apercu).expect("PNG valide");
    assert_eq!((m.largeur, m.hauteur), (136, 104));
    assert_eq!(sha256_hex(&m.rvb), "29d62fc8e95f51e99125c338f1ac2bc66ca6e9ebdf6c90e68f757ed48226a897");
    let e = common::relire_png(&lancer_texte(ETAGES, 4).apercu).expect("PNG valide");
    assert_eq!((e.largeur, e.hauteur), (192, 56));
    assert_eq!(sha256_hex(&e.rvb), "dc7a36227d6aba62d2402f5c09c08061b1977e41f9cd06a403b47bd475f0f266");
}

/// juste : les erreurs des directives d'un réseau (V-LRES-08) sont `DirectiveInconnue(ligne)` avec la règle de priorité
/// de la doc de `lancer` (« Ce que fait l'étape 1, partie "les réseaux" ») : (1) la première directive FAUTIVE dans
/// l'ordre du fichier ; (2) un fichier qui a une directive de réseau et une `source` ou un `puits` : la ligne de la
/// **première** `source` ou `puits` ; (3) il manque `debit`, `loi`, `pas` ou des nourritures (moins de deux) : la
/// **première directive restante** ; (4) la 2e nourriture murée : sa ligne ; la 1re murée ou séparée de la 2e : la
/// ligne de la 1re ; la 2e d'abord quand les deux sont murées ; puis la première des autres nourritures qui n'est pas
/// dans le plan, avec sa propre ligne. Les bornes sont comprises : `debit 0.05`, `20`, `20.0` passent, `0.04` et
/// `20.01` non ; seize nourritures passent, dix-sept non (la dix-septième est fautive) ; `loi` n'a que deux mots
/// permis.
///
/// Ne couvre pas : les erreurs de lecture (`etape_0_scenario`), les erreurs des tubes (`etape_1_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn erreurs_de_directives() {
    let e = |ligne: usize| Some(ErreurLancer::DirectiveInconnue(ligne));
    // une grille 6 × 4 à deux étages ; la ligne 1 est `grille`, la ligne 2 `etages`, la première directive est ligne 3
    let entete = "grille 6 4\netages 2 1.0 1.0\n";
    let avec = |directives: &[&str]| format!("{entete}{}\n", directives.join("\n"));
    let base = ["nourriture 0 0 0", "nourriture 5 3 0", "debit 2", "loi sigmoide", "pas 2 0.5"];
    let remplace = |i: usize, par: &str| {
        let mut v = base.to_vec();
        v[i] = par;
        avec(&v)
    };
    let ajoute = |en_plus: &[&str]| {
        let mut v = base.to_vec();
        v.extend_from_slice(en_plus);
        avec(&v)
    };
    assert_eq!(erreur_de(&avec(&base)), None);
    // des bornes comprises, l'ordre des directives, des murs et des coûts d'étage
    for ok in ["debit 0.05", "debit 0.050", "debit 20", "debit 20.0", "debit 7.25"] {
        assert_eq!(erreur_de(&remplace(2, ok)), None, "{ok:?}");
    }
    assert_eq!(erreur_de(&remplace(3, "loi lineaire")), None);
    assert_eq!(erreur_de(&remplace(4, "pas 0 0.5")), None);
    assert_eq!(erreur_de(&remplace(4, "pas 100000 0.999999")), None);
    assert_eq!(erreur_de(&ajoute(&["mur 1 1 2 2 0", "cout_etage 1 0.5", "mur 3 0 3 0 1"])), None);
    assert_eq!(
        erreur_de(&avec(&["pas 2 0.5", "loi lineaire", "debit 1", "nourriture 5 3 1", "nourriture 0 0 1"])),
        None
    );
    // seize nourritures, dix-sept
    let seize: Vec<String> = (0..16).map(|i| format!("nourriture {} {} 0", i % 6, i / 6)).collect();
    let reste = ["debit 2", "loi sigmoide", "pas 2 0.5"];
    let texte = |foods: &[String]| avec(&foods.iter().map(String::as_str).chain(reste).collect::<Vec<_>>());
    assert_eq!(erreur_de(&texte(&seize)), None, "seize nourritures");
    let mut dix_sept = seize.clone();
    dix_sept.push("nourriture 4 2 0".to_string());
    assert_eq!(erreur_de(&texte(&dix_sept)), e(19), "la dix-septième nourriture (ligne 2 + 17)");
    assert_eq!(erreur_de(&texte(&seize[..2])), None, "deux nourritures");
    // chaque faute de `debit` (la ligne 5), de `loi` (6), de `nourriture` (4)
    for mauvais in [
        "debit 0.04",
        "debit 0.0499999",
        "debit 20.01",
        "debit 0",
        "debit -1",
        "debit 1e1",
        "debit .5",
        "debit abc",
        "debit",
        "debit 1 2",
        "debit +2",
        "debit nan",
        "debit inf",
        "debit 2.",
    ] {
        assert_eq!(erreur_de(&remplace(2, mauvais)), e(5), "{mauvais:?}");
    }
    for mauvais in [
        "loi",
        "loi linéaire",
        "loi Lineaire",
        "loi LINEAIRE",
        "loi sigmoïde",
        "loi Sigmoide",
        "loi lineaire sigmoide",
        "loi cubique",
        "loi 2",
        "loi sigmoide2",
    ] {
        assert_eq!(erreur_de(&remplace(3, mauvais)), e(6), "{mauvais:?}");
    }
    for mauvais in [
        "nourriture 0 0",
        "nourriture 0 0 0 0",
        "nourriture 6 0 0",
        "nourriture 0 4 0",
        "nourriture 0 0 2",
        "nourriture -1 0 0",
        "nourriture 1.0 0 0",
        "nourriture a 0 0",
        "nourriture",
        "nourriture 0 0 0",
    ] {
        assert_eq!(erreur_de(&remplace(1, mauvais)), e(4), "{mauvais:?} (le dernier : le même nœud que la première)");
    }
    // un second `debit`, une seconde `loi`, un second `pas`, même bons : la seconde ligne
    for double in ["debit 3", "loi lineaire", "pas 3 0.5"] {
        assert_eq!(erreur_de(&ajoute(&[double])), e(8), "{double:?}");
    }
    assert_eq!(erreur_de(&ajoute(&["inconnue"])), e(8));
    assert_eq!(erreur_de(&ajoute(&["mur 9 1 2 2 0"])), e(8));
    assert_eq!(erreur_de(&ajoute(&["cout_etage 5 1.0"])), e(8));
    for mauvais in ["pas 100001 0.5", "pas 2 1", "pas 2", "pas 2 0"] {
        assert_eq!(erreur_de(&remplace(4, mauvais)), e(7), "{mauvais:?}");
    }
    // (1) l'emporte : la première directive fautive, tous genres confondus
    let mut v = base.to_vec();
    v[2] = "debit 99";
    v.push("inconnue");
    assert_eq!(erreur_de(&avec(&v)), e(5));
    assert_eq!(
        erreur_de(&avec(&["pas 100001 0.5", "debit 99", "nourriture 0 0 0", "nourriture 5 3 0", "loi sigmoide"])),
        e(3)
    );
    // (2) les deux familles : la première `source` ou `puits`
    assert_eq!(erreur_de(&ajoute(&["source 0 0 0"])), e(8));
    assert_eq!(erreur_de(&ajoute(&["puits 1 1 0"])), e(8));
    assert_eq!(erreur_de(&ajoute(&["puits 1 1 0", "source 0 1 0"])), e(8));
    assert_eq!(
        erreur_de(&avec(&[
            "puits 1 1 0",
            "nourriture 0 0 0",
            "nourriture 5 3 0",
            "debit 2",
            "loi sigmoide",
            "pas 2 0.5"
        ])),
        e(3)
    );
    assert_eq!(erreur_de(&avec(&["nourriture 0 0 0", "source 0 0 0"])), e(4), "les deux familles avant ce qui manque");
    assert_eq!(erreur_de(&avec(&["source 0 0 0", "nourriture 0 0 0", "debit 99"])), e(5), "(1) avant (2)");
    assert_eq!(
        erreur_de(&avec(&["debit 2", "source 0 0 0", "puits 5 3 0", "pas 2 0.5"])),
        e(4),
        "un `debit` avec des tubes"
    );
    assert_eq!(erreur_de(&avec(&["loi lineaire", "puits 5 3 0", "source 0 0 0", "pas 2 0.5"])), e(4));
    // (3) ce qui manque : la première directive restante
    for manque in [2, 3, 4] {
        let mut v = base.to_vec();
        v.remove(manque);
        assert_eq!(erreur_de(&avec(&v)), e(3), "sans {}", base[manque]);
    }
    let mut une_seule = base.to_vec();
    une_seule.remove(1);
    assert_eq!(erreur_de(&avec(&une_seule)), e(3), "une seule nourriture");
    assert_eq!(erreur_de(&avec(&["debit 2", "loi sigmoide", "pas 2 0.5"])), e(3), "aucune nourriture");
    assert_eq!(
        erreur_de(&avec(&["# une note", "nourriture 0 0 0", "debit 2"])),
        e(4),
        "les commentaires ne sont pas des directives"
    );
    assert_eq!(erreur_de(&avec(&["loi lineaire"])), e(3));
    assert_eq!(
        erreur_de(&avec(&["mur 1 1 1 1 0", "nourriture 0 0 0", "nourriture 5 3 0", "debit 2", "loi sigmoide"])),
        e(3)
    );
    // (4) les murs, sur une grille à un seul étage : la 2e nourriture, la 1re, les deux, puis les suivantes
    let plat = |directives: &[&str]| format!("grille 6 4\n{}\n", directives.join("\n"));
    let sur = |foods: &[&str], murs: &[&str]| {
        let mut v: Vec<&str> = foods.to_vec();
        v.extend(["debit 2", "loi sigmoide", "pas 2 0.5"]);
        v.extend(murs);
        plat(&v)
    };
    let deux = ["nourriture 0 0 0", "nourriture 5 3 0"];
    assert_eq!(erreur_de(&sur(&deux, &[])), None);
    assert_eq!(erreur_de(&sur(&deux, &["mur 5 3 5 3 0"])), e(3), "la 2e nourriture est sous un mur");
    assert_eq!(erreur_de(&sur(&deux, &["mur 0 0 0 0 0"])), e(2), "la 1re");
    assert_eq!(erreur_de(&sur(&deux, &["mur 0 0 0 0 0", "mur 5 3 5 3 0"])), e(3), "la 2e d'abord");
    assert_eq!(erreur_de(&sur(&deux, &["mur 2 0 2 3 0"])), e(2), "la 1re est séparée de la 2e");
    let trois = ["nourriture 0 0 0", "nourriture 5 3 0", "nourriture 2 2 0"];
    assert_eq!(erreur_de(&sur(&trois, &[])), None);
    assert_eq!(erreur_de(&sur(&trois, &["mur 2 2 2 2 0"])), e(4), "la 3e est sous un mur");
    let poche = ["nourriture 0 0 0", "nourriture 5 3 0", "nourriture 5 0 0"];
    assert_eq!(erreur_de(&sur(&poche, &["mur 4 0 4 0 0", "mur 5 1 5 1 0"])), e(4), "la 3e est dans une poche");
    let coupe = ["nourriture 3 0 0", "nourriture 5 3 0", "nourriture 0 0 0"];
    assert_eq!(erreur_de(&sur(&coupe, &["mur 2 0 2 3 0"])), e(4), "la 3e est séparée de la 2e, pas la 1re");
    let quatre = ["nourriture 0 0 0", "nourriture 5 3 0", "nourriture 2 2 0", "nourriture 3 3 0"];
    assert_eq!(erreur_de(&sur(&quatre, &["mur 3 3 3 3 0"])), e(5), "la 4e");
    assert_eq!(erreur_de(&sur(&quatre, &["mur 2 2 2 2 0", "mur 3 3 3 3 0"])), e(4), "la 3e avant la 4e");
    // deux étages : un mur sur tout une colonne d'un étage laisse le passage par l'autre
    let deux_etages = |murs: &[&str]| {
        let mut v = vec!["nourriture 0 0 0", "nourriture 5 3 0", "debit 2", "loi sigmoide", "pas 2 0.5"];
        v.extend(murs);
        avec(&v)
    };
    assert_eq!(erreur_de(&deux_etages(&["mur 2 0 2 3 0"])), None);
    assert_eq!(
        erreur_de(&deux_etages(&["mur 2 0 2 3 0", "mur 2 0 2 3 1"])),
        e(3),
        "séparée sur les deux étages : la 1re"
    );
    // une nourriture sur le second étage
    assert_eq!(
        erreur_de(&avec(&["nourriture 0 0 1", "nourriture 5 3 1", "debit 2", "loi sigmoide", "pas 2 0.5"])),
        None
    );
}

/// juste : un calcul qui s'arrête (V-LRES-09) : `grille 3 1`, `etages 2 1.0 10000000000000000` (un lien entre étages
/// d'un coût de `10^16`), nourritures `(0,0,0)` et `(2,0,1)`, `h = 0,5`, graine 1 : le solveur s'arrête sur un pivot
/// nul au pas numéro 4 sous la loi linéaire (`debit 1`) et au pas numéro 3 sous la sigmoïde, d'après le modèle Python
/// (la procédure de la doc du solveur avec la vérification des pivots, écrite à part) ; avec un coût de `10^17`, au pas
/// 0. `lancer` rend `ErreurLancer::Calcul(t)`, rien d'autre. Avec `pas` égal au numéro du pas fautif, tout va bien (les
/// pas d'avant ne s'arrêtent pas) ; avec zéro pas, jamais d'erreur de calcul.
///
/// Ne couvre pas : les calculs qui se trompent sans s'arrêter (`MODELE.md`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn un_calcul_qui_s_arrete() {
    let texte = |cout: &str, loi: &str, pas: u32| {
        format!(
            "grille 3 1\netages 2 1.0 {cout}\nnourriture 0 0 0\nnourriture 2 0 1\ndebit 1\nloi {loi}\npas {pas} 0.5\n"
        )
    };
    let seize_chiffres = "10000000000000000";
    assert!(erreur_de(&texte(seize_chiffres, "lineaire", 4)).is_none(), "quatre pas (0 à 3) passent");
    assert_eq!(erreur_de(&texte(seize_chiffres, "lineaire", 5)), Some(ErreurLancer::Calcul(4)));
    assert_eq!(erreur_de(&texte(seize_chiffres, "lineaire", 40)), Some(ErreurLancer::Calcul(4)));
    assert!(erreur_de(&texte(seize_chiffres, "sigmoide", 3)).is_none(), "trois pas (0 à 2) passent");
    assert_eq!(erreur_de(&texte(seize_chiffres, "sigmoide", 4)), Some(ErreurLancer::Calcul(3)));
    assert_eq!(erreur_de(&texte("100000000000000000", "lineaire", 1)), Some(ErreurLancer::Calcul(0)));
    assert_eq!(erreur_de(&texte("100000000000000000", "sigmoide", 7)), Some(ErreurLancer::Calcul(0)));
    assert!(erreur_de(&texte("100000000000000000", "sigmoide", 0)).is_none(), "zéro pas : jamais d'erreur de calcul");
    assert!(erreur_de(&texte("2", "sigmoide", 40)).is_none());
}

/// juste : les anciens scénarios gardent leur empreinte (V-LRES-10) : l'étape 0 (`grille.txt`), les tubes du cœur
/// (`chemin.txt`, un petit cas à `PHYSAR01`), la partie 1L (`labyrinthe.txt` `dcdad079…3ded`, `etages.txt`
/// `d011d6bc…df76`) ; les directives des réseaux ne les touchent pas ; un scénario de tubes qui porte un `debit` est
/// une erreur (c'est un scénario de réseau avec une `source`) ; le dossier `scenarios/` contient `reseau.txt`, qui se
/// lit et se lance, avec un coût vertical dans `[0,1 ; 10]`.
///
/// Ne couvre pas : les scénarios que d'autres étapes ajouteront.
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn les_anciens_scenarios_gardent_leur_empreinte() {
    assert_eq!(
        lancer_texte(&fichier("chemin.txt"), 1).empreinte,
        "6e3b15b88963ed142ce3441c33b249b2164d13a51087662b8927f77ddd3c2f37"
    );
    let grille = fs::read_to_string(format!("{RACINE}/references/grille-graine-1.empreinte")).unwrap();
    assert_eq!(lancer_texte(&fichier("grille.txt"), 1).empreinte, grille.trim_end());
    assert_eq!(
        lancer_texte(&fichier("labyrinthe.txt"), 1).empreinte,
        "dcdad07950dc6dbb0aaa141bc1bb7fe7c0bdbe309a1f4d75b0cbf2dcba213ded"
    );
    assert_eq!(
        lancer_texte(&fichier("etages.txt"), 1).empreinte,
        "d011d6bc3988a72730aefba5b8c90ac5f460cd8c65c0f3ece05564acd218df76"
    );
    assert_eq!(
        lancer_texte("grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.5\n", 9).empreinte,
        "ee991bd8ca038070aee07a55ce6bdb9fb2e58772d6f128fd66293f4ab854bed0"
    );
    assert_eq!(
        lancer_texte("grille 3 3\nvoisinage 8\netages 2 0.5 2.0\nsource 0 0 0\npuits 2 2 1\npas 20 0.25\n", 3)
            .empreinte,
        "77acbb3c46fd00f2722356e25036c2df616a16af6c108c058ff5c95aedd27362"
    );
    assert_eq!(
        erreur_de("grille 4 4\nsource 3 0 0\npuits 0 3 0\ndebit 1\npas 30 0.5\n"),
        Some(ErreurLancer::DirectiveInconnue(2)),
        "un `debit` avec une `source` : les deux familles"
    );
    // les directives de l'étape 0 seule restent refusées quand elles ne sont pas connues
    assert_eq!(erreur_de("grille 3 3\ninconnue\n"), Some(ErreurLancer::DirectiveInconnue(2)));
    let scenario = lire_scenario(&fichier("reseau.txt")).unwrap();
    let cout = scenario.grille.cout_vertical;
    assert!((0.1..=10.0).contains(&cout));
    assert!(lancer(&scenario, 1).is_ok());
}

/// juste : le binaire avec un réseau (V-LRES-11) : `physarum lancer scenarios/reseau.txt --graine 1 --sortie <dossier>`
/// rend le code 0 et écrit `empreinte.txt` (`70eee2fd…713a` puis `\n`), `mesures.csv` (401 lignes, le SHA-256 du modèle
/// Python) et `apercu.png` (184 × 184 pixels) ; une nourriture sous un mur rend le code 1, un message sans `panicked`
/// qui contient `ligne 2` (la ligne de la nourriture) et n'écrit rien ; un calcul qui s'arrête (`10^16`, loi linéaire,
/// 5 pas) rend le code 1, un message qui contient `pas 4`, et n'écrit rien.
///
/// Ne couvre pas : les erreurs d'arguments (voir `etape_0_binaire`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn le_binaire_avec_un_reseau() {
    let dossier = common::dossier_temporaire("1r_binaire");
    fs::create_dir_all(&dossier).unwrap();
    let sortie = dossier.join("sortie");
    let scenario = format!("{RACINE}/scenarios/reseau.txt");
    let lancer_binaire = |scenario: &str, sortie: &std::path::Path| {
        Command::new(env!("CARGO_BIN_EXE_physarum"))
            .args(["lancer", scenario, "--graine", "1", "--sortie", sortie.to_str().unwrap()])
            .output()
            .unwrap()
    };
    let resultat = lancer_binaire(&scenario, &sortie);
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    assert_eq!(
        fs::read_to_string(sortie.join("empreinte.txt")).unwrap(),
        "70eee2fd1df3835e54ebd2fde05600c3a478d1286ed8766957c9d35cef47713a\n"
    );
    let mesures = fs::read_to_string(sortie.join("mesures.csv")).unwrap();
    assert_eq!(mesures.lines().count(), 401);
    assert_eq!(sha256_hex(mesures.as_bytes()), "2db91f181d435473db07b0e45ffc1119e6b2f533733a51a5c39734cbed8aa975");
    let lu = common::relire_png(&fs::read(sortie.join("apercu.png")).unwrap()).expect("PNG valide");
    assert_eq!((lu.largeur, lu.hauteur), (184, 184));
    // une nourriture sous un mur, puis un calcul qui s'arrête
    for (nom, texte, attendu) in [
        (
            "mur.txt",
            "grille 4 4\nnourriture 0 0 0\nmur 0 0 0 0 0\nnourriture 3 3 0\ndebit 1\nloi lineaire\npas 2 0.5\n",
            "ligne 2",
        ),
        (
            "calcul.txt",
            "grille 3 1\netages 2 1.0 10000000000000000\nnourriture 0 0 0\nnourriture 2 0 1\n\
debit 1\nloi lineaire\npas 5 0.5\n",
            "pas 4",
        ),
    ] {
        let fautif = dossier.join(nom);
        fs::write(&fautif, texte).unwrap();
        let absent = dossier.join(format!("absent-{nom}")).join("sortie");
        let resultat = lancer_binaire(fautif.to_str().unwrap(), &absent);
        let erreur = String::from_utf8_lossy(&resultat.stderr).into_owned();
        assert_eq!(resultat.status.code(), Some(1), "{nom} : stderr : {erreur}");
        assert!(erreur.contains(attendu) && !erreur.contains("panicked"), "{nom} : {erreur}");
        assert!(!dossier.join(format!("absent-{nom}")).exists(), "{nom} : rien n'est créé");
    }
    let _ = fs::remove_dir_all(&dossier);
}
