//! Étape 2, partie 2T : l'état de l'agent, `souvenir::Agent` (`zero`, `new`, `concentrations`, `masse`). Valeurs
//! attendues : `references/etape-2t-valeurs.md`, identifiants `V-AGT-xx`.
//!
//! La source est la doc de `souvenir` (« L'état »). Tout y est exact : des constructeurs, des erreurs dans un ordre
//! fixé, une somme de gauche à droite dont l'ordre se voit aux bits.

#[path = "common/transport_2t.rs"]
mod transport_2t;

use physarum_lab::graphe::{Graphe, Noeud};
use physarum_lab::souvenir::{Agent, ErreurAgent};
use transport_2t::{bits, chaine};

/// juste : `Agent::zero` (V-AGT-01) rend une concentration `+0.0` par nœud, dans l'ordre des indices ; sa masse est
/// `+0.0`. Sur un graphe d'un seul nœud sans arête : un nombre. Sur la chaîne de quatre nœuds : quatre.
///
/// Ne couvre pas : les pas de transport (`etape_2t_recette.rs`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn zero_a_une_concentration_nulle_par_noeud() {
    let seul = Graphe::new(vec![Noeud { x: 0, y: 0, etage: 0 }], vec![]).unwrap();
    let agent = Agent::zero(&seul);
    assert_eq!(bits(agent.concentrations()), vec![0_u64], "un nœud, un zéro positif");
    assert_eq!(agent.masse().to_bits(), 0.0_f64.to_bits());
    let g = chaine(&[1.0, 1.0, 1.0]);
    let agent = Agent::zero(&g);
    assert_eq!(agent.concentrations().len(), 4, "une concentration par nœud");
    assert_eq!(bits(agent.concentrations()), vec![0_u64; 4]);
    assert_eq!(agent.masse().to_bits(), 0.0_f64.to_bits());
}

/// juste : `Agent::new` (V-AGT-02) garde les valeurs données, bit pour bit et dans l'ordre (y compris `-0.0`, accepté
/// tel quel, et un très petit nombre positif), et `concentrations()` les rend. Deux agents faits des mêmes valeurs sont
/// égaux (`PartialEq`), deux agents de valeurs différentes ne le sont pas.
///
/// Ne couvre pas : les erreurs (`les_erreurs_de_new`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn new_garde_les_valeurs_donnees() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    let valeurs = vec![0.25, -0.0, 1e-300, 3.5];
    let agent = Agent::new(&g, valeurs.clone()).unwrap();
    assert_eq!(bits(agent.concentrations()), bits(&valeurs));
    assert_eq!(agent.concentrations()[1].to_bits(), 0x8000_0000_0000_0000, "-0.0 est gardé tel quel");
    let autre = Agent::new(&g, valeurs.clone()).unwrap();
    assert_eq!(agent, autre);
    let mut differentes = valeurs.clone();
    differentes[3] = 3.75;
    assert_ne!(agent, Agent::new(&g, differentes).unwrap());
    assert_ne!(agent, Agent::zero(&g));
}

/// juste : les erreurs de `Agent::new` (V-AGT-04), dans l'ordre de la doc : `Taille` d'abord (même si une valeur est
/// fautive), puis `Concentration(i)` avec **le plus petit** indice fautif. Sont fautifs : NaN, `+inf`, `-inf`, un
/// négatif, même minuscule (`-1e-300`) ; ne le sont pas : `0.0`, `-0.0`, `f64::MAX`.
///
/// Ne couvre pas : les erreurs du pas (`etape_2t_erreurs.rs`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn les_erreurs_de_new() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    assert_eq!(Agent::new(&g, vec![0.0; 3]), Err(ErreurAgent::Taille), "trois pour quatre nœuds");
    assert_eq!(Agent::new(&g, vec![0.0; 5]), Err(ErreurAgent::Taille), "cinq pour quatre nœuds");
    assert_eq!(Agent::new(&g, vec![]), Err(ErreurAgent::Taille), "aucune");
    assert_eq!(Agent::new(&g, vec![f64::NAN, 0.0, 0.0]), Err(ErreurAgent::Taille), "la taille passe avant la valeur");
    for mauvaise in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.0, -1e-300] {
        for rang in 0..4 {
            let mut v = vec![1.0; 4];
            v[rang] = mauvaise;
            assert_eq!(Agent::new(&g, v), Err(ErreurAgent::Concentration(rang)), "{mauvaise} au rang {rang}");
        }
    }
    let v = vec![0.0, f64::NAN, 1.0, -2.0];
    assert_eq!(Agent::new(&g, v), Err(ErreurAgent::Concentration(1)), "le plus petit indice fautif");
    let v = vec![-0.0, 0.0, f64::MAX, 1.0];
    assert!(Agent::new(&g, v).is_ok(), "-0.0, 0.0 et f64::MAX sont acceptés");
}

/// juste : `Agent::masse` (V-AGT-03) est la somme des concentrations **de gauche à droite à partir de `0.0`** (une
/// somme de flottants dépend de son ordre). Trois cas où l'ordre se voit : `[0.1, 0.2, 0.3]` donne `0.6000000000000001`
/// (et non `0.6`, que donnerait `0.1 + (0.2 + 0.3)`) ; `[1.0, 1e-16, 1e-16]` donne exactement `1.0` (chaque `1e-16` est
/// perdu devant `1.0` ; une somme des petits d'abord donnerait `1.0000000000000002`) ; et `[1e-16, 1e-16, 1.0]` donne
/// `1.0000000000000002` (les petits d'abord s'additionnent). Un seul nœud : sa concentration ; zéro nœud n'existe pas
/// (un graphe a un nœud au moins).
///
/// Ne couvre pas : la masse au long d'un transport (`etape_2t_masse.rs`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn la_masse_est_la_somme_de_gauche_a_droite() {
    let g = chaine(&[1.0, 1.0]);
    let a = Agent::new(&g, vec![0.1, 0.2, 0.3]).unwrap();
    assert_eq!(a.masse().to_bits(), 0x3FE3_3333_3333_3334, "0.6000000000000001");
    assert_ne!(a.masse(), 0.6);
    let a = Agent::new(&g, vec![1.0, 1e-16, 1e-16]).unwrap();
    assert_eq!(a.masse().to_bits(), 1.0_f64.to_bits(), "les petits perdus devant 1.0");
    let a = Agent::new(&g, vec![1e-16, 1e-16, 1.0]).unwrap();
    assert_eq!(a.masse().to_bits(), 0x3FF0_0000_0000_0001, "1.0000000000000002");
    let seul = Graphe::new(vec![Noeud { x: 0, y: 0, etage: 0 }], vec![]).unwrap();
    let a = Agent::new(&seul, vec![0.7]).unwrap();
    assert_eq!(a.masse().to_bits(), 0.7_f64.to_bits());
}
