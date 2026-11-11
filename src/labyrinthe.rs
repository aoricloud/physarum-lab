//! Les murs et les étages d'une grille : le labyrinthe de Nakagaki et les puits entre deux étages (étape 1, partie 1L).
//!
//! Ce module ne simule rien. Il **construit le graphe** sur lequel `crate::tubes` simule : une grille (`Grille`) dont
//! on retire des cellules (les murs), dont on ne garde que la partie reliée au puits, et dont chaque étage peut être
//! plus « rapide » qu'un autre. Les valeurs attendues des tests, chacune avec sa source, sont dans
//! `references/etape-1l-valeurs.md` (identifiants `V-LAB-xx`, `V-NAK-xx`, `V-ETA-xx`, `V-LLAB-xx`, `V-APL-xx`).
//!
//! # Ce que c'est, et d'où ça vient
//!
//! Nakagaki, Yamada et Tóth (Nature 407, 2000, p. 470) font traverser un labyrinthe au plasmode : il en explore tous
//! les couloirs, puis ne garde que le plus court chemin. Notre labyrinthe est **un dessin à nous** (l'article n'en
//! donne que les longueurs, voir `MODELE.md`) : un graphe de grille en voisinage 4 où chaque couloir est d'une cellule
//! de large. Les murs, les étages plus ou moins chers et la correspondance avec la grille sont des choix de ce dépôt.
//! [`construire`] accepte aussi le voisinage 8 et ne lui fait aucun cas particulier : une diagonale entre deux cellules
//! gardées reste, même quand les deux cellules du coin qu'elle traverse sont murées (un mur ne coupe que ses cellules).
//!
//! # Les murs : on retire des cellules, pas des arêtes
//!
//! Un [`Mur`] est un rectangle de cellules d'un étage, bornes comprises : `x0 <= x <= x1`, `y0 <= y <= y1`. Un mur
//! retire des **cellules** (des nœuds, avec toutes leurs arêtes) : retirer seulement des arêtes laisserait un nœud sans
//! arête, que le solveur refuse (`ErreurSolveur::Isole`). Deux murs qui se recouvrent retirent l'union de leurs
//! cellules (rien de plus). Un mur d'une seule cellule (`x0 == x1`, `y0 == y1`) est permis, et un mur peut couvrir un
//! étage entier. Un mur ne retire que les cellules de son étage : la cellule au-dessus ou au-dessous reste.
//!
//! # La construction, dans cet ordre
//!
//! [`construire`] fait, pour une grille `grille`, des murs, des coûts d'étage, une `source` et un `puits` (deux indices
//! de nœud **de la grille**, `Grille::indice`) :
//!
//! 1. le graphe complet `grille.graphe()` (`G`) ;
//! 2. les cellules murées sont retirées ;
//! 3. **seule la composante connexe du puits est gardée** : tout ce qui n'est pas relié au puits par des cellules non
//!    murées disparaît aussi, avec ses arêtes (une poche fermée par des murs, une cellule isolée, une impasse coupée du
//!    reste). La source doit y être ;
//! 4. les nœuds gardés sont **renumérotés dans l'ordre croissant de leur indice de grille** (`0, 1, 2, …`) ; un nœud
//!    garde sa position `(x, y)` et son étage : le dessin ne bouge pas, un mur y est un trou ;
//! 5. les arêtes gardées sont celles de `G` dont les **deux** bouts sont gardés ; elles gardent leur longueur ; leur
//!    coût est celui de `G` pour un lien entre étages, et `cout * facteur(e)` (une seule multiplication, dans cet
//!    ordre) pour une arête dont les deux bouts sont dans le même étage `e` (voir « Le coût d'un étage ») ;
//! 6. le graphe est bâti par `Graphe::new` : l'ordre des arêtes est l'ordre canonique (`a < b`, triées par `(a, b)`).
//!    Comme la renumérotation **conserve l'ordre** des nœuds, les arêtes gardées sont rangées **dans le même ordre
//!    relatif** que dans `G` : la `k`-ième arête du plan est la `k`-ième arête gardée de `G` (les tests le vérifient).
//!
//! Une grille **sans mur ni coût d'étage** donne exactement `grille.graphe()` (mêmes nœuds, mêmes arêtes, mêmes octets
//! de `Graphe::octets`), à condition que la source et le puits soient deux nœuds de la grille. La correspondance est
//! `Plan::noeuds[i]` : `Some(j)` si le nœud `i` de la grille est gardé (il porte le numéro `j` dans le graphe du plan),
//! `None` s'il est muré ou séparé du puits ; `Plan::noeuds` a `grille.graphe().nb_noeuds()` valeurs, et ses `Some` sont
//! `0, 1, 2, …` dans l'ordre.
//!
//! # Le coût d'un étage
//!
//! Un [`CoutEtage`] donne un **facteur** à un étage `e` (fini, dans `[FACTEUR_MIN ; FACTEUR_MAX]`, bornes comprises) :
//! le coût de chaque arête dont les deux bouts sont dans l'étage `e` est multiplié par ce facteur (`1.0` pour un étage
//! qui n'en a pas : `cout * 1.0` rend les mêmes bits). La **longueur** ne change pas (la longueur sert à mesurer et à
//! dessiner), et un **lien entre étages garde** le coût `cout_vertical` de la grille. Un étage de facteur `f < 1` est
//! plus rapide qu'un autre : un flux y paie moins ; c'est ce qui rend un puits (un lien entre étages) utile ou non.
//!
//! # Les erreurs, dans cet ordre
//!
//! [`construire`] vérifie, et rend la **première** erreur de cette liste ; il ne panique sur aucune entrée.
//!
//! 1. `ErreurLabyrinthe::Grille` : `grille.graphe()` échoue ;
//! 2. `ErreurLabyrinthe::Mur(i)` : `i` est le plus petit rang, dans `murs`, d'un mur fautif : `x0 > x1`, ou `y0 > y1`,
//!    ou `x1 >= largeur`, ou `y1 >= hauteur`, ou `etage >= etages` ;
//! 3. `ErreurLabyrinthe::Facteur(i)` : `i` est le plus petit rang, dans `couts`, d'un coût fautif : un étage `>=
//!    etages`, ou un facteur hors de `[FACTEUR_MIN ; FACTEUR_MAX]` (NaN et l'infini compris), ou un étage **déjà
//!    donné** par un rang plus petit (c'est le second qui est fautif, même si son facteur est bon ; comme le plus
//!    petit rang fautif est rendu, un rang plus petit qui « donne » l'étage est forcément un coût bon) ;
//! 4. `ErreurLabyrinthe::Noeud` : `source` ou `puits` n'est pas un indice de nœud de la grille (il vaut au moins
//!    `largeur * hauteur * etages`), ou les deux sont le même ;
//! 5. `ErreurLabyrinthe::PuitsMure` : le puits est sous un mur ;
//! 6. `ErreurLabyrinthe::SourceMuree` : la source est sous un mur ;
//! 7. `ErreurLabyrinthe::SourceCoupee` : la source n'est pas dans la composante connexe du puits.
//!
//! `Graphe::new` ne peut pas échouer sur le graphe ainsi bâti (des arêtes de `G`, sans doublon ni boucle, et au moins
//! le puits) ; s'il échouait quand même, [`construire`] rendrait `ErreurLabyrinthe::Grille`.
//!
//! # Les bornes des facteurs
//!
//! `FACTEUR_MIN = 0.1` et `FACTEUR_MAX = 10.0` : au-delà d'un rapport de `10^10` entre deux conductances, le solveur
//! perd sa précision sans le dire (voir « Ce que le solveur ne garantit pas » dans `crate::solveur`). Les scénarios du
//! dépôt gardent aussi leur coût vertical dans `[0,1 ; 10]` : c'est une règle des scénarios que les tests vérifient sur
//! les fichiers de `scenarios/`, pas une vérification de `lancer` (qui accepte un coût vertical plus étalé et rend
//! `Calcul(pas)` quand le calcul s'arrête).

