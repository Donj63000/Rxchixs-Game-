# AGENTS.md - Rxchixs (Codex)

Ce depot correspond au projet **Rxchixs**: un jeu 2D top-down sur grille, style RimWorld-like, qui simule une usine vivante en continu.
Le jeu demarre avec une usine deja fonctionnelle (layout + zones + machines + stocks + personnel), et le joueur peut tout modifier (construire, deplacer, vendre, ameliorer, reconfigurer).

## Intention produit
- Le systeme tourne seul; le joueur intervient pour optimiser.
- L'argent est l'indicateur principal.
- Toute feature doit etre lisible via un chemin causal: actions -> KPI -> finances.

---

## Contraintes non negociables
- Rust stable uniquement.
- Pas d'`unsafe` sauf justification explicite + commentaire + test cible.
- Separation stricte simulation / rendu.
- Tick simulation fixe (cible 60 Hz) avec horloge de simulation explicite.
- Aucune logique metier dependante du framerate ou du temps reel OS.
- Data-driven partout ou possible (RON/JSON pour donnees de depart, layout, zones, machines, process).
- L'usine de depart doit rester jouable, rentable et stable sans micro-management.

---

## Stack et dependances
- Stack attendue: `macroquad`, `serde`, `ron`.
- Dependances minimales.
- Ne pas ajouter de crate production sans justification solide:
  - gain concret;
  - alternatives considerees;
  - impact maintenance.
- Pas d'assets externes requis (placeholders formes/couleurs acceptes).

---

## Workflow d'execution
Avant de coder:
- Lire la structure existante et la respecter.
- Eviter toute reorganisation du depot sans necessite.
- En cas de doute, faire une passe lecture (modules, flux, invariants).

Pour toute modification de code:
- Planifier explicitement des taches de verification.
- Ajouter ou mettre a jour les tests unitaires pertinents.
- Si test impossible, documenter pourquoi + proposer une verification alternative.
- Ne pas hesiter a faire du multi-fichiers quand la logique grandit (eviter les fichiers monolithiques).
- Si une fonctionnalite melange plusieurs responsabilites, la decouper en modules/fichiers par domaine logique.

