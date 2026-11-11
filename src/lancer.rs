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
//! **Les murs et les étages (étape 1, partie 1L).** Deux directives de plus, facultatives, en autant d'exemplaires
//! qu'on veut pour la première et au plus un par étage pour la seconde (elles ne servent qu'avec les trois directives
//! des tubes ; voir `crate::labyrinthe`) :
//!
//! ```text
//! mur <x0> <y0> <x1> <y1> <etage>     un rectangle de cellules murées, bornes comprises
//! cout_etage <etage> <facteur>        le coût des arêtes de cet étage est multiplié par <facteur>
//! ```
//!
//! Tous les arguments de `mur` sont des entiers (comme ci-dessus) avec `x0 <= x1 < largeur`, `y0 <= y1 < hauteur` et
//! `etage < etages` ; deux murs qui se recouvrent sont permis (l'union des cellules). Pour `cout_etage`, `etage <
//! etages` et `<facteur>` est un flottant décimal (`Directive::flottant`) dans `[0,1 ; 10]`, **bornes comprises**
//! (`0.1` et `10` passent, `0.09` et `10.01` non) ; un étage n'a qu'un `cout_etage`. `lancer` construit le graphe avec
//! `labyrinthe::construire(grille, murs, couts, source, puits)`, les murs et les coûts **dans l'ordre du fichier** : la
//! simulation, l'empreinte, les mesures et l'aperçu portent sur le **graphe du plan** (les cellules murées ou séparées
//! du puits n'y sont plus). Dans l'écriture canonique, la source et le puits sont leurs indices **dans ce graphe** ;
//! sans mur, ce sont ceux de la grille. Un scénario sans `mur` ni `cout_etage` est simulé exactement comme avant.
//!
//! **Les erreurs.** Celles des directives sont `ErreurLancer::DirectiveInconnue(ligne)`, rendue avant tout calcul
//! (l'autre erreur, `Calcul`, est dite plus bas : « Un calcul qui s'arrête »). Une directive est
//! **fautive** si son nom n'est ni `source`, ni `puits`, ni `pas`, ni `mur`, ni `cout_etage` ; ou si elle n'a pas le
//! bon nombre d'arguments ou un argument mal écrit ou hors bornes (y compris un `mur` ou un `cout_etage` hors de la
//! grille, un `mur` dont `x0 > x1` ou `y0 > y1`, un facteur hors de `[0,1 ; 10]`) ; ou si c'est la **seconde**
//! `source`, `puits` ou `pas` du fichier, ou le **second** `cout_etage` d'un même étage (même si ses arguments sont
//! bons). La ligne rendue est, dans cet ordre de priorité :
//! 1. la ligne de la **première directive fautive** dans l'ordre du fichier ;
//! 2. sinon, si `source` et `puits` désignent le même nœud, la ligne de celle des deux qui vient **en second** dans le
//!    fichier ;
//! 3. sinon, s'il manque l'une des trois directives des tubes, la ligne de la **première directive restante** du
//!    fichier (celle qui ouvre la liste de `scenario.directives`) : un `mur` ou un `cout_etage` sans les trois
//!    directives des tubes n'est pas ignoré, c'est cette erreur ;
//! 4. sinon, les erreurs de `labyrinthe::construire` : le puits sous un mur (`PuitsMure`) rend la ligne de `puits` ; la
//!    source sous un mur (`SourceMuree`) ou séparée du puits par les murs (`SourceCoupee`) rend la ligne de `source`.
//!    Ses autres erreurs (`Grille`, `Mur`, `Facteur`, `Noeud`) ne peuvent pas venir d'un scénario lu : les points 1 et
//!    2 les ont déjà rendues ; si l'une venait quand même, `lancer` rendrait la ligne de `source`.
//!
//! Sans aucune directive restante, c'est l'étape 0 (rien à simuler).
//!
//! **La simulation.** Avec `alea = Alea::new(graine)`, les tubes de départ sont `Tubes::au_hasard(&graphe, &mut alea)`
//! (`graphe` est le graphe du plan ; aucun autre tirage n'est fait ensuite). Puis `nombre` fois `tubes.avancer(&graphe,
//! source, puits, 1.0, h)` : le débit vaut toujours 1. Le résultat de chaque pas est celui de la doc de `crate::tubes`,
//! bit pour bit.
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
//! ...               graphe.octets()  (le graphe du plan : murs et coûts d'étage compris)
//! 4 octets          l'indice du nœud source (dans le graphe du plan), en u32
//! 4 octets          l'indice du nœud puits (dans le graphe du plan), en u32
//! 4 octets          nombre, en u32
//! 8 octets          h.to_bits(), en u64
//! 8 octets × m      la conductivité finale de chaque arête dans l'ordre des indices : to_bits(), en u64
//! 16 octets         alea.octets(), l'état du générateur APRÈS les tirages de Tubes::au_hasard
//! ```
//!
//! soit `8 + L + 12 + 8 + 8 m + 16 = L + 44 + 8 m` octets, `L` étant la longueur de l'écriture du graphe et `m` le
//! nombre d'arêtes. L'étiquette reste `PHYSAR01` : la forme ne change pas, le graphe y est écrit en entier. Une graine
//! ou un `h` ou un nombre de pas ou une source différents donnent des empreintes différentes.
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
//! conductivités finales, la référence `1.0` (le débit) et la même règle d'échelle que ci-dessus. Avec des murs,
//! `graphe` est celui du plan et **le cadre du treillis se calcule sur ses nœuds** (`rendu::treillis(&graphe)`) : un
//! mur y est un trou, et si les murs retirent toute la première ou la dernière colonne, la première ou la dernière
//! ligne, ou tout un étage extrême, l'image rétrécit d'autant (le treillis de `rendu` lui-même ne change pas).
//!
//! # Le binaire `physarum`
//!
//! `physarum lancer <scénario> --graine <n> --sortie <dossier>` (voir `src/main.rs`) lit le fichier de scénario,
//! appelle [`lancer`] et écrit `empreinte.txt` (l'empreinte puis `\n`), `mesures.csv` et `apercu.png` dans le dossier.

