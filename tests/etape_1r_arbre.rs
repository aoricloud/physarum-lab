//! Étape 1, partie 1R : l'arbre couvrant minimal des nourritures, `reseaux::arbre_couvrant` (`TL_MST` et `MD_MST`).
//! Valeurs attendues : `references/etape-1r-valeurs.md`, identifiants `V-MST-xx`.
//!
//! Source : Tero et al. 2010 normalisent TL, MD et FT par « la valeur correspondante de l'arbre couvrant minimal » (p.
//! 440), sans formule. Tout ce qui est testé est donc **la doc de `reseaux`** (clôture métrique par les longueurs,
//! Kruskal, égalités au plus petit `(distance, i, j)`, sommes dans un ordre fixe), jugée par : des graphes écrits à la
//! main dont les valeurs viennent du modèle Python `ref_reseaux.py` et se vérifient à la main (écrites en commentaire),
//! un arbre naïf écrit ici (Bellman-Ford sur les longueurs, toutes les paires triées, union-find, parcours de l'arbre),
//! et une énumération de tous les arbres couvrants pour la minimalité.
// Les boucles à indices suivent les procédures de la doc ligne pour ligne.
#![allow(clippy::needless_range_loop)]

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Arete, Graphe, Noeud};
use physarum_lab::reseaux::{Arbre, ErreurReseau, arbre_couvrant};

/// Un graphe de `n` nœuds alignés et d'arêtes `(a, b, longueur, coût)`.
fn graphe_lc(n: usize, aretes: &[(usize, usize, f64, f64)]) -> Graphe {
    let noeuds = (0..n).map(|i| Noeud { x: i as i32, y: 0, etage: 0 }).collect();
    let aretes = aretes.iter().map(|&(a, b, longueur, cout)| Arete { a, b, longueur, cout }).collect();
    Graphe::new(noeuds, aretes).expect("graphe valide")
}

/// Un graphe où le coût de chaque arête est sa longueur.
fn graphe_l(n: usize, aretes: &[(usize, usize, f64)]) -> Graphe {
    let v: Vec<(usize, usize, f64, f64)> = aretes.iter().map(|&(a, b, l)| (a, b, l, l)).collect();
    graphe_lc(n, &v)
}

fn verifier(arbre: &Arbre, paires: &[(usize, usize)], tl: u64, md: u64, nom: &str) {
    assert_eq!(arbre.paires, paires, "{nom} : les paires gardées, dans l'ordre");
    assert_eq!(arbre.longueur.to_bits(), tl, "{nom} : TL_MST = {}", arbre.longueur);
    assert_eq!(arbre.distance_moyenne.to_bits(), md, "{nom} : MD_MST = {}", arbre.distance_moyenne);
}

