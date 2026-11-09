---
name: verbatim-prose-retouchee
description: Un fichier donné « à recopier tel quel » se recopie à l'octet : ne corrige pas un mot, ne recoupe pas une ligne, ne traduis rien, même si le texte te paraît maladroit.
---

## What happened
Leçon reprise de Givre Lab, le second banc de Lydite. Quatre fois, une main chargée de recopier un fichier éprouvé par un juge l'a retouché : « Git for Windows » traduit en « Git pour Windows » dans une fiche de valeurs, une ligne de doc de 127 caractères coupée en deux dans un fichier de signatures, deux lignes de commentaire recoupées, un mot « corrigé » dans une autre fiche. Le code et les tests, eux, étaient recopiés à l'identique.

## Why
Le fichier a été éprouvé tel quel, et la session le compare ensuite à l'original octet par octet : un seul écart demande une tâche de plus pour le remettre. Une main « améliore » volontiers un texte ; ce n'est pas ce qui lui est demandé.

## How it is checked
Après l'écriture, relis le fichier contre le texte de ta tâche, ligne par ligne, puis vérifie qu'il finit par un seul saut de ligne et n'a aucun retour chariot. Si un outil de mise en forme a changé le fichier (`cargo fmt` sur un `.rs`), garde ce que l'outil donne : le texte de la tâche est déjà formaté, l'outil ne doit rien y changer.

## Pitfall
Les lignes longues, les mots anglais dans une phrase française, une tournure inhabituelle sont voulus. Si tu crois voir une vraie erreur, recopie quand même à l'octet et dis-la dans ton compte rendu.

## Files
- src/
- tests/
- references/
- MODELE.md
- FEUILLE-DE-ROUTE.md
- README.md
- apercus/
- scenarios/

## Falls when
Lydite compare lui-même à l'octet un fichier que le plan donne à recopier.
