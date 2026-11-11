//! Étape 1, partie 1L : `labyrinthe::construire`, le plan d'une grille murée et à étages. Valeurs attendues :
//! `references/etape-1l-valeurs.md`, identifiants `V-LAB-xx`.
//!
//! La source est la documentation de `labyrinthe` (« La construction », « Le coût d'un étage », « Les erreurs »). Les
//! graphes attendus sont **écrits à la main** (le test dit la renumérotation et l'ordre des arêtes), de petites grilles
//! murées sont dessinées en caractères (`#` un mur, `.` une cellule libre, `S` et `T` la source et le puits), et une
//! construction naïve écrite dans ce fichier (une relaxation jusqu'à stabilité à la place d'un parcours, la géométrie
//! des voisins à la place de `Grille::graphe`) est comparée sur des grilles tirées au hasard.

use physarum_lab::alea::Alea;
use physarum_lab::chemins::plus_courts;
use physarum_lab::graphe::{Graphe, Grille, Noeud, Voisinage};
use physarum_lab::labyrinthe::{CoutEtage, ErreurLabyrinthe, FACTEUR_MAX, FACTEUR_MIN, Mur, Plan, construire};

fn grille(largeur: u32, hauteur: u32, etages: u32, voisinage: Voisinage, lv: f64, cv: f64) -> Grille {
    Grille { largeur, hauteur, etages, voisinage, longueur_verticale: lv, cout_vertical: cv }
}

fn mur(x0: u32, y0: u32, x1: u32, y1: u32, etage: u32) -> Mur {
    Mur { x0, y0, x1, y1, etage }
}

fn cout(etage: u32, facteur: f64) -> CoutEtage {
    CoutEtage { etage, facteur }
}

/// Une grille à un étage dessinée en caractères : `#` un mur (un `Mur` d'une cellule), `S` la source, `T` le puits,
/// tout le reste libre. Rend (grille 4 voisins, murs, source, puits en indices de grille).
fn dessin(lignes: &[&str]) -> (Grille, Vec<Mur>, usize, usize) {
    let g = grille(lignes[0].len() as u32, lignes.len() as u32, 1, Voisinage::Quatre, 1.0, 1.0);
    let (mut murs, mut s, mut t) = (Vec::new(), None, None);
    for (y, ligne) in lignes.iter().enumerate() {
        assert_eq!(ligne.len() as u32, g.largeur, "lignes de même longueur");
        for (x, c) in ligne.chars().enumerate() {
            let (x, y) = (x as u32, y as u32);
            match c {
                '#' => murs.push(mur(x, y, x, y, 0)),
                'S' => s = Some(g.indice(x, y, 0)),
                'T' => t = Some(g.indice(x, y, 0)),
                _ => {}
            }
        }
    }
    (g, murs, s.expect("une source S"), t.expect("un puits T"))
}

/// Compare le graphe d'un plan à ce qu'on attend, **au bit près** : les nœuds `(x, y, étage)` dans l'ordre des indices
/// et les arêtes `(a, b, longueur, coût)` dans l'ordre des indices d'arête.
#[track_caller]
fn verifie(graphe: &Graphe, noeuds: &[(i32, i32, u32)], aretes: &[(usize, usize, f64, f64)]) {
    assert!(!noeuds.is_empty() && !aretes.is_empty(), "un attendu vide ne prouve rien");
    let lus: Vec<(i32, i32, u32)> = graphe.noeuds().iter().map(|n| (n.x, n.y, n.etage)).collect();
    assert_eq!(lus, noeuds, "les nœuds");
    assert_eq!(graphe.nb_aretes(), aretes.len(), "le nombre d'arêtes");
    for (e, (lue, attendue)) in graphe.aretes().iter().zip(aretes).enumerate() {
        assert_eq!((lue.a, lue.b), (attendue.0, attendue.1), "les bouts de l'arête {e}");
        assert_eq!(lue.longueur.to_bits(), attendue.2.to_bits(), "la longueur de l'arête {e}");
        assert_eq!(lue.cout.to_bits(), attendue.3.to_bits(), "le coût de l'arête {e}");
    }
}

fn plan(g: &Grille, murs: &[Mur], couts: &[CoutEtage], s: usize, t: usize) -> Plan {
    construire(g, murs, couts, s, t).unwrap_or_else(|e| panic!("construire : {e:?}"))
}

fn erreur(g: &Grille, murs: &[Mur], couts: &[CoutEtage], s: usize, t: usize) -> ErreurLabyrinthe {
    construire(g, murs, couts, s, t).expect_err("une erreur était attendue")
}

/// juste : sans mur ni coût d'étage, le plan est exactement `Grille::graphe` (V-LAB-01) : mêmes nœuds, mêmes arêtes,
/// mêmes octets de `Graphe::octets` (longueurs et coûts compris, les diagonales en `√2`, un lien vertical de longueur
/// et de coût à part) ; la correspondance est l'identité `Some(0), Some(1), …` ; la source et le puits gardent leurs
/// indices. Une grille 4 × 3, une grille 3 × 3 à deux étages en voisinage 8, une grille 2 × 2 à trois étages, une
/// grille 2 × 1 : le même résultat avec un coût d'étage de `1.0` donné explicitement (`cout * 1.0` rend les mêmes
/// bits).
///
/// Ne couvre pas : les grilles murées (tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn sans_mur_c_est_la_grille() {
    let cas = [
        grille(4, 3, 1, Voisinage::Quatre, 1.0, 1.0),
        grille(3, 3, 2, Voisinage::Huit, 0.5, 2.0),
        grille(2, 2, 3, Voisinage::Quatre, 0.25, 0.75),
        grille(2, 1, 1, Voisinage::Huit, 1.0, 1.0),
    ];
    for g in cas {
        let complet = g.graphe().unwrap();
        let n = complet.nb_noeuds();
        assert!(n >= 2 && complet.nb_aretes() >= 1);
        for (s, t) in [(0, n - 1), (n - 1, 0)] {
            let p = plan(&g, &[], &[], s, t);
            assert_eq!(p.graphe, complet, "{g:?}");
            assert_eq!(p.graphe.octets(), complet.octets(), "{g:?} : les octets");
            let identite: Vec<Option<usize>> = (0..n).map(Some).collect();
            assert_eq!(p.noeuds, identite, "{g:?} : la correspondance");
            assert_eq!((p.source, p.puits), (s, t), "{g:?}");
        }
        // un facteur de 1.0 donné à chaque étage : les mêmes bits
        let tous: Vec<CoutEtage> = (0..g.etages).map(|e| cout(e, 1.0)).collect();
        assert_eq!(plan(&g, &[], &tous, 0, n - 1).graphe.octets(), complet.octets(), "{g:?} : facteurs 1.0");
    }
}

