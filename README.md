# Physarum Lab

Un simulateur libre du blob (*Physarum polycephalum*), en Rust, orienté recherche : un organisme sans cerveau, fait
d'un réseau de tubes, qui trouve le plus court chemin d'un labyrinthe, se souvient d'où était la nourriture, apprend à
franchir un pont qu'il évitait, et transmet ce qu'il a appris en fusionnant avec un autre. Chaque mécanisme est refait
en nombres d'après son article, avec un rendu qu'on prend plaisir à regarder : le réseau qui se resserre sur le chemin.

**Où il en est** : l'étape 0, le socle, est faite : le réseau (un graphe sur un ou plusieurs étages), le calcul
exact du flux dans les tubes, l'empreinte de l'état, l'aperçu. Aucun blob ne pousse encore : c'est l'étape 1. La V0, six
étapes du socle à la page web, est décrite dans [FEUILLE-DE-ROUTE.md](FEUILLE-DE-ROUTE.md) ; ce que chaque étape
établit, et ce qu'elle n'établit pas, est dans [MODELE.md](MODELE.md). Les trois premières étapes de modèle refont
des expériences publiées ; la quatrième (la transmission par fusion) est un modèle à nous, présenté comme tel.

## Ce qui le tient

- Rust sans dépendance ; même scénario et même graine, même état et mêmes nombres, sous Linux comme sous Windows.
- La preuve est l'état simulé et des nombres, chacun avec sa référence publiée ; les images sont pour l'œil.
- Les tests d'une étape s'écrivent avant son code, et un juge les éprouve avant qu'ils soient gelés.
- Les règles du dépôt sont dans [AGENTS.md](AGENTS.md) ; chaque modèle, ses équations et ses limites, dans
  [MODELE.md](MODELE.md).

## Comment il se construit

Physarum Lab est le troisième banc d'essai de [Lydite](https://github.com/aoricloud/lydite), un orchestrateur d'agents :
ses mains travaillent sur des plans de vagues, chacune sur une branche `lydite/…`, et GitHub fusionne une PR quand sa
CI est verte sous Linux et sous Windows et que Lydite a rejoué sa preuve sur la tête de la PR. L'humain qui mène le
dépôt en garde les réglages et regarde après coup. Les deux premiers bancs sont
[Mycelium Lab](https://github.com/aoricloud/mycelium-lab) et [Givre Lab](https://github.com/aoricloud/givre-lab).

## Licence

MIT, voir [LICENSE](LICENSE).
