Tu as déjà un socle “RimWorld-like” solide (grille + collisions + A* + click-to-move + éditeur + save RON). La prochaine étape logique, pour vraiment “avancer” vers Rxchixs, c’est de brancher le noyau simulation aquarium : un temps de simulation explicite + une boucle économique + un mini-framework data-driven qui produit déjà des KPI → argent. C’est exactement ce que demandent la vision (l’usine tourne seule, timers type soufflerie ~8h, argent vérité du système)

Vision-générale-du-jeu

, le cadre Zones/Blocs

environnement-du-jeu

, et le système Jobs/Priorités/Réservations

Jobs-–-Système-&-Priorités

.

Je te donne donc un plan en “marches” (chaque marche = un résultat visible), et je te donne aussi le code pour la marche 1 (copier-coller, sans refactor violent) ✅🙂

Étape 1 — Noyau “aquarium” minimal : Horloge de simulation + Économie + mini-ligne de prod data-driven

Objectif concret à l’écran :

Le jeu continue comme avant.

En debug (F1), tu vois apparaître :

l’heure de simulation (D0 08:32…),

le cash qui évolue,

des compteurs (raw/wip/sold),

et tu peux équilibrer via un fichier RON (sans recompiler).

Pourquoi c’est la “bonne” prochaine étape :

Tes docs imposent un système de temps (heures/jours) pour des timers longs (soufflerie 8h, etc.)

Vision-générale-du-jeu

.

L’argent doit être “la vérité”, lisible en continu

Vision-générale-du-jeu

.

Et ça prépare exactement le couplage futur : Barres/compétences → perf/risque → KPI → argent

Caractéristiques-et-compétence

.

A) Crée un nouveau fichier src/sim.rs (fichier complet)

Copie-colle ceci tel quel :

