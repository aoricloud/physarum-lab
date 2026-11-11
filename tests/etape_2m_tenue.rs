//! Étape 2, partie 2M : **l'empreinte tient-elle après le retrait du stimulus ?** Le test constate ce qui est mesuré ;
//! **le résultat dépend de la loi, et il est négatif sous la loi linéaire**. Valeurs attendues :
//! `references/etape-2m-valeurs.md`, identifiants `V-TEN-xx`.
//!
//! **Notre modèle**, sans chiffre publié (Kramar et Alim 2021 ne donnent aucune durée de persistance chiffrée au-delà
//! de 45 minutes d'expérience, et leur modèle est un seul tube). Les seuils viennent d'une campagne de **50 graines
//! (101 à 150) autres que celles du test** ; le test joue les graines 1 à 6. **Le seuil de l'empreinte, `0.01` sur `E`,
//! a été fixé avant la campagne** (`SEUIL_EMPREINTE`). Les instants sont des nombres de pas depuis le début : le
//! stimulus finit au pas 100, donc « +300 » veut dire le pas 400.

#[path = "common/memoire_2m.rs"]
mod memoire_2m;
#[path = "common/transport_2t.rs"]
mod transport_2t;

use memoire_2m::*;

const GRAINES: [u64; 6] = [1, 2, 3, 4, 5, 6];

fn moyenne(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// `E` de chaque graine aux `instants`, avec l'état avec stimulus (et sa masse d'agent) : `serie[graine][instant]`.
fn serie(
    b: &Banc,
    r: &Reglage,
    famille: u8,
    stimuli: &[Stim],
    instants: &[usize],
    effacer_a: Option<usize>,
) -> Vec<Vec<(f64, f64)>> {
    GRAINES
        .iter()
        .map(|&g| {
            let avec = jouer(b, r, famille, g, stimuli, instants, effacer_a);
            let tem = jouer(b, r, famille, g, &[], instants, None);
            (0..instants.len()).map(|i| (ecart(b, &avec[i].d, &tem[i].d).0, avec[i].masse)).collect()
        })
        .collect()
}

/// juste : **sous la loi sigmoïde, l'empreinte tient** (V-TEN-01). Réglage standard. Sur 50 graines (101 à 150), `E`
/// vaut `+0.0890` à la fin du stimulus (pas 100), `+0.0379` à +100, `+0.0367` à +200, `+0.0366` à +300 (écart-type
/// `0.0193`, pire cas `+0.0079`) puis **la même valeur** à +500 et à +1000 (l'écart entre +300 et +1000 ne dépasse
/// jamais `1.9e-4`) et à +3000 ; la part des graines au-dessus du seuil `0.01` est `0.98` à +100 et `0.96` de +200 à
/// +3000 (deux graines sur 50 restent sous le seuil ; leur `E` reste positif). L'empreinte **tient donc, et n'évolue
/// plus** : le réseau a atteint un état stable où un détour que l'agent a fait passer au-dessus de son seuil de survie
/// (la loi sigmoïde est bistable) continue de porter du flux. Sur les graines 1 à 6 (moyenne `0.0453`, pire `0.0197`),
/// le test demande : `E(400) >= 0.003` pour chaque graine (marge `2.6` sur le pire des 50) ; `E(400)` moyen `>= 0.015`
/// ; au moins 4 graines sur 6 au-dessus du seuil ; `|E(1100) - E(400)| <= 5e-4` (marge `2.6`) ; `E(1100) >= 0.003`.
///
/// Ne couvre pas : l'effet à `α` petit (`α = 1` ne laisse que `+0.0063` en moyenne sur 50 graines, et `α = 2`
/// `+0.0106`), la loi linéaire (V-TEN-03).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn sous_la_loi_sigmoide_l_empreinte_tient() {
    let b = banc();
    let s = serie(&b, &STANDARD, 0, &[STIM_HAUT], &[100, 400, 1100], None);
    assert_eq!(s.len(), GRAINES.len());
    for (i, ligne) in s.iter().enumerate() {
        assert_eq!(ligne.len(), 3);
        assert!(ligne[1].0 >= 0.003, "graine {} : E(400) = {}", GRAINES[i], ligne[1].0);
        assert!(ligne[2].0 >= 0.003, "graine {} : E(1100) = {}", GRAINES[i], ligne[2].0);
        assert!((ligne[2].0 - ligne[1].0).abs() <= 5e-4, "graine {} : le plateau bouge : {ligne:?}", GRAINES[i]);
    }
    let e400: Vec<f64> = s.iter().map(|l| l[1].0).collect();
    assert!(moyenne(&e400) >= 0.015, "E(400) moyen = {}", moyenne(&e400));
    let au_dessus = e400.iter().filter(|&&e| e > SEUIL_EMPREINTE).count();
    assert!(au_dessus >= 4, "{au_dessus} graines sur 6 au-dessus du seuil : {e400:?}");
}

