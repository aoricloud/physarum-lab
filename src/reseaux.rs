//! Les réseaux entre plusieurs sources de nourriture : la loi sigmoïde, la paire tirée à chaque pas, l'arbre couvrant
//! minimal et les trois mesures de Tero (étape 1, partie « les réseaux »). Ce que les tests d'acceptation prouvent est
//! dans `references/etape-1r-valeurs.md` (identifiants `V-LOI-xx`, `V-TIR-xx`, `V-MST-xx`, `V-MES-xx`, `V-ESP-xx`,
//! `V-LRES-xx`, `V-APR-xx`).
//!
//! # Ce que c'est, et d'où ça vient
//!
//! Tero et al. (Science 327, 2010, p. 441) font construire au plasmode un réseau entre des sources de nourriture (les
//! villes de la région de Tokyo) : à **chaque pas de temps**, une source de nourriture tirée au hasard fait circuler le
//! flux `I0` et une seconde, tirée au hasard, est le puits ; chaque tube s'adapte, `dD/dt = f(|Q|) - D`, avec la
//! sigmoïde `f(Q) = Q^γ / (1 + Q^γ)`. Plus `I0` est grand, plus le réseau garde de routes de rechange (plus de tubes,
//! meilleure tenue à une panne, coût plus élevé) ; à `I0` faible il tend vers l'arbre couvrant minimal.
//!
//! - **Ce qu'on a lu** (`references/etape-1r-valeurs.md`, section « Sources ») : l'article principal et le supplément
//!   de Tero 2010 (la paire tirée à chaque pas, la forme de `f`, `γ = 1,8` et `I0 = 2` pour une simulation typique,
//!   `I0` de `0,2` à `7,2` à la Fig. 3A, les trois mesures **en mots**) ; Tero, Kobayashi et Nakagaki 2007 (la loi
//!   sigmoïde de type II, le pas semi-implicite).
//! - **Ce qui n'est écrit nulle part dans ce qu'on a lu** : le maillage, le nombre de nœuds, le pas de temps, le schéma
//!   d'intégration, les conductivités de départ, la formule des trois mesures. **Tout cela est à nous** (dit dans
//!   `MODELE.md`) : un pas d'Euler comme au cœur, des conductivités de départ comme au cœur, des mesures dont les
//!   formules sont écrites ici. Aucune valeur de l'article n'est visée.
//! - **`γ = 2`, pas `1,8`** : `powf` est interdit (les fonctions du système peuvent différer d'une machine à l'autre),
//!   et `Q^2` s'écrit `Q * Q`. La loi d'ici est donc `Q·Q / (1 + Q·Q)`, qui n'est **pas** celle de l'article.
//!
//! # La loi
//!
//! [`Loi::valeur`] est `f(Q)`, écrite à la lettre (le résultat est celui de ces opérations, bit pour bit) :
//!
//! ```text
//! Lineaire :  f(q) = |q|                                  (celle du cœur : `q.abs()`)
//! Sigmoide :  q2 = q * q ;  f(q) = 1.0  si q2 vaut +inf,  sinon  q2 / (1.0 + q2)
//! ```
//!
//! `NaN` rend `NaN` pour les deux. La sigmoïde est paire, `f(0) = +0.0`, `f(1) = 0.5`, croissante, et **strictement
//! inférieure à 1** tant que `|q| <= 1e6` : un tube ne devient jamais plus conducteur que `f(I0)`. La conductivité
//! s'adapte par le même pas d'Euler qu'au cœur, avec la loi choisie (`h` dans `]0, 1[`) :
//!
//! ```text
//! d    = D[e] + h * (f(Q[e]) - D[e])           (soustraction, produit, somme ; `f` = Loi::valeur)
//! D[e] = d  si d >= PLANCHER,  sinon PLANCHER   (`PLANCHER` de `crate::tubes`)
//! ```
//!
//! Avec `Loi::Lineaire`, un pas de [`Reseau::avancer_paire`] est, **bit pour bit**, un pas de `Tubes::avancer` (mêmes
//! conductances, mêmes pressions, mêmes flux, même mise à jour).
//!
//! # Le pas à plusieurs sources de nourriture
//!
//! Un [`Reseau`] a des conductivités (une par arête, comme `Tubes`), `k >= 2` **nœuds de nourriture** distincts (leur
//! ordre est celui de la liste donnée : le **rang** `i` d'une nourriture est sa place dans cette liste), un débit `I0`
//! (fini, `> 0`) et une [`Loi`]. Un pas :
//!
//! 1. **le tirage** ([`tirer_paire`]) : deux tirages, toujours (même pour `k = 2`), avec le générateur `alea` :
//!    ```text
//!    i = alea.borne(k)            le rang de la source, uniforme dans 0..k
//!    j = alea.borne(k - 1)        puis, si j >= i, j = j + 1 : le rang du puits, uniforme parmi les autres
//!    ```
//! (`k` et `k - 1` convertis en `u32` ; la paire ordonnée `(i, j)` est uniforme parmi les `k (k - 1)` paires de rangs
//! différents) ;
//! 2. **le calcul**, avec les conductivités actuelles, comme `Tubes::avancer` : `g[e] = D[e] / cout[e]`, un vecteur
//!    d'injections de `nb_noeuds` fois `+0.0` avec `inj[source] = I0` (la source est la nourriture de rang `i`), `p =
//!    solveur::pressions(graphe, g, inj, puits)` (le puits, la nourriture de rang `j`, est la masse), `Q =
//!    solveur::flux(graphe, g, p)` ;
//! 3. **la mise à jour** de toutes les conductivités, par indice croissant, après le calcul de `Q` (voir « La loi »).
//!
//! # Les conductivités de départ
//!
//! [`Reseau::au_hasard`] prend `Tubes::au_hasard(graphe, alea)` (pour chaque arête, par indice croissant, `0.5 + 0.5 *
//! suivant_f64()`), **après** avoir vérifié les nourritures et le débit. Les pas tirent ensuite dans le même `alea`.
//!
//! # Les vérifications, dans cet ordre
//!
//! - [`Reseau::new`] : `ErreurReseau::Taille` si `conductivites.len()` n'est pas `graphe.nb_aretes()` ;
//!   `ErreurReseau::Conductivite(e)` (le plus petit indice d'une conductivité qui n'est pas un nombre fini `>=
//!   PLANCHER`) ; `ErreurReseau::Nourriture` si `nourritures` a moins de deux nœuds, un indice `>= graphe.nb_noeuds()`
//!   ou deux fois le même nœud ; `ErreurReseau::Debit` si `debit` n'est pas un nombre fini `> 0.0`.
//! - [`Reseau::au_hasard`] : `Nourriture`, puis `Debit` (les mêmes règles) ; **avant** tout tirage : sur une erreur,
//!   `alea` n'a pas bougé.
//! - [`Reseau::avancer_paire`] : `Taille` (`graphe.nb_aretes()` n'est pas le nombre de conductivités) ; `Nourriture`
//!   (un nœud de nourriture `>= graphe.nb_noeuds()`) ; `Noeud` (`source` ou `puits` n'est pas un nœud du graphe, ou les
//!   deux sont le même nœud : `source` et `puits` n'ont pas besoin d'être des nourritures) ; `Pas` (`h` n'est pas un
//!   nombre fini avec `0.0 < h < 1.0`) ; puis l'erreur du solveur, `Solveur(erreur)`.
//! - [`Reseau::avancer`] : `Taille`, `Nourriture`, `Pas`, puis `Solveur(erreur)`, dans cet ordre.
//!
//! **Sur une erreur, rien n'a changé** : ni les conductivités, ni (pour `avancer`) le générateur, qui n'avance que si
//! le pas réussit.
//!
//! # L'arbre couvrant minimal des nourritures
//!
//! La **distance** de deux nourritures est la longueur du plus court chemin du graphe **par les longueurs**
//! (`Arete::longueur`, pas `Arete::cout`) : celle de `chemins::plus_courts` sur le graphe où le coût de chaque arête
//! est remplacé par sa longueur (mêmes nœuds, mêmes arêtes dans le même ordre). La distance de la nourriture de rang
//! `i` à celle de rang `j` (`i < j`) est lue dans les `distances` du résultat de `plus_courts` **depuis `i`**. L'arbre
//! ([`arbre_couvrant`]) est celui de Kruskal sur ces distances (la clôture métrique) :
//!
//! ```text
//! les paires (i, j), i < j, triées par (distance, i, j) croissant  (la distance par `<`, puis i, puis j)
//! pour chaque paire dans cet ordre : si i et j ne sont pas déjà reliés par les paires gardées, on la garde
//! ```
//!
//! Il garde `k - 1` paires. **`TL_MST`** est la somme de leurs distances, **dans l'ordre où on les a gardées, à partir
//! de `0.0`**. **`MD_MST`** est la moyenne, sur les `k (k - 1) / 2` paires `i < j`, de la distance **dans l'arbre** :
//! pour chaque `i`, on part de `d[i] = 0.0` et on ajoute, en descendant l'arbre, la distance de chaque paire gardée
//! (`d[v] = d[u] + distance(u, v)`, la somme part donc du nœud `i` ; `distance(u, v)` est celle de la paire
//! `(min(u, v), max(u, v))`, lue depuis le plus petit rang ; l'ordre du parcours est libre : il n'y a qu'un chemin
//! dans l'arbre) ; `MD_MST` est la somme des `d_i[j]` pour `i < j`
//! dans l'ordre `(0,1), (0,2), …, (1,2), …` (à partir de `0.0`) **divisée** par `(k (k - 1) / 2) as f64`.
//! `ErreurReseau::Coupee` si deux nourritures ne sont pas reliées dans le graphe (distance infinie).
//!
//! # Les trois mesures du réseau (à nous : l'article ne les donne qu'en mots)
//!
//! Un tube est **vif** à partir d'un `seuil` : `conductivites[e] >= seuil` (comme `mesures::tubes_vifs`). [`SEUIL_VIF`]
//! est celui de `lancer` pour les réseaux. Soit le **réseau vif** : tous les nœuds du graphe et les seuls tubes vifs.
//!
//! - **TL** : la longueur totale des tubes vifs, `mesures::tubes_vifs(...).1` (la somme des `Arete::longueur` dans
//!   l'ordre des indices).
//! - **MD** ([`distance_moyenne`]) : la moyenne, sur les `k (k - 1) / 2` paires `i < j`, de la distance **dans le
//!   réseau vif** (les longueurs ; `chemins::plus_courts` depuis `i` sur le réseau vif avec les longueurs pour coûts),
//!   sommée dans l'ordre `(0,1), (0,2), …` à partir de `0.0`, divisée par `(k (k - 1) / 2) as f64`. **`+inf` si une
//!   paire n'est pas reliée** par les tubes vifs (une nourriture coupée).
//! - **FT** ([`tenue`]) : notre lecture de « la probabilité de déconnecter une partie du réseau lors de la défaillance
//!   d'un seul lien », en **1 moins** cette probabilité (Tero 2010 : la note de lecture dit que c'est une déduction).
//!   Pour chaque tube vif `e`, on le retire seul du réseau vif ; **la panne coupe** si deux nourritures qui étaient
//!   reliées par le réseau vif ne le sont plus. `FT` est la part des tubes vifs dont la panne **ne coupe pas** :
//!   `(nombre de tubes sûrs) as f64 / (nombre de tubes vifs) as f64`, et **`0.0` sans aucun tube vif**. Un arbre dont
//!   chaque tube est sur un chemin entre deux nourritures a `FT = 0.0` ; un anneau, `FT = 1.0` ; un tube vif qui n'est
//!   sur aucun chemin entre deux nourritures (une branche morte) ne coupe rien et compte comme sûr.
//! - Les **rapports** (`Mesures::tl_sur_mst`, `Mesures::md_sur_mst`) : `TL / TL_MST` et `MD / MD_MST` (une division de
//!   flottants ; `+inf` si `MD` l'est).
//!
//! # Les cas dégénérés
//!
//! - Deux nourritures (`k = 2`) : le minimum ; le tirage fait quand même deux tirages (`borne(2)` puis `borne(1)`, qui
//!   consomme un tirage et rend 0) ; l'arbre n'a qu'une paire, `TL_MST` et `MD_MST` valent la distance des deux.
//! - Un graphe d'un seul nœud ne peut pas avoir deux nourritures : `Nourriture`. Un graphe sans arête n'a aucune
//!   conductivité : `Reseau::new(graphe, vec![], …)` réussit s'il a deux nœuds ou plus (mais le pas rend alors
//!   `Solveur(Isole)`).
//! - Aucun tube vif : `TL = 0.0`, `MD = +inf`, `FT = 0.0`. Zéro pas dans `lancer` : l'état de départ, sans aucune ligne
//!   de mesures.
//!
//! # Le seuil des tubes vifs
//!
//! [`SEUIL_VIF`] vaut `0.001`, **absolu** (il ne dépend pas de `I0`) : un tube qui ne porte plus de flux s'amincit
//! jusqu'à `PLANCHER = 1e-6` et y reste ; un tube qui en porte une part reste bien au-dessus. **Mesuré** sur les
//! réseaux de `tests/etape_1r_esprit.rs` (grille 10 × 10, 800 pas à `h = 0,1`, sigmoïde, `I0` de 1 à 8, 40 graines
//! autres que celles des tests, 68 400 conductivités) : 35 seulement (0,05 %) sont entre `1e-6` (exclu) et `1e-3`, des
//! tubes qui meurent encore ; toutes les autres sont exactement au plancher, ou à `0,0125` ou plus. Tout seuil de
//! `1e-3` à `1e-2` donne donc presque les mêmes tubes vifs. Sous la loi linéaire, au contraire, les conductivités
//! s'étalent sans trou (le choix du seuil y est une convention).
//!
//! # L'aperçu
//!
//! [`apercu`] est `tubes::apercu` (même treillis, mêmes couleurs), puis chaque nœud de nourriture est peint : le carré
//! de `echelle x echelle` pixels de son point du treillis (colonne `etage * (tl + ECART) + 2 * (x - xmin)`, ligne `2 *
//! (y - ymin)`, en points du treillis, donc les pixels `echelle * colonne ..` et `echelle * ligne ..` de l'image ;
//! `xmin` et `ymin` sont les plus petits `x` et `y` de **tous** les nœuds du graphe, `tl` la largeur d'un panneau,
//! `2 * (xmax - xmin) + 1`, comme dans `crate::rendu`, qui ne les expose pas : on les recalcule) prend la couleur
//! [`NOURRITURE`].

