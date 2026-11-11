//! Étape 2, partie 2T : la recette du pas de transport, à la lettre, sur de petits graphes (une chaîne, une étoile, un
//! nœud seul). Valeurs attendues : `references/etape-2t-valeurs.md`, identifiants `V-REC-xx`.
//!
//! La source est la doc de `souvenir` (« Le pas, à la lettre »). Les cas dyadiques (des valeurs de la forme `m / 2^k`
//! où tout reste exact) se calculent à la main : le calcul est écrit en commentaire. Les autres valeurs viennent du
//! modèle Python `ref_souvenir.py`, écrit d'après la seule doc (une boucle par arête, quand le code jugé en fait une
//! par nœud), et se vérifient à la main là où c'est possible. Chaque test joue le pas par `pas_sans_decoupage` **et**
//! par `Agent::avancer` (un `h` sous le pas maximal : un seul sous-pas, mêmes bits).

#[path = "common/transport_2t.rs"]
mod transport_2t;

use physarum_lab::graphe::{Graphe, Noeud};
use physarum_lab::souvenir::{Agent, Bilan, Transport, pas_maximal, pas_sans_decoupage, taux_de_depart};
use transport_2t::{bits, chaine, graphe_l};

/// Joue `attendu.len()` pas de durée `h` : à chaque pas, `pas_sans_decoupage` et `Agent::avancer` rendent les mêmes
/// concentrations (aux bits attendus) et le même bilan `[sortie, perte, apport]` (aux bits attendus), avec `sous_pas`
/// égal à 1. `attendu` : pour chaque pas, les concentrations et le bilan.
fn jouer(
    graphe: &Graphe,
    flux: &[f64],
    depart: &[f64],
    tr: &Transport,
    apports: &[(usize, f64)],
    h: f64,
    attendu: &[(Vec<f64>, [f64; 3])],
) {
    assert!(!attendu.is_empty(), "il faut des pas attendus");
    let mut agent = Agent::new(graphe, depart.to_vec()).unwrap();
    let mut courant = depart.to_vec();
    for (t, (c, bilan)) in attendu.iter().enumerate() {
        let (suite, b) = pas_sans_decoupage(graphe, flux, &courant, tr, apports, h).unwrap();
        assert_eq!(bits(&suite), bits(c), "pas {} : concentrations ({suite:?})", t + 1);
        assert_eq!(bits(&[b.sortie, b.perte, b.apport]), bits(bilan), "pas {} : bilan ({b:?})", t + 1);
        assert_eq!(b.sous_pas, 1);
        let bp: Bilan = agent.avancer(graphe, flux, tr, apports, h).unwrap();
        assert_eq!(bits(agent.concentrations()), bits(c), "pas {} : Agent::avancer", t + 1);
        assert_eq!(bp, b, "pas {} : le même bilan, un seul sous-pas", t + 1);
        courant = suite;
    }
}

