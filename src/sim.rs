use crate::gestion::{
    EmployeeRole, LineOperationalState, PersonnelState, ProductionLineId, ProductionLineState,
    SalesState, SimCommand, StockState,
};
use ron::{
    de::from_str as ron_from_str,
    ser::{PrettyConfig, to_string_pretty as ron_to_string_pretty},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

const FACTORY_LAYOUT_PATH: &str = "data/starter_factory.ron";
const STARTER_SIM_CONFIG_SCHEMA_VERSION: u32 = 1;
const FACTORY_LAYOUT_SCHEMA_VERSION: u32 = 1;
const RESERVATION_TTL_SECONDS: f64 = 8.0;
const ACTION_STATUS_TTL_SIM_SECONDS: f64 = 240.0;
const RACK_NIVEAU_COUNT: usize = 6;
const SAC_CAPACITY_UNITS: u32 = 14;
const SACS_PAR_BOX: u32 = 21;
const MODERN_CYCLE_LAVAGE_S: f64 = 16.0;
const MODERN_CYCLE_COUPE_S: f64 = 11.0;
const MODERN_CYCLE_FOUR_S: f64 = 42.0;
const MODERN_CYCLE_FLOC_S: f64 = 14.0;
const MODERN_CYCLE_SORTEX_S: f64 = 9.0;
const FACTORY_SIM_SAVE_SCHEMA_VERSION: u32 = 1;
const MAIN_PRODUCTION_LINE_ID: ProductionLineId = 1;
const TEMP_CONTRACT_SECONDS: f64 = 2.0 * 3600.0;

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
            schema_version: STARTER_SIM_CONFIG_SCHEMA_VERSION,
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

    #[allow(dead_code)]
    pub fn load(path: &str) -> Result<Self, String> {
        let raw =
            fs::read_to_string(path).map_err(|e| format!("echec lecture config simu: {e}"))?;
        let cfg: Self =
            ron_from_str(&raw).map_err(|e| format!("echec lecture RON config simu: {e}"))?;
        cfg.validate()?;
        Ok(cfg)
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

    #[allow(dead_code)]
    pub fn load_or_create(path: &str) -> Self {
        Self::load_or_create_with_warning(path).0
    }

    fn load_or_create_with_warning(path: &str) -> (Self, Option<String>) {
        let raw = match fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(err) if err.kind() == ErrorKind::NotFound => {
                let cfg = Self::default();
                let warning = cfg.save(path).err().map(|save_err| {
                    format!("config simu par defaut active, ecriture impossible: {save_err}")
                });
                return (cfg, warning);
            }
            Err(err) => {
                return (
                    Self::default(),
                    Some(format!("config simu illisible, defaut non persiste: {err}")),
                );
            }
        };

        match ron_from_str::<Self>(&raw) {
            Ok(cfg) => match cfg.validate() {
                Ok(()) => (cfg, None),
                Err(err) => (
                    Self::default(),
                    Some(format!("config simu invalide, defaut non persiste: {err}")),
                ),
            },
            Err(err) => (
                Self::default(),
                Some(format!(
                    "config simu RON invalide, defaut non persiste: {err}"
                )),
            ),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.schema_version != STARTER_SIM_CONFIG_SCHEMA_VERSION {
            return Err(format!(
                "schema config simu invalide: attendu={} recu={}",
                STARTER_SIM_CONFIG_SCHEMA_VERSION, self.schema_version
            ));
        }
        let finite_non_negative = [
            ("time_scale", self.time_scale),
            ("starting_cash", self.starting_cash),
            ("wage_per_hour", self.wage_per_hour),
            ("raw_delivery_per_hour", self.raw_delivery_per_hour),
            ("sale_price", self.sale_price),
        ];
        for (label, value) in finite_non_negative {
            if !value.is_finite() || value < 0.0 {
                return Err(format!("{label} doit etre fini et >= 0"));
            }
        }
        let positive_cycles = [
            ("machine_a_cycle_s", self.machine_a_cycle_s),
            ("machine_b_cycle_s", self.machine_b_cycle_s),
        ];
        for (label, value) in positive_cycles {
            if !value.is_finite() || value <= 0.0 {
                return Err(format!("{label} doit etre fini et > 0"));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

    pub fn opposite(self) -> Self {
        match self {
            Self::East => Self::West,
            Self::West => Self::East,
            Self::South => Self::North,
            Self::North => Self::South,
        }
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
            Self::InputHopper => (8, 3),
            Self::Conveyor => (1, 1),
            Self::FluidityTank => (5, 5),
            Self::Cutter => (3, 3),
            Self::DistributorBelt => (7, 1),
            Self::DryerOven => (20, 10),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockRenderView {
    pub id: BlockId,
    pub kind: BlockKind,
    pub tile: (i32, i32),
    pub footprint: (i32, i32),
    pub orientation: BlockOrientation,
    pub raw_qty: u32,
    pub rack_levels: [bool; RACK_NIVEAU_COUNT],
}

impl BlockDebugView {
    pub fn render_view(&self) -> BlockRenderView {
        BlockRenderView {
            id: self.id,
            kind: self.kind,
            tile: self.tile,
            footprint: self.footprint,
            orientation: self.orientation,
            raw_qty: self.raw_qty,
            rack_levels: self.rack_levels,
        }
    }
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

#[derive(Clone, Debug)]
struct ModernLineReadinessCache {
    dirty: bool,
    reason: Option<String>,
}

impl Default for ModernLineReadinessCache {
    fn default() -> Self {
        Self {
            dirty: true,
            reason: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct FactoryLayoutAsset {
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

fn checked_layout_tile_count(w: i32, h: i32, label: &str) -> Result<usize, String> {
    if w < 4 || h < 4 {
        return Err(format!("{label}: dimensions trop petites ({w}x{h})"));
    }
    let count = i64::from(w)
        .checked_mul(i64::from(h))
        .ok_or_else(|| format!("{label}: dimensions en overflow ({w}x{h})"))?;
    if count <= 0 || count as usize > crate::MAX_MAP_TILES {
        return Err(format!("{label}: dimensions trop grandes ({w}x{h})"));
    }
    Ok(count as usize)
}

impl FactoryLayoutAsset {
    fn validate(&self) -> Result<(), String> {
        if self.schema_version != FACTORY_LAYOUT_SCHEMA_VERSION {
            return Err(format!(
                "schema layout usine invalide: attendu={} recu={}",
                FACTORY_LAYOUT_SCHEMA_VERSION, self.schema_version
            ));
        }
        let expected_tiles = checked_layout_tile_count(self.map_w, self.map_h, "layout usine")?;
        if self.zones.w != self.map_w || self.zones.h != self.map_h {
            return Err(format!(
                "dimensions zones incoherentes: zones={}x{} layout={}x{}",
                self.zones.w, self.zones.h, self.map_w, self.map_h
            ));
        }
        if self.zones.zones.len() != expected_tiles {
            return Err(format!(
                "nombre de zones incoherent: {} != {}",
                self.zones.zones.len(),
                expected_tiles
            ));
        }
        if self.agent_tile.0 < 0
            || self.agent_tile.1 < 0
            || self.agent_tile.0 >= self.map_w
            || self.agent_tile.1 >= self.map_h
        {
            return Err(format!(
                "agent hors layout: ({}, {})",
                self.agent_tile.0, self.agent_tile.1
            ));
        }

        let mut seen_ids = HashSet::new();
        for (idx, block) in self.blocks.iter().enumerate() {
            if block.id == 0 || !seen_ids.insert(block.id) {
                return Err(format!("id bloc duplique ou invalide: {}", block.id));
            }
            let footprint = block.kind.footprint_for_orientation(block.orientation);
            if footprint.0 <= 0 || footprint.1 <= 0 {
                return Err(format!("empreinte bloc invalide: {}", block.kind.label()));
            }
            if block.origin_tile.0 < 0
                || block.origin_tile.1 < 0
                || block.origin_tile.0 + footprint.0 > self.map_w
                || block.origin_tile.1 + footprint.1 > self.map_h
            {
                return Err(format!(
                    "bloc {} hors layout a ({}, {})",
                    block.kind.label(),
                    block.origin_tile.0,
                    block.origin_tile.1
                ));
            }
            for previous in &self.blocks[..idx] {
                let previous_footprint = previous
                    .kind
                    .footprint_for_orientation(previous.orientation);
                let overlaps = block.origin_tile.0 < previous.origin_tile.0 + previous_footprint.0
                    && block.origin_tile.0 + footprint.0 > previous.origin_tile.0
                    && block.origin_tile.1 < previous.origin_tile.1 + previous_footprint.1
                    && block.origin_tile.1 + footprint.1 > previous.origin_tile.1;
                if overlaps {
                    return Err(format!(
                        "blocs superposes: #{} {} et #{} {}",
                        previous.id,
                        previous.kind.label(),
                        block.id,
                        block.kind.label()
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct FactorySimSaveAsset {
    #[serde(default = "default_factory_sim_save_schema_version")]
    pub schema_version: u32,
    pub clock: SimClock,
    pub economy: Economy,
    pub personnel: PersonnelState,
    pub stock: StockState,
    pub sales: SalesState,
    pub production_lines: Vec<ProductionLineState>,
    pub line: StarterLineState,
    pub layout: FactoryLayoutAsset,
}

fn default_factory_sim_save_schema_version() -> u32 {
    FACTORY_SIM_SAVE_SCHEMA_VERSION
}

pub struct FactorySim {
    pub clock: SimClock,
    pub economy: Economy,
    pub config: StarterSimConfig,
    pub line: StarterLineState,
    personnel: PersonnelState,
    stock: StockState,
    production_lines: Vec<ProductionLineState>,
    sales: SalesState,
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
    sale_office_present: bool,
    build_status: String,
    build_status_ttl_s: f64,
    production_status: String,
    modern_line_cache: ModernLineReadinessCache,
}

impl FactorySim {
    #[allow(dead_code)]
    pub fn new(config: StarterSimConfig, map_w: i32, map_h: i32) -> Self {
        let layout = Self::default_layout(map_w, map_h, &config);
        Self::from_layout(config, layout)
    }

    pub fn load_or_default(path: &str, map_w: i32, map_h: i32) -> Self {
        let (cfg, cfg_warning) = StarterSimConfig::load_or_create_with_warning(path);
        let (layout, layout_warning) =
            Self::load_or_create_layout(FACTORY_LAYOUT_PATH, map_w, map_h, &cfg);
        let mut sim = Self::from_layout(cfg, layout);
        let warnings = [cfg_warning, layout_warning]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        if !warnings.is_empty() {
            sim.set_status_line(format!(
                "Donnees de demarrage degradees: {}",
                warnings.join(" | ")
            ));
        }
        sim
    }

    fn from_layout(config: StarterSimConfig, mut layout: FactoryLayoutAsset) -> Self {
        layout.schema_version = FACTORY_LAYOUT_SCHEMA_VERSION;
        layout.map_w = layout.map_w.max(1);
        layout.map_h = layout.map_h.max(1);
        if checked_layout_tile_count(layout.map_w, layout.map_h, "layout usine").is_err() {
            layout = Self::default_layout(crate::MAP_W, crate::MAP_H, &config);
        }
        if layout.blocks.is_empty() {
            layout = Self::default_layout(layout.map_w, layout.map_h, &config);
        }
        if layout.zones.w != layout.map_w
            || layout.zones.h != layout.map_h
            || checked_layout_tile_count(layout.map_w, layout.map_h, "layout usine")
                .map(|expected| layout.zones.zones.len() != expected)
                .unwrap_or(true)
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

        let mut production_lines = vec![ProductionLineState::main_line()];
        let personnel = PersonnelState::sandbox_start(0.0, MAIN_PRODUCTION_LINE_ID);
        if let Some(lead) = personnel.team_lead_for_line(MAIN_PRODUCTION_LINE_ID) {
            production_lines[0].assigned_lead_id = Some(lead.id);
        }
        let stock = StockState::sandbox_start();
        let mut line = StarterLineState::new();
        line.raw = stock.raw_line_input;

        Self {
            clock: SimClock::new(),
            economy: Economy::new(config.starting_cash),
            config,
            line,
            personnel,
            stock,
            production_lines,
            sales: SalesState::default(),
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
            sale_office_present: false,
            build_status: String::new(),
            build_status_ttl_s: 0.0,
            production_status: "Simulation initialisee".to_string(),
            modern_line_cache: ModernLineReadinessCache::default(),
        }
    }

    fn set_action_status(&mut self, status: impl Into<String>) {
        self.build_status = status.into();
        self.build_status_ttl_s = ACTION_STATUS_TTL_SIM_SECONDS;
    }

    fn tick_action_status(&mut self, dt_sim: f64) {
        if self.build_status.is_empty() || dt_sim <= 0.0 {
            return;
        }
        self.build_status_ttl_s = (self.build_status_ttl_s - dt_sim).max(0.0);
        if self.build_status_ttl_s <= f64::EPSILON {
            self.build_status.clear();
        }
    }

    fn set_production_status(&mut self, status: impl Into<String>) {
        self.production_status = status.into();
    }

    fn mark_modern_line_cache_dirty(&mut self) {
        self.modern_line_cache.dirty = true;
    }

    fn cached_modern_line_readiness_reason(&mut self) -> Option<String> {
        if self.modern_line_cache.dirty {
            self.modern_line_cache.reason = self.modern_line_readiness_reason_uncached();
            self.modern_line_cache.dirty = false;
        }
        self.modern_line_cache.reason.clone()
    }

    fn refresh_production_status(&mut self, modern_readiness_reason: Option<&str>) {
        if self.line.finished > 0 && !self.sales_operational() {
            self.set_production_status(format!(
                "Vente en attente: {} (stock fini={})",
                self.sales_block_reason(),
                self.line.finished
            ));
            return;
        }

        if self.main_line_state().status == LineOperationalState::Bloquee {
            let reason = self.main_line_state().block_reason.clone();
            self.set_production_status(format!("Ligne 1 bloquee: {reason}"));
            return;
        }

        if let Some(reason) = modern_readiness_reason {
            if self.modern_line_present() {
                self.set_production_status(format!(
                    "Ligne de production non operationnelle: {reason}"
                ));
            } else {
                self.set_production_status(format!(
                    "Ligne legacy active | finis={} ventes={}",
                    self.line.finished, self.line.sold_total
                ));
            }
            return;
        }

        self.set_production_status(format!(
            "Ligne complete active | sacs bleus={} sacs rouges={} boxes bleues={}",
            self.line.sacs_bleus_total, self.line.sacs_rouges_total, self.line.boxes_bleues_total
        ));
    }

    fn sale_office_count(&self) -> usize {
        self.blocks
            .iter()
            .filter(|block| {
                block.kind == BlockKind::Seller
                    && self.zones.get(block.origin_tile) == ZoneKind::Support
            })
            .count()
    }

    fn refresh_static_capabilities(&mut self) {
        self.sale_office_present = self.sale_office_count() > 0;
    }

    fn main_line_state(&self) -> &ProductionLineState {
        self.production_lines
            .iter()
            .find(|line| line.id == MAIN_PRODUCTION_LINE_ID)
            .unwrap_or(&self.production_lines[0])
    }

    fn main_line_state_mut(&mut self) -> &mut ProductionLineState {
        let index = self
            .production_lines
            .iter()
            .position(|line| line.id == MAIN_PRODUCTION_LINE_ID)
            .unwrap_or(0);
        &mut self.production_lines[index]
    }

    fn line_has_work_in_progress(&self) -> bool {
        self.line.wip > 0
            || self.line.washed > 0
            || self.line.sliced > 0
            || self.line.dehydrated > 0
            || self.line.flakes > 0
            || self.line.machine_a_busy
            || self.line.machine_b_busy
            || self.line.lavage_busy
            || self.line.coupe_busy
            || self.line.four_busy
            || self.line.floc_busy
            || self.line.sortex_busy
    }

    fn production_staffing_factor(&self) -> f64 {
        self.main_line_state().staffing_factor.max(0.1)
    }

    fn refresh_line_statuses(&mut self, modern_readiness_reason: Option<&str>) -> Option<String> {
        let Some(lead_id) = self
            .personnel
            .team_lead_for_line(MAIN_PRODUCTION_LINE_ID)
            .map(|lead| lead.id)
        else {
            let line = self.main_line_state_mut();
            line.assigned_lead_id = None;
            line.set_blocked("aucun chef d'equipe assigne");
            return Some("aucun chef d'equipe assigne".to_string());
        };

        if self.modern_line_present()
            && let Some(reason) = modern_readiness_reason
        {
            let line = self.main_line_state_mut();
            line.assigned_lead_id = Some(lead_id);
            line.set_blocked(reason.to_string());
            return Some(reason.to_string());
        }

        if self.line.raw == 0 && !self.line_has_work_in_progress() {
            let reason = if self.stock.raw_receiving > 0
                && self.personnel.available_role_count(EmployeeRole::Cariste) == 0
            {
                "stock entree vide, caristes absents"
            } else {
                "stock entree vide"
            };
            let line = self.main_line_state_mut();
            line.assigned_lead_id = Some(lead_id);
            line.set_blocked(reason);
            return Some(reason.to_string());
        }

        let active_temps = self.personnel.active_temps_for_lead(lead_id);
        self.main_line_state_mut().set_active(lead_id, active_temps);
        None
    }

    fn tick_team_leads_and_temps(&mut self, dt_sim: f64) {
        self.personnel.tick_temp_contracts(dt_sim);

        let Some(lead) = self
            .personnel
            .team_lead_for_line(MAIN_PRODUCTION_LINE_ID)
            .cloned()
        else {
            return;
        };

        if !self.stock.has_any_raw_for_line() && !self.line_has_work_in_progress() {
            let _ = self
                .personnel
                .release_finished_temps_without_stock(MAIN_PRODUCTION_LINE_ID);
            return;
        }

        let Some(policy) = lead.temp_policy.clone() else {
            return;
        };
        if !policy.enabled || self.economy.cash < policy.min_cash_reserve {
            return;
        }

        let max_temps = policy.max_temps.min(3) as usize;
        let desired = if self.stock.raw_line_input >= 90 || self.stock.raw_receiving >= 300 {
            max_temps.min(2)
        } else if self.stock.raw_line_input > 0 || self.stock.raw_receiving > 0 {
            max_temps.min(1)
        } else {
            0
        };
        let mut active = self.personnel.active_temps_for_lead(lead.id);
        while active < desired {
            if self
                .personnel
                .hire_temp_for_lead(
                    lead.id,
                    MAIN_PRODUCTION_LINE_ID,
                    self.clock.seconds(),
                    TEMP_CONTRACT_SECONDS,
                )
                .is_err()
            {
                break;
            }
            active += 1;
        }
    }

    fn tick_payroll(&mut self, dt_hours: f64) {
        let payroll = self.personnel.hourly_payroll_eur();
        self.economy.spend(payroll * dt_hours);
    }

    fn sync_line_raw_from_stock(&mut self) {
        self.line.raw = self.stock.raw_line_input;
    }

    fn sync_stock_raw_from_line(&mut self) {
        self.stock.raw_line_input = self.line.raw.min(crate::gestion::RAW_LINE_INPUT_CAPACITY);
    }

    fn tick_sales(&mut self, dt_hours: f64) {
        let admins = self
            .personnel
            .available_role_count(EmployeeRole::AdministrateurVente);
        let offices = self.sale_office_count();
        let (sold, revenue) = self.sales.tick(
            dt_hours,
            &mut self.line.finished,
            admins,
            offices,
            self.config.sale_price,
        );
        if sold > 0 {
            self.line.sold_total = self.line.sold_total.saturating_add(sold);
            self.economy.earn(revenue);
        }
    }

    pub fn step(&mut self, real_dt_seconds: f32) {
        let real_dt = real_dt_seconds as f64;
        if !real_dt.is_finite() || real_dt <= 0.0 {
            return;
        }

        let dt_sim = real_dt * self.config.time_scale.max(0.0);
        self.clock.advance(dt_sim);
        self.tick_action_status(dt_sim);
        let dt_hours = dt_sim / 3600.0;

        self.stock.tick_purchase_orders(dt_sim);
        self.refresh_static_capabilities();
        self.tick_payroll(dt_hours);
        self.tick_team_leads_and_temps(dt_sim);
        let caristes = self.personnel.available_role_count(EmployeeRole::Cariste);
        self.stock.tick_cariste_transfer(dt_hours, caristes);
        self.sync_line_raw_from_stock();

        let modern_readiness_reason = self.cached_modern_line_readiness_reason();
        let production_block_reason =
            self.refresh_line_statuses(modern_readiness_reason.as_deref());
        if production_block_reason.is_none() {
            if modern_readiness_reason.is_some() {
                self.tick_legacy_line(dt_sim, dt_hours);
            } else {
                self.tick_modern_line(dt_sim);
            }
            self.sync_stock_raw_from_line();
        }

        self.tick_sales(dt_hours);
        self.tick_reservations(dt_sim);
        self.sync_blocks_from_line();
        self.refresh_jobs();
        self.tick_agent(dt_sim);
        self.refresh_kpi(dt_hours);
        self.refresh_production_status(modern_readiness_reason.as_deref());
    }

    pub fn toggle_zone_overlay(&mut self) {
        self.show_zone_overlay = !self.show_zone_overlay;
        self.set_status_line(if self.show_zone_overlay {
            "Surcouche zones : activee".to_string()
        } else {
            "Surcouche zones : desactivee".to_string()
        });
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

    fn job_target_label(&self, kind: &JobKind) -> String {
        match *kind {
            JobKind::Haul {
                from_block,
                to_block,
                ..
            } => format!("B{from_block}->B{to_block}"),
            JobKind::OperateMachine { block_id } => format!("B{block_id}"),
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
        self.set_status_line(if self.build_mode {
            "Mode construction : actif".to_string()
        } else {
            "Mode construction : arret".to_string()
        });
    }

    pub fn build_mode_enabled(&self) -> bool {
        self.build_mode
    }

    pub fn cycle_block_brush(&mut self) {
        self.block_brush = self.block_brush.next_player_buyable();
        self.floor_paint_mode = false;
        self.zone_paint_mode = false;
        self.pending_zone_rect_start = None;
        self.set_status_line(format!(
            "Brosse blocs : {}",
            self.block_brush.buyable_label()
        ));
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
        self.set_status_line(format!("Brosse zones : {}", self.zone_brush.label()));
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
        self.set_status_line(format!("Brosse sols : {}", self.floor_brush.label()));
    }

    pub fn toggle_zone_paint_mode(&mut self) {
        self.pending_zone_rect_start = None;
        self.zone_paint_mode = !self.zone_paint_mode;
        if self.zone_paint_mode {
            self.floor_paint_mode = false;
        }
        self.set_status_line(if self.zone_paint_mode {
            "Peinture zones : activee".to_string()
        } else {
            "Peinture zones : desactivee".to_string()
        });
    }

    pub fn cash(&self) -> f64 {
        self.economy.cash
    }

    pub fn personnel(&self) -> &PersonnelState {
        &self.personnel
    }

    pub fn stock(&self) -> &StockState {
        &self.stock
    }

    pub fn sales_state(&self) -> &SalesState {
        &self.sales
    }

    pub fn main_production_line(&self) -> &ProductionLineState {
        self.main_line_state()
    }

    pub fn payroll_per_hour(&self) -> f64 {
        self.personnel.hourly_payroll_eur()
    }

    pub fn sales_capacity_per_hour(&self) -> f64 {
        crate::gestion::vente::SalesState::capacity_units_per_hour(
            self.personnel
                .available_role_count(EmployeeRole::AdministrateurVente),
            self.sale_office_count(),
        )
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
        self.set_status_line(format!(
            "Brosse blocs : {}",
            self.block_brush.buyable_label()
        ));
    }

    pub fn block_orientation(&self) -> BlockOrientation {
        self.block_orientation
    }

    pub fn set_block_orientation(&mut self, orientation: BlockOrientation) {
        self.block_orientation = orientation;
        self.set_status_line(format!(
            "Orientation bloc : {}",
            self.block_orientation.label()
        ));
    }

    pub fn zone_brush(&self) -> ZoneKind {
        self.zone_brush
    }

    pub fn set_zone_brush(&mut self, kind: ZoneKind) {
        self.zone_brush = kind;
        self.pending_zone_rect_start = None;
        self.set_status_line(format!("Brosse zones : {}", self.zone_brush.label()));
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
        self.set_status_line(if self.zone_paint_mode {
            "Peinture zones : activee".to_string()
        } else {
            "Peinture zones : desactivee".to_string()
        });
    }

    pub fn floor_brush(&self) -> BuildFloorKind {
        self.floor_brush
    }

    pub fn set_floor_brush(&mut self, kind: BuildFloorKind) {
        self.floor_brush = kind;
        self.floor_paint_mode = true;
        self.zone_paint_mode = false;
        self.pending_zone_rect_start = None;
        self.set_status_line(format!("Brosse sols : {}", self.floor_brush.label()));
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
        self.set_status_line(if self.floor_paint_mode {
            "Peinture sols : activee".to_string()
        } else {
            "Peinture sols : desactivee".to_string()
        });
    }

    pub fn pending_move_block(&self) -> Option<BlockId> {
        self.pending_move_block
    }

    pub fn clear_pending_move_block(&mut self) {
        self.pending_move_block = None;
        self.set_status_line("Deplacement annule");
    }

    pub fn sales_manager_assigned(&self) -> bool {
        self.personnel
            .available_role_count(EmployeeRole::AdministrateurVente)
            > 0
    }

    pub fn toggle_sales_manager_assigned(&mut self) {
        if let Some(admin_id) = self
            .personnel
            .employees
            .iter()
            .find(|employee| employee.role == EmployeeRole::AdministrateurVente)
            .map(|employee| employee.id)
        {
            match self.apply_command(SimCommand::FireEmployee {
                employee_id: admin_id,
            }) {
                Ok(msg) | Err(msg) => self.set_status_line(msg),
            }
        } else {
            match self.apply_command(SimCommand::HireEmployee {
                role: EmployeeRole::AdministrateurVente,
            }) {
                Ok(msg) | Err(msg) => self.set_status_line(msg),
            }
        }
    }

    pub fn sales_operational(&self) -> bool {
        self.sale_office_present
            && self
                .personnel
                .available_role_count(EmployeeRole::AdministrateurVente)
                > 0
    }

    pub fn sales_block_reason(&self) -> &'static str {
        if !self.sale_office_present {
            "Bureau de vente manquant dans zone vente"
        } else if self
            .personnel
            .available_role_count(EmployeeRole::AdministrateurVente)
            == 0
        {
            "Aucun administrateur de vente"
        } else {
            "Operationnel"
        }
    }

    pub fn apply_command(&mut self, command: SimCommand) -> Result<String, String> {
        match command {
            SimCommand::HireEmployee { role } => {
                let cost = role.hiring_cost_eur();
                if self.economy.cash < cost {
                    return Err(format!(
                        "Tresorerie insuffisante: {:.0} EUR requis pour recruter {}",
                        cost,
                        role.label()
                    ));
                }
                if cost > 0.0 {
                    self.economy.spend(cost);
                }
                let id = self.personnel.hire(role, self.clock.seconds())?;
                if role == EmployeeRole::ChefEquipe
                    && self
                        .personnel
                        .team_lead_for_line(MAIN_PRODUCTION_LINE_ID)
                        .is_none()
                {
                    self.personnel.assign_to_line(id, MAIN_PRODUCTION_LINE_ID)?;
                    self.main_line_state_mut().assigned_lead_id = Some(id);
                }
                let name = self
                    .personnel
                    .employee(id)
                    .map(|employee| employee.name.clone())
                    .unwrap_or_else(|| format!("#{id}"));
                Ok(format!("{} recrute: {}", role.label(), name))
            }
            SimCommand::FireEmployee { employee_id } => {
                let role = self
                    .personnel
                    .employee(employee_id)
                    .map(|employee| employee.role)
                    .ok_or_else(|| format!("employe introuvable: {employee_id}"))?;
                self.personnel.fire(employee_id)?;
                if role == EmployeeRole::ChefEquipe
                    && self.main_line_state().assigned_lead_id == Some(employee_id)
                {
                    let line = self.main_line_state_mut();
                    line.assigned_lead_id = None;
                    line.set_blocked("aucun chef d'equipe assigne");
                }
                Ok(format!("Employe licencie: #{employee_id}"))
            }
            SimCommand::AssignEmployeeToLine {
                employee_id,
                line_id,
            } => {
                if line_id != MAIN_PRODUCTION_LINE_ID {
                    return Err(format!("ligne inconnue: {line_id}"));
                }
                self.personnel.assign_to_line(employee_id, line_id)?;
                self.main_line_state_mut().assigned_lead_id = Some(employee_id);
                Ok(format!("Chef #{employee_id} assigne a la ligne {line_id}"))
            }
            SimCommand::SetLineTempPolicy {
                line_id,
                enabled,
                max_temps,
            } => {
                if line_id != MAIN_PRODUCTION_LINE_ID {
                    return Err(format!("ligne inconnue: {line_id}"));
                }
                let lead_id = self
                    .personnel
                    .team_lead_for_line(line_id)
                    .map(|employee| employee.id)
                    .ok_or_else(|| "aucun chef assigne a la ligne".to_string())?;
                self.personnel
                    .set_temp_policy(lead_id, enabled, max_temps.min(3))?;
                Ok(format!(
                    "Interim ligne {line_id}: {} max {}",
                    if enabled { "ON" } else { "OFF" },
                    max_temps.min(3)
                ))
            }
            SimCommand::BuyRawStock { qty } => {
                let (order_id, cost) = self.stock.place_raw_order(qty, self.economy.cash)?;
                self.economy.spend(cost);
                Ok(format!(
                    "Commande matiere #{order_id}: {qty} unites ({cost:.0} EUR)"
                ))
            }
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

    fn modern_stage_cycle_s(&self, kind: BlockKind, base_cycle_s: f64) -> f64 {
        let speed = self
            .first_block_by_kind(kind)
            .map(|block| zone_rules(self.zones.get(block.origin_tile)).speed_multiplier)
            .unwrap_or(1.0)
            .max(0.1);
        (base_cycle_s / (speed * self.production_staffing_factor())).max(0.001)
    }

    fn modern_sorted_units_total(&self) -> u64 {
        u64::from(self.line.sacs_bleus_total)
            .saturating_mul(u64::from(SAC_CAPACITY_UNITS))
            .saturating_add(
                u64::from(self.line.sacs_rouges_total)
                    .saturating_mul(u64::from(SAC_CAPACITY_UNITS)),
            )
            .saturating_add(u64::from(self.line.blue_bag_fill))
            .saturating_add(u64::from(self.line.red_bag_fill))
    }

    fn modern_next_sortex_unit_is_blue(&self) -> bool {
        let next_unit_index = self.modern_sorted_units_total().saturating_add(1);
        !next_unit_index.is_multiple_of(5)
    }

    pub fn modern_line_ready(&self) -> bool {
        for kind in MODERN_LINE_REQUIRED_KINDS {
            if self.first_block_by_kind(kind).is_none() {
                return false;
            }
        }

        let mut frontier = self.block_indices_of_kind(BlockKind::InputHopper);
        if frontier.is_empty() {
            return false;
        }
        frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::FluidityTank,
            &[BlockKind::Conveyor],
        );
        if frontier.is_empty() {
            return false;
        }
        frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::Cutter,
            &[BlockKind::Conveyor],
        );
        if frontier.is_empty() {
            return false;
        }
        frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::DistributorBelt,
            &[BlockKind::Conveyor],
        );
        if frontier.is_empty() {
            return false;
        }
        frontier = self.reachable_targets_from_starts(&frontier, BlockKind::DryerOven, &[]);
        if frontier.is_empty() {
            return false;
        }
        frontier = self.reachable_targets_from_starts(&frontier, BlockKind::OvenExitConveyor, &[]);
        if frontier.is_empty() {
            return false;
        }
        frontier = self.reachable_targets_from_starts(&frontier, BlockKind::Flaker, &[]);
        if frontier.is_empty() {
            return false;
        }
        let sortex_frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::Sortex,
            &[BlockKind::SuctionPipe],
        );
        if sortex_frontier.is_empty() {
            return false;
        }
        if self
            .reachable_targets_from_starts(
                &sortex_frontier,
                BlockKind::BlueBagChute,
                &[BlockKind::SuctionPipe],
            )
            .is_empty()
        {
            return false;
        }
        if self
            .reachable_targets_from_starts(
                &sortex_frontier,
                BlockKind::RedBagChute,
                &[BlockKind::SuctionPipe],
            )
            .is_empty()
        {
            return false;
        }
        true
    }

    pub fn modern_line_ready_cached_for_render(&self) -> bool {
        !self.modern_line_cache.dirty && self.modern_line_cache.reason.is_none()
    }

    pub fn modern_lavage_busy(&self) -> bool {
        self.line.lavage_busy
    }

    pub fn modern_coupe_busy(&self) -> bool {
        self.line.coupe_busy
    }

    pub fn modern_four_busy(&self) -> bool {
        self.line.four_busy
    }

    pub fn modern_floc_busy(&self) -> bool {
        self.line.floc_busy
    }

    pub fn modern_sortex_busy(&self) -> bool {
        self.line.sortex_busy
    }

    pub fn modern_lavage_progress_ratio(&self) -> f32 {
        if !self.line.lavage_busy {
            0.0
        } else {
            (self.line.lavage_progress_s
                / self.modern_stage_cycle_s(BlockKind::FluidityTank, MODERN_CYCLE_LAVAGE_S))
            .clamp(0.0, 1.0) as f32
        }
    }

    pub fn modern_coupe_progress_ratio(&self) -> f32 {
        if !self.line.coupe_busy {
            0.0
        } else {
            (self.line.coupe_progress_s
                / self.modern_stage_cycle_s(BlockKind::Cutter, MODERN_CYCLE_COUPE_S))
            .clamp(0.0, 1.0) as f32
        }
    }

    pub fn modern_four_progress_ratio(&self) -> f32 {
        if !self.line.four_busy {
            0.0
        } else {
            (self.line.four_progress_s
                / self.modern_stage_cycle_s(BlockKind::DryerOven, MODERN_CYCLE_FOUR_S))
            .clamp(0.0, 1.0) as f32
        }
    }

    pub fn modern_floc_progress_ratio(&self) -> f32 {
        if !self.line.floc_busy {
            0.0
        } else {
            (self.line.floc_progress_s
                / self.modern_stage_cycle_s(BlockKind::Flaker, MODERN_CYCLE_FLOC_S))
            .clamp(0.0, 1.0) as f32
        }
    }

    pub fn modern_sortex_progress_ratio(&self) -> f32 {
        if !self.line.sortex_busy {
            0.0
        } else {
            (self.line.sortex_progress_s
                / self.modern_stage_cycle_s(BlockKind::Sortex, MODERN_CYCLE_SORTEX_S))
            .clamp(0.0, 1.0) as f32
        }
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
                        orientation,
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

    pub fn valider_pose_bloc_script(
        &self,
        world: &crate::World,
        kind: BlockKind,
        tile: (i32, i32),
        orientation: BlockOrientation,
    ) -> Result<(i32, i32), String> {
        if !kind.is_player_buyable() {
            return Err("Bloc non achetable par le joueur".to_string());
        }
        self.can_place_block_at(world, kind, tile, orientation, None)
    }

    pub fn poser_bloc_script(
        &mut self,
        world: &crate::World,
        kind: BlockKind,
        tile: (i32, i32),
        orientation: BlockOrientation,
        facturer_capex: bool,
    ) -> Result<BlockId, String> {
        let footprint = self.valider_pose_bloc_script(world, kind, tile, orientation)?;
        let capex = kind.capex();
        if facturer_capex && self.economy.cash < capex {
            return Err(format!(
                "Tresorerie insuffisante: {} EUR requis",
                format_int_fr(capex.round() as i64)
            ));
        }

        let id = self.next_block_id;
        self.next_block_id = self.next_block_id.saturating_add(1);
        if facturer_capex && capex > 0.0 {
            self.economy.spend(capex);
        }

        let mut block = self.make_block(id, kind, tile, orientation);
        block.footprint = footprint;
        self.blocks.push(block);
        self.mark_modern_line_cache_dirty();

        let (guidance, connected) = if kind.is_modern_line_component() {
            self.modern_line_placement_guidance(kind, tile, footprint, orientation)
        } else {
            (String::new(), true)
        };
        let mut status = format!(
            "Script pose {} #{} [{} {}x{}]",
            kind.buyable_label(),
            id,
            orientation.label(),
            footprint.0,
            footprint.1
        );
        if !guidance.is_empty() {
            if connected {
                status.push_str(" | ");
            } else {
                status.push_str(" | Alerte: ");
            }
            status.push_str(&guidance);
        }
        self.set_status_line(status);
        Ok(id)
    }

    fn next_modern_line_step(&self) -> Option<BlockKind> {
        let mut frontier = self.block_indices_of_kind(BlockKind::InputHopper);
        if frontier.is_empty() {
            return Some(BlockKind::InputHopper);
        }
        frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::FluidityTank,
            &[BlockKind::Conveyor],
        );
        if frontier.is_empty() {
            return Some(BlockKind::FluidityTank);
        }
        frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::Cutter,
            &[BlockKind::Conveyor],
        );
        if frontier.is_empty() {
            return Some(BlockKind::Cutter);
        }
        frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::DistributorBelt,
            &[BlockKind::Conveyor],
        );
        if frontier.is_empty() {
            return Some(BlockKind::DistributorBelt);
        }
        frontier = self.reachable_targets_from_starts(&frontier, BlockKind::DryerOven, &[]);
        if frontier.is_empty() {
            return Some(BlockKind::DryerOven);
        }
        frontier = self.reachable_targets_from_starts(&frontier, BlockKind::OvenExitConveyor, &[]);
        if frontier.is_empty() {
            return Some(BlockKind::OvenExitConveyor);
        }
        frontier = self.reachable_targets_from_starts(&frontier, BlockKind::Flaker, &[]);
        if frontier.is_empty() {
            return Some(BlockKind::Flaker);
        }
        let sortex_frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::Sortex,
            &[BlockKind::SuctionPipe],
        );
        if sortex_frontier.is_empty() {
            return Some(BlockKind::Sortex);
        }
        if self
            .reachable_targets_from_starts(
                &sortex_frontier,
                BlockKind::BlueBagChute,
                &[BlockKind::SuctionPipe],
            )
            .is_empty()
        {
            return Some(BlockKind::BlueBagChute);
        }
        if self
            .reachable_targets_from_starts(
                &sortex_frontier,
                BlockKind::RedBagChute,
                &[BlockKind::SuctionPipe],
            )
            .is_empty()
        {
            return Some(BlockKind::RedBagChute);
        }
        MODERN_LINE_GUIDE_ORDER
            .iter()
            .copied()
            .find(|kind| !self.blocks.iter().any(|block| block.kind == *kind))
    }

    fn modern_line_touching_kinds(
        &self,
        kind: BlockKind,
        tile: (i32, i32),
        footprint: (i32, i32),
        orientation: BlockOrientation,
    ) -> Vec<BlockKind> {
        let mut touched: Vec<BlockKind> = Vec::new();

        for block in &self.blocks {
            if !block.kind.is_modern_line_component() {
                continue;
            }
            if !Self::modern_line_blocks_touch(
                (kind, tile, footprint, orientation),
                (
                    block.kind,
                    block.origin_tile,
                    block.footprint,
                    block.orientation,
                ),
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
        orientation: BlockOrientation,
    ) -> (String, bool) {
        let touching = self.modern_line_touching_kinds(kind, tile, footprint, orientation);
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

        if let Some(reason) = self.modern_line_readiness_reason_uncached() {
            return (format!("Connexion a corriger: {reason}"), false);
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
            self.set_status_line(format!("Source deplacement=#{} {}", block_id, label));
        } else {
            self.set_status_line("Source deplacement: aucun bloc ici");
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
                self.purge_jobs_referencing_block(removed.id);
                self.mark_modern_line_cache_dirty();
                self.economy.earn(removed.kind.capex() * 0.6);
                self.set_status_line(format!(
                    "Vendu #{} {}",
                    removed.id,
                    removed.kind.buyable_label()
                ));
            } else {
                self.set_status_line("Aucun bloc a vendre");
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
                    self.mark_modern_line_cache_dirty();
                    self.set_status_line(format!(
                        "Deplace #{} -> ({}, {})",
                        move_id, tile.0, tile.1
                    ));
                } else {
                    self.set_status_line("Deplacement impossible: destination occupee");
                }
            } else {
                self.pending_move_block = None;
                self.set_status_line("Deplacement annule: source introuvable");
            }
            return;
        }

        if !self.block_brush.is_player_buyable() {
            self.set_status_line("Bloc non achetable par le joueur");
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
                self.set_status_line(reason);
                return;
            }
        };
        let (placement_guidance, placement_connected) =
            if self.block_brush.is_modern_line_component() {
                self.modern_line_placement_guidance(
                    self.block_brush,
                    tile,
                    footprint,
                    self.block_orientation,
                )
            } else {
                (String::new(), true)
            };

        let id = self.next_block_id;
        let capex = self.block_brush.capex();
        if self.economy.cash < capex {
            self.set_status_line(format!(
                "Tresorerie insuffisante: {} EUR requis",
                format_int_fr(capex.round() as i64)
            ));
            return;
        }
        self.next_block_id = self.next_block_id.saturating_add(1);
        self.economy.spend(capex);
        let mut block = self.make_block(id, self.block_brush, tile, self.block_orientation);
        block.footprint = footprint;
        self.blocks.push(block);
        self.mark_modern_line_cache_dirty();
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
        self.set_status_line(status);
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
        self.set_status_line(format!("Layout usine sauvegarde: {FACTORY_LAYOUT_PATH}"));
        Ok(())
    }

    pub(crate) fn to_save_asset(&self) -> FactorySimSaveAsset {
        FactorySimSaveAsset {
            schema_version: FACTORY_SIM_SAVE_SCHEMA_VERSION,
            clock: self.clock,
            economy: self.economy.clone(),
            personnel: self.personnel.clone(),
            stock: self.stock.clone(),
            sales: self.sales.clone(),
            production_lines: self.production_lines.clone(),
            line: self.line.clone(),
            layout: FactoryLayoutAsset {
                schema_version: FACTORY_LAYOUT_SCHEMA_VERSION,
                map_w: self.map_w,
                map_h: self.map_h,
                zones: self.zones.clone(),
                blocks: self.blocks.clone(),
                agent_tile: self.agent.tile,
            },
        }
    }

    pub(crate) fn from_save_asset(
        config: StarterSimConfig,
        asset: FactorySimSaveAsset,
    ) -> Result<Self, String> {
        if asset.schema_version > FACTORY_SIM_SAVE_SCHEMA_VERSION {
            return Err(format!(
                "schema simulation futur non supporte ({} > {})",
                asset.schema_version, FACTORY_SIM_SAVE_SCHEMA_VERSION
            ));
        }
        asset.layout.validate()?;
        let mut sim = Self::from_layout(config, asset.layout);
        sim.clock = asset.clock;
        sim.economy = asset.economy;
        sim.personnel = asset.personnel;
        sim.stock = asset.stock;
        sim.sales = asset.sales;
        sim.production_lines = if asset.production_lines.is_empty() {
            vec![ProductionLineState::main_line()]
        } else {
            asset.production_lines
        };
        sim.line = asset.line;
        sim.line.raw = sim.stock.raw_line_input;
        sim.refresh_static_capabilities();
        sim.mark_modern_line_cache_dirty();
        Ok(sim)
    }

    pub fn zone_kind_at_tile(&self, tile: (i32, i32)) -> ZoneKind {
        self.zones.get(tile)
    }

    pub fn block_debug_views(&self) -> Vec<BlockDebugView> {
        self.block_debug_views_with_options(true)
    }

    pub fn block_render_views(&self) -> impl Iterator<Item = BlockRenderView> + '_ {
        self.blocks.iter().map(|block| BlockRenderView {
            raw_qty: block.inventory.total_of(ItemKind::Raw),
            id: block.id,
            kind: block.kind,
            tile: block.origin_tile,
            footprint: block.footprint,
            orientation: block.orientation,
            rack_levels: block.rack_palettes,
        })
    }

    fn block_debug_views_with_options(
        &self,
        include_inventory_summary: bool,
    ) -> Vec<BlockDebugView> {
        self.blocks
            .iter()
            .map(|block| {
                let inventory_summary = if include_inventory_summary {
                    self.block_inventory_summary(block)
                } else {
                    String::new()
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

    fn block_inventory_summary(&self, block: &BlockInstance) -> String {
        match block.kind {
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
        }
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
        if self.build_status.is_empty() {
            &self.production_status
        } else {
            &self.build_status
        }
    }

    pub fn set_status_line(&mut self, status: impl Into<String>) {
        self.set_action_status(status);
    }

    fn apply_zone_rect_click(&mut self, tile: (i32, i32), right_click: bool) {
        let zone_target = if right_click {
            ZoneKind::Neutral
        } else {
            self.zone_brush
        };
        let Some(start) = self.pending_zone_rect_start.take() else {
            self.pending_zone_rect_start = Some(tile);
            self.set_status_line(format!(
                "Zone {}: coin 1 fixe en ({}, {}), clique le coin oppose",
                zone_target.label(),
                tile.0,
                tile.1
            ));
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
            self.set_status_line(format!(
                "Tresorerie insuffisante: {} EUR requis pour zone {} ({} tuiles)",
                format_int_fr(total_cost.round() as i64),
                zone_target.label(),
                changed_tiles
            ));
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

        self.set_status_line(format!(
            "Zone {} appliquee sur rectangle ({}, {}) -> ({}, {}) [{} tuiles]",
            zone_target.label(),
            min_x,
            min_y,
            max_x,
            max_y,
            changed_tiles
        ));
    }

    fn apply_floor_click(&mut self, world: &mut crate::World, tile: (i32, i32), right_click: bool) {
        if tile.0 <= 0 || tile.1 <= 0 || tile.0 >= self.map_w - 1 || tile.1 >= self.map_h - 1 {
            self.set_status_line("Pose sol impossible sur la bordure");
            return;
        }
        if world.is_solid(tile.0, tile.1) {
            self.set_status_line("Pose sol impossible: mur present");
            return;
        }

        let next_tile = if right_click {
            crate::Tile::Floor
        } else {
            self.floor_brush.to_tile()
        };
        if world.get(tile.0, tile.1) == next_tile {
            self.set_status_line("Aucun changement de sol");
            return;
        }

        let capex = if right_click {
            0.0
        } else {
            self.floor_brush.capex_par_tuile_eur()
        };
        if capex > 0.0 && self.economy.cash < capex {
            self.set_status_line(format!(
                "Tresorerie insuffisante: {} EUR requis",
                format_int_fr(capex.round() as i64)
            ));
            return;
        }
        if capex > 0.0 {
            self.economy.spend(capex);
        }

        world.set(tile.0, tile.1, next_tile);
        self.set_status_line(if right_click {
            format!("Sol reinitialise @ ({}, {})", tile.0, tile.1)
        } else {
            format!(
                "{} pose @ ({}, {})",
                self.floor_brush.label(),
                tile.0,
                tile.1
            )
        });
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

    fn tiles_rect_touch_cardinal(
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

        let gap_x = if a_max_x < b_min_x {
            b_min_x - a_max_x
        } else if b_max_x < a_min_x {
            a_min_x - b_max_x
        } else {
            0
        };
        let gap_y = if a_max_y < b_min_y {
            b_min_y - a_max_y
        } else if b_max_y < a_min_y {
            a_min_y - b_max_y
        } else {
            0
        };

        (gap_x <= 1 && gap_y == 0) || (gap_y <= 1 && gap_x == 0)
    }

    fn spans_overlap(a_min: i32, a_max: i32, b_min: i32, b_max: i32) -> bool {
        a_min <= b_max && a_max >= b_min
    }

    fn touching_side(
        origin: (i32, i32),
        size: (i32, i32),
        other_origin: (i32, i32),
        other_size: (i32, i32),
    ) -> Option<BlockOrientation> {
        let min_x = origin.0;
        let min_y = origin.1;
        let max_x = origin.0 + size.0 - 1;
        let max_y = origin.1 + size.1 - 1;
        let o_min_x = other_origin.0;
        let o_min_y = other_origin.1;
        let o_max_x = other_origin.0 + other_size.0 - 1;
        let o_max_y = other_origin.1 + other_size.1 - 1;

        if o_min_x == max_x + 1 && Self::spans_overlap(min_y, max_y, o_min_y, o_max_y) {
            return Some(BlockOrientation::East);
        }
        if o_max_x + 1 == min_x && Self::spans_overlap(min_y, max_y, o_min_y, o_max_y) {
            return Some(BlockOrientation::West);
        }
        if o_min_y == max_y + 1 && Self::spans_overlap(min_x, max_x, o_min_x, o_max_x) {
            return Some(BlockOrientation::South);
        }
        if o_max_y + 1 == min_y && Self::spans_overlap(min_x, max_x, o_min_x, o_max_x) {
            return Some(BlockOrientation::North);
        }

        None
    }

    fn modern_side_allows_output(
        kind: BlockKind,
        orientation: BlockOrientation,
        side: BlockOrientation,
    ) -> bool {
        match kind {
            // Start of line: only one outlet extremity.
            BlockKind::InputHopper => side == orientation,
            // Linear blocks push along orientation.
            BlockKind::Conveyor
            | BlockKind::DistributorBelt
            | BlockKind::DryerOven
            | BlockKind::OvenExitConveyor => side == orientation,
            // Chutes are terminal sinks.
            BlockKind::BlueBagChute | BlockKind::RedBagChute => false,
            _ => true,
        }
    }

    fn modern_side_allows_input(
        kind: BlockKind,
        orientation: BlockOrientation,
        side: BlockOrientation,
    ) -> bool {
        match kind {
            // Hopper is a source, not an inbound target.
            BlockKind::InputHopper => false,
            // Linear blocks accept from opposite extremity.
            BlockKind::Conveyor
            | BlockKind::DistributorBelt
            | BlockKind::DryerOven
            | BlockKind::OvenExitConveyor => side == orientation.opposite(),
            _ => true,
        }
    }

    fn modern_line_flow_from_to(
        a: (BlockKind, (i32, i32), (i32, i32), BlockOrientation),
        b: (BlockKind, (i32, i32), (i32, i32), BlockOrientation),
    ) -> bool {
        let (a_kind, a_origin, a_size, a_orientation) = a;
        let (b_kind, b_origin, b_size, b_orientation) = b;
        if !Self::tiles_rect_touch_cardinal(a_origin, a_size, b_origin, b_size) {
            return false;
        }

        let Some(a_side_to_b) = Self::touching_side(a_origin, a_size, b_origin, b_size) else {
            return false;
        };
        let b_side_to_a = a_side_to_b.opposite();

        Self::modern_side_allows_output(a_kind, a_orientation, a_side_to_b)
            && Self::modern_side_allows_input(b_kind, b_orientation, b_side_to_a)
    }

    fn modern_line_blocks_touch(
        a: (BlockKind, (i32, i32), (i32, i32), BlockOrientation),
        b: (BlockKind, (i32, i32), (i32, i32), BlockOrientation),
    ) -> bool {
        Self::modern_line_flow_from_to(a, b) || Self::modern_line_flow_from_to(b, a)
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
        Self::modern_line_flow_from_to(
            (a.kind, a.origin_tile, a.footprint, a.orientation),
            (b.kind, b.origin_tile, b.footprint, b.orientation),
        )
    }

    fn block_indices_of_kind(&self, kind: BlockKind) -> Vec<usize> {
        self.blocks
            .iter()
            .enumerate()
            .filter_map(|(idx, block)| (block.kind == kind).then_some(idx))
            .collect()
    }

    fn reachable_targets_from_starts(
        &self,
        starts: &[usize],
        target_kind: BlockKind,
        allowed_middle: &[BlockKind],
    ) -> Vec<usize> {
        if starts.is_empty() || !self.blocks.iter().any(|block| block.kind == target_kind) {
            return Vec::new();
        }

        let mut is_start = vec![false; self.blocks.len()];
        let mut visited = vec![false; self.blocks.len()];
        let mut queue = std::collections::VecDeque::new();
        for &idx in starts {
            if idx >= self.blocks.len() {
                continue;
            }
            is_start[idx] = true;
            visited[idx] = true;
            queue.push_back(idx);
        }
        let mut targets = Vec::new();

        while let Some(idx) = queue.pop_front() {
            for (n, seen) in visited.iter_mut().enumerate() {
                if *seen {
                    continue;
                }
                if !Self::block_footprints_touch(&self.blocks[idx], &self.blocks[n]) {
                    continue;
                }
                let nk = self.blocks[n].kind;
                let traversable = nk == target_kind || is_start[n] || allowed_middle.contains(&nk);
                if !traversable {
                    continue;
                }
                *seen = true;
                queue.push_back(n);
                if nk == target_kind {
                    targets.push(n);
                }
            }
        }

        targets
    }

    fn modern_line_present(&self) -> bool {
        self.blocks
            .iter()
            .any(|block| block.kind.is_modern_line_component())
    }

    fn modern_line_readiness_reason_uncached(&self) -> Option<String> {
        for kind in MODERN_LINE_REQUIRED_KINDS {
            if self.first_block_by_kind(kind).is_none() {
                return Some(format!("Bloc manquant: {}", kind.buyable_label()));
            }
        }

        let mut frontier = self.block_indices_of_kind(BlockKind::InputHopper);
        if frontier.is_empty() {
            return Some(format!(
                "Bloc manquant: {}",
                BlockKind::InputHopper.buyable_label()
            ));
        }
        frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::FluidityTank,
            &[BlockKind::Conveyor],
        );
        if frontier.is_empty() {
            return Some(
                "Connexion invalide: Entree ligne -> Bac fluidite (via convoyeur)".to_string(),
            );
        }
        frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::Cutter,
            &[BlockKind::Conveyor],
        );
        if frontier.is_empty() {
            return Some(
                "Connexion invalide: Bac fluidite -> Coupeuse (via convoyeur)".to_string(),
            );
        }
        frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::DistributorBelt,
            &[BlockKind::Conveyor],
        );
        if frontier.is_empty() {
            return Some("Connexion invalide: Coupeuse -> Tapis repartiteur".to_string());
        }
        frontier = self.reachable_targets_from_starts(&frontier, BlockKind::DryerOven, &[]);
        if frontier.is_empty() {
            return Some(
                "Connexion invalide: Tapis repartiteur -> Four deshydratation".to_string(),
            );
        }
        frontier = self.reachable_targets_from_starts(&frontier, BlockKind::OvenExitConveyor, &[]);
        if frontier.is_empty() {
            return Some("Connexion invalide: Four -> Tapis sortie four".to_string());
        }
        frontier = self.reachable_targets_from_starts(&frontier, BlockKind::Flaker, &[]);
        if frontier.is_empty() {
            return Some("Connexion invalide: Tapis sortie four -> Floconneuse".to_string());
        }
        let sortex_frontier = self.reachable_targets_from_starts(
            &frontier,
            BlockKind::Sortex,
            &[BlockKind::SuctionPipe],
        );
        if sortex_frontier.is_empty() {
            return Some("Connexion invalide: Floconneuse -> Sortex (via tuyaux)".to_string());
        }
        if self
            .reachable_targets_from_starts(
                &sortex_frontier,
                BlockKind::BlueBagChute,
                &[BlockKind::SuctionPipe],
            )
            .is_empty()
        {
            return Some("Connexion invalide: Sortex -> Descente sac bleu".to_string());
        }
        if self
            .reachable_targets_from_starts(
                &sortex_frontier,
                BlockKind::RedBagChute,
                &[BlockKind::SuctionPipe],
            )
            .is_empty()
        {
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

        let staffing_speed = self.production_staffing_factor();
        let cycle_a = (self.config.machine_a_cycle_s
            / (machine_a_zone_speed.max(0.1) * staffing_speed))
            .max(0.001);
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

        let cycle_b = (self.config.machine_b_cycle_s
            / (machine_b_zone_speed.max(0.1) * staffing_speed))
            .max(0.001);
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

        let _ = dt_hours;
    }

    fn tick_modern_line(&mut self, dt_sim: f64) {
        self.line.descente_bleue_beacon_s = (self.line.descente_bleue_beacon_s - dt_sim).max(0.0);
        self.line.descente_rouge_beacon_s = (self.line.descente_rouge_beacon_s - dt_sim).max(0.0);

        let lavage_cycle_s =
            self.modern_stage_cycle_s(BlockKind::FluidityTank, MODERN_CYCLE_LAVAGE_S);
        let coupe_cycle_s = self.modern_stage_cycle_s(BlockKind::Cutter, MODERN_CYCLE_COUPE_S);
        let four_cycle_s = self.modern_stage_cycle_s(BlockKind::DryerOven, MODERN_CYCLE_FOUR_S);
        let floc_cycle_s = self.modern_stage_cycle_s(BlockKind::Flaker, MODERN_CYCLE_FLOC_S);
        let sortex_cycle_s = self.modern_stage_cycle_s(BlockKind::Sortex, MODERN_CYCLE_SORTEX_S);

        if !self.line.lavage_busy && self.line.raw > 0 {
            self.line.raw -= 1;
            self.line.lavage_busy = true;
            self.line.lavage_progress_s = 0.0;
        }
        if self.line.lavage_busy {
            self.line.lavage_progress_s += dt_sim;
            if self.line.lavage_progress_s >= lavage_cycle_s {
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
            if self.line.coupe_progress_s >= coupe_cycle_s {
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
            if self.line.four_progress_s >= four_cycle_s {
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
            if self.line.floc_progress_s >= floc_cycle_s {
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
            if self.line.sortex_progress_s >= sortex_cycle_s {
                self.line.sortex_busy = false;
                self.line.sortex_progress_s = 0.0;
                if self.modern_next_sortex_unit_is_blue() {
                    self.line.blue_bag_fill = self.line.blue_bag_fill.saturating_add(1);
                    if self.line.blue_bag_fill >= SAC_CAPACITY_UNITS {
                        self.line.blue_bag_fill = 0;
                        self.line.sacs_bleus_total = self.line.sacs_bleus_total.saturating_add(1);
                        self.line.descente_bleue_beacon_s = 7.0;
                    }
                } else {
                    self.kpi.scrap_total = self.kpi.scrap_total.saturating_add(1);
                    self.line.red_bag_fill = self.line.red_bag_fill.saturating_add(1);
                    if self.line.red_bag_fill >= SAC_CAPACITY_UNITS {
                        self.line.red_bag_fill = 0;
                        self.line.sacs_rouges_total = self.line.sacs_rouges_total.saturating_add(1);
                        self.line.descente_rouge_beacon_s = 7.0;
                    }
                }
            }
        }

        self.sync_modern_finished_boxes();

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

    fn sync_modern_finished_boxes(&mut self) {
        let expected_boxes = self.line.sacs_bleus_total / SACS_PAR_BOX;
        if expected_boxes <= self.line.boxes_bleues_total {
            return;
        }

        let new_boxes = expected_boxes - self.line.boxes_bleues_total;
        self.line.boxes_bleues_total = expected_boxes;
        self.line.finished = self.line.finished.saturating_add(new_boxes);
        self.line.produced_finished_total =
            self.line.produced_finished_total.saturating_add(new_boxes);
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

    fn job_kind_references_block(kind: &JobKind, block_id: BlockId) -> bool {
        match kind {
            JobKind::Haul {
                from_block,
                to_block,
                ..
            } => *from_block == block_id || *to_block == block_id,
            JobKind::OperateMachine {
                block_id: job_block,
            } => *job_block == block_id,
        }
    }

    fn purge_jobs_by_ids(&mut self, removed_job_ids: &[JobId]) {
        if removed_job_ids.is_empty() {
            return;
        }
        if self
            .agent
            .current_job
            .is_some_and(|job_id| removed_job_ids.contains(&job_id))
        {
            self.agent.current_job = None;
            self.agent.job_progress_s = 0.0;
            self.agent.decision_debug = "job annule: cible invalide".to_string();
        }
        self.jobs.retain(|job| !removed_job_ids.contains(&job.id));
        self.reservations
            .retain(|_, reservation| !removed_job_ids.contains(&reservation.job_id));
    }

    fn purge_jobs_referencing_block(&mut self, block_id: BlockId) {
        let removed_job_ids = self
            .jobs
            .iter()
            .filter(|job| Self::job_kind_references_block(&job.kind, block_id))
            .map(|job| job.id)
            .collect::<Vec<_>>();
        self.purge_jobs_by_ids(&removed_job_ids);
    }

    fn purge_jobs_with_missing_blocks(&mut self) {
        let block_ids = self
            .blocks
            .iter()
            .map(|block| block.id)
            .collect::<HashSet<_>>();
        let removed_job_ids = self
            .jobs
            .iter()
            .filter(|job| match &job.kind {
                JobKind::Haul {
                    from_block,
                    to_block,
                    ..
                } => !block_ids.contains(from_block) || !block_ids.contains(to_block),
                JobKind::OperateMachine { block_id } => !block_ids.contains(block_id),
            })
            .map(|job| job.id)
            .collect::<Vec<_>>();
        self.purge_jobs_by_ids(&removed_job_ids);
    }

    fn tick_reservations(&mut self, dt_sim: f64) {
        self.reservations.retain(|_, reservation| {
            reservation.ttl_s -= dt_sim;
            reservation.ttl_s > 0.0
        });
    }

    fn refresh_jobs(&mut self) {
        self.purge_jobs_with_missing_blocks();
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
                    JobState::Pending
                        | JobState::Claimed
                        | JobState::InProgress
                        | JobState::Blocked(_)
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
                    if self.agent.decision_debug != "tache terminee" {
                        self.agent.decision_debug.clear();
                        self.agent.decision_debug.push_str("tache terminee");
                    }
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
            .filter(|(_, job)| matches!(job.state, JobState::Pending | JobState::Blocked(_)))
            .max_by_key(|(_, job)| job.priority)
            .map(|(idx, _)| idx)
        {
            let job_id = self.jobs[job_idx].id;
            let keys = self.reservation_keys_for_job(&self.jobs[job_idx].kind);
            if self.try_reserve_all(keys, job_id).is_ok() {
                let job_kind = self.jobs[job_idx].kind.clone();
                let job_priority = self.jobs[job_idx].priority;
                let job_label = self.job_kind_label(&job_kind);
                let job_target = self.job_target_label(&job_kind);
                let agent_id = self.agent.id;
                self.jobs[job_idx].state = JobState::Claimed;
                self.jobs[job_idx].assigned_agent = Some(agent_id);
                self.jobs[job_idx].score_debug = format!(
                    "priorite={} fatigue={:.1} stress={:.1}",
                    job_priority, self.agent.fatigue, self.agent.stress
                );
                self.agent.current_job = Some(job_id);
                self.agent.job_progress_s = 0.0;
                self.agent.decision_debug = format!(
                    "job=#{job_id} {job_label} cible={job_target} priorite={job_priority} score:{}",
                    self.jobs[job_idx].score_debug
                );
            } else {
                let job_kind = self.jobs[job_idx].kind.clone();
                let job_priority = self.jobs[job_idx].priority;
                let job_target = self.job_target_label(&job_kind);
                if !matches!(
                    self.jobs[job_idx].state,
                    JobState::Blocked(ref reason) if reason == "conflit reservation"
                ) {
                    self.jobs[job_idx].state = JobState::Blocked("conflit reservation".to_string());
                }
                self.agent.decision_debug = format!(
                    "bloque job=#{job_id} cible={job_target} priorite={job_priority} raison=conflit reservation"
                );
            }
        } else {
            if self.agent.decision_debug != "inactif(aucune tache en attente)" {
                self.agent.decision_debug.clear();
                self.agent
                    .decision_debug
                    .push_str("inactif(aucune tache en attente)");
            }
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
        path: &str,
        map_w: i32,
        map_h: i32,
        config: &StarterSimConfig,
    ) -> (FactoryLayoutAsset, Option<String>) {
        match Self::load_layout_asset(path) {
            Ok(layout) => {
                if layout.map_w != map_w || layout.map_h != map_h {
                    return (
                        Self::default_layout(map_w, map_h, config),
                        Some(format!(
                            "layout usine ignore car dimensions {}x{} != carte {}x{}",
                            layout.map_w, layout.map_h, map_w, map_h
                        )),
                    );
                }
                if layout.blocks.is_empty() {
                    return (
                        Self::default_layout(map_w, map_h, config),
                        Some("layout usine vide, defaut actif".to_string()),
                    );
                }
                (layout, None)
            }
            Err(err) if err.contains("echec lecture layout usine") => {
                let layout = Self::default_layout(map_w, map_h, config);
                let warning =
                    Self::save_layout_static_to_path(path, &layout)
                        .err()
                        .map(|save_err| {
                            format!(
                                "layout usine par defaut actif, ecriture impossible: {save_err}"
                            )
                        });
                (layout, warning)
            }
            Err(err) => {
                let layout = Self::default_layout(map_w, map_h, config);
                (
                    layout,
                    Some(format!("layout usine invalide, defaut non persiste: {err}")),
                )
            }
        }
    }

    fn load_layout_asset(path: &str) -> Result<FactoryLayoutAsset, String> {
        let raw = fs::read_to_string(path).map_err(|e| {
            if e.kind() == ErrorKind::NotFound {
                format!("echec lecture layout usine: {e}")
            } else {
                format!("layout usine illisible: {e}")
            }
        })?;
        let layout: FactoryLayoutAsset =
            ron_from_str(&raw).map_err(|e| format!("echec lecture RON layout usine: {e}"))?;
        layout.validate()?;
        Ok(layout)
    }

    fn save_layout_asset(&self, layout: &FactoryLayoutAsset) -> Result<(), String> {
        Self::save_layout_static(layout)
    }

    fn save_layout_static(layout: &FactoryLayoutAsset) -> Result<(), String> {
        Self::save_layout_static_to_path(FACTORY_LAYOUT_PATH, layout)
    }

    fn save_layout_static_to_path(path: &str, layout: &FactoryLayoutAsset) -> Result<(), String> {
        layout.validate()?;
        let payload = ron_to_string_pretty(layout, PrettyConfig::new().depth_limit(4))
            .map_err(|e| format!("echec serialisation layout usine: {e}"))?;
        if let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .map_err(|e| format!("echec creation dossier layout usine: {e}"))?;
        }
        fs::write(path, payload).map_err(|e| format!("echec ecriture layout usine: {e}"))
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
            self.status_line()
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

    fn temp_ron_path(name: &str) -> std::path::PathBuf {
        let path =
            std::env::temp_dir().join(format!("rxchixs_sim_{}_{}.ron", name, std::process::id()));
        let _ = fs::remove_file(&path);
        path
    }

    #[test]
    fn sim_clock_formats_hhmm() {
        let mut clock = SimClock::new();
        assert_eq!(clock.format_hhmm(), "00:00");
        clock.advance(60.0 * 60.0 * 25.0);
        assert_eq!(clock.day_index(), 1);
        assert_eq!(clock.format_hhmm(), "01:00");
    }

    #[test]
    fn invalid_sim_config_falls_back_without_overwriting_file() {
        let path = temp_ron_path("bad_config");
        let raw = "(schema_version: 99, time_scale: -1.0)";
        fs::write(&path, raw).expect("config invalide ecrite");

        let (cfg, warning) =
            StarterSimConfig::load_or_create_with_warning(path.to_str().expect("path utf8"));

        assert_eq!(cfg.schema_version, STARTER_SIM_CONFIG_SCHEMA_VERSION);
        assert!(
            warning
                .as_deref()
                .is_some_and(|msg| msg.contains("defaut non persiste"))
        );
        assert_eq!(fs::read_to_string(&path).expect("lecture config"), raw);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn invalid_factory_layout_falls_back_without_overwriting_file() {
        let path = temp_ron_path("bad_layout");
        let raw = "(schema_version: 999, map_w: 25, map_h: 15)";
        fs::write(&path, raw).expect("layout invalide ecrit");

        let (layout, warning) = FactorySim::load_or_create_layout(
            path.to_str().expect("path utf8"),
            25,
            15,
            &StarterSimConfig::default(),
        );

        assert_eq!(layout.schema_version, FACTORY_LAYOUT_SCHEMA_VERSION);
        assert!(
            warning
                .as_deref()
                .is_some_and(|msg| msg.contains("defaut non persiste"))
        );
        assert_eq!(fs::read_to_string(&path).expect("lecture layout"), raw);

        let _ = fs::remove_file(path);
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
        for _ in 0..4_000 {
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
    fn action_status_survives_production_tick_until_deterministic_expiry() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let mut world = crate::World::new_room(25, 15);
        sim.toggle_build_mode();

        sim.apply_build_click(&mut world, (6, 11), false);
        assert!(sim.status_line().contains("Place"));

        sim.step(1.0 / 60.0);
        assert!(sim.status_line().contains("Place"));

        for _ in 0..ACTION_STATUS_TTL_SIM_SECONDS as usize {
            sim.step(1.0 / 60.0);
        }
        assert!(!sim.status_line().contains("Place"));
    }

    #[test]
    fn sales_blocked_status_is_visible_after_tick() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let admin_id = sim
            .personnel()
            .employees
            .iter()
            .find(|employee| employee.role == EmployeeRole::AdministrateurVente)
            .map(|employee| employee.id)
            .expect("sandbox admin should exist");
        sim.apply_command(SimCommand::FireEmployee {
            employee_id: admin_id,
        })
        .expect("admin should be fireable");
        sim.line.finished = 2;

        sim.step(1.0 / 60.0);

        assert!(sim.status_line().contains("Vente en attente"));
        assert!(sim.status_line().contains("administrateur"));
        assert_eq!(sim.line.finished, 2);
    }

    #[test]
    fn production_requires_assigned_team_lead() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let lead_id = sim
            .personnel
            .team_lead_for_line(MAIN_PRODUCTION_LINE_ID)
            .map(|lead| lead.id)
            .expect("sandbox lead should exist");
        sim.apply_command(SimCommand::FireEmployee {
            employee_id: lead_id,
        })
        .expect("lead should be fireable");
        sim.stock.raw_line_input = 20;
        sim.line.raw = 20;

        for _ in 0..300 {
            sim.step(1.0 / 60.0);
        }

        assert_eq!(sim.line.produced_wip_total, 0);
        assert!(sim.status_line().contains("chef"));
    }

    #[test]
    fn cariste_is_required_to_feed_line_from_receiving_stock() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let cariste_id = sim
            .personnel
            .employees
            .iter()
            .find(|employee| employee.role == EmployeeRole::Cariste)
            .map(|employee| employee.id)
            .expect("sandbox cariste should exist");
        sim.apply_command(SimCommand::FireEmployee {
            employee_id: cariste_id,
        })
        .expect("cariste should be fireable");
        sim.stock.raw_receiving = 500;
        sim.stock.raw_line_input = 0;
        sim.line.raw = 0;

        sim.step(60.0);

        assert_eq!(sim.stock.raw_line_input, 0);
        assert!(sim.status_line().contains("caristes"));
    }

    #[test]
    fn bought_raw_stock_is_paid_delivered_and_moved_by_cariste() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        sim.stock.raw_receiving = 0;
        sim.stock.raw_line_input = 0;
        sim.line.raw = 0;
        let cash_before = sim.cash();

        sim.apply_command(SimCommand::BuyRawStock { qty: 500 })
            .expect("raw stock purchase should succeed");
        assert!(sim.cash() < cash_before);
        assert_eq!(sim.stock.pending_raw_qty(), 500);

        sim.step((crate::gestion::stock::RAW_DELIVERY_DELAY_S / sim.config.time_scale) as f32);

        assert_eq!(sim.stock.pending_raw_qty(), 0);
        assert!(sim.stock.raw_receiving > 0 || sim.stock.raw_line_input > 0);
        assert!(sim.stock.raw_line_input <= crate::gestion::stock::RAW_LINE_INPUT_CAPACITY);
    }

    #[test]
    fn finished_goods_are_sold_progressively_by_admins() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let seller_tile = sim
            .blocks
            .iter()
            .find(|block| block.kind == BlockKind::Seller)
            .map(|block| block.origin_tile)
            .expect("default seller should exist");
        sim.zones.set(seller_tile, ZoneKind::Support);
        sim.line.finished = 10;

        sim.step(1.0 / 60.0);
        assert_eq!(
            sim.line.finished, 10,
            "one second should not sell a full box"
        );

        for _ in 0..1_200 {
            sim.step(1.0 / 60.0);
        }
        assert!(sim.line.sold_total > 0);
    }

    #[test]
    fn team_lead_never_creates_more_than_three_temps() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let lead_id = sim
            .personnel
            .team_lead_for_line(MAIN_PRODUCTION_LINE_ID)
            .map(|lead| lead.id)
            .expect("sandbox lead should exist");
        sim.personnel
            .set_temp_policy(lead_id, true, 99)
            .expect("temp policy should clamp");
        sim.stock.raw_receiving = 2_000;
        sim.stock.raw_line_input = 120;

        for _ in 0..60 {
            sim.step(1.0 / 60.0);
        }

        assert!(sim.personnel.active_temps_for_lead(lead_id) <= 3);
    }

    #[test]
    fn factory_layout_validation_rejects_overlapping_block_footprints() {
        let layout = FactoryLayoutAsset {
            schema_version: FACTORY_LAYOUT_SCHEMA_VERSION,
            map_w: 12,
            map_h: 12,
            zones: ZoneLayer::new(12, 12, ZoneKind::Neutral),
            blocks: vec![
                BlockInstance {
                    id: 1,
                    kind: BlockKind::Storage,
                    origin_tile: (3, 3),
                    footprint: (1, 1),
                    ..BlockInstance::default()
                },
                BlockInstance {
                    id: 2,
                    kind: BlockKind::Buffer,
                    origin_tile: (3, 3),
                    footprint: (1, 1),
                    ..BlockInstance::default()
                },
            ],
            agent_tile: (4, 4),
        };

        let err = layout.validate().expect_err("layout superpose refuse");
        assert!(err.contains("superposes"));
    }

    #[test]
    fn modern_line_cache_is_invalidated_by_placement_and_sale() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 80, 60);
        let mut world = crate::World::new_room(80, 60);
        sim.blocks.clear();
        sim.mark_modern_line_cache_dirty();
        assert!(sim.cached_modern_line_readiness_reason().is_some());
        assert!(!sim.modern_line_cache.dirty);

        sim.toggle_build_mode();
        sim.set_block_brush(BlockKind::InputHopper);
        sim.apply_build_click(&mut world, (20, 20), false);
        assert!(sim.modern_line_cache.dirty);

        assert!(sim.cached_modern_line_readiness_reason().is_some());
        assert!(!sim.modern_line_cache.dirty);

        sim.apply_build_click(&mut world, (20, 20), true);
        assert!(sim.modern_line_cache.dirty);
    }

    #[test]
    fn render_ready_read_uses_cached_modern_line_state_only() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 80, 60);
        sim.blocks.clear();
        sim.mark_modern_line_cache_dirty();

        assert!(!sim.modern_line_ready_cached_for_render());

        let reason = sim.cached_modern_line_readiness_reason();
        assert_eq!(sim.modern_line_ready_cached_for_render(), reason.is_none());

        sim.mark_modern_line_cache_dirty();
        assert!(!sim.modern_line_ready_cached_for_render());
    }

    #[test]
    fn agent_debug_label_exposes_blocked_job_priority_target_and_reason() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let job_id = 99;
        sim.jobs.push(Job {
            id: job_id,
            kind: JobKind::OperateMachine { block_id: 1 },
            state: JobState::Pending,
            priority: 42,
            score_debug: String::new(),
            assigned_agent: None,
        });
        sim.reservations.insert(
            ReservationKey::BlockInput(1),
            Reservation {
                job_id: 777,
                ttl_s: RESERVATION_TTL_SECONDS,
            },
        );

        sim.tick_agent(1.0 / 60.0);

        let label = &sim.agent_debug_views()[0].label;
        assert!(label.contains("job=#99"));
        assert!(label.contains("cible=B1"));
        assert!(label.contains("priorite=42"));
        assert!(label.contains("raison=conflit reservation"));
    }

    #[test]
    fn selling_block_purges_related_jobs_and_reservations() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let mut world = crate::World::new_room(25, 15);
        let machine = sim
            .blocks
            .iter()
            .find(|block| block.kind == BlockKind::MachineA)
            .expect("machine A attendue");
        let machine_id = machine.id;
        let machine_tile = machine.origin_tile;
        let job_id = 44;
        sim.jobs.push(Job {
            id: job_id,
            kind: JobKind::OperateMachine {
                block_id: machine_id,
            },
            state: JobState::InProgress,
            priority: 60,
            score_debug: "test".to_string(),
            assigned_agent: Some(sim.agent.id),
        });
        sim.agent.current_job = Some(job_id);
        sim.agent.job_progress_s = 2.0;
        sim.reservations.insert(
            ReservationKey::BlockInput(machine_id),
            Reservation {
                job_id,
                ttl_s: RESERVATION_TTL_SECONDS,
            },
        );

        sim.toggle_build_mode();
        sim.apply_build_click(&mut world, machine_tile, true);

        assert!(sim.block_index_by_id(machine_id).is_none());
        assert!(sim.jobs.iter().all(|job| job.id != job_id));
        assert_eq!(sim.agent.current_job, None);
        assert!(sim.reservations.is_empty());
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
    fn minimal_block_debug_views_keep_structure_without_inventory_strings() {
        let sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let full = sim.block_debug_views();
        let minimal = sim.block_debug_views_with_options(false);

        assert_eq!(full.len(), minimal.len());
        for (lhs, rhs) in full.iter().zip(minimal.iter()) {
            assert_eq!(lhs.id, rhs.id);
            assert_eq!(lhs.kind, rhs.kind);
            assert_eq!(lhs.tile, rhs.tile);
            assert_eq!(lhs.footprint, rhs.footprint);
            assert_eq!(lhs.orientation, rhs.orientation);
            assert_eq!(lhs.raw_qty, rhs.raw_qty);
            assert_eq!(lhs.rack_levels, rhs.rack_levels);
            assert!(rhs.inventory_summary.is_empty());
        }
    }

    #[test]
    fn block_render_views_match_debug_structure_without_inventory_strings() {
        let sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        let full = sim.block_debug_views();
        let render: Vec<BlockRenderView> = sim.block_render_views().collect();

        assert_eq!(full.len(), render.len());
        for (lhs, rhs) in full.iter().zip(render.iter()) {
            assert_eq!(lhs.render_view(), *rhs);
        }
    }

    #[test]
    fn refresh_jobs_does_not_duplicate_existing_blocked_kind() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        sim.jobs.clear();

        let storage_id = sim
            .blocks
            .iter()
            .find(|b| b.kind == BlockKind::Storage)
            .expect("storage block should exist")
            .id;
        let machine_a_id = sim
            .blocks
            .iter()
            .find(|b| b.kind == BlockKind::MachineA)
            .expect("machine A block should exist")
            .id;
        let blocked_kind = JobKind::Haul {
            from_block: storage_id,
            to_block: machine_a_id,
            item_kind: ItemKind::Raw,
            qty: 1,
        };
        sim.jobs.push(Job {
            id: 9000,
            kind: blocked_kind.clone(),
            state: JobState::Blocked("conflit reservation".to_string()),
            priority: 50,
            score_debug: String::new(),
            assigned_agent: None,
        });
        sim.line.raw = 24;

        for _ in 0..20 {
            sim.refresh_jobs();
        }

        let same_kind = sim
            .jobs
            .iter()
            .filter(|job| job.kind == blocked_kind)
            .count();
        assert_eq!(
            same_kind, 1,
            "refresh_jobs should not append duplicates for an already blocked kind"
        );
    }

    #[test]
    fn tick_agent_retries_blocked_job_when_reservations_are_available() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 25, 15);
        sim.jobs.clear();
        sim.reservations.clear();

        let machine_a_id = sim
            .blocks
            .iter()
            .find(|b| b.kind == BlockKind::MachineA)
            .expect("machine A block should exist")
            .id;
        let blocked_job_id = 77;
        sim.jobs.push(Job {
            id: blocked_job_id,
            kind: JobKind::OperateMachine {
                block_id: machine_a_id,
            },
            state: JobState::Blocked("conflit reservation".to_string()),
            priority: 60,
            score_debug: String::new(),
            assigned_agent: None,
        });

        sim.agent.current_job = None;
        sim.agent.job_progress_s = 0.0;
        sim.tick_agent(1.0 / 60.0);

        assert_eq!(sim.agent.current_job, Some(blocked_job_id));
        let job = sim
            .jobs
            .iter()
            .find(|j| j.id == blocked_job_id)
            .expect("blocked test job should still exist");
        assert!(matches!(job.state, JobState::Claimed));
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
        assert!(sim.sales_block_reason().contains("administrateur"));
    }

    #[test]
    fn dryer_oven_footprint_rotates_with_orientation() {
        let horizontal = BlockKind::DryerOven.footprint_for_orientation(BlockOrientation::East);
        let vertical = BlockKind::DryerOven.footprint_for_orientation(BlockOrientation::South);
        assert_eq!(horizontal, (20, 10));
        assert_eq!(vertical, (10, 20));
    }

    #[test]
    fn input_hopper_footprint_rotates_with_orientation() {
        let horizontal = BlockKind::InputHopper.footprint_for_orientation(BlockOrientation::East);
        let vertical = BlockKind::InputHopper.footprint_for_orientation(BlockOrientation::South);
        assert_eq!(horizontal, (8, 3));
        assert_eq!(vertical, (3, 8));
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
        let Some(preview) = sim.build_block_preview(&world, (20, 20)) else {
            panic!("preview should exist");
        };

        assert!(preview.can_place, "preview doit etre placable ici");
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

        assert!(sim.build_mode_enabled());
        sim.set_block_brush(BlockKind::InputHopper);
        sim.set_block_orientation(BlockOrientation::East);
        sim.apply_build_click(&mut world, (10, 20), false);

        sim.set_block_brush(BlockKind::FluidityTank);
        sim.set_block_orientation(BlockOrientation::East);
        let Some(preview) = sim.build_block_preview(&world, (18, 20)) else {
            panic!("preview should exist");
        };

        assert!(preview.can_place);
        assert!(preview.connects_to_line);
        assert!(preview.guidance.contains("Etape conseillee"));
    }

    #[test]
    fn modern_line_next_step_is_connectivity_driven() {
        fn place(
            sim: &mut FactorySim,
            world: &mut crate::World,
            kind: BlockKind,
            tile: (i32, i32),
            orientation: BlockOrientation,
        ) {
            let before = sim
                .block_debug_views()
                .into_iter()
                .filter(|b| b.kind == kind)
                .count();
            sim.set_block_brush(kind);
            sim.set_block_orientation(orientation);
            sim.apply_build_click(world, tile, false);
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
        }

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

        place(
            &mut sim,
            &mut world,
            BlockKind::InputHopper,
            (10, 20),
            BlockOrientation::East,
        );

        assert_eq!(
            sim.next_modern_line_step(),
            Some(BlockKind::FluidityTank),
            "apres entree ligne, la prochaine etape est bac fluidite"
        );

        place(
            &mut sim,
            &mut world,
            BlockKind::Conveyor,
            (18, 22),
            BlockOrientation::East,
        );
        place(
            &mut sim,
            &mut world,
            BlockKind::FluidityTank,
            (19, 20),
            BlockOrientation::East,
        );

        assert_eq!(
            sim.next_modern_line_step(),
            Some(BlockKind::Cutter),
            "apres raccord flux, la suivante etape doit etre la coupeuse"
        );
    }

    #[test]
    fn modern_line_rejects_side_connection_from_input_hopper() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 120, 90);
        let mut world = crate::World::new_room(120, 90);
        sim.toggle_build_mode();

        for tile in sim.block_debug_views().into_iter().map(|b| b.tile) {
            sim.apply_build_click(&mut world, tile, true);
        }

        let mut place = |kind: BlockKind, tile: (i32, i32)| {
            let before = sim
                .block_debug_views()
                .into_iter()
                .filter(|b| b.kind == kind)
                .count();
            sim.set_block_brush(kind);
            sim.set_block_orientation(BlockOrientation::East);
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

        place(BlockKind::InputHopper, (10, 20));
        place(BlockKind::Conveyor, (13, 23));
        place(BlockKind::FluidityTank, (14, 23));

        assert_eq!(
            sim.next_modern_line_step(),
            Some(BlockKind::FluidityTank),
            "connexion laterale de l'entree ligne ne doit pas valider le flux vers le bac"
        );
    }

    #[test]
    fn modern_line_rejects_reverse_conveyor_direction_after_input_hopper() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 120, 90);
        let mut world = crate::World::new_room(120, 90);
        sim.toggle_build_mode();

        for tile in sim.block_debug_views().into_iter().map(|b| b.tile) {
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
        place(BlockKind::Conveyor, (18, 22), BlockOrientation::West);
        place(BlockKind::FluidityTank, (19, 20), BlockOrientation::East);

        assert_eq!(
            sim.next_modern_line_step(),
            Some(BlockKind::FluidityTank),
            "convoyeur inverse apres entree ligne ne doit pas valider la connexion vers le bac"
        );
    }

    #[test]
    fn modern_line_rejects_side_connection_from_dryer_oven() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 140, 100);
        let mut world = crate::World::new_room(140, 100);
        sim.toggle_build_mode();

        for tile in sim.block_debug_views().into_iter().map(|b| b.tile) {
            sim.apply_build_click(&mut world, tile, true);
        }

        let mut place = |kind: BlockKind, tile: (i32, i32)| {
            let before = sim
                .block_debug_views()
                .into_iter()
                .filter(|b| b.kind == kind)
                .count();
            sim.set_block_brush(kind);
            sim.set_block_orientation(BlockOrientation::East);
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

        place(BlockKind::InputHopper, (10, 20));
        place(BlockKind::Conveyor, (18, 22));
        place(BlockKind::FluidityTank, (19, 20));
        place(BlockKind::Conveyor, (24, 22));
        place(BlockKind::Cutter, (25, 21));
        place(BlockKind::DistributorBelt, (28, 22));
        place(BlockKind::DryerOven, (35, 17));
        place(BlockKind::OvenExitConveyor, (40, 27));

        assert_eq!(
            sim.next_modern_line_step(),
            Some(BlockKind::OvenExitConveyor),
            "connexion laterale du four ne doit pas valider le tapis sortie four"
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
        place(BlockKind::Conveyor, (18, 22), BlockOrientation::East);
        place(BlockKind::FluidityTank, (19, 20), BlockOrientation::East);
        place(BlockKind::Cutter, (70, 50), BlockOrientation::East);
        place(BlockKind::DistributorBelt, (73, 51), BlockOrientation::East);
        place(BlockKind::DryerOven, (80, 42), BlockOrientation::East);
        place(
            BlockKind::OvenExitConveyor,
            (100, 51),
            BlockOrientation::East,
        );
        place(BlockKind::Flaker, (107, 50), BlockOrientation::East);
        place(BlockKind::SuctionPipe, (110, 51), BlockOrientation::East);
        place(BlockKind::Sortex, (111, 49), BlockOrientation::East);
        place(BlockKind::BlueBagChute, (115, 49), BlockOrientation::East);
        place(BlockKind::RedBagChute, (115, 52), BlockOrientation::East);

        sim.build_status.clear();
        sim.build_status_ttl_s = 0.0;
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
        place(BlockKind::Conveyor, (18, 22), BlockOrientation::East);
        place(BlockKind::FluidityTank, (19, 20), BlockOrientation::East);
        place(BlockKind::Conveyor, (24, 22), BlockOrientation::East);
        place(BlockKind::Cutter, (25, 21), BlockOrientation::East);
        place(BlockKind::DistributorBelt, (28, 22), BlockOrientation::East);
        place(BlockKind::DryerOven, (35, 17), BlockOrientation::East);
        place(
            BlockKind::OvenExitConveyor,
            (55, 22),
            BlockOrientation::East,
        );
        place(BlockKind::Flaker, (62, 21), BlockOrientation::East);
        place(BlockKind::SuctionPipe, (65, 22), BlockOrientation::East);
        place(BlockKind::SuctionPipe, (66, 22), BlockOrientation::East);
        place(BlockKind::Sortex, (67, 20), BlockOrientation::East);
        place(BlockKind::BlueBagChute, (71, 20), BlockOrientation::East);
        place(BlockKind::RedBagChute, (71, 23), BlockOrientation::East);

        for _ in 0..1000 {
            sim.step(1.0 / 60.0);
        }

        assert!(!sim.status_line().contains("non operationnelle"));
        assert!(sim.line.produced_wip_total > 0);
        assert!(sim.line.sacs_bleus_total + sim.line.sacs_rouges_total > 0);
    }

    #[test]
    fn modern_sortex_counts_red_outputs_as_scrap() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 80, 60);
        let lead_id = sim
            .personnel
            .team_lead_for_line(MAIN_PRODUCTION_LINE_ID)
            .map(|lead| lead.id)
            .expect("sandbox lead should exist");
        sim.main_line_state_mut().set_active(lead_id, 2);
        sim.line.flakes = 5;

        for _ in 0..5 {
            sim.tick_modern_line(MODERN_CYCLE_SORTEX_S);
        }

        assert_eq!(sim.line.blue_bag_fill, 4);
        assert_eq!(sim.line.red_bag_fill, 1);
        assert_eq!(sim.kpi.scrap_total, 1);
        assert_eq!(sim.modern_sorted_units_total(), 5);
    }

    #[test]
    fn modern_stage_cycles_use_block_zone_speed() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 80, 60);
        let world = crate::World::new_room(80, 60);
        sim.blocks.clear();
        for y in 20..23 {
            for x in 20..23 {
                sim.zones.set((x, y), ZoneKind::Processing);
            }
        }
        sim.poser_bloc_script(
            &world,
            BlockKind::Cutter,
            (20, 20),
            BlockOrientation::East,
            false,
        )
        .expect("cutter should be placeable");
        sim.line.washed = 1;

        let lead_id = sim
            .personnel
            .team_lead_for_line(MAIN_PRODUCTION_LINE_ID)
            .map(|lead| lead.id)
            .expect("sandbox lead should exist");
        sim.main_line_state_mut().set_active(lead_id, 2);
        let boosted_cycle_s = MODERN_CYCLE_COUPE_S
            / (zone_rules(ZoneKind::Processing).speed_multiplier
                * sim.production_staffing_factor());
        assert!(boosted_cycle_s < MODERN_CYCLE_COUPE_S);
        sim.tick_modern_line(boosted_cycle_s + 0.01);

        assert!(!sim.line.coupe_busy);
        assert!(sim.line.four_busy);
    }

    #[test]
    fn modern_box_sync_batches_blue_bags_without_iterating_each_box() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 80, 60);
        sim.line.sacs_bleus_total = SACS_PAR_BOX * 3;
        sim.line.boxes_bleues_total = 1;

        sim.sync_modern_finished_boxes();

        assert_eq!(sim.line.boxes_bleues_total, 3);
        assert_eq!(sim.line.finished, 2);
        assert_eq!(sim.line.produced_finished_total, 2);
    }

    #[test]
    fn modern_line_ready_rejects_pairwise_connected_but_disjoint_chain() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 220, 220);
        let world = crate::World::new_room(220, 220);
        sim.blocks.clear();

        let mut place = |kind: BlockKind, tile: (i32, i32)| {
            sim.poser_bloc_script(&world, kind, tile, BlockOrientation::East, false)
                .unwrap_or_else(|err| {
                    panic!("placement failed for {:?} at {:?}: {err}", kind, tile)
                });
        };

        // Segment A: Input -> Fluidity.
        place(BlockKind::InputHopper, (20, 20));
        place(BlockKind::Conveyor, (28, 22));
        place(BlockKind::FluidityTank, (29, 20));

        // Segment B: Fluidity -> Cutter (different fluidity tank).
        place(BlockKind::FluidityTank, (20, 60));
        place(BlockKind::Conveyor, (25, 62));
        place(BlockKind::Cutter, (26, 61));

        // Segment C: Cutter -> Distributor (different cutter).
        place(BlockKind::Cutter, (20, 100));
        place(BlockKind::Conveyor, (23, 101));
        place(BlockKind::DistributorBelt, (24, 101));

        // Segment D: Distributor -> Dryer (different distributor).
        place(BlockKind::DistributorBelt, (60, 100));
        place(BlockKind::DryerOven, (67, 91));

        // Segment E: Dryer -> Oven exit (different dryer).
        place(BlockKind::DryerOven, (100, 90));
        place(BlockKind::OvenExitConveyor, (110, 100));

        // Segment F: Oven exit -> Flaker (different oven exit).
        place(BlockKind::OvenExitConveyor, (140, 100));
        place(BlockKind::Flaker, (147, 99));

        // Segment G: Flaker -> Sortex via suction (different flaker).
        place(BlockKind::Flaker, (170, 100));
        place(BlockKind::SuctionPipe, (173, 101));
        place(BlockKind::Sortex, (174, 99));

        // Segment H: Sortex -> blue/red chutes via suction (different sortex).
        place(BlockKind::Sortex, (170, 140));
        place(BlockKind::SuctionPipe, (174, 141));
        place(BlockKind::SuctionPipe, (174, 142));
        place(BlockKind::BlueBagChute, (175, 140));
        place(BlockKind::RedBagChute, (175, 143));

        assert!(
            !sim.modern_line_ready(),
            "line should be rejected: pairwise links exist but no coherent end-to-end chain"
        );
    }

    #[test]
    fn modern_line_rejects_diagonal_only_link_between_stages() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 120, 90);
        let world = crate::World::new_room(120, 90);
        sim.blocks.clear();

        let mut place = |kind: BlockKind, tile: (i32, i32)| {
            sim.poser_bloc_script(&world, kind, tile, BlockOrientation::East, false)
                .unwrap_or_else(|err| {
                    panic!("placement failed for {:?} at {:?}: {err}", kind, tile)
                });
        };

        // Input -> conveyor is cardinally adjacent.
        place(BlockKind::InputHopper, (10, 20));
        place(BlockKind::Conveyor, (18, 22));
        // Conveyor -> tank is only corner-to-corner (diagonal), should not validate.
        place(BlockKind::FluidityTank, (19, 23));
        place(BlockKind::Conveyor, (24, 25));
        place(BlockKind::Cutter, (25, 24));
        place(BlockKind::DistributorBelt, (28, 25));
        place(BlockKind::DryerOven, (35, 16));
        place(BlockKind::OvenExitConveyor, (55, 25));
        place(BlockKind::Flaker, (62, 24));
        place(BlockKind::SuctionPipe, (65, 25));
        place(BlockKind::SuctionPipe, (66, 25));
        place(BlockKind::Sortex, (67, 23));
        place(BlockKind::BlueBagChute, (71, 23));
        place(BlockKind::RedBagChute, (71, 26));

        assert!(
            !sim.modern_line_ready(),
            "line should be rejected when one stage only connects diagonally"
        );
    }

    #[test]
    fn build_preview_does_not_mark_diagonal_only_connection_as_linked() {
        let mut sim = FactorySim::new(StarterSimConfig::default(), 120, 90);
        let world = crate::World::new_room(120, 90);
        sim.blocks.clear();
        sim.toggle_build_mode();

        sim.poser_bloc_script(
            &world,
            BlockKind::InputHopper,
            (10, 20),
            BlockOrientation::East,
            false,
        )
        .expect("input should be placeable");
        sim.poser_bloc_script(
            &world,
            BlockKind::Conveyor,
            (18, 22),
            BlockOrientation::East,
            false,
        )
        .expect("conveyor should be placeable");

        sim.set_block_brush(BlockKind::FluidityTank);
        sim.set_block_orientation(BlockOrientation::East);
        let preview = sim
            .build_block_preview(&world, (19, 23))
            .expect("preview should exist in build mode");
        assert!(preview.can_place, "placement should still be valid");
        assert!(
            !preview.connects_to_line,
            "diagonal-only candidate should not be considered connected"
        );
    }

    #[test]
    fn scripted_block_placement_works_without_build_mode() {
        let cfg = StarterSimConfig::default();
        let mut sim = FactorySim::new(cfg, 60, 40);
        let world = crate::World::new_room(60, 40);
        assert!(!sim.build_mode_enabled());

        let id = sim
            .poser_bloc_script(
                &world,
                BlockKind::InputHopper,
                (24, 24),
                BlockOrientation::East,
                false,
            )
            .expect("scripted placement should succeed");
        assert!(id > 0);
        assert_eq!(
            sim.block_kind_at_tile((24, 24)),
            Some(BlockKind::InputHopper),
            "block should be present at requested origin"
        );
    }

    #[test]
    fn scripted_validation_rejects_overlap() {
        let cfg = StarterSimConfig::default();
        let mut sim = FactorySim::new(cfg, 60, 40);
        let world = crate::World::new_room(60, 40);

        sim.poser_bloc_script(
            &world,
            BlockKind::FluidityTank,
            (24, 24),
            BlockOrientation::East,
            false,
        )
        .expect("initial scripted placement should succeed");

        let err = sim
            .valider_pose_bloc_script(&world, BlockKind::Cutter, (26, 26), BlockOrientation::East)
            .expect_err("overlap should be rejected");
        assert!(err.contains("occupee"), "unexpected error: {err}");
    }
}
