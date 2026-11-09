//! Étape 0 : `scenario`, la lecture d'un fichier de scénario. Valeurs attendues : `references/etape-0-valeurs.md`,
//! identifiants `V-SCEN-xx`.
//!
//! Ici la source est la définition du format écrite dans la documentation de `scenario` (une convention de l'étape 0, pas
//! une valeur publiée) ; les cas sont des textes écrits à la main dont le résultat se lit sans calcul. Pour « directive
//! inconnue », ces tests n'emploient que `inconnue 1 2` et d'autres noms qu'aucune étape suivante ne réclamera (`x_1`,
//! `autre`, `sans_argument`) : jamais `source`, `puits`, `nourriture`, `mur`, `pas`, `stimulus`, `pont`, `sel`…

use physarum_lab::graphe::{Grille, Voisinage};
use physarum_lab::scenario::{COTE_MAX, Directive, ETAGES_MAX, ErreurLecture, Scenario, lire_scenario};

fn lire(texte: &str) -> Result<Scenario, ErreurLecture> {
    lire_scenario(texte)
}

fn invalide(texte: &str) -> ErreurLecture {
    lire(texte).expect_err(&format!("{texte:?} devrait être refusé"))
}

fn directive(ligne: usize, nom: &str, args: &[&str]) -> Directive {
    Directive { ligne, nom: nom.to_string(), args: args.iter().map(|a| a.to_string()).collect() }
}

/// La grille par défaut : un étage, voisinage 4, lien vertical `1.0` et `1.0`.
fn plate(largeur: u32, hauteur: u32) -> Grille {
    Grille { largeur, hauteur, etages: 1, voisinage: Voisinage::Quatre, longueur_verticale: 1.0, cout_vertical: 1.0 }
}

/// juste : `grille 64 48` seule donne la grille par défaut : largeur puis hauteur, un étage, voisinage 4, `1.0` et `1.0`
/// pour le lien vertical ; avec `voisinage 8` et `etages 2 0.5 3`, dans n'importe quel ordre, la grille complète
/// (V-SCEN-01) ; les bornes publiques valent 256 et 8.
///
/// Ne couvre pas : les erreurs (autres tests).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn la_grille_et_ses_defauts() {
    let s = lire("grille 64 48\n").unwrap();
    assert_eq!(s.grille, plate(64, 48));
    assert!(s.directives.is_empty());
    assert_eq!(lire("grille 5 9").unwrap().grille, plate(5, 9), "sans saut de ligne final");
    let complete = Grille {
        largeur: 24,
        hauteur: 16,
        etages: 2,
        voisinage: Voisinage::Huit,
        longueur_verticale: 0.5,
        cout_vertical: 3.0,
    };
    for texte in [
        "grille 24 16\nvoisinage 8\netages 2 0.5 3\n",
        "etages 2 0.5 3\ngrille 24 16\nvoisinage 8\n",
        "voisinage 8\netages 2 0.5 3\ngrille 24 16\n",
        "# en tête\n\nvoisinage 8\n# au milieu\netages 02 0.50 3.0\n  grille \t 24   16  \n",
    ] {
        let s = lire(texte).unwrap();
        assert_eq!(s.grille, complete, "{texte:?}");
        assert!(s.directives.is_empty(), "{texte:?}");
    }
    assert_eq!(lire("grille 3 2\nvoisinage 8\n").unwrap().grille, Grille { voisinage: Voisinage::Huit, ..plate(3, 2) });
    assert_eq!(lire("voisinage 4\ngrille 3 2\n").unwrap().grille, plate(3, 2));
    assert_eq!(
        lire("grille 3 2\netages 3 0.25 7\n").unwrap().grille,
        Grille { etages: 3, longueur_verticale: 0.25, cout_vertical: 7.0, ..plate(3, 2) }
    );
    assert_eq!((COTE_MAX, ETAGES_MAX), (256, 8));
    // la grille d'un scénario lu est toujours valide
    assert!(lire("grille 256 1\netages 8 0.001 1000\n").unwrap().grille.graphe().is_ok());
}

