//! Étape 1, partie 1L : le labyrinthe de Nakagaki sur une grille (`scenarios/labyrinthe.txt`). Valeurs attendues :
//! `references/etape-1l-valeurs.md`, identifiants `V-NAK-xx`.
//!
//! **Source.** Nakagaki, Yamada, Tóth, Nature 407, 470 (2000), page imprimée 470 (la seule lue : voir `MODELE.md`).
//! Elle donne les plus courts chemins de quatre « routes » : α1 41 mm, α2 33 mm, β1 44 mm, β2 45 mm (± 1), dit que la
//! route par α2, environ 22 % plus courte que celle par α1, est toujours choisie, et que β1 et β2 sont à peu près
//! également peuplées parce que leurs longueurs (environ 2 %) se perdent dans les méandres du tube. **Notre lecture,
//! écrite comme telle** : le tableau de la Fig. 1d croise les α (lignes) et les β (colonnes), donc on lit les routes
//! **en série** : de la source à une jonction par α1 ou α2, puis de la jonction au puits par β1 ou β2. L'article ne
//! décrit pas le dessin : celui d'ici est à nous (deux cellules par millimètre, voisinage 4, couloirs d'une cellule :
//! 82, 66, 88 et 90 cellules ; trois impasses, que l'article montre se rétracter).
//!
//! **Ce que le modèle fait, et ce que l'article dit.** L'article n'a aucun chiffre de dynamique à reproduire : nos
//! tests constatent ce que la loi linéaire fait de ce dessin, avec des tolérances mesurées sur 50 graines autres que
//! celles du test. Chez nous, sans bruit, la route la plus courte (α2 puis β1) est retenue dans tous les cas, α1 meurt
//! vite, et β2, qui n'est que 2 % plus longue que β1, meurt neuf fois plus lentement qu'α1 mais **finit par mourir** :
//! « à peu près le même nombre de tubes » chez β1 et β2, dit l'article, est ce qu'on voit au début (à 20 pas, `0,54` et
//! `0,46`), pas à la fin. Les chiffres de l'article ne sont pas atteints (il n'en donne pas pour ce temps-là).

use physarum_lab::alea::Alea;
use physarum_lab::chemins::plus_courts;
use physarum_lab::graphe::{Grille, Voisinage};
use physarum_lab::labyrinthe::{Mur, Plan, construire};
use physarum_lab::scenario::{Scenario, lire_scenario};
use physarum_lab::tubes::{PLANCHER, Tubes};

/// Le labyrinthe : `#` un mur, `S` la source, `T` le puits, `J` la jonction, et pour chaque couloir sa route : `a` α1
/// (82 arêtes), `A` α2 (66), `b` β1 (88), `B` β2 (90), `x` les impasses (17 arêtes). Dessiné par
/// `outils-1L/maze_nakagaki.py`, recoupé avec `scenarios/labyrinthe.txt` par `le_dessin_est_celui_du_scenario`.
const DESSIN: [&str; 29] = [
    "#########################################",
    "#####################################x###",
    "#####################################x###",
    "##x##################################x###",
    "##x#################BBB#BBB#BBB#BBB#BBB##",
    "##x#################B#B#B#B#B#B#B#B#B#B##",
    "##x#################B#B#B#B#B#B#B#B#B#B##",
    "##x#################B#B#B#B#B#B#B#B#B#B##",
    "##x#AAA#AAA#AAA#AAA#B#B#B#B#B#B#B#B#B#B##",
    "##x#A#A#A#A#A#A#A#A#B#B#B#B#B#B#B#B#B#B##",
    "##x#A#A#A#A#A#A#A#A#B#B#B#B#B#BBB#BBB#B##",
    "##x#A#A#A#A#A#A#A#A#B#BBB#BBB#########B##",
    "##x#A#A#A#A#A#A#A#A#B#################B##",
    "##x#A#A#A#A#A#A#A#A#B#################B##",
    "##SAA#AAA#AAA#AAA#AAJbb#bbb#bbb#bbb#bbT##",
    "##a#################a#b#b#b#b#b#b#b#b####",
    "##a#################a#b#b#b#b#b#b#b#b####",
    "##a#################a#b#b#b#b#b#b#b#b####",
    "##a#aaa#aaa#########a#b#b#b#b#b#b#b#b####",
    "##a#a#a#a#a#aaa#aaa#a#b#b#b#b#b#b#b#b####",
    "##a#a#a#a#a#a#a#a#a#a#b#b#b#b#b#b#b#b####",
    "##a#a#a#a#a#a#a#a#a#a#b#b#b#b#b#b#b#b####",
    "##a#a#a#a#a#a#a#a#a#a#b#b#b#b#b#b#bbb####",
    "##a#a#a#a#a#a#a#a#a#a#bbb#bbb#bbb########",
    "##aaa#aaa#aaa#aaa#aaa####################",
    "###########x#############################",
    "###########x#############################",
    "###########x#############################",
    "#########################################",
];

