//! Étape 0 : les règles de `AGENTS.md` qu'un test peut lire dans les sources. Valeurs attendues :
//! `references/etape-0-valeurs.md`, identifiants `V-REG-xx`.
//!
//! Pourquoi ce fichier : une fonction du système (`powf`, `exp`, `mul_add`…) donne souvent les mêmes nombres sur deux
//! machines, pas toujours ; aucun test de valeur ne le voit sur une seule machine, seule la lecture des sources le voit.
//! Ces tests lisent `src/` (hors commentaires et chaînes) et `Cargo.toml` ; ils s'appliquent aussi aux étapes suivantes.

use std::path::{Path, PathBuf};

/// Les fonctions `f64` qui passent par la bibliothèque mathématique du système (règle 2 de `AGENTS.md` : la liste interdite,
/// plus leurs cousines).
const INTERDITS: &[&str] = &[
    "sin", "cos", "tan", "asin", "acos", "atan", "atan2", "exp", "exp2", "exp_m1", "ln", "ln_1p", "log", "log2",
    "log10", "powf", "powi", "hypot", "cbrt", "mul_add", "sinh", "cosh", "tanh", "asinh", "acosh", "atanh", "sin_cos",
    "ln_gamma", "erf", "erfc",
];

/// Les noms d'ordre d'itération non fixé, d'horloge ou de fil d'exécution (règle 2).
const NOMS_INTERDITS: &[&str] = &["HashMap", "HashSet", "SystemTime", "Instant", "RandomState"];

/// Les chemins interdits (règle 2 : ni horloge ni fil d'exécution dans la simulation).
const CHEMINS_INTERDITS: &[&str] = &["thread::spawn", "thread::scope", "std::thread"];

/// Le code d'une source Rust sans ses commentaires (`//`, `/* */` imbriqués), ses chaînes (`"…"`, `r#"…"#`) ni ses
/// caractères : ils sont remplacés par des espaces, de façon que les positions et les lignes ne bougent pas.
fn code_nu(source: &str) -> String {
    let c: Vec<char> = source.chars().collect();
    let mut sortie = String::with_capacity(source.len());
    let blanchir = |ch: char| if ch == '\n' { '\n' } else { ' ' };
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        let suivant = c.get(i + 1).copied().unwrap_or('\0');
        if ch == '/' && suivant == '/' {
            while i < c.len() && c[i] != '\n' {
                sortie.push(' ');
                i += 1;
            }
        } else if ch == '/' && suivant == '*' {
            let mut profondeur = 0;
            while i < c.len() {
                if c[i] == '/' && c.get(i + 1) == Some(&'*') {
                    profondeur += 1;
                    sortie.push_str("  ");
                    i += 2;
                } else if c[i] == '*' && c.get(i + 1) == Some(&'/') {
                    profondeur -= 1;
                    sortie.push_str("  ");
                    i += 2;
                    if profondeur == 0 {
                        break;
                    }
                } else {
                    sortie.push(blanchir(c[i]));
                    i += 1;
                }
            }
        } else if ch == 'r'
            && (suivant == '"' || suivant == '#')
            && (i == 0 || !(c[i - 1].is_alphanumeric() || c[i - 1] == '_'))
        {
            // chaîne brute r"…" ou r#"…"#
            let mut j = i + 1;
            let mut dieses = 0;
            while c.get(j) == Some(&'#') {
                dieses += 1;
                j += 1;
            }
            if c.get(j) != Some(&'"') {
                sortie.push(ch);
                i += 1;
                continue;
            }
            j += 1;
            loop {
                if j >= c.len() {
                    break;
                }
                if c[j] == '"' && (0..dieses).all(|k| c.get(j + 1 + k) == Some(&'#')) {
                    j += 1 + dieses;
                    break;
                }
                j += 1;
            }
            for &x in &c[i..j.min(c.len())] {
                sortie.push(blanchir(x));
            }
            i = j;
        } else if ch == '"' {
            sortie.push(' ');
            i += 1;
            while i < c.len() && c[i] != '"' {
                if c[i] == '\\' && i + 1 < c.len() {
                    sortie.push(' ');
                    i += 1;
                }
                sortie.push(blanchir(c[i]));
                i += 1;
            }
            sortie.push(' ');
            i += 1;
        } else if ch == '\'' {
            // un caractère ('x', '\n', '\'') ou une durée de vie ('a)
            let est_caractere = suivant == '\\' || c.get(i + 2) == Some(&'\'');
            if est_caractere {
                sortie.push(' ');
                i += 1;
                while i < c.len() && c[i] != '\'' {
                    if c[i] == '\\' {
                        sortie.push(' ');
                        i += 1;
                    }
                    sortie.push(' ');
                    i += 1;
                }
                sortie.push(' ');
                i += 1;
            } else {
                sortie.push(ch);
                i += 1;
            }
        } else {
            sortie.push(ch);
            i += 1;
        }
    }
    sortie
}

