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

# ExecPlan - Chariot elevateur (Clark jaune) conduisible + manutention caisses

Date: 2026-02-22
Portee: ajout d'un vehicule gameplay conduit par le joueur, avec manipulation de caisses, rendu detaille et verifications completes.

## Objectifs observables
- Ajouter un chariot elevateur jaune visible en jeu des le demarrage.
- Permettre au joueur de monter/descendre du chariot.
- Permettre de deplacer des caisses via fourches (charger/decharger).
- Rendre le chariot visuellement travaille (carrosserie, masts, fourches, roues, gyrophare, details cabine).
- Garder la boucle de simulation fixe et deterministic-friendly.

## Invariants
- Tick fixe conserve: aucune logique vehicule dependante du framerate render.
- Aucune mutation de schema de sauvegarde carte necessaire pour cette feature (etat runtime).
- Conduite et manutention sans fallback silencieux: action impossible => raison explicite dans HUD.
- Deplacement chariot respecte collisions murs (meme grille monde que le joueur).
- Deplacement de caisse: pas de duplication/perte silencieuse.

## Milestones
1. Creer un module dedie `src/chariot_elevateur.rs` (etat, orientation, mouvement, interaction caisses).
2. Integrer le chariot dans `GameState` et l'initialiser a un spawn jouable.
3. Integrer commandes gameplay:
   - `E`: monter/descendre
   - `F`: charger/decharger caisse
4. Integrer update fixe:
   - conduite chariot en pas fixe
   - synchronisation position joueur conducteur
5. Ajouter rendu detaille du chariot + caisse transportee.
6. Ajouter feedback HUD/debug pour etat conduite/cargo.
7. Ajouter tests (unitaire + integration cible), puis `fmt`, `clippy`, `test`, `run` smoke.

## Fichiers impactes (prevus)
- `src/main.rs`
- `src/modes.rs`
- `src/rendu.rs`
- `src/chariot_elevateur.rs` (nouveau)
- `docs/PLANS.md`

## Risques
- Conflits d'inputs avec commandes de deplacement existantes.
- Artefacts de layering visuel entre props/personnages/chariot.
- Regressions comportementales du joueur hors conduite.

## Strategie de test
- Tests unitaires `chariot_elevateur`:
  - detection caisse transportable
  - cycle charge/decharge deterministe
- Test d'integration cible:
  - `GameState` initialise un chariot valide sur tuile marchable
- Validation complete:
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo run` (smoke lancement)

## Risques
- Interactions trop frequentes si les seuils sont trop permissifs.
- Regressions sur l'historique si les logs ne couvrent plus certains cas attendus.

## Strategie de verification
- `cargo fmt`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo run` smoke test

# ExecPlan - Systeme de sauvegardes nommees + horodate + menu robuste

Date: 2026-02-22
Portee: remplacer la sauvegarde unique par un systeme multi-sauvegardes avec UX complete dans le menu pause.

## Objectifs observables
- Pouvoir creer plusieurs sauvegardes nommees.
- Afficher une horodate lisible pour chaque sauvegarde.
- Proposer un menu de sauvegarde avec saisie du nom et liste des slots existants.
- Proposer un menu de chargement avec selection explicite d'une sauvegarde.
- Rendre le tout robuste: schema versionne, erreurs explicites, liste tolerante aux fichiers invalides.

## Invariants
- Tick simulation fixe conserve (pause = simulation completement gelee).
- Pas de fallback silencieux: toute erreur de sauvegarde/chargement doit etre visible.
- Schema de sauvegarde versionne (`schema_version`) et verifie au chargement.
- Donnees chargees sanitisees avant reconstruction de l'etat de jeu.

## Milestones
1. Creer un module `src/sauvegarde.rs` dedie:
   - format de fichier,
   - nommage/horodatage,
   - listing des slots,
   - chargement/sauvegarde robustes.
2. Integrer l'etat UI pause necessaire dans `GameState`.
3. Transformer le menu pause:
   - ecran sauvegarder (nom + horodate + liste + actions),
   - ecran charger (liste + selection + action charger).
