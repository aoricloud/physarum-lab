//! Étape 0 : `alea` (PCG32). Les valeurs attendues sont décrites, avec leur source, dans
//! `references/etape-0-valeurs.md` (identifiants `V-ALEA-xx`, cités dans chaque test).
//!
//! Deux familles de sources : le vecteur officiel de la démonstration PCG (graine 42, flux 54), lu sur pcg-random.org ;
//! et un modèle Python de `pcg_basic.c` (la transcription du C de référence), qui reproduit d'abord le vecteur officiel
//! puis produit les autres valeurs. Aucune valeur ne vient du code Rust.

use physarum_lab::alea::Alea;

/// Les six premiers tirages officiels de la graine 42, flux 54 (V-ALEA-01).
const OFFICIEL: [u32; 6] = [0xA15C02B7, 0x7B47F409, 0xBA1D3330, 0x83D2F293, 0xBFA4784B, 0xCBED606E];

fn tirages(alea: &mut Alea, n: usize) -> Vec<u32> {
    (0..n).map(|_| alea.suivant()).collect()
}

/// juste : les six premiers tirages de la graine 42 et du flux 54 sont ceux du programme de démonstration officiel de
/// PCG (V-ALEA-01, pcg-random.org/using-pcg-c-basic.html, ligne « 32bit » de la ronde 1).
///
/// Ne couvre pas : les autres graines et flux (voir `octets_apres_amorcage` et `etat_apres_tirages`) ; la qualité
/// statistique du générateur.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn officiel_six_premiers_tirages() {
    let mut alea = Alea::avec_flux(42, 54);
    assert_eq!(tirages(&mut alea, 6), OFFICIEL);
}

/// juste : après les six tirages, 65 pièces (`borne(2)`, `H` si non nul) puis 33 dés (`borne(6) + 1`) donnent ce que
/// la démonstration officielle imprime pour la ronde 1, lignes « Coins » et « Rolls » en entier (V-ALEA-02 et V-ALEA-03 ;
/// l'ordre des appels est celui de `pcg32-demo.c` : 6 tirages, 65 pièces, 33 dés).
///
/// Ne couvre pas : les bornes avec rejet (pour 2 le seuil est 0, pour 6 il vaut 4 et un rejet est improbable : voir
/// `borne_*` pour le rejet).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn officiel_pieces_et_des() {
    let mut alea = Alea::avec_flux(42, 54);
    tirages(&mut alea, 6);
    let pieces: String = (0..65).map(|_| if alea.borne(2) != 0 { 'H' } else { 'T' }).collect();
    assert_eq!(pieces, "HHTTTHTHHHTHTTTHHHHHTTTHHHTHTHTHTTHTTTHHHHHHTTTTHHTTTTTHTTTTTTTHT");
    let des: Vec<u32> = (0..33).map(|_| alea.borne(6) + 1).collect();
    assert_eq!(
        des,
        [3, 4, 1, 1, 2, 2, 3, 2, 4, 3, 2, 4, 3, 3, 5, 2, 3, 1, 3, 1, 5, 1, 4, 1, 5, 6, 4, 6, 6, 2, 6, 3, 3]
    );
}

