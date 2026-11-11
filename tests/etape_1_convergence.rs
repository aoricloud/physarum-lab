//! Étape 1 : la convergence des tubes vers le plus court chemin (le cœur de l'étape). Valeurs attendues :
//! `references/etape-1-valeurs.md`, identifiants `V-CONV-xx`.
//!
//! La source est le **théorème 2** de Bonifaci, Mehlhorn et Varma (J. Theor. Biol. 309, 2012 ; arXiv:1106.0423 v3, lu)
//! : pour `D' = |Q| - D`, la dynamique est attirée par l'ensemble des flux de coût minimal, et si le plus court chemin
//! source-puits est **unique**, elle converge vers le flux de valeur 1 sur ce chemin, avec un potentiel source-puits
//! qui tend vers sa longueur. Tero, Kobayashi et Nakagaki (2007, version d'auteur lue) disent la même chose pour `f`
//! linéaire, quelles que soient les conductivités initiales. Ces tests **constatent** la convergence de notre pas
//! d'Euler à `h` fini et avec un plancher ; ils ne la démontrent pas (la preuve de Becchetti et al. 2013 a d'autres
//! hypothèses).
//!
//! Le chemin est calculé et son unicité vérifiée EN DEHORS du code jugé : `chemins::plus_courts` (éprouvé dans
//! `etape_1_chemins`) pour le chemin, et le Bellman-Ford naïf du test en retirant tour à tour chaque arête du chemin
//! pour le second meilleur, qui doit être plus long d'au moins 5 %.
//!
//! Ce que le plancher change, mesuré : les tubes morts ne tombent pas à zéro mais à `PLANCHER`, et ils laissent passer
//! un flux de l'ordre de `PLANCHER` par tube : les tubes du chemin sont à `1 - 7e-6` et non à `1`, et un tube mort
//! adjacent à `4e-6`. Les tolérances ci-dessous sont des mesures de cette fuite, pas de la précision du solveur.

#[path = "common/graphes_au_hasard.rs"]
mod graphes_au_hasard;

use graphes_au_hasard::{bellman_ford, graphe_au_hasard, second_meilleur};
use physarum_lab::alea::Alea;
use physarum_lab::chemins::plus_courts;
use physarum_lab::graphe::{Arete, Graphe, Grille, Noeud, Voisinage};
use physarum_lab::tubes::{PLANCHER, Tubes};

/// Un graphe connexe au hasard de 10 à 30 nœuds, des coûts dans `[1 ; 3[`, la source 0, le puits `n - 1`, dont le plus
/// court chemin est unique avec une marge de 5 % (sinon `None` : le graphe ne vérifie pas l'hypothèse du théorème).
/// Rend le graphe, le puits, le chemin (indices d'arêtes) et sa longueur.
fn graphe_a_chemin_unique(alea: &mut Alea) -> Option<(Graphe, usize, Vec<usize>, f64)> {
    let n = 10 + alea.borne(21) as usize;
    let g = graphe_au_hasard(alea, n, n, 1.0, 2.0);
    let chemins = plus_courts(&g, 0);
    let chemin = chemins.vers(&g, n - 1)?;
    let longueur = chemins.distances[n - 1];
    assert!(longueur.is_finite(), "le graphe est connexe");
    (second_meilleur(&g, 0, n - 1, &chemin) >= 1.05 * longueur).then_some((g, n - 1, chemin, longueur))
}

/// Ce qu'on mesure après les pas : l'écart maximal des tubes du chemin à `debit`, la plus grande conductivité hors du
/// chemin, et l'écart relatif de la pression de la source à la longueur du chemin.
fn mesures(tubes: &Tubes, chemin: &[usize], debit: f64, pression: f64, longueur: f64) -> (f64, f64, f64) {
    let (mut ecart, mut hors) = (0.0_f64, 0.0_f64);
    for (e, &d) in tubes.conductivites().iter().enumerate() {
        if chemin.contains(&e) {
            ecart = ecart.max((d - debit).abs());
        } else {
            hors = hors.max(d);
        }
    }
    (ecart, hors, (pression - longueur).abs() / longueur)
}

