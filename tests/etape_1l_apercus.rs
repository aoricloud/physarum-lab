//! Étape 1, partie 1L : les aperçus des scénarios à murs et à étages (le PNG que `lancer` écrit). Valeurs attendues :
//! `references/etape-1l-valeurs.md`, identifiants `V-APL-xx`.
//!
//! La source est la doc de `tubes::apercu` (le treillis de `rendu`, la couleur d'un point dit l'épaisseur des tubes) et
//! de `lancer` (l'aperçu porte sur le graphe du **plan**, son cadre est celui de ses nœuds : **un mur est un trou**, le
//! treillis de `rendu` ne change pas). Les SHA-256 des octets RVB viennent du modèle Python `ref_labyrinthe.py` (le
//! treillis coloré, écrit d'après la doc ; jamais du prototype Rust) ; les dessins en caractères sont écrits ici
//! d'après la doc. Un aperçu est pour l'œil : ces tests disent qu'il est celui de la doc, pas qu'il est joli.

mod common;

use physarum_lab::alea::Alea;
use physarum_lab::empreinte::sha256_hex;
use physarum_lab::graphe::{Grille, Voisinage};
use physarum_lab::labyrinthe::{CoutEtage, Mur, construire};
use physarum_lab::lancer::lancer;
use physarum_lab::rendu::{FOND, PUITS, treillis};
use physarum_lab::scenario::lire_scenario;
use physarum_lab::tubes::{Tubes, VIF, apercu};

const RACINE: &str = env!("CARGO_MANIFEST_DIR");

/// Un cas en voisinage 8 à deux étages, deux coûts d'étage et deux murs sur deux étages.
const RICHE: &str = concat!(
    "grille 5 4",
    "\n",
    "voisinage 8",
    "\n",
    "etages 2 0.5 3.0",
    "\n",
    "cout_etage 0 2.0",
    "\n",
    "cout_etage 1 0.25",
    "\n",
    "mur 1 1 2 2 0",
    "\n",
    "mur 2 1 3 3 1",
    "\n",
    "source 0 0 0",
    "\n",
    "puits 4 3 1",
    "\n",
    "pas 12 0.4",
    "\n"
);

/// Un cas à poche fermée : l'anneau muré autour de `(2, 2)`, centre libre mais enfermé.
const POCHE: &str = concat!(
    "grille 5 5",
    "\n",
    "mur 1 1 3 1 0",
    "\n",
    "mur 1 3 3 3 0",
    "\n",
    "mur 1 2 1 2 0",
    "\n",
    "mur 3 2 3 2 0",
    "\n",
    "source 0 0 0",
    "\n",
    "puits 4 4 0",
    "\n",
    "pas 6 0.5",
    "\n"
);

fn fichier(nom: &str) -> String {
    std::fs::read_to_string(format!("{RACINE}/scenarios/{nom}")).unwrap()
}

/// Les pixels RVB de l'aperçu de `lancer` pour ce scénario : (largeur, hauteur, octets). Le PNG est relu par le
/// décodeur du test.
fn image(texte: &str, graine: u64) -> (u32, u32, Vec<u8>) {
    let sortie = lancer(&lire_scenario(texte).unwrap(), graine).unwrap();
    let lu = common::relire_png(&sortie.apercu).expect("PNG valide");
    (lu.largeur, lu.hauteur, lu.rvb)
}

/// La couleur du point `(col, ligne)` du treillis, dans une image agrandie `echelle` fois ; vérifie que tout le carré
/// est uni.
fn point(img: &(u32, u32, Vec<u8>), echelle: u32, col: u32, ligne: u32) -> [u8; 3] {
    let l = img.0 as usize;
    let pixel = |px: usize, py: usize| -> [u8; 3] {
        let i = 3 * (py * l + px);
        [img.2[i], img.2[i + 1], img.2[i + 2]]
    };
    let e = echelle as usize;
    let c = pixel(col as usize * e, ligne as usize * e);
    for dy in 0..e {
        for dx in 0..e {
            assert_eq!(pixel(col as usize * e + dx, ligne as usize * e + dy), c, "le carré du point ({col}, {ligne})");
        }
    }
    c
}

