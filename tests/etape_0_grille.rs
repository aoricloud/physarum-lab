//! Étape 0 : `graphe::Grille`, la grille à un ou plusieurs étages. Valeurs attendues : `references/etape-0-valeurs.md`,
//! identifiants `V-GRILLE-xx`.
//!
//! La source est la définition écrite dans la documentation de `graphe` (« La grille »). Les petits cas se lisent sur un
//! dessin ; les autres sont comparés à un générateur naïf écrit ici, dans un autre ordre de boucles que la définition.

mod common;

use physarum_lab::graphe::{Arete, ErreurGraphe, Grille, Noeud, Voisinage};

const RACINE_DE_DEUX: f64 = f64::from_bits(0x3FF6_A09E_667F_3BCD);

fn grille(largeur: u32, hauteur: u32, etages: u32, voisinage: Voisinage) -> Grille {
    Grille { largeur, hauteur, etages, voisinage, longueur_verticale: 0.5, cout_vertical: 3.0 }
}

fn arete(a: usize, b: usize, longueur: f64, cout: f64) -> Arete {
    Arete { a, b, longueur, cout }
}

/// Le générateur naïf : étage par étage, puis ligne par ligne, il pose les nœuds et relie chaque nœud à ses voisins de
/// gauche, du haut, des deux diagonales du haut et de l'étage du dessous ; puis il trie par insertion.
fn naive(g: &Grille) -> (Vec<Noeud>, Vec<Arete>) {
    let (l, h, n) = (g.largeur as usize, g.hauteur as usize, g.etages as usize);
    let indice = |x: usize, y: usize, e: usize| (y * l + x) * n + e;
    let mut noeuds = vec![Noeud { x: 0, y: 0, etage: 0 }; l * h * n];
    let mut aretes: Vec<Arete> = Vec::new();
    for e in 0..n {
        for y in 0..h {
            for x in 0..l {
                noeuds[indice(x, y, e)] = Noeud { x: x as i32, y: y as i32, etage: e as u32 };
                if x > 0 {
                    aretes.push(arete(indice(x - 1, y, e), indice(x, y, e), 1.0, 1.0));
                }
                if y > 0 {
                    aretes.push(arete(indice(x, y - 1, e), indice(x, y, e), 1.0, 1.0));
                }
                if g.voisinage == Voisinage::Huit && y > 0 {
                    if x > 0 {
                        aretes.push(arete(indice(x - 1, y - 1, e), indice(x, y, e), RACINE_DE_DEUX, RACINE_DE_DEUX));
                    }
                    if x + 1 < l {
                        aretes.push(arete(indice(x + 1, y - 1, e), indice(x, y, e), RACINE_DE_DEUX, RACINE_DE_DEUX));
                    }
                }
                if e > 0 {
                    aretes.push(arete(indice(x, y, e - 1), indice(x, y, e), g.longueur_verticale, g.cout_vertical));
                }
            }
        }
    }
    let mut triees: Vec<Arete> = Vec::new();
    for a in aretes {
        let place = triees.iter().position(|t| (t.a, t.b) > (a.a, a.b)).unwrap_or(triees.len());
        triees.insert(place, a);
    }
    (noeuds, triees)
}

/// juste : l'indice du nœud `(x, y, etage)` est `(y * largeur + x) * etages + etage`, et sa position est
/// `Noeud { x, y, etage }` (V-GRILLE-01, la formule de la doc) : sur une grille de 4 × 3 à 2 étages, `(0,0,0)` → 0,
/// `(0,0,1)` → 1, `(1,0,0)` → 2, `(3,0,1)` → 7, `(0,1,0)` → 8, `(3,2,1)` → 23.
///
/// Ne couvre pas : un nœud hors de la grille (voir `indice_hors_de_la_grille`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn indices_et_positions() {
    let g = grille(4, 3, 2, Voisinage::Quatre);
    for ((x, y, e), attendu) in [
        ((0, 0, 0), 0),
        ((0, 0, 1), 1),
        ((1, 0, 0), 2),
        ((3, 0, 1), 7),
        ((0, 1, 0), 8),
        ((2, 1, 1), 13),
        ((3, 2, 1), 23),
    ] {
        assert_eq!(g.indice(x, y, e), attendu, "({x}, {y}, {e})");
    }
    let graphe = g.graphe().unwrap();
    assert_eq!(graphe.nb_noeuds(), 24);
    for y in 0..3 {
        for x in 0..4 {
            for e in 0..2 {
                assert_eq!(graphe.noeuds()[g.indice(x, y, e)], Noeud { x: x as i32, y: y as i32, etage: e });
            }
        }
    }
    // un seul étage : l'indice est y * largeur + x
    let g = grille(5, 4, 1, Voisinage::Quatre);
    assert_eq!((g.indice(0, 0, 0), g.indice(4, 0, 0), g.indice(0, 1, 0), g.indice(4, 3, 0)), (0, 4, 5, 19));
}