use crate::alea::Alea;
use crate::empreinte::sha256_hex;
use crate::graphe::Grille;
use crate::image::ecrire_png;
use crate::labyrinthe::{CoutEtage, ErreurLabyrinthe, FACTEUR_MAX, FACTEUR_MIN, Mur, construire};
use crate::mesures::tubes_vifs;
use crate::rendu::{apercu_graphe, treillis};
use crate::scenario::{Directive, Scenario};
use crate::tubes::{Tubes, apercu as apercu_tubes};

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
    // Sans aucune directive restante, c'est l'étape 0 : la grille seule, rien à simuler.
    if scenario.directives.is_empty() {
        return Ok(lancer_grille_seule(scenario, graine));
    }

    // Les trois directives des tubes, toutes les trois ou aucune, vérifiées avant tout calcul.
    let demande = lire_directives(&scenario.directives, &scenario.grille)?;

    // Le graphe du plan : les murs retirés, la composante du puits gardée, les coûts d'étage appliqués.
    let plan = construire(&scenario.grille, &demande.murs, &demande.couts, demande.source, demande.puits).map_err(
        |erreur| match erreur {
            ErreurLabyrinthe::PuitsMure => ErreurLancer::DirectiveInconnue(demande.ligne_puits),
            _ => ErreurLancer::DirectiveInconnue(demande.ligne_source),
        },
    )?;
    let graphe = plan.graphe;
    let (source, puits) = (plan.source, plan.puits);

    // La simulation : le générateur tout juste amorcé, les tubes au hasard, puis `nombre` pas de longueur `h`.
    let mut alea = Alea::new(graine);
    let mut tubes = Tubes::au_hasard(&graphe, &mut alea);
    let mut mesures = String::from("temps,pression,vifs,longueur\n");
    for t in 0..demande.nombre {
        // La ligne `t` décrit l'état juste avant le pas `t` ; sa pression est celle que ce pas calcule.
        let (vifs, longueur) = tubes_vifs(&graphe, tubes.conductivites(), 0.5);
        let etat = tubes.avancer(&graphe, source, puits, 1.0, demande.h).map_err(|_| ErreurLancer::Calcul(t))?;
        let pression = etat.pressions[source];
        mesures.push_str(&format!("{t},{pression:.9},{vifs},{longueur:.6}\n"));
    }

    // L'écriture canonique de l'état avec tubes, après les `nombre` pas.
    let mut octets = Vec::new();
    octets.extend_from_slice(b"PHYSAR01");
    octets.extend_from_slice(&graphe.octets());
    octets.extend_from_slice(&(source as u32).to_le_bytes());
    octets.extend_from_slice(&(puits as u32).to_le_bytes());
    octets.extend_from_slice(&demande.nombre.to_le_bytes());
    octets.extend_from_slice(&demande.h.to_bits().to_le_bytes());
    for &conductivite in tubes.conductivites() {
        octets.extend_from_slice(&conductivite.to_bits().to_le_bytes());
    }
    octets.extend_from_slice(&alea.octets());
    let empreinte = sha256_hex(&octets);

    // L'aperçu : l'échelle `512 / TL` ramenée entre 1 et 8 (division entière), la référence 1.0 (le débit).
    let (tl, _th) = treillis(&graphe);
    let echelle = (512 / tl).clamp(1, 8);
    let image = apercu_tubes(&graphe, tubes.conductivites(), 1.0, echelle);
    let apercu = ecrire_png(image.largeur, image.hauteur, &image.rvb);

    Ok(Sortie { empreinte, mesures, apercu })
}

