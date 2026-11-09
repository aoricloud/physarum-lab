//! Le fichier de scénario : une directive par ligne (étape 0, puis chaque étape).
//!
//! # Le format
//!
//! Le texte est découpé en lignes (`\n` ; un seul `\r` en fin de ligne est retiré) ; la première ligne est la ligne 1,
//! **les lignes ignorées comptent**. Seuls l'espace et la tabulation sont des blancs. Une ligne est :
//! - **ignorée** si, une fois ses blancs de bord retirés, elle est vide ou commence par `#` (un commentaire tient sur
//!   toute la ligne : `grille 3 3 # note` n'est pas un commentaire de fin de ligne, c'est une directive à cinq champs) ;
//! - sinon une **directive** : un nom puis zéro ou plusieurs arguments, tous séparés par un ou plusieurs espaces ou
//!   tabulations. Le nom est **non vide** et fait de lettres minuscules ASCII `a-z`, de chiffres `0-9` et de `_`
//!   (`Grille`, `grille-2`, `é` sont mal formés). Un argument est un mot quelconque sans espace.
//!
//! Trois directives sont comprises ici ; elles décrivent la [`Grille`] du scénario et peuvent venir dans n'importe quel
//! ordre :
//! - `grille <largeur> <hauteur>` : **obligatoire, une seule fois** ; deux entiers de 1 à [`COTE_MAX`] ;
//! - `voisinage <4|8>` : facultative, une fois au plus ; l'argument est exactement `4` (`Voisinage::Quatre`, le défaut
//!   quand la directive manque) ou `8` (`Voisinage::Huit`) : `04`, `08`, `6` sont refusés ;
//! - `etages <nombre> <longueur> <cout>` : facultative, une fois au plus ; `nombre` est un entier de 1 à
//!   [`ETAGES_MAX`] ; `longueur` et `cout` sont deux flottants (écrits comme le veut [`Directive::flottant`]) finis et
//!   **strictement positifs** : la longueur et le coût d'un lien vertical. Quand la directive manque : un étage, et
//!   `1.0` pour les deux nombres.
//!
//! Un entier est écrit en **chiffres décimaux ASCII seulement** : ni signe (`+5` est refusé), ni espace, ni point ;
//! les zéros de tête sont permis (`064` vaut 64). Il doit tenir dans les bornes (une valeur trop grande pour un `u32`
//! est simplement hors bornes).
//!
//! **Toute autre directive bien formée est conservée sans être interprétée** dans [`Scenario::directives`], dans
//! l'ordre du fichier, avec son numéro de ligne : les étapes suivantes y liront leurs propres directives. Les doublons
//! de ces directives sont conservés tous ; [`Scenario::directive`] rend la première. Ce module ne dit pas si un nom est
//! connu : c'est `lancer` qui refuse une directive qu'il ne comprend pas.
//!
//! # Les erreurs
//!
//! La **première ligne fautive, dans l'ordre du fichier,** donne `ErreurLecture::LigneInvalide(numéro)` ; elle l'emporte
//! sur `Manquante`. Sont fautives : un nom mal formé ; `grille`, `voisinage` ou `etages` avec le mauvais nombre
//! d'arguments ou un argument mal écrit ou hors bornes ; une **seconde** `grille`, un second `voisinage` ou un second
//! `etages` (c'est la seconde ligne qui est fautive, même si ses arguments sont bons). Si aucune ligne n'est fautive
//! mais qu'aucune `grille` n'a été donnée (fichier vide, que des commentaires, que d'autres directives, même
//! `voisinage` ou `etages`), l'erreur est `ErreurLecture::Manquante("grille")`.

use crate::graphe::{Grille, Voisinage};

/// La borne haute de `largeur` et de `hauteur` d'une `grille`.
pub const COTE_MAX: u32 = 256;

/// La borne haute du nombre d'étages.
pub const ETAGES_MAX: u32 = 8;

/// Une directive du fichier que `lire_scenario` ne comprend pas elle-même : son nom et ses arguments tels qu'écrits.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub struct Directive {
    /// Le numéro de la ligne du fichier (à partir de 1, lignes ignorées comptées).
    pub ligne: usize,
    /// Le nom (lettres minuscules, chiffres, `_`).
    pub nom: String,
    /// Les arguments, dans l'ordre, tels qu'écrits (zéro ou plus).
    pub args: Vec<String>,
}

