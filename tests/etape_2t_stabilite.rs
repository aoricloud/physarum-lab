//! Étape 2, partie 2T : la stabilité du schéma et le pas borné (`pas_maximal`, `nombre_de_sous_pas`,
//! `pas_sans_decoupage`, `Agent::avancer`). Valeurs attendues : `references/etape-2t-valeurs.md`, identifiants
//! `V-STA-xx`.
//!
//! La source est la doc de `souvenir` (« Le taux de départ et le pas maximal », « Le pas public et son découpage », «
//! Ce que l'arrondi coûte »). Le schéma garde les concentrations `>= 0` si `h * r_i <= 1` pour chaque nœud ; le pas
//! maximal est le plus grand flottant `h` tel que `h * r_max <= 1.0` en flottants. Les cas limites sont construits pour
//! que le nœud qui fixe le maximum n'ait aucun apport d'agent (alors un cran au-dessus du pas maximal, sa concentration
//! devient négative). Les valeurs viennent de `ref_souvenir.py` et se vérifient à la main (écrites en commentaire).

#[path = "common/graphes_au_hasard.rs"]
mod graphes_au_hasard;
#[path = "common/transport_2t.rs"]
mod transport_2t;

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Graphe, Noeud};
use physarum_lab::souvenir::{
    Agent, ErreurAgent, MAX_SOUS_PAS, Transport, nombre_de_sous_pas, pas_maximal, pas_sans_decoupage, taux_de_depart,
};
use transport_2t::{
    bits, chaine, graphe_l, naif_avancer, naif_pas_maximal, naif_sous_pas, naif_taux, precedent, suivant,
};

/// Vrai si `x` est un zéro positif ou un nombre strictement positif : ni négatif, ni `-0.0`, ni NaN.
fn non_negatif(x: f64) -> bool {
    x >= 0.0 && x.to_bits() >> 63 == 0
}

/// juste : `nombre_de_sous_pas` (V-STA-01) est **le plus petit `n >= 1` tel que `h / (n as f64) <= h_max`** en
/// flottants : une table de quinze cas calculée par essai de chaque `n` (modèle Python, lent et indépendant) ;
/// `h <= h_max` donne 1 (l'égalité comprise, et `h_max` infini) ; un `h` juste au-dessus d'un multiple
/// (`3.0000000000000004` pour `h_max = 1`) demande un sous-pas de plus ; `0.7 * 3.0` (`2.0999999999999996`) pour `0.7`
/// donne 3, `7` pour `0.7` donne 10 ; `16.8` pour `2.4` donne 7 (le quotient arrondi `7.000000000000001` aurait
/// donné 8 par un plafond : le plus petit `n` est 7) et `21.78` pour `1.98` donne 12 (le quotient arrondi donne 11).
/// Au-delà de [`MAX_SOUS_PAS`] : un nombre plus grand que `MAX_SOUS_PAS`, sans calculer de boucle de millions de tours
/// (`1e300 / 1e-300`). `1e6` pour `1.0` donne exactement `1_000_000`.
///
/// Ne couvre pas : les paniques (`les_paniques_de_nombre_de_sous_pas`), l'emploi dans le pas public.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn le_nombre_de_sous_pas() {
    let table: [(f64, f64, usize); 15] = [
        (1.0, 1.0, 1),
        (1.0, f64::INFINITY, 1),
        (3.0, 1.0, 3),
        (3.000_000_000_000_000_4, 1.0, 4),
        (0.5, 1.0, 1),
        (10.0, 3.0, 4),
        (20.0, 2.0, 10),
        (1.000_000_000_000_000_2, 1.0, 2),
        (7.0, 0.7, 10),
        (0.7 * 3.0, 0.7, 3),
        (1e-3, 1e-3, 1),
        (2.5, 0.5, 5),
        (16.8, 2.4, 7),
        (4.2, 0.6, 7),
        (21.78, 1.98, 12),
    ];
    assert_eq!(0.7 * 3.0, 2.0999999999999996, "la valeur de la table est bien celle du produit");
    for (h, h_max, n) in table {
        assert_eq!(nombre_de_sous_pas(h, h_max), n, "h = {h}, h_max = {h_max}");
        assert_eq!(naif_sous_pas(h, h_max), n, "le modèle naïf du test est d'accord : h = {h}, h_max = {h_max}");
    }
    assert_eq!(nombre_de_sous_pas(1e6, 1.0), 1_000_000, "exactement MAX_SOUS_PAS : accepté");
    assert!(nombre_de_sous_pas(2e6, 1.0) > MAX_SOUS_PAS);
    assert!(nombre_de_sous_pas(1e300, 1e-300) > MAX_SOUS_PAS);
    assert_eq!(MAX_SOUS_PAS, 1_000_000);
}