/// juste : sont ignorées les lignes vides, celles qui ne contiennent que des espaces ou des tabulations, celles dont le
/// premier caractère non blanc est `#` (même indenté, même `#` seul) ; les champs peuvent être séparés par plusieurs
/// espaces ou des tabulations ; les blancs de bord et un seul `\r` final (fin de ligne Windows) sont retirés ; l'espace et
/// la tabulation sont les seuls blancs : un saut de page ou une espace insécable font partie d'un mot (V-SCEN-02).
///
/// Ne couvre pas : les autres caractères blancs d'Unicode que le saut de page et l'espace insécable.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn commentaires_blancs_et_fins_de_ligne() {
    let texte = "# un commentaire\n\n   \n\t\n  # indenté\n#\n   grille \t 8   6  \n# fin";
    let s = lire(texte).unwrap();
    assert_eq!(s.grille, plate(8, 6));
    assert!(s.directives.is_empty());
    let s = lire("# c\r\ngrille 3 4\r\nvoisinage 8\r\n\r\n").unwrap();
    assert_eq!(s.grille, Grille { voisinage: Voisinage::Huit, ..plate(3, 4) });
    assert!(s.directives.is_empty());
    // un seul `\r` final est retiré : un second reste dans le dernier mot, qui n'est plus un entier
    assert_eq!(invalide("grille 3 4\r\r\n"), ErreurLecture::LigneInvalide(1));
    assert_eq!(lire("grille 3 4\nx a\rb\r\n").unwrap().directives, [directive(2, "x", &["a\rb"])]);
    // seuls l'espace et la tabulation sont des blancs : un saut de page ou une espace insécable font partie d'un mot
    assert_eq!(invalide("\u{0C}grille 3 4\n"), ErreurLecture::LigneInvalide(1));
    assert_eq!(invalide("grille 3 4\n\u{A0}\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("grille 3\u{0C}4\n"), ErreurLecture::LigneInvalide(1));
    assert_eq!(invalide("grille\u{A0}3 4\n"), ErreurLecture::LigneInvalide(1));
    assert_eq!(lire("grille 3 4\nx a\u{A0}b\u{0C}c\n").unwrap().directives, [directive(2, "x", &["a\u{A0}b\u{0C}c"])]);
}

/// juste : les bornes de `grille` : acceptés `grille 1 1`, `grille 256 256`, `grille 256 1`, `grille 1 256` ; refusés
/// (ligne 1) un côté nul, `257`, et un entier qui ne tient pas dans un `u32` ; les zéros de tête sont permis (`064` vaut
/// 64) (V-SCEN-03).
///
/// Ne couvre pas : les autres entiers hors bornes que ceux-là.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn bornes_de_la_grille() {
    for (texte, largeur, hauteur) in
        [("grille 1 1", 1, 1), ("grille 256 256", 256, 256), ("grille 256 1", 256, 1), ("grille 1 256", 1, 256)]
    {
        assert_eq!(lire(texte).unwrap().grille, plate(largeur, hauteur), "{texte}");
    }
    for texte in [
        "grille 0 5",
        "grille 5 0",
        "grille 0 0",
        "grille 257 1",
        "grille 1 257",
        "grille 4294967297 1",
        "grille 1 99999999999999999999",
        "grille 4294967552 4294967552",
        "grille 18446744073709551621 3",
        "grille 3 18446744073709551872",
    ] {
        assert_eq!(invalide(texte), ErreurLecture::LigneInvalide(1), "{texte}");
    }
    assert_eq!(lire("grille 064 048").unwrap().grille, plate(64, 48));
}

