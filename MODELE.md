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

## Étape 1 (le cœur) : les tubes qui s'adaptent, et le plus court chemin

Le blob y est un réseau de tubes dont la conductivité suit le flux qui les traverse. Les valeurs attendues des tests,
chacune avec sa source, sont dans `references/etape-1-valeurs.md`. Cette section couvre le cœur (`chemins`, `tubes`,
`mesures`, les directives `source`, `puits`, `pas` de `lancer`) ; le labyrinthe, les étages et les réseaux viennent
dans les parties suivantes de l'étape.

### Les plus courts chemins : `chemins`

Dijkstra sur les coûts (`Arete::cout`), écrit pour comparer, jamais pour simuler. La doc de `chemins` fixe la
procédure (le nœud de plus petite distance, le plus petit indice à égalité, une comparaison stricte pour garder le
premier parent) : les distances sont, bit pour bit, celles d'un Bellman-Ford naïf, et les parents sont ceux de la
procédure. Il sert de témoin : le plus court chemin est calculé ici, puis le réseau de tubes doit le retrouver.

### Le modèle : `tubes`

Le modèle de Tero, Kobayashi et Nakagaki (2007) : `dD/dt = f(|Q|) - D`, avec la loi linéaire `f(Q) = |Q|` du théorème
de Bonifaci, Mehlhorn et Varma (2012) et le pas d'Euler de Becchetti et al. (2013) : `D <- D + h (|Q| - D)`, avec
`0 < h < 1`. Le flux d'un tube est `Q = D (p_a - p_b) / cout` : c'est le **coût** que le flux paie, la longueur ne
sert qu'aux mesures et au dessin. Un flux `I` entre à la source et sort au puits (la masse du solveur). Les
conductivités de départ sont tirées dans `[0,5 ; 1,0[` (Tero 2007).

Ce qui est **à nous**, sans article derrière :

- le **plancher de conductivité** `PLANCHER = 1e-6` : un tube ne descend pas dessous. Sans lui, le rapport des
  conductances dépasserait `2^53` et le solveur s'arrêterait sur un pivot nul (voir l'étape 0). Le théorème vaut pour
  le modèle sans plancher : avec lui, un tube mort est tenu à `PLANCHER` et non à zéro, et il laisse passer un flux
  de l'ordre de `PLANCHER` ; les tubes du chemin sont à quelques `1e-6` de `I`, pas à `I` ;
- le **schéma explicite** (Euler) : Tero 2007 emploie un schéma semi-implicite dont le pas n'est pas publié ;
- la **loi linéaire seule** : la sigmoïde de Tero et al. (2010) a un exposant 1,8 et `powf` est interdit ; elle
  viendra avec `γ = 2` (partie « les réseaux ») et sera dite telle.

### Les directives et les sorties : `lancer`, `mesures`

`source <x> <y> <etage>`, `puits <x> <y> <etage>` et `pas <nombre> <h>`, toutes les trois ou aucune. L'état est
écrit sous l'étiquette `PHYSAR01` (le graphe, la source, le puits, le nombre de pas, les bits de `h`, les bits de
chaque conductivité finale, l'état du générateur après ses tirages). Les mesures sont une ligne par pas : la pression
de la source, le nombre de tubes vifs (`mesures::tubes_vifs` : conductivité `>= 0,5`) et leur longueur. L'aperçu
colore chaque tube selon `D / I` (un niveau de 0 à 255, un mélange entier) et les nœuds d'où part un lien entre
étages vers la teinte des puits.

Un calcul qui s'arrête n'est pas un plantage : si le solveur rend une erreur à un pas, `lancer` rend
`ErreurLancer::Calcul(pas)` et le binaire sort avec le code 1 et un message, sans rien écrire.

### Ce que cette étape n'établit pas

- **La convergence est constatée, pas démontrée.** Le théorème 2 de Bonifaci et al. est continu et sans plancher ; la
  preuve de Becchetti et al. pour un pas fini suppose des conductivités de départ égales à 1, des longueurs d'au
  moins 1 et un `h` petit. Les tests constatent la convergence de notre schéma sur 20 graphes au hasard, un graphe à
  deux étages et une grille, avec des tolérances mesurées sur d'autres graphes (voir la fiche). Ils ne disent rien
  des graphes de plus de 30 nœuds, ni de la vitesse de convergence, qui dépend du graphe : environ 500 pas à
  `h = 0,5` pour les plus lents des graphes au hasard, plus de 400 pour la grille de 16 × 16 du scénario d'exemple.
- **Plusieurs plus courts chemins** : le théorème ne montre que l'attirance vers l'ensemble des flux de coût minimal
  ; on teste cela (la conservation sur le sous-graphe des plus courts chemins), pas la convergence vers un point, et
  la répartition entre les chemins dépend de l'état initial.
