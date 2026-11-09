//! Étape 0 : `image`, CRC-32, Adler-32 et l'écrivain PNG. Valeurs attendues : `references/etape-0-valeurs.md`,
//! identifiants `V-IMG-xx`.
//!
//! Sources : la valeur de contrôle `0xCBF43926` du CRC-32/ISO-HDLC (catalogue de CRC de R. Williams,
//! reveng.sourceforge.io) ; l'Adler-32 de « Wikipedia » (article Adler-32 de Wikipédia anglais, `A = 920`, `B = 4582`) ;
//! la structure du PNG (spécification PNG, W3C, 3e édition : §5.2 signature, §5.3 chunks, §11.2.1 IHDR, §9.2 filtre 0,
//! §5.5 CRC) et du flux zlib (RFC 1950 §2.2, RFC 1951 §3.2.3 et §3.2.4 : blocs stockés, `LEN`, `NLEN`) ; les autres valeurs
//! viennent de Python `zlib` (`crc32`, `adler32`, `decompress`), un outil indépendant. Le PNG est relu par un décodeur écrit
//! dans `tests/common/mod.rs` (son propre CRC-32 et son propre Adler-32) et non par le code testé.

mod common;

use common::{adler32_naif, crc32_naif, relire_png};
use physarum_lab::alea::Alea;
use physarum_lab::image::{adler32, crc32, ecrire_png};

/// 256 octets `(i · 7 + 3) mod 256` répétés 400 fois (102 400 octets) : le jeu de V-IMG-03.
fn motif() -> Vec<u8> {
    (0..256 * 400).map(|i| (((i % 256) * 7 + 3) % 256) as u8).collect()
}

/// juste : le CRC-32 de `123456789` est `0xCBF43926` (valeur de contrôle du catalogue de CRC, V-IMG-01), celui de la suite
/// vide est 0, celui de `IEND` est `0xAE426082` (c'est le CRC écrit à la fin de tout PNG), celui de `0..=255` est
/// `0x29058C73` et celui de 102 400 octets du motif est `0x2CD27A54` (Python `zlib.crc32`, V-IMG-03) ; et sur 60 messages
/// pseudo-aléatoires de 0 à 400 octets il est égal au CRC bit par bit écrit dans le test.
///
/// Ne couvre pas : les messages de plus de 102 400 octets.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn crc32_valeurs() {
    assert_eq!(crc32(b"123456789"), 0xCBF43926);
    assert_eq!(crc32(b""), 0);
    assert_eq!(crc32(b"IEND"), 0xAE426082);
    let tous: Vec<u8> = (0..=255).collect();
    assert_eq!(crc32(&tous), 0x29058C73);
    assert_eq!(crc32(&motif()), 0x2CD27A54);
    let mut alea = Alea::new(5);
    for _ in 0..60 {
        let longueur = alea.borne(401) as usize;
        let donnees: Vec<u8> = (0..longueur).map(|_| alea.suivant() as u8).collect();
        assert_eq!(crc32(&donnees), crc32_naif(&donnees), "{longueur} octets");
    }
}

/// juste : l'Adler-32 de `Wikipedia` est `0x11E60398` (V-IMG-02), celui de la suite vide est 1, ceux de 5 552 et 5 553 « a »
/// sont `0x47333829` et `0x7FBD388A` (la frontière où l'on doit réduire modulo 65521 pour ne pas déborder 32 bits), celui de
/// 100 000 octets `0xFF` est `0x149A302C`, celui de `0..=255` est `0xADF67F81`, celui du motif de 102 400 octets est
/// `0xC63D43AA` (Python `zlib.adler32`, V-IMG-03) ; et sur 40 messages pseudo-aléatoires de longueurs variées, dont autour
/// de 5 552 et de 65 521, il est égal à l'Adler-32 à modulo à chaque octet écrit dans le test.
///
/// Ne couvre pas : les messages de plus de 102 400 octets.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn adler32_valeurs() {
    assert_eq!(adler32(b"Wikipedia"), 0x11E60398);
    assert_eq!(adler32(b""), 1);
    assert_eq!(adler32(&vec![b'a'; 5552]), 0x47333829);
    assert_eq!(adler32(&vec![b'a'; 5553]), 0x7FBD388A);
    assert_eq!(adler32(&vec![0xFF; 100_000]), 0x149A302C);
    let tous: Vec<u8> = (0..=255).collect();
    assert_eq!(adler32(&tous), 0xADF67F81);
    assert_eq!(adler32(&motif()), 0xC63D43AA);
    let mut alea = Alea::new(6);
    let mut longueurs = vec![5551, 5552, 5553, 11104, 11105, 65520, 65521, 65522, 131_072];
    longueurs.extend((0..31).map(|_| alea.borne(70_000) as usize));
    for longueur in longueurs {
        let donnees: Vec<u8> = (0..longueur).map(|_| alea.suivant() as u8).collect();
        assert_eq!(adler32(&donnees), adler32_naif(&donnees), "{longueur} octets");
    }
}

