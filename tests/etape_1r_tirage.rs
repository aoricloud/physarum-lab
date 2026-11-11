//! Étape 1, partie 1R : le tirage de la paire (source, puits) à chaque pas : `reseaux::tirer_paire` et son emploi par
//! `Reseau::avancer`. Valeurs attendues : `references/etape-1r-valeurs.md`, identifiants `V-TIR-xx`.
//!
//! Source : Tero et al. 2010 (Science 327, p. 441) : « à chaque pas de temps, une source de nourriture tirée au hasard
//! » est la source et une seconde, tirée au hasard, le puits ; **la façon de tirer n'est pas publiée** : la doc de
//! `reseaux` la fixe (deux tirages `borne(k)` puis `borne(k - 1)`, toujours), à nous. La preuve est un tirage naïf
//! écrit dans le test avec `Alea::borne` (qui a ses propres vecteurs à l'étape 0), les suites du modèle Python
//! `ref_reseaux.py` (PCG32 transcrit du C de référence), des comptes exacts, et en plus des fréquences avec des seuils
//! mesurés.

#[path = "common/graphes_au_hasard.rs"]
mod graphes_au_hasard;

use graphes_au_hasard::graphe_pondere;
use physarum_lab::alea::Alea;
use physarum_lab::reseaux::{Loi, Reseau, tirer_paire};

/// Le tirage naïf d'après la doc : `i = borne(k)`, `j = borne(k - 1)`, puis `j + 1` si `j >= i`.
fn paire_naive(alea: &mut Alea, k: u32) -> (usize, usize) {
    let i = alea.borne(k);
    let mut j = alea.borne(k - 1);
    if j >= i {
        j += 1;
    }
    (i as usize, j as usize)
}

/// Les suites du modèle Python : `(graine, k, les douze (ou seize) premières paires, l'état final en hexadécimal)`.
#[allow(clippy::type_complexity)]
fn suites_python() -> Vec<(u64, usize, Vec<(usize, usize)>, &'static str)> {
    vec![
        (
            1,
            2,
            vec![(1, 0), (1, 0), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (0, 1), (1, 0), (1, 0), (1, 0), (0, 1)],
            "cbfc39affa187a206d00000000000000",
        ),
        (
            1,
            3,
            vec![(1, 0), (1, 0), (1, 2), (1, 2), (2, 1), (2, 0), (0, 1), (2, 0), (2, 0), (2, 0), (0, 2), (0, 2)],
            "cbfc39affa187a206d00000000000000",
        ),
        (
            7,
            5,
            vec![
                (3, 0),
                (3, 2),
                (0, 4),
                (1, 2),
                (0, 4),
                (3, 2),
                (4, 2),
                (0, 2),
                (2, 3),
                (1, 2),
                (4, 0),
                (1, 3),
                (3, 1),
                (3, 4),
                (4, 0),
                (2, 3),
            ],
            "315f1925f64351316d00000000000000",
        ),
        (
            3,
            16,
            vec![
                (5, 9),
                (6, 0),
                (0, 2),
                (0, 13),
                (10, 4),
                (2, 13),
                (11, 15),
                (14, 3),
                (15, 10),
                (4, 10),
                (11, 0),
                (13, 5),
            ],
            "e56aa9fbba8b8cee6d00000000000000",
        ),
        (
            2,
            4,
            vec![
                (0, 3),
                (0, 3),
                (0, 1),
                (0, 2),
                (2, 1),
                (1, 0),
                (2, 1),
                (0, 2),
                (0, 2),
                (0, 1),
                (3, 0),
                (3, 0),
                (0, 1),
                (0, 2),
                (3, 2),
                (2, 0),
            ],
            "d0e69886644783e36d00000000000000",
        ),
    ]
}

fn hex(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

/// juste : les suites de paires du modèle Python (V-TIR-01) : pour cinq couples (graine, `k`) (`k` = 2, 3, 5, 16, 4),
/// les 12 ou 16 premières paires de `tirer_paire` sont celles de `ref_reseaux.tirer_paire` (PCG32 transcrit du C de
/// référence, `bounded` comme `pcg32_boundedrand_r`) ; l'état du générateur après ces tirages est celui du modèle
/// (octets de `Alea::octets`). Avec la graine 1, `k = 2` donne `(1, 0), (1, 0), (0, 1)…`.
///
/// Ne couvre pas : les propriétés statistiques (`frequences_exactes_et_statistiques`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn les_paires_du_modele_python() {
    for (graine, k, paires, etat) in suites_python() {
        let mut alea = Alea::new(graine);
        let tirees: Vec<(usize, usize)> = (0..paires.len()).map(|_| tirer_paire(&mut alea, k)).collect();
        assert_eq!(tirees, paires, "graine {graine}, k = {k}");
        assert_eq!(hex(&alea.octets()), etat, "graine {graine}, k = {k} : l'état du générateur");
    }
}