/// juste : les paniques de `nombre_de_sous_pas` (V-STA-01) : `h` nul, négatif, NaN ou infini ; `h_max` nul, négatif ou
/// NaN (l'infini est permis). Un cas positif d'abord, puis `catch_unwind` sur le seul appel fautif.
///
/// Ne couvre pas : la table des valeurs (`le_nombre_de_sous_pas`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn les_paniques_de_nombre_de_sous_pas() {
    assert_eq!(nombre_de_sous_pas(2.0, 1.0), 2);
    let paniques = |h: f64, h_max: f64| std::panic::catch_unwind(move || nombre_de_sous_pas(h, h_max)).is_err();
    for h in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(paniques(h, 1.0), "h = {h}");
    }
    for h_max in [0.0, -1.0, f64::NAN] {
        assert!(paniques(1.0, h_max), "h_max = {h_max}");
    }
}

/// juste : le pas maximal est **le plus grand flottant** `h` tel que `h * r_max <= 1.0` (V-STA-02). Sur quatre cas
/// construits, dont les valeurs viennent de `ref_souvenir.py` : la chaîne de flux `0.5` (`r_max = 0.5`, `h_max = 2`) ;
/// l'étoile à trois feuilles de diffusion `0.25` (`r_max = 0.75`, `h_max = 1.3333333333333335`, bits
/// `0x3FF5555555555556`) ; un nœud seul de perte `0.3` (`3.3333333333333335`, `0x400AAAAAAAAAAAAB`) ; une chaîne
/// `0-1-2` de flux `[0.25, 1]` où le puits et un nœud intérieur fixent ensemble le maximum `1`. À chaque fois `h *
/// r_max <= 1.0` et le flottant suivant donne `> 1.0`, calculés avec le `r_max` du taux de départ du modèle du test. Un
/// nœud seul de perte `0.09` : `1 / 0.09` vaut `0x402638E38E38E38E`, mais le plus grand `h` qui garde `h * 0.09 <= 1`
/// est un cran au-dessus (`0x402638E38E38E38F`) : un code qui rend `1.0 / r_max` sans monter échoue. Sans rien qui
/// parte (flux nuls, `κ = 0`, `k = 0`) : `f64::INFINITY`.
///
/// Ne couvre pas : des graphes au hasard (`le_pas_maximal_sur_des_graphes_au_hasard`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn le_pas_maximal_sur_des_cas_construits() {
    let un_noeud = Graphe::new(vec![Noeud { x: 0, y: 0, etage: 0 }], vec![]).unwrap();
    let etoile = graphe_l(4, &[(0, 1, 1.0), (0, 2, 1.0), (0, 3, 1.0)]);
    let cas: Vec<(Graphe, Vec<f64>, f64, f64, u64)> = vec![
        (chaine(&[1.0, 1.0, 1.0]), vec![0.5, 0.5, 0.5], 0.0, 0.0, 0x4000_0000_0000_0000),
        (etoile, vec![0.0, 0.0, 0.0], 0.25, 0.0, 0x3FF5_5555_5555_5556),
        (un_noeud.clone(), vec![], 0.0, 0.3, 0x400A_AAAA_AAAA_AAAB),
        (chaine(&[1.0, 1.0]), vec![0.25, 1.0], 0.0, 0.0, 0x3FF0_0000_0000_0000),
        (un_noeud.clone(), vec![], 0.0, 0.09, 0x4026_38E3_8E38_E38F),
    ];
    for (g, q, kappa, k, attendu) in cas {
        let tr = Transport { diffusion: kappa, perte: k };
        let h = pas_maximal(&g, &q, &tr).unwrap();
        assert_eq!(h.to_bits(), attendu, "le pas maximal (κ = {kappa}, k = {k})");
        let (r, _) = naif_taux(&g, &q, kappa, k);
        let r_max = r.iter().fold(0.0, |m: f64, &x| if x > m { x } else { m });
        assert!(h * r_max <= 1.0, "h * r_max = {}", h * r_max);
        assert!(suivant(h) * r_max > 1.0, "le suivant dépasse : {}", suivant(h) * r_max);
    }
    assert_eq!((1.0_f64 / 0.09).to_bits(), 0x4026_38E3_8E38_E38E, "1/0.09 est un cran trop bas");
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    assert_eq!(pas_maximal(&un_noeud, &[], &tr).unwrap(), f64::INFINITY);
}

