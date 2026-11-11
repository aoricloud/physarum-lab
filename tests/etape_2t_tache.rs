//! Étape 2, partie 2T : « la tache suit les tubes » : sur une grille où un réseau a convergé, l'agent lâché près de la
//! source est porté par les flux, donc il reste sur les nœuds des tubes vifs ; sur des tubes uniformes il s'étale.
//! Valeurs attendues : `references/etape-2t-valeurs.md`, identifiants `V-TAC-xx`.
//!
//! La source est le mécanisme de Kramar et Alim (PNAS 2021) : l'agent libéré au stimulus est **advecté par les flux
//! cytoplasmiques** (aucune valeur de l'article n'est visée : leur modèle est un seul tube, voir `souvenir`). Les
//! nombres sont ceux de **notre** modèle, **mesurés** sur 40 graines autres que celles du test (`mes_2t_b.rs`), avec la
//! marge dite. Un réseau convergé est presque indépendant de la graine (le plus court chemin y est unique) : la marge
//! vient donc surtout de la géométrie, trois géométries sont jouées.

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Graphe, Grille, Voisinage};
use physarum_lab::mesures::tubes_vifs;
use physarum_lab::souvenir::{Agent, Transport};
use physarum_lab::tubes::Tubes;

/// Une géométrie : source, puits, lâcher sur le chemin, lâcher hors du chemin (des `(x, y)` de l'étage 0), et le nombre
/// de tubes vifs attendu (la longueur du chemin droit).
struct Geometrie {
    source: (u32, u32),
    puits: (u32, u32),
    lacher_sur: (u32, u32),
    lacher_hors: (u32, u32),
    vifs: usize,
}

const GEOMETRIES: [Geometrie; 3] = [
    Geometrie { source: (1, 4), puits: (10, 4), lacher_sur: (2, 4), lacher_hors: (2, 5), vifs: 9 },
    Geometrie { source: (2, 2), puits: (9, 2), lacher_sur: (3, 2), lacher_hors: (3, 3), vifs: 7 },
    Geometrie { source: (1, 6), puits: (8, 6), lacher_sur: (2, 6), lacher_hors: (2, 5), vifs: 7 },
];

fn grille() -> Grille {
    Grille {
        largeur: 12,
        hauteur: 9,
        etages: 1,
        voisinage: Voisinage::Huit,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    }
}

/// Le réseau convergé : 300 pas des tubes de `h = 0.1` (source, puits, débit 1) de conductivités de départ
/// `Tubes::au_hasard` de la graine ; rend les conductivités et le flux du dernier pas.
fn converge(g: &Graphe, source: usize, puits: usize, graine: u64) -> (Vec<f64>, Vec<f64>) {
    let mut alea = Alea::new(graine);
    let mut tubes = Tubes::au_hasard(g, &mut alea);
    let mut q = vec![];
    for _ in 0..300 {
        q = tubes.avancer(g, source, puits, 1.0, 0.1).unwrap().flux;
    }
    (tubes.conductivites().to_vec(), q)
}

/// La part de la masse sur les nœuds des tubes vifs (`conductivité >= 0.5`) après 30 pas de `0.1`, pour une libération
/// de 1 au nœud `lacher`, `κ = 0.01`, `k = 0`.
fn part_sur_les_vifs(g: &Graphe, d: &[f64], q: &[f64], lacher: usize) -> f64 {
    let mut sur_vif = vec![false; g.nb_noeuds()];
    for (e, a) in g.aretes().iter().enumerate() {
        if d[e] >= 0.5 {
            sur_vif[a.a] = true;
            sur_vif[a.b] = true;
        }
    }
    let mut c0 = vec![0.0; g.nb_noeuds()];
    c0[lacher] = 1.0;
    let mut agent = Agent::new(g, c0).unwrap();
    let tr = Transport { diffusion: 0.01, perte: 0.0 };
    for _ in 0..30 {
        agent.avancer(g, q, &tr, &[], 0.1).unwrap();
    }
    let mut s = 0.0;
    for (i, &c) in agent.concentrations().iter().enumerate() {
        if sur_vif[i] {
            s += c;
        }
    }
    s / agent.masse()
}

