//! Étape 2, partie 2M : `lancer` avec un agent (directives `agent` et `stimulus`) : l'empreinte (`PHYSAR03`), les
//! mesures, l'aperçu, les erreurs de directives, le calcul qui s'arrête, le binaire, et les anciens scénarios qui ne
//! bougent pas. Valeurs attendues : `references/etape-2m-valeurs.md`, identifiants `V-LAN-xx`.
//!
//! L'empreinte est vérifiée d'abord **à la main** : l'écriture canonique de deux petits cas écrite octet par octet
//! d'après la doc de `lancer` et de `graphe`, son SHA-256 calculé par le modèle Python `ref_memoire.py` (écrit d'après
//! la seule doc, sans lien avec le code Rust : mêmes opérations `f64` dans le même ordre, donc mêmes bits) ; puis sur
//! `scenarios/souvenir.txt` et deux autres scénarios simulés par ce modèle. Seul `regression_reference_souvenir`
//! compare à un fichier produit par notre code.

mod common;

use std::fs;
use std::process::Command;

use physarum_lab::empreinte::sha256_hex;
use physarum_lab::lancer::{ErreurLancer, Sortie, lancer};
use physarum_lab::scenario::lire_scenario;

const RACINE: &str = env!("CARGO_MANIFEST_DIR");

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

/// L'écriture canonique du graphe d'une grille de `largeur` colonnes sur une ligne, à un étage, voisinage 4, arêtes de
/// longueur et de coût 1 (doc de `graphe`), écrite ici octet par octet.
fn graphe_ligne(largeur: u32) -> Vec<u8> {
    let mut o = largeur.to_le_bytes().to_vec();
    for x in 0..largeur as i32 {
        o.extend_from_slice(&x.to_le_bytes());
        o.extend_from_slice(&0_i32.to_le_bytes());
        o.extend_from_slice(&0_u32.to_le_bytes());
    }
    o.extend_from_slice(&(largeur - 1).to_le_bytes());
    for a in 0..largeur - 1 {
        o.extend_from_slice(&a.to_le_bytes());
        o.extend_from_slice(&(a + 1).to_le_bytes());
        o.extend_from_slice(&f(1.0));
        o.extend_from_slice(&f(1.0));
    }
    o
}

/// Un stimulus d'un état : le nœud du plan, le débit, le début et la fin.
type Stim = (u32, f64, u32, u32);

/// L'état de `PHYSAR03`, écrit champ par champ comme dans la doc de `lancer` : l'étiquette, le graphe, la famille, `a`
/// et les nœuds d'alimentation, les bits du débit, l'octet de la loi, le nombre de pas, les bits de `h`, ceux de `α`,
/// `c_0`, `κ`, `k`, le nombre de stimuli et chaque stimulus, les bits de chaque conductivité finale puis de chaque
/// concentration finale, l'état du générateur.
#[allow(clippy::too_many_arguments)]
fn etat(
    graphe: Vec<u8>,
    famille: u8,
    alim: &[u32],
    debit: f64,
    loi: u8,
    nombre: u32,
    h: f64,
    reglage: [f64; 4],
    stimuli: &[Stim],
    d_bits: &[u64],
    c_bits: &[u64],
    alea: [u8; 16],
) -> Vec<u8> {
    let mut o = b"PHYSAR03".to_vec();
    o.extend_from_slice(&graphe);
    o.push(famille);
    o.extend_from_slice(&(alim.len() as u32).to_le_bytes());
    for v in alim {
        o.extend_from_slice(&v.to_le_bytes());
    }
    o.extend_from_slice(&f(debit));
    o.push(loi);
    o.extend_from_slice(&nombre.to_le_bytes());
    o.extend_from_slice(&f(h));
    for r in reglage {
        o.extend_from_slice(&f(r));
    }
    o.extend_from_slice(&(stimuli.len() as u32).to_le_bytes());
    for &(noeud, debit_s, debut, fin) in stimuli {
        o.extend_from_slice(&noeud.to_le_bytes());
        o.extend_from_slice(&f(debit_s));
        o.extend_from_slice(&debut.to_le_bytes());
        o.extend_from_slice(&fin.to_le_bytes());
    }
    for d in d_bits {
        o.extend_from_slice(&d.to_le_bytes());
    }
    for c in c_bits {
        o.extend_from_slice(&c.to_le_bytes());
    }
    o.extend_from_slice(&alea);
    o
}

fn hex_vers_octets(hex: &str) -> [u8; 16] {
    let mut o = [0_u8; 16];
    for (i, octet) in o.iter_mut().enumerate() {
        *octet = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap();
    }
    o
}

const PETIT_A: &str =
    "grille 3 1\nsource 0 0 0\npuits 2 0 0\npas 2 0.25\nagent 2.0 1.0 0.1 0.5\nstimulus 1 0 0 1.5 0 2\n";
const PETIT_B: &str = "grille 3 1\nnourriture 2 0 0\nnourriture 0 0 0\nnourriture 1 0 0\ndebit 2.0\nloi sigmoide\n\
pas 2 0.25\nagent 4.0 0.5 0.0 0.0\nstimulus 1 0 0 2.0 0 1\nstimulus 1 0 0 1.0 1 2\n";

/// Un cas de tubes à un mur et deux stimuli (l'un finit quand l'autre commence à moitié), un cas de réseau à deux
/// étages.
const TUBES_MUR: &str = "grille 9 7\nvoisinage 4\nmur 4 0 4 4 0\nsource 0 3 0\npuits 8 3 0\nagent 3.0 0.5 0.05 0.1\n\
stimulus 2 1 0 1.0 0 20\nstimulus 6 5 0 2.0 10 30\npas 40 0.2\n";
const RESEAU_ETAGES: &str = "grille 6 4\nvoisinage 8\netages 2 0.5 2.0\ncout_etage 1 0.5\nmur 2 1 3 2 1\n\
nourriture 0 0 0\nnourriture 5 3 1\nnourriture 5 0 0\ndebit 2\nloi sigmoide\nagent 6.0 0.2 0.3 0.05\n\
stimulus 4 2 1 3.0 0 15\npas 30 0.15\n";

