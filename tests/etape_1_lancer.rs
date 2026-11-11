//! Étape 1 : `lancer` avec les tubes (directives `source`, `puits`, `pas`) : l'empreinte, les mesures et l'aperçu.
//! Valeurs attendues : `references/etape-1-valeurs.md`, identifiants `V-LTUB-xx`.
//!
//! L'empreinte est vérifiée d'abord **à la main** : l'écriture canonique de deux petits cas, écrite octet par octet
//! d'après la doc de `lancer` et de `graphe`, et son SHA-256 calculé par Python `hashlib`, un outil indépendant ; puis
//! sur d'autres scénarios, dont l'état a été simulé par le modèle Python `ref_tubes.py` (écrit d'après la seule doc,
//! aucun lien avec ce dépôt : mêmes opérations `f64` dans le même ordre, donc mêmes bits). Seul le test
//! `regression_reference_chemin_graine_1` compare à un fichier de référence produit par notre propre code.

mod common;

use physarum_lab::empreinte::sha256_hex;
use physarum_lab::lancer::{ErreurLancer, Sortie, lancer};
use physarum_lab::rendu::treillis;
use physarum_lab::scenario::lire_scenario;
use physarum_lab::tubes::{Tubes, apercu};

fn lancer_texte(texte: &str, graine: u64) -> Sortie {
    let scenario = lire_scenario(texte).expect("scénario valide");
    lancer(&scenario, graine).unwrap_or_else(|e| panic!("lancer {texte:?} : {e:?}"))
}

fn erreur_de(texte: &str) -> Option<ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario lisible"), 1).err()
}

/// L'écriture canonique de `grille 2 1` avec `source`, `puits`, `nombre` pas de `h` (bits donnés), la conductivité
/// finale de l'unique arête (bits) et le générateur (16 octets), écrite octet par octet d'après la doc de `lancer` («
/// L'écriture canonique de l'état avec tubes »).
fn octets_grille_2_1(source: u32, puits: u32, nombre: u32, h_bits: u64, d_bits: u64, alea: [u8; 16]) -> Vec<u8> {
    let mut o: Vec<u8> = Vec::new();
    o.extend_from_slice(b"PHYSAR01"); // 50 48 59 53 41 52 30 31
    o.extend_from_slice(&[2, 0, 0, 0]); // le graphe : n = 2
    o.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // le nœud 0 : (0, 0, étage 0)
    o.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // le nœud 1 : (1, 0, étage 0)
    o.extend_from_slice(&[1, 0, 0, 0]); // m = 1
    o.extend_from_slice(&[0, 0, 0, 0, 1, 0, 0, 0]); // l'arête : a = 0, b = 1
    o.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0xF0, 0x3F]); // sa longueur 1.0
    o.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0xF0, 0x3F]); // son coût 1.0
    o.extend_from_slice(&source.to_le_bytes());
    o.extend_from_slice(&puits.to_le_bytes());
    o.extend_from_slice(&nombre.to_le_bytes());
    o.extend_from_slice(&h_bits.to_le_bytes());
    o.extend_from_slice(&d_bits.to_le_bytes());
    o.extend_from_slice(&alea);
    o
}

fn hex_vers_octets(hex: &str) -> [u8; 16] {
    let mut octets = [0_u8; 16];
    for (i, o) in octets.iter_mut().enumerate() {
        *o = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap();
    }
    octets
}

