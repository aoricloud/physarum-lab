//! Étape 1, partie 1L : `lancer` avec des murs et des étages (directives `mur` et `cout_etage`) : l'empreinte, les
//! mesures, les erreurs de directives, les scénarios du dépôt, le binaire. Valeurs attendues :
//! `references/etape-1l-valeurs.md`, identifiants `V-LLAB-xx`.
//!
//! L'empreinte est vérifiée d'abord **à la main** : l'écriture canonique de deux petits cas, écrite octet par octet
//! d'après la doc de `lancer` et de `graphe` (le graphe du **plan**, la source et le puits par leur indice **dans le
//! plan**), et son SHA-256 calculé par le modèle Python `ref_labyrinthe.py` (écrit d'après la seule doc, sans lien avec
//! le code Rust : mêmes opérations `f64` dans le même ordre, donc mêmes bits) ; puis sur d'autres scénarios dont l'état
//! a été simulé par ce modèle. Seul `regression_reference_labyrinthe_graine_1` compare à un fichier produit par notre
//! code.

mod common;

use std::fs;
use std::process::Command;

use physarum_lab::empreinte::sha256_hex;
use physarum_lab::lancer::{ErreurLancer, Sortie, lancer};
use physarum_lab::scenario::lire_scenario;

const RACINE: &str = env!("CARGO_MANIFEST_DIR");

/// Un cas en voisinage 8 à deux étages, deux coûts d'étage et deux murs sur deux étages.
const RICHE: &str = concat!(
    "grille 5 4",
    "\n",
    "voisinage 8",
    "\n",
    "etages 2 0.5 3.0",
    "\n",
    "cout_etage 0 2.0",
    "\n",
    "cout_etage 1 0.25",
    "\n",
    "mur 1 1 2 2 0",
    "\n",
    "mur 2 1 3 3 1",
    "\n",
    "source 0 0 0",
    "\n",
    "puits 4 3 1",
    "\n",
    "pas 12 0.4",
    "\n"
);

/// Un cas à poche fermée : l'anneau muré autour de `(2, 2)`, centre libre mais enfermé.
const POCHE: &str = concat!(
    "grille 5 5",
    "\n",
    "mur 1 1 3 1 0",
    "\n",
    "mur 1 3 3 3 0",
    "\n",
    "mur 1 2 1 2 0",
    "\n",
    "mur 3 2 3 2 0",
    "\n",
    "source 0 0 0",
    "\n",
    "puits 4 4 0",
    "\n",
    "pas 6 0.5",
    "\n"
);

fn lancer_texte(texte: &str, graine: u64) -> Sortie {
    let scenario = lire_scenario(texte).expect("scénario valide");
    lancer(&scenario, graine).unwrap_or_else(|e| panic!("lancer {texte:?} : {e:?}"))
}

fn erreur_de(texte: &str) -> Option<ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario lisible"), 1).err()
}

fn fichier(nom: &str) -> String {
    fs::read_to_string(format!("{RACINE}/scenarios/{nom}")).unwrap_or_else(|e| panic!("scenarios/{nom} : {e}"))
}

/// Un `f64` en huit octets petit-boutistes.
fn f(x: f64) -> [u8; 8] {
    x.to_bits().to_le_bytes()
}

/// L'écriture canonique d'un graphe (doc de `graphe`) : `n`, les nœuds `(x, y, étage)`, `m`, les arêtes `(a, b,
/// longueur, coût)`, écrite ici octet par octet.
fn graphe_octets(noeuds: &[(i32, i32, u32)], aretes: &[(u32, u32, f64, f64)]) -> Vec<u8> {
    let mut o = (noeuds.len() as u32).to_le_bytes().to_vec();
    for &(x, y, e) in noeuds {
        o.extend_from_slice(&x.to_le_bytes());
        o.extend_from_slice(&y.to_le_bytes());
        o.extend_from_slice(&e.to_le_bytes());
    }
    o.extend_from_slice(&(aretes.len() as u32).to_le_bytes());
    for &(a, b, longueur, cout) in aretes {
        o.extend_from_slice(&a.to_le_bytes());
        o.extend_from_slice(&b.to_le_bytes());
        o.extend_from_slice(&f(longueur));
        o.extend_from_slice(&f(cout));
    }
    o
}

