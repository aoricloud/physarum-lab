//! L'agent porté par le flux : une concentration par nœud, advectée par les flux des tubes (étape 2, partie 2T).
//!
//! Un stimulus (de la nourriture, du sel) libère dans le blob un agent soluble ; le liquide qui circule dans les tubes
//! le porte. Ce module pose **seulement ce transport** : où va l'agent, pas ce qu'il fait aux tubes. L'effet de l'agent
//! sur les tubes (la mémoire) est la partie 2M, plus tard ; elle lira les concentrations à chaque pas
//! ([`Agent::concentrations`]) et les fera agir sur la mise à jour des tubes. Ce que les tests d'acceptation prouvent
//! est dans `references/etape-2t-valeurs.md` (identifiants `V-AGT-xx`, `V-REC-xx`, `V-MAS-xx`, `V-STA-xx`, `V-BIT-xx`,
//! `V-TAC-xx`, `V-ERR-xx`, `V-APE-xx`).
//!
//! # D'où cela vient, et ce qui est à nous
//!
//! - **Ce qui vient de Kramar et Alim** (PNAS 118(10), e2007815118, 2021) : le mécanisme (un agent chimique soluble,
//!   libéré au stimulus, **advecté par les flux cytoplasmiques**), la décroissance lente de l'agent (`k_deg = 0,001
//!   s⁻¹` dans leur modèle), une diffusivité moléculaire `κ`, et une libération qui n'a lieu qu'à l'endroit du
//!   stimulus.
//! - **Ce qui est à nous** : leur modèle est celui d'**un seul tube** élastique (advection, dispersion de Taylor,
//!   décroissance, libération unique). **Aucune source ne donne ce transport sur un graphe.** Tout le reste est un
//!   choix de ce dépôt : une concentration par **nœud** ; chaque nœud est une **chambre de volume 1** (la masse d'agent
//!   est la somme des concentrations ; aucune source ne fixe un volume de nœud) ; l'advection en **schéma amont**
//!   explicite ; une diffusion à **un seul coefficient** `κ` (la dispersion de Taylor, `κ + a² ū² / 48 κ` chez eux,
//!   n'est **pas** reprise : le dépôt n'a pas de rayon de tube, et l'exposant de `ū` n'a pas pu être relu sur l'image
//!   de l'article) ; les bords (la source et le puits des tubes) ; la perte linéaire `k` ; le pas borné.
//! - **Ce qui n'a pas été lu** : le supplément de l'article (figures S1 à S13) ; le pas de temps, le maillage et le
//!   schéma de leur simulation (Crank-Nicolson pondéré, valeurs non données). Les nombres de ce dépôt (`κ`, `k`, les
//!   pas) sont ceux que les tests choisissent : aucun n'est une valeur publiée.
//!
//! # L'état, et ce que « une concentration » veut dire
//!
//! [`Agent`] garde une concentration par nœud (`nb_noeuds` nombres, dans l'ordre des indices), chacune **finie et `>=
//! 0.0`** (`-0.0` est accepté à la construction, tel quel ; un pas n'en produit jamais). **Chaque nœud est une chambre
//! de volume 1** : la quantité d'agent d'un nœud est sa concentration, et la **masse** de l'agent est la somme des
//! concentrations, **de gauche à droite à partir de `0.0`** ([`Agent::masse`]).
//!
//! # Les flux
//!
//! Le transport lit les flux **d'un pas des tubes** (`Etat::flux` de `Tubes::avancer`, ou celui de `PasReseau::etat` de
//! `Reseau::avancer`), rien d'autre : `flux[e]` est le débit de liquide de l'arête `e`, **positif quand le liquide va
//! du nœud `a` vers le nœud `b`** (`a < b`), en unités de volume par unité de temps. Pour un nœud `i` et une arête `e`
//! de `i`, le **débit sortant** de `i` par `e` est `q = flux[e]` si `i == arete.a`, et `q = -flux[e]` si `i == arete.b`
//! (une négation exacte) ; `q > 0` veut dire que `i` donne du liquide à l'autre bout (`i` est le **donneur** de `e`),
//! `q < 0` qu'il en reçoit (`i` est le **receveur**).
//!
//! # Les bords : où le liquide entre et sort du réseau
//!
//! Le module ne reçoit ni la source ni le puits : il les déduit des flux. Pour chaque nœud `i`, `s_i` est la somme des
//! débits sortants `q` de **toutes** ses arêtes (`s_i = 0.0`, puis `s_i = s_i + q` pour chaque arête de
//! `graphe.incidentes(i)`, dans l'ordre). Le liquide est conservé dans un tube sans fuite : si `s_i > 0.0`, plus de
//! liquide part de `i` par ses arêtes qu'il n'en arrive : du liquide **entre** au nœud `i` (la source), et il entre
//! **sans agent**. Si `s_i < 0.0`, du liquide **quitte** le réseau en `i` (le puits) et **emporte** de l'agent : la
//! **sortie** du nœud est `sortie_i = -s_i` (sinon `sortie_i = 0.0`), un débit de liquide, et il emporte l'agent à la
//! concentration de `i`. **La même règle s'applique à tous les nœuds**, y compris ceux de l'intérieur, où `s_i` vaut
//! zéro **à l'arrondi près** : un `s_i` négatif de l'ordre de `1e-16` fait sortir un millionième de milliardième de la
//! concentration à chaque pas. Ce que cela coûte a été mesuré (voir « Ce que l'arrondi coûte » plus bas).
//!
//! # Le pas, à la lettre
//!
//! Un pas de durée `h` (tout se calcule à partir des concentrations `c` **d'avant** le pas, puis s'applique ; une
//! concentration mise à jour n'influence pas la suivante dans le même pas) prend les flux `Q`, les [`Transport`] (`κ =
//! diffusion`, `k = perte`, tous deux `>= 0`) et une liste d'apports `(nœud, débit d'agent)`. Les opérations sont
//! celles ci-dessous, **dans cet ordre, avec ces parenthèses** (`+ - * /` seulement ; chaque ligne s'évalue
//! parenthèses d'abord, puis de gauche à droite, une opération `f64` à la fois) ;
//! `L_e` est la `longueur` de l'arête `e` (jamais son coût) ; chaque somme part de `0.0` et ajoute ses termes **dans
//! l'ordre indiqué** (une somme de flottants dépend de son ordre). Le résultat est, bit pour bit, celui de cette
//! recette.
//!
//! ```text
//! pour chaque nœud i, par indice croissant :                          (« le taux de départ » r_i, voir plus bas)
//!     r = 0.0 ; s = 0.0
//!     pour chaque arête e de graphe.incidentes(i), dans l'ordre :      (j l'autre bout, q le débit sortant de i par e)
//!         s = s + q
//!         si q > 0.0 :  r = r + q                                      (advection : i est le donneur)
//!         r = r + kappa / L_e                                          (diffusion, toujours, même si kappa vaut 0.0)
//!     sortie_i = -s  si s < 0.0, sinon 0.0
//!     r = r + sortie_i
//!     r = r + k
//!     r_i = r
//!
//! pour chaque nœud i, par indice croissant :
//!     g = 0.0
//!     pour chaque arête e de graphe.incidentes(i), dans l'ordre :      (j et q : comme dans le premier bloc)
//!         si q < 0.0 :  g = g + (h * (-q)) * c[j]                      (advection : i reçoit de j, le donneur)
//!         g = g + (h * (kappa / L_e)) * c[j]                           (diffusion : i reçoit de j, toujours)
//!     c_apres[i] = (1.0 - h * r_i) * c[i] + g                          (ce qui reste, plus ce qui arrive)
//!
//! sortie = 0.0 ; perte = 0.0                                           (le bilan, par nœud croissant)
//! pour chaque nœud i, par indice croissant :
//!     sortie = sortie + (h * sortie_i) * c[i]
//!     perte  = perte  + (h * k) * c[i]
//! apport = 0.0
//! pour chaque (nœud, debit) de la liste des apports, dans l'ordre de la liste :
//!     c_apres[nœud] = c_apres[nœud] + h * debit                        (après le calcul de tous les c_apres ci-dessus)
//!     apport = apport + h * debit
//! ```
//!
//! C'est, en mots : sur chaque arête de flux `Q` non nul, la quantité `h · |Q| · c_donneur` passe du donneur au
//! receveur (**advection, schéma amont**) ; sur chaque arête, `h · κ · (c_j − c_i) / L_e` passe de `j` vers `i`
//! (**diffusion**, signée ; ici écrite en deux parts positives, ce qui revient au même en réels et donne des termes `>=
//! 0` aux flottants) ; le liquide qui sort au puits emporte `h · sortie_i · c_i` (**la sortie**) ; `h · k · c_i`
//! disparaît (**la perte**, la décroissance de l'article) ; `h · debit` arrive au nœud d'un apport (**l'apport**, par
//! où 2M posera un stimulus). Le terme `(1.0 - h * r_i) * c[i]` regroupe tout ce que `i` donne : advection, diffusion,
//! sortie et perte. Les sommes par arête vont dans l'ordre de `incidentes`, c'est-à-dire des indices d'arête croissants
//! : **une boucle par arête qui ajoute chaque terme au nœud concerné, dans l'ordre des arêtes, donne les mêmes bits**
//! (les tests le font ainsi).
//!
//! Le [`Bilan`] d'un pas contient `sortie`, `perte` et `apport` tels que calculés ci-dessus, et le nombre de sous-pas
//! (`1` pour [`pas_sans_decoupage`]). En réels, `masse_apres = masse_avant - sortie - perte + apport` : c'est la
//! conservation de la masse ; aux flottants, elle tient à l'arrondi près (tolérance mesurée dans les tests), et
//! **exactement** sur des valeurs dyadiques choisies (voir « Ce que l'arrondi coûte »).
//!
//! # Le taux de départ et le pas maximal
//!
//! `r_i` est **ce qui part du nœud `i` par unité de temps, rapporté à sa concentration** : le liquide qu'il donne à ses
//! voisins, la diffusion vers chacun d'eux, la sortie et la perte. Le schéma garde les concentrations `>= 0` si, pour
//! chaque nœud, `h * r_i <= 1` : alors `1.0 - h * r_i >= 0.0` (une soustraction de flottants d'un nombre `<= 1.0` à
//! `1.0` ne donne jamais un négatif) et tout le reste du pas n'ajoute que des termes `>= 0`. [`taux_de_depart`] rend
//! les `r_i`. [`pas_maximal`] rend **le plus grand flottant `h` tel que `h * r_max <= 1.0` calculé en flottants**,
//! `r_max` étant le plus grand des `r_i` (comparaisons `r > m` en partant de `0.0`) : on prend `h = 1.0 / r_max`, on le
//! baisse d'un cran (`f64::from_bits(h.to_bits() - 1)`) tant que `h * r_max > 1.0`, puis on le monte d'un cran
//! (`h.to_bits() + 1`) tant que le suivant garde `h * r_max <= 1.0`. Si `r_max == 0.0`, le pas maximal est
//! `f64::INFINITY` (rien ne part jamais d'aucun nœud). La descente d'un cran ne sert que si `1.0 / r_max` déborde (un
//! `r_max` sous-normal, comme une perte de `1e-320`) : `h` vaut alors `f64::INFINITY`, `h * r_max` aussi, et la
//! descente rend `f64::MAX`. Hors de ce cas, sur deux millions de nombres essayés par le modèle Python,
//! `(1.0 / r) * r` n'a jamais dépassé `1.0`.
//!
//! # Le pas public et son découpage
//!
//! [`Agent::avancer`] accepte un `h` quelconque (fini, `> 0.0`) : il le **découpe** en `n` sous-pas égaux. Avec `h_max
//! = pas_maximal(...)`, `n` est **le plus petit entier `>= 1` tel que `h / (n as f64) <= h_max`**, comparé en flottants
//! ([`nombre_de_sous_pas`]) ; chaque sous-pas dure `h_sub = h / (n as f64)` (pour `n = 1`, exactement `h`) et applique
//! la recette ci-dessus avec `h_sub`, **les apports comptant `h_sub * debit` à chaque sous-pas** (donc `h * debit` en
//! tout). Les flux sont les mêmes à chaque sous-pas. Le bilan rendu est la **somme des bilans des sous-pas** (chaque
//! total part de `0.0`, ajoute dans l'ordre des sous-pas), son nombre de sous-pas est `n`. Si `n` dépasse
//! [`MAX_SOUS_PAS`] (un million), c'est l'erreur `SousPas`. [`pas_sans_decoupage`] fait **un seul** pas de la recette
//! avec le `h` donné, sans le comparer à `h_max` : au-dessus de la limite, il rend des concentrations négatives (c'est
//! ce que les tests éprouvent) ; il ne touche aucun [`Agent`] (il prend et rend des tableaux).
//!
//! # Ce que l'arrondi coûte
//!
//! Mesuré sur des grilles de `6 × 6`, `8 × 8` et `12 × 10` aux flux de vrais pas des tubes (`Tubes::avancer`, 1, 30 et
//! 300 pas), des concentrations au hasard, quatre transports `(κ, k)` dont des nuls (`mes_2t_a.rs`, `mes_2t_d.rs`).
//!
//! - **À la limite exacte** (`h = pas_maximal`) : `1.0 - h * r_i` est `>= 0.0` par construction, donc **aucune
//!   concentration n'est négative, ni `-0.0`** : mesuré sur 1 800 cas, 132 000 concentrations : **0 négatif, 0
//!   `-0.0`**. La recette « naïve » `c + (gains - pertes)` (une somme signée des échanges), essayée à côté, rend **204
//!   négatifs** minuscules (le pire `-8.9e-16`) : c'est pour cela que la recette regroupe ce qui part dans `1.0 - h *
//!   r_i`, et il n'y a ni `max(0.0)` ni seuil : le schéma est exact là où il le dit. Un cran au-dessus du pas maximal,
//!   une concentration peut devenir négative (le nœud de taux maximal sans apport : `-2.2e-16` pour une concentration
//!   `1`).
//! - **Aux nœuds intérieurs**, `s_i` est nul à l'arrondi près et la même règle s'y applique : le plus grand `|s_i|`
//!   d'un nœud autre que la source et le puits vaut `5.6e-15` pour des flux de l'ordre de `1` (`8e-14` de la somme des
//!   `|q|` du nœud) ; sur 200 pas au pas maximal, une libération de masse `1` perd par ces nœuds **au plus `9.2e-15`**
//!   de sa masse (40 graines) : l'agent qui sort ainsi est compté dans la `sortie` du bilan, qui reste juste. Aucun
//!   seuil n'est appliqué à `s_i` (un `s_i` négatif de `-2.8e-17` fait sortir de l'agent, voir les tests).
//! - **La masse** : en réels `masse_apres = masse_avant - sortie - perte + apport` ; aux flottants l'écart d'un pas
//!   vaut au plus `3.0e-14` (masses de l'ordre de `1` à `3`, soit `3e-15` en relatif) et, composé sur 100 pas,
//!   `6.4e-14` (40 graines). Sur des valeurs dyadiques choisies, tout est exact (les cas sont vérifiés en fractions).
//!
//! # Les vérifications, dans cet ordre
//!
//! Pour [`pas_sans_decoupage`] et [`Agent::avancer`] (la première qui s'applique ; **sur une erreur, rien n'a changé**)
//! :
//!
//! 1. `Taille` : `concentrations.len()` n'est pas `graphe.nb_noeuds()` ou `flux.len()` n'est pas `graphe.nb_aretes()` ;
//! 2. `Concentration(i)` : le plus petit `i` dont la concentration n'est pas un nombre fini `>= 0.0` ;
//! 3. `Flux(e)` : la plus petite arête `e` dont le flux n'est pas fini ;
//! 4. `Diffusion` : `diffusion` n'est pas un nombre fini `>= 0.0` ; sinon `Perte` : `perte` n'est pas un nombre fini
//!    `>= 0.0` (si les deux sont fautifs, c'est `Diffusion`) ;
//! 5. `Pas` : `h` n'est pas un nombre fini `> 0.0` ;
//! 6. `Apport(r)` : le plus petit rang `r` de la liste dont le nœud n'existe pas, ou dont le débit n'est pas un nombre
//!    fini `>= 0.0` (`-0.0` est accepté) ; pour un même rang, une seule erreur ;
//! 7. `Debordement` : un taux de départ n'est pas fini (avant tout calcul de concentration), ou, après le calcul, une
//!    concentration ou l'un des trois totaux du bilan ne l'est pas (dans [`Agent::avancer`] : à chaque sous-pas, sur
//!    les concentrations et les trois totaux de ce sous-pas ; les totaux sommés des sous-pas ne sont pas revérifiés) ;
//! 8. `SousPas` (seulement [`Agent::avancer`]) : `n > MAX_SOUS_PAS`, vu après le point 7 sur les taux.
//!
//! [`taux_de_depart`] et [`pas_maximal`] font les vérifications 1 (pour les flux seuls : `flux.len()` n'est pas
//! `graphe.nb_aretes()`), 3 et 4 sur leurs seules entrées, puis 7 sur les taux.
//! [`Agent::new`] fait `Taille` puis `Concentration(i)`.
//!
//! # Les cas dégénérés
//!
//! - Un graphe sans arête : les flux sont vides, tous les `r_i` valent `k` : l'agent ne fait que décroître.
//! - Des flux tous nuls et `κ = 0` : `r_i = k` : rien ne bouge que la perte. Avec `k = 0.0` aussi, `r_max = 0.0`, le
//!   pas maximal est infini et `n = 1`.
//! - Une liste d'apports vide, ou un débit `0.0` : aucun apport.
//! - Un même nœud plusieurs fois dans les apports : chaque entrée s'ajoute, dans l'ordre.
//!
//! # L'aperçu
//!
//! [`apercu`] dessine les tubes comme `tubes::apercu` (mêmes panneaux, mêmes positions, mêmes dimensions, avec
//! `reference_tubes` pour référence et l'échelle 1 pour le treillis), puis colore les **nœuds** selon la concentration
//! rapportée à `reference_agent` : c'est « la tache d'agent qui suit les tubes ». Le niveau et le mélange sont ceux de
//! `tubes::apercu`, en entiers, sauf une division de flottants :
//!
//! ```text
//! niveau(c) = 255                        si t >= 1.0, avec t = c / reference_agent
//!           = (t * 255.0).round()        si 0.0 < t < 1.0       (round : à l'entier le plus proche, les moitiés
//!                                                                vers l'extérieur ; 0.5 donne 127.5, donc 128)
//!           = 0                          sinon                  (zéro, négatif, NaN)
//! melange(a, b, n) = (a * (255 - n) + b * n + 127) / 255          par canal, en entiers non signés, division entière
//! ```
//!
//! On part du treillis de `tubes::apercu(graphe, conductivites, reference_tubes, 1)`, puis **chaque nœud `i`, par
//! indice croissant**, repeint son point (celui de `tubes::apercu`) en `melange(c, AGENT, niveau(concentrations[i]))`,
//! où `c` est la couleur que le point a **à ce moment-là** (un niveau 0 laisse le point tel qu'il est ; deux nœuds au
//! même point : le second mélange sur le résultat du premier). Chaque point devient enfin un carré de `echelle ×
//! echelle` pixels, comme dans `rendu`. [`AGENT`] est la couleur d'un nœud de niveau 255.