/// Fait `pas` pas et rend la pression de la source du dernier.
fn avancer(tubes: &mut Tubes, g: &Graphe, source: usize, puits: usize, debit: f64, h: f64, pas: usize) -> f64 {
    let mut pression = 0.0;
    for _ in 0..pas {
        pression = tubes.avancer(g, source, puits, debit, h).unwrap().pressions[source];
    }
    pression
}

/// juste : sur 20 graphes connexes au hasard (de 10 à 30 nœuds, coûts réels dans `[1 ; 3[`) dont le plus court chemin
/// est UNIQUE avec 5 % de marge (V-CONV-01 ; la marge est vérifiée par le Bellman-Ford du test, graphe par graphe),
/// après 800 pas de `h = 0,5` et un débit 1 : chaque arête du chemin a une conductivité à moins de `2e-5` de 1, toutes
/// les autres sont à moins de `1e-5` (tout près du plancher `1e-6`), la pression de la source est à moins de `2e-14`
/// (en relatif) de la longueur du chemin, et les arêtes qui valent plus de 0,5 sont exactement celles du chemin. Les
/// conductivités de départ sont `Tubes::au_hasard` (12 graphes), uniformes à 1 (le cas de Becchetti ; 4 graphes) ou
/// larges, dans `[0,01 ; 5,01[` (4 graphes) : le théorème vaut pour toute conductivité de départ strictement positive.
///
/// **Mesures** (prototype, d'autres graines que celles du test, le même générateur de graphes : quatre séries, de 50 à
/// 100 graphes chacune, soit 350 graphes, un pas de 0,5 à 800 pas, les trois modes de départ) : écart maximal des
/// arêtes du chemin 5,3e-6, 6,6e-6, 6,2e-6 et 6,9e-6 selon la série ; maximum hors du chemin 3,9e-6, 3,2e-6, 2,8e-6 et
/// 3,5e-6 ; pression au plus 1,1e-15. Les tubes du chemin ne sont pas à 1 mais à quelques 1e-6 de 1 : c'est la fuite du
/// plancher (voir l'en-tête). La convergence est atteinte vers 500 pas pour les cas les plus lents (à 400 pas : 2,8e-5
/// à 1,5e-4 d'écart). Sur les 20 graphes du test : 5,4e-6, 2,1e-6 et 5e-16. Marge minimale du second meilleur chemin
/// dans les séries : 1,0501 (le filtre l'impose). Tolérances `2e-5` (3 fois la mesure), `1e-5` (2,5 fois) et `2e-14`
/// (18 fois, l'arrondi du solveur).
///
/// Ne couvre pas : les graphes de plus de 30 nœuds ; les `h` hors de `]0 ; 1[` ; le cas de plusieurs plus courts
/// chemins (voir `egalite_de_plusieurs_chemins`) ; la vitesse de convergence (un seul `N` est testé, et la mesure dit
/// qu'elle dépend du graphe).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn chemin_unique_au_hasard() {
    let mut alea = Alea::new(4_242_000);
    let (mut gardes, mut essais) = (0, 0);
    let (mut pire_chemin, mut pire_hors, mut pire_p) = (0.0_f64, 0.0_f64, 0.0_f64);
    while gardes < 20 {
        essais += 1;
        assert!(essais <= 80, "trop de graphes sans marge de 5 % : {essais} essais, {gardes} gardés");
        let Some((g, puits, chemin, longueur)) = graphe_a_chemin_unique(&mut alea) else { continue };
        let mut tubes = match gardes % 5 {
            0..=2 => Tubes::au_hasard(&g, &mut alea),
            3 => Tubes::uniformes(&g, 1.0).unwrap(),
            _ => {
                let d: Vec<f64> = (0..g.nb_aretes()).map(|_| 0.01 + 5.0 * alea.suivant_f64()).collect();
                Tubes::new(&g, d).unwrap()
            }
        };
        gardes += 1;
        let pression = avancer(&mut tubes, &g, 0, puits, 1.0, 0.5, 800);
        let (ecart, hors, rel) = mesures(&tubes, &chemin, 1.0, pression, longueur);
        pire_chemin = pire_chemin.max(ecart);
        pire_hors = pire_hors.max(hors);
        pire_p = pire_p.max(rel);
        let vifs: Vec<usize> = (0..g.nb_aretes()).filter(|&e| tubes.conductivites()[e] > 0.5).collect();
        let mut attendu = chemin.clone();
        attendu.sort_unstable();
        assert_eq!(vifs, attendu, "graphe {gardes} : les arêtes qui dépassent 0,5 sont celles du chemin");
        assert!(tubes.conductivites().iter().all(|&d| d.is_finite() && d >= PLANCHER));
    }
    assert!(pire_chemin <= 2e-5, "écart des arêtes du chemin à 1 : {pire_chemin:e}");
    assert!(pire_hors <= 1e-5, "plus grande conductivité hors du chemin : {pire_hors:e}");
    assert!(pire_p <= 2e-14, "pression de la source contre la longueur du chemin : {pire_p:e}");
}

