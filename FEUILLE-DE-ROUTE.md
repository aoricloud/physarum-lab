# Physarum Lab — la feuille de route de la V0

Ce document est le plan de la V0. Ses étapes deviennent des plans de vagues, et il change par PR, comme le code.
Seules l'autonomie de Lydite et ses trois limites sont décidées par l'humain ; le reste est proposé, et se corrige en
route.

## Ce que c'est

Un simulateur libre du blob (*Physarum polycephalum*), en Rust, orienté recherche, avec un rendu qu'on prend plaisir à
regarder. Le blob est une seule cellule géante, sans cerveau, faite d'un réseau de tubes où circule son cytoplasme. Il
pousse, relie ses sources de nourriture par un réseau court et robuste, se souvient d'où la nourriture se trouvait,
apprend à franchir une substance qu'il évitait, et transmet cet apprentissage à un autre blob en fusionnant avec lui.
Les trois premières étapes de modèle refont des expériences publiées ; la quatrième est un modèle à nous, présenté
comme tel, avec des critères fixés d'avance.

C'est le troisième banc d'essai de Lydite, après Mycelium Lab et Givre Lab : trois bancs montrent si l'orchestrateur
tient d'un domaine à l'autre. Ce que les deux premiers ont appris s'applique dès le départ ; la leçon principale est
qu'un test faux fait tricher les mains (elles plient le code pour le contenter), d'où le juge des tests ci-dessous.

## Ce que la V0 est, et ce qu'elle n'est pas

- **Elle est** : un réseau de tubes sur un graphe (une grille, sur un ou plusieurs étages reliés par des liens
  verticaux, ou un graphe quelconque) ; le flux calculé exactement à chaque pas ; une graine fixe, donc même graine,
  même état ; des mesures en nombres ; des expériences publiées refaites ; une page web où l'on pose la nourriture et
  le sel et où l'on regarde.
- **Elle n'est pas encore** : les oscillations du cytoplasme, la croissance du front sur un milieu continu, la chimie
  de la cellule, la 3D libre. Tout cela attend une V0 qui tourne.

## Les principes

1. **Rust sans dépendance** : générateur pseudo-aléatoire, écrivain PNG, empreinte et solveur linéaire écrits dans le
   dépôt. La CI reste simple et rien n'est à auditer.
2. **Déterminisme strict sur toutes les machines** : même graine, même état et mêmes nombres, sous Linux comme sous
   Windows. Seulement les opérations exactes de `f64` (`+ - * /`, `sqrt`) : ni `pow`, ni `exp`, ni `sin`. La loi de
   renforcement est linéaire (`D' = |Q| − D`, la forme du théorème de Bonifaci, Mehlhorn et Varma), une décroissance
   s'écrit en produit, et le solveur parcourt ses nœuds dans un ordre fixe : l'ordre des opérations fait partie du
   résultat. La liste exacte est dans `AGENTS.md`.
3. **La preuve, c'est l'état et les nombres** : l'empreinte de l'état par scénario et par graine
   (`references/<scenario>.empreinte`) et des mesures avec une tolérance. Les aperçus (`apercus/`) sont pour l'œil :
   ils sont commités pour se voir dans le diff de la PR, y compris sur un téléphone, et ne sont pas une preuve.
4. **Chaque mécanisme arrive avec sa référence** (article, figure, équation) et sa reproduction en nombres. Une valeur
   publiée qu'on n'atteint pas se dit telle quelle dans `MODELE.md`, et ce qui est à nous y est dit comme tel, avec
   ses critères fixés d'avance : un texte sourcé se relit autant que le code.
5. **Les tests avant les mains, et un juge des tests** (voir plus bas).

## Les modules

| Module | Rôle |
|---|---|
| `alea` | PCG32, la graine |
| `graphe` | les nœuds (position entière, étage), les arêtes (longueur et coût), les grilles à un ou plusieurs étages |
| `solveur` | le flux de Kirchhoff : les pressions aux nœuds et le flux dans chaque arête, dans un ordre fixe |
| `tubes` | le modèle de Tero : la conductance de chaque tube croît avec son flux et décroît sans lui |
| `chemins` | les plus courts chemins (Dijkstra) et l'arbre couvrant minimal, pour comparer |
| `mesures` | longueur du réseau, efficacité du transport, tenue à une coupure |
| `souvenir` | la trace qu'un stimulus laisse dans les diamètres des tubes (un agent porté par le flux) |
| `habituation` | le pont à franchir, la substance absorbée, l'aversion qui s'adoucit |
| `fusion` | deux réseaux qui fusionnent, et ce qui passe de l'un à l'autre |
| `vitrine` | les fonctions exportées vers la page web |
| `scenario`, `empreinte`, `image`, `rendu`, `lancer` | le fichier de scénario ; l'empreinte (SHA-256) sur une écriture canonique ; l'écrivain PNG ; les aperçus ; le lancer d'un scénario |
| binaire `physarum` | `physarum lancer <scénario> --graine <n> --sortie <dossier>` : l'empreinte, les mesures en CSV, les aperçus |

