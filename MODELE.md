# Physarum Lab : le modèle

Ce document nomme, pour chaque mécanisme, sa référence publiée, ses choix et ses limites. Une section par étape ; chaque
section finit par ce que ses tests n'établissent pas. Ce qui est à nous, sans article derrière, y est dit comme tel.

## Étape 0 : le socle (générateur, graphe, solveur, empreinte, image)

Rien ne simule encore de blob : l'étape pose ce dont toutes les suivantes se servent et qui doit être identique, au bit
près, sous Linux et sous Windows. Les valeurs attendues de ses tests, chacune avec sa source, sont dans
`references/etape-0-valeurs.md`.

### Le générateur pseudo-aléatoire : `alea`

PCG32 (variante XSH-RR 64/32) de Melissa O'Neill, écrit à l'identique du code C de référence `pcg_basic.c`
(pcg-random.org). `Alea::avec_flux(graine, flux)` est exactement `pcg32_srandom_r` de la référence ; `Alea::new(graine)`
prend le flux 54. Les sorties de la graine 42 et du flux 54 sont celles de la démonstration officielle (`pcg32-demo`).
`borne(n)` suit `pcg32_boundedrand_r` (tirage sans biais par rejet).

### Le réseau : `graphe`

Le blob est un réseau de tubes : un graphe non orienté, sans boucle ni arête double. Un nœud a une position entière et
un étage ; une arête a une longueur (ce qu'on mesure et qu'on dessine) et un coût (ce que le flux paiera pour la
traverser). Les arêtes sont rangées dans un ordre canonique (`a < b`, triées par `(a, b)`) que le solveur, l'empreinte
et le rendu suivent tous : c'est une convention de ce dépôt, pas une valeur publiée.

La grille a un ou plusieurs étages. Dans un étage, chaque nœud est relié à ses quatre voisins (longueur et coût 1) ou
à ses huit voisins (les diagonales ont pour longueur et pour coût `√2`, la racine carrée d'IEEE 754, la même partout).
Entre deux étages qui se suivent, chaque case a un lien vertical, dont la longueur et le coût se règlent à part : à
l'étape 1, c'est ce coût qui décidera où le réseau garde des puits.

### Le flux de Kirchhoff : `solveur`

Chaque tube a une conductance ; le flux d'un tube est sa conductance fois la différence de pression entre ses deux
bouts (la loi d'Ohm), et en chaque nœud ce qui sort égale ce qui est injecté (la loi de Kirchhoff). Un nœud, la masse,
a la pression zéro. Le système est résolu par une factorisation `L · D · Lᵀ` sans pivotage, les nœuds pris dans l'ordre
de leurs indices. Comme une somme de flottants dépend de son ordre, la doc de `solveur` fixe chaque opération : le
résultat est celui d'une procédure de référence, au bit près, et la mise en œuvre ne fait que sauter les zéros de la
matrice. C'est ce qui rend l'état des étapes suivantes identique d'une machine à l'autre sans aucune fonction du
système.

Les tests le comparent à des réseaux dont la solution se calcule à la main ou en fractions exactes : une chaîne, le
pont de Wheatstone, et le cube de douze résistances égales (7/12 entre deux sommets voisins, 3/4 en diagonale d'une
face, 5/6 en grande diagonale). Ils vérifient les lois elles-mêmes sur des graphes tirés au hasard, et les bits contre
la procédure de référence écrite à part, dans le test et en Python.

### Le scénario, l'empreinte, l'image, le lancement

- Le scénario : un fichier d'une directive par ligne. L'étape 0 n'interprète que la grille (`grille`, `voisinage`,
  `etages`) ; les autres directives sont gardées dans l'ordre, pour les étapes suivantes, et `lancer` refuse celles
  qu'il ne comprend pas.
- L'empreinte : SHA-256 (NIST FIPS 180-4) sur l'écriture canonique de l'état, décrite octet par octet dans la doc de
  `lancer` et de `graphe`. Les vecteurs de test sont ceux du NIST (exemples de la norme et fichiers du CAVP).
- L'image : un écrivain PNG (spécification PNG du W3C ; zlib, RFC 1950 ; deflate, RFC 1951), RVB 8 bits, blocs deflate
  stockés, CRC-32 et Adler-32 écrits dans le dépôt ; sa sortie est relue par un décodeur indépendant écrit dans les
  tests.
- L'aperçu : les étages côte à côte, les nœuds, les arêtes, et en couleur les nœuds d'où part un lien vers un autre
  étage. Il est pour l'œil, jamais une preuve.
- Le lancement : `physarum lancer <scénario> --graine <n> --sortie <dossier>` écrit l'empreinte, les mesures en CSV et
  l'aperçu.

### Ce que cette étape n'établit pas

- **La précision du solveur a une limite, et elle est connue.** Quand un groupe de nœuds ne tient à la masse que par
  des conductances `r` fois plus petites que les autres, ses pressions ne sont justes qu'à environ `r × 2⁻⁵³` près : sur
  une chaîne de conductances `1, 10⁻¹⁰, 1`, l'erreur mesurée est de `8 × 10⁻⁸`, et avec `10⁻³⁰⁰` le calcul s'arrête sur un
  pivot nul. Le modèle de l'étape 1, où des tubes meurent, devra borner ce rapport lui-même.
- L'égalité entre Linux et Windows n'est établie que par la suite de tests, verte des deux côtés, et par la CI, qui
  rejoue l'empreinte de référence (`references/grille-graine-1.empreinte`) sous les deux systèmes : une empreinte figée
  prouve que rien n'a bougé, pas que c'est juste. La justesse de cette empreinte-ci vient d'ailleurs : elle a été
  calculée à part, avant le code.
- Les bits du solveur sont vérifiés sur des réseaux de moins de quarante nœuds ; sur une grille de 80 × 80, seules
  les lois de Kirchhoff et la durée le sont.
- La garde des sources (`tests/etape_0_regles.rs`) lit le texte de `src/` : elle ne voit ni une macro, ni un
  `include!`, ni une fonction du système atteinte sans que son nom soit écrit dans `src/`. Elle refuse le type
  `HashMap` en entier, ce qui est plus strict que la règle du dépôt.
- Le générateur n'est éprouvé que par ses vecteurs officiels et par `borne` ; son uniformité statistique n'est pas
  mesurée ici.
- Aucun lecteur d'images réel n'a ouvert un aperçu dans les tests : le PNG est relu par le décodeur du test.
- L'aperçu ne dessine que les arêtes entre nœuds voisins d'un même étage : une arête plus longue d'un graphe
  quelconque n'apparaît pas.
