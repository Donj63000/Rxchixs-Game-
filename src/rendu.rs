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
    static BROKEN_GARLIC_CRATE_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static BOX_CARTON_VIDE_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static BOX_SAC_BLEU_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static BOX_SAC_ROUGE_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static BOX_SAC_VERT_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static PALETTE_LOGISTIQUE_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static BUREAU_PC_ON_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static BUREAU_PC_OFF_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static LAVABO_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
    static MAIN_MENU_BACKGROUND_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
}

const PROP_TEXTURE_VISUAL_SCALE: f32 = 1.15;
const CHARIOT_VISUAL_SCALE: f32 = 2.44;

fn scaled_prop_texture_placement(
    base_x: f32,
    base_y: f32,
    base_w: f32,
    base_h: f32,
) -> (f32, f32, Vec2) {
    let scaled_w = base_w * PROP_TEXTURE_VISUAL_SCALE;
    let scaled_h = base_h * PROP_TEXTURE_VISUAL_SCALE;
    let offset_x = (scaled_w - base_w) * 0.5;
    let offset_y = (scaled_h - base_h) * 0.5;
    (
        base_x - offset_x,
        base_y - offset_y,
        vec2(scaled_w, scaled_h),
    )
}

fn draw_prop_texture_scaled(
    texture: &Texture2D,
    base_x: f32,
    base_y: f32,
    base_w: f32,
    base_h: f32,
) {
    let (draw_x, draw_y, draw_size) = scaled_prop_texture_placement(base_x, base_y, base_w, base_h);
    draw_texture_ex(
        texture,
        draw_x,
        draw_y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(draw_size),
            ..Default::default()
        },
    );
}

fn chariot_basis_vectors(heading_rad: f32) -> (Vec2, Vec2) {
    let forward = vec2(heading_rad.cos(), heading_rad.sin());
    let side = vec2(-forward.y, forward.x);
    (forward, side)
}

fn chariot_rotate_basis(forward: Vec2, side: Vec2, angle_rad: f32) -> (Vec2, Vec2) {
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let rotated_forward = forward * c + side * s;
    let rotated_side = side * c - forward * s;
    (rotated_forward, rotated_side)
}

fn chariot_point(center: Vec2, forward: Vec2, side: Vec2, f: f32, s: f32) -> Vec2 {
    center + forward * f + side * s
}

#[allow(clippy::too_many_arguments)]
fn draw_chariot_quad(
    center: Vec2,
    forward: Vec2,
    side: Vec2,
    f0: f32,
    f1: f32,
    s0: f32,
    s1: f32,
    color: Color,
) {
    let p0 = chariot_point(center, forward, side, f0, s0);
    let p1 = chariot_point(center, forward, side, f1, s0);
    let p2 = chariot_point(center, forward, side, f1, s1);
    let p3 = chariot_point(center, forward, side, f0, s1);
    draw_triangle(p0, p1, p2, color);
    draw_triangle(p0, p2, p3, color);
}

#[allow(clippy::too_many_arguments)]
fn draw_chariot_frame(
    center: Vec2,
    forward: Vec2,
    side: Vec2,
    f0: f32,
    f1: f32,
    s0: f32,
    s1: f32,
    thickness: f32,
    color: Color,
) {
    let p0 = chariot_point(center, forward, side, f0, s0);
    let p1 = chariot_point(center, forward, side, f1, s0);
    let p2 = chariot_point(center, forward, side, f1, s1);
    let p3 = chariot_point(center, forward, side, f0, s1);
    draw_line(p0.x, p0.y, p1.x, p1.y, thickness, color);
    draw_line(p1.x, p1.y, p2.x, p2.y, thickness, color);
    draw_line(p2.x, p2.y, p3.x, p3.y, thickness, color);
    draw_line(p3.x, p3.y, p0.x, p0.y, thickness, color);
}

fn chariot_cargo_colors(kind: PropKind) -> (Color, Color) {
    match kind {
        PropKind::BoxSacBleu => (rgba(72, 126, 198, 255), rgba(206, 232, 255, 195)),
        PropKind::BoxSacRouge => (rgba(188, 88, 84, 255), rgba(252, 220, 214, 198)),
        PropKind::BoxSacVert => (rgba(80, 158, 108, 255), rgba(210, 245, 220, 198)),
        PropKind::CaisseAilBrut => (rgba(178, 142, 98, 255), rgba(84, 62, 42, 212)),
        PropKind::CaisseAilCasse => (rgba(206, 170, 112, 255), rgba(94, 70, 42, 212)),
        PropKind::PaletteLogistique => (rgba(162, 116, 72, 255), rgba(86, 62, 40, 220)),
        _ => (rgba(152, 120, 86, 255), rgba(66, 48, 34, 214)),
    }
}

fn draw_chariot_wheel(center: Vec2, forward: Vec2, side: Vec2, tire: Color, rim: Color) {
    draw_chariot_quad(center, forward, side, -2.2, 2.2, -1.35, 1.35, tire);
    draw_chariot_frame(center, forward, side, -2.2, 2.2, -1.35, 1.35, 0.8, rim);
    draw_chariot_quad(center, forward, side, -0.55, 0.55, -1.05, 1.05, rim);
    draw_chariot_quad(
        center,
        forward,
        side,
        -0.2,
        0.2,
        -0.45,
        0.45,
        rgba(56, 62, 74, 255),
    );
}

pub(crate) fn draw_chariot_elevateur(
    chariot: &ChariotElevateur,
    palette: &Palette,
    time: f32,
    _driver_character: Option<&CharacterRecord>,
    debug: bool,
) {
    let center = chariot.pos;
    let (forward_unit, side_unit) = chariot_basis_vectors(chariot.heading_rad);
    let visual_scale = CHARIOT_VISUAL_SCALE;
    let forward = forward_unit * visual_scale;
    let side = side_unit * visual_scale;
    let moving = chariot.velocity.length_squared() > 4.0;
    let speed_ratio = (chariot.vitesse_longitudinale.abs() / 128.0).clamp(0.0, 1.0);
    let braquage_rad = chariot.angle_braquage * 0.72;
    let (front_wheel_forward, front_wheel_side) = chariot_rotate_basis(forward, side, braquage_rad);
    let fourche_lift_px = chariot.fourche_hauteur * 7.8 * visual_scale;
    let fourche_offset = vec2(0.0, -fourche_lift_px);

    let yellow_main = rgba(238, 190, 42, 255);
    let yellow_dark = rgba(188, 138, 32, 255);
    let steel = rgba(108, 120, 136, 255);
    let steel_dark = rgba(74, 84, 98, 255);
    let steel_high = rgba(186, 198, 216, 255);
    let mast_black = rgba(24, 28, 34, 255);
    let mast_black_soft = rgba(38, 44, 54, 255);
    let fork_black = rgba(14, 18, 24, 255);
    let fork_edge = rgba(92, 106, 124, 210);
    let cabin_tint = rgba(38, 52, 66, 230);
    let tire = rgba(22, 26, 34, 255);
    let rim = rgba(146, 156, 170, 255);

    // Ground shadow and suspension wobble.
    let wobble = (time * 5.4 + chariot.phase_anim).sin() * (0.16 + speed_ratio * 0.24);
    draw_chariot_quad(
        center + vec2(0.0, 2.2 + wobble * 0.5),
        forward,
        side,
        -15.2,
        16.8,
        -10.3,
        10.3,
        with_alpha(palette.shadow_hard, 0.34),
    );
    draw_chariot_quad(
        center + vec2(0.0, 2.8),
        forward,
        side,
        12.4,
        20.8,
        -5.6,
        5.6,
        with_alpha(palette.shadow_hard, 0.18 + chariot.fourche_hauteur * 0.14),
    );
    if speed_ratio > 0.12 {
        for i in 0..3 {
            let drift = (time * 1.35 + i as f32 * 0.31).fract();
            let puff = chariot_point(
                center + vec2(0.0, 0.8),
                forward,
                side,
                -14.8 - drift * 2.6,
                3.0 + i as f32 * 1.1,
            );
            draw_circle(
                puff.x + side_unit.x * drift * 2.2,
                puff.y - 1.0 - drift * 2.3,
                0.9 + drift * 1.4,
                with_alpha(rgba(84, 98, 114, 255), (0.16 * (1.0 - drift)) * speed_ratio),
            );
        }
    }

    // Wheels with front steering.
    let rear_left = chariot_point(center, forward, side, -8.0, -8.2);
    let rear_right = chariot_point(center, forward, side, -8.0, 8.2);
    let front_left = chariot_point(center, forward, side, 8.4, -8.2);
    let front_right = chariot_point(center, forward, side, 8.4, 8.2);
    draw_chariot_wheel(rear_left, forward, side, tire, rim);
    draw_chariot_wheel(rear_right, forward, side, tire, rim);
    draw_chariot_wheel(front_left, front_wheel_forward, front_wheel_side, tire, rim);
    draw_chariot_wheel(
        front_right,
        front_wheel_forward,
        front_wheel_side,
        tire,
        rim,
    );

    // Main chassis and side skirts.
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        -9.5,
        10.4,
        -7.0,
        7.0,
        yellow_main,
    );
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        -10.2,
        10.9,
        -7.6,
        -6.0,
        yellow_dark,
    );
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        -10.2,
        10.9,
        6.0,
        7.6,
        yellow_dark,
    );
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        -2.8,
        7.4,
        -6.8,
        -5.6,
        rgba(248, 238, 214, 230),
    );
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        -2.8,
        7.4,
        5.6,
        6.8,
        rgba(248, 238, 214, 230),
    );
    draw_chariot_frame(
        center + vec2(0.0, wobble),
        forward,
        side,
        -3.1,
        7.6,
        -6.9,
        -5.4,
        0.7,
        rgba(54, 42, 22, 205),
    );
    draw_chariot_frame(
        center + vec2(0.0, wobble),
        forward,
        side,
        -3.1,
        7.6,
        5.4,
        6.9,
        0.7,
        rgba(54, 42, 22, 205),
    );
    draw_chariot_frame(
        center + vec2(0.0, wobble),
        forward,
        side,
        -9.6,
        10.4,
        -7.0,
        7.0,
        1.2,
        rgba(58, 44, 18, 220),
    );

    // Rear counterweight (plain finish, no warning hatching).
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        -14.0,
        -6.2,
        -7.6,
        7.6,
        steel_dark,
    );

    // Cabin floor and overhead guard.
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        -4.2,
        5.2,
        -5.3,
        5.3,
        steel_dark,
    );
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        -2.0,
        2.6,
        -4.1,
        4.1,
        cabin_tint,
    );
    draw_chariot_frame(
        center + vec2(0.0, wobble),
        forward,
        side,
        -2.8,
        3.0,
        -5.4,
        5.4,
        1.1,
        steel,
    );
    draw_chariot_frame(
        center + vec2(0.0, wobble),
        forward,
        side,
        -2.8,
        3.0,
        -3.6,
        3.6,
        1.0,
        steel_high,
    );

    // Mast rails and cross-beam.
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        8.5,
        15.0,
        -4.4,
        -2.5,
        mast_black,
    );
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        8.5,
        15.0,
        2.5,
        4.4,
        mast_black,
    );
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        9.0,
        12.2,
        -3.9,
        3.9,
        mast_black_soft,
    );
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        11.0,
        12.1,
        -5.0,
        5.0,
        mast_black_soft,
    );
    draw_chariot_quad(
        center + vec2(0.0, wobble),
        forward,
        side,
        9.4,
        14.6,
        -0.8,
        0.8,
        mast_black,
    );
    draw_chariot_frame(
        center + vec2(0.0, wobble),
        forward,
        side,
        9.5,
        14.7,
        -1.0,
        1.0,
        0.7,
        with_alpha(fork_edge, 0.65),
    );

    // Black carriage + 2 parallel black forks with lift animation.
    let fourche_center = center + fourche_offset;
    draw_chariot_quad(
        fourche_center,
        forward,
        side,
        12.6,
        14.1,
        -5.9,
        5.9,
        mast_black_soft,
    );
    draw_chariot_quad(
        fourche_center,
        forward,
        side,
        13.4,
        23.4,
        -4.85,
        -3.35,
        fork_black,
    );
    draw_chariot_quad(
        fourche_center,
        forward,
        side,
        13.4,
        23.4,
        3.35,
        4.85,
        fork_black,
    );
    draw_chariot_quad(
        fourche_center,
        forward,
        side,
        13.15,
        14.0,
        -5.15,
        5.15,
        mast_black,
    );
    draw_chariot_frame(
        fourche_center,
        forward,
        side,
        13.3,
        23.5,
        -5.0,
        5.0,
        0.85,
        with_alpha(fork_edge, 0.56),
    );

    // Front and rear lights.
    let front_light = chariot_point(center + vec2(0.0, wobble), forward, side, 10.8, 0.0);
    draw_circle(
        front_light.x,
        front_light.y,
        1.9 + (moving as i32 as f32) * 0.25,
        rgba(255, 244, 186, 255),
    );
    draw_circle(
        front_light.x,
        front_light.y,
        3.1,
        with_alpha(rgba(255, 214, 138, 255), 0.3 + speed_ratio * 0.12),
    );
    let front_glow_radius = 7.5 + speed_ratio * 4.6;
    draw_circle(
        front_light.x + forward_unit.x * 1.9,
        front_light.y + forward_unit.y * 1.9,
        front_glow_radius,
        with_alpha(rgba(255, 224, 176, 255), 0.05 + speed_ratio * 0.08),
    );
    let rear_light_left = chariot_point(center + vec2(0.0, wobble), forward, side, -13.1, -4.3);
    let rear_light_right = chariot_point(center + vec2(0.0, wobble), forward, side, -13.1, 4.3);
    draw_circle(
        rear_light_left.x,
        rear_light_left.y,
        1.2,
        rgba(216, 62, 52, 255),
    );
    draw_circle(
        rear_light_right.x,
        rear_light_right.y,
        1.2,
        rgba(216, 62, 52, 255),
    );

    // Roof beacon (active only when driving).
    let beacon = chariot_point(center + vec2(0.0, wobble), forward, side, -0.4, 0.0);
    draw_circle(
        beacon.x,
        beacon.y - 7.2 + wobble,
        1.65,
        rgba(24, 28, 34, 255),
    );
    if chariot.pilote_a_bord {
        let beacon_pulse = ((time * 6.0 + chariot.phase_anim).sin() * 0.5 + 0.5).powf(1.4);
        draw_circle(
            beacon.x,
            beacon.y - 7.2 + wobble,
            1.35,
            rgba(238, 194, 62, 255),
        );
        draw_circle(
            beacon.x,
            beacon.y - 7.2 + wobble,
            3.2 + beacon_pulse * 2.4,
            with_alpha(rgba(252, 216, 88, 255), 0.18 + beacon_pulse * 0.17),
        );
    }

    // Driver silhouette when boarded.
    if chariot.pilote_a_bord {
        draw_chariot_quad(
            center + vec2(0.0, wobble),
            forward,
            side,
            -1.4,
            1.6,
            -2.1,
            2.1,
            rgba(56, 92, 132, 255),
        );
        let head = chariot_point(center + vec2(0.0, wobble), forward, side, -0.7, 0.0);
        draw_circle(
            head.x,
            head.y - 2.8 + wobble * 0.6,
            1.85,
            rgba(220, 188, 156, 255),
        );
        draw_circle(
            head.x + side.x * 0.7,
            head.y + side.y * 0.7 - 3.6,
            0.7,
            rgba(32, 28, 24, 255),
        );
    }

    // Carried cargo rendered above forks.
    if let Some(kind) = chariot.caisse_chargee {
        let cargo_center = chariot_point(fourche_center, forward, side, 17.2, 0.0);
        let bob = (time * 3.2 + chariot.phase_anim).sin() * 0.3;
        let (fill, edge) = chariot_cargo_colors(kind);
        let cargo_scale = 1.0 + chariot.fourche_hauteur * 0.12;
        let cargo_f = 2.9 * cargo_scale;
        let cargo_s = 3.2 * cargo_scale;
        draw_chariot_quad(
            chariot_point(center + vec2(0.0, 1.9), forward, side, 17.2, 0.0),
            forward,
            side,
            -cargo_f,
            cargo_f,
            -cargo_s,
            cargo_s,
            with_alpha(palette.shadow_hard, 0.16 + chariot.fourche_hauteur * 0.2),
        );
        draw_chariot_quad(
            cargo_center + vec2(0.0, bob),
            forward,
            side,
            -cargo_f,
            cargo_f,
            -cargo_s,
            cargo_s,
            fill,
        );
        draw_chariot_frame(
            cargo_center + vec2(0.0, bob),
            forward,
            side,
            -cargo_f,
            cargo_f,
            -cargo_s,
            cargo_s,
            1.0,
            edge,
        );
        draw_chariot_quad(
            cargo_center + vec2(0.0, bob),
            forward,
            side,
            -3.2,
            3.2,
            -0.45,
            0.45,
            with_alpha(edge, 0.68),
        );
    }

    if debug {
        let front = chariot_point(center, forward, side, 22.0, 0.0);
        draw_line(
            center.x,
            center.y,
            front.x,
            front.y,
            1.3,
            Color::from_rgba(252, 228, 122, 230),
        );
        let steer_front = chariot_point(center, forward, side, 9.6, 0.0);
        let steer_tip = chariot_point(steer_front, front_wheel_forward, front_wheel_side, 6.0, 0.0);
        draw_line(
            steer_front.x,
            steer_front.y,
            steer_tip.x,
            steer_tip.y,
            1.1,
            Color::from_rgba(170, 236, 255, 225),
        );
    }
}

