//! Étape 0 : `solveur`, le flux de Kirchhoff. Valeurs attendues : `references/etape-0-valeurs.md`, identifiants
//! `V-SOLV-xx`.
//!
//! Trois sortes de preuves, toutes indépendantes du code jugé :
//! - des réseaux dont les pressions se calculent à la main ou en rationnels exacts (une chaîne, le pont de Wheatstone, le
//!   cube de résistances : 7/12, 3/4 et 5/6) ;
//! - les lois elles-mêmes, vérifiées sur des graphes tirés au hasard : ce qui sort de chaque nœud égale ce qui y entre ;
//! - la **procédure de référence** de la doc, écrite ici telle quelle, dense et naïve (`reference`) : `pressions` doit en
//!   rendre les bits exacts. Quelques bits viennent aussi d'une transcription Python de cette procédure.
// Les boucles à indices de ce fichier suivent la procédure de la doc ligne pour ligne : on ne les réécrit pas en itérateurs.
#![allow(clippy::needless_range_loop)]

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Arete, Graphe, Grille, Noeud, Voisinage};
use physarum_lab::solveur::{ErreurSolveur, flux, pressions};

fn graphe(n: usize, paires: &[(usize, usize)]) -> Graphe {
    let noeuds = (0..n).map(|i| Noeud { x: i as i32, y: 0, etage: 0 }).collect();
    let aretes = paires.iter().map(|&(a, b)| Arete { a, b, longueur: 1.0, cout: 1.0 }).collect();
    Graphe::new(noeuds, aretes).expect("graphe valide")
}

fn grille(largeur: u32, hauteur: u32, etages: u32, voisinage: Voisinage) -> Graphe {
    Grille { largeur, hauteur, etages, voisinage, longueur_verticale: 1.0, cout_vertical: 1.0 }.graphe().unwrap()
}

/// Un graphe connexe tiré au hasard : un arbre (chaque nœud `i >= 1` relié à un nœud plus petit), plus `en_plus` essais
/// d'arêtes supplémentaires (les boucles et les doubles sont sautés).
fn graphe_au_hasard(alea: &mut Alea, n: usize, en_plus: usize) -> Graphe {
    let mut paires: Vec<(usize, usize)> = (1..n).map(|i| (alea.borne(i as u32) as usize, i)).collect();
    for _ in 0..en_plus {
        let (a, b) = (alea.borne(n as u32) as usize, alea.borne(n as u32) as usize);
        let (a, b) = (a.min(b), a.max(b));
        if a != b && !paires.contains(&(a, b)) {
            paires.push((a, b));
        }
    }
    graphe(n, &paires)
}

/// Des conductances au hasard dans `[0,25 ; 4,25[`.
fn conductances_au_hasard(alea: &mut Alea, m: usize) -> Vec<f64> {
    (0..m).map(|_| 0.25 + 4.0 * alea.suivant_f64()).collect()
}

/// La procédure de référence de la doc de `solveur`, recopiée ligne pour ligne, dense, sans rien sauter.
fn reference(g: &Graphe, conductances: &[f64], injections: &[f64], masse: usize) -> Vec<f64> {
    let n = g.nb_noeuds();
    let rang = |i: usize| if i < masse { i } else { i - 1 };
    let nn = n - 1;
    let mut a = vec![vec![0.0_f64; nn]; nn];
    let mut s = vec![0.0_f64; nn];
    for i in (0..n).filter(|&i| i != masse) {
        let mut somme = 0.0;
        for e in 0..g.nb_aretes() {
            if g.aretes()[e].a == i || g.aretes()[e].b == i {
                somme += conductances[e];
            }
        }
        a[rang(i)][rang(i)] = somme;
        s[rang(i)] = injections[i] + 0.0;
    }
    for (e, arete) in g.aretes().iter().enumerate() {
        if arete.a != masse && arete.b != masse && conductances[e] > 0.0 {
            a[rang(arete.a)][rang(arete.b)] = -conductances[e];
            a[rang(arete.b)][rang(arete.a)] = -conductances[e];
        }
    }
    let mut l = vec![vec![0.0_f64; nn]; nn];
    let mut d = vec![0.0_f64; nn];
    for i in 0..nn {
        for j in 0..i {
            let mut t = a[i][j];
            for k in 0..j {
                t -= (l[i][k] * d[k]) * l[j][k];
            }
            l[i][j] = t / d[j];
        }
        let mut t = a[i][i];
        for k in 0..i {
            t -= (l[i][k] * d[k]) * l[i][k];
        }
        d[i] = t;
    }
    let mut y = vec![0.0_f64; nn];
    for i in 0..nn {
        let mut t = s[i];
        for k in 0..i {
            t -= l[i][k] * y[k];
        }
        y[i] = t;
    }
    let mut x: Vec<f64> = (0..nn).map(|i| y[i] / d[i]).collect();
    for k in (1..nn).rev() {
        for i in 0..k {
            x[i] -= l[k][i] * x[k];
        }
    }
    (0..n).map(|i| if i == masse { 0.0 } else { x[rang(i)] }).collect()
}