/// juste : l'advection seule sur une chaîne `0-1-2-3` (V-REC-01), dyadique, à la main. Les arêtes `(0,1)`, `(1,2)`,
/// `(2,3)` ont un flux de `0.5` chacune (le liquide va de `0` vers `3`), `κ = 0`, `k = 0`, `h = 1`, sans apport.
/// - le nœud 0 : `s = +0.5` (la source : le liquide entre sans agent) ; les nœuds 1 et 2 : `s = 0` ; le nœud 3 : `s =
///   -0.5` (le puits : `sortie = 0.5`) ; les taux de départ valent tous `0.5` (`0.5` donné par 0, 1, 2, et la sortie de
///   3) et le pas maximal `1 / 0.5 = 2` ;
/// - de `c = [1, 0, 0, 0]` : `c0' = (1 - 0.5) * 1 = 0.5`, `c1' = 0.5 * 0 + (1 * 0.5) * 1 = 0.5`, puis `[0.25, 0.5,
///   0.25, 0]`, `[0.125, 0.375, 0.375, 0.125]` (la masse vaut encore 1), puis, au pas 4, le puits emporte `(1 * 0.5) *
///   0.125 = 0.0625` : `[0.0625, 0.25, 0.375, 0.25]` (la masse vaut `0.9375 = 1 - 0.0625`), et au pas 5 il emporte `0.5
///   * 0.25 = 0.125`. Le sens inverse (flux `-0.5`) : le miroir exact (le puits est alors le nœud 0 : `sortie_0 = 0.5`)
///   : `c = [0, 0, 0, 1]` donne `[0, 0, 0.5, 0.5]`, `[0, 0.25, 0.5, 0.25]`, `[0.125, 0.375, 0.375, 0.125]`, puis au pas
///   4 une sortie de `0.0625` au nœud 0 : `[0.25, 0.375, 0.25, 0.0625]`. Un code qui ferait aller l'agent à
///   contre-courant, qui lirait `|Q|` pour `q`, ou qui ne tiendrait pas compte du sens `a -> b`, échoue ici.
///
/// Ne couvre pas : la diffusion, la perte, les apports (les tests suivants) ; des longueurs différentes.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn l_advection_sur_une_chaine() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    let q = [0.5, 0.5, 0.5];
    assert_eq!(bits(&taux_de_depart(&g, &q, &tr).unwrap()), bits(&[0.5; 4]), "les taux de départ");
    assert_eq!(pas_maximal(&g, &q, &tr).unwrap().to_bits(), 2.0_f64.to_bits(), "le pas maximal");
    let attendu = vec![
        (vec![0.5, 0.5, 0.0, 0.0], [0.0, 0.0, 0.0]),
        (vec![0.25, 0.5, 0.25, 0.0], [0.0, 0.0, 0.0]),
        (vec![0.125, 0.375, 0.375, 0.125], [0.0, 0.0, 0.0]),
        (vec![0.0625, 0.25, 0.375, 0.25], [0.0625, 0.0, 0.0]),
        (vec![0.03125, 0.15625, 0.3125, 0.3125], [0.125, 0.0, 0.0]),
    ];
    jouer(&g, &q, &[1.0, 0.0, 0.0, 0.0], &tr, &[], 1.0, &attendu);
    // Le sens inverse.
    let q = [-0.5, -0.5, -0.5];
    assert_eq!(bits(&taux_de_depart(&g, &q, &tr).unwrap()), bits(&[0.5; 4]));
    assert_eq!(pas_maximal(&g, &q, &tr).unwrap().to_bits(), 2.0_f64.to_bits());
    let attendu = vec![
        (vec![0.0, 0.0, 0.5, 0.5], [0.0, 0.0, 0.0]),
        (vec![0.0, 0.25, 0.5, 0.25], [0.0, 0.0, 0.0]),
        (vec![0.125, 0.375, 0.375, 0.125], [0.0, 0.0, 0.0]),
        (vec![0.25, 0.375, 0.25, 0.0625], [0.0625, 0.0, 0.0]),
    ];
    jouer(&g, &q, &[0.0, 0.0, 0.0, 1.0], &tr, &[], 1.0, &attendu);
}

/// juste : la diffusion seule (V-REC-02), dyadique, à la main. Flux nuls sur la chaîne `0-1-2-3` de longueurs 1, `κ =
/// 0.5`, `h = 0.5`, `c = [1, 0, 0, 0]` : les taux de départ sont `[0.5, 1, 1, 0.5]` (un voisin pour les bouts, deux
/// pour l'intérieur), le pas maximal `1`, et chaque nœud reçoit `(h * κ / L) * c_voisin = 0.25 * c_voisin`. `c0' = (1 -
/// 0.5 * 0.5) * 1 = 0.75`, `c1' = (1 - 0.5) * 0 + 0.25 * 1 + 0.25 * 0 = 0.25` : `[0.75, 0.25, 0, 0]` ; puis `[0.625,
/// 0.3125, 0.0625, 0]` ; `[0.546875, 0.328125, 0.109375, 0.015625]` ; `[0.4921875, 0.328125, 0.140625, 0.0390625]`. La
/// masse reste exactement `1`, le bilan reste nul (aucun nœud n'a de `s` non nul : `0.0` exactement).
///
/// Ne couvre pas : les longueurs différentes (`la_diffusion_lit_la_longueur_pas_le_cout`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn la_diffusion_seule() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    let tr = Transport { diffusion: 0.5, perte: 0.0 };
    let q = [0.0, 0.0, 0.0];
    assert_eq!(bits(&taux_de_depart(&g, &q, &tr).unwrap()), bits(&[0.5, 1.0, 1.0, 0.5]));
    assert_eq!(pas_maximal(&g, &q, &tr).unwrap().to_bits(), 1.0_f64.to_bits());
    let attendu = vec![
        (vec![0.75, 0.25, 0.0, 0.0], [0.0, 0.0, 0.0]),
        (vec![0.625, 0.3125, 0.0625, 0.0], [0.0, 0.0, 0.0]),
        (vec![0.546875, 0.328125, 0.109375, 0.015625], [0.0, 0.0, 0.0]),
        (vec![0.4921875, 0.328125, 0.140625, 0.0390625], [0.0, 0.0, 0.0]),
    ];
    jouer(&g, &q, &[1.0, 0.0, 0.0, 0.0], &tr, &[], 0.5, &attendu);
}