/// juste : un entier est écrit en chiffres décimaux ASCII seulement : sont refusés (ligne 1) `+5`, `-5`, `5.0`, `0x10`,
/// `1e2`, `5_0`, `5a`, des chiffres non ASCII (`٣`), l'absence d'argument ou un argument en trop, et un `#` de fin de
/// ligne (V-SCEN-04) ; les mêmes directives bien écrites se lisent (un code qui refuse tout échoue).
///
/// Ne couvre pas : `voisinage` et `etages` (test suivant).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn entiers_et_nombre_d_arguments() {
    for texte in [
        "grille +5 4",
        "grille 5 +4",
        "grille -5 4",
        "grille 5.0 4",
        "grille 0x10 4",
        "grille 1e2 4",
        "grille 5_0 4",
        "grille 5a 4",
        "grille ٣ 4",
        "grille",
        "grille 3",
        "grille 3 4 5",
        "grille 3 3 # une note",
    ] {
        assert_eq!(invalide(texte), ErreurLecture::LigneInvalide(1), "{texte}");
    }
    assert_eq!(lire("grille 5 4").unwrap().grille, plate(5, 4));
    assert_eq!(lire("grille 3 4").unwrap().grille, plate(3, 4));
}

/// juste : `voisinage` n'accepte que `4` ou `8`, écrits exactement ainsi ; `etages` veut un entier de 1 à 8 puis deux
/// flottants décimaux finis et strictement positifs (V-SCEN-05, la doc). Chaque ligne fautive est la ligne 2 ; la même
/// ligne bien écrite se lit.
///
/// Ne couvre pas : la notation scientifique, volontairement refusée (voir `directive_flottant`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn voisinage_et_etages() {
    for ligne in [
        "voisinage",
        "voisinage 6",
        "voisinage 04",
        "voisinage 08",
        "voisinage 4 8",
        "voisinage huit",
        "voisinage +8",
        "voisinage 8.0",
        "etages",
        "etages 2",
        "etages 2 1.0",
        "etages 2 1.0 1.0 1.0",
        "etages 0 1.0 1.0",
        "etages 9 1.0 1.0",
        "etages 2.0 1.0 1.0",
        "etages -2 1.0 1.0",
        "etages 2 0 1.0",
        "etages 2 0.0 1.0",
        "etages 2 -0.0 1.0",
        "etages 2 -1.5 1.0",
        "etages 2 1.0 0",
        "etages 2 1.0 -3",
        "etages 2 1e0 1.0",
        "etages 2 1.0 +1.0",
        "etages 2 .5 1.0",
        "etages 2 1. 1.0",
        "etages 2 inf 1.0",
        "etages 2 1.0 nan",
        "etages 2 1,5 1.0",
        "etages 4294967298 1.0 1.0",
        "etages 18446744073709551618 1.0 1.0",
    ] {
        assert_eq!(invalide(&format!("grille 3 3\n{ligne}\n")), ErreurLecture::LigneInvalide(2), "{ligne}");
    }
    let enorme = "9".repeat(400);
    assert_eq!(invalide(&format!("grille 3 3\netages 2 {enorme} 1\n")), ErreurLecture::LigneInvalide(2));
    for (ligne, etages, longueur, cout) in [
        ("etages 1 1.0 1.0", 1, 1.0, 1.0),
        ("etages 8 1 1", 8, 1.0, 1.0),
        ("etages 2 0.1 123456789.123456789", 2, 0.1, 123_456_789.123_456_79),
        ("etages 03 007.25 0.000001", 3, 7.25, 0.000_001),
    ] {
        let s = lire(&format!("grille 3 3\n{ligne}\n")).unwrap();
        assert_eq!(
            s.grille,
            Grille { etages, longueur_verticale: longueur, cout_vertical: cout, ..plate(3, 3) },
            "{ligne}"
        );
    }
    assert_eq!(
        lire("grille 3 3\netages 2 0.1 123456789.123456789\n").unwrap().grille.cout_vertical.to_bits(),
        0x419D_6F34_547E_6B75
    );
}

