//! Les tubes : le modèle de Tero, avec la loi linéaire de Bonifaci, un pas d'Euler (étape 1).
//!
//! Le blob est un réseau de tubes. Une source de nourriture y pousse un flux vers un puits ; un tube qui porte du flux
//! s'épaissit, un tube qui n'en porte pas s'amincit. Ce module en est le cœur : il fait **un pas** de cette adaptation
//! ([`Tubes::avancer`]) et dessine l'état ([`apercu`]). Ce que les tests d'acceptation prouvent est dans
//! `references/etape-1-valeurs.md` (identifiants `V-TUBE-xx`).
//!
//! # Le modèle, et d'où il vient
//!
//! Chaque arête `e` du [`Graphe`] est un tube de **conductivité** `D[e] > 0`. Le flux dans le tube est (en réels) `Q[e]
//! = D[e] * (p[a] - p[b]) / cout[e]` (`a < b` les bouts) : c'est la loi de Poiseuille de Tero, Kobayashi et Nakagaki
//! (J. Theor. Biol. 244, 2007, éq. 1) ; le solveur de l'étape 0 le calcule avec la **conductance** `g[e] = D[e] /
//! cout[e]` (ici `cout` joue le rôle de la longueur du tube dans l'article). Un flux `I` entre à la source et sort au
//! puits, dont la pression est zéro (Tero 2007, p. 7 : « `p_2 = 0` comme niveau de pression de base »). L'adaptation
//! est
//!
//! ```text
//! dD/dt = f(|Q|) - D        avec  f(Q) = |Q|        (loi LINÉAIRE : μ = 1 chez Tero 2007, f = identité chez Bonifaci)
//! ```
//!
//! - **Ce qu'on a lu** (voir `references/etape-1-valeurs.md`, section « Sources ») : Tero 2007 (version d'auteur de
//!   2006) donne l'équation sans dimension `dD/dt = f(|Q|) - D` (éq. 5), le flux total constant, les conductivités
//!   initiales « tirées au hasard, uniformément dans [0,5 ; 1,0] », et dit que pour `f` linéaire « le chemin le plus
//!   court est sélectionné quelles que soient les conductivités initiales ». Bonifaci, Mehlhorn et Varma
//!   (arXiv:1106.0423 v3, J. Theor. Biol. 309, 2012) démontrent, pour `D' = |Q| - D`, le **théorème 2** : la dynamique
//!   est attirée par l'ensemble des flux de coût minimal, et si le plus court chemin est unique elle converge vers le
//!   flux de valeur 1 sur ce chemin (le potentiel source-puits tend vers sa longueur). Becchetti et al. (ICALP 2013)
//!   discrétisent par la méthode d'Euler, `x(t+1) - x(t) = h (|q| - x(t))`, avec `h` dans `]0, 1[` : c'est le pas
//!   d'ici.
//! - **Ce qu'on n'a pas lu** : le pas `Δt` de Tero 2007 n'est pas publié (son schéma est semi-implicite, le nôtre est
//!   explicite : c'est un choix, dit dans `MODELE.md`) ; la preuve de Becchetti pour un pas fini a des hypothèses
//!   (conductivités initiales égales à 1, longueurs `>= 1`, `h` petit) que nos tests ne reprennent pas. Les tests
//!   **constatent** la convergence, ils ne la démontrent pas.
//!
//! # Le pas, à la lettre
//!
//! Pour une `source`, un `puits`, un débit `I` (`debit`) et un pas de temps `h`, [`Tubes::avancer`] fait ceci, dans cet
//! ordre, et le résultat est **celui de ces opérations, bit pour bit** (une opération `f64` par ligne, dans l'ordre et
//! avec les parenthèses écrits) :
//!
//! ```text
//! g[e]  = D[e] / cout[e]                                       pour chaque arête e, par indice croissant
//! inj   = un vecteur de nb_noeuds fois +0.0, puis inj[source] = I
//! p     = solveur::pressions(graphe, g, inj, puits)            (le puits est la masse ; son injection n'est pas lue)
//! Q     = solveur::flux(graphe, g, p)
//! pour chaque arête e, par indice croissant :
//!     d    = D[e] + h * (|Q[e]| - D[e])                        (cette écriture : soustraction, produit, somme)
//!     D[e] = d  si d >= PLANCHER,  sinon PLANCHER              (un NaN deviendrait PLANCHER ; il ne se produit pas)
//! ```
//!
//! Les pressions et les flux rendus dans [`Etat`] sont ceux de `p` et `Q` : calculés avec les conductivités d'**avant**
//! la mise à jour. Une seule mise à jour par appel ; la mise à jour de toutes les arêtes se fait après le calcul de `Q`
//! (une arête mise à jour n'influence pas la suivante dans le même pas).
//!
//! # Le plancher de conductivité (à nous, aucun article)
//!
//! [`PLANCHER`] vaut `1e-6`. Un tube sans flux s'amincit comme `(1 - h)^n` ; sans borne, le rapport entre la plus
//! grande et la plus petite conductance dépasserait `2^53`, le solveur perdrait sa précision puis s'arrêterait sur un
//! pivot nul (voir « Ce que le solveur ne garantit pas » dans `crate::solveur`), et un nœud ne tenant plus que par un
//! tube mort serait coupé de la masse. Avec le plancher, toute conductivité reste `>= PLANCHER` : le graphe reste relié
//! et le rapport reste borné. Le théorème de Bonifaci vaut pour le modèle **sans** plancher : les tests disent la marge
//! (un tube mort est tenu à `PLANCHER`, pas à zéro, et une conductivité peut remonter depuis le plancher si le flux
//! revient).
//!
//! # Les vérifications de [`Tubes::avancer`], dans cet ordre
//!
//! 1. `graphe.nb_aretes()` doit être le nombre de conductivités, sinon `ErreurTubes::Taille` ;
//! 2. `source` et `puits` doivent être deux indices de nœud **différents**, sinon `ErreurTubes::Noeud` ;
//! 3. `debit` doit être fini et `> 0.0`, sinon `ErreurTubes::Debit` ;
//! 4. `h` doit être fini avec `0.0 < h < 1.0`, sinon `ErreurTubes::Pas` ;
//! 5. l'erreur du solveur, s'il en rend une, sous la forme `ErreurTubes::Solveur(erreur)` (par exemple
//!    `ErreurSolveur::Isole(i)` quand le graphe n'est pas connexe : `i` est le plus petit nœud qui n'est pas relié au
//!    puits).
//!
//! **Sur une erreur, rien n'a changé** : les conductivités sont exactement ce qu'elles étaient.
//!
//! # Les cas dégénérés
//!
//! - Un tube entre la source et le puits, seul (deux nœuds, une arête) : `Q = I` à chaque pas, donc `D <- D + h (I -
//!   D)` (à l'arrondi près : le solveur divise puis multiplie).
//! - Un graphe qui n'est pas connexe a un nœud sans chemin vers le puits : la vérification 5 rend `Solveur(Isole(i))`.
//! - Un graphe sans arête a zéro conductivité : `Tubes::new(graphe, vec![])` réussit.
//!
//! # L'aperçu des tubes
//!
//! [`apercu`] dessine sur le **même treillis** que `rendu::apercu_graphe` (voir la documentation de `crate::rendu` :
//! mêmes panneaux par étage, même [`ECART`](crate::rendu::ECART), mêmes positions, mêmes dimensions `TL * echelle` sur
//! `th * echelle`), mais la couleur d'un point dit l'épaisseur des tubes. Tout est en entiers, sauf le niveau d'une
//! conductivité `d` (avec `t = d / reference`, une division de flottants) :
//!
//! ```text
//! niveau(d) = 255                        si t >= 1.0
//!           = (t * 255.0).round()        si 0.0 < t < 1.0     (round : à l'entier le plus proche, les moitiés vers
//!                                                              l'extérieur ; 0.5 donne 127.5, donc 128)
//!           = 0                          sinon                (zéro, négatif, NaN)
//! melange(a, b, n) = (a * (255 - n) + b * n + 127) / 255        par canal, en entiers non signés, division entière
//!                                                                (n = 0 donne a, n = 255 donne b)
//! ```
//!
//! `FOND` et `PUITS` sont ceux de `crate::rendu`, [`VIF`] est la couleur d'un tube de niveau 255. Soit `niv[e]` le
//! niveau de l'arête `e`, et, pour un nœud `i`, `du_noeud[i]` le plus grand `niv[e]` de **toutes** ses arêtes (celles
//! que le treillis ne dessine pas comprises : une arête longue, un lien entre étages ; 0 s'il n'en a aucune) et
//! `du_lien[i]` le plus grand `niv[e]` de ses arêtes **entre deux étages différents** (rien s'il n'en a pas). Le
//! cadre du treillis (`xmin`, `ymin`, `xmax`, `ymax`, le plus grand étage) se calcule sur tous les nœuds, comme dans
//! `rendu`. On peint le treillis dans cet ordre, chaque couche recouvrant les précédentes :
//!
//! 1. tout en `FOND` ;
//! 2. chaque arête `e` dont les deux bouts sont au même étage et dont les positions diffèrent d'au plus 1 en `x` et
//!    d'au plus 1 en `y` a un point du milieu (celui de `rendu`) ; ce point prend `melange(FOND, VIF, n)`, où `n` est
//!    le plus grand `niv[e]` des arêtes qui ont ce milieu (les deux diagonales d'une maille se croisent au même point :
//!    la plus épaisse l'emporte, quel que soit l'ordre des indices) ;
//! 3. chaque nœud `i`, par indice croissant : son point prend `melange(FOND, VIF, du_noeud[i])` (deux nœuds au même
//!    point : le dernier recouvre) ;
//! 4. chaque nœud `i` qui a un `du_lien[i]`, par indice croissant : son point prend `melange(c, PUITS, du_lien[i])`, où
//!    `c` est la couleur que le point a **à ce moment-là** (un lien entre étages colore ses deux bouts dans la teinte
//!    des puits, d'autant plus que le lien est épais ; un lien de niveau 0 laisse le point tel qu'il est).
//!
//! Puis chaque point devient un carré de `echelle x echelle` pixels, comme dans `rendu`. Un tube sous `1/510` de la
//! référence reste donc invisible (niveau 0 : la couleur du fond).