/// Un scénario sans aucune directive restante : l'étape 0 (la grille seule, la graine, rien à simuler).
fn lancer_grille_seule(scenario: &Scenario, graine: u64) -> Sortie {
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

    Sortie { empreinte, mesures, apercu }
}

/// Les valeurs des trois directives des tubes, une fois vérifiées.
struct TubesDemandes {
    /// L'indice de la source dans la grille, et la ligne de sa directive.
    source: usize,
    ligne_source: usize,
    /// L'indice du puits dans la grille, et la ligne de sa directive.
    puits: usize,
    ligne_puits: usize,
    nombre: u32,
    h: f64,
    /// Les murs et les coûts d'étage, dans l'ordre du fichier.
    murs: Vec<Mur>,
    couts: Vec<CoutEtage>,
}

/// Vérifie les directives des tubes et rend leurs valeurs (voir « Les erreurs » dans la documentation du module).
///
/// La ligne rendue suit cet ordre : la première directive fautive dans l'ordre du fichier ; sinon la seconde de
/// `source` et `puits` quand elles désignent le même nœud ; sinon la première directive restante quand il en manque une.
fn lire_directives(directives: &[Directive], grille: &Grille) -> Result<TubesDemandes, ErreurLancer> {
    let mut source: Option<(usize, usize)> = None; // (ligne, indice du nœud)
    let mut puits: Option<(usize, usize)> = None;
    let mut pas: Option<(u32, f64)> = None;
    let mut murs: Vec<Mur> = Vec::new();
    let mut couts: Vec<CoutEtage> = Vec::new();
    let mut fautive: Option<usize> = None;
    for directive in directives {
        let faute = match directive.nom.as_str() {
            "source" => {
                if source.is_some() {
                    Some(directive.ligne)
                } else {
                    match noeud(directive, grille) {
                        Some(indice) => {
                            source = Some((directive.ligne, indice));
                            None
                        }
                        None => Some(directive.ligne),
                    }
                }
            }
            "puits" => {
                if puits.is_some() {
                    Some(directive.ligne)
                } else {
                    match noeud(directive, grille) {
                        Some(indice) => {
                            puits = Some((directive.ligne, indice));
                            None
                        }
                        None => Some(directive.ligne),
                    }
                }
            }
            "pas" => {
                if pas.is_some() {
                    Some(directive.ligne)
                } else {
                    match nombre_et_h(directive) {
                        Some(valeur) => {
                            pas = Some(valeur);
                            None
                        }
                        None => Some(directive.ligne),
                    }
                }
            }
            "mur" => match mur(directive, grille) {
                Some(valeur) => {
                    murs.push(valeur);
                    None
                }
                None => Some(directive.ligne),
            },
            "cout_etage" => match cout_etage(directive, grille) {
                Some(valeur) if !couts.iter().any(|cout| cout.etage == valeur.etage) => {
                    couts.push(valeur);
                    None
                }
                _ => Some(directive.ligne),
            },
            _ => Some(directive.ligne),
        };
        if fautive.is_none() {
            fautive = faute;
        }
    }
    if let Some(ligne) = fautive {
        return Err(ErreurLancer::DirectiveInconnue(ligne));
    }

    // Deux nœuds différents : la ligne de celle des deux directives qui vient en second.
    if let (Some((ligne_source, indice_source)), Some((ligne_puits, indice_puits))) = (source, puits)
        && indice_source == indice_puits
    {
        return Err(ErreurLancer::DirectiveInconnue(ligne_source.max(ligne_puits)));
    }

    // Il manque une directive : la ligne de la première directive restante du fichier.
    let (Some((ligne_source, source)), Some((ligne_puits, puits)), Some((nombre, h))) = (source, puits, pas) else {
        let premiere = directives.first().expect("la liste des directives n'est pas vide");
        return Err(ErreurLancer::DirectiveInconnue(premiere.ligne));
    };

    Ok(TubesDemandes { source, ligne_source, puits, ligne_puits, nombre, h, murs, couts })
}