fn bits(valeurs: &[f64]) -> Vec<u64> {
    valeurs.iter().map(|v| v.to_bits()).collect()
}

/// Ce qui sort du nœud `i` par ses arêtes, moins ce qui y est injecté, et l'échelle des termes de cette somme.
fn residu(g: &Graphe, conductances: &[f64], p: &[f64], injections: &[f64], i: usize) -> (f64, f64) {
    let (mut somme, mut echelle) = (-injections[i], injections[i].abs());
    for &e in g.incidentes(i) {
        let arete = g.aretes()[e];
        let autre = if arete.a == i { arete.b } else { arete.a };
        somme += conductances[e] * (p[i] - p[autre]);
        echelle += conductances[e] * (p[i].abs() + p[autre].abs());
    }
    (somme, echelle)
}

fn proche(obtenu: f64, attendu: f64) -> bool {
    (obtenu - attendu).abs() <= 1e-12 * attendu.abs().max(1.0)
}

/// juste : une chaîne `0 - 1 - 2 - 3` de conductances 1, une unité injectée en 0, la masse en 3 : les pressions sont
/// exactement `3, 2, 1, 0` et chaque flux vaut `1` (V-SOLV-01, la loi d'Ohm en série ; tout le calcul est exact en `f64`).
/// Avec les conductances `2, 4, 8` : `0,875 ; 0,375 ; 0,125 ; 0`. Avec la masse en 1 et l'injection en 3 : `0, 0, 1, 2`,
/// rien ne coule dans l'arête `(0,1)`, et les flux des deux autres valent `-1` (le liquide va du grand indice vers le petit).
///
/// Ne couvre pas : les réseaux maillés (tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn chaine_exacte() {
    let g = graphe(4, &[(0, 1), (1, 2), (2, 3)]);
    let p = pressions(&g, &[1.0, 1.0, 1.0], &[1.0, 0.0, 0.0, 0.0], 3).unwrap();
    assert_eq!(p, [3.0, 2.0, 1.0, 0.0]);
    assert_eq!(flux(&g, &[1.0, 1.0, 1.0], &p), [1.0, 1.0, 1.0]);
    let p = pressions(&g, &[2.0, 4.0, 8.0], &[1.0, 0.0, 0.0, 0.0], 3).unwrap();
    assert_eq!(p, [0.875, 0.375, 0.125, 0.0]);
    assert_eq!(flux(&g, &[2.0, 4.0, 8.0], &p), [1.0, 1.0, 1.0]);
    let p = pressions(&g, &[1.0, 1.0, 1.0], &[0.0, 0.0, 0.0, 1.0], 1).unwrap();
    assert_eq!(p, [0.0, 0.0, 1.0, 2.0]);
    assert_eq!(flux(&g, &[1.0, 1.0, 1.0], &p), [0.0, -1.0, -1.0]);
    // un seul nœud : sa pression est zéro
    let seul = graphe(1, &[]);
    assert_eq!(bits(&pressions(&seul, &[], &[5.0], 0).unwrap()), [0]);
}

/// juste : le pont de Wheatstone (V-SOLV-02) : quatre nœuds, arêtes `(0,1) (0,2) (1,2) (1,3) (2,3)` de conductances
/// `1, 2, 5, 3, 4`, une unité injectée en 0, la masse en 3. Solution exacte en rationnels (élimination en fractions,
/// outil Python) : pressions `74/155, 21/155, 23/155, 0` ; flux `53/155, 102/155, -2/31, 63/155, 92/155`. Tolérance
/// `1e-12` en relatif : l'erreur observée est de l'ordre de `1e-16`, une faute de signe ou d'indice est de l'ordre de 1.
///
/// Ne couvre pas : les bits (voir `bits_de_la_procedure_de_reference`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn pont_de_wheatstone() {
    let g = graphe(4, &[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)]);
    let c = [1.0, 2.0, 5.0, 3.0, 4.0];
    let p = pressions(&g, &c, &[1.0, 0.0, 0.0, 0.0], 3).unwrap();
    assert_eq!(p.len(), 4);
    for (obtenu, attendu) in p.iter().zip([74.0 / 155.0, 21.0 / 155.0, 23.0 / 155.0, 0.0]) {
        assert!(proche(*obtenu, attendu), "pression {obtenu} au lieu de {attendu}");
    }
    let q = flux(&g, &c, &p);
    assert_eq!(q.len(), 5);
    for (obtenu, attendu) in q.iter().zip([53.0 / 155.0, 102.0 / 155.0, -2.0 / 31.0, 63.0 / 155.0, 92.0 / 155.0]) {
        assert!(proche(*obtenu, attendu), "flux {obtenu} au lieu de {attendu}");
    }
}