use crate::alea::Alea;
use crate::chemins::plus_courts;
use crate::graphe::{Arete, Graphe};
use crate::mesures::tubes_vifs;
use crate::rendu::{ECART, Image};
use crate::solveur::{ErreurSolveur, flux, pressions};
use crate::tubes::{Etat, PLANCHER, Tubes};

/// Le seuil d'un tube vif pour les réseaux dans `lancer` : `0.001`.
pub const SEUIL_VIF: f64 = 0.001;

/// La couleur d'un nœud de nourriture dans [`apercu`].
pub const NOURRITURE: [u8; 3] = [235, 60, 110];

/// La loi de renforcement `f` (voir « La loi » dans la documentation du module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loi {
    /// `f(q) = |q|`.
    Lineaire,
    /// `f(q) = q * q / (1 + q * q)` (Tero 2010 avec `γ = 2`).
    Sigmoide,
}

impl Loi {
    /// `f(q)` à la lettre (voir « La loi » dans la documentation du module).
    pub fn valeur(self, q: f64) -> f64 {
        match self {
            Loi::Lineaire => q.abs(),
            Loi::Sigmoide => {
                let q2 = q * q;
                if q2 == f64::INFINITY { 1.0 } else { q2 / (1.0 + q2) }
            }
        }
    }
}

