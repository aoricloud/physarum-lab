//! Étape 2, partie 2T : la conservation de la masse (le bilan `masse avant = masse après + sortie + perte - apport`).
//! Valeurs attendues : `references/etape-2t-valeurs.md`, identifiants `V-MAS-xx`.
//!
//! La source est la doc de `souvenir` (« Le pas, à la lettre » : le `Bilan` ; « Ce que l'arrondi coûte »). Sur des
//! valeurs dyadiques la conservation est **exacte** (les cas dyadiques sont vérifiés en arithmétique exacte par
//! `ref_souvenir.py`, fractions) ; sur des flux réels de tubes elle tient à une tolérance **mesurée**, dite en
//! commentaire de chaque test.

#[path = "common/transport_2t.rs"]
mod transport_2t;

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Grille, Voisinage};
use physarum_lab::souvenir::{Agent, Transport, pas_maximal, pas_sans_decoupage};
use physarum_lab::tubes::Tubes;
use transport_2t::{bits, graphe_l, naif_taux, somme};

/// L'anneau de quatre nœuds `0-1-2-3-0` : les arêtes `(0,1)`, `(0,3)`, `(1,2)`, `(2,3)` de longueur 1 ; le flux `[0.5,
/// -0.5, 0.5, 0.5]` fait tourner le liquide `0 -> 1 -> 2 -> 3 -> 0` (l'arête `(0,3)` a `a = 0`, `b = 3` : le liquide y
/// va de `b` vers `a`, d'où `-0.5`) ; aucun nœud n'a de `s` non nul.
fn anneau() -> physarum_lab::graphe::Graphe {
    graphe_l(4, &[(0, 1, 1.0), (0, 3, 1.0), (1, 2, 1.0), (2, 3, 1.0)])
}

/// juste : **sur un anneau, la masse se conserve exactement** (V-MAS-01), valeurs dyadiques : `h = 0.5`, `c = [1, 2, 0,
/// 0.5]` (masse `3.5`). Trois transports : l'advection seule (flux `[0.5, -0.5, 0.5, 0.5]`, `κ = 0`), la diffusion
/// seule (flux nuls, `κ = 0.25`), les deux ; sans perte ni apport. À chacun des 20 pas : la masse vaut **exactement**
/// `3.5` (`masse().to_bits()`), le bilan est exactement nul (`s_i = 0.0` à chaque nœud : la sortie est nulle), et les
/// concentrations finales sont les bits de `ref_souvenir.py` (qui a vérifié, en fractions exactes, que chaque pas de
/// ces trois cas est exact en flottants). Un code qui perdrait de la masse à chaque tour (un arrondi, une sortie
/// parasite) ou qui dédoublerait un échange échoue ici.
///
/// Ne couvre pas : source et puits (`le_bilan_sur_des_flux_de_tubes`), la perte, l'apport.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn l_anneau_conserve_la_masse_exactement() {
    let g = anneau();
    let depart = vec![1.0, 2.0, 0.0, 0.5];
    let cas = [
        (
            "l'advection",
            [0.5, -0.5, 0.5, 0.5],
            0.0,
            [0x3FEC_1C5E_BCC0_0000_u64, 0x3FEC_3CDF_ACA0_0000, 0x3FEB_E39F_C340_0000, 0x3FEB_C321_D360_0000],
        ),
        (
            "la diffusion",
            [0.0; 4],
            0.25,
            [0x3FEC_0CFC_81B9_1000, 0x3FEC_137C_A295_9800, 0x3FEB_F301_FE46_F000, 0x3FEB_EC84_DD6A_6800],
        ),
        (
            "les deux",
            [0.5, -0.5, 0.5, 0.5],
            0.25,
            [0x3FEB_FFF4_AE5A_5000, 0x3FEB_FFF3_9973_D800, 0x3FEC_000B_51A5_B000, 0x3FEC_000C_668C_2800],
        ),
    ];
    for (nom, q, kappa, finales) in cas {
        let tr = Transport { diffusion: kappa, perte: 0.0 };
        let h_max = pas_maximal(&g, &q, &tr).unwrap();
        assert!(0.5 <= h_max, "{nom} : h = 0.5 est sous le pas maximal {h_max}");
        let mut agent = Agent::new(&g, depart.clone()).unwrap();
        let mut courant = depart.clone();
        for t in 0..20 {
            let b = agent.avancer(&g, &q, &tr, &[], 0.5).unwrap();
            assert_eq!(agent.masse().to_bits(), 3.5_f64.to_bits(), "{nom}, pas {t} : la masse");
            assert_eq!(bits(&[b.sortie, b.perte, b.apport]), vec![0_u64; 3], "{nom}, pas {t} : le bilan nul");
            let (suite, _) = pas_sans_decoupage(&g, &q, &courant, &tr, &[], 0.5).unwrap();
            assert_eq!(bits(agent.concentrations()), bits(&suite), "{nom}, pas {t} : les deux fonctions");
            courant = suite;
        }
        assert_eq!(bits(agent.concentrations()), finales.to_vec(), "{nom} : les concentrations après 20 pas");
    }
}