use ron::{
de::from_str as ron_from_str,
ser::{to_string_pretty as ron_to_string_pretty, PrettyConfig},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SimClock {
sim_seconds: f64,
}

impl SimClock {
pub fn new() -> Self {
Self { sim_seconds: 0.0 }
}

    pub fn advance(&mut self, dt_seconds: f64) {
        if dt_seconds.is_finite() && dt_seconds > 0.0 {
            self.sim_seconds += dt_seconds;
        }
    }

    pub fn seconds(&self) -> f64 {
        self.sim_seconds
    }

    pub fn hours(&self) -> f64 {
        self.sim_seconds / 3600.0
    }

    pub fn day_index(&self) -> u64 {
        (self.sim_seconds / 86_400.0).floor() as u64
    }

    pub fn hour_of_day(&self) -> f64 {
        (self.hours() % 24.0 + 24.0) % 24.0
    }

    pub fn minute_of_hour(&self) -> u32 {
        ((self.sim_seconds / 60.0).floor() as u64 % 60) as u32
    }

    pub fn format_hhmm(&self) -> String {
        let h = self.hour_of_day().floor() as u32;
        let m = self.minute_of_hour();
        format!("{:02}:{:02}", h, m)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Economy {
pub cash: f64,
pub revenue_total: f64,
pub cost_total: f64,
}

impl Economy {
pub fn new(starting_cash: f64) -> Self {
Self {
cash: starting_cash,
revenue_total: 0.0,
cost_total: 0.0,
}
}

    pub fn earn(&mut self, amount: f64) {
        if amount.is_finite() && amount > 0.0 {
            self.cash += amount;
            self.revenue_total += amount;
        }
    }

    pub fn spend(&mut self, amount: f64) {
        if amount.is_finite() && amount > 0.0 {
            self.cash -= amount;
            self.cost_total += amount;
        }
    }

    pub fn profit(&self) -> f64 {
        self.revenue_total - self.cost_total
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StarterSimConfig {
pub schema_version: u32,
pub time_scale: f64, // dt_sim = dt_real * time_scale
pub starting_cash: f64,

    pub worker_count: u32,
    pub wage_per_hour: f64,

    pub raw_delivery_per_hour: f64,
    pub machine_a_cycle_s: f64,
    pub machine_b_cycle_s: f64,

    pub sale_price: f64,
}

impl Default for StarterSimConfig {
fn default() -> Self {
Self {
schema_version: 1,
time_scale: 60.0, // 1 seconde réelle = 1 minute de sim (60x)
starting_cash: 25_000.0,

            worker_count: 3,
            wage_per_hour: 18.5,

            raw_delivery_per_hour: 40.0,
            machine_a_cycle_s: 90.0,
            machine_b_cycle_s: 120.0,

            sale_price: 45.0,
        }
    }
}

impl StarterSimConfig {
fn pretty_config() -> PrettyConfig {
PrettyConfig::new()
.depth_limit(4)
.enumerate_arrays(true)
.separate_tuple_members(true)
}

    pub fn load(path: &str) -> Result<Self, String> {
        let raw = fs::read_to_string(path).map_err(|e| format!("read sim config failed: {e}"))?;
        ron_from_str(&raw).map_err(|e| format!("parse sim config failed: {e}"))
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let payload = ron_to_string_pretty(self, Self::pretty_config())
            .map_err(|e| format!("serialize sim config failed: {e}"))?;

        if let Some(parent) = Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("create sim config dir failed: {e}"))?;
            }
        }

        fs::write(path, payload).map_err(|e| format!("write sim config failed: {e}"))
    }

    pub fn load_or_create(path: &str) -> Self {
        match Self::load(path) {
            Ok(cfg) => cfg,
            Err(_) => {
                let cfg = Self::default();
                let _ = cfg.save(path);
                cfg
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct StarterLineState {
pub raw: u32,
pub wip: u32,
pub finished: u32,

    pub sold_total: u32,
    pub produced_wip_total: u32,
    pub produced_finished_total: u32,

    delivery_accum: f64,

    machine_a_busy: bool,
    machine_b_busy: bool,
    machine_a_progress: f64,
    machine_b_progress: f64,
}

impl StarterLineState {
pub fn new() -> Self {
Self {
raw: 0,
wip: 0,
finished: 0,
sold_total: 0,
produced_wip_total: 0,
produced_finished_total: 0,
delivery_accum: 0.0,
machine_a_busy: false,
machine_b_busy: false,
machine_a_progress: 0.0,
machine_b_progress: 0.0,
}
}
}

pub struct FactorySim {
pub clock: SimClock,
pub economy: Economy,
pub config: StarterSimConfig,
pub line: StarterLineState,
}

impl FactorySim {
pub fn new(config: StarterSimConfig) -> Self {
let economy = Economy::new(config.starting_cash);
Self {
clock: SimClock::new(),
economy,
config,
line: StarterLineState::new(),
}
}

    pub fn load_or_default(path: &str) -> Self {
        let cfg = StarterSimConfig::load_or_create(path);
        Self::new(cfg)
    }

    pub fn step(&mut self, real_dt_seconds: f32) {
        let real_dt = real_dt_seconds as f64;
        if !real_dt.is_finite() || real_dt <= 0.0 {
            return;
        }

        // Convert real time -> simulated time
        let dt_sim = real_dt * self.config.time_scale.max(0.0);
        self.clock.advance(dt_sim);

        let dt_hours = dt_sim / 3600.0;

        // 1) Delivery (entrée matière)
        let per_hour = self.config.raw_delivery_per_hour.max(0.0);
        self.line.delivery_accum += per_hour * dt_hours;

        let delivered = self.line.delivery_accum.floor() as u32;
        if delivered > 0 {
            self.line.raw = self.line.raw.saturating_add(delivered);
            self.line.delivery_accum -= delivered as f64;
        }

        // 2) Wages (OPEX)
        let wage = self.config.wage_per_hour.max(0.0);
        let workers = self.config.worker_count as f64;
        self.economy.spend(wage * workers * dt_hours);

        // 3) Machine A: raw -> wip
        let cycle_a = self.config.machine_a_cycle_s.max(0.001);
        if !self.line.machine_a_busy && self.line.raw > 0 {
            self.line.raw -= 1;
            self.line.machine_a_busy = true;
            self.line.machine_a_progress = 0.0;
        }
        if self.line.machine_a_busy {
            self.line.machine_a_progress += dt_sim;
            if self.line.machine_a_progress >= cycle_a {
                self.line.machine_a_busy = false;
                self.line.machine_a_progress = 0.0;
                self.line.wip = self.line.wip.saturating_add(1);
                self.line.produced_wip_total = self.line.produced_wip_total.saturating_add(1);
            }
        }

        // 4) Machine B: wip -> finished
        let cycle_b = self.config.machine_b_cycle_s.max(0.001);
        if !self.line.machine_b_busy && self.line.wip > 0 {
            self.line.wip -= 1;
            self.line.machine_b_busy = true;
            self.line.machine_b_progress = 0.0;
        }
        if self.line.machine_b_busy {
            self.line.machine_b_progress += dt_sim;
            if self.line.machine_b_progress >= cycle_b {
                self.line.machine_b_busy = false;
                self.line.machine_b_progress = 0.0;
                self.line.finished = self.line.finished.saturating_add(1);
                self.line.produced_finished_total =
                    self.line.produced_finished_total.saturating_add(1);
            }
        }

        // 5) Sale (sortie produit => argent)
        if self.line.finished > 0 {
            let sold = self.line.finished;
            self.line.finished = 0;
            self.line.sold_total = self.line.sold_total.saturating_add(sold);
            self.economy.earn(sold as f64 * self.config.sale_price.max(0.0));
        }
    }

    pub fn debug_hud(&self) -> String {
        let day = self.clock.day_index();
        let cash = self.economy.cash;
        let profit = self.economy.profit();

        let machine_a = if self.line.machine_a_busy { "busy" } else { "idle" };
        let machine_b = if self.line.machine_b_busy { "busy" } else { "idle" };

        format!(
            "SIM time=D{} {} (hours={:.2})\nCash={:.2} | Rev={:.2} | Cost={:.2} | Profit={:.2}\nLine raw={} wip={} finished={} sold_total={}\nMachineA={} MachineB={}",
            day,
            self.clock.format_hhmm(),
            self.clock.hours(),
            cash,
            self.economy.revenue_total,
            self.economy.cost_total,
            profit,
            self.line.raw,
            self.line.wip,
            self.line.finished,
            self.line.sold_total,
            machine_a,
            machine_b
        )
    }
}

#[cfg(test)]
mod tests {
use super::*;

    #[test]
    fn sim_clock_formats_hhmm() {
        let mut clock = SimClock::new();
        assert_eq!(clock.format_hhmm(), "00:00");
        clock.advance(60.0 * 60.0 * 25.0); // 25 hours
        assert_eq!(clock.day_index(), 1);
        assert_eq!(clock.format_hhmm(), "01:00");
    }

    #[test]
    fn factory_sim_generates_sales_and_costs() {
        let cfg = StarterSimConfig {
            schema_version: 1,
            time_scale: 60.0,
            starting_cash: 0.0,
            worker_count: 1,
            wage_per_hour: 10.0,
            raw_delivery_per_hour: 3600.0, // 1 raw / sim second
            machine_a_cycle_s: 5.0,
            machine_b_cycle_s: 5.0,
            sale_price: 2.0,
        };

        let mut sim = FactorySim::new(cfg);

        for _ in 0..120 {
            sim.step(1.0 / 60.0);
        }

        assert!(sim.line.sold_total > 0);
        assert!(sim.economy.revenue_total > 0.0);
        assert!(sim.economy.cost_total > 0.0);
    }
}

B) Crée le fichier de config data-driven data/starter_sim.ron

Crée un dossier data/ à la racine du repo.

Crée data/starter_sim.ron avec :

(
schema_version: 1,

    // Temps: dt_sim = dt_real * time_scale
    // 60.0 => 1 seconde réelle = 1 minute de simulation (60x)
    time_scale: 60.0,

    starting_cash: 25000.0,

    // Coûts humains (OPEX)
    worker_count: 3,
    wage_per_hour: 18.5,

    // Entrée matière (unités / heure de simulation)
    raw_delivery_per_hour: 40.0,

    // Deux machines en série (durées en secondes de simulation)
    machine_a_cycle_s: 90.0,
    machine_b_cycle_s: 120.0,

    // Vente: argent / unité finie
    sale_price: 45.0,
)

C) Patch src/main.rs (modifs précises)

En haut du fichier, juste après mod character;, ajoute :

mod sim;

Dans les constantes (là où tu as MAP_FILE_PATH), ajoute :

const SIM_CONFIG_PATH: &str = "data/starter_sim.ron";

Dans struct GameState, ajoute un champ :

    sim: sim::FactorySim,

(Je te conseille de le mettre juste après palette ou après world, peu importe.)

Dans build_game_state_from_map(...), avant de retourner GameState { ... }, crée la sim :

    let sim = sim::FactorySim::load_or_default(SIM_CONFIG_PATH);

Puis dans l’initialisation GameState { ... }, ajoute :

        sim,

Dans run_play_frame, dans la boucle fixed tick :

Actuellement tu as :

    while *accumulator >= FIXED_DT {
        update_player(...);
        update_npc_wanderer(...);
        *accumulator -= FIXED_DT;
    }

Change en :

    while *accumulator >= FIXED_DT {
        state.sim.step(FIXED_DT);

        update_player(&mut state.player, &state.world, state.last_input, FIXED_DT);
        update_npc_wanderer(&mut state.npc, &state.world, &state.player, FIXED_DT);

        *accumulator -= FIXED_DT;
    }

Dans l’overlay debug (le gros format!(...)), ajoute les lignes de sim.

Repère la string qui finit par :
mutation_permille={} visual={}

Ajoute \n{} à la fin :

        let info = format!(
            "... mutation_permille={} visual={}\n{}",
            ...
            state.character_catalog.mutation_permille(),
            player_visual,
            state.sim.debug_hud(),
        );

Petit HUD même quand debug = off (optionnel mais agréable) :

Dans le else de if state.debug { ... } else { ... } (tout en bas de run_play_frame), après le draw_text("Mode Jeu | ..."), ajoute :

        let hud = format!(
            "Sim {} | Cash {:.0}€",
            state.sim.clock.format_hhmm(),
            state.sim.economy.cash
        );
        draw_text(&hud, 12.0, 48.0, 22.0, Color::from_rgba(200, 224, 236, 255));

Validation attendue :

Tu lances cargo run.

Tu vas en jeu.

F1 → tu vois le bloc “SIM … Cash= …”.

Le cash bouge tout seul (aquarium) 💸

Étape 2 — Passer du “compteur de pipeline” à un vrai framework Items/Blocs/Zones

Là, tu quittes le jouet “raw/wip” et tu construis les briques définitives (mais toujours simples).

2.1 Items génériques + lots (traçabilité)
À coder :

ItemKind (enum data-driven via RON plus tard)

LotId (u64) et propagation “lot in → lot out” (base de traçabilité)

Vision-générale-du-jeu

ItemStack { kind, qty, lot_id }

2.2 Blocs physiques (machines, stockage, buffers) sur la grille
À coder :

BlockKind (Storage, Machine, Buffer, Seller…)

BlockInstance { id, kind, origin_tile, footprint, state }

Inventory pour les blocs de stockage

MachineState { recipe_id, progress, wear } (usure viendra ensuite)

environnement-du-jeu

Résultat visible :

Tu poses (hardcodé au début, puis data-driven) 3 blocs sur la map.

Tu vois leurs inventories dans l’overlay debug.

2.3 Zones logiques (pas juste une couleur)
À coder :

ZoneId, ZoneKind (placeholder A/B/C au début)

ZoneLayer: Vec<ZoneId> (1 par tuile)

ZoneRules (accès, fatigue, bonus vitesse, risques…)

environnement-du-jeu

Résultat visible :

Toggle overlay (ex: F6) qui colore les zones.

Une zone peut moduler un taux de prod (pour tester).

Étape 3 — Jobs + Réservations (le cœur RimWorld-like)

Tu implémentes le “framework générique” de tes docs Jobs

Jobs-–-Système-&-Priorités

.

3.1 Job board
À coder :

JobId

JobState = Pending | Claimed | InProgress | Blocked(reason) | Done

JobKind minimal :

Haul { from_block, to_block, item_kind, qty }

OperateMachine { block_id }

3.2 Réservations anti-conflits
À coder :

ReservationKey :

BlockInput(block_id), BlockOutput(block_id)

InventorySlot(block_id, slot)

(plus tard) Tile(x,y) ou “parking slot”

ReservationTable (HashMap key -> reservation)

TTL / libération systématique (zéro deadlock silencieux)

Jobs-–-Système-&-Priorités

Résultat visible :

Overlay debug jobs : nombre, blocked reasons, réservations actives.

Étape 4 — Agents IA (1er vrai employé autonome)

Tu relies enfin la sim au monde (pathfinding + déplacement + exécution de jobs).

À coder :

Agent { pos, speed, role, skills_stub, needs_stub, current_job, decision_debug }

Tick IA :

Filtrage faisabilité (réservations, accès, pathfinding)

Scoring (priorité + urgence + distance + compétence + fatigue/stress)

Jobs-–-Système-&-Priorités

Exécution en sous-étapes (checkpoints) pour interruption/reprise propre

Jobs-–-Système-&-Priorités

Résultat visible :

Un agent va chercher des items, les dépose, opère la machine, etc.

Au-dessus de sa tête : “Haul raw → Machine (score: …)”.

Étape 5 — Boucle économique/KPI “lisible” (pas juste cash)

À coder :

KPI globaux : débit, rebuts, downtime, OTIF (placeholder), coûts, profit

Vision-générale-du-jeu

KPI locaux par zone (cadence cible, tampon recommandé)

environnement-du-jeu

Couplage humain : fatigue/stress → erreurs → rebuts → profit ↓

Caractéristiques-et-compétence

Résultat visible :

Un mini panneau KPI (même en texte au début).

Tu vois “pourquoi l’argent baisse” (wages↑, downtime↑, scrap↑).

Étape 6 — Construction/optimisation + sauvegarde de layout (le vrai gameplay)

À coder :

Build mode : placer/déplacer/vendre blocs (coûts CAPEX, refunds)

environnement-du-jeu

Étendre l’éditeur pour peindre zones + placer blocs (et sauver tout en RON)

Charger au démarrage : “starter factory” complète (layout + zones + blocs + stocks + staff)

Vision-générale-du-jeu

Résultat visible :

Tu modifies le layout et tu vois l’impact direct sur flux/KPI/argent.