/// juste : un mur d'une seule cellule au centre d'une grille 3 × 3 (V-LAB-02) : le nœud du milieu (indice 4) disparaît
/// avec ses quatre arêtes ; les huit nœuds restants sont renumérotés dans l'ordre croissant de leur indice de grille
/// (`0 1 2 3 5 6 7 8` deviennent `0 1 2 3 4 5 6 7`) ; la correspondance est `[0, 1, 2, 3, None, 4, 5, 6, 7]` ; les huit
/// arêtes de l'anneau sont `(0,1) (0,3) (1,2) (2,5) (3,6) (5,8) (6,7) (7,8)` dans la grille et `(0,1) (0,3) (1,2) (2,4)
/// (3,5) (4,7) (5,6) (6,7)` dans le plan : **le même ordre relatif** (c'est l'ordre canonique de `Graphe::new` conservé
/// par la renumérotation) ; les positions `(x, y)` ne changent pas (le mur est un trou). La source `(0,0)` est le nœud
/// 0, le puits `(2,2)` le nœud 7.
///
/// Ne couvre pas : les poches fermées (voir `poche_fermee_retiree`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn un_mur_d_une_cellule() {
    let g = grille(3, 3, 1, Voisinage::Quatre, 1.0, 1.0);
    let p = plan(&g, &[mur(1, 1, 1, 1, 0)], &[], g.indice(0, 0, 0), g.indice(2, 2, 0));
    let noeuds = [(0, 0, 0), (1, 0, 0), (2, 0, 0), (0, 1, 0), (2, 1, 0), (0, 2, 0), (1, 2, 0), (2, 2, 0)];
    let aretes: Vec<(usize, usize, f64, f64)> = [(0, 1), (0, 3), (1, 2), (2, 4), (3, 5), (4, 7), (5, 6), (6, 7)]
        .iter()
        .map(|&(a, b)| (a, b, 1.0, 1.0))
        .collect();
    verifie(&p.graphe, &noeuds, &aretes);
    let attendu = [Some(0), Some(1), Some(2), Some(3), None, Some(4), Some(5), Some(6), Some(7)];
    assert_eq!(p.noeuds, attendu);
    assert_eq!((p.source, p.puits), (0, 7));
    // l'ordre relatif des arêtes : la k-ième arête du plan est la k-ième arête gardée de la grille complète
    let complet = g.graphe().unwrap();
    let gardees: Vec<(usize, usize)> =
        complet.aretes().iter().filter(|e| e.a != 4 && e.b != 4).map(|e| (e.a, e.b)).collect();
    let inverse: Vec<usize> = (0..9).filter(|i| attendu[*i].is_some()).collect();
    let du_plan: Vec<(usize, usize)> = p.graphe.aretes().iter().map(|e| (inverse[e.a], inverse[e.b])).collect();
    assert_eq!(du_plan, gardees);
}

/// juste : une poche fermée disparaît avec ses arêtes, même sans mur (V-LAB-03). La grille 5 × 5 dont on mure l'anneau
/// `x, y` dans `1..=3` sauf le centre `(2, 2)` (deux murs de trois cellules, deux d'une) laisse la cellule `(2, 2)`
/// libre mais enfermée : elle n'est pas dans la composante du puits, donc `None` dans la correspondance, et le plan est
/// l'anneau extérieur, un cycle de 16 nœuds et de 16 arêtes. Le test écrit le dessin, le nombre de nœuds, de arêtes, la
/// correspondance de `(2,2)` et des quatre coins, et que chaque nœud du plan a exactement deux arêtes. Un couloir mort
/// (une impasse) relié au puits, lui, reste.
///
/// Ne couvre pas : la composante quand la source est dans la poche (voir `erreurs_dans_leur_ordre`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn poche_fermee_retiree() {
    let (g, murs, s, t) = dessin(&["S....", ".###.", ".#.#.", ".###.", "....T"]);
    assert_eq!(murs.len(), 8);
    let p = plan(&g, &murs, &[], s, t);
    assert_eq!((p.graphe.nb_noeuds(), p.graphe.nb_aretes()), (16, 16));
    assert_eq!(p.noeuds[g.indice(2, 2, 0)], None, "la poche");
    let coins = [g.indice(0, 0, 0), g.indice(4, 0, 0), g.indice(0, 4, 0), g.indice(4, 4, 0)];
    assert_eq!(coins.map(|i| p.noeuds[i]), [Some(0), Some(4), Some(11), Some(15)]);
    assert_eq!((p.source, p.puits), (0, 15));
    assert_eq!(p.noeuds.iter().filter(|c| c.is_some()).count(), 16);
    for i in 0..16 {
        assert_eq!(p.graphe.incidentes(i).len(), 2, "le nœud {i} de l'anneau");
    }
    assert!(p.graphe.est_connexe());
    // le même dessin, une impasse libre de plus (un couloir de deux cellules collé à l'anneau) : elle reste
    let (g, murs, s, t) = dessin(&["S.....", ".###..", ".#.#..", ".###..", "....T."]);
    let p = plan(&g, &murs, &[], s, t);
    assert_eq!(p.noeuds.iter().filter(|c| c.is_some()).count(), 16 + 5, "l'anneau, plus la colonne x = 5");
    assert_eq!(p.noeuds[g.indice(2, 2, 0)], None);
    // une poche sans mur d'un côté : une cellule libre isolée dans un coin (0, 0) par deux murs
    let (g, murs, s, t) = dessin(&[".#..", "#...", "...T", "...S"]);
    let p = plan(&g, &murs, &[], s, t);
    assert_eq!(p.noeuds[g.indice(0, 0, 0)], None, "la cellule du coin, isolée");
    assert_eq!(p.graphe.nb_noeuds(), 16 - 3);
}