/// juste : la diffusion lit la **longueur** de l'arête, jamais son coût (V-REC-02). Une chaîne de longueurs `1, 2, 4`
/// (les coûts valent trois fois plus : 3, 6, 12), `κ = 0.5`, flux nuls, `h = 0.5` : le coefficient de diffusion de
/// chaque arête est `κ / L = 0.5, 0.25, 0.125` ; les taux de départ sont `[0.5, 0.75, 0.375, 0.125]`, le pas maximal `1
/// / 0.75 = 1.3333333333333335` (bits `0x3FF5555555555556`). De `c = [1, 0, 0, 0]` : `c0' = (1 - 0.5 * 0.5) * 1 =
/// 0.75`, `c1' = 0.5 * 0.5 * 1 = 0.25` (reçu par l'arête de longueur 1), puis `[0.625, 0.34375, 0.03125, 0]`, puis
/// `[0.5546875, 0.375, 0.068359375, 0.001953125]`.
///
/// Ne couvre pas : l'advection avec des longueurs (`tout_ensemble_dyadique`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn la_diffusion_lit_la_longueur_pas_le_cout() {
    let g = chaine(&[1.0, 2.0, 4.0]);
    let tr = Transport { diffusion: 0.5, perte: 0.0 };
    let q = [0.0, 0.0, 0.0];
    assert_eq!(bits(&taux_de_depart(&g, &q, &tr).unwrap()), bits(&[0.5, 0.75, 0.375, 0.125]));
    assert_eq!(pas_maximal(&g, &q, &tr).unwrap().to_bits(), 0x3FF5_5555_5555_5556);
    let attendu = vec![
        (vec![0.75, 0.25, 0.0, 0.0], [0.0, 0.0, 0.0]),
        (vec![0.625, 0.34375, 0.03125, 0.0], [0.0, 0.0, 0.0]),
        (vec![0.5546875, 0.375, 0.068359375, 0.001953125], [0.0, 0.0, 0.0]),
    ];
    jouer(&g, &q, &[1.0, 0.0, 0.0, 0.0], &tr, &[], 0.5, &attendu);
}