/// juste : **la perte seule** est exacte et son bilan aussi (V-MAS-02), dyadique : un nœud seul, `k = 0.5`, `h = 0.5`
/// (`1 - h k = 0.75`), `c = 1` : `c_n = 0.75^n` (écrit par produits successifs dans le test), à chaque pas `masse_avant
/// = masse_apres + perte` exactement, la perte cumulée est `1 - 0.75^20`. Même chose pour deux nœuds reliés par une
/// arête de flux nul (`κ = 0`) : chacun décroît pour son compte (`[1, 0.5]` donne `[0.75, 0.375]`, et ainsi de suite),
/// la masse perdue est la somme.
///
/// Ne couvre pas : la perte avec transport (`le_bilan_sur_des_flux_de_tubes`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn la_perte_seule_est_exacte() {
    let seul = graphe_l(1, &[]);
    let tr = Transport { diffusion: 0.0, perte: 0.5 };
    let mut agent = Agent::new(&seul, vec![1.0]).unwrap();
    let mut attendu = 1.0_f64;
    let mut cumul = 0.0;
    for t in 0..20 {
        let avant = agent.masse();
        let b = agent.avancer(&seul, &[], &tr, &[], 0.5).unwrap();
        attendu *= 0.75;
        cumul += b.perte;
        assert_eq!(agent.concentrations()[0].to_bits(), attendu.to_bits(), "pas {t}");
        assert_eq!(agent.masse(), avant - b.perte, "pas {t} : le bilan exact");
        assert_eq!(b.sortie.to_bits(), 0, "pas {t} : rien ne sort");
    }
    assert_eq!(cumul, 1.0 - attendu, "la perte cumulée");
    let paire = graphe_l(2, &[(0, 1, 1.0)]);
    let mut agent = Agent::new(&paire, vec![1.0, 0.5]).unwrap();
    let mut a = (1.0_f64, 0.5_f64);
    for t in 0..10 {
        let b = agent.avancer(&paire, &[0.0], &tr, &[], 0.5).unwrap();
        a = (a.0 * 0.75, a.1 * 0.75);
        assert_eq!(bits(agent.concentrations()), bits(&[a.0, a.1]), "paire, pas {t}");
        assert_eq!(b.sous_pas, 1);
    }
}