/// Pourquoi un réseau n'a pas pu être construit, avancé ou mesuré (voir les vérifications dans la documentation du
/// module).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurReseau {
    /// Le nombre de conductivités n'est pas le nombre d'arêtes du graphe.
    Taille,
    /// La conductivité de cette arête (le plus petit indice fautif) n'est pas un nombre fini `>= PLANCHER`.
    Conductivite(usize),
    /// Les nœuds de nourriture sont moins de deux, ou l'un n'est pas un nœud du graphe, ou deux sont le même nœud.
    Nourriture,
    /// Le débit n'est pas un nombre fini `> 0.0`.
    Debit,
    /// Le pas `h` n'est pas un nombre fini avec `0.0 < h < 1.0`.
    Pas,
    /// `source` ou `puits` n'est pas un nœud du graphe, ou les deux sont le même.
    Noeud,
    /// Deux nourritures ne sont pas reliées dans le graphe : pas d'arbre couvrant.
    Coupee,
    /// Le solveur de l'étape 0 a refusé ou échoué.
    Solveur(ErreurSolveur),
}

/// Ce qu'un pas a fait : la source et le puits (des indices de nœud) et ce que le calcul a rendu.
#[derive(Clone, Debug, PartialEq)]
pub struct PasReseau {
    /// Le nœud source du pas.
    pub source: usize,
    /// Le nœud puits du pas.
    pub puits: usize,
    /// Les pressions et les flux calculés avec les conductivités d'AVANT la mise à jour (comme `Tubes::avancer`).
    pub etat: Etat,
}