/// juste : tout ensemble, dyadique (V-REC-03) : la chaîne de longueurs `1, 2, 4`, flux `[0.5, 0.25, 0.25]`, `κ = 0.5`,
/// `k = 0.125`, un apport de `0.5` au nœud 0, `h = 0.5`, `c = [1, 0.5, 0.25, 0.125]` (masse `1.875`).
/// - `s = [+0.5, -0.25, 0, -0.25]` : le nœud 0 est la source, le nœud 1 laisse sortir `0.25` de liquide (un puits au
///   milieu de la chaîne : la règle est la même pour tous les nœuds), le nœud 3 aussi ;
/// - les taux : nœud 0 `0.5 + 0.5 + 0.125 = 1.125` ; nœud 1 `0.25 + (0.5 + 0.25) + 0.25 + 0.125 = 1.375` ;
///   nœud 2 `0.25 + (0.25 + 0.125) + 0.125 = 0.75` ; nœud 3 `0.125 + 0.25 + 0.125 = 0.5` ; pas maximal
///   `1 / 1.375 = 0.7272727272727273` (bits `0x3FE745D1745D1746`) ;
/// - pas 1 : `c = [0.8125, 0.6875, 0.2890625, 0.140625]`, sortie `0.078125` (`0.5 * 0.25 * 0.5 + 0.5 * 0.25 * 0.125`),
///   perte `0.1171875` (`0.5 * 0.125 * 1.875`), apport `0.25` : la masse passe de `1.875` à `1.9296875 = 1.875 -
///   0.078125 - 0.1171875 + 0.25` **exactement** ; les pas 2 et 3 donnent les valeurs écrites ci-dessous (le bilan
///   `masse_apres == masse_avant - sortie - perte + apport` est exact à chaque pas).
///
/// Ne couvre pas : les valeurs non dyadiques (`tout_ensemble_non_dyadique`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn tout_ensemble_dyadique() {
    let g = chaine(&[1.0, 2.0, 4.0]);
    let tr = Transport { diffusion: 0.5, perte: 0.125 };
    let q = [0.5, 0.25, 0.25];
    assert_eq!(bits(&taux_de_depart(&g, &q, &tr).unwrap()), bits(&[1.125, 1.375, 0.75, 0.5]));
    assert_eq!(pas_maximal(&g, &q, &tr).unwrap().to_bits(), 0x3FE7_45D1_745D_1746);
    let apports = [(0, 0.5)];
    let attendu = vec![
        (vec![0.8125, 0.6875, 0.2890625, 0.140625], [0.078125, 0.1171875, 0.25]),
        (vec![0.77734375, 0.6572265625, 0.361328125, 0.15966796875], [0.103515625, 0.12060546875, 0.25]),
        (vec![0.75439453125, 0.63922119140625, 0.400115966796875, 0.1875], [0.10211181640625, 0.122222900390625, 0.25]),
    ];
    jouer(&g, &q, &[1.0, 0.5, 0.25, 0.125], &tr, &apports, 0.5, &attendu);
    // Le bilan, exact à chaque pas.
    let mut c = vec![1.0, 0.5, 0.25, 0.125];
    for _ in 0..3 {
        let avant = c.iter().fold(0.0, |s, &x| s + x);
        let (suite, b) = pas_sans_decoupage(&g, &q, &c, &tr, &apports, 0.5).unwrap();
        let apres = suite.iter().fold(0.0, |s, &x| s + x);
        assert_eq!(apres, avant - b.sortie - b.perte + b.apport, "le bilan exact d'un cas dyadique");
        c = suite;
    }
}

/// juste : tout ensemble, non dyadique (V-REC-04), valeurs de `ref_souvenir.py`. La chaîne de longueurs 1, flux `[0.9,
/// 0.6, 0.3]` (trois nœuds, dont deux au milieu, où du liquide sort : `s = [0.9, -0.30000000000000004, -0.3, -0.3]`),
/// `κ = 0.3`, `k = 0.01`, apports `[(0, 0.5), (2, 0.25)]`, `h = 0.1`, `c = [0.3, 1.0, 0.2, 0.7]` : les taux de départ
/// `[1.21, 1.51, 1.21, 0.61]` (aux bits), le pas maximal `0.6622516556291391`, puis trois pas, concentrations et bilan
/// aux bits.
///
/// Ne couvre pas : des graphes autres que la chaîne (`l_ordre_des_sommes`, les tests de bits contre le pas naïf).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn tout_ensemble_non_dyadique() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    let tr = Transport { diffusion: 0.3, perte: 0.01 };
    let q = [0.9, 0.6, 0.3];
    let taux = taux_de_depart(&g, &q, &tr).unwrap();
    let attendu_taux = [0x3FF3_5C28_F5C2_8F5C_u64, 0x3FF8_28F5_C28F_5C29, 0x3FF3_5C28_F5C2_8F5C, 0x3FE3_851E_B851_EB85];
    assert_eq!(bits(&taux), attendu_taux.to_vec());
    assert_eq!(pas_maximal(&g, &q, &tr).unwrap().to_bits(), 0x3FE5_312A_6254_C4AA);
    let pas = [
        (
            [0x3FD5_FF2E_48E8_A71D_u64, 0x3FEC_8312_6E97_8D50, 0x3FD3_F487_FCB9_23A3, 0x3FE5_6AE7_D566_CF42],
            [0x3FAD_2F1A_9FBE_76C9_u64, 0x3F62_05BC_01A3_6E2F, 0x3FB3_3333_3333_3334],
        ),
        (
            [0x3FD8_3EF3_C635_D9D5, 0x3FE9_D369_33A0_407D, 0x3FD9_8ECE_647C_815C, 0x3FE4_B5B4_4CA3_46C1],
            [0x3FAC_C165_907D_9126, 0x3F62_26DE_911A_6F18, 0x3FB3_3333_3333_3334],
        ),
        (
            [0x3FDA_0FCB_B194_991A, 0x3FE7_C3A5_A99D_47CF, 0x3FDD_F4E2_0573_506A, 0x3FE4_3695_A30A_9EA3],
            [0x3FAC_7891_9D5D_03CE, 0x3F62_49BA_016F_CBFE, 0x3FB3_3333_3333_3334],
        ),
    ];
    let apports = [(0, 0.5), (2, 0.25)];
    let mut agent = Agent::new(&g, vec![0.3, 1.0, 0.2, 0.7]).unwrap();
    let mut courant = vec![0.3, 1.0, 0.2, 0.7];
    for (t, (c, b)) in pas.iter().enumerate() {
        let (suite, bilan) = pas_sans_decoupage(&g, &q, &courant, &tr, &apports, 0.1).unwrap();
        assert_eq!(bits(&suite), c.to_vec(), "pas {} : concentrations", t + 1);
        assert_eq!(bits(&[bilan.sortie, bilan.perte, bilan.apport]), b.to_vec(), "pas {} : bilan", t + 1);
        let bp = agent.avancer(&g, &q, &tr, &apports, 0.1).unwrap();
        assert_eq!(bits(agent.concentrations()), c.to_vec(), "pas {} : Agent::avancer", t + 1);
        assert_eq!(bp, bilan);
        courant = suite;
    }
}