use crate::alea::Alea;
use crate::graphe::Graphe;
use crate::rendu::{ECART, FOND, Image, PUITS};
use crate::solveur::{ErreurSolveur, flux, pressions};

/// Le plancher de conductivité : `1e-6` (voir « Le plancher de conductivité » dans la documentation du module).
pub const PLANCHER: f64 = 1e-6;

/// La couleur vive d'un tube épais dans [`apercu`] : un tube de niveau 255 (voir « L'aperçu des tubes »).
pub const VIF: [u8; 3] = [60, 220, 255];

/// Pourquoi des tubes n'ont pas pu être construits ou avancés (voir les vérifications dans la documentation du module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurTubes {
    /// Le nombre de conductivités n'est pas le nombre d'arêtes du graphe.
    Taille,
    /// La conductivité de cette arête (le plus petit indice fautif) n'est pas un nombre fini `>= PLANCHER`.
    Conductivite(usize),
    /// `source` ou `puits` n'est pas un indice de nœud, ou les deux sont le même nœud.
    Noeud,
    /// Le débit n'est pas un nombre fini `> 0.0`.
    Debit,
    /// Le pas `h` n'est pas un nombre fini avec `0.0 < h < 1.0`.
    Pas,
    /// Le solveur de l'étape 0 a refusé ou échoué.
    Solveur(ErreurSolveur),
}

