//! Étape 1 : `tubes::apercu`, l'aperçu des tubes (la couleur dit l'épaisseur). Valeurs attendues :
//! `references/etape-1-valeurs.md`, identifiants `V-APER-xx`.
//!
//! La source est la définition écrite dans la documentation de `tubes` (« L'aperçu des tubes ») : un niveau de 0 à 255
//! par conductivité, un mélange entier par canal, le même treillis que `rendu::apercu_graphe`. Les couleurs attendues
//! viennent du modèle Python `ref_tubes.py` (entiers de la doc, aucun lien avec le code Rust). Chaque treillis attendu
//! est dessiné ici en caractères, d'après la définition et recoupé par le modèle Python.
// Les boucles à indices de ce fichier suivent la définition de la doc.
#![allow(clippy::needless_range_loop)]

use physarum_lab::graphe::{Arete, Graphe, Grille, Noeud, Voisinage};
use physarum_lab::rendu::{FOND, Image, PUITS, apercu_graphe};
use physarum_lab::tubes::{VIF, apercu};

fn grille(largeur: u32, hauteur: u32, etages: u32, voisinage: Voisinage) -> Graphe {
    Grille { largeur, hauteur, etages, voisinage, longueur_verticale: 0.5, cout_vertical: 1.0 }.graphe().unwrap()
}

/// Le dessin en caractères : un caractère par point du treillis, d'après une palette `(caractère, couleur)`. Vérifie au
/// passage que l'image a la bonne longueur et que chaque carré de `echelle × echelle` pixels est d'une seule couleur de
/// la palette.
fn dessin(image: &Image, echelle: u32, palette: &[(char, [u8; 3])]) -> Vec<String> {
    assert_eq!(image.rvb.len(), 3 * image.largeur as usize * image.hauteur as usize, "longueur de l'image");
    assert_eq!((image.largeur % echelle, image.hauteur % echelle), (0, 0), "dimensions multiples de l'échelle");
    let (l, e) = (image.largeur as usize, echelle as usize);
    let pixel = |px: usize, py: usize| -> [u8; 3] {
        let i = 3 * (py * l + px);
        [image.rvb[i], image.rvb[i + 1], image.rvb[i + 2]]
    };
    let mut lignes = Vec::new();
    for ty in 0..image.hauteur as usize / e {
        let mut ligne = String::new();
        for tx in 0..l / e {
            let couleur = pixel(tx * e, ty * e);
            for dy in 0..e {
                for dx in 0..e {
                    assert_eq!(pixel(tx * e + dx, ty * e + dy), couleur, "le carré du point ({tx}, {ty}) est uni");
                }
            }
            let Some(&(c, _)) = palette.iter().find(|(_, couleur_connue)| *couleur_connue == couleur) else {
                panic!("couleur inconnue {couleur:?} au point ({tx}, {ty})");
            };
            ligne.push(c);
        }
        lignes.push(ligne);
    }
    lignes
}

/// Les couleurs de la fiche, calculées par `ref_tubes.py` (`melange(FOND, VIF, niveau)`), pour les niveaux 0, 51, 128
/// et 255.
const N0: [u8; 3] = [12, 16, 24];
const N51: [u8; 3] = [22, 57, 70];
const N128: [u8; 3] = [36, 118, 140];
const N255: [u8; 3] = [60, 220, 255];

/// juste : les couleurs de la doc (V-APER-01) : `VIF = (60, 220, 255)`, `FOND = (12, 16, 24)`, `PUITS = (240, 170,
/// 40)`.
///
/// Ne couvre pas : le dessin (tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn couleurs() {
    assert_eq!(VIF, [60, 220, 255]);
    assert_eq!((FOND, PUITS), ([12, 16, 24], [240, 170, 40]));
    assert_eq!(N255, VIF);
    assert_eq!(N0, FOND);
}