pub(crate) fn draw_chargeur_clark(
    chargeur: &ChargeurClark,
    chariot: &ChariotElevateur,
    player_pos: Vec2,
    palette: &Palette,
    time: f32,
    debug: bool,
) {
    let base = chargeur.base_pos;
    let beacon_pulse = ((time * 2.8).sin() * 0.5 + 0.5).powf(1.25);
    let body_w = 22.0;
    let body_h = 26.0;
    let base_rect = Rect::new(base.x - body_w * 0.5, base.y - body_h * 0.5, body_w, body_h);

    // Accessibility cue: subtle glow ring so the charger is easy to spot.
    draw_circle(
        base.x,
        base.y + 1.0,
        19.0 + beacon_pulse * 3.8,
        with_alpha(rgba(86, 188, 218, 255), 0.08 + beacon_pulse * 0.08),
    );
    draw_circle_lines(
        base.x,
        base.y + 1.0,
        14.0 + beacon_pulse * 2.0,
        1.0,
        with_alpha(rgba(120, 210, 236, 255), 0.24),
    );

    draw_rectangle(
        base_rect.x + 1.8,
        base_rect.y + body_h - 4.0,
        body_w - 2.8,
        5.0,
        with_alpha(palette.shadow_hard, 0.35),
    );
    draw_rectangle(
        base_rect.x,
        base_rect.y,
        body_w,
        body_h,
        rgba(26, 30, 38, 255),
    );
    draw_rectangle(
        base_rect.x + 2.0,
        base_rect.y + 2.0,
        body_w - 4.0,
        body_h * 0.44,
        rgba(42, 52, 66, 250),
    );
    draw_rectangle_lines(
        base_rect.x + 0.5,
        base_rect.y + 0.5,
        body_w - 1.0,
        body_h - 1.0,
        1.0,
        rgba(122, 142, 168, 182),
    );

    let led_col = if chargeur.cable_branche && chariot.est_en_charge {
        let pulse = (time * 4.5).sin() * 0.5 + 0.5;
        color_lerp(rgba(84, 196, 116, 255), rgba(168, 242, 176, 255), pulse)
    } else {
        color_lerp(
            rgba(124, 138, 154, 255),
            rgba(156, 206, 224, 255),
            beacon_pulse * 0.45,
        )
    };
    draw_circle(base_rect.x + body_w - 5.0, base_rect.y + 5.0, 1.9, led_col);

    let cable_start = chargeur.point_prise();
    let (forward, side) = chariot_basis_vectors(chariot.heading_rad);
    let chariot_socket = chariot_point(
        chariot.pos,
        forward * CHARIOT_VISUAL_SCALE,
        side * CHARIOT_VISUAL_SCALE,
        -12.5,
        5.8,
    );
    let cable_end = if chargeur.cable_branche {
        chariot_socket
    } else if chargeur.cable_tenu {
        player_pos + vec2(0.0, -8.5)
    } else {
        cable_start + vec2(2.0, 9.0)
    };

    let cable_vec = cable_end - cable_start;
    let cable_len = cable_vec.length();
    if cable_len > 0.1 {
        let steps = ((cable_len / 12.0).ceil() as i32).clamp(4, 22) as usize;
        let sag = (8.0 + (cable_len / 28.0).clamp(0.0, 14.0)).min(18.0);
        let mut prev = cable_start;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let mut p = cable_start + cable_vec * t;
            let arc = (t * (1.0 - t)) * 4.0;
            p.y += sag * arc;
            draw_line(prev.x, prev.y, p.x, p.y, 3.2, rgba(8, 10, 14, 246));
            draw_line(prev.x, prev.y, p.x, p.y, 1.2, rgba(84, 96, 112, 205));
            prev = p;
        }
    }

    draw_circle(cable_end.x, cable_end.y, 2.1, rgba(18, 20, 24, 255));
    draw_circle(cable_end.x, cable_end.y, 1.05, rgba(138, 152, 170, 232));

    if debug {
        draw_circle_lines(
            base.x,
            base.y,
            CHARGEUR_INTERACTION_RADIUS,
            1.0,
            rgba(112, 186, 212, 120),
        );
    }
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

