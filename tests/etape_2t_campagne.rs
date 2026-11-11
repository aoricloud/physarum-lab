//! Étape 2, partie 2T : les tests ajoutés par le juge après la campagne de fautes. Chacun ferme une faiblesse que la
//! campagne a trouvée (une faute plausible que les autres tests laissaient passer). Valeurs attendues :
//! `references/etape-2t-valeurs.md`, identifiants `V-CAM-xx`. Les valeurs se calculent à la main d'après la doc de
//! `souvenir` ; aucune ne vient du prototype.

#[path = "common/transport_2t.rs"]
mod transport_2t;

use physarum_lab::graphe::{Arete, Graphe, Noeud};
use physarum_lab::souvenir::{AGENT, Agent, ErreurAgent, Transport, apercu, pas_maximal, pas_sans_decoupage};
use physarum_lab::tubes::apercu as apercu_tubes;
use transport_2t::graphe_l;

/// **V-CAM-01** : un taux sous-normal. `1.0 / r_max` déborde (infini) ; le plus grand `h` tel que `h * r_max <= 1.0`
/// est alors `f64::MAX` (`f64::INFINITY * r_max` dépasse 1) : c'est le seul cas où la descente d'un cran sert.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn pas_maximal_quand_un_sur_taux_deborde() {
    let g = graphe_l(1, &[]);
    for perte in [1e-320, 5e-324, 1e-310] {
        let t = Transport { diffusion: 0.0, perte };
        let h = pas_maximal(&g, &[], &t).unwrap();
        assert_eq!(h.to_bits(), f64::MAX.to_bits(), "perte {perte:e}");
        assert!(h * perte <= 1.0);
    }
}

/// **V-CAM-02** : un `Debordement` au troisième sous-pas laisse l'agent tel qu'avant l'appel, même si les deux premiers
/// sous-pas étaient finis. Chaîne `0-1`, flux `1` (de 0 vers 1), pas maximal `1`, `h = 2.1` : trois sous-pas de `0.7`.
/// Deux apports de `0.7 * f64::MAX`. En fraction de `f64::MAX` : `[0.49, 0.49]` après le premier sous-pas,
/// `[0.637, 0.98]` après le deuxième (`0.3 * 0.49 + 0.49` ; `0.3 * 0.49 + 0.7 * 0.49 + 0.49`), puis
/// `0.3 * 0.98 + 0.7 * 0.637 + 0.49 = 1.2299` au nœud 1 : il déborde.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn debordement_tardif_laisse_l_agent_intact() {
    let g = graphe_l(2, &[(0, 1, 1.0)]);
    let t = Transport { diffusion: 0.0, perte: 0.0 };
    let d = 0.7 * f64::MAX;
    let mut agent = Agent::zero(&g);
    let avant = agent.clone();
    let r = agent.avancer(&g, &[1.0], &t, &[(0, d), (1, d)], 2.1);
    assert_eq!(r, Err(ErreurAgent::Debordement));
    assert_eq!(agent, avant, "sur une erreur, l'agent n'a pas changé");
    // Le témoin : avec `h = 1.4` (deux sous-pas de `0.7`), le même appel réussit et finit à `[0.637, 0.98]`.
    let mut temoin = Agent::zero(&g);
    let bilan = temoin.avancer(&g, &[1.0], &t, &[(0, d), (1, d)], 1.4).expect("deux sous-pas finis");
    assert_eq!(bilan.sous_pas, 2);
    let c = temoin.concentrations();
    assert!(c[0] > 0.63 * f64::MAX && c[0] < 0.64 * f64::MAX, "{c:?}");
    assert!(c[1] > 0.97 * f64::MAX && c[1] < 0.99 * f64::MAX, "{c:?}");
}

/// **V-CAM-03** : un `Debordement` par un seul des trois totaux du bilan. Les concentrations d'après sont finies,
/// mais la somme des apports (`0.9 * MAX` deux fois), des pertes (`c = 0.9 * MAX` sur deux nœuds, `h * k = 1`, donc
/// `c_apres = 0`) ou des sorties (deux puits, `h * sortie_i = 1`) dépasse le plus grand flottant. Aux deux fonctions
/// publiques ; sur l'erreur, l'agent n'a pas changé.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn debordement_par_un_seul_total_du_bilan() {
    let m = 0.9 * f64::MAX;
    let zero = Transport { diffusion: 0.0, perte: 0.0 };
    let perte = Transport { diffusion: 0.0, perte: 1.0 };
    // l'apport
    let g2 = graphe_l(2, &[]);
    let ap = [(0, m), (1, m)];
    assert_eq!(pas_sans_decoupage(&g2, &[], &[0.0, 0.0], &zero, &ap, 1.0), Err(ErreurAgent::Debordement), "apport");
    let mut a = Agent::zero(&g2);
    assert_eq!(a.avancer(&g2, &[], &zero, &ap, 1.0), Err(ErreurAgent::Debordement), "apport (avancer)");
    assert_eq!(a.concentrations(), &[0.0, 0.0]);
    // la perte
    let c = [m, m];
    assert_eq!(pas_sans_decoupage(&g2, &[], &c, &perte, &[], 1.0), Err(ErreurAgent::Debordement), "perte");
    let mut a = Agent::new(&g2, c.to_vec()).unwrap();
    assert_eq!(a.avancer(&g2, &[], &perte, &[], 1.0), Err(ErreurAgent::Debordement), "perte (avancer)");
    assert_eq!(a.concentrations(), &c);
    // la sortie : deux arêtes séparées, flux 1 de chaque nœud pair vers le nœud impair, qui est un puits
    let g4 = graphe_l(4, &[(0, 1, 1.0), (2, 3, 1.0)]);
    let c = [0.0, m, 0.0, m];
    assert_eq!(pas_sans_decoupage(&g4, &[1.0, 1.0], &c, &zero, &[], 1.0), Err(ErreurAgent::Debordement), "sortie");
    let mut a = Agent::new(&g4, c.to_vec()).unwrap();
    assert_eq!(a.avancer(&g4, &[1.0, 1.0], &zero, &[], 1.0), Err(ErreurAgent::Debordement), "sortie (avancer)");
    assert_eq!(a.concentrations(), &c);
}

