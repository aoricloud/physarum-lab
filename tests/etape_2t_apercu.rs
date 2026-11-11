//! Étape 2, partie 2T : `souvenir::apercu`, l'aperçu de l'agent (les tubes de `tubes::apercu`, les nœuds colorés selon
//! la concentration : « la tache d'agent qui suit les tubes »). Valeurs attendues : `references/etape-2t-valeurs.md`,
//! identifiants `V-APE-xx`.
//!
//! La source est la doc de `souvenir` (« L'aperçu ») et celle de `tubes::apercu`. Un aperçu est pour l'œil : ces tests
//! disent qu'il est celui de la doc, en entiers. Le dessin d'un petit cas est écrit ici en caractères, ses couleurs
//! viennent du modèle Python `ref_souvenir.py` (le treillis coloré de `ref_tubes.apercu_points`, puis les nœuds mêlés à
//! `AGENT`) et se vérifient à la main (écrit en commentaire) ; les empreintes (sha256) des trois grands cas sont
//! calculées par ce modèle, qui rejoue aussi les tubes et le transport.

use physarum_lab::alea::Alea;
use physarum_lab::empreinte::sha256_hex;
use physarum_lab::graphe::{Arete, Graphe, Grille, Noeud, Voisinage};
use physarum_lab::rendu::{FOND, treillis};
use physarum_lab::souvenir::{AGENT, Agent, Transport, apercu};
use physarum_lab::tubes::{Tubes, VIF, apercu as apercu_tubes};

/// Le petit réseau à la main : deux étages, un carré à ses deux diagonales sur l'étage 0 (nœuds 0 à 3), un nœud 4 sur
/// l'étage 1 au-dessus du nœud 0, tous décalés de `(3, 2)` (le cadre se calcule sur les nœuds : le décalage ne se voit
/// pas). Les tubes, dans l'ordre canonique : `(0,1)` `D = 1` (niveau 255), `(0,2)` `0,5` (128), `(0,3)` `0,2` (51),
/// `(0,4)` `0,8` (204, le lien entre étages), `(1,2)` `1` (255), `(1,3)` `0,001` (0), `(2,3)` `0,002` (1) ; référence
/// des tubes 1.
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