/// juste : le pas maximal contre le pas maximal naïf du test (V-STA-02) sur 60 graphes au hasard (de 5 à 24 nœuds,
/// quelques arêtes en plus de l'arbre), des flux de signes et de grandeurs quelconques, `κ` et `k` au hasard (dont
/// `0.0`) : mêmes bits, et `h * r_max <= 1 < suivant(h) * r_max` aux nombres du test. `taux_de_depart` rend les mêmes
/// bits que les taux naïfs (boucle par arête).
///
/// Ne couvre pas : la stabilité elle-même (`a_la_limite_aucune_concentration_n_est_negative`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn le_pas_maximal_sur_des_graphes_au_hasard() {
    let mut alea = Alea::new(2025);
    let mut cas = 0;
    for _ in 0..60 {
        let n = 5 + alea.borne(20) as usize;
        let g = graphes_au_hasard::graphe_au_hasard(&mut alea, n, n, 0.5, 2.0);
        let q: Vec<f64> = (0..g.nb_aretes()).map(|_| 2.0 * alea.suivant_f64() - 1.0).collect();
        let kappa = if alea.borne(4) == 0 { 0.0 } else { alea.suivant_f64() };
        let k = if alea.borne(4) == 0 { 0.0 } else { 0.1 * alea.suivant_f64() };
        let tr = Transport { diffusion: kappa, perte: k };
        let (r, _) = naif_taux(&g, &q, kappa, k);
        assert_eq!(bits(&taux_de_depart(&g, &q, &tr).unwrap()), bits(&r), "taux de départ, cas {cas}");
        let h = pas_maximal(&g, &q, &tr).unwrap();
        assert_eq!(h.to_bits(), naif_pas_maximal(&g, &q, kappa, k).to_bits(), "cas {cas}");
        let r_max = r.iter().fold(0.0, |m: f64, &x| if x > m { x } else { m });
        assert!(h * r_max <= 1.0 && suivant(h) * r_max > 1.0, "cas {cas}");
        cas += 1;
    }
    assert_eq!(cas, 60);
}