/// juste : le même constat avec un petit pas, `h = 0,1`, et 4 000 pas (V-CONV-02) : 5 graphes au hasard à chemin unique
/// (marge de 5 %), conductivités de départ `Tubes::au_hasard`, mêmes tolérances que `chemin_unique_au_hasard`. Mesure
/// du prototype (100 autres graphes, 4 000 pas) : 6,6e-6, 3,2e-6 et 6,3e-16 ; la convergence demande 2 500 à 3 000 pas
/// pour les plus lents, soit cinq fois plus qu'à `h = 0,5` : le temps du modèle continu est `h * N`.
///
/// Ne couvre pas : un `h` plus grand que 0,5 (le pas d'Euler est stable sur `]0 ; 1[`, pas mesuré au-delà de 0,5 ici).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn petit_pas_de_temps() {
    let mut alea = Alea::new(13_579_000);
    let (mut gardes, mut essais) = (0, 0);
    while gardes < 5 {
        essais += 1;
        assert!(essais <= 30, "trop de graphes sans marge de 5 %");
        let Some((g, puits, chemin, longueur)) = graphe_a_chemin_unique(&mut alea) else { continue };
        gardes += 1;
        let mut tubes = Tubes::au_hasard(&g, &mut alea);
        let pression = avancer(&mut tubes, &g, 0, puits, 1.0, 0.1, 4000);
        let (ecart, hors, rel) = mesures(&tubes, &chemin, 1.0, pression, longueur);
        assert!(ecart <= 2e-5 && hors <= 1e-5 && rel <= 2e-14, "graphe {gardes} : {ecart:e} {hors:e} {rel:e}");
    }
}

/// Une échelle à deux étages : six barreaux `x = 0..5`, chacun avec un nœud en bas (étage 0, indice `2 x`) et un en
/// haut (étage 1, indice `2 x + 1`) ; un lien vertical par barreau (coût 0,5) ; les arêtes de l'étage 0 coûtent 1,
/// celles de l'étage 1 coûtent 0,4 ; deux nœuds pendants (12 : au barreau 2 en bas, 13 : au barreau 4 en haut), reliés
/// chacun par un tube au barreau voisin (coûts 1 et 0,4). Le plus court chemin de `0` à `10` (le bas du dernier
/// barreau) monte tout de suite (0,5), longe l'étage 1 (5 fois 0,4) et redescend (0,5) : longueur 3. Il évite l'étage 0
/// (5) et tout autre point de montée (au moins 3,6).
fn echelle_a_deux_etages() -> Graphe {
    let mut noeuds = Vec::new();
    for x in 0..6 {
        for etage in 0..2 {
            noeuds.push(Noeud { x, y: 0, etage });
        }
    }
    noeuds.push(Noeud { x: 2, y: 1, etage: 0 });
    noeuds.push(Noeud { x: 4, y: 1, etage: 1 });
    let mut aretes = Vec::new();
    let mut tube = |a: usize, b: usize, cout: f64| aretes.push(Arete { a, b, longueur: cout, cout });
    for x in 0..6 {
        tube(2 * x, 2 * x + 1, 0.5);
    }
    for x in 0..5 {
        tube(2 * x, 2 * x + 2, 1.0);
        tube(2 * x + 1, 2 * x + 3, 0.4);
    }
    tube(4, 12, 1.0);
    tube(9, 13, 0.4);
    Graphe::new(noeuds, aretes).unwrap()
}