Apres chaque modification significative (dans cet ordre):
- `cargo fmt`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo run` (smoke test de demarrage)
- `cargo run --release` si changement perf

---

## Definition of Done (DoD)
Une tache est terminee seulement si:
- les taches planifiees sont executees;
- chaque exigence impactee est verifiee;
- les tests touches passent;
- la suite complete passe (`fmt`, `clippy`, `test`);
- un smoke run est realise;
- le compte-rendu final liste:
  - tache -> fichiers modifies;
  - tache -> tests/verifications executes;
  - resultat final (OK / limite connue).

---

## Robustesse pro (obligatoire)
- Politique bugfix: toute correction de bug ajoute un test de reproduction du bug (qui echoue avant correction et passe apres correction), sauf impossibilite documentee.
- Determinisme: toute logique simulation et tout test doivent etre deterministes (seed explicite, pas d'aleatoire implicite).
- Invariants explicites: chaque module critique (jobs, reservation, pathfinding, economie) doit documenter ses invariants de securite et coherence.
- Pas de fallback silencieux: tout comportement degrade doit etre explicite (etat, raison, impact), sans masquer une erreur metier.
- Compatibilite des saves: tout changement de schema impose gestion `schema_version` et test de chargement version precedente.
- Budget performance: toute feature touchant la boucle de tick doit verifier l'absence de regression significative (mesure avant/apres ou instrumentation equivalente).
- Tests minimaux par feature: ajouter au minimum un test unitaire de logique pure + un test d'integration cible quand pertinent.
- Tests de charge sur systemes critiques: verifier absence de deadlock/starvation sous charge (agents/jobs/reservations).
- Observabilite IA: toute decision IA doit etre auditable (score principal, contraintes, raison de blocage).
- Discipline TODO/FIXME: interdit sans reference explicite (ticket/raison/condition de sortie).
- Tracabilite de livraison: chaque livraison doit mapper `exigence -> code -> test -> verification`.

---

## Conventions de code Rust
- Code simple, lisible, modulaire.
- Preferer structs + traits clairs plutot que macros complexes.
- Erreurs via `Result` + erreurs typees (`anyhow` seulement si deja present).
- Architecture: chaque partie logique doit avoir son fichier/module dedie (UI, simulation, interactions, economie, etc.).
- Eviter les fichiers "fourre-tout" avec enormement de code: extraire des sous-modules des qu'un fichier devient trop dense.
- Nommage: utiliser des noms de fichiers en francais pour les nouveaux modules (coherents, explicites, stables).
- Eviter allocations par tick.
- Eviter `clone()` de structures lourdes dans la boucle simulation.
- Logique grille en coordonnees entieres.
- Floats reserves au rendu/interpolation.

---

## Conventions simulation
- Systeme robuste aux interruptions, blocages et etats incoherents.
- Reservations sans deadlock silencieux:
  - TTL ou liberation explicite;
  - job impossible => etat Bloque + raison + liberation + replanification.
- Les zones sont logiques (regles/KPI/risques), pas seulement visuelles.
- Les blocs sont physiques sur grille et portent l'etat (usure, panne, stats, upgrades).

---

## IA / Jobs
- Un job est une unite de travail structuree, decomposable en sous-jobs atomiques.
- Chaque sous-job doit etre reservable, interruptible, reprenable.
- Selection de job en 2 etapes:
  - faisabilite (pathfinding, autorisations, ressources reservables);
  - scoring (priorite joueur, urgence, distance, competence, fatigue/stress, risque).
- Overlay debug IA obligatoire avec au minimum:
  - job courant;
  - cible;
  - priorite;
  - pourquoi (composantes principales du score) ou raison de blocage.

---

## Sauvegarde / chargement
- Blueprints et saves versionnes (`schema_version`).
- Tolerance aux champs ajoutes (defaults).
- Les donnees de demarrage usine doivent etre loadables depuis fichier (RON/JSON).

---

## Review guidelines
Toute PR doit conserver:
- tick fixe + separation sim/rendu;
- usine de depart fonctionnelle;
- logique data-driven (pas de hardcode de contenu scenario).

Red flags P0/P1 a signaler:
- crash, panic, out-of-bounds, invalid borrow;
- regression perf evidente dans la boucle tick;
- dependance ajoutee sans justification;
- logique metier dependante du framerate/temps reel OS;
- reservation pouvant mener a deadlock silencieux.

Tests:
- Exiger des tests unitaires des que la logique est pure (simulation, pathfinding, reservations, scoring IA).
- Si test impossible: documenter la raison + proposer une validation concrete (instrumentation, overlay, scenario de repro).

---

## ExecPlans (gros chantiers)
Si la demande implique une refonte multi-fichiers, une nouvelle grosse feature, ou plus d'un sous-systeme:
- Ecrire d'abord un ExecPlan avant implementation.
- L'ExecPlan doit inclure:
  - objectifs observables;
  - invariants;
  - milestones;
  - fichiers impactes;
  - risques;
  - strategie de test.
- Si aucun fichier de plan n'existe, creer `docs/PLANS.md` (ou `PLANS.md`).

---

## Regle speciale: "applique PLAN.md"
Quand l'utilisateur demande explicitement "applique PLAN.md" (ou equivalent), l'agent doit:
1. Lire `PLAN.md` en entier.
2. Extraire toutes les taches actionnables, sans exception.
3. Construire un plan d'execution exhaustif.
4. Executer toutes les taches planifiees jusqu'a completion.
5. Verifier point par point que chaque tache du plan est bien appliquee dans le code.
6. Valider le fonctionnement global (`fmt`, `clippy`, `test`, `run` smoke).
7. Livrer un compte-rendu final avec preuves de verification et ecarts eventuels.

Si un point du plan est ambigu, contradictoire ou impossible:
- ne pas l'ignorer;
- le signaler explicitement;
- proposer une resolution concrete;
- poursuivre sur le reste sans bloquer inutilement.

---

## Notes produit
- Le jeu est un bac a sable economique et humain.
- Le joueur observe, comprend les goulots, et optimise.
- Toute derive (humains, machines, zones, flux) doit se lire dans KPI -> finances.