/// juste : **le bilan avec une source et un puits, sur des flux réels de tubes** (V-MAS-03) : sur 24 réseaux (des
/// grilles de `6 × 6` en voisinage 4, `8 × 8` et `12 × 10` en voisinage 8, quatre graines chacune, deux étapes de
/// convergence : 1 et 30 pas des tubes ; la source et le puits aux deux coins opposés), avec quatre transports `(κ, k)`
/// dont des nuls (`(0, 0)`, `(0.05, 0)`, `(0.3, 0.01)`, `(0, 0.2)`), un apport au voisin de la source, cent pas à la
/// moitié du pas maximal : à chaque pas `masse_apres = masse_avant - sortie - perte + apport` à `1e-13` près en valeur
/// absolue (les masses valent de l'ordre de `1` à `3`), et le bilan composé sur les 100 pas à `2.5e-13`. **Mesuré**
/// (`mes_2t_d.rs`, sur 40 graines autres que celles du test, 960 réseaux-transports de 100 pas) : pire écart par pas
/// `3.0e-14`, pire écart composé `6.4e-14` ; sur les graines du test : `2.1e-14` et `5.7e-14` ; tolérances `1e-13` et
/// `2.5e-13`, environ trois fois le pire observé. Chaque terme du bilan est `>= 0` ; `perte == 0` si `k == 0` ;
/// `apport` vaut `h * debit`.
///
/// Ne couvre pas : la tolérance sur d'autres tailles de réseau (mesurée jusqu'à 120 nœuds).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn le_bilan_sur_des_flux_de_tubes() {
    let formes = [(6_u32, 6_u32, Voisinage::Quatre), (8, 8, Voisinage::Huit), (12, 10, Voisinage::Huit)];
    let transports = [(0.0, 0.0), (0.05, 0.0), (0.3, 0.01), (0.0, 0.2)];
    let mut pire = 0.0_f64;
    let mut reseaux = 0;
    for (largeur, hauteur, voisinage) in formes {
        let grille = Grille { largeur, hauteur, etages: 1, voisinage, longueur_verticale: 1.0, cout_vertical: 1.0 };
        let g = grille.graphe().unwrap();
        let (src, puits) = (grille.indice(0, 0, 0), grille.indice(largeur - 1, hauteur - 1, 0));
        for graine in 1..=4_u64 {
            for pas_tubes in [1, 30] {
                let mut alea = Alea::new(graine);
                let mut tubes = Tubes::au_hasard(&g, &mut alea);
                let mut q = vec![];
                for _ in 0..pas_tubes {
                    q = tubes.avancer(&g, src, puits, 1.0, 0.1).unwrap().flux;
                }
                reseaux += 1;
                for (kappa, k) in transports {
                    let tr = Transport { diffusion: kappa, perte: k };
                    let mut c0 = vec![0.0; g.nb_noeuds()];
                    c0[grille.indice(1, 1, 0)] = 1.0;
                    c0[grille.indice(2, 1, 0)] = 0.7;
                    let mut agent = Agent::new(&g, c0).unwrap();
                    let apports = [(grille.indice(1, 0, 0), 0.3)];
                    let h = 0.5 * pas_maximal(&g, &q, &tr).unwrap();
                    let h = if h.is_finite() { h } else { 0.1 };
                    let (mut ecart_cumul, mut theorique) = (0.0_f64, agent.masse());
                    for _ in 0..100 {
                        let avant = agent.masse();
                        let b = agent.avancer(&g, &q, &tr, &apports, h).unwrap();
                        assert!(
                            b.sortie >= 0.0 && b.perte >= 0.0 && b.apport >= 0.0,
                            "des termes de bilan >= 0 : {b:?}"
                        );
                        assert_eq!(b.apport, h * 0.3, "l'apport d'un pas");
                        if k == 0.0 {
                            assert_eq!(b.perte, 0.0);
                        }
                        let ecart = agent.masse() - (avant - b.sortie - b.perte + b.apport);
                        pire = pire.max(ecart.abs());
                        assert!(ecart.abs() <= 1e-13, "écart {ecart:e} (κ = {kappa}, k = {k})");
                        theorique = theorique - b.sortie - b.perte + b.apport;
                        ecart_cumul = (agent.masse() - theorique).abs();
                    }
                    assert!(ecart_cumul <= 2.5e-13, "écart composé {ecart_cumul:e}");
                }
            }
        }
    }
    assert_eq!(reseaux, 24);
    assert!(pire > 0.0, "l'arrondi laisse bien un écart non nul quelque part ({pire:e}) : le test mesure");
}

/// juste : **sur un long transport, toute la masse finit par sortir au puits** (V-MAS-04). Une grille de `5 × 4`
/// (voisinage 4), des tubes uniformes de conductivité `1` (le flux du premier pas des tubes, source au coin `(0, 0)`,
/// puits au coin `(4, 3)`), `κ = 0.1`, `k = 0`, une libération de `1` au nœud `(1, 1)`, des pas de `h = 0.5`.
/// **Mesuré** (`mes_2t_c2.rs`) : la masse passe sous `1e-3` au pas 153, `1e-6` au pas 290, `1e-9` au pas 426, `1e-12`
/// au pas 563 (le pas maximal est `0.8333333333333329` : `h = 0.5` fait un seul sous-pas). Le test joue 600 pas : masse
/// finale `< 1e-12` ; `1 - masse - sortie cumulée` vaut zéro à `2e-14` près (mesuré : `1.6e-15`) et la sortie cumulée
/// dépasse `1 - 1e-12` ; la perte cumulée est nulle ; chaque concentration est `>= 0` à chaque pas ; et la masse ne
/// remonte jamais (elle est non croissante pas à pas). Sans diffusion (`κ = 0`), la masse sort aussi (mesuré : `1e-12`
/// au pas 255).
///
/// Ne couvre pas : le réseau convergé, où une fuite minuscule vers des tubes morts retient l'agent des milliers de pas
/// (voir `references/etape-2t-valeurs.md`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn toute_la_masse_finit_par_sortir() {
    let grille = Grille {
        largeur: 5,
        hauteur: 4,
        etages: 1,
        voisinage: Voisinage::Quatre,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let g = grille.graphe().unwrap();
    let (src, puits) = (grille.indice(0, 0, 0), grille.indice(4, 3, 0));
    let q = Tubes::uniformes(&g, 1.0).unwrap().avancer(&g, src, puits, 1.0, 0.1).unwrap().flux;
    for (kappa, attendu_pas) in [(0.1, 600), (0.0, 300)] {
        let tr = Transport { diffusion: kappa, perte: 0.0 };
        let mut c0 = vec![0.0; g.nb_noeuds()];
        c0[grille.indice(1, 1, 0)] = 1.0;
        let mut agent = Agent::new(&g, c0).unwrap();
        let (mut sortie, mut perte, mut precedente) = (0.0, 0.0, 1.0);
        for t in 0..attendu_pas {
            let b = agent.avancer(&g, &q, &tr, &[], 0.5).unwrap();
            sortie += b.sortie;
            perte += b.perte;
            assert!(agent.concentrations().iter().all(|&x| x >= 0.0), "pas {t} : une concentration négative");
            let m = agent.masse();
            assert!(m <= precedente, "pas {t} : la masse ne remonte pas ({m:e} après {precedente:e})");
            precedente = m;
        }
        assert!(agent.masse() < 1e-12, "κ = {kappa} : masse finale {:e}", agent.masse());
        let ecart = (1.0 - agent.masse() - sortie).abs();
        assert!(ecart <= 2e-14, "κ = {kappa} : 1 - masse - sortie cumulée = {ecart:e}");
        assert!(sortie > 1.0 - 1e-12, "κ = {kappa} : la sortie cumulée vaut presque 1 ({sortie})");
        assert_eq!(perte, 0.0);
    }
}

