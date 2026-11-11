//! Étape 1 : `tubes`, un pas du modèle de Tero (loi linéaire, pas d'Euler, plancher). Valeurs attendues :
//! `references/etape-1-valeurs.md`, identifiants `V-TUBE-xx`.
//!
//! Trois sortes de preuves, toutes indépendantes du code jugé :
//! - des **cas fermés** dont la réponse s'écrit avec une récurrence ou une somme : un tube seul (`Q = I` à chaque pas,
//!   donc `D <- D + h (I - D)`), une chaîne, une branche morte et le plancher, la coupe autour de la source (la somme
//!   des conductivités de ses arêtes suit `S <- S + h (I - S)`, le lemme de Bonifaci pour des tubes en parallèle, en
//!   temps discret) ;
//! - un **pas naïf** écrit dans le test (`pas_naif`), avec `solveur::pressions` et `solveur::flux` (éprouvés à l'étape
//!   0) : `avancer` doit en rendre les conductivités, les pressions et les flux aux bits, plusieurs pas de suite, sur
//!   des graphes au hasard ; une mise à l'échelle exacte ; quelques bits calculés par le modèle Python `ref_tubes.py` ;
//! - les **erreurs** de la doc, leur ordre, et « rien n'a changé ».
//!
//! Les tolérances des cas fermés sont des erreurs d'arrondi mesurées sur le prototype (écrites en commentaire).
// Les boucles à indices suivent les procédures de la doc ligne pour ligne.
#![allow(clippy::needless_range_loop)]

#[path = "common/graphes_au_hasard.rs"]
mod graphes_au_hasard;

use graphes_au_hasard::{graphe_au_hasard, graphe_pondere};
use physarum_lab::alea::Alea;
use physarum_lab::graphe::Graphe;
use physarum_lab::solveur::{ErreurSolveur, flux, pressions};
use physarum_lab::tubes::{ErreurTubes, PLANCHER, Tubes};

fn bits(valeurs: &[f64]) -> Vec<u64> {
    valeurs.iter().map(|v| v.to_bits()).collect()
}

/// Un pas du modèle écrit naïvement d'après la doc (« Le pas, à la lettre »), avec les fonctions du solveur de l'étape
/// 0.
fn pas_naif(g: &Graphe, d: &mut [f64], source: usize, puits: usize, debit: f64, h: f64) -> (Vec<f64>, Vec<f64>) {
    let mut conductances = Vec::new();
    for e in 0..g.nb_aretes() {
        conductances.push(d[e] / g.aretes()[e].cout);
    }
    let mut injections = vec![0.0; g.nb_noeuds()];
    injections[source] = debit;
    let p = pressions(g, &conductances, &injections, puits).expect("pressions");
    let q = flux(g, &conductances, &p);
    for e in 0..g.nb_aretes() {
        let nouveau = d[e] + h * (q[e].abs() - d[e]);
        d[e] = if nouveau >= PLANCHER { nouveau } else { PLANCHER };
    }
    (p, q)
}

/// Une chaîne `0 - 1 - ... - k` : les arêtes `(i, i+1)` de coût `couts[i]`.
fn chaine(couts: &[f64]) -> Graphe {
    let aretes: Vec<(usize, usize, f64)> = couts.iter().enumerate().map(|(i, &c)| (i, i + 1, c)).collect();
    graphe_pondere(couts.len() + 1, &aretes)
}