/// juste : l'empreinte est le SHA-256 de l'écriture canonique, écrite ici en toutes lettres pour deux cas (V-LTUB-01) :
/// `PHYSAR01`, l'écriture du graphe, la source, le puits, le nombre de pas (trois `u32` petit-boutistes), les bits de
/// `h`, les bits de la conductivité finale, l'état du générateur APRÈS les tirages. `grille 2 1`, `source 0 0 0`,
/// `puits 1 0 0`, graine 7, `h = 0,5` :
/// - un pas (108 octets) : `h` = `000000000000E03F`, conductivité `94611BCEA522ED3F` (0,9104794526602284 :
///   `0,8209589053204569` tirée par la graine 7, puis `D + 0,5 (1 - D)`), générateur `CFC1AAFE29BE9809
///   6D00000000000000` ;
/// - zéro pas (108 octets) : la conductivité tirée, `0x3FEA454B9C36C328` (0,8209589053204569), et le même générateur
///   (deux tirages de 32 bits par arête, quel que soit le nombre de pas).
///
/// Les SHA-256 viennent de Python `hashlib` (`c750f1c3…4f8d` et `8ca8aed2…366f`). Un `u32` grand-boutiste, une
/// étiquette fausse (`PHYSAR00`), un état déjà tiré, une conductivité avant le dernier pas ou l'ordre source/puits
/// échangé échouent ici.
///
/// Ne couvre pas : le contenu de `mesures` et de `apercu` (tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn empreinte_a_la_main() {
    let alea = hex_vers_octets("cfc1aafe29be98096d00000000000000");
    let un_pas = octets_grille_2_1(0, 1, 1, 0x3FE0_0000_0000_0000, 0x3FED_22A5_CE1B_6194, alea);
    assert_eq!(un_pas.len(), 108);
    assert_eq!(sha256_hex(&un_pas), "c750f1c306c2f412e8ca532d991c93d3ee8c16b2567a88582d20cc27d4704f8d");
    let scenario = "grille 2 1\nsource 0 0 0\npuits 1 0 0\npas 1 0.5\n";
    assert_eq!(lancer_texte(scenario, 7).empreinte, sha256_hex(&un_pas));
    let zero_pas = octets_grille_2_1(0, 1, 0, 0x3FE0_0000_0000_0000, 0x3FEA_454B_9C36_C328, alea);
    assert_eq!(sha256_hex(&zero_pas), "8ca8aed2921dd9710b191be46051ef14eb1239c6912559c2cc1823fae000366f");
    assert_eq!(lancer_texte("grille 2 1\nsource 0 0 0\npuits 1 0 0\npas 0 0.5\n", 7).empreinte, sha256_hex(&zero_pas));
    // la source et le puits échangés (indices 1 et 0) : le même écrit avec les deux u32 échangés, calculé par Python
    let echange = octets_grille_2_1(1, 0, 1, 0x3FE0_0000_0000_0000, 0x3FED_22A5_CE1B_6194, alea);
    assert_eq!(sha256_hex(&echange), "96bfb65661b6cfffcb69008dc0b3573e29d0eab2774aab0a49001e8ba40e3f9e");
    assert_eq!(lancer_texte("grille 2 1\nsource 1 0 0\npuits 0 0 0\npas 1 0.5\n", 7).empreinte, sha256_hex(&echange));
}

/// juste : l'empreinte d'autres scénarios, simulée par le modèle Python `ref_tubes.py` (V-LTUB-02) : `grille 3 2`,
/// source `(0,0,0)`, puits `(2,1,0)`, 5 pas de 0,3, graine 1 (`39fe16f1…4f2e`) ; `grille 3 3` en voisinage 8 à deux
/// étages (liens de longueur 0,5 et de coût 2), source `(0,0,0)`, puits `(2,2,1)`, 20 pas de 0,25, graine 3
/// (`77acbb3c…7362`) ; `grille 4 4`, source `(3,0,0)`, puits `(0,3,0)`, 30 pas de 0,5, graine 9 (`ee991bd8…bed0`), avec
/// les directives dans un autre ordre et des commentaires ; le même avec `h = 0,999` (`c96e6c7e…732f`). Chaque
/// changement change l'empreinte : la graine, `h` (même de peu), le nombre de pas, la source, le puits ; écrire le même
/// nombre autrement (`0.50`, `00`) ne la change pas.
///
/// Ne couvre pas : les grilles de plus de 300 nœuds (voir `le_scenario_chemin`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn empreintes_de_scenarios() {
    let cas: [(&str, u64, &str); 5] = [
        (
            "grille 3 2\nsource 0 0 0\npuits 2 1 0\npas 5 0.3\n",
            1,
            "39fe16f176a330c47981a4b311accf29da0aac9baac4d597d8b24ce204814f2e",
        ),
        (
            "grille 3 3\nvoisinage 8\netages 2 0.5 2.0\nsource 0 0 0\npuits 2 2 1\npas 20 0.25\n",
            3,
            "77acbb3c46fd00f2722356e25036c2df616a16af6c108c058ff5c95aedd27362",
        ),
        (
            "grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.5\n",
            9,
            "ee991bd8ca038070aee07a55ce6bdb9fb2e58772d6f128fd66293f4ab854bed0",
        ),
        (
            "pas 30 0.5\n# un commentaire\ngrille 4 4\n\npuits 0 3 0\nsource 3 0 0\n",
            9,
            "ee991bd8ca038070aee07a55ce6bdb9fb2e58772d6f128fd66293f4ab854bed0",
        ),
        (
            "grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.999\n",
            9,
            "c96e6c7e5de9d349ba659cc126f6ba174f90832614553eeddcc1981a77a5732f",
        ),
    ];
    for (texte, graine, attendu) in cas {
        assert_eq!(lancer_texte(texte, graine).empreinte, attendu, "{texte:?} graine {graine}");
    }
    // chaque changement change l'empreinte (le scénario de référence est `grille 4 4`, graine 9)
    let base = cas[2].2;
    for (texte, graine) in [
        ("grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.5\n", 10),
        ("grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.50001\n", 9),
        ("grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 29 0.5\n", 9),
        ("grille 4 4\nsource 2 0 0\npuits 0 3 0\npas 30 0.5\n", 9),
        ("grille 4 4\nsource 3 0 0\npuits 0 2 0\npas 30 0.5\n", 9),
        ("grille 4 4\nsource 0 3 0\npuits 3 0 0\npas 30 0.5\n", 9),
    ] {
        assert_ne!(lancer_texte(texte, graine).empreinte, base, "{texte:?} graine {graine}");
    }
    // la même valeur écrite autrement : mêmes bits, même empreinte
    assert_eq!(lancer_texte("grille 4 4\nsource 03 00 0\npuits 0 3 00\npas 030 0.500\n", 9).empreinte, base);
    assert_eq!(lancer_texte("grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.5\n", 9).empreinte, base);
}