/// juste : `indice` panique hors de la grille : `x == largeur`, `y == hauteur` ou `etage == etages` (la doc de `indice`,
/// V-GRILLE-01) ; chacun des trois est essayé, et le dernier nœud de la grille, lui, a un indice.
///
/// Ne couvre pas : le texte de la panique.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn indice_hors_de_la_grille() {
    let g = grille(4, 3, 2, Voisinage::Quatre);
    assert_eq!(g.indice(3, 2, 1), 23);
    for (x, y, e) in [(4, 0, 0), (0, 3, 0), (0, 0, 2), (4, 2, 1), (100, 100, 100)] {
        let resultat = std::panic::catch_unwind(|| g.indice(x, y, e));
        assert!(resultat.is_err(), "({x}, {y}, {e}) devrait paniquer");
    }
}

/// juste : les arêtes de trois petites grilles, écrites à la main d'après la définition (V-GRILLE-02) :
/// - 3 × 2, un étage, voisinage 4 : les sept arêtes du dessin `0-1-2 / 3-4-5` ;
/// - 2 × 2, un étage, voisinage 8 : les quatre côtés et les deux diagonales, de longueur et de coût `√2` ;
/// - 2 × 1, deux étages : une arête par étage et deux liens verticaux (longueur `0.5`, coût `3.0`).
///
/// Ne couvre pas : les grilles plus grandes (voir `contre_le_generateur_naif`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn aretes_de_petites_grilles() {
    let u = |a, b| arete(a, b, 1.0, 1.0);
    let d = |a, b| arete(a, b, RACINE_DE_DEUX, RACINE_DE_DEUX);
    let g = grille(3, 2, 1, Voisinage::Quatre).graphe().unwrap();
    assert_eq!(g.aretes(), [u(0, 1), u(0, 3), u(1, 2), u(1, 4), u(2, 5), u(3, 4), u(4, 5)]);
    let g = grille(2, 2, 1, Voisinage::Huit).graphe().unwrap();
    assert_eq!(g.aretes(), [u(0, 1), u(0, 2), d(0, 3), d(1, 2), u(1, 3), u(2, 3)]);
    // 2 × 1 à deux étages : indices (0,0,0)=0, (0,0,1)=1, (1,0,0)=2, (1,0,1)=3
    let g = grille(2, 1, 2, Voisinage::Huit).graphe().unwrap();
    assert_eq!(g.aretes(), [arete(0, 1, 0.5, 3.0), u(0, 2), u(1, 3), arete(2, 3, 0.5, 3.0)]);
    // un seul nœud : aucune arête
    let g = grille(1, 1, 1, Voisinage::Huit).graphe().unwrap();
    assert_eq!((g.nb_noeuds(), g.nb_aretes()), (1, 0));
    // une colonne de trois étages : deux liens verticaux, entre étages qui se suivent seulement
    let g = grille(1, 1, 3, Voisinage::Quatre).graphe().unwrap();
    assert_eq!(g.aretes(), [arete(0, 1, 0.5, 3.0), arete(1, 2, 0.5, 3.0)]);
}

