# Étape 2, partie 2M (la mémoire dans la forme) : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de la partie 2M attend, d'où elle vient, avec quelle tolérance et quel
critère de réussite. Chaque identifiant (`V-LOI-01`, `V-EMP-01`, `V-TEN-01`, `V-FLX-01`, `V-DEU-01`, `V-LAN-01`…) est
cité dans la doc du test qui l'emploie (`tests/etape_2m_*.rs`). Elle est séparée des résultats figés de notre propre
code (`references/*.empreinte`, étiquetés « régression » dans les tests) : **aucune valeur de cette fiche ne vient de
notre code**. Une valeur y vient d'une publication lue, d'un calcul à la main ou par un outil indépendant (nommé), d'une
construction naïve écrite dans le test, d'une **mesure sur des graines autres que celles du test** (dite telle, avec son
nombre de graines), ou d'une convention que la documentation d'un module fixe (dite telle). Les tests de cette partie
sont tous « juste » sauf `regression_reference_souvenir`.

**Le modèle de cette partie est le nôtre**, et son résultat n'était pas connu d'avance : les tests le constatent avec la
marge dite, **négatif compris** (l'empreinte s'efface sous la loi linéaire ; deux stimuli ne se cumulent pas). Aucun
chiffre publié n'est visé.

Écrite avant les calculs : les critères d'abord, les mesures ensuite (voir « Ce qui a été fixé avant de mesurer »). La
suite complète (`cargo test`, avec la fonctionnalité `etape-2m` dans `default`) dure environ une demi-minute de tests
avec les 275 tests d'avant.

## Les outils indépendants

Python 3.13 (flottants IEEE 754, `hashlib`) ; le code Rust n'est jamais l'outil des valeurs exactes. Gardés avec les
notes de la session qui a écrit ces tests, dans `outils-2M/` :

- `ref_alea.py`, `ref_graphe.py`, `ref_solveur.py`, `ref_tubes.py`, `ref_labyrinthe.py`, `ref_reseaux.py`,
  `ref_souvenir.py` : repris des étapes d'avant (le générateur, la grille, le solveur, le pas des tubes, les murs, la
  loi et le tirage de la paire, le transport de l'agent) ;
- **`ref_memoire.py`** : écrit d'après la seule doc de `souvenir` (« La mémoire dans la forme ») et de `lancer` : le pas
  couplé, les simulations des deux familles, l'écriture canonique `PHYSAR03`, les mesures en CSV, l'aperçu coloré, la
  moyenne d'une moitié et la coupe de la sonde. Il réemploie `ref_souvenir.avancer` pour le transport ;
- `valeurs_2m_a.py` à `valeurs_2m_d.py` : impriment les valeurs écrites dans les tests (`valeurs_2m_b.py` rejoue en
  Python l'écart d'empreinte et la sonde pour deux graines : **les valeurs de Rust sont celles de Python à `1e-15`**,
  voir plus bas) ;
- `mesures/campagne_2m.rs` : la campagne de 50 graines (101 à 150), qui emploie l'aide des tests
  (`tests/common/memoire_2m.rs`, mêmes définitions que dans les tests), copiée dans `examples/` le temps de la mesure ;
  `mesures/mes_2m_a.rs` à `mes_2m_d.rs` : les explorations qui ont précédé ; `mutants_2m.py` : le banc de fautes.

Dans les tests, deux constructions naïves jouent le même rôle : le pas couplé écrit à la lettre de la doc
(`common/memoire_2m.rs`, `naif_pas_couple` : le solveur, le transport naïf de 2T par arête, la recette de la loi), et la
sonde et les moyennes de moitié écrites dans le test.

## Les sources lues (et ce qui ne l'est pas)

Les lectures sont celles de `references-etape-2.md` (notes du pilote, lues par un lecteur Sonnet) ; **aucune valeur
n'est ajoutée de mémoire**, et l'article n'a pas été relu par l'écrivain de cette partie.