/// Le dessin attendu en caractères : `F` fond, `V` tube vif, `A` un nœud au niveau 255 de l'agent (la couleur `AGENT`),
/// `1` le nœud 1 (tube vif mêlé à l'agent au niveau 128), `3` le nœud 3 (mêlé au niveau 51), `4` le nœud 4 (teinte du
/// lien mêlée à l'agent au niveau 204), `B` le milieu d'un tube de niveau 128, `C` celui d'un tube de niveau 1.
fn couleur(c: char) -> [u8; 3] {
    match c {
        'F' => FOND,
        'V' => VIF,
        'A' => AGENT,
        '1' => [95, 238, 167],
        '3' => [44, 97, 72],
        '4' => [144, 238, 79],
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

/// Le point `(x, y)` du treillis (échelle 1) d'une image.
fn point(rvb: &[u8], largeur: usize, x: usize, y: usize) -> [u8; 3] {
    let i = 3 * (y * largeur + x);
    [rvb[i], rvb[i + 1], rvb[i + 2]]
}

/// juste : le petit réseau à la main (V-APE-01), au treillis de 8 × 3 points (`TL = 2 * 3 + 2 = 8`, `th = 3` : le
/// panneau de chaque étage fait 3 points de large, l'étage 1 commence à la colonne 5), les concentrations `[1, 0.5, 0,
/// 0.2, 0.8]` et les références `1` (tubes) et `1` (agent) :
/// ```text
/// A V 1 F F 4 F F      le nœud 0 (niveau 255 : AGENT pur, quel que soit le tube) ; le milieu (0,1) ; le nœud 1 :
///                      mélange (255 - 128, 128) du tube vif et de l'agent ; le nœud 4 sur l'étage 1 (colonne 5) : la
///                      teinte du lien (202, 172, 74) mêlée à l'agent au niveau 204
/// B V F F F F F F      le milieu (0,2) de niveau 128 ; le croisement (0,3)/(1,2) : le plus épais (V) ; le milieu
///                      (1,3) de niveau 0 : le fond. Les tubes ne changent pas : seuls les nœuds sont repeints
/// V C 3 F F F F F      le nœud 2 (niveau 0 : tel quel, la couleur de son plus fort tube) ; le milieu (2,3) de
///                      niveau 1 ;
///                      le nœud 3 (la teinte (22, 57, 70) mêlée à l'agent au niveau 51)
/// ```
/// Le mélange : `(a * (255 - n) + b * n + 127) / 255` par canal, en entiers ; à la main pour le nœud 1 (`n = 128`) :
/// rouge `(60 * 127 + 130 * 128 + 127) / 255 = 95`, vert `(220 * 127 + 255 * 128 + 127) / 255 = 238`, bleu
/// `(255 * 127 + 80 * 128 + 127) / 255 = 167`. Aux échelles 1, 2 et 3 (chaque point devient un carré). Hors des quatre
/// points de nœud peints (niveaux 255, 128, 51, 204), l'image est **exactement** celle de `tubes::apercu` ; le nœud 2,
/// de niveau 0, ne change pas non plus.
///
/// Ne couvre pas : d'autres graphes (`trois_apercus_calcules_a_part`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn un_petit_apercu_a_la_main() {
    let (g, d) = petit_reseau();
    let c = [1.0, 0.5, 0.0, 0.2, 0.8];
    let dessin = ["AV1FF4FF", "BVFFFFFF", "VC3FFFFF"];
    assert_eq!(treillis(&g), (8, 3));
    for echelle in [1_usize, 2, 3] {
        let image = apercu(&g, &d, &c, 1.0, 1.0, echelle as u32);
        assert_eq!((image.largeur as usize, image.hauteur as usize), (8 * echelle, 3 * echelle));
        assert_eq!(image.rvb.len(), 3 * 8 * 3 * echelle * echelle);
        assert_eq!(image.rvb, octets_attendus(&dessin, echelle), "échelle {echelle}");
    }
    // Hors des points des nœuds peints, l'image est celle des tubes.
    let moi = apercu(&g, &d, &c, 1.0, 1.0, 1);
    let tubes = apercu_tubes(&g, &d, 1.0, 1);
    assert_eq!(tubes.rvb.len(), moi.rvb.len());
    let peints = [(0, 0), (2, 0), (5, 0), (2, 2)];
    let mut differents = 0;
    for y in 0..3 {
        for x in 0..8 {
            let (a, b) = (point(&moi.rvb, 8, x, y), point(&tubes.rvb, 8, x, y));
            if peints.contains(&(x, y)) {
                assert_ne!(a, b, "le point ({x}, {y}) d'un nœud de niveau > 0 est repeint");
                differents += 1;
            } else {
                assert_eq!(a, b, "le point ({x}, {y}) n'est pas repeint");
            }
        }
    }
    assert_eq!(differents, 4);
}

/// juste : **les niveaux** (V-APE-02). Sur le petit réseau, `reference_agent = 1` : la concentration `0.5` donne le
/// niveau 128 (`0.5 * 255 = 127.5` arrondi vers l'extérieur) ; le plus grand flottant sous `0.5`
/// (`0.49999999999999994`) donne 127 : le nœud 1 devient `(95, 237, 168)` au lieu de `(95, 238, 167)`. Le nœud 3 (sa
/// couleur de tube `(22, 57, 70)`) : `0.0019` (niveau `0.4845` arrondi : 0), `0.001`, `0.0` et même une concentration
/// négative en entrée de l'aperçu (niveau 0) le laissent tel quel ; `0.0021` (niveau 1) donne `(22, 58, 70)`. La
/// saturation : `5` (au-delà de la référence) donne la couleur `AGENT` pure, comme `1`. Des concentrations toutes
/// nulles : l'image des tubes, octet pour octet, à plusieurs échelles. `reference_agent = 2` et concentrations doublées
/// : la même image qu'avec `1` et les concentrations d'origine (le niveau ne dépend que du rapport).
///
/// Ne couvre pas : le détail du mélange sur des teintes de lien (`un_petit_apercu_a_la_main`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn les_niveaux_de_concentration() {
    let (g, d) = petit_reseau();
    let nu = |c: &[f64], r: f64| apercu(&g, &d, c, 1.0, r, 1);
    // 127 contre 128.
    let n128 = nu(&[1.0, 0.5, 0.0, 0.2, 0.8], 1.0);
    assert_eq!(point(&n128.rvb, 8, 2, 0), [95, 238, 167]);
    let n127 = nu(&[1.0, 0.49999999999999994, 0.0, 0.2, 0.8], 1.0);
    assert_eq!(point(&n127.rvb, 8, 2, 0), [95, 237, 168]);
    // Le nœud 3 : niveaux 0 et 1.
    let pur = point(&apercu_tubes(&g, &d, 1.0, 1).rvb, 8, 2, 2);
    assert_eq!(pur, [22, 57, 70]);
    for c3 in [0.0019, 0.001, 0.0, -0.5] {
        let image = nu(&[0.0, 0.0, 0.0, c3, 0.0], 1.0);
        assert_eq!(point(&image.rvb, 8, 2, 2), pur, "c = {c3} : niveau 0, le point reste tel quel");
    }
    let image = nu(&[0.0, 0.0, 0.0, 0.0021, 0.0], 1.0);
    assert_eq!(point(&image.rvb, 8, 2, 2), [22, 58, 70], "niveau 1");
    // La saturation.
    let sature = nu(&[5.0, 5.0, 5.0, 5.0, 5.0], 1.0);
    let un = nu(&[1.0, 1.0, 1.0, 1.0, 1.0], 1.0);
    assert_eq!(sature, un);
    for (x, y) in [(0, 0), (2, 0), (5, 0), (2, 2)] {
        assert_eq!(point(&sature.rvb, 8, x, y), AGENT, "le nœud en ({x}, {y}) est AGENT pur");
    }
    // Des zéros : les tubes seuls.
    for echelle in [1, 3] {
        let image = apercu(&g, &d, &[0.0; 5], 1.0, 1.0, echelle);
        assert_eq!(image, apercu_tubes(&g, &d, 1.0, echelle));
    }
    // Seul le rapport compte.
    let c = [1.0, 0.5, 0.0, 0.2, 0.8];
    let doubles: Vec<f64> = c.iter().map(|x| 2.0 * x).collect();
    assert_eq!(nu(&doubles, 2.0), nu(&c, 1.0));
}