/// juste : deux murs qui se recouvrent retirent l'union de leurs cellules, rien de plus (V-LAB-04). Grille 5 × 2 : `mur
/// 1 0 2 0` et `mur 2 0 3 0` (elles se recouvrent en `(2, 0)`) retirent `(1..=3, 0)` ; le plan est le même qu'avec le
/// seul mur `1 0 3 0` ou qu'avec le premier donné deux fois. Les nœuds gardés sont `(0,0) (4,0) (0,1) … (4,1)`, les six
/// arêtes `(0,2) (1,6) (2,3) (3,4) (4,5) (5,6)` ; la source `(0,0)` est le nœud 0, le puits `(4,0)` le nœud 1.
///
/// Ne couvre pas : les murs d'étages différents (voir `un_mur_sur_tout_un_etage`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn deux_murs_qui_se_recouvrent() {
    let g = grille(5, 2, 1, Voisinage::Quatre, 1.0, 1.0);
    let (s, t) = (g.indice(0, 0, 0), g.indice(4, 0, 0));
    let deux = plan(&g, &[mur(1, 0, 2, 0, 0), mur(2, 0, 3, 0, 0)], &[], s, t);
    let noeuds = [(0, 0, 0), (4, 0, 0), (0, 1, 0), (1, 1, 0), (2, 1, 0), (3, 1, 0), (4, 1, 0)];
    let aretes: Vec<(usize, usize, f64, f64)> =
        [(0, 2), (1, 6), (2, 3), (3, 4), (4, 5), (5, 6)].iter().map(|&(a, b)| (a, b, 1.0, 1.0)).collect();
    verifie(&deux.graphe, &noeuds, &aretes);
    assert_eq!((deux.puits, deux.source), (1, 0));
    assert_eq!(deux, plan(&g, &[mur(1, 0, 3, 0, 0)], &[], s, t));
    assert_eq!(deux, plan(&g, &[mur(1, 0, 2, 0, 0), mur(1, 0, 2, 0, 0), mur(2, 0, 3, 0, 0)], &[], s, t));
    assert_eq!(deux, plan(&g, &[mur(2, 0, 3, 0, 0), mur(1, 0, 2, 0, 0)], &[], s, t), "l'ordre des murs ne compte pas");
}

/// juste : un mur couvre tout un étage (V-LAB-05) : grille 3 × 2 à deux étages (liens de longueur 0,5 et de coût 2),
/// `mur 0 0 2 1` sur l'étage 1 : il ne reste que l'étage 0, une grille 3 × 2 ordinaire (6 nœuds, 7 arêtes `(0,1) (0,3)
/// (1,2) (1,4) (2,5) (3,4) (4,5)`) : les liens verticaux ont disparu avec les cellules de l'étage 1 ; la correspondance
/// garde les indices pairs. Le même mur sur l'étage 0 avec le puits à l'étage 0 est `PuitsMure` ; avec le puits à
/// l'étage 1 et la source à l'étage 0, c'est `SourceMuree`. Un mur sur l'étage 1 ne retire pas les cellules de l'étage
/// 0.
///
/// Ne couvre pas : les murs partiels d'un étage (voir `un_mur_d_une_cellule`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn un_mur_sur_tout_un_etage() {
    let g = grille(3, 2, 2, Voisinage::Quatre, 0.5, 2.0);
    let (s, t) = (g.indice(0, 0, 0), g.indice(2, 1, 0));
    let p = plan(&g, &[mur(0, 0, 2, 1, 1)], &[], s, t);
    let noeuds = [(0, 0, 0), (1, 0, 0), (2, 0, 0), (0, 1, 0), (1, 1, 0), (2, 1, 0)];
    let aretes: Vec<(usize, usize, f64, f64)> =
        [(0, 1), (0, 3), (1, 2), (1, 4), (2, 5), (3, 4), (4, 5)].iter().map(|&(a, b)| (a, b, 1.0, 1.0)).collect();
    verifie(&p.graphe, &noeuds, &aretes);
    let attendu: Vec<Option<usize>> = (0..12).map(|i| if i % 2 == 0 { Some(i / 2) } else { None }).collect();
    assert_eq!(p.noeuds, attendu);
    assert_eq!((p.source, p.puits), (0, 5));
    let mur0 = [mur(0, 0, 2, 1, 0)];
    assert_eq!(erreur(&g, &mur0, &[], g.indice(0, 0, 1), t), ErreurLabyrinthe::PuitsMure);
    assert_eq!(erreur(&g, &mur0, &[], s, g.indice(2, 1, 1)), ErreurLabyrinthe::SourceMuree);
    // le même mur sur l'étage 0 : la source et le puits à l'étage 1 se joignent par l'étage 1
    let haut = plan(&g, &mur0, &[], g.indice(0, 0, 1), g.indice(2, 1, 1));
    assert_eq!(haut.graphe.nb_noeuds(), 6);
    assert!(haut.graphe.noeuds().iter().all(|n| n.etage == 1));
}