/// juste : le cube de résistances (V-SOLV-03) : la grille 2 × 2 à 2 étages est un cube ; toutes les conductances valant 1,
/// la résistance entre deux sommets (la pression du sommet où l'on injecte une unité, la masse étant l'autre) vaut `7/12`
/// le long d'une arête, `3/4` en diagonale d'une face et `5/6` en grande diagonale : valeurs classiques, recalculées en
/// rationnels exacts (outil Python). Tolérance `1e-12`.
///
/// Ne couvre pas : des conductances inégales entre étages (voir `kirchhoff_sur_des_grilles`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn cube_de_resistances() {
    let description = Grille {
        largeur: 2,
        hauteur: 2,
        etages: 2,
        voisinage: Voisinage::Quatre,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let g = description.graphe().unwrap();
    assert_eq!((g.nb_noeuds(), g.nb_aretes()), (8, 12));
    let c = [1.0; 12];
    let origine = description.indice(0, 0, 0);
    for ((x, y, e), attendu) in
        [((1, 0, 0), 7.0 / 12.0), ((0, 0, 1), 7.0 / 12.0), ((1, 1, 0), 0.75), ((1, 0, 1), 0.75), ((1, 1, 1), 5.0 / 6.0)]
    {
        let cible = description.indice(x, y, e);
        let mut injections = [0.0; 8];
        injections[cible] = 1.0;
        let p = pressions(&g, &c, &injections, origine).unwrap();
        assert!(proche(p[cible], attendu), "({x}, {y}, {e}) : {} au lieu de {attendu}", p[cible]);
        // et dans l'autre sens : la résistance est symétrique
        let mut injections = [0.0; 8];
        injections[origine] = 1.0;
        let p = pressions(&g, &c, &injections, cible).unwrap();
        assert!(proche(p[origine], attendu), "({x}, {y}, {e}) à l'envers : {}", p[origine]);
    }
}

/// juste : la loi de Kirchhoff sur 40 graphes connexes tirés au hasard (5 à 40 nœuds, conductances dans `[0,25 ; 4,25[`,
/// une injection au hasard dans `[-1 ; 1[` à chaque nœud, la masse au hasard) : en tout nœud autre que la masse, ce qui
/// sort par les arêtes égale l'injection, à `1e-10` près en relatif de l'échelle des termes (V-SOLV-04 ; écart observé sur
/// le prototype : moins de `2e-15`) ; la pression de la masse est `+0.0` ; ce que la masse absorbe est la somme des autres
/// injections. Un système dont la matrice est définie positive a une seule solution : des pressions qui tiennent ces lois
/// sont les bonnes.
///
/// Ne couvre pas : les conductances nulles ni très inégales (tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn kirchhoff_sur_des_graphes_au_hasard() {
    let mut alea = Alea::new(20_261_109);
    for essai in 0..40 {
        let n = 5 + alea.borne(36) as usize;
        let g = graphe_au_hasard(&mut alea, n, 2 * n);
        let c = conductances_au_hasard(&mut alea, g.nb_aretes());
        let injections: Vec<f64> = (0..n).map(|_| 2.0 * alea.suivant_f64() - 1.0).collect();
        let masse = alea.borne(n as u32) as usize;
        let p = pressions(&g, &c, &injections, masse).unwrap();
        assert_eq!(p.len(), n);
        assert_eq!(p[masse].to_bits(), 0, "essai {essai} : la pression de la masse est +0.0");
        let mut vers_la_masse = 0.0;
        for i in 0..n {
            let (somme, echelle) = residu(&g, &c, &p, &injections, i);
            if i == masse {
                vers_la_masse = somme + injections[i];
            } else {
                assert!(somme.abs() <= 1e-10 * echelle, "essai {essai}, nœud {i} : résidu {somme}, échelle {echelle}");
            }
        }
        let autres: f64 = (0..n).filter(|&i| i != masse).map(|i| injections[i]).sum();
        assert!(
            (vers_la_masse + autres).abs() <= 1e-10 * (1.0 + autres.abs()),
            "essai {essai} : la masse absorbe le reste"
        );
    }
}