/// juste : **ce n'est pas l'agent qui garde la mémoire** (V-TEN-02) : à +300 (pas 400) il ne reste de l'agent qu'une
/// masse de `0.0054` en moyenne sur 50 graines (au pire `0.0080`), contre `7.18` à la fin du stimulus ; et si on
/// **efface l'agent au pas 100** (la forme seule continue), `E(400)` vaut en moyenne `+0.0364` (au lieu de `+0.0366`),
/// avec `94 %` des graines au-dessus du seuil (au lieu de `96 %`). Un petit nombre de graines change de destin (un
/// écart de `D` jusqu'à `0.69` sur une arête pour l'une des 50) : le test ne demande que la moyenne. Sur les graines 1
/// à 6 : masse `<= 0.02` à 400 pas (marge `2.5`), et `E(400)` moyen avec l'agent effacé `>= 0.015`.
///
/// Ne couvre pas : l'écart exact entre les deux (il varie d'une graine à l'autre).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn l_agent_n_est_pas_la_memoire() {
    let b = banc();
    let normal = serie(&b, &STANDARD, 0, &[STIM_HAUT], &[100, 400], None);
    for (i, l) in normal.iter().enumerate() {
        assert!(l[0].1 > 5.0, "graine {} : masse à 100 pas = {}", GRAINES[i], l[0].1);
        assert!(l[1].1 <= 0.02, "graine {} : masse à 400 pas = {}", GRAINES[i], l[1].1);
    }
    let efface = serie(&b, &STANDARD, 0, &[STIM_HAUT], &[400], Some(100));
    let e: Vec<f64> = efface.iter().map(|l| l[0].0).collect();
    assert!(moyenne(&e) >= 0.015, "E(400) agent effacé : moyenne {}", moyenne(&e));
    assert!(efface.iter().all(|l| l[0].1 == 0.0 || l[0].1 < 0.02), "l'agent effacé ne revient pas");
}

