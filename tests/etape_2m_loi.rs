//! Étape 2, partie 2M : la loi couplée et le pas couplé (`souvenir::Memoire::avancer`) : la force nulle rend les bits
//! des tubes sans agent, un tube seul et la recette, la saturation, la moyenne des bouts, les concentrations d'AVANT,
//! le plancher, le pas contre un pas naïf, les erreurs. Valeurs attendues : `references/etape-2m-valeurs.md`,
//! identifiants `V-LOI-xx`.
//!
//! La loi `D <- D + h ((1 + α σ(c_e)) f(|Q|) - D)` est **à nous** (Kramar et Alim 2021 ne simulent qu'un seul tube
//! élastique ; seule la forme de saturation `σ(c) = c / (c_0 + c)` vient de leur éq. 2, lue sur le texte extrait du
//! PDF). Les valeurs exactes viennent du modèle Python `ref_memoire.py` (écrit d'après la seule doc) ou d'un calcul à
//! la main dit à côté.

#[path = "common/graphes_au_hasard.rs"]
mod graphes_au_hasard;
#[path = "common/memoire_2m.rs"]
mod memoire_2m;
#[path = "common/transport_2t.rs"]
mod transport_2t;

use graphes_au_hasard::{graphe_au_hasard, graphe_pondere};
use memoire_2m::naif_pas_couple;
use physarum_lab::alea::Alea;
use physarum_lab::graphe::Graphe;
use physarum_lab::reseaux::{Loi, Reseau};
use physarum_lab::solveur::ErreurSolveur;
use physarum_lab::souvenir::{Agent, Couplage, ErreurAgent, ErreurMemoire, Memoire, Transport};
use physarum_lab::tubes::{PLANCHER, Tubes};
use transport_2t::bits;

fn f(x: u64) -> f64 {
    f64::from_bits(x)
}

/// Un graphe au hasard de 12 nœuds (connexe) pour la graine `graine`, et ses conductivités de départ.
fn graphe_et_tubes(graine: u64) -> (Graphe, Vec<f64>, Alea) {
    let mut alea = Alea::new(graine);
    let g = graphe_au_hasard(&mut alea, 12, 14, 0.5, 1.5);
    let d0 = Tubes::au_hasard(&g, &mut alea).conductivites().to_vec();
    (g, d0, alea)
}

