# Rxchixs — DOC “CODEX” (source de vérité produit)

But de ce document :
- Donner à un agent de dev (Codex) une compréhension complète du jeu Rxchixs, de ses systèmes, de ses invariants et de ses contraintes.
- Servir de PRD/Spec vivante. Si une implémentation contredit ce document, c’est un bug ou un choix à expliquer explicitement.

NOTE IMPORTANTE (data-driven) :
- Les exemples (ail, tailles de sacs/box/palettes, durées, etc.) sont un “dataset de départ”.
- Rien ne doit être hardcodé “ail-only”. Tout doit être paramétrable via données (RON/JSON), afin de remplacer/étendre plus tard sans refactor.

---

## 0) Pitch / mental model

Rxchixs est un jeu sandbox 2D top-down sur grille (RimWorld-like) qui simule une usine “vivante” en continu.
- Par défaut, l’usine tourne seule (“aquarium”) : des employés autonomes, des machines, des stocks, des incidents, des flux.
- Le joueur intervient pour optimiser / perturber / réorganiser : layout, zones, machines, personnel, process.
- Le succès se lit principalement dans l’argent (profit, pertes, coûts, qualité, retards, maintenance, moral).

Le jeu démarre OBLIGATOIREMENT avec une petite usine préfaite, déjà fonctionnelle, équilibrée et rentable.
Mais tout est modifiable : construire, déplacer, vendre/démolir, améliorer, reconfigurer.

---

## 1) Piliers non négociables

1) Sandbox vivant
- Machines = état, usure, efficacité, pannes.
- Zones = règles + contexte (pas juste décor).
- Employés = compétences, besoins, limites, traits.
- Ressources = flux, délais, pertes, traçabilité.
- Décisions du joueur = conséquences économiques lisibles.

2) Usine de départ (TRÈS IMPORTANT)
- L’usine initiale est déjà jouable et rentable.
- Le joueur arrive “dans un système vivant”.
- Le layout initial doit être chargé depuis données (blueprint RON/JSON) et sauvegardable/chargeable.

3) Production par zones (framework générique)
- Une zone = règles + ressources + postes/machines + buffers + jobs/tâches + KPI locaux.
- Les zones consomment/produisent/déplacent des “items” génériques via recipes/règles data-driven.
- Le gameplay vient des goulots, des stocks, des incidents, des choix d’optimisation, et de l’impact financier.

4) IA autonome style RimWorld
- Jobs + priorités + réservations (ressources/machines/slots) + interruptions/reprise propre.
- Décisions multi-critères (urgence, distance, fatigue/stress, compétence, sécurité, disponibilité).
- IA débuggable : overlay par agent (action, cible, priorité, raison).

5) Économie/KPI = vérité du système
- Tout doit se traduire en KPI puis en argent.

---

## 2) Glossaire (terminologie canon)

- Grille / Cellule : coordonnée logique (x,y) utilisée par la simulation.
- Bloc : élément physique placé sur la grille (sol, mur, porte, machine, rack, équipement…).
- Zone (environnement) : couche logique de règles/contextes sur des cellules (travail autorisé, contraintes hygiène/accès/température, risques, KPI locaux, effets humains).
- Item : ressource ou unité manipulable (générique) : matière, caisse, sac, box, palette, big bag, consommable…
- Unité logistique (UL) : item “conteneur” transportable (caisse, box, palette, big bag).
- Lot : identifiant de traçabilité attaché à une matière/UL (obligatoire sur la matière première et sa descendance).
- Buffer : stockage tampon lié à une zone/machine (file d’attente, stockeur %, rack zone tampon, etc.).
- Job : unité de travail structurée exécutée par un agent (avec états, ressources, cible, impact KPI).
- Sous-job : action atomique d’un job (réservable, interruptible, reprenable).
- Réservation : claim/lock d’un item, d’une machine, d’un slot de stockage ou d’un panneau de réglage pour éviter collisions et doubles prises.
- KPI : métriques de performance (débit, rebuts, arrêts, OTIF, coûts, qualité, moral, maintenance…).
- OPEX : coûts d’exploitation (énergie, maintenance, salaires, consommables, pénalités).
- CAPEX : investissements (achat/installation/upgrade machines, infrastructure).

---

## 3) Monde & simulation (cadre technique)

