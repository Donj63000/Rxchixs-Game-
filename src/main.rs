mod character;
mod deplacement;
mod edition;
mod modes;
mod rendu;
mod sim;
mod ui_pawns;
mod utilitaires;

use character::{
    CharacterCatalog, CharacterFacing, CharacterRecord, CharacterRenderParams,
    build_lineage_preview, compact_visual_summary, draw_character, inspector_lines,
};
use deplacement::*;
use edition::*;
use macroquad::prelude::*;
use modes::*;
use rendu::*;
use ron::{
    de::from_str as ron_from_str,
    ser::{PrettyConfig, to_string_pretty as ron_to_string_pretty},
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::fs;
use std::path::Path;
use ui_pawns::*;
use utilitaires::*;

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
    sim_worker_character: CharacterRecord,
    pawns: Vec<PawnCard>,
    pawn_ui: PawnsUiState,
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