/// juste : **avec `α = 0`, le pas couplé est, bit pour bit, le pas des tubes sans agent et le pas de l'agent seul**
/// (V-LOI-01). Sur cinq graphes au hasard de 12 nœuds et 14 arêtes de plus que l'arbre, douze pas de suite (source 0,
/// puits 11, `I0 = 2`, `h = 0.2`, des concentrations de départ au hasard, trois apports dont deux au même nœud, `κ =
/// 0.05`, `k = 0.1`) : les pressions, les flux et les conductivités de `Memoire::avancer` (loi linéaire) sont ceux de
/// `Tubes::avancer`, et les concentrations et le bilan sont ceux d'`Agent::avancer` avec le flux du pas. Puis sous la
/// loi sigmoïde, contre `Reseau::avancer_paire`. `(1.0 + 0.0 * σ) * f` vaut exactement `f` : un code qui remplacerait
/// la cible par `f + α σ …` ou qui lirait `c_0` quand `α = 0` serait vu.
///
/// Ne couvre pas : un `α > 0` (les autres tests).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn force_nulle_rend_les_bits_des_tubes_et_de_l_agent() {
    let tr = Transport { diffusion: 0.05, perte: 0.1 };
    let apports = [(3, 1.5), (7, 0.5), (3, 0.25)];
    for graine in 1..=5_u64 {
        let (g, d0, mut alea) = graphe_et_tubes(graine);
        let n = g.nb_noeuds();
        let c0: Vec<f64> = (0..n).map(|_| alea.suivant_f64()).collect();
        // la loi linéaire contre Tubes::avancer et Agent::avancer
        let mut tubes = Tubes::new(&g, d0.clone()).unwrap();
        let mut agent = Agent::new(&g, c0.clone()).unwrap();
        let mut m = Memoire::new(&g, d0.clone(), Agent::new(&g, c0.clone()).unwrap()).unwrap();
        for pas in 0..12 {
            let etat = tubes.avancer(&g, 0, n - 1, 2.0, 0.2).unwrap();
            let bilan = agent.avancer(&g, &etat.flux, &tr, &apports, 0.2).unwrap();
            let cp = Couplage { force: 0.0, saturation: 0.7 };
            let p = m.avancer(&g, 0, n - 1, 2.0, Loi::Lineaire, &cp, &tr, &apports, 0.2).unwrap();
            assert_eq!(bits(&p.etat.pressions), bits(&etat.pressions), "graine {graine} pas {pas} : pressions");
            assert_eq!(bits(&p.etat.flux), bits(&etat.flux), "graine {graine} pas {pas} : flux");
            assert_eq!(bits(m.conductivites()), bits(tubes.conductivites()), "graine {graine} pas {pas} : D");
            assert_eq!(bits(m.agent().concentrations()), bits(agent.concentrations()), "graine {graine} pas {pas}");
            assert_eq!(p.bilan, bilan, "graine {graine} pas {pas} : bilan");
        }
        // la loi sigmoïde contre Reseau::avancer_paire
        let mut reseau = Reseau::new(&g, d0.clone(), vec![0, n - 1], 2.0, Loi::Sigmoide).unwrap();
        let mut agent = Agent::new(&g, c0.clone()).unwrap();
        let mut m = Memoire::new(&g, d0.clone(), Agent::new(&g, c0.clone()).unwrap()).unwrap();
        for pas in 0..12 {
            let paire = reseau.avancer_paire(&g, n - 1, 0, 0.2).unwrap();
            let bilan = agent.avancer(&g, &paire.etat.flux, &tr, &apports, 0.2).unwrap();
            let cp = Couplage { force: 0.0, saturation: 1e-3 };
            let p = m.avancer(&g, n - 1, 0, 2.0, Loi::Sigmoide, &cp, &tr, &apports, 0.2).unwrap();
            assert_eq!(bits(&p.etat.flux), bits(&paire.etat.flux), "sigmoïde graine {graine} pas {pas} : flux");
            assert_eq!(bits(m.conductivites()), bits(reseau.conductivites()), "sigmoïde graine {graine} pas {pas}");
            assert_eq!(bits(m.agent().concentrations()), bits(agent.concentrations()));
            assert_eq!(p.bilan, bilan);
        }
    }
}

/// juste : **un tube seul** (V-LOI-02) : deux nœuds, une arête de longueur 2 et de coût 2, `D = 0.8`, l'agent `[1.0,
/// 0.0]` (source puis puits), `α = 3`, `c_0 = 1`, `κ = 0`, `k = 0`, `I0 = 1`, `h = 0.25`, loi linéaire. Le flux est `Q
/// = 1.0` exactement à chaque pas (le solveur divise puis multiplie : exact ici) ; la recette est écrite dans le test :
/// `c_e = (c_a + c_b) * 0.5`, `σ = c_e / (c_0 + c_e)`, `cible = (1.0 + α σ) |Q|`, `D <- D + h (cible - D)`, avec les
/// concentrations d'**avant** le pas ; les concentrations avancent par le transport de 2T (`[0.75, 0.25]`, puis
/// `[0.5625, 0.375]`, puis `[0.421875, 0.421875]`, en fractions exactes : `h Q c_donneur` passe du donneur au receveur,
/// et la sortie emporte `h s c` au puits). Les trois `D` successifs sont aussi ceux du modèle Python :
/// `0x3FF199999999999A`, `0x3FF5333333333334`, `0x3FF7BAD3560F4046`.
///
/// Ne couvre pas : plusieurs arêtes (les autres tests).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn un_tube_seul_suit_la_recette() {
    let g = graphe_pondere(2, &[(0, 1, 2.0)]);
    let agent = Agent::new(&g, vec![1.0, 0.0]).unwrap();
    let mut m = Memoire::new(&g, vec![0.8], agent).unwrap();
    let cp = Couplage { force: 3.0, saturation: 1.0 };
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    let (mut d, mut c) = (0.8_f64, [1.0_f64, 0.0_f64]);
    let attendu_d = [0x3FF1_9999_9999_999A_u64, 0x3FF5_3333_3333_3334, 0x3FF7_BAD3_560F_4046];
    let attendu_c = [[0.75, 0.25], [0.5625, 0.375], [0.421875, 0.421875]];
    for pas in 0..3 {
        let p = m.avancer(&g, 0, 1, 1.0, Loi::Lineaire, &cp, &tr, &[], 0.25).unwrap();
        assert_eq!(p.etat.flux, vec![1.0], "pas {pas} : Q = I exactement");
        // la recette, avec les concentrations d'avant
        let ce = (c[0] + c[1]) * 0.5;
        let sigma = ce / (1.0 + ce);
        let cible = (1.0 + 3.0 * sigma) * 1.0;
        d += 0.25 * (cible - d);
        assert_eq!(m.conductivites()[0].to_bits(), d.to_bits(), "pas {pas} : la recette");
        assert_eq!(m.conductivites()[0].to_bits(), attendu_d[pas], "pas {pas} : le modèle Python");
        c = attendu_c[pas];
        assert_eq!(m.agent().concentrations(), &c[..], "pas {pas} : l'agent");
    }
}