/// juste : l'empreinte d'un petit cas de **tubes**, écrite octet par octet (V-LAN-01). `grille 3 1`, `source 0 0 0`,
/// `puits 2 0 0`, `pas 2 0.25`, `agent 2.0 1.0 0.1 0.5`, `stimulus 1 0 0 1.5 0 2`, graine 7. Trois nœuds, deux arêtes ;
/// la famille est `0`, `a = 2` (nœuds 0 puis 2), débit `1.0`, loi `0` ; un stimulus au nœud 1 ; les deux conductivités
/// finales, du modèle Python, sont `3FEF4DB7298DF632` et `3FEA6CA68E9A9B05` ; les trois concentrations
/// `3F83333333333334`, `3FE2E66666666666`, `3FBA666666666666` ; l'état du générateur après quatre tirages est
/// `fd6eabdbbde5d281 6d00000000000000` ; l'état fait `92 + 154 = 246` octets ; le SHA-256 (Python `hashlib`) est
/// `161d68ce…b590`. Les directives dans un autre ordre, avec des commentaires, donnent le même état ; chaque nombre de
/// la mémoire, un stimulus de plus, la graine, la famille changent l'empreinte.
///
/// Ne couvre pas : le contenu de `mesures` et de `apercu` (voir `mesures_en_csv` et `l_apercu_avec_agent`).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn empreinte_d_un_petit_cas_de_tubes_a_la_main() {
    let d = [0x3FEF_4DB7_298D_F632, 0x3FEA_6CA6_8E9A_9B05];
    let c = [0x3F83_3333_3333_3334, 0x3FE2_E666_6666_6666, 0x3FBA_6666_6666_6666];
    let alea = hex_vers_octets("fd6eabdbbde5d2816d00000000000000");
    let octets =
        etat(graphe_ligne(3), 0, &[0, 2], 1.0, 0, 2, 0.25, [2.0, 1.0, 0.1, 0.5], &[(1, 1.5, 0, 2)], &d, &c, alea);
    assert_eq!(octets.len(), 246);
    assert_eq!(sha256_hex(&octets), "161d68ce63d7356ee6a0a7290349dd44b5c805754d5c9822f464106985ecb590");
    assert_eq!(lancer_texte(PETIT_A, 7).empreinte, sha256_hex(&octets));
    let autre = "stimulus 1 0 0 1.5 0 2\n# un commentaire\nagent 2.0 1.0 0.1 0.5\npas 2 0.25\npuits 2 0 0\n\
    grille 3 1\nsource 0 0 0\n";
    assert_eq!(lancer_texte(autre, 7).empreinte, sha256_hex(&octets));
    for (texte, graine) in [
        (PETIT_A.replace("agent 2.0", "agent 2.5"), 7),
        (PETIT_A.replace("1.0 0.1 0.5", "1.5 0.1 0.5"), 7),
        (PETIT_A.replace("0.1 0.5", "0.2 0.5"), 7),
        (PETIT_A.replace("0.1 0.5", "0.1 0.4"), 7),
        (PETIT_A.replace("1.5 0 2", "1.25 0 2"), 7),
        (PETIT_A.replace("1.5 0 2", "1.5 1 2"), 7),
        (PETIT_A.replace("1.5 0 2", "1.5 0 3"), 7),
        (PETIT_A.replace("stimulus 1 0 0", "stimulus 0 0 0"), 7),
        (format!("{PETIT_A}stimulus 2 0 0 1.0 1 2\n"), 7),
        (PETIT_A.replace("pas 2 0.25", "pas 2 0.5"), 7),
        (PETIT_A.to_string(), 8),
    ] {
        assert_ne!(lancer_texte(&texte, graine).empreinte, sha256_hex(&octets), "{texte:?}");
    }
    // l'étiquette PHYSAR02 ou la famille donneraient une autre empreinte
    let mut faux = octets.clone();
    faux[7] = b'2';
    assert_ne!(sha256_hex(&faux), sha256_hex(&octets));
}

/// juste : l'empreinte d'un petit cas de **réseau**, écrite octet par octet (V-LAN-02). `grille 3 1`, nourritures
/// `(2,0)`, `(0,0)`, `(1,0)` (les nœuds par **rang** : 2, 0, 1), `debit 2.0`, `loi sigmoide`, `pas 2 0.25`, `agent 4.0
/// 0.5 0.0 0.0` (`κ = 0` et `k = 0`), **deux stimuli au même nœud** (`1.0` puis `2.0` sur des pas différents), graine
/// 3. La famille est `1`, `a = 3`, la loi `1` ; les deux conductivités finales sont `3FF04BD84A2A441E` et
///    `3FEDBE5E7A4817F1` ; les concentrations `3FCFFFFFFFFFFFFF`, `3FDFFFFFFFFFFFFF` et `0` ; l'état du générateur (le
///    même que celui de `etape_1r_lancer` pour la graine 3 : six tirages en tout) `b550126546a0931b 6d00000000000000` ;
///    l'état fait `92 + 178 = 270` octets ; le SHA-256 est `2c351313…0771`.
///
/// Ne couvre pas : les murs et les étages (`empreintes_de_scenarios_avec_agent`).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn empreinte_d_un_petit_cas_de_reseau_a_la_main() {
    let d = [0x3FF0_4BD8_4A2A_441E, 0x3FED_BE5E_7A48_17F1];
    let c = [0x3FCF_FFFF_FFFF_FFFF, 0x3FDF_FFFF_FFFF_FFFF, 0];
    let alea = hex_vers_octets("b550126546a0931b6d00000000000000");
    let octets = etat(
        graphe_ligne(3),
        1,
        &[2, 0, 1],
        2.0,
        1,
        2,
        0.25,
        [4.0, 0.5, 0.0, 0.0],
        &[(1, 2.0, 0, 1), (1, 1.0, 1, 2)],
        &d,
        &c,
        alea,
    );
    assert_eq!(octets.len(), 270);
    assert_eq!(sha256_hex(&octets), "2c3513138cc80f617ce5917d9a63c1e311a046d946c474d202714db5cf3b0771");
    assert_eq!(lancer_texte(PETIT_B, 3).empreinte, sha256_hex(&octets));
    // les deux stimuli dans l'autre ordre du fichier : l'état écrit les stimuli dans l'ordre du fichier
    let inverse = PETIT_B.replace(
        "stimulus 1 0 0 2.0 0 1\nstimulus 1 0 0 1.0 1 2\n",
        "stimulus 1 0 0 1.0 1 2\nstimulus 1 0 0 2.0 0 1\n",
    );
    assert_ne!(lancer_texte(&inverse, 3).empreinte, sha256_hex(&octets), "l'ordre des stimuli est écrit");
    let ordre = etat(
        graphe_ligne(3),
        1,
        &[2, 0, 1],
        2.0,
        1,
        2,
        0.25,
        [4.0, 0.5, 0.0, 0.0],
        &[(1, 1.0, 1, 2), (1, 2.0, 0, 1)],
        &d,
        &c,
        alea,
    );
    assert_eq!(lancer_texte(&inverse, 3).empreinte, sha256_hex(&ordre));
}

