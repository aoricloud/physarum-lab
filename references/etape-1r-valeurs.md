# Étape 1, partie 1R (les réseaux) : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de la partie 1R attend, d'où elle vient, avec quelle tolérance et quel
critère de réussite. Chaque identifiant (`V-LOI-01`, `V-TIR-01`, `V-MST-01`, `V-MES-01`, `V-ESP-01`, `V-APR-01`,
`V-LRES-01`…) est cité dans la doc du test qui l'emploie (`tests/etape_1r_*.rs`). Elle est séparée des résultats figés
de notre propre code (`references/*.empreinte`, étiquetés « régression » dans les tests) : **aucune valeur de cette
fiche ne vient de notre code**. Une valeur y vient d'une publication lue, d'un calcul à la main ou par un outil
indépendant (nommé), d'une construction naïve écrite dans le test, ou d'une convention que la documentation d'un module
fixe (dite telle).

Écrite avant les calculs : les critères d'abord, les mesures ensuite. La suite complète de la partie (`cargo test
--features etape-1,etape-1l,etape-1r`) dure une trentaine de secondes avec les 181 tests d'avant (225 tests en tout).

## Les outils indépendants

Python 3.13 (flottants IEEE 754, `struct`, `hashlib`) ; le code Rust n'est jamais l'outil. Gardés avec les notes de la
session qui a écrit ces tests, dans `outils-1R/` :

- `ref_alea.py`, `ref_graphe.py`, `ref_solveur.py`, `ref_tubes.py`, `ref_labyrinthe.py` : repris du cœur et de 1L
  (PCG32, la grille, le solveur, le pas du modèle, l'aperçu coloré, la construction du plan) ;
- `ref_reseaux.py` : écrit d'après la seule doc de `reseaux` et de `lancer` : la loi, la paire tirée, un pas à plusieurs
  sources, l'arbre couvrant minimal (Kruskal), les trois mesures, l'état `PHYSAR02`, l'empreinte, les mesures en CSV, le
  treillis coloré avec les nourritures, et le pas où le solveur s'arrête (`pressions_verifiees`) ;
- `valeurs_1r_a.py` à `valeurs_1r_e.py` : impriment les valeurs écrites dans les tests (loi, tirages, comptes et `chi²`,
  arbres, mesures, empreintes et SHA-256) ; `mesures/` : les programmes de mesure des tolérances (`mes_1r_esprit.rs`,
  `mes_1r_tol.rs`, `mes_1r_seuil.rs`, `mes_1r.rs`, `mes_1r_alea.rs` : ils s'appuient sur `reseaux`) ; `mutants_1r.py` :
  le banc de fautes.

Dans les tests, plusieurs constructions naïves jouent le même rôle : le tirage naïf avec `Alea::borne`
(`etape_1r_tirage.rs`), le pas naïf avec `solveur::pressions` et `solveur::flux` (`etape_1r_loi.rs`), l'arbre naïf
(Bellman-Ford sur les longueurs, toutes les paires triées, union-find, parcours de l'arbre) et l'énumération de tous les
arbres couvrants (`etape_1r_arbre.rs`), les mesures naïves (relaxation de Bellman-Ford sur les tubes vifs, relaxation
d'étiquettes de composantes) (`etape_1r_mesures.rs`), les dessins en caractères (`etape_1r_apercu.rs`).

## Les sources lues (et ce qui ne l'est pas)

Les lectures sont celles de `references-etapes-0-1.md` (notes du pilote, sections B1 et B3) ; **aucune valeur n'est
ajoutée de mémoire**, et ni l'article ni son supplément n'ont été relus par l'écrivain de cette partie.

- **Tero, Takagi, Saigusa, Ito, Bebber, Fricker, Yumiki, Kobayashi, Nakagaki (2010)**, *Rules for Biologically Inspired
  Adaptive Network Design*, Science 327, 439-442 : lu (par le pilote) : l'article entier (p. 439-441) et le supplément
  (figures S1 et S2 seulement). Lu : à chaque pas de temps, une source de nourriture tirée au hasard fait circuler le
  flux `I0` (`somme_j Q_1j = I0`) et une seconde, tirée au hasard, est le puits ; l'adaptation `dD/dt = f(|Q|) - D`,
  `f(|Q|) = |Q|^γ / (1 + |Q|^γ)`, `γ > 0` ; une simulation typique `I0 = 2`, `γ = 1,8` ; la Fig. 3A fait varier `I0` de
  0,20 à 7,19 à `γ = 1,80` ; `I0 = 0,20` et `γ = 1,15` donnent un réseau proche du réseau ferroviaire ; « augmenter `I0`
  : plus de routes alternatives, meilleure FT, coût plus élevé » ; « augmenter `γ` ou baisser `I0` : vers l'arbre
  couvrant minimal » ; TL, MD, FT **en mots** (définitions des légendes), chacune normalisée par l'arbre couvrant
  minimal ; valeurs : Physarum `TL_MST = 1,75 ± 0,30`, `MD_MST = 0,85 ± 0,04`, 14 ± 4 % des pannes déconnectent ;
  ferroviaire `TL_MST ≈ 1,8`, `MD_MST = 0,85`, 4 % ; les neuf valeurs `(TL_MST, FT_MST, MD_MST)` de la Fig. S1 (une
  simulation chacune) : `I0 = 0,20 : (1,05 ; 0,00 ; 0,97)`, … `3,00 : (1,56 ; 0,97 ; 0,85)`. **Non lu / non donné** : le
  maillage, le nombre de nœuds, le pas de temps, le schéma, les conductivités de départ, **la formule des trois
  mesures**, le nombre de simulations et la graine des courbes de la Fig. 3 ; aucune page d'éditeur ; le lien `FT = 1 -
  probabilité de déconnexion` est **une déduction** des valeurs du supplément (0,00 pour un réseau proche de l'arbre,
  0,96 pour le ferroviaire). Aucune valeur de l'article n'est visée par les tests : on n'a ni son maillage ni son pas.
- **Tero, Kobayashi, Nakagaki (2007)**, J. Theor. Biol. 244, 553-564 : lu (version d'auteur de 2006) : la forme sigmoïde
  de type II, `f̃(Q̃) = (1 + a) Q̃^μ / (1 + a Q̃^μ)`, le schéma semi-implicite dont le pas n'est pas publié, les
  conductivités de départ « tirées au hasard dans [0,5 ; 1,0] ». Voir `references/etape-1-valeurs.md`.
- **Bonifaci, Mehlhorn, Varma (2012)** : le théorème 2 (loi linéaire : une seule route survit si le plus court chemin
  est unique) ; il explique, **sans le prouver ici**, que la loi linéaire ne donne pas un arbre (voir `V-ESP-02`).
- **Ce qui est à nous** (aucun article) : `γ = 2` (l'article prend 1,8) ; la façon de tirer la paire ; le pas d'Euler ;
  le `Reseau` ; la clôture métrique et Kruskal ; les formules de TL, MD et FT ; le seuil `0,001` d'un tube vif ; les
  directives `nourriture`, `debit`, `loi` ; l'état `PHYSAR02` ; la couleur des nourritures.

---

## la loi (`etape_1r_loi.rs`)

Source : la doc de `reseaux` (« La loi », « Le pas à plusieurs sources de nourriture ») ; `Tubes::avancer` (cœur, déjà
éprouvé) ; des calculs à la main ; le modèle Python `ref_reseaux.py` ; un pas naïf écrit dans le test.

- **V-LOI-01** : les valeurs de la loi. `Lineaire` : `|q|` (`-0.0` donne `+0.0`, NaN reste NaN) ; `Sigmoide` : `f(0) =
  +0.0`, `f(1) = 0,5`, `f(2) = 0,8`, `f(3) = 0,9`, `f(0,5) = 0,2` (quotients exactement arrondis), `f(0,1)`, `f(10^3)`,
  `f(10^-3)`, `f(7,25)`, `f(10^6)` aux bits du modèle Python ; `1,0` quand `q * q` déborde (`1e155`, `inf`) ; la parité
  ; la monotonie et `f < 1` jusqu'à `10^6`. Source : calcul à la main et modèle Python. Tolérance : aucune (bits).
- **V-LOI-02** : avec la loi linéaire, `avancer_paire` est `Tubes::avancer` : pressions, flux et conductivités, 40 pas,
  8 graphes de 9 à 16 nœuds, trois nourritures dont la paire change, deux débits. Source : le cœur. Tolérance : aucune.
- **V-LOI-03** : un tube seul sous la sigmoïde : `D <- D + h (f(I0) - D)` avec `f(I0) = I0² / (1 + I0²)`, la pression de
  la source `I0 * cout / D`, celle du puits `+0.0`, `D` tend vers `f(I0)` (sous 1) ; trois tubes, deux sens, 60 pas.
  Source : calcul fermé. Mesure (six cas) : écart de `D` 4,0e-16 (relatif à `f(I0)`), de `|Q| - I0` 1,5e-16, de la
  pression 2,2e-16 ; tolérance `1e-15` (facteur 2,5), arrondi du solveur.
- **V-LOI-04** : une chaîne de cinq tubes sous la sigmoïde (`I0 = 2`, `f = 0,8`), 40 pas, deux sens : chaque `D` suit sa
  récurrence, la pression de la source est `I0 * somme(cout / D)`. Mesure : écart de `D` 6,9e-16, de `|Q| - I0` 5,6e-15,
  de la pression 3,2e-15 ; tolérances `4e-15`, `1,5e-14`, `1e-14` (facteurs 5,8, 2,7, 3,2).
- **V-LOI-05** : la saturation : aucune conductivité ne dépasse `max(départ, f(I0))` ; six graphes, `I0 = 6`, 100 pas à
  `h = 0,5` : la plus grande est `< 1` sous la sigmoïde et `> 1,5` sous la loi linéaire (`I0 = 6`) ; un tube seul
  descend de `5,0` vers `0,5` sans passer dessous. Source : calcul fermé. Critère : inégalités.
- **V-LOI-06** : `avancer_paire` contre le pas naïf du test, pour les deux lois : pressions, flux et conductivités aux
  bits, 25 pas, 12 graphes de 6 à 17 nœuds, au moins un tube au plancher. Source : pas naïf. Tolérance : aucune.
- **V-LOI-07** : le plancher : une branche morte décroît comme `0,8 * 0,75^n`, aux bits, puis est tenue **exactement** à
  `1e-6` ; atteint au pas numéro 47 (`0,8 * 0,75^48 = 8,05e-7`, calculé en Python) pour les deux lois. Tolérance :
  aucune.
- **V-LOI-08** : les erreurs de `new`, `au_hasard`, `avancer_paire`, `avancer`, leur ordre (`Taille`, `Conductivite`,
  `Nourriture`, `Debit` ; `Taille`, `Nourriture`, `Noeud`, `Pas`, `Solveur`), « rien n'a changé » (conductivités et
  générateur). Source : doc. Critère : égalité.
- **V-LOI-09** : `k = 2` : `avancer` consomme exactement deux tirages. Source : doc (`borne(1)` consomme un tirage).

## le tirage (`etape_1r_tirage.rs`)

Source : Tero 2010 (« tirée au hasard ») ; **la façon de tirer est à nous** (doc de `reseaux`) ; `Alea::borne` (vecteurs
de l'étape 0) ; le modèle Python (`Pcg.bounded`, transcrit du C de `pcg_basic.c`).

- **V-TIR-01** : les suites de paires du modèle Python : (graine, `k`) = (1, 2), (1, 3), (7, 5), (3, 16), (2, 4), 12 à
  16 paires chacune, et l'état du générateur après. Tolérance : aucune.
- **V-TIR-02** : contre le tirage naïf du test : `k` de 2 à 20, trois graines, 300 tirages : même paire, même état du
  générateur après chaque tirage.
- **V-TIR-03** : `k = 2` : la paire est `(i, 1 - i)` avec `i = suivant() % 2` du premier tirage, et deux tirages de 32
  bits consommés ; calculé sans `borne`.
- **V-TIR-04** : jamais source égale au puits, toujours dans `0..k` (`k` de 2 à 16, 2 000 tirages) ; les 20 paires
  ordonnées apparaissent pour `k = 5` ; panique pour `k < 2`.
- **V-TIR-05** : fréquences : comptes **exacts** du modèle Python (`k = 4`, graine 7, 20 000 tirages ; `k = 5`, graine
  11, 100 000 tirages) ; `chi²` sous `45` pour `k = 5` (20 000 tirages, 19 degrés de liberté) et sous `330` pour `k =
  16` (24 000 tirages, 239 degrés), huit graines. Mesure sur 40 **autres** graines (5000 à 5039, modèle Python) : `k =
  5` moyenne 17,4 maximum 36,4 ; `k = 16` moyenne 238,2 maximum 305,2. Les huit graines du test : au plus 24,1 et 278,3.
- **V-TIR-06** : `Reseau::avancer` tire la paire avec `tirer_paire` : 40 pas contre un générateur parallèle et un réseau
  jumeau qui avance par `avancer_paire`.

## l'arbre (`etape_1r_arbre.rs`)

Source : doc de `reseaux` (clôture métrique, Kruskal, égalités au plus petit `(distance, i, j)`, sommes dans un ordre
fixe) ; Tero 2010 normalise par « l'arbre couvrant minimal » sans formule ; le modèle Python.

- **V-MST-01** : de petits graphes à la main : les longueurs et non les coûts (triangle `3, 3, 10` : `TL_MST = 6`,
  `MD_MST = 4`) ; la clôture métrique (nœuds de passage : `[(1,2), (0,1)]`, `TL_MST = 7`, `MD_MST = 14/3`) ; les rangs
  et non les nœuds ; deux nourritures, distance comptée depuis le rang 0 (`0,6000000000000001` contre `0,6`). Tolérance
  : aucune.
- **V-MST-02** : les égalités : le graphe complet de quatre nourritures (étoile de 0, `MD_MST = 1,5`), l'anneau
  (`[(0,1), (0,3), (1,2)]`, `MD_MST = 10/6`), les mêmes nœuds en ordre permuté (`[(0,2), (0,3), (1,2)]`, `10/6`).
- **V-MST-03** : l'ordre des sommes : chaîne de longueurs `0,1`, `0,15`, `0,2` : `TL_MST = 0,45` (bits `…CCCD`), `MD_MST
  = 0,25` exactement ; depuis le plus grand rang on aurait `0,24999999999999997`. Source : cherché en Python (30
  triplets sur 512 donnent deux bits différents).
- **V-MST-04** : les erreurs `Nourriture` puis `Coupee` ; un graphe non connexe avec des nourritures dans la même
  composante donne un arbre.
- **V-MST-05** : 60 graphes au hasard (5 à 14 nœuds, 3 à 6 nourritures, **longueurs différentes des coûts**, la moitié à
  longueurs multiples de `0,5` : 19 graphes sur 60 ont deux distances égales) contre l'arbre naïf du test : paires,
  `TL_MST` et `MD_MST` aux bits.
- **V-MST-06** : la minimalité contre une énumération de tous les arbres couvrants de la clôture (40 graphes, 4 ou 5
  nourritures). Mesure : écart au plus 8,9e-16 ; tolérance `1e-12` (autre ordre de somme).

## les mesures (`etape_1r_mesures.rs`)

Source : doc de `reseaux` (formules à nous) ; Tero 2010 (définitions en mots) ; calculs à la main (écrits en commentaire
des tests) et modèle Python `ref_reseaux.mesures_reseau`.

- **V-MES-01** : un arbre (chaîne de longueurs `1, 2, 4, 8`) : `TL = 15`, `MD = 15` (ou 10 pour trois nourritures), `FT
  = 0`, rapports `1` et `1`.
- **V-MES-02** : un anneau (`TL = 4`, `MD = 2`, `FT = 1`, `TL / TL_MST = 2`), un anneau inégal (`TL = 10`, `MD = 3`, `TL
  / TL_MST = 10/3`), un arbre plus un raccourci (`FT = 3/5 = 0,6`, `TL = 5,5`, `MD = 2,5`, `TL_MST = 4`, `MD_MST = 8/3`,
  rapports `1,375` et `0,9375`).
- **V-MES-03** : une branche morte vive (`FT = 1/3`) ; un tube sous le seuil (`FT = 1/3`) ; une nourriture coupée (`MD =
  +inf`) ; aucun tube vif (`TL = 0`, `MD = +inf`, `FT = 0`) ; deux composantes (les nourritures déjà séparées ne
  comptent pas comme « coupées par la panne » : `FT = 1/3`).
- **V-MES-04** : l'ordre des sommes de `MD` (chaîne `0,1`, `0,15`, `0,2` : `0,25` exactement, pas `0,24999999999999997`)
  et la direction des distances (`[0, 3]` : `0,6000000000000001`, `[3, 0]` : `0,6`).
- **V-MES-05** : le seuil est compris : `D = 0,5` vif, `0,5` moins un ulp non, NaN jamais ; un seuil NaN ou `+inf` :
  aucun tube vif.
- **V-MES-06** : 80 réseaux au hasard (5 à 16 nœuds, 2 à 7 nourritures, tubes à `0,1`, `0,5` ou `0,9`) contre les
  mesures naïves du test : `TL`, `MD` (infini compris) et `FT` aux bits, les rapports d'après `arbre_couvrant`.
  Couverture exigée : au moins 10 `MD` infinis, 10 `FT = 0`, 10 `FT = 1`, 20 entre les deux.
- **V-MES-07** : les erreurs de `mesurer` (`Taille`, `Nourriture`, `Coupee`, dans cet ordre) et les paniques de
  `distance_moyenne` et de `tenue`.

## « en esprit », Tero 2010 (`etape_1r_esprit.rs`)

Sources : Tero 2010 (qualitatif, voir plus haut) ; **aucune valeur publiée n'est visée**. Tolérances **mesurées** sur 40
graines (2000 à 2039) autres que celles du test (1 à 8), grille 10 × 10 à huit voisins, six nourritures, 800 pas à `h =
0,1`, seuil `0,001`. `mesures/mes_1r_esprit.rs` imprime ces tableaux.

- **V-ESP-01** : sous la sigmoïde, `I0 = 1, 2, 3, 4, 8` : en moyenne `TL / TL_MST` = 1,03, 1,24, 1,55, 1,86, 3,14 (la
  plus petite hausse 0,21 ; seuil `0,1`) ; `FT` = 0,00, 0,53, 0,91, 0,99, 1,00 (hausses 0,53, 0,39, 0,078, 0,009 ;
  seuils `0,3`, `0,2`, `0,03`, puis « pas de baisse ») ; `MD / MD_MST` = 1,06, 0,98, 0,95, 0,91, 0,85 (baisses de 0,082
  à 0,034 ; seuil `0,015`). Pire cas : pour chaque graine, `TL` et `FT` ne baissent jamais quand `I0` croît (plus petite
  différence : 0 exactement). Les huit graines du test donnent les mêmes ordres de grandeur (hausses de `TL` au moins
  0,22).
- **V-ESP-02** : à `I0 = 1`, sous la sigmoïde, `FT = 0` pour chaque graine (40 sur 40) et `TL / TL_MST` entre `0,95` et
  `1,2` (mesuré de 0,979 à 1,115) ; **sous la loi linéaire**, à `I0 = 0,5` et `1`, `FT = 1` pour chaque graine (40 sur
  40) et `TL / TL_MST` au moins `5` (mesuré 5,6 à 6,5) et `6` (6,4 à 7,2). **Contre l'attente du pilote** (« à faible
  `I0` la loi linéaire tend vers un arbre ») : le réseau reste épais ; à 3 000 pas, sur un maillage aux longueurs
  aléatoires (20 graines, `I0 = 0,5`, mesuré à part), `TL / TL_MST` baisse de 5,3 à 4,0 et `FT` reste à 1.

## l'aperçu (`etape_1r_apercu.rs`)

Source : doc de `reseaux` et de `tubes::apercu` ; calculs à la main (formule entière du niveau et du mélange) et modèle
Python `ref_reseaux.apercu`. Tolérance : aucune (octets).

- **V-APR-01** : un petit réseau à deux étages, décalé de `(3, 2)` : le dessin en caractères (8 × 3 points), aux
  échelles 1, 2 et 3 ; l'ordre des nourritures ne compte pas ; sans nourriture, `tubes::apercu` ; la nourriture sur un
  nœud lié à l'étage du dessus recouvre la teinte du lien.
- **V-APR-02** : les paniques (échelle 0, référence nulle, négative, NaN ou infinie, mauvais nombre de conductivités,
  nourriture hors du graphe).
- **V-APR-03** : douze plans murés (colonne 0 murée : `xmin = 1`), un ou deux étages : tout pixel hors des carrés de
  nourriture est celui de `tubes::apercu`, les carrés sont `NOURRITURE`.

## `lancer` (`etape_1r_lancer.rs`)

Source : la doc de `lancer` (« Ce que fait l'étape 1, partie "les réseaux" ») et de `graphe` ; le modèle Python
`ref_reseaux.py`.

- **V-LRES-01**, **V-LRES-02** : l'écriture canonique `PHYSAR02` de deux petits cas (linéaire à deux nourritures,
  sigmoïde à trois, nourritures dans le désordre), octet par octet : 165 et 169 octets ; SHA-256 `1a05ce67…61cd` et
  `d8973396…48dd`, conductivités finales et état du générateur du modèle Python.
- **V-LRES-03** : les empreintes de 14 scénarios (`reseau.txt` graines 1 et 2, ses variantes en nombre de pas, loi,
  débit, `h`, ordre des deux premières nourritures ; un cas à murs et à poche ; un cas à deux étages ; seize nourritures
  ; les débits `20` et `0.05`), du modèle Python. L'ordre des directives, les commentaires, un mur coupé ou doublé ne
  changent pas l'empreinte ; chaque paramètre la change.
- **V-LRES-04** : `references/reseau-graine-1.empreinte` (régression), produite par le binaire, valeur du modèle Python.
- **V-LRES-05** : les mesures en CSV : deux petits cas en entier, et pour quatre scénarios le nombre de lignes, la
  première, la dernière, le SHA-256 ; chaque ligne : deux rangs différents de `0..k`.
- **V-LRES-06** : `lancer` est « `au_hasard` puis `avancer` » : le test refait la simulation avec l'API et retrouve les
  mesures et l'empreinte.
- **V-LRES-07** : l'aperçu : deux petits cas à 40 × 8 pixels (couleurs calculées à la main et par Python), `reseau.txt`
  (184 × 184, SHA-256 `c3856831…cfbd9`, les six carrés de nourriture, aucun autre pixel de cette couleur), `MURS` (136 ×
  104), `ETAGES` (192 × 56).
- **V-LRES-08** : les erreurs de directives : bornes comprises (`debit` `0.05`, `20`, seize nourritures), chaque faute
  de `debit`, de `loi`, de `nourriture`, les doublons, les deux familles (`source` ou `puits` avec un réseau), ce qui
  manque, les murs (2e nourriture, 1re, les deux, 3e, 4e, poches, séparations, deux étages), et la priorité entre ces
  règles.
- **V-LRES-09** : un calcul qui s'arrête : un lien entre étages de coût `10^16` : pas 4 (loi linéaire), pas 3 (sigmoïde)
  ; `10^17` : pas 0 ; calculés par le modèle Python (la procédure du solveur avec la vérification des pivots).
- **V-LRES-10** : les anciens scénarios gardent leur empreinte (étape 0, cœur, 1L) ; `debit` avec `source` est une
  erreur.
- **V-LRES-11** : le binaire : code 0 et trois fichiers pour `reseau.txt` ; code 1 sans rien écrire pour une nourriture
  murée (`ligne 2`) et pour un calcul qui s'arrête (`pas 4`).

## Ce qui n'est pas testé

Linux contre Windows (la CI rejoue `references/reseau-graine-1.empreinte`) ; d'autres maillages, d'autres nombres de
nourritures, d'autres `γ` ; les grands réseaux (plus de quelques centaines de nœuds) ; la vitesse de convergence ; les
valeurs de l'article (aucune n'est visée).

## campagne

Source : la campagne de fautes du juge (49 codes nuls, 319 fautes ; six faiblesses fermées par ces tests,
`tests/etape_1r_campagne.rs`).

- **V-RCAM-01** : une `nourriture` qui redonne un nœud déjà donné plus haut (pas seulement juste avant) est fautive : la
  ligne 4, puis la ligne 5.
- **V-RCAM-02** : `sigmoid`, `sigmoides`, `lineair`, `linear`, `lineaires` ne sont pas des noms de loi : la ligne 5.
- **V-RCAM-03** : un treillis de plus de 512 points de large (deux étages de 150 colonnes) : échelle 1, l'image a la
  largeur et la hauteur du treillis (en-tête IHDR), une ligne de mesures.
- **V-RCAM-04** : `Reseau::au_hasard` refuse un débit infini, `NaN`, nul ou négatif (`Debit`), sans tirer.
- **V-RCAM-05** : l'ordre des nourritures d'un réseau est celui de la liste donnée.
- **V-RCAM-06** : `Taille` quand le graphe a plus d'arêtes que de conductivités, par `avancer_paire` et par `avancer` ;
  rien ne change, `alea` ne bouge pas.