/// juste : les mesures sont le CSV `temps,pression,vifs,longueur`, une ligne par pas (V-LTUB-03), dont les valeurs sont
/// celles du modèle Python : la ligne `t` décrit l'état après `t` pas (la pression est celle de la source calculée par
/// le pas `t + 1`, avec `{:.9}` ; `vifs` compte les arêtes de conductivité `>= 0,5` ; `longueur` somme leurs longueurs
/// avec `{:.6}`). Texte entier pour `grille 2 1` (un pas, zéro pas) et `grille 3 2` (cinq pas) ; pour les deux
/// scénarios plus gros, le nombre de lignes, la première, la dernière et le SHA-256 du texte entier. Les trois
/// premières lignes de `grille 3 2` montrent que `vifs` diminue de 7 à 3 quand les tubes meurent et que la pression de
/// la source monte vers la longueur du chemin.
///
/// Ne couvre pas : le seuil `0,5` à la limite exacte (aucun scénario ne tire une conductivité exactement égale à 0,5 :
/// la limite est éprouvée dans `etape_1_mesures`, sur `mesures::tubes_vifs`, que la doc de `lancer` nomme).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn mesures_en_csv() {
    let entete = "temps,pression,vifs,longueur\n";
    assert_eq!(
        lancer_texte("grille 2 1\nsource 0 0 0\npuits 1 0 0\npas 1 0.5\n", 7).mesures,
        format!("{entete}0,1.218087767,1,1.000000\n")
    );
    assert_eq!(
        lancer_texte("grille 2 1\nsource 0 0 0\npuits 1 0 0\npas 0 0.5\n", 7).mesures,
        entete,
        "zéro pas : l'en-tête seul"
    );
    assert_eq!(
        lancer_texte("grille 3 2\nsource 0 0 0\npuits 2 1 0\npas 5 0.3\n", 1).mesures,
        format!(
            "{entete}0,1.861092680,7,7.000000\n1,2.072322056,7,7.000000\n2,2.261420261,6,6.000000\n\
             3,2.424992917,4,4.000000\n4,2.561673058,3,3.000000\n"
        )
    );
    for (texte, graine, lignes, premiere, derniere, sha) in [
        (
            "grille 3 3\nvoisinage 8\netages 2 0.5 2.0\nsource 0 0 0\npuits 2 2 1\npas 20 0.25\n",
            3,
            20,
            "0,1.150726879,49,51.127417",
            "19,5.157052722,0,0.000000",
            "c4de8185b79a0e7bd9192022ee485f565ff4e3c59f41acf2fe22b002b6340aad",
        ),
        (
            "grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.5\n",
            9,
            30,
            "0,2.483005882,24,24.000000",
            "29,5.999999978,2,2.000000",
            "3f733d5f04b6807e779f45f3ffc81594c091ee0695c1110e97d84ce35b57b243",
        ),
    ] {
        let mesures = lancer_texte(texte, graine).mesures;
        let corps: Vec<&str> = mesures.lines().collect();
        assert_eq!(corps[0], entete.trim_end(), "{texte:?}");
        assert_eq!(corps.len(), lignes + 1, "une ligne par pas : {texte:?}");
        assert_eq!(corps[1], premiere, "{texte:?}");
        assert_eq!(corps[lignes], derniere, "{texte:?}");
        assert!(mesures.ends_with('\n') && !mesures.contains('\r') && !mesures.ends_with("\n\n"));
        assert_eq!(sha256_hex(mesures.as_bytes()), sha, "{texte:?}");
    }
    // la graine change les mesures
    let a = lancer_texte("grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.5\n", 9).mesures;
    let b = lancer_texte("grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.5\n", 10).mesures;
    assert_ne!(a, b);
}