/// juste : la diagonale vaut `2.0_f64.sqrt()`, de bits `0x3FF6A09E667F3BCD` (V-GRILLE-03 : la racine carrée d'IEEE 754 est
/// arrondie correctement, donc la même partout ; la valeur vient de Python `math.sqrt(2.0)`), pour la longueur et pour le
/// coût ; ce n'est ni `1.0`, ni `1.4142135623730951 * (1 + 2^-52)`.
///
/// Ne couvre pas : les autres longueurs (voir `aretes_de_petites_grilles`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn diagonale_racine_de_deux() {
    let g = grille(3, 3, 1, Voisinage::Huit).graphe().unwrap();
    let diagonales: Vec<&Arete> = g.aretes().iter().filter(|a| a.longueur != 1.0).collect();
    assert_eq!(diagonales.len(), 8, "deux diagonales par maille, quatre mailles");
    for a in diagonales {
        assert_eq!(a.longueur.to_bits(), 0x3FF6_A09E_667F_3BCD, "{a:?}");
        assert_eq!(a.cout.to_bits(), 0x3FF6_A09E_667F_3BCD, "{a:?}");
    }
}

/// juste : le nombre de nœuds est `largeur * hauteur * etages` et le nombre d'arêtes
/// `etages * ((largeur - 1) * hauteur + largeur * (hauteur - 1) + d) + (etages - 1) * largeur * hauteur`, avec
/// `d = 2 * (largeur - 1) * (hauteur - 1)` pour le voisinage 8 et 0 pour le voisinage 4 (V-GRILLE-04, un comptage : les
/// arêtes horizontales, verticales dans l'étage, diagonales, puis les liens entre étages) ; toute grille est connexe.
///
/// Ne couvre pas : quelles arêtes (voir `contre_le_generateur_naif`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn comptes_et_connexite() {
    for (l, h, n) in
        [(1, 1, 1), (5, 1, 1), (1, 6, 1), (4, 3, 1), (4, 3, 2), (7, 5, 3), (2, 2, 8), (1, 1, 4), (16, 9, 2)]
    {
        for voisinage in [Voisinage::Quatre, Voisinage::Huit] {
            let g = grille(l, h, n, voisinage).graphe().unwrap();
            let d = if voisinage == Voisinage::Huit { 2 * (l - 1) * (h - 1) } else { 0 };
            let attendu = n * ((l - 1) * h + l * (h - 1) + d) + (n - 1) * l * h;
            assert_eq!(g.nb_noeuds(), (l * h * n) as usize, "{l} × {h} × {n}");
            assert_eq!(g.nb_aretes(), attendu as usize, "{l} × {h} × {n} {voisinage:?}");
            assert!(g.est_connexe(), "{l} × {h} × {n}");
        }
    }
}

/// juste : la grille est, nœud pour nœud et arête pour arête (bouts, longueur, coût, ordre canonique), celle du générateur
/// naïf écrit dans ce fichier (V-GRILLE-05), pour des grilles de 1 à 3 étages, des deux voisinages, larges ou hautes.
///
/// Ne couvre pas : les grilles de plus de quelques centaines de nœuds.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn contre_le_generateur_naif() {
    for (l, h, n) in
        [(1, 1, 1), (2, 1, 1), (1, 2, 1), (3, 2, 1), (2, 3, 1), (5, 4, 1), (3, 2, 2), (2, 3, 2), (4, 4, 3), (6, 1, 2)]
    {
        for voisinage in [Voisinage::Quatre, Voisinage::Huit] {
            let g = grille(l, h, n, voisinage);
            let graphe = g.graphe().unwrap();
            let (noeuds, aretes) = naive(&g);
            assert_eq!(graphe.noeuds(), noeuds, "{l} × {h} × {n} {voisinage:?}");
            assert_eq!(graphe.aretes(), aretes, "{l} × {h} × {n} {voisinage:?}");
        }
    }
}