/// juste : `Alea::new(g)` est `Alea::avec_flux(g, 54)` : mêmes octets d'état, mêmes tirages ; et l'état juste après
/// `new(7)`, `new(1)`, `new(u64::MAX)` est celui du modèle (V-ALEA-04).
///
/// Ne couvre pas : les autres flux.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn new_est_le_flux_54() {
    assert_eq!(Alea::new(42), Alea::avec_flux(42, 54));
    let mut a = Alea::new(42);
    assert_eq!(tirages(&mut a, 6), OFFICIEL);
    assert_eq!(Alea::new(7).octets(), [0xD1, 0xA0, 0xBD, 0xB3, 0x86, 0xA4, 0x22, 0x05, 0x6D, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(Alea::new(1).octets(), [0xC3, 0xA5, 0x3C, 0xE8, 0x76, 0xEB, 0x36, 0xF3, 0x6D, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        Alea::new(u64::MAX).octets(),
        [0x69, 0xA7, 0x11, 0x4F, 0x1C, 0x03, 0x93, 0x42, 0x6D, 0, 0, 0, 0, 0, 0, 0]
    );
}

/// juste : l'écriture canonique de l'état (état puis incrément, `u64` petit-boutiste) juste après l'amorçage, pour quatre
/// couples (graine, flux) dont les cas limites : l'incrément `(flux << 1) | 1` perd le bit de poids fort de `flux` (flux
/// `u64::MAX` donne `0xFFFF…FF`), et l'addition de la graine se fait modulo 2^64 (V-ALEA-04, modèle Python de
/// `pcg32_srandom_r`).
///
/// Ne couvre pas : l'état après des tirages (voir `etat_apres_tirages`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn octets_apres_amorcage() {
    let cas: [((u64, u64), [u8; 16]); 4] = [
        ((42, 54), [0xF8, 0x03, 0x2E, 0x2C, 0xB8, 0x06, 0x57, 0x18, 0x6D, 0, 0, 0, 0, 0, 0, 0]),
        ((0, 0), [0x2E, 0x7F, 0x95, 0x4C, 0x2D, 0xF4, 0x51, 0x58, 0x01, 0, 0, 0, 0, 0, 0, 0]),
        (
            (u64::MAX, u64::MAX),
            [0xA5, 0x01, 0xD5, 0x66, 0xA5, 0x17, 0x5C, 0x4F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ),
        ((123456789, 987654321), [0x7B, 0x34, 0x86, 0x85, 0x9D, 0x97, 0xC2, 0xFD, 0x63, 0xD1, 0xBC, 0x75, 0, 0, 0, 0]),
    ];
    for ((graine, flux), attendu) in cas {
        assert_eq!(Alea::avec_flux(graine, flux).octets(), attendu, "graine {graine}, flux {flux}");
    }
}

/// juste : les quatre premiers tirages et l'état après eux, pour quatre couples (graine, flux) (V-ALEA-04) ; et l'état
/// de (42, 54) après un seul tirage.
///
/// Ne couvre pas : les longues séries (voir `longues_series`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn etat_apres_tirages() {
    let cas = [
        (
            (42, 54),
            [0xA15C02B7, 0x7B47F409, 0xBA1D3330, 0x83D2F293],
            [0x94, 0xF6, 0x16, 0xAA, 0x97, 0x9E, 0xBF, 0xEB, 0x6D, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            (0, 0),
            [0xE4C14788, 0x379C6516, 0x5C4AB3BB, 0x601D23E0],
            [0x1A, 0xEC, 0x17, 0x29, 0x04, 0x5B, 0x70, 0x30, 0x01, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            (u64::MAX, u64::MAX),
            [0x2675C047, 0x7779A837, 0xA145AA13, 0x5F6BE726],
            [0xE9, 0x3B, 0x8D, 0x80, 0x21, 0x7B, 0x22, 0x66, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
        ),
        (
            (123456789, 987654321),
            [0x70AA3B49, 0x2FE445CB, 0xC5EA87B6, 0x06DD9503],
            [0xCF, 0xD8, 0x71, 0x37, 0xE4, 0x28, 0x09, 0x1D, 0x63, 0xD1, 0xBC, 0x75, 0, 0, 0, 0],
        ),
    ];
    for ((graine, flux), sorties, octets) in cas {
        let mut alea = Alea::avec_flux(graine, flux);
        assert_eq!(tirages(&mut alea, 4), sorties, "graine {graine}, flux {flux}");
        assert_eq!(alea.octets(), octets, "état après 4 tirages, graine {graine}, flux {flux}");
    }
    let mut alea = Alea::avec_flux(42, 54);
    alea.suivant();
    assert_eq!(alea.octets(), [0x05, 0xBB, 0x66, 0x87, 0xD8, 0xFE, 0x47, 0x2B, 0x6D, 0, 0, 0, 0, 0, 0, 0]);
}

/// juste : le flux change la suite : pour la graine 42, les trois premiers tirages des flux 0, 1, 54, 55 ; et la graine
/// 54 avec le flux 42 n'est pas la graine 42 avec le flux 54 (V-ALEA-04).
///
/// Ne couvre pas : l'indépendance statistique des flux.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn flux_distincts() {
    let attendu: [(u64, [u32; 3]); 4] = [
        (0, [0x21B756EE, 0xC15EF750, 0x9548A9BD]),
        (1, [0x4DF1CCF9, 0xE5838752, 0x58ED9E10]),
        (54, [0xA15C02B7, 0x7B47F409, 0xBA1D3330]),
        (55, [0xADD2C78F, 0x335DE4AB, 0xB53E3ABC]),
    ];
    for (flux, sorties) in attendu {
        assert_eq!(tirages(&mut Alea::avec_flux(42, flux), 3), sorties, "flux {flux}");
    }
    assert_eq!(tirages(&mut Alea::avec_flux(54, 42), 3), [0x3FD4C3BA, 0xC773318F, 0x2C9CFE17]);
}

/// juste : `suivant_u64` fait deux tirages, le premier en poids fort : avec la graine 42 et le flux 54 ses trois premiers
/// résultats sont les paires de tirages officiels de V-ALEA-01 mises bout à bout (V-ALEA-05 ; calcul : `(a << 32) | b`).
///
/// Ne couvre pas : un `suivant_u64` appelé après un nombre impair de tirages.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn suivant_u64_deux_tirages_poids_fort_d_abord() {
    let mut alea = Alea::avec_flux(42, 54);
    assert_eq!(alea.suivant_u64(), 0xA15C02B7_7B47F409);
    assert_eq!(alea.suivant_u64(), 0xBA1D3330_83D2F293);
    assert_eq!(alea.suivant_u64(), 0xBFA4784B_CBED606E);
}

/// juste : `suivant_f64` vaut `(suivant_u64() >> 11) · 2^-53`, exactement : avec la graine 42 et le flux 54 les trois
/// premiers résultats ont pour numérateurs (le `u64` officiel décalé de 11 bits : calcul entier à partir de V-ALEA-01,
/// V-ALEA-05) 5677329748551934, 6548306420333150, 6742819640802732 ; chaque appel consomme deux tirages ; tous les
/// résultats sont dans `[0, 1)`.
///
/// Ne couvre pas : la loi (uniformité) des flottants ; les valeurs extrêmes `0` et `1 - 2^-53`.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn suivant_f64_est_exact() {
    let mut alea = Alea::avec_flux(42, 54);
    let echelle = 9007199254740992.0_f64; // 2^53
    for numerateur in [5677329748551934_u64, 6548306420333150, 6742819640802732] {
        let x = alea.suivant_f64();
        assert_eq!(x * echelle, numerateur as f64, "x = {x}");
    }
    let mut a = Alea::avec_flux(42, 54);
    let mut b = a.clone();
    a.suivant_f64();
    b.suivant();
    b.suivant();
    assert_eq!(a, b, "suivant_f64 consomme exactement deux tirages");
    let mut alea = Alea::new(5);
    for _ in 0..10_000 {
        let x = alea.suivant_f64();
        assert!((0.0..1.0).contains(&x), "x = {x}");
        assert_eq!((x * echelle).fract(), 0.0, "x n'est pas un multiple de 2^-53 : {x}");
    }
}

/// juste : `borne(n)` pour `n = 7`, `3_000_000_000`, `2^31 + 1` et `u32::MAX`, 24 résultats chacun, graine 42 et flux 54,
/// d'après le modèle de `pcg32_boundedrand_r` (V-ALEA-06). Pour `3_000_000_000` (seuil 1294967296) 10 tirages bruts sur 34
/// sont rejetés, pour `2^31 + 1` 16 sur 40 : un `r % n` sans rejet, ou un produit `r · n >> 32`, donne d'autres valeurs.
///
/// Ne couvre pas : les bornes dont aucun tirage n'est rejeté parmi ces 24 (`borne_au_seuil` couvre le bord exact).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn borne_valeurs_du_modele() {
    let cas: [(u32, [u32; 24]); 4] = [
        (7, [4, 4, 6, 0, 3, 2, 5, 1, 0, 0, 4, 6, 0, 5, 5, 4, 0, 2, 4, 2, 0, 2, 5, 5]),
        (
            3_000_000_000,
            [
                2707161783, 2068313097, 122475824, 2211639955, 215226955, 421331566, 217466285, 2167406445, 860803674,
                1181216144, 984091174, 2721289578, 2228905443, 470160530, 2998992390, 2441179440, 1442744599,
                2984805051, 261196357, 2036656260, 1597429668, 2233071061, 1838127612, 275887881,
            ],
        ),
        (
            2147483649,
            [
                559678134, 974992175, 64156306, 1067743306, 1273847917, 1069982636, 19922796, 1713320025, 2033732495,
                1836607525, 573805929, 81421794, 1322676881, 851508741, 293695791, 837321402, 1113712708, 85587412,
                1128404232, 544004037, 1680893138, 1645190127, 250740541, 1666704049,
            ],
        ),
        (
            4294967295,
            [
                2707161783, 2068313097, 3122475824, 2211639955, 3215226955, 3421331566, 3217466285, 2167406445,
                3860803674, 4181216144, 853247742, 499135993, 3984091174, 941769757, 731976663, 475758987, 2721289578,
                2228905443, 3470160530, 2998992390, 2441179440, 1442744599, 1206460561, 1214968473,
            ],
        ),
    ];
    for (n, attendu) in cas {
        let mut alea = Alea::avec_flux(42, 54);
        let obtenu: Vec<u32> = (0..24).map(|_| alea.borne(n)).collect();
        assert_eq!(obtenu, attendu, "borne({n})");
    }
}

