//! Étape 1, partie 1R : les trois mesures du réseau, `reseaux::{distance_moyenne, tenue, mesurer}` (TL, MD, FT et leurs
//! rapports à l'arbre couvrant minimal). Valeurs attendues : `references/etape-1r-valeurs.md`, identifiants `V-MES-xx`.
//!
//! Source : Tero et al. 2010 (Science 327, p. 439-440 et légende de la Fig. 3) : TL « longueur totale », MD « distance
//! minimale moyenne entre les paires de sources de nourriture », FT « la probabilité de déconnecter une partie du
//! réseau lors de la défaillance d'un seul lien », chacune normalisée par celle de l'arbre couvrant minimal ; **aucune
//! formule n'est publiée**. Les formules testées sont celles de la doc de `reseaux` (à nous), jugées par : des réseaux
//! écrits à la main (un arbre, une branche morte, un anneau, un arbre plus un raccourci, une nourriture coupée) dont
//! les valeurs se calculent à la main (écrites en commentaire) et par le modèle Python `ref_reseaux.py`, et des mesures
//! naïves écrites dans le test (relaxation, énumération) sur des réseaux tirés au hasard.
// Les boucles à indices suivent les procédures de la doc ligne pour ligne.
#![allow(clippy::needless_range_loop)]

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Arete, Graphe, Noeud};
use physarum_lab::mesures::tubes_vifs;
use physarum_lab::reseaux::{ErreurReseau, arbre_couvrant, distance_moyenne, mesurer, tenue};

/// Un graphe de `n` nœuds alignés, d'arêtes `(a, b, longueur)` dont le **coût est le triple de la longueur** (pour que
/// les mesures, qui portent sur les longueurs, ne puissent pas se tromper de champ).
fn graphe(n: usize, aretes: &[(usize, usize, f64)]) -> Graphe {
    let noeuds = (0..n).map(|i| Noeud { x: i as i32, y: 0, etage: 0 }).collect();
    let aretes = aretes.iter().map(|&(a, b, l)| Arete { a, b, longueur: l, cout: 3.0 * l }).collect();
    Graphe::new(noeuds, aretes).expect("graphe valide")
}

const SEUIL: f64 = 0.5;

/// Les valeurs attendues d'un réseau écrit à la main.
struct Attendu {
    nom: &'static str,
    tl: f64,
    md: f64,
    ft: f64,
    tl_sur_mst: f64,
    md_sur_mst: f64,
}

fn verifier(g: &Graphe, d: &[f64], foods: &[usize], a: &Attendu) {
    let m = mesurer(g, d, foods, SEUIL).unwrap_or_else(|e| panic!("{} : {e:?}", a.nom));
    assert_eq!(m.tl.to_bits(), a.tl.to_bits(), "{} : TL = {}", a.nom, m.tl);
    assert_eq!(m.md.to_bits(), a.md.to_bits(), "{} : MD = {}", a.nom, m.md);
    assert_eq!(m.ft.to_bits(), a.ft.to_bits(), "{} : FT = {}", a.nom, m.ft);
    assert_eq!(m.tl_sur_mst.to_bits(), a.tl_sur_mst.to_bits(), "{} : TL / TL_MST = {}", a.nom, m.tl_sur_mst);
    assert_eq!(m.md_sur_mst.to_bits(), a.md_sur_mst.to_bits(), "{} : MD / MD_MST = {}", a.nom, m.md_sur_mst);
    // les fonctions séparées rendent les mêmes nombres
    assert_eq!(distance_moyenne(g, d, foods, SEUIL).to_bits(), a.md.to_bits(), "{}", a.nom);
    assert_eq!(tenue(g, d, foods, SEUIL).to_bits(), a.ft.to_bits(), "{}", a.nom);
}

