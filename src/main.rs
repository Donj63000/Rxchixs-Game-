mod character;
mod sim;

use character::{
    CharacterCatalog, CharacterFacing, CharacterRecord, CharacterRenderParams,
    build_lineage_preview, compact_visual_summary, draw_character, inspector_lines,
};
use macroquad::prelude::*;
use ron::{
    de::from_str as ron_from_str,
    ser::{PrettyConfig, to_string_pretty as ron_to_string_pretty},
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::fs;
use std::path::Path;

const TILE_SIZE: f32 = 32.0;
const MAP_W: i32 = 96;
const MAP_H: i32 = 64;
const WINDOW_W: i32 = 1600;
const WINDOW_H: i32 = 900;
const FIXED_DT: f32 = 1.0 / 60.0;
const DIRECTION_HYSTERESIS: f32 = 0.18;
const WALK_CYCLE_SPEED: f32 = 9.0;
const AUTO_ACCEL: f32 = 720.0;
const AUTO_ARRIVE_RADIUS: f32 = 34.0;
const AUTO_WAYPOINT_REACH: f32 = 5.0;
const NPC_WANDER_SPEED: f32 = 92.0;
const NPC_IDLE_MIN: f32 = 0.7;
const NPC_IDLE_MAX: f32 = 2.0;
const NPC_GREETING_RADIUS: f32 = 26.0;
const NPC_GREETING_DURATION: f32 = 1.25;
const NPC_GREETING_COOLDOWN: f32 = 3.8;
const MAP_FILE_PATH: &str = "maps/main_map.ron";
const SIM_CONFIG_PATH: &str = "data/starter_sim.ron";
const MAP_FILE_VERSION: u32 = 3;
const EDITOR_UNDO_LIMIT: usize = 160;
const PLAY_CAMERA_MARGIN: f32 = 10.0;
const PLAY_CAMERA_PAN_SPEED: f32 = 880.0;
const PLAY_CAMERA_ZOOM_MIN: f32 = 0.55;
const PLAY_CAMERA_ZOOM_MAX: f32 = 2.65;
const PLAY_CAMERA_ZOOM_STEP: f32 = 0.24;
const EDITOR_CAMERA_PAN_SPEED: f32 = 980.0;
const EDITOR_CAMERA_ZOOM_MIN: f32 = 0.45;
const EDITOR_CAMERA_ZOOM_MAX: f32 = 3.2;
const EDITOR_CAMERA_ZOOM_STEP: f32 = 0.28;
const MAX_SIM_STEPS_PER_FRAME: usize = 8;

const MASK_N: u8 = 1 << 0;
const MASK_E: u8 = 1 << 1;
const MASK_S: u8 = 1 << 2;
const MASK_W: u8 = 1 << 3;

fn window_conf() -> Conf {
    Conf {
        window_title: "Rxchixs - Visual Room Prototype".to_string(),
        window_width: WINDOW_W,
        window_height: WINDOW_H,
        window_resizable: true,
        ..Default::default()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
enum Tile {
    Floor,
    FloorMetal,
    FloorWood,
    FloorMoss,
    FloorSand,
    Wall,
    WallBrick,
    WallSteel,
    WallNeon,
}

#[derive(Clone, Serialize, Deserialize)]
struct World {
    w: i32,
    h: i32,
    tiles: Vec<Tile>,
}

impl World {
    fn new_room(w: i32, h: i32) -> Self {
        let mut world = Self {
            w,
            h,
            tiles: vec![Tile::Floor; (w * h) as usize],
        };

        for x in 0..w {
            world.set(x, 0, Tile::Wall);
            world.set(x, h - 1, Tile::Wall);
        }

        for y in 0..h {
            world.set(0, y, Tile::Wall);
            world.set(w - 1, y, Tile::Wall);
        }

        for y in 4..11 {
            world.set(12, y, Tile::Wall);
        }

        for x in 5..10 {
            world.set(x, 8, Tile::Wall);
        }

        world
    }

    fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.w + x) as usize
    }

    fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.w && y >= 0 && y < self.h
    }

    fn get(&self, x: i32, y: i32) -> Tile {
        if !self.in_bounds(x, y) {
            return Tile::Wall;
        }
        self.tiles[self.idx(x, y)]
    }

    fn set(&mut self, x: i32, y: i32, tile: Tile) {
        if self.in_bounds(x, y) {
            let i = self.idx(x, y);
            self.tiles[i] = tile;
        }
    }

    fn is_solid(&self, x: i32, y: i32) -> bool {
        tile_is_wall(self.get(x, y))
    }

    fn tile_rect(x: i32, y: i32) -> Rect {
        Rect::new(
            x as f32 * TILE_SIZE,
            y as f32 * TILE_SIZE,
            TILE_SIZE,
            TILE_SIZE,
        )
    }
}

#[derive(Copy, Clone)]
struct Aabb {
    min: Vec2,
    max: Vec2,
}

impl Aabb {
    fn from_center(center: Vec2, half: Vec2) -> Self {
        Self {
            min: center - half,
            max: center + half,
        }
    }

    fn intersects_rect(&self, r: Rect) -> bool {
        self.min.x < r.x + r.w && self.max.x > r.x && self.min.y < r.y + r.h && self.max.y > r.y
    }
}

#[derive(Copy, Clone)]
struct Palette {
    bg_top: Color,
    bg_bottom: Color,
    haze: Color,
    floor_a: Color,
    floor_b: Color,
    floor_c: Color,
    floor_edge: Color,
    floor_panel: Color,
    floor_rivet: Color,
    floor_grime: Color,
    wall_top: Color,
    wall_mid: Color,
    wall_dark: Color,
    wall_outline: Color,
    shadow_soft: Color,
    shadow_hard: Color,
    vignette: Color,
    lamp_warm: Color,
    lamp_hot: Color,
    prop_crate_light: Color,
    prop_crate_dark: Color,
    prop_pipe: Color,
    prop_pipe_highlight: Color,
    dust: Color,
}

