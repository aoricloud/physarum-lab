//! Le flux de Kirchhoff : pressions aux nœuds et flux dans les arêtes, dans un ordre fixe (étape 0).
//!
//! # Le problème
//!
//! Chaque arête `e` d'un [`Graphe`], de bouts `a < b`, est un tube de **conductance** `g[e] >= 0` (zéro : le tube ne
//! laisse rien passer). Chaque nœud `i` a une pression `p[i]`. Le **flux** dans l'arête `e` est
//!
//! ```text
//! Q[e] = g[e] * (p[a] - p[b])        (positif quand le liquide va de a vers b, du petit indice vers le grand)
//! ```
//!
//! On injecte `injections[i]` à chaque nœud (positif : une source ; négatif : un puits). Un nœud, la **masse**, a la
//! pression zéro et absorbe ou fournit ce qu'il faut : son injection n'est pas lue. En tout autre nœud `i`, ce qui
//! sort par les arêtes égale ce qui est injecté (la loi de Kirchhoff) :
//!
//! ```text
//! somme, sur les arêtes e du nœud i, de  g[e] * (p[i] - p[autre bout de e])  =  injections[i]
//! ```
//!
//! [`pressions`] résout ce système ; [`flux`] en tire le flux de chaque arête.
//!
//! # Les vérifications de [`pressions`], dans cet ordre
//!
//! 1. `conductances.len()` doit être le nombre d'arêtes et `injections.len()` le nombre de nœuds, sinon
//!    `ErreurSolveur::Taille` ;
//! 2. `masse` doit être un indice de nœud, sinon `ErreurSolveur::Masse` ;
//! 3. chaque conductance doit être finie et `>= 0.0` (`-0.0` est accepté et vaut zéro), sinon
//!    `ErreurSolveur::Conductance(e)`, `e` étant le plus petit indice d'arête fautive ;
//! 4. chaque injection doit être finie (celle de la masse aussi), sinon `ErreurSolveur::Injection(i)`, `i` étant le
//!    plus petit indice de nœud fautif ;
//! 5. chaque nœud doit être relié à la masse par un chemin d'arêtes de conductance **strictement positive**, sinon
//!    `ErreurSolveur::Isole(i)`, `i` étant le plus petit indice d'un nœud qui ne l'est pas ;
//! 6. pendant le calcul, chaque pivot `d[r]` (voir plus bas), au moment où il est calculé, doit être fini et `> 0.0`,
//!    sinon `ErreurSolveur::Pivot(i)`, `i` étant le nœud du premier rang `r` fautif (seuls les pivots sont contrôlés,
//!    pas les autres nombres du calcul). Cela arrive quand une somme déborde, ou
//!    quand des nœuds ne tiennent à la masse que par une conductance si petite devant leurs autres conductances qu'elle
//!    se perd dans l'arrondi (`1.0 + 1e-300` vaut `1.0`) : le réseau est alors relié sur le papier, pas pour le calcul.
//!
//! # Ce que le solveur ne garantit pas
//!
//! La précision se perd quand les conductances s'étalent sur beaucoup d'ordres de grandeur. Si un groupe de nœuds ne
//! tient à la masse que par des conductances `r` fois plus petites que ses autres conductances, ses pressions (et le
//! flux qui passe par ces petites conductances) ne sont justes qu'à environ `r * 2^-53` près en relatif : dix chiffres
//! perdus pour `r = 1e10`, et un pivot nul (l'erreur `Pivot`) quand `r` dépasse `2^53`. Un modèle qui laisse mourir des
//! tubes doit donc borner ce rapport lui-même (un plancher de conductance, ou des tubes retirés du réseau).
//!
//! # La procédure de référence : elle fixe le résultat au bit près
//!
//! Une somme de flottants dépend de son ordre : pour que la même entrée donne les mêmes bits partout, le résultat de
//! [`pressions`] est **celui de la procédure qui suit, bit pour bit**. Soit `n` le nombre de nœuds et `N = n - 1`. Le
//! **rang** d'un nœud `i` autre que la masse est `i` si `i < masse`, et `i - 1` sinon : les inconnues sont les nœuds
//! autres que la masse, par indice croissant. Dans ce qui suit, « pour i de a à b » parcourt `a, a + 1, …, b`, bornes
//! comprises, et ne fait rien si `b < a`. Si `N` vaut 0 (un graphe d'un seul nœud, la masse), le résultat est `[+0.0]`.
//!
//! ```text
//! -- la matrice A (N × N, symétrique) et le second membre s
//! A est pleine de +0.0
//! pour chaque nœud i autre que la masse, de rang r :
//!     A[r][r] = la somme, à partir de 0.0, des g[e] des arêtes e du nœud i, par indice d'arête croissant
//!               (les arêtes qui vont à la masse comprises)
//!     s[r]    = injections[i] + 0.0                 (le « + 0.0 » change un -0.0 en +0.0)
//! pour chaque arête e de bouts a et b, tous deux autres que la masse, avec g[e] > 0.0 :
//!     A[rang(a)][rang(b)] = A[rang(b)][rang(a)] = -g[e]
//!
//! -- la factorisation A = L · diag(d) · Lᵀ, sans pivotage, ligne après ligne
//! pour i de 0 à N-1 :
//!     pour j de 0 à i-1 :
//!         t = A[i][j]
//!         pour k de 0 à j-1 :   t = t - (L[i][k] * d[k]) * L[j][k]
//!         L[i][j] = t / d[j]
//!     t = A[i][i]
//!     pour k de 0 à i-1 :       t = t - (L[i][k] * d[k]) * L[i][k]
//!     d[i] = t                                      (le pivot : fini et > 0.0, sinon Pivot)
//!
//! -- la descente
//! pour i de 0 à N-1 :
//!     t = s[i]
//!     pour k de 0 à i-1 :       t = t - L[i][k] * y[k]
//!     y[i] = t
//!
//! -- la diagonale
//! pour i de 0 à N-1 :           x[i] = y[i] / d[i]
//!
//! -- la remontée, ligne après ligne en partant de la dernière
//! pour k de N-1 à 1, en décroissant :
//!     pour i de 0 à k-1 :       x[i] = x[i] - L[k][i] * x[k]
//!
//! -- le résultat
//! p[i] = x[rang(i)] pour tout nœud i autre que la masse ;  p[masse] = +0.0
//! ```
//!
//! Chaque ligne est une opération `f64` ordinaire, dans l'ordre et avec les parenthèses écrits (`(L[i][k] * d[k]) *
//! L[j][k]`, pas `L[i][k] * (d[k] * L[j][k])` ; ni `mul_add`, ni somme regroupée ou réordonnée).
//!
//! # Ce qu'une mise en œuvre doit faire en plus : sauter les zéros du profil
//!
//! Écrite telle quelle, la procédure coûte `N³` opérations : c'est trop pour une grille. Or presque tout `L` est nul.
//! Soit `premier(i)`, pour le rang `i`, le plus petit rang `j <= i` tel que `j == i` ou qu'une arête relie les nœuds de
//! rangs `i` et `j` (toute arête du graphe entre deux nœuds autres que la masse, quelle que soit sa conductance : le
//! profil ne dépend que du graphe et de la masse). Alors `L[i][k]` vaut exactement `+0.0` pour tout `k < premier(i)` (le « profil » de la ligne `i`
//! commence à `premier(i)`), et retrancher un produit dont un facteur est ce `+0.0` ne change aucun bit (aucun `t` de
//! la procédure ne vaut jamais `-0.0`). La mise en œuvre **ne range et ne parcourt donc, pour la ligne `i`, que les
//! colonnes de `premier(i)` à `i - 1`** : dans chaque boucle sur `k` elle part de `max(premier(i), premier(j))` au lieu
//! de 0, la boucle sur `j` part de `premier(i)`, et dans la remontée `i` va de `premier(k)` à `k - 1`. L'ordre des
//! opérations qui restent est inchangé, donc les bits aussi. Le rangement est libre : par exemple un `Vec` de
//! `i - premier(i)` nombres par ligne, où `L` remplace `A` au fil du calcul, et la diagonale à part. Sur une [`Grille`](crate::graphe::Grille), où les indices
//! de deux nœuds voisins diffèrent d'au plus `(largeur + 1) * etages`, le coût tombe à environ
//! `N * ((largeur + 1) * etages)²` opérations, et la mémoire à `N * (largeur + 1) * etages` nombres : une matrice
//! `N × N` entière ne doit jamais être allouée.