/// Le nœud d'une directive `source` ou `puits`, ou `None` si elle est fautive (arguments, forme ou bornes).
fn noeud(directive: &Directive, grille: &Grille) -> Option<usize> {
    if directive.args.len() != 3 {
        return None;
    }
    let (x, y, etage) = (directive.entier(0).ok()?, directive.entier(1).ok()?, directive.entier(2).ok()?);
    // Comparaison en `u64`, avant toute conversion.
    if x >= u64::from(grille.largeur) || y >= u64::from(grille.hauteur) || etage >= u64::from(grille.etages) {
        return None;
    }
    Some(grille.indice(x as u32, y as u32, etage as u32))
}

/// Le mur d'une directive `mur`, ou `None` si elle est fautive (arguments, forme ou bornes).
fn mur(directive: &Directive, grille: &Grille) -> Option<Mur> {
    if directive.args.len() != 5 {
        return None;
    }
    let x0 = directive.entier(0).ok()?;
    let y0 = directive.entier(1).ok()?;
    let x1 = directive.entier(2).ok()?;
    let y1 = directive.entier(3).ok()?;
    let etage = directive.entier(4).ok()?;
    // Comparaisons en `u64`, avant toute conversion.
    if x0 > x1
        || y0 > y1
        || x1 >= u64::from(grille.largeur)
        || y1 >= u64::from(grille.hauteur)
        || etage >= u64::from(grille.etages)
    {
        return None;
    }
    Some(Mur { x0: x0 as u32, y0: y0 as u32, x1: x1 as u32, y1: y1 as u32, etage: etage as u32 })
}

/// Le coût d'étage d'une directive `cout_etage`, ou `None` si elle est fautive (arguments, forme ou bornes).
fn cout_etage(directive: &Directive, grille: &Grille) -> Option<CoutEtage> {
    if directive.args.len() != 2 {
        return None;
    }
    let etage = directive.entier(0).ok()?;
    if etage >= u64::from(grille.etages) {
        return None;
    }
    let facteur = directive.flottant(1).ok()?;
    if !(FACTEUR_MIN..=FACTEUR_MAX).contains(&facteur) {
        return None;
    }
    Some(CoutEtage { etage: etage as u32, facteur })
}

/// Le nombre de pas et le pas de temps d'une directive `pas`, ou `None` si elle est fautive (arguments, forme ou
/// bornes).
fn nombre_et_h(directive: &Directive) -> Option<(u32, f64)> {
    if directive.args.len() != 2 {
        return None;
    }
    let nombre = directive.entier(0).ok()?;
    if nombre > 100_000 {
        return None;
    }
    let h = directive.flottant(1).ok()?;
    if h <= 0.0 || h >= 1.0 {
        return None;
    }
    Some((nombre as u32, h))
}
