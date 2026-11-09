//! Étape 0 : le binaire `physarum` (`physarum lancer <scénario> --graine <n> --sortie <dossier>`), lancé comme un
//! processus (`env!("CARGO_BIN_EXE_physarum")`). Valeurs attendues : `references/etape-0-valeurs.md`, identifiants
//! `V-BIN-xx`.
//!
//! Le contrat est dans la documentation de `src/main.rs`. Les empreintes attendues sont celles de `etape_0_lancer.rs`
//! (SHA-256 de l'écriture canonique, calculé à part en Python) ; aucune ne vient du binaire.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn physarum(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_physarum")).args(args).output().expect("lancement de physarum")
}

fn s(chemin: &Path) -> &str {
    chemin.to_str().expect("chemin en UTF-8")
}

/// Un dossier de travail propre, avec un fichier de scénario à l'intérieur ; rend (dossier, chemin du scénario).
fn espace(nom: &str, scenario: &str) -> (PathBuf, PathBuf) {
    let dossier = common::dossier_temporaire(nom);
    fs::create_dir_all(&dossier).unwrap();
    let chemin = dossier.join("scenario.txt");
    fs::write(&chemin, scenario).unwrap();
    (dossier, chemin)
}

fn noms(dossier: &Path) -> Vec<String> {
    let mut noms: Vec<String> =
        fs::read_dir(dossier).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    noms.sort();
    noms
}

/// L'empreinte de `grille 3 2`, graine 7 (V-LANC-02, calculée à part).
const GRILLE_3_2_GRAINE_7: &str = "4d066023b79eba897bc283369d234a9ca2377cb85b9a3d8cf054c87fb1fc4008\n";

/// juste : `physarum lancer scenario.txt --graine 7 --sortie <dossier>` réussit (code 0), crée le dossier avec ses parents,
/// et y écrit exactement `empreinte.txt` (l'empreinte calculée à part de `grille 3 2` graine 7, puis `\n`), `mesures.csv`
/// (`temps,noeuds,aretes,longueur\n0,6,7,7.000000\n`) et `apercu.png` (un PNG de 40 × 24 pixels que le décodeur du test
/// relit, premier pixel `(225, 225, 215)`), et aucun autre fichier (V-BIN-01).
///
/// Ne couvre pas : les chemins avec des caractères non ASCII.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn succes_ecrit_trois_fichiers() {
    let (dossier, scenario) = espace("succes", "grille 3 2\n");
    let sortie = dossier.join("a").join("b").join("sortie");
    let resultat = physarum(&["lancer", s(&scenario), "--graine", "7", "--sortie", s(&sortie)]);
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    assert_eq!(noms(&sortie), ["apercu.png", "empreinte.txt", "mesures.csv"]);
    assert_eq!(fs::read_to_string(sortie.join("empreinte.txt")).unwrap(), GRILLE_3_2_GRAINE_7);
    assert_eq!(
        fs::read_to_string(sortie.join("mesures.csv")).unwrap(),
        "temps,noeuds,aretes,longueur\n0,6,7,7.000000\n"
    );
    let lu = common::relire_png(&fs::read(sortie.join("apercu.png")).unwrap()).expect("PNG valide");
    assert_eq!((lu.largeur, lu.hauteur), (40, 24));
    assert_eq!(lu.rvb[..3], [225, 225, 215]);
    let _ = fs::remove_dir_all(&dossier);
}

