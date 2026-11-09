---
name: main-deborde-7013
description: Ne touche que les fichiers de ta tâche ; le test d'une tâche suivante échouera, c'est voulu : ne fais pas son travail pour le faire passer.
---

## What happened
Leçon reprise de Mycelium Lab et de Givre Lab, les deux premiers bancs de Lydite. Dans deux vagues de code de Mycelium, trois tâches sur douze ont été refusées (refus 7013, hors de ses fichiers) : chaque main avait écrit une partie du travail des tâches suivantes, jusqu'à `Cargo.toml` et la référence d'empreinte, pour faire passer des tests gelés qui n'étaient pas les siens. À Givre, la tâche du binaire a fait de même une fois, puis s'est corrigée à la relance.

## Why
Les tests d'une étape sont écrits avant le code et restent rouges tant que toutes les tâches n'ont pas rendu leur part. Une main qui voit du rouge hors de sa tâche est tentée de tout réparer ; Lydite refuse alors tout son travail, le bon compris.

## How it is checked
Avant de finir, lance `git status` : seuls les fichiers nommés dans ta tâche doivent avoir changé. Joue les tests de ta tâche, ceux que son texte nomme ; les autres peuvent échouer.

## Pitfall
Un fichier de sortie (une référence d'empreinte, un aperçu) et `Cargo.toml` ont chacun leur tâche : ne les écris pas « au passage ». Si tu as dû modifier un autre fichier pour essayer ton code, remets-le dans son état d'origine avant de finir.

## Files

## Falls when
Lydite ne montre à une main que les tests de sa tâche.
