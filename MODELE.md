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

## Étape 1 (partie 1R) : les réseaux entre plusieurs sources de nourriture

Le blob y relie plusieurs sources de nourriture : à chaque pas, une paire tirée au hasard est la source et le puits d'un
débit, et les tubes s'adaptent. Rien de neuf dans le solveur ni dans le pas d'Euler du cœur ; ce qui change est la loi
de renforcement (la sigmoïde de Tero et al. 2010), le tirage de la paire, l'arbre couvrant minimal et les trois mesures.
Les valeurs attendues des tests, chacune avec sa source, sont dans `references/etape-1r-valeurs.md`.

### La loi, le pas et le tirage : `reseaux`

- **Référence** : Tero, Takagi, Saigusa, Ito, Bebber, Fricker, Yumiki, Kobayashi et Nakagaki, « Rules for Biologically
  Inspired Adaptive Network Design » (Science 327, 2010, p. 439-441 et supplément, lus) : à chaque pas, une source de
  nourriture tirée au hasard fait circuler le flux `I0`, une seconde, tirée au hasard, est le puits ; `dD/dt = f(|Q|) -
  D` avec `f(|Q|) = |Q|^γ / (1 + |Q|^γ)` ; une simulation « typique » prend `I0 = 2` et `γ = 1,8` (Fig. 4) ; la Fig. 3A
  fait varier `I0` de 0,2 à 7,2 à `γ = 1,8`.
- **`γ = 2`, pas `1,8`** : `powf` est interdit (les fonctions du système peuvent différer d'une machine à l'autre) ;
  `Q^2` s'écrit `Q * Q`. La loi d'ici est `f(Q) = Q·Q / (1 + Q·Q)`, qui n'est **pas** celle de l'article. La loi
  linéaire du cœur (`f(Q) = |Q|`) reste disponible (`Loi::Lineaire`) ; avec elle, un pas est, bit pour bit, celui de
  `Tubes::avancer`.
- **À nous, l'article ne les donne pas** : le maillage, le nombre de nœuds, le pas de temps, le schéma d'intégration
  (nous gardons le pas d'Euler du cœur, `0 < h < 1`), les conductivités de départ (celles du cœur : `0,5 + 0,5 u`), et
  **la façon de tirer la paire** (deux tirages toujours : `i = borne(k)`, `j = borne(k - 1)`, puis `j + 1` si `j >= i`).
- Les conductivités vivent dans un `Reseau` à nous (pas dans un `Tubes` reconstruit à chaque pas : un `Reseau` garde ses
  conductivités, ses nourritures, son débit et sa loi). Sur une erreur, rien ne change, générateur compris.

### L'arbre couvrant minimal et les trois mesures

- **Référence** : Tero et al. 2010 (p. 439-440, Fig. 3 et supplément, lus) : TL « longueur totale », MD « distance
  minimale moyenne entre les paires de sources de nourriture », FT « la probabilité de déconnecter une partie du réseau
  lors de la défaillance d'un seul lien », chacune normalisée par celle de l'arbre couvrant minimal. **Aucune formule
  n'est publiée** : celles d'ici sont à nous.
- **L'arbre** : celui de Kruskal sur la clôture métrique des nourritures (la distance de deux nourritures est la
  longueur du plus court chemin du graphe, par les **longueurs**), les égalités au plus petit `(distance, i, j)` ;
  `TL_MST` est la somme de ses distances, `MD_MST` la distance moyenne dans l'arbre sur les paires de nourritures.
- **TL** : la longueur totale des tubes vifs. **MD** : la distance moyenne, sur les paires de nourritures, dans le
  réseau des tubes vifs (`+inf` si une nourriture est coupée). **FT** : **notre lecture** de la probabilité de
  l'article, en **un moins** cette probabilité (la note de lecture dit que c'est une déduction, d'après les valeurs du
  supplément : 0,00 pour un réseau proche de l'arbre, 0,96 pour le réseau ferroviaire) : la part des tubes vifs dont la
  panne seule ne coupe aucune nourriture d'une autre (une branche morte ne coupe rien). Les rapports sont `TL / TL_MST`
  et `MD / MD_MST`.
- **Un tube est vif** à partir de `0,001`, seuil absolu (`reseaux::SEUIL_VIF`). Sous la sigmoïde, un tube qui ne porte
  plus de flux est tenu au plancher `1e-6` et un tube vivant reste à `0,0125` ou plus (mesuré : 35 conductivités sur 68
  400 entre les deux) : le seuil tombe dans un trou. Sous la loi linéaire il n'y a pas de trou : le seuil y est une
  convention.

### Les directives et les sorties : `lancer`

