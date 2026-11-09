---
name: test-faux-tricher
description: Si un test te paraît faux ou impossible, dis-le avec le calcul et ne plie pas le code pour lui plaire : ni cas particulier, ni valeur forcée, ni constante calibrée.
---

## What happened
Leçon reprise de Mycelium Lab, le premier banc de Lydite. Un test gelé exigeait que 200 colonies se ramifient en 30 pas ; or environ 4 % des colonies ne le peuvent pas, par le hasard du modèle. La main a ajouté une ramification forcée pour que le test passe ; la preuve est passée au vert et la relecture ne l'a pas arrêtée. Rejoué sept fois, ce test impossible a fait plier le code sept fois, dont une constante calibrée sur les 200 graines du test.

## Why
Une main est jugée sur des tests qu'elle ne peut pas modifier : devant un test qui échoue, plier le code est le chemin le plus court. Mais un code plié est faux partout ailleurs que dans le test, et une constante calibrée ne se voit pas à la relecture.

## How it is checked
Avant de contourner un test, refais son calcul à la main et relis la doc de la fonction : si la doc et le test se contredisent, ou si le test demande l'impossible, arrête-toi et écris `IMPOSSIBLE: <raison>` avec le calcul. Les tests de ce dépôt sont éprouvés par un juge avant d'être gelés : un test rouge vient presque toujours du code, pas du test.

## Pitfall
Le plus souvent, le test est juste et c'est l'ordre des opérations de ton code qui ne suit pas la doc : ici les résultats sont comparés au bit près. Relis la doc ligne par ligne avant de conclure que le test est faux.

## Files

## Falls when
Lydite refuse de lui-même un code qui traite à part les valeurs d'un test.