/// juste : le texte CSV des mesures de deux petits cas, écrit en entier (V-LAN-03) : l'en-tête de tubes
/// `temps,pression,vifs,longueur,masse` et celui de réseau `temps,source,puits,pression,vifs,longueur,masse` ;
/// `PETIT_A` : `0,3.036618473,2,2.000000,0.000000000` puis `1,2.664723275,2,2.000000,0.375000000` (la masse de la ligne
/// 1 est celle d'**avant** le pas 1, soit `h × débit × (1 - ...)` : `0.25 × 1.5 = 0.375`) ; `PETIT_B` :
/// `0,2,0,3.596297331, 2,2.000000,0.000000000` puis `1,0,1,5.959117801,2,2.000000,0.500000000` (`0.25 × 2.0`). Pour
/// `scenarios/souvenir.txt` (graine 1, 401 lignes, SHA-256 du modèle Python `3421b02d…43e6`) : les lignes 0, 1, 99,
/// 100, 101 et 399 écrites ici, `masse` égale à `0.2` à la ligne 1 (`0.1 × 2`). Les colonnes **sans la masse** d'un
/// scénario avec `agent 0.0` (force nulle) sont celles du même scénario sans `agent` ni `stimulus` : l'agent ne change
/// rien aux tubes quand `α = 0`.
///
/// Ne couvre pas : la précision (`{:.9}`) des cas qui arrondissent à la moitié (les valeurs viennent de Python, qui
/// arrondit comme Rust).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn mesures_en_csv() {
    assert_eq!(
        lancer_texte(PETIT_A, 7).mesures,
        "temps,pression,vifs,longueur,masse\n0,3.036618473,2,2.000000,0.000000000\n\
        1,2.664723275,2,2.000000,0.375000000\n"
    );
    assert_eq!(
        lancer_texte(PETIT_B, 3).mesures,
        "temps,source,puits,pression,vifs,longueur,masse\n0,2,0,3.596297331,2,2.000000,0.000000000\n\
1,0,1,5.959117801,2,2.000000,0.500000000\n"
    );
    let s = lancer_texte(&fichier("souvenir.txt"), 1);
    let lignes: Vec<&str> = s.mesures.lines().collect();
    assert_eq!(lignes.len(), 401);
    assert_eq!(lignes[0], "temps,source,puits,pression,vifs,longueur,masse");
    assert_eq!(lignes[1], "0,0,1,3.394416547,584,699.979797,0.000000000");
    assert_eq!(lignes[2], "1,1,0,3.738359202,584,699.979797,0.200000000");
    assert_eq!(lignes[100], "99,1,0,21.609916922,43,48.384776,8.573379401");
    assert_eq!(lignes[101], "100,1,0,21.556991441,42,47.384776,8.599813373");
    assert_eq!(lignes[102], "101,0,1,21.540594739,41,46.384776,8.424469248");
    assert_eq!(lignes[400], "399,1,0,41.438667692,17,20.313708,0.015223241");
    assert_eq!(sha256_hex(s.mesures.as_bytes()), "3421b02ddf232be39ebce59a31d426024b6d6295ed71caa4ba92d28f2ec543e6");
    // le scénario de tubes à un mur, ses lignes d'angle, et la force nulle
    let t = lancer_texte(TUBES_MUR, 5);
    let l: Vec<&str> = t.mesures.lines().collect();
    assert_eq!(l.len(), 41);
    assert_eq!(l[1], "0,5.092243860,95,95.000000,0.000000000");
    assert_eq!(l[2], "1,5.729656493,76,76.000000,0.200000000");
    assert_eq!(l[11], "10,10.794438292,2,2.000000,1.829271931");
    assert_eq!(l[40], "39,8.906714867,14,14.000000,7.403880595");
    assert_eq!(sha256_hex(t.mesures.as_bytes()), "008bb8adae6ee6c59e04e1d5c6f38be5d64f384116e5e81c3968e9bf2e8240a7");
    let nul = TUBES_MUR.replace("agent 3.0", "agent 0.0");
    let sans = "grille 9 7\nvoisinage 4\nmur 4 0 4 4 0\nsource 0 3 0\npuits 8 3 0\npas 40 0.2\n";
    let (a, b) = (lancer_texte(&nul, 5).mesures, lancer_texte(sans, 5).mesures);
    assert_eq!(a.lines().count(), b.lines().count());
    for (la, lb) in a.lines().zip(b.lines()).skip(1) {
        let (colonnes_a, colonnes_b): (Vec<&str>, Vec<&str>) = (la.split(',').collect(), lb.split(',').collect());
        assert_eq!(colonnes_a[..4], colonnes_b[..4], "{la} contre {lb}");
        assert_eq!(colonnes_a.len(), 5);
    }
}

