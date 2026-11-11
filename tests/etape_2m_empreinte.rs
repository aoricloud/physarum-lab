//! Étape 2, partie 2M : **l'empreinte se forme** : sur une grille, sous un stimulus d'un côté, les tubes de ce côté
//! grossissent par rapport à un témoin de même graine sans stimulus. Valeurs attendues :
//! `references/etape-2m-valeurs.md`, identifiants `V-EMP-xx`.
//!
//! **Ce modèle est le nôtre** (Kramar et Alim 2021 : un seul tube, aucun réseau ; le couplage sur un graphe n'existe
//! dans aucune source). Aucun chiffre publié n'est visé : les seuils viennent d'une campagne de **50 graines (101 à
//! 150) autres que celles du test** (`outils-2M/mesures/campagne_2m.rs`, mêmes définitions que
//! `tests/common/memoire_2m.rs`), et le test joue les graines 1 à 6. La mesure est l'écart d'empreinte `E = F_A - F_B`
//! (voir la doc de `souvenir`, « Ce qu'on mesure ») ; le réglage est le réglage standard (sigmoïde, `I0 = 3`, `α = 8`,
//! `c_0 = 1`, `κ = 0.1`, `k = 0.2`, `h = 0.1`, stimulus de débit 2 en `(7, 2)` pendant les pas `[0 ; 100[`).

#[path = "common/memoire_2m.rs"]
mod memoire_2m;
#[path = "common/transport_2t.rs"]
mod transport_2t;

use memoire_2m::*;

const GRAINES: [u64; 6] = [1, 2, 3, 4, 5, 6];

fn moyenne(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// L'écart d'empreinte `(E, F_A, F_B)` de chaque graine à l'instant `t`, pour un réglage et des stimuli.
fn ecarts(b: &Banc, r: &Reglage, stimuli: &[Stim], t: usize) -> Vec<(f64, f64, f64)> {
    GRAINES
        .iter()
        .map(|&g| {
            let avec = jouer(b, r, 0, g, stimuli, &[t], None);
            let tem = jouer(b, r, 0, g, &[], &[t], None);
            ecart(b, &avec[0].d, &tem[0].d)
        })
        .collect()
}

/// juste : **l'empreinte se forme du côté du stimulus** (V-EMP-01). Réglage standard, à la fin du stimulus (après 100
/// pas), contre le témoin de même graine : sur 50 graines (101 à 150), `E` vaut `+0.0890` en moyenne (écart-type
/// `0.0177`, de `+0.0585` à `+0.1281` : **positif pour chaque graine**), `F_A` (la moitié du stimulus) de `+0.0585` à
/// `+0.1184`, `F_B` (l'autre moitié) **jamais positif** (de `-0.0254` à `-0.0000` : les tubes de l'autre côté ne
/// grossissent pas, ils perdent un peu) ; avec le stimulus symétrique en `(7, 8)`, `E` est négatif pour chaque graine
/// (de `-0.1246` à `-0.0501`). Sur les graines 1 à 6 le test demande, pour chacune : `E >= 0.04` (marge `1.46` sur le
/// pire des 50), `F_A >= 0.04`, `F_B <= 1e-9`, `E <= -0.03` pour le stimulus symétrique ; et `E` moyen `>= 0.07` (la
/// moyenne des 50 vaut `0.0890` : marge de `0.019`, soit environ trois écarts-types de la moyenne de six graines).
///
/// Ne couvre pas : la tenue après le retrait (`etape_2m_tenue`), un seuil en absolu pour une autre grille.
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn l_empreinte_se_forme_du_cote_du_stimulus() {
    let b = banc();
    let haut = ecarts(&b, &STANDARD, &[STIM_HAUT], 100);
    let bas = ecarts(&b, &STANDARD, &[STIM_BAS], 100);
    assert_eq!(haut.len(), GRAINES.len());
    for (i, (e, fa, fb)) in haut.iter().enumerate() {
        assert!(*e >= 0.04, "graine {} : E = {e}", GRAINES[i]);
        assert!(*fa >= 0.04, "graine {} : F_A = {fa}", GRAINES[i]);
        assert!(*fb <= 1e-9, "graine {} : F_B = {fb}", GRAINES[i]);
    }
    for (i, (e, _, _)) in bas.iter().enumerate() {
        assert!(*e <= -0.03, "graine {}, stimulus en bas : E = {e}", GRAINES[i]);
    }
    let moy = moyenne(&haut.iter().map(|x| x.0).collect::<Vec<_>>());
    assert!(moy >= 0.07, "E moyen = {moy}");
}

/// juste : **sans force, aucune empreinte** (V-EMP-02) : avec `α = 0`, le stimulus ne change rien du tout. Pour chacune
/// des six graines, aux pas 100 et 400, les conductivités avec stimulus et celles du témoin sont **les mêmes bits**
/// (donc `E = 0` exactement), et la masse d'agent, elle, n'est pas nulle à 100 pas (`> 1` : le stimulus a bien eu lieu
/// : un code qui ignorerait les apports passerait le premier contrôle à vide).
///
/// Ne couvre pas : `α` petit mais non nul (V-EMP-03).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn sans_force_pas_d_empreinte() {
    let b = banc();
    let r = Reglage { force: 0.0, ..STANDARD };
    for &g in &GRAINES {
        let avec = jouer(&b, &r, 0, g, &[STIM_HAUT], &[100, 400], None);
        let tem = jouer(&b, &r, 0, g, &[], &[100, 400], None);
        assert_eq!(avec.len(), 2);
        assert!(avec[0].masse > 1.0, "graine {g} : masse {}", avec[0].masse);
        assert_eq!(tem[0].masse, 0.0);
        for i in 0..2 {
            let a: Vec<u64> = avec[i].d.iter().map(|x| x.to_bits()).collect();
            let t: Vec<u64> = tem[i].d.iter().map(|x| x.to_bits()).collect();
            assert_eq!(a, t, "graine {g}, instant {i}");
        }
    }
}

