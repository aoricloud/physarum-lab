//! L'agent porté par le flux : une concentration par nœud, advectée par les flux des tubes (étape 2, partie 2T).
//!
//! Un stimulus (de la nourriture, du sel) libère dans le blob un agent soluble ; le liquide qui circule dans les tubes
//! le porte. Ce module pose **seulement ce transport** : où va l'agent, pas ce qu'il fait aux tubes. L'effet de l'agent
//! sur les tubes (la mémoire) est la partie 2M, plus bas ; elle lit les concentrations à chaque pas
//! ([`Agent::concentrations`]) et les fait agir sur la mise à jour des tubes. Ce que les tests d'acceptation prouvent
//! est dans `references/etape-2t-valeurs.md` (identifiants `V-AGT-xx`, `V-REC-xx`, `V-MAS-xx`, `V-STA-xx`, `V-BIT-xx`,
//! `V-TAC-xx`, `V-ERR-xx`, `V-APE-xx`).
//!
//! # D'où cela vient, et ce qui est à nous
//!
//! - **Ce qui vient de Kramar et Alim** (PNAS 118(10), e2007815118, 2021) : le mécanisme (un agent chimique soluble,
//!   libéré au stimulus, **advecté par les flux cytoplasmiques**), la décroissance lente de l'agent (`k_deg = 0,001
//!   s⁻¹` dans leur modèle), une diffusivité moléculaire `κ`, et une libération qui n'a lieu qu'à l'endroit du
//!   stimulus.
//! - **Ce qui est à nous** : leur modèle est celui d'**un seul tube** élastique (advection, dispersion de Taylor,
//!   décroissance, libération unique). **Aucune source ne donne ce transport sur un graphe.** Tout le reste est un
//!   choix de ce dépôt : une concentration par **nœud** ; chaque nœud est une **chambre de volume 1** (la masse d'agent
//!   est la somme des concentrations ; aucune source ne fixe un volume de nœud) ; l'advection en **schéma amont**
//!   explicite ; une diffusion à **un seul coefficient** `κ` (la dispersion de Taylor, `κ + a² ū² / 48 κ` chez eux,
//!   n'est **pas** reprise : le dépôt n'a pas de rayon de tube, et l'exposant de `ū` n'a pas pu être relu sur l'image
//!   de l'article) ; les bords (la source et le puits des tubes) ; la perte linéaire `k` ; le pas borné.
//! - **Ce qui n'a pas été lu** : le supplément de l'article (figures S1 à S13) ; le pas de temps, le maillage et le
//!   schéma de leur simulation (Crank-Nicolson pondéré, valeurs non données). Les nombres de ce dépôt (`κ`, `k`, les
//!   pas) sont ceux que les tests choisissent : aucun n'est une valeur publiée.
//!
//! # L'état, et ce que « une concentration » veut dire
//!
//! [`Agent`] garde une concentration par nœud (`nb_noeuds` nombres, dans l'ordre des indices), chacune **finie et `>=
//! 0.0`** (`-0.0` est accepté à la construction, tel quel ; un pas n'en produit jamais). **Chaque nœud est une chambre
//! de volume 1** : la quantité d'agent d'un nœud est sa concentration, et la **masse** de l'agent est la somme des
//! concentrations, **de gauche à droite à partir de `0.0`** ([`Agent::masse`]).
//!
//! # Les flux
//!
//! Le transport lit les flux **d'un pas des tubes** (`Etat::flux` de `Tubes::avancer`, ou celui de `PasReseau::etat` de
//! `Reseau::avancer`), rien d'autre : `flux[e]` est le débit de liquide de l'arête `e`, **positif quand le liquide va
//! du nœud `a` vers le nœud `b`** (`a < b`), en unités de volume par unité de temps. Pour un nœud `i` et une arête `e`
//! de `i`, le **débit sortant** de `i` par `e` est `q = flux[e]` si `i == arete.a`, et `q = -flux[e]` si `i == arete.b`
//! (une négation exacte) ; `q > 0` veut dire que `i` donne du liquide à l'autre bout (`i` est le **donneur** de `e`),
//! `q < 0` qu'il en reçoit (`i` est le **receveur**).
//!
//! # Les bords : où le liquide entre et sort du réseau
//!
//! Le module ne reçoit ni la source ni le puits : il les déduit des flux. Pour chaque nœud `i`, `s_i` est la somme des
//! débits sortants `q` de **toutes** ses arêtes (`s_i = 0.0`, puis `s_i = s_i + q` pour chaque arête de
//! `graphe.incidentes(i)`, dans l'ordre). Le liquide est conservé dans un tube sans fuite : si `s_i > 0.0`, plus de
//! liquide part de `i` par ses arêtes qu'il n'en arrive : du liquide **entre** au nœud `i` (la source), et il entre
//! **sans agent**. Si `s_i < 0.0`, du liquide **quitte** le réseau en `i` (le puits) et **emporte** de l'agent : la
//! **sortie** du nœud est `sortie_i = -s_i` (sinon `sortie_i = 0.0`), un débit de liquide, et il emporte l'agent à la
//! concentration de `i`. **La même règle s'applique à tous les nœuds**, y compris ceux de l'intérieur, où `s_i` vaut
//! zéro **à l'arrondi près** : un `s_i` négatif de l'ordre de `1e-16` fait sortir un millionième de milliardième de la
//! concentration à chaque pas. Ce que cela coûte a été mesuré (voir « Ce que l'arrondi coûte » plus bas).
//!
//! # Le pas, à la lettre
//!
//! Un pas de durée `h` (tout se calcule à partir des concentrations `c` **d'avant** le pas, puis s'applique ; une
//! concentration mise à jour n'influence pas la suivante dans le même pas) prend les flux `Q`, les [`Transport`] (`κ =
//! diffusion`, `k = perte`, tous deux `>= 0`) et une liste d'apports `(nœud, débit d'agent)`. Les opérations sont
//! celles ci-dessous, **dans cet ordre, avec ces parenthèses** (`+ - * /` seulement ; chaque ligne s'évalue
//! parenthèses d'abord, puis de gauche à droite, une opération `f64` à la fois) ;
//! `L_e` est la `longueur` de l'arête `e` (jamais son coût) ; chaque somme part de `0.0` et ajoute ses termes **dans
//! l'ordre indiqué** (une somme de flottants dépend de son ordre). Le résultat est, bit pour bit, celui de cette
//! recette.
//!
//! ```text
//! pour chaque nœud i, par indice croissant :                          (« le taux de départ » r_i, voir plus bas)
//!     r = 0.0 ; s = 0.0
//!     pour chaque arête e de graphe.incidentes(i), dans l'ordre :      (j l'autre bout, q le débit sortant de i par e)
//!         s = s + q
//!         si q > 0.0 :  r = r + q                                      (advection : i est le donneur)
//!         r = r + kappa / L_e                                          (diffusion, toujours, même si kappa vaut 0.0)
//!     sortie_i = -s  si s < 0.0, sinon 0.0
//!     r = r + sortie_i
//!     r = r + k
//!     r_i = r
//!
//! pour chaque nœud i, par indice croissant :
//!     g = 0.0
//!     pour chaque arête e de graphe.incidentes(i), dans l'ordre :      (j et q : comme dans le premier bloc)
//!         si q < 0.0 :  g = g + (h * (-q)) * c[j]                      (advection : i reçoit de j, le donneur)
//!         g = g + (h * (kappa / L_e)) * c[j]                           (diffusion : i reçoit de j, toujours)
//!     c_apres[i] = (1.0 - h * r_i) * c[i] + g                          (ce qui reste, plus ce qui arrive)
//!
//! sortie = 0.0 ; perte = 0.0                                           (le bilan, par nœud croissant)
//! pour chaque nœud i, par indice croissant :
//!     sortie = sortie + (h * sortie_i) * c[i]
//!     perte  = perte  + (h * k) * c[i]
//! apport = 0.0
//! pour chaque (nœud, debit) de la liste des apports, dans l'ordre de la liste :
//!     c_apres[nœud] = c_apres[nœud] + h * debit                        (après le calcul de tous les c_apres ci-dessus)
//!     apport = apport + h * debit
//! ```
//!
//! C'est, en mots : sur chaque arête de flux `Q` non nul, la quantité `h · |Q| · c_donneur` passe du donneur au
//! receveur (**advection, schéma amont**) ; sur chaque arête, `h · κ · (c_j − c_i) / L_e` passe de `j` vers `i`
//! (**diffusion**, signée ; ici écrite en deux parts positives, ce qui revient au même en réels et donne des termes `>=
//! 0` aux flottants) ; le liquide qui sort au puits emporte `h · sortie_i · c_i` (**la sortie**) ; `h · k · c_i`
//! disparaît (**la perte**, la décroissance de l'article) ; `h · debit` arrive au nœud d'un apport (**l'apport**, par
//! où 2M posera un stimulus). Le terme `(1.0 - h * r_i) * c[i]` regroupe tout ce que `i` donne : advection, diffusion,
//! sortie et perte. Les sommes par arête vont dans l'ordre de `incidentes`, c'est-à-dire des indices d'arête croissants
//! : **une boucle par arête qui ajoute chaque terme au nœud concerné, dans l'ordre des arêtes, donne les mêmes bits**
//! (les tests le font ainsi).
//!
//! Le [`Bilan`] d'un pas contient `sortie`, `perte` et `apport` tels que calculés ci-dessus, et le nombre de sous-pas
//! (`1` pour [`pas_sans_decoupage`]). En réels, `masse_apres = masse_avant - sortie - perte + apport` : c'est la
//! conservation de la masse ; aux flottants, elle tient à l'arrondi près (tolérance mesurée dans les tests), et
//! **exactement** sur des valeurs dyadiques choisies (voir « Ce que l'arrondi coûte »).
//!
//! # Le taux de départ et le pas maximal
//!
//! `r_i` est **ce qui part du nœud `i` par unité de temps, rapporté à sa concentration** : le liquide qu'il donne à ses
//! voisins, la diffusion vers chacun d'eux, la sortie et la perte. Le schéma garde les concentrations `>= 0` si, pour
//! chaque nœud, `h * r_i <= 1` : alors `1.0 - h * r_i >= 0.0` (une soustraction de flottants d'un nombre `<= 1.0` à
//! `1.0` ne donne jamais un négatif) et tout le reste du pas n'ajoute que des termes `>= 0`. [`taux_de_depart`] rend
//! les `r_i`. [`pas_maximal`] rend **le plus grand flottant `h` tel que `h * r_max <= 1.0` calculé en flottants**,
//! `r_max` étant le plus grand des `r_i` (comparaisons `r > m` en partant de `0.0`) : on prend `h = 1.0 / r_max`, on le
//! baisse d'un cran (`f64::from_bits(h.to_bits() - 1)`) tant que `h * r_max > 1.0`, puis on le monte d'un cran
//! (`h.to_bits() + 1`) tant que le suivant garde `h * r_max <= 1.0`. Si `r_max == 0.0`, le pas maximal est
//! `f64::INFINITY` (rien ne part jamais d'aucun nœud). La descente d'un cran ne sert que si `1.0 / r_max` déborde (un
//! `r_max` sous-normal, comme une perte de `1e-320`) : `h` vaut alors `f64::INFINITY`, `h * r_max` aussi, et la
//! descente rend `f64::MAX`. Hors de ce cas, sur deux millions de nombres essayés par le modèle Python,
//! `(1.0 / r) * r` n'a jamais dépassé `1.0`.
//!
//! # Le pas public et son découpage
//!
//! [`Agent::avancer`] accepte un `h` quelconque (fini, `> 0.0`) : il le **découpe** en `n` sous-pas égaux. Avec `h_max
//! = pas_maximal(...)`, `n` est **le plus petit entier `>= 1` tel que `h / (n as f64) <= h_max`**, comparé en flottants
//! ([`nombre_de_sous_pas`]) ; chaque sous-pas dure `h_sub = h / (n as f64)` (pour `n = 1`, exactement `h`) et applique
//! la recette ci-dessus avec `h_sub`, **les apports comptant `h_sub * debit` à chaque sous-pas** (donc `h * debit` en
//! tout). Les flux sont les mêmes à chaque sous-pas. Le bilan rendu est la **somme des bilans des sous-pas** (chaque
//! total part de `0.0`, ajoute dans l'ordre des sous-pas), son nombre de sous-pas est `n`. Si `n` dépasse
//! [`MAX_SOUS_PAS`] (un million), c'est l'erreur `SousPas`. [`pas_sans_decoupage`] fait **un seul** pas de la recette
//! avec le `h` donné, sans le comparer à `h_max` : au-dessus de la limite, il rend des concentrations négatives (c'est
//! ce que les tests éprouvent) ; il ne touche aucun [`Agent`] (il prend et rend des tableaux).
//!
//! # Ce que l'arrondi coûte
//!
//! Mesuré sur des grilles de `6 × 6`, `8 × 8` et `12 × 10` aux flux de vrais pas des tubes (`Tubes::avancer`, 1, 30 et
//! 300 pas), des concentrations au hasard, quatre transports `(κ, k)` dont des nuls (`mes_2t_a.rs`, `mes_2t_d.rs`).
//!
//! - **À la limite exacte** (`h = pas_maximal`) : `1.0 - h * r_i` est `>= 0.0` par construction, donc **aucune
//!   concentration n'est négative, ni `-0.0`** : mesuré sur 1 800 cas, 132 000 concentrations : **0 négatif, 0
//!   `-0.0`**. La recette « naïve » `c + (gains - pertes)` (une somme signée des échanges), essayée à côté, rend **204
//!   négatifs** minuscules (le pire `-8.9e-16`) : c'est pour cela que la recette regroupe ce qui part dans `1.0 - h *
//!   r_i`, et il n'y a ni `max(0.0)` ni seuil : le schéma est exact là où il le dit. Un cran au-dessus du pas maximal,
//!   une concentration peut devenir négative (le nœud de taux maximal sans apport : `-2.2e-16` pour une concentration
//!   `1`).
//! - **Aux nœuds intérieurs**, `s_i` est nul à l'arrondi près et la même règle s'y applique : le plus grand `|s_i|`
//!   d'un nœud autre que la source et le puits vaut `5.6e-15` pour des flux de l'ordre de `1` (`8e-14` de la somme des
//!   `|q|` du nœud) ; sur 200 pas au pas maximal, une libération de masse `1` perd par ces nœuds **au plus `9.2e-15`**
//!   de sa masse (40 graines) : l'agent qui sort ainsi est compté dans la `sortie` du bilan, qui reste juste. Aucun
//!   seuil n'est appliqué à `s_i` (un `s_i` négatif de `-2.8e-17` fait sortir de l'agent, voir les tests).
//! - **La masse** : en réels `masse_apres = masse_avant - sortie - perte + apport` ; aux flottants l'écart d'un pas
//!   vaut au plus `3.0e-14` (masses de l'ordre de `1` à `3`, soit `3e-15` en relatif) et, composé sur 100 pas,
//!   `6.4e-14` (40 graines). Sur des valeurs dyadiques choisies, tout est exact (les cas sont vérifiés en fractions).
//!
//! # Les vérifications, dans cet ordre
//!
//! Pour [`pas_sans_decoupage`] et [`Agent::avancer`] (la première qui s'applique ; **sur une erreur, rien n'a changé**)
//! :
//!
//! 1. `Taille` : `concentrations.len()` n'est pas `graphe.nb_noeuds()` ou `flux.len()` n'est pas `graphe.nb_aretes()` ;
//! 2. `Concentration(i)` : le plus petit `i` dont la concentration n'est pas un nombre fini `>= 0.0` ;
//! 3. `Flux(e)` : la plus petite arête `e` dont le flux n'est pas fini ;
//! 4. `Diffusion` : `diffusion` n'est pas un nombre fini `>= 0.0` ; sinon `Perte` : `perte` n'est pas un nombre fini
//!    `>= 0.0` (si les deux sont fautifs, c'est `Diffusion`) ;
//! 5. `Pas` : `h` n'est pas un nombre fini `> 0.0` ;
//! 6. `Apport(r)` : le plus petit rang `r` de la liste dont le nœud n'existe pas, ou dont le débit n'est pas un nombre
//!    fini `>= 0.0` (`-0.0` est accepté) ; pour un même rang, une seule erreur ;
//! 7. `Debordement` : un taux de départ n'est pas fini (avant tout calcul de concentration), ou, après le calcul, une
//!    concentration ou l'un des trois totaux du bilan ne l'est pas (dans [`Agent::avancer`] : à chaque sous-pas, sur
//!    les concentrations et les trois totaux de ce sous-pas ; les totaux sommés des sous-pas ne sont pas revérifiés) ;
//! 8. `SousPas` (seulement [`Agent::avancer`]) : `n > MAX_SOUS_PAS`, vu après le point 7 sur les taux.
//!
//! [`taux_de_depart`] et [`pas_maximal`] font les vérifications 1 (pour les flux seuls : `flux.len()` n'est pas
//! `graphe.nb_aretes()`), 3 et 4 sur leurs seules entrées, puis 7 sur les taux.
//! [`Agent::new`] fait `Taille` puis `Concentration(i)`.
//!
//! # Les cas dégénérés
//!
//! - Un graphe sans arête : les flux sont vides, tous les `r_i` valent `k` : l'agent ne fait que décroître.
//! - Des flux tous nuls et `κ = 0` : `r_i = k` : rien ne bouge que la perte. Avec `k = 0.0` aussi, `r_max = 0.0`, le
//!   pas maximal est infini et `n = 1`.
//! - Une liste d'apports vide, ou un débit `0.0` : aucun apport.
//! - Un même nœud plusieurs fois dans les apports : chaque entrée s'ajoute, dans l'ordre.
//!
//! # L'aperçu
//!
//! [`apercu`] dessine les tubes comme `tubes::apercu` (mêmes panneaux, mêmes positions, mêmes dimensions, avec
//! `reference_tubes` pour référence et l'échelle 1 pour le treillis), puis colore les **nœuds** selon la concentration
//! rapportée à `reference_agent` : c'est « la tache d'agent qui suit les tubes ». Le niveau et le mélange sont ceux de
//! `tubes::apercu`, en entiers, sauf une division de flottants :
//!
//! ```text
//! niveau(c) = 255                        si t >= 1.0, avec t = c / reference_agent
//!           = (t * 255.0).round()        si 0.0 < t < 1.0       (round : à l'entier le plus proche, les moitiés
//!                                                                vers l'extérieur ; 0.5 donne 127.5, donc 128)
//!           = 0                          sinon                  (zéro, négatif, NaN)
//! melange(a, b, n) = (a * (255 - n) + b * n + 127) / 255          par canal, en entiers non signés, division entière
//! ```
//!
//! On part du treillis de `tubes::apercu(graphe, conductivites, reference_tubes, 1)`, puis **chaque nœud `i`, par
//! indice croissant**, repeint son point (celui de `tubes::apercu`) en `melange(c, AGENT, niveau(concentrations[i]))`,
//! où `c` est la couleur que le point a **à ce moment-là** (un niveau 0 laisse le point tel qu'il est ; deux nœuds au
//! même point : le second mélange sur le résultat du premier). Chaque point devient enfin un carré de `echelle ×
//! echelle` pixels, comme dans `rendu`. [`AGENT`] est la couleur d'un nœud de niveau 255.
//!
//! # La mémoire dans la forme (étape 2, partie 2M)
//!
//! Jusqu'ici l'agent est porté par les flux sans rien leur faire. Cette partie ferme la boucle : **l'agent change la
//! croissance des tubes**, et les tubes changent où va l'agent ([`Memoire`], [`Couplage`], [`PasMemoire`]).
//!
//! **Ce qui vient de Kramar et Alim** (PNAS 118(10), 2021) : le mécanisme (un agent soluble, libéré au stimulus et
//! porté par le flux, **assouplit** la paroi des tubes, qui se dilatent), et la **forme de la saturation** de l'effet :
//! leur éq. (2) écrit le module de la paroi `E = E_0 - ΔE · c / (c_0 + c)` (de type Michaelis ; lue sur le texte
//! extrait du PDF, symboles grecs perdus et rétablis par le sens). **Ce qui est à nous** : tout le reste. Leur modèle
//! est **un seul tube** élastique (il n'a ni graphe ni loi `dD/dt = f(|Q|) - D` ; le supplément, Fig. S8 « deux stimuli
//! », n'a pas pu être lu), et **aucune source ne donne le couplage « agent vers croissance des tubes » sur un graphe,
//! ni la tenue de l'empreinte, ni l'effet de deux stimuli**. Ce que fait cette partie est **notre modèle**, dit comme
//! tel, et **son résultat est mesuré et rapporté tel quel, négatif compris** (voir « Ce que les mesures ont donné »).
//!
//! ## La loi couplée (à nous)
//!
//! Là où l'agent est présent, la **cible** vers laquelle s'adapte la conductivité d'un tube grandit : la loi du pas des
//! tubes (`D <- D + h (f(|Q|) - D)`) devient
//!
//! ```text
//! D <- D + h * ((1 + α * σ(c_e)) * f(|Q|) - D)        puis le plancher,       σ(c) = c / (c_0 + c)
//! ```
//!
//! où `f` est la loi du pas ([`Loi`] de `reseaux` : linéaire `|Q|` ou sigmoïde `Q²/(1+Q²)`), `c_e` la concentration de
//! l'arête (**la moyenne des concentrations de ses deux bouts**, choix à nous), `σ` la saturation de l'article (`0 <= σ
//! < 1`, la moitié de son maximum à `c = c_0`) et `α >= 0` la force de l'assouplissement. **Sans flux, aucun effet** :
//! la cible est un multiple de `f(|Q|)`, donc un tube que le flux ne traverse pas s'amincit comme avant, agent ou pas.
//! Avec `α = 0` le pas est, **bit pour bit**, celui sans agent (`Tubes::avancer` pour [`Loi::Lineaire`],
//! `Reseau::avancer_paire` pour la sigmoïde) : `(1.0 + 0.0 * σ) * f` vaut exactement `f`.
//!
//! ## Le pas couplé, à la lettre
//!
//! [`Memoire::avancer`] fait, dans cet ordre (une opération `f64` par ligne, parenthèses d'abord, puis de gauche à
//! droite ; `c` désigne les concentrations de l'agent **d'avant** le pas) :
//!
//! ```text
//! g[e] = D[e] / cout[e]                                  pour chaque arête e, par indice croissant
//! inj  = nb_noeuds fois +0.0, puis inj[source] = debit
//! p    = solveur::pressions(graphe, g, inj, puits) ;  Q = solveur::flux(graphe, g, p)
//! agent' = Agent::avancer(graphe, Q, transport, apports, h)         (le transport de 2T, sur une copie de l'agent)
//! pour chaque arête e = (a, b), par indice croissant :
//!     ce    = (c[a] + c[b]) * 0.5                                    (c : l'agent d'AVANT le transport)
//!     sigma = ce / (c_0 + ce)
//!     cible = (1.0 + alpha * sigma) * f(Q[e])        (f = Loi::valeur, sur Q[e] signé : c'est f(|Q|) des deux lois)
//!     d     = D[e] + h * (cible - D[e])
//!     D'[e] = d  si d >= PLANCHER,  sinon PLANCHER
//! ```
//!
//! **Les conductivités se mettent à jour avec les concentrations d'AVANT le transport** (choix à nous) : l'état suivant
//! est une fonction de l'état d'avant seul (`D, c`), les deux moitiés du pas (flux, puis `D'` et `c'`) partent du même
//! état, et la mise à jour ne dépend pas du découpage en sous-pas du transport. Conséquence : un apport d'agent ne
//! touche les tubes qu'**au pas suivant** (les tests le vérifient). Le pas rend les pressions et les flux (avec les
//! conductivités d'avant) et le [`Bilan`] du transport ([`PasMemoire`]).
//!
//! ## Les vérifications du pas couplé, dans cet ordre
//!
//! 1. `Taille` : `graphe.nb_aretes()` n'est pas le nombre de conductivités, ou le nombre de concentrations n'est pas
//!    `graphe.nb_noeuds()` ;
//! 2. `Noeud` : `source` ou `puits` n'est pas un nœud, ou les deux sont le même ;
//! 3. `Debit` : `debit` n'est pas un nombre fini `> 0.0` ;
//! 4. `Pas` : `h` n'est pas un nombre fini avec `0.0 < h < 1.0` (celui des tubes ; le transport se découpe seul) ;
//! 5. `Force` : `α` n'est pas un nombre fini `>= 0.0` ;
//! 6. `Saturation` : `c_0` n'est pas un nombre fini `> 0.0` ;
//! 7. `Solveur(erreur)` : l'erreur du solveur, s'il en rend une ;
//! 8. `Agent(erreur)` : l'erreur du transport (`Diffusion`, `Perte`, `Apport(rang)`, `SousPas`, `Debordement`, dans
//!    l'ordre de « Les vérifications » plus haut ; `Flux` ne se produit que si le solveur rend un flux infini) ;
//! 9. `Debordement` : une `cible` ou une conductivité calculée `d` n'est pas un nombre fini, vérifiées arête par arête
//!    dans l'ordre des indices, la `cible` puis `d`, **avant** le plancher (par exemple `α = 1e308` avec un agent
//!    présent).
//!
//! **Sur une erreur, rien n'a changé** : ni les conductivités ni l'agent. [`Memoire::new`] fait `Taille` puis
//! `Conductivite(e)`.
//!
//! ## Les deux familles de l'étape 1
//!
//! `avancer` prend la `source`, le `puits`, le `debit` et la [`Loi`] : **une source et un puits** (le cœur : débit 1,
//! loi linéaire dans `lancer`) ou **les nourritures tirées par paire** (`reseaux` : `lancer` tire la paire avec
//! `reseaux::tirer_paire`, puis appelle `avancer` avec elle). Une seule fonction sert les deux : aucune des deux
//! familles n'a de code à elle dans le couplage.
//!
//! ## Ce qu'on mesure, et le seuil fixé d'avance
//!
//! Les tests (et `references/etape-2m-valeurs.md`) comparent toujours un état **avec stimulus** à un **témoin de même
//! graine** sans stimulus (mêmes conductivités de départ, même paire tirée à chaque pas). Sur une grille de 15 colonnes
//! et 11 lignes (voisinage 8), source en `(1, 5)`, puits en `(13, 5)` : la **moitié haute** est `y < 5`, la **basse**
//! `y > 5` ; la moyenne `Dm` d'une moitié est la somme des conductivités des arêtes dont les **deux** bouts y sont (par
//! indice croissant, à partir de `0.0`) divisée par leur nombre. **L'écart d'empreinte** est `E = (Dm_haut -
//! Dm_bas)(stimulus) - (Dm_haut - Dm_bas)(témoin)`, avec `F_A` et `F_B` ses deux termes (haut, bas). **Le seuil de
//! l'empreinte est `0.01` sur `E`, fixé avant la campagne de mesures.** La **sonde** pose un flux de 1 en `(7, 5)` avec
//! le puits en `(13, 5)` sur les conductivités gelées (`solveur`) et partage le flux qui traverse la coupe entre les
//! colonnes 10 et 11 selon que le bout en colonne 10 est en haut (`y < 5`), sur l'axe ou en bas : c'est « où part le
//! flux d'une nouvelle source ». Notre modèle n'a **pas de front qui avance** (la « migration » du plan n'existe pas
//! ici) : on ne mesure que ces parts.
//!
//! ## Ce que les mesures ont donné (50 graines, 101 à 150 ; détail dans `references/etape-2m-valeurs.md`)
//!
//! Réglage standard : loi sigmoïde, `I0 = 3`, `α = 8`, `c_0 = 1`, `κ = 0.1`, `k = 0.2`, `h = 0.1`, stimulus de débit 2
//! en `(7, 2)` pendant les pas `[0 ; 100[`.
//!
//! - **Sous la loi sigmoïde, l'empreinte se forme et ne s'efface plus** : `E` vaut en moyenne `+0.089` à la fin du
//!   stimulus (au pire `+0.059`), `+0.0366` cent pas plus tard et **exactement la même valeur 3000 pas après le
//!   retrait** (le réseau est tombé sur un état stable : un tube que l'agent a fait passer au-dessus de son seuil de
//!   survie y reste, c'est la bistabilité de la loi sigmoïde) ; 96 % des graines restent au-dessus du seuil, 4 % en
//!   restent sous (la plus basse vaut `+0.0079`, encore positive). L'agent n'y est pour rien après coup : effacé au pas
//!   100, le résultat est le même (`+0.0364`).
//! - **Sous la loi linéaire, elle s'efface** : le réseau n'a qu'un équilibre (le plus court chemin) ; `E` retombe sous
//!   le seuil entre 200 et 300 pas après le retrait, puis tend vers zéro (`+0.0001` à 1000 pas). Si l'agent s'en va
//!   lentement (`k = 0.01`), l'empreinte le suit : elle tient aussi longtemps que lui.
//! - **Un stimulus donné sur un réseau déjà convergé ne laisse rien après son retrait** (le flux ne passe plus près).
//! - **Où part le flux d'une nouvelle source** : avec le stimulus en haut, la part du flux de la sonde passant par le
//!   haut moins celle passant par le bas vaut `+0.72` en moyenne (jamais négative sur 50 graines, `+0.16` au pire),
//!   contre `+0.01` (de `-0.33` à `+0.33`) pour le témoin et `-0.66` pour le stimulus symétrique.
//! - **Deux stimuli** : du même côté ils ne s'ajoutent pas en intensité (`F_A` `+7 %` ou `-7 %` selon l'écartement)
//!   mais rendent l'empreinte régulière (la sonde ne descend plus sous `+0.67`) ; de côtés opposés ils se partagent la
//!   même « réserve » de tubes épais (`F_A` et `F_B` valent chacun environ la moitié de celui d'un stimulus seul) et la
//!   sonde choisit un côté au hasard.
//!
//! **Ce qui ne tient pas** : sous la loi linéaire (le réseau du cœur), l'empreinte s'efface ; sur un réseau convergé,
//! un stimulus n'en laisse pas ; l'empreinte ne se renforce pas avec le nombre de stimuli ; deux stimuli opposés ne se
//! cumulent pas ; et **aucun de ces chiffres n'est celui d'un article**.
use crate::alea::Alea;
use crate::graphe::Graphe;
use crate::rendu::{ECART, Image};
use crate::reseaux::Loi;
use crate::solveur::ErreurSolveur;
use crate::tubes;
use crate::tubes::Etat;