/// juste : **deux nœuds au même point** (V-APE-03) : les nœuds repeignent leur point **par indice croissant**, le
/// second mélangeant sur le résultat du premier. Trois nœuds, `(0, 0)` deux fois (nœuds 0 et 1) et `(1, 0)`, deux tubes
/// `(0,2)` et `(1,2)` de `D = 1` : le point commun est d'abord `VIF` (le plus fort tube de chaque nœud est vif). Avec
/// les concentrations `[0.5, 0.5, 0]` : deux mélanges de niveau 128 de suite : `(113, 247, 123)` (un seul mélange
/// donnerait `(95, 238, 167)`) ; avec `[1, 0.5, 0]` et avec `[0.5, 1, 0]` : `AGENT` pur (le niveau 255 recouvre tout,
/// le second ou le premier) ; avec des niveaux 1 et 27 (`[0.0021, 0.106, 0]`) : `(67, 224, 236)`, et dans l'autre
/// ordre (`[0.106, 0.0021, 0]`) `(67, 224, 235)` : l'ordre des indices se voit. Le troisième nœud (concentration
/// nulle) garde `VIF`.
///
/// Ne couvre pas : les points communs aux nœuds de deux étages différents (ils ne coïncident jamais : un étage a son
/// propre panneau).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn deux_noeuds_au_meme_point() {
    let noeuds = vec![Noeud { x: 0, y: 0, etage: 0 }, Noeud { x: 0, y: 0, etage: 0 }, Noeud { x: 1, y: 0, etage: 0 }];
    let aretes = vec![Arete { a: 0, b: 2, longueur: 1.0, cout: 1.0 }, Arete { a: 1, b: 2, longueur: 1.0, cout: 1.0 }];
    let g = Graphe::new(noeuds, aretes).unwrap();
    let d = [1.0, 1.0];
    // Niveaux 1 puis 27 : (67, 224, 236) ; 27 puis 1 : (67, 224, 235) : l'ordre se voit (ref_souvenir.py).
    let cas: [([f64; 3], [u8; 3]); 6] = [
        ([0.5, 0.5, 0.0], [113, 247, 123]),
        ([1.0, 0.5, 0.0], AGENT),
        ([0.5, 1.0, 0.0], AGENT),
        ([0.0, 0.0, 0.0], VIF),
        ([0.0021, 0.106, 0.0], [67, 224, 236]),
        ([0.106, 0.0021, 0.0], [67, 224, 235]),
    ];
    for (c, attendu) in cas {
        let image = apercu(&g, &d, &c, 1.0, 1.0, 1);
        assert_eq!((image.largeur, image.hauteur), (3, 1));
        assert_eq!(point(&image.rvb, 3, 0, 0), attendu, "concentrations {c:?}");
        assert_eq!(point(&image.rvb, 3, 1, 0), VIF, "le milieu des tubes");
        assert_eq!(point(&image.rvb, 3, 2, 0), VIF, "le troisième nœud, sans agent");
    }
}

