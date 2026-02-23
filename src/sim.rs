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
const RACK_NIVEAU_COUNT: usize = 6;
const SAC_CAPACITY_UNITS: u32 = 14;
const SACS_PAR_BOX: u32 = 21;

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
            starting_cash: 500_000.0,
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
    pub washed: u32,
    pub sliced: u32,
    pub dehydrated: u32,
    pub flakes: u32,
    pub sacs_bleus_total: u32,
    pub sacs_rouges_total: u32,
    pub boxes_bleues_total: u32,
    pub sold_total: u32,
    pub produced_wip_total: u32,
    pub produced_finished_total: u32,
    blue_bag_fill: u32,
    red_bag_fill: u32,
    lavage_busy: bool,
    coupe_busy: bool,
    four_busy: bool,
    floc_busy: bool,
    sortex_busy: bool,
    lavage_progress_s: f64,
    coupe_progress_s: f64,
    four_progress_s: f64,
    floc_progress_s: f64,
    sortex_progress_s: f64,
    descente_bleue_beacon_s: f64,
    descente_rouge_beacon_s: f64,
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
            washed: 0,
            sliced: 0,
            dehydrated: 0,
            flakes: 0,
            sacs_bleus_total: 0,
            sacs_rouges_total: 0,
            boxes_bleues_total: 0,
            sold_total: 0,
            produced_wip_total: 0,
            produced_finished_total: 0,
            blue_bag_fill: 0,
            red_bag_fill: 0,
            lavage_busy: false,
            coupe_busy: false,
            four_busy: false,
            floc_busy: false,
            sortex_busy: false,
            lavage_progress_s: 0.0,
            coupe_progress_s: 0.0,
            four_progress_s: 0.0,
            floc_progress_s: 0.0,
            sortex_progress_s: 0.0,
            descente_bleue_beacon_s: 0.0,
            descente_rouge_beacon_s: 0.0,
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
    InputHopper,
    Conveyor,
    FluidityTank,
    Cutter,
    DistributorBelt,
    DryerOven,
    OvenExitConveyor,
    Flaker,
    SuctionPipe,
    Sortex,
    BlueBagChute,
    RedBagChute,
    Storage,
    MachineA,
    MachineB,
    Buffer,
    Seller,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum BlockOrientation {
    East,
    South,
    West,
    North,
}

impl BlockOrientation {
    pub fn label(self) -> &'static str {
        match self {
            Self::East => "Est",
            Self::South => "Sud",
            Self::West => "Ouest",
            Self::North => "Nord",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
            Self::North => Self::East,
        }
    }

    pub fn is_vertical(self) -> bool {
        matches!(self, Self::North | Self::South)
    }
}

const PLAYER_BUYABLE_BLOCKS: [BlockKind; 14] = [
    BlockKind::InputHopper,
    BlockKind::Conveyor,
    BlockKind::FluidityTank,
    BlockKind::Cutter,
    BlockKind::DistributorBelt,
    BlockKind::DryerOven,
    BlockKind::OvenExitConveyor,
    BlockKind::Flaker,
    BlockKind::SuctionPipe,
    BlockKind::Sortex,
    BlockKind::BlueBagChute,
    BlockKind::RedBagChute,
    BlockKind::Buffer,
    BlockKind::Seller,
];

const MODERN_LINE_REQUIRED_KINDS: [BlockKind; 12] = [
    BlockKind::InputHopper,
    BlockKind::Conveyor,
    BlockKind::FluidityTank,
    BlockKind::Cutter,
    BlockKind::DistributorBelt,
    BlockKind::DryerOven,
    BlockKind::OvenExitConveyor,
    BlockKind::Flaker,
    BlockKind::SuctionPipe,
    BlockKind::Sortex,
    BlockKind::BlueBagChute,
    BlockKind::RedBagChute,
    ];

const MODERN_LINE_GUIDE_ORDER: [BlockKind; 10] = [
    BlockKind::InputHopper,
    BlockKind::FluidityTank,
    BlockKind::Cutter,
    BlockKind::DistributorBelt,
    BlockKind::DryerOven,
    BlockKind::OvenExitConveyor,
    BlockKind::Flaker,
    BlockKind::Sortex,
    BlockKind::BlueBagChute,
    BlockKind::RedBagChute,
];

impl BlockKind {
    fn capex(self) -> f64 {
        match self {
            Self::InputHopper => 4_800.0,
            Self::Conveyor => 220.0,
            Self::FluidityTank => 7_500.0,
            Self::Cutter => 9_600.0,
            Self::DistributorBelt => 6_200.0,
            Self::DryerOven => 24_500.0,
            Self::OvenExitConveyor => 2_200.0,
            Self::Flaker => 11_800.0,
            Self::SuctionPipe => 340.0,
            Self::Sortex => 16_400.0,
            Self::BlueBagChute => 4_700.0,
            Self::RedBagChute => 4_200.0,
            Self::Storage => 700.0,
            Self::MachineA => 1800.0,
            Self::MachineB => 2200.0,
            Self::Buffer => 1450.0,
            Self::Seller => 1750.0,
        }
    }

    pub fn capex_eur(self) -> f64 {
        self.capex()
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::InputHopper => "entree_ligne_tremie",
            Self::Conveyor => "convoyeur",
            Self::FluidityTank => "bac_fluidite",
            Self::Cutter => "coupeuse",
            Self::DistributorBelt => "tapis_repartiteur",
            Self::DryerOven => "four_deshydratation",
            Self::OvenExitConveyor => "tapis_sortie_four",
            Self::Flaker => "floconneuse",
            Self::SuctionPipe => "tuyau_aspiration",
            Self::Sortex => "sortex",
            Self::BlueBagChute => "descente_sac_bleu",
            Self::RedBagChute => "descente_sac_rouge",
            Self::Storage => "stock_interne",
            Self::MachineA => "machine_a_interne",
            Self::MachineB => "machine_b_interne",
            Self::Buffer => "rack_palettes",
            Self::Seller => "bureau_vente",
        }
    }

    pub fn buyable_label(self) -> &'static str {
        match self {
            Self::InputHopper => "Entree ligne",
            Self::Conveyor => "Convoyeur",
            Self::FluidityTank => "Bac fluidite",
            Self::Cutter => "Coupeuse",
            Self::DistributorBelt => "Tapis repartiteur",
            Self::DryerOven => "Four deshydratation",
            Self::OvenExitConveyor => "Tapis sortie four",
            Self::Flaker => "Floconneuse",
            Self::SuctionPipe => "Tuyau aspiration",
            Self::Sortex => "Sortex",
            Self::BlueBagChute => "Descente sac bleu",
            Self::RedBagChute => "Descente sac rouge",
            Self::Buffer => "Rack palettes",
            Self::Seller => "Bureau de vente",
            Self::Storage => "Stock technique",
            Self::MachineA => "Machine A (legacy)",
            Self::MachineB => "Machine B (legacy)",
        }
    }

    pub fn is_player_buyable(self) -> bool {
        PLAYER_BUYABLE_BLOCKS.contains(&self)
    }

    fn next_player_buyable(self) -> Self {
        let idx = PLAYER_BUYABLE_BLOCKS
            .iter()
            .position(|kind| *kind == self)
            .unwrap_or(0);
        PLAYER_BUYABLE_BLOCKS[(idx + 1) % PLAYER_BUYABLE_BLOCKS.len()]
    }

    pub fn base_footprint(self) -> (i32, i32) {
        match self {
            Self::InputHopper => (3, 8),
            Self::Conveyor => (1, 1),
            Self::FluidityTank => (5, 5),
            Self::Cutter => (3, 3),
            Self::DistributorBelt => (7, 1),
            Self::DryerOven => (10, 20),
            Self::OvenExitConveyor => (7, 1),
            Self::Flaker => (3, 3),
            Self::SuctionPipe => (1, 1),
            Self::Sortex => (4, 4),
            Self::BlueBagChute => (2, 3),
            Self::RedBagChute => (2, 3),
            Self::Storage | Self::MachineA | Self::MachineB | Self::Buffer | Self::Seller => (1, 1),
        }
    }

    pub fn footprint_for_orientation(self, orientation: BlockOrientation) -> (i32, i32) {
        let base = self.base_footprint();
        if orientation.is_vertical() {
            (base.1, base.0)
        } else {
            base
        }
    }

    pub fn is_modern_line_component(self) -> bool {
        MODERN_LINE_REQUIRED_KINDS.contains(&self)
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
    pub orientation: BlockOrientation,
    pub inventory: BlockInventory,
    pub machine: Option<MachineState>,
    pub rack_palettes: [bool; RACK_NIVEAU_COUNT],
}