/// juste : `tirer_paire` contre le tirage naïf du test (V-TIR-02), pour `k` de 2 à 20 (trois graines, 300 tirages
/// chacun) puis pour de grands `k` (1 000, 1 001, 1 002, 5 000 et 70 000 : 60 tirages, une graine) : la même paire **et
/// le même état du générateur** après chaque tirage (la consommation de hasard fait partie du résultat : deux tirages
/// de 32 bits au moins, davantage seulement en cas de rejet de `borne`). Les rangs sont dans `0..k` et différents.
///
/// Ne couvre pas : `k` grand au point que `borne` rejette souvent (le rejet est celui d'`Alea::borne`, éprouvé à
/// l'étape 0).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn tirer_paire_contre_le_tirage_naif() {
    let cas: Vec<(usize, u64, usize)> = (2..=20_usize)
        .flat_map(|k| [1_u64, 2, 3].map(|graine| (k, graine, 300)))
        .chain([1000, 1001, 1002, 5000, 70_000].map(|k| (k, 4, 60)))
        .collect();
    for (k, graine, nb_tirages) in cas {
        let (mut a, mut b) = (Alea::new(graine), Alea::new(graine));
        for tirage in 0..nb_tirages {
            let (i, j) = tirer_paire(&mut a, k);
            assert_eq!((i, j), paire_naive(&mut b, k as u32), "k = {k}, graine {graine}, tirage {tirage}");
            assert_eq!(a, b, "k = {k}, graine {graine}, tirage {tirage} : le générateur");
            assert!(i < k && j < k && i != j);
        }
        assert_ne!(a, Alea::new(graine), "le générateur a avancé");
    }
}

/// juste : `k = 2` consomme **toujours deux tirages de 32 bits** (V-TIR-03) : `borne(2)` en consomme un (le seuil de
/// rejet vaut 0 : aucun rejet), `borne(1)` en consomme un et rend 0. La paire est `(i, 1 - i)` avec `i = suivant() % 2`
/// pris sur le **premier** tirage, et l'état après `n` paires est celui de `2 n` appels de `suivant()` : calculé ici
/// sans passer par `borne`. Sur 500 tirages, six graines ; les deux paires `(0, 1)` et `(1, 0)` apparaissent.
///
/// Ne couvre pas : `k > 2`.
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn deux_nourritures_deux_tirages() {
    for graine in 1..=6_u64 {
        let (mut a, mut b) = (Alea::new(graine), Alea::new(graine));
        let mut vus = [0_usize; 2];
        for tirage in 0..500 {
            let (i, j) = tirer_paire(&mut a, 2);
            let premier = b.suivant();
            let _second = b.suivant();
            let attendu_i = (premier % 2) as usize;
            assert_eq!((i, j), (attendu_i, 1 - attendu_i), "graine {graine}, tirage {tirage}");
            assert_eq!(a, b, "graine {graine}, tirage {tirage}");
            vus[i] += 1;
        }
        assert!(vus[0] > 0 && vus[1] > 0, "les deux paires apparaissent : {vus:?}");
    }
}

/// juste : jamais source égale au puits, toujours dans `0..k` (V-TIR-04), pour `k` de 2 à 16, 2 000 tirages chacun ; et
/// toutes les `k (k - 1)` paires ordonnées apparaissent pour `k = 5` (20 paires) sur 2 000 tirages ; `tirer_paire`
/// **panique** pour `k = 0` et `k = 1` (cas positif d'abord).
///
/// Ne couvre pas : l'uniformité (test suivant).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn jamais_la_meme_nourriture_et_panique_sous_deux() {
    for k in 2..=16_usize {
        let mut alea = Alea::new(100 + k as u64);
        for _ in 0..2000 {
            let (i, j) = tirer_paire(&mut alea, k);
            assert!(i < k && j < k, "k = {k} : ({i}, {j})");
            assert_ne!(i, j, "k = {k}");
        }
    }
    let mut alea = Alea::new(5);
    let mut vues = [[false; 5]; 5];
    for _ in 0..2000 {
        let (i, j) = tirer_paire(&mut alea, 5);
        vues[i][j] = true;
    }
    let nb = vues.iter().flatten().filter(|&&v| v).count();
    assert_eq!(nb, 20, "les vingt paires ordonnées");
    for k in [0_usize, 1] {
        let resultat = std::panic::catch_unwind(|| {
            let mut alea = Alea::new(1);
            tirer_paire(&mut alea, k)
        });
        assert!(resultat.is_err(), "k = {k} doit paniquer");
    }
}

/// Les comptes de chaque paire ordonnée `(i, j)`, `i != j`, dans l'ordre `(0,1), (0,2), …, (1,0), (1,2), …`.
fn comptes(graine: u64, k: usize, n: usize) -> Vec<usize> {
    let mut alea = Alea::new(graine);
    let mut c = vec![vec![0_usize; k]; k];
    for _ in 0..n {
        let (i, j) = tirer_paire(&mut alea, k);
        c[i][j] += 1;
    }
    let mut plat = Vec::new();
    for (i, ligne) in c.iter().enumerate() {
        for (j, &v) in ligne.iter().enumerate() {
            if i != j {
                plat.push(v);
            }
        }
    }
    plat
}