/// juste : un tube seul entre la source et le puits (V-TUBE-01). Le flux est tout le débit : `Q = I` en valeur absolue
/// (positif si la source est le petit indice, négatif sinon), la pression de la source vaut `I * cout / D` (avant la
/// mise à jour), celle du puits `+0.0`, et la conductivité suit la récurrence `D <- D + h (I - D)` écrite ici, puis
/// tend vers `I` comme `(D0 - I) (1 - h)^n`. Trois tubes (coût, `D0`, débit, `h`), dans les deux sens, 60 pas. Les
/// écarts d'arrondi viennent du solveur, qui divise puis multiplie : mesure sur le prototype, 60 pas, six cas, en
/// relatif à `I` : au plus 1,5e-16 sur `D`, 1,4e-16 sur `|Q| - I`, 2,1e-16 sur la pression de la source, 1,3e-16 sur la
/// forme fermée ; tolérance `1e-15` pour tous.
///
/// Ne couvre pas : plus d'un tube (tests suivants), le plancher (`branche_morte_et_plancher`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn un_tube_seul() {
    let cas = [(1.0, 0.75, 1.0, 0.3), (2.5, 0.6, 3.0, 0.25), (0.7, 0.9, 0.4, 0.5)];
    for &(cout, d0, debit, h) in &cas {
        for source in [0_usize, 1] {
            let puits = 1 - source;
            let g = graphe_pondere(2, &[(0, 1, cout)]);
            let mut tubes = Tubes::new(&g, vec![d0]).unwrap();
            let (mut attendu, mut facteur) = (d0, 1.0_f64);
            let (mut pire_d, mut pire_q, mut pire_p, mut pire_forme) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
            for _ in 0..60 {
                let avant = tubes.conductivites()[0];
                let etat = tubes.avancer(&g, source, puits, debit, h).unwrap();
                let signe = if source == 0 { 1.0 } else { -1.0 };
                pire_q = pire_q.max((etat.flux[0] - signe * debit).abs() / debit);
                let p_attendue = debit * cout / avant;
                pire_p = pire_p.max((etat.pressions[source] - p_attendue).abs() / p_attendue);
                assert_eq!(etat.pressions[puits].to_bits(), 0.0_f64.to_bits(), "la pression du puits est +0.0");
                attendu += h * (debit - attendu);
                facteur *= 1.0 - h;
                let d = tubes.conductivites()[0];
                pire_d = pire_d.max((d - attendu).abs() / debit);
                // la forme fermée : D - I = (D0 - I) (1 - h)^n
                pire_forme = pire_forme.max((d - debit - (d0 - debit) * facteur).abs() / debit);
            }
            assert!(pire_q <= 1e-15, "cas {cout} {d0} {debit} {h}, source {source} : |Q| - I = {pire_q:e} I");
            assert!(pire_p <= 1e-15, "cas {cout} {d0} {debit} {h}, source {source} : pression, écart {pire_p:e}");
            assert!(pire_d <= 1e-15, "cas {cout} {d0} {debit} {h}, source {source} : D, écart {pire_d:e} I");
            assert!(
                pire_forme <= 1e-15,
                "cas {cout} {d0} {debit} {h}, source {source} : forme fermée, écart {pire_forme:e} I"
            );
            assert!((tubes.conductivites()[0] - debit).abs() < 1e-3 * (d0 - debit).abs(), "D tend vers I");
        }
    }
    // l'état rendu est de la bonne taille
    let g = graphe_pondere(2, &[(0, 1, 1.0)]);
    let mut tubes = Tubes::uniformes(&g, 0.5).unwrap();
    let etat = tubes.avancer(&g, 0, 1, 1.0, 0.5).unwrap();
    assert_eq!((etat.pressions.len(), etat.flux.len(), tubes.conductivites().len()), (2, 1, 1));
}

/// juste : une chaîne de cinq tubes de coûts et de conductivités différents entre la source (un bout) et le puits
/// (l'autre) : le même flux `I` traverse chaque tube, donc chaque `D` suit sa propre récurrence `D <- D + h (I - D)`
/// (V-TUBE-02), et la pression de la source vaut `I * somme(cout / D)` (avant la mise à jour). 40 pas, `h = 0,2`, `I =
/// 2`, dans les deux sens. Mesure sur le prototype, en relatif à `I` (ou à la pression) : l'écart de `D` à la
/// récurrence est au plus 1,2e-15, celui de `|Q| - I` 5,3e-15, celui de la pression de la source 3,0e-15 ; tolérances
/// `4e-15`, `1,5e-14` et `1e-14`.
///
/// Ne couvre pas : une chaîne dont la source est au milieu (voir `branche_morte_et_plancher`), un graphe maillé.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn chaine_de_tubes() {
    let couts = [1.0, 2.0, 0.5, 1.5, 3.0];
    let d0 = [0.9, 0.55, 0.7, 1.2, 0.35];
    let (debit, h) = (2.0, 0.2);
    let g = chaine(&couts);
    for (source, puits) in [(0_usize, 5_usize), (5, 0)] {
        let mut tubes = Tubes::new(&g, d0.to_vec()).unwrap();
        let mut attendu = d0.to_vec();
        let (mut pire_d, mut pire_q, mut pire_p) = (0.0_f64, 0.0_f64, 0.0_f64);
        for _ in 0..40 {
            let p_attendue: f64 = (0..5).map(|e| couts[e] / tubes.conductivites()[e]).sum::<f64>() * debit;
            let etat = tubes.avancer(&g, source, puits, debit, h).unwrap();
            for e in 0..5 {
                attendu[e] += h * (debit - attendu[e]);
                pire_d = pire_d.max((tubes.conductivites()[e] - attendu[e]).abs() / debit);
                pire_q = pire_q.max((etat.flux[e].abs() - debit).abs() / debit);
            }
            pire_p = pire_p.max((etat.pressions[source] - p_attendue).abs() / p_attendue);
        }
        assert!(pire_d <= 4e-15, "source {source} : D, écart {pire_d:e} I");
        assert!(pire_q <= 1.5e-14, "source {source} : |Q| - I, écart {pire_q:e} I");
        assert!(pire_p <= 1e-14, "source {source} : pression de la source, écart {pire_p:e}");
        for e in 0..5 {
            assert!((tubes.conductivites()[e] - debit).abs() < 1e-3, "chaque D tend vers I = 2");
        }
    }
}