/// L'état de `PHYSAR01` : l'étiquette, le graphe, la source et le puits (indices du plan), le nombre de pas, les bits
/// de `h`, les bits de chaque conductivité finale, l'état du générateur.
fn etat(graphe: Vec<u8>, source: u32, puits: u32, nombre: u32, h: f64, d_bits: &[u64], alea: [u8; 16]) -> Vec<u8> {
    let mut o = b"PHYSAR01".to_vec();
    o.extend_from_slice(&graphe);
    o.extend_from_slice(&source.to_le_bytes());
    o.extend_from_slice(&puits.to_le_bytes());
    o.extend_from_slice(&nombre.to_le_bytes());
    o.extend_from_slice(&f(h));
    for d in d_bits {
        o.extend_from_slice(&d.to_le_bytes());
    }
    o.extend_from_slice(&alea);
    o
}

/// L'état du générateur après `2 * m` tirages de 32 bits, `m` = 4 (graine 7) : `e9c712e9885a1389 6d00000000000000`, du
/// modèle Python (`Alea::octets` : l'état puis l'incrément, petit-boutistes).
const ALEA_7_APRES_4_ARETES: [u8; 16] = [0xE9, 0xC7, 0x12, 0xE9, 0x88, 0x5A, 0x13, 0x89, 0x6D, 0, 0, 0, 0, 0, 0, 0];

/// juste : l'empreinte d'un petit cas **muré**, écrite octet par octet (V-LLAB-01). `grille 3 2`, `mur 1 0 1 0 0` (la
/// cellule `(1, 0)`), `source 0 0 0`, `puits 2 0 0`, `pas 1 0.5`, graine 7. Le graphe du plan a cinq nœuds, les
/// cellules `(0,0)`, `(2,0)`, `(0,1)`, `(1,1)`, `(2,1)` renumérotées de 0 à 4, et quatre arêtes `(0,2) (1,4) (2,3)
/// (3,4)` (une chaîne `0-2-3-4-1`) ; **la source est le nœud 0 et le puits le nœud 1** (leurs indices dans le plan :
/// `(2, 0)` est l'indice 2 de la grille mais le nœud 1 du plan). L'état fait 240 octets ; les quatre conductivités
/// finales, du modèle Python, sont `3FED22A5CE1B6194`, `3FE8CC5E27EDBB33`, `3FEDC961D2C3EC75`, `3FEE8483AC761D88` ; le
/// SHA-256 (Python `hashlib`) est `8ce3c3a3…1d40`. Ni les indices de la grille (0 et 2), ni le graphe complet, ni des
/// arêtes dans un autre ordre ne donnent cette empreinte.
///
/// Ne couvre pas : le contenu de `mesures` et de `apercu` (voir `mesures_en_csv` et `etape_1l_apercus`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn empreinte_d_un_cas_mure_a_la_main() {
    let noeuds = [(0, 0, 0), (2, 0, 0), (0, 1, 0), (1, 1, 0), (2, 1, 0)];
    let aretes = [(0, 2, 1.0, 1.0), (1, 4, 1.0, 1.0), (2, 3, 1.0, 1.0), (3, 4, 1.0, 1.0)];
    let d = [0x3FED_22A5_CE1B_6194, 0x3FE8_CC5E_27ED_BB33, 0x3FED_C961_D2C3_EC75, 0x3FEE_8483_AC76_1D88];
    let octets = etat(graphe_octets(&noeuds, &aretes), 0, 1, 1, 0.5, &d, ALEA_7_APRES_4_ARETES);
    assert_eq!(octets.len(), 240);
    assert_eq!(sha256_hex(&octets), "8ce3c3a356328bdb7640a2542b7f1908977ec1fa52a5605d1e386ee8bb231d40");
    let texte = "grille 3 2\nmur 1 0 1 0 0\nsource 0 0 0\npuits 2 0 0\npas 1 0.5\n";
    assert_eq!(lancer_texte(texte, 7).empreinte, sha256_hex(&octets));
    // la source et le puits par leurs indices de grille (0 et 2) : une autre empreinte
    let faux = etat(graphe_octets(&noeuds, &aretes), 0, 2, 1, 0.5, &d, ALEA_7_APRES_4_ARETES);
    assert_ne!(sha256_hex(&faux), sha256_hex(&octets));
    // les directives dans un autre ordre, un commentaire, un mur donné deux fois : le même état
    let autre = "pas 1 0.5\n# un mur\nmur 1 0 1 0 0\nmur 1 0 1 0 0\npuits 2 0 0\ngrille 3 2\nsource 0 0 0\n";
    assert_eq!(lancer_texte(autre, 7).empreinte, sha256_hex(&octets));
    // sans le mur, c'est un autre graphe, donc une autre empreinte
    assert_ne!(lancer_texte("grille 3 2\nsource 0 0 0\npuits 2 0 0\npas 1 0.5\n", 7).empreinte, sha256_hex(&octets));
}