pub(crate) fn set_broken_garlic_crate_texture(texture: Option<Texture2D>) {
    BROKEN_GARLIC_CRATE_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn broken_garlic_crate_texture() -> Option<Texture2D> {
    BROKEN_GARLIC_CRATE_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_box_carton_vide_texture(texture: Option<Texture2D>) {
    BOX_CARTON_VIDE_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn box_carton_vide_texture() -> Option<Texture2D> {
    BOX_CARTON_VIDE_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_box_sac_bleu_texture(texture: Option<Texture2D>) {
    BOX_SAC_BLEU_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn box_sac_bleu_texture() -> Option<Texture2D> {
    BOX_SAC_BLEU_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_box_sac_rouge_texture(texture: Option<Texture2D>) {
    BOX_SAC_ROUGE_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn box_sac_rouge_texture() -> Option<Texture2D> {
    BOX_SAC_ROUGE_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_box_sac_vert_texture(texture: Option<Texture2D>) {
    BOX_SAC_VERT_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn box_sac_vert_texture() -> Option<Texture2D> {
    BOX_SAC_VERT_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_palette_logistique_texture(texture: Option<Texture2D>) {
    PALETTE_LOGISTIQUE_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn palette_logistique_texture() -> Option<Texture2D> {
    PALETTE_LOGISTIQUE_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_bureau_pc_on_texture(texture: Option<Texture2D>) {
    BUREAU_PC_ON_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn bureau_pc_on_texture() -> Option<Texture2D> {
    BUREAU_PC_ON_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_bureau_pc_off_texture(texture: Option<Texture2D>) {
    BUREAU_PC_OFF_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn bureau_pc_off_texture() -> Option<Texture2D> {
    BUREAU_PC_OFF_TEXTURE.with(|slot| slot.borrow().clone())
}

pub(crate) fn set_lavabo_texture(texture: Option<Texture2D>) {
    LAVABO_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
        }
        *slot.borrow_mut() = prepared;
    });
}

fn lavabo_texture() -> Option<Texture2D> {
    LAVABO_TEXTURE.with(|slot| slot.borrow().clone())
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

fn draw_text_lisible(text: &str, x: f32, y: f32, font_size: f32, fill: Color) {
    draw_text_shadowed(
        text,
        x,
        y,
        font_size,
        fill,
        ui_shadow_color_for_text(fill),
        ui_shadow_offset(font_size),
    );
}

fn draw_text_chip(
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    fill: Color,
    bg: Color,
    border: Color,
) {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    let pad_x = (font_size * 0.32).clamp(3.0, 6.0);
    let pad_top = (font_size * 0.22).clamp(2.0, 4.0);
    let rect = Rect::new(
        x - pad_x,
        y - font_size - pad_top,
        dims.width + pad_x * 2.0,
        font_size + pad_top + 5.0,
    );
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        (rect.w - 1.0).max(1.0),
        (rect.h - 1.0).max(1.0),
        1.0,
        border,
    );
    draw_text_lisible(text, x, y, font_size, fill);
}

pub(crate) fn draw_character_inspector_panel(state: &GameState, time: f32) {
    let panel_w = 380.0;
    let panel_h = 222.0;
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

    draw_text_lisible(
        "Inspecteur personnage (F2: afficher/masquer, F3: regenerer)",
        panel_x + 10.0,
        panel_y + 18.0,
        17.0,
        Color::from_rgba(240, 248, 255, 255),
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
        draw_text_lisible(&title, px - 22.0, py + 26.0, 14.0, WHITE);
        let summary = compact_visual_summary(record);
        draw_text_lisible(
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
            draw_text_lisible(
                line,
                panel_x + 10.0,
                panel_y + 126.0 + i as f32 * 12.5,
                12.0,
                Color::from_rgba(190, 210, 220, 255),
            );
        }
    }
}

pub(crate) fn draw_background(palette: &Palette, _time: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let lines = sh.max(1.0) as i32;

    for y in 0..lines {
        let t = y as f32 / (lines - 1).max(1) as f32;
        let c = color_lerp(palette.bg_top, palette.bg_bottom, t);
        draw_line(0.0, y as f32, sw, y as f32, 1.0, c);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TypeArbreExterieur {
    Chene,
    Peuplier,
    Pin,
}

const EXTERIOR_TREE_SCALE_MULTIPLIER: f32 = 3.0;

fn tile_is_exterior_ground(world: &World, x: i32, y: i32, tile: Tile) -> bool {
    if !tile_is_floor(tile) {
        return false;
    }
    if x <= 1 || y <= 1 || x >= world.w - 2 || y >= world.h - 2 {
        return false;
    }
    let (fx0, fx1, fy0, fy1) = starter_factory_bounds(world.w, world.h);
    !(x >= fx0 - 2 && x <= fx1 + 2 && y >= fy0 - 2 && y <= fy1 + 2)
}

fn tile_in_logistics_lane(world: &World, x: i32, y: i32) -> bool {
    let (fx0, fx1, fy0, fy1) = starter_factory_bounds(world.w, world.h);
    let span_y = (fy1 - fy0).max(6);
    let road_y = clamp_i32(fy0 + span_y / 2, fy0 + 2, fy1 - 2);
    let ship_y = clamp_i32(fy0 + (span_y * 3) / 4, fy0 + 2, fy1 - 2);
    let south_door_x = (fx0 + fx1) / 2;

    ((y - road_y).abs() <= 2 && x <= fx0 + 10)
        || ((y - ship_y).abs() <= 2 && x >= fx1 - 10)
        || ((x - south_door_x).abs() <= 2 && y >= fy1 - 1)
}

fn exterior_tree_type_for_tile(
    world: &World,
    x: i32,
    y: i32,
    tile: Tile,
) -> Option<TypeArbreExterieur> {
    if !matches!(tile, Tile::Floor | Tile::FloorMoss | Tile::FloorSand) {
        return None;
    }
    if !tile_is_exterior_ground(world, x, y, tile) || tile_in_logistics_lane(world, x, y) {
        return None;
    }

    const TREE_CELL_SIZE: i32 = 5;
    let cell_x = x.div_euclid(TREE_CELL_SIZE);
    let cell_y = y.div_euclid(TREE_CELL_SIZE);
    let cell_hash = hash_with_salt(cell_x, cell_y, 0xD19C);

    if cell_hash % 100 >= 84 {
        return None;
    }

    let local_x = (cell_hash % TREE_CELL_SIZE as u32) as i32;
    let local_y = ((cell_hash >> 3) % TREE_CELL_SIZE as u32) as i32;
    let anchor_x = cell_x * TREE_CELL_SIZE + local_x;
    let anchor_y = cell_y * TREE_CELL_SIZE + local_y;
    if x != anchor_x || y != anchor_y {
        return None;
    }

    Some(match (cell_hash >> 7) % 3 {
        0 => TypeArbreExterieur::Chene,
        1 => TypeArbreExterieur::Peuplier,
        _ => TypeArbreExterieur::Pin,
    })
}

fn draw_exterior_tree(kind: TypeArbreExterieur, x: i32, y: i32, palette: &Palette, time: f32) {
    let rect = World::tile_rect(x, y);
    let h = hash_with_salt(x, y, 0xB77A);
    let jitter_x = ((h & 0x7) as f32 - 3.5) * 0.45;
    let jitter_y = (((h >> 3) & 0x7) as f32 - 3.5) * 0.38;
    let scale = (0.88 + ((h >> 7) & 0x3) as f32 * 0.11) * EXTERIOR_TREE_SCALE_MULTIPLIER;
    let sway = (time * 0.52 + h as f32 * 0.013).sin() * (0.42 + ((h >> 9) & 0x3) as f32 * 0.08);

    let base_x = rect.x + rect.w * 0.5 + jitter_x;
    let base_y = rect.y + rect.h * 0.58 + jitter_y;

    draw_circle(
        base_x + 2.8,
        base_y + 11.2,
        8.6 * scale,
        with_alpha(palette.shadow_hard, 0.30),
    );

    let trunk_dark = rgba(78, 58, 38, 255);
    let trunk_light = rgba(114, 86, 57, 220);
    draw_rectangle(
        base_x - 2.3 * scale,
        base_y + 1.7 * scale,
        4.6 * scale,
        10.8 * scale,
        trunk_dark,
    );
    draw_rectangle(
        base_x - 0.9 * scale,
        base_y + 2.2 * scale,
        1.8 * scale,
        8.9 * scale,
        trunk_light,
    );

    match kind {
        TypeArbreExterieur::Chene => {
            let leaf_dark = rgba(42, 108, 58, 255);
            let leaf_mid = rgba(64, 136, 78, 255);
            let leaf_light = rgba(118, 186, 106, 255);
            draw_circle(
                base_x - 5.8 + sway * 0.35,
                base_y - 2.0,
                6.8 * scale,
                leaf_dark,
            );
            draw_circle(
                base_x + 5.3 + sway * 0.48,
                base_y - 2.6,
                6.3 * scale,
                leaf_mid,
            );
            draw_circle(
                base_x + 0.8 + sway * 0.32,
                base_y - 6.2,
                7.5 * scale,
                leaf_mid,
            );
            draw_circle(
                base_x - 1.1 + sway * 0.58,
                base_y - 4.4,
                4.2 * scale,
                with_alpha(leaf_light, 0.82),
            );
        }
        TypeArbreExterieur::Peuplier => {
            let leaf_dark = rgba(38, 94, 56, 255);
            let leaf_mid = rgba(62, 132, 80, 255);
            let leaf_light = rgba(126, 190, 122, 255);
            draw_circle(base_x + sway * 0.24, base_y - 8.0, 4.7 * scale, leaf_dark);
            draw_circle(base_x + sway * 0.28, base_y - 3.1, 5.3 * scale, leaf_mid);
            draw_circle(base_x + sway * 0.31, base_y + 2.2, 4.9 * scale, leaf_mid);
            draw_circle(
                base_x + sway * 0.18,
                base_y - 5.2,
                2.3 * scale,
                with_alpha(leaf_light, 0.84),
            );
        }
        TypeArbreExterieur::Pin => {
            let leaf_dark = rgba(36, 88, 54, 255);
            let leaf_mid = rgba(46, 112, 66, 255);
            let leaf_light = rgba(92, 158, 98, 235);
            draw_triangle(
                vec2(base_x + sway * 0.44, base_y - 11.0 * scale),
                vec2(base_x - 6.2 * scale + sway * 0.22, base_y - 2.2 * scale),
                vec2(base_x + 6.2 * scale + sway * 0.22, base_y - 2.2 * scale),
                leaf_dark,
            );
            draw_triangle(
                vec2(base_x + sway * 0.34, base_y - 7.0 * scale),
                vec2(base_x - 7.2 * scale + sway * 0.28, base_y + 1.4 * scale),
                vec2(base_x + 7.2 * scale + sway * 0.28, base_y + 1.4 * scale),
                leaf_mid,
            );
            draw_triangle(
                vec2(base_x + sway * 0.28, base_y - 3.8 * scale),
                vec2(base_x - 8.0 * scale + sway * 0.35, base_y + 5.8 * scale),
                vec2(base_x + 8.0 * scale + sway * 0.35, base_y + 5.8 * scale),
                leaf_light,
            );
        }
    }
}

pub(crate) fn draw_floor_tile(
    x: i32,
    y: i32,
    tile: Tile,
    palette: &Palette,
    _floor_texture: Option<&Texture2D>,
    _floor_metal_texture: Option<&Texture2D>,
    exterior_hint: bool,
) {
    let rect = World::tile_rect(x, y);
    let h = tile_hash(x, y);

    let (base_a, base_b) = match tile {
        Tile::Floor if exterior_hint => (
            color_lerp(rgba(86, 130, 76, 255), rgba(104, 146, 86, 255), 0.35),
            color_lerp(rgba(74, 116, 66, 255), rgba(94, 136, 82, 255), 0.35),
        ),
        Tile::Floor => (palette.floor_a, palette.floor_b),
        Tile::FloorMetal => (
            color_lerp(palette.floor_b, palette.wall_mid, 0.35),
            color_lerp(palette.floor_c, palette.wall_dark, 0.45),
        ),
        Tile::FloorWood => (
            color_lerp(palette.prop_crate_light, palette.prop_crate_dark, 0.18),
            color_lerp(palette.prop_crate_dark, palette.wall_dark, 0.28),
        ),
        Tile::FloorMoss => (
            color_lerp(rgba(66, 112, 82, 255), rgba(92, 146, 96, 255), 0.42),
            color_lerp(rgba(52, 88, 64, 255), rgba(76, 120, 80, 255), 0.44),
        ),
        Tile::FloorSand => (
            color_lerp(rgba(132, 120, 88, 255), rgba(98, 130, 80, 255), 0.42),
            color_lerp(rgba(118, 104, 78, 255), rgba(84, 116, 70, 255), 0.38),
        ),
        _ => (palette.floor_a, palette.floor_b),
    };

    let base = match h % 3 {
        0 => base_a,
        1 => base_b,
        _ => color_lerp(base_a, base_b, 0.55),
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, base);

    let tile_edge = if exterior_hint {
        color_lerp(palette.floor_edge, rgba(44, 88, 54, 190), 0.64)
    } else {
        palette.floor_edge
    };
    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.w - 1.0,
        rect.h - 1.0,
        1.0,
        tile_edge,
    );

    // Variation douce et homogène, sans symboles ni traits marqués.
    let soft_a = with_alpha(color_lerp(base_a, base_b, 0.66), 0.12);
    let soft_b = with_alpha(color_lerp(base_b, base_a, 0.42), 0.10);
    draw_circle(
        rect.x + 8.0 + (h & 3) as f32,
        rect.y + 10.0 + ((h >> 2) & 3) as f32,
        2.6,
        soft_a,
    );
    if h.is_multiple_of(3) {
        draw_circle(
            rect.x + 20.0 - ((h >> 3) & 3) as f32,
            rect.y + 21.0 - ((h >> 5) & 2) as f32,
            2.0,
            soft_b,
        );
    }

    match tile {
        Tile::FloorSand => {
            draw_circle(
                rect.x + 13.0 + ((h >> 1) & 2) as f32,
                rect.y + 15.0 + ((h >> 4) & 2) as f32,
                1.0,
                with_alpha(rgba(124, 136, 98, 255), 0.26),
            );
            draw_circle(
                rect.x + 18.0 - ((h >> 2) & 2) as f32,
                rect.y + 9.0 + ((h >> 5) & 2) as f32,
                0.9,
                with_alpha(rgba(116, 128, 92, 255), 0.22),
            );
        }
        Tile::FloorMoss => {
            let moss = with_alpha(rgba(100, 154, 104, 255), 0.22);
            draw_circle(
                rect.x + 9.0 + (h & 3) as f32,
                rect.y + 10.0 + ((h >> 2) & 3) as f32,
                3.0,
                moss,
            );
            if h.is_multiple_of(4) {
                draw_circle(
                    rect.x + 20.0 - (h & 2) as f32,
                    rect.y + 18.0 - ((h >> 3) & 2) as f32,
                    2.2,
                    with_alpha(rgba(82, 132, 92, 255), 0.22),
                );
            }
        }
        Tile::FloorWood => {
            let warm = with_alpha(rgba(184, 154, 110, 255), 0.11);
            draw_rectangle(rect.x + 2.0, rect.y + 6.0, rect.w - 4.0, 7.0, warm);
            draw_rectangle(
                rect.x + 2.0,
                rect.y + 19.0,
                rect.w - 4.0,
                7.0,
                with_alpha(warm, 0.7),
            );
        }
        _ => {}
    }

    let grime_strength = 0.03 + ((hash_with_salt(x, y, 13) & 0x0F) as f32 / 320.0);
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
                let exterior_hint = tile_is_exterior_ground(world, x, y, tile);
                draw_floor_tile(
                    x,
                    y,
                    tile,
                    palette,
                    floor_textures.floor.as_ref(),
                    floor_textures.floor_metal.as_ref(),
                    exterior_hint,
                );
            }
        }
    }
}

pub(crate) fn draw_exterior_ground_ambiance_region(
    world: &World,
    _palette: &Palette,
    time: f32,
    bounds: (i32, i32, i32, i32),
) {
    for y in bounds.2..=bounds.3 {
        for x in bounds.0..=bounds.1 {
            let tile = world.get(x, y);
            if !tile_is_exterior_ground(world, x, y, tile) {
                continue;
            }

            let rect = World::tile_rect(x, y);
            let h = hash_with_salt(x, y, 0x8F51);
            if h % 100 < 44 {
                let sway = (time * 0.9 + h as f32 * 0.021).sin() * 0.9;
                let blade_a = with_alpha(rgba(122, 184, 114, 255), 0.28);
                let blade_b = with_alpha(rgba(96, 154, 94, 255), 0.26);
                draw_line(
                    rect.x + 6.0 + (h & 3) as f32,
                    rect.y + 24.5,
                    rect.x + 6.8 + (h & 3) as f32 + sway * 0.2,
                    rect.y + 19.2,
                    1.0,
                    blade_a,
                );
                draw_line(
                    rect.x + 14.0 + ((h >> 2) & 3) as f32,
                    rect.y + 24.8,
                    rect.x + 14.4 + ((h >> 2) & 3) as f32 - sway * 0.22,
                    rect.y + 19.6,
                    1.0,
                    blade_b,
                );
                if h.is_multiple_of(7) {
                    draw_line(
                        rect.x + 21.0,
                        rect.y + 24.3,
                        rect.x + 21.6 + sway * 0.16,
                        rect.y + 19.4,
                        1.0,
                        blade_a,
                    );
                }
            }
            if h % 100 < 14 {
                let flower = if h & 1 == 0 {
                    rgba(236, 224, 148, 255)
                } else {
                    rgba(196, 220, 248, 255)
                };
                draw_circle(
                    rect.x + 11.0 + ((h >> 1) & 3) as f32,
                    rect.y + 14.0 + ((h >> 4) & 3) as f32,
                    0.9,
                    with_alpha(flower, 0.66),
                );
            }
        }
    }
}

pub(crate) fn draw_exterior_trees_region(
    world: &World,
    palette: &Palette,
    time: f32,
    bounds: (i32, i32, i32, i32),
) {
    for y in bounds.2..=bounds.3 {
        for x in bounds.0..=bounds.1 {
            let tile = world.get(x, y);
            if let Some(tree_type) = exterior_tree_type_for_tile(world, x, y, tile) {
                draw_exterior_tree(tree_type, x, y, palette, time);
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
            PropKind::Crate
            | PropKind::BoxCartonVide
            | PropKind::BoxSacBleu
            | PropKind::BoxSacRouge
            | PropKind::BoxSacVert
            | PropKind::CaisseAilBrut
            | PropKind::CaisseAilCasse => {
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
            PropKind::PaletteLogistique => {
                draw_rectangle(
                    x + 4.0,
                    y + 23.0,
                    24.0,
                    5.0,
                    with_alpha(palette.shadow_hard, 0.33),
                );
            }
            PropKind::BureauPcOn | PropKind::BureauPcOff => {
                draw_rectangle(
                    x + 3.0,
                    y + 22.0,
                    26.0,
                    6.0,
                    with_alpha(palette.shadow_hard, 0.34),
                );
            }
            PropKind::Lavabo => {
                draw_circle(
                    x + 16.0,
                    y + 23.5,
                    7.0,
                    with_alpha(palette.shadow_hard, 0.30),
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
    let box_carton_vide_texture = box_carton_vide_texture();
    let box_sac_bleu_texture = box_sac_bleu_texture();
    let box_sac_rouge_texture = box_sac_rouge_texture();
    let box_sac_vert_texture = box_sac_vert_texture();
    let palette_logistique_texture = palette_logistique_texture();
    let bureau_pc_on_texture = bureau_pc_on_texture();
    let bureau_pc_off_texture = bureau_pc_off_texture();
    let caisse_ail_brut_texture = storage_raw_texture();
    let caisse_ail_casse_texture = broken_garlic_crate_texture();
    let lavabo_texture = lavabo_texture();
    for prop in props {
        if !tile_in_bounds((prop.tile_x, prop.tile_y), bounds) {
            continue;
        }
        let x = prop.tile_x as f32 * TILE_SIZE;
        let y = prop.tile_y as f32 * TILE_SIZE;

        match prop.kind {
            PropKind::Crate => {
                if let Some(texture) = box_carton_vide_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 5.0, y + 7.0, 22.0, 22.0);
                } else {
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
            }
            PropKind::BoxCartonVide => {
                if let Some(texture) = box_carton_vide_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 5.0, y + 7.0, 22.0, 22.0);
                } else {
                    draw_rectangle(
                        x + 6.0,
                        y + 8.0,
                        20.0,
                        18.0,
                        color_lerp(palette.prop_crate_dark, palette.wall_dark, 0.18),
                    );
                    draw_rectangle_lines(x + 6.5, y + 8.5, 19.0, 17.0, 1.0, palette.wall_outline);
                }
            }
            PropKind::BoxSacBleu => {
                if let Some(texture) = box_sac_bleu_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 5.0, y + 7.0, 22.0, 22.0);
                } else {
                    draw_rectangle(x + 6.0, y + 8.0, 20.0, 18.0, rgba(70, 120, 182, 255));
                    draw_rectangle_lines(
                        x + 6.5,
                        y + 8.5,
                        19.0,
                        17.0,
                        1.0,
                        Color::from_rgba(190, 225, 252, 180),
                    );
                }
            }
            PropKind::BoxSacRouge => {
                if let Some(texture) = box_sac_rouge_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 5.0, y + 7.0, 22.0, 22.0);
                } else {
                    draw_rectangle(x + 6.0, y + 8.0, 20.0, 18.0, rgba(176, 82, 74, 255));
                    draw_rectangle_lines(
                        x + 6.5,
                        y + 8.5,
                        19.0,
                        17.0,
                        1.0,
                        Color::from_rgba(248, 210, 206, 180),
                    );
                }
            }
            PropKind::BoxSacVert => {
                if let Some(texture) = box_sac_vert_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 5.0, y + 7.0, 22.0, 22.0);
                } else {
                    draw_rectangle(x + 6.0, y + 8.0, 20.0, 18.0, rgba(70, 148, 100, 255));
                    draw_rectangle_lines(
                        x + 6.5,
                        y + 8.5,
                        19.0,
                        17.0,
                        1.0,
                        Color::from_rgba(206, 242, 216, 180),
                    );
                }
            }
            PropKind::CaisseAilBrut => {
                if let Some(texture) = caisse_ail_brut_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 5.0, y + 7.0, 22.0, 22.0);
                } else {
                    draw_rectangle(x + 6.0, y + 8.0, 20.0, 18.0, rgba(182, 146, 104, 255));
                    draw_rectangle_lines(
                        x + 6.5,
                        y + 8.5,
                        19.0,
                        17.0,
                        1.0,
                        Color::from_rgba(80, 60, 40, 220),
                    );
                }
            }
            PropKind::CaisseAilCasse => {
                if let Some(texture) = caisse_ail_casse_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 5.0, y + 7.0, 22.0, 22.0);
                } else {
                    draw_rectangle(x + 6.0, y + 8.0, 20.0, 18.0, rgba(206, 174, 118, 255));
                    draw_rectangle_lines(
                        x + 6.5,
                        y + 8.5,
                        19.0,
                        17.0,
                        1.0,
                        Color::from_rgba(94, 72, 44, 220),
                    );
                }
            }
            PropKind::PaletteLogistique => {
                if let Some(texture) = palette_logistique_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 4.0, y + 10.0, 24.0, 16.0);
                } else {
                    let wood = color_lerp(palette.prop_crate_light, palette.floor_b, 0.22);
                    draw_rectangle(x + 4.0, y + 18.0, 24.0, 3.0, wood);
                    draw_rectangle(x + 4.0, y + 22.0, 24.0, 3.0, wood);
                    draw_rectangle_lines(x + 4.0, y + 17.5, 24.0, 8.0, 1.0, palette.wall_outline);
                }
            }
            PropKind::BureauPcOn => {
                if let Some(texture) = bureau_pc_on_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 3.0, y + 6.0, 26.0, 24.0);
                } else {
                    draw_rectangle(x + 4.0, y + 14.0, 24.0, 10.0, rgba(92, 96, 108, 255));
                    draw_rectangle(x + 9.0, y + 9.0, 14.0, 6.0, rgba(70, 170, 120, 255));
                    draw_rectangle_lines(x + 4.5, y + 14.5, 23.0, 9.0, 1.0, palette.wall_outline);
                }
            }
            PropKind::BureauPcOff => {
                if let Some(texture) = bureau_pc_off_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 3.0, y + 6.0, 26.0, 24.0);
                } else {
                    draw_rectangle(x + 4.0, y + 14.0, 24.0, 10.0, rgba(92, 96, 108, 255));
                    draw_rectangle(x + 9.0, y + 9.0, 14.0, 6.0, rgba(76, 84, 92, 255));
                    draw_rectangle_lines(x + 4.5, y + 14.5, 23.0, 9.0, 1.0, palette.wall_outline);
                }
            }
            PropKind::Lavabo => {
                if let Some(texture) = lavabo_texture.as_ref() {
                    draw_prop_texture_scaled(texture, x + 4.0, y + 6.0, 24.0, 24.0);
                } else {
                    draw_rectangle(x + 8.0, y + 10.0, 16.0, 12.0, rgba(184, 196, 208, 255));
                    draw_circle(x + 16.0, y + 16.0, 3.0, rgba(138, 170, 198, 255));
                    draw_rectangle(x + 14.0, y + 22.0, 4.0, 4.0, rgba(122, 132, 142, 255));
                    draw_rectangle_lines(x + 8.5, y + 10.5, 15.0, 11.0, 1.0, palette.wall_outline);
                }
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
                    draw_prop_texture_scaled(texture, x + 5.0, y + 6.0 + bob, 22.0, 22.0);
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
        } else if matches!(prop.kind, PropKind::BureauPcOn) {
            let pulse = (time * 2.0 + prop.phase).sin() * 0.5 + 0.5;
            let glow = Color::new(0.48, 0.9, 0.72, 0.11 + pulse * 0.04);
            draw_circle(cx, cy + 9.0, 12.0 + pulse * 2.5, glow);
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

fn draw_menu_text_line(text: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text_shadowed(
        text,
        x,
        y,
        size,
        color,
        ui_shadow_color_for_text(color),
        ui_shadow_offset(size),
    );
}

pub(crate) fn run_main_menu_frame(
    map: &MapAsset,
    palette: &Palette,
    time: f32,
    frame_dt: f32,
    menu_state: &mut MainMenuState,
) -> MainMenuAction {
    tick_main_menu_status(menu_state, frame_dt);
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
            Color::from_rgba(8, 14, 22, 52),
        );
    } else {
        let bg_time = if menu_state.ambiance_motion {
            time * 0.58
        } else {
            0.0
        };
        draw_background(palette, bg_time);
        let (menu_camera, menu_view_rect) = fit_world_camera_to_screen(&map.world, 34.0);
        let visible_bounds = tile_bounds_from_camera(&map.world, &menu_camera, menu_view_rect, 2);
        set_camera(&menu_camera);
        draw_floor_layer_region(&map.world, palette, visible_bounds);
        draw_exterior_ground_ambiance_region(&map.world, palette, time, visible_bounds);
        draw_wall_cast_shadows_region(&map.world, palette, visible_bounds);
        draw_wall_layer_region(&map.world, palette, visible_bounds);
        draw_exterior_trees_region(&map.world, palette, time, visible_bounds);
        draw_prop_shadows_region(&map.props, palette, time, visible_bounds);
        draw_props_region(&map.props, palette, time, visible_bounds);
        draw_lighting_region(&map.props, palette, time, visible_bounds);
        begin_ui_pass();
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::from_rgba(8, 14, 22, 44),
        );
    }

    let sw = screen_width();
    let sh = screen_height();
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let left_click = is_mouse_button_pressed(MouseButton::Left);
    let wheel_y = mouse_wheel().1;
    let has_save = !menu_state.saves.is_empty();

    draw_rectangle(
        0.0,
        sh * 0.84,
        sw,
        sh * 0.16,
        Color::from_rgba(8, 14, 22, 40),
    );

    let button_w = (sw * 0.28).clamp(300.0, 420.0);
    let button_h = (sh * 0.07).clamp(48.0, 62.0);
    let button_gap = (button_h * 0.22).clamp(10.0, 14.0);
    let button_count = 6.0;
    let stack_h = button_h * button_count + button_gap * (button_count - 1.0);
    let start_y = (sh * 0.52 - stack_h * 0.5).max(sh * 0.24);
    let bx = sw * 0.5 - button_w * 0.5;
    let new_game_rect = Rect::new(bx, start_y, button_w, button_h);
    let continue_rect = Rect::new(
        bx,
        new_game_rect.y + button_h + button_gap,
        button_w,
        button_h,
    );
    let load_rect = Rect::new(
        bx,
        continue_rect.y + button_h + button_gap,
        button_w,
        button_h,
    );
    let editor_rect = Rect::new(bx, load_rect.y + button_h + button_gap, button_w, button_h);
    let options_rect = Rect::new(
        bx,
        editor_rect.y + button_h + button_gap,
        button_w,
        button_h,
    );
    let quit_rect = Rect::new(
        bx,
        options_rect.y + button_h + button_gap,
        button_w,
        button_h,
    );

    if draw_ui_button_sized(
        new_game_rect,
        "Nouvelle partie",
        mouse,
        left_click,
        false,
        21.0,
    ) || is_key_pressed(KeyCode::N)
    {
        return MainMenuAction::StartNewGame;
    }

    let continue_clicked = draw_ui_button_sized(
        continue_rect,
        "Continuer partie",
        mouse,
        left_click,
        false,
        21.0,
    );
    if !has_save {
        draw_rectangle(
            continue_rect.x,
            continue_rect.y,
            continue_rect.w,
            continue_rect.h,
            Color::from_rgba(10, 12, 16, 186),
        );
        let no_save_text = "Aucune sauvegarde";
        let no_save_dims = measure_text(no_save_text, None, 15, 1.0);
        draw_menu_text_line(
            no_save_text,
            continue_rect.x + continue_rect.w * 0.5 - no_save_dims.width * 0.5,
            continue_rect.y + continue_rect.h * 0.5 + 5.0,
            15.0,
            Color::from_rgba(164, 178, 192, 245),
        );
    }
    if (continue_clicked || is_key_pressed(KeyCode::Enter))
        && menu_state.view == MainMenuView::Principal
    {
        if !has_save {
            refresh_main_menu_saves(menu_state);
            if menu_state.saves.is_empty() {
                set_main_menu_status(menu_state, "Aucune sauvegarde disponible.");
            } else if let Some(slot) = menu_state.saves.first() {
                return MainMenuAction::StartFromSave(slot.file_name.clone());
            }
        } else if let Some(slot) = menu_state.saves.first() {
            return MainMenuAction::StartFromSave(slot.file_name.clone());
        }
    }

    if draw_ui_button_sized(
        load_rect,
        "Charger",
        mouse,
        left_click,
        menu_state.view == MainMenuView::Charger,
        21.0,
    ) {
        menu_state.view = MainMenuView::Charger;
        refresh_main_menu_saves(menu_state);
    }
    if draw_ui_button_sized(editor_rect, "Editeur", mouse, left_click, false, 21.0)
        || is_key_pressed(KeyCode::E)
    {
        return MainMenuAction::OpenEditor;
    }
    if draw_ui_button_sized(
        options_rect,
        "Options",
        mouse,
        left_click,
        menu_state.view == MainMenuView::Options,
        21.0,
    ) {
        menu_state.view = MainMenuView::Options;
    }
    if draw_ui_button_sized(quit_rect, "Quitter", mouse, left_click, false, 21.0)
        || is_key_pressed(KeyCode::Q)
    {
        return MainMenuAction::Quit;
    }

    match menu_state.view {
        MainMenuView::Principal => {
            let info_text = if let Some(slot) = menu_state.saves.first() {
                format!(
                    "Derniere sauvegarde: {} ({})",
                    slot.save_name, slot.saved_at_label
                )
            } else {
                "Aucune sauvegarde detectee.".to_string()
            };
            let info_dims = measure_text(&info_text, None, 21, 1.0);
            draw_menu_text_line(
                &info_text,
                sw * 0.5 - info_dims.width * 0.5,
                quit_rect.y + button_h + 36.0,
                21.0,
                Color::from_rgba(198, 222, 236, 255),
            );
        }
        MainMenuView::Charger => {
            let popup_w = (sw * 0.56).clamp(560.0, 860.0);
            let popup_h = (sh * 0.36).clamp(260.0, 340.0);
            let popup_rect = Rect::new(
                sw * 0.5 - popup_w * 0.5,
                quit_rect.y + button_h + 20.0,
                popup_w,
                popup_h,
            );
            draw_rectangle(
                popup_rect.x,
                popup_rect.y,
                popup_rect.w,
                popup_rect.h,
                Color::from_rgba(8, 18, 28, 210),
            );
            draw_rectangle_lines(
                popup_rect.x + 0.5,
                popup_rect.y + 0.5,
                popup_rect.w - 1.0,
                popup_rect.h - 1.0,
                1.4,
                Color::from_rgba(96, 150, 180, 208),
            );
            draw_menu_text_line(
                "Charger",
                popup_rect.x + 14.0,
                popup_rect.y + 24.0,
                24.0,
                Color::from_rgba(236, 248, 255, 255),
            );

            let action_y = popup_rect.y + 36.0;
            let action_w = ((popup_rect.w - 54.0) / 3.0).max(140.0);
            let play_rect = Rect::new(popup_rect.x + 14.0, action_y, action_w, 34.0);
            let refresh_rect =
                Rect::new(play_rect.x + play_rect.w + 10.0, action_y, action_w, 34.0);
            let close_rect = Rect::new(
                refresh_rect.x + refresh_rect.w + 10.0,
                action_y,
                action_w,
                34.0,
            );
            if draw_ui_button_sized(
                play_rect,
                "Charger selection",
                mouse,
                left_click,
                false,
                15.0,
            ) || is_key_pressed(KeyCode::Space)
            {
                if let Some(selected) = menu_state.selected_save
                    && let Some(slot) = menu_state.saves.get(selected)
                {
                    return MainMenuAction::StartFromSave(slot.file_name.clone());
                }
                set_main_menu_status(menu_state, "Selectionne une sauvegarde.");
            }
            if draw_ui_button_sized(refresh_rect, "Rafraichir", mouse, left_click, false, 15.0) {
                refresh_main_menu_saves(menu_state);
            }
            if draw_ui_button_sized(close_rect, "Fermer", mouse, left_click, false, 15.0) {
                menu_state.view = MainMenuView::Principal;
            }

            let list_rect = Rect::new(
                popup_rect.x + 14.0,
                action_y + 48.0,
                popup_rect.w - 28.0,
                popup_rect.h - 58.0,
            );
            draw_rectangle(
                list_rect.x,
                list_rect.y,
                list_rect.w,
                list_rect.h,
                Color::from_rgba(14, 26, 38, 222),
            );
            draw_rectangle_lines(
                list_rect.x + 0.5,
                list_rect.y + 0.5,
                list_rect.w - 1.0,
                list_rect.h - 1.0,
                1.2,
                Color::from_rgba(94, 138, 166, 210),
            );

            let row_h = 40.0;
            let visible_rows = ((list_rect.h - 8.0) / row_h).floor().max(1.0) as usize;
            let max_offset = menu_state.saves.len().saturating_sub(visible_rows);
            menu_state.saves_offset = menu_state.saves_offset.min(max_offset);

            if point_in_rect(mouse, list_rect) && wheel_y.abs() > f32::EPSILON {
                if wheel_y > 0.0 {
                    menu_state.saves_offset = menu_state.saves_offset.saturating_sub(1);
                } else {
                    menu_state.saves_offset = (menu_state.saves_offset + 1).min(max_offset);
                }
            }

            if menu_state.saves.is_empty() {
                draw_menu_text_line(
                    "Aucune sauvegarde trouvee dans saves/",
                    list_rect.x + 10.0,
                    list_rect.y + 24.0,
                    16.0,
                    Color::from_rgba(178, 206, 224, 255),
                );
            } else {
                let start = menu_state.saves_offset;
                let end = (start + visible_rows).min(menu_state.saves.len());
                let mut row_y = list_rect.y + 4.0;
                for idx in start..end {
                    let slot = &menu_state.saves[idx];
                    let row_rect =
                        Rect::new(list_rect.x + 4.0, row_y, list_rect.w - 8.0, row_h - 2.0);
                    let hovered = point_in_rect(mouse, row_rect);
                    let selected = menu_state.selected_save == Some(idx);
                    let fill = if selected {
                        Color::from_rgba(74, 120, 150, 230)
                    } else if hovered {
                        Color::from_rgba(52, 84, 108, 222)
                    } else {
                        Color::from_rgba(28, 46, 62, 214)
                    };
                    draw_rectangle(row_rect.x, row_rect.y, row_rect.w, row_rect.h, fill);
                    draw_rectangle_lines(
                        row_rect.x + 0.5,
                        row_rect.y + 0.5,
                        row_rect.w - 1.0,
                        row_rect.h - 1.0,
                        1.0,
                        if selected {
                            Color::from_rgba(220, 240, 252, 242)
                        } else {
                            Color::from_rgba(106, 154, 182, 196)
                        },
                    );
                    draw_menu_text_line(
                        &slot.save_name,
                        row_rect.x + 8.0,
                        row_rect.y + 18.0,
                        15.0,
                        Color::from_rgba(236, 246, 255, 255),
                    );
                    draw_menu_text_line(
                        &slot.saved_at_label,
                        row_rect.x + 8.0,
                        row_rect.y + 33.0,
                        12.0,
                        Color::from_rgba(192, 220, 238, 245),
                    );
                    if hovered && left_click {
                        menu_state.selected_save = Some(idx);
                    }
                    row_y += row_h;
                }
            }

            if is_key_pressed(KeyCode::Up)
                && let Some(selected) = menu_state.selected_save
            {
                menu_state.selected_save = Some(selected.saturating_sub(1));
            }
            if is_key_pressed(KeyCode::Down) {
                if let Some(selected) = menu_state.selected_save {
                    let max_index = menu_state.saves.len().saturating_sub(1);
                    menu_state.selected_save = Some((selected + 1).min(max_index));
                } else if !menu_state.saves.is_empty() {
                    menu_state.selected_save = Some(0);
                }
            }
            if is_key_pressed(KeyCode::Enter)
                && let Some(selected) = menu_state.selected_save
                && let Some(slot) = menu_state.saves.get(selected)
            {
                return MainMenuAction::StartFromSave(slot.file_name.clone());
            }
        }
        MainMenuView::Options => {
            let popup_w = (sw * 0.48).clamp(500.0, 700.0);
            let popup_h = (sh * 0.24).clamp(190.0, 240.0);
            let popup_rect = Rect::new(
                sw * 0.5 - popup_w * 0.5,
                quit_rect.y + button_h + 24.0,
                popup_w,
                popup_h,
            );
            draw_rectangle(
                popup_rect.x,
                popup_rect.y,
                popup_rect.w,
                popup_rect.h,
                Color::from_rgba(8, 18, 28, 208),
            );
            draw_rectangle_lines(
                popup_rect.x + 0.5,
                popup_rect.y + 0.5,
                popup_rect.w - 1.0,
                popup_rect.h - 1.0,
                1.4,
                Color::from_rgba(96, 150, 180, 208),
            );
            draw_menu_text_line(
                "Options",
                popup_rect.x + 14.0,
                popup_rect.y + 24.0,
                24.0,
                Color::from_rgba(236, 248, 255, 255),
            );

            let opt_line_h = 48.0;
            let start_y = popup_rect.y + 38.0;
            let opt1_rect = Rect::new(popup_rect.x + 14.0, start_y, popup_rect.w - 28.0, 38.0);
            let opt2_rect = Rect::new(
                popup_rect.x + 14.0,
                start_y + opt_line_h,
                popup_rect.w - 28.0,
                38.0,
            );
            if draw_ui_button_sized(
                opt1_rect,
                &format!(
                    "Afficher FPS menu: {}",
                    if menu_state.show_fps { "ON" } else { "OFF" }
                ),
                mouse,
                left_click,
                menu_state.show_fps,
                15.0,
            ) {
                menu_state.show_fps = !menu_state.show_fps;
            }
            if draw_ui_button_sized(
                opt2_rect,
                &format!(
                    "Ambiance animee: {}",
                    if menu_state.ambiance_motion {
                        "ON"
                    } else {
                        "OFF"
                    }
                ),
                mouse,
                left_click,
                menu_state.ambiance_motion,
                15.0,
            ) {
                menu_state.ambiance_motion = !menu_state.ambiance_motion;
            }
            let close_rect = Rect::new(
                popup_rect.x + popup_rect.w - 154.0,
                popup_rect.y + popup_rect.h - 44.0,
                140.0,
                30.0,
            );
            if draw_ui_button_sized(close_rect, "Fermer", mouse, left_click, false, 14.0) {
                menu_state.view = MainMenuView::Principal;
            }
        }
    }

    if is_key_pressed(KeyCode::Escape) && menu_state.view != MainMenuView::Principal {
        menu_state.view = MainMenuView::Principal;
    }

    if let Some(warn) = menu_state.saves_warning.as_deref() {
        draw_menu_text_line(
            warn,
            sw * 0.5 - (sw * 0.32),
            sh - 18.0,
            14.0,
            Color::from_rgba(244, 214, 146, 255),
        );
    }
    if let Some(status) = menu_state.status_text.as_deref()
        && menu_state.status_timer > 0.0
    {
        let status_dims = measure_text(status, None, 16, 1.0);
        draw_menu_text_line(
            status,
            sw * 0.5 - status_dims.width * 0.5,
            sh - 38.0,
            16.0,
            Color::from_rgba(244, 214, 146, 255),
        );
    }

    if menu_state.show_fps {
        draw_menu_text_line(
            &format!("FPS {}", get_fps()),
            sw - 112.0,
            24.0,
            16.0,
            Color::from_rgba(196, 228, 242, 255),
        );
    }

    MainMenuAction::None
}

