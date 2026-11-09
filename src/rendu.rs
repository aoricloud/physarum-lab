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

/// Les dimensions `(TL, th)` du treillis de ce graphe, en points (voir la documentation du module).
#[allow(unused_variables)]
pub fn treillis(graphe: &Graphe) -> (u32, u32) {
    todo!()
}

/// L'aperçu d'un graphe, agrandi `echelle` fois (voir la documentation du module).
///
/// # Panique
///
/// Si `echelle` vaut 0.
#[allow(unused_variables)]
pub fn apercu_graphe(graphe: &Graphe, echelle: u32) -> Image {
    todo!()
}