## Les étapes de la V0

Chaque étape a deux vagues : les tests, puis le code. Sa preuve est
`cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`. Les tolérances sont fixées par le juge
des tests, sur un prototype honnête.

| Étape | Ce qu'elle fait | Sa preuve | Ce qu'on voit |
|---|---|---|---|
| 0, le socle | `alea`, `graphe` (grilles et graphes, sur un ou plusieurs étages reliés par des liens verticaux, chacun avec sa longueur et son coût), `solveur` (le flux de Kirchhoff, ordre fixe), `empreinte`, `image`, `scenario`, le binaire | même graine, même empreinte sous les deux systèmes ; le solveur exact sur des réseaux dont on connaît les pressions à la main ; les lois de Kirchhoff tenues sur des graphes tirés au hasard | la grille seule, ses étages côte à côte |
| 1, il pousse | le modèle de Tero : le flux dans les tubes, les tubes qui s'épaississent avec lui et s'amincissent sans lui | **exacte** : avec un seul plus court chemin, le réseau converge vers lui (théorème de Bonifaci, Mehlhorn et Varma), contrôlé par un Dijkstra sur des graphes tirés au hasard ; le labyrinthe de Nakagaki retrouvé ; plusieurs sources de nourriture : longueur, efficacité et tenue à une coupure comparées à l'arbre couvrant minimal, comme Tero et al. (2010), en esprit ; **sur plusieurs étages** : le théorème vaut pour tout graphe, donc le plus court chemin à travers les étages reste un test exact, et le coût d'un lien vertical décide où naissent les puits (moins il coûte, plus il y en a) | le réseau qui se resserre sur le chemin ; le labyrinthe ; un réseau entre des villes ; les étages côte à côte, avec leurs puits |
| 2, il se souvient | le stimulus libère un agent « assouplissant » porté par le flux ; les tubes qui en reçoivent grossissent aux dépens des autres (Kramar et Alim, 2021) | la trace se forme du côté du stimulus et y reste après son retrait ; la migration suivante part vers lui ; deux stimuli se renforcent ou s'effacent | les tubes épais qui pointent vers l'ancienne nourriture |
| 3, il apprend | le pont à franchir : une substance absorbée en le traversant, qui adoucit l'aversion (la « mémoire qui circule » de Boussard et al., 2019) | cinq jours d'entraînement, un passage par jour : le temps d'entrée sur le pont baisse ; deux jours de repos le font remonter ; un blob habitué à une substance reste averse à une autre (Boisseau, Vogel et Dussutour, 2016) | la courbe du temps de passage, jour par jour |
| 4, il transmet (à nous) | deux réseaux qui fusionnent ; le cytoplasme se mélange par les veines, la substance passe de l'un à l'autre | les critères de l'expérience de Vogel et Dussutour (2016) : une heure de fusion ne transmet rien, trois heures transmettent un comportement proche de celui d'un blob habitué ; le modèle le dit d'avance | les deux blobs qui fusionnent, puis le naïf qui franchit le pont |
| 5, la vitrine | la page web : `wasm32-unknown-unknown` sans dépendance, quelques fonctions exportées, un canvas ; on pose la nourriture et le sel, on regarde | la page charge, et l'empreinte d'une graine y est la même qu'en natif | la page elle-même |

La vitrine se publie sur GitHub Pages une fois que l'humain l'a ouvert, son geste, une fois ; ensuite chaque fusion la
republie.

**Après la V0** : le blob qui redessine un réseau de métro, celui qui se répare quand on coupe un tube, et celui qui
calcule (de petits problèmes de tournée dont on connaît la meilleure réponse). Puis, plus loin : le blob architecte,
en trois dimensions, qui dessine des réseaux de stations sous terre ou de bases dans l'espace, avec leurs contraintes
(obstacles, coût de creuser, tenue à une panne).