/// juste : une branche morte et le plancher (V-TUBE-03). Quatre nœuds, la source en 1, le puits en 3, le tube `(0,1)`
/// est une impasse (le nœud 0 ne touche rien d'autre : aucun flux), les tubes `(1,2)` et `(2,3)` portent tout le débit.
/// Avec `D0 = 0,8`, `h = 0,5` : le tube mort suit `D0 * 0,5^n` (en relatif à `1e-9` près ; son flux est au plus
/// `1e-12`), tant que cette valeur reste `>= PLANCHER` ; elle vaut `1,53e-6` au pas 19 et `7,63e-7` au pas 20 : **à
/// partir du pas 20 le tube vaut exactement `PLANCHER`** (les bits), et le reste ensuite (30 pas de plus). Un tube déjà
/// au plancher et sans flux y reste (les bits). Les deux autres tubes tendent vers `I = 1`. Aucune valeur n'est ni
/// nulle, ni non finie.
///
/// Ne couvre pas : le plancher sous un flux qui revient (voir `remontee_depuis_le_plancher`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn branche_morte_et_plancher() {
    let g = graphe_pondere(4, &[(0, 1, 1.0), (1, 2, 2.0), (2, 3, 1.0)]);
    let mut tubes = Tubes::new(&g, vec![0.8, 0.9, 0.6]).unwrap();
    let mut mort = 0.8_f64;
    let mut premier_plancher = None;
    for pas in 1..=50 {
        let etat = tubes.avancer(&g, 1, 3, 1.0, 0.5).unwrap();
        assert!(etat.flux[0].abs() <= 1e-12, "pas {pas} : flux de l'impasse {}", etat.flux[0]);
        mort *= 0.5;
        let d = tubes.conductivites()[0];
        if mort >= PLANCHER {
            assert!((d - mort).abs() <= 1e-9 * mort, "pas {pas} : D = {d}, attendu {mort}");
            assert!(d > PLANCHER);
        } else {
            assert_eq!(d.to_bits(), PLANCHER.to_bits(), "pas {pas} : D = {d}");
            premier_plancher.get_or_insert(pas);
        }
        assert!(tubes.conductivites().iter().all(|v| v.is_finite() && *v >= PLANCHER));
    }
    assert_eq!(premier_plancher, Some(20), "0,8 * 0,5^19 = 1,53e-6 et 0,8 * 0,5^20 = 7,63e-7");
    for e in 1..3 {
        assert!((tubes.conductivites()[e] - 1.0).abs() < 1e-9, "tube {e}");
    }
    // un tube mort qui part du plancher y reste
    let mut tubes = Tubes::new(&g, vec![PLANCHER, 0.9, 0.6]).unwrap();
    for _ in 0..10 {
        tubes.avancer(&g, 1, 3, 1.0, 0.5).unwrap();
        assert_eq!(tubes.conductivites()[0].to_bits(), PLANCHER.to_bits());
    }
}

/// juste : le plancher n'est pas un piège (V-TUBE-04) : un tube au plancher qui se trouve sur le chemin du flux
/// remonte. Chaîne de deux tubes de coût 1, conductivités `PLANCHER` et `1`, source en 0, puits en 2, débit 1, `h =
/// 0,5` : le flux est 1 dans les deux tubes (la pression de la source vaut `1e6 + 1`), donc après un pas le premier
/// tube vaut `PLANCHER + 0,5 (1 - PLANCHER)` (à `1e-12` près, le solveur perdant ici quelques chiffres : le rapport des
/// conductances est `1e6`) et le second `1`. Au bout de 40 pas, les deux tendent vers 1.
///
/// Ne couvre pas : un rapport des conductances au-delà de `1e6`.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn remontee_depuis_le_plancher() {
    let g = chaine(&[1.0, 1.0]);
    let mut tubes = Tubes::new(&g, vec![PLANCHER, 1.0]).unwrap();
    let etat = tubes.avancer(&g, 0, 2, 1.0, 0.5).unwrap();
    assert!((etat.pressions[0] - (1e6 + 1.0)).abs() <= 1e-6, "p = {}", etat.pressions[0]);
    assert!((tubes.conductivites()[0] - (PLANCHER + 0.5 * (1.0 - PLANCHER))).abs() <= 1e-12);
    assert!((tubes.conductivites()[1] - 1.0).abs() <= 1e-12);
    for _ in 0..40 {
        tubes.avancer(&g, 0, 2, 1.0, 0.5).unwrap();
    }
    assert!(tubes.conductivites().iter().all(|d| (d - 1.0).abs() < 1e-9));
}