pub(crate) fn sim_zone_overlay_color(zone: sim::ZoneKind) -> Option<Color> {
    match zone {
        sim::ZoneKind::Neutral => None,
        sim::ZoneKind::Receiving => Some(Color::from_rgba(70, 126, 236, 66)),
        sim::ZoneKind::Processing => Some(Color::from_rgba(232, 122, 52, 68)),
        sim::ZoneKind::Shipping => Some(Color::from_rgba(86, 188, 132, 66)),
        sim::ZoneKind::Support => Some(Color::from_rgba(220, 146, 84, 62)),
    }
}

pub(crate) fn sim_block_overlay_color(kind: sim::BlockKind) -> Color {
    match kind {
        sim::BlockKind::InputHopper => Color::from_rgba(170, 188, 204, 255),
        sim::BlockKind::Conveyor => Color::from_rgba(58, 142, 238, 255),
        sim::BlockKind::FluidityTank => Color::from_rgba(70, 154, 198, 255),
        sim::BlockKind::Cutter => Color::from_rgba(164, 176, 196, 255),
        sim::BlockKind::DistributorBelt => Color::from_rgba(64, 140, 232, 255),
        sim::BlockKind::DryerOven => Color::from_rgba(204, 150, 120, 255),
        sim::BlockKind::OvenExitConveyor => Color::from_rgba(80, 132, 204, 255),
        sim::BlockKind::Flaker => Color::from_rgba(196, 170, 132, 255),
        sim::BlockKind::SuctionPipe => Color::from_rgba(154, 170, 188, 255),
        sim::BlockKind::Sortex => Color::from_rgba(114, 194, 148, 255),
        sim::BlockKind::BlueBagChute => Color::from_rgba(98, 162, 236, 255),
        sim::BlockKind::RedBagChute => Color::from_rgba(234, 124, 108, 255),
        sim::BlockKind::Storage => Color::from_rgba(94, 160, 230, 255),
        sim::BlockKind::MachineA => Color::from_rgba(190, 204, 220, 255),
        sim::BlockKind::MachineB => Color::from_rgba(170, 188, 210, 255),
        sim::BlockKind::Buffer => Color::from_rgba(142, 122, 208, 255),
        sim::BlockKind::Seller => Color::from_rgba(94, 196, 124, 255),
    }
}

fn sim_block_rect(tile: (i32, i32), footprint: (i32, i32)) -> Rect {
    let origin = World::tile_rect(tile.0, tile.1);
    Rect::new(
        origin.x,
        origin.y,
        origin.w * footprint.0.max(1) as f32,
        origin.h * footprint.1.max(1) as f32,
    )
}

fn block_intersects_bounds(
    tile: (i32, i32),
    footprint: (i32, i32),
    bounds: (i32, i32, i32, i32),
) -> bool {
    let x0 = tile.0;
    let y0 = tile.1;
    let x1 = x0 + footprint.0.max(1) - 1;
    let y1 = y0 + footprint.1.max(1) - 1;
    !(x1 < bounds.0 || x0 > bounds.1 || y1 < bounds.2 || y0 > bounds.3)
}

fn block_occupies_tile(block: &sim::BlockDebugView, tile: (i32, i32)) -> bool {
    tile.0 >= block.tile.0
        && tile.0 < block.tile.0 + block.footprint.0.max(1)
        && tile.1 >= block.tile.1
        && tile.1 < block.tile.1 + block.footprint.1.max(1)
}

fn orientation_axis(orientation: sim::BlockOrientation) -> Vec2 {
    match orientation {
        sim::BlockOrientation::East => vec2(1.0, 0.0),
        sim::BlockOrientation::South => vec2(0.0, 1.0),
        sim::BlockOrientation::West => vec2(-1.0, 0.0),
        sim::BlockOrientation::North => vec2(0.0, -1.0),
    }
}