impl Palette {
    fn new() -> Self {
        Self {
            bg_top: rgba(12, 15, 22, 255),
            bg_bottom: rgba(25, 30, 39, 255),
            haze: rgba(140, 180, 200, 40),
            floor_a: rgba(56, 67, 76, 255),
            floor_b: rgba(62, 72, 83, 255),
            floor_c: rgba(49, 60, 69, 255),
            floor_edge: rgba(28, 34, 42, 180),
            floor_panel: rgba(83, 98, 112, 115),
            floor_rivet: rgba(116, 130, 143, 170),
            floor_grime: rgba(8, 10, 14, 255),
            wall_top: rgba(126, 142, 154, 255),
            wall_mid: rgba(88, 102, 116, 255),
            wall_dark: rgba(56, 67, 80, 255),
            wall_outline: rgba(20, 25, 31, 210),
            shadow_soft: rgba(10, 12, 18, 120),
            shadow_hard: rgba(5, 7, 10, 190),
            vignette: rgba(3, 4, 8, 255),
            lamp_warm: rgba(243, 185, 95, 255),
            lamp_hot: rgba(255, 225, 165, 255),
            prop_crate_light: rgba(148, 116, 86, 255),
            prop_crate_dark: rgba(104, 78, 58, 255),
            prop_pipe: rgba(99, 113, 126, 255),
            prop_pipe_highlight: rgba(153, 170, 188, 255),
            dust: rgba(184, 204, 216, 255),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum PropKind {
    Crate,
    Pipe,
    Lamp,
    Banner,
    Plant,
    Bench,
    Crystal,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
struct Prop {
    tile_x: i32,
    tile_y: i32,
    kind: PropKind,
    phase: f32,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum ControlMode {
    Manual,
    AutoMove,
}

#[derive(Default)]
struct AutoMoveState {
    target_tile: Option<(i32, i32)>,
    target_world: Option<Vec2>,
    path_tiles: Vec<(i32, i32)>,
    path_world: Vec<Vec2>,
    next_waypoint: usize,
}

struct NpcWanderer {
    pos: Vec2,
    half: Vec2,
    speed: f32,
    facing: CharacterFacing,
    facing_left: bool,
    velocity: Vec2,
    is_walking: bool,
    anim_frame: usize,
    walk_cycle: f32,
    auto: AutoMoveState,
    idle_timer: f32,
    rng_state: u64,
    bubble_timer: f32,
    bubble_cooldown: f32,
}

impl NpcWanderer {
    fn new(pos: Vec2, seed: u64) -> Self {
        Self {
            pos,
            half: vec2(9.0, 13.0),
            speed: NPC_WANDER_SPEED,
            facing: CharacterFacing::Front,
            facing_left: false,
            velocity: Vec2::ZERO,
            is_walking: false,
            anim_frame: 0,
            walk_cycle: 0.0,
            auto: AutoMoveState::default(),
            idle_timer: 1.0,
            rng_state: seed ^ 0xC0FF_EE11_D00D_CAFE,
            bubble_timer: 0.0,
            bubble_cooldown: 0.8,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct OpenNode {
    f: i32,
    g: i32,
    idx: usize,
}

impl Ord for OpenNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f
            .cmp(&self.f)
            .then_with(|| other.g.cmp(&self.g))
            .then_with(|| other.idx.cmp(&self.idx))
    }
}

impl PartialOrd for OpenNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Player {
    pos: Vec2,
    half: Vec2,
    speed: f32,
    control_mode: ControlMode,
    facing: CharacterFacing,
    facing_left: bool,
    velocity: Vec2,
    is_walking: bool,
    anim_frame: usize,
    walk_cycle: f32,
    auto: AutoMoveState,
}

impl Player {
    fn new(pos: Vec2) -> Self {
        Self {
            pos,
            half: vec2(10.0, 14.0),
            speed: 140.0,
            control_mode: ControlMode::Manual,
            facing: CharacterFacing::Front,
            facing_left: false,
            velocity: Vec2::ZERO,
            is_walking: false,
            anim_frame: 0,
            walk_cycle: 0.0,
            auto: AutoMoveState::default(),
        }
    }
}

struct GameState {
    world: World,
    player: Player,
    npc: NpcWanderer,
    camera_center: Vec2,
    camera_zoom: f32,
    palette: Palette,
    sim: sim::FactorySim,
    props: Vec<Prop>,
    character_catalog: CharacterCatalog,
    lineage_seed: u64,
    lineage: Vec<CharacterRecord>,
    player_lineage_index: usize,
    npc_character: CharacterRecord,
    show_character_inspector: bool,
    debug: bool,
    last_input: Vec2,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum AppMode {
    MainMenu,
    Playing,
    Editor,
}

#[derive(Clone, Serialize, Deserialize)]
struct MapAsset {
    version: u32,
    label: String,
    world: World,
    props: Vec<Prop>,
    player_spawn: (i32, i32),
    npc_spawn: (i32, i32),
}

impl MapAsset {
    fn new_default() -> Self {
        let world = generate_starter_factory_world(MAP_W, MAP_H);
        let props = default_props(&world);
        let player_spawn = nearest_walkable_tile(&world, (8, MAP_H / 2)).unwrap_or((2, 2));
        let npc_spawn = nearest_walkable_tile(&world, (MAP_W - 10, MAP_H / 2))
            .unwrap_or((MAP_W - 4, MAP_H / 2));

        Self {
            version: MAP_FILE_VERSION,
            label: "Starter Factory".to_string(),
            world,
            props,
            player_spawn,
            npc_spawn,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum EditorBrush {
    Floor,
    FloorMetal,
    FloorWood,
    FloorMoss,
    FloorSand,
    Wall,
    WallBrick,
    WallSteel,
    WallNeon,
    Crate,
    Pipe,
    Lamp,
    Banner,
    Plant,
    Bench,
    Crystal,
    EraseProp,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum EditorTool {
    Brush,
    Rect,
}

#[derive(Clone)]
struct EditorSnapshot {
    world: World,
    props: Vec<Prop>,
    player_spawn: (i32, i32),
    npc_spawn: (i32, i32),
}

struct EditorState {
    brush: EditorBrush,
    tool: EditorTool,
    hover_tile: Option<(i32, i32)>,
    drag_start: Option<(i32, i32)>,
    show_grid: bool,
    camera_center: Vec2,
    camera_zoom: f32,
    camera_initialized: bool,
    status_text: String,
    status_timer: f32,
    undo_stack: Vec<EditorSnapshot>,
    redo_stack: Vec<EditorSnapshot>,
    stroke_active: bool,
    stroke_changed: bool,
}

impl EditorState {
    fn new() -> Self {
        Self {
            brush: EditorBrush::Wall,
            tool: EditorTool::Brush,
            hover_tile: None,
            drag_start: None,
            show_grid: true,
            camera_center: Vec2::ZERO,
            camera_zoom: 1.05,
            camera_initialized: false,
            status_text: "Editeur pret".to_string(),
            status_timer: 0.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            stroke_active: false,
            stroke_changed: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum EditorAction {
    None,
    StartPlay,
    BackToMenu,
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

fn with_alpha(mut c: Color, alpha: f32) -> Color {
    c.a = alpha.clamp(0.0, 1.0);
    c
}

fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    let k = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * k,
        a.g + (b.g - a.g) * k,
        a.b + (b.b - a.b) * k,
        a.a + (b.a - a.a) * k,
    )
}

fn tile_is_wall(tile: Tile) -> bool {
    matches!(
        tile,
        Tile::Wall | Tile::WallBrick | Tile::WallSteel | Tile::WallNeon
    )
}

fn tile_is_floor(tile: Tile) -> bool {
    !tile_is_wall(tile)
}

fn tile_label(tile: Tile) -> &'static str {
    match tile {
        Tile::Floor => "sol",
        Tile::FloorMetal => "sol_metal",
        Tile::FloorWood => "sol_bois",
        Tile::FloorMoss => "sol_mousse",
        Tile::FloorSand => "sol_sable",
        Tile::Wall => "mur",
        Tile::WallBrick => "mur_brique",
        Tile::WallSteel => "mur_acier",
        Tile::WallNeon => "mur_neon",
    }
}

fn tile_hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32)
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add((y as u32).wrapping_mul(0x85EB_CA6B));
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^ (h >> 16)
}

fn hash_with_salt(x: i32, y: i32, salt: u32) -> u32 {
    let sx = x.wrapping_add((salt as i32).wrapping_mul(31));
    let sy = y.wrapping_sub((salt as i32).wrapping_mul(17));
    tile_hash(sx, sy) ^ salt.wrapping_mul(0x27D4_EB2D)
}

fn clamp_i32(v: i32, lo: i32, hi: i32) -> i32 {
    v.max(lo).min(hi)
}

fn tiles_overlapping_aabb(world: &World, aabb: Aabb) -> (i32, i32, i32, i32) {
    let eps = 0.0001;

    let min_tx = ((aabb.min.x + eps) / TILE_SIZE).floor() as i32;
    let max_tx = ((aabb.max.x - eps) / TILE_SIZE).floor() as i32;
    let min_ty = ((aabb.min.y + eps) / TILE_SIZE).floor() as i32;
    let max_ty = ((aabb.max.y - eps) / TILE_SIZE).floor() as i32;

    (
        clamp_i32(min_tx, 0, world.w - 1),
        clamp_i32(max_tx, 0, world.w - 1),
        clamp_i32(min_ty, 0, world.h - 1),
        clamp_i32(max_ty, 0, world.h - 1),
    )
}

fn generate_starter_factory_world(w: i32, h: i32) -> World {
    let mut world = World::new_room(w, h);

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let tile = if x < w / 3 {
                if y < h / 2 {
                    Tile::FloorWood
                } else {
                    Tile::FloorMoss
                }
            } else if x > (w * 2) / 3 {
                if y < h / 2 {
                    Tile::Floor
                } else {
                    Tile::FloorSand
                }
            } else {
                Tile::FloorMetal
            };
            world.set(x, y, tile);
        }
    }

    let vertical_walls = [w / 3, (w * 2) / 3];
    for &vx in &vertical_walls {
        for y in 1..h - 1 {
            world.set(vx, y, Tile::WallSteel);
        }
        for &door_y in &[h / 6, h / 2, (h * 5) / 6] {
            for dy in -1..=1 {
                let y = clamp_i32(door_y + dy, 1, h - 2);
                world.set(vx, y, Tile::FloorMetal);
            }
        }
    }

    let horizontal_walls = [h / 3, (h * 2) / 3];
    for &hy in &horizontal_walls {
        for x in 1..w - 1 {
            world.set(x, hy, Tile::WallBrick);
        }
        for &door_x in &[w / 6, w / 2, (w * 5) / 6] {
            for dx in -1..=1 {
                let x = clamp_i32(door_x + dx, 1, w - 2);
                world.set(x, hy, Tile::FloorMetal);
            }
        }
    }

    let core_min_x = (w / 2) - 9;
    let core_max_x = (w / 2) + 9;
    let core_min_y = (h / 2) - 6;
    let core_max_y = (h / 2) + 6;

    for x in core_min_x..=core_max_x {
        world.set(x, core_min_y, Tile::WallNeon);
        world.set(x, core_max_y, Tile::WallNeon);
    }
    for y in core_min_y..=core_max_y {
        world.set(core_min_x, y, Tile::WallNeon);
        world.set(core_max_x, y, Tile::WallNeon);
    }
    for x in (w / 2) - 1..=(w / 2) + 1 {
        world.set(x, core_min_y, Tile::FloorWood);
        world.set(x, core_max_y, Tile::FloorWood);
    }
    for y in (h / 2) - 1..=(h / 2) + 1 {
        world.set(core_min_x, y, Tile::FloorWood);
        world.set(core_max_x, y, Tile::FloorWood);
    }
    for y in core_min_y + 1..core_max_y {
        for x in core_min_x + 1..core_max_x {
            if !world.is_solid(x, y) {
                world.set(x, y, Tile::FloorWood);
            }
        }
    }

    enforce_world_border(&mut world);
    world
}

fn default_props(world: &World) -> Vec<Prop> {
    let mut props = Vec::new();

    for y in (3..world.h - 3).step_by(5) {
        for x in (3..world.w - 3).step_by(6) {
            if world.is_solid(x, y) {
                continue;
            }
            let h = hash_with_salt(x, y, 0xA3) % 19;
            if h > 14 {
                continue;
            }
            let kind = match h {
                0..=3 => PropKind::Crate,
                4..=6 => PropKind::Pipe,
                7..=8 => PropKind::Lamp,
                9..=10 => PropKind::Banner,
                11..=12 => PropKind::Plant,
                13 => PropKind::Bench,
                _ => PropKind::Crystal,
            };
            props.push(Prop {
                tile_x: x,
                tile_y: y,
                kind,
                phase: prop_phase_for_tile((x, y)),
            });
        }
    }

    let hero_spots = [
        (world.w / 2, world.h / 2, PropKind::Bench),
        (world.w / 2 - 2, world.h / 2, PropKind::Lamp),
        (world.w / 2 + 2, world.h / 2, PropKind::Lamp),
    ];
    for (x, y, kind) in hero_spots {
        if world.in_bounds(x, y)
            && !world.is_solid(x, y)
            && prop_index_at_tile(&props, (x, y)).is_none()
        {
            props.push(Prop {
                tile_x: x,
                tile_y: y,
                kind,
                phase: prop_phase_for_tile((x, y)),
            });
        }
    }

    props
}

fn apply_material_variation(world: &mut World) {
    for y in 1..world.h - 1 {
        for x in 1..world.w - 1 {
            let tile = world.get(x, y);
            if tile_is_wall(tile) {
                let h = tile_hash(x, y) % 11;
                let wall = if h == 0 {
                    Tile::WallNeon
                } else if h <= 3 {
                    Tile::WallBrick
                } else if h <= 6 {
                    Tile::WallSteel
                } else {
                    Tile::Wall
                };
                world.set(x, y, wall);
            } else {
                let h = tile_hash(x, y) % 16;
                let floor = if h == 0 {
                    Tile::FloorSand
                } else if h <= 3 {
                    Tile::FloorMoss
                } else if h <= 6 {
                    Tile::FloorWood
                } else if h <= 9 {
                    Tile::FloorMetal
                } else {
                    Tile::Floor
                };
                world.set(x, y, floor);
            }
        }
    }
}

fn advance_seed(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

fn handle_fullscreen_hotkey(is_fullscreen_mode: &mut bool) -> bool {
    let alt_held = is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt);
    let toggle_requested =
        is_key_pressed(KeyCode::F11) || (alt_held && is_key_pressed(KeyCode::Enter));
    if toggle_requested {
        *is_fullscreen_mode = !*is_fullscreen_mode;
        macroquad::window::set_fullscreen(*is_fullscreen_mode);
        return true;
    }
    false
}

fn point_in_rect(point: Vec2, rect: Rect) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.w
        && point.y >= rect.y
        && point.y <= rect.y + rect.h
}

fn fit_world_camera_to_screen(world: &World, margin: f32) -> (Camera2D, Rect) {
    let sw = screen_width();
    let sh = screen_height();

    let world_size_px = vec2(world.w as f32 * TILE_SIZE, world.h as f32 * TILE_SIZE);
    let avail_w = (sw - margin * 2.0).max(1.0);
    let avail_h = (sh - margin * 2.0).max(1.0);
    let scale = (avail_w / world_size_px.x)
        .min(avail_h / world_size_px.y)
        .max(0.01);

    let view_w = (world_size_px.x * scale).max(1.0);
    let view_h = (world_size_px.y * scale).max(1.0);
    let view_rect = Rect::new((sw - view_w) * 0.5, (sh - view_h) * 0.5, view_w, view_h);

    let mut cam =
        Camera2D::from_display_rect(Rect::new(0.0, 0.0, world_size_px.x, world_size_px.y));
    // Keep world coordinates in the same orientation as the rest of the game (Y grows downward).
    cam.zoom.y = cam.zoom.y.abs();
    cam.viewport = Some((
        view_rect.x.round() as i32,
        (sh - view_rect.y - view_rect.h).round().max(0.0) as i32,
        view_rect.w.round().max(1.0) as i32,
        view_rect.h.round().max(1.0) as i32,
    ));

    (cam, view_rect)
}

fn build_world_camera_for_viewport(
    world: &World,
    center: Vec2,
    zoom: f32,
    view_rect: Rect,
    zoom_min: f32,
    zoom_max: f32,
) -> (Camera2D, Vec2, f32) {
    let world_w = (world.w as f32 * TILE_SIZE).max(1.0);
    let world_h = (world.h as f32 * TILE_SIZE).max(1.0);
    let zoom = zoom.clamp(zoom_min, zoom_max);
    let camera_w = (view_rect.w / zoom).clamp(1.0, world_w);
    let camera_h = (view_rect.h / zoom).clamp(1.0, world_h);

    let mut clamped_center = center;
    if world_w <= camera_w {
        clamped_center.x = world_w * 0.5;
    } else {
        clamped_center.x = clamped_center
            .x
            .clamp(camera_w * 0.5, world_w - camera_w * 0.5);
    }
    if world_h <= camera_h {
        clamped_center.y = world_h * 0.5;
    } else {
        clamped_center.y = clamped_center
            .y
            .clamp(camera_h * 0.5, world_h - camera_h * 0.5);
    }

    let display_rect = Rect::new(
        clamped_center.x - camera_w * 0.5,
        clamped_center.y - camera_h * 0.5,
        camera_w,
        camera_h,
    );
    let mut camera = Camera2D::from_display_rect(display_rect);
    camera.zoom.y = camera.zoom.y.abs();
    let sh = screen_height();
    camera.viewport = Some((
        view_rect.x.round() as i32,
        (sh - view_rect.y - view_rect.h).round().max(0.0) as i32,
        view_rect.w.round().max(1.0) as i32,
        view_rect.h.round().max(1.0) as i32,
    ));

    (camera, clamped_center, zoom)
}

fn build_pannable_world_camera(
    world: &World,
    center: Vec2,
    zoom: f32,
    margin: f32,
) -> (Camera2D, Rect, Vec2) {
    let sw = screen_width();
    let sh = screen_height();
    let view_rect = Rect::new(
        margin,
        margin,
        (sw - margin * 2.0).max(1.0),
        (sh - margin * 2.0).max(1.0),
    );
    let (camera, clamped_center, _) = build_world_camera_for_viewport(
        world,
        center,
        zoom,
        view_rect,
        PLAY_CAMERA_ZOOM_MIN,
        PLAY_CAMERA_ZOOM_MAX,
    );

    (camera, view_rect, clamped_center)
}

fn tile_bounds_from_camera(
    world: &World,
    camera: &Camera2D,
    view_rect: Rect,
    padding_tiles: i32,
) -> (i32, i32, i32, i32) {
    if world.w <= 0 || world.h <= 0 {
        return (0, 0, 0, 0);
    }

    let top_left = camera.screen_to_world(vec2(view_rect.x, view_rect.y));
    let bottom_right =
        camera.screen_to_world(vec2(view_rect.x + view_rect.w, view_rect.y + view_rect.h));
    let min_world_x = top_left.x.min(bottom_right.x);
    let max_world_x = top_left.x.max(bottom_right.x);
    let min_world_y = top_left.y.min(bottom_right.y);
    let max_world_y = top_left.y.max(bottom_right.y);

    let mut min_x = (min_world_x / TILE_SIZE).floor() as i32 - padding_tiles;
    let mut max_x = (max_world_x / TILE_SIZE).floor() as i32 + padding_tiles;
    let mut min_y = (min_world_y / TILE_SIZE).floor() as i32 - padding_tiles;
    let mut max_y = (max_world_y / TILE_SIZE).floor() as i32 + padding_tiles;

    min_x = clamp_i32(min_x, 0, world.w - 1);
    max_x = clamp_i32(max_x, 0, world.w - 1);
    min_y = clamp_i32(min_y, 0, world.h - 1);
    max_y = clamp_i32(max_y, 0, world.h - 1);
    (min_x, max_x, min_y, max_y)
}

fn tile_in_bounds(tile: (i32, i32), bounds: (i32, i32, i32, i32)) -> bool {
    tile.0 >= bounds.0 && tile.0 <= bounds.1 && tile.1 >= bounds.2 && tile.1 <= bounds.3
}

fn draw_ui_button(
    rect: Rect,
    label: &str,
    mouse_pos: Vec2,
    mouse_pressed: bool,
    active: bool,
) -> bool {
    draw_ui_button_sized(rect, label, mouse_pos, mouse_pressed, active, 22.0)
}

fn draw_ui_button_sized(
    rect: Rect,
    label: &str,
    mouse_pos: Vec2,
    mouse_pressed: bool,
    active: bool,
    font_size: f32,
) -> bool {
    let hovered = point_in_rect(mouse_pos, rect);
    let base = if active {
        rgba(210, 150, 82, 242)
    } else if hovered {
        rgba(98, 152, 188, 240)
    } else {
        rgba(68, 100, 128, 236)
    };
    let top_highlight = if active {
        with_alpha(rgba(255, 240, 210, 255), 0.35)
    } else if hovered {
        with_alpha(rgba(222, 244, 255, 255), 0.28)
    } else {
        with_alpha(rgba(194, 226, 246, 255), 0.20)
    };
    let border = if active {
        rgba(252, 208, 138, 252)
    } else if hovered {
        rgba(170, 220, 247, 240)
    } else {
        rgba(120, 171, 199, 224)
    };
    draw_rectangle(
        rect.x + 2.0,
        rect.y + 3.0,
        rect.w,
        rect.h,
        with_alpha(BLACK, if hovered || active { 0.30 } else { 0.24 }),
    );
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, base);
    draw_rectangle(
        rect.x + 1.5,
        rect.y + 1.5,
        (rect.w - 3.0).max(1.0),
        (rect.h * 0.44).max(1.0),
        top_highlight,
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, border);
    if hovered || active {
        draw_rectangle_lines(
            rect.x + 1.0,
            rect.y + 1.0,
            (rect.w - 2.0).max(1.0),
            (rect.h - 2.0).max(1.0),
            1.0,
            with_alpha(WHITE, 0.25),
        );
    }

    let dims = measure_text(label, None, font_size as u16, 1.0);
    let text_x = rect.x + rect.w * 0.5 - dims.width * 0.5;
    let text_y = rect.y + rect.h * 0.5 + dims.height * 0.32;
    let text_fill = if active {
        Color::from_rgba(255, 248, 232, 255)
    } else {
        Color::from_rgba(244, 252, 255, 255)
    };
    let shadow = if active {
        with_alpha(Color::from_rgba(56, 36, 18, 255), 0.82)
    } else {
        with_alpha(Color::from_rgba(8, 12, 18, 255), 0.82)
    };
    draw_text(label, text_x + 1.0, text_y + 1.0, font_size, shadow);
    draw_text(label, text_x, text_y, font_size, text_fill);

    hovered && mouse_pressed
}

fn is_border_tile(world: &World, tile: (i32, i32)) -> bool {
    tile.0 <= 0 || tile.1 <= 0 || tile.0 >= world.w - 1 || tile.1 >= world.h - 1
}

fn prop_kind_label(kind: PropKind) -> &'static str {
    match kind {
        PropKind::Crate => "caisse",
        PropKind::Pipe => "tuyau",
        PropKind::Lamp => "lampe",
        PropKind::Banner => "banniere",
        PropKind::Plant => "plante",
        PropKind::Bench => "banc",
        PropKind::Crystal => "cristal",
    }
}

fn editor_brush_label(brush: EditorBrush) -> &'static str {
    match brush {
        EditorBrush::Floor => "Sol",
        EditorBrush::FloorMetal => "Sol metal",
        EditorBrush::FloorWood => "Sol bois",
        EditorBrush::FloorMoss => "Sol mousse",
        EditorBrush::FloorSand => "Sol sable",
        EditorBrush::Wall => "Mur",
        EditorBrush::WallBrick => "Mur brique",
        EditorBrush::WallSteel => "Mur acier",
        EditorBrush::WallNeon => "Mur neon",
        EditorBrush::Crate => "Caisse",
        EditorBrush::Pipe => "Tuyau",
        EditorBrush::Lamp => "Lampe",
        EditorBrush::Banner => "Banniere",
        EditorBrush::Plant => "Plante",
        EditorBrush::Bench => "Banc",
        EditorBrush::Crystal => "Cristal",
        EditorBrush::EraseProp => "Effacer objet",
    }
}

fn editor_tool_label(tool: EditorTool) -> &'static str {
    match tool {
        EditorTool::Brush => "Pinceau",
        EditorTool::Rect => "Rectangle",
    }
}

fn editor_capture_snapshot(map: &MapAsset) -> EditorSnapshot {
    EditorSnapshot {
        world: map.world.clone(),
        props: map.props.clone(),
        player_spawn: map.player_spawn,
        npc_spawn: map.npc_spawn,
    }
}

fn editor_apply_snapshot(map: &mut MapAsset, snapshot: EditorSnapshot) {
    map.world = snapshot.world;
    map.props = snapshot.props;
    map.player_spawn = snapshot.player_spawn;
    map.npc_spawn = snapshot.npc_spawn;
}

fn editor_push_undo(editor: &mut EditorState, map: &MapAsset) {
    editor.undo_stack.push(editor_capture_snapshot(map));
    if editor.undo_stack.len() > EDITOR_UNDO_LIMIT {
        let overflow = editor.undo_stack.len() - EDITOR_UNDO_LIMIT;
        editor.undo_stack.drain(0..overflow);
    }
}

fn editor_set_status(editor: &mut EditorState, message: impl Into<String>) {
    editor.status_text = message.into();
    editor.status_timer = 3.4;
}

fn editor_reset_stroke_state(editor: &mut EditorState) {
    editor.stroke_active = false;
    editor.stroke_changed = false;
    editor.drag_start = None;
}

fn editor_save_current_map(editor: &mut EditorState, map: &mut MapAsset) {
    sanitize_map_asset(map);
    match save_map_asset(MAP_FILE_PATH, map) {
        Ok(()) => editor_set_status(editor, format!("Carte sauvegardee: {}", MAP_FILE_PATH)),
        Err(err) => editor_set_status(editor, err),
    }
}

fn editor_load_current_map(editor: &mut EditorState, map: &mut MapAsset) {
    match load_map_asset(MAP_FILE_PATH) {
        Ok(loaded) => {
            *map = loaded;
            editor.undo_stack.clear();
            editor.redo_stack.clear();
            editor_reset_stroke_state(editor);
            editor.camera_initialized = false;
            editor_set_status(editor, format!("Carte chargee: {}", MAP_FILE_PATH));
        }
        Err(err) => editor_set_status(editor, err),
    }
}

fn editor_undo(editor: &mut EditorState, map: &mut MapAsset) -> bool {
    let Some(snapshot) = editor.undo_stack.pop() else {
        editor_set_status(editor, "Annulation vide");
        return false;
    };
    editor.redo_stack.push(editor_capture_snapshot(map));
    editor_apply_snapshot(map, snapshot);
    editor_set_status(editor, "Annulation appliquee");
    true
}

fn editor_redo(editor: &mut EditorState, map: &mut MapAsset) -> bool {
    let Some(snapshot) = editor.redo_stack.pop() else {
        editor_set_status(editor, "Retablissement vide");
        return false;
    };
    editor.undo_stack.push(editor_capture_snapshot(map));
    editor_apply_snapshot(map, snapshot);
    editor_set_status(editor, "Retablissement applique");
    true
}

fn enforce_world_border(world: &mut World) {
    for x in 0..world.w {
        world.set(x, 0, Tile::Wall);
        world.set(x, world.h - 1, Tile::Wall);
    }
    for y in 0..world.h {
        world.set(0, y, Tile::Wall);
        world.set(world.w - 1, y, Tile::Wall);
    }
}

fn prop_index_at_tile(props: &[Prop], tile: (i32, i32)) -> Option<usize> {
    props
        .iter()
        .position(|prop| prop.tile_x == tile.0 && prop.tile_y == tile.1)
}

fn prop_phase_for_tile(tile: (i32, i32)) -> f32 {
    let h = tile_hash(tile.0, tile.1) & 0xFF;
    (h as f32 / 255.0) * std::f32::consts::TAU
}

fn remove_prop_at_tile(map: &mut MapAsset, tile: (i32, i32)) -> bool {
    let Some(idx) = prop_index_at_tile(&map.props, tile) else {
        return false;
    };
    map.props.swap_remove(idx);
    true
}

fn set_prop_at_tile(map: &mut MapAsset, tile: (i32, i32), kind: PropKind) -> bool {
    if !map.world.in_bounds(tile.0, tile.1) || map.world.is_solid(tile.0, tile.1) {
        return false;
    }
    if let Some(idx) = prop_index_at_tile(&map.props, tile) {
        if map.props[idx].kind == kind {
            return false;
        }
        map.props[idx].kind = kind;
        map.props[idx].phase = prop_phase_for_tile(tile);
        return true;
    }

    map.props.push(Prop {
        tile_x: tile.0,
        tile_y: tile.1,
        kind,
        phase: prop_phase_for_tile(tile),
    });
    true
}

fn set_map_tile(map: &mut MapAsset, tile: (i32, i32), tile_kind: Tile) -> bool {
    if !map.world.in_bounds(tile.0, tile.1) {
        return false;
    }
    if is_border_tile(&map.world, tile) && !tile_is_wall(tile_kind) {
        return false;
    }
    if map.world.get(tile.0, tile.1) == tile_kind {
        return false;
    }

    map.world.set(tile.0, tile.1, tile_kind);
    if tile_is_wall(tile_kind) {
        let _ = remove_prop_at_tile(map, tile);
    }
    true
}

fn editor_apply_brush(map: &mut MapAsset, brush: EditorBrush, tile: (i32, i32)) -> bool {
    match brush {
        EditorBrush::Floor => set_map_tile(map, tile, Tile::Floor),
        EditorBrush::FloorMetal => set_map_tile(map, tile, Tile::FloorMetal),
        EditorBrush::FloorWood => set_map_tile(map, tile, Tile::FloorWood),
        EditorBrush::FloorMoss => set_map_tile(map, tile, Tile::FloorMoss),
        EditorBrush::FloorSand => set_map_tile(map, tile, Tile::FloorSand),
        EditorBrush::Wall => set_map_tile(map, tile, Tile::Wall),
        EditorBrush::WallBrick => set_map_tile(map, tile, Tile::WallBrick),
        EditorBrush::WallSteel => set_map_tile(map, tile, Tile::WallSteel),
        EditorBrush::WallNeon => set_map_tile(map, tile, Tile::WallNeon),
        EditorBrush::Crate => set_prop_at_tile(map, tile, PropKind::Crate),
        EditorBrush::Pipe => set_prop_at_tile(map, tile, PropKind::Pipe),
        EditorBrush::Lamp => set_prop_at_tile(map, tile, PropKind::Lamp),
        EditorBrush::Banner => set_prop_at_tile(map, tile, PropKind::Banner),
        EditorBrush::Plant => set_prop_at_tile(map, tile, PropKind::Plant),
        EditorBrush::Bench => set_prop_at_tile(map, tile, PropKind::Bench),
        EditorBrush::Crystal => set_prop_at_tile(map, tile, PropKind::Crystal),
        EditorBrush::EraseProp => remove_prop_at_tile(map, tile),
    }
}

fn editor_apply_brush_rect(
    map: &mut MapAsset,
    brush: EditorBrush,
    start: (i32, i32),
    end: (i32, i32),
) -> bool {
    let min_x = start.0.min(end.0);
    let max_x = start.0.max(end.0);
    let min_y = start.1.min(end.1);
    let max_y = start.1.max(end.1);
    let mut changed = false;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            changed |= editor_apply_brush(map, brush, (x, y));
        }
    }
    changed
}

fn sanitize_map_asset(map: &mut MapAsset) {
    let needs_material_upgrade = map.version < MAP_FILE_VERSION;
    let needs_layout_upgrade =
        map.version < MAP_FILE_VERSION && (map.world.w < MAP_W || map.world.h < MAP_H);
    if needs_layout_upgrade {
        *map = MapAsset::new_default();
        return;
    }
    if map.world.w < 4 || map.world.h < 4 {
        map.world = generate_starter_factory_world(MAP_W, MAP_H);
    }

    let required = (map.world.w * map.world.h) as usize;
    if map.world.tiles.len() != required {
        map.world.tiles = vec![Tile::Floor; required];
    }

    if needs_material_upgrade {
        apply_material_variation(&mut map.world);
    }
    map.version = MAP_FILE_VERSION;

    enforce_world_border(&mut map.world);

    let mut occupied = HashSet::new();
    map.props.retain(|prop| {
        if !map.world.in_bounds(prop.tile_x, prop.tile_y) {
            return false;
        }
        if map.world.is_solid(prop.tile_x, prop.tile_y) {
            return false;
        }
        occupied.insert((prop.tile_x, prop.tile_y))
    });

    map.player_spawn = nearest_walkable_tile(&map.world, map.player_spawn).unwrap_or((2, 2));
    map.npc_spawn = nearest_walkable_tile(&map.world, map.npc_spawn)
        .unwrap_or((map.world.w - 3, map.world.h / 2));
}

fn serialize_map_asset(map: &MapAsset) -> Result<String, String> {
    let pretty = PrettyConfig::new()
        .depth_limit(4)
        .enumerate_arrays(true)
        .separate_tuple_members(true);
    ron_to_string_pretty(map, pretty).map_err(|err| format!("serialize map failed: {err}"))
}

fn deserialize_map_asset(raw: &str) -> Result<MapAsset, String> {
    let mut map: MapAsset = ron_from_str(raw).map_err(|err| format!("parse map failed: {err}"))?;
    sanitize_map_asset(&mut map);
    Ok(map)
}

fn save_map_asset(path: &str, map: &MapAsset) -> Result<(), String> {
    let payload = serialize_map_asset(map)?;
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|err| format!("create map dir failed: {err}"))?;
    }
    fs::write(path, payload).map_err(|err| format!("write map failed: {err}"))
}