`nourriture <x> <y> <etage>` (de deux à seize fois, nœuds différents), `debit <I0>` (dans `[0,05 ; 20]`), `loi
lineaire|sigmoide` et `pas <nombre> <h>`, toutes les quatre ; elles excluent `source` et `puits` ; `mur` et `cout_etage`
de la partie 1L restent permis (les murs retirent des cellules : toutes les nourritures doivent rester dans la
composante de la deuxième). L'état est écrit sous l'étiquette `PHYSAR02` (le graphe, les nœuds de nourriture par rang,
le débit, la loi, le nombre de pas, `h`, les conductivités finales, le générateur). Les mesures sont une ligne par pas :
la paire tirée (les rangs), la pression de la source, le nombre de tubes vifs et leur longueur (TL). L'aperçu est celui
des tubes, la référence étant `f(I0)`, avec les nourritures en rouge. Les empreintes des scénarios de l'étape 0, du cœur
et de la partie 1L ne bougent pas (testé).

### Ce que la sigmoïde fait, « en esprit »

Sur une grille de 10 × 10 à huit voisins, six nourritures, 800 pas à `h = 0,1`, loi sigmoïde, mesuré sur 40 graines
autres que celles des tests : `TL / TL_MST` vaut en moyenne 1,03, 1,24, 1,55, 1,86, 3,14 pour `I0 = 1, 2, 3, 4, 8` ;
`FT` vaut 0,00, 0,53, 0,91, 0,99, 1,00 ; `MD / MD_MST` vaut 1,06, 0,98, 0,95, 0,91, 0,85. Pour **chaque graine**, `TL`
et `FT` ne baissent jamais quand `I0` croît (la plus petite différence est 0 exactement). À `I0 = 1` le réseau est
presque l'arbre couvrant minimal : `FT = 0` pour les 40 graines, `TL / TL_MST` de 0,98 à 1,12. C'est le sens que dit
l'article (plus de routes de rechange, une meilleure tenue, un coût plus élevé quand `I0` croît), **aucun chiffre de
l'article n'est retrouvé** : ni son maillage, ni son pas de temps, ni ses mesures ne sont publiés assez pour cela, et
nos `I0` ne sont pas les siens (`γ` diffère). À `I0 = 0,05`, le réseau se défait (une nourriture coupée : `MD` infini).

**Sous la loi linéaire, le contraire de ce qu'on attendait** : à faible `I0` (0,05 à 2), `FT = 1` pour chacune des 40
graines et `TL / TL_MST` de 3,1 à 7,7 : le réseau reste épais, il ne tend pas vers un arbre. L'explication probable (non
vérifiée) est que la loi linéaire sélectionne le plus court chemin de chaque paire (théorème de Bonifaci et al.), pas
l'arbre des nourritures, et que les tubes meurent lentement ; avec des longueurs aléatoires, `FT` reste à 1 et `TL /
TL_MST` baisse encore de 5,3 à 4,0 entre 800 et 3 000 pas. Dans notre schéma, la tendance vers l'arbre à faible `I0` est
un effet de la sigmoïde.

### Ce que cette partie n'établit pas

- **Aucune valeur de l'article n'est retrouvée** : ni `TL_MST ≈ 1,75`, ni `MD_MST = 0,85`, ni les 14 % de pannes qui
  déconnectent ; ni `γ = 1,8`, ni `I0 = 2` à son échelle. Le maillage, le pas, les conductivités de départ et les
  formules des mesures sont les nôtres.
- **FT est notre lecture** d'une probabilité que l'article ne définit qu'en mots, comptée sur les nourritures et sur les
  tubes vifs ; l'article la mesure sur le réseau entier, avec une déconnexion « d'une partie du réseau ».
- **Les tendances sont constatées, pas démontrées**, sur une grille de 10 × 10 à huit voisins, six nourritures, 800 pas,
  `h = 0,1` et le seuil `0,001` ; rien sur d'autres maillages, d'autres nombres de nourritures, d'autres `γ`, ni sur la
  vitesse (le réseau fluctue à chaque pas : on mesure l'état au pas 800).
- **Le tirage de la paire est notre choix** (uniforme parmi les paires ordonnées) ; l'article dit « tirée au hasard »
  sans dire comment. Les fréquences testées le sont à un seuil mesuré (`chi²`), pas démontrées.
- **L'arbre double compte les arêtes partagées** : `TL_MST` additionne des distances de la clôture métrique, qui peuvent
  partager des arêtes du graphe ; `TL / TL_MST` peut donc être sous 1 (mesuré : 0,98).
- **La loi linéaire ne tend pas vers un arbre** dans nos mesures (voir plus haut) : l'attente de départ du découpage est
  fausse ; on ne sait pas encore pourquoi, exactement.
- **Les conductances très étalées** et les mêmes limites du solveur qu'au cœur (`10^10` à `10^16`) ; **Linux** : tout
  est joué sous Windows, la CI rejouera l'empreinte de `references/reseau-graine-1.empreinte`.