/// juste : un **arbre** (V-MES-01) : une chaîne `0-1-2-3-4` de longueurs `1, 2, 4, 8` (coûts triples), tous les tubes
/// vifs. Nourritures `{0, 4}` : `TL = 15`, `MD = 15`, `FT = 0` (chaque tube est sur le seul chemin : le retirer coupe)
/// ; l'arbre couvrant minimal est le réseau lui-même : `TL_MST = 15`, `MD_MST = 15`, rapports `1` et `1`. Nourritures
/// `{0, 2, 4}` : `MD = (3 + 15 + 12) / 3 = 10`, `FT = 0`, rapports `1` et `1` (`TL_MST = 3 + 12 = 15`, `MD_MST = 10`).
///
/// Ne couvre pas : les raccourcis (`anneau_et_raccourci`), les tubes morts (`tubes_morts_et_branches`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn un_arbre() {
    let g = graphe(5, &[(0, 1, 1.0), (1, 2, 2.0), (2, 3, 4.0), (3, 4, 8.0)]);
    let d = [1.0; 4];
    verifier(
        &g,
        &d,
        &[0, 4],
        &Attendu { nom: "arbre {0,4}", tl: 15.0, md: 15.0, ft: 0.0, tl_sur_mst: 1.0, md_sur_mst: 1.0 },
    );
    verifier(
        &g,
        &d,
        &[0, 2, 4],
        &Attendu { nom: "arbre {0,2,4}", tl: 15.0, md: 10.0, ft: 0.0, tl_sur_mst: 1.0, md_sur_mst: 1.0 },
    );
}

/// juste : un **anneau** et un **arbre plus un raccourci** (V-MES-02), tous les tubes vifs :
/// - l'anneau `0-1-2-3-0` de longueurs `1` partout, nourritures `{0, 2}` : `TL = 4`, `MD = 2`, `FT = 1` (retirer un
///   seul tube laisse l'autre moitié) ; `TL_MST = 2` (la distance des deux nourritures) : `TL / TL_MST = 2`, `MD /
///   MD_MST = 1` ;
/// - le même anneau aux longueurs `(0,1) = 1`, `(0,3) = 4`, `(1,2) = 2`, `(2,3) = 3` : `TL = 10`, `MD = min(1 + 2, 4 +
///   3) = 3`, `FT = 1`, `TL / TL_MST = 10 / 3` ;
/// - une chaîne `0-1-2-3-4` plus le tube `(1,3)` de longueur `1,5` (les tubes `(0,1)`, `(1,2)`, `(1,3)`, `(2,3)`,
///   `(3,4)` de longueurs `1, 1, 1,5, 1, 1`), nourritures `{0, 2, 4}` : `(0,1)` et `(3,4)` coupent (une nourriture n'a
///   que ce tube) ; `(1,2)`, `(2,3)` et `(1,3)`, sur le triangle `1-2-3`, ne coupent pas : `FT = 3 / 5 = 0,6` ; `TL =
///   5,5` ; les distances sont `d(0,2) = 2`, `d(0,4) = min(4, 1 + 1,5 + 1) = 3,5`, `d(2,4) = 2` : `MD = 7,5 / 3 = 2,5`
///   ; l'arbre garde `(0,2)` et `(2,4)` : `TL_MST = 4`, `MD_MST = (2 + 4 + 2) / 3 = 8 / 3` ; rapports `5,5 / 4 = 1,375`
///   et `2,5 / (8 / 3) = 0,9375`.
///
/// Ne couvre pas : les tubes sous le seuil (`tubes_morts_et_branches`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn anneau_et_raccourci() {
    let anneau = graphe(4, &[(0, 1, 1.0), (0, 3, 1.0), (1, 2, 1.0), (2, 3, 1.0)]);
    verifier(
        &anneau,
        &[1.0; 4],
        &[0, 2],
        &Attendu { nom: "anneau", tl: 4.0, md: 2.0, ft: 1.0, tl_sur_mst: 2.0, md_sur_mst: 1.0 },
    );
    let inegal = graphe(4, &[(0, 1, 1.0), (0, 3, 4.0), (1, 2, 2.0), (2, 3, 3.0)]);
    verifier(
        &inegal,
        &[1.0; 4],
        &[0, 2],
        &Attendu { nom: "anneau inégal", tl: 10.0, md: 3.0, ft: 1.0, tl_sur_mst: 10.0 / 3.0, md_sur_mst: 1.0 },
    );
    let raccourci = graphe(5, &[(0, 1, 1.0), (1, 2, 1.0), (1, 3, 1.5), (2, 3, 1.0), (3, 4, 1.0)]);
    verifier(
        &raccourci,
        &[1.0; 5],
        &[0, 2, 4],
        &Attendu {
            nom: "arbre plus raccourci",
            tl: 5.5,
            md: 2.5,
            ft: 0.6,
            tl_sur_mst: 1.375,
            md_sur_mst: 2.5 / (8.0 / 3.0),
        },
    );
    assert_eq!(2.5 / (8.0 / 3.0), 0.9375);
}