/// juste : **l'empreinte croît avec la force `α`, puis sature** (V-EMP-03). `E` moyen sur les graines 1 à 6, à la fin
/// du stimulus (100 pas) : `0` pour `α = 0`, puis `0.0089`, `0.0464`, `0.0954`, `0.1438` pour `α = 2, 4, 8, 16` (sur 50
/// graines : `0.0131`, `0.0424`, `0.0890`, `0.1430`) : strictement croissant, chaque pas d'au moins `0.005`. **Après le
/// retrait** (400 pas), le plateau sature : `E(α = 16) - E(α = 8)` vaut `0.0036` sur les six graines (`0.0017` sur les
/// 50 : `0.0383` contre `0.0366`), le test demande `|E(16) - E(8)| <= 0.01` : une force plus grande ne grossit plus ce
/// qui reste.
///
/// Ne couvre pas : `c_0` (le plateau en dépend peu : `α = 8` donne `+0.0392` à `c_0 = 0.3` et `+0.0366` à `c_0 = 1`, 50
/// graines ; non testé).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn l_empreinte_croit_avec_la_force_puis_sature() {
    let b = banc();
    let moyenne_e = |alpha: f64, t: usize| -> f64 {
        let r = Reglage { force: alpha, ..STANDARD };
        moyenne(&ecarts(&b, &r, &[STIM_HAUT], t).iter().map(|x| x.0).collect::<Vec<_>>())
    };
    let e: Vec<f64> = [0.0, 2.0, 4.0, 8.0, 16.0].iter().map(|&a| moyenne_e(a, 100)).collect();
    assert_eq!(e[0], 0.0);
    for i in 1..5 {
        assert!(e[i] >= e[i - 1] + 0.005, "E à 100 pas selon α = 0, 2, 4, 8, 16 : {e:?}");
    }
    let (e8, e16) = (moyenne_e(8.0, 400), moyenne_e(16.0, 400));
    assert!((e16 - e8).abs() <= 0.01, "E(400) : α = 8 donne {e8}, α = 16 donne {e16}");
}

