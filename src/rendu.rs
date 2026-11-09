//! Les aperçus : des images pour l'œil, jamais une preuve (étape 0, puis chaque étape). Une image est une suite d'octets
//! RVB, ligne par ligne de haut en bas, trois octets par pixel (R, V, B) : c'est ce que prend `image::ecrire_png`.
//!
//! # L'aperçu d'un graphe : les étages côte à côte
//!
//! Soit `xmin`, `xmax`, `ymin`, `ymax` les bornes des positions de **tous** les nœuds du graphe, et `emax` le plus grand
//! étage. On dessine d'abord sur un **treillis** de points, puis on agrandit.
//!
//! - Chaque étage `e` de 0 à `emax` (même sans nœud) a un **panneau** de `tl = 2 * (xmax - xmin) + 1` points de large et
//!   `th = 2 * (ymax - ymin) + 1` points de haut. Les panneaux sont posés côte à côte, de gauche à droite par étage
//!   croissant, séparés par [`ECART`] colonnes de fond : le panneau `e` commence à la colonne `e * (tl + ECART)`. Le
//!   treillis entier fait donc `TL = (emax + 1) * tl + emax * ECART` points de large et `th` de haut.
//! - Dans son panneau, le nœud de position `(x, y)` est au point `(2 * (x - xmin), 2 * (y - ymin))` (colonne, ligne ;
//!   la ligne 0 est en haut). Entre deux nœuds voisins il reste donc un point libre : celui de leur arête.
//!
//! On peint le treillis dans cet ordre, chaque couche recouvrant les précédentes :
//! 1. tout en [`FOND`] ;
//! 2. chaque arête dont les deux bouts sont **au même étage** et dont les positions diffèrent d'au plus 1 en `x` et
//!    d'au plus 1 en `y` : le point du milieu, `(xa + xb - 2 * xmin, ya + yb - 2 * ymin)` dans le panneau de cet étage,
//!    en [`ARETE`] (les deux diagonales d'une maille se croisent au même point) ; une arête plus longue dans un étage
//!    n'est pas dessinée ;
//! 3. le point de chaque nœud, en [`NOEUD`] ;
//! 4. pour chaque arête dont les deux bouts sont à **deux étages différents** (un puits) : le point de chacun de ses
//!    deux bouts, chacun dans le panneau de son propre étage, en [`PUITS`].
//!
//! Enfin chaque point du treillis devient un carré de `echelle × echelle` pixels : l'image fait `TL * echelle` pixels
//! de large et `th * echelle` de haut, et le pixel `(px, py)` a la couleur du point `(px / echelle, py / echelle)`
//! (divisions entières).

use crate::graphe::Graphe;

/// La couleur du fond.
pub const FOND: [u8; 3] = [12, 16, 24];
/// La couleur du point d'une arête.
pub const ARETE: [u8; 3] = [90, 110, 130];
/// La couleur du point d'un nœud.
pub const NOEUD: [u8; 3] = [225, 225, 215];
/// La couleur du point d'un nœud d'où part un lien vers un autre étage.
pub const PUITS: [u8; 3] = [240, 170, 40];
/// Le nombre de colonnes de fond entre deux panneaux.
pub const ECART: u32 = 2;

/// Une image RVB : `rvb` a `3 * largeur * hauteur` octets, ligne par ligne de haut en bas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    /// La largeur en pixels.
    pub largeur: u32,
    /// La hauteur en pixels.
    pub hauteur: u32,
    /// Les pixels, trois octets chacun (R, V, B).
    pub rvb: Vec<u8>,
}

/// Le cadre des positions de tous les nœuds : `(xmin, ymin, xmax, ymax, emax)`.
///
/// Les bornes sont calculées en `i64` (elles peuvent être négatives), et `emax` est le plus grand étage du graphe.
fn cadre(graphe: &Graphe) -> (i64, i64, i64, i64, u32) {
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
    (xmin, ymin, xmax, ymax, emax)
}

/// Les dimensions `(TL, th)` du treillis de ce graphe, en points (voir la documentation du module).
pub fn treillis(graphe: &Graphe) -> (u32, u32) {
    let (xmin, ymin, xmax, ymax, emax) = cadre(graphe);
    let tl = 2 * (xmax - xmin) + 1;
    let th = 2 * (ymax - ymin) + 1;
    let largeur = (emax as i64 + 1) * tl + emax as i64 * ECART as i64;
    (largeur as u32, th as u32)
}

/// L'aperçu d'un graphe, agrandi `echelle` fois (voir la documentation du module).
///
/// # Panique
///
/// Si `echelle` vaut 0.
pub fn apercu_graphe(graphe: &Graphe, echelle: u32) -> Image {
    assert!(echelle > 0, "l'échelle doit être strictement positive");
    let (xmin, ymin, xmax, ymax, emax) = cadre(graphe);
    let tl = 2 * (xmax - xmin) + 1;
    let th = 2 * (ymax - ymin) + 1;
    let largeur = (emax as i64 + 1) * tl + emax as i64 * ECART as i64;
    let pas = tl + ECART as i64;
    let l = largeur as usize;

    // Le treillis, une couleur par point, dans l'ordre des couches de la documentation du module.
    let mut points = vec![FOND; l * th as usize];
    let point_de = |noeud: &crate::graphe::Noeud| -> (i64, i64) {
        (noeud.etage as i64 * pas + 2 * (noeud.x as i64 - xmin), 2 * (noeud.y as i64 - ymin))
    };

    // 2. Le point du milieu de chaque arête dont les deux bouts sont au même étage et voisins.
    for arete in graphe.aretes() {
        let a = graphe.noeuds()[arete.a];
        let b = graphe.noeuds()[arete.b];
        if a.etage != b.etage || (a.x as i64 - b.x as i64).abs() > 1 || (a.y as i64 - b.y as i64).abs() > 1 {
            continue;
        }
        let col = a.etage as i64 * pas + (a.x as i64 + b.x as i64 - 2 * xmin);
        let ligne = a.y as i64 + b.y as i64 - 2 * ymin;
        points[ligne as usize * l + col as usize] = ARETE;
    }

    // 3. Le point de chaque nœud.
    for noeud in graphe.noeuds() {
        let (col, ligne) = point_de(noeud);
        points[ligne as usize * l + col as usize] = NOEUD;
    }

    // 4. Les deux bouts de chaque lien entre étages, chacun dans le panneau de son étage.
    for arete in graphe.aretes() {
        let a = graphe.noeuds()[arete.a];
        let b = graphe.noeuds()[arete.b];
        if a.etage == b.etage {
            continue;
        }
        for noeud in [a, b] {
            let (col, ligne) = point_de(&noeud);
            points[ligne as usize * l + col as usize] = PUITS;
        }
    }

    // L'agrandissement : chaque point du treillis devient un carré de `echelle × echelle` pixels.
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