fn draw_belt_motion(
    rect: Rect,
    orientation: sim::BlockOrientation,
    time: f32,
    base: Color,
    stripe: Color,
    border: Color,
) {
    let axis = orientation_axis(orientation);
    let horizontal = axis.x.abs() > axis.y.abs();
    let steel = with_alpha(Color::from_rgba(164, 178, 194, 255), 0.95);
    let steel_dark = with_alpha(Color::from_rgba(102, 116, 132, 255), 0.94);
    let steel_glow = with_alpha(Color::from_rgba(198, 214, 236, 255), 0.44);
    let belt = with_alpha(base, 0.68);
    let gloss = with_alpha(Color::from_rgba(186, 238, 255, 210), 0.45);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, steel_dark);
    draw_rectangle(
        rect.x + 0.8,
        rect.y + 0.8,
        (rect.w - 1.6).max(1.0),
        (rect.h - 1.6).max(1.0),
        steel,
    );
    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.w - 1.0,
        rect.h - 1.0,
        1.5,
        with_alpha(border, 0.85),
    );

    let rail_w = rect.w.min(rect.h) * 0.12;
    if horizontal {
        draw_rectangle(
            rect.x + 1.1,
            rect.y + rect.h * 0.26,
            (rect.w - 2.2).max(1.0),
            rail_w.clamp(1.5, 3.2),
            with_alpha(border, 0.26),
        );
        draw_rectangle(
            rect.x + 1.1,
            rect.y + rect.h * 0.74 - rail_w,
            (rect.w - 2.2).max(1.0),
            rail_w.clamp(1.5, 3.2),
            with_alpha(border, 0.2),
        );
    } else {
        draw_rectangle(
            rect.x + rect.w * 0.26,
            rect.y + 1.1,
            rail_w.clamp(1.5, 3.2),
            (rect.h - 2.2).max(1.0),
            with_alpha(border, 0.26),
        );
        draw_rectangle(
            rect.x + rect.w * 0.74 - rail_w,
            rect.y + 1.1,
            rail_w.clamp(1.5, 3.2),
            (rect.h - 2.2).max(1.0),
            with_alpha(border, 0.2),
        );
    }

    let lane = if horizontal {
        Rect::new(
            rect.x + 2.0,
            rect.y + rect.h * 0.33,
            (rect.w - 4.0).max(1.0),
            (rect.h * 0.34).max(1.0),
        )
    } else {
        Rect::new(
            rect.x + rect.w * 0.33,
            rect.y + 2.0,
            (rect.w * 0.34).max(1.0),
            (rect.h - 4.0).max(1.0),
        )
    };
    draw_rectangle(lane.x, lane.y, lane.w, lane.h, with_alpha(belt, 0.58));
    draw_rectangle(
        lane.x + 1.2,
        lane.y + (lane.h * 0.33),
        (lane.w - 2.4).max(1.0),
        lane.h * 0.08,
        with_alpha(gloss, 0.78),
    );
    if horizontal {
        let stroke_count = 6usize;
        for i in 0..stroke_count {
            let ratio = (i as f32 + 0.5) / stroke_count as f32;
            draw_line(
                lane.x + lane.w * ratio,
                lane.y + 1.3,
                lane.x + lane.w * ratio,
                lane.y + lane.h - 1.3,
                0.7,
                with_alpha(steel_dark, 0.26),
            );
        }
    } else {
        let stroke_count = 6usize;
        for i in 0..stroke_count {
            let ratio = (i as f32 + 0.5) / stroke_count as f32;
            draw_line(
                lane.x + 1.3,
                lane.y + lane.h * ratio,
                lane.x + lane.w - 1.3,
                lane.y + lane.h * ratio,
                0.7,
                with_alpha(steel_dark, 0.26),
            );
        }
    }

    let spacing = if horizontal {
        (rect.w * 0.16).clamp(6.0, 15.0)
    } else {
        (rect.h * 0.16).clamp(6.0, 15.0)
    };
    let phase = (time * 28.0).rem_euclid(spacing);
    let pulse = (time * 1.4).sin() * 0.16;
    let dash_w = if horizontal {
        (lane.h * 0.48).clamp(2.0, 7.0)
    } else {
        (lane.w * 0.48).clamp(2.0, 7.0)
    };
    let move_pos = if horizontal {
        axis.x >= 0.0
    } else {
        axis.y >= 0.0
    };

    if horizontal {
        let mut x = if move_pos {
            lane.x - spacing + phase
        } else {
            lane.x + lane.w + phase
        };
        let limit = if move_pos {
            lane.x + lane.w + spacing
        } else {
            lane.x - spacing
        };
        while if move_pos { x < limit } else { x > limit } {
            draw_rectangle(
                x,
                lane.y + (lane.h - dash_w) * 0.5 + pulse,
                dash_w,
                dash_w * 1.4,
                with_alpha(with_alpha(stripe, 0.9), 0.95),
            );
            x += if move_pos { spacing } else { -spacing };
        }
    } else {
        let mut y = if move_pos {
            lane.y - spacing + phase
        } else {
            lane.y + lane.h + phase
        };
        let limit = if move_pos {
            lane.y + lane.h + spacing
        } else {
            lane.y - spacing
        };
        while if move_pos { y < limit } else { y > limit } {
            draw_rectangle(
                lane.x + (lane.w - dash_w) * 0.5 + pulse,
                y,
                dash_w * 1.4,
                dash_w,
                with_alpha(with_alpha(stripe, 0.9), 0.95),
            );
            y += if move_pos { spacing } else { -spacing };
        }
    }

    draw_rectangle(
        rect.x + 1.0,
        rect.y + 1.0,
        (rect.w - 2.0).max(1.0),
        (rect.h - 2.0).max(1.0),
        steel_glow,
    );

    let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    let tip = center + axis * rect.w.min(rect.h) * 0.22;
    let base_center = center - axis * rect.w.min(rect.h) * 0.13;
    let normal = vec2(-axis.y, axis.x) * rect.w.min(rect.h) * 0.1;
    draw_triangle(
        tip,
        base_center + normal,
        base_center - normal,
        with_alpha(WHITE, 0.45),
    );
    draw_line(
        base_center.x,
        base_center.y,
        tip.x,
        tip.y,
        1.0,
        with_alpha(Color::from_rgba(210, 236, 255, 240), 0.58),
    );
}

fn fract01(v: f32) -> f32 {
    let f = v.fract();
    if f < 0.0 { f + 1.0 } else { f }
}

fn rect_center(rect: Rect) -> Vec2 {
    vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5)
}

fn rect_inset(rect: Rect, inset: f32) -> Rect {
    Rect::new(
        rect.x + inset,
        rect.y + inset,
        (rect.w - 2.0 * inset).max(0.0),
        (rect.h - 2.0 * inset).max(0.0),
    )
}

fn rect_inset_xy(rect: Rect, inset_x: f32, inset_y: f32) -> Rect {
    Rect::new(
        rect.x + inset_x,
        rect.y + inset_y,
        (rect.w - 2.0 * inset_x).max(0.0),
        (rect.h - 2.0 * inset_y).max(0.0),
    )
}

#[derive(Clone, Copy)]
struct BlockBasis {
    c: Vec2,
    axis: Vec2,
    normal: Vec2,
    along: f32,
    across: f32,
}

impl BlockBasis {
    fn p(&self, u: f32, v: f32) -> Vec2 {
        self.c + self.axis * (u * self.along) + self.normal * (v * self.across)
    }
}

fn basis_for_block(rect: Rect, orientation: sim::BlockOrientation) -> BlockBasis {
    let axis = orientation_axis(orientation);
    let normal = vec2(-axis.y, axis.x);
    let horizontal = axis.x.abs() > axis.y.abs();
    let along = if horizontal {
        rect.w * 0.5
    } else {
        rect.h * 0.5
    };
    let across = if horizontal {
        rect.h * 0.5
    } else {
        rect.w * 0.5
    };
    BlockBasis {
        c: rect_center(rect),
        axis,
        normal,
        along,
        across,
    }
}

fn draw_oriented_quad(b: &BlockBasis, u0: f32, v0: f32, u1: f32, v1: f32, color: Color) {
    let p00 = b.p(u0, v0);
    let p10 = b.p(u1, v0);
    let p11 = b.p(u1, v1);
    let p01 = b.p(u0, v1);
    draw_triangle(p00, p10, p11, color);
    draw_triangle(p00, p11, p01, color);
}

fn draw_oriented_quad_lines(
    b: &BlockBasis,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    thickness: f32,
    color: Color,
) {
    let p00 = b.p(u0, v0);
    let p10 = b.p(u1, v0);
    let p11 = b.p(u1, v1);
    let p01 = b.p(u0, v1);
    let t = thickness.max(0.6);
    draw_line(p00.x, p00.y, p10.x, p10.y, t, color);
    draw_line(p10.x, p10.y, p11.x, p11.y, t, color);
    draw_line(p11.x, p11.y, p01.x, p01.y, t, color);
    draw_line(p01.x, p01.y, p00.x, p00.y, t, color);
}

fn draw_soft_shadow_rect(rect: Rect, offset: Vec2, spread: f32, alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let steps = 4;
    for i in 0..steps {
        let t = i as f32 / (steps as f32 - 1.0);
        let grow = spread * (0.35 + t);
        let a = alpha * (1.0 - t) * 0.55;
        draw_rectangle(
            rect.x + offset.x - grow,
            rect.y + offset.y - grow,
            rect.w + grow * 2.0,
            rect.h + grow * 2.0,
            with_alpha(Color::from_rgba(0, 0, 0, 255), a),
        );
    }
}

fn draw_vertical_gradient(rect: Rect, top: Color, bottom: Color, steps: usize) {
    let steps = steps.max(1);
    let denom = (steps as f32 - 1.0).max(1.0);
    let h = rect.h / steps as f32;
    for i in 0..steps {
        let t = i as f32 / denom;
        draw_rectangle(
            rect.x,
            rect.y + i as f32 * h,
            rect.w,
            h + 0.6,
            color_lerp(top, bottom, t),
        );
    }
}

fn draw_bevel_edges(rect: Rect, thickness: f32, light: Color, dark: Color) {
    let t = thickness.max(0.7);
    draw_line(
        rect.x + 0.6,
        rect.y + 0.8,
        rect.x + rect.w - 0.8,
        rect.y + 0.8,
        t,
        light,
    );
    draw_line(
        rect.x + 0.8,
        rect.y + 0.6,
        rect.x + 0.8,
        rect.y + rect.h - 0.8,
        t,
        light,
    );
    draw_line(
        rect.x + 0.8,
        rect.y + rect.h - 0.8,
        rect.x + rect.w - 0.8,
        rect.y + rect.h - 0.8,
        t,
        dark,
    );
    draw_line(
        rect.x + rect.w - 0.8,
        rect.y + 0.8,
        rect.x + rect.w - 0.8,
        rect.y + rect.h - 0.8,
        t,
        dark,
    );
}

thread_local! {
    static LINE_GRAIN_TEX: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
}

fn with_line_grain_tex<R>(f: impl FnOnce(&Texture2D) -> R) -> R {
    LINE_GRAIN_TEX.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() {
            let w: u16 = 64;
            let h: u16 = 64;
            let mut img = Image::gen_image_color(w, h, Color::from_rgba(128, 128, 128, 255));
            for y in 0..h as i32 {
                for x in 0..w as i32 {
                    let hh = tile_hash(x, y);
                    let mut v = 118 + (hh & 0x3f) as i32 - 31;
                    v = v.clamp(72, 176);
                    let vv = v as u8;
                    img.set_pixel(x as u32, y as u32, Color::from_rgba(vv, vv, vv, 255));
                }
            }
            let tex = Texture2D::from_image(&img);
            tex.set_filter(FilterMode::Linear);
            *opt = Some(tex);
        }
        f(opt.as_ref().unwrap())
    })
}

fn draw_grain_overlay(rect: Rect, alpha: f32) {
    if alpha <= 0.0 || rect.w <= 2.0 || rect.h <= 2.0 {
        return;
    }
    with_line_grain_tex(|tex| {
        draw_texture_ex(
            tex,
            rect.x,
            rect.y,
            with_alpha(WHITE, alpha.clamp(0.0, 1.0)),
            DrawTextureParams {
                dest_size: Some(vec2(rect.w, rect.h)),
                ..Default::default()
            },
        );
    });
}

fn draw_panel(rect: Rect, base: Color, outline: Color, grain_alpha: f32) {
    let top = color_lerp(base, Color::from_rgba(255, 255, 255, 255), 0.22);
    let bottom = color_lerp(base, Color::from_rgba(0, 0, 0, 255), 0.22);
    draw_vertical_gradient(rect, top, bottom, 8);
    draw_grain_overlay(rect, grain_alpha);
    draw_bevel_edges(
        rect,
        1.1,
        with_alpha(Color::from_rgba(255, 255, 255, 255), 0.10),
        with_alpha(Color::from_rgba(0, 0, 0, 255), 0.28),
    );
    draw_rectangle_lines(
        rect.x + 0.8,
        rect.y + 0.8,
        (rect.w - 1.6).max(1.0),
        (rect.h - 1.6).max(1.0),
        1.35,
        outline,
    );
}

fn draw_rivet(p: Vec2, r: f32, base: Color) {
    let rr = r.max(0.7);
    draw_circle(p.x, p.y, rr, base);
    draw_circle(
        p.x - rr * 0.3,
        p.y - rr * 0.3,
        rr * 0.35,
        with_alpha(WHITE, 0.22),
    );
    draw_circle_lines(p.x, p.y, rr, 0.75, with_alpha(BLACK, 0.35));
}

fn draw_led(p: Vec2, r: f32, col: Color, intensity: f32) {
    let i = intensity.clamp(0.0, 1.0);
    let rr = r.max(1.0);
    draw_circle(p.x, p.y, rr * 2.4, with_alpha(col, 0.14 * i));
    draw_circle(p.x, p.y, rr * 1.5, with_alpha(col, 0.25 * i));
    draw_circle(p.x, p.y, rr, with_alpha(col, 0.82 * i));
    draw_circle(
        p.x - rr * 0.25,
        p.y - rr * 0.25,
        rr * 0.35,
        with_alpha(WHITE, 0.20 * i),
    );
}

fn draw_progress_bar(rect: Rect, ratio: f32, fg: Color) {
    let r = ratio.clamp(0.0, 1.0);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        with_alpha(Color::from_rgba(8, 12, 18, 255), 0.62),
    );
    draw_rectangle(
        rect.x + 0.8,
        rect.y + 0.8,
        ((rect.w - 1.6).max(1.0)) * r,
        (rect.h - 1.6).max(1.0),
        with_alpha(fg, 0.85),
    );
    draw_rectangle_lines(
        rect.x + 0.6,
        rect.y + 0.6,
        (rect.w - 1.2).max(1.0),
        (rect.h - 1.2).max(1.0),
        1.0,
        with_alpha(Color::from_rgba(220, 232, 244, 255), 0.22),
    );
}

fn draw_belt_payload(
    rect: Rect,
    orientation: sim::BlockOrientation,
    time: f32,
    speed: f32,
    load: f32,
) {
    let s = speed.clamp(0.0, 1.0);
    let l = load.clamp(0.0, 1.0);
    if rect.w < 4.0 || rect.h < 4.0 || l <= 0.06 {
        return;
    }
    let axis = orientation_axis(orientation);
    let horizontal = axis.x.abs() > axis.y.abs();
    let dir = (axis.x + axis.y).signum().clamp(-1.0, 1.0);
    let count = (2.0 + l * 7.0).round() as i32;

    let c0 = Color::from_rgba(236, 224, 176, 255);
    let c1 = Color::from_rgba(206, 184, 120, 255);

    for i in 0..count {
        let fi = i as f32;
        let phase = fract01(time * (0.45 + s * 1.4) + fi * 0.23);
        let t = if dir >= 0.0 { phase } else { 1.0 - phase };
        let j = fract01(((fi * 12.9898 + 78.233).sin()) * 43_758.547);
        let j2 = fract01(((fi * 9.231 + 11.91).sin()) * 17713.11);

        let r = (rect.w.min(rect.h) * (0.05 + l * 0.03) * (0.65 + j2 * 0.65)).max(1.0);
        let col = color_lerp(c0, c1, j2);

        let p = if horizontal {
            vec2(
                rect.x + t * rect.w,
                rect.y + rect.h * (0.25 + 0.5 * j2) + (j - 0.5) * rect.h * 0.16,
            )
        } else {
            vec2(
                rect.x + rect.w * (0.25 + 0.5 * j2) + (j - 0.5) * rect.w * 0.16,
                rect.y + t * rect.h,
            )
        };

        draw_circle(p.x, p.y, r, with_alpha(col, 0.52 + l * 0.28));
        draw_circle(
            p.x - r * 0.35,
            p.y - r * 0.35,
            r * 0.35,
            with_alpha(WHITE, 0.08 + l * 0.10),
        );
    }
}

fn draw_modern_offline_overlay(rect: Rect, time: f32) {
    let _ = time;
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        with_alpha(Color::from_rgba(0, 0, 0, 255), 0.20),
    );
}