use crate::graphe::Graphe;

/// Pourquoi les pressions n'ont pas pu être calculées (voir « Les vérifications » dans la documentation du module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurSolveur {
    /// `conductances` ou `injections` n'a pas la bonne longueur.
    Taille,
    /// `masse` n'est pas un indice de nœud.
    Masse,
    /// La conductance de cette arête (le plus petit indice fautif) n'est pas un nombre fini `>= 0.0`.
    Conductance(usize),
    /// L'injection de ce nœud (le plus petit indice fautif) n'est pas un nombre fini.
    Injection(usize),
    /// Ce nœud (le plus petit indice dans ce cas) n'est relié à la masse par aucun chemin d'arêtes de conductance
    /// strictement positive.
    Isole(usize),
    /// Le pivot de ce nœud n'est pas fini ou pas strictement positif : une somme a débordé, ou une conductance s'est
    /// perdue dans l'arrondi (voir la vérification 6).
    Pivot(usize),
}

/// Les pressions aux nœuds : un vecteur de `nb_noeuds` nombres, `p[masse]` valant `+0.0`. Le résultat est, bit pour
/// bit, celui de la procédure de référence de la documentation du module, calculé en sautant les zéros du profil.
///
/// # Erreurs
///
/// Celles de « Les vérifications », dans leur ordre : la première qui s'applique.
pub fn pressions(
    graphe: &Graphe,
    conductances: &[f64],
    injections: &[f64],
    masse: usize,
) -> Result<Vec<f64>, ErreurSolveur> {
    let n = graphe.nb_noeuds();
    let m = graphe.nb_aretes();
    // 1. les longueurs
    if conductances.len() != m || injections.len() != n {
        return Err(ErreurSolveur::Taille);
    }
    // 2. la masse
    if masse >= n {
        return Err(ErreurSolveur::Masse);
    }
    // 3. les conductances
    for (e, &g) in conductances.iter().enumerate() {
        if !(g.is_finite() && g >= 0.0) {
            return Err(ErreurSolveur::Conductance(e));
        }
    }
    // 4. les injections (celle de la masse aussi)
    for (i, &b) in injections.iter().enumerate() {
        if !b.is_finite() {
            return Err(ErreurSolveur::Injection(i));
        }
    }
    // 5. le plus petit nœud qu'aucun chemin d'arêtes de conductance strictement positive ne relie à la masse
    let mut vus = vec![false; n];
    vus[masse] = true;
    let mut pile = vec![masse];
    while let Some(ici) = pile.pop() {
        for &e in graphe.incidentes(ici) {
            if conductances[e] > 0.0 {
                let arete = graphe.aretes()[e];
                let autre = if arete.a == ici { arete.b } else { arete.a };
                if !vus[autre] {
                    vus[autre] = true;
                    pile.push(autre);
                }
            }
        }
    }
    if let Some(i) = (0..n).find(|&i| !vus[i]) {
        return Err(ErreurSolveur::Isole(i));
    }

    let nn = n - 1;
    if nn == 0 {
        return Ok(vec![0.0]);
    }

    // Le rang d'un nœud autre que la masse, et le nœud d'un rang.
    let rang = |i: usize| if i < masse { i } else { i - 1 };
    let noeud = |r: usize| if r < masse { r } else { r + 1 };

    // Le profil : premier[i], le plus petit rang relié au rang i par une arête du graphe.
    let mut premier: Vec<usize> = (0..nn).collect();
    for arete in graphe.aretes() {
        if arete.a != masse && arete.b != masse {
            let ra = rang(arete.a);
            let rb = rang(arete.b);
            let (bas, haut) = if ra < rb { (ra, rb) } else { (rb, ra) };
            if bas < premier[haut] {
                premier[haut] = bas;
            }
        }
    }

    // La factorisation A = L · diag(d) · Lᵀ : la ligne i ne range que les colonnes de premier(i) à i - 1.
    let mut l: Vec<Vec<f64>> = Vec::with_capacity(nn);
    let mut d = vec![0.0_f64; nn];
    for i in 0..nn {
        let debut = premier[i];
        let mut ligne = vec![0.0_f64; i - debut];
        let sommet = noeud(i);
        // La diagonale : toutes les arêtes du nœud, celles vers la masse comprises, par indice croissant.
        let mut diagonale = 0.0_f64;
        for &e in graphe.incidentes(sommet) {
            let g = conductances[e];
            diagonale += g;
            let arete = graphe.aretes()[e];
            let autre = if arete.a == sommet { arete.b } else { arete.a };
            if autre != masse && g > 0.0 {
                let j = rang(autre);
                if j >= debut && j < i {
                    ligne[j - debut] = -g;
                }
            }
        }
        for j in debut..i {
            let mut t = ligne[j - debut];
            let debut_j = premier[j];
            let bas = if debut_j > debut { debut_j } else { debut };
            for k in bas..j {
                t -= (ligne[k - debut] * d[k]) * l[j][k - debut_j];
            }
            ligne[j - debut] = t / d[j];
        }
        let mut pivot = diagonale;
        for k in debut..i {
            pivot -= (ligne[k - debut] * d[k]) * ligne[k - debut];
        }
        if !(pivot.is_finite() && pivot > 0.0) {
            return Err(ErreurSolveur::Pivot(sommet));
        }
        d[i] = pivot;
        l.push(ligne);
    }

    // La descente.
    let mut y = vec![0.0_f64; nn];
    for i in 0..nn {
        let mut t = injections[noeud(i)] + 0.0;
        let debut = premier[i];
        for k in debut..i {
            t -= l[i][k - debut] * y[k];
        }
        y[i] = t;
    }

    // La diagonale.
    let mut x: Vec<f64> = y.iter().zip(d.iter()).map(|(y, d)| y / d).collect();

    // La remontée, de la dernière ligne à la première.
    for k in (1..nn).rev() {
        let debut = premier[k];
        for i in debut..k {
            x[i] -= l[k][i - debut] * x[k];
        }
    }

    let mut p = vec![0.0_f64; n];
    for i in 0..n {
        if i != masse {
            p[i] = x[rang(i)];
        }
    }
    Ok(p)
}

/// Le flux de chaque arête, dans l'ordre des indices : `Q[e] = conductances[e] * (pressions[a] - pressions[b])`, où
/// `a < b` sont les bouts de l'arête `e` (une soustraction puis un produit, dans cet ordre).
///
/// # Panique
///
/// Si `conductances.len()` n'est pas le nombre d'arêtes ou `pressions.len()` le nombre de nœuds.
pub fn flux(graphe: &Graphe, conductances: &[f64], pressions: &[f64]) -> Vec<f64> {
    assert_eq!(conductances.len(), graphe.nb_aretes(), "une conductance par arête");
    assert_eq!(pressions.len(), graphe.nb_noeuds(), "une pression par nœud");
    graphe
        .aretes()
        .iter()
        .enumerate()
        .map(|(e, arete)| conductances[e] * (pressions[arete.a] - pressions[arete.b]))
        .collect()
}