/// Ce que le pas a calculé avant de mettre à jour les conductivités.
#[derive(Clone, Debug, PartialEq)]
pub struct Etat {
    /// Les pressions de chaque nœud (`nb_noeuds` nombres, celle du puits valant `+0.0`).
    pub pressions: Vec<f64>,
    /// Le flux de chaque arête (`nb_aretes` nombres, positif quand le liquide va du petit indice vers le grand).
    pub flux: Vec<f64>,
}

/// Les conductivités des tubes d'un graphe : une par arête, dans l'ordre des indices, chacune finie et `>= PLANCHER`.
#[derive(Clone, Debug, PartialEq)]
pub struct Tubes {
    conductivites: Vec<f64>,
}

impl Tubes {
    /// Des tubes de conductivités données (une par arête, dans l'ordre des indices).
    ///
    /// # Erreurs
    ///
    /// Dans cet ordre : `ErreurTubes::Taille` si `conductivites.len()` n'est pas `graphe.nb_aretes()` ; puis
    /// `ErreurTubes::Conductivite(e)`, `e` étant le plus petit indice d'une conductivité qui n'est pas un nombre fini
    /// `>= PLANCHER` (`PLANCHER` lui-même est accepté ; NaN, l'infini, zéro et les négatifs sont refusés).
    pub fn new(graphe: &Graphe, conductivites: Vec<f64>) -> Result<Tubes, ErreurTubes> {
        if conductivites.len() != graphe.nb_aretes() {
            return Err(ErreurTubes::Taille);
        }
        for (e, &d) in conductivites.iter().enumerate() {
            if !(d.is_finite() && d >= PLANCHER) {
                return Err(ErreurTubes::Conductivite(e));
            }
        }
        Ok(Tubes { conductivites })
    }