/// juste : le niveau d'une conductivité et le mélange entier (V-APER-02), lus sur la couleur du point de l'arête d'un
/// graphe de deux nœuds (treillis de 3 points : un nœud, l'arête, un nœud ; les trois points ont la même couleur).
/// Niveau 0 si `d / reference` n'est pas positif (zéro, négatif, NaN) ou tombe sous `1/510`, soit `1e-6`, `0,001` ; 1
/// pour `0,002` ; 51 pour `0,2` ; **77 pour `0,3`** (`0,3 * 255` vaut exactement `76,5` en flottants : l'arrondi est
/// `round`, les moitiés vers l'extérieur ; un arrondi au pair donnerait 76, une troncature 76) ; **128 pour `0,5`**
/// (`127,5` : une troncature donnerait 127) ; 204 pour `0,8` ; 255 pour `0,9999`, `1`, `7` et l'infini (au plus 255) ;
/// `3 / 6` et `0,25 / 0,5` donnent 128 (le rapport compte, pas la valeur). Les couleurs, calculées par `ref_tubes.py`
/// avec `(a * (255 - n) + b * n + 127) / 255` : 1 donne `(12, 17, 25)`, 51 `(22, 57, 70)`, 77 `(26, 78, 94)`, 128 `(36,
/// 118, 140)`, 204 `(50, 179, 209)`, 255 `VIF`.
///
/// Ne couvre pas : le mélange vers les puits (voir `deux_etages`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn niveaux_et_melange() {
    let g = Graphe::new(
        vec![Noeud { x: 0, y: 0, etage: 0 }, Noeud { x: 1, y: 0, etage: 0 }],
        vec![Arete { a: 0, b: 1, longueur: 1.0, cout: 1.0 }],
    )
    .unwrap();
    let cas: [(f64, f64, [u8; 3]); 17] = [
        (0.0, 1.0, FOND),
        (-0.0, 1.0, FOND),
        (-1.0, 1.0, FOND),
        (f64::NAN, 1.0, FOND),
        (1e-6, 1.0, FOND),
        (0.001, 1.0, FOND),
        (0.002, 1.0, [12, 17, 25]),
        (0.2, 1.0, [22, 57, 70]),
        (0.3, 1.0, [26, 78, 94]),
        (0.5, 1.0, [36, 118, 140]),
        (0.8, 1.0, [50, 179, 209]),
        (0.9999, 1.0, VIF),
        (7.0, 1.0, VIF),
        (f64::INFINITY, 1.0, VIF),
        (3.0, 6.0, [36, 118, 140]),
        (0.25, 0.5, [36, 118, 140]),
        (1.0, 1.0, VIF),
    ];
    for (d, reference, couleur) in cas {
        let image = apercu(&g, &[d], reference, 1);
        assert_eq!((image.largeur, image.hauteur), (3, 1));
        for point in 0..3 {
            assert_eq!(image.rvb[3 * point..3 * point + 3], couleur, "d = {d}, référence {reference}, point {point}");
        }
    }
}

/// juste : une grille 3 × 2 en voisinage 8, un étage, les onze arêtes de conductivités différentes (V-APER-03). Les
/// arêtes sont rangées `(0,1) (0,3) (0,4) (1,2) (1,3) (1,4) (1,5) (2,4) (2,5) (3,4) (4,5)` ; leurs niveaux, de 0 à 10 :
/// `255, 0, 128, 51, 255, 0, 128, 51, 0, 51, 0` (`c`, `.`, `b`, `a`). Le dessin attendu (`.` niveau 0, `a` 51, `b` 128,
/// `c` 255), lu à la main d'après la doc et recoupé par le modèle Python :
/// - le point d'un nœud prend le niveau de sa plus épaisse arête : le nœud 4 (arêtes aux niveaux 128, 0, 51, 51, 0) est
///   `b`, le nœud 2 (arêtes 51, 51, 0) est `a`, le nœud 1 (arêtes 255, 51, 255, 0, 128) est `c` ;
/// - les deux diagonales d'une maille se croisent au même point, et **la plus épaisse l'emporte, quel que soit l'ordre
///   des indices** : au milieu de la maille de gauche, `(0,4)` (niveau 128) et `(1,3)` (255) donnent `c` ; au milieu de
///   celle de droite, `(1,5)` (128) et `(2,4)` (51) donnent `b` (c'est la plus petite des deux qui a le plus grand
///   indice : une règle « la dernière recouvre » donnerait `a`).
///
/// Ne couvre pas : les liens entre étages (voir `deux_etages`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn grille_a_un_etage() {
    let g = grille(3, 2, 1, Voisinage::Huit);
    let d = [1.0, 1e-6, 0.5, 0.2, 1.0, 1e-6, 0.5, 0.2, 1e-6, 0.2, 1e-6];
    assert_eq!(d.len(), g.nb_aretes());
    let palette = [('.', N0), ('a', N51), ('b', N128), ('c', N255)];
    let image = apercu(&g, &d, 1.0, 1);
    assert_eq!(dessin(&image, 1, &palette), ["cccaa", ".c.b.", "cab.b"]);
    // à l'échelle 2, le premier pixel est celui du nœud 0
    let image = apercu(&g, &d, 1.0, 2);
    assert_eq!((image.largeur, image.hauteur), (10, 6));
    assert_eq!(dessin(&image, 2, &palette), ["cccaa", ".c.b.", "cab.b"]);
    assert_eq!(image.rvb[..6], [60, 220, 255, 60, 220, 255]);
}

