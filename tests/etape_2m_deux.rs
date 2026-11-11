//! Étape 2, partie 2M : **deux stimuli** : du même côté, de côtés opposés. Valeurs attendues :
//! `references/etape-2m-valeurs.md`, identifiants `V-DEU-xx`.
//!
//! **Notre modèle**, aucun chiffre publié n'est visé : le supplément de Kramar et Alim 2021 (Fig. S8, « les mémoires
//! sont posées les unes sur les autres, chaque nouveau stimulus renforçant ou affaiblissant les tubes épais existants
//! ») **n'a pas été lu**, et le texte principal ne donne aucun chiffre. Le test constate ce qui est mesuré, **dans les
//! deux sens** : les stimuli du même côté **ne s'ajoutent pas en intensité** (ils rendent l'empreinte plus régulière),
//! ceux de côtés opposés **se partagent** une même réserve de tubes épais. Les seuils viennent d'une campagne de 50
//! graines (101 à 150) autres que celles du test ; le test joue les graines 1 à 6, à 400 pas, réglage standard (chaque
//! stimulus a le débit 2 sur les pas `[0 ; 100[`).

#[path = "common/memoire_2m.rs"]
mod memoire_2m;
#[path = "common/transport_2t.rs"]
mod transport_2t;

use memoire_2m::*;

const GRAINES: [u64; 6] = [1, 2, 3, 4, 5, 6];

/// Pour chaque graine : `(F_A, F_B, haut - bas de la sonde)` de l'état avec ces stimuli, après 400 pas.
fn configuration(stimuli: &[Stim]) -> Vec<(f64, f64, f64)> {
    let b = banc();
    GRAINES
        .iter()
        .map(|&g| {
            let avec = jouer(&b, &STANDARD, 0, g, stimuli, &[400], None);
            let tem = jouer(&b, &STANDARD, 0, g, &[], &[400], None);
            let (_, fa, fb) = ecart(&b, &avec[0].d, &tem[0].d);
            let (h, _, bs) = coupe(&b, &avec[0].d, 7, 13);
            (fa, fb, h - bs)
        })
        .collect()
}

fn moy(v: &[(f64, f64, f64)], champ: usize) -> f64 {
    v.iter().map(|x| [x.0, x.1, x.2][champ]).sum::<f64>() / v.len() as f64
}

/// juste : **deux stimuli du même côté ne s'ajoutent pas en intensité, ils rendent l'empreinte régulière** (V-DEU-01).
/// Un stimulus seul en `(7, 2)` : `F_A` (la moitié haute, stimulus moins témoin) vaut `+0.0298` en moyenne sur 50
/// graines (de `+0.0079` à `+0.0495`), la sonde `haut - bas` `+0.724` (de `+0.161` à `+1.000`). Deux stimuli en `(5,
/// 2)` et `(9, 2)` : `F_A = +0.0319` (**`+7 %`**, de `+0.0103` à `+0.0472`), mais la sonde ne descend **jamais** sous
/// `+0.675` (de `+0.675` à `+0.748`, moyenne `0.693` : l'empreinte est presque la même pour toutes les graines) ; deux
/// stimuli voisins en `(7, 2)` et `(7, 3)` : `F_A = +0.0276` (`-7 %`), sonde de `+0.326` à `+0.849`. **Ils ne se
/// renforcent donc pas en intensité** (le plateau sature : voir `etape_2m_empreinte`), mais le stimulus écarté rend la
/// forme reproductible. Sur les graines 1 à 6 : le rapport des `F_A` moyens (deux écartés sur un seul) est dans `[0.8,
/// 1.3]` ; la sonde des deux écartés est `>= 0.5` pour chaque graine (marge `1.35` sur le pire des 50) ; `F_B <= 1e-9`
/// pour chaque graine (le côté opposé n'est pas renforcé).
///
/// Ne couvre pas : plus de deux stimuli, ni des stimuli décalés dans le temps.
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn deux_stimuli_du_meme_cote_ne_s_ajoutent_pas() {
    let seul = configuration(&[STIM_HAUT]);
    let deux = configuration(&[Stim { x: 5, ..STIM_HAUT }, Stim { x: 9, ..STIM_HAUT }]);
    assert_eq!(seul.len(), GRAINES.len());
    let rapport = moy(&deux, 0) / moy(&seul, 0);
    assert!((0.8..=1.3).contains(&rapport), "F_A : {} contre {} (rapport {rapport})", moy(&deux, 0), moy(&seul, 0));
    for (i, (_, fb, sonde)) in deux.iter().enumerate() {
        assert!(*sonde >= 0.5, "graine {} : sonde {sonde}", GRAINES[i]);
        assert!(*fb <= 1e-9, "graine {} : F_B = {fb}", GRAINES[i]);
    }
}

