# Étape 0 : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de l'étape 0 attend, d'où elle vient, avec quelle tolérance et quel critère de
réussite. Chaque identifiant (`V-SOLV-01`…) est cité dans la doc des tests qui l'emploient (`tests/etape_0_*.rs`). Elle est
séparée des résultats figés de notre propre code (`references/*.empreinte`, étiquetés « régression » dans les tests) : **aucune
valeur de cette fiche ne vient de notre code**. Une valeur y vient d'une publication lue, d'un calcul exact fait par un outil
indépendant (nommé), ou d'une convention que la documentation du module fixe (dite telle, avec la raison).

**Outils indépendants** : Python 3.13 (`fractions` en rationnels exacts, flottants IEEE 754, `struct`, `hashlib`, `zlib`). Le code
Rust n'est jamais l'outil. Trois scripts, gardés avec les notes de la session qui a écrit ces tests : `ref_alea.py` (PCG32,
d'après le C de référence), `ref_graphe.py` (la grille, l'écriture canonique, les mesures, le treillis de l'aperçu, écrits d'après
la seule doc des modules) et `ref_solveur.py` (la solution exacte en fractions, et la procédure de référence du solveur transcrite
opération pour opération).

**Tolérance, règle générale** : les tests exacts (entiers, octets, chaînes, bits d'un flottant) n'ont **aucune** tolérance. Seul le
solveur en a (V-SOLV-02, 03, 04, 11, 13), mesurées sur un prototype avant d'être fixées, jamais élargies pour un cas : elles
séparent une erreur d'arrondi (de l'ordre de `1e-16`) d'une faute (de l'ordre de 1).

**Ce que les tests « juste » ne disent pas** est écrit dans chaque test, ligne `Ne couvre pas :`. Les deux seuls tests
`régression :` de l'étape 0 sont `regression_reference_grille_graine_1` et `regression_reference_du_binaire`.

**D'où viennent `alea`, `empreinte`, `image` et `regles`** : ces quatre sections, leurs tests et la doc de ces modules sont repris
de l'étape 0 de Givre Lab (même licence, même maison), où un juge les a éprouvés ; les dates de lecture des sources sont celles de
ce travail.

---

## graphe : le graphe et son ordre canonique

Source : la définition écrite dans la documentation de `graphe` (une convention de l'étape 0 : l'ordre canonique sert au
solveur, à l'empreinte et au rendu). Tolérance : aucune.

| Identifiant | Valeur | Source | Critère |
|---|---|---|---|
| V-GRAPHE-01 | cinq arêtes données `3-2, 0-1, 3-1, 1-2, 3-0` se rangent `(0,1) (0,3) (1,2) (1,3) (2,3)`, chacune avec sa longueur et son coût ; les nœuds gardent leur ordre | doc : `a < b`, tri par `(a, b)` ; lu sur la liste | égalité |
| V-GRAPHE-02 | arêtes de chaque nœud du même graphe, par indice croissant : `[0,1]`, `[0,2,3]`, `[2,4]`, `[1,3,4]`, `[]` ; panique hors du graphe | lu sur la liste canonique ; doc de `incidentes` | égalité ; `catch_unwind` |
| V-GRAPHE-03 | `arete_entre` dans les deux sens ; `None` sans arête, pour `a == b`, hors du graphe | doc de `arete_entre` | égalité |
| V-GRAPHE-04 | `Vide` sans nœud ; `Arete(i)` : le plus petit rang, dans la liste donnée, d'une arête fautive (boucle, bout hors du graphe, longueur ou coût non fini ou `<= 0`, seconde arête d'une paire) | doc de `Graphe::new` ; quinze fautes insérées à chaque rang d'une liste saine | égalité |
| V-GRAPHE-05 | connexe : un nœud seul, un chemin, un cycle, une étoile ; non connexe : un nœud sans arête à côté d'autres, deux morceaux | lu sur le dessin | égalité |
| V-GRAPHE-06 | longueurs canoniques `1, 1e16, 1, 1` : la somme de gauche à droite depuis `0.0` vaut exactement `1e16` ; l'ordre inverse donnerait `1e16 + 4`, l'ordre donné `1e16 + 2` | arithmétique IEEE 754 : `1e16 + 1` tombe à mi-chemin de deux flottants (pas de 2) et s'arrondit au pair, `1e16` ; vérifié en Python | égalité des bits |
| V-GRAPHE-07 | les 56 octets de l'écriture d'un graphe de deux nœuds `(-2, 3, 1)`, `(258, -1, 0)` et d'une arête de longueur `1.5` et de coût `0.1` ; les 20 octets d'un nœud seul | écrits à la main d'après la doc (« L'écriture canonique ») ; `1.5 = 0x3FF8000000000000`, `0.1 = 0x3FB999999999999A` (Python `struct`) | égalité |

