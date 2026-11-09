//! Étape 0 : `lancer`, l'empreinte, les mesures et l'aperçu de la grille seule. Valeurs attendues :
//! `references/etape-0-valeurs.md`, identifiants `V-LANC-xx`.
//!
//! L'empreinte est vérifiée d'abord **à la main** : l'écriture canonique de deux petits cas, écrite octet par octet d'après
//! la doc de `lancer` et de `graphe`, et son SHA-256 calculé par Python `hashlib`, un outil indépendant ; puis sur d'autres
//! grilles, dont l'état a été écrit par un modèle Python de la doc (aucun lien avec ce dépôt). Seul le test final,
//! `regression_reference_grille_graine_1`, compare à un fichier de référence produit par notre propre code.

mod common;

use physarum_lab::lancer::{ErreurLancer, Sortie, lancer};
use physarum_lab::rendu::{apercu_graphe, treillis};
use physarum_lab::scenario::lire_scenario;

fn lancer_texte(texte: &str, graine: u64) -> Sortie {
    let scenario = lire_scenario(texte).expect("scénario valide");
    lancer(&scenario, graine).expect("lancer")
}

/// juste : l'empreinte est le SHA-256 de l'écriture canonique, écrite ici en toutes lettres pour deux cas (V-LANC-01) :
/// `PHYSAR00`, l'écriture du graphe (`n`, les nœuds, `m`, les arêtes), puis l'état de `Alea::new(graine)` (état puis
/// incrément `6D 00…`).
/// - `grille 1 1`, graine 0 (44 octets) : `5048595341523030 01000000 000000000000000000000000 00000000
///   9626a79b49f7e49a 6d00000000000000` ;
/// - `grille 2 1`, graine 7 (80 octets) : deux nœuds `(0,0,0)` et `(1,0,0)`, une arête `0-1` de longueur et de coût `1.0`
///   (`000000000000f03f`), puis `d1a0bdb386a42205 6d00000000000000`.
///
/// Les SHA-256 viennent de Python `hashlib`. Un `u32` grand-boutiste, une étiquette fausse ou un état déjà tiré échouent ici.
///
/// Ne couvre pas : le contenu de `mesures` et de `apercu` (autres tests).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn empreinte_canonique_a_la_main() {
    assert_eq!(
        lancer_texte("grille 1 1", 0).empreinte,
        "e07d8298c4c650883c03213c49b38643fffa60bfca6836794ac1216a86a3d459"
    );
    assert_eq!(
        lancer_texte("grille 2 1", 7).empreinte,
        "b48edde7d711ce81df31d2d14d5b83f4ee6f9dc0094141b2ca8a1394fd351f40"
    );
}

/// juste : l'empreinte de dix autres scénarios, calculée par le modèle Python de la doc (V-LANC-02) : chaque changement
/// change l'empreinte (largeur et hauteur échangées, la graine, le voisinage, un étage de plus, le coût vertical seul),
/// et l'ordre des directives ne la change pas.
///
/// Ne couvre pas : les grilles de plus de 1 000 nœuds.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn empreintes_de_dix_scenarios() {
    let cas: [(&str, u64, &str); 10] = [
        ("grille 3 2", 7, "4d066023b79eba897bc283369d234a9ca2377cb85b9a3d8cf054c87fb1fc4008"),
        ("grille 2 3", 7, "71bb260aff0dfe9dfe27382df0e9f8432deb0b3a9a5ad8aafc164769724c7b68"),
        ("grille 3 2", 8, "d1c70c118ed9ecac796d46fb5415da9ab294d50f4e2c5bc85f6fda84f70b05b9"),
        ("grille 3 2\nvoisinage 8", 7, "958a852ac06d0544089406d694723c2c2e69e97dff968ed62b2c56a73af759f9"),
        ("grille 3 2\netages 2 0.5 3", 7, "ca0220ae8eef812c1b2f386d8bec1349d2e8ea2c2908d7b9789258703b867db9"),
        ("grille 3 2\netages 2 0.5 4", 7, "4a7980946a85b8408646a5d5d363552e2cb212ffe6aa299575e6a82a098aedb7"),
        (
            "etages 2 0.5 3\nvoisinage 8\ngrille 3 2",
            7,
            "23a47254fb60736b782580746cb5e1455d25d2a7076afcfd423dd22be408335c",
        ),
        ("grille 3 3\netages 3 0.25 0.75", 5, "1c2207472ac2aa11ba16c04ab368bf5277b736fdcc2a1e5b1ff0719e7d91b378"),
        ("grille 256 1", u64::MAX, "1eceee455fd4f200f1c1849016206d2de3798b1454d5e4a72f7d44807eaa2618"),
        (
            "grille 65 2\nvoisinage 8\netages 2 2.5 0.125",
            3,
            "99a5bbc76b85c51d9b6af3b316831269e05d8832a0e69b584a28a080b5da7204",
        ),
    ];
    for (texte, graine, attendu) in cas {
        assert_eq!(lancer_texte(texte, graine).empreinte, attendu, "{texte:?} graine {graine}");
    }
    assert_eq!(lancer_texte("grille 3 2\n# un commentaire\n", 7).empreinte, cas[0].2, "un commentaire ne change rien");
    assert_eq!(lancer_texte("grille 3 2\nvoisinage 4\netages 1 9 9", 7).empreinte, cas[0].2, "les défauts écrits");
}

