Parfait 😄 On va démarrer “proprement” avec un socle qui colle déjà à l’esprit RimWorld-like / grille / simulation continue de Rxchixs : une pièce (sol + murs) = des “blocs” physiques sur grille, et un personnage “agent” qui bouge dedans avec collision + animation. Ça correspond exactement à ton cadre “blocs sur la grille” (sols, murs, etc.)

environnement-du-jeu

et c’est la base sur laquelle on branchera ensuite les zones logiques, jobs, réservations, etc.

Vision-générale-du-jeu



Jobs-–-Système-&-Priorités

Je te donne :

un plan ultra détaillé (quoi faire, dans quel ordre, pourquoi),

le code complet copy-paste (Cargo.toml + src/main.rs),

les contrôles clavier et ce que tu dois voir à l’écran.

On part sur macroquad (simple et efficace pour un proto 2D), et on garde déjà le “tick fixe” (60 Hz) séparé du rendu, comme tu l’exiges. Macroquad est actuellement en v0.4.14 sur crates.io , et on utilise ses APIs “textures from bytes” et “draw_texture_ex” .

Plan détaillé (checklist “absolument tout”)

A. Préparer le projet Rust (une seule fois)

Installe Rust stable via rustup (côté machine).

Crée un projet :

cargo new rxchixs --bin

Ouvre le dossier dans RustRover.

Dans Cargo.toml, ajoute macroquad (et on pose déjà serde+ron pour la suite “blueprints layout data-driven”, même si on ne les utilise pas encore). Serde est en 1.0.x , ron est en 0.12.0 .

B. Mettre en place la boucle de jeu “propre”
Objectif : séparation “simulation” vs “rendu”, tick fixe à 60 Hz.

Une boucle principale macroquad (async main).

Chaque frame :

lire input,

accumuler le temps (get_frame_time() renvoie la durée du dernier frame en secondes ),

exécuter 0..N steps de simulation à dt fixe (1/60),

dessiner l’état courant.

Pourquoi : c’est exactement la fondation requise pour une simulation “aquarium” (le monde tourne tout seul, stable) . Et c’est indispensable pour l’IA + pathfinding + réservations plus tard.

C. Représenter la “pièce” en grille (sol + murs)
Objectif : une carte tile-based, source de vérité pour collisions (et plus tard pathfinding).

Constantes :

TILE_SIZE = 32

MAP_W, MAP_H

enu:contentReference[oaicite:9]{index=9}}

struct World { w, h, tiles: Vec<Tile> }

World::new_room() :

remplit en Floor,

met les bords en Wall (ça crée la pièce),

ajoute 1–2 murs internes pour tester collisions.

Pourquoi : “blocs physiques sur la grille” est la base de Rxchixs (sols, murs, machines, etc.) .

D. Implémenter le personnage (entity) + collisions
Objectif : un agent qui se déplace, ne traverse pas les murs.

struct Player :

position (Vec2) en pixels (centre du collider),

half-size collider (Vec2),

speed,

direction (juste gauche/droite pour flip),
-

environnement-du-jeu

r, frame).

Input : WASD + flèches → vecteur direction normalisé.

Déplacement :

delta = input * speed * dt

Collisions AABB contre les tiles Wall :

mouvement séparé par axes (X puis Y) pour éviter les “accrochages” aux coins,

correction en poussant l’agent au bord du mur quand collision.

Pourquoi : ça te donne immédiatement le comportement “RimWorld-like” (agents qui respectent les obstacles) et c’est la base du pathfinding A* (qui viendra juste après).

E. Animation du personnage (sans assets externes)
Objectif : un sprite animé “placeholder” sans fichiers externes.

On génère 3–4 frames en code, via ASCII-art → RGBA bytes → Texture2D::from_rgba8 .

On dessine avec draw_texture_ex(&texture, ...) .

On change de frame quand le joueur bouge.

