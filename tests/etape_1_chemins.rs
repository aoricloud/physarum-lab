//! Étape 1 : `chemins`, les plus courts chemins (Dijkstra sur les coûts). Valeurs attendues :
//! `references/etape-1-valeurs.md`, identifiants `V-CHEM-xx`.
//!
//! Quatre sortes de preuves, toutes indépendantes du code jugé :
//! - de petits graphes dont les distances, les parents et les chemins se lisent à la main (entiers : tout est exact) ;
//! - la **procédure de la doc**, recopiée ici telle quelle et naïvement (`procedure_naive`, qui balaye toutes les
//!   arêtes au lieu des listes d'incidence) : les distances ET les parents doivent être les mêmes, aux bits, y compris
//!   à égalité ;
//! - un **Bellman-Ford naïf** (`common/graphes_au_hasard.rs`) : sur des graphes au hasard, les distances sont les mêmes
//!   aux bits. Ce qui est exact : l'addition de flottants est croissante et ne diminue jamais (`a + c >= a`), donc la
//!   plus petite somme faite de gauche à droite le long d'un chemin est la même pour Dijkstra et pour Bellman-Ford,
//!   même quand deux chemins ont des sommes qui diffèrent seulement par l'arrondi : on teste l'égalité des bits, sans
//!   tolérance. Les parents, eux, ne sont comparés qu'à la procédure de la doc ;
//! - les chemins rendus par `vers` : contigus, de la source à la cible, et dont la somme de gauche à droite redonne la
//!   distance aux bits.
// Les boucles à indices de ce fichier suivent les procédures de la doc ligne pour ligne.
#![allow(clippy::needless_range_loop)]

#[path = "common/graphes_au_hasard.rs"]
mod graphes_au_hasard;

use graphes_au_hasard::{bellman_ford, graphe_au_hasard, graphe_pondere};
use physarum_lab::alea::Alea;
use physarum_lab::chemins::{Chemins, plus_courts};
use physarum_lab::graphe::{Arete, Graphe, Grille, Noeud, Voisinage};

/// Un graphe aligné dont chaque arête `(a, b, longueur, coût)` a une longueur qui n'est PAS son coût.
fn graphe_a_longueurs(n: usize, aretes: &[(usize, usize, f64, f64)]) -> Graphe {
    let noeuds = (0..n).map(|i| Noeud { x: i as i32, y: 0, etage: 0 }).collect();
    let aretes = aretes.iter().map(|&(a, b, longueur, cout)| Arete { a, b, longueur, cout }).collect();
    Graphe::new(noeuds, aretes).expect("graphe valide")
}

fn bits(valeurs: &[f64]) -> Vec<u64> {
    valeurs.iter().map(|v| v.to_bits()).collect()
}

/// La procédure de la doc de `chemins`, recopiée naïvement : on balaye TOUTES les arêtes par indice croissant et on ne
/// garde que celles du nœud `u` (au lieu de `Graphe::incidentes`).
fn procedure_naive(g: &Graphe, source: usize) -> (Vec<f64>, Vec<Option<usize>>) {
    let n = g.nb_noeuds();
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
        for e in 0..g.nb_aretes() {
            let arete = g.aretes()[e];
            let v = if arete.a == u {
                arete.b
            } else if arete.b == u {
                arete.a
            } else {
                continue;
            };
            if !visite[v] {
                let candidat = distances[u] + arete.cout;
                if candidat < distances[v] {
                    distances[v] = candidat;
                    parents[v] = Some(e);
                }
            }
        }
    }
    (distances, parents)
}

/// Le chemin rendu par `vers` est contigu, part de la source, finit à `cible`, et la somme des coûts faite de gauche à
/// droite depuis `0.0` redonne `distances[cible]` aux bits.
fn verifier_chemin(g: &Graphe, c: &Chemins, cible: usize) {
    let chemin = c.vers(g, cible).unwrap_or_else(|| panic!("la cible {cible} devrait être atteinte"));
    let (mut ici, mut somme) = (c.source, 0.0_f64);
    for &e in &chemin {
        let arete = g.aretes()[e];
        assert!(arete.a == ici || arete.b == ici, "l'arête {e} ne part pas du nœud {ici}");
        ici = if arete.a == ici { arete.b } else { arete.a };
        somme += arete.cout;
    }
    assert_eq!(ici, cible, "le chemin finit à la cible");
    assert_eq!(somme.to_bits(), c.distances[cible].to_bits(), "la somme du chemin vers {cible}");
}