use crate::graphe::Graphe;
use crate::rendu::Image;

/// Le nombre de sous-pas au-delà duquel [`Agent::avancer`] refuse : `1_000_000`.
pub const MAX_SOUS_PAS: usize = 1_000_000;

/// La couleur d'un nœud dont la concentration atteint la référence dans [`apercu`] : un vert vif.
pub const AGENT: [u8; 3] = [130, 255, 80];

/// Pourquoi un pas de transport, un pas maximal ou un agent n'a pas pu être calculé (voir « Les vérifications »).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurAgent {
    /// Une longueur de tableau n'est pas la bonne (concentrations ou flux).
    Taille,
    /// La concentration de ce nœud (le plus petit indice fautif) n'est pas un nombre fini `>= 0.0`.
    Concentration(usize),
    /// Le flux de cette arête (le plus petit indice fautif) n'est pas un nombre fini.
    Flux(usize),
    /// La diffusion `κ` n'est pas un nombre fini `>= 0.0`.
    Diffusion,
    /// La perte `k` n'est pas un nombre fini `>= 0.0`.
    Perte,
    /// Le pas `h` n'est pas un nombre fini `> 0.0`.
    Pas,
    /// L'apport de ce rang dans la liste (le plus petit rang fautif) a un nœud inconnu ou un débit qui n'est pas un
    /// nombre fini `>= 0.0`.
    Apport(usize),
    /// Un nombre calculé a débordé : il n'est pas fini.
    Debordement,
    /// Le découpage demanderait plus de [`MAX_SOUS_PAS`] sous-pas.
    SousPas,
}