/// juste : les empreintes de **scénarios** avec agent simulés par le modèle Python `ref_memoire.py` (V-LAN-04) :
/// `scenarios/souvenir.txt` (grille 15 × 11 à huit voisins, deux nourritures, `debit 3`, sigmoïde, `agent 8.0 1.0 0.1
/// 0.2`, un stimulus de débit 2 en `(7, 2)` sur `[0 ; 100[`, 400 pas à `h = 0,1`) pour les graines 1 (`ff1fa667…7c5`)
/// et 2 (`c46c8bf2…6652`) ; un cas de **tubes** à un mur et deux stimuli, graine 5 (`8ee19be5…491b`) ; un cas de
/// **réseau** à deux étages avec un mur sur l'étage 1 et un stimulus sur l'étage 1, graine 4 (`988d699f…1bf9`) ; le
/// même cas de tubes **sans stimulus** (l'agent reste à zéro, `c8e3662c…7e70`) et **à force nulle** (`e1ad6548…cbb63`).
/// Deux graines donnent deux états ; chaque directive de la mémoire change l'empreinte ; `-0` (un zéro signé) change
/// l'empreinte du cas de tubes à force nulle (le signe est dans les bits).
///
/// Ne couvre pas : Linux contre Windows (c'est la CI qui rejoue l'empreinte de référence).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn empreintes_de_scenarios_avec_agent() {
    let souvenir = fichier("souvenir.txt");
    let cas: [(String, u64, &str); 7] = [
        (souvenir.clone(), 1, "ff1fa667fb9a3a6025a36f1cd2e3176e7a3f81e7f7d093e4410208584b7cf7c5"),
        (souvenir, 2, "c46c8bf24bddbcd0c2b5ee7623a9a70ffb9005cc1da1f745417455fd34b96652"),
        (TUBES_MUR.to_string(), 5, "8ee19be5a6fab76832653a0906cba1c7f0d091f46bf7cb292b96dd082a95491b"),
        (RESEAU_ETAGES.to_string(), 4, "988d699ff9afcc36518f4864dfa1363f82697f35fa78c547da2eeb8bd8031bf9"),
        (
            TUBES_MUR.replace("stimulus 2 1 0 1.0 0 20\nstimulus 6 5 0 2.0 10 30\n", ""),
            5,
            "c8e3662c79c59029ef5f4ac6112fc7ee8e387b117024cd2d8021026a179a7e70",
        ),
        (
            TUBES_MUR.replace("agent 3.0", "agent 0.0").replace("stimulus 6 5 0 2.0 10 30\n", ""),
            5,
            "e1ad654865884a3345e0c534909015221f2dd257368575ef721ffb9a388cbb63",
        ),
        (TUBES_MUR.to_string(), 6, ""),
    ];
    let mut vus = Vec::new();
    for (texte, graine, attendu) in &cas {
        let e = lancer_texte(texte, *graine).empreinte;
        if !attendu.is_empty() {
            assert_eq!(&e, attendu, "{texte:?} graine {graine}");
        }
        assert!(!vus.contains(&e), "deux cas différents donnent la même empreinte");
        vus.push(e);
    }
    // chaque directive de la mémoire compte
    let base = lancer_texte(TUBES_MUR, 5).empreinte;
    for texte in [
        TUBES_MUR.replace("agent 3.0 0.5 0.05 0.1", "agent 3.5 0.5 0.05 0.1"),
        TUBES_MUR.replace("agent 3.0 0.5 0.05 0.1", "agent 3.0 0.4 0.05 0.1"),
        TUBES_MUR.replace("agent 3.0 0.5 0.05 0.1", "agent 3.0 0.5 0.06 0.1"),
        TUBES_MUR.replace("agent 3.0 0.5 0.05 0.1", "agent 3.0 0.5 0.05 0.2"),
        TUBES_MUR.replace("stimulus 2 1 0 1.0 0 20", "stimulus 2 2 0 1.0 0 20"),
        TUBES_MUR.replace("stimulus 2 1 0 1.0 0 20", "stimulus 2 1 0 1.5 0 20"),
        TUBES_MUR.replace("stimulus 2 1 0 1.0 0 20", "stimulus 2 1 0 1.0 1 20"),
        TUBES_MUR.replace("stimulus 2 1 0 1.0 0 20", "stimulus 2 1 0 1.0 0 21"),
    ] {
        assert_ne!(lancer_texte(&texte, 5).empreinte, base, "{texte:?}");
    }
    let zero = TUBES_MUR.replace("agent 3.0", "agent 0.0").replace("stimulus 6 5 0 2.0 10 30\n", "");
    let moins_zero = TUBES_MUR.replace("agent 3.0", "agent -0").replace("stimulus 6 5 0 2.0 10 30\n", "");
    assert_ne!(
        lancer_texte(&moins_zero, 5).empreinte,
        lancer_texte(&zero, 5).empreinte,
        "le signe du zéro est dans l'état"
    );
}

/// régression : pour `scenarios/souvenir.txt` et la graine 1, l'empreinte rendue par `lancer` est celle du fichier
/// `references/souvenir-graine-1.empreinte` (l'empreinte puis un saut de ligne), que la CI rejoue sous Linux et sous
/// Windows : les opérations sont des entiers et des opérations exactes de `f64` dans un ordre fixé (`+ - * /`, aucune
/// fonction du système), donc les mêmes bits partout.
///
/// Ne couvre pas : que l'empreinte soit juste (`empreintes_de_scenarios_avec_agent` la compare au modèle Python).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn regression_reference_souvenir() {
    let reference = fs::read_to_string(format!("{RACINE}/references/souvenir-graine-1.empreinte")).unwrap();
    assert_eq!(reference.len(), 65);
    assert!(reference.ends_with('\n'));
    assert_eq!(lancer_texte(&fichier("souvenir.txt"), 1).empreinte, reference.trim_end());
}

/// La ligne de points de l'aperçu (un point par colonne du treillis, `echelle` pixels de large), sur les 8 lignes d'un
/// aperçu d'une ligne de nœuds : le même point écrit `echelle` fois, répété `echelle` fois.
fn ligne_de_points(points: &[[u8; 3]], echelle: usize) -> Vec<u8> {
    let mut rangee = Vec::new();
    for p in points {
        for _ in 0..echelle {
            rangee.extend_from_slice(p);
        }
    }
    let mut sortie = Vec::new();
    for _ in 0..echelle {
        sortie.extend_from_slice(&rangee);
    }
    sortie
}

