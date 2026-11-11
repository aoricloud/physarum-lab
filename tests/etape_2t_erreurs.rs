//! Étape 2, partie 2T : les erreurs et leur ordre (`pas_sans_decoupage`, `Agent::avancer`, `taux_de_depart`,
//! `pas_maximal`). Valeurs attendues : `references/etape-2t-valeurs.md`, identifiants `V-ERR-xx`.
//!
//! La source est la doc de `souvenir` (« Les vérifications, dans cet ordre » : 1 `Taille`, 2 `Concentration(i)`, 3
//! `Flux(e)`, 4 `Diffusion` puis `Perte`, 5 `Pas`, 6 `Apport(r)`, 7 `Debordement`, 8 `SousPas`). Un cas à une seule
//! faute pour chaque erreur, puis des cas à deux fautes qui montrent l'ordre ; et « sur une erreur, rien n'a changé ».

#[path = "common/transport_2t.rs"]
mod transport_2t;

use physarum_lab::souvenir::{Agent, ErreurAgent, Transport, pas_maximal, pas_sans_decoupage, taux_de_depart};
use transport_2t::{bits, chaine, graphe_l, naif_pas, naif_pas_maximal, naif_taux};

/// Les entrées d'un pas qui réussit : la chaîne de quatre nœuds, un flux, des concentrations, un transport, un apport
/// et un pas.
struct Entrees {
    flux: Vec<f64>,
    c: Vec<f64>,
    tr: Transport,
    apports: Vec<(usize, f64)>,
    h: f64,
}

fn bonnes() -> Entrees {
    Entrees {
        flux: vec![0.5, 0.5, 0.5],
        c: vec![1.0, 0.5, 0.25, 0.125],
        tr: Transport { diffusion: 0.1, perte: 0.01 },
        apports: vec![(0, 0.5), (3, 0.25)],
        h: 0.1,
    }
}

/// Joue les entrées par `pas_sans_decoupage` et par `Agent::avancer` (si les concentrations ont la bonne taille et sont
/// valides, l'agent est construit ; sinon on n'essaie que la fonction libre) et vérifie que les deux rendent `attendu`,
/// et que l'agent n'a pas changé.
fn verifier(e: &Entrees, attendu: ErreurAgent, nom: &str) {
    let g = chaine(&[1.0, 1.0, 1.0]);
    let libre = pas_sans_decoupage(&g, &e.flux, &e.c, &e.tr, &e.apports, e.h);
    assert_eq!(libre.map(|_| ()), Err(attendu), "{nom} : pas_sans_decoupage");
    if let Ok(mut agent) = Agent::new(&g, e.c.clone()) {
        let avant = agent.clone();
        assert_eq!(
            agent.avancer(&g, &e.flux, &e.tr, &e.apports, e.h).map(|_| ()),
            Err(attendu),
            "{nom} : Agent::avancer"
        );
        assert_eq!(agent, avant, "{nom} : rien n'a changé");
    }
}

/// juste : les bonnes entrées réussissent, aux deux fonctions, **et rendent le résultat du pas naïf du test**
/// (V-ERR-01 : le cas positif d'abord, avant tout cas d'erreur ; un `Ok` à valeur nulle ne suffit pas).
///
/// Ne couvre pas : les erreurs (les tests suivants).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn les_bonnes_entrees_reussissent() {
    let e = bonnes();
    let g = chaine(&[1.0, 1.0, 1.0]);
    let (naif, [sortie, perte, apport]) = naif_pas(&g, &e.flux, &e.c, e.tr.diffusion, e.tr.perte, &e.apports, e.h);
    let (suite, b) = pas_sans_decoupage(&g, &e.flux, &e.c, &e.tr, &e.apports, e.h).unwrap();
    assert_eq!(bits(&suite), bits(&naif));
    assert_eq!(bits(&[b.sortie, b.perte, b.apport]), bits(&[sortie, perte, apport]));
    assert!(b.perte > 0.0 && b.apport > 0.0, "une perte et un apport non nuls");
    assert_eq!(b.sous_pas, 1);
    let mut agent = Agent::new(&g, e.c.clone()).unwrap();
    assert_eq!(agent.avancer(&g, &e.flux, &e.tr, &e.apports, e.h), Ok(b));
    assert_eq!(bits(agent.concentrations()), bits(&naif));
    let (r, _) = naif_taux(&g, &e.flux, e.tr.diffusion, e.tr.perte);
    assert_eq!(bits(&taux_de_depart(&g, &e.flux, &e.tr).unwrap()), bits(&r));
    assert_eq!(pas_maximal(&g, &e.flux, &e.tr).unwrap().to_bits(), naif_pas_maximal(&g, &e.flux, 0.1, 0.01).to_bits());
}

