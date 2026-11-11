//! Aides communes aux tests de l'étape 2, partie 2M : le banc d'essai de la mémoire (une grille, une source et un
//! puits, un stimulus d'un côté, un témoin de même graine), les mesures (l'écart d'empreinte, la coupe d'une sonde) et
//! le pas couplé écrit **naïvement** d'après la seule doc de `souvenir` (section « La mémoire dans la forme »). Rien
//! n'est emprunté au code jugé de 2M ; le transport de l'agent naïf vient de `transport_2t` (2T) et le solveur de
//! `solveur` (étape 0), déjà éprouvés.
//!
//! Ce fichier n'est pas un test : il ne contient aucune fonction `#[test]`. Il est gelé avec les tests.
#![allow(dead_code)]

use physarum_lab::alea::Alea;
use physarum_lab::graphe::{Graphe, Grille, Voisinage};
use physarum_lab::reseaux::{Loi, tirer_paire};
use physarum_lab::solveur::{flux, pressions};
use physarum_lab::souvenir::{Agent, Couplage, Memoire, Transport};
use physarum_lab::tubes::PLANCHER;

use super::transport_2t;

/// Le seuil de l'écart d'empreinte, **fixé avant de mesurer** : `0.01` (voir la doc, « Ce qu'on mesure »).
pub const SEUIL_EMPREINTE: f64 = 0.01;

/// Un réglage du modèle : la loi, le débit `I0`, le couplage, le transport de l'agent, le pas `h`.
#[derive(Clone, Copy, Debug)]
pub struct Reglage {
    pub loi: Loi,
    pub debit: f64,
    pub force: f64,
    pub saturation: f64,
    pub diffusion: f64,
    pub perte: f64,
    pub h: f64,
}

/// Le réglage standard : loi sigmoïde, `I0 = 3`, `α = 8`, `c_0 = 1`, `κ = 0.1`, `k = 0.2`, `h = 0.1`.
pub const STANDARD: Reglage =
    Reglage { loi: Loi::Sigmoide, debit: 3.0, force: 8.0, saturation: 1.0, diffusion: 0.1, perte: 0.2, h: 0.1 };

/// Le réglage de la loi linéaire : `I0 = 1`, `α = 4`, `c_0 = 1`, `κ = 0.1`, `k = 0.2`, `h = 0.1`.
pub const LINEAIRE: Reglage =
    Reglage { loi: Loi::Lineaire, debit: 1.0, force: 4.0, saturation: 1.0, diffusion: 0.1, perte: 0.2, h: 0.1 };

impl Reglage {
    pub fn couplage(&self) -> Couplage {
        Couplage { force: self.force, saturation: self.saturation }
    }
    pub fn transport(&self) -> Transport {
        Transport { diffusion: self.diffusion, perte: self.perte }
    }
}

/// Un stimulus : un apport d'agent `debit` au nœud `(x, y)` (étage 0) pendant les pas `[debut ; fin[`.
#[derive(Clone, Copy, Debug)]
pub struct Stim {
    pub x: u32,
    pub y: u32,
    pub debit: f64,
    pub debut: usize,
    pub fin: usize,
}

/// Le stimulus standard : au nœud `(7, 2)`, débit 2, pas `[0 ; 100[`.
pub const STIM_HAUT: Stim = Stim { x: 7, y: 2, debit: 2.0, debut: 0, fin: 100 };

/// Le même stimulus de l'autre côté de l'axe : le nœud `(7, 8)`.
pub const STIM_BAS: Stim = Stim { x: 7, y: 8, ..STIM_HAUT };

/// Le banc : une grille de 15 colonnes et 11 lignes en voisinage 8, un étage ; la source en `(1, 5)` et le puits en
/// `(13, 5)` sur l'axe `y = 5`, qui partage la grille en une moitié haute (`y < 5`) et une moitié basse (`y > 5`).
pub struct Banc {
    pub grille: Grille,
    pub graphe: Graphe,
    pub source: usize,
    pub puits: usize,
}

pub const AXE: u32 = 5;

pub fn banc() -> Banc {
    let grille = Grille {
        largeur: 15,
        hauteur: 11,
        etages: 1,
        voisinage: Voisinage::Huit,
        longueur_verticale: 1.0,
        cout_vertical: 1.0,
    };
    let graphe = grille.graphe().expect("grille valide");
    let source = grille.indice(1, AXE, 0);
    let puits = grille.indice(13, AXE, 0);
    Banc { grille, graphe, source, puits }
}