/// juste : l'empreinte d'un petit cas **à étages**, écrite octet par octet (V-LLAB-02). `grille 2 1`, `etages 2 1.0
/// 2.0` (un lien vertical de longueur 1 et de coût 2), `cout_etage 1 0.5`, `source 0 0 0`, `puits 1 0 0`, `pas 1 0.5`,
/// graine 7. Le graphe a quatre nœuds `(0,0,0) (0,0,1) (1,0,0) (1,0,1)` et quatre arêtes : `(0,1)` vertical de coût
/// `2.0`, `(0,2)` de coût `1.0`, `(1,3)` de coût **0,5** (l'étage 1 coûte moitié moins), `(2,3)` vertical de coût `2.0`
/// ; la longueur de chacune est `1.0`. La source est le nœud 0 et le puits le nœud 2. Les quatre conductivités finales,
/// du modèle Python : `3FE144D2B6E46A50`, `3FE4AA313F24B276`, `3FE1EB8EBB8CF530`, `3FE2A6B0953F2643` ; le SHA-256 est
/// `9f048e06…60a4`. Un facteur oublié, appliqué aux liens verticaux ou à la longueur, ou à l'autre étage, change
/// l'empreinte.
///
/// Ne couvre pas : un étage entièrement muré (voir `etape_1l_construction`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn empreinte_d_un_cas_a_etages_a_la_main() {
    let noeuds = [(0, 0, 0), (0, 0, 1), (1, 0, 0), (1, 0, 1)];
    let aretes = [(0, 1, 1.0, 2.0), (0, 2, 1.0, 1.0), (1, 3, 1.0, 0.5), (2, 3, 1.0, 2.0)];
    let d = [0x3FE1_44D2_B6E4_6A50, 0x3FE4_AA31_3F24_B276, 0x3FE1_EB8E_BB8C_F530, 0x3FE2_A6B0_953F_2643];
    let octets = etat(graphe_octets(&noeuds, &aretes), 0, 2, 1, 0.5, &d, ALEA_7_APRES_4_ARETES);
    assert_eq!(octets.len(), 228);
    assert_eq!(sha256_hex(&octets), "9f048e061c8c5f96ba7ac64a212b7cc8b03eaa3b3648332529013330832e60a4");
    let texte = "grille 2 1\netages 2 1.0 2.0\ncout_etage 1 0.5\nsource 0 0 0\npuits 1 0 0\npas 1 0.5\n";
    assert_eq!(lancer_texte(texte, 7).empreinte, sha256_hex(&octets));
    // sans `cout_etage` : un autre graphe
    let sans = "grille 2 1\netages 2 1.0 2.0\nsource 0 0 0\npuits 1 0 0\npas 1 0.5\n";
    assert_ne!(lancer_texte(sans, 7).empreinte, sha256_hex(&octets));
    // un facteur de 1.0 est un facteur absent
    let un = "grille 2 1\netages 2 1.0 2.0\ncout_etage 1 1.0\ncout_etage 0 1\nsource 0 0 0\npuits 1 0 0\npas 1 0.5\n";
    assert_eq!(lancer_texte(un, 7).empreinte, lancer_texte(sans, 7).empreinte);
}