/// Les deux coefficients du transport (voir « Le pas, à la lettre »).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transport {
    /// La diffusion `κ`, un nombre fini `>= 0.0` : un seul coefficient pour toutes les arêtes.
    pub diffusion: f64,
    /// La perte `k`, un nombre fini `>= 0.0` : la part de l'agent qui disparaît par unité de temps.
    pub perte: f64,
}

/// Ce qu'un pas a fait de la masse : `masse_apres = masse_avant - sortie - perte + apport` (en réels).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bilan {
    /// L'agent emporté par le liquide qui quitte le réseau (les `sortie_i`).
    pub sortie: f64,
    /// L'agent perdu par la décroissance `k`.
    pub perte: f64,
    /// L'agent apporté par la liste des apports.
    pub apport: f64,
    /// Le nombre de sous-pas (`1` pour [`pas_sans_decoupage`]).
    pub sous_pas: usize,
}

/// L'agent d'un réseau : une concentration par nœud (voir « L'état »).
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub struct Agent {
    concentrations: Vec<f64>,
}

/// Les taux de départ `r_i` de chaque nœud (voir « Le pas, à la lettre » et « Le taux de départ et le pas maximal »).
///
/// # Erreurs
///
/// `Taille` (`flux.len()` n'est pas le nombre d'arêtes), `Flux(e)`, `Diffusion`, `Perte`, puis `Debordement` si un taux
/// n'est pas fini, dans cet ordre.
#[allow(unused_variables)]
pub fn taux_de_depart(graphe: &Graphe, flux: &[f64], transport: &Transport) -> Result<Vec<f64>, ErreurAgent> {
    todo!()
}