/// Un couloir coupé : une cellule de chaque route, hors des extrémités.
const COUPE_ALPHA1: (u32, u32) = (2, 20);
const COUPE_ALPHA2: (u32, u32) = (7, 14);
const COUPE_BETA1: (u32, u32) = (21, 14);
const COUPE_BETA2: (u32, u32) = (20, 10);

fn grille() -> Grille {
    Grille {
        largeur: DESSIN[0].len() as u32,
        hauteur: DESSIN.len() as u32,
        etages: 1,
        voisinage: Voisinage::Quatre,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    }
}

/// La cellule `(x, y)` du dessin.
fn cellule(x: u32, y: u32) -> u8 {
    DESSIN[y as usize].as_bytes()[x as usize]
}

/// Le plan du labyrinthe, avec des murs de plus (les coupures) ; la source est `S`, le puits `T`.
fn plan_avec(coupures: &[(u32, u32)]) -> Plan {
    let g = grille();
    let mut murs = Vec::new();
    for y in 0..g.hauteur {
        for x in 0..g.largeur {
            if cellule(x, y) == b'#' {
                murs.push(Mur { x0: x, y0: y, x1: x, y1: y, etage: 0 });
            }
        }
    }
    for &(x, y) in coupures {
        assert!(cellule(x, y) != b'#' && !b"STJ".contains(&cellule(x, y)), "une coupure est dans un couloir");
        murs.push(Mur { x0: x, y0: y, x1: x, y1: y, etage: 0 });
    }
    construire(&g, &murs, &[], g.indice(2, 14, 0), g.indice(38, 14, 0)).expect("le plan du labyrinthe")
}

/// Le nœud du plan à la cellule `(x, y)`.
fn noeud(plan: &Plan, x: u32, y: u32) -> usize {
    plan.noeuds[grille().indice(x, y, 0)].expect("la cellule est un couloir")
}

/// La route d'une arête du plan : `a`, `A`, `b`, `B` ou `x` (une arête qui touche une cellule d'impasse est une arête
/// d'impasse ; les autres suivent la route de leur bout qui n'est ni `S`, ni `J`, ni `T`).
fn route(plan: &Plan, e: usize) -> u8 {
    let arete = plan.graphe.aretes()[e];
    let lettre = |i: usize| {
        let n = plan.graphe.noeuds()[i];
        cellule(n.x as u32, n.y as u32)
    };
    let (la, lb) = (lettre(arete.a), lettre(arete.b));
    if la == b'x' || lb == b'x' {
        b'x'
    } else if b"aAbB".contains(&la) {
        la
    } else {
        lb
    }
}

fn indices_de(plan: &Plan, r: u8) -> Vec<usize> {
    (0..plan.graphe.nb_aretes()).filter(|&e| route(plan, e) == r).collect()
}

fn scenario_du_depot() -> (String, Scenario) {
    let texte = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/scenarios/labyrinthe.txt")).unwrap();
    let scenario = lire_scenario(&texte).expect("le scénario du labyrinthe est lisible");
    (texte, scenario)
}