impl Default for BlockInstance {
    fn default() -> Self {
        Self {
            id: 1,
            kind: BlockKind::Storage,
            origin_tile: (2, 2),
            footprint: (1, 1),
            orientation: BlockOrientation::East,
            inventory: BlockInventory::default(),
            machine: None,
            rack_palettes: [false; RACK_NIVEAU_COUNT],
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
            Self::Receiving => "stockage",
            Self::Processing => "cassage",
            Self::Shipping => "dehy_finition",
            Self::Support => "vente",
        }
    }

    pub fn capex_par_tuile_eur(self) -> f64 {
        match self {
            Self::Neutral => 0.0,
            Self::Receiving => 12.0,
            Self::Processing => 19.0,
            Self::Shipping => 21.0,
            Self::Support => 17.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum BuildFloorKind {
    Standard,
    Metal,
    Bois,
    Mousse,
    Sable,
}

impl BuildFloorKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "sol standard",
            Self::Metal => "sol metal",
            Self::Bois => "sol bois",
            Self::Mousse => "sol mousse",
            Self::Sable => "sol sable",
        }
    }

    pub fn capex_par_tuile_eur(self) -> f64 {
        match self {
            Self::Standard => 6.0,
            Self::Metal => 14.0,
            Self::Bois => 11.0,
            Self::Mousse => 9.0,
            Self::Sable => 8.0,
        }
    }

    pub fn to_tile(self) -> crate::Tile {
        match self {
            Self::Standard => crate::Tile::Floor,
            Self::Metal => crate::Tile::FloorMetal,
            Self::Bois => crate::Tile::FloorWood,
            Self::Mousse => crate::Tile::FloorMoss,
            Self::Sable => crate::Tile::FloorSand,
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
    pub footprint: (i32, i32),
    pub orientation: BlockOrientation,
    pub raw_qty: u32,
    pub inventory_summary: String,
    pub rack_levels: [bool; RACK_NIVEAU_COUNT],
}

#[derive(Clone, Debug)]
pub struct BuildBlockPreview {
    pub kind: BlockKind,
    pub tile: (i32, i32),
    pub footprint: (i32, i32),
    pub orientation: BlockOrientation,
    pub can_place: bool,
    pub guidance: String,
    pub connects_to_line: bool,
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
    floor_paint_mode: bool,
    block_brush: BlockKind,
    block_orientation: BlockOrientation,
    zone_brush: ZoneKind,
    floor_brush: BuildFloorKind,
    pending_zone_rect_start: Option<(i32, i32)>,
    pending_move_block: Option<BlockId>,
    sales_manager_assigned: bool,
    sale_office_present: bool,
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
        for block in &mut layout.blocks {
            block.footprint = block.kind.footprint_for_orientation(block.orientation);
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
            floor_paint_mode: false,
            block_brush: BlockKind::Buffer,
            block_orientation: BlockOrientation::East,
            zone_brush: ZoneKind::Processing,
            floor_brush: BuildFloorKind::Standard,
            pending_zone_rect_start: None,
            pending_move_block: None,
            sales_manager_assigned: true,
            sale_office_present: false,
            build_status: String::new(),
        }
    }

    pub fn step(&mut self, real_dt_seconds: f32) {
        let real_dt = real_dt_seconds as f64;
        if !real_dt.is_finite() || real_dt <= 0.0 {
            return;
        }

        self.sale_office_present = self.blocks.iter().any(|block| {
            block.kind == BlockKind::Seller
                && self.zones.get(block.origin_tile) == ZoneKind::Support
        });

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

        if let Some(reason) = self.modern_line_readiness_reason() {
            self.tick_legacy_line(dt_sim, dt_hours);
            if self.modern_line_present() {
                self.build_status = format!("Ligne de production non operationnelle: {reason}");
            }
        } else {
            self.tick_modern_line(dt_sim);
            self.build_status = format!(
                "Ligne complete active | sacs bleus={} sacs rouges={} boxes bleues={}",
                self.line.sacs_bleus_total,
                self.line.sacs_rouges_total,
                self.line.boxes_bleues_total
            );
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
            self.pending_zone_rect_start = None;
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
        self.block_brush = self.block_brush.next_player_buyable();
        self.floor_paint_mode = false;
        self.zone_paint_mode = false;
        self.pending_zone_rect_start = None;
        self.build_status = format!("Brosse blocs : {}", self.block_brush.buyable_label());
    }

    pub fn cycle_zone_brush(&mut self) {
        self.zone_brush = match self.zone_brush {
            ZoneKind::Neutral => ZoneKind::Receiving,
            ZoneKind::Receiving => ZoneKind::Processing,
            ZoneKind::Processing => ZoneKind::Shipping,
            ZoneKind::Shipping => ZoneKind::Support,
            ZoneKind::Support => ZoneKind::Neutral,
        };
        self.floor_paint_mode = false;
        self.zone_paint_mode = true;
        self.pending_zone_rect_start = None;
        self.build_status = format!("Brosse zones : {}", self.zone_brush.label());
    }

    pub fn cycle_floor_brush(&mut self) {
        self.floor_brush = match self.floor_brush {
            BuildFloorKind::Standard => BuildFloorKind::Metal,
            BuildFloorKind::Metal => BuildFloorKind::Bois,
            BuildFloorKind::Bois => BuildFloorKind::Mousse,
            BuildFloorKind::Mousse => BuildFloorKind::Sable,
            BuildFloorKind::Sable => BuildFloorKind::Standard,
        };
        self.floor_paint_mode = true;
        self.zone_paint_mode = false;
        self.pending_zone_rect_start = None;
        self.build_status = format!("Brosse sols : {}", self.floor_brush.label());
    }

    pub fn toggle_zone_paint_mode(&mut self) {
        self.pending_zone_rect_start = None;
        self.zone_paint_mode = !self.zone_paint_mode;
        if self.zone_paint_mode {
            self.floor_paint_mode = false;
        }
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
        self.zone_paint_mode = false;
        self.floor_paint_mode = false;
        self.pending_zone_rect_start = None;
        self.build_status = format!("Brosse blocs : {}", self.block_brush.buyable_label());
    }

    pub fn block_orientation(&self) -> BlockOrientation {
        self.block_orientation
    }

    pub fn set_block_orientation(&mut self, orientation: BlockOrientation) {
        self.block_orientation = orientation;
        self.build_status = format!("Orientation bloc : {}", self.block_orientation.label());
    }

    pub fn zone_brush(&self) -> ZoneKind {
        self.zone_brush
    }

    pub fn set_zone_brush(&mut self, kind: ZoneKind) {
        self.zone_brush = kind;
        self.pending_zone_rect_start = None;
        self.build_status = format!("Brosse zones : {}", self.zone_brush.label());
    }

    pub fn zone_paint_mode_enabled(&self) -> bool {
        self.zone_paint_mode
    }

    pub fn set_zone_paint_mode(&mut self, enabled: bool) {
        self.pending_zone_rect_start = None;
        self.zone_paint_mode = enabled;
        if self.zone_paint_mode {
            self.floor_paint_mode = false;
        }
        self.build_status = if self.zone_paint_mode {
            "Peinture zones : activee".to_string()
        } else {
            "Peinture zones : desactivee".to_string()
        };
    }

    pub fn floor_brush(&self) -> BuildFloorKind {
        self.floor_brush
    }

    pub fn set_floor_brush(&mut self, kind: BuildFloorKind) {
        self.floor_brush = kind;
        self.floor_paint_mode = true;
        self.zone_paint_mode = false;
        self.pending_zone_rect_start = None;
        self.build_status = format!("Brosse sols : {}", self.floor_brush.label());
    }

    pub fn floor_paint_mode_enabled(&self) -> bool {
        self.floor_paint_mode
    }

    pub fn set_floor_paint_mode(&mut self, enabled: bool) {
        self.floor_paint_mode = enabled;
        if self.floor_paint_mode {
            self.zone_paint_mode = false;
            self.pending_zone_rect_start = None;
        }
        self.build_status = if self.floor_paint_mode {
            "Peinture sols : activee".to_string()
        } else {
            "Peinture sols : desactivee".to_string()
        };
    }

    pub fn pending_move_block(&self) -> Option<BlockId> {
        self.pending_move_block
    }

    pub fn clear_pending_move_block(&mut self) {
        self.pending_move_block = None;
        self.build_status = "Deplacement annule".to_string();
    }

    pub fn sales_manager_assigned(&self) -> bool {
        self.sales_manager_assigned
    }

    pub fn toggle_sales_manager_assigned(&mut self) {
        self.sales_manager_assigned = !self.sales_manager_assigned;
        self.build_status = if self.sales_manager_assigned {
            "Responsable des ventes : assigne".to_string()
        } else {
            "Responsable des ventes : non assigne".to_string()
        };
    }

    pub fn sales_operational(&self) -> bool {
        self.sale_office_present && self.sales_manager_assigned
    }

    pub fn sales_block_reason(&self) -> &'static str {
        if !self.sale_office_present {
            "Bureau de vente manquant dans zone vente"
        } else if !self.sales_manager_assigned {
            "Responsable des ventes non assigne"
        } else {
            "Operationnel"
        }
    }

    pub fn descente_bleue_beacon_active(&self) -> bool {
        self.line.descente_bleue_beacon_s > 0.0
    }

    pub fn descente_rouge_beacon_active(&self) -> bool {
        self.line.descente_rouge_beacon_s > 0.0
    }

    pub fn descente_bleue_fill_ratio(&self) -> f32 {
        (self.line.blue_bag_fill as f32 / SAC_CAPACITY_UNITS as f32).clamp(0.0, 1.0)
    }

    pub fn descente_rouge_fill_ratio(&self) -> f32 {
        (self.line.red_bag_fill as f32 / SAC_CAPACITY_UNITS as f32).clamp(0.0, 1.0)
    }

    pub fn block_kind_at_tile(&self, tile: (i32, i32)) -> Option<BlockKind> {
        self.block_at_tile(tile).map(|block| block.kind)
    }

    pub fn rack_niveau_depuis_fourche(fourche_hauteur: f32) -> u8 {
        let t = fourche_hauteur.clamp(0.0, 1.0);
        let idx = (t * (RACK_NIVEAU_COUNT as f32 - 1.0)).round() as i32;
        idx.clamp(0, RACK_NIVEAU_COUNT as i32 - 1) as u8
    }

    pub fn rack_niveau_label(niveau: u8) -> &'static str {
        match niveau {
            0 => "RDC",
            1 => "N1",
            2 => "N2",
            3 => "N3",
            4 => "N4",
            _ => "N5",
        }
    }