## Ce qui serait neuf (à confirmer dans la littérature avant de l'écrire dans `MODELE.md`)

1. **La chaîne entière dans un seul simulateur** : pousser, se souvenir, apprendre, transmettre ; et le transfert par
   fusion, dont aucun modèle n'a été trouvé (recherche non exhaustive).
2. **Deux mémoires réunies** : celle de la forme du réseau (Kramar et Alim) et celle de la substance absorbée
   (Boussard et al.) ; le modèle dit laquelle porte le transfert.
3. **Des prédictions qu'un laboratoire peut tester**, annoncées comme telles : la courbe du transfert entre une et
   trois heures de fusion, le nombre de blobs naïfs qu'un seul blob formé peut instruire, le rôle de la forme du réseau
   (arbre ou maillage) dans l'apprentissage.
4. **Une science rejouable** : chaque résultat se rejoue à l'identique, sur toute machine, à partir d'une graine.

## Les tests avant les mains

Chaque étape a deux vagues. La première écrit les signatures publiques de l'étape (des corps qui paniquent) et ses
tests d'acceptation, et chaque test porte `#[cfg_attr(not(feature = "etape-<n>"), ignore)]`. Les fonctionnalités
`etape-0` à `etape-5` et les modules de `src/` sont déclarés d'avance (`Cargo.toml`, `src/lib.rs`, des fichiers de
module presque vides) : une vague de tests ne remplace que des fichiers presque vides ou neufs, et deux vagues n'éditent jamais la même ligne. Elle est fusionnée comme les autres. La
seconde remplit les corps et ajoute `etape-<n>` à `default` (la seule ligne de `Cargo.toml` qu'une vague de code
touche), ce qui lève les `ignore` sans toucher aux tests : ses tâches n'ont pas les fichiers de tests dans leurs
`fichiers`, et Lydite refuse une main qui y touche.

Une exception : quand une étape étend un module déjà rempli (`lancer`, le binaire), sa vague de tests y porte la doc
et les types publics nouveaux sans toucher au code, et sa vague de code le remplit ensuite.

**Le juge des tests** passe avant que les tests soient gelés, parce qu'un test faux ou trop faible fait tricher les
mains, ou les laisse passer à vide. Un agent plus fort que les mains vérifie trois choses :

1. **un code nul échoue** : des corps qui rendent zéro ne passent aucun test de comportement ;
2. **un prototype honnête passe, sur plusieurs graines** : une implémentation jetable, fidèle au modèle, passe tous les
   tests, sur une série de graines et pas sur une seule ; et quand la doc d'un module fixe le résultat au bit près, un
   second prototype écrit d'après la seule doc donne les mêmes bits ;
3. **chaque tolérance est mesurée sur ce prototype, et chaque référence est relue dans son article** : une valeur
   publiée n'entre jamais d'après la mémoire de quelqu'un.

**Deux fichiers par cas, et des étiquettes.** Pour chaque étape, `references/etape-<n>-valeurs.md` donne, avant que le
code existe, la référence de chaque valeur attendue : la valeur, sa source (article, page, figure, tableau, équation,
ou un calcul exact écrit à côté), la tolérance et sa raison, et le critère de réussite. Les résultats de notre code
(`references/*.empreinte`) sont un autre fichier, figé seulement une fois que le cas passe contre sa référence. Le juge
refuse une valeur attendue sans source ou tirée de notre propre code, et on n'élargit jamais une tolérance pour faire
passer un cas. Chaque test est étiqueté `juste` (un calcul exact, un simulateur naïf écrit dans le test, une valeur
publiée) ou `régression` (un résultat figé de notre code), et dit ce qu'il ne couvre pas ; seuls les tests `juste`
sont une preuve. Chaque étape prévoit des cas à solution connue, et finit par une section « Ce que cela n'établit
pas » dans `MODELE.md`.

Une propriété fausse, ou seulement vraie en moyenne, n'entre jamais telle quelle : une propriété moyenne se teste en
moyenne, sur assez de graines, avec une tolérance mesurée. Les fichiers de tests que le juge a éprouvés sont ceux qui
sont fusionnés, à l'octet.

## L'autonomie de Lydite

Lydite mène Physarum Lab en autonomie, dans trois limites décidées par l'humain :
1. la preuve et le mur restent : Lydite rejoue lui-même chaque preuve, et les mains travaillent derrière le mur ;
2. ce seul dépôt : un jeton limité à Physarum Lab ; les réglages du dépôt (règles, vérifications exigées, auto-fusion,
   Pages) restent à l'humain, et Lydite ne peut pas lever ses propres garde-fous (il ne pousse jamais `.github/`, n'a
   aucun droit d'administration) ;
