# ExecPlan - Application complete de `plan.md`

Date: 2026-02-21
Portee: Etapes 2 a 6 du plan (l'etape 1 est deja integree).

## Objectifs observables
- Ajouter un framework simulation data-driven pour:
  - items + lots
  - blocs physiques sur grille + inventories
  - zones logiques + regles
  - jobs + reservations avec TTL
  - 1 agent autonome avec decision debuggable
  - KPI globaux + KPI zone
  - build mode minimal (placer/deplacer/vendre) + peinture de zones
  - sauvegarde/chargement layout usine (starter factory) en RON
- Garder la separation simulation/rendu et le tick fixe.
- Garder une usine de depart rentable et stable.

## Invariants
- Pas de logique metier dependante du frame-rate.
- Tick simulation: uniquement via `FactorySim::step(FIXED_DT)`.
- Donnees de demarrage chargeables depuis fichier RON.
- Jobs impossibles => etat `Blocked` + liberation des reservations.
- Reservations avec expiration (TTL) et liberation explicite.
- Build mode ne doit pas casser le demarrage du jeu ni les tests existants.

## Milestones
1. Refondre `src/sim.rs` en noyau items/blocs/zones/jobs/agent/kpi/build.
2. Ajouter un asset usine starter (`data/starter_factory.ron`) avec version schema.
3. Integrer les controles play mode:
   - F6 overlay zones
   - F7 build mode
   - B/N/V/M pour brosses/modes/deplacement
   - clic gauche/droit pour placement/suppression/paint
4. Integrer overlays de debug:
   - blocs + inventories
   - jobs + blocages + reservations
   - agent + raison de decision (score)
   - KPI globaux + KPI zone
5. Sauvegarde layout:
   - F8 sauvegarde layout usine
   - chargement auto au demarrage de la simulation
6. Validation:
   - `cargo fmt`
   - `cargo clippy --all-targets --all-features -- -D warnings`
   - `cargo test`
   - `cargo run` (smoke avec timeout)

## Fichiers impactes
- `src/sim.rs`
- `src/main.rs`
- `data/starter_factory.ron`
- `docs/PLANS.md`

## Risques
- Densite du nouveau module simulation et regressions de clippy.
- Couplage UI play mode vs commandes existantes (click-to-move).
- Equilibre economique trop agressif en build mode (CAPEX/refund).

## Strategie de test
- Tests unitaires dans `src/sim.rs`:
  - progression jobs/agent
  - expiration reservations
  - serialization layout
  - impact zones sur production
  - economie/kpi non nuls
- Smoke test jeu:
  - toggle F6/F7
  - placement/suppression bloc
  - sauvegarde F8
  - verification HUD debug F1

# ExecPlan - Refonte UI Editeur + Zoom sensible + Fluidite

Date: 2026-02-21
Portee: refonte interface editeur, camera/zoom, optimisation rendu.

## 5 etapes de recherche/analyse
1. Inventorier les points de friction UX de `run_editor_frame` (densite panneau, lisibilite, feedback, raccourcis).
2. Mesurer la charge de rendu actuelle sur grande map (boucles tiles completes, overlays, grilles, props).
3. Identifier les chemins critiques de fluidite (sim steps backlog, draw calls inutiles hors viewport).
4. Definir un modele de camera editeur pan+zoom coherent avec gameplay et ergonomie souris.
5. Etablir un layout UI cible responsive (barre action, toolbox, inspector, viewport central).

## 10 etapes d'execution
1. Introduire des constantes camera/perf (zoom plus sensible, cap de simulation, vitesses de pan).
2. Ajouter un etat camera dedie a l'editeur (center/zoom/init) dans `EditorState`.
3. Mettre en place des helpers de bounds visibles (tuile min/max depuis camera+viewport).
4. Ajouter un rendu monde par region visible (floor, murs, ombres, grille) pour eviter le full redraw.
5. Adapter les overlays sim en mode jeu a ces bounds visibles.
6. Refaire `run_editor_frame` avec layout 3 zones: topbar, toolbox gauche, inspector droite.
7. Ajouter interactions camera editeur: molette sensible, pan fleches/Space+ZQSD, drag molette.
8. Conserver et harmoniser les actions critiques (save/load/undo/redo/play/menu, status, hover info).
9. Augmenter la sensibilite du zoom en mode jeu et garder clamp stable.
10. Ajouter un garde-fou anti-stutter (limite de ticks sim par frame) pour fluidite en charge.

## 10 etapes de verification
1. `cargo fmt`.
2. `cargo clippy --all-targets --all-features -- -D warnings`.
3. `cargo test` + tests unitaires ajoutes pour les nouveaux invariants perf/camera.
4. Validation manuelle: edition pinceau/rectangle sur zoom faible/fort.
5. Validation manuelle: pan editeur clavier + drag molette sur grande carte.
6. Validation manuelle: boutons topbar + raccourcis clavier sans regression.
7. Validation manuelle: hover/inspection case et points spawn P/N.
8. Validation manuelle: gameplay camera ZQSD + click-to-move limite map.
9. Validation manuelle: fluidite percue en fenetre et plein ecran.
10. Smoke run: lancement/fermeture sans crash ni blocage binaire.

# ExecPlan - Decoupage `main.rs` en modules logiques FR

Date: 2026-02-21
Portee: refactor structurel sans changement fonctionnel.

## Objectifs observables
- Reduire fortement la taille de `src/main.rs`.
- Introduire des fichiers nommes en francais pour clarifier les responsabilites:
  - `src/utilitaires.rs`
  - `src/edition.rs`
  - `src/deplacement.rs`
  - `src/rendu.rs`
- Garder le meme comportement en jeu/editeur et la meme boucle de tick fixe.

## Invariants
- Aucun changement de logique metier voulu (refactor only).
- Separation simulation/rendu intacte (`sim.rs` reste inchange sur son role).
- Compatibilite sauvegarde/chargement carte conservee.

## Milestones
1. Extraire les helpers generiques/camera/UI vers `utilitaires.rs`.
2. Extraire logique map/edition/sauvegarde vers `edition.rs`.
3. Extraire pathfinding + deplacements joueur/PNJ vers `deplacement.rs`.
4. Extraire rendu monde/props/overlays/HUD vers `rendu.rs`.
5. Reconnecter `main.rs` via `mod` + `use` et laisser les frames principales stables.
6. Validation complete (`fmt`, `clippy -D warnings`, `test`, smoke run).

## Fichiers impactes
- `src/main.rs`
- `src/utilitaires.rs` (nouveau)
- `src/edition.rs` (nouveau)
- `src/deplacement.rs` (nouveau)
- `src/rendu.rs` (nouveau)
- `docs/PLANS.md`

## Risques
- Imports/cycles de dependances entre modules.
- Fonctions devenues inaccessibles apres extraction (visibilite).
- Regressions de compilation dues aux changements de scope.

## Strategie de test
- Compiler apres extraction de chaque bloc pour isoler les erreurs.
- Executer:
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
- Smoke run `cargo run` pour verifier que menu/jeu/editeur s'ouvrent.

# ExecPlan - Suite de tests unitaires anti-regression (5 etapes)

Date: 2026-02-21
Portee: renforcer les tests unitaires sur les sous-systemes critiques pour accelerer les evolutions sans regressions.

## Objectifs observables
- Ajouter des tests unitaires pertinents sur `interactions`, `social`, `deplacement`, `utilitaires`.
- Capter les regressions de logique sociale (etats/cooldowns), de deplacement et de helpers fondamentaux.
- Conserver une suite compatible `clippy -D warnings`.

## Invariants
- Les tests doivent rester deterministes (pas de flaky).
- Chaque test doit verifier un contrat metier clair (pas de snapshots opaques).
- Pas d'introduction d'allocations lourdes ou de temps de test excessif.

## Plan en 5 etapes
1. Identifier les invariants anti-regression par sous-systeme.
2. Couvrir les contrats de `SocialActionKind` (classification + mappings).
3. Couvrir les transitions et cooldowns du moteur social.
4. Couvrir les helpers de deplacement et utilitaires purs.
5. Valider la suite complete avec `fmt`, `clippy -D warnings`, `test`.

## Fichiers impactes
- `src/interactions.rs`
- `src/social.rs`
- `src/deplacement.rs`
- `src/utilitaires.rs`
- `docs/PLANS.md`

## Strategie de test
- Unit tests localises dans chaque module pour acceder aux invariants internes.
- Validation globale:
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`

# ExecPlan - Correctifs visibilite interactions sociales

Date: 2026-02-22
Portee: rendre les interactions sociales visibles en partie reelle, reduire le bruit d'historique, renforcer la testabilite.

## Objectifs observables
- Les interactions sociales automatiques doivent etre perceptibles sans micro-manipulations.
- L'historique doit prioriser les evenements utiles par personnage.
- Le joueur doit decouvrir facilement les interactions sociales manuelles.
- Des tests doivent verrouiller les regressions principales.

## Invariants
- Determinisme social conserve (seed fixe, tick fixe).
- Aucune logique metier dependante du framerate.
- Pas de movement autonome force du joueur via auto-social.

## Milestones
1. Ajuster le moteur social auto (distance/chance/candidats) pour augmenter la visibilite.
2. Remplacer les logs sociaux globaux par des logs cibles (acteur/participants).
3. Basculer les changements d'activite du worker en categorie Travail, localement au worker.
4. Ameliorer la decouvrabilite en HUD (rappel du clic droit interaction sociale).
5. Ajouter des tests unitaires sur ciblage auto joueur et hygiene de log.

## Fichiers impactes
- `src/social.rs`
- `src/modes.rs`
- `docs/PLANS.md`

## Risques
- Interactions trop frequentes si les seuils sont trop permissifs.
- Regressions sur l'historique si les logs ne couvrent plus certains cas attendus.

## Strategie de verification
- `cargo fmt`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo run` smoke test