/// Le nombre de sous-pas au-delà duquel [`Agent::avancer`] refuse : `1_000_000`.
pub const MAX_SOUS_PAS: usize = 1_000_000;

/// La couleur d'un nœud dont la concentration atteint la référence dans [`apercu`] : un vert vif.
pub const AGENT: [u8; 3] = [130, 255, 80];

/// Pourquoi un pas de transport, un pas maximal ou un agent n'a pas pu être calculé (voir « Les vérifications »).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurAgent {
    /// Une longueur de tableau n'est pas la bonne (concentrations ou flux).
    Taille,
    /// La concentration de ce nœud (le plus petit indice fautif) n'est pas un nombre fini `>= 0.0`.
    Concentration(usize),
    /// Le flux de cette arête (le plus petit indice fautif) n'est pas un nombre fini.
    Flux(usize),
    /// La diffusion `κ` n'est pas un nombre fini `>= 0.0`.
    Diffusion,
    /// La perte `k` n'est pas un nombre fini `>= 0.0`.
    Perte,
    /// Le pas `h` n'est pas un nombre fini `> 0.0`.
    Pas,
    /// L'apport de ce rang dans la liste (le plus petit rang fautif) a un nœud inconnu ou un débit qui n'est pas un
    /// nombre fini `>= 0.0`.
    Apport(usize),
    /// Un nombre calculé a débordé : il n'est pas fini.
    Debordement,
    /// Le découpage demanderait plus de [`MAX_SOUS_PAS`] sous-pas.
    SousPas,
}