/// juste : **deux stimuli de côtés opposés se partagent la réserve** (V-DEU-02). Avec `(7, 2)` et `(7, 8)` (symétriques
/// par rapport à l'axe), sur 50 graines : `F_A = +0.0145` et `F_B = +0.0165` en moyenne, **chacun environ la moitié du
/// `F_A` d'un stimulus seul (`+0.0298`)**, leur somme `+0.0310` ; l'écart d'empreinte `E` moyen est `-0.0020` (de
/// `-0.0386` à `+0.0435` : **les deux empreintes s'annulent en moyenne**, graine par graine l'une l'emporte) ; la sonde
/// choisit un côté au hasard : `haut - bas` vaut `+0.021` en moyenne (de `-0.778` à `+0.807`), `|haut - bas|` `0.260`
/// en moyenne, contre `0.724` pour un stimulus seul. Sur les graines 1 à 6 : `F_A` moyen `<= 0.8` fois celui d'un
/// stimulus seul (le rapport vaut `0.49` sur 50 graines) ; `F_A` moyen et `F_B` moyen chacun `>= 0.005` (les deux
/// empreintes existent) ; `|E|` moyen `<= 0.025` ; `|haut - bas|` moyen `<= 0.6` fois celui d'un stimulus seul (`0.36`
/// sur 50 graines). Dit dans l'autre sens : **deux stimuli opposés se partagent, ils ne se cumulent pas**.
///
/// Ne couvre pas : des stimuli opposés **décalés dans le temps** ni de positions différentes (`(4, 2)` et `(10, 8)`
/// donnent `E = +0.0129` en moyenne, `|haut - bas|` `0.40` : non testé).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn deux_stimuli_de_cotes_opposes_se_partagent() {
    let seul = configuration(&[STIM_HAUT]);
    let opp = configuration(&[STIM_HAUT, STIM_BAS]);
    assert_eq!(opp.len(), GRAINES.len());
    assert!(moy(&opp, 0) <= 0.8 * moy(&seul, 0), "F_A : {} contre {} seul", moy(&opp, 0), moy(&seul, 0));
    assert!(moy(&opp, 0) >= 0.005, "F_A = {}", moy(&opp, 0));
    assert!(moy(&opp, 1) >= 0.005, "F_B = {}", moy(&opp, 1));
    let e_moyen = (moy(&opp, 0) - moy(&opp, 1)).abs();
    assert!(e_moyen <= 0.025, "|E| moyen = {e_moyen}");
    let abs_opp = opp.iter().map(|x| x.2.abs()).sum::<f64>() / opp.len() as f64;
    let abs_seul = seul.iter().map(|x| x.2.abs()).sum::<f64>() / seul.len() as f64;
    assert!(abs_opp <= 0.6 * abs_seul, "|haut - bas| : {abs_opp} contre {abs_seul} seul");
}

/// juste : **un second stimulus donné quand le réseau s'est stabilisé ne s'imprime pas de son côté** (V-DEU-03) : le
/// stimulus du haut sur `[0 ; 100[`, puis un second en bas sur `[300 ; 400[` (après la stabilisation) : à 700 pas, la
/// moyenne des conductivités de la moitié basse (stimulus moins témoin) est **exactement la même** qu'avec le stimulus
/// du haut seul (`F_B` identique : `|F_B(deux) - F_B(un)| = 0` pour les 50 graines, 101 à 150) ; le second stimulus ne
/// laisse donc **aucune empreinte en bas**. Il change parfois la moitié haute (`|F_A(deux) - F_A(un)|` : `1.3e-2` au
/// pire sur 50 graines, `9e-5` sur les graines 1 à 6) : l'agent du second stimulus, porté vers le puits, touche au
/// passage les tubes du détour du premier ; et pour `4 %` des graines il fait basculer une arête de plus de `1e-3`
/// (`0.46` au pire sur une arête). Le test demande, pour chaque graine : `|F_B(deux) - F_B(un)| <= 1e-9` et `|ΔF_A| <=
/// 0.03` (marge `2.3`, `ΔF_A` la différence des `F_A`). Les mémoires ne se posent donc pas les unes sur les autres
/// quand le flux ne passe plus près du second stimulus.
///
/// Ne couvre pas : un second stimulus à un moment où le réseau bouge encore (V-DEU-01 et 02).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn un_second_stimulus_tardif_ne_s_imprime_pas() {
    let b = banc();
    let tard = Stim { debut: 300, fin: 400, ..STIM_BAS };
    for &g in &GRAINES {
        let tem = jouer(&b, &STANDARD, 0, g, &[], &[700], None);
        let seul = jouer(&b, &STANDARD, 0, g, &[STIM_HAUT], &[700], None);
        let deux = jouer(&b, &STANDARD, 0, g, &[STIM_HAUT, tard], &[700], None);
        assert!(deux[0].masse < seul[0].masse + 1.0, "garde-fou");
        let (_, fa1, fb1) = ecart(&b, &seul[0].d, &tem[0].d);
        let (_, fa2, fb2) = ecart(&b, &deux[0].d, &tem[0].d);
        assert!((fb2 - fb1).abs() <= 1e-9, "graine {g} : F_B = {fb2} contre {fb1}");
        assert!((fa2 - fa1).abs() <= 0.03, "graine {g} : F_A = {fa2} contre {fa1}");
        assert!(fa1 > 0.0, "graine {g} : le premier stimulus a laissé son empreinte ({fa1})");
    }
}

/// juste : **sans force, deux stimuli ne changent rien** (V-DEU-04) : avec `α = 0`, les conductivités avec un, deux
/// stimuli (de n'importe quels côtés) sont, bit pour bit, celles du témoin sans stimulus ; la masse d'agent, elle,
/// augmente avec le nombre de stimuli (plus d'une unité de plus avec le second).
///
/// Ne couvre pas : `α > 0`.
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn deux_stimuli_sans_force_ne_changent_rien() {
    let b = banc();
    let r = Reglage { force: 0.0, ..STANDARD };
    for &g in &GRAINES[..3] {
        let tem = jouer(&b, &r, 0, g, &[], &[300], None);
        let un = jouer(&b, &r, 0, g, &[STIM_HAUT], &[100, 300], None);
        let deux = jouer(&b, &r, 0, g, &[STIM_HAUT, STIM_BAS], &[100, 300], None);
        let bits = |d: &[f64]| d.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        assert_eq!(bits(&un[1].d), bits(&tem[0].d));
        assert_eq!(bits(&deux[1].d), bits(&tem[0].d));
        assert!(un[0].masse > 1.0);
        assert!(deux[0].masse > un[0].masse + 1.0, "{} contre {}", deux[0].masse, un[0].masse);
    }
}