/// juste : l'aperçu avec agent (V-LAN-05) : `souvenir::apercu` avec la référence des tubes (`1.0` pour des tubes,
/// `loi.valeur(debit)` pour un réseau) et la référence de l'agent (la plus grande concentration finale, `1.0` si elle
/// est nulle). `PETIT_A` (40 × 8 pixels, échelle 8, cinq points : trois nœuds et deux milieux d'arêtes) : les points
/// sont ceux de `tubes::apercu` avec les nœuds mêlés à `AGENT = [130, 255, 80]` selon `c / c_max` ; `PETIT_B` de même ;
/// `scenarios/souvenir.txt` (graine 1 : 232 × 168, SHA-256 des octets RVB `983440d1…426c`) ; `TUBES_MUR`
/// (`e629395a…3e6b`, 136 × 104) ; `RESEAU_ETAGES` (`08a24f72…83deb`, 192 × 56) ; le cas sans stimulus (`c8e3662c…`,
/// l'agent nul, la référence `1.0`) a pour aperçu celui de `tubes::apercu` seul (`ea754c69…a50b`). Les valeurs viennent
/// du modèle Python.
///
/// Ne couvre pas : l'agrandissement `echelle` autre que 8 (la règle est celle de `lancer` plus haut).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn l_apercu_avec_agent() {
    let cas: [(&str, u64, (u32, u32), &str); 6] = [
        (PETIT_A, 7, (40, 8), "9f0cda50111f94ba7a2fda1ffc9ef15caa9729a3fc6d4f23f70cdf7185e67175"),
        (PETIT_B, 3, (40, 8), "017246a522ed69edd029dbb8227331c4b010c36d1954f1d7ccddc90d4012ed8e"),
        ("", 1, (232, 168), "983440d15282452d7ab560fb4107c2bbe940e55cfae94b4c34a542748e16426c"),
        (TUBES_MUR, 5, (136, 104), "e629395ac02e7dc206a1ce56585f1c56cb03e0fa4166a0e604814c333d273e6b"),
        (RESEAU_ETAGES, 4, (192, 56), "08a24f72a2bea9eb5fa96c53c5112a1887d1f5078ca5760fb38faba54e183deb"),
        ("SANS", 5, (136, 104), "ea754c6946d207a1179142e63cda1a3de759b5dd86f7c961790174a85bffa50b"),
    ];
    let souvenir = fichier("souvenir.txt");
    let sans = TUBES_MUR.replace("stimulus 2 1 0 1.0 0 20\nstimulus 6 5 0 2.0 10 30\n", "");
    for (texte, graine, (largeur, hauteur), sha) in cas {
        let texte = match texte {
            "" => souvenir.as_str(),
            "SANS" => sans.as_str(),
            t => t,
        };
        let png = common::relire_png(&lancer_texte(texte, graine).apercu).expect("PNG valide");
        assert_eq!((png.largeur, png.hauteur), (largeur, hauteur), "{texte:?}");
        assert_eq!(sha256_hex(&png.rvb), sha, "{texte:?}");
    }
    // les cinq points de PETIT_A et de PETIT_B, écrits à la main (`tubes::apercu` puis les nœuds mêlés à AGENT) : le
    // nœud 1 porte la plus grande concentration (niveau 255 : `AGENT`), les autres un mélange selon `c / c_max`
    let a = common::relire_png(&lancer_texte(PETIT_A, 7).apercu).expect("PNG valide");
    let points_a = [[60, 216, 247], [59, 215, 250], [130, 255, 80], [52, 185, 215], [66, 197, 191]];
    assert_eq!(a.rvb, ligne_de_points(&points_a, 8));
    let b = common::relire_png(&lancer_texte(PETIT_B, 3).apercu).expect("PNG valide");
    let points_b = [[95, 238, 167], [60, 220, 255], [130, 255, 80], [60, 220, 255], [60, 220, 255]];
    assert_eq!(b.rvb, ligne_de_points(&points_b, 8));
}

/// juste : **l'ordre des stimuli du fichier compte** (V-LAN-10) : deux stimuli au même nœud (`grille 3 1`, `source 0 0
/// 0`, `puits 2 0 0`, `pas 3 0.3`, `agent 2.0 1.0 0.1 0.5`, graine 7), l'un de débit `0.1` sur `[0 ; 3[`, l'autre de
/// débit `0.2` sur `[1 ; 3[` : au pas 1 le nœud porte de l'agent et reçoit les deux apports, et la somme de flottants
/// dépend de l'ordre (`(c + 0.03) + 0.06` contre `(c + 0.06) + 0.03`). Dans l'ordre `0.1` puis `0.2` : `5f8f0017…2bc0`
/// ; dans l'autre ordre du fichier : `067a8628…d468` (modèle Python) : les concentrations finales diffèrent d'un cran.
/// Un code qui rangerait les apports autrement que dans l'ordre du fichier serait vu.
///
/// Ne couvre pas : deux stimuli à des nœuds différents (leurs apports n'interfèrent pas : l'ordre ne change aucun bit).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn l_ordre_des_stimuli_au_meme_noeud_compte() {
    let base = "grille 3 1
source 0 0 0
puits 2 0 0
pas 3 0.3
agent 2.0 1.0 0.1 0.5
";
    let (a, b) = (
        "stimulus 1 0 0 0.1 0 3
",
        "stimulus 1 0 0 0.2 1 3
",
    );
    assert_eq!(
        lancer_texte(&format!("{base}{a}{b}"), 7).empreinte,
        "5f8f00175fc1315f3b91d7f38b2f601cf83d4e88082605d9873048bcef872bc0"
    );
    assert_eq!(
        lancer_texte(&format!("{base}{b}{a}"), 7).empreinte,
        "067a86287e06d44bade4b90a388d955e2fbf0dceec2e631526e6d527be27d468"
    );
}

/// Les cas de directives fautives : chaque texte et la ligne attendue.
fn verifier_lignes(cas: &[(String, usize)]) {
    for (texte, ligne) in cas {
        assert_eq!(erreur_de(texte), Some(ErreurLancer::DirectiveInconnue(*ligne)), "{texte:?}");
    }
}