/// Les deux coefficients du transport (voir « Le pas, à la lettre »).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transport {
    /// La diffusion `κ`, un nombre fini `>= 0.0` : un seul coefficient pour toutes les arêtes.
    pub diffusion: f64,
    /// La perte `k`, un nombre fini `>= 0.0` : la part de l'agent qui disparaît par unité de temps.
    pub perte: f64,
}

/// Ce qu'un pas a fait de la masse : `masse_apres = masse_avant - sortie - perte + apport` (en réels).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bilan {
    /// L'agent emporté par le liquide qui quitte le réseau (les `sortie_i`).
    pub sortie: f64,
    /// L'agent perdu par la décroissance `k`.
    pub perte: f64,
    /// L'agent apporté par la liste des apports.
    pub apport: f64,
    /// Le nombre de sous-pas (`1` pour [`pas_sans_decoupage`]).
    pub sous_pas: usize,
}

/// L'agent d'un réseau : une concentration par nœud (voir « L'état »).
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    concentrations: Vec<f64>,
}

/// Les taux de départ `r_i` de chaque nœud (voir « Le pas, à la lettre » et « Le taux de départ et le pas maximal »).
///
/// # Erreurs
///
/// `Taille` (`flux.len()` n'est pas le nombre d'arêtes), `Flux(e)`, `Diffusion`, `Perte`, puis `Debordement` si un taux
/// n'est pas fini, dans cet ordre.
pub fn taux_de_depart(graphe: &Graphe, flux: &[f64], transport: &Transport) -> Result<Vec<f64>, ErreurAgent> {
    if flux.len() != graphe.nb_aretes() {
        return Err(ErreurAgent::Taille);
    }
    for (e, &q) in flux.iter().enumerate() {
        if !q.is_finite() {
            return Err(ErreurAgent::Flux(e));
        }
    }
    if !(transport.diffusion.is_finite() && transport.diffusion >= 0.0) {
        return Err(ErreurAgent::Diffusion);
    }
    if !(transport.perte.is_finite() && transport.perte >= 0.0) {
        return Err(ErreurAgent::Perte);
    }
    let (taux, _) = taux_et_sorties(graphe, flux, transport.diffusion, transport.perte);
    if taux.iter().any(|&r| !r.is_finite()) {
        return Err(ErreurAgent::Debordement);
    }
    Ok(taux)
}