/// juste : de petits graphes lus à la main (V-CHEM-01). Les coûts sont des entiers : tout est exact. Graphe A, six
/// arêtes données dans le désordre, rangées `(0,1)=0, (0,2)=1, (1,2)=2, (1,3)=3, (2,3)=4, (3,4)=5` de coûts `4, 1, 2,
/// 1, 5, 3` : depuis 0, distances `0, 3, 1, 4, 7` (le nœud 1 par 0-2-1 qui coûte 3 et non par l'arête directe qui coûte
/// 4), parents `None, 2, 1, 3, 5`, chemin vers 4 : `[1, 2, 3, 5]`. Les LONGUEURS des arêtes sont toutes fausses exprès
/// (1 000 et plus) : seul le coût compte. Un chemin d'un seul nœud, une chaîne, et une arête dont la longueur contredit
/// le coût.
///
/// Ne couvre pas : les égalités (test suivant), les nœuds hors d'atteinte (`hors_d_atteinte`), les graphes au hasard.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn petits_graphes_a_la_main() {
    let g = graphe_a_longueurs(
        5,
        &[
            (3, 4, 1000.0, 3.0),
            (2, 1, 1001.0, 2.0),
            (0, 1, 1002.0, 4.0),
            (3, 2, 1003.0, 5.0),
            (1, 3, 1004.0, 1.0),
            (2, 0, 1005.0, 1.0),
        ],
    );
    assert_eq!(g.nb_aretes(), 6);
    let c = plus_courts(&g, 0);
    assert_eq!(c.source, 0);
    assert_eq!(c.distances, vec![0.0, 3.0, 1.0, 4.0, 7.0]);
    assert_eq!(c.distances[0].to_bits(), 0.0_f64.to_bits(), "la source est à +0.0");
    assert_eq!(c.parents, vec![None, Some(2), Some(1), Some(3), Some(5)]);
    assert_eq!(c.vers(&g, 4), Some(vec![1, 2, 3, 5]));
    assert_eq!(c.vers(&g, 3), Some(vec![1, 2, 3]));
    assert_eq!(c.vers(&g, 1), Some(vec![1, 2]));
    assert_eq!(c.vers(&g, 0), Some(vec![]), "la source elle-même : un chemin vide");
    // depuis un autre nœud, les arêtes se prennent dans l'autre sens : depuis 4, vers 0
    let c = plus_courts(&g, 4);
    assert_eq!(c.distances, vec![7.0, 4.0, 6.0, 3.0, 0.0]);
    assert_eq!(c.parents, vec![Some(1), Some(3), Some(2), Some(5), None]);
    assert_eq!(c.vers(&g, 0), Some(vec![5, 3, 2, 1]), "de la source 4 vers la cible 0, dans l'ordre du trajet");
    // un coût qui contredit la longueur : 0-1 coûte 10 (longueur 1), 0-2 et 2-1 coûtent 1 chacun (longueur 5)
    let g = graphe_a_longueurs(3, &[(0, 1, 1.0, 10.0), (0, 2, 5.0, 1.0), (1, 2, 5.0, 1.0)]);
    let c = plus_courts(&g, 0);
    assert_eq!(c.distances, vec![0.0, 2.0, 1.0]);
    assert_eq!(c.vers(&g, 1), Some(vec![1, 2]));
    // un seul nœud
    let seul = graphe_pondere(1, &[]);
    let c = plus_courts(&seul, 0);
    assert_eq!((c.distances.clone(), c.parents.clone()), (vec![0.0], vec![None]));
    assert_eq!(c.vers(&seul, 0), Some(vec![]));
}