/// Un scénario de tubes de base (les six lignes) : `grille` ligne 1, `source` 2, `puits` 3, `pas` 4, `agent` 5,
/// `stimulus` 6.
fn tubes(agent: &str, stimulus: &str) -> String {
    format!("grille 5 4\nsource 0 0 0\npuits 4 3 0\npas 5 0.5\n{agent}\n{stimulus}\n")
}

/// juste : les erreurs des directives `agent` et `stimulus` (V-LAN-06) sont `DirectiveInconnue(ligne)` avec la règle de
/// priorité de la doc de `lancer` (« Ce que fait l'étape 2, partie 2M ») : **bornes comprises** (`agent 100 0.001 10
/// 10` et `agent 0 1000 0 0` passent, `-0` aussi ; `100.5`, `0.0009`, `1000.5`, `10.5` et les négatifs non ; l'exposant
/// `1e2` et `.5` non plus) ; `stimulus` : cinq ou sept arguments, un nœud hors de la grille, un débit `<= 0` ou `>
/// 100`, `debut >= fin`, `fin > 100000` sont fautifs (`debit 100` et `fin 100000` passent) ; un second `agent` ; seize
/// `stimulus` passent, le dix-septième est fautif (sa ligne) ; **un `stimulus` sans `agent`** rend la ligne de la
/// première directive du fichier (que l'`agent` manque ou non, après les autres erreurs) ; un `agent` sans les tubes ni
/// le réseau rend aussi la première ; la première directive fautive l'emporte sur tout ; une source égale au puits rend
/// la ligne de la seconde ; un `stimulus` sur un nœud muré, ou séparé du puits par les murs, rend sa ligne, **après**
/// les erreurs de `labyrinthe::construire` (source murée) et celles des autres nourritures, avec le premier `stimulus`
/// du fichier à qui cela arrive ; les deux familles sont couvertes ; `agent` avec `source` et `nourriture` rend la
/// ligne de la première `source` ou `puits`.
///
/// Ne couvre pas : les erreurs de lecture (`etape_0_scenario`), les erreurs des tubes et des réseaux seuls.
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn erreurs_de_directives() {
    let bon_agent = "agent 2.0 1.0 0.1 0.1";
    let bon_stim = "stimulus 2 2 0 1.0 0 3";
    assert!(erreur_de(&tubes(bon_agent, bon_stim)).is_none());
    let mut cas: Vec<(String, usize)> = Vec::new();
    // un `agent` fautif : ligne 5
    for agent in [
        "agent 2.0 1.0 0.1",
        "agent 2.0 1.0 0.1 0.1 0.0",
        "agent",
        "agent a 1.0 0.1 0.1",
        "agent -1 1.0 0.1 0.1",
        "agent 100.5 1.0 0.1 0.1",
        "agent 2.0 0.0009 0.1 0.1",
        "agent 2.0 0 0.1 0.1",
        "agent 2.0 1000.5 0.1 0.1",
        "agent 2.0 1.0 10.5 0.1",
        "agent 2.0 1.0 -0.5 0.1",
        "agent 2.0 1.0 0.1 10.5",
        "agent 2.0 1.0 0.1 -0.1",
        "agent 1e2 1.0 0.1 0.1",
        "agent .5 1.0 0.1 0.1",
        "agent +2 1.0 0.1 0.1",
    ] {
        cas.push((tubes(agent, bon_stim), 5));
    }
    for stimulus in [
        "stimulus 2 2 0 1.0 0",
        "stimulus 2 2 0 1.0 0 3 4",
        "stimulus 5 2 0 1.0 0 3",
        "stimulus 2 4 0 1.0 0 3",
        "stimulus 2 2 1 1.0 0 3",
        "stimulus 2 2 0 0 0 3",
        "stimulus 2 2 0 -1.0 0 3",
        "stimulus 2 2 0 100.5 0 3",
        "stimulus 2 2 0 1.0 3 3",
        "stimulus 2 2 0 1.0 4 3",
        "stimulus 2 2 0 1.0 0 100001",
        "stimulus 2 2 0 1.0 a 3",
        "stimulus 2 2 0 1.0 0 3.0",
        "stimulus -1 2 0 1.0 0 3",
    ] {
        cas.push((tubes(bon_agent, stimulus), 6));
    }
    verifier_lignes(&cas);
    // les bornes comprises
    for agent in ["agent 100 0.001 10 10", "agent 0 1000 0 0", "agent -0 1 0 0", "agent 100.0 1000.0 10.0 10.0"] {
        assert!(erreur_de(&tubes(agent, bon_stim)).is_none(), "{agent}");
    }
    for stimulus in ["stimulus 4 3 0 100 0 3", "stimulus 0 0 0 0.001 99999 100000", "stimulus 0 0 0 1.0 0 1"] {
        assert!(erreur_de(&tubes(bon_agent, stimulus)).is_none(), "{stimulus}");
    }
    // un second agent, seize puis dix-sept stimuli
    verifier_lignes(&[(format!("{}agent 1.0 1.0 0.1 0.1\n", tubes(bon_agent, bon_stim)), 7)]);
    let seize = |n: usize| {
        let mut t = format!("grille 5 4\nsource 0 0 0\npuits 4 3 0\npas 5 0.5\n{bon_agent}\n");
        for _ in 0..n {
            t.push_str(&format!("{bon_stim}\n"));
        }
        t
    };
    assert!(erreur_de(&seize(16)).is_none());
    verifier_lignes(&[(seize(17), 5 + 17)]);
    // un `stimulus` sans `agent`, un `agent` sans tubes : la première directive restante
    verifier_lignes(&[
        (format!("grille 5 4\nsource 0 0 0\npuits 4 3 0\npas 5 0.5\n{bon_stim}\n"), 2),
        (format!("# un commentaire\n\ngrille 5 4\nsource 0 0 0\npuits 4 3 0\npas 5 0.5\n{bon_stim}\n"), 4),
        (format!("grille 5 4\n{bon_agent}\n"), 2),
        (format!("grille 5 4\nsource 0 0 0\npuits 4 3 0\n{bon_agent}\n"), 2),
        (format!("grille 5 4\n{bon_stim}\n{bon_agent}\n"), 2),
        // la première directive fautive l'emporte
        ("grille 5 4\nsource 0 0 0\npuits 4 3 0\npas 5 0.5\nstimulus 9 2 0 1.0 0 3\n".to_string(), 5),
        (
            String::from(
                "grille 5 4\nsource 0 0 0\npuits 4 3 0\nagent 2.0 1.0 0.1\nstimulus 2 2 0 1.0 0 3\npas 5 0.5\n\
                pas 5 0.5\n",
            ),
            4,
        ),
        // une source égale au puits : la seconde des deux
        (format!("grille 5 4\nsource 1 1 0\n{bon_agent}\npuits 1 1 0\npas 5 0.5\n{bon_stim}\n"), 4),
    ]);
    // les murs : un stimulus muré ou coupé du puits rend sa ligne, après les erreurs du plan
    let mur = |murs: &str, source: &str, stimuli: &str| {
        format!("grille 5 4\n{murs}{source}puits 4 3 0\npas 5 0.5\n{bon_agent}\n{stimuli}")
    };
    verifier_lignes(&[
        // mur sur (2, 2), stimulus sur (2, 2) : la ligne du stimulus (grille, mur, source, puits, pas, agent, stimulus)
        (mur("mur 2 2 2 2 0\n", "source 0 0 0\n", "stimulus 1 1 0 1.0 0 3\nstimulus 2 2 0 1.0 0 3\n"), 8),
        // deux stimuli murés : le premier du fichier
        (mur("mur 2 2 3 3 0\n", "source 0 0 0\n", "stimulus 3 3 0 1.0 0 3\nstimulus 2 2 0 1.0 0 3\n"), 7),
        // la source murée l'emporte sur un stimulus muré
        (mur("mur 0 0 0 0 0\nmur 2 2 2 2 0\n", "source 0 0 0\n", "stimulus 2 2 0 1.0 0 3\n"), 4),
        // un mur qui coupe la grille en deux : un stimulus du mauvais côté
        (mur("mur 2 0 2 3 0\n", "source 3 0 0\n", "stimulus 4 0 0 1.0 0 3\nstimulus 0 1 0 1.0 0 3\n"), 8),
    ]);
    // la famille des nourritures
    let reseau = |extra: &str, stimuli: &str| {
        format!(
            "grille 5 4\n{extra}nourriture 0 0 0\nnourriture 4 3 0\nnourriture 2 0 0\ndebit 1.5\nloi lineaire\n\
            pas 5 0.5\n\
{bon_agent}\n{stimuli}"
        )
    };
    assert!(erreur_de(&reseau("", "stimulus 1 1 0 1.0 0 3\n")).is_none());
    verifier_lignes(&[
        // un stimulus muré : sa ligne
        (reseau("mur 1 1 1 1 0\n", "stimulus 1 1 0 1.0 0 3\n"), 10),
        // une troisième nourriture murée l'emporte sur un stimulus muré
        (reseau("mur 2 0 2 0 0\nmur 1 1 1 1 0\n", "stimulus 1 1 0 1.0 0 3\n"), 6),
        // une source avec des nourritures : la première source ou puits
        (reseau("source 1 1 0\n", "stimulus 1 1 0 1.0 0 3\n"), 2),
        // un stimulus sans agent
        (
            "grille 5 4\nnourriture 0 0 0\nnourriture 4 3 0\ndebit 1.5\nloi lineaire\npas 5 0.5\n\
            stimulus 1 1 0 1.0 0 3\n"
                .to_string(),
            2,
        ),
        // un agent fautif
        (reseau("", "stimulus 1 1 0 1.0 0 3\n").replace("agent 2.0 1.0 0.1 0.1", "agent 2.0 1.0 0.1"), 8),
    ]);
}

