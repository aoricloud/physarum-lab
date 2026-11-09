//! Aides communes aux tests de l'étape 0 : un décodeur PNG indépendant du crate (son propre CRC-32 et son propre
//! Adler-32, écrits ici, lents et naïfs), un hexadécimal, des dossiers temporaires.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use std::path::PathBuf;

/// Le CRC-32 de PNG, bit par bit (polynôme réfléchi 0xEDB88320), écrit pour le décodeur de test : il ne partage rien avec
/// `physarum_lab::image::crc32`.
pub fn crc32_naif(donnees: &[u8]) -> u32 {
    let mut registre = 0xFFFF_FFFF_u32;
    for &octet in donnees {
        registre ^= u32::from(octet);
        for _ in 0..8 {
            registre = if registre & 1 == 1 { (registre >> 1) ^ 0xEDB8_8320 } else { registre >> 1 };
        }
    }
    !registre
}

/// L'Adler-32 de la RFC 1950, avec un modulo à chaque octet (aucune astuce), en `u64`.
pub fn adler32_naif(donnees: &[u8]) -> u32 {
    let (mut a, mut b) = (1_u64, 0_u64);
    for &octet in donnees {
        a = (a + u64::from(octet)) % 65521;
        b = (b + a) % 65521;
    }
    ((b << 16) | a) as u32
}

/// Ce qu'un PNG relu contient.
#[derive(Debug)]
pub struct PngLu {
    pub largeur: u32,
    pub hauteur: u32,
    /// Les pixels R, V, B, ligne par ligne, sans les octets de filtre.
    pub rvb: Vec<u8>,
    /// Les longueurs des blocs stockés, dans l'ordre.
    pub blocs: Vec<usize>,
    /// La longueur des données du chunk IDAT (le flux zlib entier).
    pub longueur_idat: usize,
}

fn u32_be(octets: &[u8]) -> u32 {
    u32::from_be_bytes([octets[0], octets[1], octets[2], octets[3]])
}