use crate::graphe::{Arete, Graphe, Grille};

/// Le plus petit facteur d'étage permis : `0.1`.
pub const FACTEUR_MIN: f64 = 0.1;

/// Le plus grand facteur d'étage permis : `10.0`.
pub const FACTEUR_MAX: f64 = 10.0;

/// Un mur : un rectangle de cellules d'un étage, bornes comprises (voir « Les murs » dans la documentation du module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Mur {
    /// La première colonne.
    pub x0: u32,
    /// La première ligne.
    pub y0: u32,
    /// La dernière colonne (au moins `x0`).
    pub x1: u32,
    /// La dernière ligne (au moins `y0`).
    pub y1: u32,
    /// L'étage du mur.
    pub etage: u32,
}

/// Le facteur de coût d'un étage (voir « Le coût d'un étage » dans la documentation du module).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoutEtage {
    /// L'étage.
    pub etage: u32,
    /// Le facteur, dans `[FACTEUR_MIN ; FACTEUR_MAX]`.
    pub facteur: f64,
}

/// Pourquoi un plan n'a pas pu être construit (voir « Les erreurs » dans la documentation du module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurLabyrinthe {
    /// La description de la grille est fautive (`Grille::graphe` échoue).
    Grille,
    /// Le mur de ce rang (dans la liste donnée, à partir de 0) est fautif : le plus petit rang fautif.
    Mur(usize),
    /// Le coût d'étage de ce rang (dans la liste donnée, à partir de 0) est fautif : le plus petit rang fautif.
    Facteur(usize),
    /// `source` ou `puits` n'est pas un nœud de la grille, ou les deux sont le même nœud.
    Noeud,
    /// Le puits est sous un mur.
    PuitsMure,
    /// La source est sous un mur.
    SourceMuree,
    /// La source n'est pas dans la composante connexe du puits.
    SourceCoupee,
}