/// Tire la paire `(rang de la source, rang du puits)` parmi `k` nourritures : deux tirages (voir « Le pas à plusieurs
/// sources de nourriture »). Les deux rangs sont dans `0..k` et différents.
///
/// # Panique
///
/// Si `k < 2` (ou si `k` ne tient pas dans un `u32`).
pub fn tirer_paire(alea: &mut Alea, k: usize) -> (usize, usize) {
    assert!(k >= 2, "il faut au moins deux nourritures");
    let borne = u32::try_from(k).expect("k doit tenir dans un u32");
    let i = alea.borne(borne) as usize;
    let mut j = alea.borne(borne - 1) as usize;
    if j >= i {
        j += 1;
    }
    (i, j)
}

/// Vrai si `nourritures` a au moins deux nœuds, tous dans le graphe et tous différents.
fn nourritures_valides(graphe: &Graphe, nourritures: &[usize]) -> bool {
    if nourritures.len() < 2 {
        return false;
    }
    for (r, &n) in nourritures.iter().enumerate() {
        if n >= graphe.nb_noeuds() || nourritures[..r].contains(&n) {
            return false;
        }
    }
    true
}

/// Vrai si `debit` est un nombre fini `> 0.0`.
fn debit_valide(debit: f64) -> bool {
    debit.is_finite() && debit > 0.0
}

