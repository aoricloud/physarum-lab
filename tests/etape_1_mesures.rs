//! Étape 1 : `mesures::tubes_vifs`, le nombre de tubes vifs et leur longueur. Valeurs attendues :
//! `references/etape-1-valeurs.md`, identifiants `V-MES-xx`.
//!
//! Ce fichier existe parce que le seuil des tubes vifs ne se teste pas à sa limite par `lancer` (aucun scénario ne tire
//! une conductivité exactement égale à 0,5) : la mesure est donc une fonction à part, que `lancer` appelle, et qui
//! s'éprouve ici sur des conductivités choisies. Toutes les valeurs sont des calculs exacts faits à la main, ou de
//! l'arithmétique IEEE 754 vérifiée avec Python.

use physarum_lab::graphe::{Arete, Graphe, Noeud};
use physarum_lab::mesures::tubes_vifs;

/// Une chaîne de `longueurs.len() + 1` nœuds alignés ; l'arête `i` relie les nœuds `i` et `i + 1`, avec la longueur
/// donnée et le coût `10 (i + 1)` (jamais égal à la longueur : une mesure qui lirait le coût se trompe).
fn chaine(longueurs: &[f64]) -> Graphe {
    let noeuds = (0..=longueurs.len()).map(|i| Noeud { x: i as i32, y: 0, etage: 0 }).collect();
    let aretes = longueurs
        .iter()
        .enumerate()
        .map(|(i, &longueur)| Arete { a: i, b: i + 1, longueur, cout: 10.0 * (i + 1) as f64 })
        .collect();
    Graphe::new(noeuds, aretes).expect("chaîne valide")
}

/// juste : un tube est vif quand sa conductivité vaut AU MOINS le seuil, le seuil compris (V-MES-01). Sur une chaîne de
/// quatre tubes de longueurs `1,5 ; 2,25 ; 4 ; 8,5` et de conductivités `0,5`, le flottant juste en dessous de 0,5
/// (`0,49999999999999994`), celui juste au-dessus (`0,5000000000000001`) et `1e-6` : au seuil `0,5`, deux tubes vifs
/// (le premier et le troisième) de longueur `1,5 + 4 = 5,5` ; au seuil `1e-6`, les quatre, de longueur `16,25` ; au
/// seuil `0,6`, aucun : `(0, 0.0)` ; au seuil du flottant juste au-dessus de 0,5, un seul (le troisième). Les sommes
/// sont exactes en flottants. La longueur est celle des arêtes, pas leur coût (10, 20, 30, 40).
///
/// Ne couvre pas : l'ordre de la somme (test suivant).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn le_seuil_est_compris() {
    let g = chaine(&[1.5, 2.25, 4.0, 8.5]);
    let dessous = f64::from_bits(0.5_f64.to_bits() - 1);
    let dessus = f64::from_bits(0.5_f64.to_bits() + 1);
    assert!(dessous < 0.5 && dessus > 0.5);
    let d = [0.5, dessous, dessus, 1e-6];
    assert_eq!(tubes_vifs(&g, &d, 0.5), (2, 5.5));
    assert_eq!(tubes_vifs(&g, &d, 1e-6), (4, 16.25));
    assert_eq!(tubes_vifs(&g, &d, dessus), (1, 4.0));
    assert_eq!(tubes_vifs(&g, &d, dessous), (3, 7.75));
    let (vifs, longueur) = tubes_vifs(&g, &d, 0.6);
    assert_eq!((vifs, longueur.to_bits()), (0, 0.0_f64.to_bits()), "aucun tube vif : zéro et +0.0");
    // le même tube, vif ou non selon le seul seuil
    assert_eq!(tubes_vifs(&g, &[0.5, 0.5, 0.5, 0.5], 0.5), (4, 16.25));
    assert_eq!(tubes_vifs(&g, &[0.5, 0.5, 0.5, 0.5], dessus), (0, 0.0));
}

