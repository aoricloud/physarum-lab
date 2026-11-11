//! Étape 1, partie 1R : `reseaux::Loi` et le pas d'un `Reseau` (`avancer_paire`, `avancer`, `au_hasard`, `new`).
//! Valeurs attendues : `references/etape-1r-valeurs.md`, identifiants `V-LOI-xx`.
//!
//! Quatre sortes de preuves, toutes indépendantes du code jugé :
//! - la **loi** : des valeurs écrites à la main ou calculées par le modèle Python `ref_reseaux.py` (aux bits), sa
//!   parité, sa saturation ;
//! - la **loi linéaire est celle du cœur** : `avancer_paire` rend, bit pour bit, ce que `Tubes::avancer` rend ;
//! - des **cas fermés** pour la sigmoïde : un tube seul (`Q = I0`, donc `D <- D + h (f(I0) - D)`), une chaîne ;
//! - un **pas naïf** écrit dans le test (`pas_naif`) avec `solveur::pressions` et `solveur::flux`, pour les deux lois ;
//!   puis les **erreurs** de la doc, leur ordre et « rien n'a changé ».
// Les boucles à indices suivent les procédures de la doc ligne pour ligne.
#![allow(clippy::needless_range_loop)]

#[path = "common/graphes_au_hasard.rs"]
mod graphes_au_hasard;

use graphes_au_hasard::{graphe_au_hasard, graphe_pondere};
use physarum_lab::alea::Alea;
use physarum_lab::graphe::Graphe;
use physarum_lab::reseaux::{ErreurReseau, Loi, Reseau};
use physarum_lab::solveur::{ErreurSolveur, flux, pressions};
use physarum_lab::tubes::{PLANCHER, Tubes};

fn bits(valeurs: &[f64]) -> Vec<u64> {
    valeurs.iter().map(|v| v.to_bits()).collect()
}

/// Un pas du modèle écrit naïvement d'après la doc (« La loi » et « Le pas à plusieurs sources de nourriture »).
fn pas_naif(
    g: &Graphe,
    d: &mut [f64],
    loi: Loi,
    source: usize,
    puits: usize,
    debit: f64,
    h: f64,
) -> (Vec<f64>, Vec<f64>) {
    let mut conductances = Vec::new();
    for e in 0..g.nb_aretes() {
        conductances.push(d[e] / g.aretes()[e].cout);
    }
    let mut injections = vec![0.0; g.nb_noeuds()];
    injections[source] = debit;
    let p = pressions(g, &conductances, &injections, puits).expect("pressions");
    let q = flux(g, &conductances, &p);
    for e in 0..g.nb_aretes() {
        let f = match loi {
            Loi::Lineaire => q[e].abs(),
            Loi::Sigmoide => {
                let q2 = q[e] * q[e];
                q2 / (1.0 + q2)
            }
        };
        let nouveau = d[e] + h * (f - d[e]);
        d[e] = if nouveau >= PLANCHER { nouveau } else { PLANCHER };
    }
    (p, q)
}

/// La sigmoïde de Tero 2010 avec `γ = 2`, écrite ici.
fn sigmoide(i0: f64) -> f64 {
    i0 * i0 / (1.0 + i0 * i0)
}

/// Une chaîne `0 - 1 - ... - k` : les arêtes `(i, i+1)` de coût `couts[i]`.
fn chaine(couts: &[f64]) -> Graphe {
    let aretes: Vec<(usize, usize, f64)> = couts.iter().enumerate().map(|(i, &c)| (i, i + 1, c)).collect();
    graphe_pondere(couts.len() + 1, &aretes)
}