    /// Des tubes de même conductivité `valeur`. Mêmes erreurs que [`Tubes::new`] (`Conductivite(0)` si `valeur` ne
    /// convient pas et que le graphe a au moins une arête ; aucune erreur de conductivité sans arête).
    pub fn uniformes(graphe: &Graphe, valeur: f64) -> Result<Tubes, ErreurTubes> {
        Tubes::new(graphe, vec![valeur; graphe.nb_aretes()])
    }

    /// Les conductivités de départ du modèle de Tero 2007 (« tirées au hasard, uniformément dans l'intervalle [0,5 ;
    /// 1,0] ») : pour chaque arête, par indice croissant, `0.5 + 0.5 * alea.suivant_f64()` (une addition d'un produit,
    /// dans cet ordre ; chaque tirage consomme **deux** tirages de 32 bits, voir `Alea::suivant_f64`). Elles sont dans
    /// `[0.5, 1.0[`, donc `>= PLANCHER`. `alea` a avancé de `2 * nb_aretes` tirages de 32 bits ensuite.
    pub fn au_hasard(graphe: &Graphe, alea: &mut Alea) -> Tubes {
        let mut conductivites = Vec::with_capacity(graphe.nb_aretes());
        for _ in 0..graphe.nb_aretes() {
            conductivites.push(0.5 + 0.5 * alea.suivant_f64());
        }
        Tubes { conductivites }
    }

    /// Les conductivités, dans l'ordre des indices d'arête.
    pub fn conductivites(&self) -> &[f64] {
        &self.conductivites
    }

