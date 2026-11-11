# Étape 1 (le cœur) : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test du cœur de l'étape 1 attend, d'où elle vient, avec quelle tolérance et
quel critère de réussite. Chaque identifiant (`V-CHEM-01`, `V-TUBE-01`, `V-CONV-01`, `V-APER-01`, `V-LTUB-01`,
`V-MES-01`, `V-BIN1-01`…) est cité dans la doc du test qui l'emploie (`tests/etape_1_*.rs`). Elle est séparée des
résultats figés de notre propre code
(`references/*.empreinte`, étiquetés « régression » dans les tests) : **aucune valeur de cette fiche ne vient de notre
code**. Une valeur y vient d'une publication lue, d'un calcul exact fait à la main ou par un outil indépendant (nommé),
d'un simulateur naïf écrit dans le test, ou d'une convention que la documentation du module fixe (dite telle).

Écrite avant les calculs, comme la fiche de l'étape 0 : les critères d'abord (sections « Critères »), les mesures
ensuite (sections « Mesures »).

## Les outils indépendants

Python 3.13 (`fractions`, flottants IEEE 754, `struct`, `hashlib`) ; le code Rust n'est jamais l'outil. Gardés avec les
notes de la session qui a écrit ces tests, dans `outils-1/` :

- `ref_alea.py`, `ref_graphe.py`, `ref_solveur.py` : repris de l'étape 0 (PCG32, la grille et l'écriture canonique, la
  procédure du solveur) ;
- `ref_tubes.py` : écrit d'après la seule doc de `tubes` et de `lancer` : les conductivités de départ, un pas du modèle,
  N pas (avec le solveur de la doc, dense ou à profil : `ref_tubes.py` vérifie que les deux donnent les mêmes bits),
  l'écriture canonique de l'état avec tubes, l'empreinte, les mesures en CSV, le niveau, le mélange et le treillis
  coloré de l'aperçu ;