fn load_map_asset(path: &str) -> Result<MapAsset, String> {
    let raw = fs::read_to_string(path).map_err(|err| format!("read map failed: {err}"))?;
    deserialize_map_asset(&raw)
}

fn build_game_state_from_map(
    map: &MapAsset,
    character_catalog: &CharacterCatalog,
    lineage_seed: u64,
) -> GameState {
    let mut map_copy = map.clone();
    sanitize_map_asset(&mut map_copy);
    let player = Player::new(tile_center(map_copy.player_spawn));
    let npc = NpcWanderer::new(tile_center(map_copy.npc_spawn), 0x9922_11AA_77CC_44DD);
    let palette = Palette::new();
    let lineage = build_lineage_preview(character_catalog, lineage_seed);
    let npc_character =
        character_catalog.spawn_founder("Wanderer", lineage_seed ^ 0x55AA_7788_1133_2244);
    let sim = sim::FactorySim::load_or_default(SIM_CONFIG_PATH, map_copy.world.w, map_copy.world.h);

    GameState {
        world: map_copy.world,
        player,
        npc,
        camera_center: tile_center(map_copy.player_spawn),
        camera_zoom: 1.15,
        palette,
        sim,
        props: map_copy.props,
        character_catalog: character_catalog.clone(),
        lineage_seed,
        lineage,
        player_lineage_index: 2,
        npc_character,
        show_character_inspector: true,
        debug: true,
        last_input: Vec2::ZERO,
    }
}

fn draw_character_inspector_panel(state: &GameState, time: f32) {
    let panel_w = 380.0;
    let panel_h = 204.0;
    let panel_x = screen_width() - panel_w - 10.0;
    let panel_y = 10.0;

    draw_rectangle(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        Color::from_rgba(12, 14, 20, 210),
    );
    draw_rectangle_lines(
        panel_x + 0.5,
        panel_y + 0.5,
        panel_w - 1.0,
        panel_h - 1.0,
        1.0,
        Color::from_rgba(90, 120, 140, 220),
    );

    draw_text(
        "Character Inspector (F2 toggle, F3 regenerate)",
        panel_x + 10.0,
        panel_y + 18.0,
        18.0,
        WHITE,
    );

    for (i, record) in state.lineage.iter().take(5).enumerate() {
        let col = (i % 3) as f32;
        let row = (i / 3) as f32;
        let px = panel_x + 28.0 + col * 122.0;
        let py = panel_y + 48.0 + row * 84.0;

        draw_character(
            record,
            CharacterRenderParams {
                center: vec2(px, py),
                scale: 0.9,
                facing: CharacterFacing::Front,
                facing_left: false,
                is_walking: i == state.player_lineage_index,
                walk_cycle: time * 6.5 + i as f32 * 0.8,
                time,
                debug: false,
            },
        );

        let title = format!("{} g{}", record.label, record.generation);
        draw_text(&title, px - 22.0, py + 26.0, 14.0, WHITE);
        let summary = compact_visual_summary(record);
        draw_text(
            &summary,
            px - 38.0,
            py + 40.0,
            12.0,
            Color::from_rgba(170, 210, 220, 255),
        );
    }

    if let Some(player_record) = state.lineage.get(state.player_lineage_index) {
        let lines = inspector_lines(player_record);
        for (i, line) in lines.iter().take(8).enumerate() {
            draw_text(
                line,
                panel_x + 10.0,
                panel_y + 120.0 + i as f32 * 11.0,
                11.0,
                Color::from_rgba(190, 210, 220, 255),
            );
        }
    }
}

fn read_input_dir() -> Vec2 {
    let mut dir = Vec2::ZERO;

    if is_key_down(KeyCode::Up) {
        dir.y -= 1.0;
    }
    if is_key_down(KeyCode::Down) {
        dir.y += 1.0;
    }
    if is_key_down(KeyCode::Left) {
        dir.x -= 1.0;
    }
    if is_key_down(KeyCode::Right) {
        dir.x += 1.0;
    }

    if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec2::ZERO
    }
}

fn read_camera_pan_input() -> Vec2 {
    let mut dir = Vec2::ZERO;

    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Z) {
        dir.y -= 1.0;
    }
    if is_key_down(KeyCode::S) {
        dir.y += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Q) {
        dir.x -= 1.0;
    }
    if is_key_down(KeyCode::D) {
        dir.x += 1.0;
    }

    if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec2::ZERO
    }
}

fn read_editor_pan_input(space_held: bool) -> Vec2 {
    let mut dir = Vec2::ZERO;

    if is_key_down(KeyCode::Up) {
        dir.y -= 1.0;
    }
    if is_key_down(KeyCode::Down) {
        dir.y += 1.0;
    }
    if is_key_down(KeyCode::Left) {
        dir.x -= 1.0;
    }
    if is_key_down(KeyCode::Right) {
        dir.x += 1.0;
    }
    if space_held {
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Z) {
            dir.y -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            dir.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Q) {
            dir.x -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            dir.x += 1.0;
        }
    }

    if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec2::ZERO
    }
}

fn select_character_facing(input: Vec2, current: CharacterFacing) -> CharacterFacing {
    if input.length_squared() <= 0.0001 {
        return current;
    }

    let ax = input.x.abs();
    let ay = input.y.abs();

    if ay > ax + DIRECTION_HYSTERESIS {
        if input.y < 0.0 {
            CharacterFacing::Back
        } else {
            CharacterFacing::Front
        }
    } else if ax > ay + DIRECTION_HYSTERESIS {
        CharacterFacing::Side
    } else {
        match current {
            CharacterFacing::Side => CharacterFacing::Side,
            CharacterFacing::Back if input.y <= 0.0 => CharacterFacing::Back,
            CharacterFacing::Front if input.y >= 0.0 => CharacterFacing::Front,
            _ => {
                if input.y < 0.0 {
                    CharacterFacing::Back
                } else {
                    CharacterFacing::Front
                }
            }
        }
    }
}

fn facing_label(facing: CharacterFacing) -> &'static str {
    match facing {
        CharacterFacing::Front => "front",
        CharacterFacing::Side => "side",
        CharacterFacing::Back => "back",
    }
}

fn control_mode_label(mode: ControlMode) -> &'static str {
    match mode {
        ControlMode::Manual => "manual",
        ControlMode::AutoMove => "auto_move",
    }
}

fn tile_from_world_clamped(world: &World, pos: Vec2) -> (i32, i32) {
    let tx = clamp_i32((pos.x / TILE_SIZE).floor() as i32, 0, world.w - 1);
    let ty = clamp_i32((pos.y / TILE_SIZE).floor() as i32, 0, world.h - 1);
    (tx, ty)
}

fn tile_center(tile: (i32, i32)) -> Vec2 {
    vec2(
        (tile.0 as f32 + 0.5) * TILE_SIZE,
        (tile.1 as f32 + 0.5) * TILE_SIZE,
    )
}

fn idx_to_tile(world: &World, idx: usize) -> (i32, i32) {
    let idx_i32 = idx as i32;
    (idx_i32 % world.w, idx_i32 / world.w)
}

fn manhattan(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn move_towards_vec2(current: Vec2, target: Vec2, max_delta: f32) -> Vec2 {
    if max_delta <= 0.0 {
        return current;
    }
    let delta = target - current;
    let dist = delta.length();
    if dist <= max_delta || dist <= f32::EPSILON {
        target
    } else {
        current + delta / dist * max_delta
    }
}

fn aabb_intersects(a: Aabb, b: Aabb) -> bool {
    a.min.x < b.max.x && a.max.x > b.min.x && a.min.y < b.max.y && a.max.y > b.min.y
}

fn nearest_walkable_tile(world: &World, desired: (i32, i32)) -> Option<(i32, i32)> {
    let start = (
        clamp_i32(desired.0, 0, world.w - 1),
        clamp_i32(desired.1, 0, world.h - 1),
    );
    if !world.is_solid(start.0, start.1) {
        return Some(start);
    }

    let mut queue = VecDeque::new();
    let mut visited = vec![false; (world.w * world.h) as usize];
    let start_idx = world.idx(start.0, start.1);
    visited[start_idx] = true;
    queue.push_back(start);

    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in dirs {
            let nx = x + dx;
            let ny = y + dy;
            if !world.in_bounds(nx, ny) {
                continue;
            }
            let idx = world.idx(nx, ny);
            if visited[idx] {
                continue;
            }
            visited[idx] = true;
            if !world.is_solid(nx, ny) {
                return Some((nx, ny));
            }
            queue.push_back((nx, ny));
        }
    }

    None
}

fn a_star_path(world: &World, start: (i32, i32), goal: (i32, i32)) -> Option<Vec<(i32, i32)>> {
    if !world.in_bounds(start.0, start.1) || !world.in_bounds(goal.0, goal.1) {
        return None;
    }
    if world.is_solid(goal.0, goal.1) {
        return None;
    }
    if start == goal {
        return Some(vec![start]);
    }

    let size = (world.w * world.h) as usize;
    let mut g_score = vec![i32::MAX; size];
    let mut came_from: Vec<Option<usize>> = vec![None; size];
    let mut closed = vec![false; size];
    let mut open = BinaryHeap::new();

    let start_idx = world.idx(start.0, start.1);
    let goal_idx = world.idx(goal.0, goal.1);
    g_score[start_idx] = 0;
    open.push(OpenNode {
        f: manhattan(start, goal),
        g: 0,
        idx: start_idx,
    });

    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    while let Some(node) = open.pop() {
        if closed[node.idx] {
            continue;
        }
        if node.idx == goal_idx {
            let mut rev = vec![goal];
            let mut cur = node.idx;
            while cur != start_idx {
                let prev = came_from[cur]?;
                cur = prev;
                rev.push(idx_to_tile(world, cur));
            }
            rev.reverse();
            return Some(rev);
        }

        closed[node.idx] = true;
        let (cx, cy) = idx_to_tile(world, node.idx);
        for (dx, dy) in dirs {
            let nx = cx + dx;
            let ny = cy + dy;
            if !world.in_bounds(nx, ny) || world.is_solid(nx, ny) {
                continue;
            }

            let nidx = world.idx(nx, ny);
            if closed[nidx] {
                continue;
            }

            let tentative_g = node.g + 1;
            if tentative_g < g_score[nidx] {
                g_score[nidx] = tentative_g;
                came_from[nidx] = Some(node.idx);
                open.push(OpenNode {
                    f: tentative_g + manhattan((nx, ny), goal),
                    g: tentative_g,
                    idx: nidx,
                });
            }
        }
    }

    None
}

fn simplify_tile_path(path: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if path.len() <= 2 {
        return path.to_vec();
    }

    let mut out = Vec::with_capacity(path.len());
    out.push(path[0]);
    let mut prev_dir = (path[1].0 - path[0].0, path[1].1 - path[0].1);

    for i in 1..path.len() - 1 {
        let dir = (path[i + 1].0 - path[i].0, path[i + 1].1 - path[i].1);
        if dir != prev_dir {
            out.push(path[i]);
            prev_dir = dir;
        }
    }

    if let Some(last) = path.last() {
        out.push(*last);
    }
    out
}

fn clear_auto_move_state(auto: &mut AutoMoveState) {
    auto.target_tile = None;
    auto.target_world = None;
    auto.path_tiles.clear();
    auto.path_world.clear();
    auto.next_waypoint = 0;
}

fn reset_auto_move(player: &mut Player) {
    player.control_mode = ControlMode::Manual;
    player.velocity = Vec2::ZERO;
    clear_auto_move_state(&mut player.auto);
}

fn reset_npc_auto_move(npc: &mut NpcWanderer) {
    npc.velocity = Vec2::ZERO;
    clear_auto_move_state(&mut npc.auto);
}

fn issue_auto_move_command(player: &mut Player, world: &World, requested_tile: (i32, i32)) -> bool {
    let start_tile = tile_from_world_clamped(world, player.pos);
    let Some(goal_tile) = nearest_walkable_tile(world, requested_tile) else {
        reset_auto_move(player);
        return false;
    };

    player.auto.target_tile = Some(goal_tile);
    player.auto.target_world = Some(tile_center(goal_tile));

    if goal_tile == start_tile {
        reset_auto_move(player);
        player.auto.target_tile = Some(goal_tile);
        player.auto.target_world = Some(tile_center(goal_tile));
        return true;
    }

    let Some(raw_path) = a_star_path(world, start_tile, goal_tile) else {
        reset_auto_move(player);
        return false;
    };

    let path_tiles = simplify_tile_path(&raw_path);
    let path_world: Vec<Vec2> = path_tiles
        .iter()
        .skip(1)
        .map(|&tile| tile_center(tile))
        .collect();

    player.auto.path_tiles = path_tiles;
    player.auto.path_world = path_world;
    player.auto.next_waypoint = 0;
    player.control_mode = ControlMode::AutoMove;
    true
}

fn step_auto_move(player: &mut Player, dt: f32) -> Vec2 {
    if player.auto.path_world.is_empty() {
        reset_auto_move(player);
        return Vec2::ZERO;
    }

    while player.auto.next_waypoint < player.auto.path_world.len() {
        let waypoint = player.auto.path_world[player.auto.next_waypoint];
        if player.pos.distance(waypoint) <= AUTO_WAYPOINT_REACH {
            player.auto.next_waypoint += 1;
        } else {
            break;
        }
    }

    if player.auto.next_waypoint >= player.auto.path_world.len() {
        reset_auto_move(player);
        return Vec2::ZERO;
    }

    let waypoint = player.auto.path_world[player.auto.next_waypoint];
    let to_waypoint = waypoint - player.pos;
    let dist = to_waypoint.length();
    if dist <= f32::EPSILON {
        return Vec2::ZERO;
    }

    let dir = to_waypoint / dist;
    let final_target = player.auto.target_world.unwrap_or(waypoint);
    let final_dist = player.pos.distance(final_target);
    let slowdown = if final_dist < AUTO_ARRIVE_RADIUS {
        (final_dist / AUTO_ARRIVE_RADIUS).clamp(0.22, 1.0)
    } else {
        1.0
    };

    let desired_velocity = dir * (player.speed * slowdown);
    player.velocity = move_towards_vec2(player.velocity, desired_velocity, AUTO_ACCEL * dt);
    player.velocity * dt
}

fn apply_control_inputs(
    player: &mut Player,
    world: &World,
    keyboard_input: Vec2,
    click_tile: Option<(i32, i32)>,
) {
    if let Some(tile) = click_tile {
        let _ = issue_auto_move_command(player, world, tile);
    }

    if keyboard_input.length_squared() > 0.0 {
        reset_auto_move(player);
        player.control_mode = ControlMode::Manual;
    }
}

fn npc_rand_u32(npc: &mut NpcWanderer) -> u32 {
    let mut x = npc.rng_state;
    if x == 0 {
        x = 0x9E37_79B9_7F4A_7C15;
    }
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    npc.rng_state = x;
    (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 32) as u32
}

fn npc_rand_f32(npc: &mut NpcWanderer) -> f32 {
    npc_rand_u32(npc) as f32 / u32::MAX as f32
}

fn npc_rand_idle_duration(npc: &mut NpcWanderer) -> f32 {
    NPC_IDLE_MIN + (NPC_IDLE_MAX - NPC_IDLE_MIN) * npc_rand_f32(npc)
}

fn npc_choose_wander_target(npc: &mut NpcWanderer, world: &World) -> Option<(i32, i32)> {
    let current_tile = tile_from_world_clamped(world, npc.pos);
    let inner_w = (world.w - 2).max(1) as u32;
    let inner_h = (world.h - 2).max(1) as u32;

    for _ in 0..30 {
        let tx = 1 + (npc_rand_u32(npc) % inner_w) as i32;
        let ty = 1 + (npc_rand_u32(npc) % inner_h) as i32;
        if world.is_solid(tx, ty) {
            continue;
        }
        if manhattan((tx, ty), current_tile) < 4 {
            continue;
        }
        return Some((tx, ty));
    }

    None
}

fn issue_npc_wander_command(
    npc: &mut NpcWanderer,
    world: &World,
    requested_tile: (i32, i32),
) -> bool {
    let start_tile = tile_from_world_clamped(world, npc.pos);
    let Some(goal_tile) = nearest_walkable_tile(world, requested_tile) else {
        reset_npc_auto_move(npc);
        return false;
    };

    if goal_tile == start_tile {
        reset_npc_auto_move(npc);
        npc.auto.target_tile = Some(goal_tile);
        npc.auto.target_world = Some(tile_center(goal_tile));
        return true;
    }

    let Some(raw_path) = a_star_path(world, start_tile, goal_tile) else {
        reset_npc_auto_move(npc);
        return false;
    };

    let path_tiles = simplify_tile_path(&raw_path);
    let path_world: Vec<Vec2> = path_tiles
        .iter()
        .skip(1)
        .map(|&tile| tile_center(tile))
        .collect();

    npc.auto.target_tile = Some(goal_tile);
    npc.auto.target_world = Some(tile_center(goal_tile));
    npc.auto.path_tiles = path_tiles;
    npc.auto.path_world = path_world;
    npc.auto.next_waypoint = 0;
    true
}

