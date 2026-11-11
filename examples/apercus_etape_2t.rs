//! Étape 2, partie 2T, les aperçus : la tache d'agent qui suit les tubes. Un réseau de 12 × 9 nœuds à huit voisins
//! converge (300 pas des tubes, source `(1, 4)`, puits `(10, 4)`), un agent est libéré en `(2, 4)` et porté par les
//! flux du réseau convergé ; les images montrent l'état aux pas 0, 15, 40 et 80 de l'agent (de durée `0,1`). Elles sont
//! écrites dans `apercus/`. Un aperçu est pour l'œil, jamais une preuve.

use std::fs;
use std::path::PathBuf;

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Grille, Voisinage};
use physarum_lab::image::ecrire_png;
use physarum_lab::souvenir::{Agent, Transport, apercu};
use physarum_lab::tubes::Tubes;

fn main() {
    let racine = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dossier = racine.join("apercus");
    fs::create_dir_all(&dossier).expect("le dossier apercus/ se crée");

    let grille = Grille {
        largeur: 12,
        hauteur: 9,
        etages: 1,
        voisinage: Voisinage::Huit,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let graphe = grille.graphe().expect("grille valide");
    let (source, puits, lacher) = (grille.indice(1, 4, 0), grille.indice(10, 4, 0), grille.indice(2, 4, 0));
    let mut alea = Alea::new(1);
    let mut tubes = Tubes::au_hasard(&graphe, &mut alea);
    let mut flux = vec![];
    for _ in 0..300 {
        flux = tubes.avancer(&graphe, source, puits, 1.0, 0.1).expect("pas des tubes").flux;
    }
    let mut concentrations = vec![0.0; graphe.nb_noeuds()];
    concentrations[lacher] = 1.0;
    let mut agent = Agent::new(&graphe, concentrations).expect("agent valide");
    let transport = Transport { diffusion: 0.01, perte: 0.0 };

    let mut pas_faits = 0;
    for (pas, fichier) in [
        (0, "etape-2t-tache-0.png"),
        (15, "etape-2t-tache-15.png"),
        (40, "etape-2t-tache-40.png"),
        (80, "etape-2t-tache-80.png"),
    ] {
        while pas_faits < pas {
            agent.avancer(&graphe, &flux, &transport, &[], 0.1).expect("pas de l'agent");
            pas_faits += 1;
        }
        let image = apercu(&graphe, tubes.conductivites(), agent.concentrations(), 1.0, 0.25, 8);
        let octets = ecrire_png(image.largeur, image.hauteur, &image.rvb);
        let chemin = dossier.join(fichier);
        fs::write(&chemin, &octets).unwrap_or_else(|erreur| panic!("écriture de {} : {erreur}", chemin.display()));
        println!("{} : {} octets", chemin.display(), octets.len());
    }
}