/// Le `chi²` des comptes contre l'uniforme sur les `k (k - 1)` paires.
fn khi2(comptes: &[usize], n: usize) -> f64 {
    let attendu = n as f64 / comptes.len() as f64;
    comptes.iter().map(|&c| (c as f64 - attendu) * (c as f64 - attendu) / attendu).sum()
}

/// juste + mesure : les fréquences (V-TIR-05). (1) **Comptes exacts** du modèle Python : `k = 4`, graine 7, 20 000
/// tirages : `1660 1628 1589 1684 1682 1675 1650 1711 1696 1685 1715 1625` (les douze paires dans l'ordre `(0,1),
/// (0,2), (0,3), (1,0), …`) ; `k = 5`, graine 11, 100 000 tirages : les vingt comptes du modèle. (2) **Statistique** :
/// le `chi²` des comptes contre l'uniforme sur 20 paires (`k = 5`, 20 000 tirages, 19 degrés de liberté) ou 240 paires
/// (`k = 16`, 24 000 tirages, 239) : mesuré sur 40 **autres** graines (5000 à 5039) par le modèle Python : `k = 5`
/// moyenne 17,4, maximum 36,4 ; `k = 16` moyenne 238,2, maximum 305,2 (les moyennes sont celles de la loi : 19 et 239)
/// ; seuils `45` et `330` (le quantile à 99,9 % du `chi²` à 19 degrés vaut 43,8 ; pour 239, environ 305 à 315). Les
/// huit graines du test (1 à 8) donnent au plus 24,1 et 278,3. Un tirage biaisé (la source ou le puits plus fréquent)
/// dépasse ces seuils (l'écart d'un tirage qui oublie le décalage `j + 1` est de plusieurs milliers).
///
/// Ne couvre pas : l'indépendance entre pas successifs.
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn frequences_exactes_et_statistiques() {
    assert_eq!(
        comptes(7, 4, 20_000),
        [1660, 1628, 1589, 1684, 1682, 1675, 1650, 1711, 1696, 1685, 1715, 1625],
        "k = 4, graine 7"
    );
    assert_eq!(
        comptes(11, 5, 100_000),
        [
            5107, 4954, 4957, 5025, 5032, 5023, 4994, 4973, 4860, 5087, 5000, 4970, 5145, 4929, 4897, 4888, 5123, 5042,
            4979, 5015
        ],
        "k = 5, graine 11"
    );
    for graine in 1..=8_u64 {
        let c5 = comptes(graine, 5, 20_000);
        assert_eq!(c5.len(), 20);
        assert_eq!(c5.iter().sum::<usize>(), 20_000);
        let x5 = khi2(&c5, 20_000);
        assert!(x5 < 45.0, "k = 5, graine {graine} : chi² = {x5}");
        let c16 = comptes(graine, 16, 24_000);
        assert_eq!(c16.len(), 240);
        let x16 = khi2(&c16, 24_000);
        assert!(x16 < 330.0, "k = 16, graine {graine} : chi² = {x16}");
    }
}

/// juste : `Reseau::avancer` tire la paire avec `tirer_paire` (V-TIR-06) : sur 40 pas, la source et le puits rendus
/// sont les nourritures des rangs tirés par un tirage naïf sur un générateur parallèle (rangs `(i, j)` :
/// `nourritures[i]` est la source), le générateur de `avancer` est à chaque pas celui du tirage naïf, et les
/// conductivités sont celles d'un réseau jumeau qui avance avec `avancer_paire` sur la même paire. Cinq nourritures
/// dont les nœuds sont dans le désordre (rang et indice de nœud ne coïncident pas), loi sigmoïde.
///
/// Ne couvre pas : les erreurs (`etape_1r_loi::erreurs_et_ordre`).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn avancer_tire_la_paire() {
    let aretes: Vec<(usize, usize, f64)> =
        (0..7).map(|i| (i, i + 1, 1.0 + 0.25 * i as f64)).chain([(0, 4, 2.0), (2, 7, 3.0)]).collect();
    let g = graphe_pondere(8, &aretes);
    let foods = vec![6, 1, 7, 3, 0];
    let mut alea = Alea::new(21);
    let mut reseau = Reseau::au_hasard(&g, &mut alea, foods.clone(), 2.0, Loi::Sigmoide).unwrap();
    let mut jumeau = reseau.clone();
    let mut parallele = alea.clone();
    for pas in 0..40 {
        let (i, j) = paire_naive(&mut parallele, 5);
        let rendu = reseau.avancer(&g, &mut alea, 0.1).unwrap();
        let attendu = jumeau.avancer_paire(&g, foods[i], foods[j], 0.1).unwrap();
        assert_eq!((rendu.source, rendu.puits), (foods[i], foods[j]), "pas {pas}");
        assert_eq!(alea, parallele, "pas {pas}");
        assert_eq!(rendu, attendu, "pas {pas}");
        assert_eq!(reseau, jumeau, "pas {pas}");
    }
}