/// juste : **la tache suit les tubes vifs** (V-TAC-01). Pour chacune des trois géométries (source et puits à sept ou
/// neuf cases sur une même ligne : le plus court chemin y est la ligne droite, unique, loin d'un zigzag de 41 % plus
/// long), six graines : après 300 pas des tubes, exactement `vifs` tubes sont vifs (9, 7, 7 : le chemin), `tubes_vifs`
/// d'accord avec le décompte du test ; un agent libéré de masse 1 près de la source (sur le chemin) avance 30 pas de
/// `0.1` (`κ = 0.01`, `k = 0`) avec le flux du réseau convergé, puis avec le flux de **tubes uniformes** (conductivité
/// 1, un pas des tubes) : la part de la masse sur les nœuds des tubes vifs du réseau convergé est
/// - `>= 0.80` avec les tubes convergés (mesuré sur 40 graines : `0.8444` au pire, `0.85` en moyenne pour la géométrie
///   1 ; `0.8588` au pire pour la 2 ; `0.8577` pour la 3),
/// - `<= 0.62` avec les tubes uniformes (mesuré : `0.5948`, `0.5944`, `0.5846` : les tubes uniformes ne dépendent pas
///   de la graine),
/// - et la différence est `>= 0.22` (mesuré au pire : `0.2496`, `0.2644`, `0.2731`) ; sa moyenne sur les graines est
///   `>= 0.83 - 0.62`. Sans diffusion (`κ = 0`) l'écart est plus grand encore (`0.97` contre `0.68`, mesuré). Le flux
///   convergé est donc bien ce qui conduit l'agent le long des tubes vifs. Les marges : environ `0.04` sur la part
///   convergée, `0.025` sur la part uniforme, `0.03` sur la différence.
///
/// Ne couvre pas : l'effet de l'agent sur les tubes (partie 2M) ; des réseaux à égalité de chemins (la grille en
/// voisinage 8 en a ; ils ne sont pas testés).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn la_tache_suit_les_tubes_vifs() {
    let grille = grille();
    let g = grille.graphe().unwrap();
    for (rang, geo) in GEOMETRIES.iter().enumerate() {
        let source = grille.indice(geo.source.0, geo.source.1, 0);
        let puits = grille.indice(geo.puits.0, geo.puits.1, 0);
        let lacher = grille.indice(geo.lacher_sur.0, geo.lacher_sur.1, 0);
        let qu = Tubes::uniformes(&g, 1.0).unwrap().avancer(&g, source, puits, 1.0, 0.1).unwrap().flux;
        let mut somme_ecart = 0.0;
        for graine in 1..=6_u64 {
            let (d, q) = converge(&g, source, puits, graine);
            let mon_compte = d.iter().filter(|&&x| x >= 0.5).count();
            assert_eq!(tubes_vifs(&g, &d, 0.5).0, mon_compte);
            assert_eq!(mon_compte, geo.vifs, "géométrie {rang}, graine {graine} : tubes vifs");
            let conv = part_sur_les_vifs(&g, &d, &q, lacher);
            let unif = part_sur_les_vifs(&g, &d, &qu, lacher);
            assert!(conv >= 0.80, "géométrie {rang}, graine {graine} : part convergée {conv}");
            assert!(unif <= 0.62, "géométrie {rang}, graine {graine} : part uniforme {unif}");
            assert!(conv - unif >= 0.22, "géométrie {rang}, graine {graine} : écart {}", conv - unif);
            somme_ecart += conv - unif;
        }
        assert!(somme_ecart / 6.0 >= 0.83 - 0.62, "géométrie {rang} : écart moyen {}", somme_ecart / 6.0);
    }
}

/// juste : **la tache lâchée hors des tubes vifs n'est pas emportée** (V-TAC-02), la réciproque : le même réseau, le
/// même transport, mais l'agent est libéré sur un nœud voisin du chemin et **hors** de ses tubes vifs : dans le réseau
/// convergé les tubes qui y arrivent ont perdu leur flux (`D` au plancher), l'agent ne bouge presque pas et n'atteint
/// que peu les nœuds des tubes vifs (la part ne vient que de la diffusion `κ = 0.01`) : la part sur les tubes vifs
/// après 30 pas est `<= 0.08` (mesuré sur 40 graines : `0.0614` au pire) et **inférieure** à celle des tubes uniformes
/// (mesuré : `0.118`, `0.111`, `0.107` contre `0.061`, `0.061`, `0.061` : un écart d'au moins `0.045`). Marge de `0.02`
/// sur le premier seuil ; le second est une inégalité stricte, de marge `0.045`.
///
/// Ne couvre pas : un lâcher sur un nœud plus éloigné des tubes (mesuré : la part tombe à `0.002`, sans test).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn la_tache_lachee_hors_des_tubes_reste_dehors() {
    let grille = grille();
    let g = grille.graphe().unwrap();
    for (rang, geo) in GEOMETRIES.iter().enumerate() {
        let source = grille.indice(geo.source.0, geo.source.1, 0);
        let puits = grille.indice(geo.puits.0, geo.puits.1, 0);
        let lacher = grille.indice(geo.lacher_hors.0, geo.lacher_hors.1, 0);
        let qu = Tubes::uniformes(&g, 1.0).unwrap().avancer(&g, source, puits, 1.0, 0.1).unwrap().flux;
        for graine in 1..=6_u64 {
            let (d, q) = converge(&g, source, puits, graine);
            assert!(g.incidentes(lacher).iter().all(|&e| d[e] < 0.5), "le nœud de lâcher est hors des tubes vifs");
            let conv = part_sur_les_vifs(&g, &d, &q, lacher);
            let unif = part_sur_les_vifs(&g, &d, &qu, lacher);
            assert!(conv <= 0.08, "géométrie {rang}, graine {graine} : part convergée {conv}");
            assert!(conv < unif, "géométrie {rang}, graine {graine} : {conv} contre {unif}");
        }
    }
}