Pourquoi : tu veux “assets pas requis”, donc on reste 100% code.

F. Debug overlay minimal (super important dès le début)
Objectif : visibilité sur ce qui se passe.

Afficher :

position en pixels,

position en tile (tx, ty),

état animation,

mode debug toggle (F1).

Option : dessiner le rectangle de collision.

Pourquoi : dans Rxchixs, tout doit être “débuggable” et lisible (même pour l’IA plus tard) .

Code prêt à copier-coller

2.1 Cargo.toml (remplace le tien)

[package]
name = "rxchixs"
version = "0.1.0"
edition = "2021"

[dependencies]
macroquad = "0.4.14"
serde = { version = "1.0", features = ["derive"] }
ron = "0.12"


Note : macroquad v0.4.14 est bien référencée sur crates.io . Serde 1.0.x et ron 0.12.0 existent bien .

2.2 src/main.rs (fichier complet)

use macroquad::texture::FilterMode;

// -----------------------------
// Config / constantes
// -----------------------------

const TILE_SIZE: f32 = 32.0;
const MAP_W: i32 = 25;
const MAP_H: i32 = 15;

const FIXED_DT: f32 = 1.0 / 60.0;

fn window_conf() -> Conf {
Conf {
window_title: "Rxchixs - Room Prototype".to_string(),
window_width: (MAP_W as f32 * TILE_SIZE) as i32,
window_height: (MAP_H as f32 * TILE_SIZE) as i32,
window_resizable: false,
..Default::default()
}
}

// -----------------------------
// Monde grille (sol/murs)
// -----------------------------

#[derive(Copy, Clone, PartialEq, Eq)]
enum Tile {
Floor,
Wall,
}

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

        // Murs bordure = pièce fermée
        for x in 0..w {
            world.set(x, 0, Tile::Wall);
            world.set(x, h - 1, Tile::Wall);
        }
        for y in 0..h {
            world.set(0, y, Tile::Wall);
            world.set(w - 1, y, Tile::Wall);
        }

        // Quelques obstacles internes (pour tester collisions)
        // Un mur vertical
        for y in 4..11 {
            world.set(12, y, Tile::Wall);
        }
        // Un petit mur horizontal
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
            return Tile::Wall; // hors carte = solide (pratique)
        }
        self.tiles[self.idx(x, y)]
    }

    fn set(&mut self, x: i32, y: i32, t: Tile) {
        if self.in_bounds(x, y) {
            let i = self.idx(x, y);
            self.tiles[i] = t;
        }
    }

    fn is_solid(&self, x: i32, y: i32) -> bool {
        self.get(x, y) == Tile::Wall
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

// -----------------------------
// Collisions AABB
// -----------------------------

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
        self.min.x < r.x + r.w
            && self.max.x > r.x
            && self.min.y < r.y + r.h
            && self.max.y > r.y
    }
}

fn clamp_i32(v: i32, lo: i32, hi: i32) -> i32 {
v.max(lo).min(hi)
}

