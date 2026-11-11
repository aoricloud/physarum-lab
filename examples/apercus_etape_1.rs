//! Étape 1, les aperçus : le réseau qui se resserre sur le chemin.
//!
//! Trois scénarios sont écrits ici, lus par `scenario::lire_scenario` et lancés par `lancer::lancer` avec la
//! graine 1. Ils ne diffèrent que par le nombre de pas : le blob est partout après 3 pas, se resserre autour de la
//! diagonale après 40, et il ne reste que le plus court chemin de la source au puits après 600. L'aperçu de chacun
//! (le champ `apercu`, déjà un PNG) est écrit dans `apercus/`. Un aperçu est pour l'œil, jamais une preuve.

use std::fs;
use std::path::PathBuf;

use physarum_lab::lancer::lancer;
use physarum_lab::scenario::lire_scenario;

fn main() {
    let dossier = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("apercus");
    fs::create_dir_all(&dossier).expect("le dossier apercus/ se crée");

    let cas: [(&str, &str); 3] = [
        ("grille 16 16\nvoisinage 8\nsource 1 1 0\npuits 14 14 0\npas 3 0.5\n", "etape-1-chemin-3.png"),
        ("grille 16 16\nvoisinage 8\nsource 1 1 0\npuits 14 14 0\npas 40 0.5\n", "etape-1-chemin-40.png"),
        ("grille 16 16\nvoisinage 8\nsource 1 1 0\npuits 14 14 0\npas 600 0.5\n", "etape-1-chemin-600.png"),
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