fn draw_input_hopper_visual(
    rect: Rect,
    orientation: sim::BlockOrientation,
    time: f32,
    active: bool,
    stock_ratio: f32,
) {
    let activity = if active { 1.0 } else { 0.0 };
    let stock = stock_ratio.clamp(0.0, 1.0);

    draw_soft_shadow_rect(rect, vec2(3.0, 3.0), 4.6, 0.22);

    let base = Color::from_rgba(118, 128, 140, 240);
    let outline = with_alpha(Color::from_rgba(220, 232, 244, 255), 0.72);
    draw_panel(rect, base, outline, 0.08);

    let r = rect.w.min(rect.h);
    let riv = (r * 0.03).max(1.0);
    draw_rivet(
        vec2(rect.x + 6.0, rect.y + 6.0),
        riv,
        with_alpha(Color::from_rgba(210, 220, 232, 255), 0.65),
    );
    draw_rivet(
        vec2(rect.x + rect.w - 6.0, rect.y + 6.0),
        riv,
        with_alpha(Color::from_rgba(210, 220, 232, 255), 0.65),
    );
    draw_rivet(
        vec2(rect.x + 6.0, rect.y + rect.h - 6.0),
        riv,
        with_alpha(Color::from_rgba(210, 220, 232, 255), 0.65),
    );
    draw_rivet(
        vec2(rect.x + rect.w - 6.0, rect.y + rect.h - 6.0),
        riv,
        with_alpha(Color::from_rgba(210, 220, 232, 255), 0.65),
    );

    let axis = orientation_axis(orientation);
    let horizontal = axis.x.abs() > axis.y.abs();

    let belt_margin = (r * 0.10).max(6.0);
    let belt_thick = if horizontal {
        rect.h * 0.22
    } else {
        rect.w * 0.22
    };
    let belt_rect = if horizontal {
        Rect::new(
            rect.x + belt_margin,
            rect.y + rect.h * 0.68,
            rect.w - belt_margin * 2.0,
            belt_thick,
        )
    } else {
        Rect::new(
            rect.x + rect.w * 0.68,
            rect.y + belt_margin,
            belt_thick,
            rect.h - belt_margin * 2.0,
        )
    };

    draw_belt_motion(
        belt_rect,
        orientation,
        time * activity,
        Color::from_rgba(38, 132, 238, 250),
        Color::from_rgba(132, 214, 255, 238),
        Color::from_rgba(206, 236, 255, 205),
    );
    draw_belt_payload(
        rect_inset(belt_rect, belt_rect.w.min(belt_rect.h) * 0.18),
        orientation,
        time,
        activity,
        (0.25 + stock * 0.75).clamp(0.0, 1.0),
    );

    let bowl_center = rect_center(rect) - axis * rect.w.max(rect.h) * 0.20;
    let rim_r = rect.w.min(rect.h) * 0.26;
    let bowl_r = rim_r * 0.78;

    draw_circle(
        bowl_center.x,
        bowl_center.y,
        rim_r,
        with_alpha(Color::from_rgba(52, 60, 70, 255), 0.75),
    );
    draw_circle(
        bowl_center.x,
        bowl_center.y,
        bowl_r,
        with_alpha(Color::from_rgba(142, 154, 168, 255), 0.62),
    );

    let fill_r = bowl_r * stock.sqrt();
    draw_circle(
        bowl_center.x,
        bowl_center.y,
        fill_r,
        with_alpha(Color::from_rgba(236, 214, 160, 255), 0.52),
    );

    if activity > 0.01 {
        for i in 0..10 {
            let a = time * 2.0 + i as f32 * 0.55;
            let rr = fill_r * (0.25 + (i as f32 * 0.11).sin().abs() * 0.45);
            draw_circle(
                bowl_center.x + a.cos() * rr * 0.35,
                bowl_center.y + a.sin() * rr * 0.35,
                (r * 0.03).max(1.0),
                with_alpha(Color::from_rgba(244, 232, 190, 255), 0.12 + stock * 0.10),
            );
        }
    }

    let gauge = Rect::new(
        rect.x + rect.w * 0.06,
        rect.y + rect.h * 0.16,
        rect.w * 0.10,
        rect.h * 0.46,
    );
    draw_rectangle(
        gauge.x,
        gauge.y,
        gauge.w,
        gauge.h,
        with_alpha(Color::from_rgba(8, 12, 18, 255), 0.55),
    );
    let fh = (gauge.h - 2.0).max(0.0) * stock;
    draw_rectangle(
        gauge.x + 1.0,
        gauge.y + (gauge.h - 1.0 - fh),
        (gauge.w - 2.0).max(0.0),
        fh,
        with_alpha(Color::from_rgba(246, 210, 116, 255), 0.68),
    );
    draw_rectangle_lines(
        gauge.x + 0.6,
        gauge.y + 0.6,
        (gauge.w - 1.2).max(1.0),
        (gauge.h - 1.2).max(1.0),
        1.0,
        with_alpha(Color::from_rgba(220, 232, 244, 255), 0.22),
    );

    let led_pos = vec2(rect.x + rect.w * 0.88, rect.y + rect.h * 0.18);
    let led_r = (r * 0.05).max(1.4);
    let led_col = if activity > 0.3 {
        Color::from_rgba(84, 248, 154, 255)
    } else {
        Color::from_rgba(160, 170, 180, 255)
    };
    draw_led(led_pos, led_r, led_col, 0.85);
}

fn draw_fluidity_tank_visual(rect: Rect, time: f32, fill_ratio: f32, active: bool) {
    let fill = fill_ratio.clamp(0.0, 1.0);
    let activity = if active { 1.0 } else { 0.0 };

    draw_soft_shadow_rect(rect, vec2(3.0, 3.0), 4.2, 0.20);

    let base = Color::from_rgba(86, 132, 132, 236);
    let outline = with_alpha(Color::from_rgba(214, 236, 244, 255), 0.70);
    draw_panel(rect, base, outline, 0.07);

    let inner = rect_inset(rect, rect.w.min(rect.h) * 0.12);
    draw_rectangle(
        inner.x,
        inner.y,
        inner.w,
        inner.h,
        with_alpha(Color::from_rgba(10, 18, 26, 255), 0.35),
    );

    let water_h = inner.h * fill;
    let wave = (time * (2.0 + activity * 2.6)).sin() * inner.w.min(inner.h) * 0.03;
    let water_y = inner.y + (inner.h - water_h).max(0.0);
    let water = Rect::new(inner.x, water_y, inner.w, water_h);

    draw_rectangle(
        water.x,
        water.y,
        water.w,
        water.h,
        with_alpha(Color::from_rgba(56, 176, 214, 255), 0.42 + 0.12 * activity),
    );
    draw_rectangle(
        water.x,
        water.y + wave,
        water.w,
        (inner.w.min(inner.h) * 0.06).max(1.0),
        with_alpha(Color::from_rgba(132, 226, 248, 255), 0.26 + 0.18 * activity),
    );

    if activity > 0.01 {
        for i in 0..10 {
            let fi = i as f32;
            let px = inner.x + inner.w * fract01(time * 0.33 + fi * 0.17);
            let py = water.y + water.h * fract01(time * 0.42 + fi * 0.31);
            let rr = (inner.w.min(inner.h) * 0.03).max(1.0);
            draw_circle(
                px,
                py,
                rr,
                with_alpha(Color::from_rgba(210, 250, 255, 255), 0.08 + 0.10 * activity),
            );
        }
    }

    let motor = vec2(rect.x + rect.w * 0.74, rect.y + rect.h * 0.20);
    let mr = rect.w.min(rect.h) * 0.10;
    draw_circle(
        motor.x,
        motor.y,
        mr,
        with_alpha(Color::from_rgba(40, 48, 58, 255), 0.70),
    );
    draw_circle(
        motor.x,
        motor.y,
        mr * 0.72,
        with_alpha(Color::from_rgba(170, 184, 198, 255), 0.55),
    );
    for k in 0..3 {
        let a = time * (2.6 + activity * 3.2) + k as f32 * (std::f32::consts::TAU / 3.0);
        draw_line(
            motor.x,
            motor.y,
            motor.x + a.cos() * mr * 0.8,
            motor.y + a.sin() * mr * 0.8,
            1.2,
            with_alpha(Color::from_rgba(220, 244, 255, 255), 0.22 + 0.25 * activity),
        );
    }

    let bar = Rect::new(
        rect.x + rect.w * 0.08,
        rect.y + rect.h * 0.78,
        rect.w * 0.22,
        rect.h * 0.10,
    );
    draw_progress_bar(bar, fill, Color::from_rgba(84, 228, 248, 255));
}

fn draw_cutter_visual(
    rect: Rect,
    orientation: sim::BlockOrientation,
    time: f32,
    active: bool,
    progress: f32,
) {
    let activity = if active { 1.0 } else { 0.0 };
    let prog = progress.clamp(0.0, 1.0);

    draw_soft_shadow_rect(rect, vec2(3.0, 3.0), 4.4, 0.22);

    let base = Color::from_rgba(122, 118, 112, 236);
    let outline = with_alpha(Color::from_rgba(226, 232, 238, 255), 0.70);
    draw_panel(rect, base, outline, 0.08);

    let belt_rect = rect_inset_xy(rect, rect.w * 0.10, rect.h * 0.40);
    draw_belt_motion(
        belt_rect,
        orientation,
        time * activity,
        Color::from_rgba(26, 150, 110, 242),
        Color::from_rgba(120, 238, 194, 228),
        Color::from_rgba(210, 250, 232, 200),
    );
    draw_belt_payload(
        rect_inset(belt_rect, belt_rect.w.min(belt_rect.h) * 0.18),
        orientation,
        time,
        activity,
        0.65,
    );

    let hood = rect_inset_xy(rect, rect.w * 0.10, rect.h * 0.10);
    draw_rectangle(
        hood.x,
        hood.y,
        hood.w,
        hood.h * 0.34,
        with_alpha(Color::from_rgba(14, 18, 24, 255), 0.22),
    );
    draw_rectangle_lines(
        hood.x + 0.7,
        hood.y + 0.7,
        (hood.w - 1.4).max(1.0),
        (hood.h * 0.34 - 1.4).max(1.0),
        1.0,
        with_alpha(Color::from_rgba(200, 220, 236, 255), 0.20),
    );

    let blade_r = rect.w.min(rect.h) * 0.18;
    for i in 0..3 {
        let x = rect.x + rect.w * (0.26 + i as f32 * 0.24);
        let y = rect.y + rect.h * 0.52;
        draw_circle(
            x,
            y,
            blade_r * 1.05,
            with_alpha(Color::from_rgba(32, 38, 44, 255), 0.55),
        );
        draw_circle(
            x,
            y,
            blade_r,
            with_alpha(Color::from_rgba(176, 186, 196, 255), 0.70),
        );
        draw_circle(
            x,
            y,
            blade_r * 0.55,
            with_alpha(Color::from_rgba(80, 88, 98, 255), 0.55),
        );
        for k in 0..4 {
            let a = time * (3.6 + activity * 6.0) + (k as f32 * std::f32::consts::TAU / 4.0);
            draw_line(
                x,
                y,
                x + a.cos() * blade_r * 0.92,
                y + a.sin() * blade_r * 0.92,
                1.0,
                with_alpha(Color::from_rgba(240, 248, 255, 255), 0.22 + 0.25 * activity),
            );
        }
    }

    let bar = Rect::new(
        rect.x + rect.w * 0.62,
        rect.y + rect.h * 0.14,
        rect.w * 0.30,
        rect.h * 0.10,
    );
    draw_progress_bar(bar, prog, Color::from_rgba(84, 248, 154, 255));

    let led_pos = vec2(rect.x + rect.w * 0.88, rect.y + rect.h * 0.86);
    draw_led(
        led_pos,
        rect.w.min(rect.h) * 0.05,
        Color::from_rgba(84, 248, 154, 255),
        0.4 + 0.6 * activity,
    );
}

fn draw_distributor_visual(
    rect: Rect,
    orientation: sim::BlockOrientation,
    time: f32,
    active: bool,
) {
    let activity = if active { 1.0 } else { 0.0 };
    draw_soft_shadow_rect(rect, vec2(2.5, 2.5), 3.6, 0.18);

    draw_belt_motion(
        rect,
        orientation,
        time * activity,
        Color::from_rgba(22, 92, 220, 252),
        Color::from_rgba(110, 198, 255, 238),
        Color::from_rgba(176, 220, 250, 198),
    );

    let body = rect_inset_xy(rect, rect.w * 0.12, rect.h * 0.18);
    draw_rectangle(
        body.x,
        body.y,
        body.w,
        body.h,
        with_alpha(Color::from_rgba(22, 28, 36, 255), 0.10),
    );
    draw_grain_overlay(body, 0.06);

    let axis = orientation_axis(orientation);
    let normal = vec2(-axis.y, axis.x);
    let pivot = rect_center(rect);
    let foot = rect.w.min(rect.h) * 0.12;

    draw_circle(
        pivot.x,
        pivot.y,
        foot * 0.95,
        with_alpha(Color::from_rgba(18, 22, 28, 255), 0.55),
    );
    draw_circle(
        pivot.x,
        pivot.y,
        foot * 0.72,
        with_alpha(Color::from_rgba(210, 220, 232, 255), 0.55),
    );

    let base_angle = axis.y.atan2(axis.x);
    let swing = (time * 1.3).sin() * 35.0_f32.to_radians() * activity;
    let arm_angle = base_angle + swing;
    let arm_len = rect.w.max(rect.h) * 0.46;
    let tip = pivot + vec2(arm_angle.cos(), arm_angle.sin()) * arm_len;

    draw_line(
        pivot.x,
        pivot.y,
        tip.x,
        tip.y,
        2.6,
        with_alpha(Color::from_rgba(210, 220, 232, 255), 0.75),
    );
    draw_circle(
        tip.x,
        tip.y,
        3.6,
        with_alpha(Color::from_rgba(248, 252, 255, 255), 0.58),
    );

    let servo = pivot - normal * rect.w.min(rect.h) * 0.20;
    draw_rectangle(
        servo.x - 6.0,
        servo.y - 4.0,
        12.0,
        8.0,
        with_alpha(Color::from_rgba(24, 28, 34, 255), 0.55),
    );
    draw_rectangle_lines(
        servo.x - 6.0,
        servo.y - 4.0,
        12.0,
        8.0,
        1.0,
        with_alpha(Color::from_rgba(220, 232, 244, 255), 0.18),
    );
    draw_led(
        servo + normal * 7.0,
        1.6,
        Color::from_rgba(84, 248, 154, 255),
        0.3 + 0.7 * activity,
    );
}

fn draw_dryer_oven_visual(rect: Rect, orientation: sim::BlockOrientation, time: f32) {
    crate::four_texture::draw_dryer_oven_visual(rect, orientation, time);
}

fn draw_flaker_visual(
    rect: Rect,
    orientation: sim::BlockOrientation,
    time: f32,
    active: bool,
    progress: f32,
) {
    let activity = if active { 1.0 } else { 0.0 };
    let prog = progress.clamp(0.0, 1.0);

    draw_soft_shadow_rect(rect, vec2(3.0, 3.0), 4.0, 0.20);

    let base = Color::from_rgba(124, 122, 116, 236);
    let outline = with_alpha(Color::from_rgba(214, 220, 226, 255), 0.70);
    draw_panel(rect, base, outline, 0.08);

    let center = rect_center(rect);
    let drum_r = rect.w.min(rect.h) * 0.26;

    draw_circle(
        center.x,
        center.y,
        drum_r * 1.08,
        with_alpha(Color::from_rgba(26, 30, 36, 255), 0.52),
    );
    draw_circle(
        center.x,
        center.y,
        drum_r,
        with_alpha(Color::from_rgba(184, 190, 198, 255), 0.75),
    );
    draw_circle(
        center.x,
        center.y,
        drum_r * 0.62,
        with_alpha(Color::from_rgba(92, 98, 108, 255), 0.55),
    );

    for i in 0..6 {
        let a = time * (2.6 + activity * 6.5) + i as f32 * (std::f32::consts::TAU / 6.0);
        draw_line(
            center.x,
            center.y,
            center.x + a.cos() * drum_r * 0.92,
            center.y + a.sin() * drum_r * 0.92,
            1.2,
            with_alpha(Color::from_rgba(240, 248, 255, 255), 0.12 + 0.25 * activity),
        );
    }

    let axis = orientation_axis(orientation);
    let chute_center = center + axis * rect.w.min(rect.h) * 0.34;
    let chute = Rect::new(chute_center.x - 6.0, chute_center.y - 9.0, 12.0, 18.0);
    draw_rectangle(
        chute.x,
        chute.y,
        chute.w,
        chute.h,
        with_alpha(Color::from_rgba(36, 42, 50, 255), 0.62),
    );
    draw_rectangle_lines(
        chute.x + 0.6,
        chute.y + 0.6,
        (chute.w - 1.2).max(1.0),
        (chute.h - 1.2).max(1.0),
        1.0,
        with_alpha(Color::from_rgba(220, 232, 244, 255), 0.18),
    );

    if activity > 0.01 {
        for i in 0..12 {
            let fi = i as f32;
            let t = fract01(time * 0.8 + fi * 0.11);
            let p = chute_center
                + axis * (t * rect.w.min(rect.h) * 0.24)
                + vec2(((fi * 7.1).sin()) * 2.4, ((fi * 5.3).cos()) * 2.4);
            draw_circle(
                p.x,
                p.y,
                1.4,
                with_alpha(Color::from_rgba(236, 224, 176, 255), 0.14 + 0.20 * activity),
            );
        }
    }

    let bar = Rect::new(
        rect.x + rect.w * 0.10,
        rect.y + rect.h * 0.12,
        rect.w * 0.32,
        rect.h * 0.10,
    );
    draw_progress_bar(bar, prog, Color::from_rgba(84, 248, 154, 255));

    draw_led(
        vec2(rect.x + rect.w * 0.84, rect.y + rect.h * 0.84),
        rect.w.min(rect.h) * 0.05,
        Color::from_rgba(84, 248, 154, 255),
        0.35 + 0.65 * activity,
    );
}