4. Ajouter des tests unitaires de persistance et horodatage.
5. Executer la verification complete (`fmt`, `clippy -D warnings`, `test`, `run` smoke).

## Fichiers impactes
- `src/sauvegarde.rs` (nouveau)
- `src/modes.rs`
- `src/main.rs`
- `src/edition.rs`
- `docs/PLANS.md`

## Risques
- Erreurs de format de date/nom de fichier.
- Regressions d'ergonomie dans le menu pause.
- Fichiers de sauvegarde corrompus ou schema futur.

## Strategie de test
- Tests unitaires `sauvegarde`:
  - format horodate stable,
  - sanitation nom,
  - roundtrip save/list/load,
  - gestion fichier invalide,
  - rejet schema futur.
- Validation complete:
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo run` (smoke)

# ExecPlan - Refonte visuelle barre HUD basse (theme moderne colore)

Date: 2026-02-22
Portee: ameliorer le rendu de la barre basse (fonds, panneaux, pills KPI, boutons) avec un style recent, colore et soigne, sans changer la logique metier.

## Objectifs observables
- La barre HUD basse est visuellement plus moderne, lisible et coloree.
- Les boutons sont plus lisibles et mieux differencies (normal, hover, actif).
- Les panneaux ont une profondeur visuelle (degrade, ombres, highlights) sans perdre la clarte.
- Les KPI restent parfaitement lisibles et priorises.

## Invariants
- Aucune logique simulation modifiee.
- Aucun comportement dependant du framerate/temps reel OS ajoute dans la simulation.
- Separation simulation/rendu preservee.
- Les interactions HUD existantes (clic, hover, scroll) restent intactes.

## 10 taches de recherche
1. Cartographier les points d'entree de rendu HUD dans `src/ui_hud.rs`.
2. Identifier les primitives visuelles communes (couleurs, cadres, boutons, pills).
3. Relever les fonctions partagees par plusieurs panneaux pour eviter les regressions.
4. Evaluer les contraintes de lisibilite texte (contraste clair/fonce).
5. Etudier les etats interactifs existants (hover, actif, inactif) et leur signal visuel.
6. Delimiter ce qui appartient strictement a la barre basse vs fenetres modales.
7. Concevoir une palette cible (base, surface, accents, danger, succes) coherente.
8. Definir les nouveaux effets autorises sans cout excessif (degrades, lignes de separation, glow subtil).
9. Verifier les points de test unitaire possibles pour helpers visuels purs.
10. Structurer un plan d'implementation incremental pour eviter un gros patch opaque.

## 10 taches d'execution robustes
1. Introduire des helpers visuels purs reutilisables (`mix_color`, `draw_vertical_gradient`, etc.).
2. Refaire le fond de barre basse avec degrade multicouche et separations propres.
3. Harmoniser les couleurs de base (`ui_col_*`) vers une direction plus recente et coloree.
4. Revoir `draw_panel_frame` pour un rendu panneau plus premium (profondeur + header plus net).
5. Reviser `draw_top_strip` pour aligner l'identite visuelle avec le nouveau theme.
6. Refaire `draw_stat_pill` (fond, accent, contraste, hover) sans casser les donnees affichees.
7. Refaire `draw_small_button` (normal/hover/actif) avec une hierarchie visuelle plus claire.
8. Ajuster les contrastes textes/ombres pour conserver la lisibilite globale.
9. S'assurer que les overlays/fenetres utilisant les memes primitives restent coherents.
10. Nettoyer le code modifie (noms, commentaires courts utiles, suppression des redondances).

## 10 taches de verification completes
1. Ajouter/adapter un test unitaire de helper visuel pur pour verrouiller le comportement attendu.
2. Verifier manuellement le rendu de la barre basse en resolution standard.
3. Verifier manuellement en petite resolution (HUD compact) la lisibilite des boutons.
4. Verifier manuellement en grande resolution (HUD etendu) la coherence du layout.
5. Verifier manuellement les etats de boutons vitesse (`Pause`, `1x`, `2x`, `4x`).
6. Verifier manuellement la lisibilite des pills KPI (notamment valeurs negatives/positives).
7. Executer `cargo fmt`.
8. Executer `cargo clippy --all-targets --all-features -- -D warnings`.
9. Executer `cargo test`.
10. Executer un smoke test `cargo run`.

# ExecPlan - Menu construction complet (prix) + zones metier + racks 5 niveaux + vente conditionnelle

Date: 2026-02-22
Portee: transformer le menu construction et la logique associee pour basculer vers des zones metier (stockage/cassage/dehy-finition/vente), un systeme de racks palettes 5 niveaux operable au Clark, et une vente dependante d'un bureau + responsable.

## Objectifs observables
- Le menu construction liste clairement les elements achetables avec prix estimes (zones, sols, racks, bureau vente).
- Le stockage est une zone rectangulaire bleue, sans bloc "machine" associe.
- "Machine A" / "Machine B" deviennent des zones metier:
  - "Zone de cassage"
  - "Zone de dehy/finition"
- Le "tampon" devient des racks palettes avec capacite superposee 5 niveaux par rack.
- Le Clark peut deposer/prendre une palette sur un niveau precis (RDC + niveaux 1..5) selon la hauteur de mat/fourches.
- La "vente" devient une zone dediee et n'est activee que si:
  - un bureau de vente est present dans la zone;
  - un responsable des ventes est assigne.

## Invariants
- Tick simulation fixe conserve (60 Hz, pas de logique metier liee au framerate).
- Separation simulation / rendu preservee.
- Pas de fallback silencieux: tout blocage metier expose une raison explicite en statut HUD.
- Compatibilite des saves preservee via `schema_version` et migration tolerance.
- Determinisme des tests conserve (pas d'aleatoire implicite).

## 10 taches de recherche
1. Auditer `src/ui_hud.rs` pour isoler les points d'entree du catalogue construction, details, couts, et actions d'application.
2. Auditer `src/sim.rs` pour identifier les couplages actuels "machines blocs" vs "zones metier".
3. Auditer `src/modes.rs` pour valider le flux clic gauche/droit build mode et integration des nouvelles actions.
4. Auditer `src/rendu.rs` pour verifier les overlays zones/couleurs et etiquetage attendu.
5. Auditer `src/chariot_elevateur.rs` pour mesurer les changements necessaires a la manutention multi-niveaux.
6. Auditer `src/sauvegarde.rs` et structures RON impactees pour schema/version/migration.
7. Inventorier les types d'objets achetables (sols, racks, bureau, zones) et definir une grille de prix estimee coherente.
8. Definir un modele de "zone rectangle" (debut/fin drag) compatible avec l'input actuel.
9. Definir le contrat metier "vente active" (bureau + responsable) et ses etats explicites.
10. Identifier les tests de reproduction a ajouter pour chaque bug/regression possible (zones, racks, vente, menu).

## 10 taches d'execution robustes
1. Introduire un catalogue data-driven d'achats (type, prix, mode d'application) pour le menu construction.
2. Mettre a jour `ui_hud` pour afficher tous les elements achetables et leurs prix (incluant sols et zones), avec details lisibles.
3. Renommer les libelles metier visibles:
   - "Zone stockage" (bleu),
   - "Zone de cassage",
   - "Zone de dehy/finition",
   - "Zone vente".
4. Implementer l'outil de peinture rectangle de zones (stockage/production/vente) dans le build mode.
5. Introduire les achats de sols (cout par tuile) dans le flux construction et leur application sur la carte.
6. Transformer le "tampon" en entite rack palettes:
   - capacite par rack,
   - niveaux adressables RDC + 1..5.
7. Etendre le Clark pour selectionner/viser le niveau de pose/reprise selon hauteur de mat/fourches.
8. Ajouter la logique metier de vente conditionnelle:
   - detection bureau de vente dans zone vente,
   - affectation responsable des ventes,
   - blocage explicite sinon.
9. Mettre a jour le debug HUD/overlay pour tracer:
   - etat des zones,
   - occupation racks par niveau,
   - etat activation vente.
10. Ajouter migration/schema save pour persister zones/racks/affectation et garantir chargement ancien schema.

## 10 taches de verification completes
1. Ajouter tests unitaires du catalogue achats (prix, disponibilite, mapping type->action).
2. Ajouter tests unitaires de couts zones/sols (debit/refund/clamp tresorerie).
3. Ajouter test unitaire de peinture rectangle zone (bornes incluses et in-bounds).
4. Ajouter test unitaire de racks (capacite max 5 niveaux, collisions de niveau, depot/reprise deterministes).
5. Ajouter test unitaire Clark->rack (mapping hauteur fourches -> niveau cible).
6. Ajouter test d'integration vente conditionnelle (sans bureau/sans responsable/avec les deux).
7. Ajouter test de compatibilite save/load (ancien schema -> migration -> fonctionnement).
8. Executer `cargo fmt`.
9. Executer `cargo clippy --all-targets --all-features -- -D warnings`.
10. Executer `cargo test` puis smoke `cargo run`.

## Fichiers impactes (prevus)
- `src/ui_hud.rs`
- `src/sim.rs`
- `src/modes.rs`
- `src/rendu.rs`
- `src/chariot_elevateur.rs`
- `src/sauvegarde.rs`
- `src/main.rs`
- `data/starter_factory.ron` (si rebasage layout de depart requis)
- `docs/PLANS.md`

## Risques
- Refonte metier large avec risque de regressions sur la simulation existante.
- Complexite d'etat pour racks multi-niveaux si non modulee proprement.
- Incoherence possible entre zone metier et rendu/overlays si migration partielle.
- Compatibilite save vulnerable si schema non versionne strictement.

## Strategie de test
- Approche incrementalement testee par sous-systeme (menu, zones, racks, vente).
- Priorite aux tests de logique pure deterministe avant verification visuelle.
- Validation complete obligatoire:
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo run` smoke.