/// **V-CAM-04** : l'ordre des vérifications avec un flux **trop long** : `Taille` passe avant une concentration
/// négative (`Concentration(0)`), comme avec un flux trop court.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn taille_des_flux_trop_longs_avant_les_concentrations() {
    let g = graphe_l(2, &[(0, 1, 1.0)]);
    let t = Transport { diffusion: 0.0, perte: 0.0 };
    let c = [-1.0, 0.0];
    assert_eq!(pas_sans_decoupage(&g, &[1.0, 2.0], &c, &t, &[], 1.0), Err(ErreurAgent::Taille));
    assert_eq!(pas_sans_decoupage(&g, &[1.0], &c, &t, &[], 1.0), Err(ErreurAgent::Concentration(0)));
}

/// Le mélange de la doc (« L'aperçu ») sur un canal : `(a * (255 - n) + b * n + 127) / 255` en entiers.
fn melange(a: [u8; 3], n: u32) -> [u8; 3] {
    let canal = |k: usize| ((u32::from(a[k]) * (255 - n) + u32::from(AGENT[k]) * n + 127) / 255) as u8;
    [canal(0), canal(1), canal(2)]
}

/// **V-CAM-05** : le niveau et le mélange, écrits à part d'après la doc, sur le pixel du nœud 0 : tous les niveaux
/// entiers (`c = n / 255`), les moitiés (`c = (k + 0.5) / 255`, où `round` va à l'écart de zéro : `0.5` donne 1,
/// `2.5` donne 3), et des concentrations juste sous la référence (`0.99` donne 252). Le pixel de base est celui de
/// `tubes::apercu`.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn tous_les_niveaux_et_tous_les_melanges() {
    let g = graphe_l(2, &[(0, 1, 1.0)]);
    let cond = [1.0];
    let base = apercu_tubes(&g, &cond, 1.0, 1);
    let depart = [base.rvb[0], base.rvb[1], base.rvb[2]];
    let niveau = |c: f64| -> u32 {
        let t = c / 1.0;
        if t >= 1.0 {
            255
        } else if t > 0.0 {
            (t * 255.0).round() as u32
        } else {
            0
        }
    };
    let mut cas: Vec<f64> = (0..=255).map(|n| f64::from(n) / 255.0).collect();
    cas.extend((0..255).map(|k| (f64::from(k) + 0.5) / 255.0));
    cas.extend([0.99, 0.996, 0.9999, 1.0, 7.0]);
    assert_eq!(niveau(0.5 / 255.0), 1);
    assert_eq!(niveau(2.5 / 255.0), 3);
    assert_eq!(niveau(0.99), 252);
    let mut vus = [false; 256];
    for &c in &cas {
        let n = niveau(c);
        vus[n as usize] = true;
        let img = apercu(&g, &cond, &[c, 0.0], 1.0, 1.0, 1);
        assert_eq!(img.rvb[0..3], melange(depart, n), "c = {c:e}, niveau {n}");
    }
    assert!(vus.iter().all(|&v| v), "tous les niveaux de 0 à 255 sont joués");
}

/// **V-CAM-06** : le mélange sur presque toutes les couleurs de départ. Deux nœuds au même point (le second mélange sur
/// le résultat du premier), tous les couples de niveaux `(n1, n2)` de 0 à 255 : les couleurs de départ du second
/// mélange parcourent ainsi tous les restes de la division entière par 255 (l'arrondi `+ 127` ne se voit que sur
/// certains).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn le_melange_sur_toutes_les_couleurs_de_depart() {
    let noeuds = vec![Noeud { x: 0, y: 0, etage: 0 }, Noeud { x: 0, y: 0, etage: 0 }];
    let g = Graphe::new(noeuds, vec![Arete { a: 0, b: 1, longueur: 1.0, cout: 1.0 }]).unwrap();
    let cond = [1.0];
    let base = apercu_tubes(&g, &cond, 1.0, 1);
    assert_eq!(base.rvb.len(), 3, "un seul point");
    let depart = [base.rvb[0], base.rvb[1], base.rvb[2]];
    for n1 in 0..=255_u32 {
        let milieu = melange(depart, n1);
        for n2 in 0..=255_u32 {
            let img = apercu(&g, &cond, &[f64::from(n1) / 255.0, f64::from(n2) / 255.0], 1.0, 1.0, 1);
            assert_eq!(img.rvb[..], melange(milieu, n2), "niveaux {n1} puis {n2}");
        }
    }
}
