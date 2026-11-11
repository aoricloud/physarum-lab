//! Étape 2, partie 2M : les tests ajoutés par le juge après la campagne de fautes. Chacun ferme une faiblesse que la
//! campagne a trouvée (une faute plausible que les autres tests laissaient passer). Valeurs attendues :
//! `references/etape-2m-valeurs.md`, identifiants `V-CAM-xx`. Les valeurs se calculent à la main d'après la doc de
//! `souvenir` et de `lancer` (écrites dans le commentaire de chaque test) ; aucune ne vient du prototype.

#[path = "common/graphes_au_hasard.rs"]
mod graphes_au_hasard;

use graphes_au_hasard::graphe_pondere;
use physarum_lab::lancer::{ErreurLancer, lancer};
use physarum_lab::reseaux::Loi;
use physarum_lab::scenario::lire_scenario;
use physarum_lab::souvenir::{Agent, Couplage, ErreurMemoire, Memoire, Transport};

fn erreur_de(texte: &str) -> Option<ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario lisible"), 1).err()
}

fn un_pas(
    memoire: &mut Memoire,
    graphe: &physarum_lab::graphe::Graphe,
    source: usize,
    puits: usize,
) -> Result<(), ErreurMemoire> {
    memoire
        .avancer(
            graphe,
            source,
            puits,
            1.0,
            Loi::Lineaire,
            &Couplage { force: 2.0, saturation: 1.0 },
            &Transport { diffusion: 0.1, perte: 0.1 },
            &[(1, 0.5)],
            0.25,
        )
        .map(|_| ())
}

/// **V-CAM-01** : `Memoire::avancer` sur un graphe qui n'est pas celui de la mémoire rend `Taille`, et rien ne change.
/// La mémoire a 4 nœuds et 4 arêtes ; les quatre autres graphes ont une arête de plus, une de moins, un nœud de plus
/// (mêmes arêtes), un nœud de moins : chaque moitié de la vérification (arêtes, nœuds) et chaque sens est éprouvé seul.
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn taille_d_un_autre_graphe() {
    let aretes = [(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0), (0, 3, 2.0)];
    let g = graphe_pondere(4, &aretes);
    let m = Memoire::new(&g, vec![0.9, 0.8, 0.7, 0.6], Agent::new(&g, vec![0.0, 1.0, 0.5, 0.0]).unwrap()).unwrap();
    let autres = [
        ("une arête de plus", graphe_pondere(4, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0), (0, 3, 2.0), (1, 3, 1.0)])),
        ("une arête de moins", graphe_pondere(4, &aretes[..3])),
        ("un nœud de plus", graphe_pondere(5, &aretes)),
        ("un nœud de moins", graphe_pondere(3, &[(0, 1, 1.0), (1, 2, 1.0), (0, 2, 1.0)])),
    ];
    for (nom, autre) in &autres {
        let mut essai = m.clone();
        assert_eq!(un_pas(&mut essai, autre, 0, 2), Err(ErreurMemoire::Taille), "{nom}");
        assert_eq!(essai, m, "{nom} : rien ne doit changer");
    }
    // le témoin : le même pas sur le graphe de la mémoire passe
    assert_eq!(un_pas(&mut m.clone(), &g, 0, 2), Ok(()));
}

/// **V-CAM-02** : une source ou un puits égal au nombre de nœuds (le premier indice hors du graphe) rend `Noeud`, et
/// rien ne change. Graphe de 4 nœuds : `(4, 0)`, `(0, 4)`, `(4, 3)`.
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn source_ou_puits_au_premier_indice_hors_du_graphe() {
    let g = graphe_pondere(4, &[(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0), (0, 3, 2.0)]);
    let m = Memoire::new(&g, vec![0.9, 0.8, 0.7, 0.6], Agent::zero(&g)).unwrap();
    for (source, puits) in [(4, 0), (0, 4), (4, 3)] {
        let mut essai = m.clone();
        assert_eq!(un_pas(&mut essai, &g, source, puits), Err(ErreurMemoire::Noeud), "source {source}, puits {puits}");
        assert_eq!(essai, m);
    }
}

/// **V-CAM-03** : l'ordre des erreurs de `lancer` pour des tubes : une source égale au puits (point 2 des tubes : la
/// ligne de la seconde des deux, ici `puits`, ligne 3) passe **avant** un `stimulus` sans `agent` (ajout 2 : la ligne
/// de la première directive restante, `source`, ligne 2).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn tubes_source_egale_au_puits_avant_un_stimulus_sans_agent() {
    let t = "grille 5 4\nsource 1 1 0\npuits 1 1 0\npas 5 0.5\nstimulus 2 2 0 1.0 0 3\n";
    assert_eq!(erreur_de(t), Some(ErreurLancer::DirectiveInconnue(3)));
}