/// juste : la **coupe autour de la source** (V-TUBE-05), le lemme de Bonifaci pour des tubes en parallèle, en temps
/// discret. Le flux total qui sort de la source est `I`, et chaque arête de la source l'évacue (la pression de la
/// source est la plus haute : aucun flux ne revient), donc `somme |Q| = I` sur ces arêtes et la somme `S` de leurs
/// conductivités suit `S <- S + h (I - S)`, à l'arrondi près, **tant qu'aucune n'est tenue par le plancher** (ce que le
/// test vérifie : toutes restent au-dessus de `1e-4`, cent fois le plancher). 12 graphes au hasard de 6 à 30 nœuds, une
/// source et un puits au hasard, `h` parmi `0,1 ; 0,25 ; 0,5`, `I` parmi `1 ; 2,5`, et respectivement 40, 25 et 10 pas
/// (de façon que le plancher reste loin). Mesure sur le prototype : l'écart de `S` à la récurrence est au plus 8,9e-16
/// (relatif à `max(S, I)`), celui de `somme |Q|` à `I` au plus 2,0e-15 (relatif à `I`), et la plus petite conductivité
/// d'une arête de la source vaut `6,8e-4` ; tolérance `6e-15` pour les deux écarts, et le test exige que la plus petite
/// reste au-dessus de `1e-4`.
///
/// Ne couvre pas : les arêtes d'une autre coupe (la somme des conductivités d'une coupe quelconque ne suit pas cette
/// loi).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn coupe_autour_de_la_source() {
    let mut alea = Alea::new(31_415_926);
    let (mut pire_s, mut pire_q, mut plus_petit) = (0.0_f64, 0.0_f64, f64::INFINITY);
    for essai in 0..12 {
        let n = 6 + alea.borne(25) as usize;
        let g = graphe_au_hasard(&mut alea, n, n, 0.5, 2.5);
        let source = alea.borne(n as u32) as usize;
        let puits = (source + 1 + alea.borne(n as u32 - 1) as usize) % n;
        let ((h, pas), debit) = ([(0.1, 40), (0.25, 25), (0.5, 10)][essai % 3], [1.0, 2.5][essai % 2]);
        let mut tubes = Tubes::au_hasard(&g, &mut alea);
        let autour = g.incidentes(source).to_vec();
        let somme = |d: &[f64]| autour.iter().fold(0.0, |s, &e| s + d[e]);
        let mut s = somme(tubes.conductivites());
        for _ in 0..pas {
            let etat = tubes.avancer(&g, source, puits, debit, h).unwrap();
            let flux_sortant = autour.iter().fold(0.0, |s, &e| s + etat.flux[e].abs());
            pire_q = pire_q.max((flux_sortant - debit).abs() / debit);
            s += h * (debit - s);
            let reel = somme(tubes.conductivites());
            pire_s = pire_s.max((reel - s).abs() / s.max(debit));
            for &e in &autour {
                plus_petit = plus_petit.min(tubes.conductivites()[e]);
            }
        }
    }
    assert!(plus_petit > 1e-4, "une arête de la source s'approche du plancher : {plus_petit:e}");
    assert!(pire_q <= 6e-15, "somme des |Q| - I : {pire_q:e} I");
    assert!(pire_s <= 6e-15, "S contre la récurrence : {pire_s:e}");
}