    pub fn build_block_preview(
        &self,
        world: &crate::World,
        tile: (i32, i32),
    ) -> Option<BuildBlockPreview> {
        if !self.build_mode
            || self.zone_paint_mode
            || self.floor_paint_mode
            || self.pending_move_block.is_some()
            || !self.block_brush.is_player_buyable()
        {
            return None;
        }
        let orientation = self.block_orientation;
        let footprint = self.block_brush.footprint_for_orientation(orientation);
        let placement = self.can_place_block_at(world, self.block_brush, tile, orientation, None);
        let (can_place, guidance, connects_to_line) = match placement {
            Ok(valid_footprint) => {
                if self.block_brush.is_modern_line_component() {
                    let (message, connects_to_line) = self.modern_line_placement_guidance(
                        self.block_brush,
                        tile,
                        valid_footprint,
                    );
                    (true, message, connects_to_line)
                } else {
                    (true, String::new(), true)
                }
            }
            Err(reason) => (false, reason, false),
        };
        Some(BuildBlockPreview {
            kind: self.block_brush,
            tile,
            footprint,
            orientation,
            can_place,
            guidance,
            connects_to_line,
        })
    }

    fn next_modern_line_step(&self) -> Option<BlockKind> {
        if self.first_block_by_kind(BlockKind::InputHopper).is_none() {
            return Some(BlockKind::InputHopper);
        }
        if !self.kinds_connected_via(
            BlockKind::InputHopper,
            BlockKind::FluidityTank,
            &[BlockKind::Conveyor],
        ) {
            return Some(BlockKind::FluidityTank);
        }
        if !self.kinds_connected_via(
            BlockKind::FluidityTank,
            BlockKind::Cutter,
            &[BlockKind::Conveyor],
        ) {
            return Some(BlockKind::Cutter);
        }
        if !self.kinds_connected_via(
            BlockKind::Cutter,
            BlockKind::DistributorBelt,
            &[BlockKind::Conveyor],
        ) {
            return Some(BlockKind::DistributorBelt);
        }
        if !self.kinds_connected_via(BlockKind::DistributorBelt, BlockKind::DryerOven, &[]) {
            return Some(BlockKind::DryerOven);
        }
        if !self.kinds_connected_via(BlockKind::DryerOven, BlockKind::OvenExitConveyor, &[]) {
            return Some(BlockKind::OvenExitConveyor);
        }
        if !self.kinds_connected_via(BlockKind::OvenExitConveyor, BlockKind::Flaker, &[]) {
            return Some(BlockKind::Flaker);
        }
        if !self.kinds_connected_via(
            BlockKind::Flaker,
            BlockKind::Sortex,
            &[BlockKind::SuctionPipe],
        ) {
            return Some(BlockKind::Sortex);
        }
        if !self.kinds_connected_via(
            BlockKind::Sortex,
            BlockKind::BlueBagChute,
            &[BlockKind::SuctionPipe],
        ) {
            return Some(BlockKind::BlueBagChute);
        }
        if !self.kinds_connected_via(
            BlockKind::Sortex,
            BlockKind::RedBagChute,
            &[BlockKind::SuctionPipe],
        ) {
            return Some(BlockKind::RedBagChute);
        }
        MODERN_LINE_GUIDE_ORDER
            .iter()
            .copied()
            .find(|kind| !self.blocks.iter().any(|block| block.kind == *kind))
    }

    fn modern_line_touching_kinds(
        &self,
        tile: (i32, i32),
        footprint: (i32, i32),
    ) -> Vec<BlockKind> {
        let expanded_origin = (tile.0 - 1, tile.1 - 1);
        let expanded_size = (footprint.0 + 2, footprint.1 + 2);
        let mut touched: Vec<BlockKind> = Vec::new();

        for block in &self.blocks {
            if !block.kind.is_modern_line_component() {
                continue;
            }
            if !Self::tiles_rect_intersect(
                expanded_origin,
                expanded_size,
                block.origin_tile,
                block.footprint,
            ) {
                continue;
            }
            if !touched.contains(&block.kind) {
                touched.push(block.kind);
            }
        }
        touched
    }

    fn modern_line_placement_guidance(
        &self,
        kind: BlockKind,
        tile: (i32, i32),
        footprint: (i32, i32),
    ) -> (String, bool) {
        let touching = self.modern_line_touching_kinds(tile, footprint);
        if !self.modern_line_present() {
            return (
                if kind == BlockKind::InputHopper {
                    "Demarrez la ligne: Entree ligne, puis Bac fluidite puis Coupeuse".to_string()
                } else {
                    "Aucune ligne en cours: commencez par l'Entree ligne".to_string()
                },
                kind == BlockKind::InputHopper,
            );
        }

        if touching.is_empty() {
            if let Some(next_step) = self.next_modern_line_step() {
                return (
                    format!(
                        "Aucun raccord: placez d'abord autour du dernier maillon: {}",
                        next_step.buyable_label()
                    ),
                    false,
                );
            }
            return (
                "Composant decouple: approchez ce bloc d'une piece ligne existante".to_string(),
                false,
            );
        }

        if let Some(next_step) = self.next_modern_line_step() {
            if kind == next_step {
                return (
                    format!("Etape conseillee: {}", next_step.buyable_label()),
                    true,
                );
            }
            if matches!(kind, BlockKind::Conveyor | BlockKind::SuctionPipe) {
                return (
                    format!(
                        "{} de liaison: continuez vers {}",
                        kind.buyable_label(),
                        next_step.buyable_label()
                    ),
                    true,
                );
            }
            return (
                format!(
                    "Etape conseillee: {} (au lieu de {})",
                    next_step.buyable_label(),
                    kind.buyable_label()
                ),
                false,
            );
        }

        if let Some(reason) = self.modern_line_readiness_reason() {
            return (
                format!("Connexion a corriger: {reason}"),
                false,
            );
        }

        ("Ligne complete: ce bloc sert de renfort".to_string(), true)
    }

    pub fn rack_store_palette(&mut self, tile: (i32, i32), niveau: u8) -> Result<(), String> {
        let idx = usize::from(niveau.min((RACK_NIVEAU_COUNT - 1) as u8));
        let Some(block_idx) = self.block_index_at_tile(tile) else {
            return Err("Aucun rack sur la tuile cible".to_string());
        };
        let block = &mut self.blocks[block_idx];
        if block.kind != BlockKind::Buffer {
            return Err("La tuile cible n'est pas un rack palettes".to_string());
        }
        if block.rack_palettes[idx] {
            return Err(format!(
                "Niveau {} deja occupe",
                Self::rack_niveau_label(niveau)
            ));
        }
        block.rack_palettes[idx] = true;
        Ok(())
    }