/// juste : **un calcul qui s'arrête** avec un agent (V-LAN-07) : `grille 3 1`, `etages 2 1.0 10000000000000000` (un
/// lien de coût `10^16`), nourritures `(0,0,0)` et `(2,0,1)`, `h = 0,5`, graine 1, **`agent 0.0 …`** (la force nulle :
/// les tubes sont ceux de `etape_1r_lancer`, donc le solveur s'arrête aux mêmes pas) : sous la loi linéaire (`debit 1`)
/// au pas numéro 4, sous la sigmoïde au pas numéro 3 (`ErreurLancer::Calcul(t)`, aucune sortie) ; avec un stimulus,
/// rien ne change tant que `α = 0` ; avec `pas` égal au pas fautif, tout va bien ; avec zéro pas, jamais d'erreur de
/// calcul.
///
/// Ne couvre pas : les erreurs du transport de l'agent dans un calcul (`etape_2m_loi` les éprouve sur `avancer`).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn un_calcul_qui_s_arrete() {
    let texte = |loi: &str, pas: u32, stimulus: &str| {
        format!(
            "grille 3 1\netages 2 1.0 10000000000000000\nnourriture 0 0 0\nnourriture 2 0 1\ndebit 1\nloi {loi}\n\
            pas {pas} 0.5\n\
agent 0.0 1.0 0.1 0.1\n{stimulus}"
        )
    };
    for stimulus in ["", "stimulus 1 0 0 1.0 0 5\n"] {
        assert!(erreur_de(&texte("lineaire", 4, stimulus)).is_none(), "quatre pas (0 à 3) passent");
        assert_eq!(erreur_de(&texte("lineaire", 5, stimulus)), Some(ErreurLancer::Calcul(4)));
        assert_eq!(erreur_de(&texte("lineaire", 40, stimulus)), Some(ErreurLancer::Calcul(4)));
        assert!(erreur_de(&texte("sigmoide", 3, stimulus)).is_none(), "trois pas (0 à 2) passent");
        assert_eq!(erreur_de(&texte("sigmoide", 4, stimulus)), Some(ErreurLancer::Calcul(3)));
        assert!(erreur_de(&texte("sigmoide", 0, stimulus)).is_none(), "zéro pas : jamais d'erreur de calcul");
    }
}