/// juste : l'aperçu avec tubes est le PNG de `tubes::apercu(graphe, conductivités finales, 1.0, echelle)` (V-LTUB-04),
/// avec `echelle = 512 / TL` ramené entre 1 et 8. Le PNG est relu par le décodeur du test ; ses dimensions sont `TL *
/// echelle` sur `th * echelle` (24 × 8 pour `grille 2 1`, 40 × 24 pour `grille 3 2`, 96 × 40 pour `grille 3 3` à deux
/// étages, 56 × 56 pour `grille 4 4`) ; ses pixels sont ceux de `tubes::apercu` appliqué aux conductivités que le test
/// recalcule lui-même avec `Tubes::au_hasard` et `Tubes::avancer` (éprouvés ailleurs), et le SHA-256 de ses octets RVB
/// est celui du modèle Python.
///
/// Ne couvre pas : l'aperçu d'un scénario plus grand (voir `le_scenario_chemin`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn apercu_des_tubes() {
    // (scénario, graine, source, puits, pas, h, (largeur, hauteur) de l'image, SHA-256 des octets RVB du modèle Python)
    #[allow(clippy::type_complexity)]
    let cas: [(&str, u64, usize, usize, usize, f64, (u32, u32), &str); 4] = [
        (
            "grille 2 1
source 0 0 0
puits 1 0 0
pas 1 0.5
",
            7,
            0,
            1,
            1,
            0.5,
            (24, 8),
            "4269fd9397e0d59114f443467ef7a9f24586887b69def6de8f761d33a36c8a5c",
        ),
        (
            "grille 3 2
source 0 0 0
puits 2 1 0
pas 5 0.3
",
            1,
            0,
            5,
            5,
            0.3,
            (40, 24),
            "4bebc2d4fd3826b876b25349db9d60415cfb9f27635bf90733446091502358cb",
        ),
        (
            "grille 3 3
voisinage 8
etages 2 0.5 2.0
source 0 0 0
puits 2 2 1
pas 20 0.25
",
            3,
            0,
            17,
            20,
            0.25,
            (96, 40),
            "c8ce3677a1592b409bd4dc2f692cc90f0b405959b0050365db7ceddeb47728aa",
        ),
        (
            "grille 4 4
source 3 0 0
puits 0 3 0
pas 30 0.5
",
            9,
            3,
            12,
            30,
            0.5,
            (56, 56),
            "f6449dc7fb2f3e75a495e04c81ed8a92b320d57c4357def586cb1c90016f1761",
        ),
    ];
    for (texte, graine, source, puits, pas, h, (largeur, hauteur), sha) in cas {
        let scenario = lire_scenario(texte).unwrap();
        let sortie = lancer(&scenario, graine).unwrap();
        let lu = common::relire_png(&sortie.apercu).unwrap_or_else(|e| panic!("{texte:?} : PNG invalide : {e}"));
        assert_eq!((lu.largeur, lu.hauteur), (largeur, hauteur), "{texte:?}");
        let graphe = scenario.grille.graphe().unwrap();
        let echelle = (512 / treillis(&graphe).0).clamp(1, 8);
        let mut alea = physarum_lab::alea::Alea::new(graine);
        let mut tubes = Tubes::au_hasard(&graphe, &mut alea);
        for _ in 0..pas {
            tubes.avancer(&graphe, source, puits, 1.0, h).unwrap();
        }
        assert_eq!(lu.rvb, apercu(&graphe, tubes.conductivites(), 1.0, echelle).rvb, "{texte:?}");
        assert_eq!(sha256_hex(&lu.rvb), sha, "{texte:?} : octets RVB du modèle Python");
    }
}

