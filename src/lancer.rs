//! Lancer un scénario : ses sorties (empreinte, mesures, aperçu) (étape 0, puis chaque étape). Le binaire `physarum`
//! n'est qu'une coque autour de [`lancer`] : les tests appellent `lancer` sans lancer de processus.
//!
//! # Ce que fait l'étape 0
//!
//! La grille seule : le scénario donne une grille (`grille`, `voisinage`, `etages`) et rien d'autre. Il n'y a encore
//! rien à simuler : l'état est le graphe de la grille et la graine. L'étape 0 ne comprend **aucune autre directive**
//! (voir [`crate::scenario::Scenario::directives`]) : la première directive restante fait refuser le scénario
//! ([`ErreurLancer::DirectiveInconnue`]). Les étapes suivantes feront consommer leurs directives par `lancer`.
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
    /// Le scénario contient une directive que ce lancer ne comprend pas ; le numéro de sa ligne (la première des
    /// directives restantes, dans l'ordre du fichier).
    DirectiveInconnue(usize),
}

/// Lance le scénario avec cette graine (voir la documentation du module). Pour le même scénario et la même graine, le
/// résultat est toujours le même, octet pour octet, sur toutes les machines.
///
/// # Erreurs
///
/// `ErreurLancer::DirectiveInconnue(ligne)` si `scenario.directives` n'est pas vide : `ligne` est le numéro de ligne
/// de sa première directive. Rien d'autre n'est calculé alors.
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