#[derive(Clone, Copy, Default)]
struct PipeConnections {
    north: bool,
    south: bool,
    east: bool,
    west: bool,
}

fn suction_pipe_connectable(kind: sim::BlockKind) -> bool {
    matches!(
        kind,
        sim::BlockKind::SuctionPipe
            | sim::BlockKind::Flaker
            | sim::BlockKind::Sortex
            | sim::BlockKind::BlueBagChute
            | sim::BlockKind::RedBagChute
    )
}

fn suction_pipe_connections(
    block: &sim::BlockDebugView,
    blocks: &[sim::BlockDebugView],
) -> PipeConnections {
    let origin = block.tile;
    let has_block_at = |tile: (i32, i32)| {
        blocks.iter().any(|other| {
            other.id != block.id
                && suction_pipe_connectable(other.kind)
                && block_occupies_tile(other, tile)
        })
    };

    PipeConnections {
        north: has_block_at((origin.0, origin.1 - 1)),
        south: has_block_at((origin.0, origin.1 + 1)),
        east: has_block_at((origin.0 + 1, origin.1)),
        west: has_block_at((origin.0 - 1, origin.1)),
    }
}

fn draw_suction_pipe_visual(rect: Rect, conn: PipeConnections, time: f32, flow: f32) {
    let f = flow.clamp(0.0, 1.0);

    draw_soft_shadow_rect(rect, vec2(2.0, 2.0), 2.6, 0.14);

    let base = Color::from_rgba(82, 90, 102, 240);
    let hi = with_alpha(Color::from_rgba(210, 224, 236, 255), 0.46);
    let lo = with_alpha(Color::from_rgba(10, 12, 14, 255), 0.36);

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, with_alpha(base, 0.55));
    draw_grain_overlay(rect, 0.04);

    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    let thick = rect.w.min(rect.h) * 0.32;
    let half = rect.w.min(rect.h) * 0.5;

    if conn.north {
        draw_rectangle(cx - thick * 0.5, rect.y, thick, half, base);
        draw_line(
            cx - thick * 0.35,
            rect.y + 1.0,
            cx - thick * 0.35,
            rect.y + half - 1.0,
            1.0,
            hi,
        );
        draw_line(
            cx + thick * 0.35,
            rect.y + 1.0,
            cx + thick * 0.35,
            rect.y + half - 1.0,
            1.0,
            lo,
        );
    }
    if conn.south {
        draw_rectangle(cx - thick * 0.5, cy, thick, half, base);
        draw_line(
            cx - thick * 0.35,
            cy + 1.0,
            cx - thick * 0.35,
            rect.y + rect.h - 1.0,
            1.0,
            hi,
        );
        draw_line(
            cx + thick * 0.35,
            cy + 1.0,
            cx + thick * 0.35,
            rect.y + rect.h - 1.0,
            1.0,
            lo,
        );
    }
    if conn.west {
        draw_rectangle(rect.x, cy - thick * 0.5, half, thick, base);
        draw_line(
            rect.x + 1.0,
            cy - thick * 0.35,
            rect.x + half - 1.0,
            cy - thick * 0.35,
            1.0,
            hi,
        );
        draw_line(
            rect.x + 1.0,
            cy + thick * 0.35,
            rect.x + half - 1.0,
            cy + thick * 0.35,
            1.0,
            lo,
        );
    }
    if conn.east {
        draw_rectangle(cx, cy - thick * 0.5, half, thick, base);
        draw_line(
            cx + 1.0,
            cy - thick * 0.35,
            rect.x + rect.w - 1.0,
            cy - thick * 0.35,
            1.0,
            hi,
        );
        draw_line(
            cx + 1.0,
            cy + thick * 0.35,
            rect.x + rect.w - 1.0,
            cy + thick * 0.35,
            1.0,
            lo,
        );
    }

    draw_circle(
        cx,
        cy,
        thick * 0.64,
        with_alpha(Color::from_rgba(36, 42, 50, 255), 0.65),
    );
    draw_circle(
        cx,
        cy,
        thick * 0.48,
        with_alpha(Color::from_rgba(176, 190, 204, 255), 0.42),
    );

    let pulse = fract01(time * (0.8 + 1.8 * f));
    let dot_r = (rect.w.min(rect.h) * 0.08).max(1.0);
    let dot_col = with_alpha(Color::from_rgba(120, 210, 255, 255), 0.10 + 0.35 * f);
    if conn.north {
        draw_circle(cx, cy - pulse * half * 0.90, dot_r, dot_col);
    }
    if conn.south {
        draw_circle(cx, cy + pulse * half * 0.90, dot_r, dot_col);
    }
    if conn.west {
        draw_circle(cx - pulse * half * 0.90, cy, dot_r, dot_col);
    }
    if conn.east {
        draw_circle(cx + pulse * half * 0.90, cy, dot_r, dot_col);
    }
}

fn draw_sortex_visual(
    rect: Rect,
    orientation: sim::BlockOrientation,
    time: f32,
    active: bool,
    progress: f32,
) {
    let activity = if active { 1.0 } else { 0.0 };
    let prog = progress.clamp(0.0, 1.0);

    draw_soft_shadow_rect(rect, vec2(3.0, 3.0), 4.6, 0.22);

    let base = Color::from_rgba(118, 122, 128, 238);
    let outline = with_alpha(Color::from_rgba(220, 232, 244, 255), 0.72);
    draw_panel(rect, base, outline, 0.08);

    let b = basis_for_block(rect, orientation);

    draw_oriented_quad(
        &b,
        -0.70,
        -0.38,
        0.30,
        0.38,
        with_alpha(Color::from_rgba(14, 18, 24, 255), 0.28),
    );
    draw_oriented_quad_lines(
        &b,
        -0.70,
        -0.38,
        0.30,
        0.38,
        1.2,
        with_alpha(Color::from_rgba(220, 232, 244, 255), 0.18),
    );

    let scan = (time * (1.4 + 2.0 * activity)).sin() * 0.32;
    let beam_col = with_alpha(Color::from_rgba(120, 220, 255, 255), 0.10 + 0.28 * activity);
    let p0 = b.p(-0.62, scan);
    let p1 = b.p(0.22, scan);
    draw_line(p0.x, p0.y, p1.x, p1.y, 2.0, beam_col);

    let in_port = Rect::new(
        rect.x + rect.w * 0.08,
        rect.y + rect.h * 0.42,
        rect.w * 0.10,
        rect.h * 0.16,
    );
    draw_rectangle(
        in_port.x,
        in_port.y,
        in_port.w,
        in_port.h,
        with_alpha(Color::from_rgba(32, 38, 46, 255), 0.55),
    );
    draw_rectangle_lines(
        in_port.x + 0.6,
        in_port.y + 0.6,
        (in_port.w - 1.2).max(1.0),
        (in_port.h - 1.2).max(1.0),
        1.0,
        with_alpha(Color::from_rgba(220, 232, 244, 255), 0.18),
    );

    draw_oriented_quad(
        &b,
        0.42,
        -0.52,
        0.86,
        -0.10,
        with_alpha(Color::from_rgba(36, 82, 220, 255), 0.52),
    );
    draw_oriented_quad(
        &b,
        0.42,
        0.10,
        0.86,
        0.52,
        with_alpha(Color::from_rgba(210, 86, 72, 255), 0.52),
    );
    draw_oriented_quad_lines(
        &b,
        0.42,
        -0.52,
        0.86,
        -0.10,
        1.1,
        with_alpha(Color::from_rgba(232, 242, 250, 255), 0.18),
    );
    draw_oriented_quad_lines(
        &b,
        0.42,
        0.10,
        0.86,
        0.52,
        1.1,
        with_alpha(Color::from_rgba(250, 232, 232, 255), 0.18),
    );

    let led_a = 0.45 + 0.55 * activity;
    draw_led(
        b.p(-0.15, -0.55),
        rect.w.min(rect.h) * 0.05,
        Color::from_rgba(84, 248, 154, 255),
        led_a,
    );
    draw_led(
        b.p(-0.05, 0.55),
        rect.w.min(rect.h) * 0.05,
        Color::from_rgba(84, 248, 154, 255),
        led_a,
    );

    let bar = Rect::new(
        rect.x + rect.w * 0.12,
        rect.y + rect.h * 0.82,
        rect.w * 0.32,
        rect.h * 0.10,
    );
    draw_progress_bar(bar, prog, Color::from_rgba(84, 248, 154, 255));
}

fn draw_bag_chute_visual(
    rect: Rect,
    orientation: sim::BlockOrientation,
    is_blue: bool,
    fill_ratio: f32,
    beacon_active: bool,
    time: f32,
) {
    let fill = fill_ratio.clamp(0.0, 1.0);
    let blink = if beacon_active {
        blink_ratio(time)
    } else {
        0.0
    };

    draw_soft_shadow_rect(rect, vec2(3.0, 3.0), 4.0, 0.20);

    let base = if is_blue {
        Color::from_rgba(90, 118, 162, 236)
    } else {
        Color::from_rgba(168, 96, 92, 236)
    };
    let outline = with_alpha(Color::from_rgba(224, 236, 248, 255), 0.70);
    draw_panel(rect, base, outline, 0.07);

    let b = basis_for_block(rect, orientation);

    draw_oriented_quad(
        &b,
        -0.86,
        -0.20,
        -0.22,
        0.20,
        with_alpha(Color::from_rgba(34, 40, 48, 255), 0.62),
    );
    draw_oriented_quad_lines(
        &b,
        -0.86,
        -0.20,
        -0.22,
        0.20,
        1.1,
        with_alpha(Color::from_rgba(220, 232, 244, 255), 0.18),
    );

    let bag_u0 = 0.02;
    let bag_u1 = 0.92;
    let bag_v0 = -0.50;
    let bag_v1 = 0.50;

    draw_oriented_quad(
        &b,
        bag_u0,
        bag_v0,
        bag_u1,
        bag_v1,
        with_alpha(Color::from_rgba(232, 242, 248, 255), 0.26),
    );
    draw_oriented_quad_lines(
        &b,
        bag_u0,
        bag_v0,
        bag_u1,
        bag_v1,
        1.2,
        with_alpha(Color::from_rgba(240, 248, 255, 255), 0.22),
    );

    let fill_start = bag_u0 + (1.0 - fill) * (bag_u1 - bag_u0);
    let fill_col = if is_blue {
        with_alpha(Color::from_rgba(54, 96, 214, 255), 0.46)
    } else {
        with_alpha(Color::from_rgba(214, 88, 70, 255), 0.46)
    };
    draw_oriented_quad(
        &b,
        fill_start,
        bag_v0 + 0.04,
        bag_u1 - 0.02,
        bag_v1 - 0.04,
        fill_col,
    );

    for i in 0..3 {
        let u = bag_u0 + 0.18 + i as f32 * 0.22;
        let p0 = b.p(u, bag_v0 + 0.06);
        let p1 = b.p(u + 0.04, bag_v1 - 0.06);
        draw_line(
            p0.x,
            p0.y,
            p1.x,
            p1.y,
            1.0,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.06),
        );
    }

    let clamp_pos = b.p(bag_u0 - 0.06, 0.0);
    draw_circle(
        clamp_pos.x,
        clamp_pos.y,
        rect.w.min(rect.h) * 0.06,
        with_alpha(Color::from_rgba(34, 40, 48, 255), 0.62),
    );
    draw_circle(
        clamp_pos.x,
        clamp_pos.y,
        rect.w.min(rect.h) * 0.04,
        with_alpha(Color::from_rgba(210, 220, 232, 255), 0.35),
    );

    let beacon_pos = b.p(-0.78, -0.72);
    let beacon_col = if is_blue {
        Color::from_rgba(80, 160, 255, 255)
    } else {
        Color::from_rgba(255, 120, 90, 255)
    };
    draw_led(beacon_pos, rect.w.min(rect.h) * 0.06, beacon_col, blink);

    let bar = Rect::new(
        rect.x + rect.w * 0.10,
        rect.y + rect.h * 0.82,
        rect.w * 0.34,
        rect.h * 0.10,
    );
    draw_progress_bar(
        bar,
        fill,
        if is_blue {
            Color::from_rgba(120, 190, 255, 255)
        } else {
            Color::from_rgba(255, 170, 140, 255)
        },
    );
}

fn draw_buffer_rack_visual(rect: Rect, rack_levels: &[bool]) {
    let frame = Color::from_rgba(196, 172, 146, 188);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(76, 64, 54, 218),
    );
    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.w - 1.0,
        rect.h - 1.0,
        1.4,
        frame,
    );
    draw_rectangle(
        rect.x + 1.6,
        rect.y + 1.6,
        (rect.w - 3.2).max(1.0),
        (rect.h - 3.2).max(1.0),
        with_alpha(Color::from_rgba(98, 84, 68, 80), 0.9),
    );
    let levels = rack_levels.len().max(1);
    for i in 0..levels {
        let t = i as f32 / levels as f32;
        let y = rect.y + rect.h - 4.0 - t * (rect.h - 8.0);
        draw_line(
            rect.x + 2.0,
            y,
            rect.x + rect.w - 2.0,
            y,
            1.1,
            Color::from_rgba(186, 154, 118, 182),
        );
        if rack_levels.get(i).copied().unwrap_or(false) {
            draw_rectangle(
                rect.x + 4.0,
                y - 3.4,
                (rect.w - 8.0).max(1.0),
                6.4,
                Color::from_rgba(146, 104, 72, 212),
            );
            draw_rectangle_lines(
                rect.x + 4.5,
                y - 2.9,
                (rect.w - 9.0).max(1.0),
                5.4,
                0.7,
                with_alpha(Color::from_rgba(234, 216, 196, 150), 0.7),
            );
        } else {
            draw_circle(
                rect.x + rect.w * 0.35,
                y - 0.6,
                0.6,
                with_alpha(Color::from_rgba(172, 152, 132, 140), 0.65),
            );
            draw_circle(
                rect.x + rect.w * 0.65,
                y - 0.6,
                0.6,
                with_alpha(Color::from_rgba(172, 152, 132, 140), 0.65),
            );
        }
    }
}

fn draw_seller_visual(rect: Rect, time: f32) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(66, 96, 76, 222),
    );
    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.w - 1.0,
        rect.h - 1.0,
        1.2,
        Color::from_rgba(184, 230, 198, 184),
    );
    draw_rectangle(
        rect.x + rect.w * 0.16,
        rect.y + rect.h * 0.54,
        rect.w * 0.68,
        rect.h * 0.28,
        Color::from_rgba(112, 84, 62, 220),
    );
    let pulse = 0.28 + ((time * 2.1).sin() * 0.5 + 0.5) * 0.52;
    draw_rectangle(
        rect.x + rect.w * 0.34,
        rect.y + rect.h * 0.2,
        rect.w * 0.32,
        rect.h * 0.25,
        with_alpha(Color::from_rgba(112, 196, 148, 255), pulse),
    );
    draw_rectangle(
        rect.x + rect.w * 0.18,
        rect.y + rect.h * 0.2,
        rect.w * 0.15,
        rect.h * 0.13,
        with_alpha(Color::from_rgba(198, 238, 208, 178), 0.4 + pulse * 0.2),
    );
    draw_circle(
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.72,
        rect.w.min(rect.h) * 0.1,
        with_alpha(
            Color::from_rgba(255, 180, 80, 200),
            0.22 + blink_ratio(time),
        ),
    );
}