/// Le pas maximal : le plus grand `h` tel que `h * r_max <= 1.0` en flottants, `f64::INFINITY` si `r_max == 0.0` (voir
/// « Le taux de départ et le pas maximal »).
///
/// # Erreurs
///
/// Celles de [`taux_de_depart`].
#[allow(unused_variables)]
pub fn pas_maximal(graphe: &Graphe, flux: &[f64], transport: &Transport) -> Result<f64, ErreurAgent> {
    todo!()
}

/// Le nombre de sous-pas : le plus petit entier `n >= 1` tel que `h / (n as f64) <= h_max` en flottants (voir « Le pas
/// public et son découpage »). `1` si `h <= h_max` (en particulier si `h_max` est infini). Si ce `n` dépasse
/// [`MAX_SOUS_PAS`], un nombre plus grand que [`MAX_SOUS_PAS`] (sa valeur exacte n'est pas définie).
///
/// # Panique
///
/// Si `h` n'est pas un nombre fini `> 0.0`, ou si `h_max` n'est pas `> 0.0` (l'infini est permis ; NaN panique).
#[allow(unused_variables)]
pub fn nombre_de_sous_pas(h: f64, h_max: f64) -> usize {
    todo!()
}

/// **Un seul pas** de la recette de « Le pas, à la lettre », avec le `h` donné, **sans le comparer au pas maximal** :
/// au-dessus de la limite il rend des concentrations négatives. Rend les nouvelles concentrations et le bilan (avec
/// `sous_pas == 1`). Ne modifie rien.
///
/// # Erreurs
///
/// Celles de « Les vérifications », points 1 à 7, dans cet ordre.
#[allow(unused_variables)]
pub fn pas_sans_decoupage(
    graphe: &Graphe,
    flux: &[f64],
    concentrations: &[f64],
    transport: &Transport,
    apports: &[(usize, f64)],
    h: f64,
) -> Result<(Vec<f64>, Bilan), ErreurAgent> {
    todo!()
}

