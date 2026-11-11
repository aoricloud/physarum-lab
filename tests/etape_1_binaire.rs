//! Étape 1 : le binaire `physarum` avec les tubes, lancé comme un processus (`env!("CARGO_BIN_EXE_physarum")`). Valeurs
//! attendues : `references/etape-1-valeurs.md`, identifiants `V-BIN1-xx`.
//!
//! Le contrat est dans la documentation de `src/main.rs`. Les empreintes, les mesures et les numéros de pas attendus
//! sont ceux de `etape_1_lancer.rs` (calculés à part par le modèle Python) ; aucun ne vient du binaire.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use physarum_lab::empreinte::sha256_hex;

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

/// juste : le binaire écrit les sorties d'un scénario de tubes (V-BIN1-01). `grille 3 2`, source `(0,0,0)`, puits
/// `(2,1,0)`, cinq pas de 0,3, graine 1 : code 0 ; `empreinte.txt` porte l'empreinte du modèle Python (`39fe16f1…4f2e`)
/// puis `\n` ; `mesures.csv` est le texte entier de V-LTUB-03 (l'en-tête et cinq lignes) ; `apercu.png` est un PNG de
/// 40 × 24 pixels dont les octets RVB ont le SHA-256 du modèle Python (`4bebc2d4…58cb`) ; aucun autre fichier.
///
/// Ne couvre pas : les erreurs d'arguments (voir `etape_0_binaire`).
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn succes_avec_tubes() {
    let (dossier, scenario) = espace("tubes_succes", "grille 3 2\nsource 0 0 0\npuits 2 1 0\npas 5 0.3\n");
    let sortie = dossier.join("sortie");
    let resultat = physarum(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&sortie)]);
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    assert_eq!(noms(&sortie), ["apercu.png", "empreinte.txt", "mesures.csv"]);
    assert_eq!(
        fs::read_to_string(sortie.join("empreinte.txt")).unwrap(),
        "39fe16f176a330c47981a4b311accf29da0aac9baac4d597d8b24ce204814f2e\n"
    );
    assert_eq!(
        fs::read_to_string(sortie.join("mesures.csv")).unwrap(),
        "temps,pression,vifs,longueur\n0,1.861092680,7,7.000000\n1,2.072322056,7,7.000000\n\
         2,2.261420261,6,6.000000\n3,2.424992917,4,4.000000\n4,2.561673058,3,3.000000\n"
    );
    let lu = common::relire_png(&fs::read(sortie.join("apercu.png")).unwrap()).expect("PNG valide");
    assert_eq!((lu.largeur, lu.hauteur), (40, 24));
    assert_eq!(sha256_hex(&lu.rvb), "4bebc2d4fd3826b876b25349db9d60415cfb9f27635bf90733446091502358cb");
    let _ = fs::remove_dir_all(&dossier);
}

/// juste : un calcul qui s'arrête est une erreur comme les autres, pas un plantage (V-BIN1-02, la doc de `src/main.rs`)
/// : code de sortie 1, un message sans `panicked` qui contient `pas <t>` (le numéro du pas où le solveur s'est arrêté,
/// celui de V-LTUB-10 : `pas 2` pour un lien entre étages d'un coût de `10^16` et la graine 1, `pas 7` pour la graine
/// 2, `pas 13` pour la graine 9 (un numéro à deux chiffres, écrit en décimal), `pas 0` pour un coût de `10^-16` et la
/// graine 7), rien d'écrit dans un dossier de sortie qui existe, et un dossier de sortie absent n'est pas créé. Le même
/// scénario arrêté avant le pas fautif (`pas 2 0.5` : seuls les pas 0 et 1 sont faits) réussit et écrit ses trois
/// fichiers.
///
/// Ne couvre pas : le texte exact du message.
#[test]
#[cfg_attr(not(feature = "etape-1"), ignore)]
fn calcul_qui_s_arrete() {
    let grand = "grille 3 3\netages 2 1 10000000000000000\nsource 0 0 0\npuits 2 2 0\npas 40 0.5\n";
    let petit = "grille 3 3\netages 2 1 0.0000000000000001\nsource 0 0 0\npuits 2 2 0\npas 40 0.5\n";
    for (i, (texte, graine, attendu)) in
        [(grand, "1", "pas 2"), (grand, "2", "pas 7"), (grand, "9", "pas 13"), (petit, "7", "pas 0")]
            .into_iter()
            .enumerate()
    {
        let (dossier, scenario) = espace(&format!("tubes_arret_{i}"), texte);
        let sortie = dossier.join("sortie_vide");
        fs::create_dir_all(&sortie).unwrap();
        let resultat = physarum(&["lancer", s(&scenario), "--graine", graine, "--sortie", s(&sortie)]);
        let erreur = String::from_utf8_lossy(&resultat.stderr).into_owned();
        assert_eq!(resultat.status.code(), Some(1), "{texte:?} graine {graine} : code ; stderr : {erreur}");
        assert!(erreur.contains(attendu), "{texte:?} graine {graine} : attendu {attendu:?} dans : {erreur}");
        assert!(!erreur.contains("panicked"), "{texte:?} graine {graine} : panique : {erreur}");
        assert!(noms(&sortie).is_empty(), "{texte:?} graine {graine} : un fichier a été écrit");
        let absent = dossier.join("absent").join("sortie");
        let resultat = physarum(&["lancer", s(&scenario), "--graine", graine, "--sortie", s(&absent)]);
        assert_eq!(resultat.status.code(), Some(1), "{texte:?} graine {graine} : code vers un dossier absent");
        assert!(!dossier.join("absent").exists(), "{texte:?} : le dossier de sortie a été créé");
        let _ = fs::remove_dir_all(&dossier);
    }
    // arrêté avant le pas fautif : tout va bien
    let avant = grand.replace("pas 40 0.5", "pas 2 0.5");
    let (dossier, scenario) = espace("tubes_arret_avant", &avant);
    let sortie = dossier.join("sortie");
    let resultat = physarum(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&sortie)]);
    assert_eq!(resultat.status.code(), Some(0), "stderr : {}", String::from_utf8_lossy(&resultat.stderr));
    assert_eq!(noms(&sortie), ["apercu.png", "empreinte.txt", "mesures.csv"]);
    assert_eq!(fs::read_to_string(sortie.join("mesures.csv")).unwrap().lines().count(), 3);
    let _ = fs::remove_dir_all(&dossier);
}