/// juste : les mesures sont le CSV `temps,noeuds,aretes,longueur\n0,<n>,<m>,<longueur>\n`, la longueur écrite avec six
/// décimales (V-LANC-03, valeurs du modèle Python : comptages de la définition, somme des longueurs dans l'ordre des
/// arêtes, `'%.6f'`). En voisinage 8, la longueur compte des `√2` : `12.656854` pour 3 × 2 (sept arêtes de 1 et quatre
/// diagonales), `3791.614716` pour le scénario d'exemple.
///
/// Ne couvre pas : les longueurs au-delà de quelques milliers.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn mesures_en_csv() {
    for (texte, attendu) in [
        ("grille 1 1", "temps,noeuds,aretes,longueur\n0,1,0,0.000000\n"),
        ("grille 2 1", "temps,noeuds,aretes,longueur\n0,2,1,1.000000\n"),
        ("grille 3 2", "temps,noeuds,aretes,longueur\n0,6,7,7.000000\n"),
        ("grille 3 2\nvoisinage 8", "temps,noeuds,aretes,longueur\n0,6,11,12.656854\n"),
        ("grille 3 2\netages 2 0.5 3", "temps,noeuds,aretes,longueur\n0,12,20,17.000000\n"),
        ("grille 3 2\nvoisinage 8\netages 2 0.5 3", "temps,noeuds,aretes,longueur\n0,12,28,28.313708\n"),
        ("grille 3 3\netages 3 0.25 0.75", "temps,noeuds,aretes,longueur\n0,27,54,40.500000\n"),
        ("grille 256 1", "temps,noeuds,aretes,longueur\n0,256,255,255.000000\n"),
        ("grille 65 2\nvoisinage 8\netages 2 2.5 0.125", "temps,noeuds,aretes,longueur\n0,260,772,1073.038672\n"),
        ("grille 24 16\nvoisinage 8\netages 2 1.0 3.0", "temps,noeuds,aretes,longueur\n0,768,3220,3791.614716\n"),
    ] {
        assert_eq!(lancer_texte(texte, 1).mesures, attendu, "{texte:?}");
    }
    assert_eq!(lancer_texte("grille 3 2", 99).mesures, lancer_texte("grille 3 2", 1).mesures, "la graine n'y est pas");
}

/// juste : l'aperçu est un PNG que le décodeur du test relit, de dimensions `TL * echelle` sur `th * echelle` avec
/// `echelle = 512 / TL` ramené entre 1 et 8 (V-LANC-04, la règle de la doc ; dimensions du modèle Python) : 40 × 24 pour
/// `grille 3 2` (échelle 8), 96 × 24 à deux étages, 504 × 8 pour 32 de large (échelle 8), 455 × 7 pour 33 (échelle 7),
/// 510 × 2 pour 128 (échelle 2), 257 × 1 pour 129 (échelle 1), 1024 × 1 pour 256 de large à deux étages (`512 / 1024`
/// vaut 0 : ramené à 1), 480 × 155 pour le scénario d'exemple (échelle 5). Ses pixels sont ceux de
/// `rendu::apercu_graphe` à cette échelle (que `etape_0_rendu` vérifie point par point).
///
/// Ne couvre pas : le dessin lui-même (voir `etape_0_rendu`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn apercu_et_son_echelle() {
    for (texte, echelle, largeur, hauteur) in [
        ("grille 3 2", 8, 40, 24),
        ("grille 3 2\netages 2 0.5 3", 8, 96, 24),
        ("grille 1 1", 8, 8, 8),
        ("grille 32 1", 8, 504, 8),
        ("grille 33 1", 7, 455, 7),
        ("grille 128 1", 2, 510, 2),
        ("grille 129 1", 1, 257, 1),
        ("grille 256 1", 1, 511, 1),
        ("grille 256 1\netages 2 1 1", 1, 1024, 1),
        ("grille 24 16\nvoisinage 8\netages 2 1.0 3.0", 5, 480, 155),
    ] {
        let scenario = lire_scenario(texte).unwrap();
        let sortie = lancer(&scenario, 1).unwrap();
        let lu = common::relire_png(&sortie.apercu).unwrap_or_else(|e| panic!("{texte:?} : PNG invalide : {e}"));
        assert_eq!((lu.largeur, lu.hauteur), (largeur, hauteur), "{texte:?}");
        let graphe = scenario.grille.graphe().unwrap();
        assert_eq!(treillis(&graphe).0 * echelle, largeur, "{texte:?}");
        assert_eq!(lu.rvb, apercu_graphe(&graphe, echelle).rvb, "{texte:?}");
    }
    // la graine ne change pas l'aperçu de la grille seule
    assert_eq!(lancer_texte("grille 3 2", 1).apercu, lancer_texte("grille 3 2", 2).apercu);
    // les premiers pixels de `grille 3 2` : huit pixels de nœud, puis huit d'arête
    let lu = common::relire_png(&lancer_texte("grille 3 2", 1).apercu).unwrap();
    assert_eq!(lu.rvb[..3], [225, 225, 215]);
    assert_eq!(lu.rvb[3 * 7..3 * 9], [225, 225, 215, 90, 110, 130]);
}

