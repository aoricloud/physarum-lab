//! L'écrivain PNG : CRC-32, Adler-32 et blocs deflate stockés, sans dépendance (étape 0). Il ne comprime rien : le
//! fichier est plus gros que l'image, mais sa structure est simple, fixée octet par octet, donc identique partout.
//!
//! # CRC-32 (celui de PNG et de zlib)
//!
//! Le CRC-32/ISO-HDLC : polynôme réfléchi `0xEDB88320`, registre initial `0xFFFFFFFF`, octets traités du premier au
//! dernier (`registre = TABLE[(registre ^ octet) & 0xFF] ^ (registre >> 8)`, où `TABLE[n]` est le résultat de huit
//! pas `si bit 0 de c : c = (c >> 1) ^ 0xEDB88320, sinon c >>= 1` à partir de `c = n`), puis `registre ^ 0xFFFFFFFF`.
//! Valeur de contrôle du catalogue de CRC (R. Williams, reveng.sourceforge.io, `CRC-32/ISO-HDLC`) : le CRC-32 de la
//! suite d'octets ASCII `123456789` est `0xCBF43926`. Le CRC-32 de la suite vide est 0.
//!
//! # Adler-32 (RFC 1950, section 8.2)
//!
//! `a = 1`, `b = 0` ; pour chaque octet : `a = (a + octet) mod 65521`, puis `b = (b + a) mod 65521` ; le résultat est
//! `(b << 16) | a`. Celui de la suite vide est 1 ; celui de `Wikipedia` est `0x11E60398` (`a = 920`, `b = 4582`,
//! exemple de l'article Adler-32). Attention : sur de longues suites les sommes dépassent 32 bits si l'on reporte le
//! modulo trop tard.
//!
//! # Le PNG écrit par [`ecrire_png`], octet par octet
//!
//! Tous les entiers de 32 bits sont en **grand-boutiste** (ordre du réseau). Un PNG RVB 8 bits, sans entrelacement :
//!
//! ```text
//! 8 octets   la signature          89 50 4E 47 0D 0A 1A 0A
//! chunk IHDR (25 octets)           00 00 00 0D | "IHDR" | largeur u32 | hauteur u32 | 08 02 00 00 00 | CRC-32
//!                                  (profondeur 8 bits, type de couleur 2 = RVB, compression 0, filtre 0, entrelacement 0)
//! chunk IDAT (12 + n octets)       n u32 | "IDAT" | le flux zlib (n octets) | CRC-32
//! chunk IEND (12 octets)           00 00 00 00 | "IEND" | AE 42 60 82
//! ```
//!
//! Un chunk est : sa longueur de données (u32, sans compter ni le type ni le CRC), son type (4 lettres ASCII), ses
//! données, puis le CRC-32 calculé sur **le type et les données** (pas sur la longueur). Il y a **un seul** chunk IDAT,
//! qui porte tout le flux zlib, et aucun autre chunk (ni `pHYs`, ni `tEXt`…) : l'ordre est signature, IHDR, IDAT, IEND, et
//! rien après IEND.
//!
//! **Les données de l'image** (avant le flux zlib) : pour chaque ligne, de haut en bas, un octet de filtre `0x00`
//! (aucun filtre) puis les `3 * largeur` octets R, V, B des pixels de la ligne, de gauche à droite. Leur nombre est
//! `L = hauteur * (1 + 3 * largeur)`.
//!
//! **Le flux zlib** (RFC 1950) : deux octets `78 01`, puis les blocs deflate stockés (RFC 1951, section 3.2.4), puis
//! l'Adler-32 des `L` octets de données en grand-boutiste (4 octets). Les données sont coupées en morceaux
//! consécutifs de **exactement 65535 octets**, sauf le dernier qui porte le reste (de 1 à 65535 octets ; si `L` est un
//! multiple de 65535, le dernier morceau est un morceau plein de 65535 octets, il n'y a jamais de bloc vide). Chaque
//! morceau est précédé de 5 octets : un octet d'en-tête (`0x00` pour un bloc non final, **`0x01` pour le dernier**
//! bloc : bit BFINAL à 1, BTYPE = 00 « stocké », puis des zéros jusqu'à l'octet), la longueur `LEN` du morceau en u16
//! **petit-boutiste**, puis `NLEN = !LEN` (le complément à un de `LEN`) en u16 **petit-boutiste**. La longueur du flux
//! zlib est donc `2 + 5 * (nombre de blocs) + L + 4`.
//!
//! Exemple (l'image de 2 × 1 pixels rouge `FF 00 00` puis bleu `00 00 FF`) : voir les tests de l'étape 0, qui donnent
//! les 75 octets du fichier un par un.

/// Le CRC-32 de `donnees` (celui de PNG et de zlib ; voir la documentation du module). `crc32(b"123456789")` vaut
/// `0xCBF43926` et `crc32(b"")` vaut 0.
#[allow(unused_variables)]
pub fn crc32(donnees: &[u8]) -> u32 {
    todo!()
}

/// L'Adler-32 de `donnees` (RFC 1950 ; voir la documentation du module). `adler32(b"Wikipedia")` vaut `0x11E60398` et
/// `adler32(b"")` vaut 1. Exact pour des données de toute longueur.
#[allow(unused_variables)]
pub fn adler32(donnees: &[u8]) -> u32 {
    todo!()
}

/// Écrit un PNG RVB 8 bits : `rvb` contient les pixels ligne par ligne, de haut en bas, chacun en trois octets R, V, B
/// (`rvb.len() = 3 * largeur * hauteur`). La structure du fichier est décrite octet par octet dans la documentation du
/// module : un seul IDAT, filtre 0 sur chaque ligne, blocs deflate stockés de 65535 octets au plus, Adler-32, IEND.
///
/// # Panique
///
/// Si `largeur` ou `hauteur` vaut 0, si `rvb.len()` n'est pas `3 * largeur * hauteur`, ou si le flux zlib dépasse
/// `2^32 - 1` octets.
#[allow(unused_variables)]
pub fn ecrire_png(largeur: u32, hauteur: u32, rvb: &[u8]) -> Vec<u8> {
    todo!()
}