/// Les taux de départ `r_i` et les sorties `sortie_i` d'un pas, sur des entrées déjà vérifiées (voir « Le pas, à la
/// lettre »). L'ordre des sommes est celui de la doc : par nœud croissant, puis par arête de `incidentes`.
fn taux_et_sorties(graphe: &Graphe, flux: &[f64], kappa: f64, k: f64) -> (Vec<f64>, Vec<f64>) {
    let n = graphe.nb_noeuds();
    let mut taux = vec![0.0; n];
    let mut sorties = vec![0.0; n];
    for i in 0..n {
        let mut r = 0.0;
        let mut s = 0.0;
        for &e in graphe.incidentes(i) {
            let arete = &graphe.aretes()[e];
            let q = if arete.a == i { flux[e] } else { -flux[e] };
            s += q;
            if q > 0.0 {
                r += q;
            }
            r += kappa / arete.longueur;
        }
        let sortie = if s < 0.0 { -s } else { 0.0 };
        r += sortie;
        r += k;
        taux[i] = r;
        sorties[i] = sortie;
    }
    (taux, sorties)
}

/// Le pas maximal : le plus grand `h` tel que `h * r_max <= 1.0` en flottants, `f64::INFINITY` si `r_max == 0.0` (voir
/// « Le taux de départ et le pas maximal »).
///
/// # Erreurs
///
/// Celles de [`taux_de_depart`].
pub fn pas_maximal(graphe: &Graphe, flux: &[f64], transport: &Transport) -> Result<f64, ErreurAgent> {
    let taux = taux_de_depart(graphe, flux, transport)?;
    Ok(pas_maximal_depuis_taux(&taux))
}