/// juste : une directive que `lancer` ne comprend pas fait refuser le scénario : `DirectiveInconnue(ligne)` avec la ligne de
/// la première des directives restantes, dans l'ordre du fichier (V-LANC-05, la doc de `lancer`) ; le même scénario sans
/// elle réussit.
///
/// Ne couvre pas : les directives des étapes suivantes.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn directive_inconnue() {
    for (texte, ligne) in [
        ("grille 3 2\ninconnue 1 2\n", 2),
        ("# c\n\nx_1\ngrille 3 2\nautre 3\n", 3),
        ("grille 3 2\nvoisinage 8\n\n\nsans_argument\ninconnue 1\n", 5),
    ] {
        let scenario = lire_scenario(texte).unwrap();
        assert_eq!(lancer(&scenario, 1), Err(ErreurLancer::DirectiveInconnue(ligne)), "{texte:?}");
    }
    assert!(lancer(&lire_scenario("grille 3 2\nvoisinage 8\n").unwrap(), 1).is_ok());
}

/// juste : le lancer est déterministe : deux lancers du même scénario et de la même graine rendent la même `Sortie`, octet
/// pour octet ; et l'empreinte a 64 chiffres hexadécimaux minuscules (V-LANC-06). Ce test ne vaut que par les autres : il
/// ne dit pas que la sortie est juste.
///
/// Ne couvre pas : l'égalité entre deux machines (c'est la CI, sous Linux et sous Windows, qui la rejoue).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn deux_lancers_identiques() {
    let texte = "grille 24 16\nvoisinage 8\netages 2 1.0 3.0\n";
    let (a, b) = (lancer_texte(texte, 1), lancer_texte(texte, 1));
    assert_eq!(a, b);
    assert_eq!(a.empreinte.len(), 64);
    assert!(a.empreinte.bytes().all(|o| o.is_ascii_digit() || (b'a'..=b'f').contains(&o)));
    assert_eq!(a.empreinte, "c336148193197cb8541fd6a58c36f149118260789f9e388771405fe52e6eaf59", "calculée à part");
    assert_eq!(lancer_texte(texte, 2).empreinte, "3e99fcca2d6414453203fba48994397ff4a829b2c610cfe371ee2948f744d908");
}

/// régression : pour `scenarios/grille.txt` et la graine 1, l'empreinte rendue par `lancer` est celle du fichier
/// `references/grille-graine-1.empreinte` (l'empreinte puis un saut de ligne), le fichier que la CI rejoue sous Linux et
/// sous Windows. Il n'existe pas dans le jeu de tests : la vague du code le produit avec le binaire (V-LANC-07).
///
/// Ne couvre pas : que l'empreinte soit juste (`deux_lancers_identiques` la compare à la valeur calculée à part :
/// `c3361481…af59`) ; l'égalité entre Linux et Windows n'est établie que par la CI qui rejoue cette référence sur les deux.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn regression_reference_grille_graine_1() {
    let racine = env!("CARGO_MANIFEST_DIR");
    let reference = std::fs::read_to_string(format!("{racine}/references/grille-graine-1.empreinte"))
        .expect("references/grille-graine-1.empreinte est absente : la vague du code la produit avec le binaire");
    let texte = std::fs::read_to_string(format!("{racine}/scenarios/grille.txt")).expect("scenarios/grille.txt");
    assert_eq!(format!("{}\n", lancer_texte(&texte, 1).empreinte), reference);
}