/// juste : **à la limite exacte, aucune concentration n'est négative** (V-STA-03). Trois cas construits pour atteindre
/// la limite, joués au pas maximal puis **un cran au-dessus** (`suivant`) par `pas_sans_decoupage` :
/// - la chaîne de flux `0.5`, `c = [1, 1, 1, 1]` : au pas maximal `2` : `[0, 1, 1, 1]` (le nœud 0 donne tout ce qu'il
///   a, `1 - 2 * 0.5 = 0` : un `+0.0` exact, pas de `-0.0`), le bilan a une sortie `1` ; un cran au-dessus
///   (`2.0000000000000004`) : `c0 = -2.220446049250313e-16 < 0` (`1 - 1.0000000000000002`, aucun apport au nœud 0) ;
/// - l'étoile à trois feuilles de diffusion `0.25`, `c = [1, 0, 0, 0]` : le centre fixe le maximum : au pas maximal `c0
///   = 0`, un cran au-dessus `c0 = -2.220446049250313e-16` ;
/// - le nœud seul de perte `0.3`, `c = 1` : au pas maximal `0`, un cran au-dessus `-2.220446049250313e-16` ;
/// - la chaîne `0-1-2` de flux `[0.25, 1]`, `c = [0, 0, 2]`, où **le puits seul fixe le maximum** (`sortie = 1`) : au
///   pas maximal `c2 = 0`, un cran au-dessus `c2 = -4.440892098500626e-16`. Et, un cran seulement : le dépassement est
///   **net** à `2 *` le pas maximal sur la chaîne (`c0 = 1 - 2 * 2 * 0.5 * 1 = -1`).
///
/// Ne couvre pas : des cas au hasard (`a_la_limite_sur_des_graphes_au_hasard`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn a_la_limite_aucune_concentration_n_est_negative() {
    let un_noeud = Graphe::new(vec![Noeud { x: 0, y: 0, etage: 0 }], vec![]).unwrap();
    let etoile = graphe_l(4, &[(0, 1, 1.0), (0, 2, 1.0), (0, 3, 1.0)]);
    // (graphe, flux, κ, k, départ, rang du nœud testé, concentration au pas maximal (bits), un cran au-dessus (bits))
    #[allow(clippy::type_complexity)]
    let cas: Vec<(Graphe, Vec<f64>, f64, f64, Vec<f64>, usize, u64, u64)> = vec![
        (chaine(&[1.0, 1.0, 1.0]), vec![0.5; 3], 0.0, 0.0, vec![1.0; 4], 0, 0, 0xBCB0_0000_0000_0000),
        (etoile, vec![0.0; 3], 0.25, 0.0, vec![1.0, 0.0, 0.0, 0.0], 0, 0, 0xBCB0_0000_0000_0000),
        (un_noeud, vec![], 0.0, 0.3, vec![1.0], 0, 0, 0xBCB0_0000_0000_0000),
        (chaine(&[1.0, 1.0]), vec![0.25, 1.0], 0.0, 0.0, vec![0.0, 0.0, 2.0], 2, 0, 0xBCC0_0000_0000_0000),
    ];
    for (g, q, kappa, k, depart, rang, au_max, au_dessus) in cas {
        let tr = Transport { diffusion: kappa, perte: k };
        let h = pas_maximal(&g, &q, &tr).unwrap();
        let (c, _) = pas_sans_decoupage(&g, &q, &depart, &tr, &[], h).unwrap();
        assert!(c.iter().all(|&x| non_negatif(x)), "au pas maximal : {c:?}");
        assert_eq!(c[rang].to_bits(), au_max, "au pas maximal, le nœud {rang} : un zéro positif exact");
        let (c, _) = pas_sans_decoupage(&g, &q, &depart, &tr, &[], suivant(h)).unwrap();
        assert_eq!(c[rang].to_bits(), au_dessus, "un cran au-dessus : {c:?}");
        assert!(c[rang] < 0.0);
    }
    // Le bilan de la chaîne, au pas maximal : le puits (nœud 3) emporte `(2 * 0.5) * 1 = 1`.
    let g = chaine(&[1.0, 1.0, 1.0]);
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    let (c, b) = pas_sans_decoupage(&g, &[0.5; 3], &[1.0; 4], &tr, &[], 2.0).unwrap();
    assert_eq!(bits(&c), bits(&[0.0, 1.0, 1.0, 1.0]));
    assert_eq!(b.sortie, 1.0);
    // Net au-dessus de la limite : deux fois le pas maximal.
    let (c, _) = pas_sans_decoupage(&g, &[0.5; 3], &[1.0; 4], &tr, &[], 4.0).unwrap();
    assert_eq!(c[0], -1.0, "c0 = (1 - 4 * 0.5) * 1");
    assert_eq!(precedent(suivant(2.0)), 2.0);
}