/// juste : les options peuvent venir dans n'importe quel ordre, et le chemin du scénario n'importe où parmi elles ; la graine
/// maximale `18446744073709551615` est acceptée (V-BIN-02 : `grille 256 1` avec cette graine a pour empreinte
/// `1eceee45…2618`, calculée à part) ; un scénario à deux étages donne un aperçu de 96 × 24 (V-BIN-03).
///
/// Ne couvre pas : les options abrégées ou écrites `--graine=7` (refusées par la doc : « option inconnue »).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn ordres_des_options_graine_maximale_et_etages() {
    let (dossier, scenario) = espace("ordres", "grille 3 2\n");
    let sc = s(&scenario);
    let (d1, d2, d3) = (dossier.join("s1"), dossier.join("s2"), dossier.join("s3"));
    for (args, sortie) in [
        (vec!["lancer", sc, "--sortie", s(&d1), "--graine", "7"], &d1),
        (vec!["lancer", "--graine", "7", sc, "--sortie", s(&d2)], &d2),
        (vec!["lancer", "--sortie", s(&d3), "--graine", "7", sc], &d3),
    ] {
        let resultat = physarum(&args);
        assert_eq!(resultat.status.code(), Some(0), "{args:?} : {}", String::from_utf8_lossy(&resultat.stderr));
        assert_eq!(fs::read_to_string(sortie.join("empreinte.txt")).unwrap(), GRILLE_3_2_GRAINE_7, "{args:?}");
    }
    let large = dossier.join("large.txt");
    fs::write(&large, "grille 256 1\n").unwrap();
    let d4 = dossier.join("s4");
    let resultat = physarum(&["lancer", s(&large), "--graine", "18446744073709551615", "--sortie", s(&d4)]);
    assert_eq!(resultat.status.code(), Some(0), "{}", String::from_utf8_lossy(&resultat.stderr));
    assert_eq!(
        fs::read_to_string(d4.join("empreinte.txt")).unwrap(),
        "1eceee455fd4f200f1c1849016206d2de3798b1454d5e4a72f7d44807eaa2618\n"
    );
    let etages = dossier.join("etages.txt");
    fs::write(&etages, "# deux étages\ngrille 3 2\netages 2 0.5 3\n").unwrap();
    let d5 = dossier.join("s5");
    let resultat = physarum(&["lancer", s(&etages), "--graine", "7", "--sortie", s(&d5)]);
    assert_eq!(resultat.status.code(), Some(0), "{}", String::from_utf8_lossy(&resultat.stderr));
    let lu = common::relire_png(&fs::read(d5.join("apercu.png")).unwrap()).expect("PNG valide");
    assert_eq!((lu.largeur, lu.hauteur), (96, 24));
    assert_eq!(lu.rvb[..3], [240, 170, 40], "le premier nœud est un puits");
    assert_eq!(
        fs::read_to_string(d5.join("mesures.csv")).unwrap(),
        "temps,noeuds,aretes,longueur\n0,12,20,17.000000\n"
    );
    assert_eq!(
        fs::read_to_string(d5.join("empreinte.txt")).unwrap(),
        "ca0220ae8eef812c1b2f386d8bec1349d2e8ea2c2908d7b9789258703b867db9\n"
    );
    let _ = fs::remove_dir_all(&dossier);
}

/// juste : un dossier de sortie qui existe déjà et contient les trois fichiers les voit remplacés par ceux du nouveau
/// lancer (graine 1 puis graine 2 : les deux empreintes sont celles, calculées à part, du scénario d'exemple
/// `scenarios/grille.txt`, `c3361481…af59` puis `3e99fcca…d908`) ; ses mesures comptent 768 nœuds et 3 220 arêtes
/// (V-BIN-04).
///
/// Ne couvre pas : un dossier de sortie sans droit d'écriture.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn dossier_existant_remplace_et_scenario_d_exemple() {
    let dossier = common::dossier_temporaire("remplace");
    let exemple = concat!(env!("CARGO_MANIFEST_DIR"), "/scenarios/grille.txt");
    for (graine, sha) in [
        ("1", "c336148193197cb8541fd6a58c36f149118260789f9e388771405fe52e6eaf59"),
        ("2", "3e99fcca2d6414453203fba48994397ff4a829b2c610cfe371ee2948f744d908"),
    ] {
        let resultat = physarum(&["lancer", exemple, "--graine", graine, "--sortie", s(&dossier)]);
        assert_eq!(resultat.status.code(), Some(0), "{}", String::from_utf8_lossy(&resultat.stderr));
        assert_eq!(noms(&dossier), ["apercu.png", "empreinte.txt", "mesures.csv"]);
        assert_eq!(fs::read_to_string(dossier.join("empreinte.txt")).unwrap(), format!("{sha}\n"), "graine {graine}");
        assert_eq!(
            fs::read_to_string(dossier.join("mesures.csv")).unwrap(),
            "temps,noeuds,aretes,longueur\n0,768,3220,3791.614716\n"
        );
    }
    let _ = fs::remove_dir_all(&dossier);
}