/// L'état de `plan` après `pas` pas de `h = 0,5` pour la graine donnée, depuis les conductivités de `Tubes::au_hasard`.
fn apres(plan: &Plan, graine: u64, pas: usize) -> Vec<f64> {
    let mut tubes = Tubes::au_hasard(&plan.graphe, &mut Alea::new(graine));
    for _ in 0..pas {
        tubes.avancer(&plan.graphe, plan.source, plan.puits, 1.0, 0.5).unwrap();
    }
    tubes.conductivites().to_vec()
}

fn extremes(d: &[f64], aretes: &[usize]) -> (f64, f64) {
    assert!(!aretes.is_empty());
    let valeurs = aretes.iter().map(|&e| d[e]);
    (valeurs.clone().fold(f64::INFINITY, f64::min), valeurs.fold(0.0, f64::max))
}

/// juste : le dessin du test est celui du scénario (V-NAK-01). Les cellules murées du dessin (`#`) sont exactement
/// l'union des rectangles des directives `mur` de `scenarios/labyrinthe.txt` (chacune lue par `Directive::entier`) ; la
/// grille est de 41 × 29 en voisinage 4 à un étage ; la source est `S` `(2, 14)` et le puits `T` `(38, 14)` ; `pas 500
/// 0.5`. Le dessin a 342 couloirs, dont `S`, `J` `(20, 14)` et `T`. Un code qui lirait mal les directives `mur` échoue
/// ici.
///
/// Ne couvre pas : que le dessin soit un labyrinthe à quatre routes (voir `les_quatre_longueurs`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn le_dessin_est_celui_du_scenario() {
    let (_, scenario) = scenario_du_depot();
    let g = scenario.grille;
    assert_eq!((g.largeur, g.hauteur, g.etages, g.voisinage), (41, 29, 1, Voisinage::Quatre));
    let mut murees = vec![false; 41 * 29];
    let mut lignes_mur = 0;
    for d in scenario.directives.iter().filter(|d| d.nom == "mur") {
        assert_eq!(d.args.len(), 5);
        let v: Vec<u64> = (0..5).map(|i| d.entier(i).unwrap()).collect();
        assert_eq!(v[4], 0, "tous les murs sont à l'étage 0");
        for y in v[1]..=v[3] {
            for x in v[0]..=v[2] {
                assert!(!murees[(y * 41 + x) as usize], "deux murs ne se recouvrent pas dans ce scénario");
                murees[(y * 41 + x) as usize] = true;
            }
        }
        lignes_mur += 1;
    }
    assert_eq!(lignes_mur, 52);
    for y in 0..29_u32 {
        for x in 0..41_u32 {
            assert_eq!(murees[(y * 41 + x) as usize], cellule(x, y) == b'#', "la cellule ({x}, {y})");
        }
    }
    let nom = |n: &str| -> Vec<u64> {
        let d = scenario.directive(n).unwrap_or_else(|| panic!("directive {n}"));
        (0..d.args.len()).map(|i| d.entier(i).unwrap()).collect()
    };
    assert_eq!(nom("source"), [2, 14, 0]);
    assert_eq!(nom("puits"), [38, 14, 0]);
    assert_eq!((cellule(2, 14), cellule(38, 14), cellule(20, 14)), (b'S', b'T', b'J'));
    let d = scenario.directive("pas").unwrap();
    assert_eq!((d.entier(0).unwrap(), d.flottant(1).unwrap()), (500, 0.5));
    assert!(scenario.directive("cout_etage").is_none());
    let libres = DESSIN.iter().flat_map(|l| l.bytes()).filter(|&c| c != b'#').count();
    assert_eq!(libres, 342);
    let plan = plan_avec(&[]);
    assert_eq!((plan.graphe.nb_noeuds(), plan.graphe.nb_aretes()), (342, 343));
    assert_eq!((plan.source, plan.puits), (noeud(&plan, 2, 14), noeud(&plan, 38, 14)));
}