/// Le dessin en caractères : `.` le fond, `o` un nœud (point pair, pair) qui n'est pas du fond, `-` un autre point qui
/// n'est pas du fond.
fn dessin(img: &(u32, u32, Vec<u8>), echelle: u32) -> Vec<String> {
    assert_eq!((img.0 % echelle, img.1 % echelle), (0, 0));
    (0..img.1 / echelle)
        .map(|ligne| {
            (0..img.0 / echelle)
                .map(|col| match (point(img, echelle, col, ligne) == FOND, col % 2 == 0 && ligne % 2 == 0) {
                    (true, _) => '.',
                    (false, true) => 'o',
                    (false, false) => '-',
                })
                .collect::<String>()
        })
        .collect()
}

fn nb_pixels_de(img: &(u32, u32, Vec<u8>), couleur: [u8; 3]) -> usize {
    img.2.chunks(3).filter(|p| *p == couleur).count()
}

fn sha(img: &(u32, u32, Vec<u8>)) -> String {
    sha256_hex(&img.2)
}

/// juste : de petits aperçus dessinés en caractères (V-APL-01). **Un mur est un trou.** `grille 3 2` avec le mur `(1,
/// 0)` : le cadre est celui des cinq nœuds gardés (x de 0 à 2, y de 0 à 1 : un treillis de 5 × 3 points, échelle 8, 40
/// × 24 pixels) ; le point du nœud muré `(2, 0)` du treillis et les points des quatre arêtes qui le touchaient sont du
/// fond. La grille 5 × 5 à poche fermée (anneau muré de `(1,1)` à `(3,3)`, centre libre mais enfermé) donne un anneau
/// de 16 nœuds et de 16 arêtes, **le centre est un trou comme le mur qui l'entoure**. Les SHA-256 des octets RVB
/// (modèle Python) : `3416344f…6daf` pour le premier et `66320605…3d72` pour le second.
///
/// Ne couvre pas : un mur au bord du cadre (l'image rétrécit : voir `le_cadre_suit_les_noeuds_gardes`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn les_murs_sont_des_trous() {
    let mure = image("grille 3 2\nmur 1 0 1 0 0\nsource 0 0 0\npuits 2 0 0\npas 1 0.5\n", 7);
    assert_eq!((mure.0, mure.1), (40, 24));
    assert_eq!(dessin(&mure, 8), ["o...o", "-...-", "o-o-o"]);
    assert_eq!(sha(&mure), "3416344f721314aff083106488bf37b4fa0fbeb3a0639c1b84ee12ff9ec96daf");
    let poche = image(POCHE, 5);
    assert_eq!((poche.0, poche.1), (72, 72));
    let anneau = [
        "o-o-o-o-o",
        "-.......-",
        "o.......o",
        "-.......-",
        "o.......o",
        "-.......-",
        "o.......o",
        "-.......-",
        "o-o-o-o-o",
    ];
    assert_eq!(dessin(&poche, 8), anneau);
    assert_eq!(sha(&poche), "663206053878aae1bfa4beabf84407dee81be33697c80afa06dac5b218043d72");
}