/// juste : **la saturation** (V-LOI-03) : sur le même tube seul (`D = 0.8`, `α = 3`, `c_0 = 1`, `I0 = 1`, `h = 0.25`,
/// l'agent `(c, c)` aux deux bouts, donc `c_e = c`), un pas rend `D' = D + h ((1 + 3 σ) - D)` avec `σ = c / (1 + c)` :
/// `c = 0` donne `0.8 + 0.25 · 0.2 = 0.85` (`0x3FEB333333333334`, **le pas sans agent**), `c = 0.25` donne `σ = 0.2` et
/// `D' = 1.0` exactement, `c = c_0 = 1` donne `σ = 0.5` (**la moitié de l'effet maximal**) et `D' = 1.225`
/// (`0x3FF399999999999A`), `c = 4` donne `D' = 1.45` (`0x3FF7333333333334`), `c = 10^6` donne `0x3FF99998D0460B43` (en
/// dessous du maximum `0.8 + 0.25 (4 - 0.8) = 1.6`, le pas avec `σ = 1` : **la saturation ne dépasse jamais `1 + α`**).
/// La suite des `D'` croît avec `c` strictement. Valeurs du modèle Python.
///
/// Ne couvre pas : une loi sigmoïde (`le_pas_couple_contre_un_pas_naif`).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn la_saturation_de_l_effet() {
    let g = graphe_pondere(2, &[(0, 1, 2.0)]);
    let cp = Couplage { force: 3.0, saturation: 1.0 };
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    let cas: [(f64, u64); 5] = [
        (0.0, 0x3FEB_3333_3333_3334),
        (0.25, 0x3FF0_0000_0000_0000),
        (1.0, 0x3FF3_9999_9999_999A),
        (4.0, 0x3FF7_3333_3333_3334),
        (1e6, 0x3FF9_9998_D046_0B43),
    ];
    let mut precedent = 0.0_f64;
    for (c, attendu) in cas {
        let mut m = Memoire::new(&g, vec![0.8], Agent::new(&g, vec![c, c]).unwrap()).unwrap();
        m.avancer(&g, 0, 1, 1.0, Loi::Lineaire, &cp, &tr, &[], 0.25).unwrap();
        let d = m.conductivites()[0];
        assert_eq!(d.to_bits(), attendu, "c = {c}");
        assert!(d > precedent, "D' croît avec c : {d} après {precedent}");
        assert!(d < 1.6, "jamais au-delà du pas avec σ = 1 : {d}");
        precedent = d;
    }
    // c = 0 : le pas sans agent (α = 0)
    let mut sans = Memoire::new(&g, vec![0.8], Agent::zero(&g)).unwrap();
    let nul = Couplage { force: 0.0, saturation: 1.0 };
    sans.avancer(&g, 0, 1, 1.0, Loi::Lineaire, &nul, &tr, &[], 0.25).unwrap();
    assert_eq!(sans.conductivites()[0].to_bits(), f(0x3FEB_3333_3333_3334).to_bits());
}