# ExecPlan - Ligne de production de `plan.md` (connexion complete + rendu detaille)

Date: 2026-02-22
Portee: appliquer integralement les blocs de la ligne de production de `plan.md` (entree -> tri -> sacs), avec connexions fonctionnelles en simulation et rendu graphique soigne par element.

## Objectifs observables
- Tous les elements de `plan.md` sont placables via le menu construction avec prix lisible.
- La ligne est validee uniquement si les elements sont interconnectes dans le bon ordre.
- Le flux matiere suit les etapes modernes (lavage, coupe, dehydratation, floconnage, tri, sacs).
- Chaque bloc de la ligne possede un rendu visuel distinct et travaille.
- Le joueur peut orienter les blocs directionnels et visualiser l'orientation active.

## Invariants
- Tick simulation fixe et deterministe conserve.
- Separation simulation/rendu respectee.
- Pas de fallback silencieux: toute ligne incomplete expose une raison explicite.
- Compatibilite saves/layout conservee (defaults serde sur nouveaux champs).

## 10 taches de recherche
1. Relire `plan.md` et extraire les contraintes dimensionnelles de chaque bloc.
2. Auditer `src/sim.rs` pour valider le modele de `BlockKind`, footprints et orientation.
3. Auditer `src/sim.rs` pour verifier la logique de connectivite (ordre et liens autorises).
4. Auditer `src/ui_hud.rs` pour recenser le catalogue construction et les couts affiches.
5. Auditer `src/modes.rs` pour les raccourcis build (selection, orientation, echap).
6. Auditer `src/rendu.rs` pour identifier le point d'entree de rendu des blocs simu.
7. Definir une grammaire visuelle par bloc (couleurs, pieces, animations, lecture en vue top-down).
8. Verifier les besoins de culling viewport pour grands footprints (four, tremie, bac).
9. Identifier les tests logiques minimaux a ajouter (placement footprint, connectivite moderne).
10. Definir la strategie de validation finale (fmt, clippy, test, smoke run).

