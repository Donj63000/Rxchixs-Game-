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

pub(crate) fn starter_factory_bounds(w: i32, h: i32) -> (i32, i32, i32, i32) {
    let w = w.max(8);
    let h = h.max(8);
    let available_w = (w - 8).max(6);
    let available_h = (h - 8).max(6);

    let target_w = ((w as f32) * 0.24).round() as i32;
    let target_h = ((h as f32) * 0.28).round() as i32;
    let factory_w = target_w.clamp(12, 44).min(available_w);
    let factory_h = target_h.clamp(10, 30).min(available_h);

    let center_x = ((w as f32) * 0.58).round() as i32;
    let center_y = ((h as f32) * 0.52).round() as i32;

    let min_x_lo = 2;
    let min_x_hi = (w - factory_w - 3).max(min_x_lo);
    let min_y_lo = 2;
    let min_y_hi = (h - factory_h - 3).max(min_y_lo);

    let min_x = (center_x - factory_w / 2).clamp(min_x_lo, min_x_hi);
    let min_y = (center_y - factory_h / 2).clamp(min_y_lo, min_y_hi);
    let max_x = (min_x + factory_w - 1).clamp(min_x + 2, w - 2);
    let max_y = (min_y + factory_h - 1).clamp(min_y + 2, h - 2);
    (min_x, max_x, min_y, max_y)
}

pub(crate) fn generate_starter_factory_world(w: i32, h: i32) -> World {
    let mut world = World::new_room(w, h);

    // Exterieur type campagne: dominant herbe, avec zones tassees et plaques de terre.
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let h0 = hash_with_salt(x, y, 0x51) % 100;
            let tile = if h0 < 58 {
                Tile::FloorMoss
            } else if h0 < 84 {
                Tile::Floor
            } else {
                Tile::FloorSand
            };
            world.set(x, y, tile);
        }
    }

    let (fx0, fx1, fy0, fy1) = starter_factory_bounds(w, h);
    let span_y = (fy1 - fy0).max(6);
    let road_y = clamp_i32(fy0 + span_y / 2, fy0 + 2, fy1 - 2);
    let ship_y = clamp_i32(fy0 + (span_y * 3) / 4, fy0 + 2, fy1 - 2);
    let south_door_x = (fx0 + fx1) / 2;

    // Axes de circulation extérieurs (accès réception + accès expédition).
    for y in road_y - 1..=road_y + 1 {
        for x in 1..=fx0 + 2 {
            if world.in_bounds(x, y) {
                world.set(x, y, Tile::Floor);
            }
        }
    }
    for y in ship_y - 1..=ship_y + 1 {
        for x in fx1 - 2..w - 1 {
            if world.in_bounds(x, y) {
                world.set(x, y, Tile::Floor);
            }
        }
    }
    for x in south_door_x - 1..=south_door_x + 1 {
        for y in fy1..h - 1 {
            if world.in_bounds(x, y) {
                world.set(x, y, Tile::Floor);
            }
        }
    }

    // Aplomb de quai côté réception et côté expédition.
    for y in road_y - 2..=road_y + 2 {
        for x in fx0 - 4..=fx0 + 2 {
            if world.in_bounds(x, y) {
                world.set(x, y, Tile::FloorWood);
            }
        }
    }
    for y in ship_y - 2..=ship_y + 2 {
        for x in fx1 - 2..=fx1 + 4 {
            if world.in_bounds(x, y) {
                world.set(x, y, Tile::FloorWood);
            }
        }
    }

    // Bâtiment usine compact.
    for y in fy0 + 1..fy1 {
        for x in fx0 + 1..fx1 {
            world.set(x, y, Tile::FloorMetal);
        }
    }

    for x in fx0..=fx1 {
        world.set(x, fy0, Tile::WallSteel);
        world.set(x, fy1, Tile::WallSteel);
    }
    for y in fy0..=fy1 {
        world.set(fx0, y, Tile::WallSteel);
        world.set(fx1, y, Tile::WallSteel);
    }

    // Portes extérieures.
    for dy in -1..=1 {
        world.set(fx0, road_y + dy, Tile::FloorMetal);
        world.set(fx1, ship_y + dy, Tile::FloorMetal);
    }
    for dx in -1..=1 {
        world.set(south_door_x + dx, fy1, Tile::FloorMetal);
    }

    // Sous-zones internes (réception / production / expédition + support).
    let p1 = fx0 + ((fx1 - fx0) / 3);
    let p2 = fx0 + ((fx1 - fx0) * 2 / 3);
    let support_wall_y = (fy1 - 6).max(fy0 + 3);

    for y in fy0 + 1..fy1 {
        for x in fx0 + 1..p1 {
            world.set(x, y, Tile::FloorWood);
        }
        for x in p2 + 1..fx1 {
            world.set(x, y, Tile::FloorWood);
        }
    }
    for y in support_wall_y + 1..fy1 {
        for x in fx0 + 1..fx1 {
            world.set(x, y, Tile::Floor);
        }
    }

    for y in fy0 + 1..fy1 {
        world.set(p1, y, Tile::WallBrick);
        world.set(p2, y, Tile::WallBrick);
    }
    for x in fx0 + 1..fx1 {
        world.set(x, support_wall_y, Tile::WallBrick);
    }

    // Ouvertures internes pour un flux lisible.
    for dy in -1..=1 {
        world.set(p1, road_y + dy, Tile::FloorMetal);
        world.set(p2, road_y + dy, Tile::FloorMetal);
        world.set(p2, ship_y + dy, Tile::FloorMetal);
    }
    for dx in -1..=1 {
        world.set(south_door_x + dx, support_wall_y, Tile::FloorMetal);
        world.set(fx0 + 3 + dx, support_wall_y, Tile::FloorMetal);
        world.set(fx1 - 3 + dx, support_wall_y, Tile::FloorMetal);
    }

    enforce_world_border(&mut world);
    world
}