## graphe : la grille

Source : la définition de la doc (« La grille »). Tolérance : aucune.

| Identifiant | Valeur | Source | Critère |
|---|---|---|---|
| V-GRILLE-01 | l'indice `(y * largeur + x) * etages + etage` : sur 4 × 3 à 2 étages, `(0,0,1)` → 1, `(1,0,0)` → 2, `(3,0,1)` → 7, `(0,1,0)` → 8, `(2,1,1)` → 13, `(3,2,1)` → 23 ; panique hors de la grille | la formule de la doc, calculée à la main | égalité ; `catch_unwind` |
| V-GRILLE-02 | les arêtes de 3 × 2 (voisinage 4), de 2 × 2 (voisinage 8), de 2 × 1 à deux étages, de 1 × 1 à trois étages | écrites à la main d'après la définition | égalité |
| V-GRILLE-03 | la diagonale `2.0_f64.sqrt()` a pour bits `0x3FF6A09E667F3BCD` | la racine carrée d'IEEE 754 est arrondie correctement, donc la même partout ; Python `math.sqrt(2.0)` | égalité des bits |
| V-GRILLE-04 | `largeur * hauteur * etages` nœuds ; `etages * ((l-1) * h + l * (h-1) + d) + (etages-1) * l * h` arêtes, `d = 2 (l-1)(h-1)` en voisinage 8, sinon 0 ; toute grille est connexe | comptage : arêtes horizontales, verticales dans l'étage, diagonales, liens entre étages | égalité |
| V-GRILLE-05 | la grille entière, nœud pour nœud et arête pour arête, pour vingt grilles (1 à 3 étages, deux voisinages) | un générateur naïf écrit dans le test, dans un autre ordre de boucles que la définition, trié par insertion | égalité |
| V-GRILLE-06 | sur 3 × 2 à 2 étages avec `0.25` et `7.0` : six liens verticaux `(0.25, 7.0)`, chacun entre les deux étages d'une même case ; longueur totale `15.5` (un étage : `7.0`) | définition ; `2 * 7 + 6 * 0.25` (exact) | égalité |
| V-GRILLE-07 | `ErreurGraphe::Grille` pour un côté ou un nombre d'étages nul, une longueur ou un coût vertical nul, négatif, infini ou NaN, même avec un seul étage | doc de `Grille::graphe` | égalité |
| V-GRILLE-08 | les 248 octets de l'écriture de la grille 3 × 2 | modèle Python `ref_graphe.py`, écrit d'après la doc | égalité |

## solveur : le flux de Kirchhoff