/// juste : les erreurs de directives (V-LTUB-05) sont `DirectiveInconnue(ligne)`, avec la règle de priorité de la doc
/// de `lancer` : (1) la ligne de la première directive FAUTIVE dans l'ordre du fichier (nom inconnu ; mauvais nombre
/// d'arguments ; argument mal écrit ou hors bornes ; seconde `source`, `puits` ou `pas`) ; (2) sinon, si `source` et
/// `puits` sont le même nœud, la ligne de celle des deux qui vient en second ; (3) sinon, s'il en manque, la ligne de
/// la première directive restante. Chaque cas ci-dessous est un texte qui se lit sans erreur (`lire_scenario`) et que
/// `lancer` refuse.
///
/// Ne couvre pas : les erreurs de lecture (voir `etape_0_scenario`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn erreurs_de_directives() {
    let ok = "grille 4 4\nsource 0 0 0\npuits 3 3 0\npas 2 0.5\n";
    assert_eq!(erreur_de(ok), None);
    let e = |ligne: usize| Some(ErreurLancer::DirectiveInconnue(ligne));
    // il manque des compagnes : la ligne de la première directive restante
    assert_eq!(erreur_de("grille 4 4\nsource 0 0 0\n"), e(2));
    assert_eq!(erreur_de("grille 4 4\npuits 3 3 0\npas 2 0.5\n"), e(2));
    assert_eq!(erreur_de("# c\ngrille 4 4\n\npas 2 0.5\nsource 0 0 0\n"), e(4));
    assert_eq!(erreur_de("grille 4 4\nsource 0 0 0\npuits 3 3 0\n"), e(2));
    assert_eq!(erreur_de("pas 2 0.5\ngrille 4 4\nsource 0 0 0\n"), e(1));
    // une directive d'un autre nom, avant ou après : la première fautive
    assert_eq!(erreur_de("grille 4 4\nsource 0 0 0\npuits 3 3 0\ninconnue 1\npas 2 0.5\n"), e(4));
    assert_eq!(erreur_de("grille 4 4\ninconnue\nsource 0 0 0\npuits 3 3 0\npas 2 0.5\n"), e(2));
    assert_eq!(erreur_de("grille 4 4\nsource 0 0\ninconnue\npuits 3 3 0\npas 2 0.5\n"), e(2), "la première fautive");
    // des doublons : la seconde est fautive, même avec de bons arguments
    assert_eq!(erreur_de("grille 4 4\nsource 0 0 0\nsource 1 1 0\npuits 3 3 0\npas 2 0.5\n"), e(3));
    assert_eq!(erreur_de("grille 4 4\npuits 3 3 0\nsource 0 0 0\npas 2 0.5\npuits 3 3 0\n"), e(5));
    assert_eq!(erreur_de("grille 4 4\nsource 0 0 0\npuits 3 3 0\npas 2 0.5\npas 2 0.5\n"), e(5));
    // une source ou un puits mal écrit ou hors de la grille
    for mauvais in [
        "source 0 0",
        "source 0 0 0 0",
        "source",
        "source a 0 0",
        "source -1 0 0",
        "source +1 0 0",
        "source 1.0 0 0",
        "source 0x1 0 0",
        "source 4 0 0",
        "source 0 4 0",
        "source 0 0 1",
        "source 99999999999999999999 0 0",
        "source 0 0 4294967296",
    ] {
        let texte = format!("grille 4 4\n{mauvais}\npuits 3 3 0\npas 2 0.5\n");
        assert_eq!(erreur_de(&texte), e(2), "{mauvais:?}");
        let texte = format!("grille 4 4\nsource 0 0 0\n{}\npas 2 0.5\n", mauvais.replace("source", "puits"));
        assert_eq!(erreur_de(&texte), e(3), "{mauvais:?} en puits");
    }
    // pas : mal écrit ou hors bornes
    for mauvais in [
        "pas",
        "pas 2",
        "pas 2 0.5 1",
        "pas -1 0.5",
        "pas 2.0 0.5",
        "pas x 0.5",
        "pas 100001 0.5",
        "pas 99999999999999999999 0.5",
        "pas 2 0",
        "pas 2 0.0",
        "pas 2 -0",
        "pas 2 1",
        "pas 2 1.0",
        "pas 2 -0.5",
        "pas 2 1.5",
        "pas 2 .5",
        "pas 2 5.",
        "pas 2 +0.5",
        "pas 2 5e-1",
        "pas 2 inf",
        "pas 2 nan",
        "pas 2 abc",
    ] {
        let texte = format!("grille 4 4\nsource 0 0 0\npuits 3 3 0\n{mauvais}\n");
        assert_eq!(erreur_de(&texte), e(4), "{mauvais:?}");
    }
    // les bornes valides : zéro pas, 100 000 pas, h très petit et très proche de 1, les zéros de tête, deux étages
    for bon in ["pas 0 0.5", "pas 100000 0.5", "pas 2 0.000001", "pas 2 0.999999999", "pas 002 0.5"] {
        let texte = format!("grille 2 1\nsource 0 0 0\npuits 1 0 0\n{bon}\n");
        assert_eq!(erreur_de(&texte), None, "{bon:?}");
    }
    assert_eq!(erreur_de("grille 4 4\netages 2 1 1\nsource 0 0 1\npuits 3 3 0\npas 2 0.5\n"), None, "l'étage 1 existe");
    assert_eq!(
        erreur_de("grille 4 4\netages 2 1 1\nsource 0 0 2\npuits 3 3 0\npas 2 0.5\n"),
        e(3),
        "l'étage 2 n'existe pas"
    );
    assert_eq!(
        erreur_de("grille 4 4\nsource 3 3 0\npuits 3 3 1\npas 2 0.5\n"),
        e(3),
        "un seul étage : l'étage 1 n'existe pas"
    );
    // le même nœud : la ligne de la seconde des deux
    assert_eq!(erreur_de("grille 4 4\nsource 1 1 0\npuits 1 1 0\npas 2 0.5\n"), e(3));
    assert_eq!(erreur_de("grille 4 4\npuits 1 1 0\nsource 01 1 0\npas 2 0.5\n"), e(3));
    assert_eq!(erreur_de("grille 4 4\npas 2 0.5\nsource 1 1 0\n\npuits 1 1 0\n"), e(5));
    assert_eq!(erreur_de("grille 4 4\npuits 1 1 0\npas 2 0.5\nsource 1 1 0\n"), e(4));
    assert_eq!(erreur_de("grille 4 4\nsource 1 1 0\npuits 1 1 0\n"), e(3), "le même nœud avant le manque");
    assert_eq!(erreur_de("grille 4 4\nsource 1 1 0\npuits 1 1 0\npas 2 2\n"), e(4), "la fautive avant le même nœud");
    assert_eq!(
        erreur_de("grille 4 4\netages 2 1 1\nsource 1 1 0\npuits 1 1 1\npas 2 0.5\n"),
        None,
        "deux étages : deux nœuds"
    );
}