fn draw_machine_cluster_visual(
    rect: Rect,
    base: Color,
    panel: Color,
    frame: Color,
    activity: f32,
    rotor_speed: f32,
    time: f32,
) {
    let shell = with_alpha(Color::from_rgba(188, 204, 216, 248), 0.94);
    let shell_dark = with_alpha(Color::from_rgba(130, 146, 162, 248), 0.95);
    let chrome = with_alpha(Color::from_rgba(230, 242, 250, 170), 0.7);
    let steel_glow = with_alpha(Color::from_rgba(168, 216, 255, 170), 0.35 + activity * 0.35);

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, shell_dark);
    draw_rectangle(
        rect.x + 0.9,
        rect.y + 0.9,
        (rect.w - 1.8).max(1.0),
        (rect.h - 1.8).max(1.0),
        shell,
    );
    draw_rectangle(
        rect.x + 0.9,
        rect.y + rect.h * 0.18,
        rect.w - 1.8,
        rect.h * 0.14,
        with_alpha(chrome, 0.42),
    );
    draw_rectangle(
        rect.x + 0.9,
        rect.y + rect.h * 0.68,
        rect.w - 1.8,
        rect.h * 0.14,
        with_alpha(chrome, 0.42),
    );
    for i in 0..7 {
        let y = rect.y + rect.h * ((i as f32 + 0.5) / 8.0);
        draw_line(
            rect.x + 1.4,
            y,
            rect.x + rect.w - 1.4,
            y + ((time * 0.5).sin() * 0.3),
            0.5,
            with_alpha(with_alpha(base, 0.2), (i as f32 / 10.0).min(0.55)),
        );
    }
    draw_rectangle_lines(
        rect.x + 0.6,
        rect.y + 0.6,
        (rect.w - 1.2).max(1.0),
        (rect.h - 1.2).max(1.0),
        1.3,
        frame,
    );
    draw_circle(
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.8,
        rect.w.min(rect.h) * 0.06,
        frame,
    );
    draw_rectangle(
        rect.x + rect.w * 0.12,
        rect.y + rect.h * 0.12,
        rect.w * 0.76,
        rect.h * 0.76,
        with_alpha(panel, 0.68),
    );
    for i in 0..5 {
        let t = (i as f32 + 0.7) * 0.11;
        draw_line(
            rect.x + rect.w * (0.12 + t),
            rect.y + rect.h * 0.18,
            rect.x + rect.w * (0.12 + t),
            rect.y + rect.h * 0.86,
            0.4,
            with_alpha(panel, 0.28 + activity * 0.1),
        );
    }
    for i in 0..6 {
        let bx = rect.x + rect.w * (0.22 + i as f32 * 0.11);
        let by = rect.y + rect.h * 0.14;
        draw_circle(bx, by, 0.9, chrome);
        draw_circle(bx, rect.y + rect.h * 0.88, 0.9, chrome);
    }

    let core = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.32);
    let core_radius = rect.w.min(rect.h) * 0.11;
    draw_circle(core.x, core.y, core_radius, with_alpha(steel_glow, 0.38));
    draw_circle(
        core.x,
        core.y,
        core_radius * 0.7,
        with_alpha(frame, 0.18 + activity * 0.55),
    );
    for i in 0..3 {
        let a = time * rotor_speed + i as f32 * (std::f32::consts::TAU / 3.0);
        draw_line(
            core.x,
            core.y,
            core.x + a.cos() * core_radius * 0.9,
            core.y + a.sin() * core_radius * 0.9,
            1.0,
            with_alpha(Color::from_rgba(170, 220, 255, 220), 0.45 + activity * 0.4),
        );
    }

    let bar_x = rect.x + rect.w * 0.76;
    let bar_y = rect.y + rect.h * 0.78;
    let bar_w = rect.w * 0.18;
    let bar_h = rect.h * 0.1;
    let fill = (activity * 0.9).clamp(0.0, 1.0);
    draw_rectangle(
        bar_x,
        bar_y,
        bar_w,
        bar_h,
        with_alpha(Color::from_rgba(8, 16, 24, 190), 0.7),
    );
    draw_rectangle(
        bar_x + 0.8,
        bar_y + 0.9,
        (bar_w - 1.6).max(1.0) * fill,
        bar_h - 1.8,
        with_alpha(steel_glow, 0.9),
    );
    draw_circle(
        rect.x + rect.w * 0.84,
        rect.y + rect.h * 0.88,
        rect.w.min(rect.h) * 0.045,
        if activity > 0.3 {
            with_alpha(Color::from_rgba(64, 248, 152, 255), 0.75)
        } else {
            with_alpha(Color::from_rgba(120, 140, 170, 180), 0.45)
        },
    );
}

fn blink_ratio(time: f32) -> f32 {
    0.18 + ((time * 7.5).sin() * 0.5 + 0.5) * 0.62
}

fn draw_modern_block_visual(
    block: &sim::BlockDebugView,
    rect: Rect,
    sim: &sim::FactorySim,
    blocks: &[sim::BlockDebugView],
    time: f32,
    modern_ready: bool,
) {
    let lavage = modern_ready && sim.modern_lavage_busy();
    let lavage_progress = if modern_ready {
        sim.modern_lavage_progress_ratio()
    } else {
        0.0
    };
    let coupe = modern_ready && sim.modern_coupe_busy();
    let four = modern_ready && sim.modern_four_busy();
    let floc = modern_ready && sim.modern_floc_busy();
    let sortex = modern_ready && sim.modern_sortex_busy();
    let line_active = lavage || coupe || four || floc || sortex;

    match block.kind {
        sim::BlockKind::InputHopper => {
            let stock_ratio = (sim.line.raw as f32 / 25.0).clamp(0.0, 1.0);
            draw_input_hopper_visual(
                rect,
                block.orientation,
                time,
                lavage || lavage_progress > 0.0,
                stock_ratio,
            );
        }
        sim::BlockKind::Conveyor => {
            draw_belt_motion(
                rect,
                block.orientation,
                time * if line_active { 1.0 } else { 0.0 },
                Color::from_rgba(26, 108, 220, 252),
                Color::from_rgba(110, 198, 255, 238),
                Color::from_rgba(176, 220, 250, 198),
            );
            draw_belt_payload(
                rect_inset(rect, rect.w.min(rect.h) * 0.18),
                block.orientation,
                time,
                if line_active { 1.0 } else { 0.0 },
                (sim.line.wip as f32 / 24.0).clamp(0.0, 1.0),
            );
        }
        sim::BlockKind::FluidityTank => {
            let fill_ratio = (sim.line.washed as f32 / 10.0).clamp(0.0, 1.0);
            draw_fluidity_tank_visual(rect, time, fill_ratio, lavage || coupe);
        }
        sim::BlockKind::Cutter => {
            draw_cutter_visual(
                rect,
                block.orientation,
                time,
                coupe,
                sim.modern_coupe_progress_ratio(),
            );
        }
        sim::BlockKind::DistributorBelt => {
            draw_distributor_visual(rect, block.orientation, time, coupe || four);
        }
        sim::BlockKind::DryerOven => {
            draw_dryer_oven_visual(rect, block.orientation, time);
            if four {
                let heat = 0.25 + 0.75 * sim.modern_four_progress_ratio();
                draw_rectangle(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    with_alpha(Color::from_rgba(255, 148, 64, 255), 0.06 * heat),
                );
            }
        }
        sim::BlockKind::OvenExitConveyor => {
            draw_belt_motion(
                rect,
                block.orientation,
                time * if four || floc || sortex { 1.0 } else { 0.0 },
                Color::from_rgba(38, 120, 220, 232),
                Color::from_rgba(122, 201, 255, 224),
                Color::from_rgba(206, 232, 250, 192),
            );
            draw_belt_payload(
                rect_inset(rect, rect.w.min(rect.h) * 0.18),
                block.orientation,
                time,
                if four || floc || sortex { 1.0 } else { 0.0 },
                (sim.line.dehydrated as f32 / 10.0).clamp(0.0, 1.0),
            );
        }
        sim::BlockKind::Flaker => {
            draw_flaker_visual(
                rect,
                block.orientation,
                time,
                floc,
                sim.modern_floc_progress_ratio(),
            );
        }
        sim::BlockKind::SuctionPipe => {
            let conn = suction_pipe_connections(block, blocks);
            let flow = if modern_ready {
                (floc as i32 + sortex as i32) as f32 * 0.5
            } else {
                0.0
            };
            draw_suction_pipe_visual(rect, conn, time, flow);
        }
        sim::BlockKind::Sortex => {
            draw_sortex_visual(
                rect,
                block.orientation,
                time,
                sortex,
                sim.modern_sortex_progress_ratio(),
            );
        }
        sim::BlockKind::BlueBagChute => draw_bag_chute_visual(
            rect,
            block.orientation,
            true,
            sim.descente_bleue_fill_ratio(),
            sim.descente_bleue_beacon_active(),
            time,
        ),
        sim::BlockKind::RedBagChute => draw_bag_chute_visual(
            rect,
            block.orientation,
            false,
            sim.descente_rouge_fill_ratio(),
            sim.descente_rouge_beacon_active(),
            time,
        ),
        sim::BlockKind::Storage => {
            draw_rectangle(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                Color::from_rgba(60, 92, 128, 224),
            );
            draw_rectangle_lines(
                rect.x + 0.5,
                rect.y + 0.5,
                rect.w - 1.0,
                rect.h - 1.0,
                1.2,
                Color::from_rgba(184, 214, 236, 190),
            );
        }
        sim::BlockKind::MachineA => {
            let activity = (time * 1.2 + block.id as f32 * 0.11).sin() * 0.5 + 0.5;
            draw_machine_cluster_visual(
                rect,
                Color::from_rgba(213, 227, 238, 228),
                Color::from_rgba(238, 248, 253, 205),
                Color::from_rgba(158, 182, 204, 214),
                activity,
                2.4,
                time,
            );
        }
        sim::BlockKind::MachineB => {
            let activity = (time * 1.6 + block.id as f32 * 0.17).sin() * 0.5 + 0.5;
            draw_machine_cluster_visual(
                rect,
                Color::from_rgba(198, 214, 228, 228),
                Color::from_rgba(232, 242, 248, 198),
                Color::from_rgba(148, 172, 198, 214),
                activity,
                2.9,
                time,
            );
        }
        sim::BlockKind::Buffer => draw_buffer_rack_visual(rect, &block.rack_levels),
        sim::BlockKind::Seller => draw_seller_visual(rect, time),
    }

    if block.kind.is_modern_line_component() && !modern_ready {
        draw_modern_offline_overlay(rect, time);
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
    let time = get_time() as f32;
    let modern_ready = sim.modern_line_ready();
    let blocks = if show_labels {
        sim.block_debug_views()
    } else {
        sim.block_debug_views_minimal()
    };
    for block in &blocks {
        if let Some(tile_bounds) = bounds
            && !block_intersects_bounds(block.tile, block.footprint, tile_bounds)
        {
            continue;
        }
        let rect = sim_block_rect(block.tile, block.footprint);
        let color = sim_block_overlay_color(block.kind);
        draw_modern_block_visual(block, rect, sim, &blocks, time, modern_ready);
        draw_rectangle_lines(
            rect.x + 1.5,
            rect.y + 1.5,
            (rect.w - 3.0).max(1.0),
            (rect.h - 3.0).max(1.0),
            1.7,
            with_alpha(color, 0.74),
        );
        if block.kind == sim::BlockKind::Storage && block.raw_qty > 0 {
            draw_storage_raw_stack(rect, block.raw_qty, storage_texture.as_ref());
        }
        if show_labels {
            let kind_label = if block.kind.is_player_buyable() {
                block.kind.buyable_label()
            } else {
                block.kind.label()
            };
            let label = format!("#{} {}", block.id, kind_label);
            draw_text_chip(
                &label,
                rect.x + 3.0,
                rect.y - 3.0,
                13.0,
                Color::from_rgba(232, 240, 248, 255),
                Color::from_rgba(10, 18, 26, 214),
                Color::from_rgba(116, 168, 204, 188),
            );
            draw_text_chip(
                &block.inventory_summary,
                rect.x + 3.0,
                rect.y + rect.h + 13.0,
                11.0,
                Color::from_rgba(180, 215, 232, 255),
                Color::from_rgba(8, 15, 22, 202),
                Color::from_rgba(92, 138, 170, 168),
            );
        }
    }
}

pub(crate) fn draw_build_block_preview(
    sim: &sim::FactorySim,
    world: &World,
    mouse_tile: Option<(i32, i32)>,
) {
    let Some(tile) = mouse_tile else {
        return;
    };
    let Some(preview) = sim.build_block_preview(world, tile) else {
        return;
    };

    let rect = sim_block_rect(preview.tile, preview.footprint);
    let time = get_time() as f32;
    let existing_blocks = sim.block_debug_views_minimal();
    let ghost = sim::BlockDebugView {
        id: 0,
        kind: preview.kind,
        tile: preview.tile,
        footprint: preview.footprint,
        orientation: preview.orientation,
        raw_qty: 0,
        inventory_summary: String::new(),
        rack_levels: [false; 6],
    };

    let modern_ready = sim.modern_line_ready();
    draw_modern_block_visual(&ghost, rect, sim, &existing_blocks, time, modern_ready);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        with_alpha(
            Color::from_rgba(230, 238, 248, 255),
            if preview.can_place { 0.26 } else { 0.12 },
        ),
    );
    let border = if !preview.can_place {
        Color::from_rgba(238, 112, 94, 242)
    } else if preview.connects_to_line {
        Color::from_rgba(110, 230, 150, 236)
    } else {
        Color::from_rgba(248, 196, 104, 236)
    };
    draw_rectangle_lines(
        rect.x + 0.8,
        rect.y + 0.8,
        (rect.w - 1.6).max(1.0),
        (rect.h - 1.6).max(1.0),
        2.4,
        border,
    );

    if !preview.guidance.is_empty() {
        let guidance_color = if !preview.can_place {
            Color::from_rgba(255, 176, 166, 236)
        } else if preview.connects_to_line {
            Color::from_rgba(228, 252, 236, 240)
        } else {
            Color::from_rgba(255, 226, 166, 228)
        };
        let bg = if !preview.can_place {
            Color::from_rgba(70, 24, 20, 198)
        } else if preview.connects_to_line {
            Color::from_rgba(18, 54, 36, 196)
        } else {
            Color::from_rgba(72, 52, 14, 192)
        };
        let border_col = if !preview.can_place {
            Color::from_rgba(255, 136, 118, 180)
        } else if preview.connects_to_line {
            Color::from_rgba(130, 236, 180, 190)
        } else {
            Color::from_rgba(236, 186, 128, 180)
        };
        draw_text_chip(
            preview.guidance.as_str(),
            rect.x + 6.0,
            rect.y + rect.h + 12.0,
            11.0,
            guidance_color,
            bg,
            border_col,
        );
    }

    let axis = orientation_axis(preview.orientation);
    let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);
    let tip = center + axis * rect.w.min(rect.h) * 0.24;
    let base = center - axis * rect.w.min(rect.h) * 0.14;
    let normal = vec2(-axis.y, axis.x) * rect.w.min(rect.h) * 0.11;
    draw_triangle(tip, base + normal, base - normal, with_alpha(border, 0.74));
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
        draw_text_chip(
            &count,
            rect.x + 3.0,
            rect.y + rect.h - 3.0,
            11.0,
            Color::from_rgba(240, 246, 252, 230),
            Color::from_rgba(20, 26, 32, 182),
            Color::from_rgba(142, 176, 196, 170),
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
            draw_text_chip(
                &agent.label,
                px - 42.0,
                py - 10.0,
                14.0,
                Color::from_rgba(255, 244, 218, 255),
                Color::from_rgba(26, 20, 12, 196),
                Color::from_rgba(190, 150, 90, 184),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(lhs: f32, rhs: f32) {
        assert!((lhs - rhs).abs() <= 1e-5, "expected {rhs}, got {lhs}");
    }

    #[test]
    fn scaled_prop_texture_placement_recenters_square_assets() {
        let base_x = 5.0;
        let base_y = 7.0;
        let base_w = 22.0;
        let base_h = 22.0;

        let (draw_x, draw_y, draw_size) =
            scaled_prop_texture_placement(base_x, base_y, base_w, base_h);

        assert_close(draw_size.x, base_w * PROP_TEXTURE_VISUAL_SCALE);
        assert_close(draw_size.y, base_h * PROP_TEXTURE_VISUAL_SCALE);
        assert_close(draw_x, base_x - (draw_size.x - base_w) * 0.5);
        assert_close(draw_y, base_y - (draw_size.y - base_h) * 0.5);
    }

    #[test]
    fn scaled_prop_texture_placement_keeps_non_square_centered() {
        let base_x = 4.0;
        let base_y = 10.0;
        let base_w = 24.0;
        let base_h = 16.0;

        let (draw_x, draw_y, draw_size) =
            scaled_prop_texture_placement(base_x, base_y, base_w, base_h);

        assert_close(draw_size.x, base_w * PROP_TEXTURE_VISUAL_SCALE);
        assert_close(draw_size.y, base_h * PROP_TEXTURE_VISUAL_SCALE);
        assert_close(draw_x, base_x - (draw_size.x - base_w) * 0.5);
        assert_close(draw_y, base_y - (draw_size.y - base_h) * 0.5);
    }

    #[test]
    fn exterior_tree_selection_is_deterministic() {
        let world = generate_starter_factory_world(168, 108);
        let tile = (17, 14);
        let first = exterior_tree_type_for_tile(&world, tile.0, tile.1, world.get(tile.0, tile.1));
        let second = exterior_tree_type_for_tile(&world, tile.0, tile.1, world.get(tile.0, tile.1));
        assert_eq!(first, second);
    }

    #[test]
    fn exterior_tree_selection_covers_all_tree_types() {
        let world = generate_starter_factory_world(168, 108);
        let mut seen_chene = false;
        let mut seen_peuplier = false;
        let mut seen_pin = false;

        for y in 0..world.h {
            for x in 0..world.w {
                let tile = world.get(x, y);
                match exterior_tree_type_for_tile(&world, x, y, tile) {
                    Some(TypeArbreExterieur::Chene) => seen_chene = true,
                    Some(TypeArbreExterieur::Peuplier) => seen_peuplier = true,
                    Some(TypeArbreExterieur::Pin) => seen_pin = true,
                    None => {}
                }
            }
        }

        assert!(seen_chene);
        assert!(seen_peuplier);
        assert!(seen_pin);
    }

    #[test]
    fn exterior_tree_selection_keeps_factory_core_clear() {
        let world = generate_starter_factory_world(168, 108);
        let (fx0, fx1, fy0, fy1) = starter_factory_bounds(world.w, world.h);

        for y in fy0..=fy1 {
            for x in fx0..=fx1 {
                let tree = exterior_tree_type_for_tile(&world, x, y, world.get(x, y));
                assert!(tree.is_none());
            }
        }
    }
}