/// juste : les 75 octets du PNG de 2 × 1 pixels (rouge `FF 00 00`, bleu `00 00 FF`), écrits un par un à la main d'après la
/// spécification (V-IMG-04), avec leurs deux CRC-32 et l'Adler-32 calculés par `zlib` de Python ; Python `zlib.decompress` relit
/// le flux IDAT et retrouve les données `00 FF 00 00 00 00 FF`. Voir la table ci-dessous.
///
/// ```text
/// octets  0..8   89 50 4E 47 0D 0A 1A 0A                     signature
/// octets  8..12  00 00 00 0D                                 IHDR : 13 octets de données
/// octets 12..16  49 48 44 52                                 « IHDR »
/// octets 16..24  00 00 00 02  00 00 00 01                    largeur 2, hauteur 1
/// octets 24..29  08 02 00 00 00                              8 bits, RVB, compression 0, filtre 0, sans entrelacement
/// octets 29..33  7B 40 E8 DD                                 CRC-32 de « IHDR » + les 13 octets
/// octets 33..37  00 00 00 12                                 IDAT : 18 octets de données = 2 + 5 + 7 + 4
/// octets 37..41  49 44 41 54                                 « IDAT »
/// octets 41..43  78 01                                       en-tête zlib
/// octets 43..48  01 07 00 F8 FF                              bloc stocké final : BFINAL = 1, LEN = 7, NLEN = 0xFFF8
/// octets 48..55  00  FF 00 00  00 00 FF                      filtre 0, pixel rouge, pixel bleu (7 octets)
/// octets 55..59  07 00 01 FF                                 Adler-32 = 0x070001FF
/// octets 59..63  55 36 BA C7                                 CRC-32 de « IDAT » + les 18 octets
/// octets 63..75  00 00 00 00  49 45 4E 44  AE 42 60 82       IEND : 0 octet, « IEND », CRC-32
/// ```
///
/// Ne couvre pas : les images de plusieurs blocs (voir `png_blocs_stockes_aux_limites`).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn png_deux_pixels_octet_par_octet() {
    #[rustfmt::skip]
    let attendu: [u8; 75] = [
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x02, 0x00, 0x00, 0x00,
        0x7B, 0x40, 0xE8, 0xDD,
        0x00, 0x00, 0x00, 0x12, 0x49, 0x44, 0x41, 0x54,
        0x78, 0x01,
        0x01, 0x07, 0x00, 0xF8, 0xFF,
        0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0xFF,
        0x07, 0x00, 0x01, 0xFF,
        0x55, 0x36, 0xBA, 0xC7,
        0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let png = ecrire_png(2, 1, &[0xFF, 0, 0, 0, 0, 0xFF]);
    assert_eq!(png.len(), 75);
    assert_eq!(png, attendu);
}

/// juste : de petites images relues par le décodeur du test (signature, trois chunks seulement, CRC recalculés, IHDR,
/// blocs stockés, Adler-32, filtre 0, pixels) : 1×1, 2×1, 1×2, 3×5, 17×13, 64×48 ; les pixels relus sont ceux qu'on a donnés
/// (des octets pseudo-aléatoires, tous distincts de position en position, donc une inversion de lignes, de colonnes ou de
/// canaux est vue), il n'y a qu'un bloc, et la longueur des données du IDAT est `2 + 5 + L + 4` (V-IMG-05).
///
/// Ne couvre pas : les images assez grandes pour plusieurs blocs.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn png_relu_petites_images() {
    let mut alea = Alea::new(8);
    for (largeur, hauteur) in [(1_u32, 1_u32), (2, 1), (1, 2), (3, 5), (17, 13), (64, 48)] {
        let rvb: Vec<u8> = (0..3 * largeur * hauteur).map(|_| alea.suivant() as u8).collect();
        let png = ecrire_png(largeur, hauteur, &rvb);
        let lu = relire_png(&png).unwrap_or_else(|e| panic!("{largeur}×{hauteur} : {e}"));
        assert_eq!((lu.largeur, lu.hauteur), (largeur, hauteur));
        assert_eq!(lu.rvb, rvb, "{largeur}×{hauteur} : pixels");
        let l = (hauteur * (1 + 3 * largeur)) as usize;
        assert_eq!(lu.blocs, [l], "{largeur}×{hauteur} : un seul bloc");
        assert_eq!(lu.longueur_idat, 2 + 5 + l + 4);
        assert_eq!(png.len(), 8 + 25 + (12 + lu.longueur_idat) + 12);
    }
}

