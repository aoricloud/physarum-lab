//! Étape 2, partie 2T : le pas du module contre un pas naïf écrit dans le test, au bit, sur des graphes au hasard et
//! sur les flux de vrais pas des tubes. Valeurs attendues : `references/etape-2t-valeurs.md`, identifiants `V-BIT-xx`.
//!
//! La source est la doc de `souvenir` (« Le pas, à la lettre », « Le pas public et son découpage »). Le pas naïf du
//! test (`common/transport_2t.rs`) est écrit d'après cette seule doc, par une boucle **sur les arêtes** (le module en
//! fait une par nœud) : mêmes opérations, mêmes parenthèses, même ordre des termes ; il rend les mêmes bits ou la doc
//! ne dit pas assez.

#[path = "common/graphes_au_hasard.rs"]
mod graphes_au_hasard;
#[path = "common/transport_2t.rs"]
mod transport_2t;

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Grille, Voisinage};
use physarum_lab::reseaux::{Loi, Reseau};
use physarum_lab::souvenir::{Agent, Transport, pas_maximal, pas_sans_decoupage};
use physarum_lab::tubes::Tubes;
use transport_2t::{bits, naif_avancer, naif_pas};

/// Des concentrations au hasard dans `[0, 2[`, dont quelques zéros.
fn concentrations(alea: &mut Alea, n: usize) -> Vec<f64> {
    (0..n).map(|_| if alea.borne(5) == 0 { 0.0 } else { 2.0 * alea.suivant_f64() }).collect()
}

/// juste : **un pas, des flux quelconques** (V-BIT-01). Sur 100 graphes au hasard (de 2 à 30 nœuds, des arêtes en plus
/// de l'arbre, coûts et longueurs différents de ceux des tubes), des flux de signes et de grandeurs quelconques (dont
/// des zéros), `κ` et `k` au hasard (dont `0.0`), une liste de zéro à trois apports (un même nœud plusieurs fois
/// possible), un `h` au hasard (au-dessus de la limite aussi : `pas_sans_decoupage` ne la regarde pas) :
/// `pas_sans_decoupage` rend les mêmes bits que le pas naïf, concentrations et bilan (`sortie`, `perte`, `apport`).
///
/// Ne couvre pas : le pas public découpé (`l_agent_sur_les_flux_des_tubes`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn un_pas_contre_le_pas_naif_sur_des_graphes_au_hasard() {
    let mut alea = Alea::new(4242);
    let mut vus = 0;
    for cas in 0..100 {
        let n = 2 + alea.borne(29) as usize;
        let g = graphes_au_hasard::graphe_au_hasard(&mut alea, n, n, 0.5, 2.0);
        let q: Vec<f64> =
            (0..g.nb_aretes()).map(|_| if alea.borne(6) == 0 { 0.0 } else { 4.0 * alea.suivant_f64() - 2.0 }).collect();
        let kappa = if alea.borne(4) == 0 { 0.0 } else { alea.suivant_f64() };
        let k = if alea.borne(4) == 0 { 0.0 } else { 0.2 * alea.suivant_f64() };
        let c = concentrations(&mut alea, n);
        let apports: Vec<(usize, f64)> =
            (0..alea.borne(4)).map(|_| (alea.borne(n as u32) as usize, 3.0 * alea.suivant_f64())).collect();
        let h = 0.01 + 0.5 * alea.suivant_f64();
        let tr = Transport { diffusion: kappa, perte: k };
        let (suite, b) = pas_sans_decoupage(&g, &q, &c, &tr, &apports, h).unwrap();
        let (naif, [sortie, perte, apport]) = naif_pas(&g, &q, &c, kappa, k, &apports, h);
        assert_eq!(bits(&suite), bits(&naif), "cas {cas} : concentrations");
        assert_eq!(bits(&[b.sortie, b.perte, b.apport]), bits(&[sortie, perte, apport]), "cas {cas} : bilan");
        assert_eq!(b.sous_pas, 1);
        vus += suite.len();
    }
    assert!(vus > 500, "assez de nœuds : {vus}");
}