fn est_mot(ch: Option<char>) -> bool {
    ch.is_some_and(|c| c.is_alphanumeric() || c == '_')
}

/// Les occurrences interdites dans un code nu, rendues en `(ligne, texte)` :
/// - un nom de `INTERDITS` précédé de `::` (chemin ou alias : `f64::ln`, `<f64>::exp`, même pour une fonction du dépôt de
///   ce nom, qu'il faut donc nommer autrement), ou précédé de `.` **et suivi de `(` ou de `::`** (un appel de méthode,
///   `x.ln()`, `x.ln::<>()` ; un champ `self.log` ou `p.exp` n'est pas un appel et reste permis) ;
/// - un nom de `INTERDITS` déclaré dans un bloc `extern { … }` (la bibliothèque C du système appelée directement), et
///   l'attribut `link_name` (qui renommerait une telle fonction) ; une fonction exportée `extern "C" fn` reste permise ;
/// - un nom de `NOMS_INTERDITS` en mot entier, un chemin de `CHEMINS_INTERDITS`.
fn fautes(code: &str) -> Vec<(usize, String)> {
    let mut fautes = Vec::new();
    let octets: Vec<char> = code.chars().collect();
    let ligne_de = |position: usize| 1 + octets[..position].iter().filter(|&&c| c == '\n').count();
    let mut positions_mots = Vec::new();
    let mut i = 0;
    while i < octets.len() {
        if octets[i].is_alphabetic() || octets[i] == '_' {
            let debut = i;
            while i < octets.len() && (octets[i].is_alphanumeric() || octets[i] == '_') {
                i += 1;
            }
            positions_mots.push((debut, i));
        } else {
            i += 1;
        }
    }
    let suivant_non_blanc = |fin: usize| octets[fin..].iter().copied().find(|c| !c.is_whitespace());
    // les blocs `extern { … }` (le `"C"` d'un `extern "C" {` est déjà blanchi) : de l'accolade ouvrante à sa fermante
    let mut blocs_extern = Vec::new();
    for &(debut, fin) in &positions_mots {
        if octets[debut..fin].iter().collect::<String>() == "extern" && suivant_non_blanc(fin) == Some('{') {
            let ouvrante = fin + octets[fin..].iter().position(|&c| c == '{').unwrap();
            let mut profondeur = 0;
            let mut fermante = ouvrante;
            while fermante < octets.len() {
                match octets[fermante] {
                    '{' => profondeur += 1,
                    '}' => profondeur -= 1,
                    _ => {}
                }
                if profondeur == 0 {
                    break;
                }
                fermante += 1;
            }
            blocs_extern.push((ouvrante, fermante));
        }
    }
    for (debut, fin) in positions_mots {
        let mot: String = octets[debut..fin].iter().collect();
        if INTERDITS.contains(&mot.as_str()) {
            let mut k = debut;
            while k > 0 && octets[k - 1].is_whitespace() {
                k -= 1;
            }
            let apres_point = k > 0 && octets[k - 1] == '.';
            let apres_chemin = k > 1 && octets[k - 1] == ':' && octets[k - 2] == ':';
            let appel = matches!(suivant_non_blanc(fin), Some('(') | Some(':'));
            let dans_extern = blocs_extern.iter().any(|&(a, b)| a < debut && debut < b);
            if apres_chemin || (apres_point && appel) || dans_extern {
                fautes.push((ligne_de(debut), format!("fonction f64 interdite `{mot}`")));
            }
        }
        if mot == "link_name" {
            fautes.push((ligne_de(debut), "attribut interdit `link_name`".to_string()));
        }
        if NOMS_INTERDITS.contains(&mot.as_str()) {
            fautes.push((ligne_de(debut), format!("nom interdit `{mot}`")));
        }
    }
    for chemin in CHEMINS_INTERDITS {
        for (position, _) in code.match_indices(chemin) {
            let avant = code[..position].chars().next_back();
            if !est_mot(avant) {
                fautes.push((1 + code[..position].matches('\n').count(), format!("chemin interdit `{chemin}`")));
            }
        }
    }
    fautes
}