/// juste : chaque erreur seule (V-ERR-02), aux deux fonctions, l'agent inchangé :
/// - `Taille` : trois concentrations (ou cinq) pour quatre nœuds ; deux flux (ou quatre) pour trois arêtes ;
/// - `Concentration(i)` : NaN, `+inf`, `-inf`, un négatif minuscule (`-1e-300`), au rang `i` (le plus petit s'il y en a
///   plusieurs) ; `-0.0` est accepté ;
/// - `Flux(e)` : NaN, `+inf`, `-inf`, au rang `e` (le plus petit) ; un flux négatif ou nul est valide ;
/// - `Diffusion` : NaN, `+inf`, négative (`-0.1`) ; `0.0` et `-0.0` sont valides ;
/// - `Perte` : de même ;
/// - `Pas` : `0.0`, `-0.1`, NaN, `+inf` ;
/// - `Apport(r)` : le nœud 4 (hors du graphe), un débit négatif, NaN, `+inf`, au rang `r` (le plus petit) ; un débit
///   `0.0` est valide.
///
/// Ne couvre pas : l'ordre entre erreurs (`l_ordre_des_verifications`), `Debordement`, `SousPas`.
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn chaque_erreur_seule() {
    let mauvais = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1e-300];
    // Taille
    for n in [3, 5, 0] {
        let mut e = bonnes();
        e.c = vec![0.5; n];
        verifier(&e, ErreurAgent::Taille, "concentrations");
    }
    for n in [2, 4, 0] {
        let mut e = bonnes();
        e.flux = vec![0.5; n];
        verifier(&e, ErreurAgent::Taille, "flux");
    }
    // Concentration
    for &m in &mauvais {
        for rang in 0..4 {
            let mut e = bonnes();
            e.c[rang] = m;
            verifier(&e, ErreurAgent::Concentration(rang), "concentration");
        }
    }
    let mut e = bonnes();
    e.c = vec![0.0, f64::NAN, 1.0, -1.0];
    verifier(&e, ErreurAgent::Concentration(1), "la plus petite concentration fautive");
    // Flux
    for &m in &mauvais[..3] {
        for rang in 0..3 {
            let mut e = bonnes();
            e.flux[rang] = m;
            verifier(&e, ErreurAgent::Flux(rang), "flux");
        }
    }
    let mut e = bonnes();
    e.flux = vec![0.0, f64::INFINITY, f64::NAN];
    verifier(&e, ErreurAgent::Flux(1), "le plus petit flux fautif");
    // Diffusion et perte
    for m in [f64::NAN, f64::INFINITY, -0.1] {
        let mut e = bonnes();
        e.tr.diffusion = m;
        verifier(&e, ErreurAgent::Diffusion, "diffusion");
        let mut e = bonnes();
        e.tr.perte = m;
        verifier(&e, ErreurAgent::Perte, "perte");
    }
    // Pas
    for m in [0.0, -0.1, f64::NAN, f64::INFINITY] {
        let mut e = bonnes();
        e.h = m;
        verifier(&e, ErreurAgent::Pas, "pas");
    }
    // Apport
    for (rang, fautif) in [(0, (4, 0.5)), (1, (4, 0.5)), (0, (0, -0.5)), (1, (3, f64::NAN)), (1, (3, f64::INFINITY))] {
        let mut e = bonnes();
        e.apports[rang] = fautif;
        verifier(&e, ErreurAgent::Apport(rang), "apport");
    }
    let mut e = bonnes();
    e.apports = vec![(1, 0.0), (9, 1.0), (2, f64::NAN)];
    verifier(&e, ErreurAgent::Apport(1), "le plus petit apport fautif");
    // Les valeurs limites valides.
    let g = chaine(&[1.0, 1.0, 1.0]);
    let mut e = bonnes();
    e.c = vec![-0.0, 0.0, 1e-300, f64::MAX / 4.0];
    e.flux = vec![-0.0, 0.0, -0.5];
    e.tr = Transport { diffusion: -0.0, perte: 0.0 };
    e.apports = vec![(2, 0.0)];
    assert!(pas_sans_decoupage(&g, &e.flux, &e.c, &e.tr, &e.apports, e.h).is_ok(), "les limites valides");
}