/// juste : la source et le puits voisins d'un mur, avec un passage (V-LAB-06) : grille 5 × 3, un mur de `(2,0)` à
/// `(2,1)`, la source `(1,1)` et le puits `(3,1)` de chaque côté. 13 nœuds gardés, 16 arêtes (22 moins les 6 qui
/// touchent `(2,0)` ou `(2,1)`) ; le plus court chemin de la source au puits passe sous le mur : 4 arêtes `(1,1) (1,2)
/// (2,2) (3,2) (3,1)`, de longueur totale 4. La source et le puits ne sont pas séparés : `Ok`.
///
/// Ne couvre pas : le cas où le mur ferme le passage (voir `erreurs_dans_leur_ordre`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn la_source_et_le_puits_voisins_d_un_mur() {
    let g = grille(5, 3, 1, Voisinage::Quatre, 1.0, 1.0);
    let (s, t) = (g.indice(1, 1, 0), g.indice(3, 1, 0));
    let p = plan(&g, &[mur(2, 0, 2, 1, 0)], &[], s, t);
    assert_eq!((p.graphe.nb_noeuds(), p.graphe.nb_aretes()), (13, 16));
    assert_eq!(p.noeuds[g.indice(2, 0, 0)], None);
    assert_eq!(p.noeuds[g.indice(2, 1, 0)], None);
    let chemins = plus_courts(&p.graphe, p.source);
    assert_eq!(chemins.distances[p.puits], 4.0);
    let aretes = chemins.vers(&p.graphe, p.puits).unwrap();
    assert_eq!(aretes.len(), 4);
    let sommets: Vec<(i32, i32)> = {
        let mut v = vec![p.source];
        for &e in &aretes {
            let a = p.graphe.aretes()[e];
            let dernier = *v.last().unwrap();
            v.push(if a.a == dernier { a.b } else { a.a });
        }
        v.iter().map(|&i| (p.graphe.noeuds()[i].x, p.graphe.noeuds()[i].y)).collect()
    };
    assert_eq!(sommets, [(1, 1), (1, 2), (2, 2), (3, 2), (3, 1)]);
}

/// juste : le coût d'un étage (V-LAB-07) : grille 3 × 2 en voisinage 8 à deux étages (liens de longueur 0,5 et de coût
/// 3), facteur 0,3 à l'étage 1. Une arête de chaque sorte : orthogonale de l'étage 0 (coût 1, inchangé), orthogonale de
/// l'étage 1 (`1 * 0,3`), diagonale de l'étage 0 (`√2`), diagonale de l'étage 1 (`√2 * 0,3`, un seul produit), lien
/// vertical (coût 3, **inchangé**) ; les **longueurs** ne changent pour aucune. Avec `[(0, 2.0), (1, 4.0)]` : les deux
/// étages sont multipliés, le lien vertical reste. Un étage sans facteur garde ses coûts (`1.0` pour l'autre étage
/// quand un seul est donné : étage 2 d'une grille à trois étages), et l'ordre des facteurs dans la liste ne compte pas.
///
/// Ne couvre pas : l'effet du facteur sur la dynamique (voir `etape_1l_etages`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn le_cout_d_un_etage() {
    let g = grille(3, 2, 2, Voisinage::Huit, 0.5, 3.0);
    let (s, t) = (g.indice(0, 0, 0), g.indice(2, 1, 0));
    let racine = 2.0_f64.sqrt();
    let complet = g.graphe().unwrap();
    for (facteurs, f0, f1) in [([cout(1, 0.3)].to_vec(), 1.0, 0.3), ([cout(0, 2.0), cout(1, 4.0)].to_vec(), 2.0, 4.0)] {
        let p = plan(&g, &[], &facteurs, s, t);
        assert_eq!(p.graphe.nb_aretes(), complet.nb_aretes());
        let (mut orthogonales, mut diagonales, mut verticales) = ([0, 0], [0, 0], 0);
        for (lue, base) in p.graphe.aretes().iter().zip(complet.aretes()) {
            assert_eq!((lue.a, lue.b), (base.a, base.b));
            assert_eq!(lue.longueur.to_bits(), base.longueur.to_bits(), "la longueur ne change pas");
            let (na, nb) = (p.graphe.noeuds()[lue.a], p.graphe.noeuds()[lue.b]);
            if na.etage != nb.etage {
                assert_eq!(lue.cout.to_bits(), 3.0_f64.to_bits(), "un lien vertical garde son coût");
                verticales += 1;
                continue;
            }
            let facteur = if na.etage == 0 { f0 } else { f1 };
            let diagonale = na.x != nb.x && na.y != nb.y;
            let attendu = if diagonale { racine * facteur } else { 1.0 * facteur };
            assert_eq!(lue.cout.to_bits(), attendu.to_bits(), "arête {:?}-{:?}", na, nb);
            let classe = if diagonale { &mut diagonales } else { &mut orthogonales };
            classe[na.etage as usize] += 1;
        }
        // une arête de chaque sorte au moins, dans chaque étage
        assert_eq!((orthogonales, diagonales, verticales), ([7, 7], [4, 4], 6));
    }
    let a = plan(&g, &[], &[cout(0, 2.0), cout(1, 4.0)], s, t);
    let b = plan(&g, &[], &[cout(1, 4.0), cout(0, 2.0)], s, t);
    assert_eq!(a, b, "l'ordre des facteurs ne compte pas");
    // trois étages, un seul facteur
    let g3 = grille(2, 1, 3, Voisinage::Quatre, 1.0, 5.0);
    let p = plan(&g3, &[], &[cout(1, 0.5)], g3.indice(0, 0, 0), g3.indice(1, 0, 0));
    let verifie_cout = |de: (u32, u32), vers: (u32, u32), attendu: f64| {
        let a = p.noeuds[g3.indice(de.0, 0, de.1)].unwrap();
        let b = p.noeuds[g3.indice(vers.0, 0, vers.1)].unwrap();
        let e = p.graphe.arete_entre(a, b).expect("l'arête existe");
        assert_eq!(p.graphe.aretes()[e].cout.to_bits(), attendu.to_bits(), "{de:?}-{vers:?}");
    };
    verifie_cout((0, 0), (1, 0), 1.0);
    verifie_cout((0, 1), (1, 1), 0.5);
    verifie_cout((0, 2), (1, 2), 1.0);
    verifie_cout((0, 0), (0, 1), 5.0);
    verifie_cout((1, 1), (1, 2), 5.0);
}

