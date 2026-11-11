//! Les mesures d'un réseau de tubes : ce qu'on compte sur un état, sans le changer (étape 1, puis chaque étape).
//!
//! À l'étape 1 (le cœur), une seule mesure : les **tubes vifs** et leur longueur ([`tubes_vifs`]). Les mesures des
//! réseaux (longueur rapportée à l'arbre couvrant minimal, distance moyenne, tenue à une coupure) viendront avec la
//! partie « les réseaux » de l'étape. Ce que les tests d'acceptation prouvent est dans `references/etape-1-valeurs.md`
//! (identifiants `V-MES-xx`).
//!
//! # Un tube vif
//!
//! Un tube est **vif** quand sa conductivité vaut **au moins** un seuil : `conductivite >= seuil`, le seuil compris.
//! C'est une comparaison de flottants et rien d'autre : une conductivité NaN n'est jamais vive, et avec un seuil NaN
//! aucun tube n'est vif. `lancer` compte les tubes vifs au seuil `0.5`, la moitié du débit (voir `crate::lancer`).

use crate::graphe::Graphe;

/// Le nombre de tubes vifs et leur longueur : `(vifs, longueur)`.
///
/// `vifs` est le nombre d'arêtes `e` telles que `conductivites[e] >= seuil`. `longueur` est la somme de leurs
/// `Arete::longueur` (jamais `Arete::cout`), ajoutées **une à une dans l'ordre des indices d'arête, à partir de
/// `0.0`** : c'est cette somme-là, bit pour bit (une somme de flottants dépend de son ordre). Sans aucun tube vif, et
/// pour un graphe sans arête, le résultat est `(0, 0.0)`.
///
/// # Panique
///
/// Si `conductivites.len()` n'est pas `graphe.nb_aretes()`.
#[allow(unused_variables)]
pub fn tubes_vifs(graphe: &Graphe, conductivites: &[f64], seuil: f64) -> (usize, f64) {
    todo!()
}