Sources : les lois de Kirchhoff et d'Ohm (énoncées dans la doc de `solveur`) ; des solutions exactes en rationnels (Python
`fractions`, élimination de Gauss exacte, `ref_solveur.py`) ; la procédure de référence de la doc, transcrite deux fois hors du
code jugé : en Python (`ref_solveur.py`) et dans le fichier de test (`reference`, dense, sans rien sauter).

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-SOLV-01 | chaîne `0-1-2-3`, conductances 1, une unité en 0, masse en 3 : pressions `3, 2, 1, 0`, flux `1, 1, 1` ; conductances `2, 4, 8` : `0.875, 0.375, 0.125, 0` ; masse en 1, injection en 3 : `0, 0, 1, 2`, flux `0, -1, -1` | loi d'Ohm en série, à la main ; tout le calcul est exact en `f64` | aucune | égalité |
| V-SOLV-02 | pont de Wheatstone, arêtes `(0,1) (0,2) (1,2) (1,3) (2,3)` de conductances `1, 2, 5, 3, 4`, une unité en 0, masse en 3 : pressions `74/155, 21/155, 23/155, 0` ; flux `53/155, 102/155, -2/31, 63/155, 92/155` | solution exacte en fractions (`ref_solveur.py`) | `1e-12` en relatif (erreur observée : `4e-16`) | écart sous la tolérance |
| V-SOLV-03 | le cube de résistances unité (la grille 2 × 2 à 2 étages) : `7/12` entre deux sommets voisins, `3/4` en diagonale d'une face, `5/6` en grande diagonale | valeurs classiques des manuels d'électricité, recalculées en fractions exactes (`ref_solveur.py`) | `1e-12` | écart sous la tolérance, dans les deux sens |
| V-SOLV-04 | en tout nœud autre que la masse, ce qui sort par les arêtes égale l'injection ; la pression de la masse est `+0.0` ; la masse absorbe la somme des autres injections ; avec une source et la masse pour puits, toute pression est entre 0 et celle de la source | les lois elles-mêmes, sur 40 graphes connexes au hasard et trois grilles ; la matrice du système est définie positive : des pressions qui tiennent ces lois sont les seules | résidu sous `1e-10` fois l'échelle des termes (observé : sous `2e-15`) | chaque nœud |
| V-SOLV-05 | bits des pressions : chaîne de cinq nœuds (masse en 2, conductances `0.1, 0.2, 0.3, 0.7`, injections `1, 0, 99, 0, -0.25`) : `0x402E000000000000, 0x4013FFFFFFFFFFFF, 0, 0xBFEAAAAAAAAAAAAA, 0xBFF30C30C30C30C3` ; pont de Wheatstone (`0.1, 0.7, 1.3, 0.3, 2.9` ; injections `0.5, 0, -0.2, 0.3` ; masse en 1) : `0x3FEC9858B545E7E4, 0, 0x3FD3A55D0C0D7FC0, 0x3FD7CDDC52EC3BC6` ; pont de V-SOLV-02 : `0x3FDE8E09E8E09E8E, 0x3FC157891578915A, 0x3FC2FE592FE59300, 0` | la procédure de référence transcrite en Python (`ref_solveur.py`), flottants IEEE 754, mêmes opérations dans le même ordre | aucune | égalité des bits |
| V-SOLV-06 | sur 30 graphes au hasard et trois grilles, masse au début, au milieu, à la fin, avec et sans conductances nulles : les bits de la procédure de référence | la procédure de la doc recopiée dans le test (`reference`) | aucune | égalité des bits |
| V-SOLV-07 | injections multipliées par 4 : pressions multipliées par 4 ; conductances multipliées par 8 : pressions divisées par 8 et flux inchangés | une multiplication par une puissance de 2 est exacte en `f64` (hors débordement) et traverse chaque ligne de la procédure | aucune | égalité des bits |
| V-SOLV-08 | la pression de la masse est `+0.0` ; son injection n'est pas lue ; une injection `-0.0` donne les bits de `+0.0` ; une conductance `-0.0` vaut zéro | doc de `solveur` (le problème, les vérifications, le « + 0.0 ») | aucune | égalité des bits |
| V-SOLV-09 | l'ordre des erreurs : `Taille`, `Masse`, `Conductance(e)` (plus petit indice), `Injection(i)` (plus petit indice) | doc (« Les vérifications ») | — | égalité |
| V-SOLV-10 | `Isole(i)` : le plus petit nœud sans chemin de conductances strictement positives vers la masse | doc, vérification 5 ; lu sur le dessin de chaque réseau | — | égalité |
| V-SOLV-11 | `Pivot(1)` quand `1e308 + 1e308` déborde ; `Pivot(1)` sur la chaîne `1, 1e-300, 1` (`1.0 + 1e-300` vaut `1.0` : le pivot `1 - (-1 * 1) * (-1)` est nul) ; avec `1e-10`, les pressions exactes `2 + 1e10, 1 + 1e10, 1, 0` | doc, vérification 6 et « Ce que le solveur ne garantit pas » ; arithmétique IEEE 754 à la main | `1e-6` en relatif pour le cas `1e-10` (observé : `8,3e-8` ; attendu : `1e10 * 2^-53`) | égalité ; écart sous la tolérance |
| V-SOLV-12 | `flux` : pressions `5, 3, 0` et conductances `2, 10, 0.5` sur le triangle : `4, 50, 1.5` ; une soustraction puis un produit ; panique si une longueur ne convient pas (trop courte ou trop longue) | définition de la doc | aucune | égalité (et des bits) |
| V-SOLV-13 | une grille de 80 × 80 se résout en moins de dix secondes, Kirchhoff tenu en chaque nœud, même résistance dans les deux sens entre deux coins | doc (« sauter les zéros du profil ») : environ vingt millions d'opérations ; la procédure dense en ferait plus de quarante milliards | dix secondes (la durée mesurée sur le prototype est de 30 millisecondes) ; résidu sous `1e-9` fois l'échelle | durée et résidus |

## scenario