/// juste : les égalités se règlent comme la doc le dit (V-CHEM-02), sur deux graphes lus à la main. Le losange `0-1,
/// 0-2, 1-3, 2-3` (coûts 1) a deux plus courts chemins vers 3, de longueur 2 : la procédure visite 1 avant 2 (plus
/// petit indice à distance égale), le candidat de l'arête `2-3` est égal et ne remplace pas : `parents[3] = Some(2)`
/// (l'arête `1-3`) et `vers(3) = [0, 2]`. Un `<=` ou un choix du plus grand indice donneraient l'arête 3. Le graphe B
/// (`0-1:1, 0-3:2, 1-2:1, 2-4:1, 3-4:1`) range les nœuds 3 et 2 à la même distance 2 mais découvre 3 AVANT 2 : le plus
/// petit indice (2) est visité d'abord, donc `parents[4] = Some(3)` (l'arête `2-4`), `vers(4) = [0, 2, 3]`, et non
/// l'arête `3-4` d'un ordre de découverte.
///
/// Ne couvre pas : les égalités sur graphes au hasard (voir `contre_la_procedure_naive`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn egalites_plus_petit_indice() {
    let g = graphe_pondere(4, &[(0, 1, 1.0), (0, 2, 1.0), (1, 3, 1.0), (2, 3, 1.0)]);
    let c = plus_courts(&g, 0);
    assert_eq!(c.distances, vec![0.0, 1.0, 1.0, 2.0]);
    assert_eq!(c.parents, vec![None, Some(0), Some(1), Some(2)]);
    assert_eq!(c.vers(&g, 3), Some(vec![0, 2]));
    let g = graphe_pondere(5, &[(3, 4, 1.0), (1, 2, 1.0), (0, 3, 2.0), (2, 4, 1.0), (0, 1, 1.0)]);
    let c = plus_courts(&g, 0);
    assert_eq!(c.distances, vec![0.0, 1.0, 2.0, 2.0, 3.0]);
    assert_eq!(c.parents, vec![None, Some(0), Some(2), Some(1), Some(3)]);
    assert_eq!(c.vers(&g, 4), Some(vec![0, 2, 3]));
    // depuis 4 : 2 et 3 sont à 1 ; 2 (le plus petit) est visité d'abord et offre 1 (arête 2) ; puis 3 offre 0 à 3
    // (arête 1) ; puis 1, à 2, offre 0 à 3 aussi (arête 0) : un candidat ÉGAL, qui ne remplace pas, donc `parents[0] =
    // Some(1)`.
    let c = plus_courts(&g, 4);
    assert_eq!(c.distances, vec![3.0, 2.0, 1.0, 1.0, 0.0]);
    assert_eq!(c.parents, vec![Some(1), Some(2), Some(3), Some(4), None]);
}

/// juste : un nœud hors d'atteinte garde `+inf` et `None`, et `vers` rend `None` (V-CHEM-03). Un graphe en trois
/// morceaux (`0-1-2`, `3-4`, `5` seul) ; depuis un nœud de chaque morceau ; deux coûts `1e308` qui ne donnent que
/// `+inf` en somme (`1e308 + 1e308` déborde, et `+inf` n'est pas `<` à `+inf` : le nœud 2 reste hors d'atteinte) ; un
/// graphe d'un seul nœud.
///
/// Ne couvre pas : la panique de `vers` hors du graphe (voir `paniques`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn hors_d_atteinte() {
    let inf = f64::INFINITY;
    let g = graphe_pondere(6, &[(0, 1, 1.0), (1, 2, 2.0), (3, 4, 5.0)]);
    let c = plus_courts(&g, 0);
    assert_eq!(c.distances, vec![0.0, 1.0, 3.0, inf, inf, inf]);
    assert_eq!(c.parents, vec![None, Some(0), Some(1), None, None, None]);
    assert_eq!(c.vers(&g, 2), Some(vec![0, 1]));
    for cible in 3..6 {
        assert_eq!(c.vers(&g, cible), None, "cible {cible}");
    }
    let c = plus_courts(&g, 4);
    assert_eq!(c.distances, vec![inf, inf, inf, 5.0, 0.0, inf]);
    assert_eq!(c.parents, vec![None, None, None, Some(2), None, None]);
    assert_eq!(c.vers(&g, 3), Some(vec![2]));
    assert_eq!(c.vers(&g, 0), None);
    let c = plus_courts(&g, 5);
    assert_eq!(c.distances, vec![inf, inf, inf, inf, inf, 0.0]);
    assert_eq!(c.vers(&g, 5), Some(vec![]));
    assert_eq!(c.vers(&g, 1), None);
    // le débordement : 1e308 + 1e308 vaut +inf
    assert!((1e308_f64 + 1e308).is_infinite());
    let g = graphe_pondere(3, &[(0, 1, 1e308), (1, 2, 1e308)]);
    let c = plus_courts(&g, 0);
    assert_eq!(c.distances, vec![0.0, 1e308, inf]);
    assert_eq!(c.parents, vec![None, Some(0), None]);
    assert_eq!(c.vers(&g, 1), Some(vec![0]));
    assert_eq!(c.vers(&g, 2), None);
}