/// juste : **sous la loi linéaire, l'empreinte s'efface** (V-TEN-03) : la loi `f(Q) = |Q|` du cœur n'a qu'un équilibre
/// (le plus court chemin), donc le stimulus ne peut pas laisser de forme durable. Réglage linéaire (`I0 = 1`, `α = 4`,
/// `c_0 = 1`, `κ = 0.1`, `k = 0.2`). `E` est presque indépendant de la graine (le réseau converge vers le même état) :
/// sur 50 graines, `+0.0368` à la fin du stimulus (de `+0.0357` à `+0.0383`), `+0.0233` à +100, `+0.0130` à +200 (de
/// `+0.0120` à `+0.0145`), **`+0.0069` à +300** (de `+0.0062` à `+0.0079`, **sous le seuil `0.01` pour les 50
/// graines**), `+0.0019` à +500, `+0.0001` à +1000. **L'empreinte reste donc au-dessus du seuil entre 200 et 300 pas
/// après le retrait, puis tend vers zéro.** Le test demande, pour les graines 1 à 6 : `E(100) >= 0.033` ; `E(300) >=
/// 0.010` ; `E(400) < 0.01` (le seuil) ; `E(600) <= 0.003` ; `E(1100) <= 0.0003` ; et la suite des six instants (100,
/// 200, 300, 400, 600, 1100) **décroît strictement**.
///
/// Ne couvre pas : une autre `α` sous la loi linéaire (`α = 10` avec `c_0 = 0.3` tient plus longtemps : `+0.0089` à
/// +500 sur 50 graines, mesuré sans test).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn sous_la_loi_lineaire_l_empreinte_s_efface() {
    let b = banc();
    let s = serie(&b, &LINEAIRE, 0, &[STIM_HAUT], &[100, 200, 300, 400, 600, 1100], None);
    assert_eq!(s.len(), GRAINES.len());
    for (i, l) in s.iter().enumerate() {
        assert_eq!(l.len(), 6);
        let e: Vec<f64> = l.iter().map(|x| x.0).collect();
        assert!(e[0] >= 0.033, "graine {} : E(100) = {}", GRAINES[i], e[0]);
        assert!(e[2] >= 0.010, "graine {} : E(300) = {}", GRAINES[i], e[2]);
        assert!(e[3] < SEUIL_EMPREINTE, "graine {} : E(400) = {}", GRAINES[i], e[3]);
        assert!(e[4] <= 0.003, "graine {} : E(600) = {}", GRAINES[i], e[4]);
        assert!(e[5] <= 0.0003, "graine {} : E(1100) = {}", GRAINES[i], e[5]);
        for k in 1..6 {
            assert!(e[k] < e[k - 1], "graine {} : E décroît : {e:?}", GRAINES[i]);
        }
    }
}

/// juste : **si l'agent reste, l'empreinte le suit** (V-TEN-04) : sous la loi linéaire avec `k = 0.01` (l'agent
/// disparaît lentement : sa masse vaut encore `6.8` à +500 et `2.8` à +1000, 50 graines), `E` vaut `+0.0123` à +500 (de
/// `+0.0111` à `+0.0130`, **encore au-dessus du seuil**, contre `+0.0019` avec `k = 0.2`) puis `+0.0013` à +1000 (de
/// `+0.0011` à `+0.0014`) : l'empreinte tombe sous le seuil quand l'agent est devenu trop dilué pour assouplir (`σ(c)`
/// petit). Sur les graines 1 à 6 : `E(600) >= 0.010` (marge `1.11`), `E(1100) <= 0.002`, et la masse d'agent à 600 pas
/// est `>= 3` (l'agent est encore là) alors qu'avec `k = 0.2` `E(600) <= 0.003`.
///
/// Ne couvre pas : la durée exacte du passage sous le seuil (entre 500 et 1000 pas après le retrait).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn si_l_agent_reste_l_empreinte_le_suit() {
    let b = banc();
    let lent = Reglage { perte: 0.01, ..LINEAIRE };
    let s = serie(&b, &lent, 0, &[STIM_HAUT], &[600, 1100], None);
    for (i, l) in s.iter().enumerate() {
        assert!(l[0].0 >= 0.010, "graine {} : E(600) = {}", GRAINES[i], l[0].0);
        assert!(l[1].0 <= 0.002, "graine {} : E(1100) = {}", GRAINES[i], l[1].0);
        assert!(l[0].1 >= 3.0, "graine {} : masse à 600 pas = {}", GRAINES[i], l[0].1);
    }
    let vite = serie(&b, &LINEAIRE, 0, &[STIM_HAUT], &[600], None);
    for (i, l) in vite.iter().enumerate() {
        assert!(l[0].0 <= 0.003, "graine {} : E(600) avec k = 0.2 : {}", GRAINES[i], l[0].0);
    }
}