- `valeurs_tubes_1.py`, `valeurs_apercu_1.py`, `valeurs_lancer_1.py` : les scripts qui impriment les valeurs écrites
  dans les tests ; `valeurs_calcul_1.py` : le pas où le solveur s'arrête (V-LTUB-10) ; `mutants_etape_1.py` : le banc
  de mutants ; `mesures/` : les deux programmes de mesure des tolérances
  de la convergence (ils s'appuient sur `tests/common/graphes_au_hasard.rs`).

Dans les tests, trois simulateurs naïfs jouent le même rôle : `procedure_naive` (`etape_1_chemins.rs`, la procédure de
la doc de `chemins` en balayant toutes les arêtes), le Bellman-Ford de `tests/common/graphes_au_hasard.rs`, et
`pas_naif` (`etape_1_tubes.rs`, un pas de la doc avec `solveur::pressions` et `solveur::flux`, éprouvés à l'étape 0).

## Les sources lues (et ce qui ne l'est pas)

Les lectures sont celles de `references-etapes-0-1.md` (notes du pilote) ; aucune valeur n'est ajoutée de mémoire. « Lu
» signifie lu dans le texte de l'article ; « notice » signifie lu dans une notice.

- **Tero, Kobayashi, Nakagaki 2007**, J. Theor. Biol. 244, 553-564 : lu sur la version d'auteur du 11 juillet 2006
  (HUSCAP), dont les numéros de section et d'équation ne sont pas ceux de la revue. Lu : le flux de Poiseuille
  `Q = D (p_i - p_j) / L` (éq. 1), la conservation, `dD/dt = f(|Q|) - D` sans dimension (éq. 4-5), le flux total
  constant, la pression nulle au puits (`p_2 = 0`), les conductivités de départ « tirées au hasard, uniformément dans
  [0,5 ; 1,0] », le résultat pour `f` linéaire (« le chemin le plus court est sélectionné quelles que soient les
  conductivités initiales ») et, pour deux chemins de même longueur, « un point quelconque du segment » selon l'état
  initial (tableau p. 18). **Non lu / non donné** : le pas `Δt` (le schéma est semi-implicite, le nôtre est explicite) ;
  la numérotation de la revue.
- **Bonifaci, Mehlhorn, Varma 2012**, J. Theor. Biol. 309, 121-133 : lu sur l'arXiv v3 (1106.0423), pas sur la revue.
  Lu : `D' = |Q| - D`, le théorème 2 (attirance vers l'ensemble des flux de coût minimal ; convergence vers le flux de
  valeur 1 sur le chemin quand il est unique ; le potentiel source-puits tend vers `L*`), la conjecture de convergence
  vers un point dans le cas de plusieurs chemins, la fonction de Lyapunov `V = Σ L_e D_e` (pour le flux minimal). Pas de
  temps discret : absent de ce texte. **Non lu** : la version de la revue ; le numéro de page exact du théorème.
- **Becchetti, Bonifaci, Dirnberger, Karrenbauer, Mehlhorn 2013**, ICALP (LNCS 7966, 472-483) : lu sur le PDF final de
  la page de Mehlhorn. Lu : la mise à jour d'Euler `x(t+1) - x(t) = h (|q| - x(t))` avec `h` dans `]0, 1[`. **Non
  vérifié** : la condition exacte sur `h` (symboles perdus à l'extraction), la version longue ; le DOI et la pagination
  viennent d'un résultat de recherche.
- **Ce qui est à nous** (aucun article) : le plancher `PLANCHER = 1e-6` ; le choix du schéma explicite ; `cout` à la
  place de la longueur dans le flux ; la formule de l'aperçu ; la forme de l'écriture canonique et des mesures.

---

## chemins

Source : la procédure de la doc de `chemins` (Dijkstra simple, égalités par le plus petit indice, comparaison stricte),
recopiée naïvement dans le test, plus des calculs à la main sur de petits graphes d'entiers. Tolérance : **aucune**
(entiers, égalités, bits).

- **V-CHEM-01** : graphe A (six arêtes `(0,1)=4`, `(0,2)=1`, `(1,2)=2`, `(1,3)=1`, `(2,3)=5`, `(3,4)=3`, rangées dans
  cet ordre, indices 0 à 5) depuis 0 : distances `0, 3, 1, 4, 7`, parents `None, 2, 1, 3, 5`, chemin vers 4 `[1, 2, 3,
  5]` ; depuis 4 : distances `7, 4, 6, 3, 0`, parents `1, 3, 2, 5, None`. Les longueurs sont fausses exprès. Source :
  calcul à la main sur la liste. Critère : égalité.
- **V-CHEM-02** : le losange `0-1, 0-2, 1-3, 2-3` : `parents[3] = arête 1-3` (indice 2), `vers(3) = [0, 2]` ; le
  graphe B : `parents[4] = arête 2-4` (indice 3), `vers(4) = [0, 2, 3]` (le nœud 2 est visité avant le 3 bien que 3 soit
  découvert avant) ; depuis 4 dans B : `parents[0] = arête 1` (le candidat égal de l'arête 0 ne remplace pas). Source :
  la procédure de la doc, déroulée à la main. Critère : égalité.
- **V-CHEM-03** : un nœud hors d'atteinte garde `+inf` et `None` ; `vers` rend `None` ; `1e308 + 1e308 = +inf` laisse le
  nœud hors d'atteinte. Source : doc de `chemins` ; arithmétique IEEE 754 (Python). Critère : égalité.
- **V-CHEM-04** : la chaîne de coûts `1e16, 1, 1` : `distances[3] = 1e16` (somme de gauche à droite : `1e16 + 1`
  s'arrondit au pair), pas `1e16 + 2`. Source : arithmétique IEEE 754 vérifiée avec Python. Critère : égalité des bits.
- **V-CHEM-05** : grille 3 × 2 à deux étages, lien vertical à `0,25` : la distance de `(x, y, e)` est `x + y + 0,25 e`
  (exacte en `f64`) ; avec un lien à `3` : `(2,1,0)` à 3 et `(2,1,1)` à 6. Source : définition de la grille (étape 0).
  Critère : égalité ; chemins contigus dont la somme redonne la distance aux bits.
- **V-CHEM-06** : 60 graphes au hasard (2 à 40 nœuds, coûts dans `[0,001 ; 100[`), 3 sources chacun : les distances sont
  les mêmes **aux bits** que celles du Bellman-Ford naïf du test. Source : simulateur naïf dans le test. Raison de
  l'égalité aux bits (et non à `1e-12`) : l'addition de flottants est croissante et ne diminue jamais, donc le minimum
  des sommes faites de gauche à droite le long des chemins est le même pour les deux méthodes. Mesure : égalité sur 180
  couples (graphe, source), plus de 1000 nœuds atteints et plus de 50 hors d'atteinte. Critère : égalité des bits.
- **V-CHEM-07** : 40 graphes à coûts entiers dans `{1, 2, 3}` et les grilles 5 × 5, 4 × 3 × 2 (lien à 0,5) et 3 × 3 × 2
  en voisinage 8 (depuis chaque nœud) : distances et parents égaux à ceux de `procedure_naive`, aux bits. Source :
  simulateur naïf dans le test. Critère : égalité.
- **V-CHEM-08** : `plus_courts` panique pour une source hors du graphe, `vers` pour une cible hors du graphe. Source :
  doc.

## tubes

Sources : la doc de `tubes` ; des récurrences écrites dans le test ; un pas naïf avec le solveur éprouvé ; le modèle
Python `ref_tubes.py`. Tolérances : **mesurées** sur le prototype (écrites en commentaire du test), sauf les cas à bits.

- **V-TUBE-01** : un tube seul, `|Q| = I`, `D <- D + h (I - D)`, pression de la source `I cout / D`, pression du puits
  `+0.0` ; trois cas (coût, `D0`, `I`, `h`), deux sens, 60 pas. Source : récurrence écrite dans le test (calcul exact :
  le solveur d'un nœud divise, le flux multiplie). Mesure (en relatif à `I`) : `D` au plus 1,5e-16, `|Q| - I` 1,4e-16,
  pression 2,1e-16, forme fermée `(D0 - I)(1 - h)^n` 1,3e-16. Tolérance `1e-15` (5 à 7 fois la mesure : quelques ulps).
- **V-TUBE-02** : une chaîne de cinq tubes : chaque `D` suit sa récurrence, `|Q| = I` partout, pression de la source
  `I Σ cout / D`. 40 pas, `h = 0,2`, `I = 2`, deux sens. Mesure : `D` 1,2e-15, `|Q| - I` 5,3e-15, pression 3,0e-15.
  Tolérances `4e-15`, `1,5e-14`, `1e-14` (3 fois : l'erreur grossit avec les rapports de conductances).
- **V-TUBE-03** : une branche morte : `D0 * 0,5^n` jusqu'au pas 19 (1,53e-6), puis exactement `PLANCHER` à partir du pas
  20 (7,63e-7 serait en dessous), 50 pas ; un tube au plancher sans flux y reste. Source : récurrence et arithmétique
  (`0,8 * 2^-19 = 1,526e-6`, `0,8 * 2^-20 = 7,63e-7`). Tolérance `1e-9` en relatif sur la décroissance (le flux de
  l'impasse est au plus `1e-12` ; mesuré par le modèle Python : exactement 0) ; le plancher est exact (les bits).
- **V-TUBE-04** : un tube au plancher sur le chemin du flux remonte : `PLANCHER + 0,5 (1 - PLANCHER)` après un pas, à
  `1e-12` (le rapport des conductances est `1e6` ; écart mesuré par le modèle Python : 0, pression de la source
  `1000001` exacte). Source : récurrence.
- **V-TUBE-05** : la coupe autour de la source : `S <- S + h (I - S)` et `Σ|Q| = I` sur ses arêtes. Source : le lemme de
  Bonifaci pour des tubes en parallèle, en temps discret : le flux total qui quitte la source est `I`, chaque arête de
  la source l'évacue (la pression de la source est la plus haute), donc `Σ|Q| = I` et la somme des mises à jour est `S +
  h (I - S)` (déduction faite ici, en une ligne, valable tant qu'aucune arête n'est tenue par le plancher). 12 graphes
  au hasard, `h` parmi `0,1 ; 0,25 ; 0,5`, `I` parmi `1 ; 2,5`, 40, 25 ou 10 pas. Mesure : `S` 8,9e-16 (relatif à
  `max(S, I)`), `Σ|Q| - I` 2,0e-15 (relatif), plus petite conductivité d'une arête de la source `6,8e-4` (donc jamais
  près du plancher). Tolérance `6e-15` (3 fois) ; exigence : la plus petite reste `> 1e-4`.
- **V-TUBE-06** : `Tubes::new` : `Taille` d'abord, puis `Conductivite(e)` pour le plus petit indice ; `PLANCHER`
  accepté, son prédécesseur flottant refusé ; zéro, négatifs, infinis, NaN refusés ; `uniformes` pareil ; `au_hasard` :
  la formule de la doc avec un générateur jumeau, l'état du générateur après (`2 * m` tirages de 32 bits) ; valeurs de
  `ref_tubes.py` pour les graines 1 (trois arêtes), 7 (deux) et 0 (une) : `0x3FE9B6BDDA90C320`, `0x3FECE97F8EF822C0`,
  `0x3FE943E8CF21B75E` ; `0x3FEA454B9C36C328`, `0x3FE198BC4FDB7666` ; `0x3FE47C28B93B98F6` ; générateurs `3dce35a1…`,
  `fd6eabdb…`, `1ca3a65f…`. Source : doc de `tubes` ; PCG32 (`ref_alea.py`). Critère : égalité des bits.
- **V-TUBE-07** : l'ordre des erreurs d'`avancer` : `Taille`, `Noeud`, `Debit`, `Pas`, `Solveur(Isole(i))` ; rien n'a
  changé sur une erreur ; les bornes valides (`h` dénormalisé, `h = 0,9999999`, débit `0,001`) passent. Source : doc.
- **V-TUBE-08** : `avancer` contre `pas_naif`, aux bits (conductivités, pressions, flux), 20 graphes au hasard de 3 à 40
  nœuds, 12 pas chacun (240 pas), `h` parmi `0,05 ; 0,3 ; 0,5 ; 0,9`, `I` parmi `1 ; 2,5 ; 0,3`. Source : simulateur
  naïf dans le test. Critère : égalité des bits.
- **V-TUBE-09** : trois pas sur le pont de Wheatstone (coûts `1 ; 2 ; 1,5 ; 0,5 ; 3`, `D0 = 0,9 ; 0,6 ; 0,75 ; 0,5 ; 1`,
  `I = 1,5`, `h = 0,3`) : les bits des conductivités, des pressions et des flux, donnés par `ref_tubes.py`. Le premier
  pas donne en décimal `0,9675 ; 0,5325 ; 0,525 ; 0,6875 ; 0,8125` (vérifié à la main : `D + 0,3 (|Q| - D)` avec les
  flux `1,125 ; 0,375 ; 0 ; 1,125 ; 0,375`). Critère : égalité des bits.
- **V-TUBE-10** : multiplier `D0` et `I` par 4 multiplie par 4 les conductivités et les flux et laisse les pressions :
  une puissance de 2 traverse chaque opération sans arrondi. 8 graphes, 6 pas, `h = 0,5`. Source : arithmétique exacte.
  Critère : égalité des bits (et des résultats non vides : le test compare deux sorties entre elles).
- **V-TUBE-11** : `Taille` dans les deux sens : des tubes de deux arêtes avancés sur un graphe de trois, et l'inverse ;
  rien n'a changé. Source : doc (vérification 1 d'`avancer`). Ajouté après la campagne de fautes du juge.

## convergence

Source : le théorème 2 de Bonifaci, Mehlhorn et Varma (2012) et l'énoncé de Tero et al. (2007) pour `f` linéaire (voir «
Les sources »). Le chemin et sa marge viennent de `chemins::plus_courts` et du Bellman-Ford du test (retirer tour à tour
chaque arête du chemin) : le second meilleur chemin est plus long d'au moins 5 %. Tolérances **mesurées** sur d'autres
graines que celles du test ; elles mesurent la fuite du plancher, pas l'arrondi (voir `etape_1_convergence.rs`).

- **V-CONV-01** : 20 graphes au hasard (10 à 30 nœuds, coûts dans `[1 ; 3[`, chemin unique à 5 %), `h = 0,5`, 800 pas :
  les arêtes du chemin à moins de `2e-5` de 1, toutes les autres sous `1e-5`, la pression de la source à `2e-14`
  (relatif) de la longueur, et les arêtes au-dessus de 0,5 exactement celles du chemin ; départs au hasard, uniformes à
  1 ou larges (`[0,01 ; 5,01[`). Mesure (350 autres graphes en quatre séries) : 5,3e-6 à 6,9e-6 ; 2,8e-6 à 3,9e-6 ; au
  plus 1,1e-15. Marge minimale observée du second meilleur : 1,0501. Convergence atteinte vers 500 pas pour les cas
  les plus lents (à 400 pas : 2,8e-5 à 1,5e-4). Tolérances `2e-5` (3 fois), `1e-5` (2,5 fois), `2e-14` (18 fois).
- **V-CONV-02** : le même constat à `h = 0,1`, 4000 pas, 5 graphes. Mesure (100 autres graphes) : 6,6e-6 ; 3,2e-6 ;
  6,3e-16. Mêmes tolérances.
- **V-CONV-03** : une échelle à deux étages construite à la main (`Graphe::new`), le plus court chemin monte par un lien
  et redescend plus loin (longueur 3, second meilleur 3,6, marge 20 %) : 200 pas, `h = 0,5`, cinq graines de départ :
  les sept arêtes du chemin à `2e-6` de 1, **toutes** les autres (impasses comprises) exactement au plancher, pression à
  `2e-14` de 3. Mesure (50 autres graines) : 7,7e-7 ; exactement le plancher ; 3,0e-15. Tolérances `2e-6` (2,6 fois), le
  plancher exact, `2e-14` (7 fois).
- **V-CONV-04** : le même graphe avec `I = 2,5` et `I = 0,2`, 400 pas : écart des arêtes du chemin 7,7e-7 pour les deux
  débits (la fuite du plancher est absolue), les autres exactement au plancher, pression 1,3e-15 et 3,0e-16, tolérances
  `2e-6`, plancher exact, `2e-14`. La pression de la source tend vers la longueur du chemin quel que soit `I`.
- **V-CONV-05** : plusieurs plus courts chemins (grille 5 × 4 en voisinage 4, source `(1,1)`, puits `(3,2)`, `G0` de
  sept arêtes) : ce que le théorème dit (attirance vers les flux de coût minimal) et rien de plus : hors de `G0` le
  plancher exact ; pression à `2e-14` de 3 ; `Σ L D` à `5e-5` de 3 ; les `D` de `G0`, orientés de la source vers le
  puits, forment un flux de valeur 1 (résidu au plus `4e-6`) ; la répartition entre les chemins dépend de l'état initial
  (Tero 2007, tableau p. 18) : la première arête de `G0` varie de plus de `0,05` d'une graine à l'autre. Mesure :
  plancher exact ; 3e-16 ; 2,09e-5 ; 9,5e-7 ; première arête entre 0,567 et 0,678 (écart 0,11). Le théorème ne dit pas
  que `D` converge vers un point : le test ne le prétend pas.

## apercu

Source : la définition de la doc de `tubes` (« L'aperçu des tubes ») ; les couleurs et les dessins viennent de
`ref_tubes.py` (entiers) et de dessins à la main recoupés. Tolérance : **aucune** (octets).

- **V-APER-01** : `VIF = (60, 220, 255)`, `FOND = (12, 16, 24)`, `PUITS = (240, 170, 40)`.
- **V-APER-02** : niveau et mélange : niveau 0 pour `0`, `-0`, négatif, NaN, `1e-6`, `0,001` ; 1 pour `0,002` ; 51 pour
  `0,2` ; **77 pour `0,3`** (`0,3 * 255` vaut exactement `76,5` en flottants : demi vers l'extérieur) ; **128 pour
  `0,5`** ; 204 pour `0,8` ; 255 pour `0,9999`, `7`, l'infini ; le rapport compte (`3 / 6`, `0,25 / 0,5`). Couleurs :
  niveau 1 `(12, 17, 25)`, 51 `(22, 57, 70)`, 77 `(26, 78, 94)`, 128 `(36, 118, 140)`, 204 `(50, 179, 209)`, 255 `VIF`.
  Source : `ref_tubes.py`.
- **V-APER-03** : la grille 3 × 2 en voisinage 8, onze arêtes de niveaux `255, 0, 128, 51, 255, 0, 128, 51, 0, 51, 0` :
  dessin `cccaa / .c.b. / cab.b` (le point d'un nœud prend le niveau de sa plus épaisse arête ; sur un croisement de
  diagonales, la plus épaisse l'emporte). Source : dessin à la main d'après la doc, recoupé par `ref_tubes.py`.
- **V-APER-04** : deux étages (3 × 2, voisinage 4) : `AVBVC..AeDeE / V.V.V..e.e.e / VVBVA..eeDeA`, avec les couleurs
  `B (150, 195, 147)`, `C (96, 210, 212)`, `D (138, 144, 90)`, `E (66, 80, 64)` de `ref_tubes.py` (mélange du point du
  nœud vers `PUITS` au niveau du lien) ; un lien de niveau 0 laisse le nœud tel qu'il est.
- **V-APER-05** : le même treillis que `rendu::apercu_graphe` : un point est du fond dans un aperçu si et seulement s'il
  l'est dans l'autre, quand toutes les conductivités valent la référence (grille 4 × 3 × 2 en voisinage 8, le graphe
  quelconque de `etape_0_rendu`), échelles 1 à 4. Source : l'étape 0 (éprouvée).
- **V-APER-06** : le SHA-256 des octets RVB d'un cas riche (grille 4 × 3 × 2, voisinage 8, 24 nœuds, 70 arêtes, `D[e] =
  ((e * 37 + 11) % 23) / 20`) : `5886ef40…bf9a8` (référence 1, échelle 3, 48 × 15) et `7967a5ec…5634` (référence 0,5,
  échelle 1, 16 × 5). Source : `ref_tubes.py`.
- **V-APER-07** : paniques : échelle 0 ; référence nulle, négative, infinie, NaN ; mauvais nombre de conductivités.
- **V-APER-08** : un nœud sans arête est du fond ; l'image d'un nœud seul fait 1 × 1 (× échelle) ; une arête de niveau
  255 colore ses deux nœuds en `VIF`.
- **V-APER-09** : le cadre compte tous les nœuds : le nœud 0 à l'étage 1 et le nœud 1 à l'étage 0, à la même position,
  reliés : 4 × 1, `P..P` (lien de niveau 255), `e..e` (niveau 51, `e = (66, 80, 64)`). Source : dessin à la main d'après
  la doc, recoupé par `valeurs_campagne_1.py` (le modèle Python). Ajouté après la campagne de fautes du juge, comme
  les deux suivants.
- **V-APER-10** : deux liens entre étages sur un même nœud : le plus épais compte. Trois étages d'un nœud, liens de
  niveaux 255 et 51 : `P..P..e` (7 × 1) ; 51 et 255 : `e..P..P`. Même source.
- **V-APER-11** : deux nœuds A et B au même point, C à côté, arêtes A-B, A-C, B-C : niveaux 51, 255, 128 : `bcc` (le
  milieu de A-B, puis A, puis B qui recouvre) ; niveaux 255, 51, 128 : `cbb`. Même source.

## lancer avec les tubes

Source : la doc de `lancer` (« Ce que fait l'étape 1 »). Les empreintes et les mesures sont **simulées par
`ref_tubes.py`** (flottants Python, mêmes opérations dans le même ordre) : les mêmes bits sont attendus. Tolérance :
aucune.

- **V-LTUB-01** : l'écriture canonique de `grille 2 1`, graine 7, `h = 0,5` : un pas (108 octets, `PHYSAR01`, source 0,
  puits 1, nombre 1, `h` `000000000000E03F`, conductivité `94611BCEA522ED3F`, générateur `CFC1AAFE29BE9809
  6D00000000000000`), SHA-256 `c750f1c3…4f8d` ; zéro pas, SHA-256 `8ca8aed2…366f` ; source et puits échangés
  `96bfb656…3f9e`. Source : octets écrits à la main d'après la doc ; SHA-256 de Python `hashlib` ; la conductivité
  `0,8209589053204569` est le premier tirage de la graine 7 (`ref_alea.py`), et `0,9104794526602284` vaut `0,5 (1 + D0)`
  à un ulp près (un tube seul).
- **V-LTUB-02** : empreintes de quatre scénarios simulés (`grille 3 2`, `grille 3 3` à deux étages en voisinage 8,
  `grille 4 4` deux fois — l'ordre des directives et les commentaires ne changent rien —, `h = 0,999`) : `39fe16f1…`,
  `77acbb3c…`, `ee991bd8…`, `c96e6c7e…`. Chaque changement (graine, `h`, nombre de pas, source, puits) change
  l'empreinte.
- **V-LTUB-03** : les mesures : `grille 2 1` (une ligne), zéro pas (l'en-tête seul), `grille 3 2` (cinq lignes en
  toutes lettres), et pour deux scénarios plus gros le nombre de lignes, la première, la dernière et le SHA-256 du
  texte. Le seuil des tubes vifs (`>= 0,5`) ne se teste pas à sa limite par `lancer` : aucune conductivité de départ
  n'est exactement 0,5 (`0.5 + 0.5 * u` avec `u = 0` seulement). La limite est éprouvée sur `mesures::tubes_vifs`, que
  `lancer` appelle (V-MES-01).
- **V-LTUB-04** : l'aperçu : dimensions (`24 × 8`, `40 × 24`, `96 × 40`, `56 × 56`), pixels de `tubes::apercu` sur les
  conductivités recalculées avec l'API, SHA-256 des octets RVB de `ref_tubes.py`.
- **V-LTUB-05** : les erreurs de directives, avec la règle de priorité de la doc. Source : doc ; chaque texte est
  lisible par `lire_scenario`.
- **V-LTUB-06** : `scenarios/chemin.txt` (16 × 16, voisinage 8, source `(1,1)`, puits `(14,14)`, 600 pas de 0,5), graine
  1 : empreinte `6e3b15b8…2f37` (`ref_tubes.py`, écrite avant le code), graine 2 `bf66d262…b8db` ; 600 lignes de
  mesures, la dernière `599,18.384876222,13,18.384776` (treize arêtes vives, `13 √2 = 18,384776`) ; aperçu 248 × 248,
  SHA-256 `764418f3…6331` (le même pour les deux graines : à 600 pas, l'image ne dépend plus de la graine) ; les 14
  nœuds de la diagonale à la couleur vive.
- **V-LTUB-07** : `references/chemin-graine-1.empreinte` égale l'empreinte de `lancer` (**régression** : le fichier
  vient du binaire ; `le_scenario_chemin` compare la même valeur au modèle Python).
- **V-LTUB-08** : l'étape 0 ne change pas : mêmes empreintes (`e07d8298…`, `4d066023…`, `c3361481…`), mêmes mesures,
  mêmes refus de directives inconnues.
- **V-LTUB-09** : 100 000 pas sur `grille 2 1` : 100 000 lignes, la dernière `99999,1.000000000,1,1.000000`.
- **V-LTUB-10** : un calcul qui s'arrête rend `ErreurLancer::Calcul(t)`, sans panique. `grille 3 3` à deux étages (lien
  de longueur 1), source `(0,0,0)`, puits `(2,2,0)`, pas de 0,5 : coût du lien `10^16` : `t = 2` (graine 1), `7` (graine
  2), `3` (graine 7), `13` (graine 9) ; coût `10^-16` : `t = 1` (graines 1 et 2), `0` (graine 7) ; coût `10^15` : aucune
  erreur en 40 pas. Le scénario arrêté avant le pas `t` réussit. Source : `valeurs_calcul_1.py`, le modèle Python avec
  le contrôle du pivot de la doc du solveur (« fini et `> 0.0`, sinon `Pivot` »), écrit et joué avant de regarder le
  code. Critère : égalité. Ajouté par le juge : le premier prototype paniquait, contre la doc du binaire (« sans
  `panicked` »).
- **V-LTUB-11** : l'échelle de l'aperçu : `512 / TL` (la largeur du treillis) entre 1 et 8 : `grille 40 2` : `TL = 79`,
  échelle 6, image 474 × 18 ; `grille 256 1` : `TL = 511`, 511 × 1 ; `grille 256 1` à deux étages : `TL = 1024`,
  échelle ramenée à 1, 1024 × 1, sans panique. Source : calcul à la main sur le treillis de l'étape 0. Ajouté après la
  campagne de fautes du juge.

## mesures

Source : la doc de `mesures` ; arithmétique IEEE 754 vérifiée avec Python. Tolérance : **aucune** (entiers, bits).

- **V-MES-01** : le seuil est compris : quatre tubes de longueurs `1,5 ; 2,25 ; 4 ; 8,5` et de conductivités `0,5`, le
  flottant juste en dessous, celui juste au-dessus, `1e-6` : `(2, 5,5)` au seuil `0,5` ; `(4, 16,25)` au seuil `1e-6` ;
  `(1, 4)` au seuil du flottant juste au-dessus de 0,5 ; `(3, 7,75)` à celui juste en dessous ; `(0, +0.0)` au seuil
  `0,6`. Les coûts (10, 20, 30, 40) ne comptent pas. Source : calcul exact à la main. Ajouté par le juge : le seuil ne
  se testait pas à sa limite par `lancer`.
- **V-MES-02** : la somme suit l'ordre des indices : longueurs `1e16 ; 1 ; 1` : `1e16` ; `1 ; 1 ; 1e16` : `1e16 + 2` ;
  `0,1 ; 0,2 ; 0,3` : `0,6000000000000001` (`0x3FE3333333333334`) ; `0,3 ; 0,2 ; 0,1` : `0,6` ; un tube mort n'entre
  pas dans la somme. Source : arithmétique IEEE 754 (Python).
- **V-MES-03** : NaN jamais vif, seuil NaN : aucun tube vif ; l'infini est vif ; une conductivité négative ne l'est pas
  (pas de valeur absolue) ; un nœud seul : `(0, +0.0)` ; panique
  si le nombre de conductivités n'est pas le nombre d'arêtes. Source : doc.

## le binaire avec les tubes

Source : la doc de `src/main.rs` ; les valeurs sont celles de V-LTUB-02, V-LTUB-03, V-LTUB-04 et V-LTUB-10.

- **V-BIN1-01** : `grille 3 2`, source `(0,0,0)`, puits `(2,1,0)`, cinq pas de 0,3, graine 1 : code 0, trois fichiers
  et aucun autre ; `empreinte.txt` = `39fe16f1…4f2e` puis `\n` ; `mesures.csv` = le texte de V-LTUB-03 ; `apercu.png`
  de 40 × 24, SHA-256 des octets RVB `4bebc2d4…58cb`.
- **V-BIN1-02** : un calcul qui s'arrête : code 1, un message sans `panicked` qui contient `pas <t>` (`pas 2`, `pas 7`,
  `pas 13`, `pas 0` : les cas de V-LTUB-10), rien d'écrit, un dossier de sortie absent n'est pas créé ; arrêté avant le
  pas fautif, le lancer réussit (trois fichiers, deux lignes de mesures).

## Ce que ces valeurs n'établissent pas

Voir la section « Ce que cette étape n'établit pas » de `MODELE.md` (étape 1).