/// juste : **l'ordre des vérifications** (V-ERR-03), la première qui s'applique gagne : une entrée à deux fautes rend
/// l'erreur du point le plus bas : `Taille` avant `Concentration` (des concentrations de mauvaise taille et fautives) ;
/// `Concentration` avant `Flux` ; `Flux` avant `Diffusion` ; `Diffusion` avant `Perte` ; `Perte` avant `Pas` ; `Pas`
/// avant `Apport` ; `Apport` avant `Debordement` (un apport fautif et un taux qui déborde) ; `Taille` d'un flux avant
/// `Concentration` ; toutes les fautes à la fois : `Taille`.
///
/// Ne couvre pas : `Debordement` avant `SousPas` (`debordement_et_sous_pas`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn l_ordre_des_verifications() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    let appel = |e: &Entrees| pas_sans_decoupage(&g, &e.flux, &e.c, &e.tr, &e.apports, e.h).map(|_| ());
    let mut e = bonnes();
    e.c = vec![f64::NAN, 1.0, 1.0];
    assert_eq!(appel(&e), Err(ErreurAgent::Taille), "Taille avant Concentration");
    let mut e = bonnes();
    e.flux = vec![0.5, 0.5];
    e.c[2] = f64::NAN;
    assert_eq!(appel(&e), Err(ErreurAgent::Taille), "Taille d'un flux avant Concentration");
    let mut e = bonnes();
    e.c[3] = -1.0;
    e.flux[0] = f64::NAN;
    assert_eq!(appel(&e), Err(ErreurAgent::Concentration(3)), "Concentration avant Flux");
    let mut e = bonnes();
    e.flux[2] = f64::INFINITY;
    e.tr.diffusion = -1.0;
    assert_eq!(appel(&e), Err(ErreurAgent::Flux(2)), "Flux avant Diffusion");
    let mut e = bonnes();
    e.tr = Transport { diffusion: f64::NAN, perte: -1.0 };
    assert_eq!(appel(&e), Err(ErreurAgent::Diffusion), "Diffusion avant Perte");
    let mut e = bonnes();
    e.tr.perte = -1.0;
    e.h = 0.0;
    assert_eq!(appel(&e), Err(ErreurAgent::Perte), "Perte avant Pas");
    let mut e = bonnes();
    e.h = -1.0;
    e.apports = vec![(7, 1.0)];
    assert_eq!(appel(&e), Err(ErreurAgent::Pas), "Pas avant Apport");
    // Un apport fautif et un taux qui déborde : l'apport passe avant.
    let etoile = graphe_l(4, &[(0, 1, 1.0), (0, 2, 1.0), (0, 3, 1.0)]);
    let r = pas_sans_decoupage(
        &etoile,
        &[1e308; 3],
        &[0.0; 4],
        &Transport { diffusion: 0.0, perte: 0.0 },
        &[(9, 1.0)],
        0.1,
    );
    assert_eq!(r.map(|_| ()), Err(ErreurAgent::Apport(0)), "Apport avant Debordement");
    let mut e = bonnes();
    e.c = vec![f64::NAN];
    e.flux = vec![f64::NAN];
    e.tr = Transport { diffusion: -1.0, perte: -1.0 };
    e.h = -1.0;
    e.apports = vec![(99, -1.0)];
    assert_eq!(appel(&e), Err(ErreurAgent::Taille), "toutes les fautes à la fois");
}