/// Les tubes d'un réseau entre des sources de nourriture (voir la documentation du module).
#[derive(Clone, Debug, PartialEq)]
pub struct Reseau {
    conductivites: Vec<f64>,
    nourritures: Vec<usize>,
    debit: f64,
    loi: Loi,
}

impl Reseau {
    /// Un réseau de conductivités données (une par arête, par indice), de nourritures (par rang), de débit et de loi.
    ///
    /// # Erreurs
    ///
    /// Celles de « Les vérifications » : `Taille`, `Conductivite(e)`, `Nourriture`, `Debit`, dans cet ordre.
    pub fn new(
        graphe: &Graphe,
        conductivites: Vec<f64>,
        nourritures: Vec<usize>,
        debit: f64,
        loi: Loi,
    ) -> Result<Reseau, ErreurReseau> {
        if conductivites.len() != graphe.nb_aretes() {
            return Err(ErreurReseau::Taille);
        }
        for (e, &d) in conductivites.iter().enumerate() {
            if !(d.is_finite() && d >= PLANCHER) {
                return Err(ErreurReseau::Conductivite(e));
            }
        }
        if !nourritures_valides(graphe, &nourritures) {
            return Err(ErreurReseau::Nourriture);
        }
        if !debit_valide(debit) {
            return Err(ErreurReseau::Debit);
        }
        Ok(Reseau { conductivites, nourritures, debit, loi })
    }

    /// Un réseau aux conductivités de départ de `Tubes::au_hasard(graphe, alea)` (voir « Les conductivités de départ
    /// »).
    ///
    /// # Erreurs
    ///
    /// `Nourriture`, puis `Debit`, avant tout tirage : sur une erreur, `alea` n'a pas bougé.
    pub fn au_hasard(
        graphe: &Graphe,
        alea: &mut Alea,
        nourritures: Vec<usize>,
        debit: f64,
        loi: Loi,
    ) -> Result<Reseau, ErreurReseau> {
        if !nourritures_valides(graphe, &nourritures) {
            return Err(ErreurReseau::Nourriture);
        }
        if !debit_valide(debit) {
            return Err(ErreurReseau::Debit);
        }
        let tubes = Tubes::au_hasard(graphe, alea);
        Ok(Reseau { conductivites: tubes.conductivites().to_vec(), nourritures, debit, loi })
    }

    /// Les conductivités, dans l'ordre des indices d'arête.
    pub fn conductivites(&self) -> &[f64] {
        &self.conductivites
    }

    /// Les nœuds de nourriture, par rang (dans l'ordre donné à la construction).
    pub fn nourritures(&self) -> &[usize] {
        &self.nourritures
    }

    /// Le débit `I0`.
    pub fn debit(&self) -> f64 {
        self.debit
    }

    /// La loi.
    pub fn loi(&self) -> Loi {
        self.loi
    }

    /// Un pas pour la paire donnée (voir « Le pas à plusieurs sources de nourriture », points 2 et 3) : la source
    /// injecte `I0`, le puits est la masse ; les conductivités sont mises à jour avec la loi du réseau.
    ///
    /// # Erreurs
    ///
    /// `Taille`, `Nourriture`, `Noeud`, `Pas`, `Solveur(erreur)`, dans cet ordre (voir « Les vérifications ») ; rien
    /// n'a changé sur une erreur.
    pub fn avancer_paire(
        &mut self,
        graphe: &Graphe,
        source: usize,
        puits: usize,
        h: f64,
    ) -> Result<PasReseau, ErreurReseau> {
        if graphe.nb_aretes() != self.conductivites.len() {
            return Err(ErreurReseau::Taille);
        }
        if self.nourritures.iter().any(|&n| n >= graphe.nb_noeuds()) {
            return Err(ErreurReseau::Nourriture);
        }
        if source >= graphe.nb_noeuds() || puits >= graphe.nb_noeuds() || source == puits {
            return Err(ErreurReseau::Noeud);
        }
        if !(h.is_finite() && h > 0.0 && h < 1.0) {
            return Err(ErreurReseau::Pas);
        }
        self.calculer_et_mettre_a_jour(graphe, source, puits, h)
    }