/// juste : l'empreinte de scénarios à murs et à étages, simulée par le modèle Python `ref_labyrinthe.py` (V-LLAB-03) :
/// `scenarios/labyrinthe.txt` (342 nœuds, 343 arêtes, 500 pas) pour les graines 1 (`dcdad079…3ded`) et 2
/// (`f09a4947…3b09`) ; le même à 20 pas (`63835287…dbf`) et à 100 pas (`05c95556…7070`) ; `scenarios/etages.txt`
/// (`d011d6bc…df76`), `scenarios/etages-cher.txt` (`ed43638b…54a7`), `scenarios/etages-pont.txt` (`2e1ce39f…e03b`),
/// graine 1 ; un cas en voisinage 8 à deux étages avec deux coûts d'étage et deux murs sur deux étages
/// (`bd540817…7579`, graine 3) ; un cas à poche fermée (`3cebd9b8…be7a`, graine 5). L'ordre des directives, les
/// doublons de murs et un mur coupé en deux rectangles ne changent pas l'empreinte ; retirer un mur, changer un facteur
/// d'un millième ou la graine la change.
///
/// Ne couvre pas : Linux contre Windows (c'est la CI qui rejoue l'empreinte de référence).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn empreintes_de_scenarios_murs_et_etages() {
    let (riche, poche) = (RICHE, POCHE);
    let labyrinthe = fichier("labyrinthe.txt");
    let cas: [(String, u64, &str); 9] = [
        (labyrinthe.clone(), 1, "dcdad07950dc6dbb0aaa141bc1bb7fe7c0bdbe309a1f4d75b0cbf2dcba213ded"),
        (labyrinthe.clone(), 2, "f09a4947f075e05ce0661d9290f71f4f9649813b2bb41cc54f7543eec7fb3b09"),
        (
            labyrinthe.replace("pas 500 0.5", "pas 20 0.5"),
            1,
            "638352879a82dfc5073d05055c0ea8d1d96b6eb9763aa8718de453579c15edbf",
        ),
        (
            labyrinthe.replace("pas 500 0.5", "pas 100 0.5"),
            1,
            "05c95556f7e3175d9200b228f428658517a3ee072e119b23005b89cdd8e97070",
        ),
        (fichier("etages.txt"), 1, "d011d6bc3988a72730aefba5b8c90ac5f460cd8c65c0f3ece05564acd218df76"),
        (fichier("etages-cher.txt"), 1, "ed43638bbbf1b2fdf88cb9cecb81c76d239cc075546d8aaed87fc90bf79354a7"),
        (fichier("etages-pont.txt"), 1, "2e1ce39fa7c0d4240414a708bc9efd90d744e9dd2f1120fcbb5c9a8915eee03b"),
        (riche.to_string(), 3, "bd540817e2c7953142f326bfe8d50a045a20b23b2bb67e7d2a5fd3c9c72e7579"),
        (poche.to_string(), 5, "3cebd9b852a79a96a6c584a9e94969e64e1fecb5904eedd98d16a94f1c36be7a"),
    ];
    for (texte, graine, attendu) in &cas {
        assert_eq!(&lancer_texte(texte, *graine).empreinte, attendu, "graine {graine}");
    }
    // l'ordre des directives et des murs, les doublons, un mur coupé en deux : le même état
    let desordre = [
        "pas 12 0.4",
        "mur 2 1 3 3 1",
        "mur 1 1 2 2 0",
        "mur 2 1 3 3 1",
        "cout_etage 1 0.25",
        "source 0 0 0",
        "grille 5 4",
        "cout_etage 0 2.0",
        "voisinage 8",
        "mur 1 1 1 2 0",
        "mur 2 1 2 2 0",
        "puits 4 3 1",
        "etages 2 0.5 3.0",
    ]
    .join("\n")
        + "\n";
    assert_eq!(lancer_texte(&desordre, 3).empreinte, cas[7].2);
    // chaque changement change l'empreinte
    for (texte, graine) in [
        (riche.replace("mur 1 1 2 2 0\n", ""), 3),
        (riche.replace("cout_etage 1 0.25", "cout_etage 1 0.251"), 3),
        (riche.replace("cout_etage 0 2.0", "cout_etage 0 2.001"), 3),
        (riche.replace("mur 2 1 3 3 1", "mur 2 1 3 2 1"), 3),
        (riche.to_string(), 4),
    ] {
        assert_ne!(lancer_texte(&texte, graine).empreinte, cas[7].2, "{texte:?}");
    }
}

/// régression : pour `scenarios/labyrinthe.txt` et la graine 1, l'empreinte rendue par `lancer` est celle du fichier
/// `references/labyrinthe-graine-1.empreinte` (l'empreinte puis un saut de ligne), que la CI rejoue sous Linux et sous
/// Windows ; le fichier est produit par le binaire, et sa valeur est celle du modèle Python (V-LLAB-04).
///
/// Ne couvre pas : que l'empreinte soit juste (`empreintes_de_scenarios_murs_et_etages` la compare au modèle Python).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn regression_reference_labyrinthe_graine_1() {
    let reference = fs::read_to_string(format!("{RACINE}/references/labyrinthe-graine-1.empreinte")).unwrap();
    assert_eq!(reference.len(), 65);
    assert!(reference.ends_with('\n'));
    assert_eq!(lancer_texte(&fichier("labyrinthe.txt"), 1).empreinte, reference.trim_end());
}