/// juste : le bord exact du rejet. Des graines fabriquées en inversant l'amorçage (V-ALEA-07) donnent un premier tirage
/// brut égal au seuil, puis au seuil moins un. Pour `n = 3_000_000_000` (seuil 1294967296) : un tirage égal au seuil est
/// accepté (résultat `1294967296`, un seul tirage consommé), un tirage inférieur d'une unité est rejeté (le tirage
/// suivant, 3984252137, donne `984252137`, deux tirages consommés). Pour `n = u32::MAX` (seuil 1) : un premier tirage 1
/// est accepté (résultat 1) ; un premier tirage 0 est rejeté, le deuxième (0 encore) aussi, le troisième 1203932051
/// donne 1203932051 (trois tirages consommés). Un `>` pour un `>=`, ou un `<=` pour un `<`, échoue ici.
///
/// Ne couvre pas : les autres flux que 54.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn borne_au_seuil() {
    // (graine, n, premier tirage brut, résultat de borne(n), état après l'appel)
    let cas: [(u64, u32, u32, u32, [u8; 16]); 4] = [
        (
            17401024240561992018,
            3_000_000_000,
            1294967296,
            1294967296,
            [0x6D, 0x00, 0x00, 0x78, 0xF9, 0xEA, 0x77, 0x5D, 0x6D, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            2631533607250342226,
            3_000_000_000,
            1294967295,
            984252137,
            [0x96, 0x26, 0xA7, 0x3B, 0x98, 0xC1, 0xFA, 0x12, 0x6D, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            4099389142363320658,
            u32::MAX,
            1,
            1,
            [0x6D, 0x00, 0x00, 0x68, 0xF9, 0xAB, 0x64, 0x6A, 0x6D, 0, 0, 0, 0, 0, 0, 0],
        ),
        (
            18396926780031676754,
            u32::MAX,
            0,
            1203932051,
            [0xCB, 0x32, 0xD4, 0x45, 0x44, 0xF4, 0x98, 0x0B, 0x6D, 0, 0, 0, 0, 0, 0, 0],
        ),
    ];
    for (graine, n, premier, resultat, etat) in cas {
        assert_eq!(Alea::avec_flux(graine, 54).suivant(), premier, "le premier tirage brut de la graine {graine}");
        let mut alea = Alea::avec_flux(graine, 54);
        assert_eq!(alea.borne(n), resultat, "borne({n}) pour la graine {graine}");
        assert_eq!(alea.octets(), etat, "état après borne({n}) pour la graine {graine}");
    }
}

/// juste : `borne(1)` consomme un tirage et rend 0 ; pour une puissance de deux `n`, aucun rejet (seuil 0) et
/// `borne(n)` est `tirage & (n - 1)` (calcul exact, par définition de `r % n`, V-ALEA-09). Ancrage non circulaire : le
/// premier tirage officiel `0xA15C02B7` de (42, 54) masqué donne `1`, `0xB7`, `0x02B7` et `0x215C02B7` pour `n = 2`, `2^8`,
/// `2^16` et `2^31`, et la suite de `borne(256)` n'est pas nulle (un générateur qui rend toujours zéro échoue).
///
/// Ne couvre pas : les bornes qui ne sont pas des puissances de deux.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn borne_un_et_puissances_de_deux() {
    for (n, attendu) in [(2_u32, 1_u32), (256, 0xB7), (65536, 0x02B7), (1 << 31, 0x215C_02B7)] {
        assert_eq!(Alea::avec_flux(42, 54).borne(n), attendu, "premier tirage officiel réduit modulo {n}");
    }
    let mut alea = Alea::avec_flux(42, 54);
    assert!((0..50).map(|_| alea.borne(256)).any(|v| v != 0));
    let mut a = Alea::new(11);
    let mut b = a.clone();
    assert_eq!(a.borne(1), 0);
    b.suivant();
    assert_eq!(a, b, "borne(1) consomme exactement un tirage");
    for exposant in [1_u32, 2, 8, 16, 31] {
        let n = 1_u32 << exposant;
        let mut a = Alea::new(u64::from(exposant));
        let mut b = a.clone();
        for _ in 0..200 {
            assert_eq!(a.borne(n), b.suivant() & (n - 1), "n = {n}");
        }
    }
}