fn add_prop_if_walkable_unique(
    props: &mut Vec<Prop>,
    world: &World,
    tile: (i32, i32),
    kind: PropKind,
) {
    if !world.in_bounds(tile.0, tile.1) || world.is_solid(tile.0, tile.1) {
        return;
    }
    if prop_index_at_tile(props, tile).is_some() {
        return;
    }
    props.push(Prop {
        tile_x: tile.0,
        tile_y: tile.1,
        kind,
        phase: prop_phase_for_tile(tile),
        rotation_quarter: 0,
    });
}

pub(crate) fn default_props(world: &World) -> Vec<Prop> {
    let mut props = Vec::new();

    let (fx0, fx1, fy0, fy1) = starter_factory_bounds(world.w, world.h);
    let span_y = (fy1 - fy0).max(6);
    let road_y = clamp_i32(fy0 + span_y / 2, fy0 + 2, fy1 - 2);
    let ship_y = clamp_i32(fy0 + (span_y * 3) / 4, fy0 + 2, fy1 - 2);
    let support_y = (fy1 - 3).max(fy0 + 2);

    // Éclairage du site: allée principale + sortie expédition.
    for x in (4..fx0 - 1).step_by(9) {
        add_prop_if_walkable_unique(&mut props, world, (x, road_y - 2), PropKind::Lamp);
    }
    for x in ((fx1 + 4).max(4)..world.w - 4).step_by(10) {
        add_prop_if_walkable_unique(&mut props, world, (x, ship_y + 2), PropKind::Lamp);
    }
    for y in ((fy1 + 3).max(4)..world.h - 4).step_by(9) {
        add_prop_if_walkable_unique(&mut props, world, ((fx0 + fx1) / 2 - 2, y), PropKind::Lamp);
        add_prop_if_walkable_unique(&mut props, world, ((fx0 + fx1) / 2 + 2, y), PropKind::Lamp);
    }

    // Exterieur: touches decoratives legeres, priorite au vegetal.
    for y in (4..world.h - 4).step_by(8) {
        for x in (4..world.w - 4).step_by(10) {
            if x >= fx0 - 6 && x <= fx1 + 6 && y >= fy0 - 6 && y <= fy1 + 6 {
                continue;
            }
            let h = hash_with_salt(x, y, 0xA3) % 100;
            if h > 14 {
                continue;
            }
            let kind = match h % 12 {
                0..=5 => PropKind::Plant,
                6 => PropKind::Bench,
                7 => PropKind::Lamp,
                8 => PropKind::BoxCartonVide,
                9 => PropKind::PaletteLogistique,
                10 => PropKind::CaisseAilBrut,
                _ => PropKind::CaisseAilCasse,
            };
            add_prop_if_walkable_unique(&mut props, world, (x, y), kind);
        }
    }

    // Réception: stock initial et outillage.
    let recv_x0 = fx0 + 2;
    let recv_x1 = (fx0 + ((fx1 - fx0) / 3) - 2).max(recv_x0);
    for x in (recv_x0..=recv_x1).step_by(3) {
        for y in (fy0 + 2..=road_y + 2).step_by(3) {
            let slot = hash_with_salt(x, y, 0xA7) % 3;
            let kind = match slot {
                0 => PropKind::CaisseAilBrut,
                1 => PropKind::BoxCartonVide,
                _ => PropKind::PaletteLogistique,
            };
            add_prop_if_walkable_unique(&mut props, world, (x, y), kind);
        }
    }

    // Zone de production: tuyauterie + signalétique.
    let proc_x0 = (fx0 + ((fx1 - fx0) / 3) + 2).min(fx1 - 3);
    let proc_x1 = (fx0 + ((fx1 - fx0) * 2 / 3) - 2).max(proc_x0);
    for x in (proc_x0..=proc_x1).step_by(4) {
        add_prop_if_walkable_unique(&mut props, world, (x, road_y - 1), PropKind::Pipe);
        add_prop_if_walkable_unique(&mut props, world, (x, road_y + 3), PropKind::Pipe);
    }
    for x in (proc_x0 + 1..=proc_x1).step_by(5) {
        add_prop_if_walkable_unique(&mut props, world, (x, road_y + 1), PropKind::CaisseAilCasse);
    }
    add_prop_if_walkable_unique(
        &mut props,
        world,
        ((proc_x0 + proc_x1) / 2 - 1, road_y + 4),
        PropKind::BoxSacRouge,
    );
    add_prop_if_walkable_unique(
        &mut props,
        world,
        ((proc_x0 + proc_x1) / 2 + 2, road_y + 4),
        PropKind::BoxSacVert,
    );
    add_prop_if_walkable_unique(&mut props, world, (proc_x0 + 1, fy0 + 2), PropKind::Banner);
    add_prop_if_walkable_unique(&mut props, world, (proc_x1 - 1, fy0 + 2), PropKind::Banner);

    // Expédition: palettes/caisses prêtes au départ.
    let ship_x0 = (fx0 + ((fx1 - fx0) * 2 / 3) + 2).min(fx1 - 2);
    for y in (ship_y - 3..=ship_y + 3).step_by(2) {
        add_prop_if_walkable_unique(&mut props, world, (ship_x0, y), PropKind::PaletteLogistique);
        add_prop_if_walkable_unique(&mut props, world, (ship_x0 + 1, y), PropKind::BoxSacBleu);
        add_prop_if_walkable_unique(&mut props, world, (fx1 + 2, y), PropKind::PaletteLogistique);
        add_prop_if_walkable_unique(&mut props, world, (fx1 + 3, y), PropKind::BoxSacVert);
    }

    // Support: petit coin vie/administratif.
    add_prop_if_walkable_unique(
        &mut props,
        world,
        (fx0 + 3, support_y),
        PropKind::BureauPcOn,
    );
    add_prop_if_walkable_unique(&mut props, world, (fx0 + 6, support_y), PropKind::Lavabo);
    add_prop_if_walkable_unique(
        &mut props,
        world,
        (fx1 - 3, support_y),
        PropKind::BureauPcOff,
    );
    add_prop_if_walkable_unique(&mut props, world, (fx1 - 6, support_y), PropKind::Plant);
    add_prop_if_walkable_unique(
        &mut props,
        world,
        ((fx0 + fx1) / 2, support_y - 1),
        PropKind::Lamp,
    );

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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
        shadow.a = (shadow.a + 0.05).clamp(0.0, 0.64);
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
    // For this UI theme, keep text light unless background is truly very bright.
    // This avoids the "all texts look black" failure mode on medium blue/orange panels.
    if relative_luminance(eff_bg) > 0.72 {
        dark
    } else {
        light
    }
}

