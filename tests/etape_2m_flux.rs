//! Étape 2, partie 2M : **où part le flux d'une nouvelle source** (ce qui remplace « la migration » du plan : notre
//! modèle n'a pas de front qui avance). Valeurs attendues : `references/etape-2m-valeurs.md`, identifiants `V-FLX-xx`.
//!
//! **Notre modèle**, sans chiffre publié. La sonde est définie dans `tests/common/memoire_2m.rs` et dans la doc de
//! `souvenir` (« Ce qu'on mesure ») : un flux de 1 injecté en `(7, 5)` (sur l'axe), le puits en `(13, 5)`, les
//! conductivités gelées après 400 pas, et la part du flux qui traverse la coupe entre les colonnes 10 et 11 **en haut**
//! (`y < 5`, le côté de l'ancien stimulus), **sur l'axe**, **en bas**. Les seuils viennent d'une campagne de 50 graines
//! (101 à 150) autres que celles du test ; le test joue les graines 1 à 6.

#[path = "common/memoire_2m.rs"]
mod memoire_2m;
#[path = "common/transport_2t.rs"]
mod transport_2t;

use memoire_2m::*;

const GRAINES: [u64; 6] = [1, 2, 3, 4, 5, 6];

/// La mesure `haut - bas` de la coupe d'une sonde `(7, 5) -> (13, 5)` pour chaque graine : avec le stimulus en haut,
/// avec le stimulus symétrique en bas, et pour le témoin ; état après 400 pas.
fn mesures() -> Vec<(f64, f64, f64)> {
    let b = banc();
    GRAINES
        .iter()
        .map(|&g| {
            let haut = jouer(&b, &STANDARD, 0, g, &[STIM_HAUT], &[400], None);
            let bas = jouer(&b, &STANDARD, 0, g, &[STIM_BAS], &[400], None);
            let tem = jouer(&b, &STANDARD, 0, g, &[], &[400], None);
            let f = |d: &[f64]| {
                let (h, _, bs) = coupe(&b, d, 7, 13);
                h - bs
            };
            (f(&haut[0].d), f(&bas[0].d), f(&tem[0].d))
        })
        .collect()
}

/// juste : **le flux d'une nouvelle source part du côté de l'ancien stimulus** (V-FLX-01). Après l'empreinte (le
/// stimulus a fini au pas 100, la mesure est au pas 400, l'agent a disparu), `haut - bas` de la sonde vaut, sur 50
/// graines : `+0.724` en moyenne avec le stimulus en haut (**positif pour chaque graine**, de `+0.161` à `+1.000`),
/// `-0.665` avec le stimulus symétrique en bas (négatif pour 49 graines sur 50 : une graine donne `+0.089`), et
/// `+0.011` pour le témoin (de `-0.326` à `+0.326` : le témoin n'a pas de côté, ses tubes de départ au hasard en
/// donnent un). Contre le stimulus symétrique de même graine, **l'écart `haut - bas(stimulus bas)` vaut `+1.389` en
/// moyenne, de `+0.656` à `+1.709`** ; contre le témoin, `+0.713` (de `+0.121` à `+1.228`). Sur les graines 1 à 6, le
/// test demande, pour chacune : `haut - bas >= 0.05` avec le stimulus en haut (marge `3.2` sur le pire des 50) ; sa
/// différence avec la mesure du stimulus en bas `>= 0.1` (la graine 2 donne `+0.333`, sous le pire des 50 : le pire des
/// 50 n'est pas une borne pour d'autres graines) ; sa différence avec le témoin `>= 0.05` ; la moyenne de cette
/// différence `>= 0.6` ; et la moyenne de la mesure du stimulus en bas `<= -0.3`. Le chiffre est **fort mais pas
/// universel** : la mesure d'une graine isolée peut être faible (`+0.15` pour la graine 2).
///
/// Ne couvre pas : une sonde posée ailleurs (une source à `(5, 5)` donne autre chose), ni la direction d'un front.
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn le_flux_d_une_nouvelle_source_part_du_cote_de_l_ancien_stimulus() {
    let m = mesures();
    assert_eq!(m.len(), GRAINES.len());
    for (i, (h, bs, t)) in m.iter().enumerate() {
        let g = GRAINES[i];
        assert!(*h >= 0.05, "graine {g} : haut - bas (stimulus en haut) = {h}");
        assert!(h - bs >= 0.1, "graine {g} : {h} contre le stimulus en bas {bs}");
        assert!(h - t >= 0.05, "graine {g} : {h} contre le témoin {t}");
    }
    let moy_bas = m.iter().map(|x| x.1).sum::<f64>() / m.len() as f64;
    assert!(moy_bas <= -0.3, "stimulus en bas : moyenne {moy_bas}");
    let moy_diff = m.iter().map(|x| x.0 - x.1).sum::<f64>() / m.len() as f64;
    assert!(moy_diff >= 0.6, "haut - bas : l'écart moyen entre les deux stimuli vaut {moy_diff}");
}

/// juste : **la sonde est juste** (V-FLX-02) : les trois parts de la coupe sont celles d'un flux de 1 qui va de `(7,
/// 5)` à `(13, 5)`, **donc `haut + axe + bas = 1`** à `1e-9` près (mesuré : `2.4e-14` au pire sur 50 graines), pour
/// chaque graine, pour le témoin et pour le stimulus ; et sur des tubes **tous égaux** (`D = 1`), la grille est
/// symétrique par rapport à l'axe : `|haut - bas| <= 1e-9` (la sonde n'a aucun biais de côté), alors que `haut` et
/// `bas` valent chacun plus de `0.05` (le flux se répartit : un code qui rendrait zéro partout échouerait).
///
/// Ne couvre pas : le détail des arêtes de la coupe (la doc de `coupe` dans `tests/common/memoire_2m.rs`).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn la_sonde_est_juste() {
    let b = banc();
    for &g in &GRAINES {
        for stimuli in [vec![], vec![STIM_HAUT]] {
            let etat = jouer(&b, &STANDARD, 0, g, &stimuli, &[400], None);
            let (h, a, bs) = coupe(&b, &etat[0].d, 7, 13);
            assert!((h + a + bs - 1.0).abs() <= 1e-9, "graine {g} : {h} + {a} + {bs}");
        }
    }
    let uniformes = vec![1.0; b.graphe.nb_aretes()];
    let (h, a, bs) = coupe(&b, &uniformes, 7, 13);
    assert!((h - bs).abs() <= 1e-9, "tubes égaux : {h} contre {bs}");
    assert!(h > 0.05 && bs > 0.05 && a > 0.0, "tubes égaux : {h}, {a}, {bs}");
    assert!((h + a + bs - 1.0).abs() <= 1e-9);
}
