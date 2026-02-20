# AGENTS.md — Rxchixs (Codex)

Ce dépôt correspond au projet **Rxchixs** : un jeu 2D top-down sur grille, type “RimWorld-like”, qui simule une usine vivante (“aquarium”) en continu.
Le jeu démarre avec une **usine déjà fonctionnelle** (layout + zones + machines + stocks + personnel) et le joueur peut tout modifier (construire/déplacer/vendre/améliorer/reconfigurer).

Objectif produit : le système tourne seul, le joueur intervient pour optimiser, et l’argent/KPI :contentReference[oaicite:13]{index=13}y:contentReference[oaicite:14]{index=14}s:contentReference[oaicite:15]{index=15}h:contentReference[oaicite:16]{index=16}
- **Rust stable uniquement.** Pas d’`unsafe` sauf justification + commentaire + test ciblé.
- **Séparation stricte simulation / rendu.**
    - La simulation doit pouvoir tourner sans rendu (testable).
    - Tick fixe (cible : 60 Hz) avec une horloge de simulation explicite (pas `std::time::Instant` comme source de logique métier).
- **Data-driven partout où c’est possible.**
    - Layout usine initial + zones + machines + recettes/process doivent pouvoir être décrits via RON/JSON.
    - Évite de hardcoder un scénario unique : l’ail est un dataset de départ, pas une logique figée.
- **Le départ doit rester jouable** : une usine “par défaut” rentable et stable doit exister et tourner sans micro-management.
- **IA style RimWorld (exigence majeure).**
    - Système de Jobs + priorités + réservations (items/machines/slots) + interruptions/reprise propre.
    - Tout choix IA doit être **déboggable** (raison affichable).
- **Économie/KPI d’abord.**
    - Toute feature doit avoir un chemin causal lisible vers KPI → argent (coûts, rebuts, retards, maintenance, moral).

---

## Stack & dépendances

- Stack attendue : `macroquad` (rendu/input 2D), `serde` + `ron` (blueprints/sauvegardes).
- **Dépendances minimales.** N’ajoute pas de nouvelle crate “production” sans forte raison.
    - Si tu proposes une dépendance : explique le gain, les alternatives, et l’impact maintenance.
- Pas d’assets externes requis : placeholders (formes/couleurs) OK.

---

## Comment travailler dans le repo (workflow Codex)

Avant de coder :
- Lis la structure existante et **respecte-la**. Ne réorganise pas le dépôt sans nécessité.
- Si tu n’es pas sûr, commence par faire une passe “lecture” : comprendre modules, flux, invariants.

Après chaque modification significative :
- Lance (dans cet ordre) :
    - `cargo fmt`
    - `cargo clippy --all-targets --all-features -- -D warnings`
    - `cargo test`
- Vérifie que le jeu démarre :
    - `cargo run`
    - (optionnel) `cargo run --release` si tu touches aux perfs

---

## Conventions de code (Rust)

- Style : simple, lisible, modulaire. Préfère des structs + traits clairs plutôt que des macros “magiques”.
- Erreurs : utilise `Result` + erreurs typées (ou `anyhow` uniquement si déjà présent).
- Perf simulation : évite allocations par tick, évite `clone()` sur des structures lourdes dans la boucle principale.
- Coordonnées grille : préfère des entiers (cellules) pour la logique. Les floats sont uniquement pour le rendu/interpolation.

---

## Conventions de simulation (très important)

- Le “monde” est une simulation continue : les systèmes doivent être robustes aux interruptions, aux blocages, et aux états incohérents (ex: un slot devient indisponible).
- Évite les deadlocks de réservation :
    - Les réservations ont un TTL ou une libération explicite.
    - Si un job devient impossible, il passe en “Bloqué” avec raison, libère ses réservations, et l’agent re-planifie.
- Les “zones” sont **logiques** (règles/KPI/risques), pas juste visuelles.
- Les “blocs” sont **physiques** sur grille et portent l’état (usure, panne, stats, upgrades, etc.).

---

## IA / Jobs (règles de conception)

- Un Job = unité de travail structurée, décomposable en sous-jobs atomiques :
    - réservable, interruptible, reprenable.
- La sélection de job doit suivre :
    - faisabilité (pathfinding, autorisations, ressources réservables) puis scoring (priorité joueur, urgence, distance, compétence, fatigue/stress, risque).
- Ajoute/maintient un overlay debug IA affichant au minimum :
    - job courant, cible, priorité, et “pourquoi” (composantes principales du score), ou raison de blocage.

---

## Sauvegarde / chargement

- Les blueprints et saves doivent être compatibles versionnement :
    - inclure une version de schema (ex: `schema_version`) et être tolérant aux champs ajoutés (defaults).
- Toute donnée “démarrage usine” doit être loadable depuis fichier (RON/JSON).

---

## Review guidelines (pour Codex Review / PR)

- Toute PR doit garder :
    - simulation tick fixe + séparation sim/rendu
    - usine de départ fonctionnelle
    - logique data-driven (pas de hardcode de contenu)
- Red flags à signaler (P0/P1) :
    - crash, panic, borrow invalid, accès out-of-bounds
    - régression perf évidente dans la boucle tick
    - ajout de dépendance non justifiée
    - logique métier dépendante du framerate ou du temps réel OS
    - réservation pouvant créer deadlock silencieux
- Exige des tests unitaires dès que la logique est pure (simulation, pathfinding, réservation, scoring IA).
    - Si un test est impossible : expliquer pourquoi et proposer un moyen de validation (ex: instrumentation + overlay).

---

## ExecPlans (pour gros chantiers)

Si la demande implique une refonte multi-fichiers, une nouvelle grosse feature, ou >1 zone/sous-système :
- Écris d’abord un **ExecPlan** (plan d’exécution) avant d’implémenter.
- L’ExecPlan doit lister : objectifs observables, invariants, étapes/milestones, fichiers impactés, risques, stratégie de test.
- Si aucun fichier de plan n’existe dans le repo, crée `PLANS.md` (ou `docs/PLANS.md`) et ajoute le plan dedans.

---

## Notes produit (rappel de l’intention)

- Le jeu est un bac à sable économique et humain : le joueur observe, comprend les goulots, et optimise.
- L’argent est l’indicateur principal : toute dérive (humains, machines, zones, flux) doit se lire dans KPI → finances.