/// juste : **à la limite, sur des graphes au hasard** (V-STA-03) : sur 80 cas (graphes au hasard de 5 à 24 nœuds, flux
/// d'un à trente pas des tubes, la source au nœud 0 et le puits au dernier, `κ` nul un cas sur deux, `k` au hasard,
/// concentrations au hasard dans `[0, 3[`, un apport sur un nœud), `pas_sans_decoupage` au pas maximal ne rend **aucune
/// concentration négative, ni `-0.0`** (le regroupement `1.0 - h * r_i` rend `>= 0` toute la partie « ce qui reste »).
/// Mesuré (`mes_2t_a.rs`, 1800 cas de grilles de 36 à 120 nœuds aux flux réels de tubes, 132 000 concentrations) : **0
/// négatif, 0 `-0.0`** ; la recette « naïve » `c + (gains - pertes)` essayée à côté en rend 204 (le pire `-8.88e-16`).
/// Le test refait la comparaison : une recette à variation signée, écrite ici, rend au moins un négatif sur ces cas
/// (sans quoi le test ne mesurerait rien).
///
/// Ne couvre pas : l'au-dessus de la limite (`a_la_limite_aucune_concentration_n_est_negative`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn a_la_limite_sur_des_graphes_au_hasard() {
    let mut alea = Alea::new(77);
    let mut negatifs_signes = 0;
    let mut total = 0;
    for cas in 0..80 {
        let n = 5 + alea.borne(20) as usize;
        let g = graphes_au_hasard::graphe_au_hasard(&mut alea, n, n, 0.5, 2.0);
        let mut tubes = physarum_lab::tubes::Tubes::au_hasard(&g, &mut alea);
        let mut q = vec![];
        for _ in 0..(1 + alea.borne(30)) {
            q = tubes.avancer(&g, 0, n - 1, 1.0, 0.1).unwrap().flux;
        }
        let kappa = if cas % 2 == 0 { 0.0 } else { alea.suivant_f64() };
        let k = 0.1 * alea.suivant_f64();
        let tr = Transport { diffusion: kappa, perte: k };
        let c: Vec<f64> = (0..n).map(|_| 3.0 * alea.suivant_f64()).collect();
        let h = pas_maximal(&g, &q, &tr).unwrap();
        let apports = [(alea.borne(n as u32) as usize, alea.suivant_f64())];
        let (suite, _) = pas_sans_decoupage(&g, &q, &c, &tr, &apports, h).unwrap();
        assert!(suite.iter().all(|&x| non_negatif(x)), "au pas maximal : {suite:?}");
        total += suite.len();
        // La recette à variation signée : c + (somme signée des échanges), nœud par nœud.
        let (r, sortie) = naif_taux(&g, &q, kappa, k);
        let mut v = vec![0.0; n];
        for (e, a) in g.aretes().iter().enumerate() {
            for (i, j, qq) in [(a.a, a.b, q[e]), (a.b, a.a, -q[e])] {
                if qq > 0.0 {
                    v[i] -= (h * qq) * c[i];
                } else if qq < 0.0 {
                    v[i] += (h * (-qq)) * c[j];
                }
                v[i] += ((h * kappa) * (c[j] - c[i])) / a.longueur;
            }
        }
        for i in 0..n {
            v[i] -= (h * sortie[i]) * c[i];
            v[i] -= (h * k) * c[i];
            let _ = r[i];
            if c[i] + v[i] < 0.0 {
                negatifs_signes += 1;
            }
        }
    }
    assert!(total > 500, "assez de concentrations : {total}");
    // Mesuré sur ces 80 cas : 13 négatifs de la recette signée sur 1139 concentrations ; on exige seulement qu'elle en
    // rende au moins un (sinon la comparaison ne prouverait rien).
    assert!(negatifs_signes >= 1, "la recette à variation signée aurait dû rendre un négatif ({negatifs_signes})");
}