/// juste : chacune des trois directives ne figure qu'une fois : une seconde `grille`, un second `voisinage`, un second
/// `etages` est la ligne fautive, même si ses arguments sont bons et même identiques ; le numéro est celui de la seconde,
/// lignes ignorées comprises (V-SCEN-06).
///
/// Ne couvre pas : trois occurrences ou plus (la deuxième suffit à échouer).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn directive_en_double() {
    assert_eq!(invalide("grille 3 3\ngrille 4 4\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("grille 3 3\ngrille 3 3\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("grille 3 3\nvoisinage 8\nvoisinage 8\n"), ErreurLecture::LigneInvalide(3));
    assert_eq!(invalide("voisinage 4\ngrille 3 3\n# c\n\nvoisinage 8\n"), ErreurLecture::LigneInvalide(5));
    assert_eq!(invalide("etages 2 1 1\ngrille 3 3\netages 2 1 1\n"), ErreurLecture::LigneInvalide(3));
    assert_eq!(invalide("etages 2 1 1\netages 3 1 1\n"), ErreurLecture::LigneInvalide(2), "avant même `Manquante`");
}

/// juste : le nom d'une directive est fait de minuscules ASCII, de chiffres et de `_`, et n'est pas vide : sont mal formés
/// (ligne fautive 2) `Grille`, `GRILLE`, `grille-2`, `a.b`, `é`, `x:1` ; sont bien formés `9x`, `_`, `a_1b2` (conservés comme
/// directives) (V-SCEN-07).
///
/// Ne couvre pas : les noms très longs.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn noms_de_directive() {
    for nom in ["Grille", "GRILLE", "grille-2", "a.b", "é", "x:1", "inconnue!", "Voisinage", "Etages"] {
        let texte = format!("grille 2 2\n{nom} 3 3\n");
        assert_eq!(invalide(&texte), ErreurLecture::LigneInvalide(2), "{nom}");
    }
    let s = lire("grille 2 2\n9x 1\n_\na_1b2 z\n").unwrap();
    assert_eq!(s.directives, [directive(2, "9x", &["1"]), directive(3, "_", &[]), directive(4, "a_1b2", &["z"])]);
}

/// juste : le numéro de ligne d'une erreur compte les lignes ignorées (vides, blanches, commentaires), à partir de 1 ; la
/// première ligne fautive l'emporte sur les suivantes et sur `Manquante` (V-SCEN-08).
///
/// Ne couvre pas : un fichier de plus de 100 lignes.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn numero_de_ligne() {
    assert_eq!(invalide("# a\n\n   \ngrille 3\n"), ErreurLecture::LigneInvalide(4));
    assert_eq!(invalide("grille 2 2\n\n\nBad 1\n"), ErreurLecture::LigneInvalide(4));
    assert_eq!(invalide("Bad\ngrille 0 0\n"), ErreurLecture::LigneInvalide(1));
    assert_eq!(invalide("inconnue 1 2\nBad\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("grille 2 2\nvoisinage 5\ngrille 9 9\n"), ErreurLecture::LigneInvalide(2));
    assert_eq!(invalide("\n\n\n\n\netages 9 1 1\n"), ErreurLecture::LigneInvalide(6));
    assert_eq!(invalide("\r\n# x\r\ngrille 3\r\n"), ErreurLecture::LigneInvalide(3));
    assert_eq!(
        invalide("voisinage 7\n"),
        ErreurLecture::LigneInvalide(1),
        "une ligne fautive l'emporte sur `Manquante`"
    );
}

/// juste : sans `grille`, l'erreur est `Manquante("grille")` : texte vide, que des blancs, que des commentaires, que
/// d'autres directives, même `voisinage` et `etages` bien écrits (V-SCEN-09) ; les mêmes textes suivis d'une `grille` se
/// lisent (un code qui rend toujours `Manquante` échoue).
///
/// Ne couvre pas : l'ordre entre `Manquante` et une ligne fautive (voir `numero_de_ligne`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn grille_manquante() {
    for texte in [
        "",
        "\n\n",
        "   \n\t\n",
        "# rien\n\n",
        "inconnue 1 2\n",
        "# c\nx_1\nautre a b\n",
        "voisinage 8\n",
        "etages 2 1 1\nvoisinage 4\n",
    ] {
        assert_eq!(invalide(texte), ErreurLecture::Manquante("grille"), "{texte:?}");
        let avec_grille = format!("{texte}\ngrille 7 1\n");
        let lu = lire(&avec_grille).map(|s| (s.grille.largeur, s.grille.hauteur));
        assert_eq!(lu, Ok((7, 1)), "{avec_grille:?}");
    }
}