/// juste : toute erreur donne le code de sortie 1, un message sur la sortie d'erreur qui n'est pas une panique (pas de
/// `panicked`), et rien d'écrit ni de créé : aucun dossier de sortie n'apparaît. Cas : aucun argument ; `lancer` seul ; une
/// autre commande ; la graine, la sortie ou le scénario absents ; une option donnée deux fois ; un second scénario ; une
/// option inconnue ; une option sans valeur ; une graine `abc`, `-1`, `+5`, `1.5`, vide, `0x10`, ` 5`, `18446744073709551616`
/// ou `--sortie` ; un fichier absent ou qui est un dossier (V-BIN-05, la doc de `src/main.rs`).
///
/// Ne couvre pas : les erreurs de scénario (voir `erreurs_de_scenario`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn erreurs_d_arguments_et_de_fichier() {
    let (dossier, scenario) = espace("erreurs_args", "grille 3 2\n");
    let sc = s(&scenario);
    let sortie = dossier.join("sortie");
    let so = s(&sortie);
    let absent = dossier.join("absent.txt");
    let cas: Vec<Vec<&str>> = vec![
        vec![],
        vec!["lancer"],
        vec!["autre", sc, "--graine", "1", "--sortie", so],
        vec!["lancer", sc, "--sortie", so],
        vec!["lancer", sc, "--graine", "1"],
        vec!["lancer", "--graine", "1", "--sortie", so],
        vec!["lancer", sc, "--graine", "1", "--graine", "2", "--sortie", so],
        vec!["lancer", sc, "--graine", "1", "--sortie", so, "--sortie", so],
        vec!["lancer", sc, sc, "--graine", "1", "--sortie", so],
        vec!["lancer", sc, "--graine", "1", "--sortie", so, "--bidon", "x"],
        vec!["lancer", sc, "--graine", "1", "--sortie", so, "--bidon"],
        vec!["lancer", "--bidon", sc, "--graine", "1", "--sortie", so],
        vec!["lancer", sc, "--graine=1", "--sortie", so],
        vec!["lancer", sc, "--graine", "1", "--sortie"],
        vec!["lancer", sc, "--graine"],
        vec!["lancer", sc, "--graine", "abc", "--sortie", so],
        vec!["lancer", sc, "--graine", "-1", "--sortie", so],
        vec!["lancer", sc, "--graine", "+5", "--sortie", so],
        vec!["lancer", sc, "--graine", "1.5", "--sortie", so],
        vec!["lancer", sc, "--graine", "", "--sortie", so],
        vec!["lancer", sc, "--graine", "0x10", "--sortie", so],
        vec!["lancer", sc, "--graine", " 5", "--sortie", so],
        vec!["lancer", sc, "--graine", "18446744073709551616", "--sortie", so],
        vec!["lancer", sc, "--graine", "--sortie", so],
        vec!["lancer", s(&absent), "--graine", "1", "--sortie", so],
        vec!["lancer", s(&dossier), "--graine", "1", "--sortie", so],
    ];
    for args in cas {
        let resultat = physarum(&args);
        let erreur = String::from_utf8_lossy(&resultat.stderr).into_owned();
        assert_eq!(resultat.status.code(), Some(1), "{args:?} : code de sortie ; stderr : {erreur}");
        assert!(!erreur.trim().is_empty(), "{args:?} : le message d'erreur est vide");
        assert!(!erreur.contains("panicked"), "{args:?} : le binaire a paniqué : {erreur}");
        assert!(!sortie.exists(), "{args:?} : le dossier de sortie a été créé");
    }
    // le même appel, bien écrit, réussit (un binaire qui échoue toujours ne passe pas)
    let resultat = physarum(&["lancer", sc, "--graine", "1", "--sortie", so]);
    assert_eq!(resultat.status.code(), Some(0), "{}", String::from_utf8_lossy(&resultat.stderr));
    let _ = fs::remove_dir_all(&dossier);
}