/// juste : la loi de Kirchhoff sur des grilles (V-SOLV-04) : 9 × 7 en voisinage 8, 6 × 5 à 3 étages, 12 × 1 à 2 étages ;
/// conductances au hasard, une source et un puits de même débit (la masse est le puits), puis une injection partout.
/// Avec une seule source et la masse pour puits, toute pression est entre 0 et celle de la source (principe du maximum).
///
/// Ne couvre pas : les grilles de plus de 100 nœuds de large (voir `grande_grille_sans_matrice_dense`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn kirchhoff_sur_des_grilles() {
    let mut alea = Alea::new(7);
    for g in [grille(9, 7, 1, Voisinage::Huit), grille(6, 5, 3, Voisinage::Quatre), grille(12, 1, 2, Voisinage::Huit)] {
        let n = g.nb_noeuds();
        let c = conductances_au_hasard(&mut alea, g.nb_aretes());
        let source = alea.borne(n as u32) as usize;
        let puits = (source + 1 + alea.borne(n as u32 - 1) as usize) % n;
        let mut injections = vec![0.0; n];
        injections[source] = 1.5;
        injections[puits] = -1.5;
        let p = pressions(&g, &c, &injections, puits).unwrap();
        for i in (0..n).filter(|&i| i != puits) {
            let (somme, echelle) = residu(&g, &c, &p, &injections, i);
            assert!(somme.abs() <= 1e-10 * echelle, "nœud {i} : résidu {somme}");
            let marge = 1e-12 * p[source];
            assert!(
                p[i] >= -marge && p[i] <= p[source] + marge,
                "nœud {i} : pression {} hors de [0, {}]",
                p[i],
                p[source]
            );
        }
        assert!(p[source] > 0.0);
        let partout: Vec<f64> = (0..n).map(|_| alea.suivant_f64() - 0.5).collect();
        let p = pressions(&g, &c, &partout, 0).unwrap();
        for i in 1..n {
            let (somme, echelle) = residu(&g, &c, &p, &partout, i);
            assert!(somme.abs() <= 1e-10 * echelle, "nœud {i} : résidu {somme}");
        }
    }
}

/// juste : les bits de la procédure de référence, calculés à part par sa transcription en Python (flottants IEEE 754,
/// mêmes opérations dans le même ordre) (V-SOLV-05) :
/// - une chaîne de cinq nœuds, masse au milieu (2), conductances `0.1, 0.2, 0.3, 0.7`, injections `1, 0, 99, 0, -0.25` :
///   la seconde pression est `4.999999999999999` (`0x4013FFFFFFFFFFFF`), pas `5.0` ;
/// - le pont de Wheatstone de conductances `0.1, 0.7, 1.3, 0.3, 2.9`, injections `0.5, 0, -0.2, 0.3`, masse en 1.
///
/// Ne couvre pas : les grands graphes (voir `bits_de_la_procedure_de_reference`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn bits_calcules_a_part() {
    let g = graphe(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    let p = pressions(&g, &[0.1, 0.2, 0.3, 0.7], &[1.0, 0.0, 99.0, 0.0, -0.25], 2).unwrap();
    assert_eq!(
        bits(&p),
        [
            0x402E_0000_0000_0000,
            0x4013_FFFF_FFFF_FFFF,
            0x0000_0000_0000_0000,
            0xBFEA_AAAA_AAAA_AAAA,
            0xBFF3_0C30_C30C_30C3
        ]
    );
    let g = graphe(4, &[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)]);
    let p = pressions(&g, &[0.1, 0.7, 1.3, 0.3, 2.9], &[0.5, 0.0, -0.2, 0.3], 1).unwrap();
    assert_eq!(bits(&p), [0x3FEC_9858_B545_E7E4, 0x0000_0000_0000_0000, 0x3FD3_A55D_0C0D_7FC0, 0x3FD7_CDDC_52EC_3BC6]);
    let g = graphe(4, &[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)]);
    let p = pressions(&g, &[1.0, 2.0, 5.0, 3.0, 4.0], &[1.0, 0.0, 0.0, 0.0], 3).unwrap();
    assert_eq!(bits(&p), [0x3FDE_8E09_E8E0_9E8E, 0x3FC1_5789_1578_915A, 0x3FC2_FE59_2FE5_9300, 0]);
}