/// juste : les quatre plus courts chemins sont **exactement** 82, 66, 88 et 90 (V-NAK-02), mesurés par
/// `chemins::plus_courts` sur le graphe du plan en coupant tour à tour les autres routes (un mur d'une cellule au
/// milieu d'un couloir : le dessin dit où). Source `S` à jonction `J` : 82 quand α2 est coupée, 66 quand α1 l'est ; de
/// `J` au puits `T` : 88 quand β2 est coupée, 90 quand β1 l'est. Ce sont 41, 33, 44 et 45 mm à deux cellules par
/// millimètre (article : 41, 33, 44 et 45 ± 1 mm, page 470). Le plus court chemin entier est α2 puis β1 : 154, et les
/// quatre combinaisons coupées donnent 170, 172, 154 et 156. Le nombre d'arêtes de chaque route (82, 66, 88, 90, et 17
/// d'impasses) recoupe.
///
/// Ne couvre pas : le dessin de l'article, que nous n'avons pas (notre lecture en série est dite dans la doc du
/// fichier).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn les_quatre_longueurs() {
    let longueur = |coupures: &[(u32, u32)], de: (u32, u32), vers: (u32, u32)| -> f64 {
        let plan = plan_avec(coupures);
        let chemins = plus_courts(&plan.graphe, noeud(&plan, de.0, de.1));
        chemins.distances[noeud(&plan, vers.0, vers.1)]
    };
    let (s, jonction, t) = ((2, 14), (20, 14), (38, 14));
    assert_eq!(longueur(&[COUPE_ALPHA2], s, jonction), 82.0, "α1 : 41 mm");
    assert_eq!(longueur(&[COUPE_ALPHA1], s, jonction), 66.0, "α2 : 33 mm");
    assert_eq!(longueur(&[COUPE_BETA2], jonction, t), 88.0, "β1 : 44 mm");
    assert_eq!(longueur(&[COUPE_BETA1], jonction, t), 90.0, "β2 : 45 mm");
    assert_eq!(longueur(&[], s, t), 154.0);
    assert_eq!(longueur(&[COUPE_ALPHA2, COUPE_BETA2], s, t), 170.0);
    assert_eq!(longueur(&[COUPE_ALPHA2, COUPE_BETA1], s, t), 172.0);
    assert_eq!(longueur(&[COUPE_ALPHA1, COUPE_BETA2], s, t), 154.0);
    assert_eq!(longueur(&[COUPE_ALPHA1, COUPE_BETA1], s, t), 156.0);
    // les quatre routes sont exactement des chemins : leurs arêtes se comptent
    let plan = plan_avec(&[]);
    let comptes: Vec<usize> = b"aAbBx".iter().map(|&r| indices_de(&plan, r).len()).collect();
    assert_eq!(comptes, [82, 66, 88, 90, 17]);
    // le plus court chemin entier : ses arêtes sont celles de α2 puis de β1
    let chemin = plus_courts(&plan.graphe, plan.source).vers(&plan.graphe, plan.puits).unwrap();
    assert_eq!(chemin.len(), 154);
    assert!(chemin.iter().all(|&e| route(&plan, e) == b'A' || route(&plan, e) == b'b'));
}