/// Le réseau d'un cas à empreinte : les tubes de Tero (`Tubes::au_hasard`, graine `graine`) avancent de `pas_tubes` pas
/// (source, puits, débit 1, `h = 0.1`) ; le flux du dernier pas fait avancer un agent libéré à `lacher` (concentration
/// 1) de `pas_agent` pas de `0.1` ; rend l'empreinte de l'aperçu et ses dimensions.
#[allow(clippy::too_many_arguments)]
fn cas_a_empreinte(
    grille: &Grille,
    source: usize,
    puits: usize,
    pas_tubes: usize,
    graine: u64,
    lacher: usize,
    transport: &Transport,
    pas_agent: usize,
    references: (f64, f64),
    echelle: u32,
) -> (String, u32, u32) {
    let g = grille.graphe().unwrap();
    let mut alea = Alea::new(graine);
    let mut tubes = Tubes::au_hasard(&g, &mut alea);
    let mut q = vec![];
    for _ in 0..pas_tubes {
        q = tubes.avancer(&g, source, puits, 1.0, 0.1).unwrap().flux;
    }
    let mut c0 = vec![0.0; g.nb_noeuds()];
    c0[lacher] = 1.0;
    let mut agent = Agent::new(&g, c0).unwrap();
    for _ in 0..pas_agent {
        agent.avancer(&g, &q, transport, &[], 0.1).unwrap();
    }
    let image = apercu(&g, tubes.conductivites(), agent.concentrations(), references.0, references.1, echelle);
    (sha256_hex(&image.rvb), image.largeur, image.hauteur)
}