/// juste : `pressions` rend, bit pour bit, ce que rend la procédure de référence de la doc, écrite dans ce fichier sans
/// rien sauter (`reference`) (V-SOLV-06) : sur 30 graphes au hasard (4 à 30 nœuds), sur des grilles (7 × 5 en voisinage
/// 8, 4 × 3 à 2 étages, 3 × 3 à 3 étages en voisinage 8), la masse au début, au milieu, à la fin ; puis les mêmes avec une
/// conductance sur cinq mise à zéro hors d'un arbre couvrant (le réseau reste relié).
///
/// Ne couvre pas : les réseaux de plus de 100 nœuds (la procédure dense y serait lente) ; l'ordre y est le même.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn bits_de_la_procedure_de_reference() {
    let mut alea = Alea::new(3);
    let mut graphes: Vec<Graphe> = Vec::new();
    for _ in 0..30 {
        let n = 4 + alea.borne(27) as usize;
        graphes.push(graphe_au_hasard(&mut alea, n, n + n / 2));
    }
    graphes.push(grille(7, 5, 1, Voisinage::Huit));
    graphes.push(grille(4, 3, 2, Voisinage::Quatre));
    graphes.push(grille(3, 3, 3, Voisinage::Huit));
    for (numero, g) in graphes.iter().enumerate() {
        let n = g.nb_noeuds();
        let mut c = conductances_au_hasard(&mut alea, g.nb_aretes());
        let injections: Vec<f64> = (0..n).map(|_| 2.0 * alea.suivant_f64() - 1.0).collect();
        for masse in [0, n / 2, n - 1] {
            let p = pressions(g, &c, &injections, masse).unwrap();
            assert_eq!(bits(&p), bits(&reference(g, &c, &injections, masse)), "graphe {numero}, masse {masse}");
        }
        // des conductances nulles, hors d'un arbre couvrant : pour un graphe au hasard, les arêtes `(parent, i)` de
        // l'arbre sont les premières données ; on garde donc, pour chaque nœud, sa première arête vers un indice plus petit.
        let mut garde = vec![false; g.nb_aretes()];
        for i in 1..n {
            let premiere = g.incidentes(i).iter().copied().find(|&e| g.aretes()[e].b == i);
            garde[premiere.expect("tout nœud a un voisin plus petit")] = true;
        }
        for e in 0..g.nb_aretes() {
            if !garde[e] && e % 5 == 0 {
                c[e] = 0.0;
            }
        }
        for masse in [0, n / 2, n - 1] {
            let p = pressions(g, &c, &injections, masse).unwrap();
            assert_eq!(bits(&p), bits(&reference(g, &c, &injections, masse)), "graphe {numero} troué, masse {masse}");
        }
    }
}

/// juste : deux changements d'échelle exacts en `f64` (une multiplication par une puissance de 2 ne perd aucun bit)
/// (V-SOLV-07) : des injections multipliées par 4 donnent des pressions multipliées par 4, bit pour bit ; des conductances
/// multipliées par 8 donnent des pressions divisées par 8, bit pour bit, et les mêmes flux.
///
/// Ne couvre pas : les facteurs qui ne sont pas des puissances de 2.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn changements_d_echelle_exacts() {
    let mut alea = Alea::new(11);
    for _ in 0..10 {
        let n = 6 + alea.borne(20) as usize;
        let g = graphe_au_hasard(&mut alea, n, n);
        let c = conductances_au_hasard(&mut alea, g.nb_aretes());
        let injections: Vec<f64> = (0..n).map(|_| 2.0 * alea.suivant_f64() - 1.0).collect();
        let masse = alea.borne(n as u32) as usize;
        let p = pressions(&g, &c, &injections, masse).unwrap();
        assert_eq!(p.len(), n);
        assert!(p.iter().any(|v| v.abs() > 1e-9), "des injections non nulles donnent des pressions non nulles");
        let quatre: Vec<f64> = injections.iter().map(|b| 4.0 * b).collect();
        let p4 = pressions(&g, &c, &quatre, masse).unwrap();
        assert_eq!(bits(&p4), bits(&p.iter().map(|v| 4.0 * v).collect::<Vec<f64>>()));
        let huit: Vec<f64> = c.iter().map(|v| 8.0 * v).collect();
        let p8 = pressions(&g, &huit, &injections, masse).unwrap();
        assert_eq!(bits(&p8), bits(&p.iter().map(|v| v / 8.0).collect::<Vec<f64>>()));
        assert_eq!(bits(&flux(&g, &huit, &p8)), bits(&flux(&g, &c, &p)));
    }
}