fn tiles_overlapping_aabb(world: &World, aabb: Aabb) -> (i32, i32, i32, i32) {
// On prend une marge minuscule pour éviter qu'un bord pile sur la ligne de tile
// fasse varier le floor/ceil.
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

// -----------------------------
// Sprites (générés en code)
// -----------------------------

struct PlayerSprites {
idle: Texture2D,
walk: Vec<Texture2D>, // frames de marche
}

fn texture_from_ascii(art: &[&str]) -> Texture2D {
let h = art.len();
assert!(h > 0, "ASCII art vide");
let w = art[0].chars().count();
assert!(w > 0, "ASCII art largeur 0");

    for (i, line) in art.iter().enumerate() {
        assert!(
            line.chars().count() == w,
            "Ligne ASCII art {} a une largeur différente",
            i
        );
    }

    let mut bytes: Vec<u8> = Vec::with_capacity(w * h * 4);

    for line in art {
        for c in line.chars() {
            let (r, g, b, a) = match c {
                '.' => (0, 0, 0, 0),                 // transparent
                '#' => (20, 20, 20, 255),            // outline
                '@' => (240, 200, 80, 255),          // body
                'o' => (255, 255, 255, 255),         // highlight
                _ => (255, 0, 255, 255),             // magenta = debug
            };
            bytes.extend_from_slice(&[r, g, b, a]);
        }
    }

    // Texture2D::from_rgba8 existe bien et attend width/height en u16 + bytes RGBA :contentReference[oaicite:18]{index=18}
    let tex = Texture2D::from_rgba8(w as u16, h as u16, &bytes);
    tex.set_filter(FilterMode::Nearest);
    tex
}

fn make_player_sprites() -> PlayerSprites {
// 16x16, sera upscale en 32x32 à l'affichage
const IDLE: [&str; 16] = [
"................",
"................",
".......##.......",
"......#@@#......",
".....#@@@@#.....",
".....#@@@@#.....",
"......#@@#......",
".......##.......",
"......#@@#......",
".....#@@@@#.....",
".....#@..@#.....",
".....#@..@#.....",
"......#..#......",
"......#..#......",
"................",
"................",
];

    const WALK1: [&str; 16] = [
        "................",
        "................",
        ".......##.......",
        "......#@@#......",
        ".....#@@@@#.....",
        ".....#@@@@#.....",
        "......#@@#......",
        ".......##.......",
        "......#@@#......",
        ".....#@@@@#.....",
        "....#@@..@@#....",
        "....#@@..@@#....",
        ".....#....#.....",
        "......#..#......",
        "......#..#......",
        "................",
    ];

    const WALK2: [&str; 16] = [
        "................",
        "................",
        ".......##.......",
        "......#@@#......",
        ".....#@@@@#.....",
        ".....#@@@@#.....",
        "......#@@#......",
        ".......##.......",
        "......#@@#......",
        ".....#@@@@#.....",
        ".....#@..@#.....",
        "....#@@..@@#....",
        "......#..#......",
        ".....#....#.....",
        "......#..#......",
        "................",
    ];

    let idle = texture_from_ascii(&IDLE);
    let walk1 = texture_from_ascii(&WALK1);
    let walk2 = texture_from_ascii(&WALK2);

    PlayerSprites {
        idle,
        walk: vec![walk1, walk2],
    }
}

// -----------------------------
// Player
// -----------------------------

struct Player {
pos: Vec2,       // centre du collider (en pixels)
half: Vec2,      // demi-taille collider
speed: f32,      // pixels/s
facing_left: bool,

    is_walking: bool,
    anim_timer: f32,
    anim_frame: usize,
}

impl Player {
fn new(pos: Vec2) -> Self {
Self {
pos,
half: vec2(9.0, 11.0), // collider un peu plus petit que le sprite
speed: 120.0,
facing_left: false,

            is_walking: false,
            anim_timer: 0.0,
            anim_frame: 0,
        }
    }

    fn aabb(&self) -> Aabb {
        Aabb::from_center(self.pos, self.half)
    }
}

fn read_input_dir() -> Vec2 {
let mut dir = vec2(0.0, 0.0);

    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
        dir.y -= 1.0;
    }
    if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
        dir.y += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
        dir.x -= 1.0;
    }
    if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
        dir.x += 1.0;
    }

    if dir.length_squared() > 0.0 {
        dir = dir.normalize();
    }
    dir
}