/// juste : les valeurs de la loi (V-LOI-01). `Lineaire` est la valeur absolue (`-0.0` donne `+0.0`, NaN reste NaN).
/// `Sigmoide` est `q2 / (1 + q2)` avec `q2 = q * q` : `f(0) = +0.0`, `f(1) = 0.5`, `f(2) = 0.8`, `f(3) = 0.9`, `f(0.5)
/// = 0.2` (des quotients exactement arrondis : les littéraux de Rust), et, pour les autres arguments, les bits du
/// modèle Python `ref_reseaux.py` (mêmes opérations dans le même ordre) ; la parité ; `+inf` quand `q * q` déborde
/// (`1e155`, `inf`) vaut `1.0` ; NaN rend NaN. Tero et al. 2010, p. 441 : `f(|Q|) = |Q|^γ / (1 + |Q|^γ)`, ici `γ = 2`
/// (l'article prend 1,8).
///
/// Ne couvre pas : la valeur de `γ` de l'article (`powf` est interdit : on ne la retrouve pas).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn valeurs_de_la_loi() {
    let lineaire = [(3.5, 3.5), (-2.0, 2.0), (0.0, 0.0), (-0.0, 0.0), (1e-300, 1e-300), (-1e300, 1e300)];
    for (q, attendu) in lineaire {
        assert_eq!(Loi::Lineaire.valeur(q).to_bits(), f64::to_bits(attendu), "lineaire({q})");
    }
    assert_eq!(Loi::Lineaire.valeur(f64::NEG_INFINITY), f64::INFINITY);
    assert!(Loi::Lineaire.valeur(f64::NAN).is_nan());
    let sig: [(f64, u64); 14] = [
        (0.0, 0),
        (-0.0, 0),
        (1.0, 0x3FE0_0000_0000_0000),
        (2.0, 0x3FE9_9999_9999_999A),
        (3.0, 0x3FEC_CCCC_CCCC_CCCD),
        (0.5, 0x3FC9_9999_9999_999A),
        (0.1, 0x3F84_46F8_6562_D9FC),
        (1e3, 0x3FEF_FFFD_E721_2F18),
        (1e-3, 0x3EB0_C6F6_873C_67ED),
        (7.25, 0x3FEF_670E_A319_6412),
        (1e6, 0x3FEF_FFFF_FFFF_DCD1),
        (1e154, 0x3FF0_0000_0000_0000),
        (1e155, 0x3FF0_0000_0000_0000),
        (f64::INFINITY, 0x3FF0_0000_0000_0000),
    ];
    for (q, attendu) in sig {
        assert_eq!(Loi::Sigmoide.valeur(q).to_bits(), attendu, "sigmoide({q})");
        assert_eq!(Loi::Sigmoide.valeur(-q).to_bits(), attendu, "sigmoide(-{q}) : la loi est paire");
    }
    assert_eq!(Loi::Sigmoide.valeur(2.0), 0.8);
    assert_eq!(Loi::Sigmoide.valeur(3.0), 0.9);
    assert_eq!(Loi::Sigmoide.valeur(0.5), 0.2);
    assert!(Loi::Sigmoide.valeur(f64::NAN).is_nan());
    // croissante et strictement sous 1 tant que |q| <= 1e6
    let mut precedent = -1.0;
    for q in [0.0, 1e-3, 0.1, 0.5, 1.0, 2.0, 7.25, 100.0, 1e3, 1e6] {
        let f = Loi::Sigmoide.valeur(q);
        assert!(f > precedent && f < 1.0, "f({q}) = {f}");
        precedent = f;
    }
    // la sigmoïde n'est pas la loi linéaire
    assert_ne!(Loi::Sigmoide.valeur(2.0), Loi::Lineaire.valeur(2.0));
    assert_ne!(Loi::Sigmoide, Loi::Lineaire);
}

/// juste : avec la loi linéaire, `avancer_paire` est, bit pour bit, `Tubes::avancer` (V-LOI-02) : les pressions, les
/// flux et les conductivités de chaque pas, 40 pas, sur 8 graphes connexes tirés au hasard (8 à 15 nœuds), trois
/// nourritures dont la paire change à chaque pas, deux débits. `Tubes::avancer` est déjà éprouvé par les tests du cœur
/// ; c'est l'équivalence que la doc promet.
///
/// Ne couvre pas : la sigmoïde (`pas_naif_aux_bits`), le tirage de la paire (`etape_1r_tirage`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn la_loi_lineaire_est_celle_du_coeur() {
    let paires = [(0_usize, 1_usize), (1, 2), (2, 0), (0, 2), (2, 1), (1, 0)];
    for graine in 1..=8_u64 {
        let mut alea = Alea::new(graine);
        let n = 8 + graine as usize;
        let g = graphe_au_hasard(&mut alea, n, 6, 0.5, 2.0);
        let d0 = Tubes::au_hasard(&g, &mut alea).conductivites().to_vec();
        let foods = vec![0, n / 2, n - 1];
        let debit = if graine % 2 == 0 { 1.0 } else { 2.5 };
        let mut tubes = Tubes::new(&g, d0.clone()).unwrap();
        let mut reseau = Reseau::new(&g, d0.clone(), foods.clone(), debit, Loi::Lineaire).unwrap();
        for t in 0..40 {
            let (i, j) = paires[t % 6];
            let a = tubes.avancer(&g, foods[i], foods[j], debit, 0.3).unwrap();
            let b = reseau.avancer_paire(&g, foods[i], foods[j], 0.3).unwrap();
            assert_eq!((b.source, b.puits), (foods[i], foods[j]));
            assert_eq!(a.pressions.len(), g.nb_noeuds());
            assert_eq!(bits(&a.pressions), bits(&b.etat.pressions), "graine {graine}, pas {t}");
            assert_eq!(bits(&a.flux), bits(&b.etat.flux), "graine {graine}, pas {t}");
            assert_eq!(bits(tubes.conductivites()), bits(reseau.conductivites()), "graine {graine}, pas {t}");
        }
        assert_ne!(bits(reseau.conductivites()), bits(&d0), "les conductivités ont bougé");
    }
}