/// Un scénario lu. On l'obtient par [`lire_scenario`] (le type est `non_exhaustive` : les étapes suivantes peuvent y
/// ajouter des champs, et un test ne le construit jamais par un littéral).
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
#[allow(dead_code)]
pub struct Scenario {
    /// La grille, donnée par `grille`, `voisinage` et `etages` (toujours valide : `grille.graphe()` réussit).
    pub grille: Grille,
    /// Toutes les autres directives, dans l'ordre du fichier (voir la documentation du module).
    pub directives: Vec<Directive>,
}

/// Pourquoi un texte de scénario n'a pas pu être lu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurLecture {
    /// La ligne de ce numéro (à partir de 1) est fautive, ou une directive en rend une (voir [`Directive::entier`]).
    LigneInvalide(usize),
    /// Une directive obligatoire manque : le nom de ce qui manque. À l'étape 0, c'est toujours `"grille"`.
    Manquante(&'static str),
}

/// Lit un texte de scénario (voir la documentation du module pour le format et les erreurs).
pub fn lire_scenario(texte: &str) -> Result<Scenario, ErreurLecture> {
    let mut grille: Option<(u32, u32)> = None;
    let mut voisinage: Option<Voisinage> = None;
    let mut etages: Option<(u32, f64, f64)> = None;
    let mut directives: Vec<Directive> = Vec::new();

    for (rang, brute) in texte.split('\n').enumerate() {
        let ligne = rang + 1;
        // Un seul `\r` en fin de ligne est retiré ; les blancs de bord sont l'espace et la tabulation seuls.
        let contenu = brute.strip_suffix('\r').unwrap_or(brute).trim_matches([' ', '\t']);
        if contenu.is_empty() || contenu.starts_with('#') {
            continue;
        }
        let mut champs = contenu.split([' ', '\t']).filter(|champ| !champ.is_empty());
        let nom = champs.next().expect("un contenu non vide a un premier champ");
        if !nom_valide(nom) {
            return Err(ErreurLecture::LigneInvalide(ligne));
        }
        let args: Vec<String> = champs.map(str::to_owned).collect();
        match nom {
            "grille" => {
                if grille.is_some() || args.len() != 2 {
                    return Err(ErreurLecture::LigneInvalide(ligne));
                }
                let Some(largeur) = cote(&args[0]) else {
                    return Err(ErreurLecture::LigneInvalide(ligne));
                };
                let Some(hauteur) = cote(&args[1]) else {
                    return Err(ErreurLecture::LigneInvalide(ligne));
                };
                grille = Some((largeur, hauteur));
            }
            "voisinage" => {
                if voisinage.is_some() || args.len() != 1 {
                    return Err(ErreurLecture::LigneInvalide(ligne));
                }
                voisinage = Some(match args[0].as_str() {
                    "4" => Voisinage::Quatre,
                    "8" => Voisinage::Huit,
                    _ => return Err(ErreurLecture::LigneInvalide(ligne)),
                });
            }
            "etages" => {
                if etages.is_some() || args.len() != 3 {
                    return Err(ErreurLecture::LigneInvalide(ligne));
                }
                let Some(nombre) = entier_u64(&args[0]).filter(|v| (1..=u64::from(ETAGES_MAX)).contains(v)) else {
                    return Err(ErreurLecture::LigneInvalide(ligne));
                };
                let Some(longueur) = flottant_fini(&args[1]).filter(|v| *v > 0.0) else {
                    return Err(ErreurLecture::LigneInvalide(ligne));
                };
                let Some(cout) = flottant_fini(&args[2]).filter(|v| *v > 0.0) else {
                    return Err(ErreurLecture::LigneInvalide(ligne));
                };
                etages = Some((nombre as u32, longueur, cout));
            }
            _ => directives.push(Directive { ligne, nom: nom.to_owned(), args }),
        }
    }

    let (largeur, hauteur) = grille.ok_or(ErreurLecture::Manquante("grille"))?;
    let (etages, longueur_verticale, cout_vertical) = etages.unwrap_or((1, 1.0, 1.0));
    Ok(Scenario {
        grille: Grille {
            largeur,
            hauteur,
            etages,
            voisinage: voisinage.unwrap_or(Voisinage::Quatre),
            longueur_verticale,
            cout_vertical,
        },
        directives,
    })
}

