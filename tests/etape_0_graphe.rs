//! Étape 0 : `graphe::Graphe`, le graphe rangé dans son ordre canonique. Valeurs attendues :
//! `references/etape-0-valeurs.md`, identifiants `V-GRAPHE-xx`.
//!
//! Ici la source est la définition écrite dans la documentation de `graphe` (une convention de l'étape 0, pas une valeur
//! publiée) ; les cas sont de petits graphes écrits à la main, dont le résultat se lit sans calcul.

use physarum_lab::graphe::{Arete, ErreurGraphe, Graphe, Noeud};

fn noeuds(n: usize) -> Vec<Noeud> {
    (0..n).map(|i| Noeud { x: i as i32, y: 0, etage: 0 }).collect()
}

fn arete(a: usize, b: usize, longueur: f64, cout: f64) -> Arete {
    Arete { a, b, longueur, cout }
}

/// Le graphe d'essai : cinq nœuds, cinq arêtes données dans le désordre et pour moitié à l'envers ; le nœud 4 est seul.
/// Dans l'ordre canonique : (0,1) (0,3) (1,2) (1,3) (2,3).
fn essai() -> Graphe {
    Graphe::new(
        noeuds(5),
        vec![
            arete(3, 2, 5.0, 50.0),
            arete(0, 1, 1.0, 10.0),
            arete(3, 1, 4.0, 40.0),
            arete(1, 2, 3.0, 30.0),
            arete(3, 0, 2.0, 20.0),
        ],
    )
    .expect("graphe valide")
}

/// juste : les arêtes sont rangées avec `a < b` et triées par `(a, b)`, quel que soit l'ordre donné ; chaque arête garde sa
/// longueur et son coût (V-GRAPHE-01 : l'ordre attendu se lit sur la liste) ; les nœuds gardent leur ordre et leur position.
///
/// Ne couvre pas : les grands graphes (voir `etape_0_grille`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn ordre_canonique_des_aretes() {
    let g = essai();
    assert_eq!((g.nb_noeuds(), g.nb_aretes()), (5, 5));
    assert_eq!(
        g.aretes(),
        [
            arete(0, 1, 1.0, 10.0),
            arete(0, 3, 2.0, 20.0),
            arete(1, 2, 3.0, 30.0),
            arete(1, 3, 4.0, 40.0),
            arete(2, 3, 5.0, 50.0),
        ]
    );
    let places =
        vec![Noeud { x: -7, y: 3, etage: 2 }, Noeud { x: 5, y: -1, etage: 0 }, Noeud { x: 5, y: -1, etage: 0 }];
    let g = Graphe::new(places.clone(), vec![arete(2, 0, 1.5, 2.5)]).unwrap();
    assert_eq!(g.noeuds(), places, "les nœuds gardent leur ordre, même à la même position");
    assert_eq!(g.aretes(), [arete(0, 2, 1.5, 2.5)]);
    // le même graphe donné dans un autre ordre est le même graphe
    let autre = Graphe::new(
        noeuds(5),
        vec![
            arete(1, 0, 1.0, 10.0),
            arete(2, 1, 3.0, 30.0),
            arete(0, 3, 2.0, 20.0),
            arete(2, 3, 5.0, 50.0),
            arete(1, 3, 4.0, 40.0),
        ],
    )
    .unwrap();
    assert_eq!(autre, essai());
}

/// juste : `incidentes(i)` rend les indices des arêtes du nœud `i`, croissants ; vide pour un nœud seul (V-GRAPHE-02, lu sur
/// la liste canonique : le nœud 3 touche les arêtes 1, 3 et 4).
///
/// Ne couvre pas : un indice de nœud hors du graphe (voir `incidentes_hors_du_graphe`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn incidentes_par_indice_croissant() {
    let g = essai();
    assert_eq!(g.incidentes(0), [0, 1]);
    assert_eq!(g.incidentes(1), [0, 2, 3]);
    assert_eq!(g.incidentes(2), [2, 4]);
    assert_eq!(g.incidentes(3), [1, 3, 4]);
    assert_eq!(g.incidentes(4), [0_usize; 0]);
}

/// juste : `incidentes` panique pour un indice qui n'est pas un nœud (la doc de `incidentes`, V-GRAPHE-02), alors que le
/// dernier nœud, lui, a sa liste.
///
/// Ne couvre pas : le texte de la panique.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn incidentes_hors_du_graphe() {
    let g = essai();
    assert_eq!(g.incidentes(3), [1, 3, 4]);
    for hors in [5, 6, usize::MAX] {
        let resultat = std::panic::catch_unwind(|| g.incidentes(hors).len());
        assert!(resultat.is_err(), "incidentes({hors}) devrait paniquer");
    }
}