/// juste : les murs ne retirent que les cellules de leur étage, et un facteur ne change que son étage, ensemble
/// (V-LAB-08) : grille 4 × 1 à deux étages (lien de longueur 1 et de coût 2), `mur 1 0 1 0` sur l'étage 1, facteur 0,5
/// à l'étage 1, source `(0,0)` étage 0, puits `(3,0)` étage 0. Le graphe attendu, écrit à la main, a 7 nœuds (les huit
/// cellules moins `(1,0,1)`) renumérotés `(0,0,0)=0 (0,0,1)=1 (1,0,0)=2 (2,0,0)=3 (2,0,1)=4 (3,0,0)=5 (3,0,1)=6`, et 7
/// arêtes : `(0,1)` vertical, `(0,2)`, `(1,0,1)` n'existant plus, `(2,3)`, `(3,4)` vertical, `(3,5)`, `(4,6)` à coût
/// 0,5, `(5,6)` vertical.
///
/// Ne couvre pas : un facteur sur un étage entièrement muré.
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn murs_et_etages_ensemble() {
    let g = grille(4, 1, 2, Voisinage::Quatre, 1.0, 2.0);
    let p = plan(&g, &[mur(1, 0, 1, 0, 1)], &[cout(1, 0.5)], g.indice(0, 0, 0), g.indice(3, 0, 0));
    let noeuds = [(0, 0, 0), (0, 0, 1), (1, 0, 0), (2, 0, 0), (2, 0, 1), (3, 0, 0), (3, 0, 1)];
    let aretes = [
        (0, 1, 1.0, 2.0),
        (0, 2, 1.0, 1.0),
        (2, 3, 1.0, 1.0),
        (3, 4, 1.0, 2.0),
        (3, 5, 1.0, 1.0),
        (4, 6, 1.0, 0.5),
        (5, 6, 1.0, 2.0),
    ];
    verifie(&p.graphe, &noeuds, &aretes);
    assert_eq!(p.noeuds[g.indice(1, 0, 1)], None);
    assert_eq!((p.source, p.puits), (0, 5));
}

/// juste : les erreurs et leur ordre (V-LAB-09), chacune seule puis en concurrence. Grille 4 × 3 à deux étages.
/// `Grille` : une description fautive (largeur 0, coût vertical 0) l'emporte sur tout, même sur un mur fautif. `Mur(i)`
/// : le plus petit rang fautif, pour chacune des cinq fautes (`x0 > x1`, `y0 > y1`, `x1 = largeur`, `y1 = hauteur`,
/// `etage = etages`) mise au rang 1 derrière un mur juste ; deux murs fautifs : le premier ; une valeur énorme
/// (`u32::MAX`) ne panique pas. `Facteur(i)` : un étage `>= etages`, un facteur `0.09`, `10.5`, NaN, l'infini, `0`,
/// `-1`, un étage donné deux fois (le second est fautif, même avec un bon facteur) ; les bornes `0.1` et `10.0` sont
/// acceptées. `Mur` l'emporte sur `Facteur`, `Facteur` sur `Noeud`. `Noeud` : source ou puits hors de la grille
/// (`usize::MAX` compris), ou les deux égaux. `PuitsMure` avant `SourceMuree` avant `SourceCoupee`, et `Noeud` avant
/// tout cela.
///
/// Ne couvre pas : les erreurs de `lancer` (voir `etape_1l_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn erreurs_dans_leur_ordre() {
    let g = grille(4, 3, 2, Voisinage::Quatre, 1.0, 1.0);
    let (s, t) = (g.indice(0, 0, 0), g.indice(3, 2, 0));
    let bon = mur(1, 1, 1, 1, 0);
    // Grille
    for fautive in [
        grille(0, 3, 2, Voisinage::Quatre, 1.0, 1.0),
        grille(4, 0, 2, Voisinage::Quatre, 1.0, 1.0),
        grille(4, 3, 0, Voisinage::Quatre, 1.0, 1.0),
        grille(4, 3, 2, Voisinage::Quatre, 0.0, 1.0),
        grille(4, 3, 2, Voisinage::Quatre, 1.0, 0.0),
        grille(4, 3, 2, Voisinage::Quatre, 1.0, f64::NAN),
    ] {
        assert_eq!(erreur(&fautive, &[mur(9, 9, 9, 9, 9)], &[cout(9, 0.0)], 99, 99), ErreurLabyrinthe::Grille);
    }
    // Mur(i), chaque faute au rang 1
    let fautes = [
        mur(2, 0, 1, 0, 0),
        mur(0, 2, 0, 1, 0),
        mur(0, 0, 4, 0, 0),
        mur(0, 0, 0, 3, 0),
        mur(0, 0, 0, 0, 2),
        mur(0, 0, u32::MAX, 0, 0),
        mur(u32::MAX, 0, u32::MAX, 0, 0),
        mur(0, 0, 0, 0, u32::MAX),
    ];
    for faute in fautes {
        assert_eq!(erreur(&g, &[bon, faute], &[], s, t), ErreurLabyrinthe::Mur(1), "{faute:?}");
        assert_eq!(erreur(&g, &[faute, bon], &[], s, t), ErreurLabyrinthe::Mur(0), "{faute:?}");
        assert_eq!(erreur(&g, &[bon, faute, fautes[0]], &[], s, t), ErreurLabyrinthe::Mur(1), "le premier fautif");
        // un mur fautif l'emporte sur un facteur fautif et sur des nœuds fautifs
        assert_eq!(erreur(&g, &[faute], &[cout(9, 1.0)], usize::MAX, usize::MAX), ErreurLabyrinthe::Mur(0));
    }
    // les bornes des murs sont comprises : le coin `(3, 2)` et l'étage 1
    assert!(construire(&g, &[mur(3, 2, 3, 2, 1), mur(0, 0, 3, 2, 1)], &[], s, t).is_ok());
    // Facteur(i)
    let mauvais = [
        cout(2, 1.0),
        cout(u32::MAX, 1.0),
        cout(0, 0.09),
        cout(0, 10.5),
        cout(0, f64::NAN),
        cout(0, f64::INFINITY),
        cout(0, 0.0),
        cout(0, -1.0),
        cout(0, f64::NEG_INFINITY),
    ];
    for faute in mauvais {
        assert_eq!(erreur(&g, &[], &[cout(1, 2.0), faute], s, t), ErreurLabyrinthe::Facteur(1), "{faute:?}");
        assert_eq!(erreur(&g, &[], &[faute], s, t), ErreurLabyrinthe::Facteur(0), "{faute:?}");
        assert_eq!(erreur(&g, &[], &[faute, mauvais[0]], s, t), ErreurLabyrinthe::Facteur(0), "le premier fautif");
        assert_eq!(erreur(&g, &[], &[faute], usize::MAX, s), ErreurLabyrinthe::Facteur(0), "avant Noeud");
    }
    assert_eq!(erreur(&g, &[], &[cout(1, 2.0), cout(1, 3.0)], s, t), ErreurLabyrinthe::Facteur(1), "étage en double");
    assert_eq!(erreur(&g, &[], &[cout(0, 0.5), cout(1, 2.0), cout(0, 0.5)], s, t), ErreurLabyrinthe::Facteur(2));
    assert_eq!(erreur(&g, &[bon, mur(0, 0, 9, 0, 0)], &[cout(9, 1.0)], s, t), ErreurLabyrinthe::Mur(1));
    assert_eq!((FACTEUR_MIN, FACTEUR_MAX), (0.1, 10.0));
    assert!(construire(&g, &[], &[cout(0, 0.1), cout(1, 10.0)], s, t).is_ok(), "les bornes sont comprises");
    // Noeud
    let n = g.graphe().unwrap().nb_noeuds();
    for (a, b) in [(n, t), (s, n), (usize::MAX, t), (s, usize::MAX), (s, s), (n, n)] {
        assert_eq!(erreur(&g, &[], &[], a, b), ErreurLabyrinthe::Noeud, "({a}, {b})");
        assert_eq!(erreur(&g, &[bon], &[], a, b), ErreurLabyrinthe::Noeud, "avant les murés");
    }
    // PuitsMure, SourceMuree, SourceCoupee : l'ordre
    let sur_s = mur(0, 0, 0, 0, 0);
    let sur_t = mur(3, 2, 3, 2, 0);
    assert_eq!(erreur(&g, &[sur_t], &[], s, t), ErreurLabyrinthe::PuitsMure);
    assert_eq!(erreur(&g, &[sur_s], &[], s, t), ErreurLabyrinthe::SourceMuree);
    assert_eq!(erreur(&g, &[sur_s, sur_t], &[], s, t), ErreurLabyrinthe::PuitsMure, "le puits d'abord");
    // un mur qui enferme la source : la source est libre mais coupée du puits (un mur sur toute la colonne 1 de l'étage
    // 0 et de l'étage 1 sépare la colonne 0 du reste)
    let coupe = [mur(1, 0, 1, 2, 0), mur(1, 0, 1, 2, 1)];
    assert_eq!(erreur(&g, &coupe, &[], s, t), ErreurLabyrinthe::SourceCoupee);
    assert_eq!(erreur(&g, &coupe, &[], g.indice(0, 1, 1), t), ErreurLabyrinthe::SourceCoupee, "étage du haut aussi");
    // avec la colonne 1 murée à l'étage 0 seulement, le passage existe par l'étage 1
    assert!(construire(&g, &coupe[..1], &[], s, t).is_ok(), "le passage par l'étage du haut");
    assert_eq!(erreur(&g, &[sur_s, sur_t], &[cout(9, 1.0)], s, t), ErreurLabyrinthe::Facteur(0));
}