/// juste : la somme se fait de GAUCHE À DROITE depuis la source (V-CHEM-04) : sur la chaîne `0-1-2-3` de coûts `1e16,
/// 1, 1`, `1e16 + 1` est à mi-chemin de deux flottants et s'arrondit au pair (`1e16`), deux fois : `distances[3]` vaut
/// exactement `1e16`, alors qu'une somme faite depuis la cible donnerait `1e16 + 2` (`1 + 1 = 2` est exact). Calcul
/// d'arithmétique IEEE 754, vérifié avec Python (`0.0 + 1e16 + 1 + 1 == 1e16`, `1e16 + (1 + 1) == 1e16 + 2`).
///
/// Ne couvre pas : l'ordre des sommes dans un graphe plus riche (voir `contre_bellman_ford`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn somme_de_gauche_a_droite() {
    let g = graphe_pondere(4, &[(0, 1, 1e16), (1, 2, 1.0), (2, 3, 1.0)]);
    let c = plus_courts(&g, 0);
    assert_eq!(c.distances[3].to_bits(), 1e16_f64.to_bits());
    assert_ne!(c.distances[3].to_bits(), (1e16_f64 + 2.0).to_bits());
    assert_eq!(c.distances[2].to_bits(), 1e16_f64.to_bits());
    // depuis l'autre bout, le chemin se fait dans l'autre sens : 1 + 1 = 2, puis 2 + 1e16 = 1e16 + 2
    let c = plus_courts(&g, 3);
    assert_eq!(c.distances[0].to_bits(), (1e16_f64 + 2.0).to_bits());
}

/// juste : deux étages (V-CHEM-05), en voisinage 4, 3 colonnes, 2 lignes, un lien vertical de coût 0,25 (une puissance
/// de 2 : tout est exact). Le nœud `(x, y, e)` est à la distance `x + y + 0,25 e` de `(0, 0, 0)` ; le lien vertical
/// `(0,0,0)-(0,0,1)` est le chemin vers `(0, 0, 1)`. Avec un lien à 3 (cher), `(0, 0, 1)` est à 3 et `(2, 1, 1)` à 6 :
/// on monte au plus court. Chaque chemin rendu est contigu et redonne sa distance aux bits (`verifier_chemin`).
///
/// Ne couvre pas : un étage dont les arêtes sont moins chères (voir le test de convergence à deux étages).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn grille_a_deux_etages() {
    let grille = |vertical: f64| Grille {
        largeur: 3,
        hauteur: 2,
        etages: 2,
        voisinage: Voisinage::Quatre,
        longueur_verticale: 1.0,
        cout_vertical: vertical,
    };
    let g = grille(0.25).graphe().unwrap();
    let source = grille(0.25).indice(0, 0, 0);
    let c = plus_courts(&g, source);
    for y in 0..2_u32 {
        for x in 0..3_u32 {
            for e in 0..2_u32 {
                let i = grille(0.25).indice(x, y, e);
                assert_eq!(c.distances[i], f64::from(x + y) + 0.25 * f64::from(e), "({x}, {y}, {e})");
                verifier_chemin(&g, &c, i);
            }
        }
    }
    let haut = grille(0.25).indice(0, 0, 1);
    let lien = g.arete_entre(source, haut).expect("le lien vertical");
    assert_eq!(c.vers(&g, haut), Some(vec![lien]));
    assert_eq!(c.parents[haut], Some(lien));
    // un lien cher : on y monte quand même une fois, au plus tard
    let g = grille(3.0).graphe().unwrap();
    let c = plus_courts(&g, source);
    let (bas, haut) = (grille(3.0).indice(2, 1, 0), grille(3.0).indice(2, 1, 1));
    assert_eq!((c.distances[bas], c.distances[haut]), (3.0, 6.0));
    assert_eq!(c.vers(&g, haut).unwrap().len(), 4, "trois pas sur l'étage 0, puis un lien");
    verifier_chemin(&g, &c, haut);
}