/// juste + mesure : la route la plus courte est choisie, **quelle que soit la graine** des conductivités de départ
/// (V-NAK-03). Pour les graines 1 à 12, après 150 pas de `h = 0,5` : toutes les arêtes d'α1 sont exactement au
/// `PLANCHER` (un tube mort y reste : sa conductivité décroît comme `(1 - h)^n` puis le flux qui le traverse, `D` fois
/// le rapport des longueurs `66/82 < 1`, ne la relève pas), toutes celles d'α2 sont `>= 0,999` (mesuré sur 50 graines :
/// `1,000` à quatre chiffres), et à 200 pas les tubes vifs (`>= 0,5`) sont **exactement les arêtes du plus court
/// chemin** (`chemins::plus_courts` : α2 puis β1), ni plus, ni moins : β1 est entre `0,89` et `0,91` et β2 entre `0,09`
/// et `0,11` (50 graines : `0,8973` à `0,9094` et `0,0906` à `0,1027`). Les valeurs de la graine 1 (`D` d'α2 `1,0`
/// au-delà de `1e-9`, β1 `0,84785`, β2 `0,15215` à 150 pas ; `0,90697` et `0,093027` à 200) sont celles du modèle
/// Python.
///
/// Ne couvre pas : une graine tirée parce qu'elle passe (les graines 1 à 12 sont écrites avant la mesure, et les 50 de
/// la mesure sont d'autres : 1000 à 1049).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn alpha2_est_choisie_quelle_que_soit_la_graine() {
    let plan = plan_avec(&[]);
    let routes: Vec<Vec<usize>> = b"aAbBx".iter().map(|&r| indices_de(&plan, r)).collect();
    let chemin = plus_courts(&plan.graphe, plan.source).vers(&plan.graphe, plan.puits).unwrap();
    assert_eq!(chemin.len(), 154);
    for graine in 1..=12_u64 {
        let d = apres(&plan, graine, 150);
        assert!(routes[0].iter().all(|&e| d[e] == PLANCHER), "graine {graine} : α1 au plancher");
        assert!(routes[1].iter().all(|&e| d[e] >= 0.999), "graine {graine} : α2 proche du débit");
        assert!(routes[4].iter().all(|&e| d[e] == PLANCHER), "graine {graine} : les impasses au plancher");
        let d = apres(&plan, graine, 200);
        let vifs: Vec<usize> = (0..d.len()).filter(|&e| d[e] >= 0.5).collect();
        let mut attendu = chemin.clone();
        attendu.sort_unstable();
        assert_eq!(vifs, attendu, "graine {graine} : les tubes vifs sont le plus court chemin");
        let (bmin, _) = extremes(&d, &routes[2]);
        let (_, bmax) = extremes(&d, &routes[3]);
        assert!((0.89..=0.91).contains(&bmin), "graine {graine} : β1 {bmin}");
        assert!((0.09..=0.11).contains(&bmax), "graine {graine} : β2 {bmax}");
    }
    // la graine 1, au modèle Python près : β1 et β2 à 150 et à 200 pas (quatre chiffres, tubes de même valeur à 1e-12)
    for (pas, b1, b2) in [(150, 0.84785, 0.15215), (200, 0.90697, 0.093027)] {
        let d = apres(&plan, 1, pas);
        let (lo, hi) = extremes(&d, &routes[2]);
        assert!((lo - b1).abs() < 1e-5 && hi - lo < 1e-12, "β1 à {pas} pas : {lo}");
        let (lo, hi) = extremes(&d, &routes[3]);
        assert!((lo - b2).abs() < 1e-5 && hi - lo < 1e-12, "β2 à {pas} pas : {lo}");
    }
}

/// juste + mesure : β1 et β2 meurent bien plus lentement qu'α1 (V-NAK-04). Le « pas de mort » d'une route est le
/// premier nombre de pas faits après lequel la plus grande conductivité de la route est `< 0,01`. Pour les graines 1 à
/// 12 le modèle Python donne **exactement** : impasses 7 ; α1 45 ; β2 408, 415, 411, 412, 412, 410, 411, 409, 410, 414,
/// 410, 410 ; β1 jamais. Mesuré sur 50 autres graines : α1 44 ou 45, β2 de 405 à 418, impasses 7. Le test écrit ces
/// valeurs exactes, puis le rapport que la mesure permet de dire : β2 meurt au moins 8 fois plus tard qu'α1 (408 / 45 =
/// 9,07 au plus petit rapport) et, à 60 pas, quand α1 est tout au plus à `0,003`, β2 est encore à `0,32` au moins et β1
/// à `0,64` au moins (50 graines : `0,3233` et `0,6457`). La raison : les deux routes de chaque paire se partagent le
/// débit à peu près comme leurs conductances, et la différence qui les sépare décroît au rythme de l'écart relatif de
/// leurs longueurs (22 % pour α, 2,3 % pour β). **Ce n'est pas l'article** : il ne donne que la phrase « à peu près le
/// même nombre de tubes » ; nous ne prétendons retrouver aucun chiffre.
///
/// Ne couvre pas : l'effet d'un bruit sur le flux, qui rendrait β1 et β2 indiscernables (l'article, 2 %).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn beta_meurt_bien_plus_lentement_qu_alpha1() {
    let plan = plan_avec(&[]);
    let routes: Vec<Vec<usize>> = b"aAbBx".iter().map(|&r| indices_de(&plan, r)).collect();
    let morts_beta2 = [408_usize, 415, 411, 412, 412, 410, 411, 409, 410, 414, 410, 410];
    for (k, graine) in (1..=12_u64).enumerate() {
        let mut tubes = Tubes::au_hasard(&plan.graphe, &mut Alea::new(graine));
        let mut mort: [Option<usize>; 5] = [None; 5];
        let mut a60 = (0.0, 0.0, 0.0);
        for pas in 0..=430 {
            let d = tubes.conductivites();
            for (r, aretes) in routes.iter().enumerate() {
                let (_, haut) = extremes(d, aretes);
                if mort[r].is_none() && haut < 0.01 {
                    mort[r] = Some(pas);
                }
            }
            if pas == 60 {
                a60 = (extremes(d, &routes[0]).1, extremes(d, &routes[3]).0, extremes(d, &routes[2]).0);
            }
            tubes.avancer(&plan.graphe, plan.source, plan.puits, 1.0, 0.5).unwrap();
        }
        assert_eq!(mort, [Some(45), None, None, Some(morts_beta2[k]), Some(7)], "graine {graine}");
        assert!(morts_beta2[k] >= 8 * 45, "graine {graine} : β2 meurt au moins 8 fois plus tard");
        assert!(a60.0 <= 0.003 && a60.1 >= 0.32 && a60.2 >= 0.64, "graine {graine} à 60 pas : {a60:?}");
    }
}