// --- une construction naïve, écrite ici, et des grilles tirées au hasard ---------------------------------------------

/// Les coordonnées `(x, y, étage)` du nœud `i` d'une grille (indice `(y * largeur + x) * etages + etage`).
fn coord(g: &Grille, i: usize) -> (i64, i64, i64) {
    let (l, e) = (g.largeur as usize, g.etages as usize);
    ((i / e % l) as i64, (i / e / l) as i64, (i % e) as i64)
}

/// Les voisins par la géométrie seule, sans `Grille::graphe` : `Some((longueur, coût de base))` si les deux cellules
/// sont reliées, `None` sinon.
fn lien(g: &Grille, a: usize, b: usize) -> Option<(f64, f64)> {
    let ((xa, ya, ea), (xb, yb, eb)) = (coord(g, a), coord(g, b));
    let (dx, dy, de) = ((xa - xb).abs(), (ya - yb).abs(), (ea - eb).abs());
    if de == 1 && dx == 0 && dy == 0 {
        return Some((g.longueur_verticale, g.cout_vertical));
    }
    if de != 0 {
        return None;
    }
    match (g.voisinage, dx, dy) {
        (_, 1, 0) | (_, 0, 1) => Some((1.0, 1.0)),
        (Voisinage::Huit, 1, 1) => Some((2.0_f64.sqrt(), 2.0_f64.sqrt())),
        _ => None,
    }
}

type Naif = Result<(Vec<(i32, i32, u32)>, Vec<(usize, usize, f64, f64)>, Vec<Option<usize>>), ErreurLabyrinthe>;