/// Le plus grand flottant `h` tel que `h * r_max <= 1.0`, à partir de taux finis (voir « Le taux de départ et le pas
/// maximal »).
fn pas_maximal_depuis_taux(taux: &[f64]) -> f64 {
    let mut r_max = 0.0;
    for &r in taux {
        if r > r_max {
            r_max = r;
        }
    }
    if r_max == 0.0 {
        return f64::INFINITY;
    }
    let mut h = 1.0 / r_max;
    while h * r_max > 1.0 {
        h = f64::from_bits(h.to_bits() - 1);
    }
    loop {
        let suivant = f64::from_bits(h.to_bits() + 1);
        if suivant * r_max <= 1.0 {
            h = suivant;
        } else {
            break;
        }
    }
    h
}

/// Le nombre de sous-pas : le plus petit entier `n >= 1` tel que `h / (n as f64) <= h_max` en flottants (voir « Le pas
/// public et son découpage »). `1` si `h <= h_max` (en particulier si `h_max` est infini). Si ce `n` dépasse
/// [`MAX_SOUS_PAS`], un nombre plus grand que [`MAX_SOUS_PAS`] (sa valeur exacte n'est pas définie).
///
/// # Panique
///
/// Si `h` n'est pas un nombre fini `> 0.0`, ou si `h_max` n'est pas `> 0.0` (l'infini est permis ; NaN panique).
pub fn nombre_de_sous_pas(h: f64, h_max: f64) -> usize {
    assert!(h.is_finite() && h > 0.0, "le pas doit être un nombre fini strictement positif");
    assert!(h_max > 0.0, "le pas maximal doit être strictement positif (l'infini est permis)");
    if h <= h_max {
        return 1;
    }
    let quotient = h / h_max;
    if !quotient.is_finite() || quotient > (MAX_SOUS_PAS + 2) as f64 {
        return MAX_SOUS_PAS + 1;
    }
    let mut n = quotient.ceil() as usize;
    while h / (n as f64) > h_max {
        n += 1;
    }
    while n > 1 && h / ((n - 1) as f64) <= h_max {
        n -= 1;
    }
    n
}