/// juste : `arete_entre(a, b)` rend l'indice de l'arête dans les deux sens, `None` sans arête, pour `a == b`, et pour un
/// indice hors du graphe, sans jamais paniquer (V-GRAPHE-03).
///
/// Ne couvre pas : la vitesse de la recherche.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn arete_entre_deux_noeuds() {
    let g = essai();
    for (a, b, attendu) in [(0, 1, 0), (0, 3, 1), (1, 2, 2), (1, 3, 3), (2, 3, 4)] {
        assert_eq!(g.arete_entre(a, b), Some(attendu), "{a}-{b}");
        assert_eq!(g.arete_entre(b, a), Some(attendu), "{b}-{a}");
    }
    for (a, b) in [(0, 2), (2, 0), (0, 4), (4, 3), (1, 1), (4, 4), (0, 5), (5, 0), (7, 9), (usize::MAX, 0), (5, 5)] {
        assert_eq!(g.arete_entre(a, b), None, "{a}-{b}");
    }
}

/// juste : les erreurs de `Graphe::new` (V-GRAPHE-04, la doc de `new`) : `Vide` sans nœud, même avec des arêtes ; sinon
/// `Arete(i)` où `i` est le plus petit rang, dans la liste donnée, d'une arête fautive : une boucle, un bout hors du graphe,
/// une longueur ou un coût nul, négatif, infini ou NaN, ou la seconde arête d'une même paire (dans un sens ou dans l'autre).
/// Les mêmes listes sans la faute sont acceptées (un code qui refuse tout échoue).
///
/// Ne couvre pas : trois arêtes sur la même paire (la deuxième suffit).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn erreurs_de_construction() {
    assert_eq!(Graphe::new(vec![], vec![]), Err(ErreurGraphe::Vide));
    assert_eq!(Graphe::new(vec![], vec![arete(0, 1, 1.0, 1.0)]), Err(ErreurGraphe::Vide));
    let bonnes = || vec![arete(0, 1, 1.0, 1.0), arete(1, 2, 1.0, 1.0), arete(2, 3, 1.0, 1.0)];
    assert!(Graphe::new(noeuds(4), bonnes()).is_ok());
    assert!(Graphe::new(noeuds(1), vec![]).is_ok(), "un nœud seul, sans arête");
    let infini = f64::INFINITY;
    let fautives = [
        arete(2, 2, 1.0, 1.0),
        arete(0, 4, 1.0, 1.0),
        arete(4, 0, 1.0, 1.0),
        arete(9, 9, 1.0, 1.0),
        arete(0, 2, 0.0, 1.0),
        arete(0, 2, -1.0, 1.0),
        arete(0, 2, infini, 1.0),
        arete(0, 2, f64::NAN, 1.0),
        arete(0, 2, 1.0, 0.0),
        arete(0, 2, 1.0, -0.5),
        arete(0, 2, 1.0, infini),
        arete(0, 2, 1.0, f64::NAN),
        arete(0, 2, -0.0, 1.0),
        arete(1, 0, 7.0, 7.0),
        arete(2, 1, 1.0, 1.0),
    ];
    for fautive in fautives {
        for place in 0..=3 {
            let mut liste = bonnes();
            liste.insert(place, fautive);
            // une arête double : c'est la seconde de la paire qui est fautive
            let attendu = if fautive.a.min(fautive.b) == 0 && fautive.a.max(fautive.b) == 1 && place == 0 {
                1
            } else if fautive.a.min(fautive.b) == 1 && fautive.a.max(fautive.b) == 2 && place <= 1 {
                2
            } else {
                place
            };
            assert_eq!(Graphe::new(noeuds(4), liste), Err(ErreurGraphe::Arete(attendu)), "{fautive:?} au rang {place}");
        }
    }
    // la première fautive l'emporte, quelle que soit sa faute
    let liste = vec![arete(0, 1, 1.0, 1.0), arete(1, 0, 1.0, 1.0), arete(2, 2, 1.0, 1.0)];
    assert_eq!(Graphe::new(noeuds(3), liste), Err(ErreurGraphe::Arete(1)));
    let liste = vec![arete(0, 1, 1.0, 1.0), arete(2, 2, 1.0, 1.0), arete(1, 0, 1.0, 1.0)];
    assert_eq!(Graphe::new(noeuds(3), liste), Err(ErreurGraphe::Arete(1)));
    // une arête fautive suivie de son double : la première des deux
    let liste = vec![arete(1, 2, 1.0, 1.0), arete(0, 1, -1.0, 1.0), arete(1, 0, 1.0, 1.0)];
    assert_eq!(Graphe::new(noeuds(3), liste), Err(ErreurGraphe::Arete(1)));
    // le double vient bien après : c'est lui, pas la première occurrence
    let liste = vec![arete(0, 1, 1.0, 1.0), arete(1, 2, 1.0, 1.0), arete(0, 2, 1.0, 1.0), arete(2, 1, 9.0, 9.0)];
    assert_eq!(Graphe::new(noeuds(3), liste), Err(ErreurGraphe::Arete(3)));
}