/// juste : un tube seul entre deux nourritures, sous la sigmoïde (V-LOI-03) : `Q = I0` à chaque pas, donc la
/// conductivité suit `D <- D + h (f(I0) - D)` avec `f(I0) = I0² / (1 + I0²)` écrite ici, et tend vers `f(I0)`, qui est
/// **sous 1** (`0,5` pour `I0 = 1`, `0,9` pour `I0 = 3`) ; la pression de la source vaut `I0 * cout / D` (avant la mise
/// à jour) et celle du puits `+0.0`. Trois tubes (coût, `D0`, `I0`, `h`), dans les deux sens, 60 pas. Mesure sur le
/// prototype (les six cas, 60 pas), en relatif à `f(I0)` ou à la pression : écart de `D` à la récurrence au plus
/// 4,0e-16, écart de `|Q| - I0` (relatif à `I0`) au plus 1,5e-16, écart de la pression 2,2e-16 ; tolérance `1e-15`
/// (facteur 2,5 : l'écart vient de l'arrondi du solveur, qui divise puis multiplie).
///
/// Ne couvre pas : plus d'un tube (`chaine_sous_la_sigmoide`), le plancher.
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn un_tube_seul_sous_la_sigmoide() {
    let cas = [(1.0, 0.75, 1.0, 0.3), (2.5, 0.6, 3.0, 0.25), (0.7, 0.9, 0.4, 0.5)];
    for &(cout, d0, i0, h) in &cas {
        for source in [0_usize, 1] {
            let puits = 1 - source;
            let g = graphe_pondere(2, &[(0, 1, cout)]);
            let mut reseau = Reseau::new(&g, vec![d0], vec![0, 1], i0, Loi::Sigmoide).unwrap();
            let (mut attendu, cible) = (d0, sigmoide(i0));
            let (mut pire_d, mut pire_q, mut pire_p) = (0.0_f64, 0.0_f64, 0.0_f64);
            for _ in 0..60 {
                let avant = reseau.conductivites()[0];
                let pas = reseau.avancer_paire(&g, source, puits, h).unwrap();
                let signe = if source == 0 { 1.0 } else { -1.0 };
                pire_q = pire_q.max((pas.etat.flux[0] - signe * i0).abs() / i0);
                let p_attendue = i0 * cout / avant;
                pire_p = pire_p.max((pas.etat.pressions[source] - p_attendue).abs() / p_attendue);
                assert_eq!(pas.etat.pressions[puits].to_bits(), 0.0_f64.to_bits());
                attendu += h * (cible - attendu);
                pire_d = pire_d.max((reseau.conductivites()[0] - attendu).abs() / cible);
            }
            assert!(pire_q <= 1e-15, "cas {cout} {d0} {i0} {h}, source {source} : |Q| - I0 = {pire_q:e} I0");
            assert!(pire_p <= 1e-15, "cas {cout} {d0} {i0} {h}, source {source} : pression, écart {pire_p:e}");
            assert!(pire_d <= 1e-15, "cas {cout} {d0} {i0} {h}, source {source} : D, écart {pire_d:e}");
            let d = reseau.conductivites()[0];
            assert!((d - cible).abs() < 1e-3 * cible && d < 1.0, "D tend vers f(I0) = {cible}, trouvé {d}");
        }
    }
    assert_eq!(sigmoide(1.0), 0.5);
    let g = graphe_pondere(2, &[(0, 1, 1.0)]);
    let mut reseau = Reseau::new(&g, vec![0.5], vec![0, 1], 1.0, Loi::Sigmoide).unwrap();
    // `f(1) = 0.5` : une conductivité déjà à 0.5 y reste, au bit près
    reseau.avancer_paire(&g, 0, 1, 0.5).unwrap();
    assert_eq!(reseau.conductivites()[0].to_bits(), 0.5_f64.to_bits());
}