    /// Un pas du modèle (voir « Le pas, à la lettre » et « Les vérifications » dans la documentation du module) :
    /// calcule les pressions et les flux avec les conductivités actuelles, les rend, et met les conductivités à jour.
    ///
    /// # Erreurs
    ///
    /// Celles de « Les vérifications », dans leur ordre : la première qui s'applique ; alors rien n'a changé.
    pub fn avancer(
        &mut self,
        graphe: &Graphe,
        source: usize,
        puits: usize,
        debit: f64,
        h: f64,
    ) -> Result<Etat, ErreurTubes> {
        // 1. la taille
        if graphe.nb_aretes() != self.conductivites.len() {
            return Err(ErreurTubes::Taille);
        }
        // 2. les nœuds
        if source >= graphe.nb_noeuds() || puits >= graphe.nb_noeuds() || source == puits {
            return Err(ErreurTubes::Noeud);
        }
        // 3. le débit
        if !(debit.is_finite() && debit > 0.0) {
            return Err(ErreurTubes::Debit);
        }
        // 4. le pas
        if !(h.is_finite() && h > 0.0 && h < 1.0) {
            return Err(ErreurTubes::Pas);
        }
        // 5. le solveur
        let mut conductances = Vec::with_capacity(graphe.nb_aretes());
        for e in 0..graphe.nb_aretes() {
            conductances.push(self.conductivites[e] / graphe.aretes()[e].cout);
        }
        let mut injections = vec![0.0; graphe.nb_noeuds()];
        injections[source] = debit;
        let p = pressions(graphe, &conductances, &injections, puits).map_err(ErreurTubes::Solveur)?;
        let q = flux(graphe, &conductances, &p);
        // La mise à jour, seulement une fois tous les flux calculés.
        let mut mises_a_jour = Vec::with_capacity(self.conductivites.len());
        for (e, &q_e) in q.iter().enumerate() {
            let d = self.conductivites[e] + h * (q_e.abs() - self.conductivites[e]);
            mises_a_jour.push(if d >= PLANCHER { d } else { PLANCHER });
        }
        self.conductivites = mises_a_jour;
        Ok(Etat { pressions: p, flux: q })
    }
}

