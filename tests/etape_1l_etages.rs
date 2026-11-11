//! Étape 1, partie 1L : les puits entre deux étages. Valeurs attendues : `references/etape-1l-valeurs.md`, identifiants
//! `V-ETA-xx`.
//!
//! **Ce que le modèle permet de dire.** Un lien entre étages (un « puits » du dessin) est un tube comme un autre, de
//! coût `cout_vertical`. Deux étages, la source et le puits sur celui du bas, celui du haut moins cher (`cout_etage 1
//! f`, `f < 1`) : le plus court chemin **monte puis redescend** si `2 c + L_haut < L_bas` (`c` le coût vertical,
//! `L_haut = f * L_bas` quand le chemin du haut a la même étendue), sinon il **reste en bas**. Avec la loi linéaire
//! `f(Q) = |Q|`, le théorème de Bonifaci, Mehlhorn et Varma (2012) dit que la dynamique est attirée par les flux de
//! coût minimal : **une seule route survit**. Les liens verticaux vifs sont donc au nombre de **deux** (monter,
//! redescendre) ou de **zéro**, et rien entre les deux : c'est la seule chose que la loi linéaire permet de dire des
//! puits. La loi sigmoïde de la partie « les réseaux » gardera plusieurs routes ; ces tests-ci n'en disent rien.
//! L'article de Nakagaki n'a aucun étage : tout ceci est à nous (voir `MODELE.md`).
//!
//! **Les valeurs de `c` écartées.** Les coûts `c` entre 4 et 6 (`2 c + 10` entre 18 et 22, contre 20) sont hors de la
//! suite : deux chemins presque égaux (à moins de 10 %) convergent lentement et leur tri dépend de la graine, et à
//! l'égalité (`c = 5`) le théorème ne choisit pas. La mesure à 5 % (`c = 4,5` et `5,5`) donne encore le bon nombre, à
//! 300 pas, sur 50 graines, mais rien ne le garantit : on ne l'écrit pas.

use physarum_lab::alea::Alea;
use physarum_lab::chemins::plus_courts;
use physarum_lab::graphe::{Grille, Voisinage};
use physarum_lab::labyrinthe::{CoutEtage, ErreurLabyrinthe, Mur, Plan, construire};
use physarum_lab::tubes::{PLANCHER, Tubes};

/// Les coûts verticaux de la suite : de `0,1` à `10` (les bornes des scénarios), sans la zone de `4` à `6` (voir plus
/// haut). Sept coûts « qui montent » (`2 c + 10 < 20`, marge d'au moins 10 %) et trois « qui restent en bas ».
const SUITE: [f64; 10] = [0.1, 0.25, 0.5, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 10.0];

/// Deux étages de 21 × 3 cases, le facteur 0,5 à l'étage du haut, la source `(0, 1)` et le puits `(20, 1)` à l'étage du
/// bas, un lien vertical de longueur 1 et de coût `c`.
fn etages(c: f64) -> (Grille, Plan) {
    let g = Grille {
        largeur: 21,
        hauteur: 3,
        etages: 2,
        voisinage: Voisinage::Quatre,
        longueur_verticale: 1.0,
        cout_vertical: c,
    };
    let plan =
        construire(&g, &[], &[CoutEtage { etage: 1, facteur: 0.5 }], g.indice(0, 1, 0), g.indice(20, 1, 0)).unwrap();
    (g, plan)
}

fn verticale(plan: &Plan, e: usize) -> bool {
    let a = plan.graphe.aretes()[e];
    plan.graphe.noeuds()[a.a].etage != plan.graphe.noeuds()[a.b].etage
}

/// Les arêtes du plus court chemin, triées.
fn chemin_de(plan: &Plan) -> Vec<usize> {
    let mut chemin = plus_courts(&plan.graphe, plan.source).vers(&plan.graphe, plan.puits).unwrap();
    chemin.sort_unstable();
    chemin
}

