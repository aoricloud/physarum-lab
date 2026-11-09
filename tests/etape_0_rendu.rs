//! Étape 0 : `rendu`, l'aperçu d'un graphe, ses étages côte à côte. Valeurs attendues : `references/etape-0-valeurs.md`,
//! identifiants `V-REND-xx`.
//!
//! La source est la définition écrite dans la documentation de `rendu`. Chaque treillis attendu est dessiné ici en
//! caractères, d'après cette définition : `.` le fond, `-` le point d'une arête, `o` celui d'un nœud, `P` celui d'un nœud
//! d'où part un lien vers un autre étage. Les dessins des grilles ont été recoupés par un modèle Python indépendant.

use physarum_lab::graphe::{Arete, Graphe, Grille, Noeud, Voisinage};
use physarum_lab::rendu::{ARETE, ECART, FOND, Image, NOEUD, PUITS, apercu_graphe, treillis};

fn grille(largeur: u32, hauteur: u32, etages: u32, voisinage: Voisinage) -> Graphe {
    Grille { largeur, hauteur, etages, voisinage, longueur_verticale: 1.0, cout_vertical: 1.0 }.graphe().unwrap()
}

fn noeud(x: i32, y: i32, etage: u32) -> Noeud {
    Noeud { x, y, etage }
}

fn arete(a: usize, b: usize) -> Arete {
    Arete { a, b, longueur: 1.0, cout: 1.0 }
}

/// Le dessin en caractères d'une image agrandie `echelle` fois : un caractère par point du treillis. Vérifie au passage
/// que l'image a la bonne longueur et que chaque carré de `echelle × echelle` pixels est d'une seule couleur connue.
fn dessin(image: &Image, echelle: u32) -> Vec<String> {
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
            ligne.push(match couleur {
                c if c == FOND => '.',
                c if c == ARETE => '-',
                c if c == NOEUD => 'o',
                c if c == PUITS => 'P',
                autre => panic!("couleur inconnue {autre:?} au point ({tx}, {ty})"),
            });
        }
        lignes.push(ligne);
    }
    lignes
}

/// juste : les couleurs et l'écart sont ceux de la doc (V-REND-01) : fond `(12, 16, 24)`, arête `(90, 110, 130)`, nœud
/// `(225, 225, 215)`, puits `(240, 170, 40)`, écart de 2 colonnes ; les quatre couleurs sont différentes.
///
/// Ne couvre pas : le dessin (tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn couleurs_et_ecart() {
    assert_eq!((FOND, ARETE, NOEUD, PUITS), ([12, 16, 24], [90, 110, 130], [225, 225, 215], [240, 170, 40]));
    assert_eq!(ECART, 2);
}

/// juste : la grille 3 × 2 à un étage (V-REND-02) : un treillis de 5 × 3 points, les nœuds aux points pairs, le point de
/// chaque arête entre ses deux nœuds, le fond au centre de chaque maille en voisinage 4 ; en voisinage 8 les deux
/// diagonales d'une maille se croisent au centre, qui devient un point d'arête. À l'échelle 1, un pixel par point :
/// le premier pixel est un nœud, le second une arête.
///
/// Ne couvre pas : l'agrandissement (voir `agrandissement`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn grille_a_un_etage() {
    let g = grille(3, 2, 1, Voisinage::Quatre);
    assert_eq!(treillis(&g), (5, 3));
    let image = apercu_graphe(&g, 1);
    assert_eq!((image.largeur, image.hauteur), (5, 3));
    assert_eq!(dessin(&image, 1), ["o-o-o", "-.-.-", "o-o-o"]);
    assert_eq!(image.rvb[..6], [225, 225, 215, 90, 110, 130]);
    let g = grille(3, 2, 1, Voisinage::Huit);
    assert_eq!(dessin(&apercu_graphe(&g, 1), 1), ["o-o-o", "-----", "o-o-o"]);
    let g = grille(2, 3, 1, Voisinage::Quatre);
    assert_eq!(treillis(&g), (3, 5));
    assert_eq!(dessin(&apercu_graphe(&g, 1), 1), ["o-o", "-.-", "o-o", "-.-", "o-o"]);
    let g = grille(1, 1, 1, Voisinage::Quatre);
    assert_eq!(treillis(&g), (1, 1));
    assert_eq!(apercu_graphe(&g, 1), Image { largeur: 1, hauteur: 1, rvb: NOEUD.to_vec() });
    let g = grille(4, 1, 1, Voisinage::Huit);
    assert_eq!(dessin(&apercu_graphe(&g, 1), 1), ["o-o-o-o"]);
}

/// juste : les étages côte à côte (V-REND-03) : la grille 3 × 2 à 2 étages donne deux panneaux de 5 points séparés par 2
/// colonnes de fond, soit un treillis de 12 × 3 ; chaque nœud, d'où part un lien vertical, est un puits ; à 3 étages et
/// 3 × 3 : 19 × 5 points. L'étage 0 est à gauche.
///
/// Ne couvre pas : un étage sans nœud (voir `graphe_quelconque`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn etages_cote_a_cote() {
    let g = grille(3, 2, 2, Voisinage::Quatre);
    assert_eq!(treillis(&g), (12, 3));
    assert_eq!(dessin(&apercu_graphe(&g, 1), 1), ["P-P-P..P-P-P", "-.-.-..-.-.-", "P-P-P..P-P-P"]);
    let g = grille(3, 2, 2, Voisinage::Huit);
    assert_eq!(dessin(&apercu_graphe(&g, 1), 1), ["P-P-P..P-P-P", "-----..-----", "P-P-P..P-P-P"]);
    let g = grille(3, 3, 3, Voisinage::Quatre);
    assert_eq!(treillis(&g), (19, 5));
    assert_eq!(
        dessin(&apercu_graphe(&g, 1), 1),
        [
            "P-P-P..P-P-P..P-P-P",
            "-.-.-..-.-.-..-.-.-",
            "P-P-P..P-P-P..P-P-P",
            "-.-.-..-.-.-..-.-.-",
            "P-P-P..P-P-P..P-P-P",
        ]
    );
}