/// L'aperçu des tubes : les mêmes panneaux, le même écart, les mêmes positions que `rendu::apercu_graphe`, agrandis
/// `echelle` fois, où la couleur dit l'épaisseur des tubes (voir « L'aperçu des tubes » dans la documentation du
/// module).
///
/// # Panique
///
/// Si `echelle` vaut 0, si `reference` n'est pas un nombre fini `> 0.0`, ou si `conductivites.len()` n'est pas le
/// nombre d'arêtes du graphe.
pub fn apercu(graphe: &Graphe, conductivites: &[f64], reference: f64, echelle: u32) -> Image {
    assert!(echelle > 0, "l'échelle doit être strictement positive");
    assert!(reference.is_finite() && reference > 0.0, "la référence doit être un nombre fini strictement positif");
    assert_eq!(conductivites.len(), graphe.nb_aretes(), "il faut une conductivité par arête");

    // Le cadre, sur tous les nœuds, le premier compris.
    let noeuds = graphe.noeuds();
    let premier = noeuds[0];
    let mut xmin = premier.x as i64;
    let mut xmax = xmin;
    let mut ymin = premier.y as i64;
    let mut ymax = ymin;
    let mut emax = premier.etage;
    for noeud in &noeuds[1..] {
        let x = noeud.x as i64;
        let y = noeud.y as i64;
        xmin = xmin.min(x);
        xmax = xmax.max(x);
        ymin = ymin.min(y);
        ymax = ymax.max(y);
        emax = emax.max(noeud.etage);
    }
    let tl = 2 * (xmax - xmin) + 1;
    let th = 2 * (ymax - ymin) + 1;
    let largeur = (emax as i64 + 1) * tl + emax as i64 * ECART as i64;
    let pas = tl + ECART as i64;
    let l = largeur as usize;

    // Le niveau de chaque arête et les deux maxima de chaque nœud.
    let mut niv = Vec::with_capacity(conductivites.len());
    for &d in conductivites {
        niv.push(niveau(d, reference));
    }
    let mut du_noeud = vec![0_u8; graphe.nb_noeuds()];
    let mut du_lien: Vec<Option<u8>> = vec![None; graphe.nb_noeuds()];
    for (e, arete) in graphe.aretes().iter().enumerate() {
        du_noeud[arete.a] = du_noeud[arete.a].max(niv[e]);
        du_noeud[arete.b] = du_noeud[arete.b].max(niv[e]);
        if noeuds[arete.a].etage != noeuds[arete.b].etage {
            du_lien[arete.a] = Some(du_lien[arete.a].map_or(niv[e], |m| m.max(niv[e])));
            du_lien[arete.b] = Some(du_lien[arete.b].map_or(niv[e], |m| m.max(niv[e])));
        }
    }

    // 1. Tout en FOND.
    let mut points = vec![FOND; l * th as usize];

    // 2. Le point du milieu de chaque arête d'un même étage et voisine, le plus épais l'emportant.
    let mut milieu = vec![0_u8; l * th as usize];
    for (e, arete) in graphe.aretes().iter().enumerate() {
        let a = noeuds[arete.a];
        let b = noeuds[arete.b];
        if a.etage != b.etage || (a.x as i64 - b.x as i64).abs() > 1 || (a.y as i64 - b.y as i64).abs() > 1 {
            continue;
        }
        let col = a.etage as i64 * pas + (a.x as i64 + b.x as i64 - 2 * xmin);
        let ligne = a.y as i64 + b.y as i64 - 2 * ymin;
        let i = ligne as usize * l + col as usize;
        milieu[i] = milieu[i].max(niv[e]);
    }
    for (i, &n) in milieu.iter().enumerate() {
        points[i] = melange(FOND, VIF, n);
    }

    // 3. Le point de chaque nœud, par indice croissant.
    for (i, noeud) in noeuds.iter().enumerate() {
        let col = noeud.etage as i64 * pas + 2 * (noeud.x as i64 - xmin);
        let ligne = 2 * (noeud.y as i64 - ymin);
        points[ligne as usize * l + col as usize] = melange(FOND, VIF, du_noeud[i]);
    }

    // 4. Les bouts des liens entre étages, vers la teinte des puits.
    for (i, noeud) in noeuds.iter().enumerate() {
        let Some(n) = du_lien[i] else { continue };
        let col = noeud.etage as i64 * pas + 2 * (noeud.x as i64 - xmin);
        let ligne = 2 * (noeud.y as i64 - ymin);
        let j = ligne as usize * l + col as usize;
        points[j] = melange(points[j], PUITS, n);
    }

    // L'agrandissement : chaque point devient un carré de `echelle × echelle` pixels.
    let grand_largeur = l * echelle as usize;
    let grand_hauteur = th as usize * echelle as usize;
    let mut rvb = Vec::with_capacity(3 * grand_largeur * grand_hauteur);
    for ligne in 0..th as usize {
        let mut rangee = Vec::with_capacity(3 * grand_largeur);
        for &couleur in &points[ligne * l..(ligne + 1) * l] {
            for _ in 0..echelle {
                rangee.extend_from_slice(&couleur);
            }
        }
        for _ in 0..echelle {
            rvb.extend_from_slice(&rangee);
        }
    }

    Image { largeur: grand_largeur as u32, hauteur: grand_hauteur as u32, rvb }
}

/// Le niveau d'une conductivité `d` pour une référence `reference` (voir « L'aperçu des tubes »).
fn niveau(d: f64, reference: f64) -> u8 {
    let t = d / reference;
    if t >= 1.0 {
        255
    } else if t > 0.0 {
        (t * 255.0).round() as u8
    } else {
        0
    }
}

/// Le mélange entier de deux couleurs, canal par canal, au niveau `n` (voir « L'aperçu des tubes »).
fn melange(a: [u8; 3], b: [u8; 3], n: u8) -> [u8; 3] {
    let n = u32::from(n);
    let canal = |a: u8, b: u8| -> u8 { ((u32::from(a) * (255 - n) + u32::from(b) * n + 127) / 255) as u8 };
    [canal(a[0], b[0]), canal(a[1], b[1]), canal(a[2], b[2])]
}