Source : la définition du format, dans la documentation de `scenario` (une convention de l'étape 0, pas une valeur publiée). Les
bits des flottants viennent de Python (`float`, arrondi au plus proche). Tolérance : aucune.

| Identifiant | Ce qu'il fixe | Source |
|---|---|---|
| V-SCEN-01 | `grille 64 48` seule : un étage, voisinage 4, `1.0` et `1.0` ; `grille`, `voisinage 8`, `etages 2 0.5 3` dans n'importe quel ordre ; bornes 256 et 8 | doc (« Le format ») |
| V-SCEN-02 | lignes vides, blanches, commentaires (même indentés) ignorés ; l'espace et la tabulation, seuls blancs (un saut de page ou une espace insécable font partie d'un mot) ; un seul `\r` final retiré | doc |
| V-SCEN-03 | bornes de `grille` : 1 à 256 ; zéros de tête permis ; un entier plus grand que `u32::MAX` ou que `u64::MAX` est hors bornes, sans rebouclage | doc |
| V-SCEN-04 | un entier : chiffres décimaux ASCII seulement ; le nombre d'arguments ; un `#` de fin de ligne est un argument | doc |
| V-SCEN-05 | `voisinage` : `4` ou `8` écrits ainsi ; `etages` : un entier de 1 à 8, deux flottants décimaux finis et strictement positifs ; `123456789.123456789` a pour bits `0x419D6F34547E6B75` | doc ; Python `float` |
| V-SCEN-06 | une seconde `grille`, un second `voisinage`, un second `etages` : la seconde ligne est fautive | doc (« Les erreurs ») |
| V-SCEN-07 | un nom : minuscules ASCII, chiffres, `_`, non vide | doc |
| V-SCEN-08 | le numéro de ligne compte les lignes ignorées ; la première ligne fautive l'emporte, aussi sur `Manquante` | doc |
| V-SCEN-09 | sans `grille` : `Manquante("grille")`, même avec `voisinage` ou `etages` | doc |
| V-SCEN-10 | toute autre directive est gardée, dans l'ordre, avec sa ligne et ses arguments ; `directive(nom)` rend la première de ce nom entier (pas d'un début de nom) | doc |
| V-SCEN-11 | `x 1 # note` : trois arguments | doc |
| V-SCEN-12 | `Directive::entier` : `12`, `0`, `u64::MAX`, `007` ; refus de `u64::MAX + 1`, `+3`, `-1`, `abc`, `1.5`, d'un argument absent | doc |
| V-SCEN-13 | `Directive::flottant` : `0.1` = `0x3FB999999999999A`, `-0.0` = `0x8000000000000000`, `007.25` = `0x401D000000000000`, `123456789.123456789` = `0x419D6F34547E6B75`, `-3.14159265358979323846` = `0xC00921FB54442D18` ; refus de `1e3`, `+1`, `.5`, `5.`, `inf`, `nan`… | doc ; Python `float` et `struct` |
| V-SCEN-14 | `scenarios/grille.txt` : 24 × 16, voisinage 8, deux étages, liens `1.0` et `3.0` | le contenu du fichier |

## rendu

Source : la définition de la doc de `rendu` (« L'aperçu d'un graphe ») ; les treillis des grilles recoupés par le modèle Python
`ref_graphe.py`. Tolérance : aucune (octets). Dans les dessins : `.` fond, `-` arête, `o` nœud, `P` puits.

| Identifiant | Valeur | Source |
|---|---|---|
| V-REND-01 | fond `(12, 16, 24)`, arête `(90, 110, 130)`, nœud `(225, 225, 215)`, puits `(240, 170, 40)`, écart 2 | doc (une convention) |
| V-REND-02 | 3 × 2 en voisinage 4 : `o-o-o / -.-.- / o-o-o` ; en voisinage 8 : `o-o-o / ----- / o-o-o` ; 2 × 3 ; 1 × 1 ; 4 × 1 | définition ; modèle Python |
| V-REND-03 | 3 × 2 à 2 étages : `P-P-P..P-P-P / -.-.-..-.-.- / P-P-P..P-P-P` (12 × 3 points) ; 3 × 3 à 3 étages : 19 × 5 points | définition ; modèle Python |
| V-REND-04 | un graphe de huit nœuds : le cadre part d'un nœud qui n'est pas le premier, un étage vide garde son panneau, une arête trop longue n'est pas dessinée, deux nœuds à la même position, un lien entre étages de positions voisines sans point d'arête : `P-o.o.............P...... / ..-..-................... / ..o...P.................. / ......................... / ......................P..` ; un nœud seul loin de l'origine ; un nœud seul à l'étage 2 | dessiné à la main d'après la définition ; modèle Python `ref_graphe.py` |
| V-REND-05 | à l'échelle `k`, chaque point est un carré uni de `k × k` pixels (image `12k × 3k` pour 3 × 2 à 2 étages, `k` de 1 à 5) ; panique à l'échelle 0 | définition (« divisions entières ») |

## lancer

Source : la doc de `lancer` (l'écriture canonique, les mesures, l'aperçu). Les empreintes sont le SHA-256 (Python `hashlib`) de
l'écriture canonique produite par le modèle Python `ref_graphe.py` ; deux d'entre elles sont aussi écrites octet par octet dans le
test. Tolérance : aucune.

