# Rxchixs 
Un “factory sim” 2D top‑down sur grille, style RimWorld-like, où l’usine tourne toute seule (mode “aquarium”) et où le joueur intervient pour optimiser le layout, les machines, les flux et les humains.

Le jeu démarre **avec une usine déjà fonctionnelle et rentable** (starter factory). Ensuite, tout est modifiable : construire, déplacer, vendre/démolir, améliorer, reconfigurer.

---

## TL;DR (l’idée en 15 secondes)
- Simulation continue : l’usine vit, produit, s’encrasse, tombe en panne, se réorganise… même sans le joueur.
- Le joueur agit comme un manager/architecte : il observe les goulots, casse ou répare l’équilibre, et optimise.
- L’argent et les KPI sont la “vérité” du système : profit, pertes, rebuts, retards, maintenance, moral.

---

## Pilliers de design 
1) **Sandbox vivant**
- Machines avec état (usure → panne), rendements, coûts d’exploitation.
- Ressources et flux (stocks, délais, pertes).
- Humains autonomes (besoins, compétences, traits, décisions).
- Chaque décision du joueur a une conséquence économique lisible.

2) **Usine de départ (très important)**
- Le jeu commence avec une mini‑usine déjà équilibrée et jouable.
- Mais tout doit rester re‑architecturable : tu peux la repenser entièrement.

3) **Production par ZONES (framework générique, data‑driven)**
- Le jeu est structuré par zones de travail (logique), pas par un scénario figé.
- Chaque zone = règles + ressources + postes/machines + buffers + jobs/tâches + KPI locaux.

4) **IA style RimWorld**
- Système de jobs + priorités + réservations (ressources/machines/slots).
- Interruptions et reprise propre.
- IA débuggable : on doit pouvoir comprendre “pourquoi il fait ça”.

---

## Gameplay : ce que tu fais concrètement 
- Designer un layout (couloirs, zones, stockage, machines).
- Éviter les congestions et les “tampons” mal placés.
- Arbitrer cadence vs qualité vs maintenance vs fatigue humaine.
- Investir : upgrades machines, confort, sécurité, stockage, recrutement.
- Gérer les incidents : bourrages, pannes, dérives qualité, retards, saturation des racks…

---

## Systèmes clés (vue d’ensemble)

### Zones (environnements logiques)
Une zone n’est pas juste une couleur au sol. Elle définit :
- ce qu’on y fait (types de jobs),
- les contraintes (accès piéton/cariste, hygiène, température, bruit, sécurité),
- des KPI locaux (cadence cible, qualité attendue, rebut acceptable, stock tampon),
- des risques (bourrage, accident, panne, contamination),
- des effets humains (fatigue/stress, moral, vitesse d’exécution).

### Blocs (éléments physiques)
Tout ce qui se place sur la grille : sols, murs, portes, machines, racks, équipements…
Chaque bloc a un coût, un état (neuf → usé → panne), des stats, et génère des interactions gameplay.
La plupart des éléments sont “upgradables” par niveaux (crescendo), et peuvent se dégrader/casser.

### Personnages (humains vivants)
Modèle en barres 0→100 (data‑driven) :
- besoins vitaux (manger, boire, dormir, toilettes),
- qualité de vie (hygiène, social, confort, calme/stress…),
- compétences (production/qualité/logistique/maintenance/management…),
- traits (motivation, discipline, fiabilité, patience, courage…).
Les états humains influencent mécaniquement la performance (cadence/qualité) et le risque (erreurs/accidents), donc les KPI et l’argent.

### IA : Jobs / Priorités / Réservations
- Le monde génère des jobs (événements, seuils, timers, demandes inter‑zones).
- Chaque agent choisit un job via : faisabilité → score multi‑critères (priorité, urgence, distance, compétence, fatigue/stress…).
- Réservations obligatoires : éviter que deux agents prennent la même ressource / machine / destination.
- Système d’interruptions + reprise aux “points sûrs”.
- Ordres joueur : politique (grille de priorités façon RimWorld) + ordre direct ponctuel (si faisable).

### Économie & KPI (le centre du jeu)
- OPEX : énergie, salaires, maintenance, consommables, pénalités…
- Revenus : vente de produits finis.
- KPI : débit, rebuts, downtime, OTIF expédition, moral, incidents…
➡️ Toute mécanique doit avoir un chemin causal clair vers KPI → argent.

---

## Process de production “starter” (dataset de départ : ail) 
Le contenu évoluera, mais sert de base pour la starter factory :
1) Réception & stockage (lots / traçabilité)
2) Soufflerie : séchage ~8h (timer)
3) Cassage : séparation pelures/gousses → gousses au frigo, déchets en big bags
4) Déshydratation : lavage → découpe → séchoir (réglages impact qualité/cadence/énergie)
5) Finition : tri (sacs bleus conformes / sacs rouges rejets) + box
6) Palettisation & stockage
7) Vente (génère l’argent pour investir)

IMPORTANT : rien n’est “ail‑only”. Le framework doit rester générique et paramétrable via données (RON/JSON).

---

## Tech / contraintes 
- Rust stable.
- Rendu 2D / input : macroquad.
- Données & saves : serde + RON/JSON (blueprints layouts, zones, recipes…).
- Simulation séparée du rendu, tick fixe.
- Pathfinding A* sur grille.
- Réservations ressources/machines/slots pour éviter les collisions logiques.

---

## Démarrer (dev)
Prérequis :
- Rust stable (toolchain à jour)

Commandes utiles :
- Lancer le jeu :
  - `cargo run`
- Tests :
  - `cargo test`
- Formatage :
  - `cargo fmt`
- Lint :
  - `cargo clippy --all-targets --all-features -- -D warnings`

---

## Contribution 
Ce projet vise une base très “systémique” (simulation + IA + économie). Pour contribuer proprement :
- privilégie les systèmes data‑driven (évite le hardcode de contenu),
- garde la séparation simulation / rendu,
- ajoute des overlays/debug quand tu touches à l’IA, aux réservations ou aux flux,
- garde l’usine de départ jouable (ne pas casser le “starter aquarium”).

---

## Statut
Prototype / en construction. Le but est d’itérer vite avec des placeholders (formes/couleurs), tout en gardant une architecture solide et extensible.

---