impl Agent {
    /// Un agent à zéro : `graphe.nb_noeuds()` concentrations égales à `+0.0`.
    #[allow(unused_variables)]
    pub fn zero(graphe: &Graphe) -> Agent {
        todo!()
    }

    /// Un agent de concentrations données (une par nœud, dans l'ordre des indices).
    ///
    /// # Erreurs
    ///
    /// Dans cet ordre : `ErreurAgent::Taille` si `concentrations.len()` n'est pas `graphe.nb_noeuds()` ; puis
    /// `ErreurAgent::Concentration(i)`, `i` étant le plus petit indice d'une concentration qui n'est pas un nombre fini
    /// `>= 0.0` (`0.0` et `-0.0` sont acceptés ; NaN, l'infini et les négatifs sont refusés).
    #[allow(unused_variables)]
    pub fn new(graphe: &Graphe, concentrations: Vec<f64>) -> Result<Agent, ErreurAgent> {
        todo!()
    }

    /// Les concentrations, dans l'ordre des indices de nœud.
    pub fn concentrations(&self) -> &[f64] {
        todo!()
    }

    /// La masse d'agent : la somme des concentrations, de gauche à droite à partir de `0.0` (`0.0` pour zéro nœud).
    pub fn masse(&self) -> f64 {
        todo!()
    }