fn step_npc_auto_move(npc: &mut NpcWanderer, dt: f32) -> Vec2 {
    if npc.auto.path_world.is_empty() {
        return Vec2::ZERO;
    }

    while npc.auto.next_waypoint < npc.auto.path_world.len() {
        let waypoint = npc.auto.path_world[npc.auto.next_waypoint];
        if npc.pos.distance(waypoint) <= AUTO_WAYPOINT_REACH {
            npc.auto.next_waypoint += 1;
        } else {
            break;
        }
    }

    if npc.auto.next_waypoint >= npc.auto.path_world.len() {
        return Vec2::ZERO;
    }

    let waypoint = npc.auto.path_world[npc.auto.next_waypoint];
    let to_waypoint = waypoint - npc.pos;
    let dist = to_waypoint.length();
    if dist <= f32::EPSILON {
        return Vec2::ZERO;
    }

    let dir = to_waypoint / dist;
    let final_target = npc.auto.target_world.unwrap_or(waypoint);
    let final_dist = npc.pos.distance(final_target);
    let slowdown = if final_dist < AUTO_ARRIVE_RADIUS {
        (final_dist / AUTO_ARRIVE_RADIUS).clamp(0.22, 1.0)
    } else {
        1.0
    };
    let desired_velocity = dir * (npc.speed * slowdown);
    npc.velocity = move_towards_vec2(npc.velocity, desired_velocity, AUTO_ACCEL * 0.75 * dt);
    npc.velocity * dt
}

fn move_npc_axis(npc: &mut NpcWanderer, world: &World, delta: f32, is_x_axis: bool) {
    if delta.abs() <= f32::EPSILON {
        return;
    }

    if is_x_axis {
        npc.pos.x += delta;
    } else {
        npc.pos.y += delta;
    }

    let mut aabb = Aabb::from_center(npc.pos, npc.half);
    let (min_tx, max_tx, min_ty, max_ty) = tiles_overlapping_aabb(world, aabb);

    for ty in min_ty..=max_ty {
        for tx in min_tx..=max_tx {
            if !world.is_solid(tx, ty) {
                continue;
            }

            let tile = World::tile_rect(tx, ty);
            if !aabb.intersects_rect(tile) {
                continue;
            }

            if is_x_axis {
                if delta > 0.0 {
                    npc.pos.x = tile.x - npc.half.x;
                } else {
                    npc.pos.x = tile.x + tile.w + npc.half.x;
                }
            } else if delta > 0.0 {
                npc.pos.y = tile.y - npc.half.y;
            } else {
                npc.pos.y = tile.y + tile.h + npc.half.y;
            }

            aabb = Aabb::from_center(npc.pos, npc.half);
        }
    }
}

fn update_npc_wanderer(npc: &mut NpcWanderer, world: &World, player: &Player, dt: f32) {
    npc.bubble_timer = (npc.bubble_timer - dt).max(0.0);
    npc.bubble_cooldown = (npc.bubble_cooldown - dt).max(0.0);

    let player_aabb = Aabb::from_center(player.pos, player.half);
    let npc_aabb = Aabb::from_center(npc.pos, npc.half);
    let close_enough = npc.pos.distance(player.pos) <= NPC_GREETING_RADIUS;
    let overlap = aabb_intersects(player_aabb, npc_aabb);
    if (close_enough || overlap) && npc.bubble_cooldown <= 0.0 {
        npc.bubble_timer = NPC_GREETING_DURATION;
        npc.bubble_cooldown = NPC_GREETING_COOLDOWN;
    }

    let had_active_path =
        !npc.auto.path_world.is_empty() && npc.auto.next_waypoint < npc.auto.path_world.len();
    let path_finished =
        npc.auto.path_world.is_empty() || npc.auto.next_waypoint >= npc.auto.path_world.len();
    if path_finished {
        if npc.idle_timer > 0.0 {
            npc.idle_timer = (npc.idle_timer - dt).max(0.0);
        } else if let Some(target_tile) = npc_choose_wander_target(npc, world) {
            if issue_npc_wander_command(npc, world, target_tile) {
                npc.idle_timer = 0.0;
            } else {
                npc.idle_timer = 0.35;
            }
        } else {
            npc.idle_timer = 0.35;
        }
    }

    let requested_delta = step_npc_auto_move(npc, dt);
    let before = npc.pos;
    move_npc_axis(npc, world, requested_delta.x, true);
    move_npc_axis(npc, world, requested_delta.y, false);
    let moved = npc.pos - before;

    let facing_source = if moved.length_squared() > 0.0001 {
        moved.normalize()
    } else if npc.velocity.length_squared() > 0.0001 {
        npc.velocity.normalize()
    } else {
        Vec2::ZERO
    };

    if facing_source.x < -0.05 {
        npc.facing_left = true;
    } else if facing_source.x > 0.05 {
        npc.facing_left = false;
    }

    npc.facing = select_character_facing(facing_source, npc.facing);
    npc.is_walking = moved.length_squared() > 0.0005;

    if npc.is_walking {
        let expected = (npc.speed * dt).max(0.001);
        let speed_scale = (moved.length() / expected).clamp(0.3, 1.2);
        npc.walk_cycle += dt * WALK_CYCLE_SPEED * speed_scale;
        if npc.walk_cycle > std::f32::consts::TAU {
            npc.walk_cycle -= std::f32::consts::TAU;
        }
        npc.anim_frame = ((npc.walk_cycle / std::f32::consts::PI) as usize) & 1;
    } else {
        npc.walk_cycle *= 0.82;
        npc.anim_frame = 0;
        npc.velocity = Vec2::ZERO;
    }

    let reached_end =
        !npc.auto.path_world.is_empty() && npc.auto.next_waypoint >= npc.auto.path_world.len();
    if reached_end {
        reset_npc_auto_move(npc);
    }

    if had_active_path && reached_end {
        npc.idle_timer = npc_rand_idle_duration(npc);
    }
}

fn move_player_axis(player: &mut Player, world: &World, delta: f32, is_x_axis: bool) {
    if delta.abs() <= f32::EPSILON {
        return;
    }

    if is_x_axis {
        player.pos.x += delta;
    } else {
        player.pos.y += delta;
    }

    let mut aabb = Aabb::from_center(player.pos, player.half);
    let (min_tx, max_tx, min_ty, max_ty) = tiles_overlapping_aabb(world, aabb);

    for ty in min_ty..=max_ty {
        for tx in min_tx..=max_tx {
            if !world.is_solid(tx, ty) {
                continue;
            }

            let tile = World::tile_rect(tx, ty);
            if !aabb.intersects_rect(tile) {
                continue;
            }

            if is_x_axis {
                if delta > 0.0 {
                    player.pos.x = tile.x - player.half.x;
                } else {
                    player.pos.x = tile.x + tile.w + player.half.x;
                }
            } else if delta > 0.0 {
                player.pos.y = tile.y - player.half.y;
            } else {
                player.pos.y = tile.y + tile.h + player.half.y;
            }

            aabb = Aabb::from_center(player.pos, player.half);
        }
    }
}

fn update_player(player: &mut Player, world: &World, input: Vec2, dt: f32) {
    let requested_delta = match player.control_mode {
        ControlMode::Manual => {
            player.velocity = input * player.speed;
            player.velocity * dt
        }
        ControlMode::AutoMove => step_auto_move(player, dt),
    };

    let before = player.pos;
    move_player_axis(player, world, requested_delta.x, true);
    move_player_axis(player, world, requested_delta.y, false);
    let moved = player.pos - before;

    let facing_source = if moved.length_squared() > 0.0001 {
        moved.normalize()
    } else if player.velocity.length_squared() > 0.0001 {
        player.velocity.normalize()
    } else {
        Vec2::ZERO
    };

    if facing_source.x < -0.05 {
        player.facing_left = true;
    } else if facing_source.x > 0.05 {
        player.facing_left = false;
    }

    player.facing = select_character_facing(facing_source, player.facing);
    player.is_walking = moved.length_squared() > 0.0005;

    if player.is_walking {
        let expected = (player.speed * dt).max(0.001);
        let speed_scale = (moved.length() / expected).clamp(0.35, 1.35);
        player.walk_cycle += dt * WALK_CYCLE_SPEED * speed_scale;
        if player.walk_cycle > std::f32::consts::TAU {
            player.walk_cycle -= std::f32::consts::TAU;
        }
        player.anim_frame = ((player.walk_cycle / std::f32::consts::PI) as usize) & 1;
    } else {
        player.walk_cycle *= 0.82;
        player.anim_frame = 0;
        if player.control_mode == ControlMode::Manual {
            player.velocity = Vec2::ZERO;
        }
    }

    if player.control_mode == ControlMode::AutoMove
        && player.auto.next_waypoint >= player.auto.path_world.len()
    {
        reset_auto_move(player);
    }
}

fn wall_mask_4(world: &World, x: i32, y: i32) -> u8 {
    let mut mask = 0;
    if tile_is_wall(world.get(x, y - 1)) {
        mask |= MASK_N;
    }
    if tile_is_wall(world.get(x + 1, y)) {
        mask |= MASK_E;
    }
    if tile_is_wall(world.get(x, y + 1)) {
        mask |= MASK_S;
    }
    if tile_is_wall(world.get(x - 1, y)) {
        mask |= MASK_W;
    }
    mask
}

fn draw_background(palette: &Palette, time: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let lines = sh.max(1.0) as i32;

    for y in 0..lines {
        let t = y as f32 / (lines - 1).max(1) as f32;
        let c = color_lerp(palette.bg_top, palette.bg_bottom, t);
        draw_line(0.0, y as f32, sw, y as f32, 1.0, c);
    }

    let haze_x = sw * 0.5 + (time * 0.23).sin() * sw * 0.18;
    let haze_y = sh * 0.2 + (time * 0.31).cos() * 8.0;
    draw_circle(haze_x, haze_y, sw * 0.35, with_alpha(palette.haze, 0.07));
    draw_circle(
        sw * 0.2,
        sh * 0.75,
        sw * 0.25,
        with_alpha(palette.haze, 0.05),
    );
}

fn draw_floor_tile(x: i32, y: i32, tile: Tile, palette: &Palette) {
    let rect = World::tile_rect(x, y);
    let h = tile_hash(x, y);

    let (base_a, base_b, panel_tint) = match tile {
        Tile::Floor => (palette.floor_a, palette.floor_b, palette.floor_panel),
        Tile::FloorMetal => (
            color_lerp(palette.floor_b, palette.wall_mid, 0.35),
            color_lerp(palette.floor_c, palette.wall_dark, 0.45),
            with_alpha(palette.prop_pipe_highlight, 0.75),
        ),
        Tile::FloorWood => (
            color_lerp(palette.prop_crate_light, palette.prop_crate_dark, 0.18),
            color_lerp(palette.prop_crate_dark, palette.wall_dark, 0.28),
            with_alpha(palette.prop_crate_light, 0.62),
        ),
        Tile::FloorMoss => (
            color_lerp(palette.floor_a, rgba(58, 96, 76, 255), 0.55),
            color_lerp(palette.floor_c, rgba(42, 71, 58, 255), 0.5),
            with_alpha(rgba(118, 172, 132, 255), 0.42),
        ),
        Tile::FloorSand => (
            color_lerp(rgba(138, 124, 92, 255), palette.floor_b, 0.28),
            color_lerp(rgba(122, 107, 81, 255), palette.floor_c, 0.28),
            with_alpha(rgba(188, 164, 126, 255), 0.46),
        ),
        _ => (palette.floor_a, palette.floor_b, palette.floor_panel),
    };

    let base = match h % 3 {
        0 => base_a,
        1 => base_b,
        _ => color_lerp(base_a, base_b, 0.55),
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, base);

    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.w - 1.0,
        rect.h - 1.0,
        1.0,
        palette.floor_edge,
    );

    if h & 1 == 0 {
        draw_line(
            rect.x + 4.0,
            rect.y + rect.h * 0.5,
            rect.x + rect.w - 4.0,
            rect.y + rect.h * 0.5,
            1.0,
            panel_tint,
        );
    } else {
        draw_line(
            rect.x + rect.w * 0.5,
            rect.y + 4.0,
            rect.x + rect.w * 0.5,
            rect.y + rect.h - 4.0,
            1.0,
            panel_tint,
        );
    }

    if h.is_multiple_of(7) {
        draw_line(
            rect.x + 8.0,
            rect.y + 9.0,
            rect.x + rect.w - 9.0,
            rect.y + rect.h - 8.0,
            1.0,
            with_alpha(panel_tint, 0.32),
        );
    }

    if h.is_multiple_of(5) {
        draw_circle(rect.x + 5.0, rect.y + 5.0, 1.2, palette.floor_rivet);
        draw_circle(
            rect.x + rect.w - 5.0,
            rect.y + 5.0,
            1.2,
            palette.floor_rivet,
        );
        draw_circle(
            rect.x + 5.0,
            rect.y + rect.h - 5.0,
            1.2,
            palette.floor_rivet,
        );
        draw_circle(
            rect.x + rect.w - 5.0,
            rect.y + rect.h - 5.0,
            1.2,
            palette.floor_rivet,
        );
    }

    let grime_strength = 0.04 + ((hash_with_salt(x, y, 13) & 0x0F) as f32 / 255.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        with_alpha(palette.floor_grime, grime_strength),
    );
}

fn draw_floor_layer_region(world: &World, palette: &Palette, bounds: (i32, i32, i32, i32)) {
    for y in bounds.2..=bounds.3 {
        for x in bounds.0..=bounds.1 {
            let tile = world.get(x, y);
            if tile_is_floor(tile) {
                draw_floor_tile(x, y, tile, palette);
            }
        }
    }
}

fn draw_wall_cast_shadows_region(world: &World, palette: &Palette, bounds: (i32, i32, i32, i32)) {
    for y in bounds.2..=bounds.3 {
        for x in bounds.0..=bounds.1 {
            if !tile_is_wall(world.get(x, y)) {
                continue;
            }
            let rect = World::tile_rect(x, y);

            if tile_is_floor(world.get(x, y + 1)) {
                draw_rectangle(
                    rect.x + 2.0,
                    rect.y + rect.h - 2.0,
                    rect.w - 4.0,
                    7.0,
                    with_alpha(palette.shadow_hard, 0.35),
                );
                draw_rectangle(
                    rect.x + 3.0,
                    rect.y + rect.h + 3.0,
                    rect.w - 6.0,
                    4.0,
                    with_alpha(palette.shadow_soft, 0.20),
                );
            }

            if tile_is_floor(world.get(x + 1, y)) {
                draw_rectangle(
                    rect.x + rect.w - 2.0,
                    rect.y + 2.0,
                    6.0,
                    rect.h - 4.0,
                    with_alpha(palette.shadow_hard, 0.18),
                );
            }
        }
    }
}

fn draw_wall_tile(world: &World, x: i32, y: i32, tile: Tile, palette: &Palette) {
    let rect = World::tile_rect(x, y);
    let mask = wall_mask_4(world, x, y);
    let h = tile_hash(x, y);
    let (wall_top, wall_mid, wall_dark, wall_outline) = match tile {
        Tile::Wall => (
            palette.wall_top,
            palette.wall_mid,
            palette.wall_dark,
            palette.wall_outline,
        ),
        Tile::WallBrick => (
            color_lerp(palette.prop_crate_light, palette.wall_top, 0.34),
            color_lerp(palette.prop_crate_dark, palette.wall_mid, 0.42),
            color_lerp(palette.prop_crate_dark, palette.wall_dark, 0.58),
            color_lerp(palette.wall_outline, palette.prop_crate_dark, 0.32),
        ),
        Tile::WallSteel => (
            color_lerp(palette.wall_top, palette.prop_pipe_highlight, 0.45),
            color_lerp(palette.wall_mid, palette.prop_pipe, 0.36),
            color_lerp(palette.wall_dark, palette.prop_pipe, 0.26),
            color_lerp(palette.wall_outline, palette.prop_pipe_highlight, 0.15),
        ),
        Tile::WallNeon => (
            color_lerp(rgba(120, 112, 168, 255), palette.wall_top, 0.25),
            color_lerp(rgba(92, 84, 142, 255), palette.wall_mid, 0.25),
            color_lerp(rgba(66, 58, 112, 255), palette.wall_dark, 0.22),
            color_lerp(rgba(170, 228, 255, 255), palette.wall_outline, 0.35),
        ),
        _ => (
            palette.wall_top,
            palette.wall_mid,
            palette.wall_dark,
            palette.wall_outline,
        ),
    };

    for band in 0..4 {
        let t0 = band as f32 / 4.0;
        let t1 = (band + 1) as f32 / 4.0;
        let top = color_lerp(wall_top, wall_mid, t0);
        let bottom = color_lerp(wall_top, wall_mid, t1);
        let band_y = rect.y + rect.h * t0;
        let band_h = rect.h * (t1 - t0) + 0.5;
        draw_rectangle(rect.x, band_y, rect.w, band_h, color_lerp(top, bottom, 0.5));
    }

    if mask & MASK_N == 0 {
        draw_rectangle(rect.x, rect.y, rect.w, 4.0, wall_top);
        draw_rectangle(
            rect.x,
            rect.y + 4.0,
            rect.w,
            1.5,
            with_alpha(wall_dark, 0.7),
        );
    }
    if mask & MASK_S == 0 {
        draw_rectangle(rect.x, rect.y + rect.h - 4.0, rect.w, 4.0, wall_dark);
    }
    if mask & MASK_W == 0 {
        draw_rectangle(rect.x, rect.y, 3.0, rect.h, with_alpha(wall_dark, 0.9));
    }
    if mask & MASK_E == 0 {
        draw_rectangle(
            rect.x + rect.w - 3.0,
            rect.y,
            3.0,
            rect.h,
            with_alpha(wall_dark, 0.9),
        );
    }

    if mask & MASK_N == 0 && mask & MASK_W == 0 {
        draw_rectangle(rect.x, rect.y, 5.0, 5.0, with_alpha(wall_top, 0.95));
    }
    if mask & MASK_N == 0 && mask & MASK_E == 0 {
        draw_rectangle(
            rect.x + rect.w - 5.0,
            rect.y,
            5.0,
            5.0,
            with_alpha(wall_top, 0.95),
        );
    }

    if h & 1 == 0 {
        draw_line(
            rect.x + 6.0,
            rect.y + 10.0,
            rect.x + rect.w - 7.0,
            rect.y + 10.0,
            1.0,
            with_alpha(wall_outline, 0.30),
        );
    }
    if h.is_multiple_of(4) {
        draw_line(
            rect.x + 8.0,
            rect.y + 18.0,
            rect.x + rect.w - 8.0,
            rect.y + 18.0,
            1.0,
            with_alpha(wall_outline, 0.25),
        );
    }

    if matches!(tile, Tile::WallBrick) {
        for by in 7..28 {
            if by % 8 == 0 {
                draw_line(
                    rect.x + 2.0,
                    rect.y + by as f32,
                    rect.x + rect.w - 2.0,
                    rect.y + by as f32,
                    1.0,
                    with_alpha(wall_outline, 0.32),
                );
            }
        }
        draw_line(
            rect.x + rect.w * 0.5,
            rect.y + 6.0,
            rect.x + rect.w * 0.5,
            rect.y + rect.h - 6.0,
            1.0,
            with_alpha(wall_outline, 0.25),
        );
    } else if matches!(tile, Tile::WallSteel) {
        draw_rectangle(
            rect.x + 5.0,
            rect.y + 6.0,
            rect.w - 10.0,
            3.0,
            with_alpha(palette.prop_pipe_highlight, 0.22),
        );
        draw_rectangle(
            rect.x + 5.0,
            rect.y + 20.0,
            rect.w - 10.0,
            2.5,
            with_alpha(palette.prop_pipe_highlight, 0.18),
        );
    } else if matches!(tile, Tile::WallNeon) {
        draw_rectangle(
            rect.x + 2.0,
            rect.y + 2.0,
            rect.w - 4.0,
            2.0,
            with_alpha(rgba(136, 255, 236, 255), 0.48),
        );
        draw_rectangle(
            rect.x + 2.0,
            rect.y + rect.h - 4.0,
            rect.w - 4.0,
            2.0,
            with_alpha(rgba(136, 255, 236, 255), 0.42),
        );
    }

    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.w - 1.0,
        rect.h - 1.0,
        1.0,
        wall_outline,
    );
}