/// juste : de petits graphes à la main (V-MST-01), valeurs du modèle Python et calculs à la main :
/// - **les longueurs, pas les coûts** : un triangle 0-1-2 de longueurs `3, 3, 10` (arêtes `(0,1)`, `(1,2)`, `(0,2)`) et
///   de coûts `5, 5, 1` : les distances sont `3, 3, 6` (`(0,2)` passe par 1), l'arbre garde `(0,1)` puis `(1,2)`,
///   `TL_MST = 6`, `MD_MST = (3 + 6 + 3) / 3 = 4` ; par les coûts il aurait gardé `(0,2)` ;
/// - **la clôture métrique** : des nourritures 0, 1, 2 reliées par des nœuds de passage 3 et 4 (`(0,3) = 2`, `(1,3) =
///   2`, `(1,4) = 1,5`, `(2,4) = 1,5`) : distances `d(0,1) = 4`, `d(1,2) = 3`, `d(0,2) = 7` ; les paires gardées dans
///   l'ordre de Kruskal, `[(1, 2), (0, 1)]` (la plus courte d'abord), `TL_MST = 7`, `MD_MST = (4 + 7 + 3) / 3 = 14 / 3`
///   ;
/// - **les rangs, pas les nœuds** : une chaîne `0-1-2-3` de longueurs `1, 2, 4` et des nourritures `[3, 0, 2]` : les
///   paires de rangs sont `(0,1)` (nœuds 3 et 0, distance 7), `(0,2)` (3 et 2 : 4), `(1,2)` (0 et 2 : 3) ; l'arbre
///   garde `[(1, 2), (0, 2)]`, `TL_MST = 7`, `MD_MST = (7 + 4 + 3) / 3` ;
/// - **les deux nourritures** : une seule paire ; `TL_MST` et `MD_MST` valent la distance, **comptée depuis le rang 0**
///   : une chaîne de longueurs `0,1, 0,2, 0,3` donne `0,6000000000000001` pour les nourritures `[0, 3]` (`(0,1 + 0,2) +
///   0,3`) et `0,6` pour `[3, 0]` (`(0,3 + 0,2) + 0,1`).
///
/// Ne couvre pas : les égalités (`les_egalites_au_plus_petit_indice`), l'ordre des sommes de `MD_MST`
/// (`l_ordre_des_sommes`), les erreurs.
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn petits_graphes_a_la_main() {
    let triangle = graphe_lc(3, &[(0, 1, 3.0, 5.0), (0, 2, 10.0, 1.0), (1, 2, 3.0, 5.0)]);
    let a = arbre_couvrant(&triangle, &[0, 1, 2]).unwrap();
    verifier(&a, &[(0, 1), (1, 2)], 6.0_f64.to_bits(), 4.0_f64.to_bits(), "triangle");
    let passages = graphe_l(5, &[(0, 3, 2.0), (1, 3, 2.0), (1, 4, 1.5), (2, 4, 1.5)]);
    let a = arbre_couvrant(&passages, &[0, 1, 2]).unwrap();
    verifier(&a, &[(1, 2), (0, 1)], 7.0_f64.to_bits(), 0x4012_AAAA_AAAA_AAAB, "passages (MD = 14/3)");
    assert_eq!(a.distance_moyenne, 14.0 / 3.0);
    let chaine = graphe_l(4, &[(0, 1, 1.0), (1, 2, 2.0), (2, 3, 4.0)]);
    let a = arbre_couvrant(&chaine, &[3, 0, 2]).unwrap();
    verifier(&a, &[(1, 2), (0, 2)], 7.0_f64.to_bits(), 0x4012_AAAA_AAAA_AAAB, "rangs (MD = 14/3)");
    let decimales = graphe_l(4, &[(0, 1, 0.1), (1, 2, 0.2), (2, 3, 0.3)]);
    let a = arbre_couvrant(&decimales, &[0, 3]).unwrap();
    verifier(&a, &[(0, 1)], 0x3FE3_3333_3333_3334, 0x3FE3_3333_3333_3334, "[0, 3]");
    assert_eq!(a.longueur, 0.1 + 0.2 + 0.3, "0,6000000000000001 : la somme de gauche à droite");
    let a = arbre_couvrant(&decimales, &[3, 0]).unwrap();
    verifier(&a, &[(0, 1)], 0x3FE3_3333_3333_3333, 0x3FE3_3333_3333_3333, "[3, 0]");
    assert_eq!(a.longueur, 0.3 + 0.2 + 0.1, "0,6 : la somme depuis le nœud 3");
    assert_ne!(0.1 + 0.2 + 0.3, 0.3 + 0.2 + 0.1);
}