/// juste : la masse (V-SOLV-08, la doc) : sa pression est `+0.0` ; son injection n'est pas lue (la changer ne change aucun
/// bit du résultat) ; une injection `-0.0` donne les bits d'une injection `+0.0` ; une conductance `-0.0` est acceptée et
/// vaut zéro. Sur la chaîne `0 - 1 - 2` de conductances 1 : injecter 1 en 0 avec la masse en 2 donne `2, 1, 0`.
///
/// Ne couvre pas : les erreurs (tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn la_masse_et_les_zeros() {
    let g = graphe(3, &[(0, 1), (1, 2)]);
    let p = pressions(&g, &[1.0, 1.0], &[1.0, 0.0, 0.0], 2).unwrap();
    assert_eq!(bits(&p), bits(&[2.0, 1.0, 0.0]));
    for injection_de_la_masse in [-1.0, 123.0, -0.0, 1e300] {
        let q = pressions(&g, &[1.0, 1.0], &[1.0, 0.0, injection_de_la_masse], 2).unwrap();
        assert_eq!(bits(&q), bits(&p), "injection de la masse {injection_de_la_masse}");
    }
    // aucune injection : toutes les pressions sont +0.0, que les zéros donnés soient +0.0 ou -0.0
    for zero in [0.0, -0.0] {
        let q = pressions(&g, &[1.0, 1.0], &[zero, zero, zero], 1).unwrap();
        assert_eq!(bits(&q), [0, 0, 0]);
    }
    // une conductance -0.0 vaut zéro : le triangle 0-1-2 privé de l'arête (0,2) est la chaîne
    let triangle = graphe(3, &[(0, 1), (0, 2), (1, 2)]);
    let q = pressions(&triangle, &[1.0, -0.0, 1.0], &[1.0, 0.0, 0.0], 2).unwrap();
    assert_eq!(bits(&q), bits(&p));
    let q = pressions(&triangle, &[1.0, 0.0, 1.0], &[1.0, 0.0, 0.0], 2).unwrap();
    assert_eq!(bits(&q), bits(&p));
}

/// juste : les erreurs, dans l'ordre de la doc (V-SOLV-09) : `Taille` avant tout (une conductance ou une injection de
/// trop ou de moins), puis `Masse`, puis `Conductance(e)` pour le plus petit indice fautif (négatif, infini, NaN), puis
/// `Injection(i)` (infini, NaN ; celle de la masse aussi), chacune l'emportant sur les suivantes ; le même appel sans la
/// faute réussit.
///
/// Ne couvre pas : `Isole` et `Pivot` (tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn erreurs_dans_l_ordre() {
    let g = graphe(4, &[(0, 1), (1, 2), (2, 3)]);
    let c = [1.0, 1.0, 1.0];
    let b = [1.0, 0.0, 0.0, 0.0];
    assert!(pressions(&g, &c, &b, 3).is_ok());
    assert_eq!(pressions(&g, &c[..2], &b, 3), Err(ErreurSolveur::Taille));
    assert_eq!(pressions(&g, &[1.0; 4], &b, 3), Err(ErreurSolveur::Taille));
    assert_eq!(pressions(&g, &c, &b[..3], 3), Err(ErreurSolveur::Taille));
    assert_eq!(pressions(&g, &c, &[0.0; 5], 3), Err(ErreurSolveur::Taille));
    assert_eq!(pressions(&g, &[f64::NAN; 2], &b, 9), Err(ErreurSolveur::Taille), "Taille avant Masse et Conductance");
    assert_eq!(pressions(&g, &c, &b, 4), Err(ErreurSolveur::Masse));
    assert_eq!(pressions(&g, &[1.0, -1.0, f64::NAN], &b, 4), Err(ErreurSolveur::Masse), "Masse avant Conductance");
    assert_eq!(pressions(&g, &c, &b, usize::MAX), Err(ErreurSolveur::Masse));
    for (fautives, attendu) in [
        ([-1.0, 1.0, 1.0], 0),
        ([1.0, 1.0, -1e-300], 2),
        ([1.0, f64::INFINITY, 1.0], 1),
        ([1.0, f64::NAN, -1.0], 1),
        ([1.0, 1.0, f64::NEG_INFINITY], 2),
    ] {
        assert_eq!(pressions(&g, &fautives, &b, 3), Err(ErreurSolveur::Conductance(attendu)), "{fautives:?}");
        let injections = [f64::NAN, 0.0, 0.0, 0.0];
        assert_eq!(pressions(&g, &fautives, &injections, 3), Err(ErreurSolveur::Conductance(attendu)));
    }
    for (fautives, attendu) in [
        ([f64::NAN, 0.0, 0.0, 0.0], 0),
        ([1.0, 0.0, f64::INFINITY, f64::NAN], 2),
        ([1.0, 0.0, 0.0, f64::NEG_INFINITY], 3),
    ] {
        assert_eq!(pressions(&g, &c, &fautives, 3), Err(ErreurSolveur::Injection(attendu)), "{fautives:?}");
        // l'injection fautive l'emporte sur un nœud isolé
        assert_eq!(pressions(&g, &[1.0, 0.0, 1.0], &fautives, 3), Err(ErreurSolveur::Injection(attendu)));
    }
}