fn move_with_collision(player: &mut Player, world: &World, delta: Vec2) {
// Déplacement séparé par axes (plus stable qu'un seul sweep)
if delta.x != 0.0 {
player.pos.x += delta.x;
let mut aabb = player.aabb();
let (min_tx, max_tx, min_ty, max_ty) = tiles_overlapping_aabb(world, aabb);

        for ty in min_ty..=max_ty {
            for tx in min_tx..=max_tx {
                if !world.is_solid(tx, ty) {
                    continue;
                }
                let r = World::tile_rect(tx, ty);
                if aabb.intersects_rect(r) {
                    if delta.x > 0.0 {
                        // on va à droite : colle à la gauche du mur
                        player.pos.x = r.x - player.half.x;
                    } else {
                        // on va à gauche : colle à la droite du mur
                        player.pos.x = r.x + r.w + player.half.x;
                    }
                    aabb = player.aabb();
                }
            }
        }
    }

    if delta.y != 0.0 {
        player.pos.y += delta.y;
        let mut aabb = player.aabb();
        let (min_tx, max_tx, min_ty, max_ty) = tiles_overlapping_aabb(world, aabb);

        for ty in min_ty..=max_ty {
            for tx in min_tx..=max_tx {
                if !world.is_solid(tx, ty) {
                    continue;
                }
                let r = World::tile_rect(tx, ty);
                if aabb.intersects_rect(r) {
                    if delta.y > 0.0 {
                        // on va en bas : colle en haut du mur
                        player.pos.y = r.y - player.half.y;
                    } else {
                        // on va en haut : colle en bas du mur
                        player.pos.y = r.y + r.h + player.half.y;
                    }
                    aabb = player.aabb();
                }
            }
        }
    }
}

fn update_player(player: &mut Player, world: &World, input_dir: Vec2, dt: f32) {
let moving = input_dir.length_squared() > 0.0;
player.is_walking = moving;

    // Facing (left/right) : on ne change que si input horizontal
    if input_dir.x < -0.01 {
        player.facing_left = true;
    } else if input_dir.x > 0.01 {
        player.facing_left = false;
    }

    let delta = input_dir * player.speed * dt;
    move_with_collision(player, world, delta);

    // Animation : 2 frames de marche (simple), idle = frame fixe
    if player.is_walking {
        player.anim_timer += dt;
        let frame_duration = 0.12; // ~8 fps
        if player.anim_timer >= frame_duration {
            player.anim_timer -= frame_duration;
            player.anim_frame = (player.anim_frame + 1) % 2;
        }
    } else {
        player.anim_timer = 0.0;
        player.anim_frame = 0;
    }
}

// -----------------------------
// Rendering
// -----------------------------

fn draw_world(world: &World) {
for y in 0..world.h {
for x in 0..world.w {
let px = x as f32 * TILE_SIZE;
let py = y as f32 * TILE_SIZE;

            match world.get(x, y) {
                Tile::Floor => {
                    // léger damier pour lire la grille
                    let c = if (x + y) % 2 == 0 {
                        Color::from_rgba(70, 70, 74, 255)
                    } else {
                        Color::from_rgba(62, 62, 66, 255)
                    };
                    draw_rectangle(px, py, TILE_SIZE, TILE_SIZE, c);
                }
                Tile::Wall => {
                    draw_rectangle(px, py, TILE_SIZE, TILE_SIZE, Color::from_rgba(30, 30, 35, 255));
                    draw_rectangle_lines(px, py, TILE_SIZE, TILE_SIZE, 1.0, Color::from_rgba(10, 10, 12, 255));
                }
            }
        }
    }
}

fn draw_player(player: &Player, sprites: &PlayerSprites, debug: bool) {
let sprite_w = 32.0;
let sprite_h = 32.0;

    let tex = if player.is_walking {
        &sprites.walk[player.anim_frame]
    } else {
        &sprites.idle
    };

    // Ancrage "pieds" : le bas du sprite = bas du collider
    let draw_x = player.pos.x - sprite_w / 2.0;
    let draw_y = (player.pos.y + player.half.y) - sprite_h;

    draw_texture_ex(
        tex,
        draw_x,
        draw_y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(sprite_w, sprite_h)),
            flip_x: player.facing_left,
            ..Default::default()
        },
    );

    if debug {
        let aabb = player.aabb();
        draw_rectangle_lines(
            aabb.min.x,
            aabb.min.y,
            aabb.max.x - aabb.min.x,
            aabb.max.y - aabb.min.y,
            1.0,
            RED,
        );
    }
}

