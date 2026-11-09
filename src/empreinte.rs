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
pub fn sha256(donnees: &[u8]) -> [u8; 32] {
    let mut h = H_INITIAL;
    let (blocs, reste) = donnees.as_chunks::<64>();
    for bloc in blocs {
        traiter_bloc(&mut h, bloc);
    }

    // Bourrage (section 5.1.1) : 0x80, des zéros, puis L en bits sur 64 bits grand-boutistes. Le reste occupe les
    // derniers octets d'un bloc s'il tient dans les 56 premiers octets, sinon un second bloc.
    let mut fin = [0_u8; 128];
    fin[..reste.len()].copy_from_slice(reste);
    fin[reste.len()] = 0x80;
    let longueur = if reste.len() < 56 { 64 } else { 128 };
    let longueur_bits = (donnees.len() as u64).wrapping_mul(8);
    fin[longueur - 8..longueur].copy_from_slice(&longueur_bits.to_be_bytes());
    for bloc in fin[..longueur].as_chunks::<64>().0 {
        traiter_bloc(&mut h, bloc);
    }

    let mut empreinte = [0_u8; 32];
    for (mot, octets) in h.iter().zip(empreinte.as_chunks_mut::<4>().0) {
        octets.copy_from_slice(&mot.to_be_bytes());
    }
    empreinte
}

/// Le SHA-256 de `donnees` écrit en 64 chiffres hexadécimaux **minuscules** (deux par octet, du premier octet au
/// dernier, sans séparateur ni préfixe).
pub fn sha256_hex(donnees: &[u8]) -> String {
    const CHIFFRES: [u8; 16] = *b"0123456789abcdef";
    let empreinte = sha256(donnees);
    let mut texte = String::with_capacity(64);
    for octet in empreinte {
        texte.push(CHIFFRES[usize::from(octet >> 4)] as char);
        texte.push(CHIFFRES[usize::from(octet & 0x0F)] as char);
    }
    texte
}

/// L'état initial `H` (les 32 premiers bits des parties fractionnaires des racines carrées des huit premiers nombres
/// premiers).
const H_INITIAL: [u32; 8] =
    [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];

/// Les 64 constantes `K` (les 32 premiers bits des parties fractionnaires des racines cubiques des 64 premiers nombres
/// premiers).
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98,
    0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
    0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8,
    0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819,
    0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
    0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
    0xc67178f2,
];

/// `σ0(x) = rotd(x,7) ^ rotd(x,18) ^ (x >> 3)` (échéancier).
fn petite_sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}

/// `σ1(x) = rotd(x,17) ^ rotd(x,19) ^ (x >> 10)` (échéancier).
fn petite_sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}

/// `Σ0(x) = rotd(x,2) ^ rotd(x,13) ^ rotd(x,22)`.
fn grande_sigma0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}

/// `Σ1(x) = rotd(x,6) ^ rotd(x,11) ^ rotd(x,25)`.
fn grande_sigma1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}

/// `Ch(x,y,z) = (x & y) ^ (!x & z)`.
fn ch(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (!x & z)
}

/// `Maj(x,y,z) = (x & y) ^ (x & z) ^ (y & z)`.
fn maj(x: u32, y: u32, z: u32) -> u32 {
    (x & y) ^ (x & z) ^ (y & z)
}

/// Traite un bloc de 64 octets : l'échéancier `W`, les 64 tours, puis l'ajout des registres à `h` (modulo 2^32).
fn traiter_bloc(h: &mut [u32; 8], bloc: &[u8]) {
    let mut w = [0_u32; 64];
    for (t, octets) in bloc.as_chunks::<4>().0.iter().enumerate() {
        w[t] = u32::from_be_bytes(*octets);
    }
    for t in 16..64 {
        w[t] = petite_sigma1(w[t - 2])
            .wrapping_add(w[t - 7])
            .wrapping_add(petite_sigma0(w[t - 15]))
            .wrapping_add(w[t - 16]);
    }

    let mut a = h[0];
    let mut b = h[1];
    let mut c = h[2];
    let mut d = h[3];
    let mut e = h[4];
    let mut f = h[5];
    let mut g = h[6];
    let mut hh = h[7];
    for t in 0..64 {
        let t1 = hh.wrapping_add(grande_sigma1(e)).wrapping_add(ch(e, f, g)).wrapping_add(K[t]).wrapping_add(w[t]);
        let t2 = grande_sigma0(a).wrapping_add(maj(a, b, c));
        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }

    h[0] = h[0].wrapping_add(a);
    h[1] = h[1].wrapping_add(b);
    h[2] = h[2].wrapping_add(c);
    h[3] = h[3].wrapping_add(d);
    h[4] = h[4].wrapping_add(e);
    h[5] = h[5].wrapping_add(f);
    h[6] = h[6].wrapping_add(g);
    h[7] = h[7].wrapping_add(hh);
}
