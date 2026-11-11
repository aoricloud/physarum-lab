//! Lancer un scénario : ses sorties (empreinte, mesures, aperçu) (étape 0, puis chaque étape). Le binaire `physarum`
//! n'est qu'une coque autour de [`lancer`] : les tests appellent `lancer` sans lancer de processus.
//!
//! # Ce que fait l'étape 0
//!
//! La grille seule : le scénario donne une grille (`grille`, `voisinage`, `etages`) et rien d'autre. Il n'y a encore
//! rien à simuler : l'état est le graphe de la grille et la graine. Un scénario **sans aucune autre directive** est
//! lancé ainsi (voir [`crate::scenario::Scenario::directives`]) ; avec d'autres directives, voir plus bas (« Ce que
//! fait l'étape 1 »).
//!
//! Dans ce qui suit, `graphe` est `scenario.grille.graphe()` (il réussit toujours pour un scénario lu).
//!
//! # L'écriture canonique de l'état
//!
//! L'empreinte est le SHA-256 (`empreinte::sha256_hex`) de cette suite d'octets, dans cet ordre :
//!
//! ```text
//! 8 octets    l'étiquette ASCII "PHYSAR00"  (50 48 59 53 41 52 30 30)
//! ...         graphe.octets()               (voir « L'écriture canonique » de `crate::graphe`)
//! 16 octets   Alea::new(graine).octets()    (le générateur tout juste amorcé, sans aucun tirage)
//! ```
//!
//! Deux graines différentes, deux grilles différentes (largeur et hauteur échangées, un autre voisinage, un étage de
//! plus, un autre coût vertical) donnent des empreintes différentes.
//!
//! # Les mesures
//!
//! Le texte CSV `temps,noeuds,aretes,longueur\n0,<n>,<m>,<longueur>\n` : l'en-tête, puis une ligne. `<n>` et `<m>` sont
//! le nombre de nœuds et le nombre d'arêtes du graphe, en décimal sans zéro de tête ; `<longueur>` est
//! `graphe.longueur_totale()` écrit par `format!("{:.6}", …)` (six décimales, sans aucun arrondi à la main). Chaque
//! ligne, la dernière comprise, se termine par un seul `\n` (pas de `\r`).
//!
//! # L'aperçu
//!
//! Le PNG (`image::ecrire_png`) de `rendu::apercu_graphe(&graphe, echelle)`, où `echelle` vaut `512 / TL` (division
//! entière, `TL` étant la largeur du treillis, `rendu::treillis(&graphe).0`), ramené entre 1 et 8 (`clamp(1, 8)` : un
//! quotient nul devient 1) : une petite grille est agrandie huit fois, une grande ne l'est pas.
//!
//! # Ce que fait l'étape 1 : les tubes
//!
//! Un scénario qui porte les directives des tubes est **simulé**. Les trois directives viennent **toutes les trois ou
//! aucune** (chacune une fois, dans n'importe quel ordre, parmi les autres lignes) :
//!
//! ```text
//! source <x> <y> <etage>     le nœud d'où le flux entre
//! puits <x> <y> <etage>      le nœud par où il sort (la masse du solveur)
//! pas <nombre> <h>           <nombre> pas de temps, chacun de longueur <h>
//! ```
//!
//! `x`, `y`, `etage` et `nombre` sont des entiers en chiffres décimaux ASCII seulement (`Directive::entier` : ni signe
//! ni point ; les zéros de tête sont permis) ; `h` est un flottant décimal (`Directive::flottant` : ni exposant ni
//! `+`). `source` et `puits` désignent le nœud `grille.indice(x, y, etage)` : il faut `x < largeur`, `y < hauteur`,
//! `etage < etages`, et les deux nœuds doivent être **différents**. `nombre` va de 0 à 100000 (bornes comprises) et `h`
//! vérifie `0 < h < 1`.
//!
//! **Les erreurs.** Celles des directives sont `ErreurLancer::DirectiveInconnue(ligne)`, rendue avant tout calcul
//! (l'autre erreur, `Calcul`, est dite plus bas : « Un calcul qui s'arrête »). Une directive est
//! **fautive** si son nom n'est ni `source`, ni `puits`, ni `pas` ; ou si elle n'a pas le bon nombre d'arguments ou un
//! argument mal écrit ou hors bornes ; ou si c'est la **seconde** `source`, `puits` ou `pas` du fichier (même si ses
//! arguments sont bons). La ligne rendue est, dans cet ordre de priorité :
//! 1. la ligne de la **première directive fautive** dans l'ordre du fichier ;
//! 2. sinon, si `source` et `puits` désignent le même nœud, la ligne de celle des deux qui vient **en second** dans le
//!    fichier ;
//! 3. sinon, s'il manque l'une des trois directives, la ligne de la **première directive restante** du fichier (celle
//!    qui ouvre la liste de `scenario.directives`).
//!
//! Sans aucune directive restante, c'est l'étape 0 (rien à simuler).
//!
//! **La simulation.** Avec `alea = Alea::new(graine)`, les tubes de départ sont `Tubes::au_hasard(&graphe, &mut alea)`
//! (aucun autre tirage n'est fait ensuite). Puis `nombre` fois `tubes.avancer(&graphe, source, puits, 1.0, h)` : le
//! débit vaut toujours 1. Le résultat de chaque pas est celui de la doc de `crate::tubes`, bit pour bit.
//!
//! **Un calcul qui s'arrête.** Les pas sont numérotés à partir de 0 : le pas numéro `t` est celui qui donne la ligne
//! `t` des mesures. Si `avancer` rend une erreur au pas numéro `t` (le solveur s'arrête sur un pivot nul quand les
//! conductances sont trop étalées : par exemple avec un lien entre étages d'un coût de `10^16`, voir « Ce que le
//! solveur ne garantit pas » dans `crate::solveur`), `lancer` s'arrête là et rend `ErreurLancer::Calcul(t)`, sans
//! aucune sortie. `lancer` ne panique pour aucun scénario lu. Avec `pas 0 <h>`, aucun pas n'est fait : il ne peut pas
//! y avoir d'erreur de calcul.
//!
//! **L'écriture canonique de l'état avec tubes.** L'empreinte est le SHA-256 de cette suite d'octets (tous les entiers
//! en petit-boutiste), après les `nombre` pas :
//!
//! ```text
//! 8 octets          l'étiquette ASCII "PHYSAR01"  (50 48 59 53 41 52 30 31)
//! ...               graphe.octets()
//! 4 octets          l'indice du nœud source, en u32
//! 4 octets          l'indice du nœud puits, en u32
//! 4 octets          nombre, en u32
//! 8 octets          h.to_bits(), en u64
//! 8 octets × m      la conductivité finale de chaque arête dans l'ordre des indices : to_bits(), en u64
//! 16 octets         alea.octets(), l'état du générateur APRÈS les tirages de Tubes::au_hasard
//! ```
//!
//! soit `8 + L + 12 + 8 + 8 m + 16 = L + 44 + 8 m` octets, `L` étant la longueur de l'écriture du graphe et `m` le
//! nombre d'arêtes. Une graine ou un `h` ou un nombre de pas ou une source différents donnent des empreintes
//! différentes.
//!
//! **Les mesures avec tubes.** Un en-tête, puis **une ligne par pas** : `nombre` lignes, aucune si `nombre` vaut 0.
//!
//! ```text
//! temps,pression,vifs,longueur\n
//! <t>,<pression>,<vifs>,<longueur>\n          pour t = 0, 1, …, nombre - 1
//! ```
//!
//! La ligne `t` décrit l'état après `t` pas, c'est-à-dire les conductivités telles qu'elles sont **juste avant** le pas
//! numéro `t` (les pas sont numérotés à partir de 0 : voir « Un calcul qui s'arrête »). `<t>` est en décimal, sans
//! zéro de tête. `<vifs>` et `<longueur>` sont les deux nombres rendus par `mesures::tubes_vifs(&graphe,
//! conductivités, 0.5)` sur ces conductivités, donc **avant** d'appeler `avancer` (voir `crate::mesures` : un tube
//! est **vif** à partir de `0.5`, la moitié du débit, la borne comprise ; la longueur est la somme des `longueur` des
//! tubes vifs, pas de leurs coûts) ; `<vifs>` s'écrit en décimal et `<longueur>` par `format!("{:.6}", …)`.
//! `<pression>` est `etat.pressions[source]` de l'`Etat` que rend le pas numéro `t` (cet appel d'`avancer` la calcule
//! avec ces mêmes conductivités, avant de les mettre à jour), écrite par `format!("{:.9}", …)`. La dernière ligne se
//! termine par un seul `\n`. L'état après les `nombre` pas n'est pas dans le CSV (il est dans l'empreinte et dans
//! l'aperçu).
//!
//! **L'aperçu avec tubes.** Le PNG de `tubes::apercu(&graphe, tubes.conductivites(), 1.0, echelle)` avec les
//! conductivités finales, la référence `1.0` (le débit) et la même règle d'échelle que ci-dessus.
//!
//! # Le binaire `physarum`
//!
//! `physarum lancer <scénario> --graine <n> --sortie <dossier>` (voir `src/main.rs`) lit le fichier de scénario,
//! appelle [`lancer`] et écrit `empreinte.txt` (l'empreinte puis `\n`), `mesures.csv` et `apercu.png` dans le dossier.