/// juste : les égalités (V-MST-02). Les paires sont triées par `(distance, i, j)` et Kruskal garde la première qui ne
/// ferme pas de cycle : (1) **un graphe complet** de quatre nourritures, toutes les arêtes de longueur 1 : six paires à
/// distance 1, l'arbre est l'**étoile** de 0 : `[(0,1), (0,2), (0,3)]`, `TL_MST = 3`, `MD_MST = (1 + 1 + 1 + 2 + 2 + 2)
/// / 6 = 1,5` (un arbre qui ne garderait pas la plus petite paire d'abord en ferait un chemin, de `MD_MST = 10/6`) ;
/// (2) **un anneau** `0-1-2-3-0` de longueurs 1 : les distances sont `1, 1, 1, 1` (les quatre côtés) et `2, 2` (les
/// diagonales) ; Kruskal garde `(0,1)`, `(0,3)`, `(1,2)` et refuse `(2,3)` (le cycle) : `[(0,1), (0,3), (1,2)]`,
/// `TL_MST = 3`, tracé `3-0-1-2` : `MD_MST = (1 + 2 + 1 + 1 + 2 + 3) / 6 = 10 / 6` ; (3) **les mêmes quatre nœuds dans
/// l'ordre `[2, 0, 3, 1]`** : l'égalité se résout sur les **rangs** `(i, j)`, pas sur les indices de nœud : l'arbre est
/// `[(0,2), (0,3), (1,2)]`, tracé `3-0-2-1`, `MD_MST = (2 + 1 + 1 + 1 + 3 + 2) / 6`.
///
/// Ne couvre pas : les distances inégales (`petits_graphes_a_la_main`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn les_egalites_au_plus_petit_indice() {
    let k4 = graphe_l(4, &[(0, 1, 1.0), (0, 2, 1.0), (0, 3, 1.0), (1, 2, 1.0), (1, 3, 1.0), (2, 3, 1.0)]);
    let a = arbre_couvrant(&k4, &[0, 1, 2, 3]).unwrap();
    verifier(&a, &[(0, 1), (0, 2), (0, 3)], 3.0_f64.to_bits(), 1.5_f64.to_bits(), "K4");
    let anneau = graphe_l(4, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0), (0, 3, 1.0)]);
    let a = arbre_couvrant(&anneau, &[0, 1, 2, 3]).unwrap();
    verifier(&a, &[(0, 1), (0, 3), (1, 2)], 3.0_f64.to_bits(), 0x3FFA_AAAA_AAAA_AAAB, "anneau");
    assert_eq!(a.distance_moyenne, 10.0 / 6.0);
    // les mêmes nœuds donnés dans un autre ordre : les rangs changent, donc l'arbre aussi rangs 0, 1, 2, 3 = nœuds 2,
    // 0, 3, 1 : distances (rang 0-1) = d(2,0) = 2, (0-2) = d(2,3) = 1, (0-3) = d(2,1) = 1, (1-2) = d(0,3) = 1, (1-3) =
    // d(0,1) = 1, (2-3) = d(3,1) = 2
    let a = arbre_couvrant(&anneau, &[2, 0, 3, 1]).unwrap();
    // triées : (1,0,2) (1,0,3) (1,1,2) (1,1,3) (2,0,1) (2,2,3) : gardées (0,2) (0,3) (1,2) ; (1,3) ferme un cycle
    // (0-2-1 et 0-3 : le rang 1 est déjà relié au rang 3 par 1-2-0-3) tracé : 3-0-2-1 : (0,1) = 2, (0,2) = 1, (0,3) =
    // 1, (1,2) = 1, (1,3) = 3, (2,3) = 2 : 10 / 6
    verifier(&a, &[(0, 2), (0, 3), (1, 2)], 3.0_f64.to_bits(), (10.0_f64 / 6.0).to_bits(), "rangs permutés");
}

/// juste : l'ordre des sommes de `MD_MST` et de `TL_MST` (V-MST-03). Une chaîne `0-1-2-3` de longueurs `0,1`, `0,15`,
/// `0,2` et les quatre nœuds pour nourritures : `TL_MST = (0,1 + 0,15) + 0,2 = 0,45` (les paires gardées dans l'ordre
/// de Kruskal) ; `MD_MST` est la somme, pour `i < j`, de la distance dans l'arbre **comptée depuis `i`** (de nœud en
/// nœud : `d[v]` est `d[u]` augmenté de la distance de la paire), soit `0,1`, `(0,1 + 0,15)`, `((0,1 + 0,15) + 0,2)`,
/// `0,15`, `(0,15 + 0,2)`, `0,2` sommés de gauche à droite à partir de `0.0`, divisés par 6 : **`0,25` exactement**
/// (bits `3FD0000000000000`) ; depuis `j` on obtiendrait `0,24999999999999997` (cherché en Python : 30 triplets de
/// longueurs décimales sur 512 donnent deux bits différents).
///
/// Ne couvre pas : les distances dans le graphe (même règle, `etape_1r_mesures`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn l_ordre_des_sommes() {
    let g = graphe_l(4, &[(0, 1, 0.1), (1, 2, 0.15), (2, 3, 0.2)]);
    let a = arbre_couvrant(&g, &[0, 1, 2, 3]).unwrap();
    verifier(&a, &[(0, 1), (1, 2), (2, 3)], 0x3FDC_CCCC_CCCC_CCCD, 0x3FD0_0000_0000_0000, "chaîne décimale");
    assert_eq!(a.longueur, (0.1 + 0.15) + 0.2);
    // depuis le plus grand rang, la somme serait autre
    let depuis_j: f64 = {
        let seg = [0.1, 0.15, 0.2];
        let mut s = 0.0_f64;
        for i in 0..4 {
            for j in (i + 1)..4 {
                let mut d = 0.0_f64;
                for w in seg[i..j].iter().rev() {
                    d += w;
                }
                s += d;
            }
        }
        s / 6.0
    };
    assert_eq!(depuis_j.to_bits(), 0.24999999999999997_f64.to_bits());
    assert_ne!(depuis_j, a.distance_moyenne);
}