/// juste : une chaîne de cinq tubes entre deux nourritures (les deux bouts), sous la sigmoïde (V-LOI-04) : le même flux
/// `I0` traverse chaque tube, donc chaque `D` suit sa propre récurrence `D <- D + h (f(I0) - D)`, et la pression de la
/// source vaut `I0 * somme(cout / D)` (avant la mise à jour). 40 pas, `h = 0,2`, `I0 = 2` (`f = 0,8`), dans les deux
/// sens. Mesure sur le prototype, en relatif à `f(I0)` (ou à la pression) : écart de `D` au plus 6,9e-16, de `|Q| - I0`
/// 5,6e-15 (relatif à `I0`), de la pression 3,2e-15 ; tolérances `4e-15`, `1,5e-14` et `1e-14` (les mêmes que la chaîne
/// du cœur ; facteurs 5,8, 2,7 et 3,2).
///
/// Ne couvre pas : un graphe maillé (`pas_naif_aux_bits`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn chaine_sous_la_sigmoide() {
    let couts = [1.0, 2.0, 0.5, 1.5, 3.0];
    let d0 = [0.9, 0.55, 0.7, 1.2, 0.35];
    let (i0, h) = (2.0, 0.2);
    let g = chaine(&couts);
    for (source, puits) in [(0_usize, 5_usize), (5, 0)] {
        let mut reseau = Reseau::new(&g, d0.to_vec(), vec![0, 5], i0, Loi::Sigmoide).unwrap();
        let mut attendu = d0.to_vec();
        let cible = sigmoide(i0);
        assert_eq!(cible, 0.8);
        let (mut pire_d, mut pire_q, mut pire_p) = (0.0_f64, 0.0_f64, 0.0_f64);
        for _ in 0..40 {
            let p_attendue: f64 = (0..5).map(|e| couts[e] / reseau.conductivites()[e]).sum::<f64>() * i0;
            let pas = reseau.avancer_paire(&g, source, puits, h).unwrap();
            for e in 0..5 {
                attendu[e] += h * (cible - attendu[e]);
                pire_d = pire_d.max((reseau.conductivites()[e] - attendu[e]).abs() / cible);
                pire_q = pire_q.max((pas.etat.flux[e].abs() - i0).abs() / i0);
            }
            pire_p = pire_p.max((pas.etat.pressions[source] - p_attendue).abs() / p_attendue);
        }
        assert!(pire_d <= 4e-15, "source {source} : D, écart {pire_d:e}");
        assert!(pire_q <= 1.5e-14, "source {source} : |Q| - I0, écart {pire_q:e}");
        assert!(pire_p <= 1e-14, "source {source} : pression de la source, écart {pire_p:e}");
        for e in 0..5 {
            assert!((reseau.conductivites()[e] - cible).abs() < 1e-3, "chaque D tend vers f(I0) = 0,8");
        }
    }
}