    pub fn rack_take_palette(&mut self, tile: (i32, i32), niveau: u8) -> Result<(), String> {
        let idx = usize::from(niveau.min((RACK_NIVEAU_COUNT - 1) as u8));
        let Some(block_idx) = self.block_index_at_tile(tile) else {
            return Err("Aucun rack sur la tuile cible".to_string());
        };
        let block = &mut self.blocks[block_idx];
        if block.kind != BlockKind::Buffer {
            return Err("La tuile cible n'est pas un rack palettes".to_string());
        }
        if !block.rack_palettes[idx] {
            return Err(format!("Niveau {} vide", Self::rack_niveau_label(niveau)));
        }
        block.rack_palettes[idx] = false;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_sales_office_present_for_test(&mut self, present: bool) {
        self.sale_office_present = present;
    }

    pub fn select_move_source(&mut self, tile: (i32, i32)) {
        if let Some((block_id, block_kind)) = self.block_at_tile(tile).map(|b| (b.id, b.kind)) {
            self.pending_move_block = Some(block_id);
            let label = if block_kind.is_player_buyable() {
                block_kind.buyable_label()
            } else {
                block_kind.label()
            };
            self.build_status = format!("Source deplacement=#{} {}", block_id, label);
        } else {
            self.build_status = "Source deplacement: aucun bloc ici".to_string();
        }
    }

    pub fn apply_build_click(
        &mut self,
        world: &mut crate::World,
        tile: (i32, i32),
        right_click: bool,
    ) {
        if !self.build_mode
            || tile.0 < 0
            || tile.1 < 0
            || tile.0 >= self.map_w
            || tile.1 >= self.map_h
        {
            return;
        }

        if self.zone_paint_mode {
            self.apply_zone_rect_click(tile, right_click);
            return;
        }

        if self.floor_paint_mode {
            self.apply_floor_click(world, tile, right_click);
            return;
        }

        if right_click {
            if let Some(index) = self.block_index_at_tile(tile) {
                let removed = self.blocks.remove(index);
                self.economy.earn(removed.kind.capex() * 0.6);
                self.build_status =
                    format!("Vendu #{} {}", removed.id, removed.kind.buyable_label());
            } else {
                self.build_status = "Aucun bloc a vendre".to_string();
            }
            return;
        }

        if let Some(move_id) = self.pending_move_block {
            if let Some(idx) = self.block_index_by_id(move_id) {
                let kind = self.blocks[idx].kind;
                let orientation = self.blocks[idx].orientation;
                if self
                    .can_place_block_at(&*world, kind, tile, orientation, Some(move_id))
                    .is_ok()
                {
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

        if !self.block_brush.is_player_buyable() {
            self.build_status = "Bloc non achetable par le joueur".to_string();
            return;
        }
        let placement = self.can_place_block_at(
            &*world,
            self.block_brush,
            tile,
            self.block_orientation,
            None,
        );
        let footprint = match placement {
            Ok(footprint) => footprint,
            Err(reason) => {
                self.build_status = reason;
                return;
            }
        };
        let (placement_guidance, placement_connected) = if self.block_brush.is_modern_line_component() {
            self.modern_line_placement_guidance(self.block_brush, tile, footprint)
        } else {
            (String::new(), true)
        };

        let id = self.next_block_id;
        let capex = self.block_brush.capex();
        if self.economy.cash < capex {
            self.build_status = format!(
                "Tresorerie insuffisante: {} EUR requis",
                format_int_fr(capex.round() as i64)
            );
            return;
        }
        self.next_block_id = self.next_block_id.saturating_add(1);
        self.economy.spend(capex);
        let mut block = self.make_block(id, self.block_brush, tile, self.block_orientation);
        block.footprint = footprint;
        self.blocks.push(block);
        let mut status = format!(
            "Place {} #{} [{} {}x{}]",
            self.block_brush.buyable_label(),
            id,
            self.block_orientation.label(),
            footprint.0,
            footprint.1
        );
        if !placement_guidance.is_empty() {
            if placement_connected {
                status.push_str(" | ");
            } else {
                status.push_str(" | Alerte: ");
            }
            status.push_str(&placement_guidance);
        }
        self.build_status = status;
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
            .map(|block| {
                let inventory_summary = match block.kind {
                    BlockKind::Buffer => {
                        let occ = block
                            .rack_palettes
                            .iter()
                            .filter(|occupied| **occupied)
                            .count();
                        format!("rack palettes: {occ}/{RACK_NIVEAU_COUNT}")
                    }
                    BlockKind::BlueBagChute => format!(
                        "bleu fill={:.0}% sacs={} boxes={}",
                        self.descente_bleue_fill_ratio() * 100.0,
                        self.line.sacs_bleus_total,
                        self.line.boxes_bleues_total
                    ),
                    BlockKind::RedBagChute => format!(
                        "rouge fill={:.0}% sacs={}",
                        self.descente_rouge_fill_ratio() * 100.0,
                        self.line.sacs_rouges_total
                    ),
                    BlockKind::Sortex => format!(
                        "tri flakes={} bleu={} rouge={}",
                        self.line.flakes, self.line.sacs_bleus_total, self.line.sacs_rouges_total
                    ),
                    _ => format!(
                        "mat:{} enc:{} fini:{} rebut:{}",
                        block.inventory.total_of(ItemKind::Raw),
                        block.inventory.total_of(ItemKind::Wip),
                        block.inventory.total_of(ItemKind::Finished),
                        block.inventory.total_of(ItemKind::Scrap)
                    ),
                };
                BlockDebugView {
                    raw_qty: block.inventory.total_of(ItemKind::Raw),
                    id: block.id,
                    kind: block.kind,
                    tile: block.origin_tile,
                    footprint: block.footprint,
                    orientation: block.orientation,
                    inventory_summary,
                    rack_levels: block.rack_palettes,
                }
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
            format!("zone={} (rectangle)", self.zone_brush.label())
        } else if self.floor_paint_mode {
            format!("sol={}", self.floor_brush.label())
        } else {
            format!(
                "bloc={} orient={}",
                self.block_brush.buyable_label(),
                self.block_orientation.label()
            )
        };
        let move_hint = self
            .pending_move_block
            .map(|id| format!(" source_depl=#{}", id))
            .unwrap_or_default();
        let zone_hint = self
            .pending_zone_rect_start
            .map(|(x, y)| format!(" zone_coin1=({}, {})", x, y))
            .unwrap_or_default();
        format!(
            "Construction [{mode}] | {paint}{move_hint}{zone_hint} | F7: activer/desactiver | B: bloc | T: orientation | N: zone | V: zones | K: sols | M: source deplacement | clic: appliquer | clic droit: vendre/reinitialiser | F8: sauvegarder"
        )
    }

    pub fn status_line(&self) -> &str {
        &self.build_status
    }

    fn apply_zone_rect_click(&mut self, tile: (i32, i32), right_click: bool) {
        let zone_target = if right_click {
            ZoneKind::Neutral
        } else {
            self.zone_brush
        };
        let Some(start) = self.pending_zone_rect_start.take() else {
            self.pending_zone_rect_start = Some(tile);
            self.build_status = format!(
                "Zone {}: coin 1 fixe en ({}, {}), clique le coin oppose",
                zone_target.label(),
                tile.0,
                tile.1
            );
            return;
        };

        let min_x = start.0.min(tile.0).max(0);
        let max_x = start.0.max(tile.0).min(self.map_w - 1);
        let min_y = start.1.min(tile.1).max(0);
        let max_y = start.1.max(tile.1).min(self.map_h - 1);

        let mut changed_tiles = 0usize;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                if self.zones.get((x, y)) != zone_target {
                    changed_tiles += 1;
                }
            }
        }

        let total_cost = zone_target.capex_par_tuile_eur() * changed_tiles as f64;
        if total_cost > 0.0 && self.economy.cash < total_cost {
            self.build_status = format!(
                "Tresorerie insuffisante: {} EUR requis pour zone {} ({} tuiles)",
                format_int_fr(total_cost.round() as i64),
                zone_target.label(),
                changed_tiles
            );
            return;
        }
        if total_cost > 0.0 {
            self.economy.spend(total_cost);
        }

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                self.zones.set((x, y), zone_target);
            }
        }

        self.build_status = format!(
            "Zone {} appliquee sur rectangle ({}, {}) -> ({}, {}) [{} tuiles]",
            zone_target.label(),
            min_x,
            min_y,
            max_x,
            max_y,
            changed_tiles
        );
    }

    fn apply_floor_click(&mut self, world: &mut crate::World, tile: (i32, i32), right_click: bool) {
        if tile.0 <= 0 || tile.1 <= 0 || tile.0 >= self.map_w - 1 || tile.1 >= self.map_h - 1 {
            self.build_status = "Pose sol impossible sur la bordure".to_string();
            return;
        }
        if world.is_solid(tile.0, tile.1) {
            self.build_status = "Pose sol impossible: mur present".to_string();
            return;
        }

        let next_tile = if right_click {
            crate::Tile::Floor
        } else {
            self.floor_brush.to_tile()
        };
        if world.get(tile.0, tile.1) == next_tile {
            self.build_status = "Aucun changement de sol".to_string();
            return;
        }

        let capex = if right_click {
            0.0
        } else {
            self.floor_brush.capex_par_tuile_eur()
        };
        if capex > 0.0 && self.economy.cash < capex {
            self.build_status = format!(
                "Tresorerie insuffisante: {} EUR requis",
                format_int_fr(capex.round() as i64)
            );
            return;
        }
        if capex > 0.0 {
            self.economy.spend(capex);
        }

        world.set(tile.0, tile.1, next_tile);
        self.build_status = if right_click {
            format!("Sol reinitialise @ ({}, {})", tile.0, tile.1)
        } else {
            format!(
                "{} pose @ ({}, {})",
                self.floor_brush.label(),
                tile.0,
                tile.1
            )
        };
    }

    fn first_block_by_kind(&self, kind: BlockKind) -> Option<&BlockInstance> {
        self.blocks.iter().find(|block| block.kind == kind)
    }

    fn block_index_by_id(&self, block_id: BlockId) -> Option<usize> {
        self.blocks.iter().position(|block| block.id == block_id)
    }

    fn tile_in_block(tile: (i32, i32), block: &BlockInstance) -> bool {
        let x0 = block.origin_tile.0;
        let y0 = block.origin_tile.1;
        let x1 = x0 + block.footprint.0 - 1;
        let y1 = y0 + block.footprint.1 - 1;
        tile.0 >= x0 && tile.0 <= x1 && tile.1 >= y0 && tile.1 <= y1
    }

    fn tiles_rect_intersect(
        a_origin: (i32, i32),
        a_size: (i32, i32),
        b_origin: (i32, i32),
        b_size: (i32, i32),
    ) -> bool {
        let a_min_x = a_origin.0;
        let a_min_y = a_origin.1;
        let a_max_x = a_origin.0 + a_size.0 - 1;
        let a_max_y = a_origin.1 + a_size.1 - 1;
        let b_min_x = b_origin.0;
        let b_min_y = b_origin.1;
        let b_max_x = b_origin.0 + b_size.0 - 1;
        let b_max_y = b_origin.1 + b_size.1 - 1;
        a_min_x <= b_max_x && a_max_x >= b_min_x && a_min_y <= b_max_y && a_max_y >= b_min_y
    }

    fn can_place_block_at(
        &self,
        world: &crate::World,
        kind: BlockKind,
        origin: (i32, i32),
        orientation: BlockOrientation,
        ignore_block_id: Option<BlockId>,
    ) -> Result<(i32, i32), String> {
        let footprint = kind.footprint_for_orientation(orientation);
        if footprint.0 <= 0 || footprint.1 <= 0 {
            return Err("Footprint bloc invalide".to_string());
        }

        let max_x = origin.0 + footprint.0 - 1;
        let max_y = origin.1 + footprint.1 - 1;
        if origin.0 < 0
            || origin.1 < 0
            || max_x >= self.map_w
            || max_y >= self.map_h
            || !world.in_bounds(origin.0, origin.1)
            || !world.in_bounds(max_x, max_y)
        {
            return Err("Construction impossible: footprint hors carte".to_string());
        }

        for y in origin.1..=max_y {
            for x in origin.0..=max_x {
                if world.is_solid(x, y) {
                    return Err("Construction impossible: footprint sur mur".to_string());
                }
            }
        }

        let overlaps_existing = self.blocks.iter().any(|block| {
            if Some(block.id) == ignore_block_id {
                return false;
            }
            Self::tiles_rect_intersect(origin, footprint, block.origin_tile, block.footprint)
        });
        if overlaps_existing {
            return Err("Construction impossible: destination occupee".to_string());
        }

        Ok(footprint)
    }

    fn block_index_at_tile(&self, tile: (i32, i32)) -> Option<usize> {
        self.blocks
            .iter()
            .position(|block| Self::tile_in_block(tile, block))
    }

    fn block_at_tile(&self, tile: (i32, i32)) -> Option<&BlockInstance> {
        self.block_index_at_tile(tile)
            .and_then(|idx| self.blocks.get(idx))
    }

    fn make_block(
        &self,
        id: BlockId,
        kind: BlockKind,
        tile: (i32, i32),
        orientation: BlockOrientation,
    ) -> BlockInstance {
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
            footprint: kind.footprint_for_orientation(orientation),
            orientation,
            inventory: BlockInventory::default(),
            machine,
            rack_palettes: [false; RACK_NIVEAU_COUNT],
        }
    }