/// juste : les mesures en CSV d'un scénario muré ou à étages (V-LLAB-05) : le même format qu'au cœur
/// (`temps,pression,vifs, longueur`, une ligne par pas), les nombres du modèle Python. Les mesures portent sur le
/// graphe du **plan** : le nombre de tubes vifs au départ (`343` au labyrinthe, `4` pour le cas à une cellule murée)
/// est celui des arêtes du plan, et `longueur` somme leurs longueurs. Texte entier pour les deux petits cas ; pour les
/// autres : le nombre de lignes, la première, la dernière et le SHA-256 du texte. Au labyrinthe, la dernière ligne
/// (`154` tubes vifs, de longueur `154`, une pression de `154,007069516`) est le plus court chemin, 154 cellules, et la
/// pression de la source tend vers sa longueur : Bonifaci et al., théorème 2. Pour `etages.txt` : 22 tubes vifs (20 de
/// l'étage du haut et 2 liens verticaux), pression `12,000001122` (`2 * 1 + 20 * 0,5`) ; `etages-cher.txt` : 20 tubes
/// vifs, pression `20,000002191` ; `etages-pont.txt` : 12 tubes vifs, pression `7,000000000` (à neuf décimales).
///
/// Ne couvre pas : le seuil des tubes vifs à sa limite (voir `etape_1_mesures`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn mesures_en_csv() {
    let entete = "temps,pression,vifs,longueur\n";
    assert_eq!(
        lancer_texte("grille 3 2\nmur 1 0 1 0 0\nsource 0 0 0\npuits 2 0 0\npas 1 0.5\n", 7).mesures,
        format!("{entete}0,5.299269501,4,4.000000\n")
    );
    assert_eq!(
        lancer_texte("grille 2 1\netages 2 1.0 2.0\ncout_etage 1 0.5\nsource 0 0 0\npuits 1 0 0\npas 1 0.5\n", 7)
            .mesures,
        format!("{entete}0,1.348724918,4,4.000000\n")
    );
    let labyrinthe = fichier("labyrinthe.txt");
    #[allow(clippy::type_complexity)]
    let cas: [(String, u64, usize, &str, &str, &str); 8] = [
        (
            labyrinthe.clone(),
            1,
            500,
            "0,111.018817480,343,343.000000",
            "499,154.007069516,154,154.000000",
            "77177bed1945a625fc5992f0545c1722f354f6c60e73b7f6cea8c7e77f12e988",
        ),
        (
            labyrinthe,
            2,
            500,
            "0,113.831500359,343,343.000000",
            "499,154.007655939,154,154.000000",
            "be7b6d510684af324e272ef4ebc0d9b84fd401e09ea395d28e51846739d2812b",
        ),
        (
            fichier("etages.txt"),
            1,
            300,
            "0,3.543391611,267,267.000000",
            "299,12.000001122,22,22.000000",
            "e6cf6f5cc527c48f1938d81dcbc8dd77d743a310b84ae89945ada3bf6c011234",
        ),
        (
            fichier("etages-cher.txt"),
            1,
            300,
            "0,4.447171613,267,267.000000",
            "299,20.000002191,20,20.000000",
            "98e424930232ccd32dc09de98d537d5df207eda25fc5ae0d3dcaf985c338f97f",
        ),
        (
            fichier("etages-pont.txt"),
            1,
            300,
            "0,1.613377441,316,316.000000",
            "299,7.000000000,12,12.000000",
            "0e0e04b861cce259cac657279d8c184e0b3d60d40fc8fa525c2b33ea950ba8ae",
        ),
        (
            RICHE.to_string(),
            3,
            12,
            "0,2.462837418,65,66.455844",
            "11,5.060064481,7,6.914214",
            "96557468d4f412fbde6016556d4c51f8f4c05ddff0056cbf278f87e22b4b67c9",
        ),
        (
            POCHE.to_string(),
            5,
            6,
            "0,4.791248075,16,16.000000",
            "5,7.831227805,16,16.000000",
            "4a74c381ba033c176419312ed93de546d9bbfed614221746a3a5748e68ce3d76",
        ),
        (
            fichier("labyrinthe.txt").replace("pas 500 0.5", "pas 20 0.5"),
            1,
            20,
            "0,111.018817480,343,343.000000",
            "19,156.493591297,154,154.000000",
            "4a1e79d4bcea67003525dcfd090fb22cb81689a4660d73ae708e4b6972eb058b",
        ),
    ];
    for (texte, graine, lignes, premiere, derniere, sha) in cas {
        let mesures = lancer_texte(&texte, graine).mesures;
        let corps: Vec<&str> = mesures.lines().collect();
        assert_eq!(corps[0], entete.trim_end());
        assert_eq!(corps.len(), lignes + 1, "une ligne par pas");
        assert_eq!(corps[1], premiere);
        assert_eq!(corps[lignes], derniere);
        assert!(mesures.ends_with('\n') && !mesures.contains('\r') && !mesures.ends_with("\n\n"));
        assert_eq!(sha256_hex(mesures.as_bytes()), sha);
    }
}