/// juste : les erreurs (V-MST-04), dans leur ordre : `Nourriture` (aucune nourriture, une seule, un nœud hors du
/// graphe, un nœud en double), puis `Coupee` (deux nourritures, une paire quelconque, qui ne sont pas reliées dans le
/// graphe, même si les autres le sont) ; `Nourriture` l'emporte sur `Coupee`. Un graphe non connexe dont toutes les
/// nourritures sont dans la même composante donne un arbre.
///
/// Ne couvre pas : les arbres de plus de deux nourritures dans un graphe non connexe.
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn erreurs() {
    let g = graphe_l(6, &[(0, 1, 1.0), (1, 2, 1.0), (3, 4, 2.0)]); // 0-1-2, 3-4, et 5 isolé
    for mauvaises in [vec![], vec![0], vec![0, 6], vec![9, 10], vec![0, 1, 0], vec![2, 2]] {
        assert_eq!(arbre_couvrant(&g, &mauvaises), Err(ErreurReseau::Nourriture), "{mauvaises:?}");
    }
    assert_eq!(arbre_couvrant(&g, &[0, 3]), Err(ErreurReseau::Coupee));
    assert_eq!(arbre_couvrant(&g, &[0, 1, 4]), Err(ErreurReseau::Coupee));
    assert_eq!(arbre_couvrant(&g, &[5, 0]), Err(ErreurReseau::Coupee), "un nœud isolé");
    assert_eq!(arbre_couvrant(&g, &[0, 3, 0]), Err(ErreurReseau::Nourriture), "Nourriture avant Coupee");
    // un graphe non connexe dont les nourritures sont dans la même composante : un arbre
    let a = arbre_couvrant(&g, &[0, 2]).unwrap();
    verifier(&a, &[(0, 1)], 2.0_f64.to_bits(), 2.0_f64.to_bits(), "dans une composante");
    let a = arbre_couvrant(&g, &[3, 4]).unwrap();
    verifier(&a, &[(0, 1)], 2.0_f64.to_bits(), 2.0_f64.to_bits(), "dans l'autre");
    // le graphe d'un seul nœud : pas de paire possible
    let seul = graphe_l(1, &[]);
    assert_eq!(arbre_couvrant(&seul, &[0]), Err(ErreurReseau::Nourriture));
    assert_eq!(arbre_couvrant(&seul, &[0, 0]), Err(ErreurReseau::Nourriture));
}