/// juste : `est_connexe` : un chemin, un cycle, un nœud seul sont connexes ; un nœud sans arête à côté d'autres, deux
/// morceaux, ne le sont pas (V-GRAPHE-05, lu sur le dessin de chaque graphe).
///
/// Ne couvre pas : les très grands graphes.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn connexite() {
    let u = |a, b| arete(a, b, 1.0, 1.0);
    assert!(Graphe::new(noeuds(1), vec![]).unwrap().est_connexe());
    assert!(Graphe::new(noeuds(4), vec![u(0, 1), u(1, 2), u(2, 3)]).unwrap().est_connexe());
    assert!(Graphe::new(noeuds(4), vec![u(3, 0), u(1, 3), u(2, 1), u(0, 2)]).unwrap().est_connexe());
    assert!(Graphe::new(noeuds(6), vec![u(5, 0), u(5, 1), u(5, 2), u(5, 3), u(5, 4)]).unwrap().est_connexe());
    assert!(!essai().est_connexe(), "le nœud 4 est seul");
    assert!(!Graphe::new(noeuds(2), vec![]).unwrap().est_connexe());
    assert!(!Graphe::new(noeuds(4), vec![u(0, 1), u(2, 3)]).unwrap().est_connexe());
    assert!(
        !Graphe::new(noeuds(5), vec![u(1, 2), u(2, 3), u(3, 4), u(4, 1)]).unwrap().est_connexe(),
        "le nœud 0 est seul"
    );
}

/// juste : `longueur_totale` part de `0.0` et ajoute les longueurs dans l'ordre des indices (V-GRAPHE-06, calcul exact en
/// `f64`). Quatre arêtes de longueurs canoniques `1, 1e16, 1, 1` : `((0 + 1) + 1e16) + 1 + 1` vaut exactement `1e16`
/// (chaque `+ 1` tombe à mi-chemin de deux flottants et s'arrondit au pair), alors que l'ordre inverse donne `1e16 + 4` et
/// l'ordre donné `1e16 + 2`. Les coûts ne comptent pas. Sans arête : `0.0`.
///
/// Ne couvre pas : une somme compensée, que la doc interdit et que ce cas distingue aussi (`1e16 + 3` arrondi).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn longueur_totale_dans_l_ordre_des_indices() {
    // ordre donné : (1,2) (2,3) (0,2) (0,1) ; ordre canonique : (0,1) (0,2) (1,2) (2,3)
    let g = Graphe::new(
        noeuds(4),
        vec![arete(1, 2, 1.0, 9.0), arete(2, 3, 1.0, 9.0), arete(0, 2, 1e16, 9.0), arete(0, 1, 1.0, 9.0)],
    )
    .unwrap();
    assert_eq!(g.longueur_totale().to_bits(), 1e16_f64.to_bits());
    assert_ne!(1e16_f64.to_bits(), (1e16_f64 + 2.0).to_bits(), "le cas distingue bien les ordres");
    assert_eq!(Graphe::new(noeuds(3), vec![]).unwrap().longueur_totale().to_bits(), 0.0_f64.to_bits());
    assert_eq!(essai().longueur_totale(), 15.0);
}

/// juste : l'écriture canonique, octet par octet (V-GRAPHE-07, écrite à la main d'après la doc) : `n` en `u32`
/// petit-boutiste, puis `x` (`i32`), `y` (`i32`), `etage` (`u32`) de chaque nœud, puis `m`, puis `a`, `b` (`u32`), les bits
/// de la longueur et ceux du coût (`u64`) de chaque arête dans l'ordre canonique. Deux nœuds `(-2, 3, 1)` et `(258, -1, 0)`,
/// une arête donnée à l'envers `(1, 0)` de longueur `1.5` (`0x3FF8000000000000`) et de coût `0.1`
/// (`0x3FB999999999999A`). La longueur de l'écriture est `8 + 12 n + 24 m`.
///
/// Ne couvre pas : plus de 2^32 nœuds.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn ecriture_canonique() {
    let g = Graphe::new(
        vec![Noeud { x: -2, y: 3, etage: 1 }, Noeud { x: 258, y: -1, etage: 0 }],
        vec![arete(1, 0, 1.5, 0.1)],
    )
    .unwrap();
    #[rustfmt::skip]
    let attendu: [u8; 56] = [
        2, 0, 0, 0,
        0xFE, 0xFF, 0xFF, 0xFF, 3, 0, 0, 0, 1, 0, 0, 0,
        2, 1, 0, 0, 0xFF, 0xFF, 0xFF, 0xFF, 0, 0, 0, 0,
        1, 0, 0, 0,
        0, 0, 0, 0, 1, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0xF8, 0x3F,
        0x9A, 0x99, 0x99, 0x99, 0x99, 0x99, 0xB9, 0x3F,
    ];
    assert_eq!(g.octets(), attendu);
    let seul = Graphe::new(vec![Noeud { x: 0, y: 0, etage: 0 }], vec![]).unwrap();
    assert_eq!(seul.octets(), [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(essai().octets().len(), 8 + 12 * 5 + 24 * 5);
    // la longueur et le coût ne s'échangent pas, ni les bouts
    let h = Graphe::new(
        vec![Noeud { x: -2, y: 3, etage: 1 }, Noeud { x: 258, y: -1, etage: 0 }],
        vec![arete(0, 1, 0.1, 1.5)],
    )
    .unwrap();
    assert_ne!(h.octets(), attendu);
}