Le monde est une simulation continue, séparée du rendu :
- Tick fixe (ex: 60 Hz).
- Le rendu peut interpoler, mais la logique est déterminée par le tick de simulation.
- La simulation doit pouvoir tourner “headless” (sans rendu) pour tests/sim.

La carte est une grille :
- Pathfinding (A*) pour agents piétons et caristes.
- Gestion des obstacles (blocs), des cellules réservées (anti-collisions), et des zones interdites.

---

## 4) Système Zones (environnements logiques)

Une zone n’est PAS une simple couleur au sol. Une zone = règles, contraintes et contexte qui modifient la simulation.

Chaque zone doit définir au minimum :
1) Ce qu’on y fait :
- types de jobs disponibles (travailler, stocker, manutentionner, contrôler, nettoyer, maintenir, etc.)

2) Contraintes :
- accès autorisé (piétons/caristes)
- hygiène minimum
- température
- bruit toléré
- règles de sécurité
- contrôles d’accès (badge, sens de circulation, etc.)

3) KPI locaux :
- cadence cible
- qualité attendue
- taux de rebut acceptable
- niveau de stock tampon recommandé

4) Risques :
- bourrages, erreurs, accidents, pannes, contamination, stress

5) Effets humains :
- fatigue/stress +/-
- moral +/-
- vitesse d’exécution +/-

Exemples de zones génériques (non figées) :
- Zone logistique : bonus vitesse cariste, risque collision/congestion ↑ si mal marquée.
- Zone “propre” : qualité ↑ + prix vente ↑, mais coût nettoyage ↑ et contraintes ↑.
- Zone froide : conservation ↑, mais fatigue/stress ↑ sans équipement.
- Zone humide : utile à certains process, mais risque glissade/pannes ↑.
- Zone bureau/admin : stress ↓, organisation ↑, mais pas de production directe.

Les zones doivent être data-driven (paramétrables RON/JSON).

---

## 5) Système Blocs (éléments physiques)

Un bloc = tout ce qu’on place sur la grille :
- coût achat + installation
- coût d’exploitation (énergie/maintenance/consommables)
- état (neuf → usé → panne)
- stats (bonus/malus)
- interactions gameplay (crée jobs, stocke, transforme, bloque, sécurise…)

Familles de blocs à prévoir (canon) :
A) Sols : vitesse, fatigue, accidents, hygiène, durabilité.
B) Murs/cloisons : isolation thermique/sonore, résistance, entretien.
C) Portes/contrôle d’accès : temps ouverture, fiabilité, isolation, badge/sens.
D) Stockage (sol, racks, buffers) : capacité, vitesse d’accès, sécurité, conditions (froid), traçabilité.
E) Machines/postes : cadence, rendement, qualité, énergie, pannes, skill requis, bruit/chaleur.
F) Logistique interne : voies caristes, marquages, zones de retournement, couloirs dédiés (convoyeurs plus tard).
G) Sécurité/ergonomie : barrières, extincteurs, alarmes, EPI, tapis anti-fatigue, signalétique, formations.
H) Confort/social : salle pause, toilettes, vestiaires, eau, repos… (impact besoins/moral).
I) Contrôle/monitoring/admin : tableau KPI, poste qualité, scanner lots, alarmes incidents.

Upgrades (crescendo) :
- Chaque bloc important a des niveaux.
- Coût augmente par niveau.
- Bonus en % selon type (ex: machines cadence +, pannes -, énergie -).

Dégradation / casse :
- Usure si maintenance ignorée.
- Pannes par aléa + surcharge + mauvais réglages + saleté.
- Incidents humains (fatigue/stress/erreurs).
- Congestion (layout mal pensé).
- Explosion des coûts (énergie, maintenance, rebuts).

C’est le cœur “aquarium” : tu touches une variable, tout le système réagit et ça se voit dans les KPI (et surtout l’argent).

---

## 6) Items, unités logistiques, traçabilité (lots)

Le jeu manipule des items génériques, souvent sous forme d’unités logistiques :
- Caisses (bois / plastiques grises)
- Sacs (ex: bleus conformes, rouges rejets)
- Box (conteneurs de sacs)
- Palettes (unités expédiables)
- Big bags (matière ou déchets)
- Consommables (film, étiquettes, etc. — abstraits au début)

Traçabilité :
- À la réception, chaque livraison reçoit un numéro de lot.
- Ce lot doit “se propager” à travers la production.
- La qualité peut bloquer un lot (statut OK / à surveiller / non conforme / bloqué).
- Interdiction d’expédier un lot “Bloqué”.