fn fichiers_rust(dossier: &Path, dedans: &mut Vec<PathBuf>) {
    for entree in std::fs::read_dir(dossier).expect("lecture de src/") {
        let chemin = entree.unwrap().path();
        if chemin.is_dir() {
            fichiers_rust(&chemin, dedans);
        } else if chemin.extension().is_some_and(|e| e == "rs") {
            dedans.push(chemin);
        }
    }
}

/// juste : le détecteur lui-même voit ce qu'il doit voir : `x.ln()`, `f64::exp(1.0)`, `y . powi(2)`, `let f = f64::ln;`,
/// `x.ln ::<>()`, `<f64>::exp(y)`, `HashMap`, `std::thread::spawn`, une fonction `log` déclarée dans un bloc `extern "C"`
/// et un `link_name` ; et ne voit pas les mêmes noms dans un commentaire, une doc, une chaîne, une chaîne brute ou d'octets,
/// un caractère, ni des identifiants voisins (`exp_neg`, `EXP_NEG`, `ln_m`, `logarithme`, `exposant`, `explorer`, `lnk`), ni
/// la variable `exp` d'un calcul, ni un champ (`self.exp`, `self.log`), ni une fonction exportée `extern "C" fn`, ni les
/// fonctions permises (`sqrt`, `abs`, `floor`) (V-REG-01, textes écrits ici).
///
/// Ne couvre pas : les appels cachés par une macro ou un `include!`, ni une fonction du système atteinte sans que son nom
/// soit écrit dans `src/`.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn detecteur_de_fautes() {
    for (source, attendu) in [
        ("fn f(x: f64) -> f64 { x.ln() }", 1),
        ("fn f() -> f64 { f64::exp(1.0) }", 1),
        ("fn f(y: f64) -> f64 { y\n  .\n  powi(2) }", 1),
        ("let f = f64::ln;", 1),
        ("let m: HashMap<u8, u8> = HashMap::new();", 2),
        ("std::thread::spawn(|| ());", 2),
        ("let a = x.sin() + y.cos();", 2),
        ("let b = x.ln ::<>() + <f64>::exp(y) + f64 :: log2(z) + x.tan ();", 4),
        ("unsafe extern \"C\" {\n    fn log(x: f64) -> f64;\n}", 1),
        ("extern \"C\" { #[link_name = \"exp\"] fn e(x: f64) -> f64; }", 1),
    ] {
        assert_eq!(fautes(&code_nu(source)).len(), attendu, "{source:?}");
    }
    for source in [
        "// x.ln() et HashMap dans un commentaire\nfn f() {}",
        "/// doc : `x.exp()` `f64::ln`\nfn f() {}",
        "/* x.ln() /* imbriqué x.exp() */ encore x.sin() */ fn f() {}",
        "let s = \"x.ln() HashMap f64::exp\"; let t = \"guillemet \\\" x.ln()\";",
        "let s = r#\"x.ln() \"HashMap\" \"#; let u = r\"f64::exp\";",
        "let c = '\"'; let d = '\\''; let e: &'static str = \"x\";",
        "let a = exp_neg(3) + EXP_NEG[2] + ln_m + logarithme(2.0) + exposant as f64;",
        "let exp = 3; let ln = 2; let log = exp + ln; let sin = 1;",
        "fn f<'a>(x: &'a str) -> &'a str { x }",
        "self.exp = 3; let n = self.log.len() + p.ln; let explorer = lnk + tangente + sinus;",
        "#[unsafe(no_mangle)]\npub extern \"C\" fn exporter(x: u32) -> u32 { x }",
        "let r = x.sqrt().abs().floor() + y.to_bits().ilog2() as f64;",
        "let s = br\"x.ln()\"; let t = b\"x.exp()\";",
    ] {
        assert_eq!(fautes(&code_nu(source)), [], "{source:?}");
    }
}

