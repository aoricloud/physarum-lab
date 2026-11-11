//! Étape 1, partie 1R : les tests ajoutés par la campagne de fautes du juge (V-RCAM-01 à V-RCAM-06 dans la fiche), un
//! par faiblesse trouvée (une faute plausible qui passait les autres tests). Chacun passe sur le prototype juste et
//! échoue sur sa faute.

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Arete, Graphe, Noeud};
use physarum_lab::lancer::{ErreurLancer, lancer};
use physarum_lab::rendu::treillis;
use physarum_lab::reseaux::{ErreurReseau, Loi, Reseau};
use physarum_lab::scenario::lire_scenario;

fn erreur_de(texte: &str) -> Option<ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario lisible"), 1).err()
}

/// Une chaîne de `n` nœuds alignés, longueur et coût 1.
fn chaine(n: usize) -> Graphe {
    let noeuds = (0..n).map(|i| Noeud { x: i as i32, y: 0, etage: 0 }).collect();
    let aretes = (0..n - 1).map(|i| Arete { a: i, b: i + 1, longueur: 1.0, cout: 1.0 }).collect();
    Graphe::new(noeuds, aretes).expect("graphe valide")
}

/// juste : une `nourriture` qui redonne un nœud déjà donné plus haut, pas seulement juste avant, est fautive
/// (V-RCAM-01).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn une_nourriture_en_double_non_voisine() {
    let texte = concat!(
        "grille 6 4\nnourriture 0 0 0\nnourriture 5 3 0\nnourriture 0 0 0\n",
        "debit 2\nloi sigmoide\npas 2 0.5\n"
    );
    assert_eq!(erreur_de(texte), Some(ErreurLancer::DirectiveInconnue(4)));
    let texte = concat!(
        "grille 6 4\nnourriture 0 0 0\nnourriture 5 3 0\nnourriture 1 1 0\nnourriture 5 3 0\n",
        "debit 2\nloi sigmoide\npas 2 0.5\n"
    );
    assert_eq!(erreur_de(texte), Some(ErreurLancer::DirectiveInconnue(5)));
}

/// juste : seuls `lineaire` et `sigmoide` sont des noms de loi ; un nom voisin est fautif (V-RCAM-02).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn des_noms_de_loi_voisins_sont_refuses() {
    for mauvais in ["sigmoid", "sigmoides", "lineair", "linear", "lineaires"] {
        let texte = format!("grille 6 4\nnourriture 0 0 0\nnourriture 5 3 0\ndebit 2\nloi {mauvais}\npas 2 0.5\n");
        assert_eq!(erreur_de(&texte), Some(ErreurLancer::DirectiveInconnue(5)), "{mauvais:?}");
    }
}

/// juste : sur un treillis plus large que 512 points (deux étages de 150 colonnes), `512 / TL` vaut 0 et l'échelle est
/// ramenée à 1 : le lancer réussit et l'image a la largeur du treillis (V-RCAM-03).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn un_treillis_plus_large_que_512_a_l_echelle_1() {
    let texte = concat!(
        "grille 150 2\netages 2 1.0 1.0\nnourriture 0 0 0\nnourriture 149 1 1\n",
        "debit 1\nloi lineaire\npas 1 0.5\n"
    );
    let scenario = lire_scenario(texte).unwrap();
    let (largeur, hauteur) = treillis(&scenario.grille.graphe().unwrap());
    assert!(largeur > 512, "{largeur}");
    let sortie = lancer(&scenario, 1).expect("le lancer réussit");
    assert_eq!(&sortie.apercu[..8], &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]);
    let lu = |i: usize| {
        u32::from_be_bytes([sortie.apercu[i], sortie.apercu[i + 1], sortie.apercu[i + 2], sortie.apercu[i + 3]])
    };
    assert_eq!((lu(16), lu(20)), (largeur, hauteur), "l'en-tête IHDR : largeur et hauteur à l'échelle 1");
    // les mesures d'un réseau (l'en-tête des réseaux, une ligne) : le scénario a bien été lancé comme un réseau
    assert_eq!(sortie.mesures.lines().count(), 2);
    assert_eq!(sortie.mesures.lines().next(), Some("temps,source,puits,pression,vifs,longueur"));
    assert!(sortie.mesures.lines().nth(1).is_some_and(|l| l.starts_with("0,0,1,") || l.starts_with("0,1,0,")));
}

/// juste : `au_hasard` refuse un débit infini, `NaN`, nul ou négatif comme `new`, avant tout tirage (V-RCAM-04).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn au_hasard_refuse_un_debit_non_fini() {
    let g = chaine(4);
    for debit in [f64::INFINITY, f64::NAN, f64::NEG_INFINITY, 0.0, -1.0] {
        let mut alea = Alea::new(5);
        let avant = alea.octets();
        let r = Reseau::au_hasard(&g, &mut alea, vec![0, 3], debit, Loi::Sigmoide);
        assert_eq!(r.err(), Some(ErreurReseau::Debit), "{debit:?}");
        assert_eq!(alea.octets(), avant, "{debit:?} : alea n'a pas bougé");
    }
}

/// juste : l'ordre des nourritures est celui de la liste donnée (le rang), pas l'ordre des nœuds (V-RCAM-05).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn l_ordre_des_nourritures_est_celui_de_la_liste() {
    let g = chaine(5);
    let r = Reseau::new(&g, vec![0.5; 4], vec![3, 0, 4, 1], 1.0, Loi::Lineaire).unwrap();
    assert_eq!(r.nourritures(), &[3, 0, 4, 1]);
    let mut alea = Alea::new(1);
    let r = Reseau::au_hasard(&g, &mut alea, vec![4, 2, 0], 1.0, Loi::Sigmoide).unwrap();
    assert_eq!(r.nourritures(), &[4, 2, 0]);
}

/// juste : `Taille` aussi quand le graphe a plus d'arêtes que le réseau n'a de conductivités ; rien ne change, et
/// `alea` ne bouge pas (V-RCAM-06).
#[test]
#[cfg_attr(not(feature = "etape-1r"), ignore)]
fn avancer_paire_taille_dans_les_deux_sens() {
    let petit = chaine(3);
    let mut r = Reseau::new(&petit, vec![0.5, 0.5], vec![0, 2], 1.0, Loi::Sigmoide).unwrap();
    let grand = {
        let noeuds = (0..3).map(|i| Noeud { x: i, y: 0, etage: 0 }).collect();
        let aretes = vec![
            Arete { a: 0, b: 1, longueur: 1.0, cout: 1.0 },
            Arete { a: 1, b: 2, longueur: 1.0, cout: 1.0 },
            Arete { a: 0, b: 2, longueur: 2.0, cout: 2.0 },
        ];
        Graphe::new(noeuds, aretes).unwrap()
    };
    let avant = r.clone();
    assert_eq!(r.avancer_paire(&grand, 0, 2, 0.5).err(), Some(ErreurReseau::Taille));
    assert_eq!(r, avant);
    let mut alea = Alea::new(3);
    assert_eq!(r.avancer(&grand, &mut alea, 0.5).err(), Some(ErreurReseau::Taille));
    assert_eq!(r, avant);
    assert_eq!(alea.octets(), Alea::new(3).octets());
}
