//! Le binaire `physarum` : `physarum lancer <scénario> --graine <n> --sortie <dossier>` (étape 0, puis chaque étape).
//!
//! - `<scénario>` : le chemin d'un fichier de scénario (voir `physarum_lab::scenario`). `--graine <n>` : un entier non
//!   signé de 64 bits écrit en chiffres décimaux seulement (sans signe). `--sortie <dossier>` : le dossier des sorties,
//!   créé (avec ses parents) s'il manque. Les deux options figurent **chacune une fois**, dans n'importe quel ordre, et
//!   le chemin du scénario est le seul argument qui ne soit pas une option ni la valeur d'une option.
//! - Il lit le fichier, appelle `physarum_lab::lancer::lancer` et écrit dans le dossier trois fichiers, et aucun autre
//!   : `empreinte.txt` (l'empreinte, 64 chiffres hexadécimaux, puis un `\n`), `mesures.csv` (les mesures, telles que
//!   rendues) et `apercu.png` (l'aperçu, tel que rendu). Un dossier qui contient déjà ces fichiers les voit remplacés.
//! - Il ne rend rien d'autre que ces fichiers (rien d'imposé sur la sortie standard).
//! - **Toute erreur** (aucune commande ou une commande autre que `lancer` ; une option inconnue, répétée, sans valeur
//!   ou absente ; une graine mal écrite ou qui dépasse 64 bits ; un scénario en trop ; un fichier illisible ; un
//!   scénario invalide ou refusé par `lancer` ; un calcul qui s'arrête ; un dossier impossible à créer) : un message
//!   d'une ligne au moins sur la sortie d'erreur (sans `panicked`, car ce n'est pas un plantage), le **code de sortie
//!   1**, et **rien d'écrit ni de créé** (le dossier de sortie n'est pas créé quand le scénario est refusé ou que le
//!   calcul s'arrête). Le message d'une erreur de scénario contient `ligne <n>` (le numéro décimal de la ligne
//!   fautive, par exemple `ligne 7`, ou, pour une directive que `lancer` refuse, le numéro de la ligne de cette
//!   directive) ; celui d'une grille manquante contient le mot `grille` ; celui d'un calcul qui s'arrête
//!   (`ErreurLancer::Calcul(t)`, étape 1) contient `pas <t>` (le numéro décimal du pas, par exemple `pas 0`).
//! - **Une écriture qui échoue** est une erreur comme les autres : avant de rendre l'erreur, le binaire retire les
//!   fichiers qu'il a écrits pendant ce lancer ; il ne reste dans le dossier de sortie que ce qui s'y trouvait avant.
//! - Code de sortie 0 quand tout est écrit.

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use physarum_lab::lancer::{ErreurLancer, lancer};
use physarum_lab::scenario::{ErreurLecture, lire_scenario};

