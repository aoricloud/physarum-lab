//! Étape 1, partie 1R : `reseaux::apercu`, l'aperçu d'un réseau (les tubes de `tubes::apercu` et, par-dessus, les nœuds
//! de nourriture). Valeurs attendues : `references/etape-1r-valeurs.md`, identifiants `V-APR-xx`.
//!
//! La source est la doc de `reseaux` (« L'aperçu ») et celle de `tubes::apercu`. Un aperçu est pour l'œil : ces tests
//! disent qu'il est celui de la doc, en entiers. Le dessin d'un petit cas est écrit ici en caractères, ses couleurs
//! viennent du modèle Python `ref_reseaux.py` (le treillis coloré de `ref_tubes.apercu_points`, puis les nourritures
//! peintes) et se vérifient à la main (écrit en commentaire).

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Arete, Graphe, Grille, Noeud, Voisinage};
use physarum_lab::labyrinthe::{Mur, construire};
use physarum_lab::rendu::{ECART, FOND, treillis};
use physarum_lab::reseaux::{NOURRITURE, apercu};
use physarum_lab::tubes::{Tubes, VIF, apercu as apercu_tubes};

/// Le petit réseau à la main : deux étages, un carré à ses deux diagonales sur l'étage 0 (nœuds 0 à 3), un nœud 4 sur
/// l'étage 1 au-dessus du nœud 0, tous décalés de `(3, 2)` (le cadre se calcule sur les nœuds : le décalage ne se voit
/// pas). Les tubes, dans l'ordre canonique : `(0,1)` `D = 1` (niveau 255), `(0,2)` `0,5` (128), `(0,3)` `0,2` (51),
/// `(0,4)` `0,8` (204, le lien entre étages), `(1,2)` `1` (255), `(1,3)` `0,001` (0), `(2,3)` `0,002` (1) ; référence
/// 1.
fn petit_reseau() -> (Graphe, Vec<f64>) {
    let noeuds = vec![
        Noeud { x: 3, y: 2, etage: 0 },
        Noeud { x: 4, y: 2, etage: 0 },
        Noeud { x: 3, y: 3, etage: 0 },
        Noeud { x: 4, y: 3, etage: 0 },
        Noeud { x: 3, y: 2, etage: 1 },
    ];
    let paires = [(0, 1), (0, 2), (0, 3), (0, 4), (1, 2), (1, 3), (2, 3)];
    let aretes = paires.iter().map(|&(a, b)| Arete { a, b, longueur: 1.0, cout: 1.0 }).collect();
    let d = vec![1.0, 0.5, 0.2, 0.8, 1.0, 0.001, 0.002];
    (Graphe::new(noeuds, aretes).unwrap(), d)
}

/// Le dessin attendu en caractères : `F` fond, `V` tube vif (niveau 255), `N` nourriture, `A` le nœud 0 (vif et lié à
/// l'étage 1 au niveau 204), `B` le milieu d'un tube de niveau 128, `C` celui d'un tube de niveau 1.
fn couleur(c: char) -> [u8; 3] {
    match c {
        'F' => FOND,
        'V' => VIF,
        'N' => NOURRITURE,
        'A' => [204, 180, 83],
        'B' => [36, 118, 140],
        'C' => [12, 17, 25],
        _ => panic!("couleur inconnue {c}"),
    }
}

fn octets_attendus(dessin: &[&str], echelle: usize) -> Vec<u8> {
    let mut rvb = Vec::new();
    for ligne in dessin {
        let mut rangee = Vec::new();
        for c in ligne.chars() {
            for _ in 0..echelle {
                rangee.extend_from_slice(&couleur(c));
            }
        }
        for _ in 0..echelle {
            rvb.extend_from_slice(&rangee);
        }
    }
    rvb
}