/// juste : les erreurs de `Tubes::new`, `Tubes::uniformes` et `Tubes::au_hasard` (V-TUBE-06). `new` : `Taille` d'abord
/// (trop ou pas assez de valeurs, même si l'une est fautive), puis `Conductivite(e)` pour le plus petit indice fautif ;
/// `PLANCHER` lui-même est accepté, le flottant juste au-dessous est refusé, comme zéro, un négatif, l'infini et NaN.
/// `uniformes` : mêmes règles (`Conductivite(0)`), et sans arête aucune valeur n'est lue. `au_hasard` : pour chaque
/// arête, par indice croissant, `0.5 + 0.5 * suivant_f64()` (réécrit ici avec un `Alea` jumeau), l'état du générateur
/// après est celui de `2 * nb_aretes` tirages de 32 bits ; valeurs du modèle Python pour la graine 1
/// (`0x3FE9B6BDDA90C320`, `0x3FECE97F8EF822C0`, `0x3FE943E8CF21B75E`, générateur `3dce35a13968f346 6d00000000000000`),
/// la graine 7 et la graine 0.
///
/// Ne couvre pas : `avancer` (tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn construction_et_erreurs() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    assert!(Tubes::new(&g, vec![1.0, 2.0, 3.0]).is_ok());
    assert_eq!(Tubes::new(&g, vec![1.0, 2.0, 3.0]).unwrap().conductivites(), &[1.0, 2.0, 3.0]);
    assert_eq!(Tubes::new(&g, vec![1.0, 1.0]), Err(ErreurTubes::Taille));
    assert_eq!(Tubes::new(&g, vec![1.0; 4]), Err(ErreurTubes::Taille));
    assert_eq!(Tubes::new(&g, vec![]), Err(ErreurTubes::Taille));
    assert_eq!(Tubes::new(&g, vec![f64::NAN, 1.0]), Err(ErreurTubes::Taille), "la taille d'abord");
    let sous_le_plancher = f64::from_bits(PLANCHER.to_bits() - 1);
    assert!(sous_le_plancher < PLANCHER && sous_le_plancher > 0.0);
    assert!(Tubes::new(&g, vec![PLANCHER, PLANCHER, PLANCHER]).is_ok(), "PLANCHER est accepté");
    assert!(Tubes::new(&g, vec![1.0, 1e300, 1.0]).is_ok());
    for (mauvaise, nom) in [
        (sous_le_plancher, "juste sous le plancher"),
        (0.0, "zéro"),
        (-0.0, "moins zéro"),
        (-1.0, "négatif"),
        (f64::INFINITY, "infini"),
        (f64::NEG_INFINITY, "moins l'infini"),
        (f64::NAN, "NaN"),
    ] {
        assert_eq!(Tubes::new(&g, vec![1.0, mauvaise, 1.0]), Err(ErreurTubes::Conductivite(1)), "{nom}");
        assert_eq!(Tubes::new(&g, vec![1.0, 1.0, mauvaise]), Err(ErreurTubes::Conductivite(2)), "{nom}");
        assert_eq!(
            Tubes::new(&g, vec![mauvaise, 1.0, mauvaise]),
            Err(ErreurTubes::Conductivite(0)),
            "{nom}, le plus petit"
        );
        assert_eq!(Tubes::uniformes(&g, mauvaise), Err(ErreurTubes::Conductivite(0)), "uniformes, {nom}");
    }
    assert_eq!(Tubes::uniformes(&g, 0.25).unwrap().conductivites(), &[0.25, 0.25, 0.25]);
    assert!(Tubes::uniformes(&g, PLANCHER).is_ok());
    let sans_arete = graphe_pondere(2, &[]);
    assert!(Tubes::new(&sans_arete, vec![]).is_ok());
    assert!(Tubes::uniformes(&sans_arete, f64::NAN).is_ok(), "aucune arête : aucune valeur à lire");
    assert!(Tubes::uniformes(&sans_arete, 1.0).unwrap().conductivites().is_empty());

    // au_hasard : la formule de la doc, un générateur jumeau
    let g = chaine(&[1.0; 6]);
    let mut alea = Alea::new(1);
    let mut jumeau = Alea::new(1);
    let tubes = Tubes::au_hasard(&g, &mut alea);
    let attendu: Vec<f64> = (0..6).map(|_| 0.5 + 0.5 * jumeau.suivant_f64()).collect();
    assert_eq!(bits(tubes.conductivites()), bits(&attendu));
    assert_eq!(alea, jumeau, "l'état du générateur après");
    let mut compte = Alea::new(1);
    for _ in 0..12 {
        compte.suivant();
    }
    assert_eq!(alea, compte, "deux tirages de 32 bits par arête");
    assert!(tubes.conductivites().iter().all(|&d| (0.5..1.0).contains(&d)));
    // les valeurs du modèle Python
    for (graine, m, valeurs, etat) in [
        (
            1_u64,
            3_usize,
            vec![0x3FE9B6BDDA90C320_u64, 0x3FECE97F8EF822C0, 0x3FE943E8CF21B75E],
            "3dce35a13968f3466d00000000000000",
        ),
        (7, 2, vec![0x3FEA454B9C36C328, 0x3FE198BC4FDB7666], "fd6eabdbbde5d2816d00000000000000"),
        (0, 1, vec![0x3FE47C28B93B98F6], "1ca3a65fe9db79bc6d00000000000000"),
    ] {
        let g = chaine(&vec![1.0; m]);
        let mut alea = Alea::new(graine);
        let tubes = Tubes::au_hasard(&g, &mut alea);
        assert_eq!(bits(tubes.conductivites()), valeurs, "graine {graine}");
        let hex: String = alea.octets().iter().map(|o| format!("{o:02x}")).collect();
        assert_eq!(hex, etat, "graine {graine} : le générateur après");
    }
    // un graphe sans arête : aucun tirage
    let mut alea = Alea::new(5);
    let tubes = Tubes::au_hasard(&sans_arete, &mut alea);
    assert!(tubes.conductivites().is_empty());
    assert_eq!(alea, Alea::new(5), "aucun tirage sans arête");
}