Le stockage doit gérer :
- emplacements (slots) réservables
- statuts (en transit / stocké / réservé / bloqué qualité)
- conditions (froid/humide)
- éventuellement politiques FIFO/FEFO (configurable plus tard)

---

## 7) Process industriel “dataset de départ” (ail) — vision globale

Ce process sert d’exemple concret et de “starter factory”, mais le framework doit rester générique.

1) Réception et stockage
- L’ail arrive en stock.
- Chaque livraison reçoit un lot.
- Les caristes rangent les lots dans les zones de stockage.

2) Soufflerie (séchage initial)
- L’ail est stocké dans des caisses, empilées par piles de 5.
- Temps de séjour environ 8 heures.
- Nécessite un système de temps (heures/jours/cadence/planning).
- Quand le timer termine, le lot devient “prêt”.

3) Cassage (séparation pelures / gousses)
- Les opérateurs récupèrent l’ail séché.
- Ils séparent pelures et gousses.
- Sorties :
    - gousses → stockage frigo en caisses grises (avec traçabilité lot)
    - pelures → big bags déchets vers benne
- Environnement : bruyant + poussiéreux + salissant → fatigue ↑, risques ↑, besoin nettoyage régulier.

4) Déshydratation (ligne)
- lavage → découpe → grand séchoir.
- Durée de séchage varie selon réglages (impact qualité/cadence/énergie).
- Paramètres pilotables : vitesses tapis/vis/Hz, température four, vitesse four.

5) Finition (tri et ensachage)
- Machine de tri (“Sortex”) sépare :
    - lanières → sacs bleus (conformes)
    - rejets → sacs rouges
- Opérateurs :
    - surveiller la ligne
    - éviter bourrages
    - assurer continuité
    - placer sacs dans box dédiés

Paramètres/conditionnement (data-driven) :
- Vision “macro” : sac bleu 25kg, sac rouge 15kg, 24 sacs par palette.
- Exemple “job spec” (plus granulaire) : box complet à 21 sacs + clôture box → zone tampon cariste.
  => IMPORTANT : ces quantités sont des paramètres DATA, pas des constantes hardcodées. Le design doit supporter les deux (et l’itération d’équilibrage).

6) Palettisation et stockage
- Palette complète : pesée, filmée, déposée en zone d’enlèvement.
- Cariste range en racks en attente de vente.

7) Vente
- Administration des ventes vend des palettes (par commandes clients).
- Chaque vente génère de l’argent.
- Argent sert à : améliorer machines, embaucher, agrandir, optimiser.

---

## 8) Personnages (IA) : besoins, compétences, traits

Objectif :
- Produire des comportements crédibles et déboggables.
- Assurer une boucle économique lisible : états humains → performance/risque → KPI → argent.
- Modèle data-driven (RON/JSON), itérable sans refactor.

Convention barres :
- Score ∈ [0..100]
- 0 = critique, 100 = optimal
- Seuils génériques :
    - 0–10 : critique (danger / arrêt)
    - 10–25 : très bas
    - 25–50 : bas
    - 50–80 : fonctionnel
    - 80–100 : confort / bonus

Dynamique :
- Évolution continue (tick) : dérive naturelle + effets zone + charge.
- Évolution événementielle : repas, pause, incident, interaction, réussite/échec, changement de zone.

8.1 Besoins vitaux (dominants, “contraintes dures” sous seuil)
- Manger (énergie)
- Boire (hydratation)
- Dormir (sommeil)
- Toilettes

Quand bas :
- vitesse/cadence ↓
- erreurs ↑
- accidents ↑
- décisions mauvaises ↑

8.2 Besoins “qualité de vie” (contraintes molles / dérives longues)
- Hygiène
- Divertissement
- Social
- Confort (température/ergonomie/bruit)
- Calme (résilience mentale ; inverse du stress)
- Confort physique / douleur inversée (option)

Ces barres doivent générer de la micro-histoire + des dérives industrielles :
- stress ↑ ⇒ erreurs ↑ ⇒ rebut ↑ ⇒ marge ↓
- confort ↓ ⇒ fatigue ↑ ⇒ accidents/pannes ↑ ⇒ downtime ↑

8.3 Compétences (0..100) — familles
Technique / maintenance :
- Mécanique
- Électricité / automation