/// juste : un graphe à deux étages construit à la main avec `Graphe::new` (V-CONV-03, `echelle_a_deux_etages`) : le
/// plus court chemin MONTE par un lien vertical et REDESCEND plus loin (il contient exactement deux liens entre étages
/// et cinq arêtes de l'étage du haut), sa longueur est 3 et le second meilleur est à 3,6 (marge 20 %) ; le test le
/// vérifie avec `plus_courts` et le Bellman-Ford. Après 200 pas de `h = 0,5` (cinq graines de départ
/// `Alea::new(1..=5)`), les sept arêtes du chemin sont à moins de `2e-6` de 1 ; TOUTES les autres, les deux tubes
/// pendants et les arêtes de l'étage du bas comprises, valent exactement `PLANCHER` ; la pression de la source est à
/// moins de `2e-14` (relatif) de 3.
///
/// **Mesures** (prototype, 50 autres graines `1001..=1050`, aux pas 100, 200, 300 et 400 : identiques) : écart des
/// arêtes du chemin 7,7e-7, maximum hors du chemin exactement `1e-6` (le plancher), pression 3,0e-15 ; la convergence
/// est donc atteinte avant 100 pas. Tolérances `2e-6` (2,6 fois), le plancher exact, `2e-14` (7 fois).
///
/// Ne couvre pas : des étages reliés par des liens qui coûtent moins que les arêtes (le plus court chemin y change) ;
/// plus de deux étages.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn chemin_qui_monte_et_redescend() {
    let g = echelle_a_deux_etages();
    let (source, puits) = (0, 10);
    let chemins = plus_courts(&g, source);
    let chemin = chemins.vers(&g, puits).unwrap();
    assert_eq!(chemin.len(), 7);
    let entre_etages = |e: &usize| g.noeuds()[g.aretes()[*e].a].etage != g.noeuds()[g.aretes()[*e].b].etage;
    assert_eq!(chemin.iter().filter(|e| entre_etages(e)).count(), 2, "il monte et redescend");
    assert_eq!(chemin.iter().filter(|&&e| g.noeuds()[g.aretes()[e].a].etage == 1 && !entre_etages(&e)).count(), 5);
    assert!((chemins.distances[puits] - 3.0).abs() < 1e-12);
    let second = second_meilleur(&g, source, puits, &chemin);
    assert!((second - 3.6).abs() < 1e-12 && second >= 1.05 * chemins.distances[puits], "second meilleur {second}");
    assert!((bellman_ford(&g, source, None)[puits] - 3.0).abs() < 1e-12);
    for graine in 1..=5 {
        let mut alea = Alea::new(graine);
        let mut tubes = Tubes::au_hasard(&g, &mut alea);
        let pression = avancer(&mut tubes, &g, source, puits, 1.0, 0.5, 200);
        let (ecart, _, rel) = mesures(&tubes, &chemin, 1.0, pression, 3.0);
        assert!(ecart <= 2e-6, "graine {graine} : écart des arêtes du chemin {ecart:e}");
        for (e, &d) in tubes.conductivites().iter().enumerate() {
            if !chemin.contains(&e) {
                assert_eq!(d.to_bits(), PLANCHER.to_bits(), "graine {graine}, arête {e} : {d:e}");
            }
        }
        assert!(rel <= 2e-14, "graine {graine} : pression, écart relatif {rel:e}");
    }
}