/// juste : la longueur est la somme des longueurs des tubes vifs, ajoutées une à une dans l'ordre des indices, à partir
/// de `0.0` (V-MES-02) : une somme de flottants dépend de son ordre. Longueurs `1e16 ; 1 ; 1`, tous vifs : `1e16 + 1`
/// s'arrondit à `1e16` (au pair), deux fois, donc `1e16` ; une somme faite dans l'autre sens donnerait `1e16 + 2`. Les
/// mêmes longueurs rangées `1 ; 1 ; 1e16` donnent `1e16 + 2` (`10000000000000002`). Longueurs `0,1 ; 0,2 ; 0,3` :
/// `(0,1 + 0,2) + 0,3 = 0,6000000000000001` (bits `0x3FE3333333333334`), alors que `0,1 + (0,2 + 0,3)` vaut `0,6`. Un
/// tube qui n'est pas vif n'entre pas dans la somme : `1e16 ; 1 ; 1` avec le premier tube mort donne `2`.
///
/// Ne couvre pas : les graphes de plus de trois arêtes (la règle est la même).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn la_somme_suit_l_ordre_des_indices() {
    let tous = [1.0, 1.0, 1.0];
    let (vifs, longueur) = tubes_vifs(&chaine(&[1e16, 1.0, 1.0]), &tous, 0.5);
    assert_eq!((vifs, longueur.to_bits()), (3, 1e16_f64.to_bits()));
    let (vifs, longueur) = tubes_vifs(&chaine(&[1.0, 1.0, 1e16]), &tous, 0.5);
    assert_eq!((vifs, longueur.to_bits()), (3, 10_000_000_000_000_002.0_f64.to_bits()));
    let (vifs, longueur) = tubes_vifs(&chaine(&[0.1, 0.2, 0.3]), &tous, 0.5);
    assert_eq!((vifs, longueur.to_bits()), (3, 0x3FE3_3333_3333_3334));
    let (vifs, longueur) = tubes_vifs(&chaine(&[0.3, 0.2, 0.1]), &tous, 0.5);
    assert_eq!((vifs, longueur.to_bits()), (3, 0.6_f64.to_bits()));
    assert_eq!(tubes_vifs(&chaine(&[1e16, 1.0, 1.0]), &[0.25, 1.0, 1.0], 0.5), (2, 2.0));
    assert_eq!(tubes_vifs(&chaine(&[1e16, 1.0, 1.0]), &[1.0, 0.25, 1.0], 0.5), (2, 1e16));
}

/// juste : les cas du bord (V-MES-03, la doc de `mesures`) : une conductivité NaN n'est jamais vive ; un seuil NaN ne
/// rend aucun tube vif ; une conductivité infinie est vive ; une conductivité négative ne l'est pas (la comparaison ne
/// prend pas de valeur absolue) ; un graphe d'un seul nœud, sans arête, donne `(0, 0.0)` ; et `tubes_vifs` panique si
/// le nombre de conductivités n'est pas le nombre d'arêtes (une de moins, une de trop).
///
/// Ne couvre pas : le message de la panique.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn nan_graphe_sans_arete_et_panique() {
    let g = chaine(&[1.5, 2.25, 4.0]);
    assert_eq!(tubes_vifs(&g, &[f64::NAN, 1.0, f64::NAN], 0.5), (1, 2.25));
    assert_eq!(tubes_vifs(&g, &[1.0, 1.0, 1.0], f64::NAN), (0, 0.0));
    assert_eq!(tubes_vifs(&g, &[f64::INFINITY, 0.1, 0.7], 0.5), (2, 5.5));
    assert_eq!(tubes_vifs(&g, &[-0.8, 0.9, f64::NEG_INFINITY], 0.5), (1, 2.25));
    assert_eq!(tubes_vifs(&g, &[-0.8, 0.9, -0.1], -0.5), (2, 6.25), "un seuil négatif se compare de même");
    let seul = Graphe::new(vec![Noeud { x: 0, y: 0, etage: 0 }], vec![]).expect("un nœud seul");
    let (vifs, longueur) = tubes_vifs(&seul, &[], 0.5);
    assert_eq!((vifs, longueur.to_bits()), (0, 0.0_f64.to_bits()));
    // la panique, après les cas qui réussissent : un code qui ne ferait que paniquer échoue plus haut
    assert!(std::panic::catch_unwind(|| tubes_vifs(&g, &[1.0, 1.0], 0.5)).is_err(), "une conductivité de moins");
    assert!(std::panic::catch_unwind(|| tubes_vifs(&g, &[1.0, 1.0, 1.0, 1.0], 0.5)).is_err(), "une de trop");
    assert!(std::panic::catch_unwind(|| tubes_vifs(&seul, &[1.0], 0.5)).is_err(), "une conductivité sans arête");
}
