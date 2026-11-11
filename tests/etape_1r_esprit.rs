//! Étape 1, partie 1R : Tero et al. 2010 « en esprit » : la longueur et la tenue aux pannes du réseau croissent avec le
//! débit `I0`, sous la loi sigmoïde, sur une grille et six sources de nourriture. Valeurs attendues :
//! `references/etape-1r-valeurs.md`, identifiants `V-ESP-xx`.
//!
//! **Ce que l'article dit** (Science 327, 2010, p. 439-441, Fig. 3 et supplément, lu) : plus `I0` est grand, plus le
//! réseau garde de routes de rechange, avec une meilleure tenue à une panne et un coût (longueur) plus élevé ; à `I0`
//! faible, il tend vers l'arbre couvrant minimal. **Ce que l'article ne publie pas** : le maillage, le nombre de nœuds,
//! le pas de temps, le schéma, les conductivités de départ, les formules de TL, MD et FT. **Aucune valeur publiée n'est
//! visée** : ces tests disent que notre schéma (Euler, `γ = 2`, grille de 10 × 10 à huit voisins, `h = 0,1`, 800 pas,
//! tube vif au seuil `SEUIL_VIF`) va dans le même sens que l'article, avec des marges mesurées, rien de plus. Les
//! chiffres des `I0` ne sont pas ceux de l'article (`γ = 1,8`, `I0` de 0,2 à 7,2 chez lui, sur un maillage inconnu).
//!
//! Chaque seuil ci-dessous a été **mesuré sur 40 graines autres** que celles du test (2000 à 2039 : `outils-1R/
//! mesures/mes_1r_esprit.rs`) ; les graines du test sont 1 à 8, écrites ici, jamais choisies parce qu'elles passent.

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Grille, Voisinage};
use physarum_lab::reseaux::{Loi, Mesures, Reseau, SEUIL_VIF, mesurer};