/// juste : la saturation (V-LOI-05). Sous la sigmoïde, aucune conductivité ne dépasse `max(D de départ, f(I0))`, donc
/// reste **sous 1** quand les conductivités de départ le sont, même avec un grand débit : sur 6 graphes au hasard, `I0
/// = 6`, 100 pas à `h = 0,5`, la plus grande conductivité reste `< 1` ; avec la loi linéaire, au même débit, les tubes
/// que traverse le flux dépassent 1 (la plus grande approche `I0`, qui vaut 6 : ici `> 1,5`, ce que la sigmoïde
/// interdit). Un tube seul tombant d'une conductivité de départ de `5,0` descend vers `f(I0)` sans jamais passer sous.
///
/// Ne couvre pas : la valeur exacte de la saturation sur un graphe maillé (celle des cas fermés).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn saturation() {
    for graine in 1..=6_u64 {
        let mut alea = Alea::new(graine);
        let g = graphe_au_hasard(&mut alea, 10, 6, 0.5, 2.0);
        let d0 = Tubes::au_hasard(&g, &mut alea).conductivites().to_vec();
        let (mut sig, mut lin) = (
            Reseau::new(&g, d0.clone(), vec![0, 9], 6.0, Loi::Sigmoide).unwrap(),
            Reseau::new(&g, d0, vec![0, 9], 6.0, Loi::Lineaire).unwrap(),
        );
        for _ in 0..100 {
            sig.avancer_paire(&g, 0, 9, 0.5).unwrap();
            lin.avancer_paire(&g, 0, 9, 0.5).unwrap();
        }
        let max = |d: &[f64]| d.iter().cloned().fold(0.0, f64::max);
        let (m_sig, m_lin) = (max(sig.conductivites()), max(lin.conductivites()));
        assert!(m_sig < 1.0 && m_sig > 0.0, "graine {graine} : sigmoïde, max {m_sig}");
        assert!(m_lin > 1.5, "graine {graine} : linéaire, max {m_lin}");
    }
    let g = graphe_pondere(2, &[(0, 1, 1.0)]);
    let mut reseau = Reseau::new(&g, vec![5.0], vec![0, 1], 1.0, Loi::Sigmoide).unwrap();
    let mut precedent = 5.0;
    for _ in 0..80 {
        reseau.avancer_paire(&g, 0, 1, 0.25).unwrap();
        let d = reseau.conductivites()[0];
        assert!(d < precedent && d >= 0.5, "D décroît vers 0,5 sans passer dessous : {d}");
        precedent = d;
    }
}

/// juste : `avancer_paire` contre un pas naïf écrit dans le test (V-LOI-06), pour **les deux lois** : les pressions,
/// les flux et les conductivités de chaque pas, aux bits, 25 pas, sur 12 graphes connexes tirés au hasard (6 à 17
/// nœuds), des paires et des débits qui varient, `h` entre `0,05` et `0,9`. Le pas naïf emploie `solveur::pressions` et
/// `solveur::flux` (éprouvés à l'étape 0). Une conductivité qui descend au plancher y reste tenue par `PLANCHER` (un
/// graphe de chemin long et un débit fort l'y poussent : le test exige qu'au moins un tube y soit arrivé).
///
/// Ne couvre pas : les erreurs (`erreurs_et_ordre`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn pas_naif_aux_bits() {
    let mut au_plancher = 0;
    for (num, loi) in [Loi::Lineaire, Loi::Sigmoide].into_iter().cycle().take(12).enumerate() {
        let graine = 100 + num as u64;
        let mut alea = Alea::new(graine);
        let n = 6 + num;
        let g = graphe_au_hasard(&mut alea, n, 4 + num, 0.3, 3.0);
        let d0 = Tubes::au_hasard(&g, &mut alea).conductivites().to_vec();
        let foods = vec![0, n / 3, n / 2, n - 1];
        let debit = [0.5, 1.0, 2.0, 7.0][num % 4];
        let h = [0.05, 0.3, 0.5, 0.9][num % 4];
        let mut reseau = Reseau::new(&g, d0.clone(), foods.clone(), debit, loi).unwrap();
        let mut d = d0;
        for _ in 0..25 {
            let i = alea.borne(4) as usize;
            let j = (i + 1 + alea.borne(3) as usize) % 4;
            let (p, q) = pas_naif(&g, &mut d, loi, foods[i], foods[j], debit, h);
            let pas = reseau.avancer_paire(&g, foods[i], foods[j], h).unwrap();
            assert_eq!(pas.etat.pressions.len(), n);
            assert_eq!(bits(&pas.etat.pressions), bits(&p), "{loi:?}, graine {graine}");
            assert_eq!(bits(&pas.etat.flux), bits(&q), "{loi:?}, graine {graine}");
            assert_eq!(bits(reseau.conductivites()), bits(&d), "{loi:?}, graine {graine}");
        }
        au_plancher += reseau.conductivites().iter().filter(|&&x| x == PLANCHER).count();
        assert_eq!((reseau.debit(), reseau.loi(), reseau.nourritures()), (debit, loi, &foods[..]));
    }
    assert!(au_plancher > 0, "au moins un tube est descendu au plancher");
}