- **L'effet du plancher** n'est mesuré que pour `1e-6` ; une autre valeur changerait les fuites, pas la conclusion.
- **Des conductances très étalées** (un lien entre étages d'un coût de `10^16`, ou de `10^-16`) arrêtent le calcul :
  `lancer` rend alors `Calcul(pas)`, et c'est testé. Mais le solveur ne s'arrête pas toujours : entre un rapport de
  `10^10` et le pivot nul, il peut rendre des pressions fausses sans le dire (voir l'étape 0). Aucun test ne mesure
  la justesse du modèle dans cette zone ; les scénarios du dépôt gardent des coûts entre 0,1 et 10.
- **L'aperçu est pour l'œil** : sa formule est fixée en entiers pour que les octets soient les mêmes partout, mais
  elle ne prouve rien du modèle.
- **L'égalité entre Linux et Windows** n'est établie que par la CI qui rejoue l'empreinte de référence
  (`references/chemin-graine-1.empreinte`) sous les deux systèmes ; elle prouve que rien n'a bougé, pas que c'est
  juste. Sa justesse vient du modèle Python `ref_tubes.py`, qui l'a calculée avant le code.

## Étape 1 (partie 1L) : le labyrinthe et les étages

Le blob y traverse un labyrinthe et choisit, entre deux étages, de monter ou de rester en bas. Rien de neuf dans la
dynamique : c'est celle du cœur (les tubes de la loi linéaire de Bonifaci) sur des graphes que le module `labyrinthe`
construit. Les valeurs attendues des tests, chacune avec sa source, sont dans `references/etape-1l-valeurs.md`.

### Les murs et les étages : `labyrinthe`

- **Un mur retire des cellules, pas des arêtes.** Un nœud sans arête ferait rendre `Isole` au solveur ; retirer la
  cellule entière (le nœud et toutes ses arêtes) laisse un graphe sain. Un mur est un rectangle de cellules d'un étage
  (directive `mur <x0> <y0> <x1> <y1> <etage>`, bornes comprises).
- **Seule la composante du puits est gardée** : une poche fermée par des murs, une cellule isolée, une impasse coupée du
  reste disparaissent avec leurs arêtes. Si la source n'est pas dans cette composante, c'est une erreur.
- **Les nœuds gardés sont renumérotés dans l'ordre croissant de leur indice de grille**, et les arêtes gardées restent
  dans l'ordre canonique de `Graphe::new` (le même ordre relatif que dans la grille complète : la renumérotation
  conserve l'ordre). La source et le puits de l'état écrit sont leurs indices dans ce graphe ; l'étiquette de l'état
  reste `PHYSAR01` (la forme ne change pas : le graphe y est écrit en entier, murs et coûts compris). Sans mur ni coût
  d'étage, le graphe, l'état et l'empreinte sont ceux d'avant, au bit près (testé sur les scénarios du cœur et de
  l'étape 0).
- **Le coût d'un étage** (directive `cout_etage <etage> <facteur>`) : le coût de chaque arête dont les deux bouts sont
  dans l'étage est multiplié par le facteur, dans `[0,1 ; 10]` ; la longueur ne change pas, un lien entre étages garde
  son coût vertical. Les bornes évitent un rapport de conductances au-delà de `10^10`, où le solveur se trompe sans le
  dire (voir l'étape 0 et le cœur) ; le coût vertical des scénarios du dépôt reste lui aussi dans `[0,1 ; 10]` (un test
  le vérifie sur les fichiers de `scenarios/`, `lancer` ne le refuse pas).
- **À nous, sans article derrière** : tout cela (les murs, la composante, le coût d'un étage, les formats des
  directives).

### Le labyrinthe de Nakagaki

Nakagaki, Yamada et Tóth (Nature 407, 2000, p. 470) font traverser un labyrinthe au plasmode. **Ce qui a été lu** : la
page imprimée 470 seule : quatre chemins possibles (α1, α2, β1, β2) dont les plus courts mesurent `41`, `33`, `44` et
`45 ± 1 mm` ; la route par α2, environ 22 % plus courte que celle par α1, est toujours choisie ; β1 et β2 portent « à
peu près le même nombre de tubes » parce que leur différence (environ 2 %) se perd dans les méandres du tube ; les
impasses se rétractent. **Ce qui n'a pas été lu** : le dessin du labyrinthe, la structure des routes (le texte ne la
décrit pas), le texte complet de l'éditeur. **Notre lecture** : le tableau de la Fig. 1d croise les α (lignes) et les β
(colonnes) ; nous lisons donc les routes **en série** : de la source à une jonction par α1 ou α2, puis de la jonction au
puits par β1 ou β2.

Le labyrinthe de `scenarios/labyrinthe.txt` est **un dessin à nous**, sur une grille 41 × 29 en voisinage 4, couloirs
d'une cellule, à **deux cellules par millimètre** : les quatre plus courts chemins mesurent exactement **82, 66, 88 et
90** cellules (à une cellule par millimètre, `44` et `45` ne pourraient pas relier les mêmes bouts : la parité d'une
grille l'interdit ; à deux, `88` et `90` le peuvent), plus trois impasses. La source est en `(2, 14)`, la jonction en
`(20, 14)` et le puits en `(38, 14)`. Les tests mesurent les quatre longueurs par `chemins::plus_courts` sur le graphe,
en coupant tour à tour les autres routes.

**Ce que la loi linéaire en fait** (graines 1 à 12, `h = 0,5`, tolérances mesurées sur 50 autres graines) :

- la route la plus courte, α2 puis β1 (154 cellules), est retenue **quelle que soit la graine** : à 150 pas, les tubes
  d'α1 sont exactement au plancher, ceux d'α2 à `>= 0,999` ; à 200 pas, les tubes vifs (`>= 0,5`) sont exactement ceux
  du plus court chemin de Dijkstra ; à 600 pas, la pression de la source est à `154,002` à `154,003`, par le haut
  (Bonifaci : le potentiel source-puits tend vers `L*`) ;
- α1 meurt vite (sa conductivité maximale passe sous `0,01` au pas 45 pour les 12 graines) ; **β1 et β2 meurent bien
  plus lentement** : β2 passe sous `0,01` entre les pas 408 et 415 (mesuré de 405 à 418 sur 50 graines), au moins huit
  fois plus tard. L'écart des longueurs est de 22 % pour α, de 2,3 % pour β : la différence entre deux routes d'une
  paire décroît au rythme de leur écart relatif ;
- **chez nous, sans bruit, β2 finit par mourir** (à 600 pas, `<= 0,004`), alors que dans l'article « à peu près le même
  nombre de tubes » se forment par β1 et β2, parce que l'écart de 2 % se perd dans l'erreur expérimentale. Au début,
  nous voyons la même chose (à 20 pas, β1 est entre `0,538` et `0,572` et β2 entre `0,428` et `0,462`, sur 50 graines).
  **Aucun chiffre de l'article n'est retrouvé** : il n'y en a pas pour ce temps-là ;
- les impasses meurent (leur conductivité tombe sous `0,01` au pas 7 et au plancher à 20 pas), comme l'article les
  montre se rétracter.

### Les puits entre deux étages

Deux étages, la source et le puits sur celui du bas, celui du haut moins cher (`cout_etage 1 f`, `f < 1`) : le plus
court chemin monte puis redescend si `2 c + L_haut < L_bas` (`c` le coût d'un lien vertical), sinon il reste en bas.
Pour une grille 21 × 3 et `f = 0,5` : `L_bas = 20`, `L_haut = 10`, le seuil est `c = 5`. Sur une suite de coûts
verticaux de `0,1` à `10` (la zone de `4` à `6` écartée : deux chemins à moins de 10 % l'un de l'autre convergent
lentement, et à l'égalité le théorème ne choisit pas), **le nombre de liens verticaux vifs après 300 pas ne croît jamais
quand `c` croît** : il vaut 2 (monter, redescendre) ou 0, comme le dit Dijkstra, et les arêtes vives sont exactement
celles du plus court chemin. C'est **la seule chose que la loi linéaire permet de dire des puits** : une seule route
survit (théorème 2 de Bonifaci et al.), donc deux puits ou aucun. **La loi sigmoïde de la partie « les réseaux » gardera
plusieurs routes** ; ces tests-ci n'en disent rien. Un pont (un mur sur tout un étage, la seule route passe par l'autre)
est aussi testé.

### Ce que cette partie n'établit pas

- **Le dessin de l'article.** Nous ne l'avons pas ; notre lecture « en série » est une déduction du tableau de la Fig.
  1d, pas une phrase du texte. Les longueurs 82, 66, 88 et 90 sont celles de notre dessin, (l'article donne `± 1 mm`,
  soit `± 2` cellules).
- **Aucun chiffre de dynamique de l'article n'est retrouvé** : il n'en publie pas. Les pas de mort, les rapports et les
  seuils sont des constats de notre schéma (explicite, avec plancher, sans bruit), avec des tolérances mesurées sur 50
  graines.
- **Le bruit.** L'article attribue l'égalité entre β1 et β2 à l'erreur expérimentale ; notre modèle n'a pas de bruit,
  donc β2 meurt.
- **Plusieurs routes ou plusieurs puits** : la loi linéaire n'en garde qu'une ; la partie « les réseaux » reprendra la
  question avec la loi sigmoïde.
- **Les coûts verticaux à moins de 10 % du seuil** et les égalités de coût : aucun test ne dit ce qui s'y passe.
- **La vitesse** : les nombres de pas sont ceux de ce dessin, de `h = 0,5` et de ces graines.
- **Les grands labyrinthes** : 342 nœuds au plus ici ; rien sur la précision du solveur à 10 000 nœuds.
- **Linux** : tout est joué sous Windows ; la CI rejouera l'empreinte de `references/labyrinthe-graine-1.empreinte`.