/// Un nom de directive bien formé : non vide, fait de lettres minuscules ASCII, de chiffres et de `_`.
fn nom_valide(nom: &str) -> bool {
    !nom.is_empty() && nom.bytes().all(|octet| octet.is_ascii_lowercase() || octet.is_ascii_digit() || octet == b'_')
}

/// Un entier en chiffres décimaux ASCII seulement (non vide, sans signe), d'au plus `u64::MAX`.
fn entier_u64(mot: &str) -> Option<u64> {
    if mot.is_empty() {
        return None;
    }
    let mut valeur: u64 = 0;
    for octet in mot.bytes() {
        if !octet.is_ascii_digit() {
            return None;
        }
        valeur = valeur.checked_mul(10)?.checked_add(u64::from(octet - b'0'))?;
    }
    Some(valeur)
}

/// Un côté de grille : `1..=COTE_MAX`.
fn cote(mot: &str) -> Option<u32> {
    let valeur = entier_u64(mot)?;
    if (1..=u64::from(COTE_MAX)).contains(&valeur) { Some(valeur as u32) } else { None }
}

/// Un flottant décimal simple, fini : `-?chiffres(.chiffres)?` (voir [`Directive::flottant`]).
fn flottant_fini(mot: &str) -> Option<f64> {
    if !forme_flottant(mot) {
        return None;
    }
    let valeur: f64 = mot.parse().ok()?;
    if valeur.is_finite() { Some(valeur) } else { None }
}

/// La forme d'un flottant, vérifiée **avant** `parse` : un `-` facultatif, des chiffres (au moins un), puis
/// facultativement un `.` suivi d'au moins un chiffre.
fn forme_flottant(mot: &str) -> bool {
    let chiffres = mot.strip_prefix('-').unwrap_or(mot);
    let mut parties = chiffres.split('.');
    let entiere = parties.next().unwrap_or_default();
    if entiere.is_empty() || !entiere.bytes().all(|octet| octet.is_ascii_digit()) {
        return false;
    }
    match parties.next() {
        None => true,
        Some(decimale) => {
            !decimale.is_empty() && decimale.bytes().all(|octet| octet.is_ascii_digit()) && parties.next().is_none()
        }
    }
}

impl Scenario {
    /// La première directive de ce nom, dans l'ordre du fichier, parmi [`Scenario::directives`], ou `None` (toujours
    /// `None` pour `grille`, `voisinage` et `etages`, qui n'y sont pas).
    pub fn directive(&self, nom: &str) -> Option<&Directive> {
        self.directives.iter().find(|directive| directive.nom == nom)
    }
}

impl Directive {
    /// L'argument `i` (le premier est 0) lu comme un entier non signé : des chiffres décimaux ASCII seulement (non vide,
    /// sans signe ; les zéros de tête sont permis), valeur d'au plus `u64::MAX`.
    ///
    /// # Erreurs
    ///
    /// `ErreurLecture::LigneInvalide(self.ligne)` si l'argument est absent, n'est pas écrit ainsi, ou dépasse `u64::MAX`.
    pub fn entier(&self, i: usize) -> Result<u64, ErreurLecture> {
        self.args.get(i).and_then(|mot| entier_u64(mot)).ok_or(ErreurLecture::LigneInvalide(self.ligne))
    }

    /// L'argument `i` lu comme un flottant, en notation décimale simple : un `-` facultatif, des chiffres (au moins un),
    /// puis facultativement un `.` suivi d'au moins un chiffre. Ni `+`, ni exposant (`1e3`), ni `.5`, ni `5.`, ni
    /// `inf` ni `nan`. La valeur est l'arrondi au plus proche du nombre écrit (c'est ce que fait `str::parse::<f64>`,
    /// qui n'utilise pas la bibliothèque mathématique du système) ; `-0` donne `-0.0`. La forme se vérifie **avant**
    /// d'appeler `parse`, qui accepterait à lui seul `+1`, `1e3`, `.5` et `inf`.
    ///
    /// # Erreurs
    ///
    /// `ErreurLecture::LigneInvalide(self.ligne)` si l'argument est absent, n'a pas cette forme, ou n'est pas fini
    /// (quatre cents chiffres `9`).
    pub fn flottant(&self, i: usize) -> Result<f64, ErreurLecture> {
        self.args.get(i).and_then(|mot| flottant_fini(mot)).ok_or(ErreurLecture::LigneInvalide(self.ligne))
    }
}