#[inline]
pub(crate) fn ui_shadow_color_for_text(text: Color) -> Color {
    // Opposite polarity shadow (dark shadow for light text, and vice-versa).
    if relative_luminance(text) > 0.5 {
        with_alpha(Color::from_rgba(24, 38, 56, 255), 0.20)
    } else {
        with_alpha(Color::from_rgba(250, 254, 255, 255), 0.16)
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
    crate::render_safety::ensure_default_material();
    // Very light one-pass drop shadow to avoid any "black highlight" effect.
    let off = shadow_offset.max(0.75);
    let shadow = with_alpha(shadow_color, (shadow_color.a * 0.65).clamp(0.0, 0.12));
    draw_text(text, x + off * 0.55, y + off * 0.75, font_size, shadow);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_i32_respects_bounds() {
        assert_eq!(clamp_i32(5, 0, 10), 5);
        assert_eq!(clamp_i32(-4, 0, 10), 0);
        assert_eq!(clamp_i32(42, 0, 10), 10);
    }

    #[test]
    fn advance_seed_is_deterministic_and_changes_value() {
        let seed = 0x1234_5678_9ABC_DEF0;
        let next_a = advance_seed(seed);
        let next_b = advance_seed(seed);
        assert_eq!(next_a, next_b);
        assert_ne!(next_a, seed);
    }

    #[test]
    fn tile_label_and_kind_helpers_are_consistent() {
        assert!(tile_is_wall(Tile::Wall));
        assert!(tile_is_wall(Tile::WallBrick));
        assert!(!tile_is_wall(Tile::Floor));
        assert!(tile_is_floor(Tile::FloorMoss));
        assert!(!tile_is_floor(Tile::WallNeon));
        assert_eq!(tile_label(Tile::WallSteel), "mur_acier");
        assert_eq!(tile_label(Tile::FloorWood), "sol_bois");
    }

    #[test]
    fn hash_with_salt_is_stable_for_same_inputs() {
        let h1 = hash_with_salt(12, 7, 99);
        let h2 = hash_with_salt(12, 7, 99);
        let h3 = hash_with_salt(12, 7, 100);
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn starter_factory_bounds_stays_inside_world() {
        for (w, h) in [(25, 15), (96, 64), (168, 108)] {
            let (fx0, fx1, fy0, fy1) = starter_factory_bounds(w, h);
            assert!(fx0 >= 1 && fy0 >= 1);
            assert!(fx1 <= w - 2 && fy1 <= h - 2);
            assert!(fx1 - fx0 >= 4);
            assert!(fy1 - fy0 >= 4);
        }
    }

    #[test]
    fn generated_world_keeps_large_outdoor_area() {
        let world = generate_starter_factory_world(168, 108);
        let (fx0, fx1, fy0, fy1) = starter_factory_bounds(world.w, world.h);
        let mut outside = 0usize;
        let mut outside_open = 0usize;
        for y in 1..world.h - 1 {
            for x in 1..world.w - 1 {
                let in_factory = x >= fx0 && x <= fx1 && y >= fy0 && y <= fy1;
                if in_factory {
                    continue;
                }
                outside += 1;
                if !world.is_solid(x, y) {
                    outside_open += 1;
                }
            }
        }
        assert!(outside > 10_000);
        assert!(outside_open * 100 / outside >= 96);
    }

    #[test]
    fn light_text_shadow_alpha_stays_below_halo_threshold() {
        let light_text = Color::from_rgba(242, 250, 255, 255);
        let shadow = ui_shadow_color_for_text(light_text);
        assert!(shadow.a <= 0.22);
        assert!(shadow.r <= 0.12 && shadow.g <= 0.16 && shadow.b <= 0.24);
    }

    #[test]
    fn dark_text_uses_light_shadow_with_controlled_alpha() {
        let dark_text = Color::from_rgba(12, 14, 18, 255);
        let shadow = ui_shadow_color_for_text(dark_text);
        assert!(shadow.a <= 0.18);
        assert!(shadow.r >= 0.97 && shadow.g >= 0.99 && shadow.b >= 0.99);
    }

    #[test]
    fn ui_shadow_offset_is_deterministic_and_clamped() {
        let tiny = ui_shadow_offset(4.0);
        let medium_a = ui_shadow_offset(22.0);
        let medium_b = ui_shadow_offset(22.0);
        let huge = ui_shadow_offset(160.0);
        assert!((tiny - 0.85).abs() < 0.0001);
        assert!((medium_a - medium_b).abs() < 0.0001);
        assert!(huge <= 1.75);
    }

    #[test]
    fn ui_prefers_light_text_on_medium_dark_panels() {
        let medium_blue = Color::from_rgba(70, 102, 128, 255);
        let text = ui_best_text_color_for_bg(medium_blue);
        assert!(relative_luminance(text) > 0.8);
    }

    #[test]
    fn ui_uses_dark_text_only_on_very_bright_panels() {
        let bright_bg = Color::from_rgba(242, 240, 224, 255);
        let text = ui_best_text_color_for_bg(bright_bg);
        assert!(relative_luminance(text) < 0.1);
    }

    #[test]
    fn default_props_are_unique_and_walkable() {
        use std::collections::HashSet;

        let world = generate_starter_factory_world(168, 108);
        let props = default_props(&world);
        let mut occupied = HashSet::new();
        for prop in props {
            let tile = (prop.tile_x, prop.tile_y);
            assert!(world.in_bounds(tile.0, tile.1));
            assert!(!world.is_solid(tile.0, tile.1));
            assert!(occupied.insert(tile));
        }
    }
}
