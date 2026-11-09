# Physarum Lab : les règles du dépôt

Ce dépôt est mené par Lydite : des mains (des agents de toute sorte) y travaillent sur des plans de vagues, chacune
sur une branche `lydite/…`, et GitHub fusionne une PR quand sa CI est verte sous Linux et sous Windows. Ces règles
valent pour toute main. Le plan est dans `FEUILLE-DE-ROUTE.md` ; chaque modèle, ses équations, sa référence et ses
limites, dans `MODELE.md`.

## La preuve

`cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`, verte sous Linux et sous Windows.

## Les règles

1. **Sans dépendance** : rien dans `[dependencies]`, `[dev-dependencies]` ni `[build-dependencies]`. Le générateur
   pseudo-aléatoire, l'écrivain PNG, l'empreinte et le solveur linéaire s'écrivent dans le dépôt.
2. **Déterministe sur toutes les machines, pour l'état comme pour les mesures** : même scénario et même graine, même
   état et mêmes nombres, sous Linux comme sous Windows.
   - sur un `f64`, seuls sont permis `+`, `-`, `*`, `/`, les comparaisons, `sqrt`, `abs`, `floor`, `ceil`, `trunc`,
     `round`, `min`, `max`, `clamp`, `copysign`, `is_finite`, `is_nan`, `to_bits` et `from_bits` ; tout le reste passe
     par la bibliothèque mathématique du système et peut différer d'un système à l'autre : ni `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `atan2`, `exp`, `exp2`,
     `ln`, `log`, `log2`, `log10`, `powf`, `powi`, `hypot`, `cbrt`, `mul_add`, `sinh`, `cosh`, `tanh`. Une loi du
     modèle s'écrit avec `+ - * /` : la loi de renforcement est linéaire, une décroissance est un produit ;
   - **l'ordre des opérations fait partie du résultat** : une somme de flottants dépend de son ordre. Là où la doc d'une
     fonction donne un ordre (les arêtes par indice croissant, les nœuds par indice croissant, une somme de gauche à
     droite), on le suit à la lettre, sans regrouper ni réordonner ;
   - les positions et les comptages sont des entiers ;
   - aucune `HashMap` ni aucun `HashSet` parcourus : des `Vec` ou des `BTreeMap` ;
   - un ordre de mise à jour fixe, sans horloge ni fil d'exécution dans la simulation.
3. **La preuve, c'est l'état et les nombres** : l'empreinte de l'état simulé, par scénario et par graine
   (`references/<scénario>.empreinte`), et des mesures avec une tolérance. Les aperçus de `apercus/` sont pour l'œil :
   les embellir ne change aucune référence.
4. **Un changement voulu de l'état** met à jour son empreinte de référence dans la même PR, et la PR le dit, avec la
   raison pour laquelle cette empreinte est la même sous Linux et sous Windows (l'arithmétique employée : entiers,
   opérations exactes de `f64` dans un ordre fixe ; aucune fonction du système). Une empreinte prouve que rien n'a
   bougé, pas que c'est juste.
5. **Les tests d'acceptation d'une étape** s'écrivent avant son code, par une vague à eux, et un juge les éprouve avant
   qu'ils soient gelés (`FEUILLE-DE-ROUTE.md`, « Les tests avant les mains »). La vague du code ne touche pas à leurs
   fichiers. Chaque test d'une étape porte `#[cfg_attr(not(feature = "etape-<n>"), ignore)]`.
6. **Un test qui semble faux se dit** dans le compte rendu de la tâche, sans le toucher et sans plier le code pour lui
   plaire : ni cas particulier, ni valeur forcée, ni constante calibrée sur une graine. Aucune tâche ne supprime, ne
   marque `ignore` ni n'assouplit un test, et **aucune tolérance ne s'élargit pour faire passer un cas** : un changement
   de tolérance est une PR à lui, qui dit pourquoi.
7. **Chaque mécanisme arrive avec sa référence publiée** (un article, une figure ou une équation), nommée dans
   `MODELE.md`, et sa reproduction en nombres. Une valeur publiée qu'on n'atteint pas se dit telle quelle dans
   `MODELE.md`, et ce qui est à nous (un modèle sans article) s'y dit aussi. **Une valeur attendue a toujours une
   source** : une fiche par étape, `references/etape-<n>-valeurs.md`, donne pour chaque cas la valeur, sa source
   (article, page, figure, tableau ou équation, ou un calcul exact écrit à côté), la tolérance et sa raison ; une
   valeur sans source, ou tirée du code qu'elle juge, n'entre pas. Les résultats de notre code figés
   (`references/*.empreinte`) sont un autre fichier, figé seulement une fois que le cas passe contre sa référence.
   - **Chaque test dit ce qu'il est et ce qu'il ne couvre pas** : sa doc commence par `juste :` (un calcul exact, un
     simulateur naïf écrit dans le test, une valeur publiée avec sa source) ou `régression :` (un résultat de notre
     propre code, figé), et une ligne `Ne couvre pas :`. Seuls les tests « juste » sont une preuve.
   - **Chaque étape écrit ses critères avant de calculer** (`references/etape-<n>-valeurs.md`) et finit par une section
     « Ce que cela n'établit pas » dans `MODELE.md`.
8. **Rien sous `.github/`** : la CI et la fusion restent à l'humain. **Aucun `.cargo/`** : la CI le refuse, car sa
   configuration peut changer ce que `cargo test` joue.
9. **Les fonctionnalités et les modules sont déclarés d'avance** : `etape-0` à `etape-5` dans `Cargo.toml`, les modules
   dans `src/lib.rs`. Une tâche d'une vague de code ajoute `etape-<n>` à `default` quand le plan le dit, et ne touche
   pas aux autres lignes de ces deux fichiers.
10. **Le profil `dev` est optimisé** (`opt-level = 3` dans `Cargo.toml`) pour que les tests de simulation tournent vite :
    ne pas le changer, ni `[lints]`, ni `rustfmt.toml`, ni `rust-toolchain.toml`, hors d'une tâche qui le demande.
11. **Rejouer sur tout le crate** : `cargo test --no-fail-fast`, jamais filtré sur le seul test qui vient de passer ;
    un remaniement du code, sans changement voulu de l'état, se prouve octet par octet (les empreintes de tous les
    scénarios, avant et après, sont identiques).
12. **Une main ne touche que les fichiers de sa tâche.** Le test d'une tâche suivante échouera : c'est voulu, et elle
    ne fait pas le travail de cette tâche pour le faire passer.