    fn block_footprints_touch(a: &BlockInstance, b: &BlockInstance) -> bool {
        let expanded_origin = (a.origin_tile.0 - 1, a.origin_tile.1 - 1);
        let expanded_size = (a.footprint.0 + 2, a.footprint.1 + 2);
        Self::tiles_rect_intersect(expanded_origin, expanded_size, b.origin_tile, b.footprint)
    }

    fn block_indices_of_kind(&self, kind: BlockKind) -> Vec<usize> {
        self.blocks
            .iter()
            .enumerate()
            .filter_map(|(idx, block)| (block.kind == kind).then_some(idx))
            .collect()
    }

    fn kinds_connected_via(
        &self,
        from_kind: BlockKind,
        to_kind: BlockKind,
        allowed_middle: &[BlockKind],
    ) -> bool {
        let starts = self.block_indices_of_kind(from_kind);
        if starts.is_empty() || self.block_indices_of_kind(to_kind).is_empty() {
            return false;
        }

        let mut visited = vec![false; self.blocks.len()];
        let mut queue = std::collections::VecDeque::new();
        for idx in starts {
            visited[idx] = true;
            queue.push_back(idx);
        }

        while let Some(idx) = queue.pop_front() {
            if self.blocks[idx].kind == to_kind {
                return true;
            }
            for (n, seen) in visited.iter_mut().enumerate() {
                if *seen {
                    continue;
                }
                if !Self::block_footprints_touch(&self.blocks[idx], &self.blocks[n]) {
                    continue;
                }
                let nk = self.blocks[n].kind;
                if nk == to_kind || nk == from_kind || allowed_middle.contains(&nk) {
                    *seen = true;
                    queue.push_back(n);
                }
            }
        }

        false
    }

    fn modern_line_present(&self) -> bool {
        self.blocks
            .iter()
            .any(|block| block.kind.is_modern_line_component())
    }

    fn modern_line_readiness_reason(&self) -> Option<String> {
        for kind in MODERN_LINE_REQUIRED_KINDS {
            if self.first_block_by_kind(kind).is_none() {
                return Some(format!("Bloc manquant: {}", kind.buyable_label()));
            }
        }

        if !self.kinds_connected_via(
            BlockKind::InputHopper,
            BlockKind::FluidityTank,
            &[BlockKind::Conveyor],
        ) {
            return Some(
                "Connexion invalide: Entree ligne -> Bac fluidite (via convoyeur)".to_string(),
            );
        }
        if !self.kinds_connected_via(
            BlockKind::FluidityTank,
            BlockKind::Cutter,
            &[BlockKind::Conveyor],
        ) {
            return Some(
                "Connexion invalide: Bac fluidite -> Coupeuse (via convoyeur)".to_string(),
            );
        }
        if !self.kinds_connected_via(
            BlockKind::Cutter,
            BlockKind::DistributorBelt,
            &[BlockKind::Conveyor],
        ) {
            return Some("Connexion invalide: Coupeuse -> Tapis repartiteur".to_string());
        }
        if !self.kinds_connected_via(BlockKind::DistributorBelt, BlockKind::DryerOven, &[]) {
            return Some(
                "Connexion invalide: Tapis repartiteur -> Four deshydratation".to_string(),
            );
        }
        if !self.kinds_connected_via(BlockKind::DryerOven, BlockKind::OvenExitConveyor, &[]) {
            return Some("Connexion invalide: Four -> Tapis sortie four".to_string());
        }
        if !self.kinds_connected_via(BlockKind::OvenExitConveyor, BlockKind::Flaker, &[]) {
            return Some("Connexion invalide: Tapis sortie four -> Floconneuse".to_string());
        }
        if !self.kinds_connected_via(
            BlockKind::Flaker,
            BlockKind::Sortex,
            &[BlockKind::SuctionPipe],
        ) {
            return Some("Connexion invalide: Floconneuse -> Sortex (via tuyaux)".to_string());
        }
        if !self.kinds_connected_via(
            BlockKind::Sortex,
            BlockKind::BlueBagChute,
            &[BlockKind::SuctionPipe],
        ) {
            return Some("Connexion invalide: Sortex -> Descente sac bleu".to_string());
        }
        if !self.kinds_connected_via(
            BlockKind::Sortex,
            BlockKind::RedBagChute,
            &[BlockKind::SuctionPipe],
        ) {
            return Some("Connexion invalide: Sortex -> Descente sac rouge".to_string());
        }
        None
    }