/// juste : les tubes morts, les branches mortes et les nourritures coupées (V-MES-03), seuil `0,5` :
/// - une **branche morte** vive : `0-1-2` et `1-3`, nourritures `{0, 2}`, tout vif : `(0,1)` et `(1,2)` coupent,
///   `(1,3)` ne coupe rien (il ne mène à aucune nourriture) : `FT = 1 / 3`, `TL = 3` (la branche compte), `MD = 2`, `TL
///   / TL_MST = 3 / 2` ;
/// - l'anneau inégal dont le tube `(0,3)` est sous le seuil : les tubes vifs `(0,1)`, `(1,2)`, `(2,3)` ; `(2,3)` est
///   une branche morte (le nœud 3 n'a plus que lui) : `FT = 1 / 3`, `TL = 1 + 2 + 3 = 6`, `MD = 3`, rapports `2` et `1`
///   ;
/// - **une nourriture coupée** : `0-1-2` vifs et `(1,3)` sous le seuil, nourritures `{0, 2, 3}` : la nourriture 3 n'est
///   plus reliée, donc **`MD = +inf`** (et `MD / MD_MST = +inf`) ; `FT = 0` (chacun des deux tubes vifs sépare 0 de 2),
///   `TL = 2`, `TL / TL_MST = 2 / 4` (`TL_MST` = deux distances de 2) ;
/// - **aucun tube vif** : `TL = 0`, `MD = +inf`, `FT = 0` (par définition : pas de panne possible, pas de tenue),
///   rapports `0` et `+inf` ;
/// - **deux composantes** : tubes `(0,1)`, `(2,3)`, `(0,4)` vifs, nourritures `{0, 1, 2, 3, 5}` dont `5` est isolée :
///   `MD = +inf`, `FT = 1 / 3` (`(0,4)` est une branche morte ; `(0,1)` et `(2,3)` coupent chacun une paire reliée) ;
///   les nourritures déjà séparées (5 de tout le reste) ne comptent pas comme « coupées par la panne ».
///
/// Ne couvre pas : le seuil à sa limite (`le_seuil_est_compris`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn tubes_morts_et_branches() {
    let branche = graphe(4, &[(0, 1, 1.0), (1, 2, 1.0), (1, 3, 1.0)]);
    verifier(
        &branche,
        &[1.0; 3],
        &[0, 2],
        &Attendu { nom: "branche", tl: 3.0, md: 2.0, ft: 1.0 / 3.0, tl_sur_mst: 1.5, md_sur_mst: 1.0 },
    );
    let inegal = graphe(4, &[(0, 1, 1.0), (0, 3, 4.0), (1, 2, 2.0), (2, 3, 3.0)]);
    verifier(
        &inegal,
        &[1.0, 0.1, 1.0, 1.0],
        &[0, 2],
        &Attendu { nom: "anneau coupé", tl: 6.0, md: 3.0, ft: 1.0 / 3.0, tl_sur_mst: 2.0, md_sur_mst: 1.0 },
    );
    verifier(
        &branche,
        &[1.0, 1.0, 0.1],
        &[0, 2, 3],
        &Attendu {
            nom: "nourriture coupée",
            tl: 2.0,
            md: f64::INFINITY,
            ft: 0.0,
            tl_sur_mst: 0.5,
            md_sur_mst: f64::INFINITY,
        },
    );
    verifier(
        &branche,
        &[0.1; 3],
        &[0, 2],
        &Attendu {
            nom: "aucun tube vif",
            tl: 0.0,
            md: f64::INFINITY,
            ft: 0.0,
            tl_sur_mst: 0.0,
            md_sur_mst: f64::INFINITY,
        },
    );
    // deux composantes : une nourriture isolée dans un graphe qui reste connexe sur le papier
    let deux = graphe(6, &[(0, 1, 1.0), (0, 4, 1.0), (2, 3, 2.0), (3, 5, 9.0), (4, 5, 9.0)]);
    let d = [1.0, 1.0, 1.0, 0.1, 0.1];
    assert_eq!(distance_moyenne(&deux, &d, &[0, 1, 2, 3, 5], SEUIL), f64::INFINITY);
    assert_eq!(tenue(&deux, &d, &[0, 1, 2, 3, 5], SEUIL).to_bits(), (1.0_f64 / 3.0).to_bits());
    // les mêmes tubes vifs, sans la nourriture isolée : la panne de (0,1) coupe encore 0 de 1
    assert_eq!(tenue(&deux, &d, &[0, 1, 2, 3], SEUIL).to_bits(), (1.0_f64 / 3.0).to_bits());
    assert_eq!(distance_moyenne(&deux, &d, &[0, 1], SEUIL), 1.0);
    assert_eq!(distance_moyenne(&deux, &d, &[2, 3], SEUIL), 2.0);
}

