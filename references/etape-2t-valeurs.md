# Étape 2, partie 2T (le transport de l'agent) : les valeurs attendues, leur source, leur tolérance

Cette fiche dit, pour chaque valeur qu'un test de la partie 2T attend, d'où elle vient, avec quelle tolérance et quel
critère de réussite. Chaque identifiant (`V-AGT-01`, `V-REC-01`, `V-MAS-01`, `V-STA-01`, `V-BIT-01`, `V-TAC-01`,
`V-ERR-01`, `V-APE-01`…) est cité dans la doc du test qui l'emploie (`tests/etape_2t_*.rs`). Elle est séparée des
résultats figés de notre propre code (`references/*.empreinte`, étiquetés « régression » dans les tests) : **aucune
valeur de cette fiche ne vient de notre code**. Une valeur y vient d'une publication lue, d'un calcul à la main ou par
un outil indépendant (nommé), d'une construction naïve écrite dans le test, ou d'une convention que la documentation
d'un module fixe (dite telle). Les tests de cette partie sont **tous « juste »** : aucune référence d'empreinte nouvelle
(`lancer` ne change pas).

Écrite avant les calculs : les critères d'abord, les mesures ensuite. La suite complète (`cargo test`, avec la
fonctionnalité `etape-2t` dans `default`) dure une vingtaine de secondes avec les 233 tests d'avant (275 tests en tout).

## Les outils indépendants

Python 3.13 (flottants IEEE 754, `fractions`, `hashlib`) ; le code Rust n'est jamais l'outil. Gardés avec les notes de
la session qui a écrit ces tests, dans `outils-2T/` :

- `ref_alea.py`, `ref_graphe.py`, `ref_solveur.py`, `ref_tubes.py` : repris du cœur (PCG32, la grille, le solveur, le
  pas des tubes, l'aperçu coloré) ;
- `ref_souvenir.py` : écrit d'après la seule doc de `souvenir` : le taux de départ, le pas de la recette, le pas maximal
  (`math.nextafter`), le nombre de sous-pas (par essai de chaque `n`), le pas public découpé, le même pas en fractions
  exactes (`pas_exact`), le treillis coloré avec les nœuds mêlés à `AGENT`. **La structure diffère exprès** du code Rust
  : ici une boucle par arête, là une boucle par nœud ;
- `valeurs_2t_a.py`, `valeurs_2t_b.py`, `valeurs_2t_apercu.py` : impriment les valeurs écrites dans les tests ;
  `chercher_etoile_2t.py`, `chercher_apports_2t.py`, `chercher_limite_2t.py` : cherchent les cas où l'ordre des sommes
  change les bits et où le pas maximal n'est pas `1 / r_max` ; `mesures/` : les programmes de mesure des tolérances
  (`mes_2t_a.rs` à `mes_2t_d.rs`, copiés dans `examples/` le temps de la mesure) ; `mutants_2t.py` : le banc de fautes.

Dans les tests, trois constructions naïves jouent le même rôle : le pas de la recette par une boucle **sur les arêtes**
(`common/transport_2t.rs`, `naif_taux`, `naif_pas`), le pas maximal par montée ou descente d'un cran, le nombre de
sous-pas par essai de chaque `n`, et une recette à variation signée (`etape_2t_stabilite.rs`).

## Les sources lues (et ce qui ne l'est pas)

Les lectures sont celles de `references-etape-2.md` (notes du pilote, lues par un lecteur Sonnet) ; **aucune valeur
n'est ajoutée de mémoire**, et l'article n'a pas été relu par l'écrivain de cette partie.

- **Kramar et Alim (2021)**, PNAS 118(10), e2007815118 : lu : l'article (6 pages) sans son supplément. Lu : le mécanisme
  (« un agent chimique soluble est libéré dans le cytoplasme à l'endroit du stimulus ; il assouplit la paroi du tube ;
  cet effet se propage dans le réseau quand l'agent est advecté par les flux cytoplasmiques ») ; le modèle **d'un seul
  tube** (advection-dispersion de Taylor avec une décroissance `k_deg = 0,001 s⁻¹`, `κ = 10⁻¹⁰ m²/s`, libération unique)
  ; l'observation **sur un réseau vivant** (réorganisation en 45 min, dilatation jusqu'à un facteur 2, front à 15 µm/s,
  plus vite que la diffusion). **Non lu / non donné** : le supplément (Fig. S1 à S13, dont S8 : deux stimuli) ; le pas
  de temps, le maillage, le paramètre de Crank-Nicolson, la largeur du profil gaussien, l'unité de la concentration ;
  l'exposant de la vitesse dans le coefficient de dispersion de Taylor (symboles grecs perdus à l'extraction du texte).
  **Aucune simulation sur un graphe** n'y figure : le transport sur un graphe est à nous.
