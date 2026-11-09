//! Les graphes : nœuds, arêtes à longueur et coût, grilles à un ou plusieurs étages (étape 0).
//!
//! Le blob est un réseau de tubes : un graphe non orienté. Ce module ne simule rien ; il range les nœuds et les arêtes
//! dans un **ordre fixe**, que le solveur, l'empreinte et le rendu suivent tous.
//!
//! # Le graphe
//!
//! - Les **nœuds** sont numérotés de 0 à `n - 1`, dans l'ordre donné à [`Graphe::new`]. Un nœud a une position entière
//!   `(x, y)` et un `etage` : ils servent au dessin et à la géométrie, pas au graphe lui-même (rien n'interdit deux
//!   nœuds à la même position).
//! - Une **arête** relie deux nœuds différents `a` et `b`, et porte deux nombres strictement positifs et finis : sa
//!   `longueur` (géométrique : ce qu'on mesure et qu'on dessine) et son `cout` (ce que le flux paie pour la traverser ;
//!   sur une grille, le coût d'une arête d'un étage est sa longueur, et le coût d'un lien vertical se règle à part).
//!   Il n'y a **ni boucle** (`a == b`) **ni arête double** (deux arêtes sur la même paire de nœuds).
//! - **L'ordre des arêtes est canonique** : chaque arête est rangée avec `a < b` (les deux bouts donnés dans l'autre
//!   sens sont échangés), et les arêtes sont triées par `(a, b)` croissant (d'abord `a`, puis `b`). L'**indice** d'une
//!   arête est son rang dans cet ordre, à partir de 0. L'ordre dans lequel les arêtes ont été données ne compte pas.
//!
//! # La grille
//!
//! Une [`Grille`] décrit `etages` étages identiques de `largeur × hauteur` nœuds. Le nœud de la colonne `x`
//! (`0 <= x < largeur`), de la ligne `y` (`0 <= y < hauteur`) et de l'étage `e` (`0 <= e < etages`) a pour indice
//!
//! ```text
//! (y * largeur + x) * etages + e
//! ```
//!
//! (les étages d'une même case se suivent : le solveur y gagne un profil étroit) et pour position
//! `Noeud { x, y, etage: e }`. Ses arêtes sont, pour chaque étage :
//! - `(x, y, e)`–`(x + 1, y, e)` et `(x, y, e)`–`(x, y + 1, e)` : longueur `1.0`, coût `1.0` ;
//! - si le voisinage est [`Voisinage::Huit`], en plus les deux diagonales de chaque maille, `(x, y, e)`–`(x + 1, y + 1, e)`
//!   et `(x + 1, y, e)`–`(x, y + 1, e)` : longueur et coût `2.0_f64.sqrt()` (bits `0x3FF6A09E667F3BCD`) ;
//!
//! et, entre deux étages qui se suivent, un **lien vertical** par case, `(x, y, e)`–`(x, y, e + 1)`, de longueur
//! `longueur_verticale` et de coût `cout_vertical`. Il n'y a pas d'autre arête.
//!
//! # L'écriture canonique
//!
//! [`Graphe::octets`] rend cette suite d'octets, tous les entiers en petit-boutiste :
//!
//! ```text
//! 4 octets          n, le nombre de nœuds, en u32
//! 12 octets × n     chaque nœud dans l'ordre des indices : x en i32, y en i32, etage en u32
//! 4 octets          m, le nombre d'arêtes, en u32
//! 24 octets × m     chaque arête dans l'ordre des indices : a en u32, b en u32,
//!                   longueur.to_bits() en u64, cout.to_bits() en u64
//! ```
//!
//! soit `8 + 12 n + 24 m` octets (les indices et les comptes sont convertis par `as u32` : un graphe de plus de
//! `u32::MAX` nœuds ou arêtes n'est pas pris en charge).

/// Un nœud : sa position entière et son étage (voir la documentation du module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Noeud {
    /// La colonne.
    pub x: i32,
    /// La ligne.
    pub y: i32,
    /// L'étage (0 pour un graphe à un seul étage).
    pub etage: u32,
}

/// Une arête : ses deux bouts (des indices de nœuds), sa longueur et son coût. Dans un [`Graphe`], `a < b` toujours.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arete {
    /// Un bout.
    pub a: usize,
    /// L'autre bout.
    pub b: usize,
    /// La longueur géométrique, strictement positive et finie.
    pub longueur: f64,
    /// Le coût de traversée, strictement positif et fini.
    pub cout: f64,
}

/// Pourquoi un graphe n'a pas pu être construit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurGraphe {
    /// Aucun nœud.
    Vide,
    /// L'arête de ce rang **dans la liste donnée** (à partir de 0) est fautive ; c'est la première fautive de la liste
    /// (voir [`Graphe::new`]).
    Arete(usize),
    /// La description de la grille est fautive (voir [`Grille::graphe`]).
    Grille,
}

/// Un graphe non orienté, rangé dans l'ordre canonique (voir la documentation du module).
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub struct Graphe {
    noeuds: Vec<Noeud>,
    aretes: Vec<Arete>,
    /// `incidences[debuts[i]..debuts[i + 1]]` : les indices des arêtes du nœud `i`, croissants.
    debuts: Vec<usize>,
    incidences: Vec<usize>,
}