/// juste : deux étages (V-APER-04) : la grille 3 × 2 en voisinage 4 à 2 étages, deux panneaux de 5 points séparés par 2
/// colonnes de fond (12 × 3). Conductivités : arêtes de l'étage 0 à 1 (niveau 255), celles de l'étage 1 à 0,2 (niveau
/// 51), et les six liens entre étages à `1 ; 0,5 ; 0,2` pour `y = 0` (`x = 0, 1, 2`) et `1e-6 ; 0,5 ; 1` pour `y = 1`.
/// Un nœud d'où part un lien passe de la couleur de son niveau (le plus grand de ses arêtes, **liens compris**) vers
/// `PUITS`, au niveau du lien :
/// - étage 0 : le nœud est au niveau 255 (`VIF`), puis `melange(VIF, PUITS, N)` : `A` (240, 170, 40) pour `N = 255`,
///   `B` (150, 195, 147) pour 128, `C` (96, 210, 212) pour 51, et `VIF` (`V`) pour un lien de niveau 0 : il reste tel
///   qu'il est ;
/// - étage 1 : le nœud est au niveau `max(51, N)`, puis le même mélange : `N = 255` donne `A`, `N = 128` donne `D`
///   (138, 144, 90), `N = 51` donne `E` (66, 80, 64), et `N = 0` laisse le niveau 51 (`e`, (22, 57, 70)).
///
/// Les valeurs de `B` à `E` sont celles de `ref_tubes.py`. Un lien de niveau 0 ne cache pas le nœud ; les arêtes de
/// l'étage 1 sont `e`.
///
/// Ne couvre pas : trois étages ; des liens entre étages dont les positions diffèrent (voir `silhouette_du_treillis`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn deux_etages() {
    let g = grille(3, 2, 2, Voisinage::Quatre);
    let d: Vec<f64> = g
        .aretes()
        .iter()
        .map(|arete| {
            let (a, b) = (g.noeuds()[arete.a], g.noeuds()[arete.b]);
            if a.etage != b.etage {
                [[1.0, 0.5, 0.2], [1e-6, 0.5, 1.0]][a.y as usize][a.x as usize]
            } else if a.etage == 0 {
                1.0
            } else {
                0.2
            }
        })
        .collect();
    assert_eq!(d.len(), 20);
    let palette = [
        ('.', FOND),
        ('V', VIF),
        ('e', N51),
        ('A', PUITS),
        ('B', [150, 195, 147]),
        ('C', [96, 210, 212]),
        ('D', [138, 144, 90]),
        ('E', [66, 80, 64]),
    ];
    let attendu = ["AVBVC..AeDeE", "V.V.V..e.e.e", "VVBVA..eeDeA"];
    assert_eq!(dessin(&apercu(&g, &d, 1.0, 1), 1, &palette), attendu);
    assert_eq!(dessin(&apercu(&g, &d, 1.0, 3), 3, &palette), attendu);
}

