use super::*;
use std::cell::RefCell;

#[derive(Clone)]
struct FloorTileTextures {
    floor: Option<Texture2D>,
    floor_metal: Option<Texture2D>,
}

thread_local! {
    static FLOOR_TILE_TEXTURES: RefCell<FloorTileTextures> = const {
        RefCell::new(FloorTileTextures {
            floor: None,
            floor_metal: None,
        })
    };
    static POT_DE_FLEUR_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static STORAGE_RAW_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static MAIN_MENU_BACKGROUND_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
}

pub(crate) fn set_floor_tile_textures(floor: Option<Texture2D>, floor_metal: Option<Texture2D>) {
    FLOOR_TILE_TEXTURES.with(|slot| {
        let prepared_floor = floor;
        if let Some(tex) = prepared_floor.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }

        let prepared_floor_metal = floor_metal;
        if let Some(tex) = prepared_floor_metal.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }

        *slot.borrow_mut() = FloorTileTextures {
            floor: prepared_floor,
            floor_metal: prepared_floor_metal,
        };
    });
}

fn floor_tile_textures() -> FloorTileTextures {
    FLOOR_TILE_TEXTURES.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_pot_de_fleur_texture(texture: Option<Texture2D>) {
    POT_DE_FLEUR_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn pot_de_fleur_texture() -> Option<Texture2D> {
    POT_DE_FLEUR_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_storage_raw_texture(texture: Option<Texture2D>) {
    STORAGE_RAW_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn storage_raw_texture() -> Option<Texture2D> {
    STORAGE_RAW_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_main_menu_background_texture(texture: Option<Texture2D>) {
    MAIN_MENU_BACKGROUND_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Linear);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn main_menu_background_texture() -> Option<Texture2D> {
    MAIN_MENU_BACKGROUND_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn draw_character_inspector_panel(state: &GameState, time: f32) {
    let panel_w = 380.0;
    let panel_h = 204.0;
    let panel_x = screen_width() - panel_w - 10.0;
    // Keep it below the pawn bar (so the new UI is always usable).
    let panel_y = 10.0 + 74.0 + 10.0;

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
        "Inspecteur personnage (F2 afficher/masquer, F3 regenerer)",
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
                gesture: CharacterGesture::None,
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

pub(crate) fn draw_background(palette: &Palette, time: f32) {
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

pub(crate) fn draw_floor_tile(
    x: i32,
    y: i32,
    tile: Tile,
    palette: &Palette,
    floor_texture: Option<&Texture2D>,
    floor_metal_texture: Option<&Texture2D>,
) {
    let rect = World::tile_rect(x, y);
    let h = tile_hash(x, y);

    let mapped_texture = match tile {
        Tile::Floor => floor_texture,
        Tile::FloorMetal => floor_metal_texture,
        _ => None,
    };

    if let Some(texture) = mapped_texture {
        draw_texture_ex(
            texture,
            rect.x,
            rect.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(rect.w, rect.h)),
                ..Default::default()
            },
        );
        let grime_strength = 0.02 + ((hash_with_salt(x, y, 13) & 0x0F) as f32 / 380.0);
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            with_alpha(palette.floor_grime, grime_strength),
        );
        return;
    }

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

pub(crate) fn draw_floor_layer_region(
    world: &World,
    palette: &Palette,
    bounds: (i32, i32, i32, i32),
) {
    let floor_textures = floor_tile_textures();
    for y in bounds.2..=bounds.3 {
        for x in bounds.0..=bounds.1 {
            let tile = world.get(x, y);
            if tile_is_floor(tile) {
                draw_floor_tile(
                    x,
                    y,
                    tile,
                    palette,
                    floor_textures.floor.as_ref(),
                    floor_textures.floor_metal.as_ref(),
                );
            }
        }
    }
}

pub(crate) fn draw_wall_cast_shadows_region(
    world: &World,
    palette: &Palette,
    bounds: (i32, i32, i32, i32),
) {
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

pub(crate) fn draw_wall_tile(world: &World, x: i32, y: i32, tile: Tile, palette: &Palette) {
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

pub(crate) fn draw_wall_layer_region(
    world: &World,
    palette: &Palette,
    bounds: (i32, i32, i32, i32),
) {
    for y in bounds.2..=bounds.3 {
        for x in bounds.0..=bounds.1 {
            let tile = world.get(x, y);
            if tile_is_wall(tile) {
                draw_wall_tile(world, x, y, tile, palette);
            }
        }
    }
}

pub(crate) fn draw_prop_shadows_region(
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

pub(crate) fn draw_props_region(
    props: &[Prop],
    palette: &Palette,
    time: f32,
    bounds: (i32, i32, i32, i32),
) {
    let pot_texture = pot_de_fleur_texture();
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
                if let Some(texture) = pot_texture.as_ref() {
                    let bob = (time * 1.9 + prop.phase).sin() * 0.35;
                    draw_texture_ex(
                        texture,
                        x + 5.0,
                        y + 6.0 + bob,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(22.0, 22.0)),
                            ..Default::default()
                        },
                    );
                } else {
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

pub(crate) fn draw_lighting_region(
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

pub(crate) fn draw_ambient_dust(palette: &Palette, time: f32) {
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

pub(crate) fn draw_vignette(palette: &Palette) {
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

pub(crate) fn draw_auto_move_overlay(player: &Player) {
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

pub(crate) fn draw_npc_wander_overlay(npc: &NpcWanderer) {
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

pub(crate) fn draw_editor_grid_region(bounds: (i32, i32, i32, i32)) {
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

pub(crate) fn run_main_menu_frame(map: &MapAsset, palette: &Palette, time: f32) -> Option<AppMode> {
    clear_background(palette.bg_bottom);
    begin_ui_pass();
    if let Some(texture) = main_menu_background_texture().as_ref() {
        draw_texture_ex(
            texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::from_rgba(0, 0, 0, 48),
        );
    } else {
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
        begin_ui_pass();
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
            Color::from_rgba(6, 9, 13, 120),
        );
    }

    let center_x = screen_width() * 0.5;
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let click = is_mouse_button_pressed(MouseButton::Left);
    let play_rect = Rect::new(center_x - 140.0, screen_height() * 0.42, 280.0, 58.0);
    let editor_rect = Rect::new(center_x - 140.0, play_rect.y + 74.0, 280.0, 58.0);

    let play_clicked = draw_ui_button(play_rect, "Jouer", mouse, click, false);
    let editor_clicked = draw_ui_button(editor_rect, "Editeur", mouse, click, false);

    if play_clicked || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
        return Some(AppMode::Playing);
    }
    if editor_clicked || is_key_pressed(KeyCode::E) {
        return Some(AppMode::Editor);
    }
    None
}

pub(crate) fn sim_zone_overlay_color(zone: sim::ZoneKind) -> Option<Color> {
    match zone {
        sim::ZoneKind::Neutral => None,
        sim::ZoneKind::Receiving => Some(Color::from_rgba(86, 122, 224, 62)),
        sim::ZoneKind::Processing => Some(Color::from_rgba(218, 114, 42, 66)),
        sim::ZoneKind::Shipping => Some(Color::from_rgba(64, 180, 122, 62)),
        sim::ZoneKind::Support => Some(Color::from_rgba(172, 130, 220, 58)),
    }
}

pub(crate) fn sim_block_overlay_color(kind: sim::BlockKind) -> Color {
    match kind {
        sim::BlockKind::Storage => Color::from_rgba(88, 160, 222, 255),
        sim::BlockKind::MachineA => Color::from_rgba(240, 154, 72, 255),
        sim::BlockKind::MachineB => Color::from_rgba(252, 120, 82, 255),
        sim::BlockKind::Buffer => Color::from_rgba(142, 122, 208, 255),
        sim::BlockKind::Seller => Color::from_rgba(94, 196, 124, 255),
    }
}

pub(crate) fn draw_sim_zone_overlay_region(sim: &sim::FactorySim, bounds: (i32, i32, i32, i32)) {
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

pub(crate) fn draw_sim_blocks_overlay(
    sim: &sim::FactorySim,
    show_labels: bool,
    bounds: Option<(i32, i32, i32, i32)>,
) {
    let storage_texture = storage_raw_texture();
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
        if block.kind == sim::BlockKind::Storage && block.raw_qty > 0 {
            draw_storage_raw_stack(rect, block.raw_qty, storage_texture.as_ref());
        }
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

fn draw_storage_raw_stack(rect: Rect, raw_qty: u32, texture: Option<&Texture2D>) {
    let pile = raw_qty.clamp(1, 3) as usize;
    for i in 0..pile {
        let s = 14.0;
        let x = rect.x + rect.w - s - 4.0 - i as f32 * 5.5;
        let y = rect.y + rect.h - s - 4.0 - i as f32 * 3.2;
        if let Some(tex) = texture {
            draw_texture_ex(
                tex,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(s, s)),
                    ..Default::default()
                },
            );
        } else {
            let col = Color::from_rgba(188, 150, 104, 225);
            draw_rectangle(x, y, s, s, col);
            draw_rectangle_lines(x, y, s, s, 1.0, Color::from_rgba(80, 60, 40, 220));
        }
    }

    if raw_qty > 1 {
        let count = format!("x{}", raw_qty);
        draw_text(
            &count,
            rect.x + 3.0,
            rect.y + rect.h - 4.0,
            11.0,
            Color::from_rgba(240, 246, 252, 230),
        );
    }
}

pub(crate) fn draw_sim_agent_overlay(sim: &sim::FactorySim, show_label: bool) {
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