/// juste : les erreurs d'`avancer`, leur ORDRE, et « rien n'a changé » (V-TUBE-07). Un graphe de deux morceaux (`0-1`
/// et `2-3`) et des paramètres valides font `Solveur(Isole(2))` (le plus petit nœud non relié au puits 1). Ordre : (1)
/// `Taille` (un graphe qui n'a pas autant d'arêtes que de conductivités, même avec tout le reste fautif) ; (2) `Noeud`
/// (source ou puits hors du graphe, ou source = puits), même avec un débit et un pas fautifs ; (3) `Debit` (zéro,
/// négatif, NaN, infini), même avec un pas fautif ; (4) `Pas` (zéro, un, négatif, plus que un, NaN, infini) ; (5)
/// l'erreur du solveur. Chaque appel qui échoue laisse les conductivités égales (aux bits) à ce qu'elles étaient. Les
/// bornes valides passent : `h` juste sous 1 et juste au-dessus de 0 (un dénormalisé), un débit petit.
///
/// Ne couvre pas : `Solveur(Pivot(_))` (un rapport de conductances au-delà de `2^53`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn erreurs_et_leur_ordre() {
    let deux_morceaux = graphe_pondere(4, &[(0, 1, 1.0), (2, 3, 1.0)]);
    let connexe = chaine(&[1.0, 1.0, 1.0]);
    let tubes = Tubes::new(&connexe, vec![0.5, 0.6, 0.7]).unwrap();
    let avant = tubes.clone();
    let essayer = |graphe: &Graphe, source: usize, puits: usize, debit: f64, h: f64| -> Result<(), ErreurTubes> {
        let mut t = tubes.clone();
        let resultat = t.avancer(graphe, source, puits, debit, h).map(|_| ());
        if resultat.is_err() {
            assert_eq!(bits(t.conductivites()), bits(avant.conductivites()), "rien n'a changé sur une erreur");
        }
        resultat
    };
    // le cas positif d'abord
    assert_eq!(essayer(&connexe, 0, 3, 1.0, 0.5), Ok(()));
    assert_eq!(essayer(&connexe, 3, 0, 0.001, f64::from_bits(1)), Ok(()), "h dénormalisé et débit petit : valides");
    assert_eq!(essayer(&connexe, 1, 2, 7.0, 0.999_999_9), Ok(()));
    // 1. Taille : le graphe de quatre arêtes... ici deux morceaux de deux arêtes contre trois conductivités
    assert_eq!(essayer(&deux_morceaux, 9, 9, f64::NAN, f64::NAN), Err(ErreurTubes::Taille));
    assert_eq!(essayer(&graphe_pondere(2, &[(0, 1, 1.0)]), 0, 1, 1.0, 0.5), Err(ErreurTubes::Taille));
    // 2. Noeud
    for (s, p) in [(1, 1), (0, 4), (4, 0), (usize::MAX, 0), (0, usize::MAX), (4, 4), (2, 2)] {
        assert_eq!(essayer(&connexe, s, p, 0.0, 7.0), Err(ErreurTubes::Noeud), "source {s}, puits {p}");
    }
    // 3. Debit
    for debit in [0.0, -0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(essayer(&connexe, 0, 3, debit, 0.5), Err(ErreurTubes::Debit), "débit {debit}");
        assert_eq!(essayer(&connexe, 0, 3, debit, 1.0), Err(ErreurTubes::Debit), "le débit avant le pas");
    }
    // 4. Pas
    for h in [0.0, -0.0, 1.0, -0.5, 1.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(essayer(&connexe, 0, 3, 1.0, h), Err(ErreurTubes::Pas), "pas {h}");
    }
    // 5. le solveur : le graphe est connexe seulement pour ses quatre nœuds s'il y a trois arêtes ; ici deux morceaux
    let mut t = Tubes::new(&deux_morceaux, vec![0.5, 0.9]).unwrap();
    let avant = t.clone();
    assert_eq!(t.avancer(&deux_morceaux, 0, 1, 1.0, 0.5), Err(ErreurTubes::Solveur(ErreurSolveur::Isole(2))));
    assert_eq!(t, avant, "rien n'a changé");
    assert_eq!(t.avancer(&deux_morceaux, 3, 2, 1.0, 0.5), Err(ErreurTubes::Solveur(ErreurSolveur::Isole(0))));
    assert_eq!(t, avant, "rien n'a changé");
    assert_eq!(t.avancer(&deux_morceaux, 3, 2, 1.0, 1.0), Err(ErreurTubes::Pas), "le pas avant le solveur");
}