/// juste : le MÊME treillis que `rendu::apercu_graphe` (V-APER-05) : pour chaque point, le point est du fond dans
/// l'aperçu des tubes si et seulement s'il l'est dans celui du graphe, quand toutes les conductivités valent la
/// référence (aucun niveau 0). Essayé sur une grille 4 × 3 en voisinage 8 à 2 étages, sur le graphe quelconque de
/// `etape_0_rendu` (huit nœuds, un cadre qui ne vient pas du premier nœud, un étage vide, une arête trop longue, deux
/// nœuds à la même position, des liens entre étages à positions voisines) ; mêmes dimensions de l'image à chaque
/// échelle de 1 à 4.
///
/// Ne couvre pas : les couleurs (tests précédents).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn silhouette_du_treillis() {
    let noeud = |x: i32, y: i32, etage: u32| Noeud { x, y, etage };
    let arete = |a: usize, b: usize| Arete { a, b, longueur: 1.0, cout: 1.0 };
    let quelconque = Graphe::new(
        vec![
            noeud(0, -1, 0),
            noeud(-1, -2, 0),
            noeud(1, -2, 0),
            noeud(0, -2, 0),
            noeud(1, 0, 2),
            noeud(2, -1, 0),
            noeud(-1, -2, 2),
            noeud(0, -1, 0),
        ],
        vec![arete(1, 3), arete(3, 5), arete(0, 3), arete(0, 7), arete(2, 5), arete(1, 6), arete(4, 5)],
    )
    .unwrap();
    for (nom, g) in [("grille", grille(4, 3, 2, Voisinage::Huit)), ("quelconque", quelconque)] {
        let tous_vifs = vec![1.0; g.nb_aretes()];
        for echelle in 1..=4 {
            let tubes = apercu(&g, &tous_vifs, 1.0, echelle);
            let graphe = apercu_graphe(&g, echelle);
            assert_eq!((tubes.largeur, tubes.hauteur), (graphe.largeur, graphe.hauteur), "{nom}, échelle {echelle}");
            assert_eq!(tubes.rvb.len(), graphe.rvb.len());
            let (a, b) = (graphe.rvb.chunks(3), tubes.rvb.chunks(3));
            let mut points_non_fond = 0;
            for (i, (p, q)) in a.zip(b).enumerate() {
                assert_eq!(p == &FOND[..], q == &FOND[..], "{nom}, échelle {echelle}, pixel {i}");
                points_non_fond += usize::from(p != &FOND[..]);
            }
            assert!(points_non_fond >= echelle as usize * echelle as usize, "{nom} : l'aperçu n'est pas vide");
        }
    }
}

/// juste : un nœud sans aucune arête est au niveau 0 (« 0 s'il n'en a aucune ») : son point est du fond, loin de
/// l'origine comme ailleurs, et l'image fait 1 × 1 pour un seul nœud (V-APER-08). Avec une arête de niveau 255, ses
/// deux nœuds ont la couleur vive.
///
/// Ne couvre pas : les graphes sans arête de plus d'un nœud.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn noeud_sans_arete() {
    let noeud = |x: i32, y: i32, etage: u32| Noeud { x, y, etage };
    let seul = Graphe::new(vec![noeud(1000, -1000, 0)], vec![]).unwrap();
    let image = apercu(&seul, &[], 1.0, 2);
    assert_eq!((image.largeur, image.hauteur), (2, 2));
    assert_eq!(image.rvb, FOND.repeat(4));
    let deux =
        Graphe::new(vec![noeud(-3, 5, 0), noeud(-2, 5, 0)], vec![Arete { a: 0, b: 1, longueur: 1.0, cout: 1.0 }])
            .unwrap();
    assert_eq!(apercu(&deux, &[1.0], 1.0, 1).rvb, VIF.repeat(3));
}