- **Kramar et Alim (2021)**, PNAS 118(10), e2007815118 : lu : l'article (6 pages) sans son supplément. Lu : le mécanisme
  (un agent soluble libéré au stimulus, porté par le flux, assouplit la paroi des tubes, qui se dilatent) ; le **modèle
  d'un seul tube** ; l'éq. (2), la saturation du module de la paroi `E = E_0 - ΔE c / (c_0 + c)` (symboles grecs
  rétablis par le sens, **à comparer à l'image de la page** avant de s'y fier) ; l'**expérience sur un réseau vivant**
  (« un anneau de tubes épais autour de la source consommée a imprimé la position du stimulus » ; dilatation jusqu'à un
  facteur 2 ; ce qui compte est la distance **de trajet par le flux**). **Non lu / non donné** : le **supplément** (Fig.
  S1 à S13, dont S8 « deux stimuli ») ; le pas de temps, le maillage, la largeur du profil, l'unité de `c`, `τ` ;
  **aucune durée de persistance chiffrée** au-delà de 45 minutes d'expérience (et de la figure à 310 min) ; **aucune
  simulation sur un graphe** ; l'effet de deux stimuli n'est dit qu'en mots (« les mémoires stockées dans la hiérarchie
  sont posées les unes sur les autres, chaque nouveau stimulus renforçant ou affaiblissant les tubes épais existants »).