/// **Un seul pas** de la recette de « Le pas, à la lettre », avec le `h` donné, **sans le comparer au pas maximal** :
/// au-dessus de la limite il rend des concentrations négatives. Rend les nouvelles concentrations et le bilan (avec
/// `sous_pas == 1`). Ne modifie rien.
///
/// # Erreurs
///
/// Celles de « Les vérifications », points 1 à 7, dans cet ordre.
pub fn pas_sans_decoupage(
    graphe: &Graphe,
    flux: &[f64],
    concentrations: &[f64],
    transport: &Transport,
    apports: &[(usize, f64)],
    h: f64,
) -> Result<(Vec<f64>, Bilan), ErreurAgent> {
    verifier_entrees(graphe, flux, concentrations, transport, apports, h)?;
    un_pas(
        graphe,
        flux,
        concentrations,
        transport,
        apports,
        h,
        &taux_et_sorties(graphe, flux, transport.diffusion, transport.perte),
    )
}

/// Les vérifications 1 à 6 de « Les vérifications », dans leur ordre.
fn verifier_entrees(
    graphe: &Graphe,
    flux: &[f64],
    concentrations: &[f64],
    transport: &Transport,
    apports: &[(usize, f64)],
    h: f64,
) -> Result<(), ErreurAgent> {
    if concentrations.len() != graphe.nb_noeuds() || flux.len() != graphe.nb_aretes() {
        return Err(ErreurAgent::Taille);
    }
    for (i, &c) in concentrations.iter().enumerate() {
        if !(c.is_finite() && c >= 0.0) {
            return Err(ErreurAgent::Concentration(i));
        }
    }
    for (e, &q) in flux.iter().enumerate() {
        if !q.is_finite() {
            return Err(ErreurAgent::Flux(e));
        }
    }
    if !(transport.diffusion.is_finite() && transport.diffusion >= 0.0) {
        return Err(ErreurAgent::Diffusion);
    }
    if !(transport.perte.is_finite() && transport.perte >= 0.0) {
        return Err(ErreurAgent::Perte);
    }
    if !(h.is_finite() && h > 0.0) {
        return Err(ErreurAgent::Pas);
    }
    let nb_noeuds = graphe.nb_noeuds();
    for (rang, &(noeud, debit)) in apports.iter().enumerate() {
        if noeud >= nb_noeuds || !(debit.is_finite() && debit >= 0.0) {
            return Err(ErreurAgent::Apport(rang));
        }
    }
    Ok(())
}

/// Un pas de la recette de « Le pas, à la lettre », sur des entrées déjà vérifiées, avec les taux et sorties donnés :
/// vérifie `Debordement` (point 7) avant tout calcul de concentration, puis après le calcul.
fn un_pas(
    graphe: &Graphe,
    flux: &[f64],
    concentrations: &[f64],
    transport: &Transport,
    apports: &[(usize, f64)],
    h: f64,
    taux: &(Vec<f64>, Vec<f64>),
) -> Result<(Vec<f64>, Bilan), ErreurAgent> {
    let (r, sorties) = taux;
    if r.iter().any(|&x| !x.is_finite()) {
        return Err(ErreurAgent::Debordement);
    }
    let n = graphe.nb_noeuds();
    let mut gains = vec![0.0; n];
    for (i, gain) in gains.iter_mut().enumerate() {
        let mut g = 0.0;
        for &e in graphe.incidentes(i) {
            let arete = &graphe.aretes()[e];
            let j = if arete.a == i { arete.b } else { arete.a };
            let q = if arete.a == i { flux[e] } else { -flux[e] };
            if q < 0.0 {
                g += (h * (-q)) * concentrations[j];
            }
            g += (h * (transport.diffusion / arete.longueur)) * concentrations[j];
        }
        *gain = g;
    }
    let mut apres = vec![0.0; n];
    for i in 0..n {
        apres[i] = (1.0 - h * r[i]) * concentrations[i] + gains[i];
    }
    let mut sortie = 0.0;
    let mut perte = 0.0;
    for i in 0..n {
        sortie += (h * sorties[i]) * concentrations[i];
        perte += (h * transport.perte) * concentrations[i];
    }
    let mut apport = 0.0;
    for &(noeud, debit) in apports {
        apres[noeud] += h * debit;
        apport += h * debit;
    }
    if apres.iter().any(|&x| !x.is_finite()) || !sortie.is_finite() || !perte.is_finite() || !apport.is_finite() {
        return Err(ErreurAgent::Debordement);
    }
    Ok((apres, Bilan { sortie, perte, apport, sous_pas: 1 }))
}