/// juste : la longueur et le coût d'un lien vertical sont ceux de la description, sans être échangés ni confondus avec ceux
/// des arêtes d'un étage ; un seul étage n'a aucun lien vertical (V-GRILLE-06). Sur 3 × 2 à 2 étages avec `0.25` et `7.0` :
/// six liens, tous `(0.25, 7.0)`, chacun entre les deux étages de la même case.
///
/// Ne couvre pas : plus de deux étages (voir `aretes_de_petites_grilles`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn liens_verticaux() {
    let g = Grille {
        largeur: 3,
        hauteur: 2,
        etages: 2,
        voisinage: Voisinage::Quatre,
        longueur_verticale: 0.25,
        cout_vertical: 7.0,
    };
    let graphe = g.graphe().unwrap();
    let noeuds = graphe.noeuds();
    let verticaux: Vec<&Arete> = graphe.aretes().iter().filter(|a| noeuds[a.a].etage != noeuds[a.b].etage).collect();
    assert_eq!(verticaux.len(), 6);
    for a in verticaux {
        assert_eq!((a.longueur, a.cout), (0.25, 7.0));
        assert_eq!((noeuds[a.a].x, noeuds[a.a].y), (noeuds[a.b].x, noeuds[a.b].y), "même case");
        assert_eq!((noeuds[a.a].etage, noeuds[a.b].etage), (0, 1));
    }
    for a in graphe.aretes().iter().filter(|a| noeuds[a.a].etage == noeuds[a.b].etage) {
        assert_eq!((a.longueur, a.cout), (1.0, 1.0));
    }
    let plat = Grille { etages: 1, ..g }.graphe().unwrap();
    assert!(plat.aretes().iter().all(|a| (a.longueur, a.cout) == (1.0, 1.0)));
    assert_eq!(plat.longueur_totale(), 7.0);
    assert_eq!(graphe.longueur_totale(), 15.5, "deux étages de 7 et six liens de 0,25");
}

/// juste : `Grille::graphe` rend `ErreurGraphe::Grille` pour une largeur, une hauteur ou un nombre d'étages nul, et pour une
/// longueur ou un coût vertical nul, négatif, infini ou NaN, même avec un seul étage ; la même grille sans la faute est
/// acceptée (V-GRILLE-07, la doc de `graphe`).
///
/// Ne couvre pas : les bornes hautes (c'est `scenario` qui les fixe).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn erreurs_de_grille() {
    let bonne = grille(3, 2, 2, Voisinage::Huit);
    assert!(bonne.graphe().is_ok());
    assert!(Grille { etages: 1, ..bonne }.graphe().is_ok());
    for fautive in [
        Grille { largeur: 0, ..bonne },
        Grille { hauteur: 0, ..bonne },
        Grille { etages: 0, ..bonne },
        Grille { longueur_verticale: 0.0, ..bonne },
        Grille { longueur_verticale: -1.0, ..bonne },
        Grille { longueur_verticale: f64::INFINITY, ..bonne },
        Grille { longueur_verticale: f64::NAN, ..bonne },
        Grille { cout_vertical: 0.0, ..bonne },
        Grille { cout_vertical: -3.0, ..bonne },
        Grille { cout_vertical: f64::INFINITY, ..bonne },
        Grille { cout_vertical: f64::NAN, ..bonne },
        Grille { etages: 1, longueur_verticale: 0.0, ..bonne },
        Grille { etages: 1, cout_vertical: f64::NAN, ..bonne },
    ] {
        assert_eq!(fautive.graphe().err(), Some(ErreurGraphe::Grille), "{fautive:?}");
    }
}

/// juste : l'écriture canonique de la grille 3 × 2 (un étage, voisinage 4), comparée aux 248 octets écrits par un modèle
/// Python indépendant de ce dépôt (V-GRILLE-08 : six nœuds, sept arêtes de longueur et de coût `1.0`).
///
/// Ne couvre pas : les autres grilles (leurs empreintes sont dans `etape_0_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn ecriture_canonique_de_la_grille() {
    let attendu = concat!(
        "06000000",
        "000000000000000000000000",
        "010000000000000000000000",
        "020000000000000000000000",
        "000000000100000000000000",
        "010000000100000000000000",
        "020000000100000000000000",
        "07000000",
        "0000000001000000000000000000f03f000000000000f03f",
        "0000000003000000000000000000f03f000000000000f03f",
        "0100000002000000000000000000f03f000000000000f03f",
        "0100000004000000000000000000f03f000000000000f03f",
        "0200000005000000000000000000f03f000000000000f03f",
        "0300000004000000000000000000f03f000000000000f03f",
        "0400000005000000000000000000f03f000000000000f03f",
    );
    let g = grille(3, 2, 1, Voisinage::Quatre).graphe().unwrap();
    assert_eq!(common::hex(&g.octets()), attendu);
}
