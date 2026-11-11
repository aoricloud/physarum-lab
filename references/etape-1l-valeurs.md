# Étape 1, partie 1L (le labyrinthe et les étages) : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de la partie 1L attend, d'où elle vient, avec quelle tolérance et quel
critère de réussite. Chaque identifiant (`V-LAB-01`, `V-NAK-01`, `V-ETA-01`, `V-LLAB-01`, `V-APL-01`…) est cité dans la
doc du test qui l'emploie (`tests/etape_1l_*.rs`). Elle est séparée des résultats figés de notre propre code
(`references/*.empreinte`, étiquetés « régression » dans les tests) : **aucune valeur de cette fiche ne vient de notre
code**. Une valeur y vient d'une publication lue, d'un calcul à la main ou par un outil indépendant (nommé), d'une
construction naïve écrite dans le test, ou d'une convention que la documentation d'un module fixe (dite telle).

Écrite avant les calculs : les critères d'abord, les mesures ensuite. La suite complète de la partie (`cargo test
--features etape-1,etape-1l`) dure 17 s avec les 147 tests d'avant.

## Les outils indépendants

Python 3.13 (flottants IEEE 754, `struct`, `hashlib`) ; le code Rust n'est jamais l'outil. Gardés avec les notes de la
session qui a écrit ces tests, dans `outils-1L/` :

- `ref_alea.py`, `ref_graphe.py`, `ref_solveur.py`, `ref_tubes.py` : repris du cœur (PCG32, la grille, le solveur, le
  pas du modèle, l'écriture canonique, l'aperçu coloré) ;
- `ref_labyrinthe.py` : écrit d'après la seule doc de `labyrinthe` et de `lancer` : la construction du plan (murs,
  composante du puits, renumérotation, coût d'un étage), les erreurs dans leur ordre, l'état avec murs, l'empreinte, les
  mesures, l'aperçu ;
- `maze_nakagaki.py` : dessine le labyrinthe, vérifie ses longueurs par un parcours en largeur, écrit les directives
  `mur` ; `scenarios_1l.py` : écrit les quatre scénarios ; `valeurs_1l.py` et `valeurs_morts_1l.py` : impriment les
  valeurs écrites dans les tests (empreintes, mesures, aperçus, pas de mort) ; `mesures/` : les quatre programmes de
  mesure des tolérances (ils s'appuient sur `labyrinthe::construire`) ; `mutants_1l.py` : le banc de fautes.

Dans les tests, trois constructions naïves jouent le même rôle : la construction du plan de `etape_1l_construction.rs`
(une relaxation jusqu'à stabilité à la place du parcours, la géométrie des voisins à la place de `Grille::graphe`), le
dessin en caractères du labyrinthe (`etape_1l_labyrinthe.rs`) et les dessins des petits aperçus (`etape_1l_apercus.rs`).

## Les sources lues (et ce qui ne l'est pas)

Les lectures sont celles de `references-etapes-0-1.md` (notes du pilote) ; aucune valeur n'est ajoutée de mémoire.

- **Nakagaki, Yamada, Tóth 2000**, Nature 407, 470 : lu **la page imprimée 470 seule**, retrouvée dans un PDF de Science
  in School, texte extrait avec des signes dégradés (α et β rétablis par le sens). Lu : quatre chemins possibles (α1,
  α2, β1, β2) entre les deux blocs d'agar ; leurs plus courts chemins `α1 41 ± 1 mm`, `α2 33 ± 1 mm`, `β1 44 ± 1 mm`,
  `β2 45 ± 1 mm` ; le chemin par α2, « environ 22 % plus court » que celui par α1, est toujours choisi ; « à peu près le
  même nombre de tubes » par β1 et β2, car leur différence de longueur (environ 2 %) se perd dans les méandres du tube
  et reste dans l'erreur expérimentale ; 4 h après la pose les impasses se rétractent (Fig. 1b), 4 h plus tard le plus
  court chemin est sélectionné (Fig. 1c). **Non lu** : le dessin du labyrinthe (Fig. 1a, image) ; la structure des
  routes (le texte ne la décrit pas) ; le texte complet de l'éditeur, les informations supplémentaires ; le tableau de
  la Fig. 1d n'est lu que sur l'extraction (19 essais : somme du tableau, non écrite dans le texte). **Notre lecture,
  écrite comme telle** : les quatre routes sont lues en série (de la source à une jonction par α1 ou α2, puis de la
  jonction au puits par β1 ou β2), parce que le tableau de la Fig. 1d croise les α (lignes) et les β (colonnes) ; ce
  n'est pas dit dans le texte. Remarque : `33 / 41` donne 20 %, pas 22 % ; notre dessin a `66 / 82`, soit 19,5 %.
- **Bonifaci, Mehlhorn, Varma 2012** et **Becchetti et al. 2013**, **Tero, Kobayashi, Nakagaki 2007** : voir
  `references/etape-1-valeurs.md` (cœur) : le théorème 2 (la dynamique linéaire est attirée par les flux de coût
  minimal, une seule route survit quand le plus court chemin est unique) est ce que les tests des puits constatent.
- **Ce qui est à nous** (aucun article) : le dessin du labyrinthe ; la lecture en série ; deux cellules par millimètre ;
  le coût d'un étage (un facteur sur les arêtes d'un étage) ; les murs comme cellules retirées ; la composante du puits
  seule gardée ; les bornes `[0,1 ; 10]` des facteurs ; les formats des directives `mur` et `cout_etage`.

---

## construction

Source : la doc de `labyrinthe` ; des graphes écrits à la main ; la construction naïve du test. Tolérance : **aucune**
(entiers, égalités, bits des longueurs et des coûts).

- **V-LAB-01** : sans mur ni coût d'étage, le plan est `Grille::graphe` (mêmes nœuds, mêmes arêtes, mêmes octets), la
  correspondance est l'identité, la source et le puits gardent leurs indices ; un facteur `1.0` explicite ne change pas
  un bit. Quatre grilles, deux sens. Source : doc. Critère : égalité.
- **V-LAB-02** : un mur d'une cellule au centre d'une grille 3 × 3 : huit nœuds renumérotés `0 1 2 3 5 6 7 8 → 0 … 7`,
  correspondance `[0, 1, 2, 3, None, 4, 5, 6, 7]`, arêtes grille `(0,1) (0,3) (1,2) (2,5) (3,6) (5,8) (6,7) (7,8)` →
  plan `(0,1) (0,3) (1,2) (2,4) (3,5) (4,7) (5,6) (6,7)`, même ordre relatif. Source : calcul à la main. Critère :
  égalité.
- **V-LAB-03** : la poche fermée (une cellule libre au centre d'un anneau muré dans une grille 5 × 5) disparaît : 16
  nœuds et 16 arêtes, un cycle ; une impasse libre reste ; une cellule isolée dans un coin disparaît. Source : calcul à
  la main. Critère : égalité.
- **V-LAB-04** : deux murs qui se recouvrent = l'union ; même plan avec un seul mur, un mur doublé, l'ordre inversé ;
  graphe de 7 nœuds et 6 arêtes écrit à la main. Source : calcul à la main. Critère : égalité.
- **V-LAB-05** : un mur sur tout un étage : il reste l'étage 0 (6 nœuds, 7 arêtes), les liens verticaux ont disparu ;
  `PuitsMure` et `SourceMuree` quand le mur est sur l'étage du puits ou de la source. Source : calcul à la main.
- **V-LAB-06** : la source et le puits de chaque côté d'un mur avec un passage : 13 nœuds, 16 arêtes (22 − 6) ; le plus
  court chemin passe sous le mur, 4 arêtes, de longueur 4. Source : calcul à la main (dénombrement des arêtes touchées).
- **V-LAB-07** : le coût d'un étage : une arête de chaque sorte (orthogonale de l'étage 0 inchangée, orthogonale de
  l'étage 1 multipliée, diagonale `√2` multipliée en un seul produit, lien vertical inchangé), longueurs inchangées ;
  comptes d'arêtes 7 + 7 orthogonales, 4 + 4 diagonales, 6 verticales ; l'ordre des facteurs ne compte pas ; un étage
  sans facteur garde ses coûts (trois étages, un facteur). Source : doc ; arithmétique IEEE 754. Critère : égalité des
  bits.
- **V-LAB-08** : murs et facteurs ensemble sur une grille 4 × 1 à deux étages : 7 nœuds, 7 arêtes écrits à la main (dont
  `(4,6)` à coût 0,5). Source : calcul à la main. Critère : égalité des bits.
- **V-LAB-09** : les erreurs et leur ordre : `Grille`, `Mur(i)` (cinq fautes, rang le plus petit, valeurs énormes),
  `Facteur(i)` (étage hors grille, `0.09`, `10.5`, NaN, infinis, `0`, `-1`, étage double ; bornes `0.1` et `10.0`
  acceptées), `Noeud`, `PuitsMure`, `SourceMuree`, `SourceCoupee`, et leur priorité deux à deux. Source : doc. Critère :
  égalité.
- **V-LAB-10** : 800 grilles tirées au hasard contre la construction naïve du test (nœuds, arêtes aux bits,
  correspondance, source et puits, ou même erreur au même rang). Source : construction naïve dans le test. Mesure : 216
  plans réussis dont 49 avec une cellule libre retirée ; erreurs `Mur` 58, `Facteur` 52, `Noeud` 127, `PuitsMure` 224,
  `SourceMuree` 84, `SourceCoupee` 39. Exigences : 150, 30, 20 au moins.
- **V-LAB-11** : l'ordre : sur 300 grilles au hasard, la `k`-ième arête du plan est la `k`-ième arête gardée de la
  grille complète, les numéros croissent `0, 1, 2…`, les positions sont celles de la grille ; au moins 60 des plans
  vérifiés sont troués (mesuré : 91 sur 165). Source : doc. Critère : égalité.

## le labyrinthe

Sources : Nakagaki et al. 2000 p. 470 pour les quatre longueurs ; le modèle Python `ref_labyrinthe.py` pour les pas de
mort exacts ; des tolérances **mesurées** sur 50 graines (1000 à 1049), autres que celles du test (1 à 12).

- **V-NAK-01** : le dessin du test est celui de `scenarios/labyrinthe.txt` : cellules murées = union des 52 rectangles
  `mur`, grille 41 × 29 en voisinage 4, source `(2, 14)`, puits `(38, 14)`, `pas 500 0.5` ; 342 couloirs ; le plan a 342
  nœuds et 343 arêtes. Source : `maze_nakagaki.py`. Critère : égalité.
- **V-NAK-02** : les quatre plus courts chemins sont **82, 66, 88 et 90** cellules (41, 33, 44 et 45 mm à 2 cellules par
  millimètre ; article : `41, 33, 44, 45 ± 1 mm`), par `chemins::plus_courts` en coupant les autres routes ; le chemin
  entier 154, et 170, 172, 154, 156 pour les quatre coupures doubles ; 82 + 66 + 88 + 90 + 17 = 343 arêtes. Source :
  parcours en largeur de `maze_nakagaki.py` (mêmes nombres) ; article pour l'intention. Critère : égalité.
- **V-NAK-03** : après 150 pas, graines 1 à 12 : α1 exactement au `PLANCHER`, α2 `>= 0,999`, impasses au plancher ; à
  200 pas, les tubes vifs sont les 154 arêtes de α2 puis β1 (Dijkstra) ; β1 dans `[0,89 ; 0,91]`, β2 dans `[0,09 ;
  0,11]`. Mesure (50 graines) : α2 `1,000` à quatre chiffres ; β1 de `0,8973` à `0,9094`, β2 de `0,0906` à `0,1027` ;
  graine 1 par le modèle Python : β1 `0,84785`, β2 `0,15215` (150 pas), `0,90697` et `0,093027` (200 pas), tolérance
  `1e-5`.
- **V-NAK-04** : les pas de mort (premier nombre de pas faits après lequel la plus grande conductivité de la route est
  `< 0,01`), graines 1 à 12, par le modèle Python : impasses 7, α1 45, β2 408, 415, 411, 412, 412, 410, 411, 409, 410,
  414, 410, 410 ; β1 et α2 jamais. Mesure (50 graines) : α1 44 ou 45, β2 de 405 à 418. Exigences : β2 au moins 8 fois
  plus tard qu'α1 (rapport mesuré 9,07 au plus bas) ; à 60 pas, α1 `<= 0,003` (mesuré `<= 0,00215`), β2 `>= 0,32`
  (mesuré `>= 0,3233`), β1 `>= 0,64` (mesuré `>= 0,6457`). Raison : l'écart relatif des deux longueurs de chaque paire
  (22 % et 2,3 %).
- **V-NAK-05** : à 600 pas, β1 `>= 0,99` (mesuré `>= 0,9987`), β2 `<= 0,004` (mesuré `<= 0,001307`, tolérance trois fois
  la mesure), 154 tubes vifs, pression de la source dans `[154,0 ; 154,006]` (mesuré `154,00225` à `154,00259`) ; à 20
  pas β1 dans `[0,52 ; 0,59]` et β2 dans `[0,41 ; 0,48]` (mesuré `0,5377` à `0,5719` et `0,4281` à `0,4623`). Aucune
  valeur de l'article n'est visée.
- **V-NAK-06** : les impasses (17 arêtes) sont au `PLANCHER` à 20 pas ; leur plus grande conductivité est `<= 0,01` à 7
  pas et `> 0,01` à 6. Source : arithmétique (`0,75 * 2^-n`), modèle Python (7 pour les 12 graines).
- **V-NAK-07** : la structure : connexe, `343 + 1 - 342 = 2` cycles, degrés `J` 4, `S` 3, `T` 2, trois bouts d'impasse ;
  couper α2 laisse relié, couper α1 et α2 donne `SourceCoupee`. Source : dessin. Critère : égalité.

## les étages

Source : Dijkstra (`chemins`), la formule `2 c + L_haut < L_bas`, des tolérances mesurées sur 50 graines (1000 à 1049),
300 pas, `h = 0,5`.

- **V-ETA-01** : pour chaque `c` de la suite `0,1 ; 0,25 ; 0,5 ; 1 ; 2 ; 3 ; 4 ; 6 ; 8 ; 10`, le plus court chemin monte
  si `2 c + 10 < 20` (22 arêtes, deux liens verticaux en `(0, 1)` et `(20, 1)`), sinon reste en bas (20 arêtes) ; la
  distance est `2 c + 10` ou `20` à `1e-9` près. Source : calcul à la main ; Dijkstra. Les `c` de 4 à 6 sont écartés
  (voir la doc du test : convergence lente et égalité à `c = 5`).
- **V-ETA-02** : après 300 pas, graines 1 à 8 : les arêtes vives sont exactement celles du plus court chemin, `>=
  0,9999` (mesuré `>= 0,999985`), les autres `<= 3e-5` (mesuré `<= 9,95e-6`) ; la pression de la source est dans `[d ; d
  + 1e-5]` (mesuré `+3e-6` au plus) ; le nombre de liens verticaux vifs vaut 2 ou 0 et ne croît jamais avec `c`. Source
  : Bonifaci et al. 2012, théorème 2 (une seule route survit) ; la mesure.
- **V-ETA-03** : le pont (grille 15 × 5, mur sur toute la colonne 7 de l'étage 0, facteur 0,5 à l'étage 1) : distance
  exacte 7, 12 arêtes dont deux liens en `(2, 2)` et `(12, 2)` ; vives après 300 pas = ces arêtes `>= 0,9999` (mesuré
  `>= 0,999994`), les autres `<= 1e-5` (mesuré `<= 2,18e-6`), pression dans `[7 ; 7,00001]` ; le même mur à l'étage 1
  donne `SourceCoupee`. Source : calcul à la main ; mesure.

## lancer

Source : le modèle Python `ref_labyrinthe.py` (mêmes opérations dans le même ordre : mêmes bits). Tolérance : **aucune**
(octets, empreintes, textes). Le prototype est tombé juste du premier coup sur les neuf empreintes comparées.

- **V-LLAB-01** : `grille 3 2`, `mur 1 0 1 0 0`, source `(0,0,0)`, puits `(2,0,0)`, 1 pas de 0,5, graine 7 : l'état de
  240 octets écrit à la main (cinq nœuds, quatre arêtes `(0,2) (1,4) (2,3) (3,4)`, **source 0 et puits 1** : les indices
  du plan) ; conductivités `3FED22A5CE1B6194`, `3FE8CC5E27EDBB33`, `3FEDC961D2C3EC75`, `3FEE8483AC761D88` ; SHA-256
  `8ce3c3a3…1d40`.
- **V-LLAB-02** : `grille 2 1`, `etages 2 1.0 2.0`, `cout_etage 1 0.5`, source `(0,0,0)`, puits `(1,0,0)`, 1 pas, graine
  7 : état de 228 octets (quatre arêtes de coûts `2 ; 1 ; 0,5 ; 2`), conductivités `3FE144D2B6E46A50`,
  `3FE4AA313F24B276`, `3FE1EB8EBB8CF530`, `3FE2A6B0953F2643` ; SHA-256 `9f048e06…60a4`.
- **V-LLAB-03** : les empreintes de neuf scénarios (voir le test) ; l'ordre des directives, les doublons de murs et un
  mur coupé en deux ne changent rien ; chaque changement (un mur, un facteur d'un millième, la graine) change
  l'empreinte.
- **V-LLAB-04** : régression : `references/labyrinthe-graine-1.empreinte` = `dcdad079…3ded` (produite par le binaire,
  égale à la valeur du modèle Python).
- **V-LLAB-05** : les mesures : textes entiers pour les deux petits cas ; nombre de lignes, première, dernière et
  SHA-256 pour huit scénarios. Au labyrinthe, 343 tubes vifs au départ, 154 à la fin, pression finale `154,007069516`
  (graine 1).
- **V-LLAB-06** : les anciens scénarios gardent leur empreinte (`chemin.txt` : `6e3b15b8…2f37`, `grille.txt` : le
  fichier de l'étape 0, deux scénarios du cœur) ; le coût vertical de chaque fichier de `scenarios/` est dans `[0,1 ;
  10]`.
- **V-LLAB-07** : les erreurs de directives : `DirectiveInconnue(ligne)` selon la priorité de la doc (première fautive,
  même nœud, directive manquante, puis les erreurs de `construire` : puits muré = ligne de `puits`, source murée ou
  coupée = ligne de `source`). Source : doc.
- **V-LLAB-08** : le binaire : code 0, `empreinte.txt`, `mesures.csv`, `apercu.png` 511 × 371 ; une source murée : code
  1, `ligne 3`, rien d'écrit.

## apercus

Source : le modèle Python (treillis coloré, niveaux, mélanges entiers) ; des dessins en caractères écrits d'après la
doc.

- **V-APL-01** : un mur est un trou : dessins `o...o / -...- / o-o-o` (grille 3 × 2 murée en `(1,0)`) et l'anneau de 9 ×
  9 points (poche) ; SHA-256 RVB `3416344f…6daf` et `66320605…3d72`.
- **V-APL-02** : le cadre suit les nœuds gardés : 40 × 40 sans la première colonne, 56 × 24 sans la dernière ligne, un
  seul panneau si l'étage 1 est entièrement muré (40 × 24 contre 96 × 24).
- **V-APL-03** : le labyrinthe : 511 × 371 ; SHA-256 RVB à 20 pas `f0190b82…d577`, 100 pas `221676d4…3e6`, 500 pas
  `61c938bf…f694` (identique pour les graines 1 et 2) ; 657 cellules murées du cadre sont du fond, 65 nœuds d'α2, `S` et
  `J` à `VIF`, `T` à `(60, 219, 254)` (niveau 254), 81 nœuds d'α1 et 17 d'impasses au fond.
- **V-APL-04** : les étages : 504 × 30, SHA-256 `d9d3bdd6…eb1a` (monte) et `bb90844e…4110` (reste) ; quatre points
  `PUITS` exactement (4 × 36 pixels) dans le premier, aucun dans le second ; le pont : 480 × 72, `086848e5…95e5`, quatre
  points `PUITS` (4 × 64 pixels), la colonne murée de l'étage 0 est un trou.
- **V-APL-05** : l'aperçu de `lancer` est celui de `tubes::apercu` sur le graphe du plan (cas en voisinage 8 à deux
  étages, 160 × 56, SHA-256 `eb504a2c…f72d` ; le labyrinthe à 20 pas, échelle 7).

## campagne

Source : la campagne de fautes du juge (202 fautes, deux faiblesses fermées par ces deux tests).

- **V-LCAM-01** : un `cout_etage` fautif précédé d'une directive valide rend sa propre ligne (4, ou 5 après un
  `cout_etage` bon et un `mur`), pour six genres de faute. Source : doc de `lancer`, « Les erreurs », point 1.
- **V-LCAM-02** : les entiers de `mur` et de `cout_etage` acceptent les zéros de tête : même empreinte que sans ; un
  `mur 0001 …` est lu. Source : doc de `Directive::entier`.
