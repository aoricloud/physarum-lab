//! Étape 1, partie 1L, les aperçus : le labyrinthe et les étages.
//!
//! Les scénarios de `scenarios/` sont lus par `scenario::lire_scenario` et lancés par `lancer::lancer` avec la
//! graine 1. Le labyrinthe est montré après 20 pas (`pas 500 0.5` remplacé par `pas 20 0.5`) puis tel quel, après
//! 500 pas ; les trois scénarios à étages sont lancés tels quels. L'aperçu de chacun (le champ `apercu`, déjà un
//! PNG) est écrit dans `apercus/`. Un aperçu est pour l'œil, jamais une preuve.

use std::fs;
use std::path::PathBuf;

use physarum_lab::lancer::lancer;
use physarum_lab::scenario::lire_scenario;

fn main() {
    let racine = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dossier = racine.join("apercus");
    fs::create_dir_all(&dossier).expect("le dossier apercus/ se crée");

    let lire = |nom: &str| {
        let chemin = racine.join("scenarios").join(nom);
        fs::read_to_string(&chemin).unwrap_or_else(|erreur| panic!("lecture de {} : {erreur}", chemin.display()))
    };

    let labyrinthe = lire("labyrinthe.txt");
    assert!(labyrinthe.contains("pas 500 0.5"), "le labyrinthe doit avoir la ligne « pas 500 0.5 »");
    let labyrinthe_20 = labyrinthe.replace("pas 500 0.5", "pas 20 0.5");

    let cas: [(String, &str); 5] = [
        (labyrinthe_20, "etape-1l-labyrinthe-20.png"),
        (labyrinthe, "etape-1l-labyrinthe-500.png"),
        (lire("etages.txt"), "etape-1l-etages.png"),
        (lire("etages-cher.txt"), "etape-1l-etages-cher.png"),
        (lire("etages-pont.txt"), "etape-1l-etages-pont.png"),
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