Production / qualité :
- Dextérité
- Qualité / conformité

Logistique / manutention :
- Force
- Logistique (flux / cariste)

Cognitif / organisation :
- Intelligence
- Planification

Social / encadrement :
- Sociabilité
- Management

Sécurité / hygiène industrielle :
- Sécurité
- Nettoyage / hygiène indus
- Diagnostic (option)

Une compétence élevée impacte :
- vitesse d’exécution
- qualité
- risque d’erreur
- capacité à gérer incidents

Progression :
- Plus un personnage travaille un poste, plus la compétence associée augmente.
- Polyvalence = valeur stratégique ↑ mais peut générer demandes (augmentation, horaires, CDI, changement poste…) avec impacts motivation/moral/fidélité/risque départ.

8.4 Traits (biais comportemental)
- Motivation
- Discipline (procédures)
- Fiabilité (finit ce qu’il commence)
- Patience
- Courage (acceptation urgences/pénibles)
- Empathie (option social)

8.5 États synthétiques (lisibilité + debug)
- Santé
- Moral
- Fatigue physique (distinct sommeil)

---

## 9) Jobs : définition canon + framework IA

Un Job est une unité de travail structurée exécutée par un personnage :
- tâche concrète dans une zone
- objectif précis
- ressources associées
- lieu d’exécution
- impact mesurable (KPI)

Le joueur peut embaucher des intérimaires tant qu’il a l’argent :
- coût dépend poste / créneau (nuit) / compétence.

Chaque personnage :
- poste principal prioritaire
- postes secondaires (compétences associées)
- une grille de priorités (style RimWorld)

### 9.1 Standard de fiche Job (format canon, directement codable)

Champs minimum :
- Définition système : type + rôle + objectif principal.
- Entrées / Sorties système : consommations / productions / événements.
- Déclencheurs : conditions d’apparition (timer, seuil, alerte, demande inter-zone).
- Décomposition en sous-jobs : actions atomiques IA (réservables, interruptibles, reprenables).
- Contraintes & règles : réservations, compatibilités, autorité, sécurité.
- États : Idle / En cours / Interrompu / Bloqué / Terminé + spécifiques si besoin.
- Facteurs de performance : compétences + caractéristiques (vitesse/erreurs/fatigue/risque).
- Interactions : autres jobs impactés.
- Impact KPI : débit, rebuts, arrêts, OTIF, coûts.

### 9.2 Sélection d’un Job (IA haut niveau)

Deux phases :
1) Filtrage faisabilité :
- interdit par joueur
- hors compétence/autorisation
- hors shift (si règles)
- inaccessible (pathfinding)
- ressources non réservables (machine occupée, item réservé, slot plein)
- trop dangereux (si règles sécurité)

2) Scoring / classement :
- priorité joueur (biais fort)
- urgence dynamique (arrêt de ligne, congestion, deadline commande, salissure>seuil…)
- distance/temps trajet
- adéquation agent (compétence, fatigue, stress, traits)
- risque (qualité/sécurité) vs gain attendu

### 9.3 Hiérarchie “naturelle” de priorité (par défaut)

Même en autonomie totale :
1) Urgences menaçant continuité / sécurité :
- arrêt de ligne imminent
- bourrage critique
- saturation tampon critique
- panne critique
- lot bloqué qualité bloquant expédition

2) Maintien du flux :
- alimentation matière (stockeur bas)
- enlèvements cariste quand zone tampon déborde
- clôture box en finition si accumulation

3) Production normale + expédition non critique

4) Opportuniste / planifié :
- rangement long terme
- inventaires
- nettoyages non urgents
- réorganisation racks

### 9.4 Réservations / anti-conflits (obligatoire)

Dès qu’un agent choisit un Job :
- il “claim” le job
- il réserve les ressources nécessaires (items, machine, slot dépôt, console de réglage…)

Règles :
- une UL/item ne peut être réservée que par un agent à la fois
- destinations (slots) sont réservables aussi
- réservations doivent avoir un mécanisme de libération (TTL ou release explicite)
- si un job devient impossible :
    - état Bloqué (raison explicite)
    - libérer réservations
    - replanification

Granularité :
- finition : réserve 1 sac à la fois
- cariste : réserve UL + slot destination
- cassage : réserve lot/caisse selon étape
  => permet parallélisation sans chaos.

### 9.5 Interruptions & reprise propre