/// juste : **Rust et le modèle Python donnent les mêmes `E`, `F_A`, `F_B` et la même sonde** (V-EMP-04) : le modèle
/// `ref_memoire.py` (écrit d'après la doc seule, flottants Python, solveur à profil) rejoue le banc pour quelques cas
/// (`valeurs_2m_b.py`) : réglage standard, graine 1 à 100 et 400 pas (`E = 0.094184679195042795`, `F_A =
/// 0.079605941152454063`, `F_B = -0.014578738042588735` ; `E = 0.058204098767557902`, `F_A = 0.039767900799725669`,
/// `F_B = -0.018436197967832232`), graine 2 (`0.11331323795505752`, `0.094167760988799032`, `-0.019145476966258491` ;
/// `0.056183906568096104`, `0.03774763051858579`, `-0.018436276049510317`), réglage linéaire graine 1 à 100, 300 et 400
/// pas (`E = 0.035731654601014587`, `0.012806581553370913`, `0.0068043285348713383`) ; et la coupe de la sonde à 400
/// pas (`haut - bas` = `0.7216350242974805` pour la graine 1 avec stimulus, `-0.29461831702383662` pour son témoin ;
/// `0.15165803639000336` et `1.5892470327971922e-06` pour la graine 2). Tolérance `1e-13` (mêmes opérations dans le
/// même ordre : les mêmes bits sont attendus). Ce test rend aux seuils des autres tests des valeurs **exactes** : un
/// défaut qui déplacerait `E` de moins que leurs marges serait vu ici.
///
/// Ne couvre pas : les graines autres que 1 et 2 (leurs `E` sont dans la fiche, mesurés par Rust seul).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
#[allow(clippy::excessive_precision)]
fn l_ecart_d_empreinte_est_celui_du_modele_python() {
    let b = banc();
    #[allow(clippy::type_complexity, clippy::excessive_precision)]
    let cas: [(Reglage, u64, usize, f64, f64, f64); 7] = [
        (STANDARD, 1, 100, 0.094184679195042795, 0.079605941152454063, -0.014578738042588735),
        (STANDARD, 1, 400, 0.058204098767557902, 0.039767900799725669, -0.018436197967832232),
        (STANDARD, 2, 100, 0.11331323795505752, 0.094167760988799032, -0.019145476966258491),
        (STANDARD, 2, 400, 0.056183906568096104, 0.03774763051858579, -0.018436276049510317),
        (LINEAIRE, 1, 100, 0.035731654601014587, 0.028680892797340644, -0.0070507618036739408),
        (LINEAIRE, 1, 300, 0.012806581553370913, 0.0095836279213953029, -0.0032229536319756105),
        (LINEAIRE, 1, 400, 0.0068043285348713383, 0.0051699014047492543, -0.0016344271301220845),
    ];
    for (r, graine, t, e, fa, fb) in cas {
        let avec = jouer(&b, &r, 0, graine, &[STIM_HAUT], &[t], None);
        let tem = jouer(&b, &r, 0, graine, &[], &[t], None);
        let (e2, fa2, fb2) = ecart(&b, &avec[0].d, &tem[0].d);
        assert!((e2 - e).abs() <= 1e-13, "graine {graine}, {t} pas : E = {e2} contre {e}");
        assert!((fa2 - fa).abs() <= 1e-13, "graine {graine}, {t} pas : F_A = {fa2} contre {fa}");
        assert!((fb2 - fb).abs() <= 1e-13, "graine {graine}, {t} pas : F_B = {fb2} contre {fb}");
    }
    for (graine, avec_stimulus, attendu) in [
        (1, true, 0.7216350242974805),
        (1, false, -0.29461831702383662),
        (2, true, 0.15165803639000336),
        (2, false, 1.5892470327971922e-06),
    ] {
        let stimuli = if avec_stimulus { vec![STIM_HAUT] } else { vec![] };
        let etat = jouer(&b, &STANDARD, 0, graine, &stimuli, &[400], None);
        let (h, _, bs) = coupe(&b, &etat[0].d, 7, 13);
        assert!(
            (h - bs - attendu).abs() <= 1e-13,
            "sonde graine {graine} (stimulus {avec_stimulus}) : {} contre {attendu}",
            h - bs
        );
    }
}