/// juste : toute autre directive bien formée est gardée sans être interprétée, dans l'ordre du fichier, avec son numéro de
/// ligne (lignes ignorées comptées), ses arguments tels qu'écrits (séparés par des blancs, zéro permis), les doublons
/// conservés ; la grille peut venir après des directives ; `directive(nom)` rend la première de ce nom ou `None`, et jamais
/// `grille`, `voisinage` ni `etages` (V-SCEN-10).
///
/// Ne couvre pas : l'interprétation d'une directive (c'est le rôle de `lancer`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn directives_gardees_dans_l_ordre() {
    let texte =
        "# scénario\ninconnue 1 2\ngrille 8 6\n\nx_1\n  autre   a  b\tc\ninconnue 3\nvoisinage 8\nsans_argument\n";
    let s = lire(texte).unwrap();
    assert_eq!(s.grille, Grille { voisinage: Voisinage::Huit, ..plate(8, 6) });
    assert_eq!(
        s.directives,
        [
            directive(2, "inconnue", &["1", "2"]),
            directive(5, "x_1", &[]),
            directive(6, "autre", &["a", "b", "c"]),
            directive(7, "inconnue", &["3"]),
            directive(9, "sans_argument", &[]),
        ]
    );
    assert_eq!(s.directive("inconnue"), Some(&directive(2, "inconnue", &["1", "2"])));
    assert_eq!(s.directive("autre").unwrap().ligne, 6);
    assert!(s.directive("x_1").unwrap().args.is_empty());
    assert_eq!(s.directive("absent"), None);
    assert_eq!(s.directive("inconnu"), None, "le nom entier, pas un début de nom");
    assert_eq!(s.directive(""), None);
    assert_eq!(s.directive("x"), None);
    assert_eq!(s.directive("grille"), None);
    assert_eq!(s.directive("voisinage"), None);
    assert_eq!(s.directive("etages"), None);
    let s = lire("x_1 a\ngrille 4 4\nx_1 b\n").unwrap();
    assert_eq!(s.grille, plate(4, 4));
    assert_eq!(s.directives, [directive(1, "x_1", &["a"]), directive(3, "x_1", &["b"])]);
    assert_eq!(s.directive("x_1").unwrap().args, ["a"]);
}

/// juste : un `#` en fin de ligne n'est pas un commentaire : `x 1 # note` est une directive à trois arguments `1`, `#`,
/// `note` (V-SCEN-11).
///
/// Ne couvre pas : les commentaires en début de ligne (autre test).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn diese_en_fin_de_ligne() {
    let s = lire("grille 2 2\nx 1 # note\n").unwrap();
    assert_eq!(s.directives, [directive(2, "x", &["1", "#", "note"])]);
}