/// juste : les blocs stockés aux limites de 65 535 octets (RFC 1951 §3.2.4 : `LEN` sur 16 bits). Les données ont
/// `L = hauteur · (1 + 3 · largeur)` octets ; pour `L = 65 534` (2×9 362), `65 535` (28×771), `65 536` (1×16 384), `131 070`
/// (28×1 542) et `131 071` (43 690×1) les blocs sont exactement `[65534]`, `[65535]`, `[65535, 1]`, `[65535, 65535]` et
/// `[65535, 65535, 1]` (jamais de bloc vide, le dernier est plein quand `L` est un multiple), seul le dernier est marqué
/// final, et l'image relue est l'image donnée ; la longueur du IDAT est `2 + 5 · blocs + L + 4` (V-IMG-06). Une coupe à
/// 65 536, un bloc final vide ou un Adler-32 calculé par bloc échouent ici.
///
/// Ne couvre pas : les images de plus de trois blocs.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn png_blocs_stockes_aux_limites() {
    let cas: [(u32, u32, &[usize]); 5] = [
        (2, 9362, &[65534]),
        (28, 771, &[65535]),
        (1, 16384, &[65535, 1]),
        (28, 1542, &[65535, 65535]),
        (43690, 1, &[65535, 65535, 1]),
    ];
    let mut alea = Alea::new(9);
    for (largeur, hauteur, blocs) in cas {
        let rvb: Vec<u8> = (0..3 * largeur as usize * hauteur as usize).map(|_| alea.suivant() as u8).collect();
        let png = ecrire_png(largeur, hauteur, &rvb);
        let lu = relire_png(&png).unwrap_or_else(|e| panic!("{largeur}×{hauteur} : {e}"));
        assert_eq!(lu.blocs, blocs, "{largeur}×{hauteur} : blocs");
        assert_eq!(lu.rvb, rvb, "{largeur}×{hauteur} : pixels");
        let l: usize = blocs.iter().sum();
        assert_eq!(l, (hauteur * (1 + 3 * largeur)) as usize);
        assert_eq!(lu.longueur_idat, 2 + 5 * blocs.len() + l + 4);
    }
}

/// juste : le PNG est déterministe et ne dépend que de ses entrées : deux appels donnent les mêmes octets, deux images
/// différentes d'un seul octet donnent deux fichiers différents, et l'IEND final est `00 00 00 00 49 45 4E 44 AE 42 60 82`
/// (V-IMG-04).
///
/// Ne couvre pas : la comparaison avec un autre écrivain PNG.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn png_deterministe_et_iend() {
    let rvb: Vec<u8> = (0..3 * 5 * 4).map(|i| (i * 11) as u8).collect();
    let a = ecrire_png(5, 4, &rvb);
    assert_eq!(a, ecrire_png(5, 4, &rvb));
    let mut autre = rvb.clone();
    autre[59] ^= 1;
    assert_ne!(a, ecrire_png(5, 4, &autre));
    assert_eq!(a[a.len() - 12..], [0, 0, 0, 0, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]);
}

/// juste : `ecrire_png` panique pour une largeur ou une hauteur nulle et pour un nombre d'octets qui n'est pas
/// `3 · largeur · hauteur`, trop court comme trop long (V-IMG-07, la doc dit « panique »).
///
/// Ne couvre pas : le message des paniques.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn png_panique_sur_entrees_invalides() {
    use std::panic::catch_unwind;
    assert!(catch_unwind(|| ecrire_png(0, 1, &[])).is_err(), "largeur nulle");
    assert!(catch_unwind(|| ecrire_png(1, 0, &[])).is_err(), "hauteur nulle");
    assert!(catch_unwind(|| ecrire_png(2, 2, &[0; 11])).is_err(), "trop court");
    assert!(catch_unwind(|| ecrire_png(2, 2, &[0; 13])).is_err(), "trop long");
    assert!(catch_unwind(|| ecrire_png(2, 2, &[0; 12])).is_ok(), "la bonne longueur ne panique pas");
}