impl Agent {
    /// Un agent à zéro : `graphe.nb_noeuds()` concentrations égales à `+0.0`.
    pub fn zero(graphe: &Graphe) -> Agent {
        Agent { concentrations: vec![0.0; graphe.nb_noeuds()] }
    }

    /// Un agent de concentrations données (une par nœud, dans l'ordre des indices).
    ///
    /// # Erreurs
    ///
    /// Dans cet ordre : `ErreurAgent::Taille` si `concentrations.len()` n'est pas `graphe.nb_noeuds()` ; puis
    /// `ErreurAgent::Concentration(i)`, `i` étant le plus petit indice d'une concentration qui n'est pas un nombre fini
    /// `>= 0.0` (`0.0` et `-0.0` sont acceptés ; NaN, l'infini et les négatifs sont refusés).
    pub fn new(graphe: &Graphe, concentrations: Vec<f64>) -> Result<Agent, ErreurAgent> {
        if concentrations.len() != graphe.nb_noeuds() {
            return Err(ErreurAgent::Taille);
        }
        for (i, &c) in concentrations.iter().enumerate() {
            if !(c.is_finite() && c >= 0.0) {
                return Err(ErreurAgent::Concentration(i));
            }
        }
        Ok(Agent { concentrations })
    }

    /// Les concentrations, dans l'ordre des indices de nœud.
    pub fn concentrations(&self) -> &[f64] {
        &self.concentrations
    }

    /// La masse d'agent : la somme des concentrations, de gauche à droite à partir de `0.0` (`0.0` pour zéro nœud).
    pub fn masse(&self) -> f64 {
        let mut masse = 0.0;
        for &c in &self.concentrations {
            masse += c;
        }
        masse
    }

    /// Le pas public : découpe `h` en sous-pas égaux (voir « Le pas public et son découpage »), les joue avec les flux
    /// et les apports donnés, met l'agent à jour et rend le bilan total (la somme des bilans des sous-pas ; `sous_pas`
    /// vaut `n`).
    ///
    /// # Erreurs
    ///
    /// Celles de « Les vérifications », points 1 à 8, dans cet ordre ; **sur une erreur, l'agent n'a pas changé**.
    pub fn avancer(
        &mut self,
        graphe: &Graphe,
        flux: &[f64],
        transport: &Transport,
        apports: &[(usize, f64)],
        h: f64,
    ) -> Result<Bilan, ErreurAgent> {
        verifier_entrees(graphe, flux, &self.concentrations, transport, apports, h)?;
        let (r, sorties) = taux_et_sorties(graphe, flux, transport.diffusion, transport.perte);
        // Point 7 avant le point 8 : un taux non fini passe avant `SousPas`.
        if r.iter().any(|&x| !x.is_finite()) {
            return Err(ErreurAgent::Debordement);
        }
        let h_max = pas_maximal_depuis_taux(&r);
        let n = nombre_de_sous_pas(h, h_max);
        if n > MAX_SOUS_PAS {
            return Err(ErreurAgent::SousPas);
        }
        let nb_noeuds = graphe.nb_noeuds();
        let h_sous = h / (n as f64);
        // Tout se calcule dans des tampons locaux ; l'agent n'est écrit qu'à la fin, si tout a réussi.
        let mut courant = self.concentrations.clone();
        let mut suivant = vec![0.0; nb_noeuds];
        let mut gains = vec![0.0; nb_noeuds];
        let mut total_sortie = 0.0;
        let mut total_perte = 0.0;
        let mut total_apport = 0.0;
        for _ in 0..n {
            for (i, gain) in gains.iter_mut().enumerate() {
                let mut g = 0.0;
                for &e in graphe.incidentes(i) {
                    let arete = &graphe.aretes()[e];
                    let j = if arete.a == i { arete.b } else { arete.a };
                    let q = if arete.a == i { flux[e] } else { -flux[e] };
                    if q < 0.0 {
                        g += (h_sous * (-q)) * courant[j];
                    }
                    g += (h_sous * (transport.diffusion / arete.longueur)) * courant[j];
                }
                *gain = g;
            }
            for i in 0..nb_noeuds {
                suivant[i] = (1.0 - h_sous * r[i]) * courant[i] + gains[i];
            }
            let mut sortie = 0.0;
            let mut perte = 0.0;
            for i in 0..nb_noeuds {
                sortie += (h_sous * sorties[i]) * courant[i];
                perte += (h_sous * transport.perte) * courant[i];
            }
            let mut apport = 0.0;
            for &(noeud, debit) in apports {
                suivant[noeud] += h_sous * debit;
                apport += h_sous * debit;
            }
            if suivant.iter().any(|&x| !x.is_finite())
                || !sortie.is_finite()
                || !perte.is_finite()
                || !apport.is_finite()
            {
                return Err(ErreurAgent::Debordement);
            }
            total_sortie += sortie;
            total_perte += perte;
            total_apport += apport;
            std::mem::swap(&mut courant, &mut suivant);
        }
        self.concentrations = courant;
        Ok(Bilan { sortie: total_sortie, perte: total_perte, apport: total_apport, sous_pas: n })
    }
}