/// juste : l'aperçu d'un cas riche contre le modèle Python (V-APER-06) : une grille 4 × 3 à 2 étages en voisinage 8 (24
/// nœuds, 70 arêtes), `D[e] = ((e * 37 + 11) % 23) / 20` (de 0 à 1,1 : des niveaux de 0 à 255, trop grands compris). Le
/// SHA-256 des octets RVB, calculé par `ref_tubes.py` d'après la doc (treillis, niveaux, mélange, recouvrement, liens),
/// vaut `5886ef40…bf9a8` pour la référence 1 et l'échelle 3 (image de 48 × 15 pixels), et `7967a5ec…5634` pour la
/// référence 0,5 et l'échelle 1 (16 × 5).
///
/// Ne couvre pas : les échelles au-delà de 8.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn contre_le_modele_python() {
    let g = Grille {
        largeur: 4,
        hauteur: 3,
        etages: 2,
        voisinage: Voisinage::Huit,
        longueur_verticale: 0.5,
        cout_vertical: 1.0,
    }
    .graphe()
    .unwrap();
    assert_eq!((g.nb_noeuds(), g.nb_aretes()), (24, 70));
    let d: Vec<f64> = (0..70_usize).map(|e| ((e * 37 + 11) % 23) as f64 / 20.0).collect();
    let image = apercu(&g, &d, 1.0, 3);
    assert_eq!((image.largeur, image.hauteur), (48, 15));
    assert_eq!(
        physarum_lab::empreinte::sha256_hex(&image.rvb),
        "5886ef407f6668e0c9dd845f6d2785a14f5896b97d0d99573969c3f08c5bf9a8"
    );
    let image = apercu(&g, &d, 0.5, 1);
    assert_eq!((image.largeur, image.hauteur), (16, 5));
    assert_eq!(
        physarum_lab::empreinte::sha256_hex(&image.rvb),
        "7967a5ec17d377817fd4f55ade8e5c1398498b216f62d2d1bd5b4fbf22315634"
    );
}

/// juste : `apercu` panique pour une échelle nulle, une référence nulle, négative, infinie ou NaN, et un nombre de
/// conductivités qui n'est pas le nombre d'arêtes (trop peu ou trop) (V-APER-07) ; le cas positif d'abord.
///
/// Ne couvre pas : le texte des paniques.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn paniques() {
    let g = grille(2, 2, 1, Voisinage::Quatre);
    let d = [1.0; 4];
    assert_eq!(apercu(&g, &d, 1.0, 1).largeur, 3);
    assert!(std::panic::catch_unwind(|| apercu(&g, &d, 1.0, 0)).is_err(), "échelle nulle");
    for reference in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(std::panic::catch_unwind(|| apercu(&g, &d, reference, 1)).is_err(), "référence {reference}");
    }
    assert!(std::panic::catch_unwind(|| apercu(&g, &d[..3], 1.0, 1)).is_err(), "une conductivité de moins");
    assert!(std::panic::catch_unwind(|| apercu(&g, &[1.0; 5], 1.0, 1)).is_err(), "une conductivité de trop");
    assert!(std::panic::catch_unwind(|| apercu(&g, &[], 1.0, 1)).is_err(), "aucune conductivité");
}

/// Un graphe fait à la main : des nœuds `(x, y, étage)` et des arêtes `(a, b)` de longueur et de coût 1.
fn graphe_a_la_main(noeuds: &[(i32, i32, u32)], aretes: &[(usize, usize)]) -> Graphe {
    let noeuds = noeuds.iter().map(|&(x, y, etage)| Noeud { x, y, etage }).collect();
    let aretes = aretes.iter().map(|&(a, b)| Arete { a, b, longueur: 1.0, cout: 1.0 }).collect();
    Graphe::new(noeuds, aretes).expect("graphe valide")
}

/// juste : le cadre se calcule sur TOUS les nœuds, le premier compris (V-APER-09). Deux nœuds à la même position, le
/// nœud 0 à l'étage 1 et le nœud 1 à l'étage 0, reliés par un lien : deux panneaux d'un point séparés par l'écart de
/// deux points, soit 4 × 1 ; chaque bout du lien va vers la teinte des puits au niveau du lien : `P..P` pour un lien de
/// niveau 255, `e..e` pour un lien de niveau 51 (`e = (66, 80, 64)`, la couleur `E` de V-APER-04). Un cadre qui
/// prendrait l'étage le plus haut ailleurs que sur tous les nœuds (en partant de 0, ou du dernier nœud) donnerait une
/// image d'un seul panneau. Dessins faits à la main d'après la doc, recoupés par `ref_tubes.py`. Ajouté après la
/// campagne de fautes du juge.
///
/// Ne couvre pas : des positions négatives (voir `silhouette_du_treillis`, qui reprend le graphe de `etape_0_rendu`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn premier_noeud_a_l_etage_du_haut() {
    let g = graphe_a_la_main(&[(0, 0, 1), (0, 0, 0)], &[(0, 1)]);
    let palette = [('.', FOND), ('P', PUITS), ('e', [66, 80, 64])];
    let image = apercu(&g, &[1.0], 1.0, 1);
    assert_eq!((image.largeur, image.hauteur), (4, 1));
    assert_eq!(dessin(&image, 1, &palette), ["P..P"]);
    assert_eq!(dessin(&apercu(&g, &[0.2], 1.0, 1), 1, &palette), ["e..e"]);
    let image = apercu(&g, &[1.0], 1.0, 3);
    assert_eq!((image.largeur, image.hauteur), (12, 3));
    assert_eq!(dessin(&image, 3, &palette), ["P..P"]);
}