    /// Le calcul et la mise à jour d'un pas, une fois les vérifications faites (sauf celles du solveur).
    fn calculer_et_mettre_a_jour(
        &mut self,
        graphe: &Graphe,
        source: usize,
        puits: usize,
        h: f64,
    ) -> Result<PasReseau, ErreurReseau> {
        let mut conductances = Vec::with_capacity(graphe.nb_aretes());
        for e in 0..graphe.nb_aretes() {
            conductances.push(self.conductivites[e] / graphe.aretes()[e].cout);
        }
        let mut injections = vec![0.0; graphe.nb_noeuds()];
        injections[source] = self.debit;
        let p = pressions(graphe, &conductances, &injections, puits).map_err(ErreurReseau::Solveur)?;
        let q = flux(graphe, &conductances, &p);
        let mut mises_a_jour = Vec::with_capacity(self.conductivites.len());
        for (e, &q_e) in q.iter().enumerate() {
            let d = self.conductivites[e] + h * (self.loi.valeur(q_e) - self.conductivites[e]);
            mises_a_jour.push(if d >= PLANCHER { d } else { PLANCHER });
        }
        self.conductivites = mises_a_jour;
        Ok(PasReseau { source, puits, etat: Etat { pressions: p, flux: q } })
    }

    /// Un pas complet : tire la paire avec `alea` (`tirer_paire`), puis la fait avancer (voir « Le pas à plusieurs
    /// sources de nourriture »). Rend la source et le puits tirés (des indices de nœud).
    ///
    /// # Erreurs
    ///
    /// `Taille`, `Nourriture`, `Pas`, `Solveur(erreur)`, dans cet ordre ; sur une erreur, ni les conductivités ni
    /// `alea` n'ont changé (les trois premières se vérifient avant le tirage ; pour `Solveur`, le tirage se fait sur
    /// une copie de `alea`, recopiée dans `alea` seulement si le pas réussit).
    pub fn avancer(&mut self, graphe: &Graphe, alea: &mut Alea, h: f64) -> Result<PasReseau, ErreurReseau> {
        if graphe.nb_aretes() != self.conductivites.len() {
            return Err(ErreurReseau::Taille);
        }
        if self.nourritures.iter().any(|&n| n >= graphe.nb_noeuds()) {
            return Err(ErreurReseau::Nourriture);
        }
        if !(h.is_finite() && h > 0.0 && h < 1.0) {
            return Err(ErreurReseau::Pas);
        }
        let mut copie = alea.clone();
        let (i, j) = tirer_paire(&mut copie, self.nourritures.len());
        let (source, puits) = (self.nourritures[i], self.nourritures[j]);
        let pas = self.calculer_et_mettre_a_jour(graphe, source, puits, h)?;
        *alea = copie;
        Ok(pas)
    }
}

/// L'arbre couvrant minimal des nourritures, dans la clôture métrique du graphe (voir « L'arbre couvrant minimal »).
#[derive(Clone, Debug, PartialEq)]
pub struct Arbre {
    /// Les paires de rangs `(i, j)`, `i < j`, dans l'ordre où Kruskal les a gardées (`k - 1` paires).
    pub paires: Vec<(usize, usize)>,
    /// `TL_MST` : la somme de leurs distances, dans l'ordre de `paires`, à partir de `0.0`.
    pub longueur: f64,
    /// `MD_MST` : la distance moyenne dans l'arbre, sur les paires de nourritures.
    pub distance_moyenne: f64,
}

/// L'arbre couvrant minimal des nourritures (voir « L'arbre couvrant minimal »).
///
/// # Erreurs
///
/// `ErreurReseau::Nourriture` (moins de deux nœuds, un nœud hors du graphe, deux fois le même), puis
/// `ErreurReseau::Coupee` (deux nourritures non reliées dans le graphe).
pub fn arbre_couvrant(graphe: &Graphe, nourritures: &[usize]) -> Result<Arbre, ErreurReseau> {
    if !nourritures_valides(graphe, nourritures) {
        return Err(ErreurReseau::Nourriture);
    }
    let k = nourritures.len();
    let par_longueur = graphe_des_longueurs(graphe, |_| true);
    // distance[i][j] pour i < j, lue depuis i.
    let mut distance = vec![vec![0.0_f64; k]; k];
    for i in 0..k {
        let chemins = plus_courts(&par_longueur, nourritures[i]);
        for j in (i + 1)..k {
            let d = chemins.distances[nourritures[j]];
            if d == f64::INFINITY {
                return Err(ErreurReseau::Coupee);
            }
            distance[i][j] = d;
        }
    }
    // Kruskal sur les paires triées par (distance, i, j).
    let mut candidates = Vec::with_capacity(k * (k - 1) / 2);
    for (i, ligne) in distance.iter().enumerate() {
        for (j, &d) in ligne.iter().enumerate().skip(i + 1) {
            candidates.push((d, i, j));
        }
    }
    candidates.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("distances finies").then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let mut composante: Vec<usize> = (0..k).collect();
    let mut paires = Vec::with_capacity(k - 1);
    let mut longueur = 0.0;
    for &(d, i, j) in &candidates {
        let (ci, cj) = (composante[i], composante[j]);
        if ci != cj {
            for c in composante.iter_mut() {
                if *c == cj {
                    *c = ci;
                }
            }
            paires.push((i, j));
            longueur += d;
        }
    }
    // La distance dans l'arbre : pour chaque i, un parcours depuis i.
    let mut voisins: Vec<Vec<usize>> = vec![Vec::new(); k];
    for &(i, j) in &paires {
        voisins[i].push(j);
        voisins[j].push(i);
    }
    let mut somme = 0.0;
    for i in 0..k {
        let mut d = vec![f64::NAN; k];
        d[i] = 0.0;
        let mut pile = vec![i];
        while let Some(u) = pile.pop() {
            for &v in &voisins[u] {
                if d[v].is_nan() {
                    d[v] = d[u] + distance[u.min(v)][u.max(v)];
                    pile.push(v);
                }
            }
        }
        for &dj in &d[(i + 1)..k] {
            somme += dj;
        }
    }
    let distance_moyenne = somme / ((k * (k - 1) / 2) as f64);
    Ok(Arbre { paires, longueur, distance_moyenne })
}