- **Bhattacharyya, Zwicker, Alim (2023)**, Phys. Rev. E 107, 034407 : lu, **sans agent** : hors sujet de cette partie
  (la capacité de mémoire, pour 2M).
- **Ce qui est à nous** (aucun article) : les nœuds comme chambres de volume 1 ; le schéma amont explicite ; la
  diffusion à un seul coefficient (la dispersion de Taylor n'est pas reprise) ; les bords déduits des flux (`s_i`) ; la
  perte linéaire ; les apports ; le pas borné par `h * r_i <= 1` et son découpage ; la couleur `AGENT` ; le `κ`, le `k`
  et les pas des tests (aucun n'est une valeur publiée).

---

## l'état (`etape_2t_agent.rs`)

Source : la doc de `souvenir` (« L'état »). Calcul exact.

- **V-AGT-01** : `Agent::zero` : une concentration `+0.0` (bits nuls) par nœud ; masse `+0.0`. Un nœud sans arête,
  quatre nœuds.
- **V-AGT-02** : `Agent::new` garde les valeurs aux bits, `-0.0` compris ; `PartialEq`.
- **V-AGT-03** : `masse` : somme de gauche à droite à partir de `0.0` : `[0.1, 0.2, 0.3]` donne `0.6000000000000001` ;
  `[1, 1e-16, 1e-16]` donne `1.0` ; `[1e-16, 1e-16, 1]` donne `1.0000000000000002`. Source : calcul des flottants
  (Python).
- **V-AGT-04** : erreurs de `new` : `Taille` avant `Concentration(i)` (le plus petit) ; NaN, `±inf`, `-1e-300` refusés ;
  `0.0`, `-0.0`, `f64::MAX` acceptés.

## la recette (`etape_2t_recette.rs`)

Source : la doc de `souvenir` (« Le pas, à la lettre »), des calculs à la main (cas dyadiques, exacts) et
`ref_souvenir.py`. Tolérance : aucune (bits). Chaque cas est joué par `pas_sans_decoupage` et par `Agent::avancer` (un
sous-pas).

- **V-REC-01** : l'advection seule sur la chaîne `0-1-2-3`, flux `0.5` : taux `[0.5; 4]`, pas maximal `2`, cinq pas de
  `h = 1` de `c = [1, 0, 0, 0]` à la main (`[0.5, 0.5, 0, 0]`, … `[0.03125, 0.15625, 0.3125, 0.3125]`), sortie du puits
  `0.0625` puis `0.125` ; le sens inverse (flux `-0.5`) : le miroir exact.
- **V-REC-02** : la diffusion seule (chaîne de longueurs 1, `κ = 0.5`, `h = 0.5`) : quatre pas à la main ; et sur des
  longueurs `1, 2, 4` (coûts triples) : la diffusion lit la longueur, jamais le coût ; taux `[0.5, 0.75, 0.375, 0.125]`,
  pas maximal `0x3FF5555555555556`.
- **V-REC-03** : tout ensemble, dyadique : flux `[0.5, 0.25, 0.25]`, `κ = 0.5`, `k = 0.125`, apport `0.5`, `h = 0.5` :
  trois pas, concentrations et bilan exacts (et le bilan `masse_apres = masse_avant - sortie - perte + apport` exact à
  chaque pas) ; vérifié en fractions par `ref_souvenir.pas_exact`.
- **V-REC-04** : tout ensemble, non dyadique (flux `[0.9, 0.6, 0.3]`, `κ = 0.3`, `k = 0.01`, deux apports, `h = 0.1`) :
  trois pas aux bits de `ref_souvenir.py`, taux et pas maximal `0x3FE5312A6254C4AA`.
- **V-REC-05** : l'ordre des sommes : une étoile à quatre feuilles (`chercher_etoile_2t.py`) où l'ordre des arêtes et le
  mélange de l'advection et de la diffusion dans une même somme changent `r_0` (`2.1500000000000004` contre `2.15`) et
  `c0'` (`…7D40` contre `…7D42`) : les trois ordres fautifs essayés donnent tous le second résultat.
- **V-REC-06** : la sortie à l'arrondi : `s_0 = -2.7755575615628914e-17` sur une étoile (flux `[0.3, -0.1, -0.2]`) : la
  règle de la doc s'applique sans seuil ; sortie du bilan `2^-56` (`0x3C70000000000000`).
- **V-REC-07** : les apports (deux apports sur un même nœud : l'ordre de la liste change les bits : `3.0405040000000003`
  contre `3.040504`, `chercher_apports_2t.py`), un nœud seul (`k = 0.3`, `h = 0.1`), des flux nuls sans `κ` ni `k` :
  rien ne bouge et le pas maximal est infini.

## la masse (`etape_2t_masse.rs`)

Source : la doc de `souvenir` (le `Bilan`, « Ce que l'arrondi coûte ») ; calcul exact sur des valeurs dyadiques ;
mesures.

- **V-MAS-01** : l'anneau `0-1-2-3-0` (flux qui tournent, `h = 0.5`, `c = [1, 2, 0, 0.5]`, masse `3.5`) : l'advection,
  la diffusion (`κ = 0.25`) et les deux, vingt pas chacun : masse **exactement** `3.5` à chaque pas, bilan nul,
  concentrations finales aux bits de `ref_souvenir.py` (qui vérifie en fractions que chaque pas est exact). Tolérance :
  aucune.
- **V-MAS-02** : la perte seule : un nœud, `k = 0.5`, `h = 0.5` : `0.75^n` aux bits, bilan exact, perte cumulée `1 -
  0.75^20` exacte ; deux nœuds reliés par un flux nul.
- **V-MAS-03** : le bilan sur des flux réels de tubes : 24 réseaux, quatre transports, 100 pas, à la moitié du pas
  maximal : écart par pas `<= 1e-13`, composé `<= 2.5e-13`. **Mesuré** (`mes_2t_d.rs`, 40 graines autres, 960 couples
  réseau-transport) : `3.0e-14` et `6.4e-14` au pire (graines du test : `2.1e-14` et `5.7e-14`) ; facteurs 3,3 et 3,9.
- **V-MAS-04** : tout sort : grille `5 × 4`, tubes uniformes, `κ = 0.1`, `h = 0.5` : masse `< 1e-12` après 600 pas
  (mesuré au pas 563), `|1 - masse - sortie| <= 2e-14` (mesuré `1.6e-15`) ; sans `κ`, 300 pas (mesuré 255). **Sur un
  réseau convergé**, une masse de `1e-4` reste après 1 800 pas (fuite vers des tubes morts) : pas testé.
- **V-MAS-05** : ce que coûte la règle de la sortie aux nœuds intérieurs : le puits sort `1` à `5e-14`, les autres nœuds
  `<= 2e-14` (mesuré `5.1e-15`), la fuite sur 200 pas `<= 4e-14` (mesuré `9.2e-15`).

## la stabilité (`etape_2t_stabilite.rs`)

Source : la doc de `souvenir` (« Le taux de départ et le pas maximal », « Le pas public et son découpage »).

- **V-STA-01** : `nombre_de_sous_pas` : une table de douze cas (calculée par essai de chaque `n`), `1_000_000` accepté,
  plus grand que `MAX_SOUS_PAS` au-delà, les paniques (`h` nul, négatif, NaN, infini ; `h_max` nul, négatif, NaN).
- **V-STA-02** : le pas maximal est le plus grand flottant tel que `h * r_max <= 1` : cinq cas construits (`2`,
  `0x3FF5555555555556`, `0x400AAAAAAAAAAAAB`, `1`, et `0.09` où `1 / 0.09` est un cran trop bas : `0x402638E38E38E38F`),
  soixante graphes au hasard contre le pas maximal naïf, infini sans rien qui parte. Observation : sur deux millions de
  `r` tirés, `(1 / r) * r` n'a jamais dépassé `1` : la descente d'un cran de la doc n'a pas de cas qui la fasse servir.
- **V-STA-03** : à la limite exacte aucune concentration négative ni `-0.0` : quatre cas construits (le nœud de taux
  maximal sans apport ; la sortie seule ; la perte seule ; la diffusion seule), un cran au-dessus `c0 =
  -2.220446049250313e-16` (`0xBCB0000000000000`) ou `-4.44e-16` ; quatre-vingts cas au hasard : 0 négatif (mesuré 0 sur
  132 000), la recette à variation signée en rend 13 sur 1139 sur ces mêmes cas (204 sur 132 000 mesurés).
- **V-STA-04** : le pas public découpe : `h = 20` sur la chaîne (`n = 10`, résultat nul exact), un cas non dyadique (`h
  = 7`, `n = 8`, aux bits de `ref_souvenir.py`), trente graphes de flux de tubes à `10 * h_max` : mêmes bits que les
  sous-pas à la main et que le pas naïf ; sous la limite, un seul sous-pas.
- **V-STA-05** : `SousPas` au-delà d'un million, rien n'a changé ; un million accepté.

## les bits (`etape_2t_bits.rs`)

Source : un pas naïf du test, écrit d'après la doc (une boucle par arête). Tolérance : aucune.

- **V-BIT-01** : cent graphes au hasard, flux quelconques, `κ`, `k`, un à trois apports, un `h` au hasard :
  concentrations et bilan aux bits du pas naïf.
- **V-BIT-02** : l'agent sur les flux de `Tubes::avancer` (grille et graphe au hasard, 1, 9 puis 30 pas des tubes, `h` à
  0,7 et 3,3 fois le pas maximal, douze pas publics) : concentrations, bilan, nombre de sous-pas.
- **V-BIT-03** : l'agent sur les flux de `Reseau::avancer` sous la loi sigmoïde (cinq nourritures, flux changeant à
  chaque pas), soixante pas, six graines : mêmes bits, bilan de masse `<= 1e-13`.

## la tache (`etape_2t_tache.rs`)

Source : le mécanisme de Kramar et Alim (l'agent suit le flux) ; **aucun nombre publié n'est visé**. Nos mesures
(`mes_2t_b.rs`, 40 graines de 100 à 139, trois géométries) :

- **V-TAC-01** : lâcher sur le chemin : part de la masse sur les nœuds des tubes vifs après 30 pas de `0,1` (`κ = 0,01`)
  : convergé `0.8505` en moyenne, `0.8444` au pire (géométrie 1), `0.8588` (2), `0.8577` (3) ; uniforme `0.5948`,
  `0.5944`, `0.5846` (indépendant de la graine) ; écart au pire `0.2496`, `0.2644`, `0.2731`. Seuils `>= 0.80`, `<=
  0.62`, écart `>= 0.22` (marges `0.044`, `0.025`, `0.03`). Un réseau convergé est presque indépendant de la graine : la
  marge est géométrique.
- **V-TAC-02** : lâcher hors des tubes vifs : convergé `0.0606`, `0.0614`, `0.0614` au pire ; uniforme `0.118`, `0.111`,
  `0.107` ; seuil `<= 0.08` et convergé `<` uniforme (écart d'au moins `0.045`).

## les erreurs (`etape_2t_erreurs.rs`)

Source : la doc de `souvenir` (« Les vérifications, dans cet ordre »). Critère : égalité.

- **V-ERR-01** à **V-ERR-05** : le cas positif ; chaque erreur seule (`Taille`, `Concentration(i)`, `Flux(e)`,
  `Diffusion`, `Perte`, `Pas`, `Apport(r)`) aux deux fonctions et « rien n'a changé » ; l'ordre sur des entrées à deux
  fautes ; `Debordement` (taux infini, diffusion qui déborde, résultat infini sans découpage) et `SousPas` ; les erreurs
  de `taux_de_depart` et de `pas_maximal`.

## l'aperçu (`etape_2t_apercu.rs`)

Source : la doc de `souvenir` (« L'aperçu ») et de `tubes::apercu` ; `ref_souvenir.py` ; calcul à la main du mélange.

- **V-APE-01** : le petit réseau à deux étages, dessin 8 × 3 en caractères, échelles 1, 2, 3 ; hors des quatre nœuds
  peints, l'image est celle de `tubes::apercu`.
- **V-APE-02** : les niveaux (`0.5` donne 128, `0.49999999999999994` donne 127, `0.0019` donne 0, `0.0021` donne 1, la
  saturation, des zéros, le rapport seul).
- **V-APE-03** : deux nœuds au même point : le second mélange sur le premier (`(113, 247, 123)`).
- **V-APE-04** : trois empreintes (sha256 des octets RVB) de grands cas calculées par `ref_souvenir.py` (qui rejoue les
  tubes) : `1c0a1f3d…45d9` (grille 12 × 9 convergée avec une tache, échelle 4, `92 × 68`), `52ab293f…298b` (échelle 1,
  `23 × 17`), `68a91f1c…6017` (deux étages, `72 × 27`).
- **V-APE-05** : les paniques.

## la campagne de fautes du juge (`etape_2t_campagne.rs`)

Source : la doc de `souvenir` ; les valeurs se calculent à la main (écrites dans le commentaire de chaque test).
Chaque test ferme une faiblesse trouvée par la campagne (une faute plausible que les autres tests laissaient
passer). Critère : égalité.

- **V-CAM-01** : un taux sous-normal (perte `1e-320`, `5e-324`, `1e-310` sur un nœud seul) : le pas maximal est
  `f64::MAX`, le seul cas où la descente d'un cran sert.
- **V-CAM-02** : un `Debordement` au troisième sous-pas (chaîne `0-1`, flux `1`, `h = 2.1`, deux apports de
  `0.7 * f64::MAX`) laisse l'agent tel qu'avant l'appel ; le témoin (`h = 1.4`, deux sous-pas) réussit et finit à
  `[0.637, 0.98]` en fraction de `f64::MAX`.
- **V-CAM-03** : un `Debordement` par un seul total du bilan (l'apport, la perte, la sortie), concentrations d'après
  finies, aux deux fonctions publiques.
- **V-CAM-04** : un flux trop long rend `Taille` avant une concentration négative.
- **V-CAM-05** : tous les niveaux entiers, les moitiés (`0.5 / 255` donne 1, `2.5 / 255` donne 3) et `0.99`
  (niveau 252) sur le pixel d'un nœud.
- **V-CAM-06** : deux nœuds au même point, tous les couples de niveaux de 0 à 255 (65 536 cas) : l'arrondi `+ 127`
  du mélange sur presque toutes les couleurs de départ.