/// juste : les anciens scénarios gardent leur empreinte au bit (V-LLAB-06) : `scenarios/chemin.txt` graine 1
/// (`6e3b15b8…2f37`, celle du cœur) ; `scenarios/grille.txt` graine 1 (celle de `references/grille-graine-1.empreinte`,
/// l'étape 0) ; deux scénarios du cœur de `etape_1_lancer` (`grille 4 4`, graine 9, `ee991bd8…bed0` ; `grille 3 3` à
/// deux étages en voisinage 8, graine 3, `77acbb3c…7362`) ; et un scénario d'étage **sans** `cout_etage` : `1.0`
/// partout, les mêmes bits. La règle des scénarios du dépôt : le coût vertical de chaque fichier de `scenarios/` est
/// dans `[0,1 ; 10]` (au-delà d'un rapport de `10^10` le solveur se trompe sans le dire, `MODELE.md`) ; les six
/// scénarios connus à cette étape sont là, et `lancer` les accepte.
///
/// Ne couvre pas : `lancer` sur les scénarios que d'autres étapes ajouteront (leur coût vertical, si : la règle vaut
/// pour tout le dossier, et une étape suivante peut ajouter un fichier sans changer ce test).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn les_anciens_scenarios_gardent_leur_empreinte() {
    assert_eq!(
        lancer_texte(&fichier("chemin.txt"), 1).empreinte,
        "6e3b15b88963ed142ce3441c33b249b2164d13a51087662b8927f77ddd3c2f37"
    );
    let grille = fs::read_to_string(format!("{RACINE}/references/grille-graine-1.empreinte")).unwrap();
    assert_eq!(lancer_texte(&fichier("grille.txt"), 1).empreinte, grille.trim_end());
    assert_eq!(
        lancer_texte("grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.5\n", 9).empreinte,
        "ee991bd8ca038070aee07a55ce6bdb9fb2e58772d6f128fd66293f4ab854bed0"
    );
    assert_eq!(
        lancer_texte("grille 3 3\nvoisinage 8\netages 2 0.5 2.0\nsource 0 0 0\npuits 2 2 1\npas 20 0.25\n", 3)
            .empreinte,
        "77acbb3c46fd00f2722356e25036c2df616a16af6c108c058ff5c95aedd27362"
    );
    let mut noms: Vec<String> = fs::read_dir(format!("{RACINE}/scenarios"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    noms.sort();
    let connus = ["chemin.txt", "etages-cher.txt", "etages-pont.txt", "etages.txt", "grille.txt", "labyrinthe.txt"];
    for nom in connus {
        assert!(noms.iter().any(|n| n == nom), "{nom} manque dans scenarios/");
        assert!(lancer(&lire_scenario(&fichier(nom)).unwrap(), 1).is_ok(), "{nom}");
    }
    for nom in noms {
        let scenario = lire_scenario(&fichier(&nom)).unwrap();
        let cout = scenario.grille.cout_vertical;
        assert!((0.1..=10.0).contains(&cout), "{nom} : coût vertical {cout}");
    }
}

/// juste : les erreurs des directives `mur` et `cout_etage` (V-LLAB-07) sont `DirectiveInconnue(ligne)` avec la règle
/// de priorité de la doc de `lancer` : (1) la première directive FAUTIVE dans l'ordre du fichier (un `mur` à un mauvais
/// nombre d'arguments, à un argument mal écrit (`-1`, `1.5`, `+1`), à `x0 > x1` ou `y0 > y1`, à `x1 = largeur`, `y1 =
/// hauteur`, `etage = etages` ; un `cout_etage` à un étage hors de la grille, à un facteur `0.09`, `10.01`, `1e1`,
/// `.5`, ou à un étage déjà donné : le second est fautif) ; (2) la source et le puits au même nœud : celle des deux qui
/// vient en second ; (3) s'il manque une des trois directives des tubes, la première directive restante (un `mur` sans
/// les trois est une erreur) ; (4) le puits sous un mur : la ligne de `puits` ; la source sous un mur ou séparée du
/// puits : la ligne de `source`, le puits d'abord quand les deux sont sous un mur. Les bornes sont comprises :
/// `cout_etage 1 0.1`, `cout_etage 1 10`, `cout_etage 1 10.0`, `mur` au dernier numéro de colonne, de ligne et d'étage
/// passent.
///
/// Ne couvre pas : les erreurs de lecture (`etape_0_scenario`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn erreurs_de_directives() {
    let e = |ligne: usize| Some(ErreurLancer::DirectiveInconnue(ligne));
    // une grille 6 × 4 à deux étages ; la ligne 1 est `grille`, la ligne 2 `etages`
    let entete = "grille 6 4\netages 2 1.0 1.0\n";
    let fin = "source 0 0 0\npuits 5 3 0\npas 2 0.5\n";
    let avec = |milieu: &str| format!("{entete}{milieu}{fin}");
    assert_eq!(erreur_de(&avec("")), None);
    assert_eq!(erreur_de(&avec("mur 1 1 2 2 0\ncout_etage 1 0.5\n")), None);
    // des bornes comprises
    for ok in [
        "mur 5 3 5 3 1\n",
        "mur 0 0 5 3 1\n",
        "mur 0 0 0 0 1\n",
        "cout_etage 1 0.1\n",
        "cout_etage 1 10\n",
        "cout_etage 0 10.0\n",
        "cout_etage 0 0.1\ncout_etage 1 10\n",
        "mur 2 2 2 2 1\nmur 2 2 2 2 1\n",
    ] {
        assert_eq!(erreur_de(&avec(ok)), None, "{ok:?}");
    }
    // chaque faute de `mur`, seule en ligne 3 (`entete` en occupe deux)
    for mauvais in [
        "mur 1 1 2 2\n",
        "mur 1 1 2 2 0 0\n",
        "mur\n",
        "mur 1 1 2 2 -1\n",
        "mur 1.5 1 2 2 0\n",
        "mur +1 1 2 2 0\n",
        "mur 3 1 2 2 0\n",
        "mur 1 3 2 2 0\n",
        "mur 1 1 6 2 0\n",
        "mur 1 1 2 4 0\n",
        "mur 1 1 2 2 2\n",
        "mur a 1 2 2 0\n",
        "mur 1 1 2 2 99999999999999999999\n",
        "mur 0 0 0 0 4294967296\n",
    ] {
        assert_eq!(erreur_de(&avec(mauvais)), e(3), "{mauvais:?}");
    }
    // chaque faute de `cout_etage`
    for mauvais in [
        "cout_etage 2 0.5\n",
        "cout_etage 1 0.09\n",
        "cout_etage 1 10.01\n",
        "cout_etage 1 1e1\n",
        "cout_etage 1 .5\n",
        "cout_etage 1 0\n",
        "cout_etage 1 -1\n",
        "cout_etage 1\n",
        "cout_etage\n",
        "cout_etage 1 0.5 0.5\n",
        "cout_etage -1 0.5\n",
        "cout_etage 1 nan\n",
        "cout_etage 1 inf\n",
        "cout_etage 1 0.0999999\n",
        "cout_etage 1.0 0.5\n",
    ] {
        assert_eq!(erreur_de(&avec(mauvais)), e(3), "{mauvais:?}");
    }
    // un étage donné deux fois : le second ; deux étages différents : bon
    assert_eq!(erreur_de(&avec("cout_etage 1 0.5\ncout_etage 1 0.5\n")), e(4));
    assert_eq!(erreur_de(&avec("cout_etage 1 0.5\nmur 1 1 1 1 0\ncout_etage 1 2\n")), e(5));
    assert_eq!(erreur_de(&avec("cout_etage 1 0.5\ncout_etage 0 0.5\n")), None);
    // la première fautive dans l'ordre du fichier, tous genres confondus
    assert_eq!(erreur_de(&avec("mur 9 1 2 2 0\ncout_etage 1 0.01\ninconnue\n")), e(3));
    assert_eq!(erreur_de(&avec("cout_etage 1 0.01\nmur 9 1 2 2 0\n")), e(3));
    assert_eq!(
        erreur_de(&format!("{entete}source 0 0 0\npuits 5 3 0\nmur 9 1 2 2 0\npas 2 0.5\ncout_etage 9 1\n")),
        e(5)
    );
    assert_eq!(
        erreur_de(&format!("{entete}mur 1 1 2 2 0\nsource 0 0\npuits 5 3 0\npas 2 0.5\n")),
        e(4),
        "une source mal écrite"
    );
    // (1) l'emporte sur (4) : un puits muré avant un mauvais `pas`
    assert_eq!(erreur_de(&format!("{entete}mur 5 3 5 3 0\nsource 0 0 0\npuits 5 3 0\npas 2 2\n")), e(6));
    // (2) la source et le puits au même nœud : la seconde des deux ; (2) l'emporte sur (4)
    assert_eq!(erreur_de(&format!("{entete}source 0 0 0\nmur 0 0 0 0 0\npuits 0 0 0\npas 2 0.5\n")), e(5));
    assert_eq!(erreur_de(&format!("{entete}puits 0 0 0\nsource 0 0 0\nmur 0 0 0 0 0\npas 2 0.5\n")), e(4));
    // (3) l'emporte sur (4) : il manque `pas`, le puits est muré : la première directive restante
    assert_eq!(erreur_de(&format!("{entete}mur 5 3 5 3 0\nsource 0 0 0\npuits 5 3 0\n")), e(3));
    assert_eq!(erreur_de(&format!("{entete}source 0 0 0\nmur 5 3 5 3 0\npuits 5 3 0\n")), e(3));
    // des murs ou des coûts sans les trois directives des tubes : la première directive restante
    assert_eq!(erreur_de("grille 4 4\nmur 0 0 0 0 0\n"), e(2));
    assert_eq!(erreur_de("grille 4 4\n# c\ncout_etage 0 1\n"), e(3));
    assert_eq!(erreur_de("grille 4 4\nsource 0 0 0\npuits 3 3 0\nmur 1 1 1 1 0\n"), e(2));
    // (4) : le puits muré, la source murée, la source séparée du puits
    assert_eq!(erreur_de(&avec("mur 5 3 5 3 0\n")), e(5));
    assert_eq!(erreur_de(&avec("mur 0 0 0 0 0\n")), e(4));
    assert_eq!(erreur_de(&avec("mur 0 0 0 0 0\nmur 5 3 5 3 0\n")), e(6), "le puits d'abord : sa ligne");
    assert_eq!(erreur_de(&avec("mur 5 3 5 3 1\n")), None, "le mur est à l'autre étage");
    // la source est séparée du puits : un mur sur tout une colonne des deux étages
    assert_eq!(erreur_de(&avec("mur 2 0 2 3 0\nmur 2 0 2 3 1\n")), e(5));
    assert_eq!(erreur_de(&avec("mur 2 0 2 3 0\n")), None, "le passage par l'étage 1");
    // une poche : la source libre mais enfermée
    assert_eq!(erreur_de("grille 5 5\nmur 0 1 1 1 0\nmur 1 0 1 0 0\nsource 0 0 0\npuits 4 4 0\npas 2 0.5\n"), e(4));
    // un coût d'étage hors de la grille d'un seul étage
    assert_eq!(erreur_de("grille 4 4\ncout_etage 1 0.5\nsource 0 0 0\npuits 3 3 0\npas 2 0.5\n"), e(2));
    assert_eq!(erreur_de("grille 4 4\ncout_etage 0 0.5\nsource 0 0 0\npuits 3 3 0\npas 2 0.5\n"), None);
}