/// juste : l'ordre des sommes de `MD` et la direction des distances (V-MES-04) : une chaîne `0-1-2-3` de longueurs
/// `0,1`, `0,15`, `0,2`, tout vif, nourritures `[0, 1, 2, 3]` : les distances sont comptées **depuis le rang `i`** (le
/// plus petit des deux), sommées dans l'ordre `(0,1), (0,2), (0,3), (1,2), (1,3), (2,3)` à partir de `0.0` et divisées
/// par 6 : **`MD = 0,25`** (bits `3FD0000000000000`) ; comptées depuis le rang `j` on aurait `0,24999999999999997`
/// (calculé en Python et par la somme écrite dans le test). Pour deux nourritures `[0, 3]` d'une chaîne `0,1, 0,2, 0,3`
/// : `MD = 0,6000000000000001` ; `[3, 0]` : `0,6`.
///
/// Ne couvre pas : l'arbre (`etape_1r_arbre::l_ordre_des_sommes`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn l_ordre_des_sommes_de_md() {
    let g = graphe(4, &[(0, 1, 0.1), (1, 2, 0.15), (2, 3, 0.2)]);
    let md = distance_moyenne(&g, &[1.0; 3], &[0, 1, 2, 3], SEUIL);
    assert_eq!(md.to_bits(), 0x3FD0_0000_0000_0000);
    assert_ne!(md.to_bits(), 0.249_999_999_999_999_97_f64.to_bits());
    let m = mesurer(&g, &[1.0; 3], &[0, 1, 2, 3], SEUIL).unwrap();
    assert_eq!((m.tl.to_bits(), m.md.to_bits(), m.ft.to_bits()), (0.45_f64.to_bits(), md.to_bits(), 0.0_f64.to_bits()));
    let g = graphe(4, &[(0, 1, 0.1), (1, 2, 0.2), (2, 3, 0.3)]);
    assert_eq!(distance_moyenne(&g, &[1.0; 3], &[0, 3], SEUIL).to_bits(), (0.1_f64 + 0.2 + 0.3).to_bits());
    assert_eq!(distance_moyenne(&g, &[1.0; 3], &[3, 0], SEUIL).to_bits(), (0.3_f64 + 0.2 + 0.1).to_bits());
    assert_eq!(distance_moyenne(&g, &[1.0; 3], &[3, 0], SEUIL).to_bits(), 0.6_f64.to_bits());
    assert_eq!(distance_moyenne(&g, &[1.0; 3], &[0, 3], SEUIL).to_bits(), 0.600_000_000_000_000_1_f64.to_bits());
}