/// juste : `avancer` rend, aux bits, les conductivités, les pressions et les flux du pas naïf écrit dans le test
/// (V-TUBE-08), sur 20 graphes au hasard (de 3 à 40 nœuds, des coûts dans `[0,5 ; 3[`), une source et un puits au
/// hasard, `h` parmi `0,05 ; 0,3 ; 0,5 ; 0,9`, un débit parmi `1 ; 2,5 ; 0,3`, 12 pas de suite. Les pressions et les
/// flux de l'`Etat` sont ceux du calcul avec les conductivités d'AVANT la mise à jour. Une autre façon d'écrire le pas
/// (`d + h * (q - d)` en `d * (1 - h) + h * q`, la mise à jour avant le calcul des flux, un plancher oublié ou appliqué
/// avant la somme) change des bits quelque part.
///
/// Ne couvre pas : les graphes de plus de 40 nœuds ; les bits du solveur lui-même (étape 0).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn contre_un_pas_naif() {
    let mut alea = Alea::new(27_182_818);
    let mut pas_faits = 0;
    for essai in 0..20 {
        let n = 3 + alea.borne(38) as usize;
        let g = graphe_au_hasard(&mut alea, n, n, 0.5, 2.5);
        let source = alea.borne(n as u32) as usize;
        let puits = (source + 1 + alea.borne(n as u32 - 1) as usize) % n;
        let (h, debit) = ([0.05, 0.3, 0.5, 0.9][essai % 4], [1.0, 2.5, 0.3][essai % 3]);
        let mut tubes = Tubes::au_hasard(&g, &mut alea);
        let mut d = tubes.conductivites().to_vec();
        for pas in 0..12 {
            let etat = tubes.avancer(&g, source, puits, debit, h).unwrap();
            let (p, q) = pas_naif(&g, &mut d, source, puits, debit, h);
            assert_eq!(etat.pressions.len(), n);
            assert_eq!(etat.flux.len(), g.nb_aretes());
            assert_eq!(bits(&etat.pressions), bits(&p), "graphe {essai}, pas {pas} : pressions");
            assert_eq!(bits(&etat.flux), bits(&q), "graphe {essai}, pas {pas} : flux");
            assert_eq!(bits(tubes.conductivites()), bits(&d), "graphe {essai}, pas {pas} : conductivités");
            pas_faits += 1;
        }
    }
    assert_eq!(pas_faits, 240);
}

/// juste : trois pas sur le pont de Wheatstone (V-TUBE-09), calculés par le modèle Python `ref_tubes.py` (flottants
/// IEEE 754, mêmes opérations dans le même ordre, solveur de `ref_solveur.py`) : arêtes `(0,1) (0,2) (1,2) (1,3)
/// (2,3)`, coûts `1 ; 2 ; 1,5 ; 0,5 ; 3`, conductivités de départ `0,9 ; 0,6 ; 0,75 ; 0,5 ; 1`, source 0, puits 3,
/// débit 1,5, `h = 0,3`. Le premier pas donne des valeurs exactes en décimal (`0,9675 ; 0,5325 ; 0,525 ; 0,6875 ;
/// 0,8125`) : l'arête centrale `(1,2)` y est à flux nul (le pont est équilibré), sans qu'aucun plancher n'agisse.
///
/// Ne couvre pas : des graphes plus grands (voir `contre_un_pas_naif`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn pont_de_wheatstone_en_python() {
    let g = graphe_pondere(4, &[(0, 1, 1.0), (0, 2, 2.0), (1, 2, 1.5), (1, 3, 0.5), (2, 3, 3.0)]);
    let mut tubes = Tubes::new(&g, vec![0.9, 0.6, 0.75, 0.5, 1.0]).unwrap();
    let attendu_d: [[u64; 5]; 3] = [
        [0x3FEEF5C28F5C28F6, 0x3FE10A3D70A3D70A, 0x3FE0CCCCCCCCCCCD, 0x3FE6000000000000, 0x3FEA000000000000],
        [0x3FF09490E6490D0C, 0x3FDDADBC66DBCBCD, 0x3FD82D92D509F193, 0x3FEB37BA102D9A0E, 0x3FE4C845EFD265F3],
        [0x3FF1A3F473702780, 0x3FD9702E323F61FC, 0x3FD1B9C3B3CDE88B, 0x3FEF8575258A3B7E, 0x3FE07A8ADA75C482],
    ];
    let attendu_p: [[u64; 4]; 3] = [
        [0x4003000000000000, 0x3FF2000000000000, 0x3FF2000000000000, 0],
        [0x40010EC753EA713C, 0x3FECA62027383F28, 0x3FEFC849117FFA74, 0],
        [0x3FFFA064887FD356, 0x3FE86EDA5CDFB311, 0x3FEDBBDFA68B4C42, 0],
    ];
    let attendu_q: [[u64; 5]; 3] = [
        [0x3FF2000000000000, 0x3FD8000000000000, 0, 0x3FF2000000000000, 0x3FD8000000000000],
        [0x3FF325D5585D510A, 0x3FD368AA9E8ABBD2, 0xBFA18C18532B4C10, 0x3FF3B2361AF6AB6C, 0x3FD1372794255254],
        [0x3FF41D321220BA90, 0x3FCF166F6EFA2B7F, 0xBFA55C588763F112, 0x3FF4C814D65BDA18, 0x3FC9BF594D212F3C],
    ];
    for pas in 0..3 {
        let etat = tubes.avancer(&g, 0, 3, 1.5, 0.3).unwrap();
        assert_eq!(bits(&etat.pressions), attendu_p[pas], "pas {pas} : pressions");
        assert_eq!(bits(&etat.flux), attendu_q[pas], "pas {pas} : flux");
        assert_eq!(bits(tubes.conductivites()), attendu_d[pas], "pas {pas} : conductivités");
    }
}