/// juste : **l'ordre des sommes** (V-REC-05). Une étoile : le centre (nœud 0) relié aux feuilles 1 à 4 par quatre
/// arêtes de longueurs `1.5, 1, 1, 1` et de flux `[-0.49, -0.76, 0.42, 0.64]`, `κ = 0.24`, `k = 0.02`, `h = 0.2`, `c =
/// [1.54, 1.45, 0.45, 1.17, 1.49]`. Le centre est `a` de toutes ses arêtes : le taux `r_0` et la concentration `c0'`
/// dépendent de l'ordre dans lequel les quatre arêtes sont sommées et du mélange, arête par arête, de l'advection et de
/// la diffusion dans une même somme : `r_0 = 2.1500000000000004` (`0x4001333333333334`), alors que les quatre sommes
/// essayées à côté (les arêtes à l'envers ; l'advection et la diffusion sommées séparément, puis ajoutées ; les deux)
/// donnent `2.15` (`0x4001333333333333`), et `c0' = 1.2839799999999997` (`0x3FF48B2E9CCB7D40`) contre
/// `1.2839800000000001` (`0x3FF48B2E9CCB7D42`) pour les trois autres (recherche dans `chercher_etoile_2t.py`). Tout le
/// pas, aux bits, contre `ref_souvenir.py`. Le bilan : une sortie `0.34752000000000005` (le centre sort
/// `0.19000000000000006` de liquide, `s_0 = -0.19000000000000006`, et les feuilles 3 et 4 sont sources), une perte
/// `0.024399999999999998`.
///
/// Ne couvre pas : l'ordre des termes d'un apport (`les_apports_et_le_noeud_seul`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn l_ordre_des_sommes() {
    let aretes = [(0, 1, 1.5), (0, 2, 1.0), (0, 3, 1.0), (0, 4, 1.0)];
    let g = graphe_l(5, &aretes);
    let tr = Transport { diffusion: 0.24, perte: 0.02 };
    let q = [-0.49, -0.76, 0.42, 0.64];
    let c = [1.54, 1.45, 0.45, 1.17, 1.49];
    let taux = taux_de_depart(&g, &q, &tr).unwrap();
    let attendu_taux = [
        0x4001_3333_3333_3334_u64,
        0x3FE5_70A3_D70A_3D71,
        0x3FF0_51EB_851E_B852,
        0x3FE5_C28F_5C28_F5C2,
        0x3FEC_CCCC_CCCC_CCCD,
    ];
    assert_eq!(bits(&taux), attendu_taux.to_vec());
    assert_ne!(taux[0], 2.15, "2.15 serait l'ordre à l'envers ou les sommes séparées");
    let (suite, b) = pas_sans_decoupage(&g, &q, &c, &tr, &[], 0.2).unwrap();
    let attendu = [
        0x3FF4_8B2E_9CCB_7D40_u64,
        0x3FF4_E132_B55E_F1FE,
        0x3FDB_A7DA_A4FC_A42B,
        0x3FF3_6D33_0941_C822,
        0x3FF7_E2AC_3222_91FB,
    ];
    assert_eq!(bits(&suite), attendu.to_vec());
    assert_eq!(bits(&[b.sortie, b.perte, b.apport]), vec![0x3FD6_3DC4_86AD_2DCC_u64, 0x3F98_FC50_4816_F006, 0]);
}