/// juste : **la concentration d'une arête est la moyenne des concentrations de ses deux bouts** (V-LOI-04) : sur la
/// chaîne `0-1-2-3` (coûts 1, `D = [0.9, 0.8, 0.7]`, `I0 = 1` de 0 à 3, `h = 0.25`, `α = 3`, `c_0 = 1`) avec de l'agent
/// seulement au nœud 1 (`c = 2`) : les arêtes `0-1` et `1-2` voient `c_e = 1` (`σ = 0.5`), l'arête `2-3` voit `c_e = 0`
/// et garde **exactement** la valeur du pas sans agent. Avec l'agent seulement au nœud 2 : ce sont `1-2` et `2-3`, pas
/// `0-1`. Un code qui prendrait le `max`, la somme ou un seul bout se tromperait. Valeurs du modèle Python.
///
/// Ne couvre pas : les concentrations non nulles aux deux bouts avec des valeurs différentes (le pas naïf).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn la_concentration_de_l_arete_est_la_moyenne_de_ses_bouts() {
    let g = graphe_pondere(4, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0)]);
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    let avec = Couplage { force: 3.0, saturation: 1.0 };
    let sans = Couplage { force: 0.0, saturation: 1.0 };
    let d0 = vec![0.9, 0.8, 0.7];
    let sans_effet = [0x3FED_9999_9999_999A_u64, 0x3FEB_3333_3333_3333, 0x3FE8_CCCC_CCCC_CCCC];
    let cas: [([f64; 4], [u64; 3]); 2] = [
        ([0.0, 2.0, 0.0, 0.0], [0x3FF4_CCCC_CCCC_CCCD, 0x3FF3_9999_9999_9999, 0x3FE8_CCCC_CCCC_CCCC]),
        ([0.0, 0.0, 2.0, 0.0], [0x3FED_9999_9999_999A, 0x3FF3_9999_9999_9999, 0x3FF2_6666_6666_6666]),
    ];
    for (agent, attendu) in cas {
        let mut a = Memoire::new(&g, d0.clone(), Agent::new(&g, agent.to_vec()).unwrap()).unwrap();
        let mut b = Memoire::new(&g, d0.clone(), Agent::new(&g, agent.to_vec()).unwrap()).unwrap();
        a.avancer(&g, 0, 3, 1.0, Loi::Lineaire, &avec, &tr, &[], 0.25).unwrap();
        b.avancer(&g, 0, 3, 1.0, Loi::Lineaire, &sans, &tr, &[], 0.25).unwrap();
        let d: Vec<u64> = a.conductivites().iter().map(|x| x.to_bits()).collect();
        assert_eq!(d, attendu, "agent {agent:?}");
        let s: Vec<u64> = b.conductivites().iter().map(|x| x.to_bits()).collect();
        assert_eq!(s, sans_effet, "sans force, agent {agent:?}");
    }
}

/// juste : **les conductivités se mettent à jour avec les concentrations d'AVANT le transport** (V-LOI-05) : sur la
/// chaîne `0-1-2-3` (`D = [0.9, 0.8, 0.7]`, `α = 3`, `c_0 = 1`, `I0 = 1`, `h = 0.25`, loi linéaire), l'agent est nul et
/// un apport de 50 au nœud 1 pèse sur le premier pas (`c[1] = 12.5` après, `0x4029000000000000`) : les `D` après ce pas
/// sont **exactement ceux du pas sans agent** (`0x3FED99999999999A`, `0x3FEB333333333333`, `0x3FE8CCCCCCCCCCCC`) ; au
/// pas suivant, l'agent est là et ils diffèrent de ceux du pas sans agent (`0x3FF971E0387F1E04`, `0x3FF88B79D218B79F`,
/// `0x3FEA99999999999A`, contre `0x3FEE333333333334`, `0x3FEC666666666667`, `0x3FEA99999999999A`). Un code qui lirait
/// les concentrations d'après le transport changerait le premier pas.
///
/// Ne couvre pas : l'indépendance de la mise à jour vis-à-vis du découpage en sous-pas
/// (`le_pas_couple_contre_un_pas_naif` joue des sous-pas).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn les_conductivites_se_mettent_a_jour_avec_les_concentrations_d_avant() {
    let g = graphe_pondere(4, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0)]);
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    let avec = Couplage { force: 3.0, saturation: 1.0 };
    let sans = Couplage { force: 0.0, saturation: 1.0 };
    let mut a = Memoire::new(&g, vec![0.9, 0.8, 0.7], Agent::zero(&g)).unwrap();
    let mut b = a.clone();
    let apports = [(1, 50.0)];
    let bits_de = |m: &Memoire| -> Vec<u64> { m.conductivites().iter().map(|x| x.to_bits()).collect() };
    a.avancer(&g, 0, 3, 1.0, Loi::Lineaire, &avec, &tr, &apports, 0.25).unwrap();
    b.avancer(&g, 0, 3, 1.0, Loi::Lineaire, &sans, &tr, &apports, 0.25).unwrap();
    assert_eq!(a.agent().concentrations()[1].to_bits(), 0x4029_0000_0000_0000);
    assert_eq!(bits_de(&a), vec![0x3FED_9999_9999_999A, 0x3FEB_3333_3333_3333, 0x3FE8_CCCC_CCCC_CCCC]);
    assert_eq!(bits_de(&a), bits_de(&b), "premier pas : aucun effet de l'apport");
    a.avancer(&g, 0, 3, 1.0, Loi::Lineaire, &avec, &tr, &[], 0.25).unwrap();
    b.avancer(&g, 0, 3, 1.0, Loi::Lineaire, &sans, &tr, &[], 0.25).unwrap();
    assert_eq!(bits_de(&a), vec![0x3FF9_71E0_387F_1E04, 0x3FF8_8B79_D218_B79F, 0x3FEA_9999_9999_999A]);
    assert_eq!(bits_de(&b), vec![0x3FEE_3333_3333_3334, 0x3FEC_6666_6666_6667, 0x3FEA_9999_9999_999A]);
}