/// juste : le cadre suit les nœuds gardés (V-APL-02). Un mur sur toute la première colonne d'une grille 4 × 3 retire
/// les nœuds `x = 0` : le treillis part de `x = 1` (un panneau de 5 × 5 points, 40 × 40 pixels à l'échelle 8), pas de
/// `x = 0` (7 × 5) ; le même mur sur la dernière ligne rétrécit la hauteur. Un mur sur tout l'étage 1 d'une grille à
/// deux étages retire le second panneau : l'image est celle d'un seul étage. Le dessin attendu du premier cas est un 3
/// × 3 de nœuds.
///
/// Ne couvre pas : les couleurs (voir `les_murs_sont_des_trous`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn le_cadre_suit_les_noeuds_gardes() {
    let sans_colonne = image("grille 4 3\nmur 0 0 0 2 0\nsource 1 0 0\npuits 3 2 0\npas 5 0.5\n", 1);
    assert_eq!((sans_colonne.0, sans_colonne.1), (40, 40));
    let d = dessin(&sans_colonne, 8);
    assert_eq!(d.len(), 5);
    assert_eq!(d[0].len(), 5);
    assert!(d.iter().all(|l| l.len() == 5), "{d:?}");
    let sans_ligne = image("grille 4 3\nmur 0 2 3 2 0\nsource 0 0 0\npuits 3 1 0\npas 5 0.5\n", 1);
    assert_eq!((sans_ligne.0, sans_ligne.1), (56, 24), "7 colonnes, 3 lignes de points");
    let un_etage = image("grille 3 2\netages 2 0.5 1.0\nmur 0 0 2 1 1\nsource 0 0 0\npuits 2 1 0\npas 5 0.5\n", 1);
    let deux_etages = image("grille 3 2\netages 2 0.5 1.0\nsource 0 0 0\npuits 2 1 0\npas 5 0.5\n", 1);
    assert_eq!((un_etage.0, un_etage.1), (40, 24), "un seul panneau de 5 × 3 points");
    assert_eq!((deux_etages.0, deux_etages.1), (96, 24), "(5 + 2 + 5) × 8 pixels de large");
}