/// juste : **la sortie suit la même règle à tous les nœuds, à l'arrondi près** (V-REC-06). Une étoile : le centre (nœud
/// 0) relié aux feuilles 1, 2, 3 par des arêtes de longueur 1 et de flux `[0.3, -0.1, -0.2]` : pour le centre, les
/// débits sortants sont `0.3`, `-0.1`, `-0.2`, et `s_0 = (0.3 - 0.1) - 0.2 = -2.7755575615628914e-17` (`0.3 + (-0.1)`
/// vaut `0.19999999999999998`, pas `0.2`) : un `s` **négatif**, de l'ordre de l'arrondi. La règle de la doc s'applique
/// sans seuil : `sortie_0 = 2.7755575615628914e-17`, un taux `r_0 = 0.30000000000000004` (`0.3 + 2.8e-17` arrondi), et,
/// avec `κ = 0`, `k = 0`, `h = 0.5`, `c = [1, 0, 0.5, 0.25]` (la feuille 1, le vrai puits, n'a pas d'agent : sa sortie
/// `0.3` n'emporte rien), le bilan a une sortie `(0.5 * 2.7755575615628914e-17) * 1 = 1.3877787807814457e-17` (`2^-56`,
/// bits `0x3C70000000000000`) : un code qui ignorerait un `s` minuscule, ou qui le compterait pour zéro, rend `0.0`.
/// Les concentrations d'après : `[0.9, 0.15, 0.475, 0.225]` aux bits de `ref_souvenir.py`.
///
/// Ne couvre pas : ce que coûte cette règle sur un vrai réseau (mesuré dans le rapport ; `la_masse_d_un_transport_
/// sur_des_flux_de_tubes`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn la_sortie_a_l_arrondi() {
    let g = graphe_l(4, &[(0, 1, 1.0), (0, 2, 1.0), (0, 3, 1.0)]);
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    let q = [0.3, -0.1, -0.2];
    let taux = taux_de_depart(&g, &q, &tr).unwrap();
    let attendu_taux = [0x3FD3_3333_3333_3334_u64, 0x3FD3_3333_3333_3333, 0x3FB9_9999_9999_999A, 0x3FC9_9999_9999_999A];
    assert_eq!(bits(&taux), attendu_taux.to_vec());
    let (suite, b) = pas_sans_decoupage(&g, &q, &[1.0, 0.0, 0.5, 0.25], &tr, &[], 0.5).unwrap();
    assert_eq!(b.sortie.to_bits(), 0x3C70_0000_0000_0000, "2^-56 : la sortie d'un s_i de -2.8e-17");
    assert_eq!(b.perte.to_bits(), 0);
    assert_eq!(b.apport.to_bits(), 0);
    let attendu = [0x3FEC_CCCC_CCCC_CCCD_u64, 0x3FC3_3333_3333_3333, 0x3FDE_6666_6666_6666, 0x3FCC_CCCC_CCCC_CCCD];
    assert_eq!(bits(&suite), attendu.to_vec());
}