/// L'aperçu de l'agent : les tubes de `tubes::apercu`, avec les nœuds colorés selon leur concentration (voir « L'aperçu
/// »).
///
/// # Panique
///
/// Si `echelle` vaut 0, si `reference_tubes` ou `reference_agent` n'est pas un nombre fini `> 0.0`, si
/// `conductivites.len()` n'est pas le nombre d'arêtes ou si `concentrations.len()` n'est pas le nombre de nœuds.
pub fn apercu(
    graphe: &Graphe,
    conductivites: &[f64],
    concentrations: &[f64],
    reference_tubes: f64,
    reference_agent: f64,
    echelle: u32,
) -> Image {
    assert!(echelle > 0, "l'échelle doit être strictement positive");
    assert!(
        reference_tubes.is_finite() && reference_tubes > 0.0,
        "la référence des tubes doit être un nombre fini strictement positif"
    );
    assert!(
        reference_agent.is_finite() && reference_agent > 0.0,
        "la référence de l'agent doit être un nombre fini strictement positif"
    );
    assert_eq!(conductivites.len(), graphe.nb_aretes(), "il faut une conductivité par arête");
    assert_eq!(concentrations.len(), graphe.nb_noeuds(), "il faut une concentration par nœud");

    // Le treillis des tubes, au point près.
    let mut image = tubes::apercu(graphe, conductivites, reference_tubes, 1);

    // Les positions des nœuds sur ce treillis, comme dans `tubes::apercu`.
    let noeuds = graphe.noeuds();
    let mut xmin = noeuds[0].x as i64;
    let mut xmax = xmin;
    let mut ymin = noeuds[0].y as i64;
    let mut ymax = ymin;
    for noeud in &noeuds[1..] {
        let x = noeud.x as i64;
        let y = noeud.y as i64;
        xmin = xmin.min(x);
        xmax = xmax.max(x);
        ymin = ymin.min(y);
        ymax = ymax.max(y);
    }
    let tl = 2 * (xmax - xmin) + 1;
    let pas = tl + ECART as i64;
    let l = image.largeur as usize;
    for (i, noeud) in noeuds.iter().enumerate() {
        let col = noeud.etage as i64 * pas + 2 * (noeud.x as i64 - xmin);
        let ligne = 2 * (noeud.y as i64 - ymin);
        let rang = 3 * (ligne as usize * l + col as usize);
        let avant = [image.rvb[rang], image.rvb[rang + 1], image.rvb[rang + 2]];
        let n = niveau_concentration(concentrations[i], reference_agent);
        let couleur = melange(avant, AGENT, n);
        image.rvb[rang..rang + 3].copy_from_slice(&couleur);
    }

    // Chaque point du treillis devient un carré de `echelle × echelle` pixels.
    let e = echelle as usize;
    let largeur = l * e;
    let hauteur = image.hauteur as usize * e;
    let mut rvb = Vec::with_capacity(3 * largeur * hauteur);
    for ligne in 0..image.hauteur as usize {
        let mut rangee = Vec::with_capacity(3 * largeur);
        let debut = 3 * ligne * l;
        for point in image.rvb[debut..debut + 3 * l].as_chunks::<3>().0 {
            let couleur = [point[0], point[1], point[2]];
            for _ in 0..e {
                rangee.extend_from_slice(&couleur);
            }
        }
        for _ in 0..e {
            rvb.extend_from_slice(&rangee);
        }
    }
    Image { largeur: largeur as u32, hauteur: hauteur as u32, rvb }
}

/// Le niveau d'une concentration pour une référence, en entiers (voir « L'aperçu »).
fn niveau_concentration(c: f64, reference: f64) -> u8 {
    let t = c / reference;
    if t >= 1.0 {
        255
    } else if t > 0.0 {
        (t * 255.0).round() as u8
    } else {
        0
    }
}

/// Le mélange entier de deux couleurs, canal par canal, au niveau `n` (voir « L'aperçu »).
fn melange(a: [u8; 3], b: [u8; 3], n: u8) -> [u8; 3] {
    let n = u32::from(n);
    let canal = |a: u8, b: u8| -> u8 { ((u32::from(a) * (255 - n) + u32::from(b) * n + 127) / 255) as u8 };
    [canal(a[0], b[0]), canal(a[1], b[1]), canal(a[2], b[2])]
}

// Partie 2M : la mémoire dans la forme (le couplage de l'agent aux tubes).
// ----------------------------------------------------------------------------------------------------------------

/// Les deux coefficients du couplage de l'agent aux tubes (voir « La mémoire dans la forme »).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Couplage {
    /// La force `α >= 0.0` de l'assouplissement : `0.0` n'a aucun effet (le pas est celui des tubes sans agent).
    pub force: f64,
    /// La saturation `c_0 > 0.0` : la concentration d'agent à laquelle l'effet vaut la moitié de son maximum.
    pub saturation: f64,
}

/// Pourquoi un pas couplé ou une mémoire n'a pas pu être construit ou calculé (voir « La mémoire dans la forme »).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErreurMemoire {
    /// Le nombre de conductivités n'est pas le nombre d'arêtes, ou le nombre de concentrations n'est pas le nombre de
    /// nœuds.
    Taille,
    /// La conductivité de cette arête (le plus petit indice fautif) n'est pas un nombre fini `>= PLANCHER`.
    Conductivite(usize),
    /// `source` ou `puits` n'est pas un nœud du graphe, ou les deux sont le même nœud.
    Noeud,
    /// Le débit n'est pas un nombre fini `> 0.0`.
    Debit,
    /// Le pas `h` n'est pas un nombre fini avec `0.0 < h < 1.0`.
    Pas,
    /// La force `α` n'est pas un nombre fini `>= 0.0`.
    Force,
    /// La saturation `c_0` n'est pas un nombre fini `> 0.0`.
    Saturation,
    /// Le solveur de l'étape 0 a refusé ou échoué.
    Solveur(ErreurSolveur),
    /// Le transport de l'agent a refusé ou échoué (`Diffusion`, `Perte`, `Apport(rang)`, `SousPas`, `Debordement`).
    Agent(ErreurAgent),
    /// Une cible ou une conductivité calculée n'est pas un nombre fini.
    Debordement,
}

/// Ce qu'un pas couplé a calculé : les pressions et les flux (avec les conductivités d'AVANT) et le bilan de l'agent.
#[derive(Clone, Debug, PartialEq)]
pub struct PasMemoire {
    /// Les pressions et les flux du pas, comme `Tubes::avancer`.
    pub etat: Etat,
    /// Le bilan du transport de l'agent pour ce pas (voir [`Agent::avancer`]).
    pub bilan: Bilan,
}

/// Les tubes et l'agent d'un réseau, avancés ensemble (voir « La mémoire dans la forme »).
#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub struct Memoire {
    conductivites: Vec<f64>,
    agent: Agent,
}

impl Memoire {
    /// Une mémoire de conductivités et d'agent donnés.
    ///
    /// # Erreurs
    ///
    /// Dans cet ordre : `Taille` (`conductivites.len()` n'est pas `graphe.nb_aretes()`, ou l'agent n'a pas
    /// `graphe.nb_noeuds()` concentrations) ; `Conductivite(e)` (le plus petit indice d'une conductivité qui n'est pas
    /// un nombre fini `>= PLANCHER`).
    #[allow(unused_variables)]
    pub fn new(graphe: &Graphe, conductivites: Vec<f64>, agent: Agent) -> Result<Memoire, ErreurMemoire> {
        todo!()
    }

    /// Une mémoire aux conductivités de départ de `Tubes::au_hasard(graphe, alea)` et à l'agent à zéro.
    #[allow(unused_variables)]
    pub fn au_hasard(graphe: &Graphe, alea: &mut Alea) -> Memoire {
        todo!()
    }

    /// Les conductivités, dans l'ordre des indices d'arête.
    pub fn conductivites(&self) -> &[f64] {
        todo!()
    }

    /// L'agent.
    pub fn agent(&self) -> &Agent {
        todo!()
    }

    /// Un pas couplé (voir « Le pas couplé, à la lettre » et « Les vérifications du pas couplé »).
    ///
    /// # Erreurs
    ///
    /// Celles de « Les vérifications du pas couplé », dans leur ordre ; **sur une erreur, rien n'a changé**.
    #[allow(unused_variables, clippy::too_many_arguments)]
    pub fn avancer(
        &mut self,
        graphe: &Graphe,
        source: usize,
        puits: usize,
        debit: f64,
        loi: Loi,
        couplage: &Couplage,
        transport: &Transport,
        apports: &[(usize, f64)],
        h: f64,
    ) -> Result<PasMemoire, ErreurMemoire> {
        todo!()
    }
}