/// juste : un graphe quelconque (V-REND-04, dessiné à la main d'après la doc et recoupé par le modèle Python). Huit nœuds :
/// 0 `(0,-1,0)`, 1 `(-1,-2,0)`, 2 `(1,-2,0)`, 3 `(0,-2,0)`, 4 `(1,0,2)`, 5 `(2,-1,0)`, 6 `(-1,-2,2)`, 7 `(0,-1,0)`. Le cadre
/// part de `xmin = -1` et `ymin = -2`, que porte le nœud 1 et non le premier ; `xmax = 2`, `ymax = 0` et l'étage 2 ne
/// viennent pas du dernier nœud ; l'étage 1, sans nœud, garde son panneau : 3 panneaux de 7 × 5, soit 25 × 5 points.
/// Arêtes : `1-3` et `0-3` (voisins : leur milieu est dessiné), `2-5` (une diagonale), `3-5` (trop longue : son milieu
/// `(4, 1)` reste du fond), `0-7` (deux nœuds à la même position : le milieu est le point du nœud, que le nœud recouvre),
/// `1-6` (un lien entre étages à la même position : deux puits) et `4-5` (un lien entre étages de positions voisines :
/// deux puits, et aucun point d'arête, dans aucun panneau).
///
/// Ne couvre pas : les positions au-delà de quelques unités.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn graphe_quelconque() {
    let noeuds = vec![
        noeud(0, -1, 0),
        noeud(-1, -2, 0),
        noeud(1, -2, 0),
        noeud(0, -2, 0),
        noeud(1, 0, 2),
        noeud(2, -1, 0),
        noeud(-1, -2, 2),
        noeud(0, -1, 0),
    ];
    let aretes = vec![arete(1, 3), arete(3, 5), arete(0, 3), arete(0, 7), arete(2, 5), arete(1, 6), arete(4, 5)];
    let g = Graphe::new(noeuds, aretes).unwrap();
    assert_eq!(treillis(&g), (25, 5));
    assert_eq!(
        dessin(&apercu_graphe(&g, 1), 1),
        [
            "P-o.o.............P......",
            "..-..-...................",
            "..o...P..................",
            ".........................",
            "......................P..",
        ]
    );
    // un seul nœud loin de l'origine : le cadre se réduit à lui
    let seul = Graphe::new(vec![noeud(1000, -1000, 0)], vec![]).unwrap();
    assert_eq!(treillis(&seul), (1, 1));
    assert_eq!(dessin(&apercu_graphe(&seul, 1), 1), ["o"]);
    // un seul nœud à l'étage 2 : trois panneaux d'un point, dont deux vides
    let haut = Graphe::new(vec![noeud(5, 5, 2)], vec![]).unwrap();
    assert_eq!(treillis(&haut), (7, 1));
    assert_eq!(dessin(&apercu_graphe(&haut, 1), 1), ["......o"]);
}

/// juste : l'agrandissement (V-REND-05) : à l'échelle `k`, chaque point du treillis est un carré uni de `k × k` pixels,
/// l'image fait `TL * k` sur `th * k`, et le dessin relu par carrés est celui de l'échelle 1 ; essayé pour `k` de 1 à 5 sur
/// une grille 3 × 2 à 2 étages (image de `12k × 3k`).
///
/// Ne couvre pas : les échelles au-delà de 8.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn agrandissement() {
    let g = grille(3, 2, 2, Voisinage::Quatre);
    let attendu = dessin(&apercu_graphe(&g, 1), 1);
    assert_eq!(attendu, ["P-P-P..P-P-P", "-.-.-..-.-.-", "P-P-P..P-P-P"]);
    for k in 1..=5 {
        let image = apercu_graphe(&g, k);
        assert_eq!((image.largeur, image.hauteur), (12 * k, 3 * k), "échelle {k}");
        assert_eq!(dessin(&image, k), attendu, "échelle {k}");
    }
    // à l'échelle 2, les quatre premiers pixels de la première ligne : deux du puits, deux de l'arête
    let image = apercu_graphe(&g, 2);
    assert_eq!(image.rvb[..12], [240, 170, 40, 240, 170, 40, 90, 110, 130, 90, 110, 130]);
}

/// juste : `apercu_graphe` panique pour une échelle nulle (la doc, V-REND-05) ; l'échelle 1, elle, rend une image.
///
/// Ne couvre pas : le texte de la panique.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn echelle_nulle() {
    let g = grille(2, 2, 1, Voisinage::Quatre);
    assert_eq!(apercu_graphe(&g, 1).largeur, 3);
    let resultat = std::panic::catch_unwind(|| apercu_graphe(&g, 0).largeur);
    assert!(resultat.is_err(), "une échelle nulle devrait paniquer");
}