/// juste : **le pas public avec un `h` dix fois trop grand** (V-STA-04) reste `>= 0` et rend **les mêmes bits que ses
/// sous-pas joués à la main**. Deux cas dyadiques/non dyadiques de `ref_souvenir.py` : la chaîne de flux `0.5`, `h =
/// 20` (dix fois le pas maximal `2`) : `n = 10`, sous-pas de `2`, de `c = [1, 0, 0, 0]` à `[0, 0, 0, 0]`, bilan `[1, 0,
/// 0]`, `sous_pas == 10` (l'agent a tout traversé et est sorti) ; la même chaîne avec `κ = 0.25`, `k = 0.01`, un apport
/// de `0.5` au nœud 0, `h = 7` : pas maximal `0x3FEFAEE41E6A7498`, `n = 8`, concentrations `[0.9304369198563496,
/// 0.8647433733352173, 0.7841190379171352, 0.7080841881081301]` et bilan aux bits de `ref_souvenir.py`. Puis 30 graphes
/// au hasard à flux de tubes réels : `h = 10 * h_max` : toutes les concentrations `>= 0`, mêmes bits que `n` appels à
/// `pas_sans_decoupage` de durée `h / n` (`n` du modèle naïf), mêmes bits que `naif_avancer`, bilan somme des bilans,
/// `sous_pas == n`.
///
/// Ne couvre pas : `n > MAX_SOUS_PAS` (`trop_de_sous_pas`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn le_pas_public_decoupe_et_reste_positif() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    let q = [0.5, 0.5, 0.5];
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    let mut agent = Agent::new(&g, vec![1.0, 0.0, 0.0, 0.0]).unwrap();
    let b = agent.avancer(&g, &q, &tr, &[], 20.0).unwrap();
    assert_eq!(bits(agent.concentrations()), vec![0_u64; 4]);
    assert_eq!((b.sortie, b.perte, b.apport, b.sous_pas), (1.0, 0.0, 0.0, 10));
    // Non dyadique, avec diffusion, perte et apport.
    let tr = Transport { diffusion: 0.25, perte: 0.01 };
    assert_eq!(pas_maximal(&g, &q, &tr).unwrap().to_bits(), 0x3FEF_AEE4_1E6A_7498);
    let mut agent = Agent::new(&g, vec![1.0, 0.0, 0.0, 0.0]).unwrap();
    let b = agent.avancer(&g, &q, &tr, &[(0, 0.5)], 7.0).unwrap();
    let attendu = [0x3FED_C623_A5B8_C490_u64, 0x3FEB_ABFA_4B7D_0A0D, 0x3FE9_1780_CF00_CD74, 0x3FE6_A8A0_2BD7_A6F6];
    assert_eq!(bits(agent.concentrations()), attendu.to_vec());
    assert_eq!(
        bits(&[b.sortie, b.perte, b.apport]),
        vec![0x3FF0_DD82_DADE_12C0_u64, 0x3FC4_4AED_78D6_5DC0, 0x400C_0000_0000_0000]
    );
    assert_eq!(b.sous_pas, 8);
    // Des graphes au hasard, aux flux de vrais pas des tubes.
    let mut alea = Alea::new(31);
    for cas in 0..30 {
        let n = 6 + alea.borne(14) as usize;
        let g = graphes_au_hasard::graphe_au_hasard(&mut alea, n, n, 0.5, 2.0);
        let mut tubes = physarum_lab::tubes::Tubes::au_hasard(&g, &mut alea);
        let mut q = vec![];
        for _ in 0..(1 + alea.borne(30)) {
            q = tubes.avancer(&g, 0, n - 1, 1.0, 0.1).unwrap().flux;
        }
        let (kappa, k) = (0.5 * alea.suivant_f64(), 0.1 * alea.suivant_f64());
        let tr = Transport { diffusion: kappa, perte: k };
        let c: Vec<f64> = (0..n).map(|_| 2.0 * alea.suivant_f64()).collect();
        let h_max = pas_maximal(&g, &q, &tr).unwrap();
        let h = 10.0 * h_max;
        let n_sous = naif_sous_pas(h, h_max);
        assert!(n_sous >= 10, "cas {cas} : le découpage est bien en au moins dix sous-pas ({n_sous})");
        let mut agent = Agent::new(&g, c.clone()).unwrap();
        let b = agent.avancer(&g, &q, &tr, &[], h).unwrap();
        assert!(agent.concentrations().iter().all(|&x| non_negatif(x)), "cas {cas} : une concentration négative");
        assert_eq!(b.sous_pas, n_sous, "cas {cas}");
        // Les sous-pas joués à la main.
        let h_sous = h / (n_sous as f64);
        let mut main = c.clone();
        let mut total = [0.0; 3];
        for _ in 0..n_sous {
            let (suite, bil) = pas_sans_decoupage(&g, &q, &main, &tr, &[], h_sous).unwrap();
            main = suite;
            total[0] += bil.sortie;
            total[1] += bil.perte;
            total[2] += bil.apport;
        }
        assert_eq!(bits(agent.concentrations()), bits(&main), "cas {cas} : les sous-pas à la main");
        assert_eq!(
            bits(&[b.sortie, b.perte, b.apport]),
            bits(&total),
            "cas {cas} : le bilan est la somme des sous-pas"
        );
        let (naif, tot_naif, n_naif) = naif_avancer(&g, &q, &c, kappa, k, &[], h);
        assert_eq!(n_naif, n_sous);
        assert_eq!(bits(agent.concentrations()), bits(&naif), "cas {cas} : le pas naïf");
        assert_eq!(bits(&[b.sortie, b.perte, b.apport]), bits(&tot_naif));
    }
}