/// juste : contre un Bellman-Ford naïf écrit dans le test, sur 60 graphes au hasard (V-CHEM-06), connexes ou non : de 2
/// à 40 nœuds, des coûts réels dans `[0,001 ; 100[` (des sommes qui ne tombent pas juste), 3 sources par graphe. Les
/// distances sont les mêmes aux BITS (voir l'en-tête du fichier : l'addition de flottants est croissante, donc le
/// minimum des sommes de gauche à droite est le même pour les deux méthodes) ; chaque parent `e` d'un nœud `v` part
/// d'un nœud `u` avec `distances[v] == distances[u] + cout(e)` aux bits ; les parents ne forment pas de cycle ; `vers`
/// rend, pour tout nœud atteint, un chemin contigu dont la somme de gauche à droite redonne la distance.
///
/// Ne couvre pas : les égalités (voir `contre_la_procedure_naive`) ; les graphes de plus de 40 nœuds.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn contre_bellman_ford() {
    let mut alea = Alea::new(20_261_109);
    let (mut atteints, mut hors, mut graphes) = (0_usize, 0_usize, 0_usize);
    for essai in 0..60_u32 {
        let n = 2 + alea.borne(39) as usize;
        let g = if essai % 3 == 0 {
            // des arêtes au hasard, sans arbre : le graphe est le plus souvent en plusieurs morceaux
            let mut aretes: Vec<(usize, usize, f64)> = Vec::new();
            for _ in 0..(n + alea.borne(n as u32) as usize) {
                let (a, b) = (alea.borne(n as u32) as usize, alea.borne(n as u32) as usize);
                let (a, b) = (a.min(b), a.max(b));
                if a != b && !aretes.iter().any(|&(x, y, _)| (x, y) == (a, b)) {
                    aretes.push((a, b, 0.001 + 99.999 * alea.suivant_f64()));
                }
            }
            graphe_pondere(n, &aretes)
        } else {
            let en_plus = n + alea.borne(n as u32) as usize;
            graphe_au_hasard(&mut alea, n, en_plus, 0.001, 99.999)
        };
        graphes += 1;
        for _ in 0..3 {
            let source = alea.borne(n as u32) as usize;
            let c = plus_courts(&g, source);
            let attendu = bellman_ford(&g, source, None);
            assert_eq!(c.distances.len(), n, "une distance par nœud");
            assert_eq!(c.parents.len(), n, "un parent par nœud");
            assert_eq!(bits(&c.distances), bits(&attendu), "graphe {essai}, source {source}");
            for v in 0..n {
                match c.parents[v] {
                    None => assert!(v == source || attendu[v].is_infinite(), "{v} atteint sans parent"),
                    Some(e) => {
                        let arete = g.aretes()[e];
                        let u = if arete.a == v { arete.b } else { arete.a };
                        assert!(arete.a == v || arete.b == v, "le parent {e} de {v} ne touche pas {v}");
                        assert_eq!(c.distances[v].to_bits(), (c.distances[u] + arete.cout).to_bits(), "parent de {v}");
                    }
                }
                if attendu[v].is_finite() {
                    verifier_chemin(&g, &c, v);
                    atteints += 1;
                } else {
                    assert_eq!(c.vers(&g, v), None);
                    hors += 1;
                }
            }
        }
    }
    // le test a bien vu des nœuds atteints et des nœuds hors d'atteinte
    assert_eq!(graphes, 60);
    assert!(atteints > 1000 && hors > 50, "atteints {atteints}, hors d'atteinte {hors}");
}