/// juste : le plancher, pour les deux lois (V-LOI-07) : une branche morte (un tube qui ne mène nulle part, donc sans
/// flux) décroît comme `(1 - h)^n` (la récurrence `D <- D + h (0 - D)` écrite ici, **aux bits** : mesuré, le solveur
/// rend un flux nul exact dans ce graphe) puis reste **exactement** à `PLANCHER = 1e-6`. Nœuds `0, 1, 2`, tubes `(0,1)`
/// et `(1,2)`, nourritures 0 et 1 : le tube `(1,2)` ne porte aucun flux. Départ `0,8`, `h = 0,25` : le plancher est
/// atteint au pas numéro 47 (`0,8 * 0,75^48 = 8,05e-7`, calculé en Python), pour les deux lois.
///
/// Ne couvre pas : la remontée depuis le plancher.
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn plancher_sous_la_sigmoide() {
    assert_eq!(PLANCHER, 1e-6);
    let g = graphe_pondere(3, &[(0, 1, 1.0), (1, 2, 1.0)]);
    for loi in [Loi::Lineaire, Loi::Sigmoide] {
        let mut reseau = Reseau::new(&g, vec![0.8, 0.8], vec![0, 1], 1.0, loi).unwrap();
        let mut attendu = 0.8_f64;
        let mut arrive = None;
        for pas in 0..120 {
            reseau.avancer_paire(&g, 0, 1, 0.25).unwrap();
            let nouveau = attendu + 0.25 * (0.0 - attendu);
            attendu = if nouveau >= PLANCHER { nouveau } else { PLANCHER };
            let d = reseau.conductivites()[1];
            if arrive.is_none() && d == PLANCHER {
                arrive = Some(pas);
            }
            assert_eq!(d.to_bits(), attendu.to_bits(), "{loi:?}, pas {pas}");
        }
        assert_eq!(arrive, Some(47), "{loi:?} : le plancher est atteint au pas 47");
        assert_eq!(reseau.conductivites()[1], PLANCHER);
    }
}