    fn tick_legacy_line(&mut self, dt_sim: f64, dt_hours: f64) {
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
            if self.sales_operational() {
                let sold = self.line.finished;
                self.line.finished = 0;
                self.line.sold_total = self.line.sold_total.saturating_add(sold);
                self.economy
                    .earn(sold as f64 * self.config.sale_price.max(0.0));
            } else {
                self.build_status = format!(
                    "Vente en attente: {} (stock fini={})",
                    self.sales_block_reason(),
                    self.line.finished
                );
            }
        }

        let _ = dt_hours;
    }

    fn tick_modern_line(&mut self, dt_sim: f64) {
        self.line.descente_bleue_beacon_s = (self.line.descente_bleue_beacon_s - dt_sim).max(0.0);
        self.line.descente_rouge_beacon_s = (self.line.descente_rouge_beacon_s - dt_sim).max(0.0);

        let cycle_lavage = 16.0;
        let cycle_coupe = 11.0;
        let cycle_four = 42.0;
        let cycle_floc = 14.0;
        let cycle_sortex = 9.0;

        if !self.line.lavage_busy && self.line.raw > 0 {
            self.line.raw -= 1;
            self.line.lavage_busy = true;
            self.line.lavage_progress_s = 0.0;
        }
        if self.line.lavage_busy {
            self.line.lavage_progress_s += dt_sim;
            if self.line.lavage_progress_s >= cycle_lavage {
                self.line.lavage_busy = false;
                self.line.lavage_progress_s = 0.0;
                self.line.washed = self.line.washed.saturating_add(1);
                self.line.produced_wip_total = self.line.produced_wip_total.saturating_add(1);
                if let Some(kpi) = self.zone_kpi.get_mut(&ZoneKind::Processing) {
                    kpi.produced_total = kpi.produced_total.saturating_add(1);
                }
            }
        }

        if !self.line.coupe_busy && self.line.washed > 0 {
            self.line.washed -= 1;
            self.line.coupe_busy = true;
            self.line.coupe_progress_s = 0.0;
        }
        if self.line.coupe_busy {
            self.line.coupe_progress_s += dt_sim;
            if self.line.coupe_progress_s >= cycle_coupe {
                self.line.coupe_busy = false;
                self.line.coupe_progress_s = 0.0;
                self.line.sliced = self.line.sliced.saturating_add(1);
                self.line.produced_wip_total = self.line.produced_wip_total.saturating_add(1);
            }
        }

        if !self.line.four_busy && self.line.sliced > 0 {
            self.line.sliced -= 1;
            self.line.four_busy = true;
            self.line.four_progress_s = 0.0;
        }
        if self.line.four_busy {
            self.line.four_progress_s += dt_sim;
            if self.line.four_progress_s >= cycle_four {
                self.line.four_busy = false;
                self.line.four_progress_s = 0.0;
                self.line.dehydrated = self.line.dehydrated.saturating_add(1);
                if let Some(kpi) = self.zone_kpi.get_mut(&ZoneKind::Shipping) {
                    kpi.produced_total = kpi.produced_total.saturating_add(1);
                }
            }
        }

        if !self.line.floc_busy && self.line.dehydrated > 0 {
            self.line.dehydrated -= 1;
            self.line.floc_busy = true;
            self.line.floc_progress_s = 0.0;
        }
        if self.line.floc_busy {
            self.line.floc_progress_s += dt_sim;
            if self.line.floc_progress_s >= cycle_floc {
                self.line.floc_busy = false;
                self.line.floc_progress_s = 0.0;
                self.line.flakes = self.line.flakes.saturating_add(1);
            }
        }

        if !self.line.sortex_busy && self.line.flakes > 0 {
            self.line.flakes -= 1;
            self.line.sortex_busy = true;
            self.line.sortex_progress_s = 0.0;
        }
        if self.line.sortex_busy {
            self.line.sortex_progress_s += dt_sim;
            if self.line.sortex_progress_s >= cycle_sortex {
                self.line.sortex_busy = false;
                self.line.sortex_progress_s = 0.0;
                let split_seed = self.line.sacs_bleus_total
                    + self.line.sacs_rouges_total
                    + self.line.blue_bag_fill
                    + self.line.red_bag_fill;
                let blue_path = !split_seed.is_multiple_of(5);
                if blue_path {
                    self.line.blue_bag_fill = self.line.blue_bag_fill.saturating_add(1);
                    if self.line.blue_bag_fill >= SAC_CAPACITY_UNITS {
                        self.line.blue_bag_fill = 0;
                        self.line.sacs_bleus_total = self.line.sacs_bleus_total.saturating_add(1);
                        self.line.descente_bleue_beacon_s = 7.0;
                    }
                } else {
                    self.line.red_bag_fill = self.line.red_bag_fill.saturating_add(1);
                    if self.line.red_bag_fill >= SAC_CAPACITY_UNITS {
                        self.line.red_bag_fill = 0;
                        self.line.sacs_rouges_total = self.line.sacs_rouges_total.saturating_add(1);
                        self.line.descente_rouge_beacon_s = 7.0;
                    }
                }
            }
        }

        while self.line.sacs_bleus_total
            >= (self.line.boxes_bleues_total + 1).saturating_mul(SACS_PAR_BOX)
        {
            self.line.boxes_bleues_total = self.line.boxes_bleues_total.saturating_add(1);
            self.line.finished = self.line.finished.saturating_add(1);
            self.line.produced_finished_total = self.line.produced_finished_total.saturating_add(1);
        }

        if self.line.finished > 0 {
            if self.sales_operational() {
                let sold = self.line.finished;
                self.line.finished = 0;
                self.line.sold_total = self.line.sold_total.saturating_add(sold);
                self.economy
                    .earn(sold as f64 * self.config.sale_price.max(0.0));
            } else {
                self.build_status = format!(
                    "Vente en attente: {} (boxes bleues={})",
                    self.sales_block_reason(),
                    self.line.finished
                );
            }
        }

        self.line.wip = self
            .line
            .washed
            .saturating_add(self.line.sliced)
            .saturating_add(self.line.dehydrated)
            .saturating_add(self.line.flakes)
            .saturating_add(self.line.lavage_busy as u32)
            .saturating_add(self.line.coupe_busy as u32)
            .saturating_add(self.line.four_busy as u32)
            .saturating_add(self.line.floc_busy as u32)
            .saturating_add(self.line.sortex_busy as u32);
        self.line.machine_a_busy = self.line.lavage_busy || self.line.coupe_busy;
        self.line.machine_b_busy =
            self.line.four_busy || self.line.floc_busy || self.line.sortex_busy;
        self.line.machine_a_progress = if self.line.coupe_busy {
            self.line.coupe_progress_s
        } else {
            self.line.lavage_progress_s
        };
        self.line.machine_b_progress = if self.line.sortex_busy {
            self.line.sortex_progress_s
        } else if self.line.floc_busy {
            self.line.floc_progress_s
        } else {
            self.line.four_progress_s
        };
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
        let (fx0, fx1, fy0, fy1) = crate::utilitaires::starter_factory_bounds(map_w, map_h);
        let width = (fx1 - fx0).max(6);
        let span_y = (fy1 - fy0).max(6);
        let recv_x1 = fx0 + width / 3;
        let proc_x1 = fx0 + (width * 2) / 3;
        let support_wall_y = (fy1 - 6).max(fy0 + 3);
        let support_y0 = (support_wall_y + 1).min(fy1 - 1);

        for y in fy0 + 1..fy1 {
            for x in fx0 + 1..=recv_x1 {
                zones.set((x, y), ZoneKind::Receiving);
            }
            for x in recv_x1 + 1..=proc_x1 {
                zones.set((x, y), ZoneKind::Processing);
            }
            for x in proc_x1 + 1..fx1 {
                zones.set((x, y), ZoneKind::Shipping);
            }
        }

        for y in support_y0..fy1 {
            for x in fx0 + 1..fx1 {
                zones.set((x, y), ZoneKind::Support);
            }
        }

        let flow_y = (fy0 + span_y / 2).clamp(fy0 + 2, fy1 - 2);
        let ship_y_hi = (support_y0 - 1).max(fy0 + 2);
        let ship_y = (fy0 + (span_y * 3) / 4).clamp(fy0 + 2, ship_y_hi);
        let clamp_tile =
            |x: i32, y: i32| (x.clamp(1, map_w - 2).max(1), y.clamp(1, map_h - 2).max(1));
        let storage_x = (fx0 + 2).min(recv_x1);
        let machine_a_x = (recv_x1 + 1).min(proc_x1);
        let machine_b_x = ((machine_a_x + proc_x1) / 2).max(machine_a_x);
        let buffer_x = (proc_x1 + 1).max(machine_b_x).min(fx1 - 2);
        let seller_x = (fx1 - 2).max(buffer_x);
        let storage_tile = clamp_tile(storage_x, flow_y);
        let machine_a_tile = clamp_tile(machine_a_x, flow_y);
        let machine_b_tile = clamp_tile(machine_b_x, flow_y);
        let buffer_tile = clamp_tile(buffer_x, ship_y);
        let seller_tile = clamp_tile(seller_x, support_y0.max(fy0 + 2));
        let agent_tile = clamp_tile((fx0 + fx1) / 2, (fy1 - 2).max(fy0 + 2));

        let blocks = vec![
            BlockInstance {
                id: 1,
                kind: BlockKind::Storage,
                origin_tile: storage_tile,
                footprint: (1, 1),
                orientation: BlockOrientation::East,
                inventory: BlockInventory::default(),
                machine: None,
                rack_palettes: [false; RACK_NIVEAU_COUNT],
            },
            BlockInstance {
                id: 2,
                kind: BlockKind::MachineA,
                origin_tile: machine_a_tile,
                footprint: (1, 1),
                orientation: BlockOrientation::East,
                inventory: BlockInventory::default(),
                machine: Some(MachineState {
                    recipe_id: RecipeId::RawToWip,
                    cycle_s: config.machine_a_cycle_s.max(1.0),
                    ..MachineState::default()
                }),
                rack_palettes: [false; RACK_NIVEAU_COUNT],
            },
            BlockInstance {
                id: 3,
                kind: BlockKind::MachineB,
                origin_tile: machine_b_tile,
                footprint: (1, 1),
                orientation: BlockOrientation::East,
                inventory: BlockInventory::default(),
                machine: Some(MachineState {
                    recipe_id: RecipeId::WipToFinished,
                    cycle_s: config.machine_b_cycle_s.max(1.0),
                    ..MachineState::default()
                }),
                rack_palettes: [false; RACK_NIVEAU_COUNT],
            },
            BlockInstance {
                id: 4,
                kind: BlockKind::Buffer,
                origin_tile: buffer_tile,
                footprint: (1, 1),
                orientation: BlockOrientation::East,
                inventory: BlockInventory::default(),
                machine: None,
                rack_palettes: [false; RACK_NIVEAU_COUNT],
            },
            BlockInstance {
                id: 5,
                kind: BlockKind::Seller,
                origin_tile: seller_tile,
                footprint: (1, 1),
                orientation: BlockOrientation::East,
                inventory: BlockInventory::default(),
                machine: None,
                rack_palettes: [false; RACK_NIVEAU_COUNT],
            },
        ];

        FactoryLayoutAsset {
            schema_version: FACTORY_LAYOUT_SCHEMA_VERSION,
            map_w,
            map_h,
            zones,
            blocks,
            agent_tile,
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

fn format_int_fr(v: i64) -> String {
    let sign = if v < 0 { "-" } else { "" };
    let mut n = v.unsigned_abs();
    let mut parts: Vec<String> = Vec::new();
    while n >= 1000 {
        let chunk = (n % 1000) as u32;
        parts.push(format!("{:03}", chunk));
        n /= 1000;
    }
    parts.push(format!("{}", n));
    parts.reverse();
    format!("{}{}", sign, parts.join(" "))
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
        let mut world = crate::World::new_room(25, 15);
        let cash0 = sim.economy.cash;
        let tile = (6, 11);

        sim.toggle_build_mode();
        sim.apply_build_click(&mut world, tile, false);
        assert!(sim.block_at_tile(tile).is_some());
        assert!(sim.economy.cash < cash0);

        sim.apply_build_click(&mut world, tile, true);
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

    #[test]
    fn default_layout_flow_is_coherent_with_zones() {
        let sim = FactorySim::new(StarterSimConfig::default(), 168, 108);
        let views = sim.block_debug_views();

        let find_tile = |kind: BlockKind| {
            views
                .iter()
                .find(|b| b.kind == kind)
                .map(|b| b.tile)
                .expect("required block should exist")
        };

        let storage = find_tile(BlockKind::Storage);
        let machine_a = find_tile(BlockKind::MachineA);
        let machine_b = find_tile(BlockKind::MachineB);
        let buffer = find_tile(BlockKind::Buffer);
        let seller = find_tile(BlockKind::Seller);

        // Flux global orienté réception -> production -> expédition.
        assert!(storage.0 < machine_a.0);
        assert!(machine_a.0 <= machine_b.0);
        assert!(machine_b.0 <= buffer.0);
        assert!(buffer.0 <= seller.0);

        assert_eq!(sim.zone_kind_at_tile(storage), ZoneKind::Receiving);
        assert_eq!(sim.zone_kind_at_tile(machine_a), ZoneKind::Processing);
        assert_eq!(sim.zone_kind_at_tile(machine_b), ZoneKind::Processing);
        assert_eq!(sim.zone_kind_at_tile(seller), ZoneKind::Support);
    }

    #[test]
    fn zone_paint_rectangle_spends_cash_and_applies_zone() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let mut world = crate::World::new_room(25, 15);
        let cash0 = sim.cash();
        sim.toggle_build_mode();
        sim.set_zone_brush(ZoneKind::Receiving);
        sim.set_zone_paint_mode(true);

        sim.apply_build_click(&mut world, (3, 3), false);
        sim.apply_build_click(&mut world, (5, 4), false);

        assert_eq!(sim.zone_kind_at_tile((3, 3)), ZoneKind::Receiving);
        assert_eq!(sim.zone_kind_at_tile((5, 4)), ZoneKind::Receiving);
        assert!(sim.cash() < cash0);
    }

    #[test]
    fn floor_paint_spends_cash_and_changes_tile() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let mut world = crate::World::new_room(25, 15);
        let tile = (4, 4);
        sim.toggle_build_mode();
        sim.set_floor_brush(BuildFloorKind::Metal);
        let cash0 = sim.cash();

        sim.apply_build_click(&mut world, tile, false);

        assert_eq!(world.get(tile.0, tile.1), crate::Tile::FloorMetal);
        assert!(sim.cash() < cash0);
    }

    #[test]
    fn rack_store_and_take_palette_by_level() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let rack_tile = sim
            .block_debug_views()
            .into_iter()
            .find(|block| block.kind == BlockKind::Buffer)
            .expect("default layout must include one rack")
            .tile;

        assert!(sim.rack_store_palette(rack_tile, 3).is_ok());
        assert!(sim.rack_store_palette(rack_tile, 3).is_err());
        assert!(sim.rack_take_palette(rack_tile, 3).is_ok());
        assert!(sim.rack_take_palette(rack_tile, 3).is_err());
    }

    #[test]
    fn sales_requires_office_and_manager() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        sim.set_sales_office_present_for_test(true);
        assert!(sim.sales_operational());
        sim.toggle_sales_manager_assigned();
        assert!(!sim.sales_operational());
        assert!(sim.sales_block_reason().contains("Responsable"));
    }

    #[test]
    fn dryer_oven_footprint_rotates_with_orientation() {
        let horizontal = BlockKind::DryerOven.footprint_for_orientation(BlockOrientation::East);
        let vertical = BlockKind::DryerOven.footprint_for_orientation(BlockOrientation::South);
        assert_eq!(horizontal, (10, 20));
        assert_eq!(vertical, (20, 10));
    }

    #[test]
    fn build_mode_rejects_overlapping_large_footprints() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 120, 90);
        let mut world = crate::World::new_room(120, 90);
        sim.toggle_build_mode();

        let existing_tiles = sim
            .block_debug_views()
            .into_iter()
            .map(|b| b.tile)
            .collect::<Vec<_>>();
        for tile in existing_tiles {
            sim.apply_build_click(&mut world, tile, true);
        }

        sim.set_block_brush(BlockKind::InputHopper);
        sim.set_block_orientation(BlockOrientation::East);
        sim.apply_build_click(&mut world, (8, 72), false);
        let cash_after_first = sim.cash();
        let count_after_first = sim
            .block_debug_views()
            .into_iter()
            .filter(|b| b.kind == BlockKind::InputHopper)
            .count();

        sim.set_block_brush(BlockKind::FluidityTank);
        sim.apply_build_click(&mut world, (9, 73), false);
        let count_after_second = sim
            .block_debug_views()
            .into_iter()
            .filter(|b| b.kind == BlockKind::FluidityTank)
            .count();

        assert_eq!(count_after_first, 1);
        assert_eq!(count_after_second, 0);
        assert_eq!(sim.cash(), cash_after_first);
        assert!(sim.status_line().contains("destination occupee"));
    }

    #[test]
    fn build_preview_advises_start_of_line_when_no_modern_component() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 120, 90);
        let mut world = crate::World::new_room(120, 90);

        sim.toggle_build_mode();
        for tile in sim.block_debug_views().into_iter().map(|b| b.tile) {
            sim.apply_build_click(&mut world, tile, true);
        }

        sim.set_block_brush(BlockKind::FluidityTank);
        sim.set_block_orientation(BlockOrientation::East);
        let Some(preview) = sim.build_block_preview(&world, (10, 10)) else {
            panic!("preview should exist");
        };

        assert!(!preview.connects_to_line);
        assert!(preview.guidance.contains("Aucune ligne en cours"));
    }

    #[test]
    fn build_preview_guides_modern_line_order() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 120, 90);
        let mut world = crate::World::new_room(120, 90);

        sim.toggle_build_mode();
        for tile in sim.block_debug_views().into_iter().map(|b| b.tile) {
            sim.apply_build_click(&mut world, tile, true);
        }

        sim.toggle_build_mode();
        sim.set_block_brush(BlockKind::InputHopper);
        sim.set_block_orientation(BlockOrientation::East);
        sim.apply_build_click(&mut world, (10, 10), false);

        sim.set_block_brush(BlockKind::FluidityTank);
        sim.set_block_orientation(BlockOrientation::East);
        let Some(preview) = sim.build_block_preview(&world, (13, 10)) else {
            panic!("preview should exist");
        };

        assert!(preview.can_place);
        assert!(preview.connects_to_line);
        assert!(preview.guidance.contains("Etape conseillee"));
    }

    #[test]
    fn modern_line_next_step_is_connectivity_driven() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 120, 90);
        let mut world = crate::World::new_room(120, 90);

        sim.toggle_build_mode();
        for tile in sim.block_debug_views().into_iter().map(|b| b.tile) {
            sim.apply_build_click(&mut world, tile, true);
        }

        assert_eq!(
            sim.next_modern_line_step(),
            Some(BlockKind::InputHopper),
            "premier bloc attendu"
        );

        sim.set_block_brush(BlockKind::InputHopper);
        sim.set_block_orientation(BlockOrientation::East);
        sim.apply_build_click(&mut world, (10, 10), false);

        assert_eq!(
            sim.next_modern_line_step(),
            Some(BlockKind::FluidityTank),
            "apres entree ligne, la prochaine etape est bac fluidite"
        );

        sim.set_block_brush(BlockKind::Conveyor);
        sim.set_block_orientation(BlockOrientation::East);
        sim.apply_build_click(&mut world, (13, 13), false);
        sim.set_block_brush(BlockKind::FluidityTank);
        sim.set_block_orientation(BlockOrientation::East);
        sim.apply_build_click(&mut world, (14, 11), false);

        assert_eq!(
            sim.next_modern_line_step(),
            Some(BlockKind::Cutter),
            "apres raccord flux, la suivante etape doit etre la coupeuse"
        );
    }

    #[test]
    fn modern_line_reports_connectivity_issue_when_chain_is_broken() {
        let cfg = StarterSimConfig {
            starting_cash: 500_000.0,
            raw_delivery_per_hour: 3600.0,
            ..StarterSimConfig::default()
        };
        let mut sim = FactorySim::new(cfg, 120, 90);
        let mut world = crate::World::new_room(120, 90);
        sim.toggle_build_mode();

        let existing_tiles = sim
            .block_debug_views()
            .into_iter()
            .map(|b| b.tile)
            .collect::<Vec<_>>();
        for tile in existing_tiles {
            sim.apply_build_click(&mut world, tile, true);
        }

        let mut place = |kind: BlockKind, tile: (i32, i32), orientation: BlockOrientation| {
            let before = sim
                .block_debug_views()
                .into_iter()
                .filter(|b| b.kind == kind)
                .count();
            sim.set_block_brush(kind);
            sim.set_block_orientation(orientation);
            sim.apply_build_click(&mut world, tile, false);
            let after = sim
                .block_debug_views()
                .into_iter()
                .filter(|b| b.kind == kind)
                .count();
            assert!(
                after > before,
                "placement failed for {:?} at {:?}: {}",
                kind,
                tile,
                sim.status_line()
            );
        };

        place(BlockKind::InputHopper, (10, 20), BlockOrientation::East);
        place(BlockKind::Conveyor, (13, 23), BlockOrientation::East);
        place(BlockKind::FluidityTank, (14, 21), BlockOrientation::East);
        place(BlockKind::Cutter, (70, 50), BlockOrientation::East);
        place(BlockKind::DistributorBelt, (73, 51), BlockOrientation::East);
        place(BlockKind::DryerOven, (80, 42), BlockOrientation::East);
        place(
            BlockKind::OvenExitConveyor,
            (90, 51),
            BlockOrientation::East,
        );
        place(BlockKind::Flaker, (97, 50), BlockOrientation::East);
        place(BlockKind::SuctionPipe, (100, 51), BlockOrientation::East);
        place(BlockKind::Sortex, (101, 49), BlockOrientation::East);
        place(BlockKind::BlueBagChute, (105, 49), BlockOrientation::East);
        place(BlockKind::RedBagChute, (105, 52), BlockOrientation::East);

        sim.step(1.0 / 60.0);

        assert!(sim.status_line().contains("Connexion invalide"));
    }

    #[test]
    fn modern_line_processes_material_when_chain_is_complete() {
        let cfg = StarterSimConfig {
            starting_cash: 500_000.0,
            raw_delivery_per_hour: 3600.0,
            ..StarterSimConfig::default()
        };
        let mut sim = FactorySim::new(cfg, 120, 90);
        let mut world = crate::World::new_room(120, 90);
        sim.toggle_build_mode();

        let mut place = |kind: BlockKind, tile: (i32, i32), orientation: BlockOrientation| {
            sim.set_block_brush(kind);
            sim.set_block_orientation(orientation);
            sim.apply_build_click(&mut world, tile, false);
        };

        place(BlockKind::InputHopper, (10, 20), BlockOrientation::East);
        place(BlockKind::Conveyor, (13, 23), BlockOrientation::East);
        place(BlockKind::FluidityTank, (14, 21), BlockOrientation::East);
        place(BlockKind::Conveyor, (19, 23), BlockOrientation::East);
        place(BlockKind::Cutter, (20, 22), BlockOrientation::East);
        place(BlockKind::DistributorBelt, (23, 23), BlockOrientation::East);
        place(BlockKind::DryerOven, (30, 14), BlockOrientation::East);
        place(
            BlockKind::OvenExitConveyor,
            (40, 23),
            BlockOrientation::East,
        );
        place(BlockKind::Flaker, (47, 22), BlockOrientation::East);
        place(BlockKind::SuctionPipe, (50, 23), BlockOrientation::East);
        place(BlockKind::SuctionPipe, (51, 23), BlockOrientation::East);
        place(BlockKind::Sortex, (52, 21), BlockOrientation::East);
        place(BlockKind::BlueBagChute, (56, 21), BlockOrientation::East);
        place(BlockKind::RedBagChute, (56, 24), BlockOrientation::East);

        for _ in 0..1000 {
            sim.step(1.0 / 60.0);
        }

        assert!(!sim.status_line().contains("non operationnelle"));
        assert!(sim.line.produced_wip_total > 0);
        assert!(sim.line.sacs_bleus_total + sim.line.sacs_rouges_total > 0);
    }
}