## 10 taches d'execution robustes
1. Finaliser les enums/donnees de simulation pour tous les blocs de `plan.md`.
2. Verrouiller les footprints orientes et la validation de placement multi-tiles.
3. Finaliser la verification de connectivite de la ligne moderne dans l'ordre metier.
4. Finaliser le tick moderne (etapes de production + sacs bleus/rouges + boxes bleues).
5. Etendre `BUILD_MENU_BLOCKS` avec tous les blocs achetables de la ligne et descriptions metier.
6. Integrer le raccourci `T` pour rotation d'orientation des blocs en mode construction.
7. Etendre le rendu overlay avec couleurs exhaustives pour tous les `BlockKind`.
8. Implementer un rendu detaille dedie par bloc (tremie, convoyeurs, bac eau animee, coupeuse, repartiteur, four, sortie, floconneuse, tuyaux, sortex, descentes sacs).
9. Adapter l'affichage debug/labels aux footprints (positionnement + culling par rectangle).
10. Mettre a jour la documentation et les statuts de build pour rendre la ligne lisible en jeu.

## 10 taches de verification completes
1. Ajouter un test unitaire: footprint oriente du four s'inverse bien (horizontal/vertical).
2. Ajouter un test unitaire: refus de placement en collision footprint.
3. Ajouter un test unitaire: la ligne moderne exige les blocs requis et la connectivite.
4. Verifier manuellement en jeu: placement/rotation des blocs avec `T`.
5. Verifier manuellement en jeu: activation de la ligne complete et message explicite en cas d'erreur.
6. Verifier manuellement en jeu: lisibilite visuelle de chaque bloc de la ligne.
7. Executer `cargo fmt`.
8. Executer `cargo clippy --all-targets --all-features -- -D warnings`.
9. Executer `cargo test`.
10. Executer un smoke run `cargo run`.