/// mesure : sans bruit, β2 finit par mourir et β1 par rester (V-NAK-05). Pour les graines 1 à 12 après 600 pas : toute
/// la route β1 est `>= 0,99` (50 graines : `0,9987` au moins) et toute la route β2 est `<= 0,004` (50 graines :
/// `0,0013` au plus, la tolérance est trois fois la mesure : la fuite du plancher et le rythme de la mort). Les 154
/// tubes vifs sont ceux d'α2 et de β1. À 20 pas, au contraire, β1 est dans `[0,52 ; 0,59]` et β2 dans `[0,41 ; 0,48]`
/// (50 graines : `0,5377` à `0,5719` et `0,4281` à `0,4623`) : les deux routes sont à peu près également peuplées,
/// comme dans l'article au début, et le restent assez longtemps pour qu'une expérience bruitée ne les sépare pas. La
/// pression de la source tend vers 154, la longueur du plus court chemin (Bonifaci : le potentiel source-puits tend
/// vers `L*`) : à 600 pas, elle est dans `[154,0 ; 154,006]` (50 graines : `154,00225` à `154,00259`, par le haut : les
/// tubes morts laissent passer un peu de flux).
///
/// Ne couvre pas : un autre `h` (tout est à `h = 0,5`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn beta2_finit_par_mourir() {
    let plan = plan_avec(&[]);
    let routes: Vec<Vec<usize>> = b"aAbBx".iter().map(|&r| indices_de(&plan, r)).collect();
    for graine in 1..=12_u64 {
        let mut tubes = Tubes::au_hasard(&plan.graphe, &mut Alea::new(graine));
        let mut pression = 0.0;
        for pas in 0..=600 {
            if pas == 20 {
                let d = tubes.conductivites();
                let (b1, b2) = (extremes(d, &routes[2]), extremes(d, &routes[3]));
                assert!(b1.0 >= 0.52 && b1.1 <= 0.59, "graine {graine} : β1 à 20 pas {b1:?}");
                assert!(b2.0 >= 0.41 && b2.1 <= 0.48, "graine {graine} : β2 à 20 pas {b2:?}");
            }
            if pas < 600 {
                pression =
                    tubes.avancer(&plan.graphe, plan.source, plan.puits, 1.0, 0.5).unwrap().pressions[plan.source];
            }
        }
        let d = tubes.conductivites();
        assert!(extremes(d, &routes[2]).0 >= 0.99, "graine {graine} : β1");
        assert!(extremes(d, &routes[3]).1 <= 0.004, "graine {graine} : β2");
        assert_eq!(d.iter().filter(|&&x| x >= 0.5).count(), 154, "graine {graine} : les tubes vifs");
        assert!((154.0..=154.006).contains(&pression), "graine {graine} : pression {pression}");
    }
}