// -----------------------------
// Game State
// -----------------------------

struct GameState {
world: World,
player: Player,
sprites: PlayerSprites,
debug: bool,
last_input: Vec2,
}

impl GameState {
fn new() -> Self {
let world = World::new_room(MAP_W, MAP_H);

        // Spawn joueur dans la pièce (tile 2,2)
        let spawn = vec2((2.5 * TILE_SIZE), (2.5 * TILE_SIZE));
        let player = Player::new(spawn);

        let sprites = make_player_sprites();

        Self {
            world,
            player,
            sprites,
            debug: true,
            last_input: vec2(0.0, 0.0),
        }
    }
}

// -----------------------------
// Main loop
// -----------------------------

#[macroquad::main(window_conf)]
async fn main() {
let mut state = GameState::new();

    let mut accumulator = 0.0_f32;

    loop {
        // Toggle debug
        if is_key_pressed(KeyCode::F1) {
            state.debug = !state.debug;
        }

        // Macroquad: get_frame_time() = durée du dernier frame en secondes :contentReference[oaicite:19]{index=19}
        let frame_dt = get_frame_time().min(0.25); // anti "spiral of death"
        accumulator += frame_dt;

        // Input lu 1 fois par frame (ok pour un proto)
        state.last_input = read_input_dir();

        // Simulation fixed tick
        while accumulator >= FIXED_DT {
            update_player(&mut state.player, &state.world, state.last_input, FIXED_DT);
            accumulator -= FIXED_DT;
        }

        // Render
        clear_background(Color::from_rgba(18, 18, 22, 255));
        draw_world(&state.world);
        draw_player(&state.player, &state.sprites, state.debug);

        // Debug overlay (texte)
        if state.debug {
            let tx = (state.player.pos.x / TILE_SIZE).floor() as i32;
            let ty = (state.player.pos.y / TILE_SIZE).floor() as i32;

            // surligner la tile du perso
            let rect = World::tile_rect(tx, ty);
            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, YELLOW);

            let info = format!(
                "F1: debug on/off\npos(px)=({:.1},{:.1}) tile=({},{})\nwalking={} frame={} facing_left={}\ninput=({:.2},{:.2})",
                state.player.pos.x,
                state.player.pos.y,
                tx,
                ty,
                state.player.is_walking,
                state.player.anim_frame,
                state.player.facing_left,
                state.last_input.x,
                state.last_input.y
            );
            draw_text(&info, 12.0, 20.0, 18.0, WHITE);
        }

        next_frame().await;
    }
}


Comment exécuter + contrôles

Dans le dossier du projet :

cargo run

Contrôles :

Déplacement : ZQSD (WASD) ou flèches

F1 : toggle debug (collider + texte + surbrillance de tile)

Ce que tu dois voir :

Une pièce rectangulaire en tiles (sol en damier, murs autour).

Deux obstacles internes (un mur vertical + un mur horizontal).

Un personnage pixel-art “placeholder” qui s’anime quand tu bouges et qui ne traverse pas les murs.

Pourquoi ce prototype est “le bon premier pas” pour Rxchixs

Tu viens d’implémenter exactement les briques nécessaires au futur système :

Grille + “blocs” physiques (sol/mur) = la base de tout placement/édition/construction

Tick fixe séparé du rendu = simulation aquarium stable, essentielle pour que “l’usine tourne” sans le joueur

Un “agent” qui se déplace + collision = la pré-brique pour A*, réservations, jobs, interruptions/reprise

Si tu veux, prochaine étape logique (sans refaire l’architecture) : on remplace World::new_room() par un chargement RON d’un blueprint de pièce (sols/murs/portes) pour coller à ton exigence “layout de départ chargé depuis des données” 😄