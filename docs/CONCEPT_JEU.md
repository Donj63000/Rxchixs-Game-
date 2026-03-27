# Concept du jeu - Rxchixs

Ce document sert de reference produit pour expliquer clairement ce que cherche a etre **Rxchixs** au-dela du prototype actuel.

## Vision

Rxchixs est un factory sim 2D top-down ou le joueur prend la direction d'une usine deja en activite. La production ne depend pas d'un clic manuel permanent. Le systeme vit de lui-meme, avec ses flux, ses retards, ses derives et ses arbitrages.

L'objectif n'est pas simplement d'empiler des machines. L'objectif est de comprendre une organisation industrielle vivante, puis de l'ameliorer sans casser sa stabilite economique.

## Fantaisie joueur

Le joueur n'incarne pas un simple operateur. Il agit comme un melange de :

- directeur de production,
- architecte de flux,
- responsable amelioration continue,
- observateur des KPI et des derives humaines / materielles.

La satisfaction recherchee vient de la lecture du systeme :

- identifier le vrai goulot,
- comprendre pourquoi une ligne sous-performe,
- corriger proprement le layout ou les priorites,
- voir la rentabilite remonter grace a une meilleure organisation.

## Promesse centrale

Chaque action du joueur doit suivre un chemin causal lisible :

`action -> comportement systeme -> KPI -> impact financier`

Exemples :

- deplacer un buffer reduit une congestion, ameliore le debit et augmente les ventes,
- negliger une zone sensible fait monter les erreurs, le rebut ou les incidents,
- surcharger une ligne peut ameliorer le volume a court terme mais degrader la qualite, la disponibilite ou le moral.

Si une mecanique n'est pas lisible a travers cette chaine de valeur, elle ne remplit pas completement la promesse du jeu.

## Boucle de jeu cible

1. Observer l'etat de l'usine et detecter les symptomes visibles.
2. Diagnostiquer les causes reelles dans les flux, les machines, les zones ou les humains.
3. Modifier l'organisation de l'usine.
4. Verifier l'effet sur les KPI, la tresorerie et la stabilite.
5. Reinvestir pour franchir un nouveau plafond de performance.

Le jeu doit encourager l'analyse systemique plutot que l'execution repetitive.

## Starter factory

Le jeu commence avec une usine de depart deja fonctionnelle, rentable et stable sans micro-management. C'est une contrainte produit forte.

Cette starter factory a plusieurs roles :

- donner immediatement une matiere interessante a observer,
- eviter une ouverture de partie vide,
- fournir un socle de comparaison avant / apres optimisation,
- servir de scenario de validation permanent pour le projet.

Le joueur peut ensuite tout modifier :

- construire,
- deplacer,
- vendre ou demolir,
- repeindre les zones,
- changer les flux,
- reconfigurer les postes et les priorites.

## Pourquoi une ligne d'ail au depart

Le prototype s'appuie sur une ligne de transformation d'ail parce qu'elle fournit un cas concret riche :

- plusieurs etapes de transformation,
- contraintes d'hygiene et de logistique,
- arbitrages qualite / cadence / couts,
- representation visuelle facile a lire,
- possibilite de rattacher chaque etape a une valeur economique.

Ce choix ne doit pas enfermer le jeu dans un seul theme. La ligne d'ail est un premier dataset, pas une limite d'architecture.

## Piliers systemiques

### 1. Usine vivante

Le monde doit continuer a tourner sans intervention constante du joueur.

Cela implique :

- des flux qui avancent,
- des equipements qui consomment, saturent ou se bloquent,
- des ressources qui transitent entre zones,
- des incidents lisibles,
- des consequences mesurables dans le temps.

### 2. Lecture economique

L'argent est l'indicateur de synthese, mais il doit rester explicable.

Le joueur doit pouvoir relier :

- revenus,
- couts d'exploitation,
- rebuts,
- temps d'arret,
- qualite de service,
- performance humaine,

a des causes concretes dans l'usine.

### 3. Couche humaine credible

Les personnages ne sont pas decoratifs. Ils doivent participer a la comprehension du systeme.

Leurs etats peuvent influencer :

- la cadence,
- la qualite,
- les erreurs,
- le risque,
- la reactivite,
- le moral global.

Cette couche humaine ne doit pas casser la lisibilite du jeu. Elle doit enrichir l'analyse.

### 4. Zones comme outil de design

Une zone n'est pas seulement une couleur au sol. C'est une enveloppe logique qui porte :

- des regles,
- des contraintes,
- des KPI attendus,
- des risques,
- des impacts sur les flux et les humains.

Les zones servent autant a structurer le jeu qu'a guider le joueur dans sa lecture du systeme.

### 5. IA auditable

Quand un agent choisit un job, le joueur ou le developpeur doit pouvoir comprendre pourquoi.

Le systeme doit rester lisible sur :

- la faisabilite,
- les reservations,
- la priorite,
- la raison d'un blocage,
- les grands facteurs de score.

L'objectif n'est pas d'avoir une IA opaque mais une IA systemique comprehensible.

## Invariants techniques

Pour rester fidele au concept, certains invariants sont structurants :

- simulation et rendu separes,
- tick fixe avec horloge explicite,
- aucune logique metier dependante du framerate,
- donnees de depart chargeables depuis des fichiers,
- saves et blueprints versionnes,
- determinisme suffisant pour tester et deboguer proprement,
- pas de fallback silencieux sur les systemes critiques.

## Ce que le prototype doit toujours demontrer

Meme avant la version finale, le prototype doit montrer ces qualites :

- une usine deja exploitable,
- des flux observables,
- des decisions de construction comprehensibles,
- des KPI relies aux causes,
- une base technique propre a faire grandir.

## Positionnement

Rxchixs se situe a la croisee de plusieurs plaisirs de jeu :

- lecture de simulation,
- optimisation industrielle,
- sandbox de construction,
- management leger mais significatif,
- amelioration continue mesurable.

Le but n'est pas d'etre un clone d'un autre jeu, mais de reprendre le plaisir d'un monde autonome et lisible applique a la logique d'une usine.

## Resume

Rxchixs veut etre un jeu ou l'on regarde une usine vivre, ou l'on comprend reellement ce qui la freine, puis ou l'on transforme cette comprehension en profit, en stabilite et en elegance de production.