/// juste : le scénario d'exemple `scenarios/chemin.txt` (V-LTUB-06) : une grille 16 × 16 en voisinage 8, la source en
/// `(1, 1)`, le puits en `(14, 14)` (la diagonale : 13 arêtes de longueur `√2`), 600 pas de 0,5. Pour la graine 1,
/// l'empreinte est celle du modèle Python `ref_tubes.py` (`6e3b15b8…2f37`), écrite avant le code ; la graine 2 donne
/// `bf66d262…b8db`. Les mesures ont 600 lignes ; la première est `0,1.584097569,930,1116.396103` (les 930 arêtes sont
/// vives au départ) et la dernière `599,18.384876222,13,18.384776` : treize arêtes vives, de longueur `13 √2 =
/// 18,384776`, et une pression de la source qui approche la longueur du chemin par le haut ; le SHA-256 du texte entier
/// est `04c88cff…428f`. L'aperçu fait 248 × 248 pixels (échelle 8), son SHA-256 RVB est `764418f3…6331` (le même pour
/// la graine 2 : à 600 pas, l'image ne dépend plus de la graine), et ses nœuds de la diagonale sont à la couleur vive.
///
/// Ne couvre pas : que le résultat soit stable entre Linux et Windows (c'est la CI qui rejoue l'empreinte de
/// référence).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn le_scenario_chemin() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let texte = std::fs::read_to_string(format!("{racine}/scenarios/chemin.txt")).expect("scenarios/chemin.txt");
    let scenario = lire_scenario(&texte).expect("le scénario d'exemple est lisible");
    assert_eq!((scenario.grille.largeur, scenario.grille.hauteur, scenario.directives.len()), (16, 16, 3));
    let sortie = lancer(&scenario, 1).unwrap();
    assert_eq!(sortie.empreinte, "6e3b15b88963ed142ce3441c33b249b2164d13a51087662b8927f77ddd3c2f37");
    assert_eq!(
        lancer(&scenario, 2).unwrap().empreinte,
        "bf66d262f53e97461ebdd225280a1482a999a9fce7c912cef057ac088c6cb8db"
    );
    let lignes: Vec<&str> = sortie.mesures.lines().collect();
    assert_eq!(lignes.len(), 601);
    assert_eq!(lignes[1], "0,1.584097569,930,1116.396103");
    assert_eq!(lignes[2], "1,2.885421727,44,53.941125");
    assert_eq!(lignes[600], "599,18.384876222,13,18.384776");
    assert_eq!(
        sha256_hex(sortie.mesures.as_bytes()),
        "04c88cff583452ef321e0431d8e0495091573a0b47690750770d838893cb428f"
    );
    let lu = common::relire_png(&sortie.apercu).expect("PNG valide");
    assert_eq!(lancer(&scenario, 2).unwrap().apercu, sortie.apercu, "convergé : l'image ne dépend plus de la graine");
    assert_eq!((lu.largeur, lu.hauteur), (248, 248));
    assert_eq!(sha256_hex(&lu.rvb), "764418f3582de6eee07d507a1bc5d2eb1efed14252a823680d1fd5a2fa5a6331");
    // les nœuds de la diagonale, de (1,1) à (14,14), sont à la couleur vive : leur point est (2x, 2y) du treillis,
    // agrandi huit fois
    for k in 1..=14_usize {
        let (px, py) = (2 * k * 8, 2 * k * 8);
        let i = 3 * (py * 248 + px);
        assert_eq!(lu.rvb[i..i + 3], [60, 220, 255], "nœud ({k}, {k})");
    }
    // un nœud hors de la diagonale est du fond
    let i = 3 * ((2 * 5 * 8) * 248 + 2 * 9 * 8);
    assert_eq!(lu.rvb[i..i + 3], [12, 16, 24], "le nœud (9, 5)");
}