/// juste : **le pas couplé contre un pas naïf** (V-LOI-06) : sur quatre graphes au hasard de 12 nœuds, pour chaque loi
/// (linéaire, sigmoïde), dix pas de suite avec `α = 5`, `c_0 = 0.6`, `κ = 0.2`, `k = 0.05`, `I0 = 6`, `h = 0.9` (assez
/// grand pour que le transport se découpe en plusieurs sous-pas : le test le vérifie), deux apports ; l'état (les
/// conductivités et les concentrations), les pressions, les flux et le bilan de `Memoire::avancer` sont, au bit, ceux
/// d'un pas écrit dans le test (`tests/common/memoire_2m.rs`, d'après la doc seule : `solveur`, le transport naïf de 2T
/// et la recette de la loi couplée). Les deux états sont avancés indépendamment.
///
/// Ne couvre pas : le plancher à `α > 0` (`le_plancher_tient_sans_flux`) ni la famille des nourritures
/// (`etape_2m_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn le_pas_couple_contre_un_pas_naif() {
    let tr = Transport { diffusion: 0.2, perte: 0.05 };
    let cp = Couplage { force: 5.0, saturation: 0.6 };
    let apports = [(2, 1.5), (9, 0.75)];
    let mut sous_pas_max = 0;
    for sigmoide in [false, true] {
        let loi = if sigmoide { Loi::Sigmoide } else { Loi::Lineaire };
        for graine in 1..=4_u64 {
            let (g, d0, mut alea) = graphe_et_tubes(graine + 40);
            let n = g.nb_noeuds();
            let c0: Vec<f64> = (0..n).map(|_| 2.0 * alea.suivant_f64()).collect();
            let mut m = Memoire::new(&g, d0.clone(), Agent::new(&g, c0.clone()).unwrap()).unwrap();
            let (mut d, mut c) = (d0, c0);
            for pas in 0..10 {
                let attendu = naif_pas_couple(&g, &d, &c, 0, n - 1, 6.0, sigmoide, 5.0, 0.6, 0.2, 0.05, &apports, 0.9);
                let p = m.avancer(&g, 0, n - 1, 6.0, loi, &cp, &tr, &apports, 0.9).unwrap();
                let nom = format!("sigmoïde {sigmoide} graine {graine} pas {pas}");
                assert_eq!(bits(&p.etat.pressions), bits(&attendu.pressions), "{nom} : pressions");
                assert_eq!(bits(&p.etat.flux), bits(&attendu.flux), "{nom} : flux");
                assert_eq!(bits(m.conductivites()), bits(&attendu.d), "{nom} : D");
                assert_eq!(bits(m.agent().concentrations()), bits(&attendu.c), "{nom} : c");
                let b = [p.bilan.sortie, p.bilan.perte, p.bilan.apport];
                assert_eq!(bits(&b), bits(&attendu.bilan), "{nom} : bilan");
                assert_eq!(p.bilan.sous_pas, attendu.sous_pas, "{nom} : sous-pas");
                sous_pas_max = sous_pas_max.max(p.bilan.sous_pas);
                d = attendu.d;
                c = attendu.c;
            }
        }
    }
    assert!(sous_pas_max >= 2, "le transport doit se découper : {sous_pas_max} sous-pas au plus");
}