impl Graphe {
    /// Construit un graphe. Les nœuds gardent leur ordre ; les arêtes sont rangées dans l'ordre canonique (`a < b`,
    /// triées par `(a, b)`).
    ///
    /// # Erreurs
    ///
    /// - `ErreurGraphe::Vide` si `noeuds` est vide (même si des arêtes sont données).
    /// - Sinon `ErreurGraphe::Arete(i)`, où `i` est le plus petit rang, dans la liste `aretes` telle que donnée, d'une
    ///   arête fautive. Une arête est fautive si : ses deux bouts sont le même nœud ; ou un de ses bouts n'est pas un
    ///   indice de nœud (`>= n`) ; ou sa `longueur` ou son `cout` n'est pas un nombre fini strictement positif (zéro,
    ///   un négatif, un infini, NaN sont refusés) ; ou une arête **de rang plus petit** dans la liste relie la même
    ///   paire de nœuds, dans un sens ou dans l'autre (c'est la seconde des deux qui est fautive, que la première soit
    ///   elle-même fautive ou non).
    #[allow(unused_variables)]
    pub fn new(noeuds: Vec<Noeud>, aretes: Vec<Arete>) -> Result<Graphe, ErreurGraphe> {
        todo!()
    }

    /// Le nombre de nœuds.
    pub fn nb_noeuds(&self) -> usize {
        todo!()
    }

    /// Le nombre d'arêtes.
    pub fn nb_aretes(&self) -> usize {
        todo!()
    }

    /// Les nœuds, dans l'ordre de leurs indices.
    pub fn noeuds(&self) -> &[Noeud] {
        todo!()
    }

    /// Les arêtes, dans l'ordre canonique (celui de leurs indices) ; chacune a `a < b`.
    pub fn aretes(&self) -> &[Arete] {
        todo!()
    }

    /// Les indices des arêtes dont `noeud` est un bout, **par indice croissant**. Vide pour un nœud sans arête.
    ///
    /// # Panique
    ///
    /// Si `noeud` n'est pas un indice de nœud.
    #[allow(unused_variables)]
    pub fn incidentes(&self, noeud: usize) -> &[usize] {
        todo!()
    }

    /// L'indice de l'arête qui relie `a` et `b` (dans un sens ou dans l'autre), ou `None` s'il n'y en a pas, si
    /// `a == b`, ou si l'un des deux n'est pas un indice de nœud. Ne panique jamais.
    #[allow(unused_variables)]
    pub fn arete_entre(&self, a: usize, b: usize) -> Option<usize> {
        todo!()
    }

    /// Vrai si tout nœud se joint à tout autre en suivant des arêtes (un graphe d'un seul nœud est connexe).
    pub fn est_connexe(&self) -> bool {
        todo!()
    }

    /// La somme des longueurs des arêtes : on part de `0.0` et on ajoute la longueur de chaque arête **dans l'ordre des
    /// indices** (une somme de flottants dépend de son ordre : ni tri, ni regroupement). `0.0` sans arête.
    pub fn longueur_totale(&self) -> f64 {
        todo!()
    }

    /// L'écriture canonique du graphe (voir « L'écriture canonique » dans la documentation du module).
    pub fn octets(&self) -> Vec<u8> {
        todo!()
    }
}

/// Le voisinage d'un nœud dans son étage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Voisinage {
    /// Les quatre voisins : gauche, droite, haut, bas.
    Quatre,
    /// Les huit voisins : les quatre, plus les quatre diagonales.
    Huit,
}

/// La description d'une grille à un ou plusieurs étages (voir « La grille » dans la documentation du module).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grille {
    /// Le nombre de colonnes, au moins 1.
    pub largeur: u32,
    /// Le nombre de lignes, au moins 1.
    pub hauteur: u32,
    /// Le nombre d'étages, au moins 1.
    pub etages: u32,
    /// Le voisinage dans un étage.
    pub voisinage: Voisinage,
    /// La longueur d'un lien vertical, strictement positive et finie (lue même s'il n'y a qu'un étage).
    pub longueur_verticale: f64,
    /// Le coût d'un lien vertical, strictement positif et fini (lu même s'il n'y a qu'un étage).
    pub cout_vertical: f64,
}

impl Grille {
    /// L'indice du nœud de la colonne `x`, de la ligne `y` et de l'étage `etage` : `(y * largeur + x) * etages + etage`.
    ///
    /// # Panique
    ///
    /// Si `x >= largeur`, `y >= hauteur` ou `etage >= etages`.
    #[allow(unused_variables)]
    pub fn indice(&self, x: u32, y: u32, etage: u32) -> usize {
        todo!()
    }

    /// Le graphe de la grille (voir « La grille » dans la documentation du module).
    ///
    /// # Erreurs
    ///
    /// `ErreurGraphe::Grille` si `largeur`, `hauteur` ou `etages` vaut 0, ou si `longueur_verticale` ou `cout_vertical`
    /// n'est pas un nombre fini strictement positif (même quand il n'y a qu'un étage).
    pub fn graphe(&self) -> Result<Graphe, ErreurGraphe> {
        todo!()
    }
}
