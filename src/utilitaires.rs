use super::*;

pub(crate) fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

pub(crate) fn with_alpha(mut c: Color, alpha: f32) -> Color {
    c.a = alpha.clamp(0.0, 1.0);
    c
}

pub(crate) fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    let k = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * k,
        a.g + (b.g - a.g) * k,
        a.b + (b.b - a.b) * k,
        a.a + (b.a - a.a) * k,
    )
}

pub(crate) fn tile_is_wall(tile: Tile) -> bool {
    matches!(
        tile,
        Tile::Wall | Tile::WallBrick | Tile::WallSteel | Tile::WallNeon
    )
}

pub(crate) fn tile_is_floor(tile: Tile) -> bool {
    !tile_is_wall(tile)
}

pub(crate) fn tile_label(tile: Tile) -> &'static str {
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

pub(crate) fn tile_hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32)
        .wrapping_mul(0x9E37_79B9)
        .wrapping_add((y as u32).wrapping_mul(0x85EB_CA6B));
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^ (h >> 16)
}

pub(crate) fn hash_with_salt(x: i32, y: i32, salt: u32) -> u32 {
    let sx = x.wrapping_add((salt as i32).wrapping_mul(31));
    let sy = y.wrapping_sub((salt as i32).wrapping_mul(17));
    tile_hash(sx, sy) ^ salt.wrapping_mul(0x27D4_EB2D)
}

pub(crate) fn clamp_i32(v: i32, lo: i32, hi: i32) -> i32 {
    v.max(lo).min(hi)
}

