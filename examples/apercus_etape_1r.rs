//! Étape 1, partie 1R, les aperçus : un réseau entre six nourritures à trois débits.
//!
//! Le scénario `scenarios/reseau.txt` est lu par `scenario::lire_scenario` et lancé par `lancer::lancer` avec la
//! graine 1, tel quel (`debit 3`) puis avec la ligne `debit 3` remplacée par `debit 1` et par `debit 8`. L'aperçu de
//! chacun (le champ `apercu`, déjà un PNG) est écrit dans `apercus/`. Un aperçu est pour l'œil, jamais une preuve.

use std::fs;
use std::path::PathBuf;

use physarum_lab::lancer::lancer;
use physarum_lab::scenario::lire_scenario;

fn main() {
    let racine = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dossier = racine.join("apercus");
    fs::create_dir_all(&dossier).expect("le dossier apercus/ se crée");

    let chemin_reseau = racine.join("scenarios").join("reseau.txt");
    let reseau = fs::read_to_string(&chemin_reseau)
        .unwrap_or_else(|erreur| panic!("lecture de {} : {erreur}", chemin_reseau.display()));
    assert!(reseau.contains("debit 3"), "le réseau doit avoir la ligne « debit 3 »");

    let cas: [(String, &str); 3] = [
        (reseau.replace("debit 3", "debit 1"), "etape-1r-reseau-debit-1.png"),
        (reseau.clone(), "etape-1r-reseau-debit-3.png"),
        (reseau.replace("debit 3", "debit 8"), "etape-1r-reseau-debit-8.png"),
    ];

    for (texte, fichier) in cas {
        let scenario =
            lire_scenario(&texte).unwrap_or_else(|erreur| panic!("scénario {fichier:?} invalide : {erreur:?}"));
        let sortie = lancer(&scenario, 1).unwrap_or_else(|erreur| panic!("scénario {fichier:?} refusé : {erreur:?}"));
        let chemin = dossier.join(fichier);
        fs::write(&chemin, &sortie.apercu)
            .unwrap_or_else(|erreur| panic!("écriture de {} : {erreur}", chemin.display()));
        println!("{} : {} octets", chemin.display(), sortie.apercu.len());
    }
}