/// juste : un débit autre que 1 (V-CONV-04). Sur l'échelle à deux étages, avec `I = 2,5` et `I = 0,2`, 400 pas de `h =
/// 0,5` : les arêtes du chemin tendent vers `I` (à moins de `2e-6`), les autres sont exactement au plancher, et la
/// pression de la source vers la longueur du chemin, 3, **quel que soit le débit** (à `2e-14` relatif) : à l'équilibre
/// le tube porte `I`, donc la chute de pression `cout * Q / D` vaut son coût.
///
/// **Mesures** (prototype, graine 77) : écart des arêtes du chemin 7,7e-7 pour les deux débits (la fuite du plancher
/// est absolue : elle ne dépend pas de `I`), les autres exactement au plancher, pression 1,3e-15 (`I = 2,5`) et 3,0e-16
/// (`I = 0,2`) ; tolérance `2e-6` sur l'écart, le plancher exact, `2e-14`.
///
/// Ne couvre pas : un débit de plus de 1e3 (le rapport des conductances approcherait de la limite du solveur).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn debit_quelconque() {
    let g = echelle_a_deux_etages();
    let chemin = plus_courts(&g, 0).vers(&g, 10).unwrap();
    for debit in [2.5, 0.2] {
        let mut alea = Alea::new(77);
        let mut tubes = Tubes::au_hasard(&g, &mut alea);
        let pression = avancer(&mut tubes, &g, 0, 10, debit, 0.5, 400);
        let (ecart, hors, rel) = mesures(&tubes, &chemin, debit, pression, 3.0);
        assert!(ecart <= 2e-6, "I = {debit} : écart des arêtes du chemin {ecart:e}");
        for (e, &d) in tubes.conductivites().iter().enumerate() {
            if !chemin.contains(&e) {
                assert_eq!(d.to_bits(), PLANCHER.to_bits(), "I = {debit}, arête {e} : {d:e}");
            }
        }
        assert!(hors <= PLANCHER);
        assert!(rel <= 2e-14, "I = {debit} : pression, écart relatif {rel:e}");
    }
}