- **Bhattacharyya, Zwicker, Alim (2023)**, Phys. Rev. E 107, 034407 : lu, **sans agent** (un autre modèle : exposants
  1/3 et 2/3 sur des flux moyennés, mémoire = des liens qui s'éteignent) : non employé ; ses exposants non entiers
  seraient interdits par `AGENTS.md`.
- **Ce qui est à nous** (aucun article) : le couplage de l'agent à la croissance (la cible `(1 + α σ) f(|Q|)`) ; la
  concentration d'une arête (la moyenne de ses bouts) ; la mise à jour avec les concentrations d'**avant** le transport
  ; les deux familles servies par une seule fonction ; le stimulus (un apport à un nœud) ; le banc de mesure (la grille,
  les moitiés, `E`, la sonde) ; le seuil `0.01` ; les nombres `α`, `c_0`, `κ`, `k`, `I0`, `h`.

## Ce qui a été fixé avant de mesurer

Écrit avant la campagne : (1) **la mesure** : `E = (Dm_haut - Dm_bas)(stimulus) - (Dm_haut - Dm_bas)(témoin)`, le témoin
ayant la même graine (mêmes tubes de départ, mêmes paires) et pas de stimulus ; (2) **le seuil de l'empreinte** : `0.01`
sur `E` ; (3) **les graines de mesure** : 101 à 150 (les tests jouent 1 à 6) ; (4) **le réglage standard** : sigmoïde,
`I0 = 3`, `α = 8`, `c_0 = 1`, `κ = 0.1`, `k = 0.2`, `h = 0.1`, stimulus de débit 2 en `(7, 2)` sur les pas `[0 ; 100[`,
grille 15 × 11 à huit voisins, source `(1, 5)` et puits `(13, 5)`. Le réglage standard a été **choisi après un
balayage** (`α` de 1 à 16, `c_0` de 0,3 à 3, `κ` de 0,03 à 0,3, `k` de 0,01 à 0,2, `I0` de 1 à 6, voisinage 4 ou 8, loi
linéaire ou sigmoïde, famille 0 ou 1 : voir « Le balayage ») comme celui où l'empreinte tient le plus et le plus
régulièrement ; les autres réglages sont donnés à côté, **y compris ceux où elle ne tient pas**.

Un pire cas mesuré sur 50 graines **n'est pas une borne** pour d'autres graines : le test pose son seuil sous le pire
des 50 avec une marge dite, et une graine des tests (la graine 2 de `V-FLX-01`) est tombée sous le pire des 50 ; le
seuil du test est alors plus bas que le pire des 50, et dit pourquoi.

---

## la loi couplée et le pas (`etape_2m_loi.rs`)

Source : la doc de `souvenir` (« La loi couplée », « Le pas couplé, à la lettre », « Les vérifications du pas couplé »),
`ref_memoire.py` (bits) et des calculs à la main. Tolérance : aucune (bits), sauf mention.

- **V-LOI-01** : avec `α = 0`, le pas couplé est celui de `Tubes::avancer` (loi linéaire) et de `Reseau::avancer_paire`
  (sigmoïde), et le transport est celui d'`Agent::avancer` : mêmes bits sur cinq graphes de 12 nœuds, douze pas. Source
  : `(1.0 + 0.0 * σ) * f = f` exactement (écrit dans la doc).
- **V-LOI-02** : un tube seul (longueur 2, `D = 0.8`, agent `[1, 0]`, `α = 3`, `c_0 = 1`, `κ = k = 0`, `I0 = 1`, `h =
  0.25`) : `Q = 1.0` exactement ; `D` après 1, 2, 3 pas : `0x3FF199999999999A`, `0x3FF5333333333334`,
  `0x3FF7BAD3560F4046` (Python) ; concentrations `[0.75, 0.25]`, `[0.5625, 0.375]`, `[0.421875, 0.421875]` (fractions
  exactes).
- **V-LOI-03** : la saturation : mêmes données, agent `(c, c)` : `D'` pour `c = 0, 0.25, 1, 4, 10^6` :
  `0x3FEB333333333334` (= le pas sans agent), `1.0`, `0x3FF399999999999A` (`σ = 1/2`), `0x3FF7333333333334`,
  `0x3FF99998D0460B43` (Python) ; `< 1.6` (la cible `σ = 1`), strictement croissant.
- **V-LOI-04** : la concentration d'une arête est la moyenne de ses bouts : chaîne `0-1-2-3`, agent `2` au nœud 1 ou au
  nœud 2 seul : les arêtes qui touchent le nœud portent l'effet à `σ(1)`, l'autre est inchangée aux bits (Python).
- **V-LOI-05** : les conductivités se mettent à jour avec les concentrations d'avant le transport : un apport de 50 ne
  change pas le premier pas (bits du pas sans agent), change le second (Python : `0x3FF971E0387F1E04`,
  `0x3FF88B79D218B79F`, `0x3FEA99999999999A`).
- **V-LOI-06** : le pas couplé contre `naif_pas_couple` (écrit dans le test d'après la doc) : quatre graphes, deux lois,
  dix pas, `h = 0.9` et `I0 = 6` (le transport se découpe : au moins 2 sous-pas, vérifié), bits de `D`, `c`, des
  pressions, des flux et du bilan.
- **V-LOI-07** : le plancher : une arête pendante (agent aux deux bouts, `α = 8`) tombe **exactement** à `PLANCHER` en
  60 pas de `h = 0.5` ; son flux reste sous `1e-12` ; aucune conductivité sous `PLANCHER` ; les arêtes du chemin sont
  `>= 1`.
- **V-LOI-08** : les erreurs du pas couplé et leur ordre (`Taille`, `Noeud`, `Debit`, `Pas`, `Force`, `Saturation`,
  `Solveur`, `Agent(...)`, `Debordement`), deux fautes cumulées, rien ne change sur une erreur.
- **V-LOI-09** : les erreurs de construction (`Taille` avant `Conductivite(e)`, `PLANCHER` accepté), `au_hasard` : les
  bits de `Tubes::au_hasard`, le même état du générateur, un agent à zéro.

## l'empreinte se forme (`etape_2m_empreinte.rs`)

Source : **mesure** sur 50 graines (101 à 150) par `mesures/campagne_2m.rs` ; les tests jouent les graines 1 à 6. `E`,
`F_A`, `F_B` : voir « Ce qui a été fixé avant de mesurer ».

- **V-EMP-01** : à la fin du stimulus (100 pas), réglage standard : `E` moyen `+0.0890` (écart-type `0.0177`, de
  `+0.0585` à `+0.1281`), `F_A` de `+0.0585` à `+0.1184`, `F_B` de `-0.0254` à `-0.0000` ; stimulus symétrique : `E` de
  `-0.1246` à `-0.0501`. Graines 1 à 6 : moyenne `0.0954`, pire `0.0694`. Seuils : chaque graine `E >= 0.04` (marge
  `1.46`), `F_A >= 0.04`, `F_B <= 1e-9`, `E <= -0.03` (symétrique) ; `E` moyen `>= 0.07`.
- **V-EMP-02** : `α = 0` : conductivités avec stimulus = témoin aux bits (donc `E = 0`), à 100 et 400 pas ; la masse
  d'agent `> 1` à 100 pas (le stimulus a eu lieu).
- **V-EMP-04** : Rust et Python donnent les mêmes `E`, `F_A`, `F_B` et la même sonde (modèle `ref_memoire.py`,
  `valeurs_2m_b.py`) : réglage standard, graines 1 et 2 à 100 et 400 pas ; linéaire, graine 1 à 100, 300 et 400 pas ;
  coupe de la sonde de `(7, 5)` à `(13, 5)` à 400 pas pour les graines 1 et 2 avec et sans stimulus ; tolérance `1e-13`
  (mêmes opérations dans le même ordre).
- **V-EMP-03** : `E` moyen à 100 pas selon `α = 0, 2, 4, 8, 16` : `0`, `0.0131`, `0.0424`, `0.0890`, `0.1430` (50
  graines) ; graines 1 à 6 : `0`, `0.0089`, `0.0464`, `0.0954`, `0.1438` ; chaque pas `>= 0.005`. À 400 pas : `E(16) -
  E(8)` vaut `0.0017` (50 graines : `0.0383` contre `0.0366`) et `0.0036` (graines 1 à 6) ; seuil `<= 0.01`.

## l'empreinte tient-elle ? (`etape_2m_tenue.rs`)

Source : **mesure** sur 50 graines (101 à 150) ; les instants sont des nombres de pas (le stimulus finit au pas 100).

- **V-TEN-01** (sigmoïde, réglage standard) : `E` à 100, 200, 300, 400, 600, 1100 et 3100 pas : `+0.0890`, `+0.0379`,
  `+0.0367`, `+0.0366`, `+0.0366`, `+0.0366`, `+0.0366` (écart-type de `E(400)` : `0.0193` ; pire `+0.0079`) ; part des
  graines `> 0.01` : `1.00`, `0.98`, `0.96`, `0.96`… ; `|E(1100) - E(400)|` au plus `1.9e-4`. Graines 1 à 6 : `E(400)`
  moyen `0.0453`, pire `0.0197`, plateau à `5e-5`. Seuils : `E(400) >= 0.003` (marge `2.6` sur le pire des 50), moyenne
  `>= 0.015`, au moins 4 graines sur 6 au-dessus de `0.01`, `|E(1100) - E(400)| <= 5e-4` (marge `2.6`).
- **V-TEN-02** (l'agent n'est pas la mémoire) : masse d'agent à 400 pas : `0.0054` en moyenne, `0.0080` au pire ; avec
  l'agent effacé au pas 100 : `E(400)` moyen `+0.0364` (`94 %` au-dessus du seuil contre `96 %`) ; un écart de `D`
  jusqu'à `0.69` sur une arête pour l'une des 50 (la graine change de destin). Seuils : masse `<= 0.02` (marge `2.5`),
  `E` moyen effacé `>= 0.015`.
- **V-TEN-03** (linéaire, `I0 = 1`, `α = 4`, `c_0 = 1`, `k = 0.2`) : `E` à 100, 200, 300, 400, 600, 1100 pas :
  `+0.0368`, `+0.0233`, `+0.0130`, `+0.0069`, `+0.0019`, `+0.0001` ; presque indépendant de la graine (de `+0.0357` à
  `+0.0383` à 100 pas ; de `+0.0062` à `+0.0079` à 400 pas : **sous le seuil pour les 50 graines**). **Durée au-dessus
  du seuil : entre 200 et 300 pas après le retrait.** Seuils : `E(100) >= 0.033`, `E(300) >= 0.010`, `E(400) < 0.01`,
  `E(600) <= 0.003`, `E(1100) <= 0.0003`, la suite décroît strictement.
- **V-TEN-04** (linéaire, `k = 0.01`) : `E(600) = +0.0123` (de `+0.0111` à `+0.0130`), `E(1100) = +0.0013` ; masse
  d'agent `6.8` puis `2.8`. Seuils : `E(600) >= 0.010`, `E(1100) <= 0.002`, masse à 600 `>= 3`, et avec `k = 0.2`
  `E(600) <= 0.003`.
- **V-TEN-05** (un stimulus sur `[100 ; 200[`) : à la fin `E` moyen `+0.0057` (au pire `+0.0361`, `32 %` au-dessus du
  seuil), à +300 : `|E|` au plus `4.7e-5`. Seuils : `E <= 0.04` à la fin, `|E(500)| <= 2e-4`.
- **V-TEN-06** (famille des nourritures, deux nourritures, paire tirée) : sigmoïde `E(400)` `+0.0372` (`+0.0077` à
  `+0.0740`, `96 %`) ; linéaire `E(100) = +0.0369`, `E(400) = +0.0072` (`+0.0065` à `+0.0082`), `E(1100) = +0.0001`.
  Graines 1 à 6 : sigmoïde `0.0487` (pire `0.0223`) ; linéaire `0.0365`, `0.0070`, `0.0001`.

## où part le flux d'une nouvelle source (`etape_2m_flux.rs`)

Source : **mesure** sur 50 graines ; la sonde est définie dans la doc de `souvenir` et dans `common/memoire_2m.rs` ;
`haut - bas` est la différence des parts de la coupe (colonnes 10 et 11) du flux de la sonde de `(7, 5)` à `(13, 5)`,
mesurée à 400 pas.

- **V-FLX-01** : stimulus en haut : `+0.724` en moyenne (de `+0.161` à `+1.000` : **positif pour chaque graine**) ;
  stimulus symétrique : `-0.665` (négatif pour 49 graines sur 50, `+0.089` pour l'autre) ; témoin : `+0.011` (de
  `-0.326` à `+0.326`) ; différence avec le stimulus symétrique : `+1.389` (de `+0.656` à `+1.709`) ; avec le témoin :
  `+0.713` (de `+0.121` à `+1.228`). Graines 1 à 6 : `+0.541`, `-0.561`, `-0.151` ; différences `+1.102` (**pire
  `+0.333`, sous le pire des 50 : le pire des 50 n'est pas une borne**) et `+0.692` (pire `+0.152`). Seuils : chaque
  graine `haut - bas >= 0.05` (marge `3.2`), différence avec le stimulus symétrique `>= 0.1`, avec le témoin `>= 0.05` ;
  moyennes : `<= -0.3` (stimulus symétrique), `>= 0.6` (différence).
- **V-FLX-02** : la sonde : `haut + axe + bas = 1` à `1e-9` (mesuré : `2.4e-14` au pire sur 50 graines) ; tubes égaux :
  `|haut - bas| <= 1e-9` (mesuré `2.2e-16` ; parts `0.4332`, `0.1336`, `0.4332`), chaque part `> 0.05`. Source : la
  conservation du flux ; la symétrie de la grille.

## deux stimuli (`etape_2m_deux.rs`)

Source : **mesure** sur 50 graines, à 400 pas. **Le supplément de l'article (Fig. S8) n'a pas été lu : aucun chiffre
publié n'est visé.**

- **V-DEU-01** (même côté) : un seul stimulus : `F_A = +0.0298`, sonde `+0.724` (de `+0.161` à `+1.000`) ; `(5, 2)` et
  `(9, 2)` : `F_A = +0.0319` (`+7 %`), sonde de `+0.675` à `+0.748` (moyenne `0.693`) ; `(7, 2)` et `(7, 3)` : `F_A =
  +0.0276` (`-7 %`), sonde de `+0.326` à `+0.849`. Seuils : rapport des `F_A` moyens `[0.8, 1.3]`, sonde `>= 0.5` par
  graine, `F_B <= 1e-9`.
- **V-DEU-02** (côtés opposés) : `(7, 2)` et `(7, 8)` : `F_A = +0.0145`, `F_B = +0.0165` (somme `+0.0310`), `E =
  -0.0020` (de `-0.0386` à `+0.0435`), sonde `haut - bas = +0.021` (de `-0.778` à `+0.807`), `|haut - bas| = 0.260`.
  `(4, 2)` et `(10, 8)` : `E = +0.0129`, `|haut - bas| = 0.402` (non testé). Seuils : `F_A` moyen `<= 0.8` fois celui
  d'un stimulus seul (le rapport vaut `0.49`), `F_A` et `F_B` moyens `>= 0.005`, `|E|` moyen `<= 0.025`, `|haut - bas|`
  moyen `<= 0.6` fois celui d'un stimulus seul (`0.36`).
- **V-DEU-03** (un second stimulus tardif, `[300 ; 400[` en bas) : `|F_B(deux) - F_B(un)| = 0` pour les 50 graines ;
  `|F_A(deux) - F_A(un)|` au plus `1.3e-2` ; `4 %` des graines changent une arête de plus de `1e-3` (jusqu'à `0.46`).
  Graines 1 à 6 : `F_A` à `9e-5` près. Seuils : `|ΔF_B| <= 1e-9`, `|ΔF_A| <= 0.03`.
- **V-DEU-04** : `α = 0` : un ou deux stimuli ne changent aucun bit des conductivités ; la masse augmente avec le nombre
  de stimuli.

## lancer avec un agent (`etape_2m_lancer.rs`)

Source : `ref_memoire.py` (le modèle Python de `lancer`), la doc de `lancer` (« Ce que fait l'étape 2, partie 2M ») et
de `graphe` (l'écriture du graphe), `valeurs_2m_a.py` et `valeurs_2m_d.py`. Tolérance : aucune (octets).

- **V-LAN-01** : `PETIT_A` (tubes, grille 3 × 1, un stimulus, graine 7) : 246 octets, SHA-256
  `161d68ce63d7356ee6a0a7290349dd44b5c805754d5c9822f464106985ecb590` ; conductivités `3FEF4DB7298DF632`,
  `3FEA6CA68E9A9B05` ; concentrations `3F83333333333334`, `3FE2E66666666666`, `3FBA666666666666` ; générateur
  `fd6eabdbbde5d2816d00000000000000`.
- **V-LAN-02** : `PETIT_B` (réseau, trois nourritures, deux stimuli au même nœud, graine 3) : 270 octets, SHA-256
  `2c3513138cc80f617ce5917d9a63c1e311a046d946c474d202714db5cf3b0771` ; conductivités `3FF04BD84A2A441E`,
  `3FEDBE5E7A4817F1` ; concentrations `3FCFFFFFFFFFFFFF`, `3FDFFFFFFFFFFFFF`, `0` ; générateur
  `b550126546a0931b6d00000000000000`.
- **V-LAN-03** : les mesures en CSV : `PETIT_A`, `PETIT_B` en entier ; `souvenir.txt` (graine 1) : 401 lignes, SHA-256
  `3421b02ddf232be39ebce59a31d426024b6d6295ed71caa4ba92d28f2ec543e6`, six lignes écrites ; `TUBES_MUR` (graine 5) :
  `008bb8ad…40a7` ; la force nulle : les colonnes d'avant la masse sont celles du scénario sans agent.
- **V-LAN-04** : les empreintes : `souvenir.txt` graine 1
  `ff1fa667fb9a3a6025a36f1cd2e3176e7a3f81e7f7d093e4410208584b7cf7c5`, graine 2
  `c46c8bf24bddbcd0c2b5ee7623a9a70ffb9005cc1da1f745417455fd34b96652` ; `TUBES_MUR` graine 5 `8ee19be5…491b` ;
  `RESEAU_ETAGES` graine 4 `988d699f…1bf9` ; sans stimulus `c8e3662c…7e70` ; force nulle `e1ad6548…cbb63`.
- **V-LAN-05** : les aperçus (SHA-256 des octets RVB) : `PETIT_A` `9f0cda50…7175`, `PETIT_B` `017246a5…ed8e`,
  `souvenir.txt` `983440d1…426c` (232 × 168), `TUBES_MUR` `e629395a…3e6b`, `RESEAU_ETAGES` `08a24f72…83deb`, sans
  stimulus `ea754c69…a50b` ; cinq points écrits à la main pour `PETIT_A` et `PETIT_B`.
- **V-LAN-06** : les erreurs de directives : 60 cas (formes, bornes, doublons, priorité, murs, deux familles) ; les
  lignes sont celles de la règle de la doc.
- **V-LAN-07** : un calcul qui s'arrête : `Calcul(4)` (linéaire) et `Calcul(3)` (sigmoïde) pour un lien de coût `10^16`
  avec la force nulle (les pas de `etape_1r_lancer`).
- **V-LAN-08** : les scénarios du dossier (la règle, jamais la liste) : anciens (`chemin.txt` `6e3b15b8…2f37`,
  `labyrinthe.txt` `dcdad079…3ded`, `etages.txt` `d011d6bc…df76`, `grille.txt`) ; un petit cas de tubes du cœur.
- **V-LAN-10** : l'ordre des stimuli du fichier : `grille 3 1`, tubes, `agent 2.0 1.0 0.1 0.5`, `pas 3 0.3`, graine 7,
  deux stimuli au nœud 1 (débit `0.1` sur `[0 ; 3[` et `0.2` sur `[1 ; 3[`) : `5f8f00175fc1315f…2bc0` dans cet ordre,
  `067a86287e06d44b…d468` dans l'autre (la somme de flottants dépend de l'ordre ; Python).
- **V-LAN-09** : le binaire : code 0 et `empreinte.txt`, `mesures.csv` (401 lignes), `apercu.png` (232 × 168) ; un
  stimulus sous un mur : code 1, `ligne 3`, rien d'écrit.

## Le balayage (50 graines, réglage standard sauf mention, `E` à +300 pas : moyenne, pire, part au-dessus du seuil)

Un balayage a précédé le choix du réglage (`outils-2M/mesures/balayage_*.txt`). Sigmoïde, `I0 = 3`, `k = 0.2` : `α = 1`
: `+0.0063`, `0.20` ; `α = 2` : `+0.0106`, `0.32` ; `α = 4` : `+0.0219`, `0.72` ; **`α = 8` : `+0.0366`, `0.96`** ; `α =
16` : `+0.0383`, `0.98` ; `α = 8` avec `c_0 = 0.3` : `+0.0392`, `0.96` ; avec `κ = 0.03` : `+0.0360`, `0.94` ; avec `κ =
0.3` : `+0.0315`, `0.94` ; avec `k = 0.01` : `+0.0420` (l'agent reste) ; débit `I0 = 2` : `+0.0262` ; `4` : `+0.0440` ;
`6` : `+0.0246` (avec `α = 4`) ; stimulus de débit `6` : `+0.0391`. Voisinage 4 (`α = 4`) : `+0.0096`, `0.18`.
**Linéaire** : `α = 4` : `+0.0069` à +300 (sous le seuil) ; `α = 10`, `c_0 = 0.3` : `+0.0089` à +500, `+0.0004` à +1000
; `α = 30`, `c_0 = 0.3`, `k = 0.01` : `+0.1679` à +300 (loin au-dessus du seuil) puis `+0.0894` à +500 et `+0.0249` à
+1000 : **même avec une très grande force, l'empreinte linéaire finit par s'effacer**. Stimulus dans `[100 ; 200[` :
`+0.0000`.

## la campagne de fautes du juge (`etape_2m_campagne.rs`)

Source : la doc de `souvenir` et de `lancer` ; les valeurs se calculent à la main (écrites dans le commentaire de
chaque test). Chaque test ferme une faiblesse trouvée par la campagne (une faute plausible que les autres tests
laissaient passer). Critère : égalité.

- **V-CAM-01** : `Memoire::avancer` sur un graphe d'une arête de plus, d'une de moins, d'un nœud de plus, d'un de
  moins que la mémoire : `Taille`, mémoire inchangée ; sur son graphe, le même pas passe.
- **V-CAM-02** : une source ou un puits égal au nombre de nœuds : `Noeud`, mémoire inchangée.
- **V-CAM-03** : tubes, source égale au puits et un `stimulus` sans `agent` : la ligne du `puits` (3), pas celle de la
  première directive (2).
- **V-CAM-04** : réseau avec une `source` et un `stimulus` sans `agent` : la ligne de la `source` (4), pas 2.
- **V-CAM-05** : l'aperçu avec agent : grilles 40 × 2, 130 × 1, 3 × 40 : PNG de 474 × 18, 259 × 1, 40 × 632.
- **V-CAM-06** : tubes avec agent (`α = 0`), lien entre étages de `10^16` : cinq pas passent, six ou sept rendent
  `Calcul(5)`, comme sans agent.
- **V-CAM-07** : avec agent, puits muré : ligne 4 ; réseau, 2e nourriture murée : ligne 4, 1re : ligne 3.

## Ce qui n'a pas été mesuré

- **Linux** : toutes les mesures sont faites sous Windows ; les opérations sont `+ - * /` et `sqrt`, donc les mêmes bits
  attendus ; la CI le dira.
- **Une autre grille** : un seul banc (15 × 11, huit voisins, un étage) ; le voisinage 4 donne une empreinte plus faible
  (`+0.0096`, `0.18` avec `α = 4`) ; les étages et les murs ne sont éprouvés que par `lancer` (empreintes).
- **Le grand nombre de stimuli** (plus de deux) et les stimuli décalés dans le temps (un seul cas, `V-DEU-03`).
- **La sensibilité à `h`** : `h = 0.1` partout (le pas du transport se découpe seul ; celui des tubes est fixé par la
  loi).