/// juste : le seuil d'un tube vif est **compris** (V-MES-05) : `D >= seuil`, comme `mesures::tubes_vifs`. Une chaîne de
/// cinq tubes de longueurs `1, 2, 3, 4, 5` et de conductivités `0,5` (le seuil : vif), `0,5` moins un ulp (pas vif),
/// `0,5` plus un ulp (vif), `NaN` (jamais vif), `1e300` (vif) : les tubes vifs sont le premier, le troisième et le
/// dernier, donc `TL = 1 + 3 + 5 = 9` (la somme des **longueurs**), aux bits de `tubes_vifs` ; un seuil NaN ou `+inf`
/// ne laisse aucun tube vif (`TL = 0`, `MD = +inf`, `FT = 0`) ; un seuil `0` rend vifs tous les tubes sauf NaN (`TL = 1
/// + 2 + 3 + 5`) ; toute la chaîne à `0,5` pile est vive au seuil `0,5` (`TL = 15`, `MD = 15`, `FT = 0`).
///
/// Ne couvre pas : le détail de `tubes_vifs` (`etape_1_mesures`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn le_seuil_est_compris() {
    let g = graphe(6, &[(0, 1, 1.0), (1, 2, 2.0), (2, 3, 3.0), (3, 4, 4.0), (4, 5, 5.0)]);
    let juste_dessous = f64::from_bits(0.5_f64.to_bits() - 1);
    let juste_dessus = f64::from_bits(0.5_f64.to_bits() + 1);
    let d = [0.5, juste_dessous, juste_dessus, f64::NAN, 1e300];
    let m = mesurer(&g, &d, &[0, 5], SEUIL).unwrap();
    assert_eq!(m.tl, 9.0, "les tubes vifs sont le premier, le troisième et le dernier : 1 + 3 + 5");
    assert_eq!(m.tl.to_bits(), tubes_vifs(&g, &d, SEUIL).1.to_bits());
    assert_eq!(tubes_vifs(&g, &d, SEUIL).0, 3);
    // un seuil NaN : aucun tube vif
    let nul = mesurer(&g, &d, &[0, 5], f64::NAN).unwrap();
    assert_eq!((nul.tl, nul.md, nul.ft), (0.0, f64::INFINITY, 0.0));
    let inf = mesurer(&g, &d, &[0, 5], f64::INFINITY).unwrap();
    assert_eq!((inf.tl, inf.md, inf.ft), (0.0, f64::INFINITY, 0.0));
    // un seuil de zéro : tout sauf NaN
    let zero = mesurer(&g, &d, &[0, 5], 0.0).unwrap();
    assert_eq!(zero.tl, 1.0 + 2.0 + 3.0 + 5.0);
    // tout vif : la chaîne entière
    let tout = mesurer(&g, &[0.5; 5], &[0, 5], 0.5).unwrap();
    assert_eq!((tout.tl, tout.md, tout.ft), (15.0, 15.0, 0.0));
}

/// Les distances par relaxation sur les tubes vifs (longueurs), depuis `source`.
fn bellman_vif(g: &Graphe, vifs: &[bool], source: usize) -> Vec<f64> {
    let mut d = vec![f64::INFINITY; g.nb_noeuds()];
    d[source] = 0.0;
    loop {
        let mut change = false;
        for (e, arete) in g.aretes().iter().enumerate() {
            if !vifs[e] {
                continue;
            }
            for (u, v) in [(arete.a, arete.b), (arete.b, arete.a)] {
                if d[u] + arete.longueur < d[v] {
                    d[v] = d[u] + arete.longueur;
                    change = true;
                }
            }
        }
        if !change {
            return d;
        }
    }
}

/// L'étiquette de composante de chaque nœud (le plus petit indice de sa composante), par relaxation jusqu'à stabilité,
/// avec les arêtes `vifs` sauf `sauf`.
fn composantes_naives(g: &Graphe, vifs: &[bool], sauf: Option<usize>) -> Vec<usize> {
    let mut etiquette: Vec<usize> = (0..g.nb_noeuds()).collect();
    loop {
        let mut change = false;
        for (e, arete) in g.aretes().iter().enumerate() {
            if !vifs[e] || Some(e) == sauf {
                continue;
            }
            let m = etiquette[arete.a].min(etiquette[arete.b]);
            if etiquette[arete.a] != m || etiquette[arete.b] != m {
                etiquette[arete.a] = m;
                etiquette[arete.b] = m;
                change = true;
            }
        }
        if !change {
            return etiquette;
        }
    }
}

/// Les mesures naïves : `(TL, MD, FT)` sur les tubes `>= seuil`.
fn mesures_naives(g: &Graphe, d: &[f64], foods: &[usize], seuil: f64) -> (f64, f64, f64) {
    let vifs: Vec<bool> = d.iter().map(|&x| x >= seuil).collect();
    let mut tl = 0.0;
    for (e, arete) in g.aretes().iter().enumerate() {
        if vifs[e] {
            tl += arete.longueur;
        }
    }
    let k = foods.len();
    let mut somme = 0.0;
    for i in 0..k {
        let depuis = bellman_vif(g, &vifs, foods[i]);
        for j in (i + 1)..k {
            somme += depuis[foods[j]];
        }
    }
    let md = somme / (k * (k - 1) / 2) as f64;
    let nb = vifs.iter().filter(|&&v| v).count();
    if nb == 0 {
        return (tl, md, 0.0);
    }
    let avant = composantes_naives(g, &vifs, None);
    let mut surs = 0;
    for e in 0..g.nb_aretes() {
        if !vifs[e] {
            continue;
        }
        let apres = composantes_naives(g, &vifs, Some(e));
        let mut coupe = false;
        for i in 0..k {
            for j in (i + 1)..k {
                let (a, b) = (foods[i], foods[j]);
                if avant[a] == avant[b] && apres[a] != apres[b] {
                    coupe = true;
                }
            }
        }
        if !coupe {
            surs += 1;
        }
    }
    (tl, md, surs as f64 / nb as f64)
}

