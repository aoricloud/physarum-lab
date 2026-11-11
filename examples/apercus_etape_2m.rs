//! Étape 2, partie 2M, les aperçus : la mémoire dans la forme. Le scénario `scenarios/souvenir.txt` (grille de 15 × 11,
//! deux nourritures, un stimulus d'agent au-dessus du milieu sur les cent premiers pas) joué 100 puis 400 pas, et le
//! même sous la loi linéaire. Les images sont écrites dans `apercus/`. Un aperçu est pour l'œil, jamais une preuve.

use std::fs;
use std::path::PathBuf;

use physarum_lab::lancer::lancer;
use physarum_lab::scenario::lire_scenario;

fn main() {
    let racine = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dossier = racine.join("apercus");
    fs::create_dir_all(&dossier).expect("le dossier apercus/ se crée");
    let souvenir = fs::read_to_string(racine.join("scenarios/souvenir.txt")).expect("scenarios/souvenir.txt");
    let lineaire =
        souvenir.replace("loi sigmoide", "loi lineaire").replace("debit 3", "debit 1").replace("8.0 1.0", "4.0 1.0");
    for (texte, pas, fichier) in [
        (&souvenir, 100, "etape-2m-souvenir-100.png"),
        (&souvenir, 400, "etape-2m-souvenir-400.png"),
        (&lineaire, 400, "etape-2m-lineaire-400.png"),
    ] {
        let texte = texte.replace("pas 400 0.1", &format!("pas {pas} 0.1"));
        let scenario = lire_scenario(&texte).expect("scénario valide");
        let sortie = lancer(&scenario, 1).expect("le scénario se lance");
        fs::write(dossier.join(fichier), sortie.apercu).expect("l'image s'écrit");
    }
}
