//! Étape 0, l'aperçu : la grille seule, puis ses étages côte à côte.
//!
//! Deux scénarios sont écrits ici, lus par `scenario::lire_scenario` et lancés par `lancer::lancer` avec la graine 1.
//! L'aperçu de chacun (le champ `apercu`, déjà un PNG) est écrit dans `apercus/`. Un aperçu est pour l'œil, jamais une
//! preuve.

use std::fs;
use std::path::PathBuf;

use physarum_lab::lancer::lancer;
use physarum_lab::scenario::lire_scenario;

fn main() {
    let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus");
    fs::create_dir_all(&dossier).expect("le dossier apercus/ se crée");

    let cas: [(&str, &str); 2] = [
        ("grille 12 8\n", "etape-0-grille.png"),
        ("grille 12 8\nvoisinage 8\netages 2 1.0 3.0\n", "etape-0-deux-etages.png"),
    ];

    for (texte, fichier) in cas {
        let scenario =
            lire_scenario(texte).unwrap_or_else(|erreur| panic!("scénario {fichier:?} invalide : {erreur:?}"));
        let sortie = lancer(&scenario, 1).unwrap_or_else(|erreur| panic!("scénario {fichier:?} refusé : {erreur:?}"));
        let chemin = dossier.join(fichier);
        fs::write(&chemin, &sortie.apercu)
            .unwrap_or_else(|erreur| panic!("écriture de {} : {erreur}", chemin.display()));
        println!("{} : {} octets", chemin.display(), sortie.apercu.len());
    }
}