/// Le graphe où le coût de chaque arête gardée par `garder` (selon son indice) est sa longueur : mêmes nœuds, mêmes
/// arêtes dans le même ordre.
fn graphe_des_longueurs(graphe: &Graphe, garder: impl Fn(usize) -> bool) -> Graphe {
    let aretes: Vec<Arete> = graphe
        .aretes()
        .iter()
        .enumerate()
        .filter(|&(e, _)| garder(e))
        .map(|(_, a)| Arete { a: a.a, b: a.b, longueur: a.longueur, cout: a.longueur })
        .collect();
    Graphe::new(graphe.noeuds().to_vec(), aretes).expect("les arêtes d'un graphe valide restent valides")
}

/// Les vérifications communes de `distance_moyenne` et `tenue`.
fn verifier_mesure(graphe: &Graphe, conductivites: &[f64], nourritures: &[usize]) {
    assert_eq!(conductivites.len(), graphe.nb_aretes(), "il faut une conductivité par arête");
    assert!(nourritures_valides(graphe, nourritures), "il faut au moins deux nourritures distinctes du graphe");
}

/// `MD` : la distance moyenne entre les nourritures dans le réseau vif (voir « Les trois mesures du réseau »), `+inf`
/// si une paire n'est pas reliée.
///
/// # Panique
///
/// Si `conductivites.len()` n'est pas `graphe.nb_aretes()`, ou si `nourritures` a moins de deux nœuds, un nœud hors du
/// graphe ou deux fois le même.
pub fn distance_moyenne(graphe: &Graphe, conductivites: &[f64], nourritures: &[usize], seuil: f64) -> f64 {
    verifier_mesure(graphe, conductivites, nourritures);
    let k = nourritures.len();
    let vif = graphe_des_longueurs(graphe, |e| conductivites[e] >= seuil);
    let mut somme = 0.0;
    for (i, &depart) in nourritures.iter().enumerate() {
        let chemins = plus_courts(&vif, depart);
        for &arrivee in &nourritures[(i + 1)..] {
            let d = chemins.distances[arrivee];
            if d == f64::INFINITY {
                return f64::INFINITY;
            }
            somme += d;
        }
    }
    somme / ((k * (k - 1) / 2) as f64)
}

/// Les composantes (par un représentant) du réseau vif privé de l'arête `sauf` (s'il y en a une).
fn composantes(graphe: &Graphe, conductivites: &[f64], seuil: f64, sauf: Option<usize>) -> Vec<usize> {
    fn racine(rep: &mut [usize], mut x: usize) -> usize {
        while rep[x] != x {
            rep[x] = rep[rep[x]];
            x = rep[x];
        }
        x
    }
    let mut rep: Vec<usize> = (0..graphe.nb_noeuds()).collect();
    for (e, arete) in graphe.aretes().iter().enumerate() {
        if conductivites[e] >= seuil && Some(e) != sauf {
            let (ra, rb) = (racine(&mut rep, arete.a), racine(&mut rep, arete.b));
            if ra != rb {
                rep[ra.max(rb)] = ra.min(rb);
            }
        }
    }
    (0..graphe.nb_noeuds()).map(|v| racine(&mut rep, v)).collect()
}