/// juste : un `h` sous le pas maximal est **un seul sous-pas, exactement `h`** (V-STA-04) : `Agent::avancer` et
/// `pas_sans_decoupage` rendent les mêmes bits, `sous_pas == 1` ; au pas maximal lui-même aussi (l'égalité compte : `h
/// <= h_max`) ; le flottant suivant demande 2 sous-pas.
///
/// Ne couvre pas : des découpages en `n > 2` (`le_pas_public_decoupe_et_reste_positif`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn sous_la_limite_un_seul_sous_pas() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    let q = [0.5, 0.5, 0.5];
    let tr = Transport { diffusion: 0.1, perte: 0.0 };
    let h_max = pas_maximal(&g, &q, &tr).unwrap();
    let depart = vec![1.0, 0.3, 0.2, 0.6];
    for (h, n) in [(0.1, 1), (1.0, 1), (h_max, 1), (suivant(h_max), 2)] {
        let mut agent = Agent::new(&g, depart.clone()).unwrap();
        let b = agent.avancer(&g, &q, &tr, &[], h).unwrap();
        assert_eq!(b.sous_pas, n, "h = {h}");
        if n == 1 {
            let (suite, bil) = pas_sans_decoupage(&g, &q, &depart, &tr, &[], h).unwrap();
            assert_eq!(bits(agent.concentrations()), bits(&suite));
            assert_eq!(bil, b);
        }
    }
}

/// juste : plus de `MAX_SOUS_PAS` sous-pas est l'erreur `SousPas`, et rien n'a changé (V-STA-05) : un nœud seul de
/// perte `1` (`h_max = 1`) et un `h` de `2e6` demandent deux millions de sous-pas ; exactement un million est accepté
/// (un nœud seul : un million de pas de la recette tiennent dans quelques dizaines de millisecondes) et rend `c = 0`
/// (le premier sous-pas, de durée 1, vide le nœud), `sous_pas == 1_000_000`, une perte totale égale à `c` de départ.
///
/// Ne couvre pas : les autres erreurs (`etape_2t_erreurs.rs`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn trop_de_sous_pas() {
    let seul = Graphe::new(vec![Noeud { x: 0, y: 0, etage: 0 }], vec![]).unwrap();
    let tr = Transport { diffusion: 0.0, perte: 1.0 };
    let mut agent = Agent::new(&seul, vec![0.75]).unwrap();
    let avant = agent.clone();
    assert_eq!(agent.avancer(&seul, &[], &tr, &[], 2e6), Err(ErreurAgent::SousPas));
    assert_eq!(agent, avant, "rien n'a changé");
    let b = agent.avancer(&seul, &[], &tr, &[], 1e6).unwrap();
    assert_eq!(b.sous_pas, 1_000_000);
    assert_eq!(agent.concentrations()[0].to_bits(), 0, "vidé dès le premier sous-pas");
    assert_eq!(b.perte, 0.75);
}