/// juste : le plus court chemin monte puis redescend si `2 c + L_haut < L_bas`, sinon il reste en bas (V-ETA-01), **par
/// Dijkstra** (`chemins::plus_courts`), pour chaque `c` de la suite : `L_bas = 20`, `L_haut = 0,5 * 20 = 10`. Il y a
/// alors 22 arêtes (deux liens verticaux et vingt arêtes de l'étage du haut, chacune de coût 0,5) ou 20 arêtes (le
/// chemin du bas), la distance est `2 c + 10` ou `20` (à `1e-9` près : une somme de 22 termes), et les liens verticaux
/// du chemin sont ceux de la colonne 0 et de la colonne 20 sur la ligne 1. La longueur géométrique (ce que `mesures`
/// compte) ne dépend pas du facteur : 22 ou 20. Les coûts de la suite sont à plus de 10 % du seuil `c = 5`.
///
/// Ne couvre pas : la dynamique (voir `les_liens_vifs_ne_croissent_jamais_avec_le_cout_vertical`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn le_plus_court_chemin_monte_ou_reste() {
    for c in SUITE {
        let (g, plan) = etages(c);
        let (l_bas, l_haut) = (20.0, 0.5 * 20.0);
        let monte = 2.0 * c + l_haut < l_bas;
        assert_eq!(monte, c < 5.0);
        assert!(((2.0 * c + l_haut) - l_bas).abs() >= 2.0, "c = {c} : à plus de 10 % du seuil");
        let chemins = plus_courts(&plan.graphe, plan.source);
        let attendu = if monte { 2.0 * c + 10.0 } else { 20.0 };
        assert!(
            (chemins.distances[plan.puits] - attendu).abs() < 1e-9,
            "c = {c} : distance {}",
            chemins.distances[plan.puits]
        );
        let chemin = chemin_de(&plan);
        let verticales: Vec<usize> = chemin.iter().copied().filter(|&e| verticale(&plan, e)).collect();
        assert_eq!(chemin.len(), if monte { 22 } else { 20 }, "c = {c}");
        assert_eq!(verticales.len(), if monte { 2 } else { 0 }, "c = {c}");
        let longueur: f64 = chemin.iter().map(|&e| plan.graphe.aretes()[e].longueur).sum();
        assert_eq!(longueur, chemin.len() as f64, "toutes les longueurs valent 1");
        if monte {
            let colonnes: Vec<(i32, i32)> = verticales
                .iter()
                .map(|&e| {
                    let n = plan.graphe.noeuds()[plan.graphe.aretes()[e].a];
                    (n.x, n.y)
                })
                .collect();
            assert_eq!(colonnes, [(0, 1), (20, 1)], "c = {c} : on monte en 0 et on redescend en 20");
        }
        assert_eq!(g.cout_vertical, c);
    }
}

/// juste + mesure : le nombre de liens verticaux vifs (`D >= 0,5`) après `N = 300` pas **ne croît jamais quand `c`
/// croît** (V-ETA-02), et vaut ce que dit Dijkstra : 2 pour les coûts qui montent, 0 pour ceux qui restent. Graines 1 à
/// 8, la suite de dix coûts, `h = 0,5`. Plus précisément, pour chaque graine et chaque `c` : les arêtes vives sont
/// exactement celles du plus court chemin, celles-ci sont `>= 0,9999` et toutes les autres `<= 3e-5` (50 graines, 1000
/// à 1049 : `0,999985` au plus bas, `9,95e-6` au plus haut : les fuites du plancher) ; la pression de la source est
/// entre la distance (`2 c + 10` ou `20`) et la distance plus `1e-5` (50 graines : jusqu'à `+3e-6`) : le potentiel
/// source-puits tend vers la longueur du chemin par le haut (Bonifaci et al. 2012, théorème 2).
///
/// Ne couvre pas : les coûts à moins de 10 % du seuil (voir la doc du fichier), ni plus d'une route.
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn les_liens_vifs_ne_croissent_jamais_avec_le_cout_vertical() {
    let mut comptes = vec![Vec::new(); 8];
    for c in SUITE {
        let (_, plan) = etages(c);
        let chemin = chemin_de(&plan);
        let distance = plus_courts(&plan.graphe, plan.source).distances[plan.puits];
        for (k, graine) in (1..=8_u64).enumerate() {
            let mut tubes = Tubes::au_hasard(&plan.graphe, &mut Alea::new(graine));
            let mut pression = 0.0;
            for _ in 0..300 {
                pression =
                    tubes.avancer(&plan.graphe, plan.source, plan.puits, 1.0, 0.5).unwrap().pressions[plan.source];
            }
            let d = tubes.conductivites();
            let vifs: Vec<usize> = (0..d.len()).filter(|&e| d[e] >= 0.5).collect();
            assert_eq!(vifs, chemin, "c = {c}, graine {graine} : les tubes vifs sont le plus court chemin");
            assert!(chemin.iter().all(|&e| d[e] >= 0.9999), "c = {c}, graine {graine} : le chemin près du débit");
            let reste = (0..d.len()).filter(|e| !chemin.contains(e)).map(|e| d[e]).fold(0.0, f64::max);
            assert!(
                (PLANCHER..=3e-5).contains(&reste),
                "c = {c}, graine {graine} : le reste près du plancher ({reste})"
            );
            assert!(
                pression >= distance - 1e-9 && pression <= distance + 1e-5,
                "c = {c}, graine {graine} : pression {pression} pour une distance de {distance}"
            );
            let verticaux = vifs.iter().filter(|&&e| verticale(&plan, e)).count();
            assert_eq!(verticaux, if c < 5.0 { 2 } else { 0 }, "c = {c}, graine {graine}");
            comptes[k].push(verticaux);
        }
    }
    for (k, suite) in comptes.iter().enumerate() {
        assert_eq!(suite.len(), SUITE.len());
        assert!(
            suite.windows(2).all(|w| w[0] >= w[1]),
            "graine {}: les liens vifs croissent avec c : {suite:?}",
            k + 1
        );
        assert_eq!(suite[0], 2, "des liens vifs au début de la suite : pas un code nul");
        assert_eq!(*suite.last().unwrap(), 0, "aucun à la fin");
    }
}