/// juste : le binaire avec des murs (V-LLAB-08) : `physarum lancer scenarios/labyrinthe.txt --graine 1 --sortie
/// <dossier>` rend le code 0 et écrit `empreinte.txt` (`dcdad079…3ded` puis `\n`), `mesures.csv` (500 lignes de pas, le
/// SHA-256 du modèle Python) et `apercu.png` (511 × 371 pixels, échelle 7) ; un scénario dont la source est sous un mur
/// rend le code 1, un message sans `panicked` qui contient `ligne 4` (la ligne de `source`), et n'écrit rien (le
/// dossier de sortie n'est pas créé).
///
/// Ne couvre pas : les erreurs d'arguments (voir `etape_0_binaire`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn le_binaire_avec_des_murs() {
    let dossier = common::dossier_temporaire("1l_binaire");
    fs::create_dir_all(&dossier).unwrap();
    let sortie = dossier.join("sortie");
    let scenario = format!("{RACINE}/scenarios/labyrinthe.txt");
    let resultat = Command::new(env!("CARGO_BIN_EXE_physarum"))
        .args(["lancer", &scenario, "--graine", "1", "--sortie", sortie.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    assert_eq!(
        fs::read_to_string(sortie.join("empreinte.txt")).unwrap(),
        "dcdad07950dc6dbb0aaa141bc1bb7fe7c0bdbe309a1f4d75b0cbf2dcba213ded\n"
    );
    let mesures = fs::read_to_string(sortie.join("mesures.csv")).unwrap();
    assert_eq!(mesures.lines().count(), 501);
    assert_eq!(sha256_hex(mesures.as_bytes()), "77177bed1945a625fc5992f0545c1722f354f6c60e73b7f6cea8c7e77f12e988");
    let lu = common::relire_png(&fs::read(sortie.join("apercu.png")).unwrap()).expect("PNG valide");
    assert_eq!((lu.largeur, lu.hauteur), (511, 371));
    // une source sous un mur
    let faux = dossier.join("faux.txt");
    fs::write(&faux, "grille 4 4\nmur 0 0 0 0 0\nsource 0 0 0\npuits 3 3 0\npas 2 0.5\n").unwrap();
    let absent = dossier.join("absent").join("sortie");
    let resultat = Command::new(env!("CARGO_BIN_EXE_physarum"))
        .args(["lancer", faux.to_str().unwrap(), "--graine", "1", "--sortie", absent.to_str().unwrap()])
        .output()
        .unwrap();
    let erreur = String::from_utf8_lossy(&resultat.stderr).into_owned();
    assert_eq!(resultat.status.code(), Some(1), "stderr : {erreur}");
    assert!(erreur.contains("ligne 3") && !erreur.contains("panicked"), "{erreur}");
    assert!(!dossier.join("absent").exists(), "rien n'est créé");
    let _ = fs::remove_dir_all(&dossier);
}