/// juste : aucun fichier de `src/` n'appelle une fonction `f64` du système interdite par la règle 2 de `AGENTS.md` (`sin`,
/// `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `exp`, `exp2`, `ln`, `log`, `log2`, `log10`, `powf`, `powi`, `hypot`, `cbrt`,
/// `mul_add`, `sinh`, `cosh`, `tanh` et leurs cousines), n'emploie `HashMap`, `HashSet`, `RandomState`, une horloge
/// (`SystemTime`, `Instant`) ni un fil d'exécution (`std::thread`) ; commentaires et chaînes exclus (V-REG-02). La règle dit
/// « aucune `HashMap` parcourue » : on interdit ici le type entier, plus simple à lire dans les sources.
///
/// Ne couvre pas : les fonctions du système appelées par un autre nom, les `unsafe`, et l'ordre de mise à jour des
/// simulations (les étapes suivantes le testent par l'empreinte).
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn sources_sans_fonction_du_systeme() {
    let mut fichiers = Vec::new();
    fichiers_rust(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut fichiers);
    assert!(fichiers.len() >= 8, "src/ devrait contenir au moins les modules de l'étape 0");
    let mut toutes = Vec::new();
    for fichier in fichiers {
        let source = std::fs::read_to_string(&fichier).unwrap();
        for (ligne, faute) in fautes(&code_nu(&source)) {
            toutes.push(format!("{}:{ligne} : {faute}", fichier.display()));
        }
    }
    assert!(toutes.is_empty(), "règle 2 de AGENTS.md violée :\n{}", toutes.join("\n"));
}

/// juste : `Cargo.toml` n'a aucune dépendance (ni `[dependencies]`, ni `[dev-dependencies]`, ni `[build-dependencies]`, ni
/// celles d'une cible), `Cargo.lock` ne contient aucun paquet venu d'un registre ou d'un dépôt (aucune ligne `source =`), et
/// le dossier `.cargo/` n'existe pas (règle 1 et règle 8 de `AGENTS.md`) (V-REG-03).
///
/// Ne couvre pas : les dépendances ajoutées par un autre fichier que `Cargo.toml` à la racine.
#[test]
#[cfg_attr(not(feature = "etape-0"), ignore)]
fn sans_dependance_ni_cargo_local() {
    let racine = Path::new(env!("CARGO_MANIFEST_DIR"));
    let toml = std::fs::read_to_string(racine.join("Cargo.toml")).unwrap();
    let mut section = String::new();
    for ligne in toml.lines() {
        let ligne = ligne.trim();
        if ligne.starts_with('[') {
            section = ligne.to_string();
            assert!(
                !section.contains("dependencies") || section == "[dependencies]",
                "section de dépendances : {section}"
            );
            continue;
        }
        if section.contains("dependencies") {
            assert!(ligne.is_empty() || ligne.starts_with('#'), "dépendance déclarée : {ligne}");
        }
    }
    assert!(!toml.contains("[dev-dependencies]") && !toml.contains("[build-dependencies]"));
    let verrou = std::fs::read_to_string(racine.join("Cargo.lock")).unwrap();
    assert!(!verrou.lines().any(|l| l.trim_start().starts_with("source =")), "Cargo.lock contient un paquet externe");
    assert!(!racine.join(".cargo").exists(), "un dossier .cargo/ existe");
}