/// juste : `borne(0)` panique (V-ALEA-10 : la doc le dit).
///
/// Ne couvre pas : le message de la panique.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
#[should_panic]
fn borne_zero_panique() {
    Alea::new(1).borne(0);
}

/// juste : de longues séries comparées au modèle (V-ALEA-08) : après 100 000 tirages de (42, 54), le OU exclusif de tous
/// les tirages, leur somme modulo 2^64, le dernier tirage et l'état ; et 1 000 tirages de (`u64::MAX`, `u64::MAX`). Une
/// multiplication ou une addition qui panique au débordement, ou un `u32` pour l'état, échoue ici.
///
/// Ne couvre pas : les séries de plus de 100 000 tirages.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn longues_series() {
    let mut alea = Alea::avec_flux(42, 54);
    let (mut xor, mut somme, mut dernier) = (0_u32, 0_u64, 0_u32);
    for _ in 0..100_000 {
        dernier = alea.suivant();
        xor ^= dernier;
        somme = somme.wrapping_add(u64::from(dernier));
    }
    assert_eq!(xor, 0x842705F9);
    assert_eq!(somme, 214430088307331);
    assert_eq!(dernier, 1187223689);
    assert_eq!(alea.octets(), [0x58, 0xBB, 0x39, 0x3F, 0x80, 0x4C, 0xB4, 0x0F, 0x6D, 0, 0, 0, 0, 0, 0, 0]);

    let mut alea = Alea::avec_flux(u64::MAX, u64::MAX);
    let (mut xor, mut dernier) = (0_u32, 0_u32);
    for _ in 0..1000 {
        dernier = alea.suivant();
        xor ^= dernier;
    }
    assert_eq!(xor, 0xAAE84890);
    assert_eq!(dernier, 323556842);
    assert_eq!(
        alea.octets(),
        [0x4D, 0x39, 0xA7, 0xD0, 0xC8, 0x0B, 0xEA, 0x63, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
    );
}

/// juste : l'égalité de deux générateurs est celle de leur état : deux générateurs de même graine sont égaux, un tirage
/// de l'un les distingue, un clone suit la même suite que l'original (V-ALEA-11 : la doc de `Alea`).
///
/// Ne couvre pas : l'égalité entre deux états atteints par des chemins différents.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn egalite_et_clone() {
    let mut a = Alea::new(9);
    let b = Alea::new(9);
    assert_eq!(a, b);
    let mut copie = a.clone();
    a.suivant();
    assert_ne!(a, b);
    assert_ne!(Alea::new(9), Alea::new(10));
    assert_ne!(Alea::avec_flux(9, 1), Alea::avec_flux(9, 2));
    copie.suivant();
    assert_eq!(a, copie);
}