/// La construction de la doc, écrite autrement : les murs cellule par cellule, la composante du puits par relaxation
/// jusqu'à stabilité, les arêtes par paires de cellules gardées (a < b) dans l'ordre lexicographique.
fn naif(g: &Grille, murs: &[Mur], couts: &[CoutEtage], s: usize, t: usize) -> Naif {
    let positif = |v: f64| v.is_finite() && v > 0.0;
    if g.largeur == 0 || g.hauteur == 0 || g.etages == 0 || !positif(g.longueur_verticale) || !positif(g.cout_vertical)
    {
        return Err(ErreurLabyrinthe::Grille);
    }
    let n = (g.largeur * g.hauteur * g.etages) as usize;
    if let Some(i) = murs
        .iter()
        .position(|m| m.x0 > m.x1 || m.y0 > m.y1 || m.x1 >= g.largeur || m.y1 >= g.hauteur || m.etage >= g.etages)
    {
        return Err(ErreurLabyrinthe::Mur(i));
    }
    let mut facteur = vec![1.0; g.etages as usize];
    for (i, c) in couts.iter().enumerate() {
        let dehors = c.etage >= g.etages || !(0.1..=10.0).contains(&c.facteur);
        let double = couts[..i].iter().any(|d| d.etage == c.etage);
        if dehors || double {
            return Err(ErreurLabyrinthe::Facteur(i));
        }
        facteur[c.etage as usize] = c.facteur;
    }
    if s >= n || t >= n || s == t {
        return Err(ErreurLabyrinthe::Noeud);
    }
    let mure = |i: usize| {
        let (x, y, e) = coord(g, i);
        murs.iter().any(|m| {
            (m.x0 as i64..=m.x1 as i64).contains(&x) && (m.y0 as i64..=m.y1 as i64).contains(&y) && m.etage as i64 == e
        })
    };
    if mure(t) {
        return Err(ErreurLabyrinthe::PuitsMure);
    }
    if mure(s) {
        return Err(ErreurLabyrinthe::SourceMuree);
    }
    let mut garde: Vec<bool> = (0..n).map(|i| i == t).collect();
    loop {
        let mut change = false;
        for a in 0..n {
            for b in 0..n {
                if garde[a] && !garde[b] && !mure(b) && lien(g, a, b).is_some() {
                    garde[b] = true;
                    change = true;
                }
            }
        }
        if !change {
            break;
        }
    }
    if !garde[s] {
        return Err(ErreurLabyrinthe::SourceCoupee);
    }
    let mut corr = vec![None; n];
    let mut noeuds = Vec::new();
    for i in 0..n {
        if garde[i] {
            corr[i] = Some(noeuds.len());
            let (x, y, e) = coord(g, i);
            noeuds.push((x as i32, y as i32, e as u32));
        }
    }
    let mut aretes = Vec::new();
    for a in 0..n {
        for b in a + 1..n {
            if let (true, true, Some((longueur, base))) = (garde[a], garde[b], lien(g, a, b)) {
                let (ea, eb) = (coord(g, a).2, coord(g, b).2);
                let cout = if ea == eb { base * facteur[ea as usize] } else { base };
                aretes.push((corr[a].unwrap(), corr[b].unwrap(), longueur, cout));
            }
        }
    }
    Ok((noeuds, aretes, corr))
}

/// juste : sur des grilles tirées au hasard, `construire` est la construction naïve du test (V-LAB-10) : mêmes nœuds,
/// mêmes arêtes (au bit), même correspondance, même source et même puits renumérotés ; ou la même erreur, au même rang.
/// 800 cas (graine de l'aléa fixée) : largeur 1 à 7, hauteur 1 à 6, un à trois étages, voisinage 4 ou 8, zéro à huit
/// murs en rectangles (un sur soixante à bornes fautives), parfois des cellules murées une à une, un coût par étage au
/// plus (quelques facteurs, étages ou doublons fautifs), source et puits tirés (parfois hors de la grille). Le test
/// exige assez de cas de chaque issue pour ne pas passer à vide : au moins 150 plans réussis dont 30 avec au moins une
/// cellule retirée qui n'est pas murée (une poche ou une impasse coupée), et au moins 20 de chacune des six issues
/// d'erreur (mesuré : 216 plans réussis dont 49 avec une cellule libre retirée ; erreurs `Mur` 58, `Facteur` 52,
/// `Noeud` 127, `PuitsMure` 224, `SourceMuree` 84, `SourceCoupee` 39).
///
/// Ne couvre pas : les grilles de plus de 126 cellules.
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn contre_la_construction_naive() {
    let mut alea = Alea::new(20_261_111);
    let mut tirer = |n: u32| alea.borne(n);
    let (mut reussis, mut avec_poche) = (0, 0);
    let mut erreurs: Vec<ErreurLabyrinthe> = Vec::new();
    let facteurs = [0.1, 0.3, 0.5, 1.0, 2.0, 3.7, 10.0];
    for _ in 0..800 {
        let (largeur, hauteur, etages) = (1 + tirer(7), 1 + tirer(6), 1 + tirer(3));
        let voisinage = if tirer(2) == 0 { Voisinage::Quatre } else { Voisinage::Huit };
        let lv = [0.5, 1.0, 2.0][tirer(3) as usize];
        let cv = [0.25, 1.0, 3.0][tirer(3) as usize];
        let g = grille(largeur, hauteur, etages, voisinage, lv, cv);
        let n = (largeur * hauteur * etages) as usize;
        let (s, t) = (tirer(n as u32 + 1) as usize, tirer(n as u32 + 1) as usize);
        let mut murs = Vec::new();
        for _ in 0..tirer(9) {
            let (x0, y0) = (tirer(largeur), tirer(hauteur));
            let (x1, y1) = (x0 + tirer(largeur - x0), y0 + tirer(hauteur - y0));
            let mut m = mur(x0, y0, x1, y1, tirer(etages));
            if tirer(60) == 0 {
                m.x1 += largeur; // hors de la grille
            }
            murs.push(m);
        }
        if tirer(2) == 0 {
            // des cellules murées une à une, avec une densité tirée : elles font des poches et des impasses
            let densite = 20 + tirer(40);
            for i in 0..n {
                if tirer(100) < densite && (i != s && i != t || tirer(15) == 0) {
                    let (x, y, e) = coord(&g, i);
                    murs.push(mur(x as u32, y as u32, x as u32, y as u32, e as u32));
                }
            }
        }
        let mut couts = Vec::new();
        let decalage = tirer(etages);
        for k in 0..tirer(etages + 1) {
            let mut etage = (k + decalage) % etages;
            if tirer(40) == 0 {
                etage = etages; // hors de la grille
            } else if k > 0 && tirer(40) == 0 {
                etage = (k - 1 + decalage) % etages; // un étage déjà donné
            }
            let facteur = if tirer(40) == 0 { 12.0 } else { facteurs[tirer(facteurs.len() as u32) as usize] };
            couts.push(cout(etage, facteur));
        }
        let attendu = naif(&g, &murs, &couts, s, t);
        match (construire(&g, &murs, &couts, s, t), attendu) {
            (Ok(p), Ok((noeuds, aretes, corr))) => {
                assert!(noeuds.len() >= 2, "la source et le puits sont deux nœuds gardés");
                verifie(&p.graphe, &noeuds, &aretes);
                assert_eq!(p.noeuds, corr, "{g:?} {murs:?} {couts:?}");
                assert_eq!((Some(p.source), Some(p.puits)), (corr[s], corr[t]));
                assert!(p.source != p.puits);
                reussis += 1;
                let murees = (0..n).filter(|&i| murs.iter().any(|m| coord_dans(&g, i, m))).count();
                if corr.iter().filter(|c| c.is_none()).count() > murees {
                    avec_poche += 1;
                }
            }
            (Err(lue), Err(attendue)) => {
                assert_eq!(lue, attendue, "{g:?} {murs:?} {couts:?} ({s}, {t})");
                erreurs.push(lue);
            }
            (lu, attendu) => panic!("{g:?} {murs:?} {couts:?} ({s}, {t}) : {lu:?} contre {attendu:?}"),
        }
    }
    let compte = |e: fn(&ErreurLabyrinthe) -> bool| erreurs.iter().filter(|x| e(x)).count();
    let comptes = [
        compte(|e| matches!(e, ErreurLabyrinthe::Mur(_))),
        compte(|e| matches!(e, ErreurLabyrinthe::Facteur(_))),
        compte(|e| *e == ErreurLabyrinthe::Noeud),
        compte(|e| *e == ErreurLabyrinthe::PuitsMure),
        compte(|e| *e == ErreurLabyrinthe::SourceMuree),
        compte(|e| *e == ErreurLabyrinthe::SourceCoupee),
    ];
    assert!(reussis >= 150, "{reussis} plans réussis ; erreurs {comptes:?} ; poches {avec_poche}");
    assert!(avec_poche >= 30, "{avec_poche} plans avec une cellule libre retirée");
    assert!(comptes.iter().all(|&c| c >= 20), "issues rares : {comptes:?}");
}