impl Banc {
    pub fn noeud(&self, x: u32, y: u32) -> usize {
        self.grille.indice(x, y, 0)
    }
    fn apports(&self, stimuli: &[Stim], t: usize) -> Vec<(usize, f64)> {
        stimuli.iter().filter(|s| s.debut <= t && t < s.fin).map(|s| (self.noeud(s.x, s.y), s.debit)).collect()
    }
}

/// Ce qu'une partie du banc a donné à un instant : les conductivités et la masse d'agent.
#[derive(Clone, Debug)]
pub struct Instant {
    pub d: Vec<f64>,
    pub masse: f64,
}

/// Joue `Memoire::avancer` (famille 0 : `source` et `puits` fixes ; `famille == 1` : la paire de deux nourritures tirée
/// à chaque pas par `tirer_paire`, le même `alea` que celui des conductivités de départ, après elles) avec les stimuli
/// donnés, et rend l'état aux `instants` (croissants ; l'instant `t` est l'état après `t` pas). `effacer_a` : si
/// `Some(t)`, l'agent est remis à zéro juste avant le pas `t` (la forme seule continue).
pub fn jouer(
    b: &Banc,
    r: &Reglage,
    famille: u8,
    graine: u64,
    stimuli: &[Stim],
    instants: &[usize],
    effacer_a: Option<usize>,
) -> Vec<Instant> {
    let mut alea = Alea::new(graine);
    let mut m = Memoire::au_hasard(&b.graphe, &mut alea);
    let (cp, tr) = (r.couplage(), r.transport());
    let mut sortie = Vec::with_capacity(instants.len());
    let fin = *instants.last().expect("au moins un instant");
    for t in 0..=fin {
        if instants.contains(&t) {
            sortie.push(Instant { d: m.conductivites().to_vec(), masse: m.agent().masse() });
        }
        if t == fin {
            break;
        }
        if effacer_a == Some(t) {
            m = Memoire::new(&b.graphe, m.conductivites().to_vec(), Agent::zero(&b.graphe)).unwrap();
        }
        let (s, p) = if famille == 1 {
            let (i, j) = tirer_paire(&mut alea, 2);
            let f = [b.source, b.puits];
            (f[i], f[j])
        } else {
            (b.source, b.puits)
        };
        m.avancer(&b.graphe, s, p, r.debit, r.loi, &cp, &tr, &b.apports(stimuli, t), r.h).unwrap();
    }
    sortie
}

/// La moyenne des conductivités des arêtes dont les deux bouts sont dans la moitié haute (`haut = true`, `y < 5`) ou
/// basse (`y > 5`) : somme par indice d'arête croissant à partir de `0.0`, divisée par leur nombre.
pub fn moyenne_demi(b: &Banc, d: &[f64], haut: bool) -> f64 {
    let noeuds = b.graphe.noeuds();
    let dans = |i: usize| if haut { noeuds[i].y < AXE as i32 } else { noeuds[i].y > AXE as i32 };
    let (mut somme, mut n) = (0.0, 0_u32);
    for (e, a) in b.graphe.aretes().iter().enumerate() {
        if dans(a.a) && dans(a.b) {
            somme += d[e];
            n += 1;
        }
    }
    somme / f64::from(n)
}

/// L'écart d'empreinte `(E, F_A, F_B)` d'un état avec stimulus contre le témoin de même graine : `F_A` est la
/// différence des moyennes de la moitié haute (le côté du stimulus standard), `F_B` celle de la moitié basse, `E = F_A
/// - F_B`.
pub fn ecart(b: &Banc, avec: &[f64], temoin: &[f64]) -> (f64, f64, f64) {
    let fa = moyenne_demi(b, avec, true) - moyenne_demi(b, temoin, true);
    let fb = moyenne_demi(b, avec, false) - moyenne_demi(b, temoin, false);
    (fa - fb, fa, fb)
}