/// **V-CAM-04** : pour un réseau, une `source` (point 2 des réseaux : la ligne de la première `source` ou `puits`,
/// ligne 4) passe **avant** un `stimulus` sans `agent` (ajout 2 : la première directive restante, ligne 2).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn reseau_source_avant_un_stimulus_sans_agent() {
    let t = "grille 5 4\nnourriture 0 0 0\nnourriture 4 3 0\nsource 1 1 0\ndebit 1.5\nloi lineaire\npas 5 0.5\n\
             stimulus 1 1 0 1.0 0 3\n";
    assert_eq!(erreur_de(t), Some(ErreurLancer::DirectiveInconnue(4)));
}

fn largeur_hauteur_png(png: &[u8]) -> (u32, u32) {
    let lire = |i: usize| u32::from_be_bytes([png[i], png[i + 1], png[i + 2], png[i + 3]]);
    (lire(16), lire(20))
}

/// **V-CAM-05** : l'échelle de l'aperçu avec agent : `512 / TL` ramenée entre 1 et 8, où `TL = 2 (xmax - xmin) + 1`
/// est la **largeur** du treillis et `th = 2 (ymax - ymin) + 1` sa hauteur (un étage). Le PNG mesure `TL × échelle`
/// sur `th × échelle` :
/// - grille 40 × 2 : `TL = 79`, `512 / 79 = 6`, `th = 3` : 474 × 18 ;
/// - grille 130 × 1 : `TL = 259`, `512 / 259 = 1`, `th = 1` : 259 × 1 ;
/// - grille 3 × 40 : `TL = 5`, `512 / 5 = 102` ramené à 8, `th = 79` : 40 × 632.
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn l_echelle_de_l_apercu_avec_agent() {
    for ((l, h), attendu) in [((40, 2), (474, 18)), ((130, 1), (259, 1)), ((3, 40), (40, 632))] {
        let t = format!(
            "grille {l} {h}\nsource 0 0 0\npuits {} {} 0\npas 1 0.5\nagent 1.0 1.0 0.1 0.1\nstimulus 0 0 0 1.0 0 1\n",
            l - 1,
            h - 1
        );
        let s = lancer(&lire_scenario(&t).unwrap(), 1).unwrap();
        assert_eq!(largeur_hauteur_png(&s.apercu), attendu, "grille {l} × {h}");
    }
}

/// **V-CAM-06** : un calcul qui s'arrête, **pour des tubes avec agent** : `grille 3 1`, `etages 2 1.0 10^16`, source
/// `(0, 0, 0)`, puits `(2, 0, 1)`, `h = 0.5`, `agent 0.0 …` (`α = 0`) : le pas couplé est alors celui de
/// `Tubes::avancer` (V-LOI-01) sur les mêmes conductivités de départ, donc il s'arrête au même pas que le même scénario
/// sans agent : le pas numéro 5 (mesuré à part avec le binaire de l'étape 1, sans agent). Cinq pas passent ; avec six
/// ou sept, `Calcul(5)`.
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn un_calcul_qui_s_arrete_pour_des_tubes_avec_agent() {
    let texte = |pas: u32, agent: &str| {
        format!("grille 3 1\netages 2 1.0 10000000000000000\nsource 0 0 0\npuits 2 0 1\npas {pas} 0.5\n{agent}")
    };
    for agent in ["", "agent 0.0 1.0 0.1 0.1\n"] {
        assert!(erreur_de(&texte(5, agent)).is_none(), "cinq pas passent ({agent:?})");
        assert_eq!(erreur_de(&texte(6, agent)), Some(ErreurLancer::Calcul(5)), "{agent:?}");
        assert_eq!(erreur_de(&texte(7, agent)), Some(ErreurLancer::Calcul(5)), "{agent:?}");
    }
}

/// **V-CAM-07** : les lignes des erreurs du plan avec agent (point 4 des tubes et des réseaux, inchangé) : un puits
/// muré rend la ligne de `puits` (4) ; pour un réseau, la 2e nourriture murée rend la sienne (4), la 1re la sienne (3).
#[test]
#[cfg_attr(not(feature = "etape-2m"), ignore)]
fn les_lignes_des_erreurs_du_plan_avec_agent() {
    let agent = "agent 1.0 1.0 0.1 0.1\n";
    let tubes = format!("grille 5 4\nmur 4 3 4 3 0\nsource 0 0 0\npuits 4 3 0\npas 2 0.5\n{agent}");
    assert_eq!(erreur_de(&tubes), Some(ErreurLancer::DirectiveInconnue(4)), "puits muré");
    let reseau = |mur: &str| {
        format!("grille 5 4\n{mur}\nnourriture 0 0 0\nnourriture 4 3 0\ndebit 1.5\nloi lineaire\npas 2 0.5\n{agent}")
    };
    assert_eq!(erreur_de(&reseau("mur 4 3 4 3 0")), Some(ErreurLancer::DirectiveInconnue(4)), "2e nourriture murée");
    assert_eq!(erreur_de(&reseau("mur 0 0 0 0 0")), Some(ErreurLancer::DirectiveInconnue(3)), "1re nourriture murée");
}
