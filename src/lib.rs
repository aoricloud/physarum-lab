//! Physarum Lab : le blob (*Physarum polycephalum*) simulé comme un réseau de tubes qui s'adapte, ses mesures et son
//! rendu. Le binaire `physarum` les lance ; `FEUILLE-DE-ROUTE.md` dit quelle étape remplit quel module. Les modules
//! sont déclarés d'avance, ici, pour que deux vagues n'éditent pas ce fichier ensemble.

pub mod alea;
pub mod chemins;
pub mod empreinte;
pub mod fusion;
pub mod graphe;
pub mod habituation;
pub mod image;
pub mod labyrinthe;
pub mod lancer;
pub mod mesures;
pub mod rendu;
pub mod scenario;
pub mod solveur;
pub mod souvenir;
pub mod tubes;
pub mod vitrine;
