# Better snake


## Snap a snake

- Attraper un serpent  (plus grand ou plus petit dans un piège.
Le serpent adverse est bloqué dans un tunnel et doit suivre le couloir jusqu’au bout.

- Accompagner le serpent jusqu’au bout le la ligne et le faire se planter dans le mur.

- Accompagner le serpent un petit bout et fermer la route pour le « snapper »

https://play.battlesnake.io/g/a50f654f-1b96-4d6f-8dbd-0d3af95d0764/

    turn 20
    turn 237
    Turn 267

https://play.battlesnake.io/g/42484fd6-e94f-4f2c-bdfb-098935fd02b2/

    Turn 106 si snakes move down

https://play.battlesnake.io/g/42484fd6-e94f-4f2c-bdfb-098935fd02b2/

    turn 165 Peu importe le sens boblee meurt enfermé 

https://play.battlesnake.io/g/42484fd6-e94f-4f2c-bdfb-098935fd02b2/

    turn 87, Superslimey snap starter boy

https://play.battlesnake.io/g/9983830d-73ab-4c8d-9b83-3d5a49f8d957/

    Turn 111 à 113, Schneider peut snapper boblee

https://play.battlesnake.io/g/c896dce2-77ac-4380-9ec4-ca5869935590/
 
    Tour 223 , snap !!!!!

https://play.battlesnake.io/g/b9ef0731-86e5-403e-ab98-685d8b53e41b/

    Tour 249, snap !!!

## anti_snap_a_snake

Prévoir si un snake peut me snapper et guider vers la bonne route

## Tail /shape / maze management

Donner une forme plus standard au serpent pour occuper la totalité de la maze au bout d’un certain temps.

Depth-First Search, Maze Algorithm

http://www.migapro.com/depth-first-search/

https://stackoverflow.com/questions/32999136/perfect-snake-ai#33003391

## Mange la pomme

- Plus gourmand, ne pas tourner en rond
- Aller manger la pomme pour laquelle you est le plus près.
- Aller manger la pomme qui m'appartient, la pomme de laquelle personne n'est aussi près que moi.

https://play.battlesnake.io/g/42484fd6-e94f-4f2c-bdfb-098935fd02b2/

    Turn 9

https://play.battlesnake.io/g/9983830d-73ab-4c8d-9b83-3d5a49f8d957/

    Turn 17

https://play.battlesnake.io/g/11b09dfd-3e16-4106-9cf3-081a4364efae/

    Tour 35 doit aller manger la pomme

https://play.battlesnake.io/g/c896dce2-77ac-4380-9ec4-ca5869935590/

    Tour 44 deux pommes à manger !!!!!

https://play.battlesnake.io/g/3b0b1c0f-1cca-4c90-8faf-2cb79792c17a/#

    Tour 13, se jette sur lui même, timeout ?

## Piège à nourriture

- Laisser la nourriture sur les bords
- Utiliser la nouriture comme appât pour le snap.
- snap a snake en attirant avec la food

## Utilise la place a disposition

Dans un espace clos, utiliser la place à disposition au maximum avec la queue comme porte de sortie

## Psychanalyse les snake

Enregistrer le comportement des snakes dans les parties
Psy analyser les snakes en 1-1

## Calcul des possibles

Ne pas déduire la queue des snakes qui mangent une pomme

## Neural network

Tuner les poids de chaque action avec un réseau neuronal / algorythmes génétiques
