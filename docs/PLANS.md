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