/// juste : les impasses meurent (V-NAK-06) : l'article les montre se rétracter (Fig. 1b). Un tube sans flux perd la
/// moitié de sa conductivité à chaque pas (`h = 0,5`) : après 7 pas, `<= 0,01`, après 20 pas, il est au `PLANCHER`
/// exactement (`0,75 * 2^-20 < 1e-6`). Pour les graines 1 à 12, les 17 arêtes d'impasse (une rétraction : trois
/// couloirs de 11, 3 et 3 cellules, avec leur arête d'attache) sont au `PLANCHER` à 20 pas, et à 7 pas la plus grande
/// est `<= 0,01` (la mort est au pas 7 exactement, voir `beta_meurt_bien_plus_lentement_qu_alpha1`) alors qu'à 6 pas
/// elle est `> 0,01`.
///
/// Ne couvre pas : la rétraction des impasses d'un maillage plus fin.
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn les_impasses_meurent() {
    let plan = plan_avec(&[]);
    let impasses = indices_de(&plan, b'x');
    assert_eq!(impasses.len(), 17);
    for graine in 1..=12_u64 {
        let d = apres(&plan, graine, 20);
        assert!(impasses.iter().all(|&e| d[e] == PLANCHER), "graine {graine} : au plancher à 20 pas");
        assert!(extremes(&apres(&plan, graine, 7), &impasses).1 <= 0.01, "graine {graine} : mort à 7 pas");
        assert!(extremes(&apres(&plan, graine, 6), &impasses).1 > 0.01, "graine {graine} : pas encore mort à 6 pas");
    }
}

/// juste : le labyrinthe n'a que les quatre routes et une jonction (V-NAK-07) : le graphe du plan a 342 nœuds et 343
/// arêtes (deux cycles, un par anneau), est connexe, la jonction `J` a quatre arêtes, `S` trois (α1, α2, l'impasse),
/// `T` deux (β1, β2) ; les nœuds des impasses n'ont qu'une arête au bout (trois bouts : `(2, 3)`, `(11, 27)`, `(37,
/// 1)`). Retirer le seul mur d'un couloir (α2 par exemple) laisse le labyrinthe relié (l'autre route reste) ; retirer
/// en même temps α1 et α2 coupe la source du puits (`SourceCoupee`).
///
/// Ne couvre pas : la dynamique.
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn la_structure_du_labyrinthe() {
    let plan = plan_avec(&[]);
    assert!(plan.graphe.est_connexe());
    assert_eq!(plan.graphe.nb_aretes() + 1 - plan.graphe.nb_noeuds(), 2, "deux cycles");
    let degre = |x, y| plan.graphe.incidentes(noeud(&plan, x, y)).len();
    assert_eq!((degre(20, 14), degre(2, 14), degre(38, 14)), (4, 3, 2));
    assert_eq!((degre(2, 3), degre(11, 27), degre(37, 1)), (1, 1, 1));
    let bouts = (0..plan.graphe.nb_noeuds()).filter(|&i| plan.graphe.incidentes(i).len() == 1).count();
    assert_eq!(bouts, 3, "seules les trois impasses ont un bout");
    let g = grille();
    let (s, t) = (g.indice(2, 14, 0), g.indice(38, 14, 0));
    let mut murs: Vec<Mur> = Vec::new();
    for y in 0..g.hauteur {
        for x in 0..g.largeur {
            if cellule(x, y) == b'#' {
                murs.push(Mur { x0: x, y0: y, x1: x, y1: y, etage: 0 });
            }
        }
    }
    let coupe = |(x, y): (u32, u32)| Mur { x0: x, y0: y, x1: x, y1: y, etage: 0 };
    let mut une = murs.clone();
    une.push(coupe(COUPE_ALPHA2));
    assert!(construire(&g, &une, &[], s, t).is_ok());
    murs.push(coupe(COUPE_ALPHA1));
    murs.push(coupe(COUPE_ALPHA2));
    assert_eq!(construire(&g, &murs, &[], s, t).unwrap_err(), physarum_lab::labyrinthe::ErreurLabyrinthe::SourceCoupee);
}