## Fichiers impactes (prevus)
- `docs/PLANS.md`
- `src/sim.rs`
- `src/ui_hud.rs`
- `src/modes.rs`
- `src/rendu.rs`

## Risques
- Surcharge visuelle si les details graphiques masquent la lecture gameplay.
- Regressions de performance si le rendu des gros blocs n'est pas culle correctement.
- Regressions de compatibilite layout si les footprints ne sont pas normalises au chargement.

## Strategie de test
- Tests unitaires de logique pure pour footprint/connectivite.
- Verification manuelle du rendu et de l'ergonomie build.
- Validation complete:
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo run` smoke.

# ExecPlan - Couverture unitaire transverse (modules restants)

Date: 2026-02-23
Portee: completer la couverture unitaire sur les modules encore peu/pas testes et verrouiller les invariants de logique pure.

## Objectifs observables
- Ajouter des tests unitaires deterministes sur `edition`, `historique`, `ui_pawns`, `four_texture`, `render_safety`.
- Couvrir prioritairement les helpers de logique pure (mapping, bornes, etiquettes, determinisme, clamping).
- Garder la separation simulation/rendu (pas de test dependant d'un contexte GPU ou d'un framerate).

## Invariants
- Tests 100% deterministes (seed explicite, pas d'aléatoire implicite).
- Aucun fallback silencieux dans les assertions: chaque echec doit exprimer un contrat metier clair.
- Pas d'ajout de dependance runtime.

## Milestones
1. Auditer les fonctions non testees par module.
2. Ajouter les tests unitaires locaux (`#[cfg(test)]`) pour les chemins logiques purs.
3. Corriger les warnings existants bloquant `clippy -D warnings`.
4. Lancer la validation complete: `fmt`, `clippy`, `test`, `run` smoke.

## Fichiers impactes
- `docs/PLANS.md`
- `src/edition.rs`
- `src/historique.rs`
- `src/ui_pawns.rs`
- `src/four_texture.rs`
- `src/render_safety.rs`
- `src/rendu.rs`

## Risques
- Fonctions UI/rendu non testables hors contexte macroquad (GPU/window state).
- Echecs preexistants dans la suite pouvant masquer la qualite des nouveaux tests.

## Strategie de test
- Tests unitaires de logique pure par module.
- Validation outillage:
  - `cargo fmt`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo run` smoke.