/// juste : **ce que coûte la règle de la sortie aux nœuds intérieurs** (V-MAS-05). Sur le réseau convergé d'une grille
/// de `12 × 9` (voisinage 8, source `(1, 4)`, puits `(10, 4)`, 300 pas des tubes de `h = 0.1`, quatre graines), pour
/// chaque nœud on calcule `s_i` par le modèle naïf du test (la même règle que la doc) : le puits a `sortie = 1` à
/// `5e-14` près (le débit total) ; **tout autre nœud a une sortie `<= 2e-14`** ; et sur 200 pas au pas maximal, une
/// libération de `1` près de la source perd par les nœuds autres que le puits au plus `4e-14` de sa masse. **Mesuré**
/// (`mes_2t_d.rs`, 40 graines autres que celles du test) : `|sortie_puits - 1|` au plus `1.2e-14`, sortie hors puits au
/// plus `5.1e-15`, fuite sur 200 pas au plus `9.2e-15` (sur les graines du test : `8.2e-15`, `2.7e-15`, `4.8e-15`) ;
/// les tolérances valent environ quatre fois le pire observé. La règle fait donc fuir quelques milliardièmes de
/// millionième de la masse sur 200 pas, jamais plus ; une règle à seuil serait un autre modèle (la doc n'en a pas).
///
/// Ne couvre pas : d'autres débits que 1, d'autres tailles de grille.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn la_regle_de_la_sortie_coute_un_arrondi() {
    let grille = Grille {
        largeur: 12,
        hauteur: 9,
        etages: 1,
        voisinage: Voisinage::Huit,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let g = grille.graphe().unwrap();
    let (src, puits, lacher) = (grille.indice(1, 4, 0), grille.indice(10, 4, 0), grille.indice(2, 4, 0));
    let mut pire_sortie = 0.0_f64;
    let mut pire_fuite = 0.0_f64;
    for graine in 1..=4_u64 {
        let mut alea = Alea::new(graine);
        let mut tubes = Tubes::au_hasard(&g, &mut alea);
        let mut q = vec![];
        for _ in 0..300 {
            q = tubes.avancer(&g, src, puits, 1.0, 0.1).unwrap().flux;
        }
        let (_, sortie_i) = naif_taux(&g, &q, 0.0, 0.0);
        assert!((sortie_i[puits] - 1.0).abs() <= 5e-14, "le puits : {}", sortie_i[puits]);
        for (i, &s) in sortie_i.iter().enumerate() {
            if i != puits {
                pire_sortie = pire_sortie.max(s);
                assert!(s <= 2e-14, "le nœud {i} sort {s:e}");
            }
        }
        let tr = Transport { diffusion: 0.0, perte: 0.0 };
        let h = pas_maximal(&g, &q, &tr).unwrap();
        let mut c = vec![0.0; g.nb_noeuds()];
        c[lacher] = 1.0;
        let mut fuite = 0.0;
        for _ in 0..200 {
            for i in 0..g.nb_noeuds() {
                if i != puits {
                    fuite += (h * sortie_i[i]) * c[i];
                }
            }
            c = pas_sans_decoupage(&g, &q, &c, &tr, &[], h).unwrap().0;
        }
        pire_fuite = f64::max(pire_fuite, fuite);
        assert!(fuite <= 4e-14, "fuite par les nœuds intérieurs : {fuite:e}");
        assert!(somme(&c) > 0.0);
    }
    assert!(pire_sortie > 0.0, "l'arrondi laisse bien une sortie non nulle quelque part : le test mesure");
    assert!(pire_fuite > 0.0, "et une fuite non nulle ({pire_fuite:e})");
}