fn main() -> std::process::ExitCode {
    match executer(std::env::args_os().skip(1).collect()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("physarum : {message}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// Les trois choses qu'un lancer demande, une fois les arguments vérifiés.
struct Demande {
    scenario: PathBuf,
    graine: u64,
    sortie: PathBuf,
}

/// Vérifie et range les arguments (le nom du programme excepté).
fn lire_arguments(args: &[OsString]) -> Result<Demande, String> {
    let Some(commande) = args.first() else {
        return Err("aucune commande : attendu `lancer`".to_owned());
    };
    if commande.as_os_str() != OsStr::new("lancer") {
        return Err(format!("commande inconnue : {}", commande.to_string_lossy()));
    }
    let mut scenario: Option<PathBuf> = None;
    let mut graine: Option<u64> = None;
    let mut sortie: Option<PathBuf> = None;
    let mut rang = 1;
    while rang < args.len() {
        let argument = &args[rang];
        if argument.as_os_str() == OsStr::new("--graine") {
            if graine.is_some() {
                return Err("l'option --graine est répétée".to_owned());
            }
            let valeur = args.get(rang + 1).ok_or_else(|| "l'option --graine est sans valeur".to_owned())?;
            graine = Some(graine_lue(valeur)?);
            rang += 2;
        } else if argument.as_os_str() == OsStr::new("--sortie") {
            if sortie.is_some() {
                return Err("l'option --sortie est répétée".to_owned());
            }
            let valeur = args.get(rang + 1).ok_or_else(|| "l'option --sortie est sans valeur".to_owned())?;
            sortie = Some(PathBuf::from(valeur));
            rang += 2;
        } else if argument.to_string_lossy().starts_with('-') {
            return Err(format!("option inconnue : {}", argument.to_string_lossy()));
        } else if scenario.is_some() {
            return Err(format!("scénario en trop : {}", argument.to_string_lossy()));
        } else {
            scenario = Some(PathBuf::from(argument));
            rang += 1;
        }
    }
    Ok(Demande {
        scenario: scenario.ok_or_else(|| "le scénario manque".to_owned())?,
        graine: graine.ok_or_else(|| "l'option --graine manque".to_owned())?,
        sortie: sortie.ok_or_else(|| "l'option --sortie manque".to_owned())?,
    })
}

/// La graine : des chiffres décimaux ASCII seulement (ni signe ni blanc), tenant dans un `u64`.
fn graine_lue(valeur: &OsString) -> Result<u64, String> {
    let texte = valeur.to_str().ok_or_else(|| "la graine n'est pas écrite en chiffres décimaux".to_owned())?;
    if texte.is_empty() || !texte.bytes().all(|octet| octet.is_ascii_digit()) {
        return Err(format!("graine mal écrite : {}", valeur.to_string_lossy()));
    }
    texte.parse::<u64>().map_err(|_| format!("graine hors des 64 bits : {texte}"))
}

/// Lit le scénario, le lance, puis écrit les trois sorties (et rien avant que tout ait réussi).
fn executer(args: Vec<OsString>) -> Result<(), String> {
    let demande = lire_arguments(&args)?;
    let texte = std::fs::read_to_string(&demande.scenario)
        .map_err(|erreur| format!("scénario illisible ({}) : {erreur}", demande.scenario.display()))?;
    let scenario = lire_scenario(&texte).map_err(|erreur| message_de_lecture(&demande.scenario, erreur))?;
    let sortie = lancer(&scenario, demande.graine).map_err(|erreur| message_de_lancer(&demande.scenario, erreur))?;
    std::fs::create_dir_all(&demande.sortie)
        .map_err(|erreur| format!("dossier de sortie impossible à créer ({}) : {erreur}", demande.sortie.display()))?;
    let mut ecrits: Vec<PathBuf> = Vec::new();
    ecrire(&demande.sortie.join("empreinte.txt"), format!("{}\n", sortie.empreinte).as_bytes(), &mut ecrits)?;
    ecrire(&demande.sortie.join("mesures.csv"), sortie.mesures.as_bytes(), &mut ecrits)?;
    ecrire(&demande.sortie.join("apercu.png"), &sortie.apercu, &mut ecrits)
}

/// Le message d'une erreur de lecture : le numéro de la ligne fautive, ou le nom de la directive manquante.
fn message_de_lecture(chemin: &Path, erreur: ErreurLecture) -> String {
    match erreur {
        ErreurLecture::LigneInvalide(ligne) => format!("scénario {} : ligne {ligne} invalide", chemin.display()),
        ErreurLecture::Manquante(nom) => {
            format!("scénario {} : directive obligatoire manquante : {nom}", chemin.display())
        }
    }
}

/// Le message d'une erreur de lancer : le numéro de la ligne de la directive incomprise, ou le numéro du pas où le
/// calcul s'est arrêté.
fn message_de_lancer(chemin: &Path, erreur: ErreurLancer) -> String {
    match erreur {
        ErreurLancer::DirectiveInconnue(ligne) => {
            format!("scénario {} : directive inconnue à la ligne {ligne}", chemin.display())
        }
        ErreurLancer::Calcul(pas) => {
            format!("scénario {} : le calcul s'arrête au pas {pas} (conductances trop étalées)", chemin.display())
        }
    }
}

/// Écrit un fichier de sortie, en remplaçant celui qui existe ; si l'écriture échoue, retire ceux que ce lancer a déjà
/// écrits (sans s'arrêter si un retrait échoue), puis rend l'erreur de l'écriture.
fn ecrire(chemin: &Path, contenu: &[u8], ecrits: &mut Vec<PathBuf>) -> Result<(), String> {
    match std::fs::write(chemin, contenu) {
        Ok(()) => {
            ecrits.push(chemin.to_path_buf());
            Ok(())
        }
        Err(erreur) => {
            for ecrit in ecrits.iter() {
                let _ = std::fs::remove_file(ecrit);
            }
            Err(format!("écriture impossible ({}) : {erreur}", chemin.display()))
        }
    }
}