/// juste : **sans flux, aucun effet ; et le plancher tient** (V-LOI-07) : une arête pendante (le nœud 3 n'est relié que
/// par l'arête `1-3`, que le flux ne traverse pas : le flux y est nul à l'arrondi près) dont les deux bouts portent de
/// l'agent (`c = 5`) et `α = 8` : au bout de 60 pas de `h = 0.5`, sa conductivité vaut **exactement `PLANCHER`**, comme
/// sans agent ; aucune conductivité n'est jamais sous `PLANCHER` ; les deux arêtes du chemin source-puits (`I0 = 1`)
/// restent au moins égales à `1` (la loi linéaire les ramène vers `1`, l'agent peut les renforcer). Un code qui ferait
/// croître un tube sans flux parce que l'agent y est serait vu.
///
/// Ne couvre pas : un tube qui revient du plancher quand le flux revient (V-TUBE du cœur).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn le_plancher_tient_sans_flux() {
    // 0-1, 1-2 (le chemin), 1-3 (la pendante) ; source 0, puits 2
    let g = graphe_pondere(4, &[(0, 1, 1.0), (1, 2, 1.0), (1, 3, 1.0)]);
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    let cp = Couplage { force: 8.0, saturation: 1.0 };
    let mut m = Memoire::new(&g, vec![1.0, 1.0, 0.9], Agent::new(&g, vec![0.0, 5.0, 0.0, 5.0]).unwrap()).unwrap();
    for pas in 0..60 {
        let p = m.avancer(&g, 0, 2, 1.0, Loi::Lineaire, &cp, &tr, &[], 0.5).unwrap();
        assert!(p.etat.flux[2].abs() < 1e-12, "pas {pas} : l'arête pendante ne porte aucun flux : {}", p.etat.flux[2]);
        assert!(m.conductivites().iter().all(|&d| d >= PLANCHER), "pas {pas} : sous le plancher");
    }
    assert_eq!(m.conductivites()[2].to_bits(), PLANCHER.to_bits(), "la pendante est exactement au plancher");
    assert!(
        m.conductivites()[0] >= 1.0 && m.conductivites()[1] >= 1.0,
        "le chemin est renforcé : {:?}",
        m.conductivites()
    );
}

/// Une configuration valide à partir de laquelle chaque cas d'erreur change une chose.
struct Cas {
    g: Graphe,
    m: Memoire,
    source: usize,
    puits: usize,
    debit: f64,
    cp: Couplage,
    tr: Transport,
    apports: Vec<(usize, f64)>,
    h: f64,
}

fn cas() -> Cas {
    let g = graphe_pondere(4, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0), (0, 3, 2.0)]);
    let m = Memoire::new(&g, vec![0.9, 0.8, 0.7, 0.6], Agent::new(&g, vec![0.0, 1.0, 0.5, 0.0]).unwrap()).unwrap();
    Cas {
        g,
        m,
        source: 0,
        puits: 3,
        debit: 1.0,
        cp: Couplage { force: 2.0, saturation: 1.0 },
        tr: Transport { diffusion: 0.1, perte: 0.1 },
        apports: vec![(1, 0.5)],
        h: 0.25,
    }
}

impl Cas {
    /// Le résultat de `avancer` sur une **copie** de la mémoire, et la mémoire d'après.
    fn jouer(&self) -> (Result<(), ErreurMemoire>, Memoire) {
        let mut m = self.m.clone();
        let r = m
            .avancer(
                &self.g,
                self.source,
                self.puits,
                self.debit,
                Loi::Lineaire,
                &self.cp,
                &self.tr,
                &self.apports,
                self.h,
            )
            .map(|_| ());
        (r, m)
    }
}