use crate::alea::Alea;
use crate::empreinte::sha256_hex;
use crate::image::ecrire_png;
use crate::rendu::{apercu_graphe, treillis};
use crate::scenario::Scenario;

/// Ce que produit un lancer.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Sortie {
    /// L'empreinte de l'état : 64 chiffres hexadécimaux minuscules (voir la documentation du module).
    pub empreinte: String,
    /// Les mesures, en CSV (voir la documentation du module).
    pub mesures: String,
    /// L'aperçu, un fichier PNG complet (voir la documentation du module).
    pub apercu: Vec<u8>,
}

/// Pourquoi un scénario n'a pas pu être lancé.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurLancer {
    /// Le scénario contient une directive que ce lancer ne comprend pas, ou mal écrite, en double, ou dont il manque
    /// les compagnes ; le numéro de sa ligne (voir « Les erreurs » dans la documentation du module).
    DirectiveInconnue(usize),
    /// Le calcul des tubes s'est arrêté : le numéro du pas (compté à partir de 0) où `Tubes::avancer` a rendu une
    /// erreur (voir « Un calcul qui s'arrête » dans la documentation du module).
    Calcul(u32),
}

/// Lance le scénario avec cette graine (voir la documentation du module). Pour le même scénario et la même graine, le
/// résultat est toujours le même, octet pour octet, sur toutes les machines.
///
/// # Erreurs
///
/// `ErreurLancer::DirectiveInconnue(ligne)` pour une directive fautive (voir « Les erreurs » dans la documentation du
/// module) : rien d'autre n'est calculé alors. `ErreurLancer::Calcul(t)` si le pas numéro `t` des tubes échoue (voir «
/// Un calcul qui s'arrête »). Dans les deux cas, aucune sortie.
pub fn lancer(scenario: &Scenario, graine: u64) -> Result<Sortie, ErreurLancer> {
    // L'étape 0 ne comprend aucune autre directive : la première restante fait refuser le scénario, avant tout
    // calcul.
    if let Some(premiere) = scenario.directives.first() {
        return Err(ErreurLancer::DirectiveInconnue(premiere.ligne));
    }

    let graphe = scenario.grille.graphe().expect("la grille d'un scénario lu donne toujours un graphe");

    // L'écriture canonique de l'état : l'étiquette, le graphe, puis la graine tout juste amorcée.
    let mut octets = Vec::new();
    octets.extend_from_slice(b"PHYSAR00");
    octets.extend_from_slice(&graphe.octets());
    octets.extend_from_slice(&Alea::new(graine).octets());
    let empreinte = sha256_hex(&octets);

    // Les mesures : l'en-tête, puis une ligne, la longueur en six décimales.
    let mesures = format!(
        "temps,noeuds,aretes,longueur\n0,{},{},{:.6}\n",
        graphe.nb_noeuds(),
        graphe.nb_aretes(),
        graphe.longueur_totale()
    );

    // L'aperçu : l'échelle `512 / TL` ramenée entre 1 et 8 (division entière).
    let (tl, _th) = treillis(&graphe);
    let echelle = (512 / tl).clamp(1, 8);
    let image = apercu_graphe(&graphe, echelle);
    let apercu = ecrire_png(image.largeur, image.hauteur, &image.rvb);

    Ok(Sortie { empreinte, mesures, apercu })
}