/// juste : la procédure de la doc recopiée dans le test (`procedure_naive`) donne les mêmes distances ET les mêmes
/// parents, aux bits, que `plus_courts`, y compris à égalité (V-CHEM-07) : sur 40 graphes à coûts entiers dans `{1, 2,
/// 3}` (beaucoup d'égalités), sur des grilles en voisinage 8 (coûts 1 et `√2`, des sommes qui diffèrent d'un ulp) à un
/// et deux étages, et depuis chaque nœud d'une petite grille. Un `<=` pour `<`, un plus grand indice à égalité, un
/// parcours des arêtes dans un autre ordre, ou une relaxation vers un voisin déjà visité changent un parent quelque
/// part.
///
/// Ne couvre pas : les graphes de plus de 100 nœuds.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn contre_la_procedure_naive() {
    let mut alea = Alea::new(5_551_212);
    let mut egalites = 0_usize;
    for essai in 0..40 {
        let n = 3 + alea.borne(38) as usize;
        let g = graphe_au_hasard(&mut alea, n, 2 * n, 1.0, 0.0);
        // les coûts entiers dans {1, 2, 3} : on reconstruit le graphe avec ces coûts
        let aretes: Vec<(usize, usize, f64)> =
            g.aretes().iter().map(|a| (a.a, a.b, f64::from(1 + alea.borne(3)))).collect();
        let g = graphe_pondere(n, &aretes);
        let source = alea.borne(n as u32) as usize;
        let c = plus_courts(&g, source);
        let (distances, parents) = procedure_naive(&g, source);
        assert_eq!(bits(&c.distances), bits(&distances), "graphe {essai}");
        assert_eq!(c.parents, parents, "graphe {essai}");
        // combien de nœuds ont un second chemin de même longueur : le test voit des égalités
        for v in 0..n {
            if let Some(e) = c.parents[v] {
                let arete = g.aretes()[e];
                let u = if arete.a == v { arete.b } else { arete.a };
                let autres = g.incidentes(v).iter().filter(|&&f| f != e).any(|&f| {
                    let a = g.aretes()[f];
                    let w = if a.a == v { a.b } else { a.a };
                    c.distances[w] + a.cout == c.distances[v] && w != u
                });
                egalites += usize::from(autres);
            }
        }
    }
    assert!(egalites > 40, "le test doit voir des égalités : {egalites}");
    for (largeur, hauteur, etages, vertical) in [(5, 5, 1, 1.0), (4, 3, 2, 0.5), (3, 3, 2, 1.0)] {
        let grille = Grille {
            largeur,
            hauteur,
            etages,
            voisinage: Voisinage::Huit,
            longueur_verticale: 1.0,
            cout_vertical: vertical,
        };
        let g = grille.graphe().unwrap();
        for source in 0..g.nb_noeuds() {
            let c = plus_courts(&g, source);
            let (distances, parents) = procedure_naive(&g, source);
            assert_eq!(bits(&c.distances), bits(&distances), "grille {largeur}x{hauteur}x{etages}, source {source}");
            assert_eq!(c.parents, parents, "grille {largeur}x{hauteur}x{etages}, source {source}");
            assert_eq!(bits(&c.distances), bits(&bellman_ford(&g, source, None)), "contre Bellman-Ford");
        }
    }
}

/// juste : `plus_courts` panique si la source n'est pas un nœud, `vers` si la cible n'est pas un nœud (V-CHEM-08) ; le
/// cas positif d'abord (le dernier nœud est une source et une cible valides).
///
/// Ne couvre pas : le texte des paniques.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn paniques() {
    let g = graphe_pondere(3, &[(0, 1, 1.0), (1, 2, 1.0)]);
    let c = plus_courts(&g, 2);
    assert_eq!(c.vers(&g, 2), Some(vec![]));
    assert_eq!(c.vers(&g, 0), Some(vec![1, 0]));
    assert!(std::panic::catch_unwind(|| plus_courts(&g, 3)).is_err(), "source hors du graphe");
    assert!(std::panic::catch_unwind(|| plus_courts(&g, usize::MAX)).is_err(), "source énorme");
    assert!(std::panic::catch_unwind(|| c.vers(&g, 3)).is_err(), "cible hors du graphe");
}
