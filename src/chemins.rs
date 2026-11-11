//! Les plus courts chemins (Dijkstra) : ce que le réseau de tubes doit retrouver (étape 1).
//!
//! Le blob cherche, dit le théorème de Bonifaci, Mehlhorn et Varma (2012), le plus court chemin entre une source et un
//! puits. Ce module le calcule d'une autre façon, par la méthode de Dijkstra, pour que les tests comparent les deux.
//!
//! # Le problème
//!
//! Dans un [`Graphe`], la longueur d'un chemin est la **somme des coûts** (`Arete::cout`, jamais `Arete::longueur`) de
//! ses arêtes. [`plus_courts`] rend, pour une `source`, la distance de la source à chaque nœud et, pour chaque nœud,
//! l'arête par laquelle on y arrive sur un plus court chemin ; [`Chemins::vers`] en tire le chemin lui-même.
//!
//! # La procédure : elle fixe le résultat au bit près et les égalités
//!
//! Plusieurs plus courts chemins peuvent être de même longueur, et une somme de flottants dépend de son ordre : pour
//! que deux mises en œuvre rendent les mêmes distances, les mêmes parents et les mêmes bits, le résultat de
//! [`plus_courts`] est **celui de la procédure qui suit**, quelle que soit la structure de données employée (une file
//! de priorité, un tas, un balayage). `n` est le nombre de nœuds.
//!
//! ```text
//! distances[v] = +inf pour tout nœud v, puis distances[source] = 0.0
//! parents[v]   = None pour tout nœud v
//! visite[v]    = faux pour tout nœud v
//! répéter :
//!     u = le nœud non visité de plus petite distance FINIE ; à égalité, le plus petit indice.
//!         S'il n'y en a aucun, la procédure s'arrête.
//!     visite[u] = vrai
//!     pour chaque arête e de u, par indice d'arête croissant (Graphe::incidentes), de bouts u et v :
//!         si v n'est pas visité :
//!             candidat = distances[u] + cout(e)                  (une addition de flottants, dans cet ordre)
//!             si candidat < distances[v]  (comparaison STRICTE) :
//!                 distances[v] = candidat ; parents[v] = Some(e)
//! ```
//!
//! Conséquences que les tests vérifient :
//! - à longueurs égales, le parent gardé est celui de la **première** arête qui a offert cette distance dans l'ordre de
//!   la procédure (un candidat égal ne remplace pas) ;
//! - un nœud hors d'atteinte garde `f64::INFINITY` et `None` ; une somme qui déborde (`1e308 + 1e308` vaut `+inf`)
//!   n'est jamais `<` à `+inf` : le nœud reste hors d'atteinte, comme s'il n'était relié que par un chemin infini ;
//! - la source garde la distance `+0.0` et le parent `None` ; sur un graphe d'un seul nœud, le résultat est `distances
//!   == [0.0]` et `parents == [None]` ;
//! - **les distances sont les mêmes, bit pour bit, que celles d'un Bellman-Ford naïf** (relâcher toutes les arêtes,
//!   dans les deux sens, jusqu'à ce que plus rien ne bouge) : l'addition de flottants est croissante (`a <= b` donne
//!   `a + c <= b + c`) et ne diminue jamais (`a + c >= a`), donc la plus petite somme faite « de gauche à droite le
//!   long d'un chemin » est la même quelle que soit la méthode. Les parents, eux, ne sont les mêmes qu'à égalité près :
//!   seule la procédure ci-dessus dit lequel est gardé.
//!
//! # La mise en œuvre
//!
//! Le balayage simple (chercher le nœud à chaque tour en parcourant les `n` nœuds) coûte `n²` et suffit aux graphes de
//! ce dépôt (quelques centaines de nœuds). Une autre structure est permise si le résultat est celui de la procédure.

use crate::graphe::Graphe;

/// Le résultat de [`plus_courts`] : pour la `source`, la distance à chaque nœud et l'arête par laquelle on y arrive.
#[derive(Clone, Debug, PartialEq)]
pub struct Chemins {
    /// Le nœud de départ.
    pub source: usize,
    /// `distances[v]` : la somme des coûts le long du plus court chemin de `source` à `v` (`+0.0` pour la source), ou
    /// `f64::INFINITY` si `v` est hors d'atteinte. Un vecteur de `nb_noeuds` nombres.
    pub distances: Vec<f64>,
    /// `parents[v]` : l'indice de l'arête par laquelle le plus court chemin arrive à `v`, `None` pour la source et pour
    /// les nœuds hors d'atteinte. Un vecteur de `nb_noeuds` valeurs.
    pub parents: Vec<Option<usize>>,
}

/// Les plus courts chemins depuis `source`, par la procédure de la documentation du module.
///
/// # Panique
///
/// Si `source` n'est pas un indice de nœud du graphe.
pub fn plus_courts(graphe: &Graphe, source: usize) -> Chemins {
    let n = graphe.nb_noeuds();
    assert!(source < n, "le nœud {source} n'est pas dans le graphe");
    let mut distances = vec![f64::INFINITY; n];
    let mut parents: Vec<Option<usize>> = vec![None; n];
    let mut visite = vec![false; n];
    distances[source] = 0.0;
    loop {
        let mut choix: Option<usize> = None;
        for v in 0..n {
            if !visite[v] && distances[v] < f64::INFINITY {
                match choix {
                    None => choix = Some(v),
                    Some(w) => {
                        if distances[v] < distances[w] {
                            choix = Some(v);
                        }
                    }
                }
            }
        }
        let Some(u) = choix else { break };
        visite[u] = true;
        for &e in graphe.incidentes(u) {
            let arete = &graphe.aretes()[e];
            let v = if arete.a == u { arete.b } else { arete.a };
            if !visite[v] {
                let candidat = distances[u] + arete.cout;
                if candidat < distances[v] {
                    distances[v] = candidat;
                    parents[v] = Some(e);
                }
            }
        }
    }
    Chemins { source, distances, parents }
}

impl Chemins {
    /// Les indices des arêtes du plus court chemin de la source à `cible`, **dans l'ordre du trajet**, de la source
    /// vers la cible : on remonte les parents de `cible` jusqu'à la source, puis on renverse. `Some(vec![])` pour la
    /// source elle-même, `None` si `cible` est hors d'atteinte (distance infinie). `graphe` est celui qu'on a donné à
    /// [`plus_courts`] (avec un autre, le résultat n'est pas défini).
    ///
    /// # Panique
    ///
    /// Si `cible` n'est pas un indice de nœud du graphe.
    pub fn vers(&self, graphe: &Graphe, cible: usize) -> Option<Vec<usize>> {
        assert!(cible < graphe.nb_noeuds(), "le nœud {cible} n'est pas dans le graphe");
        if self.distances[cible].is_infinite() {
            return None;
        }
        let mut chemin = Vec::new();
        let mut ici = cible;
        while ici != self.source {
            let e = self.parents[ici].expect("un nœud atteint a un parent");
            chemin.push(e);
            let arete = &graphe.aretes()[e];
            ici = if arete.a == ici { arete.b } else { arete.a };
        }
        chemin.reverse();
        Some(chemin)
    }
}