/// juste : le petit réseau à la main (V-APR-01), au treillis de 8 × 3 points (`TL = 2 * 3 + 2 = 8`, `th = 3` : le
/// panneau de chaque étage fait 3 points de large, l'étage 1 commence à la colonne 5) :
/// ```text
/// A V V F F N F F      le nœud 0 (vif, lien de niveau 204 : mélange de VIF vers PUITS), le milieu (0,1), le nœud 1 ;
///                      la nourriture 4 sur l'étage 1 (colonne 5) : la nourriture recouvre la teinte du lien
/// B V F F F F F F      le milieu (0,2) de niveau 128 ; le croisement des diagonales (0,3) de 51 et (1,2) de 255 :
///                      la plus épaisse l'emporte (V) ; le milieu (1,3) de niveau 0 : le fond
/// V C N F F F F F      le nœud 2 (son plus fort tube est (1,2) de 255) ; le milieu (2,3) de niveau 1 ; la
///                      nourriture 3
/// ```
/// aux échelles 1, 2 et 3 (chaque point devient un carré). Avec les nourritures `[3, 4]`, puis `[4, 3]` (l'ordre ne
/// compte pas) : la même image ; sans nourriture, c'est `tubes::apercu` pur (le nœud 3 y a la teinte de son plus fort
/// tube, `(0,3)` de niveau 51 : `(22, 57, 70)` ; le nœud 4 y a la teinte de son lien, ni le fond ni la nourriture) ;
/// avec une seule nourriture, l'autre point reste tel quel.
///
/// Ne couvre pas : d'autres graphes (`contre_l_apercu_des_tubes`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn un_petit_apercu_a_la_main() {
    let (g, d) = petit_reseau();
    let dessin = ["AVVFFNFF", "BVFFFFFF", "VCNFFFFF"];
    for echelle in [1_usize, 2, 3] {
        let image = apercu(&g, &d, &[3, 4], 1.0, echelle as u32);
        assert_eq!((image.largeur as usize, image.hauteur as usize), (8 * echelle, 3 * echelle));
        assert_eq!(image.rvb.len(), 3 * 8 * 3 * echelle * echelle);
        assert_eq!(image.rvb, octets_attendus(&dessin, echelle), "échelle {echelle}");
        assert_eq!(apercu(&g, &d, &[4, 3], 1.0, echelle as u32), image, "l'ordre des nourritures ne compte pas");
    }
    // sans nourriture : tubes::apercu ; le nœud 3 prend le niveau de son plus fort tube, 51 : melange(FOND, VIF, 51) =
    // ((12 * 204 + 60 * 51 + 127) / 255, (16 * 204 + 220 * 51 + 127) / 255, (24 * 204 + 255 * 51 + 127) / 255)
    let nu = apercu(&g, &d, &[], 1.0, 1);
    assert_eq!(nu, apercu_tubes(&g, &d, 1.0, 1));
    let point =
        |x: usize, y: usize| [nu.rvb[3 * (y * 8 + x)], nu.rvb[3 * (y * 8 + x) + 1], nu.rvb[3 * (y * 8 + x) + 2]];
    assert_eq!(point(2, 2), [22, 57, 70], "le nœud 3, sans nourriture");
    assert_ne!(point(5, 0), NOURRITURE);
    assert_ne!(point(5, 0), FOND);
    // une seule nourriture : l'autre point reste ce qu'il était
    let une = apercu(&g, &d, &[3], 1.0, 1);
    let point_une =
        |x: usize, y: usize| [une.rvb[3 * (y * 8 + x)], une.rvb[3 * (y * 8 + x) + 1], une.rvb[3 * (y * 8 + x) + 2]];
    assert_eq!((point_une(2, 2), point_une(5, 0)), (NOURRITURE, point(5, 0)));
}

/// juste : les paniques (V-APR-02), cas positif d'abord : `echelle = 0`, une référence nulle, négative, NaN ou infinie,
/// un nombre de conductivités qui n'est pas celui des arêtes, une nourriture hors du graphe (nœud 5 d'un graphe de 5
/// nœuds).
///
/// Ne couvre pas : les messages.
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn paniques() {
    let (g, d) = petit_reseau();
    assert!(!apercu(&g, &d, &[3, 4], 1.0, 1).rvb.is_empty());
    let essai = |nom: &str, f: Box<dyn FnOnce() + std::panic::UnwindSafe>| {
        assert!(std::panic::catch_unwind(f).is_err(), "{nom} doit paniquer");
    };
    let (g1, d1) = (g.clone(), d.clone());
    essai("échelle 0", Box::new(move || drop(apercu(&g1, &d1, &[3], 1.0, 0))));
    for reference in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        let (g1, d1) = (g.clone(), d.clone());
        essai("référence", Box::new(move || drop(apercu(&g1, &d1, &[3], reference, 1))));
    }
    let (g1, d1) = (g.clone(), d[..6].to_vec());
    essai("six conductivités", Box::new(move || drop(apercu(&g1, &d1, &[3], 1.0, 1))));
    let (g1, d1) = (g.clone(), d.clone());
    essai("nourriture hors du graphe", Box::new(move || drop(apercu(&g1, &d1, &[3, 5], 1.0, 1))));
}