/// juste : **les erreurs du pas couplé et leur ordre** (V-LOI-08). Une configuration valide réussit ; chaque cas change
/// une chose et rend la bonne erreur ; **quand deux fautes se cumulent, c'est la première de la liste de la doc** :
/// `Taille` (conductivités ou concentrations en trop), `Noeud` (source égale au puits ou hors du graphe), `Debit` (0,
/// NaN, infini, négatif), `Pas` (0, 1, 1.5, NaN, négatif), `Force` (négative, NaN, infinie), `Saturation` (0, négative,
/// NaN, infinie), `Solveur(Isole(i))` (un graphe non connexe), `Agent(Diffusion)`, `Agent(Perte)`,
/// `Agent(Apport(rang))` (le plus petit rang fautif), `Agent(SousPas)` (`κ = 1e7` : plus d'un million de sous-pas),
/// `Debordement` (`α = 1e308` avec de l'agent et `I0 = 100` : la cible dépasse le plus grand flottant). **Sur une
/// erreur, rien n'a changé** : la mémoire est égale, champ pour champ, à ce qu'elle était.
///
/// Ne couvre pas : `Agent(Flux)` ni `Agent(Concentration)` (le solveur rend des flux finis et l'agent est valide par
/// construction).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn les_erreurs_du_pas_couple() {
    let (ok, apres) = cas().jouer();
    assert_eq!(ok, Ok(()));
    assert_ne!(apres, cas().m, "un pas réussi change l'état");

    let attendre = |c: &Cas, erreur: ErreurMemoire, nom: &str| {
        let (r, m) = c.jouer();
        assert_eq!(r, Err(erreur), "{nom}");
        assert_eq!(m, c.m, "{nom} : rien ne doit changer");
    };
    // chacune seule
    let mut c = cas();
    c.source = 3;
    attendre(&c, ErreurMemoire::Noeud, "source = puits");
    let mut c = cas();
    c.puits = 4;
    attendre(&c, ErreurMemoire::Noeud, "puits hors du graphe");
    let mut c = cas();
    c.source = 9;
    attendre(&c, ErreurMemoire::Noeud, "source hors du graphe");
    for debit in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut c = cas();
        c.debit = debit;
        attendre(&c, ErreurMemoire::Debit, &format!("débit {debit}"));
    }
    for h in [0.0, 1.0, 1.5, -0.1, f64::NAN, f64::INFINITY] {
        let mut c = cas();
        c.h = h;
        attendre(&c, ErreurMemoire::Pas, &format!("h {h}"));
    }
    for force in [-0.5, f64::NAN, f64::INFINITY] {
        let mut c = cas();
        c.cp.force = force;
        attendre(&c, ErreurMemoire::Force, &format!("force {force}"));
    }
    for saturation in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let mut c = cas();
        c.cp.saturation = saturation;
        attendre(&c, ErreurMemoire::Saturation, &format!("saturation {saturation}"));
    }
    let mut c = cas();
    c.tr.diffusion = -0.1;
    attendre(&c, ErreurMemoire::Agent(ErreurAgent::Diffusion), "diffusion négative");
    let mut c = cas();
    c.tr.perte = f64::NAN;
    attendre(&c, ErreurMemoire::Agent(ErreurAgent::Perte), "perte NaN");
    let mut c = cas();
    c.tr.diffusion = f64::INFINITY;
    c.tr.perte = -1.0;
    attendre(&c, ErreurMemoire::Agent(ErreurAgent::Diffusion), "diffusion avant perte");
    let mut c = cas();
    c.apports = vec![(1, 0.5), (7, 0.5), (2, -1.0)];
    attendre(&c, ErreurMemoire::Agent(ErreurAgent::Apport(1)), "apport au nœud 7 : le rang 1");
    let mut c = cas();
    c.apports = vec![(1, 0.5), (2, f64::NAN), (9, 1.0)];
    attendre(&c, ErreurMemoire::Agent(ErreurAgent::Apport(1)), "apport NaN : le plus petit rang");
    let mut c = cas();
    c.tr.diffusion = 1e7;
    attendre(&c, ErreurMemoire::Agent(ErreurAgent::SousPas), "κ = 1e7 : plus d'un million de sous-pas");
    let mut c = cas();
    c.cp = Couplage { force: 1e308, saturation: 1e-3 };
    c.debit = 100.0;
    attendre(&c, ErreurMemoire::Debordement, "α = 1e308 avec un flux de 100 : la cible déborde");
    // un graphe non connexe : le nœud 4 n'est relié à rien
    let g5 = graphe_pondere(5, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0), (0, 3, 2.0)]);
    let m5 = Memoire::new(&g5, vec![0.9, 0.8, 0.7, 0.6], Agent::zero(&g5)).unwrap();
    let mut c = cas();
    c.g = g5;
    c.m = m5;
    attendre(&c, ErreurMemoire::Solveur(ErreurSolveur::Isole(4)), "graphe non connexe");

    // deux fautes cumulées : la première de la liste
    let mut c = cas();
    c.g = graphe_pondere(3, &[(0, 1, 1.0), (1, 2, 1.0)]); // 2 arêtes pour 4 conductivités
    c.source = 1;
    c.puits = 1;
    attendre(&c, ErreurMemoire::Taille, "Taille avant Noeud");
    let mut c = cas();
    c.source = 2;
    c.puits = 2;
    c.debit = 0.0;
    attendre(&c, ErreurMemoire::Noeud, "Noeud avant Debit");
    let mut c = cas();
    c.debit = -1.0;
    c.h = 2.0;
    attendre(&c, ErreurMemoire::Debit, "Debit avant Pas");
    let mut c = cas();
    c.h = 2.0;
    c.cp.force = -1.0;
    attendre(&c, ErreurMemoire::Pas, "Pas avant Force");
    let mut c = cas();
    c.cp = Couplage { force: -1.0, saturation: 0.0 };
    attendre(&c, ErreurMemoire::Force, "Force avant Saturation");
    let g5 = graphe_pondere(5, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0), (0, 3, 2.0)]);
    let m5 = Memoire::new(&g5, vec![0.9, 0.8, 0.7, 0.6], Agent::zero(&g5)).unwrap();
    let mut c = cas();
    c.g = g5;
    c.m = m5;
    c.cp.saturation = 0.0;
    attendre(&c, ErreurMemoire::Saturation, "Saturation avant Solveur");
    c.cp.saturation = 1.0;
    c.tr.diffusion = -1.0;
    attendre(&c, ErreurMemoire::Solveur(ErreurSolveur::Isole(4)), "Solveur avant Agent");
}