/// juste : `Isole(i)` : le plus petit nœud qu'aucun chemin d'arêtes de conductance strictement positive ne relie à la
/// masse (V-SOLV-10, la doc) : un graphe en deux morceaux ; un nœud sans arête ; une chaîne coupée par une conductance
/// nulle (`+0.0` ou `-0.0`), selon le côté où est la masse ; toutes les conductances nulles. Un réseau relié par un détour
/// réussit malgré une conductance nulle.
///
/// Ne couvre pas : les conductances très petites mais positives (elles relient).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn noeud_isole_de_la_masse() {
    let deux_morceaux = graphe(5, &[(0, 1), (2, 3), (3, 4)]);
    let b = [0.0; 5];
    assert_eq!(pressions(&deux_morceaux, &[1.0; 3], &b, 0), Err(ErreurSolveur::Isole(2)));
    assert_eq!(pressions(&deux_morceaux, &[1.0; 3], &b, 4), Err(ErreurSolveur::Isole(0)));
    let avec_un_seul = graphe(4, &[(0, 1), (1, 3)]);
    assert_eq!(pressions(&avec_un_seul, &[1.0; 2], &[0.0; 4], 0), Err(ErreurSolveur::Isole(2)));
    assert_eq!(pressions(&avec_un_seul, &[1.0; 2], &[0.0; 4], 2), Err(ErreurSolveur::Isole(0)));
    let chaine = graphe(4, &[(0, 1), (1, 2), (2, 3)]);
    for zero in [0.0, -0.0] {
        assert_eq!(pressions(&chaine, &[1.0, zero, 1.0], &[0.0; 4], 3), Err(ErreurSolveur::Isole(0)));
        assert_eq!(pressions(&chaine, &[1.0, zero, 1.0], &[0.0; 4], 0), Err(ErreurSolveur::Isole(2)));
        assert_eq!(pressions(&chaine, &[zero, 1.0, 1.0], &[0.0; 4], 2), Err(ErreurSolveur::Isole(0)));
        assert_eq!(pressions(&chaine, &[zero, zero, zero], &[0.0; 4], 1), Err(ErreurSolveur::Isole(0)));
        assert_eq!(pressions(&chaine, &[zero, zero, zero], &[0.0; 4], 0), Err(ErreurSolveur::Isole(1)));
    }
    let minuscules = pressions(&chaine, &[1e-300, 1e-300, 1e-300], &[1e-300, 0.0, 0.0, 0.0], 3);
    assert_eq!(minuscules, Ok(vec![3.0, 2.0, 1.0, 0.0]), "des conductances minuscules relient");
    let cycle = graphe(4, &[(0, 1), (1, 2), (2, 3), (0, 3)]);
    assert!(pressions(&cycle, &[1.0, 0.0, 1.0, 1.0], &[1.0, 0.0, 0.0, 0.0], 2).is_ok(), "le détour relie");
}

/// juste : `Pivot(i)`, les deux cas de la doc (V-SOLV-11), tous deux fixés par la procédure de référence :
/// - une somme qui déborde : au nœud 1 de la chaîne `0 - 1 - 2` arrivent deux conductances de `1e308`, dont la somme est
///   infinie ; avec la masse en 2, le pivot du nœud 0 (`1e308`) est bon et celui du nœud 1 ne l'est pas ; avec la masse en
///   0, le nœud 1 est le premier rang ; avec `1e307`, rien ne déborde ;
/// - une conductance perdue dans l'arrondi : sur la chaîne `0 - 1 - 2 - 3` de conductances `1, 1e-300, 1`, masse en 3, les
///   nœuds 0 et 1 ne tiennent à la masse que par `1e-300` ; `1.0 + 1e-300` vaut `1.0`, et le pivot du nœud 1,
///   `1 - (-1 * 1) * (-1)`, vaut exactement zéro. Avec `1e-10` à la place, le calcul passe, mais `1 + 1e-10 - 1` n'a
///   gardé que sept chiffres de `1e-10` : les pressions exactes `2 + 1e10, 1 + 1e10, 1, 0` sont rendues à `1e-6` près en
///   relatif (écart observé : `8,3e-8`), pas à `1e-12` (la doc : « Ce que le solveur ne garantit pas »).
///
/// Ne couvre pas : un pivot négatif (non construit ici).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn pivot_fautif() {
    let g = graphe(3, &[(0, 1), (1, 2)]);
    assert_eq!(pressions(&g, &[1e308, 1e308], &[1.0, 0.0, 0.0], 2), Err(ErreurSolveur::Pivot(1)));
    assert_eq!(pressions(&g, &[1e308, 1e308], &[1.0, 0.0, 0.0], 0), Err(ErreurSolveur::Pivot(1)));
    assert!(pressions(&g, &[1e307, 1e307], &[1.0, 0.0, 0.0], 2).is_ok());
    let chaine = graphe(4, &[(0, 1), (1, 2), (2, 3)]);
    assert_eq!(pressions(&chaine, &[1.0, 1e-300, 1.0], &[1.0, 0.0, 0.0, 0.0], 3), Err(ErreurSolveur::Pivot(1)));
    let p = pressions(&chaine, &[1.0, 1e-10, 1.0], &[1.0, 0.0, 0.0, 0.0], 3).unwrap();
    assert_eq!(p.len(), 4);
    for (obtenu, attendu) in p.iter().zip([2.0 + 1e10, 1.0 + 1e10, 1.0]) {
        assert!((obtenu - attendu).abs() <= 1e-6 * attendu, "{obtenu} au lieu de {attendu}");
    }
}