/// `FT` : la part des tubes vifs dont la panne seule ne coupe aucune nourriture d'une autre (voir « Les trois mesures
/// du réseau »), `0.0` sans tube vif.
///
/// # Panique
///
/// Comme [`distance_moyenne`].
pub fn tenue(graphe: &Graphe, conductivites: &[f64], nourritures: &[usize], seuil: f64) -> f64 {
    verifier_mesure(graphe, conductivites, nourritures);
    let k = nourritures.len();
    let entier = composantes(graphe, conductivites, seuil, None);
    let mut vifs = 0_usize;
    let mut surs = 0_usize;
    for e in 0..graphe.nb_aretes() {
        let vif = conductivites[e] >= seuil;
        if !vif {
            continue;
        }
        vifs += 1;
        let sans = composantes(graphe, conductivites, seuil, Some(e));
        let mut coupe = false;
        for i in 0..k {
            for j in (i + 1)..k {
                let (a, b) = (nourritures[i], nourritures[j]);
                if entier[a] == entier[b] && sans[a] != sans[b] {
                    coupe = true;
                }
            }
        }
        if !coupe {
            surs += 1;
        }
    }
    if vifs == 0 { 0.0 } else { surs as f64 / vifs as f64 }
}

/// Les trois mesures d'un réseau et leurs rapports à l'arbre couvrant minimal.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesures {
    /// `TL` : la longueur totale des tubes vifs.
    pub tl: f64,
    /// `MD` : la distance moyenne dans le réseau vif (`+inf` si une nourriture est coupée).
    pub md: f64,
    /// `FT` : la tenue à une panne simple.
    pub ft: f64,
    /// `TL / TL_MST`.
    pub tl_sur_mst: f64,
    /// `MD / MD_MST`.
    pub md_sur_mst: f64,
}

/// Les mesures d'un réseau au `seuil` (voir « Les trois mesures du réseau »).
///
/// # Erreurs
///
/// `ErreurReseau::Taille` (`conductivites.len()` n'est pas `graphe.nb_aretes()`), puis celles d'`arbre_couvrant`
/// (`Nourriture`, `Coupee`).
pub fn mesurer(
    graphe: &Graphe,
    conductivites: &[f64],
    nourritures: &[usize],
    seuil: f64,
) -> Result<Mesures, ErreurReseau> {
    if conductivites.len() != graphe.nb_aretes() {
        return Err(ErreurReseau::Taille);
    }
    let arbre = arbre_couvrant(graphe, nourritures)?;
    let tl = tubes_vifs(graphe, conductivites, seuil).1;
    let md = distance_moyenne(graphe, conductivites, nourritures, seuil);
    let ft = tenue(graphe, conductivites, nourritures, seuil);
    Ok(Mesures { tl, md, ft, tl_sur_mst: tl / arbre.longueur, md_sur_mst: md / arbre.distance_moyenne })
}

/// L'aperçu d'un réseau : `tubes::apercu`, puis les nœuds de nourriture en [`NOURRITURE`] (voir « L'aperçu »).
///
/// # Panique
///
/// Dans les cas de `tubes::apercu` (`echelle` nulle, `reference` qui n'est pas un nombre fini `> 0.0`, nombre de
/// conductivités qui n'est pas celui des arêtes), ou si un nœud de `nourritures` n'est pas un nœud du graphe.
pub fn apercu(graphe: &Graphe, conductivites: &[f64], nourritures: &[usize], reference: f64, echelle: u32) -> Image {
    let mut image = crate::tubes::apercu(graphe, conductivites, reference, echelle);
    let noeuds = graphe.noeuds();
    let xmin = noeuds.iter().map(|n| i64::from(n.x)).min().expect("un graphe a un nœud");
    let ymin = noeuds.iter().map(|n| i64::from(n.y)).min().expect("un graphe a un nœud");
    let xmax = noeuds.iter().map(|n| i64::from(n.x)).max().expect("un graphe a un nœud");
    let tl = 2 * (xmax - xmin) + 1;
    let echelle = echelle as usize;
    let largeur = image.largeur as usize;
    for &n in nourritures {
        assert!(n < graphe.nb_noeuds(), "le nœud {n} n'est pas dans le graphe");
        let noeud = noeuds[n];
        let colonne = (i64::from(noeud.etage) * (tl + i64::from(ECART)) + 2 * (i64::from(noeud.x) - xmin)) as usize;
        let ligne = (2 * (i64::from(noeud.y) - ymin)) as usize;
        for py in (echelle * ligne)..(echelle * (ligne + 1)) {
            for px in (echelle * colonne)..(echelle * (colonne + 1)) {
                let debut = 3 * (py * largeur + px);
                image.rvb[debut..debut + 3].copy_from_slice(&NOURRITURE);
            }
        }
    }
    image
}