Interruption autorisée uniquement à des “points sûrs” (checkpoints) :
- entre sous-jobs
- ou quand l’objet transporté peut être posé dans un endroit valide

Exemple :
- un cariste ne lâche pas une palette au milieu d’un couloir ; il finit le dépôt ou va au point “safe”.
- un opérateur peut interrompre un nettoyage si buffer devient critique.

### 9.6 Contrôle joueur

Deux leviers :
- Programmation : grille de priorités par agent (style RimWorld)
- Ordre direct : assigner un job précis (prioritaire), sauf contraintes dures

Si ordre impossible :
- l’UI doit afficher une raison claire (slot plein, lot bloqué, machine réservée, accès interdit…).

### 9.7 IA débuggable (overlay obligatoire)

Pour chaque agent, afficher :
- job courant
- cible
- priorité
- raison principale (composantes score)
- cause de blocage (si Bloqué)

---

## 10) Jobs et zones “dataset de départ” (exemples concrets)

Ces exemples servent à valider le framework (ils peuvent évoluer), mais les patterns doivent rester génériques.

### 10.1 Zone Finition / Conditionnement
Job : Opérateur de ligne (déshydratation/finition)
- Flux : sacs bleus (conformes) / sacs rouges (rejets) → box → zone tampon cariste
- Sous-jobs atomiques :
    - attendre sac sur descente
    - réserver sac
    - transporter
    - contrôler poids
    - ajuster poids (si hors tolérance)
    - sceller
    - étiqueter
    - placer dans box
    - incrémenter compteur
    - si box pleine → clôture box (filmer, étiqueter, déplacer tampon, reset)

Contraintes :
- séparation stricte bleu/rouge
- un sac = un seul opérateur
- buffer descente peut saturer → risque bourrage → job intervention prioritaire.

KPI :
- débit sortie
- non-conformité
- temps attente cariste
- arrêts ligne
- coût MO/unité

### 10.2 Zone Préparation / Alimentation ligne
Rôle : transformer matière brute (ail/oignon/échalote) en flux régulé vers déshydratation.

Cas oignon/échalote :
- big bags suspendus → stockeur amont → épierreur → stockeurs/tapis → coupeuse → four

Cas ail :
- caisses grises → vidage dans vis sans fin → stockeur tampon (%) → aval

Paramètres pilotables :
- vitesse vis
- vitesses tapis (Hz)
- température four
- vitesse four

Objectif :
- flux continu stable
- éviter surcharge/sous-alimentation
- optimiser cadence sans dépasser seuils critiques
- qualité (clair, sec)

Jobs :
- Opérateur Préparation (surveillance + micro-interventions + nettoyage)
- Chef d’équipe Préparation (pilotage process + alimentation matière + réglages)

KPI :
- micro-arrêts/arrêts
- bourrages
- salissure moyenne
- débit stable vers four
- énergie (four) / coûts

### 10.3 Zone Cassage / séparation gousses
Job : Casseur
- Timer soufflerie 8h (mise en cycle / sortie lot prêt)
- Transport clark (cariste-like mais poste dédié)
- Réglage cadence casseuse selon lot
- Gestion déchets pelures
- Nettoyage fréquent

KPI :
- rendement matière
- pertes
- énergie machine
- qualité matière vers préparation
- charge nocturne

### 10.4 Zone Stockage / Logistique interne (transverse)
Job : Cariste
- transporte UL : big bags, caisses bois, caisses grises, box, palettes
- reçoit demandes pickup/drop-off + réceptions + inventaires + désengorgement

Priorité stricte :
- arrêt de ligne/congestion > appro prod > enlèvements > rangement > inventaire

Contraintes :
- UL et slots réservables
- si pas de destination → Bloqué + alerte

KPI :
- temps pickup→drop-off
- temps attente opérateurs
- arrêts faute appro
- précision inventaire
- coût logistique

### 10.5 Services transverses (supervision)
- Responsable production : arbitrage global (goulot, réallocation, consignes, redémarrage)
- Direction : objectifs/budget/stratégie (rythme lent), peut imposer pression/audit en mode challenge
- Qualité : contrôle lot, blocage, demande rework/nettoyage/réglage, impact audits/retours
- Maintenance : curatif + préventif, consignation sécurité, redémarrage
- ADV / Expédition : commandes, réservation stock, picking (cariste), staging, expédition, OTIF