/// juste : `flux` rend `conductances[e] * (pressions[a] - pressions[b])` pour chaque arête, `a < b` (V-SOLV-12, la
/// définition) : sur le triangle, des pressions `5, 3, 0` et des conductances `2, 10, 0.5` donnent `4, 50, 1.5` ; le signe
/// suit le sens du petit indice vers le grand ; une conductance nulle donne un flux nul.
///
/// Ne couvre pas : la conservation (voir les tests de Kirchhoff).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn flux_de_chaque_arete() {
    let g = graphe(3, &[(0, 1), (0, 2), (1, 2)]);
    assert_eq!(flux(&g, &[2.0, 10.0, 0.5], &[5.0, 3.0, 0.0]), [4.0, 50.0, 1.5]);
    assert_eq!(flux(&g, &[2.0, 10.0, 0.5], &[0.0, 3.0, 5.0]), [-6.0, -50.0, -1.0]);
    assert_eq!(flux(&g, &[0.0, 1.0, 0.0], &[5.0, 3.0, 0.0]), [0.0, 5.0, 0.0]);
    // l'ordre des opérations : une soustraction, puis un produit
    let q = flux(&g, &[0.1, 0.1, 0.1], &[0.3, 0.1, 0.0]);
    assert_eq!(bits(&q), bits(&[0.1 * (0.3 - 0.1), 0.1 * (0.3 - 0.0), 0.1 * (0.1 - 0.0)]));
    for (c, p) in [
        (vec![1.0; 2], vec![0.0; 3]),
        (vec![1.0; 3], vec![0.0; 2]),
        (vec![1.0; 4], vec![0.0; 3]),
        (vec![1.0; 3], vec![0.0; 4]),
    ] {
        let g = &g;
        assert!(std::panic::catch_unwind(move || flux(g, &c, &p)).is_err(), "longueurs fausses : panique attendue");
    }
}

/// juste : une grille de 80 × 80 (6 400 nœuds) se résout sans matrice dense (V-SOLV-13, la doc : « sauter les zéros du
/// profil ») : en sautant les zéros, il y a environ vingt millions d'opérations, soit quelques dizaines de millisecondes ;
/// la procédure dense en demanderait plus de quarante milliards et 300 Mo. Le test laisse dix secondes, plus de cent fois
/// la durée mesurée sur le prototype, et vérifie Kirchhoff en chaque nœud ainsi que la symétrie de la résistance entre deux
/// coins.
///
/// Ne couvre pas : la durée exacte, ni les bits sur cette grille.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn grande_grille_sans_matrice_dense() {
    let g = grille(80, 80, 1, Voisinage::Quatre);
    let n = g.nb_noeuds();
    let mut alea = Alea::new(80);
    let c: Vec<f64> = (0..g.nb_aretes()).map(|_| 0.5 + alea.suivant_f64()).collect();
    let mut injections = vec![0.0; n];
    injections[0] = 1.0;
    let debut = std::time::Instant::now();
    let p = pressions(&g, &c, &injections, n - 1).unwrap();
    let duree = debut.elapsed();
    assert!(duree.as_secs() < 10, "la résolution a pris {duree:?} : une matrice dense ?");
    for i in 0..n - 1 {
        let (somme, echelle) = residu(&g, &c, &p, &injections, i);
        assert!(somme.abs() <= 1e-9 * echelle, "nœud {i} : résidu {somme}");
    }
    let mut retour = vec![0.0; n];
    retour[n - 1] = 1.0;
    let q = pressions(&g, &c, &retour, 0).unwrap();
    assert!((p[0] - q[n - 1]).abs() <= 1e-9 * p[0], "résistance {} dans un sens, {} dans l'autre", p[0], q[n - 1]);
    assert!(p[0] > 1.0, "entre deux coins d'une grille de 80 × 80, la résistance dépasse 1 : {}", p[0]);
}
