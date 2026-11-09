//! Étape 0 : le binaire `physarum` quand une écriture échoue à mi-chemin. Valeurs attendues :
//! `references/etape-0-valeurs.md`, identifiant `V-BIN-08`.
//!
//! Ce fichier complète `etape_0_binaire.rs`, gelé avant lui : la campagne de fautes du juge a montré qu'aucun test ne
//! faisait échouer une écriture, et que le binaire laissait alors derrière lui ce qu'il venait d'écrire, contre sa doc
//! (« toute erreur : rien d'écrit ni de créé »). Pour faire échouer une écriture sans toucher aux droits, le dossier de
//! sortie contient d'avance un **dossier** qui porte le nom d'un des trois fichiers : on ne peut pas écrire un fichier
//! par-dessus un dossier, sous Linux comme sous Windows.

mod common;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn physarum(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_physarum")).args(args).output().expect("lancement de physarum")
}

fn s(chemin: &Path) -> &str {
    chemin.to_str().expect("chemin en UTF-8")
}

fn noms(dossier: &Path) -> Vec<String> {
    let mut noms: Vec<String> =
        fs::read_dir(dossier).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    noms.sort();
    noms
}

/// juste : quand un des trois fichiers ne peut pas être écrit, le binaire rend le code 1, un message qui n'est pas une
/// panique, et ne laisse dans le dossier de sortie **aucun** des fichiers qu'il a écrits pendant ce lancer : il n'y reste que
/// ce qui s'y trouvait avant (V-BIN-08, la doc de `src/main.rs` : « toute erreur […] rien d'écrit ni de créé » ; « un fichier
/// impossible à écrire »). Essayé pour chacun des trois noms, quel que soit l'ordre dans lequel le binaire écrit ; le dossier
/// qui bloque est intact ; le même lancer vers un dossier vide réussit et écrit les trois fichiers.
///
/// Ne couvre pas : une panne en cours d'écriture d'un fichier (disque plein), ni le retrait d'un dossier de sortie que le
/// binaire aurait créé lui-même avant l'échec.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn echec_d_ecriture_ne_laisse_rien() {
    for bloque in ["empreinte.txt", "mesures.csv", "apercu.png"] {
        let dossier = common::dossier_temporaire(&format!("ecriture_{bloque}"));
        fs::create_dir_all(&dossier).unwrap();
        let scenario = dossier.join("scenario.txt");
        fs::write(&scenario, "grille 3 2\n").unwrap();
        let sortie = dossier.join("sortie");
        // un dossier à la place du fichier, avec un témoin dedans : l'écriture de ce fichier échoue
        fs::create_dir_all(sortie.join(bloque)).unwrap();
        fs::write(sortie.join(bloque).join("temoin"), "intact").unwrap();
        let resultat = physarum(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&sortie)]);
        let erreur = String::from_utf8_lossy(&resultat.stderr).into_owned();
        assert_eq!(resultat.status.code(), Some(1), "{bloque} : code de sortie ; stderr : {erreur}");
        assert!(!erreur.trim().is_empty(), "{bloque} : le message d'erreur est vide");
        assert!(!erreur.contains("panicked"), "{bloque} : le binaire a paniqué : {erreur}");
        assert_eq!(noms(&sortie), [bloque], "{bloque} : un fichier écrit pendant ce lancer est resté");
        assert_eq!(fs::read_to_string(sortie.join(bloque).join("temoin")).unwrap(), "intact", "{bloque}");
        // sans le dossier qui bloque, le même lancer réussit
        let libre = dossier.join("libre");
        let resultat = physarum(&["lancer", s(&scenario), "--graine", "1", "--sortie", s(&libre)]);
        assert_eq!(resultat.status.code(), Some(0), "{}", String::from_utf8_lossy(&resultat.stderr));
        assert_eq!(noms(&libre), ["apercu.png", "empreinte.txt", "mesures.csv"]);
        let _ = fs::remove_dir_all(&dossier);
    }
}