/// juste : un nœud d'où partent deux liens entre étages va vers la teinte des puits au niveau du PLUS ÉPAIS des deux
/// (V-APER-10, la doc : « `du_lien[i]` le plus grand `niv[e]` de ses arêtes entre deux étages différents »). Trois
/// nœuds à la même position sur trois étages, deux liens : avec les niveaux 255 (étages 0-1) et 51 (étages 1-2), le
/// nœud du milieu est à `PUITS` (niveau 255) : `P..P..e` ; avec 51 puis 255 : `e..P..P`. Trois panneaux d'un point et
/// deux écarts : 7 × 1. Dessins faits à la main d'après la doc, recoupés par `ref_tubes.py`. Ajouté après la campagne
/// de fautes du juge.
///
/// Ne couvre pas : plus de trois étages.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn deux_liens_le_plus_epais_compte() {
    let g = graphe_a_la_main(&[(0, 0, 0), (0, 0, 1), (0, 0, 2)], &[(0, 1), (1, 2)]);
    let palette = [('.', FOND), ('P', PUITS), ('e', [66, 80, 64])];
    let image = apercu(&g, &[1.0, 0.2], 1.0, 1);
    assert_eq!((image.largeur, image.hauteur), (7, 1));
    assert_eq!(dessin(&image, 1, &palette), ["P..P..e"]);
    assert_eq!(dessin(&apercu(&g, &[0.2, 1.0], 1.0, 1), 1, &palette), ["e..P..P"]);
}

/// juste : l'ordre des couches quand des points se recouvrent (V-APER-11, la doc : les milieux d'abord, puis les nœuds
/// par indice croissant, « deux nœuds au même point : le dernier recouvre », et le point d'un nœud part du fond, pas de
/// la couleur déjà posée). `Graphe::new` permet deux nœuds à la même position : A et B en `(0, 0)`, C en `(1, 0)`,
/// arêtes A-B, A-C, B-C. Le milieu de A-B tombe sur le point de A et de B. Avec les niveaux 51 (A-B), 255 (A-C) et 128
/// (B-C) : le point 0 reçoit le milieu de A-B (51), puis A (255, sa plus épaisse arête), puis B (128) : il reste à 128
/// ; le point 1 est le milieu de A-C et de B-C, le plus épais l'emporte (255) ; C est à 255 : `bcc`. Avec 255, 51 et
/// 128 : `cbb`. Dessins faits à la main d'après la doc, recoupés par `ref_tubes.py`. Ajouté après la campagne de fautes
/// du juge.
///
/// Ne couvre pas : deux nœuds au même point reliés à un autre étage.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn deux_noeuds_au_meme_point() {
    let g = graphe_a_la_main(&[(0, 0, 0), (0, 0, 0), (1, 0, 0)], &[(0, 1), (0, 2), (1, 2)]);
    let palette = [('.', N0), ('a', N51), ('b', N128), ('c', N255)];
    let image = apercu(&g, &[0.2, 1.0, 0.5], 1.0, 1);
    assert_eq!((image.largeur, image.hauteur), (3, 1));
    assert_eq!(dessin(&image, 1, &palette), ["bcc"]);
    assert_eq!(dessin(&apercu(&g, &[1.0, 0.2, 0.5], 1.0, 1), 1, &palette), ["cbb"]);
}
