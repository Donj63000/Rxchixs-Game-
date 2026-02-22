use ron::{
    de::from_str as ron_from_str,
    ser::{PrettyConfig, to_string_pretty as ron_to_string_pretty},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

const FACTORY_LAYOUT_PATH: &str = "data/starter_factory.ron";
const FACTORY_LAYOUT_SCHEMA_VERSION: u32 = 1;
const RESERVATION_TTL_SECONDS: f64 = 8.0;

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
        format!("{h:02}:{m:02}")
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
    pub time_scale: f64,
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
            time_scale: 60.0,
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
        let raw =
            fs::read_to_string(path).map_err(|e| format!("echec lecture config simu: {e}"))?;
        ron_from_str(&raw).map_err(|e| format!("echec lecture RON config simu: {e}"))
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let payload = ron_to_string_pretty(self, Self::pretty_config())
            .map_err(|e| format!("echec serialisation config simu: {e}"))?;

        if let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .map_err(|e| format!("echec creation dossier config simu: {e}"))?;
        }

        fs::write(path, payload).map_err(|e| format!("echec ecriture config simu: {e}"))
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

pub type BlockId = u32;
pub type JobId = u64;
pub type AgentId = u32;
pub type LotId = u64;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    Raw,
    Wip,
    Finished,
    Scrap,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ItemStack {
    pub kind: ItemKind,
    pub qty: u32,
    pub lot_id: LotId,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct BlockInventory {
    pub stacks: Vec<ItemStack>,
}

impl BlockInventory {
    fn total_of(&self, kind: ItemKind) -> u32 {
        self.stacks
            .iter()
            .filter(|stack| stack.kind == kind)
            .map(|stack| stack.qty)
            .sum()
    }

    fn add_stack(&mut self, stack: ItemStack) {
        if stack.qty == 0 {
            return;
        }
        if let Some(existing) = self
            .stacks
            .iter_mut()
            .find(|it| it.kind == stack.kind && it.lot_id == stack.lot_id)
        {
            existing.qty = existing.qty.saturating_add(stack.qty);
        } else {
            self.stacks.push(stack);
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum BlockKind {
    Storage,
    MachineA,
    MachineB,
    Buffer,
    Seller,
}

impl BlockKind {
    fn capex(self) -> f64 {
        match self {
            Self::Storage => 700.0,
            Self::MachineA => 1800.0,
            Self::MachineB => 2200.0,
            Self::Buffer => 900.0,
            Self::Seller => 1200.0,
        }
    }

    pub fn capex_eur(self) -> f64 {
        self.capex()
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Storage => "stockage",
            Self::MachineA => "machine_a",
            Self::MachineB => "machine_b",
            Self::Buffer => "tampon",
            Self::Seller => "vente",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum RecipeId {
    RawToWip,
    WipToFinished,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct MachineState {
    pub recipe_id: RecipeId,
    pub progress_s: f64,
    pub cycle_s: f64,
    pub wear: f64,
}

impl Default for MachineState {
    fn default() -> Self {
        Self {
            recipe_id: RecipeId::RawToWip,
            progress_s: 0.0,
            cycle_s: 60.0,
            wear: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BlockInstance {
    pub id: BlockId,
    pub kind: BlockKind,
    pub origin_tile: (i32, i32),
    pub footprint: (i32, i32),
    pub inventory: BlockInventory,
    pub machine: Option<MachineState>,
}

impl Default for BlockInstance {
    fn default() -> Self {
        Self {
            id: 1,
            kind: BlockKind::Storage,
            origin_tile: (2, 2),
            footprint: (1, 1),
            inventory: BlockInventory::default(),
            machine: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ZoneKind {
    Neutral,
    Receiving,
    Processing,
    Shipping,
    Support,
}

impl ZoneKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Neutral => "neutre",
            Self::Receiving => "reception",
            Self::Processing => "production",
            Self::Shipping => "expedition",
            Self::Support => "support",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ZoneRules {
    pub speed_multiplier: f64,
    pub fatigue_factor: f64,
    pub risk_factor: f64,
    pub target_per_hour: f64,
}

fn zone_rules(kind: ZoneKind) -> ZoneRules {
    match kind {
        ZoneKind::Neutral => ZoneRules {
            speed_multiplier: 1.0,
            fatigue_factor: 0.8,
            risk_factor: 0.4,
            target_per_hour: 20.0,
        },
        ZoneKind::Receiving => ZoneRules {
            speed_multiplier: 0.95,
            fatigue_factor: 0.6,
            risk_factor: 0.3,
            target_per_hour: 22.0,
        },
        ZoneKind::Processing => ZoneRules {
            speed_multiplier: 1.15,
            fatigue_factor: 1.3,
            risk_factor: 0.9,
            target_per_hour: 30.0,
        },
        ZoneKind::Shipping => ZoneRules {
            speed_multiplier: 1.05,
            fatigue_factor: 0.75,
            risk_factor: 0.5,
            target_per_hour: 26.0,
        },
        ZoneKind::Support => ZoneRules {
            speed_multiplier: 0.85,
            fatigue_factor: 0.4,
            risk_factor: 0.2,
            target_per_hour: 14.0,
        },
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ZoneLayer {
    pub w: i32,
    pub h: i32,
    pub zones: Vec<ZoneKind>,
}

impl ZoneLayer {
    fn new(w: i32, h: i32, fill: ZoneKind) -> Self {
        Self {
            w,
            h,
            zones: vec![fill; (w * h).max(1) as usize],
        }
    }

    fn get(&self, tile: (i32, i32)) -> ZoneKind {
        if tile.0 < 0 || tile.1 < 0 || tile.0 >= self.w || tile.1 >= self.h {
            return ZoneKind::Neutral;
        }
        self.zones[(tile.1 * self.w + tile.0) as usize]
    }

    fn set(&mut self, tile: (i32, i32), zone: ZoneKind) {
        if tile.0 < 0 || tile.1 < 0 || tile.0 >= self.w || tile.1 >= self.h {
            return;
        }
        let idx = (tile.1 * self.w + tile.0) as usize;
        self.zones[idx] = zone;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobKind {
    Haul {
        from_block: BlockId,
        to_block: BlockId,
        item_kind: ItemKind,
        qty: u32,
    },
    OperateMachine {
        block_id: BlockId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobState {
    Pending,
    Claimed,
    InProgress,
    Blocked(String),
    Done,
}

#[derive(Clone, Debug)]
pub struct Job {
    pub id: JobId,
    pub kind: JobKind,
    pub state: JobState,
    pub priority: i32,
    pub score_debug: String,
    pub assigned_agent: Option<AgentId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReservationKey {
    BlockInput(BlockId),
    BlockOutput(BlockId),
    InventorySlot(BlockId, u16),
}

#[derive(Clone, Debug)]
pub struct Reservation {
    pub job_id: JobId,
    pub ttl_s: f64,
}

#[derive(Clone, Debug)]
pub struct SimAgent {
    pub id: AgentId,
    pub tile: (i32, i32),
    pub speed_tiles_per_s: f64,
    pub fatigue: f64,
    pub stress: f64,
    pub current_job: Option<JobId>,
    pub job_progress_s: f64,
    pub decision_debug: String,
}

#[derive(Clone, Debug, Default)]
pub struct FactoryKpi {
    pub throughput_per_hour: f64,
    pub scrap_total: u32,
    pub downtime_minutes: f64,
    pub otif: f64,
}

#[derive(Clone, Debug, Default)]
pub struct ZoneKpi {
    pub produced_total: u32,
    pub target_per_hour: f64,
}

#[derive(Clone, Debug)]
pub struct BlockDebugView {
    pub id: BlockId,
    pub kind: BlockKind,
    pub tile: (i32, i32),
    pub raw_qty: u32,
    pub inventory_summary: String,
}

#[derive(Clone, Debug)]
pub struct AgentDebugView {
    pub world_pos: (f32, f32),
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
struct FactoryLayoutAsset {
    schema_version: u32,
    map_w: i32,
    map_h: i32,
    zones: ZoneLayer,
    blocks: Vec<BlockInstance>,
    agent_tile: (i32, i32),
}

impl Default for FactoryLayoutAsset {
    fn default() -> Self {
        Self {
            schema_version: FACTORY_LAYOUT_SCHEMA_VERSION,
            map_w: 25,
            map_h: 15,
            zones: ZoneLayer::new(25, 15, ZoneKind::Neutral),
            blocks: Vec::new(),
            agent_tile: (4, 10),
        }
    }
}

pub struct FactorySim {
    pub clock: SimClock,
    pub economy: Economy,
    pub config: StarterSimConfig,
    pub line: StarterLineState,
    zones: ZoneLayer,
    blocks: Vec<BlockInstance>,
    jobs: Vec<Job>,
    reservations: HashMap<ReservationKey, Reservation>,
    agent: SimAgent,
    kpi: FactoryKpi,
    zone_kpi: BTreeMap<ZoneKind, ZoneKpi>,
    next_block_id: BlockId,
    next_job_id: JobId,
    map_w: i32,
    map_h: i32,
    show_zone_overlay: bool,
    build_mode: bool,
    zone_paint_mode: bool,
    block_brush: BlockKind,
    zone_brush: ZoneKind,
    pending_move_block: Option<BlockId>,
    build_status: String,
}

impl FactorySim {
    #[allow(dead_code)]
    pub fn new(config: StarterSimConfig, map_w: i32, map_h: i32) -> Self {
        let layout = Self::default_layout(map_w, map_h, &config);
        Self::from_layout(config, layout)
    }

    pub fn load_or_default(path: &str, map_w: i32, map_h: i32) -> Self {
        let cfg = StarterSimConfig::load_or_create(path);
        let layout = Self::load_or_create_layout(map_w, map_h, &cfg);
        Self::from_layout(cfg, layout)
    }

    fn from_layout(config: StarterSimConfig, mut layout: FactoryLayoutAsset) -> Self {
        layout.schema_version = FACTORY_LAYOUT_SCHEMA_VERSION;
        layout.map_w = layout.map_w.max(1);
        layout.map_h = layout.map_h.max(1);
        if layout.blocks.is_empty() {
            layout = Self::default_layout(layout.map_w, layout.map_h, &config);
        }
        if layout.zones.w != layout.map_w
            || layout.zones.h != layout.map_h
            || layout.zones.zones.len() != (layout.map_w * layout.map_h) as usize
        {
            layout.zones = ZoneLayer::new(layout.map_w, layout.map_h, ZoneKind::Neutral);
        }

        let next_block_id = layout
            .blocks
            .iter()
            .map(|block| block.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let mut zone_kpi = BTreeMap::new();
        for zone in [
            ZoneKind::Neutral,
            ZoneKind::Receiving,
            ZoneKind::Processing,
            ZoneKind::Shipping,
            ZoneKind::Support,
        ] {
            zone_kpi.insert(
                zone,
                ZoneKpi {
                    produced_total: 0,
                    target_per_hour: zone_rules(zone).target_per_hour,
                },
            );
        }

        Self {
            clock: SimClock::new(),
            economy: Economy::new(config.starting_cash),
            config,
            line: StarterLineState::new(),
            zones: layout.zones,
            blocks: layout.blocks,
            jobs: Vec::new(),
            reservations: HashMap::new(),
            agent: SimAgent {
                id: 1,
                tile: layout.agent_tile,
                speed_tiles_per_s: 1.8,
                fatigue: 10.0,
                stress: 7.0,
                current_job: None,
                job_progress_s: 0.0,
                decision_debug: "inactif".to_string(),
            },
            kpi: FactoryKpi::default(),
            zone_kpi,
            next_block_id,
            next_job_id: 1,
            map_w: layout.map_w,
            map_h: layout.map_h,
            show_zone_overlay: false,
            build_mode: false,
            zone_paint_mode: false,
            block_brush: BlockKind::Storage,
            zone_brush: ZoneKind::Processing,
            pending_move_block: None,
            build_status: String::new(),
        }
    }

    pub fn step(&mut self, real_dt_seconds: f32) {
        let real_dt = real_dt_seconds as f64;
        if !real_dt.is_finite() || real_dt <= 0.0 {
            return;
        }

        let dt_sim = real_dt * self.config.time_scale.max(0.0);
        self.clock.advance(dt_sim);
        let dt_hours = dt_sim / 3600.0;

        let per_hour = self.config.raw_delivery_per_hour.max(0.0);
        self.line.delivery_accum += per_hour * dt_hours;
        let delivered = self.line.delivery_accum.floor() as u32;
        if delivered > 0 {
            self.line.raw = self.line.raw.saturating_add(delivered);
            self.line.delivery_accum -= delivered as f64;
        }

        let wage = self.config.wage_per_hour.max(0.0);
        let workers = self.config.worker_count as f64;
        self.economy.spend(wage * workers * dt_hours);

        let machine_a_zone_speed = self
            .first_block_by_kind(BlockKind::MachineA)
            .map(|block| zone_rules(self.zones.get(block.origin_tile)).speed_multiplier)
            .unwrap_or(1.0);
        let machine_b_zone_speed = self
            .first_block_by_kind(BlockKind::MachineB)
            .map(|block| zone_rules(self.zones.get(block.origin_tile)).speed_multiplier)
            .unwrap_or(1.0);

        let cycle_a = (self.config.machine_a_cycle_s / machine_a_zone_speed.max(0.1)).max(0.001);
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
                if let Some(kpi) = self.zone_kpi.get_mut(&ZoneKind::Processing) {
                    kpi.produced_total = kpi.produced_total.saturating_add(1);
                }
            }
        }

        let cycle_b = (self.config.machine_b_cycle_s / machine_b_zone_speed.max(0.1)).max(0.001);
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
                if let Some(kpi) = self.zone_kpi.get_mut(&ZoneKind::Shipping) {
                    kpi.produced_total = kpi.produced_total.saturating_add(1);
                }
            }
        }

        if self.line.finished > 0 {
            let sold = self.line.finished;
            self.line.finished = 0;
            self.line.sold_total = self.line.sold_total.saturating_add(sold);
            self.economy
                .earn(sold as f64 * self.config.sale_price.max(0.0));
        }

        self.tick_reservations(dt_sim);
        self.sync_blocks_from_line();
        self.refresh_jobs();
        self.tick_agent(dt_sim);
        self.refresh_kpi(dt_hours);
    }

    pub fn toggle_zone_overlay(&mut self) {
        self.show_zone_overlay = !self.show_zone_overlay;
        self.build_status = if self.show_zone_overlay {
            "Surcouche zones : activee".to_string()
        } else {
            "Surcouche zones : desactivee".to_string()
        };
    }

    // --- Public, stable accessors for UI/debug (no string parsing) ---
    pub fn primary_agent_tile(&self) -> (i32, i32) {
        self.agent.tile
    }

    pub fn primary_agent_fatigue(&self) -> f64 {
        self.agent.fatigue
    }

    pub fn primary_agent_stress(&self) -> f64 {
        self.agent.stress
    }

    pub fn primary_agent_current_job_id(&self) -> Option<JobId> {
        self.agent.current_job
    }

    pub fn job_brief(&self, job_id: JobId) -> Option<String> {
        let job = self.jobs.iter().find(|j| j.id == job_id)?;
        let brief = match job.kind {
            JobKind::Haul {
                from_block,
                to_block,
                item_kind,
                qty,
            } => {
                let item_label = match item_kind {
                    ItemKind::Raw => "Matiere",
                    ItemKind::Wip => "Encours",
                    ItemKind::Finished => "Produit fini",
                    ItemKind::Scrap => "Rebut",
                };
                format!("Transport: {qty} x {item_label} (B{from_block} -> B{to_block})")
            }
            JobKind::OperateMachine { block_id } => {
                format!("Operation machine (B{block_id})")
            }
        };
        Some(brief)
    }

    fn job_kind_label(&self, kind: &JobKind) -> String {
        match *kind {
            JobKind::Haul {
                from_block,
                to_block,
                item_kind,
                qty,
            } => {
                let item_label = match item_kind {
                    ItemKind::Raw => "matiere",
                    ItemKind::Wip => "encours",
                    ItemKind::Finished => "produit fini",
                    ItemKind::Scrap => "rebut",
                };
                format!("transport {qty}x {item_label} (B{from_block}->B{to_block})")
            }
            JobKind::OperateMachine { block_id } => {
                format!("operation machine (B{block_id})")
            }
        }
    }

    pub fn zone_overlay_enabled(&self) -> bool {
        self.show_zone_overlay
    }

    pub fn toggle_build_mode(&mut self) {
        self.build_mode = !self.build_mode;
        if !self.build_mode {
            self.pending_move_block = None;
        }
        self.build_status = if self.build_mode {
            "Mode construction : actif".to_string()
        } else {
            "Mode construction : arret".to_string()
        };
    }

    pub fn build_mode_enabled(&self) -> bool {
        self.build_mode
    }

    pub fn cycle_block_brush(&mut self) {
        self.block_brush = match self.block_brush {
            BlockKind::Storage => BlockKind::MachineA,
            BlockKind::MachineA => BlockKind::MachineB,
            BlockKind::MachineB => BlockKind::Buffer,
            BlockKind::Buffer => BlockKind::Seller,
            BlockKind::Seller => BlockKind::Storage,
        };
        self.build_status = format!("Brosse blocs : {}", self.block_brush.label());
    }

    pub fn cycle_zone_brush(&mut self) {
        self.zone_brush = match self.zone_brush {
            ZoneKind::Neutral => ZoneKind::Receiving,
            ZoneKind::Receiving => ZoneKind::Processing,
            ZoneKind::Processing => ZoneKind::Shipping,
            ZoneKind::Shipping => ZoneKind::Support,
            ZoneKind::Support => ZoneKind::Neutral,
        };
        self.build_status = format!("Brosse zones : {}", self.zone_brush.label());
    }

    pub fn toggle_zone_paint_mode(&mut self) {
        self.zone_paint_mode = !self.zone_paint_mode;
        self.build_status = if self.zone_paint_mode {
            "Peinture zones : activee".to_string()
        } else {
            "Peinture zones : desactivee".to_string()
        };
    }

    pub fn cash(&self) -> f64 {
        self.economy.cash
    }

    pub fn revenue_total(&self) -> f64 {
        self.economy.revenue_total
    }

    pub fn cost_total(&self) -> f64 {
        self.economy.cost_total
    }

    pub fn profit_total(&self) -> f64 {
        self.economy.profit()
    }

    pub fn sold_total(&self) -> u32 {
        self.line.sold_total
    }

    pub fn throughput_per_hour(&self) -> f64 {
        self.kpi.throughput_per_hour
    }

    pub fn otif(&self) -> f64 {
        self.kpi.otif
    }

    pub fn blocks(&self) -> &[BlockInstance] {
        &self.blocks
    }

    pub fn block_brush(&self) -> BlockKind {
        self.block_brush
    }

    pub fn set_block_brush(&mut self, kind: BlockKind) {
        self.block_brush = kind;
        self.build_status = format!("Brosse blocs : {}", self.block_brush.label());
    }

    pub fn zone_brush(&self) -> ZoneKind {
        self.zone_brush
    }

    pub fn set_zone_brush(&mut self, kind: ZoneKind) {
        self.zone_brush = kind;
        self.build_status = format!("Brosse zones : {}", self.zone_brush.label());
    }

    pub fn zone_paint_mode_enabled(&self) -> bool {
        self.zone_paint_mode
    }

    pub fn set_zone_paint_mode(&mut self, enabled: bool) {
        self.zone_paint_mode = enabled;
        self.build_status = if self.zone_paint_mode {
            "Peinture zones : activee".to_string()
        } else {
            "Peinture zones : desactivee".to_string()
        };
    }

    pub fn pending_move_block(&self) -> Option<BlockId> {
        self.pending_move_block
    }

    pub fn clear_pending_move_block(&mut self) {
        self.pending_move_block = None;
        self.build_status = "Deplacement annule".to_string();
    }

    pub fn select_move_source(&mut self, tile: (i32, i32)) {
        if let Some((block_id, block_kind)) = self.block_at_tile(tile).map(|b| (b.id, b.kind)) {
            self.pending_move_block = Some(block_id);
            self.build_status = format!("Source deplacement=#{} {}", block_id, block_kind.label());
        } else {
            self.build_status = "Source deplacement: aucun bloc ici".to_string();
        }
    }

    pub fn apply_build_click(&mut self, tile: (i32, i32), right_click: bool) {
        if !self.build_mode
            || tile.0 < 0
            || tile.1 < 0
            || tile.0 >= self.map_w
            || tile.1 >= self.map_h
        {
            return;
        }

        if self.zone_paint_mode {
            if right_click {
                self.zones.set(tile, ZoneKind::Neutral);
            } else {
                self.zones.set(tile, self.zone_brush);
            }
            self.build_status = format!(
                "Zone {} @ ({}, {})",
                self.zones.get(tile).label(),
                tile.0,
                tile.1
            );
            return;
        }

        if right_click {
            if let Some(index) = self.block_index_at_tile(tile) {
                let removed = self.blocks.remove(index);
                self.economy.earn(removed.kind.capex() * 0.6);
                self.build_status = format!("Vendu #{} {}", removed.id, removed.kind.label());
            } else {
                self.build_status = "Aucun bloc a vendre".to_string();
            }
            return;
        }

        if let Some(move_id) = self.pending_move_block {
            if let Some(idx) = self.block_index_by_id(move_id) {
                if self.block_index_at_tile(tile).is_none() {
                    self.blocks[idx].origin_tile = tile;
                    self.pending_move_block = None;
                    self.build_status = format!("Deplace #{} -> ({}, {})", move_id, tile.0, tile.1);
                } else {
                    self.build_status = "Deplacement impossible: destination occupee".to_string();
                }
            } else {
                self.pending_move_block = None;
                self.build_status = "Deplacement annule: source introuvable".to_string();
            }
            return;
        }

        if self.block_index_at_tile(tile).is_some() {
            self.build_status = "Construction impossible: destination occupee".to_string();
            return;
        }

        let id = self.next_block_id;
        self.next_block_id = self.next_block_id.saturating_add(1);
        self.economy.spend(self.block_brush.capex());
        self.blocks
            .push(self.make_block(id, self.block_brush, tile));
        self.build_status = format!("Place {} #{}", self.block_brush.label(), id);
    }

    pub fn save_layout(&mut self) -> Result<(), String> {
        let layout = FactoryLayoutAsset {
            schema_version: FACTORY_LAYOUT_SCHEMA_VERSION,
            map_w: self.map_w,
            map_h: self.map_h,
            zones: self.zones.clone(),
            blocks: self.blocks.clone(),
            agent_tile: self.agent.tile,
        };
        self.save_layout_asset(&layout)?;
        self.build_status = format!("Layout usine sauvegarde: {FACTORY_LAYOUT_PATH}");
        Ok(())
    }

    pub fn zone_kind_at_tile(&self, tile: (i32, i32)) -> ZoneKind {
        self.zones.get(tile)
    }

    pub fn block_debug_views(&self) -> Vec<BlockDebugView> {
        self.blocks
            .iter()
            .map(|block| BlockDebugView {
                raw_qty: block.inventory.total_of(ItemKind::Raw),
                id: block.id,
                kind: block.kind,
                tile: block.origin_tile,
                inventory_summary: format!(
                    "mat:{} enc:{} fini:{} rebut:{}",
                    block.inventory.total_of(ItemKind::Raw),
                    block.inventory.total_of(ItemKind::Wip),
                    block.inventory.total_of(ItemKind::Finished),
                    block.inventory.total_of(ItemKind::Scrap)
                ),
            })
            .collect()
    }

    pub fn agent_debug_views(&self) -> Vec<AgentDebugView> {
        vec![AgentDebugView {
            world_pos: (
                self.agent.tile.0 as f32 + 0.5,
                self.agent.tile.1 as f32 + 0.5,
            ),
            label: self.agent.decision_debug.clone(),
        }]
    }

    pub fn short_hud(&self) -> String {
        format!(
            "Simulation {} | Tresorerie {:.0} EUR | Ventes {} | Cadence {:.1}/h | Service {:.0}%",
            self.clock.format_hhmm(),
            self.economy.cash,
            self.line.sold_total,
            self.kpi.throughput_per_hour,
            self.kpi.otif * 100.0
        )
    }

    pub fn build_hint_line(&self) -> String {
        let mode = if self.build_mode { "ACTIF" } else { "ARRET" };
        let paint = if self.zone_paint_mode {
            format!("zone={}", self.zone_brush.label())
        } else {
            format!("bloc={}", self.block_brush.label())
        };
        let move_hint = self
            .pending_move_block
            .map(|id| format!(" source_depl=#{}", id))
            .unwrap_or_default();
        format!(
            "Construction [{mode}] | {paint}{move_hint} | F7: activer/desactiver | B: bloc | N: zone | V: peinture | M: source deplacement | clic: appliquer | clic droit: vendre/reinitialiser | F8: sauvegarder"
        )
    }

    pub fn status_line(&self) -> &str {
        &self.build_status
    }

    fn first_block_by_kind(&self, kind: BlockKind) -> Option<&BlockInstance> {
        self.blocks.iter().find(|block| block.kind == kind)
    }

    fn block_index_by_id(&self, block_id: BlockId) -> Option<usize> {
        self.blocks.iter().position(|block| block.id == block_id)
    }

    fn block_index_at_tile(&self, tile: (i32, i32)) -> Option<usize> {
        self.blocks
            .iter()
            .position(|block| block.origin_tile == tile)
    }

    fn block_at_tile(&self, tile: (i32, i32)) -> Option<&BlockInstance> {
        self.block_index_at_tile(tile)
            .and_then(|idx| self.blocks.get(idx))
    }

    fn make_block(&self, id: BlockId, kind: BlockKind, tile: (i32, i32)) -> BlockInstance {
        let machine = match kind {
            BlockKind::MachineA => Some(MachineState {
                recipe_id: RecipeId::RawToWip,
                cycle_s: self.config.machine_a_cycle_s.max(1.0),
                ..MachineState::default()
            }),
            BlockKind::MachineB => Some(MachineState {
                recipe_id: RecipeId::WipToFinished,
                cycle_s: self.config.machine_b_cycle_s.max(1.0),
                ..MachineState::default()
            }),
            _ => None,
        };
        BlockInstance {
            id,
            kind,
            origin_tile: tile,
            footprint: (1, 1),
            inventory: BlockInventory::default(),
            machine,
        }
    }

    fn sync_blocks_from_line(&mut self) {
        if let Some(storage) = self
            .blocks
            .iter_mut()
            .find(|block| block.kind == BlockKind::Storage)
        {
            storage.inventory.stacks.clear();
            if self.line.raw > 0 {
                storage.inventory.add_stack(ItemStack {
                    kind: ItemKind::Raw,
                    qty: self.line.raw,
                    lot_id: 1,
                });
            }
        }
        if let Some(machine_b) = self
            .blocks
            .iter_mut()
            .find(|block| block.kind == BlockKind::MachineB)
        {
            machine_b.inventory.stacks.clear();
            if self.line.wip > 0 {
                machine_b.inventory.add_stack(ItemStack {
                    kind: ItemKind::Wip,
                    qty: self.line.wip,
                    lot_id: 2,
                });
            }
        }
    }

    fn reservation_keys_for_job(&self, kind: &JobKind) -> Vec<ReservationKey> {
        match kind {
            JobKind::Haul {
                from_block,
                to_block,
                ..
            } => vec![
                ReservationKey::BlockOutput(*from_block),
                ReservationKey::BlockInput(*to_block),
                ReservationKey::InventorySlot(*from_block, 0),
            ],
            JobKind::OperateMachine { block_id } => vec![
                ReservationKey::BlockInput(*block_id),
                ReservationKey::BlockOutput(*block_id),
            ],
        }
    }

    fn try_reserve_all(&mut self, keys: Vec<ReservationKey>, job_id: JobId) -> Result<(), ()> {
        for key in &keys {
            if self.reservations.contains_key(key) {
                return Err(());
            }
        }
        for key in keys {
            self.reservations.insert(
                key,
                Reservation {
                    job_id,
                    ttl_s: RESERVATION_TTL_SECONDS,
                },
            );
        }
        Ok(())
    }

    fn touch_reservations(&mut self, job_id: JobId) {
        for reservation in self.reservations.values_mut() {
            if reservation.job_id == job_id {
                reservation.ttl_s = RESERVATION_TTL_SECONDS;
            }
        }
    }

    fn release_reservations(&mut self, job_id: JobId) {
        self.reservations
            .retain(|_, reservation| reservation.job_id != job_id);
    }

    fn tick_reservations(&mut self, dt_sim: f64) {
        self.reservations.retain(|_, reservation| {
            reservation.ttl_s -= dt_sim;
            reservation.ttl_s > 0.0
        });
    }

    fn refresh_jobs(&mut self) {
        self.jobs.retain(|job| !matches!(job.state, JobState::Done));

        let storage_id = self
            .blocks
            .iter()
            .find(|b| b.kind == BlockKind::Storage)
            .map(|b| b.id);
        let machine_a_id = self
            .blocks
            .iter()
            .find(|b| b.kind == BlockKind::MachineA)
            .map(|b| b.id);
        let machine_b_id = self
            .blocks
            .iter()
            .find(|b| b.kind == BlockKind::MachineB)
            .map(|b| b.id);
        let seller_id = self
            .blocks
            .iter()
            .find(|b| b.kind == BlockKind::Seller)
            .map(|b| b.id);

        if let (Some(storage), Some(machine_a)) = (storage_id, machine_a_id)
            && self.line.raw > 0
        {
            self.ensure_job(
                JobKind::Haul {
                    from_block: storage,
                    to_block: machine_a,
                    item_kind: ItemKind::Raw,
                    qty: 1,
                },
                50,
                "alimenter machine A",
            );
            self.ensure_job(
                JobKind::OperateMachine {
                    block_id: machine_a,
                },
                60,
                "operer A",
            );
        }
        if let Some(machine_b) = machine_b_id
            && self.line.wip > 0
        {
            self.ensure_job(
                JobKind::OperateMachine {
                    block_id: machine_b,
                },
                70,
                "operer B",
            );
        }
        if let (Some(machine_b), Some(seller)) = (machine_b_id, seller_id)
            && self.line.produced_finished_total > self.line.sold_total
        {
            self.ensure_job(
                JobKind::Haul {
                    from_block: machine_b,
                    to_block: seller,
                    item_kind: ItemKind::Finished,
                    qty: 1,
                },
                80,
                "expedier produits finis",
            );
        }
    }

    fn ensure_job(&mut self, kind: JobKind, priority: i32, _reason: &str) {
        let exists = self.jobs.iter().any(|job| {
            job.kind == kind
                && matches!(
                    job.state,
                    JobState::Pending | JobState::Claimed | JobState::InProgress
                )
        });
        if exists {
            return;
        }
        self.jobs.push(Job {
            id: self.next_job_id,
            kind,
            state: JobState::Pending,
            priority,
            score_debug: String::new(),
            assigned_agent: None,
        });
        self.next_job_id = self.next_job_id.saturating_add(1);
    }

    fn tick_agent(&mut self, dt_sim: f64) {
        if let Some(job_id) = self.agent.current_job {
            if let Some(job_idx) = self.jobs.iter().position(|job| job.id == job_id) {
                self.jobs[job_idx].state = JobState::InProgress;
                self.agent.job_progress_s += dt_sim;
                self.touch_reservations(job_id);
                let zone = self.zones.get(self.agent.tile);
                let rules = zone_rules(zone);
                self.agent.fatigue =
                    (self.agent.fatigue + dt_sim / 3600.0 * rules.fatigue_factor).clamp(0.0, 100.0);
                self.agent.stress =
                    (self.agent.stress + dt_sim / 3600.0 * rules.risk_factor).clamp(0.0, 100.0);
                let required_time = 4.0 / self.agent.speed_tiles_per_s.max(0.1);
                if self.agent.job_progress_s >= required_time {
                    self.jobs[job_idx].state = JobState::Done;
                    self.agent.current_job = None;
                    self.agent.job_progress_s = 0.0;
                    self.release_reservations(job_id);
                    self.agent.decision_debug = "tache terminee".to_string();
                }
                return;
            }
            self.agent.current_job = None;
            self.agent.job_progress_s = 0.0;
        }

        if let Some(job_idx) = self
            .jobs
            .iter()
            .enumerate()
            .filter(|(_, job)| matches!(job.state, JobState::Pending))
            .max_by_key(|(_, job)| job.priority)
            .map(|(idx, _)| idx)
        {
            let job_id = self.jobs[job_idx].id;
            let keys = self.reservation_keys_for_job(&self.jobs[job_idx].kind);
            if self.try_reserve_all(keys, job_id).is_ok() {
                let agent_id = self.agent.id;
                self.jobs[job_idx].state = JobState::Claimed;
                self.jobs[job_idx].assigned_agent = Some(agent_id);
                self.jobs[job_idx].score_debug = format!(
                    "priorite={} fatigue={:.1} stress={:.1}",
                    self.jobs[job_idx].priority, self.agent.fatigue, self.agent.stress
                );
                self.agent.current_job = Some(job_id);
                self.agent.job_progress_s = 0.0;
                self.agent.decision_debug = format!(
                    "{} score:{}",
                    self.job_kind_label(&self.jobs[job_idx].kind),
                    self.jobs[job_idx].score_debug
                );
            } else {
                self.jobs[job_idx].state = JobState::Blocked("conflit reservation".to_string());
                self.agent.decision_debug = "bloque: conflit reservation".to_string();
            }
        } else {
            self.agent.decision_debug = "inactif(aucune tache en attente)".to_string();
            self.agent.fatigue = (self.agent.fatigue - dt_sim / 3600.0).clamp(0.0, 100.0);
            self.agent.stress = (self.agent.stress - dt_sim / 3600.0 * 0.8).clamp(0.0, 100.0);
        }
    }

    fn refresh_kpi(&mut self, dt_hours: f64) {
        self.kpi.throughput_per_hour = self.line.sold_total as f64 / self.clock.hours().max(0.01);
        self.kpi.downtime_minutes += if self.line.machine_a_busy || self.line.machine_b_busy {
            0.0
        } else {
            dt_hours * 60.0
        };
        self.kpi.otif = if self.jobs.is_empty() {
            1.0
        } else {
            let blocked = self
                .jobs
                .iter()
                .filter(|job| matches!(job.state, JobState::Blocked(_)))
                .count();
            (1.0 - blocked as f64 / self.jobs.len() as f64).clamp(0.0, 1.0)
        };
        let stress_scrap = (self.agent.stress / 100.0 * dt_hours * 4.0).max(0.0) as u32;
        self.kpi.scrap_total = self.kpi.scrap_total.saturating_add(stress_scrap);
    }

    fn default_layout(map_w: i32, map_h: i32, config: &StarterSimConfig) -> FactoryLayoutAsset {
        let mut zones = ZoneLayer::new(map_w, map_h, ZoneKind::Neutral);
        for y in 2..6 {
            for x in 2..8 {
                zones.set((x, y), ZoneKind::Receiving);
            }
        }
        for y in 2..8 {
            for x in 8..14 {
                zones.set((x, y), ZoneKind::Processing);
            }
        }
        for y in 2..8 {
            for x in 14..20 {
                zones.set((x, y), ZoneKind::Shipping);
            }
        }

        let blocks = vec![
            BlockInstance {
                id: 1,
                kind: BlockKind::Storage,
                origin_tile: (3, 3),
                footprint: (1, 1),
                inventory: BlockInventory::default(),
                machine: None,
            },
            BlockInstance {
                id: 2,
                kind: BlockKind::MachineA,
                origin_tile: (8, 4),
                footprint: (1, 1),
                inventory: BlockInventory::default(),
                machine: Some(MachineState {
                    recipe_id: RecipeId::RawToWip,
                    cycle_s: config.machine_a_cycle_s.max(1.0),
                    ..MachineState::default()
                }),
            },
            BlockInstance {
                id: 3,
                kind: BlockKind::MachineB,
                origin_tile: (11, 4),
                footprint: (1, 1),
                inventory: BlockInventory::default(),
                machine: Some(MachineState {
                    recipe_id: RecipeId::WipToFinished,
                    cycle_s: config.machine_b_cycle_s.max(1.0),
                    ..MachineState::default()
                }),
            },
            BlockInstance {
                id: 4,
                kind: BlockKind::Seller,
                origin_tile: (17, 4),
                footprint: (1, 1),
                inventory: BlockInventory::default(),
                machine: None,
            },
        ];

        FactoryLayoutAsset {
            schema_version: FACTORY_LAYOUT_SCHEMA_VERSION,
            map_w,
            map_h,
            zones,
            blocks,
            agent_tile: (4, 10),
        }
    }

    fn load_or_create_layout(
        map_w: i32,
        map_h: i32,
        config: &StarterSimConfig,
    ) -> FactoryLayoutAsset {
        match Self::load_layout_asset() {
            Ok(mut layout) => {
                layout.map_w = map_w;
                layout.map_h = map_h;
                if layout.blocks.is_empty() {
                    layout = Self::default_layout(map_w, map_h, config);
                }
                layout
            }
            Err(_) => {
                let layout = Self::default_layout(map_w, map_h, config);
                let _ = Self::save_layout_static(&layout);
                layout
            }
        }
    }

    fn load_layout_asset() -> Result<FactoryLayoutAsset, String> {
        let raw = fs::read_to_string(FACTORY_LAYOUT_PATH)
            .map_err(|e| format!("echec lecture layout usine: {e}"))?;
        ron_from_str(&raw).map_err(|e| format!("echec lecture RON layout usine: {e}"))
    }

    fn save_layout_asset(&self, layout: &FactoryLayoutAsset) -> Result<(), String> {
        Self::save_layout_static(layout)
    }

    fn save_layout_static(layout: &FactoryLayoutAsset) -> Result<(), String> {
        let payload = ron_to_string_pretty(layout, PrettyConfig::new().depth_limit(4))
            .map_err(|e| format!("echec serialisation layout usine: {e}"))?;
        if let Some(parent) = Path::new(FACTORY_LAYOUT_PATH).parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .map_err(|e| format!("echec creation dossier layout usine: {e}"))?;
        }
        fs::write(FACTORY_LAYOUT_PATH, payload)
            .map_err(|e| format!("echec ecriture layout usine: {e}"))
    }

    pub fn debug_hud(&self) -> String {
        let day = self.clock.day_index();
        let machine_a = if self.line.machine_a_busy {
            "actif"
        } else {
            "inactif"
        };
        let machine_b = if self.line.machine_b_busy {
            "actif"
        } else {
            "inactif"
        };
        let pending_jobs = self
            .jobs
            .iter()
            .filter(|job| matches!(job.state, JobState::Pending))
            .count();
        let blocked_jobs = self
            .jobs
            .iter()
            .filter_map(|job| match &job.state {
                JobState::Blocked(reason) => Some(reason.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" | ");
        let blocked_label = if blocked_jobs.is_empty() {
            "aucun"
        } else {
            blocked_jobs.as_str()
        };
        let zone_summary = self
            .zone_kpi
            .iter()
            .map(|(zone, kpi)| {
                format!(
                    "{}={}/obj{:.0}",
                    zone.label(),
                    kpi.produced_total,
                    kpi.target_per_hour
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "Simulation J{day} {} ({:.2} h)\nFinances: tresorerie={:.2} | revenu={:.2} | cout={:.2} | profit={:.2}\nFlux ligne: matieres={} en-cours={} finis={} ventes_totales={}\nMachines: A={} | B={}\nJobs: en_attente={} | bloques={} | reservations={}\nAgent: tuile=({}, {}) fatigue={:.1} stress={:.1} job_actuel={:?}\nKPI: cadence={:.1}/h rebut={} arret={:.1}m service={:.0}%\nZones KPI: {}\nConstruction: {}\nStatut: {}",
            self.clock.format_hhmm(),
            self.clock.hours(),
            self.economy.cash,
            self.economy.revenue_total,
            self.economy.cost_total,
            self.economy.profit(),
            self.line.raw,
            self.line.wip,
            self.line.finished,
            self.line.sold_total,
            machine_a,
            machine_b,
            pending_jobs,
            blocked_label,
            self.reservations.len(),
            self.agent.tile.0,
            self.agent.tile.1,
            self.agent.fatigue,
            self.agent.stress,
            self.agent.current_job,
            self.kpi.throughput_per_hour,
            self.kpi.scrap_total,
            self.kpi.downtime_minutes,
            self.kpi.otif * 100.0,
            zone_summary,
            self.build_hint_line(),
            self.build_status
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
        clock.advance(60.0 * 60.0 * 25.0);
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
            raw_delivery_per_hour: 3600.0,
            machine_a_cycle_s: 5.0,
            machine_b_cycle_s: 5.0,
            sale_price: 2.0,
        };

        let mut sim = FactorySim::new(cfg, 25, 15);
        for _ in 0..120 {
            sim.step(1.0 / 60.0);
        }

        assert!(sim.line.sold_total > 0);
        assert!(sim.economy.revenue_total > 0.0);
        assert!(sim.economy.cost_total > 0.0);
    }

    #[test]
    fn build_mode_can_place_and_sell_block() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let cash0 = sim.economy.cash;
        let tile = (6, 11);

        sim.toggle_build_mode();
        sim.apply_build_click(tile, false);
        assert!(sim.block_at_tile(tile).is_some());
        assert!(sim.economy.cash < cash0);

        sim.apply_build_click(tile, true);
        assert!(sim.block_at_tile(tile).is_none());
    }

    #[test]
    fn storage_debug_view_exposes_raw_quantity() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        sim.line.raw = 12;
        sim.sync_blocks_from_line();

        let storage = sim
            .block_debug_views()
            .into_iter()
            .find(|block| block.kind == BlockKind::Storage)
            .expect("storage block should exist");

        assert_eq!(storage.raw_qty, 12);
    }
}