fn draw_wall_layer_region(world: &World, palette: &Palette, bounds: (i32, i32, i32, i32)) {
    for y in bounds.2..=bounds.3 {
        for x in bounds.0..=bounds.1 {
            let tile = world.get(x, y);
            if tile_is_wall(tile) {
                draw_wall_tile(world, x, y, tile, palette);
            }
        }
    }
}

fn draw_prop_shadows_region(
    props: &[Prop],
    palette: &Palette,
    time: f32,
    bounds: (i32, i32, i32, i32),
) {
    for prop in props {
        if !tile_in_bounds((prop.tile_x, prop.tile_y), bounds) {
            continue;
        }
        let x = prop.tile_x as f32 * TILE_SIZE;
        let y = prop.tile_y as f32 * TILE_SIZE;

        match prop.kind {
            PropKind::Crate => {
                draw_circle(
                    x + 17.0,
                    y + 25.5,
                    8.0,
                    with_alpha(palette.shadow_hard, 0.34),
                );
            }
            PropKind::Pipe => {
                draw_rectangle(
                    x + 4.0,
                    y + 21.0,
                    24.0,
                    6.0,
                    with_alpha(palette.shadow_soft, 0.30),
                );
            }
            PropKind::Lamp => {
                let pulse = ((time * 2.2 + prop.phase).sin() * 0.5 + 0.5) * 0.3 + 0.7;
                draw_circle(
                    x + 16.0,
                    y + 25.0,
                    6.0 * pulse,
                    with_alpha(palette.shadow_hard, 0.36),
                );
            }
            PropKind::Banner => {
                draw_rectangle(
                    x + 11.0,
                    y + 22.0,
                    11.0,
                    5.0,
                    with_alpha(palette.shadow_soft, 0.26),
                );
            }
            PropKind::Plant => {
                draw_circle(
                    x + 16.0,
                    y + 24.0,
                    7.0,
                    with_alpha(palette.shadow_hard, 0.28),
                );
            }
            PropKind::Bench => {
                draw_rectangle(
                    x + 5.0,
                    y + 24.0,
                    22.0,
                    4.0,
                    with_alpha(palette.shadow_hard, 0.31),
                );
            }
            PropKind::Crystal => {
                let pulse = ((time * 1.7 + prop.phase).sin() * 0.5 + 0.5) * 0.25 + 0.75;
                draw_circle(
                    x + 16.0,
                    y + 23.0,
                    5.5 * pulse,
                    with_alpha(palette.shadow_soft, 0.33),
                );
            }
        }
    }
}

fn draw_props_region(props: &[Prop], palette: &Palette, time: f32, bounds: (i32, i32, i32, i32)) {
    for prop in props {
        if !tile_in_bounds((prop.tile_x, prop.tile_y), bounds) {
            continue;
        }
        let x = prop.tile_x as f32 * TILE_SIZE;
        let y = prop.tile_y as f32 * TILE_SIZE;

        match prop.kind {
            PropKind::Crate => {
                draw_rectangle(x + 6.0, y + 8.0, 20.0, 18.0, palette.prop_crate_dark);
                draw_rectangle(x + 8.0, y + 10.0, 16.0, 14.0, palette.prop_crate_light);
                draw_line(
                    x + 8.0,
                    y + 10.0,
                    x + 24.0,
                    y + 24.0,
                    1.0,
                    palette.wall_outline,
                );
                draw_line(
                    x + 24.0,
                    y + 10.0,
                    x + 8.0,
                    y + 24.0,
                    1.0,
                    palette.wall_outline,
                );
                draw_rectangle_lines(x + 6.5, y + 8.5, 19.0, 17.0, 1.0, palette.wall_outline);
            }
            PropKind::Pipe => {
                let pulse = (time * 1.4 + prop.phase).sin() * 0.5 + 0.5;
                let body = color_lerp(palette.prop_pipe, palette.wall_dark, pulse * 0.18);
                draw_rectangle(x + 4.0, y + 12.0, 24.0, 6.0, body);
                draw_rectangle(x + 4.0, y + 19.0, 24.0, 5.0, body);
                draw_rectangle(x + 7.0, y + 13.0, 18.0, 1.5, palette.prop_pipe_highlight);
                draw_rectangle(x + 7.0, y + 20.0, 18.0, 1.2, palette.prop_pipe_highlight);
                draw_circle(x + 5.0, y + 15.0, 3.0, palette.prop_pipe);
                draw_circle(x + 27.0, y + 15.0, 3.0, palette.prop_pipe);
                draw_circle(x + 5.0, y + 21.5, 2.5, palette.prop_pipe);
                draw_circle(x + 27.0, y + 21.5, 2.5, palette.prop_pipe);
            }
            PropKind::Lamp => {
                let pulse = (time * 2.7 + prop.phase).sin() * 0.5 + 0.5;
                draw_rectangle(x + 14.5, y + 10.0, 3.0, 14.0, palette.wall_dark);
                draw_circle(x + 16.0, y + 8.5, 5.2, palette.lamp_warm);
                draw_circle(x + 16.0, y + 8.5, 3.4, palette.lamp_hot);
                draw_circle(
                    x + 16.0,
                    y + 8.5,
                    1.0 + pulse * 1.2,
                    with_alpha(WHITE, 0.85),
                );
            }
            PropKind::Banner => {
                let sway = (time * 1.9 + prop.phase).sin() * 1.8;
                draw_rectangle(x + 8.0, y + 7.0, 2.5, 18.0, palette.wall_dark);
                draw_rectangle(x + 9.0, y + 7.0, 14.0, 2.5, palette.wall_dark);
                let c1 = color_lerp(palette.lamp_warm, palette.prop_crate_dark, 0.18);
                let c2 = color_lerp(palette.lamp_hot, palette.prop_crate_light, 0.18);
                draw_triangle(
                    vec2(x + 11.0 + sway * 0.25, y + 10.0),
                    vec2(x + 23.0 + sway * 0.45, y + 10.0 + sway * 0.2),
                    vec2(x + 11.5 + sway, y + 22.0),
                    c1,
                );
                draw_line(
                    x + 12.0,
                    y + 13.0,
                    x + 21.0,
                    y + 13.0 + sway * 0.15,
                    1.3,
                    c2,
                );
            }
            PropKind::Plant => {
                let sway = (time * 2.1 + prop.phase).sin() * 1.1;
                let pot = color_lerp(palette.prop_crate_dark, palette.wall_dark, 0.35);
                draw_rectangle(x + 10.5, y + 17.0, 11.0, 8.0, pot);
                draw_rectangle_lines(x + 10.5, y + 17.0, 11.0, 8.0, 1.0, palette.wall_outline);
                let leaf = rgba(92, 162, 104, 255);
                draw_triangle(
                    vec2(x + 15.0, y + 18.0),
                    vec2(x + 10.0 + sway, y + 10.0),
                    vec2(x + 14.0, y + 20.0),
                    leaf,
                );
                draw_triangle(
                    vec2(x + 17.0, y + 18.0),
                    vec2(x + 22.0 + sway, y + 10.0),
                    vec2(x + 18.0, y + 21.0),
                    leaf,
                );
                draw_triangle(
                    vec2(x + 16.0, y + 18.0),
                    vec2(x + 16.0 + sway * 0.7, y + 8.5),
                    vec2(x + 16.0, y + 22.0),
                    rgba(114, 188, 122, 255),
                );
            }
            PropKind::Bench => {
                let wood_top = color_lerp(palette.prop_crate_light, palette.floor_b, 0.2);
                let wood_dark = color_lerp(palette.prop_crate_dark, palette.wall_dark, 0.35);
                draw_rectangle(x + 5.0, y + 15.0, 22.0, 4.0, wood_top);
                draw_rectangle(x + 6.0, y + 19.0, 20.0, 3.0, wood_dark);
                draw_rectangle(x + 7.0, y + 21.5, 2.5, 4.0, wood_dark);
                draw_rectangle(x + 22.5, y + 21.5, 2.5, 4.0, wood_dark);
                draw_rectangle_lines(x + 5.0, y + 15.0, 22.0, 7.0, 1.0, palette.wall_outline);
            }
            PropKind::Crystal => {
                let pulse = (time * 2.4 + prop.phase).sin() * 0.5 + 0.5;
                let core = Color::new(0.42, 0.9, 1.0, 0.88);
                let edge = Color::new(0.72, 0.98, 1.0, 0.98);
                let peak_y = y + 7.0 - pulse * 1.1;
                draw_triangle(
                    vec2(x + 16.0, peak_y),
                    vec2(x + 9.0, y + 22.0),
                    vec2(x + 23.0, y + 22.0),
                    core,
                );
                draw_line(x + 16.0, peak_y + 1.0, x + 16.0, y + 20.0, 1.4, edge);
                draw_line(
                    x + 12.0,
                    y + 18.0,
                    x + 20.0,
                    y + 18.0 + pulse * 0.8,
                    1.2,
                    with_alpha(edge, 0.8),
                );
            }
        }
    }
}

fn draw_lighting_region(
    props: &[Prop],
    palette: &Palette,
    time: f32,
    bounds: (i32, i32, i32, i32),
) {
    for prop in props {
        if !tile_in_bounds((prop.tile_x, prop.tile_y), bounds) {
            continue;
        }
        let cx = prop.tile_x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let cy = prop.tile_y as f32 * TILE_SIZE + 9.0;
        if matches!(prop.kind, PropKind::Lamp) {
            let pulse = (time * 2.2 + prop.phase).sin() * 0.5 + 0.5;

            draw_circle(
                cx,
                cy,
                18.0 + pulse * 3.0,
                with_alpha(palette.lamp_hot, 0.23 + pulse * 0.05),
            );
            draw_circle(
                cx,
                cy,
                33.0 + pulse * 5.0,
                with_alpha(palette.lamp_warm, 0.12 + pulse * 0.02),
            );
            draw_circle(
                cx,
                cy,
                55.0 + pulse * 7.0,
                with_alpha(palette.lamp_warm, 0.05 + pulse * 0.02),
            );
        } else if matches!(prop.kind, PropKind::Crystal) {
            let pulse = (time * 2.8 + prop.phase).sin() * 0.5 + 0.5;
            let glow = Color::new(0.38, 0.92, 1.0, 0.18 + pulse * 0.05);
            draw_circle(cx, cy + 8.0, 15.0 + pulse * 4.0, glow);
            draw_circle(cx, cy + 8.0, 28.0 + pulse * 5.0, with_alpha(glow, 0.45));
        }
    }
}

fn draw_ambient_dust(palette: &Palette, time: f32) {
    let sw = screen_width();
    let sh = screen_height();

    for i in 0..18 {
        let seed = i as f32;
        let speed = 6.0 + seed * 0.23;
        let x = ((seed * 74.0) + time * speed).rem_euclid(sw + 40.0) - 20.0;
        let y = ((seed * 31.0) + (time * 0.6 + seed).sin() * 22.0 + sh * 0.45).rem_euclid(sh);
        let alpha = 0.03 + ((time * 0.9 + seed).sin() * 0.5 + 0.5) * 0.03;
        draw_circle(
            x,
            y,
            1.0 + (i % 3) as f32 * 0.25,
            with_alpha(palette.dust, alpha),
        );
    }
}

fn draw_vignette(palette: &Palette) {
    let sw = screen_width();
    let sh = screen_height();
    let steps = 14;

    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let inset = t * 42.0;
        let w = sw - inset * 2.0;
        let h = sh - inset * 2.0;
        if w <= 0.0 || h <= 0.0 {
            continue;
        }

        let alpha = 0.02 + t * t * 0.10;
        let c = with_alpha(palette.vignette, alpha);
        draw_rectangle(inset, inset, w, 2.0, c);
        draw_rectangle(inset, inset + h - 2.0, w, 2.0, c);
        draw_rectangle(inset, inset, 2.0, h, c);
        draw_rectangle(inset + w - 2.0, inset, 2.0, h, c);
    }
}

fn draw_player(player: &Player, character: &CharacterRecord, time: f32, debug: bool) {
    draw_character(
        character,
        CharacterRenderParams {
            center: player.pos,
            scale: 1.0,
            facing: player.facing,
            facing_left: player.facing_left,
            is_walking: player.is_walking,
            walk_cycle: player.walk_cycle,
            time,
            debug: false,
        },
    );

    if debug {
        draw_rectangle_lines(
            player.pos.x - player.half.x,
            player.pos.y - player.half.y,
            player.half.x * 2.0,
            player.half.y * 2.0,
            1.5,
            GREEN,
        );
    }
}

fn draw_npc(npc: &NpcWanderer, character: &CharacterRecord, time: f32, debug: bool) {
    draw_character(
        character,
        CharacterRenderParams {
            center: npc.pos,
            scale: 0.96,
            facing: npc.facing,
            facing_left: npc.facing_left,
            is_walking: npc.is_walking,
            walk_cycle: npc.walk_cycle,
            time,
            debug: false,
        },
    );

    if npc.bubble_timer > 0.0 {
        let alpha = (npc.bubble_timer / NPC_GREETING_DURATION).clamp(0.0, 1.0);
        let bubble_w = 44.0;
        let bubble_h = 22.0;
        let bx = npc.pos.x - bubble_w * 0.5;
        let by = npc.pos.y - 40.0;
        let bg = Color::new(0.95, 0.98, 1.0, 0.92 * alpha);
        let border = Color::new(0.22, 0.26, 0.34, 0.95 * alpha);
        let tail = Color::new(0.95, 0.98, 1.0, 0.9 * alpha);

        draw_rectangle(bx, by, bubble_w, bubble_h, bg);
        draw_rectangle_lines(bx, by, bubble_w, bubble_h, 1.5, border);
        draw_triangle(
            vec2(npc.pos.x - 4.0, by + bubble_h - 0.5),
            vec2(npc.pos.x + 4.0, by + bubble_h - 0.5),
            vec2(npc.pos.x, by + bubble_h + 8.0),
            tail,
        );
        draw_text_ex(
            "Yo !",
            bx + 9.5,
            by + 15.5,
            TextParams {
                font_size: 16,
                color: Color::new(0.12, 0.15, 0.22, 0.98 * alpha),
                ..Default::default()
            },
        );
    }

    if debug {
        draw_rectangle_lines(
            npc.pos.x - npc.half.x,
            npc.pos.y - npc.half.y,
            npc.half.x * 2.0,
            npc.half.y * 2.0,
            1.3,
            ORANGE,
        );
    }
}

fn draw_auto_move_overlay(player: &Player) {
    if player.auto.path_world.is_empty() {
        return;
    }

    let mut prev = player.pos;
    for (i, point) in player.auto.path_world.iter().enumerate() {
        let passed = i < player.auto.next_waypoint;
        let color = if passed {
            Color::from_rgba(80, 170, 200, 110)
        } else {
            Color::from_rgba(80, 220, 255, 210)
        };
        draw_line(prev.x, prev.y, point.x, point.y, 2.0, color);
        draw_circle(point.x, point.y, 3.0, color);
        prev = *point;
    }

    if let Some(target) = player.auto.target_world {
        draw_circle_lines(
            target.x,
            target.y,
            7.0,
            2.0,
            Color::from_rgba(255, 190, 80, 230),
        );
    }
}

fn draw_npc_wander_overlay(npc: &NpcWanderer) {
    if npc.auto.path_world.is_empty() {
        return;
    }

    let mut prev = npc.pos;
    for (i, point) in npc.auto.path_world.iter().enumerate() {
        let passed = i < npc.auto.next_waypoint;
        let color = if passed {
            Color::from_rgba(170, 140, 80, 110)
        } else {
            Color::from_rgba(255, 205, 95, 200)
        };
        draw_line(prev.x, prev.y, point.x, point.y, 1.8, color);
        draw_circle(point.x, point.y, 2.6, color);
        prev = *point;
    }
}

fn draw_editor_grid_region(bounds: (i32, i32, i32, i32)) {
    for x in bounds.0..=bounds.1 + 1 {
        let px = x as f32 * TILE_SIZE;
        draw_line(
            px,
            bounds.2 as f32 * TILE_SIZE,
            px,
            (bounds.3 + 1) as f32 * TILE_SIZE,
            1.0,
            Color::from_rgba(110, 140, 165, 70),
        );
    }
    for y in bounds.2..=bounds.3 + 1 {
        let py = y as f32 * TILE_SIZE;
        draw_line(
            bounds.0 as f32 * TILE_SIZE,
            py,
            (bounds.1 + 1) as f32 * TILE_SIZE,
            py,
            1.0,
            Color::from_rgba(110, 140, 165, 70),
        );
    }
}