/// juste : les apports et le cas d'un nœud sans arête (V-REC-07), valeurs de `ref_souvenir.py`. Un graphe d'un seul
/// nœud (aucune arête, flux vide) : `r_0 = k`, le pas maximal `1 / k`. Avec `k = 0.3`, `h = 0.1` : `c ← (1 - 0.03) c` à
/// chaque pas, de `0.3` : `0.291`, `0.28226999999999997`, `0.27380189999999993` (aux bits), et la perte vaut `(h * k) *
/// c = 0.009`, `0.00873`, `0.0084681`. **Deux apports sur le même nœud** s'ajoutent un à un, dans l'ordre de la liste,
/// après le calcul : avec `k = 0.12`, `h = 0.54`, `c = 0.52`, apports `[(0, 1.89), (0, 2.84)]` la concentration d'après
/// est `3.0405040000000003` (`0x400852F3C2DADB84`), dans l'ordre inverse `3.040504` (`0x400852F3C2DADB83`) : les deux
/// ordres ne donnent pas les mêmes bits. Le bilan d'apport vaut `2.5542`.
///
/// Ne couvre pas : un apport sur un nœud d'un graphe à arêtes (`tout_ensemble_dyadique`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn les_apports_et_le_noeud_seul() {
    let seul = Graphe::new(vec![Noeud { x: 0, y: 0, etage: 0 }], vec![]).unwrap();
    let tr = Transport { diffusion: 0.0, perte: 0.3 };
    assert_eq!(bits(&taux_de_depart(&seul, &[], &tr).unwrap()), bits(&[0.3]));
    assert_eq!(pas_maximal(&seul, &[], &tr).unwrap().to_bits(), (1.0_f64 / 0.3).to_bits());
    let mut c = vec![0.3];
    let attendu =
        [(0x3FD2_9FBE_76C8_B439_u64, 0.009), (0x3FD2_10B6_30A9_1537, 0.00873), (0x3FD1_85F8_6785_4A56, 0.0084681)];
    for (t, (bits_c, perte)) in attendu.iter().enumerate() {
        let (suite, b) = pas_sans_decoupage(&seul, &[], &c, &tr, &[], 0.1).unwrap();
        assert_eq!(suite[0].to_bits(), *bits_c, "pas {}", t + 1);
        assert_eq!(b.perte, *perte, "pas {} : perte", t + 1);
        c = suite;
    }
    let tr = Transport { diffusion: 0.0, perte: 0.12 };
    let (suite, b) = pas_sans_decoupage(&seul, &[], &[0.52], &tr, &[(0, 1.89), (0, 2.84)], 0.54).unwrap();
    assert_eq!(suite[0].to_bits(), 0x4008_52F3_C2DA_DB84);
    assert_eq!(b.apport.to_bits(), 0x4004_6F00_68DB_8BAC);
    let (suite, b) = pas_sans_decoupage(&seul, &[], &[0.52], &tr, &[(0, 2.84), (0, 1.89)], 0.54).unwrap();
    assert_eq!(suite[0].to_bits(), 0x4008_52F3_C2DA_DB83, "l'ordre de la liste compte aux bits");
    assert_eq!(b.apport.to_bits(), 0x4004_6F00_68DB_8BAC);
    // Un apport nul, ou une liste vide : rien n'est ajouté.
    let (a, _) = pas_sans_decoupage(&seul, &[], &[0.52], &tr, &[], 0.54).unwrap();
    let (b0, bil) = pas_sans_decoupage(&seul, &[], &[0.52], &tr, &[(0, 0.0)], 0.54).unwrap();
    assert_eq!(bits(&a), bits(&b0));
    assert_eq!(bil.apport.to_bits(), 0);
}

/// juste : des flux nuls, `κ = 0`, `k = 0` ne changent rien, aux bits (V-REC-07, cas dégénéré de la doc) : tous les
/// taux sont `0.0`, le pas maximal est `f64::INFINITY`, et un pas de durée `h = 5` (`n = 1`) rend les mêmes bits, un
/// bilan nul, `sous_pas == 1`. Un graphe sans arête, avec des concentrations quelconques : de même.
///
/// Ne couvre pas : le découpage au-dessus du pas maximal (`etape_2t_stabilite.rs`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn rien_ne_bouge_sans_flux_ni_diffusion_ni_perte() {
    let g = chaine(&[1.0, 2.0]);
    let tr = Transport { diffusion: 0.0, perte: 0.0 };
    let c = vec![0.3, 1.7, 0.0];
    assert_eq!(bits(&taux_de_depart(&g, &[0.0, 0.0], &tr).unwrap()), vec![0_u64; 3]);
    assert_eq!(pas_maximal(&g, &[0.0, 0.0], &tr).unwrap(), f64::INFINITY);
    let (suite, b) = pas_sans_decoupage(&g, &[0.0, 0.0], &c, &tr, &[], 5.0).unwrap();
    assert_eq!(bits(&suite), bits(&c));
    assert_eq!(bits(&[b.sortie, b.perte, b.apport]), vec![0_u64; 3]);
    let mut agent = Agent::new(&g, c.clone()).unwrap();
    let b = agent.avancer(&g, &[0.0, 0.0], &tr, &[], 5.0).unwrap();
    assert_eq!(bits(agent.concentrations()), bits(&c));
    assert_eq!(b.sous_pas, 1);
    // Le même flux, mais négatif et positif nul : `-0.0` ne change rien non plus.
    let (suite, _) = pas_sans_decoupage(&g, &[-0.0, 0.0], &c, &tr, &[], 5.0).unwrap();
    assert_eq!(bits(&suite), bits(&c));
}
