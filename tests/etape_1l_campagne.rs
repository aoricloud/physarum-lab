//! Étape 1, partie 1L : tests ajoutés par la campagne de fautes (V-LCAM-01 et V-LCAM-02 dans la fiche). Chacun tue
//! au moins un survivant de la campagne ; chacun passe sur le prototype juste.

use physarum_lab::lancer::{ErreurLancer, lancer};
use physarum_lab::scenario::lire_scenario;

fn erreur_de(texte: &str) -> Option<ErreurLancer> {
    lancer(&lire_scenario(texte).expect("scénario lisible"), 1).err()
}

/// juste : un `cout_etage` fautif qui n'est PAS la première directive rend sa propre ligne (la règle (1) : « la ligne
/// de la première directive fautive »), pas celle de la première directive du fichier (V-LCAM-01). Dans
/// `erreurs_de_directives`, le `cout_etage` fautif est toujours la première directive restante (ligne 3) : rendre la
/// première ligne par erreur passait. Ici une directive valide le précède, pour chaque genre de faute.
///
/// Ne couvre pas : les fautes de `mur` (déjà dans `erreurs_de_directives`).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn un_cout_etage_fautif_apres_une_directive_valide_rend_sa_ligne() {
    // la ligne 1 est `grille`, la 2 `etages`, la 3 `source` (valide) : la faute est en ligne 4
    for mauvais in [
        "cout_etage 2 0.5\n",
        "cout_etage 1 0.09\n",
        "cout_etage 1 10.01\n",
        "cout_etage 1 1e1\n",
        "cout_etage 1\n",
        "cout_etage 1 0.5 0.5\n",
    ] {
        let texte = format!("grille 6 4\netages 2 1.0 1.0\nsource 0 0 0\n{mauvais}puits 5 3 0\npas 2 0.5\n");
        assert_eq!(erreur_de(&texte), Some(ErreurLancer::DirectiveInconnue(4)), "{mauvais:?}");
    }
    // un coût d'étage valide, puis un mur, puis un coût fautif : la ligne 5
    let texte = concat!(
        "grille 6 4\netages 2 1.0 1.0\ncout_etage 0 0.5\nmur 1 1 2 2 0\ncout_etage 9 0.5\n",
        "source 0 0 0\npuits 5 3 0\npas 2 0.5\n"
    );
    assert_eq!(erreur_de(texte), Some(ErreurLancer::DirectiveInconnue(5)));
}

fn empreinte_de(texte: &str) -> String {
    lancer(&lire_scenario(texte).expect("scénario lisible"), 1)
        .unwrap_or_else(|e| panic!("{texte:?} : {e:?}"))
        .empreinte
}

/// juste : les arguments entiers de `mur` et de `cout_etage` s'écrivent avec des zéros de tête (V-LCAM-02 ; la doc de
/// `lancer` : « des entiers (comme ci-dessus) », et `Directive::entier` : « les zéros de tête sont permis »).
/// `mur 01 01 02 02 00` est le mur `mur 1 1 2 2 0` : même empreinte ; `cout_etage 01 0.5` est `cout_etage 1 0.5`.
/// Aucun autre test de 1L n'écrit un entier de `mur` avec un zéro de tête.
///
/// Ne couvre pas : les zéros de tête de `source`, `puits` et `pas` (le cœur de l'étape 1).
#[test]
#[cfg_attr(not(feature = "etape-1l"), ignore)]
fn les_zeros_de_tete_des_entiers_de_mur_et_de_cout_etage() {
    let scenario =
        |milieu: &str| format!("grille 6 4\netages 2 1.0 1.0\n{milieu}source 0 0 0\npuits 5 3 0\npas 3 0.5\n");
    let nu = empreinte_de(&scenario("mur 1 1 2 2 0\ncout_etage 1 0.5\n"));
    let avec_zeros = empreinte_de(&scenario("mur 01 01 02 02 00\ncout_etage 01 0.5\n"));
    assert_eq!(nu, avec_zeros);
    // et le mur est bien là : sans lui, l'empreinte change
    assert_ne!(nu, empreinte_de(&scenario("cout_etage 1 0.5\n")));
    assert_eq!(erreur_de(&scenario("mur 0001 1 2 2 0\n")), None);
}