/// juste : les mesures contre les mesures naïves du test (V-MES-06), sur 80 réseaux au hasard (de 5 à 16 nœuds, un
/// arbre plus des arêtes en plus, de 2 à 7 nourritures dans un ordre quelconque, des conductivités tirées parmi `0,1`
/// (mort), `0,5` (au seuil : vif) et `0,9`, parfois toutes vives, parfois presque toutes mortes) : `TL`, `MD` et `FT`
/// aux bits (`+inf` compris), `mesurer` cohérent avec les fonctions séparées et avec `arbre_couvrant` (les rapports
/// sont `TL / TL_MST` et `MD / MD_MST`, aux bits). Les mesures naïves emploient une relaxation de Bellman-Ford sur les
/// tubes vifs et une relaxation d'étiquettes de composantes, sans rien emprunter au code jugé. Le test exige que les
/// réseaux couvrent les cas : au moins 10 où `MD` est infini, 10 où `FT` vaut 0 (dont les chaînes toutes vives, une
/// graine sur huit, dont les deux bouts sont des nourritures), 10 où il vaut 1 et 20 où il est strictement entre les
/// deux.
///
/// Ne couvre pas : les graphes de plus de 16 nœuds.
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn contre_les_mesures_naives() {
    let (mut infinis, mut zeros, mut uns, mut milieu) = (0, 0, 0, 0);
    for graine in 1..=80_u64 {
        let mut alea = Alea::new(graine);
        let n = 5 + (graine as usize % 12);
        let chaine = graine % 8 == 0; // une chaîne toute vive entre deux bouts : FT = 0
        let mut paires: Vec<(usize, usize)> =
            (1..n).map(|i| (if chaine { i - 1 } else { alea.borne(i as u32) as usize }, i)).collect();
        for _ in 0..(if chaine { 0 } else { n + graine as usize % 5 }) {
            let (a, b) = (alea.borne(n as u32) as usize, alea.borne(n as u32) as usize);
            let (a, b) = (a.min(b), a.max(b));
            if a != b && !paires.contains(&(a, b)) {
                paires.push((a, b));
            }
        }
        let aretes: Vec<(usize, usize, f64)> =
            paires.iter().map(|&(a, b)| (a, b, 0.5 + 2.0 * alea.suivant_f64())).collect();
        let g = graphe(n, &aretes);
        let part_morts = if chaine { 0 } else { [0, 1, 3, 6][graine as usize % 4] };
        let d: Vec<f64> = (0..g.nb_aretes())
            .map(|_| {
                if alea.borne(10) < part_morts {
                    0.1
                } else if alea.borne(2) == 0 && !chaine {
                    0.5
                } else {
                    0.9
                }
            })
            .collect();
        let k = 2 + (graine as usize % 6).min(n - 2);
        let mut tous: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            tous.swap(i, alea.borne(i as u32 + 1) as usize);
        }
        if chaine {
            // les deux bouts de la chaîne en tête de la liste
            for (place, bout) in [0, n - 1].into_iter().enumerate() {
                let position = tous.iter().position(|&v| v == bout).unwrap();
                tous.swap(place, position);
            }
        }
        let foods = &tous[..k];
        let (tl, md, ft) = mesures_naives(&g, &d, foods, SEUIL);
        let m = mesurer(&g, &d, foods, SEUIL).unwrap();
        assert_eq!(m.tl.to_bits(), tl.to_bits(), "graine {graine} : TL");
        assert_eq!(m.md.to_bits(), md.to_bits(), "graine {graine} : MD {} contre {}", m.md, md);
        assert_eq!(m.ft.to_bits(), ft.to_bits(), "graine {graine} : FT {} contre {}", m.ft, ft);
        assert_eq!(m.tl.to_bits(), tubes_vifs(&g, &d, SEUIL).1.to_bits());
        assert_eq!(distance_moyenne(&g, &d, foods, SEUIL).to_bits(), md.to_bits());
        assert_eq!(tenue(&g, &d, foods, SEUIL).to_bits(), ft.to_bits());
        let arbre = arbre_couvrant(&g, foods).unwrap();
        assert_eq!(m.tl_sur_mst.to_bits(), (tl / arbre.longueur).to_bits(), "graine {graine}");
        assert_eq!(m.md_sur_mst.to_bits(), (md / arbre.distance_moyenne).to_bits(), "graine {graine}");
        if md.is_infinite() {
            infinis += 1;
        }
        if ft == 0.0 {
            zeros += 1;
        } else if ft == 1.0 {
            uns += 1;
        } else {
            milieu += 1;
        }
    }
    assert!(infinis >= 10 && zeros >= 10 && uns >= 10 && milieu >= 20, "{infinis} {zeros} {uns} {milieu}");
}

