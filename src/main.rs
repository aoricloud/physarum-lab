//! Le binaire `physarum` : `physarum lancer <scénario> --graine <n> --sortie <dossier>` (étape 0).
//!
//! - `<scénario>` : le chemin d'un fichier de scénario (voir `physarum_lab::scenario`). `--graine <n>` : un entier non signé
//!   de 64 bits écrit en chiffres décimaux seulement (sans signe). `--sortie <dossier>` : le dossier des sorties, créé
//!   (avec ses parents) s'il manque. Les deux options figurent **chacune une fois**, dans n'importe quel ordre, et le
//!   chemin du scénario est le seul argument qui ne soit pas une option ni la valeur d'une option.
//! - Il lit le fichier, appelle `physarum_lab::lancer::lancer` et écrit dans le dossier trois fichiers, et aucun autre :
//!   `empreinte.txt` (l'empreinte, 64 chiffres hexadécimaux, puis un `\n`), `mesures.csv` (les mesures, telles que
//!   rendues) et `apercu.png` (l'aperçu, tel que rendu). Un dossier qui contient déjà ces fichiers les voit remplacés.
//! - Il ne rend rien d'autre que ces fichiers (rien d'imposé sur la sortie standard).
//! - **Toute erreur** (aucune commande ou une commande autre que `lancer` ; une option inconnue, répétée, sans valeur ou
//!   absente ; une graine mal écrite ou qui dépasse 64 bits ; un scénario en trop ; un fichier illisible ; un scénario
//!   invalide ou refusé par `lancer` ; un dossier impossible à créer) : un message d'une ligne au moins sur la sortie
//!   d'erreur (sans `panicked`, car ce n'est pas un plantage), le **code de sortie 1**, et **rien d'écrit ni de créé**
//!   (le dossier de sortie n'est pas créé quand le scénario est refusé). Le message d'une erreur de scénario contient
//!   `ligne <n>` (le numéro décimal de la ligne fautive, par exemple `ligne 7`, ou, pour une directive que `lancer` refuse,
//!   le numéro de la ligne de cette directive) ; celui d'une grille manquante contient le mot `grille`.
//! - Code de sortie 0 quand tout est écrit.

fn main() -> std::process::ExitCode {
    todo!()
}