/// juste : un scénario refusé donne le code 1, un message qui nomme la faute, et rien d'écrit même quand le dossier de sortie
/// existe déjà (il reste vide) : une ligne fautive (`ligne 3` pour `Mauvais 1`, `ligne 1` pour `grille 0 5`, `ligne 2` pour une
/// `grille` en double, `ligne 2` pour `voisinage 6`) ; une grille manquante (le message contient `grille`) ; une directive que
/// `lancer` ne comprend pas (`ligne 3` pour `inconnue 1 2`, la première des directives) (V-BIN-06, la doc de `src/main.rs`).
///
/// Ne couvre pas : le texte exact des messages (seul le numéro de ligne ou le mot `grille` y figure).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn erreurs_de_scenario() {
    let cas: [(&str, &str); 6] = [
        ("grille 2 2\n\nMauvais 1\n", "ligne 3"),
        ("grille 0 5\n", "ligne 1"),
        ("grille 2 2\ngrille 3 3\n", "ligne 2"),
        ("grille 2 2\nvoisinage 6\n", "ligne 2"),
        ("# rien du tout\nx 1\n", "grille"),
        ("grille 2 2\n\ninconnue 1 2\nautre 3\n", "ligne 3"),
    ];
    for (i, (texte, attendu)) in cas.into_iter().enumerate() {
        let (dossier, scenario) = espace(&format!("scenario_{i}"), texte);
        let sortie = dossier.join("sortie_vide");
        fs::create_dir_all(&sortie).unwrap();
        let resultat = physarum(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&sortie)]);
        let erreur = String::from_utf8_lossy(&resultat.stderr).into_owned();
        assert_eq!(resultat.status.code(), Some(1), "{texte:?} : code de sortie ; stderr : {erreur}");
        assert!(erreur.contains(attendu), "{texte:?} : le message devrait contenir {attendu:?} : {erreur}");
        assert!(!erreur.contains("panicked"), "{texte:?} : panique : {erreur}");
        assert!(noms(&sortie).is_empty(), "{texte:?} : un fichier a été écrit");
        // et vers un dossier qui n'existe pas encore : il n'est pas créé, ni ses parents
        let absent = dossier.join("pas_encore").join("sortie");
        let resultat = physarum(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&absent)]);
        assert_eq!(resultat.status.code(), Some(1), "{texte:?} : code de sortie vers un dossier absent");
        assert!(
            !dossier.join("pas_encore").exists(),
            "{texte:?} : le dossier de sortie a été créé pour un scénario refusé"
        );
        let _ = fs::remove_dir_all(&dossier);
    }
    // un dossier de sortie qui est un fichier : erreur, et le fichier est intact
    let (dossier, scenario) = espace("sortie_fichier", "grille 2 2\n");
    let fichier = dossier.join("pas_un_dossier");
    fs::write(&fichier, "intact").unwrap();
    let resultat = physarum(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&fichier)]);
    assert_eq!(resultat.status.code(), Some(1));
    assert!(!String::from_utf8_lossy(&resultat.stderr).contains("panicked"));
    assert_eq!(fs::read_to_string(&fichier).unwrap(), "intact");
    let _ = fs::remove_dir_all(&dossier);
}

/// régression : pour `scenarios/grille.txt` et la graine 1, `empreinte.txt` écrit par le binaire est le fichier
/// `references/grille-graine-1.empreinte` (l'empreinte puis un saut de ligne), le fichier que la CI rejoue sous Linux et sous
/// Windows. Il n'existe pas dans le jeu de tests : la vague du code le produit avec ce binaire (V-BIN-07).
///
/// Ne couvre pas : que l'empreinte soit juste (`dossier_existant_remplace_et_scenario_d_exemple` la compare à la valeur
/// calculée à part : `c3361481…af59`) ; l'égalité entre Linux et Windows n'est établie que par la CI qui rejoue cette
/// référence sur les deux.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn regression_reference_du_binaire() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let reference = fs::read_to_string(format!("{racine}/references/grille-graine-1.empreinte"))
        .expect("references/grille-graine-1.empreinte est absente : la vague du code la produit avec le binaire");
    let dossier = common::dossier_temporaire("reference");
    let resultat =
        physarum(&["lancer", &format!("{racine}/scenarios/grille.txt"), "--graine", "1", "--sortie", s(&dossier)]);
    assert_eq!(resultat.status.code(), Some(0), "{}", String::from_utf8_lossy(&resultat.stderr));
    assert_eq!(fs::read_to_string(dossier.join("empreinte.txt")).unwrap(), reference);
    let _ = fs::remove_dir_all(&dossier);
}