pub(crate) fn tiles_overlapping_aabb(world: &World, aabb: Aabb) -> (i32, i32, i32, i32) {
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

pub(crate) fn generate_starter_factory_world(w: i32, h: i32) -> World {
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

pub(crate) fn default_props(world: &World) -> Vec<Prop> {
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

pub(crate) fn apply_material_variation(world: &mut World) {
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

pub(crate) fn advance_seed(seed: u64) -> u64 {
    seed.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

pub(crate) fn handle_fullscreen_hotkey(is_fullscreen_mode: &mut bool) -> bool {
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

pub(crate) fn point_in_rect(point: Vec2, rect: Rect) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.w
        && point.y >= rect.y
        && point.y <= rect.y + rect.h
}

pub(crate) fn fit_world_camera_to_screen(world: &World, margin: f32) -> (Camera2D, Rect) {
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

pub(crate) fn build_world_camera_for_viewport(
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

pub(crate) fn build_pannable_world_camera(
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

pub(crate) fn tile_bounds_from_camera(
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

pub(crate) fn tile_in_bounds(tile: (i32, i32), bounds: (i32, i32, i32, i32)) -> bool {
    tile.0 >= bounds.0 && tile.0 <= bounds.1 && tile.1 >= bounds.2 && tile.1 <= bounds.3
}

pub(crate) fn draw_ui_button(
    rect: Rect,
    label: &str,
    mouse_pos: Vec2,
    mouse_pressed: bool,
    active: bool,
) -> bool {
    draw_ui_button_sized(rect, label, mouse_pos, mouse_pressed, active, 22.0)
}

pub(crate) fn draw_ui_button_sized(
    rect: Rect,
    label: &str,
    mouse_pos: Vec2,
    mouse_pressed: bool,
    active: bool,
    font_size: f32,
) -> bool {
    let hovered = mouse_pos.x >= rect.x
        && mouse_pos.x <= rect.x + rect.w
        && mouse_pos.y >= rect.y
        && mouse_pos.y <= rect.y + rect.h;

    let base = if active {
        Color::from_rgba(210, 150, 82, 255)
    } else if hovered {
        Color::from_rgba(98, 140, 170, 255)
    } else {
        Color::from_rgba(70, 102, 128, 255)
    };
    let border = if active {
        Color::from_rgba(250, 216, 164, 255)
    } else if hovered {
        Color::from_rgba(168, 210, 235, 255)
    } else {
        Color::from_rgba(138, 184, 210, 255)
    };

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, base);
    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.w - 1.0,
        rect.h - 1.0,
        1.6,
        border,
    );
    draw_rectangle_lines(
        rect.x + 1.5,
        rect.y + 1.5,
        rect.w - 3.0,
        rect.h - 3.0,
        0.9,
        with_alpha(border, 0.35),
    );

    let dims = measure_text(label, None, font_size as u16, 1.0);
    let text_x = rect.x + rect.w * 0.5 - dims.width * 0.5;
    let text_y = rect.y + rect.h * 0.5 + dims.height * 0.32;

    // Robust text readability: color is auto-fixed to guarantee contrast.
    let (mut text_fill, _) = ui_text_and_shadow_for_bg(base);

    // Keep a slight warm bias when active (visual feedback), but still enforce contrast.
    if active {
        let warm = Color::from_rgba(255, 248, 232, 255);
        text_fill = color_lerp(text_fill, warm, 0.35);
    }

    // Enforce minimum contrast ratio (WCAG-ish).
    text_fill = ui_ensure_text_contrast(base, text_fill, 4.5);
    let mut shadow = ui_shadow_color_for_text(text_fill);

    if active {
        // Slightly stronger shadow when active.
        shadow.a = (shadow.a + 0.08).clamp(0.0, 1.0);
    }

    let offset = ui_shadow_offset(font_size);
    draw_text_shadowed(label, text_x, text_y, font_size, text_fill, shadow, offset);

    hovered && mouse_pressed
}

// --- UI text helpers ---------------------------------------------------------

/// Shadow offset in pixels, scaled with font size (keeps the UI crisp on various DPI).
#[inline]
pub(crate) fn ui_shadow_offset(font_size: f32) -> f32 {
    // 22px is our typical UI button font size.
    (font_size / 22.0).clamp(0.85, 1.75)
}

#[inline]
fn srgb_to_linear_channel(v: f32) -> f32 {
    // WCAG relative luminance uses linear RGB.
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
fn relative_luminance(c: Color) -> f32 {
    let r = srgb_to_linear_channel(c.r.clamp(0.0, 1.0));
    let g = srgb_to_linear_channel(c.g.clamp(0.0, 1.0));
    let b = srgb_to_linear_channel(c.b.clamp(0.0, 1.0));
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

#[inline]
fn contrast_ratio(a: Color, b: Color) -> f32 {
    let la = relative_luminance(a);
    let lb = relative_luminance(b);
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

#[inline]
fn composite_over(bg: Color, under: Color) -> Color {
    // Approximate the effective background when bg is translucent.
    let a = bg.a.clamp(0.0, 1.0);
    Color::new(
        under.r * (1.0 - a) + bg.r * a,
        under.g * (1.0 - a) + bg.g * a,
        under.b * (1.0 - a) + bg.b * a,
        1.0,
    )
}

/// Pick a high-contrast UI text color for a given background.
///
/// We choose between a near-white and a near-black tint, whichever yields
/// the best WCAG contrast ratio.
#[inline]
pub(crate) fn ui_best_text_color_for_bg(bg: Color) -> Color {
    // UI is mostly dark; for translucent panels assume the scene below is also dark.
    let assumed_under = Color::from_rgba(8, 12, 18, 255);
    let eff_bg = if bg.a < 0.999 {
        composite_over(bg, assumed_under)
    } else {
        bg
    };

    let light = Color::from_rgba(244, 252, 255, 255);
    let dark = Color::from_rgba(10, 14, 18, 255);

    if contrast_ratio(light, eff_bg) >= contrast_ratio(dark, eff_bg) {
        light
    } else {
        dark
    }
}

#[inline]
pub(crate) fn ui_shadow_color_for_text(text: Color) -> Color {
    // Opposite polarity shadow (dark shadow for light text, and vice-versa).
    if relative_luminance(text) > 0.5 {
        with_alpha(Color::from_rgba(0, 0, 0, 255), 0.82)
    } else {
        with_alpha(Color::from_rgba(255, 255, 255, 255), 0.55)
    }
}

#[inline]
pub(crate) fn ui_text_and_shadow_for_bg(bg: Color) -> (Color, Color) {
    let text = ui_best_text_color_for_bg(bg);
    let shadow = ui_shadow_color_for_text(text);
    (text, shadow)
}

/// Draw text with a shadow.
#[inline]
pub(crate) fn draw_text_shadowed(
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    text_color: Color,
    shadow_color: Color,
    shadow_offset: f32,
) {
    draw_text(
        text,
        x + shadow_offset,
        y + shadow_offset,
        font_size,
        shadow_color,
    );
    draw_text(text, x, y, font_size, text_color);
}

/// Convenience: pick readable UI colors automatically (no tint preservation).
#[inline]
#[allow(dead_code)]
pub(crate) fn draw_ui_text_on(bg: Color, text: &str, x: f32, y: f32, font_size: f32) {
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);
    let offset = ui_shadow_offset(font_size);
    draw_text_shadowed(text, x, y, font_size, fill, shadow, offset);
}

#[inline]
#[allow(dead_code)]
pub(crate) fn draw_ui_text_centered_on(bg: Color, text: &str, rect: Rect, font_size: f32) {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    let x = rect.x + rect.w * 0.5 - dims.width * 0.5;
    let y = rect.y + rect.h * 0.5 + dims.height * 0.32;
    draw_ui_text_on(bg, text, x, y, font_size);
}

/// Ensure a text color keeps at least `min_ratio` contrast against the background.
#[inline]
pub(crate) fn ui_ensure_text_contrast(bg: Color, desired: Color, min_ratio: f32) -> Color {
    let assumed_under = Color::from_rgba(8, 12, 18, 255);
    let eff_bg = if bg.a < 0.999 {
        composite_over(bg, assumed_under)
    } else {
        bg
    };

    let desired = Color::new(desired.r, desired.g, desired.b, 1.0);
    if contrast_ratio(desired, eff_bg) >= min_ratio {
        return desired;
    }

    let target = ui_best_text_color_for_bg(bg);

    // Binary search smallest blend that satisfies contrast.
    let mut lo = 0.0f32;
    let mut hi = 1.0f32;
    for _ in 0..12 {
        let mid = (lo + hi) * 0.5;
        let cand = color_lerp(desired, target, mid);
        if contrast_ratio(cand, eff_bg) >= min_ratio {
            hi = mid;
        } else {
            lo = mid;
        }
    }

    color_lerp(desired, target, hi)
}

/// Same as `draw_ui_text_on`, but preserves a desired tint when possible.
#[inline]
pub(crate) fn draw_ui_text_tinted_on(
    bg: Color,
    desired_fill: Color,
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
) {
    let fill = ui_ensure_text_contrast(bg, desired_fill, 4.5);
    let shadow = ui_shadow_color_for_text(fill);
    let offset = ui_shadow_offset(font_size);
    draw_text_shadowed(text, x, y, font_size, fill, shadow, offset);
}

#[inline]
#[allow(dead_code)]
pub(crate) fn draw_ui_text_tinted_centered_on(
    bg: Color,
    desired_fill: Color,
    text: &str,
    rect: Rect,
    font_size: f32,
) {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    let x = rect.x + rect.w * 0.5 - dims.width * 0.5;
    let y = rect.y + rect.h * 0.5 + dims.height * 0.32;
    draw_ui_text_tinted_on(bg, desired_fill, text, x, y, font_size);
}