---

## 11) Comportements hors travail (Sims-like au travail)

Tout comportement (travail ou non) est modélisé comme Job.

Shifts :
- prise de poste : rejoindre zone, se positionner
- fin de poste : clôture propre (checkpoint), dépôt safe, tâches de clôture
- un agent ne “disparaît” pas au milieu d’une action critique

Hors shift :
- off-site (hors carte), besoins se rechargent selon règles simples
- option : rappel/heures sup en crise (coût moral/fatigue/risque départ)

Besoins personnels :
- jobs personnels : pause, repos, manger/boire, toilettes, se calmer
- n’interrompent pas une urgence critique sauf danger (niveau critique)

Sécurité / auto-préservation :
- si risque élevé, agent peut ralentir/demander maintenance/nettoyage plutôt que comportement absurde.

Social :
- micro-interactions quand charge faible ; modulent moral/conflits/cooperation.

Formation (si fenêtre) :
- job “formation” abstrait v0 ; progression lente ; récompense de la stabilité.

Idle intelligent :
- pas d’errance gratuite : attendre aux points logiques, petits rangements, repos.

---

## 12) Économie & KPI (boucle centrale)

Boucle argent :
- acheter matière première
- transformer via process
- vendre produit fini
- payer OPEX (salaires, énergie, maintenance, consommables)
- subir/éviter pénalités (qualité, retards, accidents, audits)
- investir CAPEX (machines, upgrades, infra, effectifs)

KPI globaux (exemples) :
- cash, profit, marge, coûts (MO/énergie/maintenance)
- débit (unités/heure), temps de cycle, OEE (si on le modélise)
- rebuts / pertes matière
- downtime (arrêts)
- OTIF (On-Time In-Full) expédition
- incidents/accidents
- moral global / turnover / absentéisme
- niveau stock (cash immobilisé), saturation zones tampons

KPI locaux par zone :
- cadence cible vs réelle
- taux rebut local
- salissure / risque panne
- congestion (buffers)
- qualité locale (défauts détectés)

Règle d’or :
- L’argent est la vérité du système. Toute feature doit avoir un chemin causal lisible vers KPI → argent.

---

## 13) Mode construction / optimisation (gameplay)

Le joueur doit pouvoir :
- acheter / placer / déplacer / vendre-démolir
- améliorer (upgrades)
- reconfigurer (zones, flux, accès)

Placement sur grille :
- collisions
- règles d’accès (portes, couloirs, zones interdites)
- accessibilité (pathfinding)

Impact direct simulation :
- capacités, coûts, maintenance, flux, qualité, sécurité, fatigue/stress.

---

## 14) Sauvegarde / chargement (data-driven)

Exigences :
- Layout usine initial = blueprint data-driven (RON/JSON)
- Le joueur peut sauvegarder/charger ses layouts.
- Les saves doivent être versionnées (schema_version) + tolérantes aux champs ajoutés (defaults).

---

## 15) Instrumentation & debug (obligatoire pour “aquarium”)

Overlays attendus :
- overlay IA par agent (job, cible, raison, blocage)
- overlay zones (type zone, contraintes, KPI locaux, risques)
- overlay machines (état usure/panne, cadence, énergie)
- overlay réservations (items/machines/slots)
- overlay flux/buffers (files, saturation, stockeurs %)
- overlay traçabilité lot (où est le lot, statut qualité, historique)

Le jeu doit être “observabilité-first” : si un joueur voit un problème, il doit pouvoir comprendre pourquoi.

---

## 16) Contraintes d’implémentation (rappel)

- Rust stable, crates minimales.
- macroquad pour rendu/input 2D.
- serde + ron (ou json) pour blueprints/saves.
- Pas d’assets externes requis (placeholders OK).
- Simulation séparée rendu, tick fixe.
- Pathfinding A* sur grille.
- Système de réservation pour éviter conflits et comportements absurdes.

---

## 17) Règles pour éviter les erreurs classiques (agent/dev)

- Ne jamais hardcoder “ail-only” : ail = contenu data-driven de départ.
- Ne pas mélanger temps réel et temps simulation.
- Ne pas faire dépendre la logique métier du framerate.
- Toujours gérer les cas Bloqué avec raison + libération réservations.
- Toujours prévoir interruptions/reprises propres.
- Toujours relier une mécanique à KPI/argent (sinon c’est du bruit).

FIN.