/// juste : **un stimulus donné sur un réseau déjà convergé ne laisse rien** (V-TEN-05) : le même stimulus, mais pendant
/// les pas `[100 ; 200[` (le réseau est alors déjà tombé sur son état stable : aucun flux ne passe près du stimulus),
/// ne laisse **aucune empreinte** à +300 : `|E| <= 2e-4` pour chaque graine (mesuré sur 50 graines : `4.7e-5` au pire),
/// alors qu'à la fin du stimulus `E` vaut au plus `+0.0361` (`+0.0057` en moyenne, `0.32` des graines au-dessus du
/// seuil) : une trace passagère. Le modèle ne marque que ce qui est encore en mouvement.
///
/// Ne couvre pas : un stimulus sur une source de nourriture qui bougerait (il n'y en a pas ici).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn un_stimulus_sur_un_reseau_converge_ne_laisse_rien() {
    let b = banc();
    let tard = Stim { debut: 100, fin: 200, ..STIM_HAUT };
    let s = serie(&b, &STANDARD, 0, &[tard], &[200, 500], None);
    for (i, l) in s.iter().enumerate() {
        assert!(l[0].0 <= 0.04, "graine {} : E à la fin = {}", GRAINES[i], l[0].0);
        assert!(l[0].1 > 5.0, "graine {} : le stimulus a eu lieu (masse {})", GRAINES[i], l[0].1);
        assert!(l[1].0.abs() <= 2e-4, "graine {} : E(+300) = {}", GRAINES[i], l[1].0);
    }
}

/// juste : **la famille des nourritures donne la même chose** (V-TEN-06) : deux nourritures `(1, 5)` et `(13, 5)`, la
/// paire (source, puits) tirée à chaque pas (`reseaux::tirer_paire`), mêmes réglages, le témoin jouant les mêmes
/// paires. Sigmoïde : `E(400)` vaut `+0.0372` en moyenne sur 50 graines (de `+0.0077` à `+0.0740`, `0.96` au-dessus du
/// seuil), identique à 1100 pas ; linéaire : `+0.0369` à la fin du stimulus, `+0.0072` à +300 (de `+0.0065` à
/// `+0.0082`, sous le seuil), `+0.0001` à +1000. Sur les graines 1 à 6 : sigmoïde, `E(400) >= 0.003` pour chaque graine
/// et `E(400)` moyen `>= 0.015` ; linéaire, `E(100) >= 0.033`, `E(400) < 0.01` et `E(1100) <= 0.0003`.
///
/// Ne couvre pas : plus de deux nourritures (la famille complète est celle de `lancer`, `etape_2m_lancer`).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn la_famille_des_nourritures_donne_la_meme_chose() {
    let b = banc();
    let sig = serie(&b, &STANDARD, 1, &[STIM_HAUT], &[400, 1100], None);
    for (i, l) in sig.iter().enumerate() {
        assert!(l[0].0 >= 0.003, "sigmoïde graine {} : E(400) = {}", GRAINES[i], l[0].0);
        assert!((l[1].0 - l[0].0).abs() <= 5e-4, "sigmoïde graine {} : le plateau bouge", GRAINES[i]);
    }
    assert!(moyenne(&sig.iter().map(|l| l[0].0).collect::<Vec<_>>()) >= 0.015);
    let lin = serie(&b, &LINEAIRE, 1, &[STIM_HAUT], &[100, 400, 1100], None);
    for (i, l) in lin.iter().enumerate() {
        assert!(l[0].0 >= 0.033, "linéaire graine {} : E(100) = {}", GRAINES[i], l[0].0);
        assert!(l[1].0 < SEUIL_EMPREINTE, "linéaire graine {} : E(400) = {}", GRAINES[i], l[1].0);
        assert!(l[2].0 <= 0.0003, "linéaire graine {} : E(1100) = {}", GRAINES[i], l[2].0);
    }
}
