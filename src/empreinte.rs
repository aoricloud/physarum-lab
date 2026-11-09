//! L'empreinte de l'état : SHA-256 (FIPS 180-4), écrit dans le dépôt, sans dépendance (étape 0). L'empreinte d'un état
//! simulé est le SHA-256 de son écriture canonique (voir `lancer` pour celle de l'étape 0).
//!
//! # L'algorithme (FIPS 180-4, section 6.2)
//!
//! Le message (une suite d'octets de `L` bits) est complété (« bourrage », section 5.1.1) : un bit 1 (l'octet `0x80`),
//! puis autant de bits 0 qu'il faut pour que la longueur totale soit congrue à 448 modulo 512 bits, puis `L` écrit sur
//! 64 bits en grand-boutiste. Le résultat est découpé en blocs de 512 bits (64 octets). Les mots de 32 bits sont lus en
//! grand-boutiste. Le hachage part de `H = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c,
//! 0x1f83d9ab, 0x5be0cd19]` (les 32 premiers bits des parties fractionnaires des racines carrées des huit premiers
//! nombres premiers) et traite chaque bloc avec les 64 constantes `K[0..64]` (les 32 premiers bits des parties
//! fractionnaires des racines cubiques des 64 premiers nombres premiers, de `0x428a2f98` à `0xc67178f2`) :
//!
//! - échéancier : `W[t]` = le mot `t` du bloc pour `t < 16`, et pour `16 <= t < 64`
//!   `W[t] = σ1(W[t-2]) + W[t-7] + σ0(W[t-15]) + W[t-16]` (modulo 2^32), avec
//!   `σ0(x) = rotd(x,7) ^ rotd(x,18) ^ (x >> 3)` et `σ1(x) = rotd(x,17) ^ rotd(x,19) ^ (x >> 10)` ;
//! - 64 tours à partir de `(a, b, c, d, e, f, g, h) = H` : `T1 = h + Σ1(e) + Ch(e,f,g) + K[t] + W[t]`,
//!   `T2 = Σ0(a) + Maj(a,b,c)`, puis `h = g`, `g = f`, `f = e`, `e = d + T1`, `d = c`, `c = b`, `b = a`, `a = T1 + T2`
//!   (tout modulo 2^32), avec `Σ0(x) = rotd(x,2) ^ rotd(x,13) ^ rotd(x,22)`, `Σ1(x) = rotd(x,6) ^ rotd(x,11) ^
//!   rotd(x,25)`, `Ch(x,y,z) = (x & y) ^ (!x & z)`, `Maj(x,y,z) = (x & y) ^ (x & z) ^ (y & z)` (`rotd` : rotation
//!   vers la droite) ;
//! - fin du bloc : chaque `H[i]` reçoit son registre (`H[0] += a`, …, `H[7] += h`, modulo 2^32).
//!
//! L'empreinte est la concaténation des huit mots de `H` en grand-boutiste : 32 octets.
//!
//! # Les vecteurs de test
//!
//! Lus dans les exemples du NIST (« SHA-256 example », SHA256.pdf et SHA2_Additional.pdf) et dans FIPS 180-2,
//! annexe B : voir le fichier de tests de l'étape 0 pour la liste.

/// Le SHA-256 de `donnees` : 32 octets (voir la documentation du module).
#[allow(unused_variables)]
pub fn sha256(donnees: &[u8]) -> [u8; 32] {
    todo!()
}

/// Le SHA-256 de `donnees` écrit en 64 chiffres hexadécimaux **minuscules** (deux par octet, du premier octet au
/// dernier, sans séparateur ni préfixe).
#[allow(unused_variables)]
pub fn sha256_hex(donnees: &[u8]) -> String {
    todo!()
}