/// juste : `Directive::entier(i)` lit l'argument `i` en chiffres décimaux ASCII seulement : `12`, `0`, `u64::MAX`, `007` (7) ;
/// l'erreur est `LigneInvalide(ligne de la directive)` pour `u64::MAX + 1`, `+3`, `-1`, `abc`, `1.5`, et pour un argument
/// absent (V-SCEN-12).
///
/// Ne couvre pas : les arguments non UTF-8 (un texte Rust est UTF-8).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn directive_entier() {
    let s = lire("grille 1 1\n\nx 12 0 18446744073709551615 18446744073709551616 +3 -1 abc 1.5 007\n").unwrap();
    let d = s.directive("x").unwrap();
    assert_eq!(d.ligne, 3);
    assert_eq!(d.entier(0), Ok(12));
    assert_eq!(d.entier(1), Ok(0));
    assert_eq!(d.entier(2), Ok(u64::MAX));
    assert_eq!(d.entier(8), Ok(7));
    for i in [3, 4, 5, 6, 7, 9, 10, 1000] {
        assert_eq!(d.entier(i), Err(ErreurLecture::LigneInvalide(3)), "argument {i}");
    }
}

/// juste : `Directive::flottant(i)` lit `-?chiffres(.chiffres)?` : `0.5`, `-2`, `0.1` (bits `0x3FB999999999999A`), `-0.0` (zéro
/// négatif), `007.25`, `123456789.123456789` (bits `0x419D6F34547E6B75`), `-3.14159265358979323846` (bits `0xC00921FB54442D18`) ;
/// l'erreur est `LigneInvalide(ligne)` pour `1e3`, `+1`, `.5`, `5.`, `inf`, `nan`, `1.2.3`, `-`, `--1`, `12abc`, `1,5`, un nombre
/// de quatre cents chiffres (infini à l'arrondi), et un argument absent (V-SCEN-13 ; arrondis au plus proche, valeurs de
/// Python `float`).
///
/// Ne couvre pas : la notation scientifique, volontairement refusée.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn directive_flottant() {
    let enormes = "9".repeat(400);
    let texte = format!(
        "grille 1 1\nf 0.5 -2 0.1 -0.0 007.25 123456789.123456789 -3.14159265358979323846 1e3 +1 .5 5. inf nan 1.2.3 - --1 12abc 1,5 {enormes}\n"
    );
    let s = lire(&texte).unwrap();
    let d = s.directive("f").unwrap();
    assert_eq!(d.ligne, 2);
    assert_eq!(d.flottant(0).unwrap().to_bits(), 0.5_f64.to_bits());
    assert_eq!(d.flottant(1).unwrap().to_bits(), (-2.0_f64).to_bits());
    assert_eq!(d.flottant(2).unwrap().to_bits(), 0x3FB9_9999_9999_999A);
    assert_eq!(d.flottant(3).unwrap().to_bits(), 0x8000_0000_0000_0000);
    assert_eq!(d.flottant(4).unwrap().to_bits(), 0x401D_0000_0000_0000);
    assert_eq!(d.flottant(5).unwrap().to_bits(), 0x419D_6F34_547E_6B75);
    assert_eq!(d.flottant(6).unwrap().to_bits(), 0xC009_21FB_5444_2D18);
    for i in 7..=18 {
        assert_eq!(d.flottant(i), Err(ErreurLecture::LigneInvalide(2)), "argument {i} : {:?}", d.args[i]);
    }
    assert_eq!(d.flottant(19), Err(ErreurLecture::LigneInvalide(2)), "argument absent");
}

/// juste : le fichier d'exemple `scenarios/grille.txt` se lit : une grille de 24 sur 16, voisinage 8, deux étages reliés
/// par des liens de longueur `1.0` et de coût `3.0`, sans autre directive (V-SCEN-14, le contenu du fichier).
///
/// Ne couvre pas : les autres fichiers de `scenarios/`.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn scenario_d_exemple_du_depot() {
    let chemin = concat!(env!("CARGO_MANIFEST_DIR"), "/scenarios/grille.txt");
    let texte = std::fs::read_to_string(chemin).expect("scenarios/grille.txt");
    let s = lire(&texte).unwrap();
    assert_eq!(
        s.grille,
        Grille {
            largeur: 24,
            hauteur: 16,
            etages: 2,
            voisinage: Voisinage::Huit,
            longueur_verticale: 1.0,
            cout_vertical: 3.0
        }
    );
    assert!(s.directives.is_empty());
}