/// juste : les erreurs, leur ordre, et « rien n'a changé » (V-LOI-08), pour `Reseau::new`, `Reseau::au_hasard`,
/// `avancer_paire` et `avancer` (doc de `reseaux`, « Les vérifications ») :
/// - `new` : `Taille`, puis `Conductivite(plus petit indice fautif)` (NaN, `inf`, zéro, négatif, sous `PLANCHER` ;
///   `PLANCHER` lui-même passe), puis `Nourriture` (aucune, une seule, hors du graphe, en double), puis `Debit` (zéro,
///   négatif, NaN, `inf`) ; chaque erreur l'emporte sur la suivante quand deux sont présentes ;
/// - `au_hasard` : `Nourriture` puis `Debit`, **avant tout tirage** (`alea` n'a pas bougé) ; sur un succès, les
///   conductivités sont celles de `Tubes::au_hasard` sur le même générateur, qui a avancé de la même façon ;
/// - `avancer_paire` : `Taille` (un autre graphe), `Nourriture` (le graphe a moins de nœuds que le plus grand indice de
///   nourriture), `Noeud` (hors du graphe, ou source égale au puits), `Pas` (`0`, `1`, négatif, NaN, `inf`), puis
///   `Solveur(Isole(i))` (graphe non connexe) ; les conductivités ne bougent pas ;
/// - `avancer` : les mêmes, et `alea` n'a pas bougé non plus.
///
/// Ne couvre pas : le détail du solveur (`etape_0_solveur`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn erreurs_et_ordre() {
    let g = chaine(&[1.0, 1.0, 1.0, 1.0]); // 5 nœuds, 4 arêtes
    let bon = vec![0.7; 4];
    let ok = |d: Vec<f64>, f: Vec<usize>, i0: f64| Reseau::new(&g, d, f, i0, Loi::Sigmoide);
    assert!(ok(bon.clone(), vec![0, 4], 1.0).is_ok());
    // la taille d'abord
    assert_eq!(ok(vec![0.7; 3], vec![0, 4], 1.0), Err(ErreurReseau::Taille));
    assert_eq!(ok(vec![0.7; 5], vec![0, 4], 1.0), Err(ErreurReseau::Taille));
    assert_eq!(ok(vec![f64::NAN; 3], vec![0], f64::NAN), Err(ErreurReseau::Taille));
    // les conductivités : le plus petit indice fautif
    for mauvais in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -1.0, 9.9e-7, -0.0] {
        let mut d = bon.clone();
        d[2] = mauvais;
        d[3] = f64::NAN;
        assert_eq!(ok(d, vec![0, 4], 1.0), Err(ErreurReseau::Conductivite(2)), "{mauvais}");
    }
    let mut d = bon.clone();
    d[1] = PLANCHER;
    assert!(ok(d, vec![0, 4], 1.0).is_ok(), "PLANCHER lui-même passe");
    assert_eq!(ok(vec![f64::NAN; 4], vec![0], 0.0), Err(ErreurReseau::Conductivite(0)), "avant Nourriture");
    // les nourritures
    for mauvaises in [vec![], vec![2], vec![0, 5], vec![7, 8], vec![1, 3, 1], vec![2, 2]] {
        assert_eq!(ok(bon.clone(), mauvaises.clone(), 1.0), Err(ErreurReseau::Nourriture), "{mauvaises:?}");
        assert_eq!(ok(bon.clone(), mauvaises, 0.0), Err(ErreurReseau::Nourriture), "avant Debit");
    }
    assert!(ok(bon.clone(), vec![4, 3, 2, 1, 0], 1.0).is_ok(), "tous les nœuds, dans le désordre");
    // le débit
    for mauvais in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0] {
        assert_eq!(ok(bon.clone(), vec![0, 4], mauvais), Err(ErreurReseau::Debit), "{mauvais}");
    }
    assert!(ok(bon.clone(), vec![0, 4], 1e-300).is_ok() && ok(bon.clone(), vec![0, 4], 1e300).is_ok());

    // au_hasard : les mêmes erreurs, avant tout tirage ; sur un succès, `Tubes::au_hasard`
    let neuf = Alea::new(5);
    for (f, i0, attendu) in [
        (vec![0], 1.0, ErreurReseau::Nourriture),
        (vec![0, 0], 1.0, ErreurReseau::Nourriture),
        (vec![0, 5], 1.0, ErreurReseau::Nourriture),
        (vec![0, 4], 0.0, ErreurReseau::Debit),
        (vec![0], 0.0, ErreurReseau::Nourriture),
        (vec![0, 4], f64::NAN, ErreurReseau::Debit),
    ] {
        let mut alea = neuf.clone();
        assert_eq!(Reseau::au_hasard(&g, &mut alea, f, i0, Loi::Lineaire), Err(attendu));
        assert_eq!(alea, neuf, "aucun tirage sur une erreur");
    }
    let (mut a, mut b) = (neuf.clone(), neuf.clone());
    let r = Reseau::au_hasard(&g, &mut a, vec![3, 0], 2.0, Loi::Sigmoide).unwrap();
    let t = Tubes::au_hasard(&g, &mut b);
    assert_eq!(bits(r.conductivites()), bits(t.conductivites()));
    assert_eq!(a, b, "le générateur a avancé de 2 * 4 tirages, comme pour Tubes::au_hasard");
    assert_ne!(a, neuf);
    assert_eq!(r.nourritures(), &[3, 0]);
    assert_eq!((r.debit(), r.loi()), (2.0, Loi::Sigmoide));

    // avancer_paire et avancer : rien ne change sur une erreur
    let autre = chaine(&[1.0, 1.0, 1.0]); // 4 nœuds, 3 arêtes : pas la même taille
    let coupe = graphe_pondere(5, &[(0, 1, 1.0), (1, 2, 1.0), (3, 4, 1.0), (0, 2, 1.0)]); // 3 et 4 isolés du puits
    let mut reseau = Reseau::new(&g, bon.clone(), vec![0, 4], 1.0, Loi::Sigmoide).unwrap();
    let avant = reseau.clone();
    let mut alea = Alea::new(9);
    let alea_avant = alea.clone();
    // Taille
    assert_eq!(reseau.avancer_paire(&autre, 0, 3, 0.5), Err(ErreurReseau::Taille));
    assert_eq!(reseau.avancer(&autre, &mut alea, 0.5), Err(ErreurReseau::Taille));
    // Nourriture : même nombre d'arêtes (4), mais 4 nœuds seulement : la nourriture 4 n'y est pas ; Taille avant tout
    let g4 = graphe_pondere(4, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0), (0, 3, 1.0)]);
    assert_eq!(reseau.avancer_paire(&g4, 0, 3, 0.5), Err(ErreurReseau::Nourriture));
    assert_eq!(reseau.avancer_paire(&g4, 0, 9, 0.0), Err(ErreurReseau::Nourriture), "avant Noeud et Pas");
    assert_eq!(reseau.avancer(&g4, &mut alea, 0.5), Err(ErreurReseau::Nourriture));
    assert_eq!(reseau.avancer(&g4, &mut alea, 7.0), Err(ErreurReseau::Nourriture), "avant Pas");
    // Noeud
    for (s, p) in [(5, 0), (0, 5), (7, 8), (2, 2), (0, 0), (4, 4)] {
        assert_eq!(reseau.avancer_paire(&g, s, p, 0.5), Err(ErreurReseau::Noeud), "({s}, {p})");
        assert_eq!(reseau.avancer_paire(&g, s, p, 5.0), Err(ErreurReseau::Noeud), "Noeud avant Pas");
    }
    // Pas
    for h in [0.0, 1.0, -0.5, 1.5, f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.0] {
        assert_eq!(reseau.avancer_paire(&g, 0, 4, h), Err(ErreurReseau::Pas), "h = {h}");
        assert_eq!(reseau.avancer(&g, &mut alea, h), Err(ErreurReseau::Pas), "h = {h}");
    }
    for h in [1e-9, 0.5, 0.999_999_999] {
        let mut copie = reseau.clone();
        assert!(copie.avancer_paire(&g, 0, 4, h).is_ok(), "h = {h}");
    }
    assert_eq!((reseau.clone(), alea.clone()), (avant.clone(), alea_avant.clone()));
    // des nœuds de nourriture quelconques, source et puits ne sont pas obligés d'en être
    let mut libre = reseau.clone();
    assert!(libre.avancer_paire(&g, 1, 3, 0.5).is_ok());
    assert_ne!(libre, reseau);
    // Solveur : un graphe non connexe ; le plus petit nœud isolé du puits
    let mut sur_coupe = Reseau::new(&coupe, vec![0.7; 4], vec![0, 1], 1.0, Loi::Lineaire).unwrap();
    let avant_coupe = sur_coupe.clone();
    assert_eq!(sur_coupe.avancer_paire(&coupe, 0, 1, 0.5), Err(ErreurReseau::Solveur(ErreurSolveur::Isole(3))));
    assert_eq!(sur_coupe.avancer_paire(&coupe, 0, 1, 2.0), Err(ErreurReseau::Pas), "Pas avant Solveur");
    let mut alea = Alea::new(9);
    let alea_avant = alea.clone();
    assert!(matches!(sur_coupe.avancer(&coupe, &mut alea, 0.5), Err(ErreurReseau::Solveur(ErreurSolveur::Isole(_)))));
    assert_eq!((sur_coupe, alea), (avant_coupe, alea_avant), "ni les tubes ni le générateur n'ont bougé");
}

/// juste : `Reseau::avancer` sur une erreur laisse le générateur intact, et sur un succès l'avance exactement du tirage
/// de la paire (deux tirages pour `k = 2`, V-LOI-09) : après un pas réussi avec deux nourritures, `alea` est celui qui
/// a fait deux `suivant()` de plus ; `borne(1)` consomme un tirage, c'est ce que la doc dit.
///
/// Ne couvre pas : le contenu de la paire (`etape_1r_tirage`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn avancer_consomme_deux_tirages_pour_deux_nourritures() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    let mut reseau = Reseau::new(&g, vec![0.7; 3], vec![0, 3], 1.0, Loi::Lineaire).unwrap();
    let mut alea = Alea::new(3);
    let mut attendu = alea.clone();
    for _ in 0..10 {
        let pas = reseau.avancer(&g, &mut alea, 0.5).unwrap();
        attendu.suivant();
        attendu.suivant();
        assert_eq!(alea, attendu);
        assert!((pas.source, pas.puits) == (0, 3) || (pas.source, pas.puits) == (3, 0));
    }
}