/// La sonde : un flux de 1 injecté en `(qx, AXE)`, le puits en `(rx, AXE)`, les conductivités `d` (les pressions de
/// `solveur::pressions` avec `g = d / cout`, les flux de `solveur::flux`). Rend `(haut, axe, bas)` : les flux qui
/// traversent la coupe entre les colonnes `10` et `11` (arêtes dont un bout est en `x = 10` et l'autre en `x = 11`,
/// comptés du bout en `x = 10` vers l'autre), rangés selon le `y` du bout en `x = 10` (`< 5`, `== 5`, `> 5`).
pub fn coupe(b: &Banc, d: &[f64], qx: u32, rx: u32) -> (f64, f64, f64) {
    let g: Vec<f64> = d.iter().enumerate().map(|(e, &x)| x / b.graphe.aretes()[e].cout).collect();
    let mut inj = vec![0.0; b.graphe.nb_noeuds()];
    inj[b.noeud(qx, AXE)] = 1.0;
    let p = pressions(&b.graphe, &g, &inj, b.noeud(rx, AXE)).unwrap();
    let q = flux(&b.graphe, &g, &p);
    let noeuds = b.graphe.noeuds();
    let (mut haut, mut axe, mut bas) = (0.0, 0.0, 0.0);
    for (e, a) in b.graphe.aretes().iter().enumerate() {
        let (na, nb) = (noeuds[a.a], noeuds[a.b]);
        let (bout, sens) = if na.x == 10 && nb.x == 11 {
            (na, 1.0)
        } else if nb.x == 10 && na.x == 11 {
            (nb, -1.0)
        } else {
            continue;
        };
        let f = sens * q[e];
        if bout.y < AXE as i32 {
            haut += f;
        } else if bout.y == AXE as i32 {
            axe += f;
        } else {
            bas += f;
        }
    }
    (haut, axe, bas)
}

/// La même chose pour un stimulus et son témoin, sur plusieurs graines : `(moyenne, minimum, maximum)` d'un vecteur.
pub fn stats(v: &[f64]) -> (f64, f64, f64) {
    let moy = v.iter().sum::<f64>() / v.len() as f64;
    let mn = v.iter().cloned().fold(f64::INFINITY, f64::min);
    let mx = v.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    (moy, mn, mx)
}

// ---------------------------------------------------------------------------------------------------------------------
// Le pas couplé naïf, d'après la doc seule.
// ---------------------------------------------------------------------------------------------------------------------

/// La loi `f`, écrite ici : `sigmoide` vaut `q * q / (1 + q * q)`, sinon `|q|`.
pub fn naif_f(sigmoide: bool, q: f64) -> f64 {
    if sigmoide {
        let q2 = q * q;
        q2 / (1.0 + q2)
    } else {
        q.abs()
    }
}

/// Ce que rend un pas couplé naïf.
pub struct PasNaif {
    pub d: Vec<f64>,
    pub c: Vec<f64>,
    pub pressions: Vec<f64>,
    pub flux: Vec<f64>,
    pub bilan: [f64; 3],
    pub sous_pas: usize,
}

/// Un pas couplé, à la lettre de la doc : les flux avec les conductivités d'avant (`solveur`), le transport de 2T avec
/// ces flux (`transport_2t::naif_avancer`), et les conductivités mises à jour avec les concentrations d'AVANT.
#[allow(clippy::too_many_arguments)]
pub fn naif_pas_couple(
    graphe: &Graphe,
    d: &[f64],
    c: &[f64],
    source: usize,
    puits: usize,
    debit: f64,
    sigmoide: bool,
    force: f64,
    saturation: f64,
    kappa: f64,
    k: f64,
    apports: &[(usize, f64)],
    h: f64,
) -> PasNaif {
    let g: Vec<f64> = (0..d.len()).map(|e| d[e] / graphe.aretes()[e].cout).collect();
    let mut inj = vec![0.0; graphe.nb_noeuds()];
    inj[source] = debit;
    let p = pressions(graphe, &g, &inj, puits).unwrap();
    let q = flux(graphe, &g, &p);
    let (c_apres, bilan, sous_pas) = transport_2t::naif_avancer(graphe, &q, c, kappa, k, apports, h);
    let mut d_apres = Vec::with_capacity(d.len());
    for (e, a) in graphe.aretes().iter().enumerate() {
        let ce = (c[a.a] + c[a.b]) * 0.5;
        let sigma = ce / (saturation + ce);
        let cible = (1.0 + force * sigma) * naif_f(sigmoide, q[e]);
        let v = d[e] + h * (cible - d[e]);
        d_apres.push(if v >= PLANCHER { v } else { PLANCHER });
    }
    PasNaif { d: d_apres, c: c_apres, pressions: p, flux: q, bilan, sous_pas }
}