/// Les distances d'un Bellman-Ford naïf depuis `source` sur les **longueurs** : on relâche toutes les arêtes, dans les
/// deux sens, jusqu'à ce que plus rien ne bouge.
fn bellman_ford_longueurs(g: &Graphe, source: usize) -> Vec<f64> {
    let mut d = vec![f64::INFINITY; g.nb_noeuds()];
    d[source] = 0.0;
    loop {
        let mut change = false;
        for arete in g.aretes() {
            for (u, v) in [(arete.a, arete.b), (arete.b, arete.a)] {
                let candidat = d[u] + arete.longueur;
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

/// L'arbre naïf : toutes les paires `(distance depuis i, i, j)` triées, un union-find naïf, puis la distance dans
/// l'arbre par un parcours récursif depuis chaque `i`. Rend `(paires gardées, TL_MST, MD_MST, la distance de chaque
/// paire)`.
#[allow(clippy::type_complexity)]
fn arbre_naif(g: &Graphe, foods: &[usize]) -> (Vec<(usize, usize)>, f64, f64, Vec<Vec<f64>>) {
    let k = foods.len();
    let depuis: Vec<Vec<f64>> = foods.iter().map(|&v| bellman_ford_longueurs(g, v)).collect();
    let mut toutes = Vec::new();
    let mut dist = vec![vec![0.0; k]; k];
    for i in 0..k {
        for j in (i + 1)..k {
            let w = depuis[i][foods[j]];
            dist[i][j] = w;
            dist[j][i] = w;
            toutes.push((w, i, j));
        }
    }
    toutes.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut racine: Vec<usize> = (0..k).collect();
    fn trouver(racine: &mut [usize], x: usize) -> usize {
        if racine[x] == x { x } else { trouver(racine, racine[x]) }
    }
    let mut gardees = Vec::new();
    for &(w, i, j) in &toutes {
        let (ri, rj) = (trouver(&mut racine, i), trouver(&mut racine, j));
        if ri != rj {
            racine[ri] = rj;
            gardees.push((i, j, w));
        }
    }
    let mut tl = 0.0;
    for &(_, _, w) in &gardees {
        tl += w;
    }
    // la distance dans l'arbre depuis `depart` : récursion, `d[v] = d[u] + w`
    fn parcourir(gardees: &[(usize, usize, f64)], u: usize, d: f64, vu: &mut Vec<Option<f64>>) {
        vu[u] = Some(d);
        for &(i, j, w) in gardees {
            let v = if i == u {
                j
            } else if j == u {
                i
            } else {
                continue;
            };
            if vu[v].is_none() {
                parcourir(gardees, v, d + w, vu);
            }
        }
    }
    let mut somme = 0.0;
    for i in 0..k {
        let mut vu = vec![None; k];
        parcourir(&gardees, i, 0.0, &mut vu);
        for j in (i + 1)..k {
            somme += vu[j].expect("l'arbre couvre toutes les nourritures");
        }
    }
    (gardees.iter().map(|&(i, j, _)| (i, j)).collect(), tl, somme / (k * (k - 1) / 2) as f64, dist)
}

/// Un graphe connexe au hasard dont les longueurs diffèrent des coûts : `n` nœuds, un arbre plus des arêtes en plus ;
/// longueurs multiples de 0,5 (`entieres`, beaucoup d'égalités) ou réelles.
fn graphe_au_hasard_lc(alea: &mut Alea, n: usize, en_plus: usize, entieres: bool) -> Graphe {
    let mut paires: Vec<(usize, usize)> = (1..n).map(|i| (alea.borne(i as u32) as usize, i)).collect();
    for _ in 0..en_plus {
        let (a, b) = (alea.borne(n as u32) as usize, alea.borne(n as u32) as usize);
        let (a, b) = (a.min(b), a.max(b));
        if a != b && !paires.contains(&(a, b)) {
            paires.push((a, b));
        }
    }
    let aretes: Vec<(usize, usize, f64, f64)> = paires
        .iter()
        .map(|&(a, b)| {
            let longueur = if entieres { 0.5 * (1 + alea.borne(5)) as f64 } else { 0.5 + 2.0 * alea.suivant_f64() };
            (a, b, longueur, 0.3 + 3.0 * alea.suivant_f64())
        })
        .collect();
    graphe_lc(n, &aretes)
}

/// juste : `arbre_couvrant` contre l'arbre naïf du test (V-MST-05), sur 60 graphes connexes au hasard (de 5 à 14 nœuds,
/// de 3 à 6 nourritures dans un ordre quelconque, une moitié à longueurs multiples de `0,5` donc pleine d'égalités,
/// l'autre à longueurs réelles, **longueurs et coûts différents**) : les mêmes paires dans le même ordre, et `TL_MST`
/// et `MD_MST` aux bits. L'arbre naïf utilise un Bellman-Ford sur les longueurs (mêmes bits que Dijkstra : l'addition
/// est croissante), toutes les paires triées par `(distance, i, j)` et un union-find ; il ne partage rien avec le code
/// jugé. Sur ces 60 graphes, 19 ont deux distances égales entre nourritures (le test exige au moins 15).
///
/// Ne couvre pas : les graphes non connexes (`erreurs`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn contre_l_arbre_naif() {
    let mut avec_egalites = 0;
    for graine in 1..=60_u64 {
        let mut alea = Alea::new(graine);
        let n = 5 + (graine as usize % 10);
        let g = graphe_au_hasard_lc(&mut alea, n, n, graine % 2 == 0);
        let k = 3 + (graine as usize % 4).min(n - 3);
        // k nœuds distincts, dans un ordre tiré
        let mut tous: Vec<usize> = (0..n).collect();
        for i in (1..n).rev() {
            tous.swap(i, alea.borne(i as u32 + 1) as usize);
        }
        let foods = &tous[..k];
        let (paires, tl, md, dist) = arbre_naif(&g, foods);
        let a = arbre_couvrant(&g, foods).unwrap();
        assert_eq!(a.paires.len(), k - 1);
        assert_eq!(a.paires, paires, "graine {graine}, nourritures {foods:?}");
        assert_eq!(a.longueur.to_bits(), tl.to_bits(), "graine {graine} : TL_MST");
        assert_eq!(a.distance_moyenne.to_bits(), md.to_bits(), "graine {graine} : MD_MST");
        // y a-t-il eu des égalités de distance ?
        let mut d: Vec<u64> = Vec::new();
        for i in 0..k {
            for j in (i + 1)..k {
                d.push(dist[i][j].to_bits());
            }
        }
        d.sort();
        if d.windows(2).any(|w| w[0] == w[1]) {
            avec_egalites += 1;
        }
    }
    assert!(avec_egalites >= 15, "le test met les égalités en jeu : {avec_egalites} graphes sur 60");
}

/// juste : la minimalité (V-MST-06), contre une **énumération** de tous les arbres couvrants de la clôture (aucun
/// Kruskal) : pour 40 graphes au hasard de 4 à 5 nourritures, `TL_MST` est, à `1e-12` près (autre ordre de somme), le
/// plus petit total sur tous les sous-ensembles de `k - 1` paires qui forment un arbre. Mesuré sur ces 40 graphes :
/// écart au plus 8,9e-16 ; tolérance `1e-12`.
///
/// Ne couvre pas : les égalités (`les_egalites_au_plus_petit_indice`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn le_plus_petit_des_arbres() {
    let mut pire = 0.0_f64;
    for graine in 1..=40_u64 {
        let mut alea = Alea::new(1000 + graine);
        let n = 8;
        let g = graphe_au_hasard_lc(&mut alea, n, 6, false);
        let k = 4 + (graine as usize % 2);
        let foods: Vec<usize> = (0..k).map(|i| (i * 3 + 1) % n).collect();
        let (_, _, _, dist) = arbre_naif(&g, &foods);
        let paires: Vec<(usize, usize)> = (0..k).flat_map(|i| ((i + 1)..k).map(move |j| (i, j))).collect();
        let mut meilleur = f64::INFINITY;
        for masque in 0..(1_u32 << paires.len()) {
            if masque.count_ones() as usize != k - 1 {
                continue;
            }
            let choix: Vec<(usize, usize)> =
                paires.iter().enumerate().filter(|(b, _)| (masque >> b) & 1 == 1).map(|(_, p)| *p).collect();
            // un arbre : k - 1 paires sans cycle (union-find)
            let mut racine: Vec<usize> = (0..k).collect();
            let mut sans_cycle = true;
            for &(i, j) in &choix {
                let (mut a, mut b) = (i, j);
                while racine[a] != a {
                    a = racine[a];
                }
                while racine[b] != b {
                    b = racine[b];
                }
                if a == b {
                    sans_cycle = false;
                } else {
                    racine[a] = b;
                }
            }
            if sans_cycle {
                let total: f64 = choix.iter().map(|&(i, j)| dist[i][j]).sum();
                meilleur = meilleur.min(total);
            }
        }
        let a = arbre_couvrant(&g, &foods).unwrap();
        pire = pire.max((a.longueur - meilleur).abs());
        assert!((a.longueur - meilleur).abs() <= 1e-12, "graine {graine} : {} contre {meilleur}", a.longueur);
    }
    assert!(pire < 1e-12, "écart observé {pire:e}");
}