fn run_main_menu_frame(map: &MapAsset, palette: &Palette, time: f32) -> Option<AppMode> {
    clear_background(palette.bg_bottom);
    draw_background(palette, time * 0.6);
    let (menu_camera, menu_view_rect) = fit_world_camera_to_screen(&map.world, 34.0);
    let visible_bounds = tile_bounds_from_camera(&map.world, &menu_camera, menu_view_rect, 2);
    set_camera(&menu_camera);
    draw_floor_layer_region(&map.world, palette, visible_bounds);
    draw_wall_cast_shadows_region(&map.world, palette, visible_bounds);
    draw_wall_layer_region(&map.world, palette, visible_bounds);
    draw_prop_shadows_region(&map.props, palette, time, visible_bounds);
    draw_props_region(&map.props, palette, time, visible_bounds);
    draw_lighting_region(&map.props, palette, time, visible_bounds);
    set_default_camera();
    draw_rectangle_lines(
        menu_view_rect.x + 0.5,
        menu_view_rect.y + 0.5,
        menu_view_rect.w - 1.0,
        menu_view_rect.h - 1.0,
        2.0,
        Color::from_rgba(130, 170, 194, 165),
    );
    draw_ambient_dust(palette, time);
    draw_vignette(palette);

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(6, 9, 13, 165),
    );

    let title = "RXCHIXS";
    let subtitle = "Prototype Jeu + Editeur de Carte";
    let title_dims = measure_text(title, None, 74, 1.0);
    let subtitle_dims = measure_text(subtitle, None, 30, 1.0);
    let center_x = screen_width() * 0.5;

    draw_text(
        title,
        center_x - title_dims.width * 0.5,
        130.0,
        74.0,
        Color::from_rgba(240, 248, 255, 255),
    );
    draw_text(
        subtitle,
        center_x - subtitle_dims.width * 0.5,
        168.0,
        30.0,
        Color::from_rgba(182, 208, 220, 255),
    );

    let mouse = vec2(mouse_position().0, mouse_position().1);
    let click = is_mouse_button_pressed(MouseButton::Left);
    let play_rect = Rect::new(center_x - 140.0, 232.0, 280.0, 58.0);
    let editor_rect = Rect::new(center_x - 140.0, 306.0, 280.0, 58.0);

    let play_clicked = draw_ui_button(play_rect, "Jouer", mouse, click, false);
    let editor_clicked = draw_ui_button(editor_rect, "Editeur", mouse, click, false);

    draw_text(
        "Raccourcis: [Enter]=Jouer  [E]=Editeur  [F11]=Plein ecran  [Esc]=Menu",
        center_x - 320.0,
        404.0,
        24.0,
        Color::from_rgba(174, 202, 216, 255),
    );
    draw_text(
        "Objectif: edition rapide, pratique et visuellement claire",
        center_x - 278.0,
        434.0,
        22.0,
        Color::from_rgba(144, 171, 188, 255),
    );

    if play_clicked || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
        return Some(AppMode::Playing);
    }
    if editor_clicked || is_key_pressed(KeyCode::E) {
        return Some(AppMode::Editor);
    }
    None
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum PlayAction {
    None,
    BackToMenu,
    OpenEditor,
}

fn sim_zone_overlay_color(zone: sim::ZoneKind) -> Option<Color> {
    match zone {
        sim::ZoneKind::Neutral => None,
        sim::ZoneKind::Receiving => Some(Color::from_rgba(86, 122, 224, 62)),
        sim::ZoneKind::Processing => Some(Color::from_rgba(218, 114, 42, 66)),
        sim::ZoneKind::Shipping => Some(Color::from_rgba(64, 180, 122, 62)),
        sim::ZoneKind::Support => Some(Color::from_rgba(172, 130, 220, 58)),
    }
}

fn sim_block_overlay_color(kind: sim::BlockKind) -> Color {
    match kind {
        sim::BlockKind::Storage => Color::from_rgba(88, 160, 222, 255),
        sim::BlockKind::MachineA => Color::from_rgba(240, 154, 72, 255),
        sim::BlockKind::MachineB => Color::from_rgba(252, 120, 82, 255),
        sim::BlockKind::Buffer => Color::from_rgba(142, 122, 208, 255),
        sim::BlockKind::Seller => Color::from_rgba(94, 196, 124, 255),
    }
}

fn draw_sim_zone_overlay_region(sim: &sim::FactorySim, bounds: (i32, i32, i32, i32)) {
    if !sim.zone_overlay_enabled() {
        return;
    }
    for y in bounds.2..=bounds.3 {
        for x in bounds.0..=bounds.1 {
            if let Some(color) = sim_zone_overlay_color(sim.zone_kind_at_tile((x, y))) {
                let tile = World::tile_rect(x, y);
                draw_rectangle(tile.x, tile.y, tile.w, tile.h, color);
            }
        }
    }
}

fn draw_sim_blocks_overlay(
    sim: &sim::FactorySim,
    show_labels: bool,
    bounds: Option<(i32, i32, i32, i32)>,
) {
    for block in sim.block_debug_views() {
        if let Some(tile_bounds) = bounds
            && !tile_in_bounds(block.tile, tile_bounds)
        {
            continue;
        }
        let rect = World::tile_rect(block.tile.0, block.tile.1);
        let color = sim_block_overlay_color(block.kind);
        draw_rectangle_lines(
            rect.x + 2.0,
            rect.y + 2.0,
            rect.w - 4.0,
            rect.h - 4.0,
            2.0,
            color,
        );
        if show_labels {
            let label = format!("#{} {}", block.id, block.kind.label());
            draw_text(
                &label,
                rect.x + 2.0,
                rect.y - 2.0,
                14.0,
                Color::from_rgba(232, 240, 248, 255),
            );
            draw_text(
                &block.inventory_summary,
                rect.x + 2.0,
                rect.y + rect.h + 13.0,
                12.0,
                Color::from_rgba(180, 215, 232, 255),
            );
        }
    }
}

fn draw_sim_agent_overlay(sim: &sim::FactorySim, show_label: bool) {
    for agent in sim.agent_debug_views() {
        let px = agent.world_pos.0 * TILE_SIZE;
        let py = agent.world_pos.1 * TILE_SIZE;
        draw_circle(px, py, 5.5, Color::from_rgba(255, 214, 122, 245));
        draw_circle_lines(px, py, 8.0, 1.6, Color::from_rgba(255, 248, 220, 245));
        if show_label {
            draw_text(
                &agent.label,
                px - 42.0,
                py - 10.0,
                14.0,
                Color::from_rgba(255, 244, 218, 255),
            );
        }
    }
}

fn run_play_frame(state: &mut GameState, frame_dt: f32, accumulator: &mut f32) -> PlayAction {
    if is_key_pressed(KeyCode::Escape) {
        return PlayAction::BackToMenu;
    }
    if is_key_pressed(KeyCode::F10) {
        return PlayAction::OpenEditor;
    }

    if is_key_pressed(KeyCode::F1) {
        state.debug = !state.debug;
    }
    if is_key_pressed(KeyCode::F2) {
        state.show_character_inspector = !state.show_character_inspector;
    }
    if is_key_pressed(KeyCode::F3) {
        state.lineage_seed = advance_seed(state.lineage_seed);
        state.lineage = build_lineage_preview(&state.character_catalog, state.lineage_seed);
        if state.player_lineage_index >= state.lineage.len() {
            state.player_lineage_index = 0;
        }
    }

    if is_key_pressed(KeyCode::F6) {
        state.sim.toggle_zone_overlay();
    }
    if is_key_pressed(KeyCode::F7) {
        state.sim.toggle_build_mode();
    }
    if is_key_pressed(KeyCode::B) {
        state.sim.cycle_block_brush();
    }
    if is_key_pressed(KeyCode::N) {
        state.sim.cycle_zone_brush();
    }
    if is_key_pressed(KeyCode::V) {
        state.sim.toggle_zone_paint_mode();
    }
    if is_key_pressed(KeyCode::F8) {
        let _ = state.sim.save_layout();
    }

    let wheel = mouse_wheel().1;
    if wheel.abs() > f32::EPSILON {
        let zoom_factor = (1.0 + wheel * PLAY_CAMERA_ZOOM_STEP).max(0.2);
        state.camera_zoom =
            (state.camera_zoom * zoom_factor).clamp(PLAY_CAMERA_ZOOM_MIN, PLAY_CAMERA_ZOOM_MAX);
    }
    if is_key_pressed(KeyCode::C) {
        state.camera_center = state.player.pos;
    }

    let pan = read_camera_pan_input();
    if pan.length_squared() > 0.0 {
        let speed = PLAY_CAMERA_PAN_SPEED / state.camera_zoom.max(0.01);
        state.camera_center += pan * speed * frame_dt;
    }

    let (world_camera, map_view_rect) = if state.world.w <= 36 && state.world.h <= 24 {
        let (camera, rect) = fit_world_camera_to_screen(&state.world, PLAY_CAMERA_MARGIN);
        state.camera_center = vec2(
            state.world.w as f32 * TILE_SIZE * 0.5,
            state.world.h as f32 * TILE_SIZE * 0.5,
        );
        (camera, rect)
    } else {
        let (camera, rect, clamped_center) = build_pannable_world_camera(
            &state.world,
            state.camera_center,
            state.camera_zoom,
            PLAY_CAMERA_MARGIN,
        );
        state.camera_center = clamped_center;
        (camera, rect)
    };

    let mouse = vec2(mouse_position().0, mouse_position().1);
    let mouse_in_map = point_in_rect(mouse, map_view_rect);
    let mouse_world = if mouse_in_map {
        let mut pos = world_camera.screen_to_world(mouse);
        let world_w = state.world.w as f32 * TILE_SIZE;
        let world_h = state.world.h as f32 * TILE_SIZE;
        pos.x = pos.x.clamp(0.0, (world_w - 0.001).max(0.0));
        pos.y = pos.y.clamp(0.0, (world_h - 0.001).max(0.0));
        Some(pos)
    } else {
        None
    };
    let mouse_tile = mouse_world.map(|pos| tile_from_world_clamped(&state.world, pos));

    if is_key_pressed(KeyCode::M)
        && let Some(tile) = mouse_tile
    {
        state.sim.select_move_source(tile);
    }

    let left_click = is_mouse_button_pressed(MouseButton::Left);
    let right_click = is_mouse_button_pressed(MouseButton::Right);
    if state.sim.build_mode_enabled() {
        if left_click && let Some(tile) = mouse_tile {
            state.sim.apply_build_click(tile, false);
        }
        if right_click && let Some(tile) = mouse_tile {
            state.sim.apply_build_click(tile, true);
        }
    }

    let click_tile = if left_click && !state.sim.build_mode_enabled() {
        mouse_tile
    } else {
        None
    };

    state.last_input = read_input_dir();
    apply_control_inputs(
        &mut state.player,
        &state.world,
        state.last_input,
        click_tile,
    );

    *accumulator = (*accumulator + frame_dt).min(FIXED_DT * MAX_SIM_STEPS_PER_FRAME as f32);
    let mut sim_steps = 0usize;
    while *accumulator >= FIXED_DT && sim_steps < MAX_SIM_STEPS_PER_FRAME {
        state.sim.step(FIXED_DT);
        update_player(&mut state.player, &state.world, state.last_input, FIXED_DT);
        update_npc_wanderer(&mut state.npc, &state.world, &state.player, FIXED_DT);
        *accumulator -= FIXED_DT;
        sim_steps += 1;
    }
    if sim_steps == MAX_SIM_STEPS_PER_FRAME && *accumulator >= FIXED_DT {
        *accumulator = 0.0;
    }

    let time = get_time() as f32;
    let visible_bounds = tile_bounds_from_camera(&state.world, &world_camera, map_view_rect, 2);

    clear_background(state.palette.bg_bottom);
    draw_background(&state.palette, time);
    set_camera(&world_camera);
    draw_floor_layer_region(&state.world, &state.palette, visible_bounds);
    draw_sim_zone_overlay_region(&state.sim, visible_bounds);
    draw_wall_cast_shadows_region(&state.world, &state.palette, visible_bounds);
    draw_wall_layer_region(&state.world, &state.palette, visible_bounds);
    draw_prop_shadows_region(&state.props, &state.palette, time, visible_bounds);
    draw_props_region(&state.props, &state.palette, time, visible_bounds);
    draw_sim_blocks_overlay(
        &state.sim,
        state.debug || state.sim.build_mode_enabled(),
        Some(visible_bounds),
    );
    if state.npc.pos.y <= state.player.pos.y {
        draw_npc(&state.npc, &state.npc_character, time, state.debug);
        if let Some(player_character) = state.lineage.get(state.player_lineage_index) {
            draw_player(&state.player, player_character, time, state.debug);
        }
    } else {
        if let Some(player_character) = state.lineage.get(state.player_lineage_index) {
            draw_player(&state.player, player_character, time, state.debug);
        }
        draw_npc(&state.npc, &state.npc_character, time, state.debug);
    }
    if state.debug {
        draw_auto_move_overlay(&state.player);
        draw_npc_wander_overlay(&state.npc);
        let tx = (state.player.pos.x / TILE_SIZE).floor() as i32;
        let ty = (state.player.pos.y / TILE_SIZE).floor() as i32;
        let tile_rect = World::tile_rect(tx, ty);
        draw_rectangle_lines(
            tile_rect.x,
            tile_rect.y,
            tile_rect.w,
            tile_rect.h,
            2.0,
            YELLOW,
        );
    }
    draw_sim_agent_overlay(&state.sim, state.debug || state.sim.build_mode_enabled());
    draw_lighting_region(&state.props, &state.palette, time, visible_bounds);
    set_default_camera();

    draw_rectangle_lines(
        map_view_rect.x + 0.5,
        map_view_rect.y + 0.5,
        map_view_rect.w - 1.0,
        map_view_rect.h - 1.0,
        2.0,
        Color::from_rgba(170, 213, 237, 135),
    );

    draw_ambient_dust(&state.palette, time);
    draw_vignette(&state.palette);

    if state.show_character_inspector {
        draw_character_inspector_panel(state, time);
    }

    if state.debug {
        let tx = (state.player.pos.x / TILE_SIZE).floor() as i32;
        let ty = (state.player.pos.y / TILE_SIZE).floor() as i32;

        let mask = wall_mask_4(&state.world, tx, ty);
        let player_visual = state
            .lineage
            .get(state.player_lineage_index)
            .map(compact_visual_summary)
            .unwrap_or_else(|| "no-character".to_string());
        let target_tile = state
            .player
            .auto
            .target_tile
            .map(|(x, y)| format!("({}, {})", x, y))
            .unwrap_or_else(|| "none".to_string());
        let npc_target_tile = state
            .npc
            .auto
            .target_tile
            .map(|(x, y)| format!("({}, {})", x, y))
            .unwrap_or_else(|| "none".to_string());
        let info = format!(
            "Mode Jeu | Esc=menu | F10=editeur | F11=plein ecran\nF1: debug on/off | F2: inspector | F3: regenerate\ncamera: ZQSD/WASD pan | molette zoom | C recentrer\nmouse: click-to-move sur la map | fleches: override manuel\nplayer pos(px)=({:.1},{:.1}) tile=({},{})\nmode={} walking={} frame={} facing={} facing_left={} walk_cycle={:.2}\ninput=({:.2},{:.2}) camera=({:.1},{:.1}) zoom={:.2} fps={}\nplayer_path_nodes={} next_wp={} target_tile={}\nnpc pos=({:.1},{:.1}) walking={} bubble={:.2}s cooldown={:.2}s npc_path_nodes={} npc_target={}\nwall_mask@tile={:04b}\nmutation_permille={} visual={}\n{}",
            state.player.pos.x,
            state.player.pos.y,
            tx,
            ty,
            control_mode_label(state.player.control_mode),
            state.player.is_walking,
            state.player.anim_frame,
            facing_label(state.player.facing),
            state.player.facing_left,
            state.player.walk_cycle,
            state.last_input.x,
            state.last_input.y,
            state.camera_center.x,
            state.camera_center.y,
            state.camera_zoom,
            get_fps(),
            state.player.auto.path_world.len(),
            state.player.auto.next_waypoint,
            target_tile,
            state.npc.pos.x,
            state.npc.pos.y,
            state.npc.is_walking,
            state.npc.bubble_timer,
            state.npc.bubble_cooldown,
            state.npc.auto.path_world.len(),
            npc_target_tile,
            mask,
            state.character_catalog.mutation_permille(),
            player_visual,
            state.sim.debug_hud(),
        );
        draw_text(&info, 12.0, 20.0, 18.0, WHITE);
    } else {
        draw_text(
            "Mode Jeu | Esc=menu | F10=editeur | F11=plein ecran",
            12.0,
            24.0,
            24.0,
            Color::from_rgba(220, 235, 242, 255),
        );
        draw_text(
            "Camera: ZQSD/WASD pan | molette zoom | C recentrer",
            12.0,
            48.0,
            20.0,
            Color::from_rgba(200, 224, 236, 255),
        );
        let hud = state.sim.short_hud();
        draw_text(&hud, 12.0, 72.0, 22.0, Color::from_rgba(200, 224, 236, 255));
        let build = state.sim.build_hint_line();
        draw_text(
            &build,
            12.0,
            96.0,
            18.0,
            Color::from_rgba(182, 210, 228, 255),
        );
        if !state.sim.status_line().is_empty() {
            draw_text(
                state.sim.status_line(),
                12.0,
                118.0,
                18.0,
                Color::from_rgba(252, 228, 182, 255),
            );
        }
    }

    PlayAction::None
}