/// juste : les aperçus du labyrinthe (V-APL-03) : 511 × 371 pixels (le cadre est celui des nœuds gardés : `x` de 2 à
/// 38, `y` de 1 à 27 : un treillis de 73 × 53 points, échelle `512 / 73 = 7`). Les SHA-256 des octets RVB, du modèle
/// Python : après 20 pas (graine 1) `f0190b82…d577` ; après 100 pas `221676d4…3e6` ; après 500 pas, pour les graines 1
/// et 2, **le même** `61c938bf…f694` (la convergence : l'image ne dépend plus de la graine), alors qu'à 20 pas les
/// graines 1 et 2 donnent deux images. Les points de couleur connue après 500 pas : chaque cellule murée **à
/// l'intérieur du cadre** est du fond ; les nœuds d'α2, `S` et `J` sont à la couleur vive `VIF` (le tube est à `D = 1`,
/// niveau 255), `T` est à `(60, 219, 254)` (le niveau 254 de β1 : `D = 0,996`) ; ceux d'α1 et des impasses, du fond
/// (des tubes au plancher : niveau 0, ils disparaissent). À 20 pas, un nœud d'α1 n'est pas du fond (`D = 0,11`) et son
/// vert est inférieur à celui d'un nœud d'α2, une impasse (au plancher depuis le pas 20) est du fond.
///
/// Ne couvre pas : l'aperçu d'un scénario sans murs (voir `etape_1_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn les_apercus_du_labyrinthe() {
    let texte = fichier("labyrinthe.txt");
    let a20 = image(&texte.replace("pas 500 0.5", "pas 20 0.5"), 1);
    let a100 = image(&texte.replace("pas 500 0.5", "pas 100 0.5"), 1);
    let a500 = image(&texte, 1);
    assert_eq!((a20.0, a20.1, a100.0, a100.1, a500.0, a500.1), (511, 371, 511, 371, 511, 371));
    assert_eq!(sha(&a20), "f0190b82c2b9460caa95d63c37254cc99d167f33225a9e3cdc7d0051f9e5d577");
    assert_eq!(sha(&a100), "221676d49a00b5e6814da303145a5539e8d4ba3bc24d5914c4a04d1e602ee3e6");
    assert_eq!(sha(&a500), "61c938bf2cf06030e7b145da15f29e1b26545bee8240f6f3b5a9c80baa11f694");
    assert_eq!(sha(&image(&texte, 2)), sha(&a500), "convergé : l'image ne dépend plus de la graine");
    assert_ne!(sha(&image(&texte.replace("pas 500 0.5", "pas 20 0.5"), 2)), sha(&a20), "à 20 pas, elle en dépend");
    // le treillis : le nœud `(x, y)` est au point `(2 (x - 2), 2 (y - 1))`
    let cellule = |img: &(u32, u32, Vec<u8>), x: u32, y: u32| point(img, 7, 2 * (x - 2), 2 * (y - 1));
    let dessin_du_labyrinthe = [
        "#########################################",
        "#####################################x###",
        "#####################################x###",
        "##x##################################x###",
        "##x#################BBB#BBB#BBB#BBB#BBB##",
        "##x#################B#B#B#B#B#B#B#B#B#B##",
        "##x#################B#B#B#B#B#B#B#B#B#B##",
        "##x#################B#B#B#B#B#B#B#B#B#B##",
        "##x#AAA#AAA#AAA#AAA#B#B#B#B#B#B#B#B#B#B##",
        "##x#A#A#A#A#A#A#A#A#B#B#B#B#B#B#B#B#B#B##",
        "##x#A#A#A#A#A#A#A#A#B#B#B#B#B#BBB#BBB#B##",
        "##x#A#A#A#A#A#A#A#A#B#BBB#BBB#########B##",
        "##x#A#A#A#A#A#A#A#A#B#################B##",
        "##x#A#A#A#A#A#A#A#A#B#################B##",
        "##SAA#AAA#AAA#AAA#AAJbb#bbb#bbb#bbb#bbT##",
        "##a#################a#b#b#b#b#b#b#b#b####",
        "##a#################a#b#b#b#b#b#b#b#b####",
        "##a#################a#b#b#b#b#b#b#b#b####",
        "##a#aaa#aaa#########a#b#b#b#b#b#b#b#b####",
        "##a#a#a#a#a#aaa#aaa#a#b#b#b#b#b#b#b#b####",
        "##a#a#a#a#a#a#a#a#a#a#b#b#b#b#b#b#b#b####",
        "##a#a#a#a#a#a#a#a#a#a#b#b#b#b#b#b#b#b####",
        "##a#a#a#a#a#a#a#a#a#a#b#b#b#b#b#b#bbb####",
        "##a#a#a#a#a#a#a#a#a#a#bbb#bbb#bbb########",
        "##aaa#aaa#aaa#aaa#aaa####################",
        "###########x#############################",
        "###########x#############################",
        "###########x#############################",
        "#########################################",
    ];
    let (mut murs, mut alpha2, mut alpha1, mut impasses, mut hubs) = (0, 0, 0, 0, 0);
    for y in 1..=27_u32 {
        for x in 2..=38_u32 {
            let c = dessin_du_labyrinthe[y as usize].as_bytes()[x as usize];
            match c {
                b'#' => {
                    assert_eq!(cellule(&a500, x, y), FOND, "le mur ({x}, {y})");
                    murs += 1;
                }
                b'A' => {
                    assert_eq!(cellule(&a500, x, y), VIF, "α2 ({x}, {y})");
                    alpha2 += 1;
                }
                b'a' => {
                    assert_eq!(cellule(&a500, x, y), FOND, "α1 ({x}, {y})");
                    alpha1 += 1;
                }
                b'x' => {
                    assert_eq!(cellule(&a500, x, y), FOND, "impasse ({x}, {y})");
                    impasses += 1;
                }
                b'S' | b'J' => {
                    assert_eq!(cellule(&a500, x, y), VIF, "{} ({x}, {y})", c as char);
                    hubs += 1;
                }
                b'T' => {
                    // le puits touche β1 (D = 0,996, niveau 254) et β2 (niveau 1) : son niveau est celui de β1, 254
                    assert_eq!(cellule(&a500, x, y), [60, 219, 254], "T ({x}, {y})");
                    hubs += 1;
                }
                _ => {}
            }
        }
    }
    assert_eq!((alpha2, alpha1, impasses, hubs), (65, 81, 17, 3), "α2 : 66 arêtes, 65 nœuds sans les extrémités");
    assert_eq!(murs, 37 * 27 - 342, "les cellules murées du cadre : 999 cellules moins 342 couloirs");
    // à 20 pas
    let (alpha1_20, alpha2_20) = (cellule(&a20, 2, 20), cellule(&a20, 4, 14));
    assert_ne!(alpha1_20, FOND, "α1 n'est pas encore éteinte à 20 pas");
    assert!(alpha1_20[1] < alpha2_20[1], "α1 est moins vive qu'α2 : {alpha1_20:?} contre {alpha2_20:?}");
    assert_eq!(cellule(&a20, 11, 26), FOND, "une impasse est déjà rétractée");
    // sans mur, le cadre serait celui de la grille entière : c'est le retrait des murs qui le fait rétrécir
    let grille = lire_scenario(&texte).unwrap().grille;
    assert_eq!(treillis(&grille.graphe().unwrap()), (81, 57));
}