| Identifiant | Valeur | Source |
|---|---|---|
| V-LANC-01 | `grille 1 1`, graine 0 (44 octets) → `e07d8298…d459` ; `grille 2 1`, graine 7 (80 octets) → `b48edde7…1f40` | octets écrits à la main d'après la doc ; l'état de `Alea::new` par `ref_alea.py` ; `hashlib` |
| V-LANC-02 | dix empreintes : `grille 3 2` graine 7 `4d066023…4008` ; `grille 2 3` `71bb260a…7b68` ; graine 8 `d1c70c11…05b9` ; voisinage 8 `958a852a…59f9` ; `etages 2 0.5 3` `ca0220ae…7db9` ; `etages 2 0.5 4` `4a798094…edb7` ; voisinage 8 et étages `23a47254…335c` ; `grille 3 3`, `etages 3 0.25 0.75`, graine 5 `1c220747…b378` ; `grille 256 1`, graine `u64::MAX` `1eceee45…2618` ; `grille 65 2`, voisinage 8, `etages 2 2.5 0.125`, graine 3 `99a5bbc7…7204` (empreintes complètes dans le test) | modèle Python ; `hashlib` |
| V-LANC-03 | les mesures : `0,6,7,7.000000` ; `0,6,11,12.656854` ; `0,12,20,17.000000` ; `0,12,28,28.313708` ; `0,27,54,40.500000` ; `0,256,255,255.000000` ; `0,260,772,1073.038672` ; `0,768,3220,3791.614716` | comptages de la définition ; somme des longueurs dans l'ordre canonique et `'%.6f'` en Python |
| V-LANC-04 | l'échelle `512 / TL` ramenée entre 1 et 8, et les dimensions de l'aperçu : 40 × 24, 96 × 24, 8 × 8, 504 × 8, 455 × 7, 510 × 2, 257 × 1, 511 × 1, 1024 × 1, 480 × 155 | la règle de la doc, calculée à la main et par le modèle Python |
| V-LANC-05 | `DirectiveInconnue(ligne)` : la première des directives restantes | doc de `lancer` |
| V-LANC-06 | deux lancers identiques ; l'empreinte a 64 chiffres hexadécimaux minuscules ; scénario d'exemple : graine 1 `c3361481…af59`, graine 2 `3e99fcca…d908` | doc ; modèle Python ; `hashlib` |
| V-LANC-07 | `references/grille-graine-1.empreinte` (régression) | produit par le binaire à la vague du code ; attendu : `c336148193197cb8541fd6a58c36f149118260789f9e388771405fe52e6eaf59` et un saut de ligne |

## binaire

Source : la documentation de `src/main.rs`. Les empreintes sont celles de V-LANC. Tolérance : aucune.

| Identifiant | Ce qu'il fixe | Source |
|---|---|---|
| V-BIN-01 | un lancer réussi écrit exactement `empreinte.txt`, `mesures.csv`, `apercu.png` (40 × 24 pour `grille 3 2`) | doc de `main.rs` ; V-LANC-02, 03, 04 |
| V-BIN-02 | les options dans tout ordre ; la graine `18446744073709551615` | doc ; V-LANC-02 |
| V-BIN-03 | un scénario à deux étages : aperçu de 96 × 24, premier pixel d'un puits | V-LANC-04 ; V-REND-03 |
| V-BIN-04 | un dossier existant voit ses trois fichiers remplacés ; le scénario d'exemple, graines 1 et 2 | doc ; V-LANC-06 |
| V-BIN-05 | toute erreur d'arguments ou de fichier : code 1, un message qui n'est pas une panique, rien de créé | doc |
| V-BIN-06 | un scénario refusé : code 1, `ligne <n>` ou `grille` dans le message, rien d'écrit, et un dossier de sortie absent n'est pas créé | doc |
| V-BIN-07 | `references/grille-graine-1.empreinte` (régression) | produit par le binaire à la vague du code |
| V-BIN-08 | quand un des trois fichiers ne peut pas être écrit (un dossier porte son nom dans le dossier de sortie) : code 1, un message qui n'est pas une panique, et aucun fichier écrit pendant ce lancer ne reste | doc de `src/main.rs` (« une écriture qui échoue ») ; test `etape_0_ecriture.rs`, ajouté après la campagne de fautes du juge |

## alea (PCG32, graine et flux)

Sources : M. E. O'Neill, PCG ; le code C de référence `pcg_basic.c` et la démonstration `pcg32-demo.c`
(`github.com/imneme/pcg-c-basic`, lus le 7/11/2026) ; la page « Using the Minimal C Implementation »
(`pcg-random.org/using-pcg-c-basic.html`, lue le 7/11/2026). `pcg32-demo.c` amorce avec `pcg32_srandom_r(&rng, 42u, 54u)` et,
à chaque ronde, tire 6 fois `pcg32_random_r`, puis 65 fois `pcg32_boundedrand_r(&rng, 2)` (pièces : `H` si non nul), puis 33 fois
`pcg32_boundedrand_r(&rng, 6)` (dés, plus un).