/// régression : pour `scenarios/chemin.txt` et la graine 1, l'empreinte rendue par `lancer` est celle du fichier
/// `references/chemin-graine-1.empreinte` (l'empreinte puis un saut de ligne), que la CI rejoue sous Linux et sous
/// Windows. Le fichier est produit par le binaire (V-LTUB-07).
///
/// Ne couvre pas : que l'empreinte soit juste (`le_scenario_chemin` la compare à la valeur du modèle Python, calculée à
/// part).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn regression_reference_chemin_graine_1() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let reference = std::fs::read_to_string(format!("{racine}/references/chemin-graine-1.empreinte"))
        .expect("references/chemin-graine-1.empreinte est absente : la vague du code la produit avec le binaire");
    let texte = std::fs::read_to_string(format!("{racine}/scenarios/chemin.txt")).expect("scenarios/chemin.txt");
    assert_eq!(format!("{}\n", lancer_texte(&texte, 1).empreinte), reference);
}

/// juste : l'étape 0 ne change pas (V-LTUB-08) : les scénarios sans directive des tubes gardent leur écriture
/// `PHYSAR00`, leurs mesures `temps,noeuds,aretes,longueur` et leur empreinte (valeurs de `etape_0_lancer`, calculées à
/// part par Python) ; et les directives que l'étape 0 refusait (`inconnue`, `x_1`, `autre`, `sans_argument`) sont
/// toujours refusées avec la même ligne, seules ou à côté d'une directive des tubes.
///
/// Ne couvre pas : le détail de l'étape 0 (ses propres tests).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn l_etape_0_ne_change_pas() {
    assert_eq!(
        lancer_texte("grille 1 1", 0).empreinte,
        "e07d8298c4c650883c03213c49b38643fffa60bfca6836794ac1216a86a3d459"
    );
    assert_eq!(
        lancer_texte("grille 3 2", 7).empreinte,
        "4d066023b79eba897bc283369d234a9ca2377cb85b9a3d8cf054c87fb1fc4008"
    );
    assert_eq!(lancer_texte("grille 3 2", 7).mesures, "temps,noeuds,aretes,longueur\n0,6,7,7.000000\n");
    let exemple = "grille 24 16\nvoisinage 8\netages 2 1.0 3.0\n";
    assert_eq!(lancer_texte(exemple, 1).empreinte, "c336148193197cb8541fd6a58c36f149118260789f9e388771405fe52e6eaf59");
    for (texte, ligne) in [
        ("grille 3 2\ninconnue 1 2\n", 2),
        ("# c\n\nx_1\ngrille 3 2\nautre 3\n", 3),
        ("grille 3 2\nvoisinage 8\n\n\nsans_argument\ninconnue 1\n", 5),
    ] {
        assert_eq!(erreur_de(texte), Some(ErreurLancer::DirectiveInconnue(ligne)), "{texte:?}");
    }
}