/// juste : `apercu` est `tubes::apercu` avec les nourritures peintes (V-APR-03), sur 12 graphes de plans murés à un ou
/// deux étages, des conductivités au hasard et deux, trois ou cinq nourritures : tous les pixels hors des carrés de
/// nourriture sont ceux de `tubes::apercu` ; le carré de `echelle x echelle` pixels de chaque nourriture (colonne
/// `etage * (tl + ECART) + 2 * (x - xmin)`, ligne `2 * (y - ymin)` du treillis du **plan**, dont le cadre est celui des
/// nœuds gardés : la colonne 0 de la grille est murée, donc `xmin = 1`) est entièrement `NOURRITURE`. Le test exige des
/// pixels peints (3 nourritures, 12 carrés d'au moins 4 pixels) et des pixels non peints.
///
/// Ne couvre pas : le dessin des tubes (`etape_1_apercu`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn contre_l_apercu_des_tubes() {
    for graine in 1..=12_u64 {
        let etages = 1 + (graine % 2) as u32;
        let grille = Grille {
            largeur: 7,
            hauteur: 5,
            etages,
            voisinage: if graine % 3 == 0 { Voisinage::Quatre } else { Voisinage::Huit },
            longueur_verticale: 1.0,
            cout_vertical: 2.0,
        };
        let murs: Vec<Mur> = (0..etages).map(|e| Mur { x0: 0, y0: 0, x1: 0, y1: 4, etage: e }).collect();
        let plan = construire(&grille, &murs, &[], grille.indice(1, 0, 0), grille.indice(6, 4, 0)).unwrap();
        let g = &plan.graphe;
        let mut alea = Alea::new(graine);
        let tubes = Tubes::au_hasard(g, &mut alea);
        let d: Vec<f64> = tubes.conductivites().iter().map(|x| x * (alea.suivant_f64() + 0.1)).collect();
        let n = g.nb_noeuds();
        let k = [2, 3, 5][graine as usize % 3];
        let mut foods: Vec<usize> = Vec::new();
        while foods.len() < k {
            let v = alea.borne(n as u32) as usize;
            if !foods.contains(&v) {
                foods.push(v);
            }
        }
        let echelle = 1 + (graine % 3) as u32;
        let image = apercu(g, &d, &foods, 1.0, echelle);
        let pur = apercu_tubes(g, &d, 1.0, echelle);
        assert_eq!((image.largeur, image.hauteur), (pur.largeur, pur.hauteur));
        let (tl, _) = treillis(g);
        let xmin = g.noeuds().iter().map(|v| v.x).min().unwrap();
        let ymin = g.noeuds().iter().map(|v| v.y).min().unwrap();
        assert_eq!(xmin, 1, "la colonne 0 est murée");
        let tl_panneau = 2 * (g.noeuds().iter().map(|v| v.x).max().unwrap() - xmin) + 1;
        assert!(tl >= tl_panneau as u32 && tl_panneau > 0);
        let mut peints = vec![false; (image.largeur * image.hauteur) as usize];
        let e = echelle as usize;
        for &v in &foods {
            let noeud = g.noeuds()[v];
            let colonne = noeud.etage as usize * (tl_panneau as usize + ECART as usize) + 2 * (noeud.x - xmin) as usize;
            let ligne = 2 * (noeud.y - ymin) as usize;
            for py in ligne * e..(ligne + 1) * e {
                for px in colonne * e..(colonne + 1) * e {
                    peints[py * image.largeur as usize + px] = true;
                }
            }
        }
        for (i, &peint) in peints.iter().enumerate() {
            let (a, b) = (&image.rvb[3 * i..3 * i + 3], &pur.rvb[3 * i..3 * i + 3]);
            if peint {
                assert_eq!(a, NOURRITURE, "graine {graine}, pixel {i}");
            } else {
                assert_eq!(a, b, "graine {graine}, pixel {i}");
            }
        }
        let nb_peints = peints.iter().filter(|&&p| p).count();
        assert!(nb_peints >= k * e * e && nb_peints < peints.len(), "graine {graine} : {nb_peints} pixels peints");
    }
}