fn run_editor_frame(
    editor: &mut EditorState,
    map: &mut MapAsset,
    palette: &Palette,
    time: f32,
) -> EditorAction {
    if editor.status_timer > 0.0 {
        editor.status_timer = (editor.status_timer - get_frame_time()).max(0.0);
    }

    let frame_dt = get_frame_time().min(0.05);
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let left_pressed = is_mouse_button_pressed(MouseButton::Left);
    let left_down = is_mouse_button_down(MouseButton::Left);
    let left_released = is_mouse_button_released(MouseButton::Left);
    let middle_down = is_mouse_button_down(MouseButton::Middle);
    let ctrl_down = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
    let space_down = is_key_down(KeyCode::Space);

    let sw = screen_width();
    let sh = screen_height();
    let margin = 10.0;
    let top_bar_h = 64.0;
    let panel_top = margin + top_bar_h + margin;
    let panel_h = (sh - panel_top - margin).max(180.0);

    let mut left_panel_w = (sw * 0.18).clamp(220.0, 300.0);
    let mut right_panel_w = (sw * 0.24).clamp(280.0, 390.0);
    let min_map_w = 420.0;
    let mut map_w = sw - left_panel_w - right_panel_w - margin * 4.0;
    if map_w < min_map_w {
        let mut deficit = min_map_w - map_w;
        let shrink_right = deficit.min((right_panel_w - 240.0).max(0.0));
        right_panel_w -= shrink_right;
        deficit -= shrink_right;
        let shrink_left = deficit.min((left_panel_w - 190.0).max(0.0));
        left_panel_w -= shrink_left;
        map_w = sw - left_panel_w - right_panel_w - margin * 4.0;
    }
    map_w = map_w.max(180.0);

    let top_bar_rect = Rect::new(margin, margin, (sw - margin * 2.0).max(240.0), top_bar_h);
    let left_panel_rect = Rect::new(margin, panel_top, left_panel_w, panel_h);
    let map_slot_rect = Rect::new(
        left_panel_rect.x + left_panel_rect.w + margin,
        panel_top,
        map_w,
        panel_h,
    );
    let right_panel_rect = Rect::new(
        map_slot_rect.x + map_slot_rect.w + margin,
        panel_top,
        right_panel_w,
        panel_h,
    );
    let map_view_rect = map_slot_rect;

    if !editor.camera_initialized {
        editor.camera_center = tile_center(map.player_spawn);
        editor.camera_zoom = 1.0;
        editor.camera_initialized = true;
    }

    let mouse_over_map = point_in_rect(mouse, map_view_rect);
    let wheel = mouse_wheel().1;
    if mouse_over_map && wheel.abs() > f32::EPSILON {
        let zoom_factor = (1.0 + wheel * EDITOR_CAMERA_ZOOM_STEP).max(0.2);
        editor.camera_zoom = (editor.camera_zoom * zoom_factor)
            .clamp(EDITOR_CAMERA_ZOOM_MIN, EDITOR_CAMERA_ZOOM_MAX);
    }
    if is_key_pressed(KeyCode::PageUp) {
        editor.camera_zoom =
            (editor.camera_zoom * 1.15).clamp(EDITOR_CAMERA_ZOOM_MIN, EDITOR_CAMERA_ZOOM_MAX);
    }
    if is_key_pressed(KeyCode::PageDown) {
        editor.camera_zoom =
            (editor.camera_zoom / 1.15).clamp(EDITOR_CAMERA_ZOOM_MIN, EDITOR_CAMERA_ZOOM_MAX);
    }
    if is_key_pressed(KeyCode::Home) {
        editor.camera_center = tile_center(map.player_spawn);
    }

    let pan_input = read_editor_pan_input(space_down);
    if pan_input.length_squared() > 0.0 {
        let pan_speed = EDITOR_CAMERA_PAN_SPEED / editor.camera_zoom.max(0.01);
        editor.camera_center += pan_input * pan_speed * frame_dt;
    }
    if middle_down && mouse_over_map {
        let delta = mouse_delta_position();
        editor.camera_center -= delta / editor.camera_zoom.max(0.01);
    }

    let (world_camera, clamped_center, clamped_zoom) = build_world_camera_for_viewport(
        &map.world,
        editor.camera_center,
        editor.camera_zoom,
        map_view_rect,
        EDITOR_CAMERA_ZOOM_MIN,
        EDITOR_CAMERA_ZOOM_MAX,
    );
    editor.camera_center = clamped_center;
    editor.camera_zoom = clamped_zoom;

    let world_size_px = vec2(
        map.world.w as f32 * TILE_SIZE,
        map.world.h as f32 * TILE_SIZE,
    );
    let world_mouse = if mouse_over_map {
        let mut pos = world_camera.screen_to_world(mouse);
        pos.x = pos.x.clamp(0.0, (world_size_px.x - 0.001).max(0.0));
        pos.y = pos.y.clamp(0.0, (world_size_px.y - 0.001).max(0.0));
        Some(pos)
    } else {
        None
    };
    editor.hover_tile = world_mouse.map(|pos| tile_from_world_clamped(&map.world, pos));
    let visible_bounds = tile_bounds_from_camera(&map.world, &world_camera, map_view_rect, 2);

    if is_key_pressed(KeyCode::Key1) {
        editor.brush = EditorBrush::Floor;
    }
    if is_key_pressed(KeyCode::Key2) {
        editor.brush = EditorBrush::FloorMetal;
    }
    if is_key_pressed(KeyCode::Key3) {
        editor.brush = EditorBrush::FloorWood;
    }
    if is_key_pressed(KeyCode::Key4) {
        editor.brush = EditorBrush::FloorMoss;
    }
    if is_key_pressed(KeyCode::Key5) {
        editor.brush = EditorBrush::FloorSand;
    }
    if is_key_pressed(KeyCode::Key6) {
        editor.brush = EditorBrush::Wall;
    }
    if is_key_pressed(KeyCode::Key7) {
        editor.brush = EditorBrush::WallBrick;
    }
    if is_key_pressed(KeyCode::Key8) {
        editor.brush = EditorBrush::WallSteel;
    }
    if is_key_pressed(KeyCode::Key9) {
        editor.brush = EditorBrush::WallNeon;
    }
    if is_key_pressed(KeyCode::Key0) {
        editor.brush = EditorBrush::Crate;
    }
    if !space_down && is_key_pressed(KeyCode::Q) {
        editor.brush = EditorBrush::Pipe;
    }
    if !space_down && is_key_pressed(KeyCode::W) {
        editor.brush = EditorBrush::Lamp;
    }
    if is_key_pressed(KeyCode::E) {
        editor.brush = EditorBrush::Banner;
    }
    if is_key_pressed(KeyCode::T) {
        editor.brush = EditorBrush::Plant;
    }
    if is_key_pressed(KeyCode::Y) {
        editor.brush = EditorBrush::Bench;
    }
    if is_key_pressed(KeyCode::U) {
        editor.brush = EditorBrush::Crystal;
    }
    if is_key_pressed(KeyCode::X) {
        editor.brush = EditorBrush::EraseProp;
    }
    if is_key_pressed(KeyCode::B) {
        editor.tool = EditorTool::Brush;
    }
    if is_key_pressed(KeyCode::R) {
        editor.tool = EditorTool::Rect;
    }
    if is_key_pressed(KeyCode::G) {
        editor.show_grid = !editor.show_grid;
    }

    if ctrl_down && is_key_pressed(KeyCode::Z) {
        let _ = editor_undo(editor, map);
    }
    if ctrl_down && is_key_pressed(KeyCode::Y) {
        let _ = editor_redo(editor, map);
    }
    if ctrl_down && is_key_pressed(KeyCode::S) {
        editor_save_current_map(editor, map);
    }
    if ctrl_down && is_key_pressed(KeyCode::L) {
        editor_load_current_map(editor, map);
    }

    if is_key_pressed(KeyCode::P)
        && let Some(tile) = editor.hover_tile
        && !map.world.is_solid(tile.0, tile.1)
    {
        map.player_spawn = tile;
        editor_set_status(editor, format!("Point joueur -> ({}, {})", tile.0, tile.1));
    }
    if is_key_pressed(KeyCode::N)
        && let Some(tile) = editor.hover_tile
        && !map.world.is_solid(tile.0, tile.1)
    {
        map.npc_spawn = tile;
        editor_set_status(editor, format!("Point PNJ -> ({}, {})", tile.0, tile.1));
    }

    let over_ui = point_in_rect(mouse, top_bar_rect)
        || point_in_rect(mouse, left_panel_rect)
        || point_in_rect(mouse, right_panel_rect);
    let can_edit_map = mouse_over_map && !over_ui;

    if editor.tool == EditorTool::Brush {
        if left_pressed && can_edit_map && editor.hover_tile.is_some() {
            editor_push_undo(editor, map);
            editor.stroke_active = true;
            editor.stroke_changed = false;
        }
        if editor.stroke_active
            && left_down
            && let Some(tile) = editor.hover_tile
            && editor_apply_brush(map, editor.brush, tile)
        {
            editor.stroke_changed = true;
            editor.redo_stack.clear();
        }
        if left_released {
            if editor.stroke_active {
                if !editor.stroke_changed {
                    let _ = editor.undo_stack.pop();
                } else {
                    sanitize_map_asset(map);
                }
            }
            editor.stroke_active = false;
            editor.stroke_changed = false;
        }
    } else {
        if left_pressed && can_edit_map {
            editor.drag_start = editor.hover_tile;
        }
        if left_released {
            if let Some(start) = editor.drag_start {
                let end = editor.hover_tile.unwrap_or(start);
                let before = editor.undo_stack.len();
                editor_push_undo(editor, map);
                let changed = editor_apply_brush_rect(map, editor.brush, start, end);
                if changed {
                    editor.redo_stack.clear();
                    sanitize_map_asset(map);
                } else {
                    editor.undo_stack.truncate(before);
                }
            }
            editor.drag_start = None;
        }
    }

    clear_background(palette.bg_bottom);
    draw_background(palette, time);

    draw_rectangle(
        map_slot_rect.x,
        map_slot_rect.y,
        map_slot_rect.w,
        map_slot_rect.h,
        Color::from_rgba(8, 12, 18, 150),
    );

    set_camera(&world_camera);

    draw_floor_layer_region(&map.world, palette, visible_bounds);
    draw_wall_cast_shadows_region(&map.world, palette, visible_bounds);
    draw_wall_layer_region(&map.world, palette, visible_bounds);
    draw_prop_shadows_region(&map.props, palette, time, visible_bounds);
    draw_props_region(&map.props, palette, time, visible_bounds);
    draw_lighting_region(&map.props, palette, time, visible_bounds);

    if editor.show_grid {
        draw_editor_grid_region(visible_bounds);
    }

    if let Some(tile) = editor.hover_tile {
        let rect = World::tile_rect(tile.0, tile.1);
        draw_rectangle_lines(
            rect.x + 1.0,
            rect.y + 1.0,
            rect.w - 2.0,
            rect.h - 2.0,
            2.2,
            Color::from_rgba(255, 212, 120, 240),
        );
    }

    if editor.tool == EditorTool::Rect
        && let Some(start) = editor.drag_start
    {
        let end = editor.hover_tile.unwrap_or(start);
        let min_x = start.0.min(end.0);
        let max_x = start.0.max(end.0);
        let min_y = start.1.min(end.1);
        let max_y = start.1.max(end.1);
        let rect = Rect::new(
            min_x as f32 * TILE_SIZE + 1.5,
            min_y as f32 * TILE_SIZE + 1.5,
            (max_x - min_x + 1) as f32 * TILE_SIZE - 3.0,
            (max_y - min_y + 1) as f32 * TILE_SIZE - 3.0,
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            2.4,
            Color::from_rgba(90, 240, 210, 235),
        );
    }

    let player_pos = tile_center(map.player_spawn);
    draw_circle_lines(
        player_pos.x,
        player_pos.y,
        10.0,
        2.2,
        Color::from_rgba(95, 230, 120, 240),
    );
    draw_text(
        "P",
        player_pos.x - 4.5,
        player_pos.y + 5.0,
        18.0,
        Color::from_rgba(95, 230, 120, 240),
    );

    let npc_pos = tile_center(map.npc_spawn);
    draw_circle_lines(
        npc_pos.x,
        npc_pos.y,
        10.0,
        2.2,
        Color::from_rgba(255, 160, 95, 240),
    );
    draw_text(
        "N",
        npc_pos.x - 5.0,
        npc_pos.y + 5.0,
        18.0,
        Color::from_rgba(255, 160, 95, 240),
    );

    set_default_camera();

    draw_rectangle_lines(
        map_slot_rect.x + 0.5,
        map_slot_rect.y + 0.5,
        map_slot_rect.w - 1.0,
        map_slot_rect.h - 1.0,
        1.8,
        Color::from_rgba(90, 126, 149, 170),
    );
    draw_rectangle_lines(
        map_view_rect.x + 0.5,
        map_view_rect.y + 0.5,
        map_view_rect.w - 1.0,
        map_view_rect.h - 1.0,
        2.2,
        Color::from_rgba(170, 213, 237, 220),
    );

    draw_ambient_dust(palette, time);
    draw_vignette(palette);

    draw_rectangle(
        top_bar_rect.x,
        top_bar_rect.y,
        top_bar_rect.w,
        top_bar_rect.h,
        Color::from_rgba(10, 18, 26, 228),
    );
    draw_rectangle_lines(
        top_bar_rect.x + 0.5,
        top_bar_rect.y + 0.5,
        top_bar_rect.w - 1.0,
        top_bar_rect.h - 1.0,
        1.8,
        Color::from_rgba(92, 133, 162, 238),
    );
    draw_text(
        "EDITEUR USINE",
        top_bar_rect.x + 16.0,
        top_bar_rect.y + 26.0,
        30.0,
        Color::from_rgba(230, 242, 250, 255),
    );
    draw_text(
        &format!(
            "{} | {}x{} | props {}",
            map.label,
            map.world.w,
            map.world.h,
            map.props.len()
        ),
        top_bar_rect.x + 16.0,
        top_bar_rect.y + 48.0,
        18.0,
        Color::from_rgba(176, 206, 223, 255),
    );

    let top_button_h = 34.0;
    let top_button_w = 102.0;
    let top_button_gap = 8.0;
    let top_button_y = top_bar_rect.y + (top_bar_rect.h - top_button_h) * 0.5;
    let mut right_cursor = top_bar_rect.x + top_bar_rect.w - 12.0;
    right_cursor -= top_button_w;
    let load_rect = Rect::new(right_cursor, top_button_y, top_button_w, top_button_h);
    right_cursor -= top_button_gap + top_button_w;
    let save_rect = Rect::new(right_cursor, top_button_y, top_button_w, top_button_h);
    right_cursor -= top_button_gap + top_button_w;
    let menu_rect = Rect::new(right_cursor, top_button_y, top_button_w, top_button_h);
    right_cursor -= top_button_gap + top_button_w;
    let play_rect = Rect::new(right_cursor, top_button_y, top_button_w, top_button_h);

    let mut action = EditorAction::None;
    if draw_ui_button(play_rect, "Jouer F5", mouse, left_pressed, false)
        || is_key_pressed(KeyCode::F5)
    {
        sanitize_map_asset(map);
        action = EditorAction::StartPlay;
    }
    if draw_ui_button(menu_rect, "Menu Esc", mouse, left_pressed, false)
        || is_key_pressed(KeyCode::Escape)
    {
        action = EditorAction::BackToMenu;
    }
    if draw_ui_button(save_rect, "Sauver", mouse, left_pressed, false) {
        editor_save_current_map(editor, map);
    }
    if draw_ui_button(load_rect, "Charger", mouse, left_pressed, false) {
        editor_load_current_map(editor, map);
    }

    draw_rectangle(
        left_panel_rect.x,
        left_panel_rect.y,
        left_panel_rect.w,
        left_panel_rect.h,
        Color::from_rgba(9, 15, 22, 222),
    );
    draw_rectangle_lines(
        left_panel_rect.x + 0.5,
        left_panel_rect.y + 0.5,
        left_panel_rect.w - 1.0,
        left_panel_rect.h - 1.0,
        1.8,
        Color::from_rgba(88, 124, 146, 232),
    );

    draw_rectangle(
        right_panel_rect.x,
        right_panel_rect.y,
        right_panel_rect.w,
        right_panel_rect.h,
        Color::from_rgba(9, 15, 22, 222),
    );
    draw_rectangle_lines(
        right_panel_rect.x + 0.5,
        right_panel_rect.y + 0.5,
        right_panel_rect.w - 1.0,
        right_panel_rect.h - 1.0,
        1.8,
        Color::from_rgba(88, 124, 146, 232),
    );

    draw_text(
        "TOOLBOX",
        left_panel_rect.x + 14.0,
        left_panel_rect.y + 24.0,
        24.0,
        Color::from_rgba(214, 232, 244, 255),
    );

    let left_pad = 14.0;
    let left_content_w = left_panel_rect.w - left_pad * 2.0;
    let undo_w = ((left_content_w - 8.0) * 0.5).max(80.0);
    let undo_rect = Rect::new(
        left_panel_rect.x + left_pad,
        left_panel_rect.y + 34.0,
        undo_w,
        30.0,
    );
    let redo_rect = Rect::new(
        undo_rect.x + undo_rect.w + 8.0,
        left_panel_rect.y + 34.0,
        undo_w,
        30.0,
    );
    if draw_ui_button_sized(undo_rect, "Annuler", mouse, left_pressed, false, 14.0) {
        let _ = editor_undo(editor, map);
    }
    if draw_ui_button_sized(redo_rect, "Retablir", mouse, left_pressed, false, 14.0) {
        let _ = editor_redo(editor, map);
    }

    let tool_label_y = left_panel_rect.y + 86.0;
    draw_text(
        "Outil",
        left_panel_rect.x + 14.0,
        tool_label_y,
        20.0,
        Color::from_rgba(190, 216, 231, 255),
    );
    let tool_brush_rect = Rect::new(
        left_panel_rect.x + left_pad,
        tool_label_y + 8.0,
        left_content_w,
        30.0,
    );
    let tool_rect_rect = Rect::new(
        left_panel_rect.x + left_pad,
        tool_label_y + 44.0,
        left_content_w,
        30.0,
    );
    if draw_ui_button_sized(
        tool_brush_rect,
        "Pinceau (B)",
        mouse,
        left_pressed,
        editor.tool == EditorTool::Brush,
        14.0,
    ) {
        editor.tool = EditorTool::Brush;
    }
    if draw_ui_button_sized(
        tool_rect_rect,
        "Rectangle (R)",
        mouse,
        left_pressed,
        editor.tool == EditorTool::Rect,
        14.0,
    ) {
        editor.tool = EditorTool::Rect;
    }

    let brushes = [
        (EditorBrush::Floor, "1 Sol"),
        (EditorBrush::FloorMetal, "2 Sol metal"),
        (EditorBrush::FloorWood, "3 Sol bois"),
        (EditorBrush::FloorMoss, "4 Sol mousse"),
        (EditorBrush::FloorSand, "5 Sol sable"),
        (EditorBrush::Wall, "6 Mur"),
        (EditorBrush::WallBrick, "7 Mur brique"),
        (EditorBrush::WallSteel, "8 Mur acier"),
        (EditorBrush::WallNeon, "9 Mur neon"),
        (EditorBrush::Crate, "0 Caisse"),
        (EditorBrush::Pipe, "Q Tuyau"),
        (EditorBrush::Lamp, "W Lampe"),
        (EditorBrush::Banner, "E Banniere"),
        (EditorBrush::Plant, "T Plante"),
        (EditorBrush::Bench, "Y Banc"),
        (EditorBrush::Crystal, "U Cristal"),
        (EditorBrush::EraseProp, "X Effacer"),
    ];

    let brush_title_y = tool_label_y + 90.0;
    draw_text(
        "Pinceaux",
        left_panel_rect.x + 14.0,
        brush_title_y,
        20.0,
        Color::from_rgba(190, 216, 231, 255),
    );
    let brush_columns = if left_panel_rect.w > 250.0 { 2 } else { 1 };
    let brush_gap = 8.0;
    let brush_button_w = ((left_content_w - brush_gap * (brush_columns as f32 - 1.0))
        / brush_columns as f32)
        .max(78.0);
    let brush_button_h = 22.0;
    let brush_step_y = 25.0;
    for (i, (brush, label)) in brushes.iter().enumerate() {
        let row = i / brush_columns;
        let col = i % brush_columns;
        let rect = Rect::new(
            left_panel_rect.x + left_pad + col as f32 * (brush_button_w + brush_gap),
            brush_title_y + 8.0 + row as f32 * brush_step_y,
            brush_button_w,
            brush_button_h,
        );
        if draw_ui_button_sized(
            rect,
            label,
            mouse,
            left_pressed,
            editor.brush == *brush,
            12.0,
        ) {
            editor.brush = *brush;
        }
    }
    let brush_rows = brushes.len().div_ceil(brush_columns);
    let left_help_y = brush_title_y + 8.0 + brush_rows as f32 * brush_step_y + 10.0;

    let toggle_grid_rect = Rect::new(
        left_panel_rect.x + left_pad,
        left_help_y,
        left_content_w,
        28.0,
    );
    if draw_ui_button_sized(
        toggle_grid_rect,
        if editor.show_grid {
            "Grille: ON (G)"
        } else {
            "Grille: OFF (G)"
        },
        mouse,
        left_pressed,
        editor.show_grid,
        13.0,
    ) {
        editor.show_grid = !editor.show_grid;
    }

    draw_text(
        "INSPECTOR",
        right_panel_rect.x + 14.0,
        right_panel_rect.y + 24.0,
        24.0,
        Color::from_rgba(214, 232, 244, 255),
    );
    let inspector_text = format!(
        "Actif: {} / {}\nCamera: x={:.0} y={:.0} zoom={:.2}\nViewport tuiles: x[{}..{}] y[{}..{}]\nSpawn joueur: ({}, {})\nSpawn PNJ: ({}, {})",
        editor_tool_label(editor.tool),
        editor_brush_label(editor.brush),
        editor.camera_center.x,
        editor.camera_center.y,
        editor.camera_zoom,
        visible_bounds.0,
        visible_bounds.1,
        visible_bounds.2,
        visible_bounds.3,
        map.player_spawn.0,
        map.player_spawn.1,
        map.npc_spawn.0,
        map.npc_spawn.1,
    );
    draw_text(
        &inspector_text,
        right_panel_rect.x + 14.0,
        right_panel_rect.y + 50.0,
        17.0,
        Color::from_rgba(186, 209, 224, 255),
    );

    if let Some(tile) = editor.hover_tile {
        let tile_kind = map.world.get(tile.0, tile.1);
        let prop_at =
            prop_index_at_tile(&map.props, tile).map(|idx| prop_kind_label(map.props[idx].kind));
        draw_text(
            &format!(
                "Case survolee: ({}, {})\nTuile: {}\nObjet: {}",
                tile.0,
                tile.1,
                tile_label(tile_kind),
                prop_at.unwrap_or("aucun")
            ),
            right_panel_rect.x + 14.0,
            right_panel_rect.y + 160.0,
            18.0,
            Color::from_rgba(214, 232, 244, 255),
        );
    } else {
        draw_text(
            "Case survolee: aucune",
            right_panel_rect.x + 14.0,
            right_panel_rect.y + 160.0,
            18.0,
            Color::from_rgba(166, 188, 204, 255),
        );
    }

    let center_cam_rect = Rect::new(
        right_panel_rect.x + 14.0,
        right_panel_rect.y + 232.0,
        right_panel_rect.w - 28.0,
        30.0,
    );
    if draw_ui_button_sized(
        center_cam_rect,
        "Centrer camera (Home)",
        mouse,
        left_pressed,
        false,
        13.0,
    ) {
        editor.camera_center = tile_center(map.player_spawn);
    }

    let set_player_rect = Rect::new(
        right_panel_rect.x + 14.0,
        right_panel_rect.y + 270.0,
        right_panel_rect.w - 28.0,
        30.0,
    );
    if draw_ui_button_sized(
        set_player_rect,
        "Definir spawn joueur (P)",
        mouse,
        left_pressed,
        false,
        13.0,
    ) && let Some(tile) = editor.hover_tile
        && !map.world.is_solid(tile.0, tile.1)
    {
        map.player_spawn = tile;
        editor_set_status(editor, format!("Point joueur -> ({}, {})", tile.0, tile.1));
    }

    let set_npc_rect = Rect::new(
        right_panel_rect.x + 14.0,
        right_panel_rect.y + 308.0,
        right_panel_rect.w - 28.0,
        30.0,
    );
    if draw_ui_button_sized(
        set_npc_rect,
        "Definir spawn PNJ (N)",
        mouse,
        left_pressed,
        false,
        13.0,
    ) && let Some(tile) = editor.hover_tile
        && !map.world.is_solid(tile.0, tile.1)
    {
        map.npc_spawn = tile;
        editor_set_status(editor, format!("Point PNJ -> ({}, {})", tile.0, tile.1));
    }

    draw_text(
        "Raccourcis:\nCtrl+S/L sauver/charger\nCtrl+Z/Y undo/redo\nF11 plein ecran\nPan: fleches ou Space+ZQSD\nZoom: molette / PageUp/Down\nDrag camera: molette maintenue",
        right_panel_rect.x + 14.0,
        right_panel_rect.y + right_panel_rect.h - 126.0,
        15.0,
        Color::from_rgba(160, 186, 202, 255),
    );

    let status_text = if editor.status_timer > 0.0 {
        editor.status_text.as_str()
    } else {
        "Pret"
    };
    draw_text(
        status_text,
        top_bar_rect.x + 16.0,
        top_bar_rect.y + top_bar_rect.h - 6.0,
        16.0,
        Color::from_rgba(252, 232, 188, 255),
    );

    action
}