3. seulement les issues de l'humain ou celles de Lydite : sur un dépôt public, le texte d'un inconnu ne devient jamais
   la consigne d'une main.

Le reste est libre : écrire ses plans, choisir l'ordre, enchaîner.

**La fusion, sans une ligne de Lydite** : l'auto-fusion du dépôt est permise, et la règle de `main` exige la CI (Linux
et Windows) sans contournement possible. `auto-fusion.yml` demande l'auto-fusion des seules PR des branches `lydite/`
du dépôt vers `main`, déclenché par `pull_request` (jamais `pull_request_target`), et seulement quand le corps de la PR
dit que Lydite a rejoué la preuve sur la tête de la PR ; GitHub fusionne quand tout est vert. La même CI refuse qu'une
dépendance ou un `.cargo/` entre.

## Les règles que les mains ne lisent pas toutes

Chaque tâche d'un plan redit les règles qui comptent pour elle : le déterminisme (aucune fonction mathématique du
système, l'ordre des opérations, aucune `HashMap` parcourue), les tests qu'on ne touche pas, le « sans dépendance »,
les fichiers qu'elle seule touche. La CI sous Linux et sous Windows attrape un état qui différerait d'un système à
l'autre : l'empreinte de référence n'y serait plus la même.

## Les références à lire

Chacune se vérifie sur l'article avant qu'un test la fige.

- T. Nakagaki, H. Yamada et Á. Tóth, *Maze-solving by an amoeboid organism*, Nature 407, 470 (2000),
  doi:10.1038/35035159.
- A. Tero, R. Kobayashi et T. Nakagaki, *A mathematical model for adaptive transport network in path finding by true
  slime mold*, J. Theor. Biol. 244, 553-564 (2007), doi:10.1016/j.jtbi.2006.07.015 : le tube s'épaissit avec le flux.
- A. Tero et al., *Rules for biologically inspired adaptive network design*, Science 327, 439-442 (2010),
  doi:10.1126/science.1177894 : un réseau d'efficacité, de tenue aux pannes et de coût comparables à ceux d'un réseau
  ferré.
- V. Bonifaci, K. Mehlhorn et G. Varma, *Physarum can compute shortest paths*, J. Theor. Biol. 309, 121-133 (2012),
  doi:10.1016/j.jtbi.2012.06.017 (arXiv:1106.0423) : la convergence vers le plus court chemin quand il est unique,
  pour `D' = |Q| − D`.
- M. Kramar et K. Alim, *Encoding memory in tube diameter hierarchy of living flow network*, PNAS 118, e2007815118
  (2021), doi:10.1073/pnas.2007815118.
- R. P. Boisseau, D. Vogel et A. Dussutour, *Habituation in non-neural organisms: evidence from slime moulds*, Proc. R.
  Soc. B 283, 20160446 (2016), doi:10.1098/rspb.2016.0446.
- D. Vogel et A. Dussutour, *Direct transfer of learned behaviour via cell fusion in non-neural organisms*, Proc. R.
  Soc. B 283, 20162382 (2016), doi:10.1098/rspb.2016.2382.
- A. Boussard, J. Delescluse, A. Pérez-Escudero et A. Dussutour, *Memory inception and preservation in slime moulds:
  the quest for a common mechanism*, Phil. Trans. R. Soc. B 374, 20180368 (2019), doi:10.1098/rstb.2018.0368.
- Ce qui existe déjà et se lit avant l'étape 4 : un modèle de l'habituation (Nishi, Tero, Nishigami et Nakagaki,
  prépublication bioRxiv, 2025, doi:10.1101/2025.09.15.675989), la capacité de mémoire des réseaux adaptatifs
  (Bhattacharyya, Zwicker et Alim, Phys. Rev. E 107, 034407, 2023), un modèle de la fusion sans comportement (Hsu et
  Schaposnik, arXiv:2106.11371).

## Les gestes de l'humain

Une fois : le dépôt, public, sous MIT ; la règle de `main` et l'auto-fusion ; un jeton limité à lui ; la première PR
(la CI et `auto-fusion.yml`). Reste : ouvrir Pages pour la vitrine. Plus aucune fusion ensuite, sauf une PR que la
garde de fusion laisse à l'humain ; son œil revient après coup.