/// juste : les aperçus à étages (V-APL-04) : `etages.txt` (le réseau monte : l'étage du haut est assez rapide) et
/// `etages-cher.txt` (il reste en bas), après 300 pas, graine 1 : 504 × 30 pixels (deux panneaux de 41 × 5 points
/// séparés de 2 colonnes, échelle 6), les SHA-256 RVB du modèle Python `d9d3bdd6…eb1a` et `bb90844e…4110`. Dans le
/// premier, les quatre nœuds des deux liens verticaux vifs (`(0, 1)` et `(20, 1)` aux deux étages) sont à la teinte
/// `PUITS` **exactement** (le lien est au niveau 255 : `melange(c, PUITS, 255)` est `PUITS`) : 4 points de 36 pixels,
/// pas un de plus dans l'image ; un nœud du milieu de l'étage du haut est à la couleur vive, le même point du bas est
/// du fond (le chemin du bas est mort). Dans le second, **aucun** pixel `PUITS` (les liens verticaux sont morts, niveau
/// 0 : ils laissent le point tel qu'il est), les nœuds `(0, 1)` du bas et `(5, 1)` du bas sont à la couleur vive et
/// ceux du haut du fond. Le pont (`etages-pont.txt`, 480 × 72, échelle 8, `086848e5…95e5`) : la colonne murée de
/// l'étage 0 est un trou, les liens `(2, 2)` et `(12, 2)` sont à la teinte `PUITS` aux deux étages (4 points de 64
/// pixels), les nœuds de la route du haut sont vifs.
///
/// Ne couvre pas : plus de deux étages.
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn les_apercus_a_etages() {
    let monte = image(&fichier("etages.txt"), 1);
    let cher = image(&fichier("etages-cher.txt"), 1);
    assert_eq!((monte.0, monte.1, cher.0, cher.1), (504, 30, 504, 30));
    assert_eq!(sha(&monte), "d9d3bdd6e607fcd131a62f583454d9349462c12ada8412d0e8572c0864d2eb1a");
    assert_eq!(sha(&cher), "bb90844ef9f091510ae0afa9a128a7c17cc8b81d9c28f64fc04d82a34bd34110");
    // le treillis : le panneau de l'étage `e` commence à la colonne `43 e` ; le nœud `(x, y)` est au point `(43 e + 2
    // x, 2 y)`
    let noeud = |img: &(u32, u32, Vec<u8>), x: u32, y: u32, e: u32| point(img, 6, 43 * e + 2 * x, 2 * y);
    for (x, e) in [(0, 0), (0, 1), (20, 0), (20, 1)] {
        assert_eq!(noeud(&monte, x, 1, e), PUITS, "lien vif en ({x}, 1) à l'étage {e}");
    }
    assert_eq!(nb_pixels_de(&monte, PUITS), 4 * 36);
    assert_eq!(noeud(&monte, 5, 1, 1), VIF);
    assert_eq!(noeud(&monte, 5, 1, 0), FOND);
    assert_eq!(nb_pixels_de(&cher, PUITS), 0);
    assert_eq!(noeud(&cher, 0, 1, 0), VIF);
    assert_eq!(noeud(&cher, 5, 1, 0), VIF);
    assert_eq!(noeud(&cher, 5, 1, 1), FOND);
    assert_eq!(noeud(&cher, 0, 1, 1), FOND);
    // le pont
    let pont = image(&fichier("etages-pont.txt"), 1);
    assert_eq!((pont.0, pont.1), (480, 72));
    assert_eq!(sha(&pont), "086848e500427e7e5c101d182bff6e73cbb98a3d3f0855041cc11c19d27895e5");
    let n = |x: u32, y: u32, e: u32| point(&pont, 8, 31 * e + 2 * x, 2 * y);
    for y in 0..5 {
        assert_eq!(n(7, y, 0), FOND, "la colonne murée de l'étage 0 : un trou en (7, {y})");
    }
    for (x, e) in [(2, 0), (2, 1), (12, 0), (12, 1)] {
        assert_eq!(n(x, 2, e), PUITS, "lien vif en ({x}, 2) à l'étage {e}");
    }
    assert_eq!(nb_pixels_de(&pont, PUITS), 4 * 64);
    assert_eq!(n(7, 2, 1), VIF, "la route du haut passe au-dessus du mur");
    assert_eq!(n(5, 2, 1), VIF);
    assert_eq!(n(7, 0, 1), FOND, "un nœud du haut hors de la route est éteint");
}

