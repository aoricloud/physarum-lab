//! Aides communes aux tests de l'étape 2T : de petits graphes, et le transport de l'agent écrit **naïvement**, d'après
//! la seule doc de `souvenir` (une boucle par arête qui ajoute chaque terme au nœud concerné ; la doc dit que l'ordre
//! par arête croissante et l'ordre par `incidentes` sont le même). Rien n'est emprunté au code jugé (hors `Graphe`).
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use physarum_lab::graphe::{Arete, Graphe, Noeud};

/// Le nombre de sous-pas au-delà duquel la doc refuse (`souvenir::MAX_SOUS_PAS`, recopié : un million).
pub const MAX_SOUS_PAS: usize = 1_000_000;

/// Un graphe de `n` nœuds alignés (position `(i, 0)`, étage 0) et d'arêtes `(a, b, longueur)` ; le **coût** de chaque
/// arête vaut trois fois sa longueur (le transport ne lit que la longueur : un code qui lirait le coût se tromperait).
pub fn graphe_l(n: usize, aretes: &[(usize, usize, f64)]) -> Graphe {
    let noeuds = (0..n).map(|i| Noeud { x: i as i32, y: 0, etage: 0 }).collect();
    let aretes = aretes.iter().map(|&(a, b, longueur)| Arete { a, b, longueur, cout: 3.0 * longueur }).collect();
    Graphe::new(noeuds, aretes).expect("graphe valide")
}

/// Une chaîne `0-1-...-n` dont les arêtes ont ces longueurs.
pub fn chaine(longueurs: &[f64]) -> Graphe {
    let aretes: Vec<(usize, usize, f64)> = longueurs.iter().enumerate().map(|(i, &l)| (i, i + 1, l)).collect();
    graphe_l(longueurs.len() + 1, &aretes)
}

/// Les bits de chaque nombre.
pub fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// Le flottant juste au-dessus de `x` (`x` fini et `>= 0.0`).
pub fn suivant(x: f64) -> f64 {
    f64::from_bits(x.to_bits() + 1)
}

/// Le flottant juste au-dessous de `x` (`x` fini et `> 0.0`).
pub fn precedent(x: f64) -> f64 {
    f64::from_bits(x.to_bits() - 1)
}

/// Les taux de départ et les sorties de chaque nœud, par une boucle sur les arêtes (voir la doc, « Le pas, à la lettre
/// »).
pub fn naif_taux(graphe: &Graphe, flux: &[f64], kappa: f64, k: f64) -> (Vec<f64>, Vec<f64>) {
    let n = graphe.nb_noeuds();
    let mut r = vec![0.0; n];
    let mut s = vec![0.0; n];
    for (e, arete) in graphe.aretes().iter().enumerate() {
        for (i, q) in [(arete.a, flux[e]), (arete.b, -flux[e])] {
            s[i] += q;
            if q > 0.0 {
                r[i] += q;
            }
            r[i] += kappa / arete.longueur;
        }
    }
    let sortie: Vec<f64> = s.iter().map(|&si| if si < 0.0 { -si } else { 0.0 }).collect();
    for i in 0..n {
        r[i] += sortie[i];
        r[i] += k;
    }
    (r, sortie)
}

/// Un pas de la recette, par une boucle sur les arêtes : rend les concentrations d'après et `[sortie, perte, apport]`.
pub fn naif_pas(
    graphe: &Graphe,
    flux: &[f64],
    c: &[f64],
    kappa: f64,
    k: f64,
    apports: &[(usize, f64)],
    h: f64,
) -> (Vec<f64>, [f64; 3]) {
    let n = graphe.nb_noeuds();
    let (r, sortie_i) = naif_taux(graphe, flux, kappa, k);
    let mut g = vec![0.0; n];
    for (e, arete) in graphe.aretes().iter().enumerate() {
        for (i, j, q) in [(arete.a, arete.b, flux[e]), (arete.b, arete.a, -flux[e])] {
            if q < 0.0 {
                g[i] += (h * (-q)) * c[j];
            }
            g[i] += (h * (kappa / arete.longueur)) * c[j];
        }
    }
    let mut apres: Vec<f64> = (0..n).map(|i| (1.0 - h * r[i]) * c[i] + g[i]).collect();
    let (mut sortie, mut perte, mut apport) = (0.0, 0.0, 0.0);
    for i in 0..n {
        sortie += (h * sortie_i[i]) * c[i];
        perte += (h * k) * c[i];
    }
    for &(i, debit) in apports {
        apres[i] += h * debit;
        apport += h * debit;
    }
    (apres, [sortie, perte, apport])
}

/// Le plus grand `h` tel que `h * r_max <= 1.0` (l'infini si `r_max == 0.0`), cherché à partir de `1 / r_max` en
/// montant ou en descendant d'un cran à la fois.
pub fn naif_pas_maximal(graphe: &Graphe, flux: &[f64], kappa: f64, k: f64) -> f64 {
    let (r, _) = naif_taux(graphe, flux, kappa, k);
    let mut r_max = 0.0;
    for &x in &r {
        if x > r_max {
            r_max = x;
        }
    }
    if r_max == 0.0 {
        return f64::INFINITY;
    }
    let mut h = 1.0 / r_max;
    while h * r_max > 1.0 {
        h = precedent(h);
    }
    while suivant(h) * r_max <= 1.0 {
        h = suivant(h);
    }
    h
}

/// Le plus petit `n >= 1` tel que `h / n <= h_max`, en essayant chaque `n` (lent : pour de petits `n` seulement).
pub fn naif_sous_pas(h: f64, h_max: f64) -> usize {
    let mut n = 1;
    while h / (n as f64) > h_max {
        n += 1;
        assert!(n <= MAX_SOUS_PAS, "trop de sous-pas pour la version naïve");
    }
    n
}

/// Le pas public naïf : le nombre de sous-pas, puis chaque sous-pas de la recette. Rend les concentrations, le bilan
/// total (chaque total part de `0.0`, ajoute dans l'ordre des sous-pas) et `n`.
pub fn naif_avancer(
    graphe: &Graphe,
    flux: &[f64],
    c: &[f64],
    kappa: f64,
    k: f64,
    apports: &[(usize, f64)],
    h: f64,
) -> (Vec<f64>, [f64; 3], usize) {
    let h_max = naif_pas_maximal(graphe, flux, kappa, k);
    let n = naif_sous_pas(h, h_max);
    let h_sous = h / (n as f64);
    let mut courant = c.to_vec();
    let mut total = [0.0; 3];
    for _ in 0..n {
        let (suite, bilan) = naif_pas(graphe, flux, &courant, kappa, k, apports, h_sous);
        courant = suite;
        for t in 0..3 {
            total[t] += bilan[t];
        }
    }
    (courant, total, n)
}

/// La somme de gauche à droite à partir de `0.0`.
pub fn somme(v: &[f64]) -> f64 {
    let mut s = 0.0;
    for &x in v {
        s += x;
    }
    s
}