/// juste : une mise à l'échelle exacte (V-TUBE-10). Multiplier toutes les conductivités de départ ET le débit par 4
/// (une puissance de 2 : exact) multiplie par 4 les conductivités et les flux à chaque pas, aux bits, et ne change pas
/// les pressions (les conductances sont multipliées par 4 et les flux aussi : `p = I / g` ne bouge pas). Six pas, `h =
/// 0,5`, 8 graphes au hasard ; aucun plancher n'agit (le plus petit `D` possible est `0,5 * 0,5^6 > 1e-3`). Un code qui
/// mélange le débit avec une constante, ou oublie de diviser par le coût, ne passe pas.
///
/// Ne couvre pas : une mise à l'échelle qui toucherait le plancher.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn mise_a_l_echelle_exacte() {
    let mut alea = Alea::new(1_618_033);
    for essai in 0..8 {
        let n = 4 + alea.borne(20) as usize;
        let g = graphe_au_hasard(&mut alea, n, n, 0.5, 2.5);
        let source = alea.borne(n as u32) as usize;
        let puits = (source + 1 + alea.borne(n as u32 - 1) as usize) % n;
        let mut simple = Tubes::au_hasard(&g, &mut alea);
        let mut quadruple = Tubes::new(&g, simple.conductivites().iter().map(|d| 4.0 * d).collect()).unwrap();
        for pas in 0..6 {
            let a = simple.avancer(&g, source, puits, 1.0, 0.5).unwrap();
            let b = quadruple.avancer(&g, source, puits, 4.0, 0.5).unwrap();
            // ce test compare deux résultats entre eux : il ne doit pas passer sur deux résultats vides
            assert_eq!((a.pressions.len(), a.flux.len()), (n, g.nb_aretes()), "graphe {essai}, pas {pas}");
            assert_eq!(simple.conductivites().len(), g.nb_aretes(), "graphe {essai}, pas {pas}");
            assert!(a.pressions[source] > 0.0, "graphe {essai}, pas {pas} : la source est sous pression");
            assert_eq!(bits(&a.pressions), bits(&b.pressions), "graphe {essai}, pas {pas} : pressions");
            let fois4 = |v: &[f64]| v.iter().map(|x| 4.0 * x).collect::<Vec<f64>>();
            assert_eq!(bits(&fois4(&a.flux)), bits(&b.flux), "graphe {essai}, pas {pas} : flux");
            assert_eq!(
                bits(&fois4(simple.conductivites())),
                bits(quadruple.conductivites()),
                "graphe {essai}, pas {pas}"
            );
        }
    }
}

/// juste : `Taille` dans les deux sens (V-TUBE-11, la vérification 1 d'`avancer` : « `graphe.nb_aretes()` doit être le
/// nombre de conductivités »). Des tubes faits pour une chaîne de deux arêtes, avancés sur une chaîne de trois, rendent
/// `Taille`, et l'inverse aussi (trop peu de conductivités comme trop) ; rien n'a changé ; sur leur propre graphe, les
/// mêmes tubes avancent. Ajouté après la campagne de fautes du juge.
///
/// Ne couvre pas : deux graphes différents de même nombre d'arêtes (la doc ne le vérifie pas).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn taille_dans_les_deux_sens() {
    let (petit, grand) = (chaine(&[1.0, 1.0]), chaine(&[1.0, 1.0, 1.0]));
    let mut tubes = Tubes::uniformes(&petit, 1.0).unwrap();
    assert_eq!(tubes.avancer(&grand, 0, 1, 1.0, 0.5).unwrap_err(), ErreurTubes::Taille, "trop peu de conductivités");
    assert_eq!(bits(tubes.conductivites()), bits(&[1.0, 1.0]));
    assert!(tubes.avancer(&petit, 0, 2, 1.0, 0.5).is_ok());
    let mut tubes = Tubes::uniformes(&grand, 1.0).unwrap();
    assert_eq!(tubes.avancer(&petit, 0, 1, 1.0, 0.5).unwrap_err(), ErreurTubes::Taille, "trop de conductivités");
    assert_eq!(bits(tubes.conductivites()), bits(&[1.0, 1.0, 1.0]));
    assert!(tubes.avancer(&grand, 0, 3, 1.0, 0.5).is_ok());
}