/// Ce que [`construire`] rend : le graphe, la correspondance avec la grille, la source et le puits renumérotés.
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    /// Le graphe : les cellules gardées, renumérotées dans l'ordre de leur indice de grille.
    pub graphe: Graphe,
    /// Pour chaque nœud de la grille (dans l'ordre de ses indices) : son numéro dans `graphe`, ou `None` s'il est muré
    /// ou séparé du puits.
    pub noeuds: Vec<Option<usize>>,
    /// L'indice de la source **dans `graphe`**.
    pub source: usize,
    /// L'indice du puits **dans `graphe`**.
    pub puits: usize,
}

/// Construit le plan : la grille, ses murs retirés, la composante du puits gardée, les étages mis à leur coût (voir «
/// La construction » dans la documentation du module). `source` et `puits` sont des indices de la grille.
///
/// # Erreurs
///
/// Celles de « Les erreurs », dans leur ordre : la première qui s'applique.
pub fn construire(
    grille: &Grille,
    murs: &[Mur],
    couts: &[CoutEtage],
    source: usize,
    puits: usize,
) -> Result<Plan, ErreurLabyrinthe> {
    let complet = grille.graphe().map_err(|_| ErreurLabyrinthe::Grille)?;
    let n = complet.nb_noeuds();

    for (i, mur) in murs.iter().enumerate() {
        if mur.x0 > mur.x1
            || mur.y0 > mur.y1
            || mur.x1 >= grille.largeur
            || mur.y1 >= grille.hauteur
            || mur.etage >= grille.etages
        {
            return Err(ErreurLabyrinthe::Mur(i));
        }
    }

    let mut facteurs: Vec<f64> = vec![1.0; grille.etages as usize];
    let mut donne: Vec<bool> = vec![false; grille.etages as usize];
    for (i, c) in couts.iter().enumerate() {
        if c.etage >= grille.etages {
            return Err(ErreurLabyrinthe::Facteur(i));
        }
        let e = c.etage as usize;
        if !(c.facteur >= FACTEUR_MIN && c.facteur <= FACTEUR_MAX) || donne[e] {
            return Err(ErreurLabyrinthe::Facteur(i));
        }
        donne[e] = true;
        facteurs[e] = c.facteur;
    }

    if source >= n || puits >= n || source == puits {
        return Err(ErreurLabyrinthe::Noeud);
    }

    let mut mure = vec![false; n];
    for mur in murs {
        for y in mur.y0..=mur.y1 {
            for x in mur.x0..=mur.x1 {
                mure[grille.indice(x, y, mur.etage)] = true;
            }
        }
    }
    if mure[puits] {
        return Err(ErreurLabyrinthe::PuitsMure);
    }
    if mure[source] {
        return Err(ErreurLabyrinthe::SourceMuree);
    }

    let mut garde = vec![false; n];
    garde[puits] = true;
    let mut pile = vec![puits];
    while let Some(u) = pile.pop() {
        for &k in complet.incidentes(u) {
            let a = &complet.aretes()[k];
            let v = if a.a == u { a.b } else { a.a };
            if !mure[v] && !garde[v] {
                garde[v] = true;
                pile.push(v);
            }
        }
    }
    if !garde[source] {
        return Err(ErreurLabyrinthe::SourceCoupee);
    }

    let mut noeuds: Vec<Option<usize>> = vec![None; n];
    let mut gardes = Vec::new();
    for (i, nd) in complet.noeuds().iter().enumerate() {
        if garde[i] {
            noeuds[i] = Some(gardes.len());
            gardes.push(*nd);
        }
    }

    let mut aretes = Vec::new();
    for a in complet.aretes() {
        if let (Some(na), Some(nb)) = (noeuds[a.a], noeuds[a.b]) {
            let ea = complet.noeuds()[a.a].etage;
            let eb = complet.noeuds()[a.b].etage;
            let cout = if ea == eb { a.cout * facteurs[ea as usize] } else { a.cout };
            aretes.push(Arete { a: na, b: nb, longueur: a.longueur, cout });
        }
    }

    let graphe = Graphe::new(gardes, aretes).map_err(|_| ErreurLabyrinthe::Grille)?;
    match (noeuds[source], noeuds[puits]) {
        (Some(s), Some(p)) => Ok(Plan { graphe, noeuds, source: s, puits: p }),
        _ => Err(ErreurLabyrinthe::Grille),
    }
}