/// juste : plusieurs plus courts chemins de même longueur (V-CONV-05). Le théorème ne dit alors que « attiré par
/// l'ensemble des flux de coût minimal » (Bonifaci et al. 2012, théorème 2 : « The dynamics are attracted by `E*` » ;
/// ils conjecturent la convergence vers un point de `E*` sans la montrer), donc le test ne vérifie que ce qui en
/// découle. Grille 5 × 4 en voisinage 4, source `(1, 1)`, puits `(3, 2)` : `G0`, le sous-graphe des arêtes de plus
/// courts chemins (distance de la source plus coût plus distance au puits égale la longueur 3), a 7 arêtes et trois
/// plus courts chemins. Après 400 pas de `h = 0,5`, pour cinq graines de départ :
/// - toute arête hors de `G0` vaut exactement `PLANCHER` ;
/// - la pression de la source est à moins de `2e-14` de 3 (relatif), et `somme(cout * D)` (la fonction de Lyapunov
///   de Bonifaci) à moins de `5e-5` de 3 ;
/// - **les conductivités de `G0`, orientées de la source vers le puits, forment un flux de valeur 1** : en chaque nœud
///   de `G0` autre que la source et le puits, ce qui entre égale ce qui sort (à `4e-6` près), la source en émet 1 et le
///   puits en reçoit 1. C'est exactement « D est proche d'un flux de coût minimal » : tout flux de valeur 1 dans `G0`
///   coûte 3.
///
/// Et le constat que le théorème ne contredit pas, mais qu'il n'affirme pas : la répartition entre les chemins **dépend
/// des conductivités de départ** (Tero 2007, tableau p. 18 : pour deux chemins de même longueur, « un point quelconque
/// du segment » selon l'état initial) : la conductivité de la première arête de la source dans `G0` diffère de plus de
/// 0,05 d'une graine à l'autre.
///
/// **Mesures** (prototype, cinq graines de départ `1..=5`, 100 à 2 000 pas : identiques) : hors de `G0`, exactement le
/// plancher ; pression au plus 3e-16 (relatif) ; `somme(cout * D)` à 2,09e-5 de 3 (les 17 arêtes hors de `G0` y
/// ajoutent chacune `1e-6`, et celles de `G0` en retirent un peu) ; résidu de conservation au plus 9,5e-7 ; première
/// arête de `G0` à 0,566 ; 0,589 ; 0,593 ; 0,656 ; 0,678 (écart 0,11). Tolérances : `2e-14`, `5e-5` (2,4 fois), `4e-6`
/// (4 fois) et 0,05 (la moitié).
///
/// Ne couvre pas : que `D` converge vers un point (non prouvé par l'article) ; un graphe où `G0` a plus de 7 arêtes.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn egalite_de_plusieurs_chemins() {
    let grille = Grille {
        largeur: 5,
        hauteur: 4,
        etages: 1,
        voisinage: Voisinage::Quatre,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let g = grille.graphe().unwrap();
    let (source, puits) = (grille.indice(1, 1, 0), grille.indice(3, 2, 0));
    let depuis_source = plus_courts(&g, source);
    let depuis_puits = plus_courts(&g, puits);
    let longueur = depuis_source.distances[puits];
    assert_eq!(longueur, 3.0);
    // G0 : l'arête (a, b) est dans un plus court chemin si d_s(a) + cout + d_p(b) = L (ou en sens inverse) ; on
    // l'oriente dans le sens de la source vers le puits
    let mut orientees: Vec<(usize, usize, usize)> = Vec::new(); // (arête, nœud de départ, nœud d'arrivée)
    for (e, arete) in g.aretes().iter().enumerate() {
        let (a, b) = (arete.a, arete.b);
        if depuis_source.distances[a] + arete.cout + depuis_puits.distances[b] == longueur {
            orientees.push((e, a, b));
        } else if depuis_source.distances[b] + arete.cout + depuis_puits.distances[a] == longueur {
            orientees.push((e, b, a));
        }
    }
    assert_eq!(orientees.len(), 7, "G0 a sept arêtes");
    let mut premiere = Vec::new();
    for graine in 1..=5 {
        let mut alea = Alea::new(graine);
        let mut tubes = Tubes::au_hasard(&g, &mut alea);
        let pression = avancer(&mut tubes, &g, source, puits, 1.0, 0.5, 400);
        let d = tubes.conductivites();
        assert_eq!(d.len(), g.nb_aretes(), "une conductivité par arête");
        for (e, &valeur) in d.iter().enumerate() {
            if !orientees.iter().any(|&(f, _, _)| f == e) {
                assert_eq!(valeur.to_bits(), PLANCHER.to_bits(), "graine {graine}, arête {e} hors de G0 : {valeur:e}");
            }
        }
        let rel = (pression - longueur).abs() / longueur;
        let lyapunov: f64 = g.aretes().iter().zip(d).map(|(a, &v)| a.cout * v).sum();
        assert!(rel <= 2e-14, "graine {graine} : pression {pression}");
        assert!((lyapunov - longueur).abs() <= 5e-5, "graine {graine} : Lyapunov {lyapunov}");
        for v in 0..g.nb_noeuds() {
            let entrant: f64 = orientees.iter().filter(|&&(_, _, fin)| fin == v).map(|&(e, _, _)| d[e]).sum();
            let sortant: f64 = orientees.iter().filter(|&&(_, debut, _)| debut == v).map(|&(e, _, _)| d[e]).sum();
            let attendu = if v == source {
                -1.0
            } else if v == puits {
                1.0
            } else {
                0.0
            };
            let residu = (entrant - sortant - attendu).abs();
            assert!(residu <= 4e-6, "graine {graine}, nœud {v} : entrant {entrant}, sortant {sortant}");
        }
        let d_premiere = orientees.iter().find(|&&(_, debut, _)| debut == source).map(|&(e, _, _)| d[e]).unwrap();
        premiere.push(d_premiere);
    }
    let (petit, grand) = premiere.iter().fold((f64::INFINITY, 0.0_f64), |(p, g), &v| (p.min(v), g.max(v)));
    assert!(grand - petit > 0.05, "la répartition ne dépend pas des conductivités de départ : {premiere:?}");
}