/// juste : `Debordement` et `SousPas` (V-ERR-04). Une étoile à trois feuilles de flux `1e308` chacune (le centre donne
/// `1e308 + 1e308 + 1e308` : le taux du centre est infini) : `Debordement`, aux deux fonctions, `taux_de_depart` et
/// `pas_maximal` aussi (le taux est vu avant tout calcul de concentration) ; une diffusion `1e300` sur une arête de
/// longueur `1e-10` (`κ / L = 1e310`, infini) : `Debordement` ; **le résultat qui déborde** : le centre d'une étoile à
/// deux feuilles où le liquide va des feuilles (concentration `1.7e308`) vers le centre, un pas de `h = 1` sans
/// découpage : le centre reçoit deux fois `1.7e308` (infini) : `Debordement` (et le pas public, qui découpe en deux
/// sous-pas de `0.5`, réussit sur le même cas : `0.5 * 1.7e308 * 2` est fini). Un nœud seul de perte `1` et un `h` de
/// `2e6` : `SousPas` (deux millions de sous-pas) ; `Debordement` passe avant `SousPas` : l'étoile à `1e308` avec ce
/// même `h` donne `Debordement`.
///
/// Ne couvre pas : un dépassement au cours des sous-pas (la doc dit seulement que rien ne change).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn debordement_et_sous_pas() {
    let etoile = graphe_l(4, &[(0, 1, 1.0), (0, 2, 1.0), (0, 3, 1.0)]);
    let sans = Transport { diffusion: 0.0, perte: 0.0 };
    let q = [1e308; 3];
    let c = [1.0; 4];
    assert_eq!(pas_sans_decoupage(&etoile, &q, &c, &sans, &[], 0.1).map(|_| ()), Err(ErreurAgent::Debordement));
    let mut agent = Agent::new(&etoile, c.to_vec()).unwrap();
    let avant = agent.clone();
    assert_eq!(agent.avancer(&etoile, &q, &sans, &[], 0.1).map(|_| ()), Err(ErreurAgent::Debordement));
    assert_eq!(agent, avant, "rien n'a changé");
    assert_eq!(taux_de_depart(&etoile, &q, &sans), Err(ErreurAgent::Debordement));
    assert_eq!(pas_maximal(&etoile, &q, &sans), Err(ErreurAgent::Debordement));
    // Une diffusion qui déborde.
    let fine = graphe_l(2, &[(0, 1, 1e-10)]);
    let forte = Transport { diffusion: 1e300, perte: 0.0 };
    assert_eq!(taux_de_depart(&fine, &[0.0], &forte), Err(ErreurAgent::Debordement));
    assert_eq!(
        pas_sans_decoupage(&fine, &[0.0], &[1.0, 1.0], &forte, &[], 0.1).map(|_| ()),
        Err(ErreurAgent::Debordement)
    );
    // Le résultat qui déborde, sans découpage ; le pas public réussit.
    let v = graphe_l(3, &[(0, 1, 1.0), (0, 2, 1.0)]);
    let q = [-1.0, -1.0];
    let c = [0.0, 1.7e308, 1.7e308];
    assert_eq!(pas_sans_decoupage(&v, &q, &c, &sans, &[], 1.0).map(|_| ()), Err(ErreurAgent::Debordement));
    let mut agent = Agent::new(&v, c.to_vec()).unwrap();
    let b = agent.avancer(&v, &q, &sans, &[], 1.0).unwrap();
    assert_eq!(b.sous_pas, 2);
    assert!(agent.concentrations().iter().all(|x| x.is_finite() && *x >= 0.0));
    // SousPas, et Debordement avant SousPas.
    let seul = graphe_l(1, &[]);
    let perte = Transport { diffusion: 0.0, perte: 1.0 };
    let mut agent = Agent::new(&seul, vec![1.0]).unwrap();
    assert_eq!(agent.avancer(&seul, &[], &perte, &[], 2e6).map(|_| ()), Err(ErreurAgent::SousPas));
    assert_eq!(agent.concentrations()[0], 1.0);
    let mut agent = Agent::new(&etoile, vec![1.0; 4]).unwrap();
    assert_eq!(agent.avancer(&etoile, &[1e308; 3], &sans, &[], 2e6).map(|_| ()), Err(ErreurAgent::Debordement));
}

/// juste : les erreurs de `taux_de_depart` et de `pas_maximal` (V-ERR-05), qui ne lisent que le graphe, les flux et le
/// transport : `Taille` (`flux.len()` n'est pas le nombre d'arêtes, même si un flux est NaN : la taille passe avant),
/// `Flux(e)` (le plus petit rang non fini), `Diffusion` puis `Perte`, puis `Debordement` (cas de l'étoile à `1e308`).
/// Elles ne lisent ni concentrations ni `h` : aucune de ces deux erreurs n'y existe pour elles.
///
/// Ne couvre pas : `Debordement` en détail (`debordement_et_sous_pas`).
#[test]
#[cfg_attr(not(feature = "etape-2t"), ignore)]
fn les_erreurs_des_taux_et_du_pas_maximal() {
    let g = chaine(&[1.0, 1.0, 1.0]);
    let ok = Transport { diffusion: 0.1, perte: 0.01 };
    assert!(taux_de_depart(&g, &[0.5; 3], &ok).is_ok());
    assert!(pas_maximal(&g, &[0.5; 3], &ok).is_ok());
    for (flux, tr, attendu) in [
        (vec![0.5; 2], ok, ErreurAgent::Taille),
        (vec![f64::NAN, 0.5], ok, ErreurAgent::Taille),
        (vec![0.5; 4], ok, ErreurAgent::Taille),
        (vec![0.5, f64::NAN, f64::INFINITY], ok, ErreurAgent::Flux(1)),
        (vec![0.5, 0.5, f64::NEG_INFINITY], ok, ErreurAgent::Flux(2)),
        (vec![0.5; 3], Transport { diffusion: -1.0, perte: 0.0 }, ErreurAgent::Diffusion),
        (vec![0.5; 3], Transport { diffusion: f64::NAN, perte: f64::NAN }, ErreurAgent::Diffusion),
        (vec![0.5; 3], Transport { diffusion: 0.0, perte: f64::INFINITY }, ErreurAgent::Perte),
        (vec![f64::NAN; 3], Transport { diffusion: -1.0, perte: -1.0 }, ErreurAgent::Flux(0)),
    ] {
        assert_eq!(taux_de_depart(&g, &flux, &tr), Err(attendu), "taux_de_depart {flux:?}");
        assert_eq!(pas_maximal(&g, &flux, &tr), Err(attendu), "pas_maximal {flux:?}");
    }
}