/// juste : **les scénarios du dossier `scenarios/`** (V-LAN-08) : la règle porte sur tout le dossier, jamais sur sa
/// liste exacte. Chaque fichier `*.txt` se lit et se lance (graine 1) ; **un scénario sans `agent` ni `stimulus` garde
/// son empreinte** : celle de `references/<nom>-graine-1.empreinte` quand elle existe, et pour les anciens scénarios
/// connus par leur nom (`chemin.txt` `6e3b15b8…2f37`, `labyrinthe.txt` `dcdad079…3ded`, `etages.txt` `d011d6bc…df76`,
/// quand ils sont là) les empreintes d'avant 2M, et un petit cas de tubes du cœur ; **un scénario avec agent** donne
/// son empreinte de référence quand elle existe, et `scenarios/souvenir.txt` existe et en est un. Une directive `agent`
/// sans `stimulus`, c'est le témoin : le même scénario avec ses lignes `agent` et `stimulus` retirées se lance aussi.
///
/// Ne couvre pas : les scénarios que d'autres étapes ajouteront (ils suivent la règle).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn les_scenarios_du_dossier_gardent_leur_empreinte() {
    let connus = [
        ("chemin.txt", "6e3b15b88963ed142ce3441c33b249b2164d13a51087662b8927f77ddd3c2f37"),
        ("labyrinthe.txt", "dcdad07950dc6dbb0aaa141bc1bb7fe7c0bdbe309a1f4d75b0cbf2dcba213ded"),
        ("etages.txt", "d011d6bc3988a72730aefba5b8c90ac5f460cd8c65c0f3ece05564acd218df76"),
    ];
    let mut noms: Vec<String> = fs::read_dir(format!("{RACINE}/scenarios"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .filter(|n| n.ends_with(".txt"))
        .collect();
    noms.sort();
    assert!(noms.iter().any(|n| n == "souvenir.txt"), "scenarios/souvenir.txt existe : {noms:?}");
    for nom in &noms {
        let texte = fichier(nom);
        let scenario = lire_scenario(&texte).unwrap_or_else(|e| panic!("{nom} : {e:?}"));
        let avec_agent = scenario.directives.iter().any(|d| d.nom == "agent" || d.nom == "stimulus");
        let sortie = lancer(&scenario, 1).unwrap_or_else(|e| panic!("{nom} : {e:?}"));
        let stem = nom.trim_end_matches(".txt");
        if let Ok(reference) = fs::read_to_string(format!("{RACINE}/references/{stem}-graine-1.empreinte")) {
            assert_eq!(sortie.empreinte, reference.trim_end(), "{nom}");
        }
        if let Some((_, attendu)) = connus.iter().find(|(n, _)| n == nom) {
            assert_eq!(&sortie.empreinte, attendu, "{nom}");
        }
        if avec_agent {
            let sans: String = texte
                .lines()
                .filter(|l| !l.starts_with("agent ") && !l.starts_with("stimulus "))
                .map(|l| format!("{l}\n"))
                .collect();
            assert!(lancer(&lire_scenario(&sans).unwrap(), 1).is_ok(), "{nom} sans agent");
        }
    }
    assert_eq!(
        lancer_texte("grille 4 4\nsource 3 0 0\npuits 0 3 0\npas 30 0.5\n", 9).empreinte,
        "ee991bd8ca038070aee07a55ce6bdb9fb2e58772d6f128fd66293f4ab854bed0"
    );
}

/// juste : le binaire avec un agent (V-LAN-09) : `physarum lancer scenarios/souvenir.txt --graine 1 --sortie <dossier>`
/// rend le code 0 et écrit `empreinte.txt` (`ff1fa667…7c5` puis `\n`), `mesures.csv` (401 lignes, SHA-256 du modèle
/// Python `3421b02d…43e6`) et `apercu.png` (232 × 168 pixels) ; un `stimulus` sur un mur rend le code 1, un message
/// sans `panicked` qui contient `ligne 3` et n'écrit rien.
///
/// Ne couvre pas : les erreurs d'arguments (voir `etape_0_binaire`).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn le_binaire_avec_un_agent() {
    let dossier = common::dossier_temporaire("2m_binaire");
    fs::create_dir_all(&dossier).unwrap();
    let sortie = dossier.join("sortie");
    let lancer_binaire = |scenario: &str, sortie: &std::path::Path| {
        Command::new(env!("CARGO_BIN_EXE_physarum"))
            .args(["lancer", scenario, "--graine", "1", "--sortie", sortie.to_str().unwrap()])
            .output()
            .unwrap()
    };
    let resultat = lancer_binaire(&format!("{RACINE}/scenarios/souvenir.txt"), &sortie);
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    assert_eq!(
        fs::read_to_string(sortie.join("empreinte.txt")).unwrap(),
        "ff1fa667fb9a3a6025a36f1cd2e3176e7a3f81e7f7d093e4410208584b7cf7c5\n"
    );
    let mesures = fs::read_to_string(sortie.join("mesures.csv")).unwrap();
    assert_eq!(mesures.lines().count(), 401);
    assert_eq!(sha256_hex(mesures.as_bytes()), "3421b02ddf232be39ebce59a31d426024b6d6295ed71caa4ba92d28f2ec543e6");
    let lu = common::relire_png(&fs::read(sortie.join("apercu.png")).unwrap()).expect("PNG valide");
    assert_eq!((lu.largeur, lu.hauteur), (232, 168));
    let fautif = dossier.join("mur.txt");
    fs::write(
        &fautif,
        "grille 4 4\nmur 2 2 2 2 0\nstimulus 2 2 0 1.0 0 3\nsource 0 0 0\npuits 3 3 0\npas 2 0.5\n\
        agent 1.0 1.0 0.1 0.1\n",
    )
    .unwrap();
    let absent = dossier.join("absent").join("sortie");
    let resultat = lancer_binaire(fautif.to_str().unwrap(), &absent);
    let erreur = String::from_utf8_lossy(&resultat.stderr).into_owned();
    assert_eq!(resultat.status.code(), Some(1), "stderr : {erreur}");
    assert!(erreur.contains("ligne 3") && !erreur.contains("panicked"), "{erreur}");
    assert!(!dossier.join("absent").exists(), "rien n'est créé");
    let _ = fs::remove_dir_all(&dossier);
}