/// juste + mesure : le pont (V-ETA-03) : un mur sur toute la colonne 7 de l'étage du bas d'une grille 15 × 5 à deux
/// étages (liens de longueur 1 et de coût 1, facteur 0,5 à l'étage du haut) coupe le passage ; la source `(2, 2)` et le
/// puits `(12, 2)` sont à l'étage du bas. La seule route passe par l'étage du haut : le plus court chemin (Dijkstra) a
/// 12 arêtes, deux liens verticaux (en `(2, 2)` et en `(12, 2)`) et dix arêtes du haut, à la distance exacte `1 + 10 *
/// 0,5 + 1 = 7`. Après 300 pas, graines 1 à 12, les arêtes vives sont exactement celles-là, `>= 0,9999` (50 graines :
/// `0,999994` au plus bas), les autres `<= 1e-5` (`2,18e-6` au plus haut) et la pression de la source est comprise
/// entre 7 et `7,00001`. Sans l'étage du haut (le même mur aussi à l'étage 1), la source est coupée du puits :
/// `SourceCoupee`. Sans le mur, la route directe par le bas (10) est plus longue que par le haut (7) : le plus court
/// chemin monte aussi.
///
/// Ne couvre pas : un pont plus long que 15 cellules.
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn le_pont_par_l_etage_du_haut() {
    let g = Grille {
        largeur: 15,
        hauteur: 5,
        etages: 2,
        voisinage: Voisinage::Quatre,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let haut = [CoutEtage { etage: 1, facteur: 0.5 }];
    let (s, t) = (g.indice(2, 2, 0), g.indice(12, 2, 0));
    let mur_bas = Mur { x0: 7, y0: 0, x1: 7, y1: 4, etage: 0 };
    let plan = construire(&g, &[mur_bas], &haut, s, t).unwrap();
    let chemins = plus_courts(&plan.graphe, plan.source);
    assert_eq!(chemins.distances[plan.puits], 7.0);
    let chemin = chemin_de(&plan);
    assert_eq!(chemin.len(), 12);
    let verticales: Vec<usize> = chemin.iter().copied().filter(|&e| verticale(&plan, e)).collect();
    let positions: Vec<(i32, i32)> = verticales
        .iter()
        .map(|&e| {
            let n = plan.graphe.noeuds()[plan.graphe.aretes()[e].a];
            (n.x, n.y)
        })
        .collect();
    assert_eq!(positions, [(2, 2), (12, 2)]);
    for graine in 1..=12_u64 {
        let mut tubes = Tubes::au_hasard(&plan.graphe, &mut Alea::new(graine));
        let mut pression = 0.0;
        for _ in 0..300 {
            pression = tubes.avancer(&plan.graphe, plan.source, plan.puits, 1.0, 0.5).unwrap().pressions[plan.source];
        }
        let d = tubes.conductivites();
        let vifs: Vec<usize> = (0..d.len()).filter(|&e| d[e] >= 0.5).collect();
        assert_eq!(vifs, chemin, "graine {graine}");
        assert!(chemin.iter().all(|&e| d[e] >= 0.9999), "graine {graine}");
        let reste = (0..d.len()).filter(|e| !chemin.contains(e)).map(|e| d[e]).fold(0.0, f64::max);
        assert!(reste <= 1e-5, "graine {graine} : {reste}");
        assert!((7.0..=7.00001).contains(&pression), "graine {graine} : pression {pression}");
    }
    // sans l'étage du haut : la source est coupée du puits
    let mur_haut = Mur { etage: 1, ..mur_bas };
    assert_eq!(construire(&g, &[mur_bas, mur_haut], &haut, s, t).unwrap_err(), ErreurLabyrinthe::SourceCoupee);
    // sans le mur, la route par le bas (10) est plus longue que par le haut (7)
    let libre = construire(&g, &[], &haut, s, t).unwrap();
    assert_eq!(plus_courts(&libre.graphe, libre.source).distances[libre.puits], 7.0);
}