| Identifiant | Valeur | Source | Tolérance | Critère |
|---|---|---|---|---|
| V-ALEA-01 | six premiers tirages de (42, 54) : `0xa15c02b7 0x7b47f409 0xba1d3330 0x83d2f293 0xbfa4784b 0xcbed606e` | page pcg-random.org ci-dessus, ronde 1, ligne « 32bit » (lue en entier). Recoupée : Rosetta Code, « Pseudo-random numbers/PCG32 » (résultat de recherche) donne en décimal `2707161783 2068313097 3122475824 2211639955 3215226955` pour les cinq premiers, égaux à ces hexadécimaux | aucune (entiers) | égalité exacte des six `u32` |
| V-ALEA-02 | les 65 pièces après les 6 tirages : `HHTTTHTHHHTHTTTHHHHHTTTHHHTHTHTHTTHTTTHHHHHHTTTTHHTTTTTHTTTTTTTHT` | même page, ronde 1, ligne « Coins », lue en entier par le juge (page HTML téléchargée le 8/11/2026, texte brut) ; le modèle V-ALEA-04 et le modèle indépendant du juge donnent la même chaîne | aucune | égalité exacte des 65 caractères |
| V-ALEA-03 | les 33 dés après les 65 pièces : `3 4 1 1 2 2 3 2 4 3 2 4 3 3 5 2 3 1 3 1 5 1 4 1 5 6 4 6 6 2 6 3 3` | même page, ronde 1, ligne « Rolls », lue en entier par le juge ; les deux modèles donnent les mêmes | aucune | égalité exacte des 33 nombres |
| V-ALEA-04 | état (8 octets petit-boutistes) et incrément après amorçage de (42, 54), (0, 0), (`u64::MAX`, `u64::MAX`), (123456789, 987654321) ; quatre premiers tirages et état après eux de ces quatre couples ; état après un tirage de (42, 54) ; premiers tirages des flux 0, 1, 54, 55 de la graine 42 et de (54, 42) ; état après amorçage de `new(7)`, `new(1)`, `new(u64::MAX)` | calcul exact en entiers par un modèle Python (`ref_alea.py`) de `pcg32_srandom_r`, `pcg32_random_r` du C de référence ; **le modèle reproduit d'abord V-ALEA-01 à 03 sans écart**, puis seulement produit ces valeurs. Valeurs listées dans `tests/etape_0_alea.rs` | aucune | égalité exacte |
| V-ALEA-05 | numérateurs de `suivant_f64` de (42, 54) : `5677329748551934`, `6548306420333150`, `6742819640802732` ; `suivant_u64` : `0xA15C02B77B47F409`, `0xBA1D333083D2F293`, `0xBFA4784BCBED606E` | arithmétique entière sur V-ALEA-01 : les `u64` sont les paires de tirages mises bout à bout (premier en poids fort), les numérateurs sont ces `u64` décalés de 11 bits (`>> 11`) | aucune ; `numérateur · 2^-53` est exact en `f64` | égalité exacte |
| V-ALEA-06 | `borne(n)` pour `n` = 7, 3000000000, 2147483649, 4294967295 : 24 résultats chacun, graine 42 flux 54 | modèle Python de `pcg32_boundedrand_r` (`seuil = (-n) mod 2^32 mod n`, rejet de `r < seuil`) ; 10 rejets sur 34 tirages pour 3000000000, 16 sur 40 pour 2147483649 | aucune | égalité exacte |
| V-ALEA-07 | quatre graines fabriquées telles que le premier tirage brut vaille exactement le seuil ou le seuil moins un (n = 3000000000 : graines 17401024240561992018 puis 2631533607250342226 ; n = `u32::MAX` : 4099389142363320658 puis 18396926780031676754), et le résultat et l'état après `borne(n)` | construction : on inverse l'amorçage (`etat = (inc + graine) · M + inc` donne la graine pour un état voulu ; l'état voulu est construit bit à bit pour que la sortie XSH-RR soit la valeur voulue, avec `rot = 0`) ; vérifié par le modèle V-ALEA-04 (premier tirage = valeur voulue) ; résultats et états par le même modèle | aucune | égalité exacte : teste le bord `r >= seuil` |
| V-ALEA-08 | après 100 000 tirages de (42, 54) : XOR `0x842705F9`, somme mod 2^64 `214430088307331`, dernier `1187223689`, état ; après 1 000 tirages de (`u64::MAX`, `u64::MAX`) : XOR `0xAAE84890`, dernier `323556842`, état | modèle Python V-ALEA-04 | aucune | égalité exacte |
| V-ALEA-09 | `borne(1)` rend 0 et consomme exactement un tirage ; pour `n = 2^k` (k = 1, 2, 8, 16, 31), `borne(n) = tirage & (n − 1)` (200 tirages chacun) | définition : seuil `(-n) mod n = 0` pour une puissance de deux (aucun rejet) et `r % 2^k = r & (2^k − 1)` | aucune | égalité exacte avec un second générateur cloné qui tire sans `borne` ; ancrage : le premier tirage officiel `0xA15C02B7` (V-ALEA-01) masqué donne `1`, `0xB7`, `0x02B7`, `0x215C02B7` pour `n` = 2, 2^8, 2^16, 2^31 |
| V-ALEA-10 | `borne(0)` panique | « Panique : si n vaut 0 » dans la doc de `borne` (la référence C divise par zéro dans ce cas) | — | `#[should_panic]` |
| V-ALEA-11 | deux générateurs de même graine sont égaux ; un tirage les distingue ; un clone suit la même suite ; graines ou flux différents donnent des générateurs différents | doc de `Alea` : l'égalité est celle des deux `u64` | aucune | égalité / inégalité |

## empreinte (SHA-256)

Sources : NIST, exemples SHA-256 : `SHA256.pdf` (« One Block Message Sample » : `abc` ; « Two Block Message Sample » :
`abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq`), `SHA2_Additional.pdf` (« SHA-256 Test Data »), FIPS 180-2 annexe B.3 ;
tous lus dans le texte des PDF le 7/11/2026 (flux décompressés avec `zlib` de Python). Tolérance : aucune (octets).

| Identifiant | Valeur | Source | Critère |
|---|---|---|---|
| V-SHA-01 | `""` → `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` | CAVP du NIST, `SHA256ShortMsg.rsp` (`shabytetestvectors.zip`, CAVS 11.0, csrc.nist.gov), entrée `Len = 0`, lue par le juge le 8/11/2026 ; recoupée par Python `hashlib` | égalité |
| V-SHA-02 | `abc` → `ba7816bf 8f01cfea 414140de 5dae2223 b00361a3 96177a9c b410ff61 f20015ad` | `SHA256.pdf`, « Message Digest is » du premier exemple | égalité |
| V-SHA-03 | message de 448 bits → `248d6a61 d20638b8 e5c02693 0c3e6039 a33ce459 64ff2167 f6ecedd4 19db06c1` | `SHA256.pdf`, second exemple | égalité |
| V-SHA-04 | un million de `a` → `cdc76e5c 9914fb92 81a1c7e2 84d73e67 f1809a48 a497200e 046d39cc c7112cd0` | FIPS 180-2, annexe B.3 | égalité |
| V-SHA-05 | `SHA2_Additional.pdf`, « SHA-256 Test Data » : #1 1 octet `0xbd` → `68325720…732b` ; #2 4 octets `c98c8e55` → `7abc22c0…d9504` ; #3 55 octets nuls → `02779466…8ef7` ; #4 56 → `d4817aa5…9fbb` ; #5 57 → `65a16cb7…1785` ; #6 64 → `f5a5fd42…fb4b` ; #7 1000 → `541b3e9d…8a53` (empreintes complètes dans le test) | `SHA2_Additional.pdf`, section « SHA-256 Test Data » (lue) ; recoupées par `hashlib` | égalité |
| V-SHA-06 | `a` répété 55, 56, 63, 64, 65, 119, 120 fois | Python `hashlib` ; recoupé par `sha256sum` (mêmes empreintes). Longueurs : frontières du bourrage (55 : tient dans un bloc ; 56 : en demande un second ; 64 ; 119/120 la même frontière au second bloc) | égalité |
| V-SHA-07 | les 256 octets `0..=255` → `40aff2e9…4880` | Python `hashlib` | égalité |
| V-SHA-08 | `h = sha256(h ‖ "a"×n)` pour n de 0 à 300 → `8c913ede…b24c` | Python `hashlib` | égalité |
| V-SHA-09 | les 65 vecteurs « SHA-256 ShortMsg » (0 à 512 bits, octets quelconques) et le premier « LongMsg » (1304 bits) du CAVP | CAVP du NIST, `SHA256ShortMsg.rsp` et `SHA256LongMsg.rsp` (`shabytetestvectors.zip`, CAVS 11.0), copiés tels quels par le juge le 8/11/2026 ; les 129 vecteurs des deux fichiers recoupés par Python `hashlib` | égalité |

## image : CRC-32, Adler-32, PNG

Sources : catalogue de CRC de R. Williams (`reveng.sourceforge.io/crc-catalogue/17plus.htm`, lu le 7/11/2026) : CRC-32/ISO-HDLC,
polynôme `0x04c11db7` réfléchi, init `0xffffffff`, xorout `0xffffffff`, **check `0xcbf43926`** ; article « Adler-32 » de Wikipédia
anglais (lu) : `Wikipedia` → `0x11E60398`, `A = 920`, `B = 4582`, modulo 65521 ; RFC 1950 (lu) §2.2 (en-tête CMF/FLG : `0x78 0x01`
vérifie `(CMF·256 + FLG) mod 31 = 0`, Adler-32 en fin de flux, grand-boutiste) ; RFC 1951 (lu) §3.2.3 (BFINAL, BTYPE) et §3.2.4
(bloc stocké : `LEN`, `NLEN` complément à un, petit-boutiste, 65535 octets au plus) ; spécification PNG, 3e édition, W3C
(`w3.org/TR/png-3/`, lue : §5.2 signature `89 50 4E 47 0D 0A 1A 0A`, §5.3 chunks : longueur, type, données, CRC sur type et données,
§11.2.1 IHDR de 13 octets, type de couleur 2 = RVB, §9.2 filtre 0 « None », §7.3 un octet de filtre par ligne). Les autres valeurs
viennent de Python `zlib` (`crc32`, `adler32`, `decompress`). Tolérance : aucune.

| Identifiant | Valeur | Source | Critère |
|---|---|---|---|
| V-IMG-01 | `crc32("123456789") = 0xCBF43926`, `crc32("") = 0`, `crc32("IEND") = 0xAE426082` | catalogue (check) ; `zlib.crc32` | égalité |
| V-IMG-02 | `adler32("Wikipedia") = 0x11E60398`, `adler32("") = 1` | Wikipédia ; RFC 1950 | égalité |
| V-IMG-03 | `crc32(0..=255) = 0x29058C73` ; `adler32(0..=255) = 0xADF67F81` ; `adler32('a'×5552) = 0x47333829`, `×5553 = 0x7FBD388A` ; `adler32(0xFF×100000) = 0x149A302C` ; motif de 102 400 octets (`(i·7+3) mod 256`) : `crc32 = 0x2CD27A54`, `adler32 = 0xC63D43AA` ; comparés aussi à un CRC bit par bit et un Adler à modulo par octet écrits dans le test | Python `zlib` | égalité |
| V-IMG-04 | les 75 octets du PNG 2 × 1 (rouge, bleu), un par un (table dans le test) ; IEND `00 00 00 00 49 45 4E 44 AE 42 60 82` | octets posés à la main d'après la spécification PNG et RFC 1950/1951 ; CRC de IHDR `7B40E8DD` et d'IDAT `5536BAC7`, Adler `070001FF` par `zlib` ; `zlib.decompress` relit le flux IDAT et retrouve `00 FF 00 00 00 00 FF` | égalité |
| V-IMG-05 | petites images 1×1, 2×1, 1×2, 3×5, 17×13, 64×48 relues par le décodeur du test : un bloc, `IDAT = 2 + 5 + L + 4` | structure de la spécification ; décodeur du test avec son propre CRC et son propre Adler | égalité |
| V-IMG-06 | blocs stockés aux limites : `L` = 65534, 65535, 65536, 131070, 131071 → `[65534]`, `[65535]`, `[65535, 1]`, `[65535, 65535]`, `[65535, 65535, 1]` ; seul le dernier est final ; pas de bloc vide | RFC 1951 §3.2.4 (65535 octets au plus) ; la coupe en morceaux de 65535, dernier plein quand `L` est un multiple, est la convention de la doc de `ecrire_png` | égalité |
| V-IMG-07 | `ecrire_png` panique pour largeur ou hauteur nulle, ou longueur ≠ `3·l·h` | « panique » écrit dans la doc | `catch_unwind` |

## regles

| Identifiant | Ce qu'il fixe | Source |
|---|---|---|
| V-REG-01 | le détecteur de fautes voit `x.ln()`, `f64::exp(…)`, `<f64>::exp`, une fonction interdite déclarée dans un bloc `extern`, `link_name`, `HashMap`… et ne voit pas les commentaires, chaînes, identifiants voisins, champs (`self.log`), ni une fonction exportée `extern "C" fn` | textes écrits dans le test |
| V-REG-02 | aucun appel `f64` interdit, aucune `HashMap`/`HashSet`, aucune horloge ni fil dans `src/` | `AGENTS.md`, règle 2 |
| V-REG-03 | aucune dépendance, aucun paquet externe dans `Cargo.lock`, pas de `.cargo/` | `AGENTS.md`, règles 1 et 8 |