fn coord_dans(g: &Grille, i: usize, m: &Mur) -> bool {
    let (x, y, e) = coord(g, i);
    (m.x0 as i64..=m.x1 as i64).contains(&x) && (m.y0 as i64..=m.y1 as i64).contains(&y) && m.etage as i64 == e
}

/// juste : l'ordre des arêtes d'un plan est celui de la grille complète (V-LAB-11) : sur 100 grilles tirées au hasard
/// (murs et coûts), la `k`-ième arête du plan est la `k`-ième arête de `Grille::graphe` dont les deux bouts sont gardés
/// (bouts retrouvés par la correspondance inverse), et les nœuds gardés sont dans l'ordre croissant de leur indice de
/// grille. Les positions et les étages des nœuds sont ceux de la grille. Au moins 60 des plans vérifiés sont troués
/// (mesuré : 91 sur 165), sans quoi un plan qui ignore les murs passerait (la campagne de fautes l'a vu).
///
/// Ne couvre pas : les coûts (voir `contre_la_construction_naive`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn l_ordre_est_celui_de_la_grille() {
    let mut alea = Alea::new(77);
    let mut tirer = |n: u32| alea.borne(n);
    let (mut verifies, mut avec_trous) = (0, 0);
    for _ in 0..300 {
        let (largeur, hauteur, etages) = (2 + tirer(6), 1 + tirer(5), 1 + tirer(2));
        let g = grille(largeur, hauteur, etages, Voisinage::Huit, 1.0, 1.0);
        let complet = g.graphe().unwrap();
        let n = complet.nb_noeuds();
        let mut murs = Vec::new();
        for _ in 0..tirer(4) {
            let (x0, y0) = (tirer(largeur), tirer(hauteur));
            murs.push(mur(x0, y0, x0 + tirer(largeur - x0), y0 + tirer(hauteur - y0), tirer(etages)));
        }
        let Ok(p) = construire(&g, &murs, &[], 0, n - 1) else { continue };
        let inverse: Vec<usize> = (0..n).filter(|&i| p.noeuds[i].is_some()).collect();
        assert!(p.noeuds.iter().flatten().copied().eq(0..inverse.len()), "les numéros croissent : 0, 1, 2, …");
        let gardees: Vec<(usize, usize, u64)> = complet
            .aretes()
            .iter()
            .filter(|e| p.noeuds[e.a].is_some() && p.noeuds[e.b].is_some())
            .map(|e| (e.a, e.b, e.longueur.to_bits()))
            .collect();
        let du_plan: Vec<(usize, usize, u64)> =
            p.graphe.aretes().iter().map(|e| (inverse[e.a], inverse[e.b], e.longueur.to_bits())).collect();
        assert_eq!(du_plan, gardees);
        let positions: Vec<Noeud> = inverse.iter().map(|&i| complet.noeuds()[i]).collect();
        assert_eq!(p.graphe.noeuds(), positions.as_slice());
        assert!(!gardees.is_empty(), "deux nœuds gardés reliés : au moins une arête");
        verifies += 1;
        if inverse.len() < n {
            avec_trous += 1;
        }
    }
    assert!(verifies >= 100, "{verifies} grilles vérifiées");
    // Un plan qui garderait tout (les murs ignorés) passerait les contrôles ci-dessus : il faut des plans troués.
    assert!(avec_trous >= 60, "{avec_trous} plans troués sur {verifies}");
}