/// juste : **trois aperçus de grands cas**, dont les empreintes (sha256 des octets RVB) sont calculées à part par
/// `ref_souvenir.py` (V-APE-04), qui rejoue les tubes (PCG32, solveur, pas), le transport et le dessin :
/// - **A** : une grille de `12 × 9` en voisinage 8, source `(1, 4)`, puits `(10, 4)`, graine 5, 300 pas des tubes
///   (convergés : neuf tubes vifs, la ligne `y = 4`), un agent libéré en `(2, 4)`, `κ = 0.01`, `k = 0`, 30 pas de
///   `0.1`, références `(1, 0.25)`, échelle 4 : `92 × 68` pixels, `1c0a1f3d…45d9` ; la même à l'échelle 1 (`23 × 17`) :
///   `52ab293f…298b` ;
/// - **B** : deux étages de `6 × 5` en voisinage 4, lien vertical de coût `0.5`, source `(0, 2)` à l'étage 0, puits
///   `(3, 2)` à l'étage 1 (le chemin monte), graine 9, 200 pas des tubes, agent libéré en `(1, 2)` à l'étage 0, `κ =
///   0.02`, `k = 0.005`, 25 pas, références `(0.5, 0.3)`, échelle 3 : `72 × 27` pixels, `68a91f1c…6017`. Le modèle
///   Python est tombé juste du premier coup (dit dans le rapport) ; ces trois valeurs se recalculent par
///   `outils-2T/valeurs_2t_apercu.py`.
///
/// Ne couvre pas : l'œil (un aperçu embelli ne change aucune empreinte de scénario, mais change celles-ci).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn trois_apercus_calcules_a_part() {
    let a = Grille {
        largeur: 12,
        hauteur: 9,
        etages: 1,
        voisinage: Voisinage::Huit,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let tr = Transport { diffusion: 0.01, perte: 0.0 };
    let (source, puits, lacher) = (a.indice(1, 4, 0), a.indice(10, 4, 0), a.indice(2, 4, 0));
    let (sha, l, h) = cas_a_empreinte(&a, source, puits, 300, 5, lacher, &tr, 30, (1.0, 0.25), 4);
    assert_eq!((l, h), (92, 68));
    assert_eq!(sha, "1c0a1f3d72c4b5681687dde9392b51dbe75298f4cdff86e5a372a3c0347145d9");
    let (sha, l, h) = cas_a_empreinte(&a, source, puits, 300, 5, lacher, &tr, 30, (1.0, 0.25), 1);
    assert_eq!((l, h), (23, 17));
    assert_eq!(sha, "52ab293f2aa14dad8b15bcad16b0b673aeded206bb9e84cdcc8984ca991f298b");
    let b = Grille {
        largeur: 6,
        hauteur: 5,
        etages: 2,
        voisinage: Voisinage::Quatre,
        longueur_verticale: 1.0,
        cout_vertical: 0.5,
    };
    let tr = Transport { diffusion: 0.02, perte: 0.005 };
    let (source, puits, lacher) = (b.indice(0, 2, 0), b.indice(3, 2, 1), b.indice(1, 2, 0));
    let (sha, l, h) = cas_a_empreinte(&b, source, puits, 200, 9, lacher, &tr, 25, (0.5, 0.3), 3);
    assert_eq!((l, h), (72, 27));
    assert_eq!(sha, "68a91f1c9c6a4c034bd22222945f5cf863ab1d7425075119f0daa2621f2b6017");
}

/// juste : les paniques de `apercu` (V-APE-05) : `echelle == 0`, `reference_tubes` ou `reference_agent` nul, négatif,
/// NaN ou infini, `conductivites` ou `concentrations` de mauvaise taille. Un cas positif d'abord, puis `catch_unwind`
/// sur le seul appel fautif.
///
/// Ne couvre pas : les couleurs (les tests précédents).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn les_paniques_de_l_apercu() {
    let (g, d) = petit_reseau();
    let c = vec![0.0; 5];
    assert!(!apercu(&g, &d, &c, 1.0, 1.0, 1).rvb.is_empty());
    let paniques = |d: Vec<f64>, c: Vec<f64>, rt: f64, ra: f64, e: u32| {
        let g = g.clone();
        std::panic::catch_unwind(move || apercu(&g, &d, &c, rt, ra, e)).is_err()
    };
    assert!(paniques(d.clone(), c.clone(), 1.0, 1.0, 0), "échelle nulle");
    for r in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(paniques(d.clone(), c.clone(), r, 1.0, 1), "reference_tubes {r}");
        assert!(paniques(d.clone(), c.clone(), 1.0, r, 1), "reference_agent {r}");
    }
    assert!(paniques(d[..6].to_vec(), c.clone(), 1.0, 1.0, 1), "une conductivité de moins");
    assert!(paniques(d.clone(), c[..4].to_vec(), 1.0, 1.0, 1), "une concentration de moins");
    assert!(paniques(d.clone(), vec![0.0; 6], 1.0, 1.0, 1), "une concentration de plus");
}