/// juste : **les erreurs de construction** (V-LOI-09) : `Memoire::new` rend `Taille` (trop ou trop peu de
/// conductivités, ou une concentration de moins ou de plus), puis `Conductivite(e)` pour la plus petite arête dont la
/// conductivité n'est pas finie `>= PLANCHER` (zéro, négatif, NaN, infini, juste sous `PLANCHER` ; `PLANCHER` lui-même
/// est accepté) ; `Taille` avant `Conductivite`. `Memoire::au_hasard` prend les conductivités de `Tubes::au_hasard` et
/// laisse le générateur dans le même état, l'agent est à zéro.
///
/// Ne couvre pas : un graphe sans arête (le cas dégénéré est celui de 2T).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn les_erreurs_de_construction() {
    let g = graphe_pondere(3, &[(0, 1, 1.0), (1, 2, 1.0)]);
    let zero = || Agent::zero(&g);
    assert!(Memoire::new(&g, vec![PLANCHER, 1.0], zero()).is_ok(), "PLANCHER lui-même est accepté");
    assert_eq!(Memoire::new(&g, vec![1.0], zero()), Err(ErreurMemoire::Taille));
    assert_eq!(Memoire::new(&g, vec![1.0, 1.0, 1.0], zero()), Err(ErreurMemoire::Taille));
    let g4 = graphe_pondere(4, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0)]);
    assert_eq!(Memoire::new(&g, vec![1.0, 1.0], Agent::zero(&g4)), Err(ErreurMemoire::Taille));
    for mauvais in [0.0, -1.0, f64::NAN, f64::INFINITY, PLANCHER * 0.999] {
        assert_eq!(Memoire::new(&g, vec![1.0, mauvais], zero()), Err(ErreurMemoire::Conductivite(1)), "{mauvais}");
        assert_eq!(Memoire::new(&g, vec![mauvais, mauvais], zero()), Err(ErreurMemoire::Conductivite(0)), "{mauvais}");
    }
    assert_eq!(Memoire::new(&g, vec![0.0], Agent::zero(&g4)), Err(ErreurMemoire::Taille), "Taille avant Conductivite");
    // au_hasard
    let (g, _, _) = graphe_et_tubes(3);
    let mut a = Alea::new(9);
    let mut b = Alea::new(9);
    let m = Memoire::au_hasard(&g, &mut a);
    let t = Tubes::au_hasard(&g, &mut b);
    assert_eq!(bits(m.conductivites()), bits(t.conductivites()));
    assert_eq!(a.octets(), b.octets(), "le même nombre de tirages");
    assert_eq!(m.agent().concentrations(), &vec![0.0; g.nb_noeuds()][..]);
    assert_eq!(m.agent().masse(), 0.0);
}