/// La grille de 10 × 10 à huit voisins et six nourritures.
fn monde() -> (physarum_lab::graphe::Graphe, Vec<usize>) {
    let grille = Grille {
        largeur: 10,
        hauteur: 10,
        etages: 1,
        voisinage: Voisinage::Huit,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let g = grille.graphe().unwrap();
    let positions = [(1, 1), (8, 2), (4, 5), (2, 8), (8, 8), (4, 8)];
    let nourritures = positions.iter().map(|&(x, y)| grille.indice(x, y, 0)).collect();
    (g, nourritures)
}

/// Un réseau de 800 pas à `h = 0,1`, et ses mesures au seuil `SEUIL_VIF`.
fn simuler(loi: Loi, i0: f64, graine: u64) -> Mesures {
    let (g, foods) = monde();
    let mut alea = Alea::new(graine);
    let mut reseau = Reseau::au_hasard(&g, &mut alea, foods.clone(), i0, loi).unwrap();
    for _ in 0..800 {
        reseau.avancer(&g, &mut alea, 0.1).unwrap();
    }
    mesurer(&g, reseau.conductivites(), &foods, SEUIL_VIF).unwrap()
}

fn moyenne(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// juste : (V-ESP-01) sous la loi **sigmoïde**, `TL / TL_MST` et `FT` **croissent avec `I0`**, `MD / MD_MST` décroît
/// (Tero et al. 2010 : « plus de routes alternatives, meilleure FT, coût plus élevé », et la distance moyenne qui
/// baisse dans le supplément, Fig. S1 ; en esprit seulement). `I0` vaut 1, 2, 3, 4 puis 8 (cinq valeurs), huit graines.
/// Mesuré sur 40 autres graines :
/// - **en moyenne** : `TL / TL_MST` moyen 1,03, 1,24, 1,55, 1,86, 3,14 (écart-type 0,03 à 0,18) : la plus petite hausse
///   entre deux `I0` consécutifs vaut 0,21 ; seuil `0,1`. `FT` moyen 0,00, 0,53, 0,91, 0,99, 1,00 : hausses 0,53, 0,39,
///   0,078 puis 0,009 ; seuils `0,3`, `0,2`, `0,03` pour les trois premières, et « pas de baisse » entre 4 et 8 (le
///   réseau y est saturé à 1). `MD / MD_MST` moyen 1,06, 0,98, 0,95, 0,91, 0,85 : baisses 0,082, 0,035, 0,034, 0,061 ;
///   seuil `0,015` ;
/// - **dans le pire cas** (chaque graine) : `TL` et `FT` ne baissent jamais quand `I0` croît : la plus petite
///   différence par graine, mesurée sur ces 40 graines, est **0 exactement** (deux `I0` voisins donnent parfois le même
///   réseau) : le test exige `>= 0`, pas une hausse stricte ; `MD / MD_MST` n'est testé qu'en moyenne.
///
/// Ne couvre pas : les valeurs de l'article, d'autres maillages, d'autres `γ`, des `I0` hors de 1 à 8 (à `I0 = 0,05` le
/// réseau se défait : `MD` infini).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn longueur_et_tenue_croissent_avec_le_debit() {
    let i0s = [1.0, 2.0, 3.0, 4.0, 8.0];
    let graines: Vec<u64> = (1..=8).collect();
    // tableau[a][s] : les mesures de I0[a], graine s
    let tableau: Vec<Vec<Mesures>> =
        i0s.iter().map(|&i0| graines.iter().map(|&s| simuler(Loi::Sigmoide, i0, s)).collect()).collect();
    let tl: Vec<Vec<f64>> = tableau.iter().map(|l| l.iter().map(|m| m.tl_sur_mst).collect()).collect();
    let ft: Vec<Vec<f64>> = tableau.iter().map(|l| l.iter().map(|m| m.ft).collect()).collect();
    let md: Vec<Vec<f64>> = tableau.iter().map(|l| l.iter().map(|m| m.md_sur_mst).collect()).collect();
    assert_eq!(tl.len(), 5);
    assert!(tl.iter().all(|l| l.len() == 8) && md.iter().flatten().all(|x| x.is_finite() && *x > 0.0));
    // en moyenne
    let (m_tl, m_ft, m_md): (Vec<f64>, Vec<f64>, Vec<f64>) = (
        tl.iter().map(|l| moyenne(l)).collect(),
        ft.iter().map(|l| moyenne(l)).collect(),
        md.iter().map(|l| moyenne(l)).collect(),
    );
    for a in 1..5 {
        assert!(m_tl[a] - m_tl[a - 1] >= 0.1, "TL / TL_MST moyen : {m_tl:?}");
        assert!(m_md[a - 1] - m_md[a] >= 0.015, "MD / MD_MST moyen : {m_md:?}");
        assert!(m_ft[a] >= m_ft[a - 1], "FT moyen : {m_ft:?}");
    }
    for (a, seuil) in [(1, 0.3), (2, 0.2), (3, 0.03)] {
        assert!(m_ft[a] - m_ft[a - 1] >= seuil, "FT moyen, hausse {a} : {m_ft:?}");
    }
    // pire cas : jamais de baisse pour une graine
    for s in 0..8 {
        for a in 1..5 {
            assert!(
                tl[a][s] >= tl[a - 1][s],
                "graine {}, I0 {} -> {} : TL {} -> {}",
                graines[s],
                i0s[a - 1],
                i0s[a],
                tl[a - 1][s],
                tl[a][s]
            );
            assert!(
                ft[a][s] >= ft[a - 1][s],
                "graine {}, I0 {} -> {} : FT {} -> {}",
                graines[s],
                i0s[a - 1],
                i0s[a],
                ft[a - 1][s],
                ft[a][s]
            );
        }
    }
    // les ordres de grandeur du constat : un réseau presque arbre à I0 = 1, maillé à I0 = 8
    assert!(m_ft[0] == 0.0 && m_ft[4] == 1.0 && m_tl[4] > 2.5);
}

/// juste : (V-ESP-02) **à faible `I0`, sous la sigmoïde, le réseau est presque l'arbre couvrant minimal** (Tero et al.
/// 2010 : « baisser `I0` : vers l'arbre couvrant minimal ») : à `I0 = 1`, `FT = 0` pour chacune des huit graines (40
/// sur 40 mesurées sur les autres graines) et `TL / TL_MST` entre `0,95` et `1,2` (mesuré de 0,979 à 1,115 sur ces 40 ;
/// il peut être **sous 1** : `TL_MST` additionne des chemins qui peuvent partager des arêtes). **Sous la loi linéaire,
/// le contraire** de ce que le pilote attendait (« à faible `I0`, le réseau tend vers un arbre ») : à `I0 = 0,5` et à
/// `I0 = 1`, `FT = 1` pour chaque graine (40 sur 40) et `TL / TL_MST` vaut au moins `5` (mesuré : 5,6 à 6,5 à `I0 =
/// 0,5`) et `6` (6,4 à 7,2 à `I0 = 1`) ; le réseau reste **épais** : les tubes des quinze chemins de paires se
/// partagent le flux et meurent lentement (explication probable, **non vérifiée** : la loi linéaire sélectionne le plus
/// court chemin de **chaque paire**, pas l'arbre des nourritures) ; à 3 000 pas, `TL / TL_MST` baisse encore (de 5,3 à
/// 4,0 sur un maillage aux longueurs aléatoires, mesuré à part) sans que `FT` bouge. Donc, dans notre schéma, **la
/// tendance vers l'arbre est un effet de la sigmoïde**, pas de la loi linéaire.
///
/// Ne couvre pas : la vitesse de convergence de la loi linéaire, d'autres maillages.
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn a_faible_debit_la_sigmoide_donne_un_arbre_et_la_loi_lineaire_un_reseau_epais() {
    for graine in 1..=8_u64 {
        let s = simuler(Loi::Sigmoide, 1.0, graine);
        assert_eq!(s.ft, 0.0, "sigmoïde, graine {graine} : FT");
        assert!((0.95..=1.2).contains(&s.tl_sur_mst), "sigmoïde, graine {graine} : TL / TL_MST = {}", s.tl_sur_mst);
        assert!(s.md.is_finite(), "toutes les nourritures sont reliées, graine {graine}");
        let l05 = simuler(Loi::Lineaire, 0.5, graine);
        let l1 = simuler(Loi::Lineaire, 1.0, graine);
        assert_eq!(l05.ft, 1.0, "linéaire, I0 = 0,5, graine {graine}");
        assert_eq!(l1.ft, 1.0, "linéaire, I0 = 1, graine {graine}");
        assert!(l05.tl_sur_mst >= 5.0, "linéaire, I0 = 0,5, graine {graine} : TL / TL_MST = {}", l05.tl_sur_mst);
        assert!(l1.tl_sur_mst >= 6.0, "linéaire, I0 = 1, graine {graine} : TL / TL_MST = {}", l1.tl_sur_mst);
        assert!(l1.tl_sur_mst > 5.0 * s.tl_sur_mst, "la loi linéaire garde bien plus de tubes que la sigmoïde");
    }
}