/// juste : l'aperçu de `lancer` est celui de `tubes::apercu` appliqué au graphe du plan et aux conductivités
/// recalculées par le test (V-APL-05) : `construire` puis `Tubes::au_hasard` et `Tubes::avancer` (éprouvés ailleurs),
/// pour un cas en voisinage 8 à deux étages avec deux coûts d'étage et deux murs, graine 3, 12 pas de 0,4 (160 × 56
/// pixels ; SHA-256 RVB du modèle Python `eb504a2c…f72d`), et pour le labyrinthe à 20 pas. L'image est la même octet
/// pour octet, et l'échelle est `512 / TL` du plan, ramenée entre 1 et 8.
///
/// Ne couvre pas : un autre `h`.
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn lancer_peint_le_graphe_du_plan() {
    let g = Grille {
        largeur: 5,
        hauteur: 4,
        etages: 2,
        voisinage: Voisinage::Huit,
        longueur_verticale: 0.5,
        cout_vertical: 3.0,
    };
    let murs = [Mur { x0: 1, y0: 1, x1: 2, y1: 2, etage: 0 }, Mur { x0: 2, y0: 1, x1: 3, y1: 3, etage: 1 }];
    let couts = [CoutEtage { etage: 0, facteur: 2.0 }, CoutEtage { etage: 1, facteur: 0.25 }];
    let plan = construire(&g, &murs, &couts, g.indice(0, 0, 0), g.indice(4, 3, 1)).unwrap();
    let mut tubes = Tubes::au_hasard(&plan.graphe, &mut Alea::new(3));
    for _ in 0..12 {
        tubes.avancer(&plan.graphe, plan.source, plan.puits, 1.0, 0.4).unwrap();
    }
    let echelle = (512 / treillis(&plan.graphe).0).clamp(1, 8);
    let attendu = apercu(&plan.graphe, tubes.conductivites(), 1.0, echelle);
    let texte = RICHE;
    let lu = image(texte, 3);
    assert_eq!((lu.0, lu.1), (attendu.largeur, attendu.hauteur));
    assert_eq!((lu.0, lu.1), (160, 56));
    assert_eq!(lu.2, attendu.rvb);
    assert_eq!(sha(&lu), "eb504a2c5e7edc75f71f506d2681d4ae9bd324d5017ca5f570dd03d84cedf72d");
    // un graphe plus large que le plan complet : le labyrinthe, 20 pas
    let texte = fichier("labyrinthe.txt").replace("pas 500 0.5", "pas 20 0.5");
    let scenario = lire_scenario(&texte).unwrap();
    let mut murs = Vec::new();
    for d in scenario.directives.iter().filter(|d| d.nom == "mur") {
        let v: Vec<u32> = (0..5).map(|i| d.entier(i).unwrap() as u32).collect();
        murs.push(Mur { x0: v[0], y0: v[1], x1: v[2], y1: v[3], etage: v[4] });
    }
    let plan =
        construire(&scenario.grille, &murs, &[], scenario.grille.indice(2, 14, 0), scenario.grille.indice(38, 14, 0))
            .unwrap();
    let mut tubes = Tubes::au_hasard(&plan.graphe, &mut Alea::new(1));
    for _ in 0..20 {
        tubes.avancer(&plan.graphe, plan.source, plan.puits, 1.0, 0.5).unwrap();
    }
    assert_eq!(treillis(&plan.graphe), (73, 53));
    assert_eq!(image(&texte, 1).2, apercu(&plan.graphe, tubes.conductivites(), 1.0, 7).rvb);
}