/// juste : les erreurs et les paniques (V-MES-07). `mesurer` rend `Taille` (autant de conductivités que d'arêtes), puis
/// `Nourriture` (moins de deux, hors du graphe, en double), puis `Coupee` (deux nourritures non reliées dans le
/// **graphe**, même si les tubes sont tous vifs) ; `Taille` l'emporte sur `Nourriture`, `Nourriture` sur `Coupee`.
/// `distance_moyenne` et `tenue` **paniquent** sur une taille fausse ou de mauvaises nourritures (cas positif d'abord).
///
/// Ne couvre pas : les valeurs (tests précédents).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn erreurs_et_paniques() {
    let g = graphe(5, &[(0, 1, 1.0), (1, 2, 1.0), (3, 4, 1.0)]);
    assert!(mesurer(&g, &[1.0; 3], &[0, 2], SEUIL).is_ok());
    assert_eq!(mesurer(&g, &[1.0; 2], &[0, 2], SEUIL), Err(ErreurReseau::Taille));
    assert_eq!(mesurer(&g, &[1.0; 4], &[0, 2], SEUIL), Err(ErreurReseau::Taille));
    assert_eq!(mesurer(&g, &[1.0; 2], &[0, 0], SEUIL), Err(ErreurReseau::Taille), "Taille avant Nourriture");
    for mauvaises in [vec![], vec![1], vec![0, 5], vec![0, 2, 0], vec![3, 3]] {
        assert_eq!(mesurer(&g, &[1.0; 3], &mauvaises, SEUIL), Err(ErreurReseau::Nourriture), "{mauvaises:?}");
    }
    assert_eq!(mesurer(&g, &[1.0; 3], &[0, 3], SEUIL), Err(ErreurReseau::Coupee));
    assert_eq!(mesurer(&g, &[1.0; 3], &[0, 3, 3], SEUIL), Err(ErreurReseau::Nourriture), "Nourriture avant Coupee");
    // les deux fonctions séparées
    assert!(distance_moyenne(&g, &[1.0; 3], &[0, 2], SEUIL).is_finite());
    assert!(tenue(&g, &[1.0; 3], &[0, 2], SEUIL).is_finite());
    assert_eq!(distance_moyenne(&g, &[1.0; 3], &[0, 3], SEUIL), f64::INFINITY, "pas de panique : c'est +inf");
    let cas: [(&str, Vec<f64>, Vec<usize>); 5] = [
        ("trop peu de conductivités", vec![1.0; 2], vec![0, 2]),
        ("trop de conductivités", vec![1.0; 4], vec![0, 2]),
        ("une seule nourriture", vec![1.0; 3], vec![0]),
        ("une nourriture hors du graphe", vec![1.0; 3], vec![0, 5]),
        ("deux fois la même nourriture", vec![1.0; 3], vec![2, 2]),
    ];
    for (nom, d, f) in cas {
        let (g1, d1, f1) = (g.clone(), d.clone(), f.clone());
        assert!(std::panic::catch_unwind(move || distance_moyenne(&g1, &d1, &f1, SEUIL)).is_err(), "MD : {nom}");
        let (g2, d2, f2) = (g.clone(), d, f);
        assert!(std::panic::catch_unwind(move || tenue(&g2, &d2, &f2, SEUIL)).is_err(), "FT : {nom}");
    }
}