/// juste : un calcul qui s'arrête rend `ErreurLancer::Calcul(t)`, `t` étant le numéro (à partir de 0) du pas où
/// `Tubes::avancer` a rendu une erreur, et `lancer` ne panique pas (V-LTUB-10, la doc de `lancer` : « Un calcul qui
/// s'arrête »). Sur `grille 3 3` à deux étages, source `(0,0,0)`, puits `(2,2,0)`, des pas de 0,5 : avec un lien entre
/// étages d'un coût de `10^16`, le solveur s'arrête sur un pivot nul au pas 2 pour la graine 1, au pas 7 pour la graine
/// 2, au pas 3 pour la graine 7, au pas 13 pour la graine 9 ; avec un coût de `10^-16`, au pas 1 pour les graines 1 et
/// 2, au pas 0 pour la graine
/// 7. Ces numéros viennent du modèle Python (`valeurs_calcul_1.py` : la procédure de la doc du solveur avec son
///    contrôle du pivot), pas de notre code. Le même scénario arrêté avant le pas fautif réussit : `pas 2 0.5` passe là
///    où `pas 3 0.5` rend `Calcul(2)`, et `pas 0 0.5` passe toujours. Un coût de `10^15` ne fait rien échouer en 40
///    pas.
///
/// Ne couvre pas : la justesse des pressions quand les conductances sont très étalées sans que le solveur s'arrête
/// (voir « Ce que le solveur ne garantit pas » dans `solveur`) ; les autres erreurs d'`avancer`, que `lancer` ne peut
/// pas provoquer (ses vérifications les écartent avant).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn calcul_qui_s_arrete() {
    let scenario =
        |cout: &str, pas: u32| format!("grille 3 3\netages 2 1 {cout}\nsource 0 0 0\npuits 2 2 0\npas {pas} 0.5\n");
    let erreur = |cout: &str, pas: u32, graine: u64| {
        lancer(&lire_scenario(&scenario(cout, pas)).expect("scénario lisible"), graine).err()
    };
    let grand = "10000000000000000";
    let petit = "0.0000000000000001";
    for (cout, graine, t) in
        [(grand, 1, 2), (grand, 2, 7), (grand, 7, 3), (grand, 9, 13), (petit, 1, 1), (petit, 2, 1), (petit, 7, 0)]
    {
        assert_eq!(erreur(cout, 40, graine), Some(ErreurLancer::Calcul(t)), "coût {cout}, graine {graine}");
        // le pas fautif est le dernier demandé : même erreur ; il n'est pas demandé : aucune erreur
        assert_eq!(erreur(cout, t + 1, graine), Some(ErreurLancer::Calcul(t)), "coût {cout}, graine {graine}");
        assert_eq!(erreur(cout, t, graine), None, "coût {cout}, graine {graine} : arrêté avant le pas {t}");
        assert_eq!(erreur(cout, 0, graine), None, "coût {cout}, graine {graine} : zéro pas");
    }
    let sortie = lancer_texte(&scenario(grand, 2), 1);
    assert_eq!(sortie.mesures.lines().count(), 3, "deux pas faits : l'en-tête et deux lignes");
    for graine in [1, 2, 7] {
        assert_eq!(erreur("1000000000000000", 40, graine), None, "un coût de 10^15, graine {graine}");
    }
}

/// juste : cent mille pas (V-LTUB-09) : la borne haute de `pas` est acceptée (`grille 2 1`, graine 1) ; les mesures ont
/// 100 000 lignes, la première porte le temps 0, la dernière le temps 99999 ; la conductivité de l'unique arête tend
/// vers le débit 1 (un tube seul : la pression de la source est celle d'un tube de conductivité 1 et de coût 1, soit
/// 1).
///
/// Ne couvre pas : 100 001 pas (refusés : voir `erreurs_de_directives`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn cent_mille_pas() {
    let sortie = lancer_texte("grille 2 1\nsource 0 0 0\npuits 1 0 0\npas 100000 0.5\n", 1);
    let lignes: Vec<&str> = sortie.mesures.lines().collect();
    assert_eq!(lignes.len(), 100_001);
    assert!(lignes[1].starts_with("0,"));
    assert_eq!(lignes[100_000], "99999,1.000000000,1,1.000000");
}

/// juste : l'échelle de l'aperçu avec tubes est `512 / TL` (division entière, `TL` la LARGEUR du treillis), ramenée
/// entre 1 et 8 (V-LTUB-11, la doc de `lancer` : « la même règle d'échelle »). `grille 40 2` : `TL = 79`, hauteur du
/// treillis 3, échelle `512 / 79 = 6`, image de 474 × 18 (une échelle prise sur la hauteur donnerait 8). `grille 256 1`
/// : `TL = 511`, échelle 1, image de 511 × 1. `grille 256 1` à deux étages : `TL = 2 × 511 + 2 = 1024`, le quotient est
/// 0 et l'échelle est ramenée à 1 : image de 1024 × 1, sans panique. Zéro pas : l'aperçu des conductivités de départ.
/// Calculs à la main sur la définition du treillis (étape 0). Ajouté après la campagne de fautes du juge.
///
/// Ne couvre pas : les couleurs de ces aperçus (voir `apercu_des_tubes`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn echelle_de_l_apercu() {
    for (texte, dimensions) in [
        ("grille 40 2\nsource 0 0 0\npuits 39 1 0\npas 0 0.5\n", (474, 18)),
        ("grille 256 1\nsource 0 0 0\npuits 255 0 0\npas 0 0.5\n", (511, 1)),
        ("grille 256 1\netages 2 1 1\nsource 0 0 0\npuits 255 0 0\npas 0 0.5\n", (1024, 1)),
        ("grille 2 1\nsource 0 0 0\npuits 1 0 0\npas 0 0.5\n", (24, 8)),
    ] {
        let sortie = lancer_texte(texte, 1);
        let lu = common::relire_png(&sortie.apercu).unwrap_or_else(|e| panic!("{texte:?} : PNG invalide : {e}"));
        assert_eq!((lu.largeur, lu.hauteur), dimensions, "{texte:?}");
    }
}