#[macroquad::main(window_conf)]
async fn main() {
    let palette = Palette::new();
    let mut map = match load_map_asset(MAP_FILE_PATH) {
        Ok(loaded) => loaded,
        Err(_) => {
            let default_map = MapAsset::new_default();
            let _ = save_map_asset(MAP_FILE_PATH, &default_map);
            default_map
        }
    };
    let loaded_version = map.version;
    sanitize_map_asset(&mut map);
    if map.version != loaded_version {
        let _ = save_map_asset(MAP_FILE_PATH, &map);
    }

    let character_catalog =
        CharacterCatalog::load_default().expect("default character catalog should be valid");
    let mut lineage_seed = 0x51A7_2026_D00D_F00D;
    let mut game_state = build_game_state_from_map(&map, &character_catalog, lineage_seed);
    let mut editor_state = EditorState::new();
    let mut mode = AppMode::MainMenu;
    let mut accumulator = 0.0;
    let mut is_fullscreen_mode = false;

    loop {
        if handle_fullscreen_hotkey(&mut is_fullscreen_mode) {
            editor_set_status(
                &mut editor_state,
                if is_fullscreen_mode {
                    "Plein ecran active"
                } else {
                    "Plein ecran desactive"
                },
            );
        }

        let frame_dt = get_frame_time().min(0.25);
        let time = get_time() as f32;

        mode = match mode {
            AppMode::MainMenu => {
                if let Some(next_mode) = run_main_menu_frame(&map, &palette, time) {
                    match next_mode {
                        AppMode::Playing => {
                            sanitize_map_asset(&mut map);
                            lineage_seed = advance_seed(lineage_seed);
                            game_state =
                                build_game_state_from_map(&map, &character_catalog, lineage_seed);
                            accumulator = 0.0;
                            AppMode::Playing
                        }
                        AppMode::Editor => AppMode::Editor,
                        AppMode::MainMenu => AppMode::MainMenu,
                    }
                } else {
                    AppMode::MainMenu
                }
            }
            AppMode::Playing => match run_play_frame(&mut game_state, frame_dt, &mut accumulator) {
                PlayAction::None => AppMode::Playing,
                PlayAction::BackToMenu => AppMode::MainMenu,
                PlayAction::OpenEditor => {
                    map.world = game_state.world.clone();
                    map.props = game_state.props.clone();
                    map.player_spawn =
                        tile_from_world_clamped(&game_state.world, game_state.player.pos);
                    map.npc_spawn = tile_from_world_clamped(&game_state.world, game_state.npc.pos);
                    sanitize_map_asset(&mut map);
                    AppMode::Editor
                }
            },
            AppMode::Editor => {
                match run_editor_frame(&mut editor_state, &mut map, &palette, time) {
                    EditorAction::None => AppMode::Editor,
                    EditorAction::BackToMenu => AppMode::MainMenu,
                    EditorAction::StartPlay => {
                        lineage_seed = advance_seed(lineage_seed);
                        game_state =
                            build_game_state_from_map(&map, &character_catalog, lineage_seed);
                        accumulator = 0.0;
                        AppMode::Playing
                    }
                }
            }
        };

        if is_key_pressed(KeyCode::F12) {
            sanitize_map_asset(&mut map);
            let _ = save_map_asset(MAP_FILE_PATH, &map);
            editor_set_status(
                &mut editor_state,
                format!("Sauvegarde auto: {}", MAP_FILE_PATH),
            );
        }

        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_hash_is_stable_and_varied() {
        assert_eq!(tile_hash(3, 7), tile_hash(3, 7));
        assert_ne!(tile_hash(3, 7), tile_hash(4, 7));
        assert_ne!(tile_hash(3, 7), tile_hash(3, 8));
    }

    #[test]
    fn room_has_closed_wall_border() {
        let world = World::new_room(10, 6);

        for x in 0..10 {
            assert_eq!(world.get(x, 0), Tile::Wall);
            assert_eq!(world.get(x, 5), Tile::Wall);
        }
        for y in 0..6 {
            assert_eq!(world.get(0, y), Tile::Wall);
            assert_eq!(world.get(9, y), Tile::Wall);
        }
    }

    #[test]
    fn wall_mask_bits_match_neighbors() {
        let mut world = World {
            w: 3,
            h: 3,
            tiles: vec![Tile::Floor; 9],
        };
        world.set(1, 1, Tile::Wall);
        world.set(1, 0, Tile::Wall);
        world.set(2, 1, Tile::Wall);
        world.set(1, 2, Tile::Wall);

        let mask = wall_mask_4(&world, 1, 1);
        assert_eq!(mask, MASK_N | MASK_E | MASK_S);
    }

    #[test]
    fn facing_selection_prefers_vertical_directions() {
        assert_eq!(
            select_character_facing(vec2(0.1, 1.0), CharacterFacing::Side),
            CharacterFacing::Front
        );
        assert_eq!(
            select_character_facing(vec2(-0.2, -1.0), CharacterFacing::Side),
            CharacterFacing::Back
        );
    }

    #[test]
    fn facing_selection_prefers_side_for_horizontal_motion() {
        assert_eq!(
            select_character_facing(vec2(1.0, 0.1), CharacterFacing::Front),
            CharacterFacing::Side
        );
        assert_eq!(
            select_character_facing(vec2(-1.0, -0.05), CharacterFacing::Back),
            CharacterFacing::Side
        );
    }

    #[test]
    fn facing_selection_uses_hysteresis_on_diagonals() {
        assert_eq!(
            select_character_facing(vec2(0.7, 0.7), CharacterFacing::Side),
            CharacterFacing::Side
        );
        assert_eq!(
            select_character_facing(vec2(0.7, 0.7), CharacterFacing::Front),
            CharacterFacing::Front
        );
        assert_eq!(
            select_character_facing(vec2(0.0, 0.0), CharacterFacing::Back),
            CharacterFacing::Back
        );
    }

    #[test]
    fn nearest_walkable_tile_finds_floor_from_blocked_click() {
        let world = World::new_room(12, 8);
        let found = nearest_walkable_tile(&world, (0, 0)).expect("should find floor");
        assert!(!world.is_solid(found.0, found.1));
    }

    #[test]
    fn a_star_finds_path_around_internal_walls() {
        let world = World::new_room(25, 15);
        let path = a_star_path(&world, (2, 2), (22, 12)).expect("path should exist");
        assert!(path.len() > 2);
        assert_eq!(path.first().copied(), Some((2, 2)));
        assert_eq!(path.last().copied(), Some((22, 12)));
        assert!(path.iter().all(|&(x, y)| !world.is_solid(x, y)));
    }

    #[test]
    fn click_command_enables_auto_move_and_keyboard_cancels_it() {
        let world = World::new_room(25, 15);
        let mut player = Player::new(tile_center((2, 2)));

        let moved = issue_auto_move_command(&mut player, &world, (20, 10));
        assert!(moved);
        assert_eq!(player.control_mode, ControlMode::AutoMove);
        assert!(!player.auto.path_world.is_empty());

        apply_control_inputs(&mut player, &world, vec2(1.0, 0.0), None);
        assert_eq!(player.control_mode, ControlMode::Manual);
        assert!(player.auto.path_world.is_empty());
        assert!(player.auto.path_tiles.is_empty());
    }

    #[test]
    fn auto_move_progresses_along_path() {
        let world = World::new_room(25, 15);
        let mut player = Player::new(tile_center((2, 2)));
        let _ = issue_auto_move_command(&mut player, &world, (20, 10));
        let before = player.pos;

        for _ in 0..120 {
            update_player(&mut player, &world, Vec2::ZERO, FIXED_DT);
        }

        assert!(player.pos.distance(before) > TILE_SIZE);
    }

    #[test]
    fn npc_wander_command_creates_path() {
        let world = World::new_room(25, 15);
        let mut npc = NpcWanderer::new(tile_center((4, 4)), 77);
        let ok = issue_npc_wander_command(&mut npc, &world, (20, 10));
        assert!(ok);
        assert!(!npc.auto.path_world.is_empty());
        assert!(npc.auto.target_tile.is_some());
    }

    #[test]
    fn npc_greeting_triggers_when_player_is_close() {
        let world = World::new_room(25, 15);
        let mut npc = NpcWanderer::new(tile_center((6, 6)), 99);
        let mut player = Player::new(tile_center((6, 6)));
        player.half = vec2(10.0, 14.0);
        npc.bubble_cooldown = 0.0;

        update_npc_wanderer(&mut npc, &world, &player, FIXED_DT);

        assert!(npc.bubble_timer > 0.0);
        assert!(npc.bubble_cooldown > 0.0);
    }

    #[test]
    fn npc_wanderer_leaves_idle_and_moves_over_time() {
        let world = World::new_room(25, 15);
        let mut npc = NpcWanderer::new(tile_center((6, 6)), 1234);
        let player = Player::new(tile_center((2, 2)));
        let start = npc.pos;
        npc.idle_timer = 0.0;

        let mut had_path = false;
        let mut walked = false;

        for _ in 0..360 {
            update_npc_wanderer(&mut npc, &world, &player, FIXED_DT);
            had_path |= !npc.auto.path_world.is_empty();
            walked |= npc.is_walking;
        }

        assert!(had_path, "npc should pick at least one wander path");
        assert!(walked, "npc should enter walking state");
        assert!(
            npc.pos.distance(start) > TILE_SIZE * 0.25,
            "npc should move away from spawn point"
        );
    }

    #[test]
    fn map_asset_roundtrip_serialization_preserves_content() {
        let mut map = MapAsset::new_default();
        assert!(set_map_tile(&mut map, (3, 3), Tile::WallBrick));
        assert!(set_prop_at_tile(&mut map, (4, 4), PropKind::Crystal));
        map.player_spawn = (2, 2);
        map.npc_spawn = (7, 7);

        let encoded = serialize_map_asset(&map).expect("map should serialize");
        let decoded = deserialize_map_asset(&encoded).expect("map should deserialize");

        assert_eq!(decoded.world.w, map.world.w);
        assert_eq!(decoded.world.h, map.world.h);
        assert_eq!(decoded.world.get(3, 3), Tile::WallBrick);
        assert!(prop_index_at_tile(&decoded.props, (4, 4)).is_some());
        assert!(
            !decoded
                .world
                .is_solid(decoded.player_spawn.0, decoded.player_spawn.1)
        );
        assert!(
            !decoded
                .world
                .is_solid(decoded.npc_spawn.0, decoded.npc_spawn.1)
        );
    }

    #[test]
    fn sanitize_map_enforces_border_and_removes_invalid_props() {
        let mut map = MapAsset::new_default();
        map.world.set(0, 3, Tile::Floor);
        map.world.set(5, 0, Tile::Floor);
        map.player_spawn = (0, 0);
        map.npc_spawn = (0, 0);
        map.props.push(Prop {
            tile_x: 0,
            tile_y: 0,
            kind: PropKind::Crate,
            phase: 0.0,
        });

        sanitize_map_asset(&mut map);

        assert_eq!(map.world.get(0, 3), Tile::Wall);
        assert_eq!(map.world.get(5, 0), Tile::Wall);
        assert!(prop_index_at_tile(&map.props, (0, 0)).is_none());
        assert!(!map.world.is_solid(map.player_spawn.0, map.player_spawn.1));
        assert!(!map.world.is_solid(map.npc_spawn.0, map.npc_spawn.1));
    }

    #[test]
    fn sanitize_upgrades_legacy_small_map_to_starter_factory_layout() {
        let mut map = MapAsset {
            version: 1,
            label: "Legacy".to_string(),
            world: World::new_room(25, 15),
            props: Vec::new(),
            player_spawn: (2, 2),
            npc_spawn: (20, 10),
        };

        sanitize_map_asset(&mut map);

        assert_eq!(map.version, MAP_FILE_VERSION);
        assert_eq!(map.world.w, MAP_W);
        assert_eq!(map.world.h, MAP_H);
        assert_eq!(map.label, "Starter Factory");
    }

    #[test]
    fn editor_brush_can_paint_floor_and_wall() {
        let mut map = MapAsset::new_default();

        assert!(editor_apply_brush(&mut map, EditorBrush::WallSteel, (6, 6)));
        assert_eq!(map.world.get(6, 6), Tile::WallSteel);
        assert!(editor_apply_brush(&mut map, EditorBrush::FloorMoss, (6, 6)));
        assert_eq!(map.world.get(6, 6), Tile::FloorMoss);
    }

    #[test]
    fn editor_brush_can_place_and_remove_props() {
        let mut map = MapAsset::new_default();
        let tile = (6, 6);

        assert!(editor_apply_brush(&mut map, EditorBrush::Crystal, tile));
        assert!(prop_index_at_tile(&map.props, tile).is_some());
        assert!(editor_apply_brush(&mut map, EditorBrush::EraseProp, tile));
        assert!(prop_index_at_tile(&map.props, tile).is_none());
    }

    #[test]
    fn all_new_wall_variants_are_solid() {
        let mut world = World::new_room(8, 8);
        world.set(3, 3, Tile::WallBrick);
        world.set(4, 3, Tile::WallSteel);
        world.set(5, 3, Tile::WallNeon);

        assert!(world.is_solid(3, 3));
        assert!(world.is_solid(4, 3));
        assert!(world.is_solid(5, 3));
        assert!(!world.is_solid(2, 2));
    }

    #[test]
    fn editor_undo_redo_restores_map_state() {
        let mut map = MapAsset::new_default();
        let mut editor = EditorState::new();
        let tile = (7, 7);
        let initial = map.world.get(tile.0, tile.1);

        editor_push_undo(&mut editor, &map);
        assert!(editor_apply_brush(&mut map, EditorBrush::WallNeon, tile));
        assert_eq!(map.world.get(tile.0, tile.1), Tile::WallNeon);

        assert!(editor_undo(&mut editor, &mut map));
        assert_eq!(map.world.get(tile.0, tile.1), initial);

        assert!(editor_redo(&mut editor, &mut map));
        assert_eq!(map.world.get(tile.0, tile.1), Tile::WallNeon);
    }
}