/// juste : **le pas public sur les flux de vrais pas des tubes** (V-BIT-02). Sur 24 graphes (une grille de `8 × 6` en
/// voisinage 8 et un graphe au hasard de 20 nœuds, six graines chacune, deux cas de `h`) : les tubes de Tero
/// (`Tubes::au_hasard`, source au nœud 0, puits au dernier nœud) avancent de 1, de 10 puis de 40 pas ; le flux de
/// chacun de ces trois états alimente un agent (concentrations au hasard, un apport au nœud 1) joué sur 12 pas publics
/// de durée `h` : `0.7` fois le pas maximal (un sous-pas) ou `3.3` fois le pas maximal (quatre sous-pas). À chaque pas
/// les concentrations, le bilan et le nombre de sous-pas sont ceux du pas public naïf du test (le nombre de sous-pas
/// par essai de chaque `n`) ; les concentrations restent `>= 0`.
///
/// Ne couvre pas : les flux d'un réseau à plusieurs sources (`l_agent_sur_les_flux_d_un_reseau_sigmoide`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn l_agent_sur_les_flux_des_tubes() {
    let grille = Grille {
        largeur: 8,
        hauteur: 6,
        etages: 1,
        voisinage: Voisinage::Huit,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let mut pas_vus = 0;
    for forme in 0..2 {
        for graine in 1..=6_u64 {
            let mut alea = Alea::new(1000 * forme + graine);
            let g = if forme == 0 {
                grille.graphe().unwrap()
            } else {
                graphes_au_hasard::graphe_au_hasard(&mut alea, 20, 25, 0.5, 2.0)
            };
            let dernier = g.nb_noeuds() - 1;
            let mut tubes = Tubes::au_hasard(&g, &mut alea);
            for etape in [1, 9, 30] {
                let mut q = vec![];
                for _ in 0..etape {
                    q = tubes.avancer(&g, 0, dernier, 1.0, 0.1).unwrap().flux;
                }
                for facteur in [0.7, 3.3] {
                    let (kappa, k) = (0.1 * alea.suivant_f64(), 0.05 * alea.suivant_f64());
                    let tr = Transport { diffusion: kappa, perte: k };
                    let h = facteur * pas_maximal(&g, &q, &tr).unwrap();
                    let depart = concentrations(&mut alea, g.nb_noeuds());
                    let apports = [(1, 0.4)];
                    let mut agent = Agent::new(&g, depart.clone()).unwrap();
                    let mut naif = depart;
                    for t in 0..12 {
                        let b = agent.avancer(&g, &q, &tr, &apports, h).unwrap();
                        let (suite, total, n) = naif_avancer(&g, &q, &naif, kappa, k, &apports, h);
                        naif = suite;
                        assert_eq!(
                            bits(agent.concentrations()),
                            bits(&naif),
                            "forme {forme}, graine {graine}, pas {t}"
                        );
                        assert_eq!(bits(&[b.sortie, b.perte, b.apport]), bits(&total), "bilan, pas {t}");
                        assert_eq!(b.sous_pas, n);
                        assert!(agent.concentrations().iter().all(|&x| x >= 0.0 && x.to_bits() >> 63 == 0));
                        pas_vus += 1;
                    }
                }
            }
        }
    }
    assert_eq!(pas_vus, 2 * 6 * 3 * 2 * 12);
}

/// juste : **les flux d'un réseau à plusieurs sources, sous la loi sigmoïde** (V-BIT-03) : sur 6 graines, une grille de
/// `7 × 7` en voisinage 8, cinq nourritures, `I0 = 2`, `h_tubes = 0.1` : à chaque pas, `Reseau::avancer` tire une paire
/// (source, puits) et rend les flux (`PasReseau::etat.flux`) ; **l'agent avance d'un pas public avec ces flux-là**, qui
/// changent à chaque pas (source et puits différents d'un pas à l'autre : le liquide entre et sort en des nœuds
/// différents, la règle des bords suit). Soixante pas, `κ = 0.1`, `k = 0.02`, `h = 0.3`, un apport de `0.05` à un nœud
/// de nourriture : concentrations, bilan et nombre de sous-pas comme ceux du pas public naïf, à chaque pas ; la masse
/// vérifie le bilan à `1e-13` près ; aucune concentration négative.
///
/// Ne couvre pas : la mémoire (partie 2M) : l'agent ne rétroagit pas sur les tubes ici.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn l_agent_sur_les_flux_d_un_reseau_sigmoide() {
    let grille = Grille {
        largeur: 7,
        hauteur: 7,
        etages: 1,
        voisinage: Voisinage::Huit,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let g = grille.graphe().unwrap();
    let nourritures: Vec<usize> =
        [(0, 0), (6, 0), (3, 3), (0, 6), (6, 6)].iter().map(|&(x, y)| grille.indice(x, y, 0)).collect();
    let tr = Transport { diffusion: 0.1, perte: 0.02 };
    let mut pas_vus = 0;
    let mut sous_pas_total = 0;
    for graine in 1..=6_u64 {
        let mut alea = Alea::new(graine);
        let mut reseau = Reseau::au_hasard(&g, &mut alea, nourritures.clone(), 2.0, Loi::Sigmoide).unwrap();
        let mut agent = Agent::zero(&g);
        let mut naif = vec![0.0; g.nb_noeuds()];
        let apports = [(nourritures[2], 0.05)];
        for t in 0..60 {
            let pas = reseau.avancer(&g, &mut alea, 0.1).unwrap();
            let avant = agent.masse();
            let b = agent.avancer(&g, &pas.etat.flux, &tr, &apports, 0.3).unwrap();
            let (suite, total, n) = naif_avancer(&g, &pas.etat.flux, &naif, 0.1, 0.02, &apports, 0.3);
            naif = suite;
            assert_eq!(bits(agent.concentrations()), bits(&naif), "graine {graine}, pas {t}");
            assert_eq!(bits(&[b.sortie, b.perte, b.apport]), bits(&total), "graine {graine}, pas {t} : bilan");
            assert_eq!(b.sous_pas, n);
            assert!(agent.concentrations().iter().all(|&x| x >= 0.0));
            let ecart = agent.masse() - (avant - b.sortie - b.perte + b.apport);
            assert!(ecart.abs() <= 1e-13, "graine {graine}, pas {t} : écart de masse {ecart:e}");
            pas_vus += 1;
            sous_pas_total += n;
        }
    }
    assert_eq!(pas_vus, 360);
    assert!(sous_pas_total >= pas_vus, "au moins un sous-pas par pas");
}