    /// Le pas public : découpe `h` en sous-pas égaux (voir « Le pas public et son découpage »), les joue avec les flux
    /// et les apports donnés, met l'agent à jour et rend le bilan total (la somme des bilans des sous-pas ; `sous_pas`
    /// vaut `n`).
    ///
    /// # Erreurs
    ///
    /// Celles de « Les vérifications », points 1 à 8, dans cet ordre ; **sur une erreur, l'agent n'a pas changé**.
    #[allow(unused_variables)]
    pub fn avancer(
        &mut self,
        graphe: &Graphe,
        flux: &[f64],
        transport: &Transport,
        apports: &[(usize, f64)],
        h: f64,
    ) -> Result<Bilan, ErreurAgent> {
        todo!()
    }
}

/// L'aperçu de l'agent : les tubes de `tubes::apercu`, avec les nœuds colorés selon leur concentration (voir « L'aperçu
/// »).
///
/// # Panique
///
/// Si `echelle` vaut 0, si `reference_tubes` ou `reference_agent` n'est pas un nombre fini `> 0.0`, si
/// `conductivites.len()` n'est pas le nombre d'arêtes ou si `concentrations.len()` n'est pas le nombre de nœuds.
#[allow(unused_variables)]
pub fn apercu(
    graphe: &Graphe,
    conductivites: &[f64],
    concentrations: &[f64],
    reference_tubes: f64,
    reference_agent: f64,
    echelle: u32,
) -> Image {
    todo!()
}