/// Relit un PNG octet par octet, en refusant tout ce que `physarum_lab::image::ecrire_png` doit respecter : la signature,
/// exactement trois chunks (IHDR de 13 octets, un seul IDAT, IEND vide) dans cet ordre et rien après, chaque CRC-32
/// recalculé, l'IHDR (RVB 8 bits, sans entrelacement), le flux zlib (`78 01`), des blocs stockés dont tous sont de
/// 65535 octets sauf le dernier (de 1 à 65535 octets, seul marqué final, en-tête `0x00` ou `0x01`, `NLEN = !LEN`),
/// l'Adler-32 final, un filtre 0 devant chaque ligne.
pub fn relire_png(png: &[u8]) -> Result<PngLu, String> {
    if png.len() < 8 || png[..8] != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return Err("signature PNG absente".into());
    }
    let mut position = 8;
    let mut chunks: Vec<(String, Vec<u8>)> = Vec::new();
    while position < png.len() {
        if position + 12 > png.len() {
            return Err(format!("chunk tronqué à l'octet {position}"));
        }
        let longueur = u32_be(&png[position..]) as usize;
        let fin_donnees = position + 8 + longueur;
        if fin_donnees + 4 > png.len() {
            return Err(format!("chunk trop long à l'octet {position}"));
        }
        let type_et_donnees = &png[position + 4..fin_donnees];
        let crc_ecrit = u32_be(&png[fin_donnees..]);
        let crc_calcule = crc32_naif(type_et_donnees);
        if crc_ecrit != crc_calcule {
            return Err(format!(
                "CRC faux au chunk de l'octet {position} : écrit {crc_ecrit:08X}, calculé {crc_calcule:08X}"
            ));
        }
        let type_chunk = String::from_utf8_lossy(&type_et_donnees[..4]).into_owned();
        chunks.push((type_chunk, type_et_donnees[4..].to_vec()));
        position = fin_donnees + 4;
    }
    let types: Vec<&str> = chunks.iter().map(|(t, _)| t.as_str()).collect();
    if types != ["IHDR", "IDAT", "IEND"] {
        return Err(format!("chunks attendus IHDR, IDAT, IEND ; trouvés {types:?}"));
    }
    let ihdr = &chunks[0].1;
    if ihdr.len() != 13 {
        return Err("IHDR n'a pas 13 octets".into());
    }
    let (largeur, hauteur) = (u32_be(&ihdr[0..]), u32_be(&ihdr[4..]));
    if ihdr[8..13] != [8, 2, 0, 0, 0] {
        return Err(format!("IHDR : profondeur, type, compression, filtre, entrelacement = {:?}", &ihdr[8..13]));
    }
    if !chunks[2].1.is_empty() {
        return Err("IEND n'est pas vide".into());
    }

    let zlib = &chunks[1].1;
    if zlib.len() < 2 || zlib[..2] != [0x78, 0x01] {
        return Err("l'en-tête zlib n'est pas 78 01".into());
    }
    let mut position = 2;
    let mut donnees: Vec<u8> = Vec::new();
    let mut blocs = Vec::new();
    loop {
        if position + 5 > zlib.len() {
            return Err("bloc stocké tronqué".into());
        }
        let entete = zlib[position];
        if entete > 1 {
            return Err(format!("en-tête de bloc {entete:#04X} : seuls 0x00 et 0x01 sont permis"));
        }
        let longueur = usize::from(u16::from_le_bytes([zlib[position + 1], zlib[position + 2]]));
        let complement = u16::from_le_bytes([zlib[position + 3], zlib[position + 4]]);
        if complement != !(longueur as u16) {
            return Err("NLEN n'est pas le complément de LEN".into());
        }
        let debut = position + 5;
        if debut + longueur > zlib.len() {
            return Err("bloc stocké plus long que le flux".into());
        }
        donnees.extend_from_slice(&zlib[debut..debut + longueur]);
        blocs.push(longueur);
        position = debut + longueur;
        if entete == 1 {
            break;
        }
        if longueur != 65535 {
            return Err(format!("un bloc non final de {longueur} octets (attendu 65535)"));
        }
    }
    let dernier = *blocs.last().unwrap();
    if dernier == 0 {
        return Err("le dernier bloc est vide".into());
    }
    if zlib.len() != position + 4 {
        return Err(format!("{} octets après le dernier bloc, attendu 4 (Adler-32)", zlib.len() - position));
    }
    let adler_ecrit = u32_be(&zlib[position..]);
    let adler_calcule = adler32_naif(&donnees);
    if adler_ecrit != adler_calcule {
        return Err(format!("Adler-32 faux : écrit {adler_ecrit:08X}, calculé {adler_calcule:08X}"));
    }

    let ligne = 3 * largeur as usize;
    if donnees.len() != hauteur as usize * (1 + ligne) {
        return Err(format!("{} octets de données, attendu {}", donnees.len(), hauteur as usize * (1 + ligne)));
    }
    let mut rvb = Vec::with_capacity(hauteur as usize * ligne);
    for (y, rangee) in donnees.chunks_exact(1 + ligne).enumerate() {
        if rangee[0] != 0 {
            return Err(format!("la ligne {y} a pour filtre {} (attendu 0)", rangee[0]));
        }
        rvb.extend_from_slice(&rangee[1..]);
    }
    Ok(PngLu { largeur, hauteur, rvb, blocs, longueur_idat: zlib.len() })
}

/// Un hexadécimal minuscule, écrit ici (sans passer par le crate).
pub fn hex(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

/// Un dossier temporaire propre à ce processus et à ce nom : supprimé s'il existe, **non créé**.
pub fn dossier_temporaire(nom: &str) -> PathBuf {
    let dossier = std::env::temp_dir().join(format!("physarum-test-{}-{}", std::process::id(), nom));
    let _ = std::fs::remove_dir_all(&dossier);
    dossier
}
