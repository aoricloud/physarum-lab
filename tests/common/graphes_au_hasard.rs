//! Aides communes aux tests de l'étape 1 : des graphes pondérés tirés au hasard, un Bellman-Ford naïf, la longueur du
//! second meilleur chemin. Tout est écrit ici, sans rien emprunter au code jugé (hors `Graphe`, `Alea` et leurs types).
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Arete, Graphe, Noeud};

/// Un graphe de `n` nœuds alignés (position `(i, 0)`, étage 0) et d'arêtes `(a, b, coût)` ; la longueur de chaque arête
/// est son coût.
pub fn graphe_pondere(n: usize, aretes: &[(usize, usize, f64)]) -> Graphe {
    let noeuds = (0..n).map(|i| Noeud { x: i as i32, y: 0, etage: 0 }).collect();
    let aretes = aretes.iter().map(|&(a, b, cout)| Arete { a, b, longueur: cout, cout }).collect();
    Graphe::new(noeuds, aretes).expect("graphe valide")
}

/// Un graphe connexe tiré au hasard : un arbre (chaque nœud `i >= 1` relié à un nœud plus petit), plus `en_plus` essais
/// d'arêtes supplémentaires (les boucles et les doubles sont sautés). Le coût de chaque arête est `cout_min + amplitude
/// * u`, `u` uniforme dans `[0, 1[` (une arête tirée après l'autre, dans l'ordre de création).
pub fn graphe_au_hasard(alea: &mut Alea, n: usize, en_plus: usize, cout_min: f64, amplitude: f64) -> Graphe {
    let mut paires: Vec<(usize, usize)> = (1..n).map(|i| (alea.borne(i as u32) as usize, i)).collect();
    for _ in 0..en_plus {
        let (a, b) = (alea.borne(n as u32) as usize, alea.borne(n as u32) as usize);
        let (a, b) = (a.min(b), a.max(b));
        if a != b && !paires.contains(&(a, b)) {
            paires.push((a, b));
        }
    }
    let aretes: Vec<(usize, usize, f64)> =
        paires.iter().map(|&(a, b)| (a, b, cout_min + amplitude * alea.suivant_f64())).collect();
    graphe_pondere(n, &aretes)
}

/// Les distances d'un Bellman-Ford naïf depuis `source`, sur les coûts : on relâche toutes les arêtes, dans les deux
/// sens, tant que quelque chose change. Si `interdite` est `Some(e)`, l'arête `e` est ignorée (comme si elle n'existait
/// pas). `+inf` pour un nœud hors d'atteinte.
pub fn bellman_ford(graphe: &Graphe, source: usize, interdite: Option<usize>) -> Vec<f64> {
    let mut d = vec![f64::INFINITY; graphe.nb_noeuds()];
    d[source] = 0.0;
    loop {
        let mut change = false;
        for (e, arete) in graphe.aretes().iter().enumerate() {
            if Some(e) == interdite {
                continue;
            }
            for (u, v) in [(arete.a, arete.b), (arete.b, arete.a)] {
                let candidat = d[u] + arete.cout;
                if candidat < d[v] {
                    d[v] = candidat;
                    change = true;
                }
            }
        }
        if !change {
            return d;
        }
    }
}

/// La longueur du meilleur chemin de `source` à `puits` **différent de `chemin`** (qui est un plus court chemin) : le
/// plus petit, sur les arêtes `e` de `chemin`, du Bellman-Ford sans `e`. Tout autre chemin simple évite au moins une
/// arête de `chemin` (un chemin simple de la source au puits qui les contiendrait toutes serait `chemin`).
pub fn second_meilleur(graphe: &Graphe, source: usize, puits: usize, chemin: &[usize]) -> f64 {
    chemin.iter().map(|&e| bellman_ford(graphe, source, Some(e))[puits]).fold(f64::INFINITY, f64::min)
}
