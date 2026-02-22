Ok ✅ Là on le fait “propre RimWorld” : une grosse barre HUD en bas, bien pensée gameplay, qui capte les clics (donc pas d’actions “derrière l’UI”), avec minimap cliquable à droite, fiche/historique du perso à côté, menu build trié, sélection des persos à gauche, et une ligne d’éco claire (avec un symbole € dessiné, donc pas dépendant de la police).

Techniquement (court + pourquoi) 🙂
On ajoute un module ui_hud.rs “immediate mode” (macroquad pur) : c’est robuste, rapide à itérer, et ça évite d’introduire une dépendance UI externe.
On réduit le viewport de la caméra au-dessus de la barre du bas : ça améliore immédiatement le confort (la map ne “passe pas sous” le HUD, les bords de caméra sont cohérents).
On met un système de vitesse sim (pause/x1/x2/x4) directement dans le HUD : utile en jeu “aquarium”.

INSTRUCTIONS D’INTÉGRATION (copier/coller)

Crée src/ui_hud.rs (NOUVEAU) et colle TOUT ceci

use super::*;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudBuildTab {
Blocs,
Zones,
Outils,
}

impl HudBuildTab {
fn label(self) -> &'static str {
match self {
HudBuildTab::Blocs => "Blocs",
HudBuildTab::Zones => "Zones",
HudBuildTab::Outils => "Outils",
}
}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudInfoTab {
Fiche,
Historique,
}

impl HudInfoTab {
fn label(self) -> &'static str {
match self {
HudInfoTab::Fiche => "Fiche",
HudInfoTab::Historique => "Historique",
}
}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimSpeed {
Pause,
X1,
X2,
X4,
}

impl SimSpeed {
pub fn factor(self) -> f32 {
match self {
SimSpeed::Pause => 0.0,
SimSpeed::X1 => 1.0,
SimSpeed::X2 => 2.0,
SimSpeed::X4 => 4.0,
}
}

    fn label(self) -> &'static str {
        match self {
            SimSpeed::Pause => "||",
            SimSpeed::X1 => "1x",
            SimSpeed::X2 => "2x",
            SimSpeed::X4 => "4x",
        }
    }
}

#[derive(Clone, Debug)]
pub struct HudUiState {
pub build_tab: HudBuildTab,
pub info_tab: HudInfoTab,
pub sim_speed: SimSpeed,
}

impl Default for HudUiState {
fn default() -> Self {
Self {
build_tab: HudBuildTab::Blocs,
info_tab: HudInfoTab::Fiche,
sim_speed: SimSpeed::X1,
}
}
}

#[derive(Clone, Copy, Debug)]
pub struct HudLayout {
pub bar_rect: Rect,
pub top_strip_rect: Rect,
pub pawn_panel: Rect,
pub build_panel: Rect,
pub info_panel: Rect,
pub minimap_panel: Rect,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HudInputResult {
pub mouse_over_ui: bool,
pub consumed_click: bool,
pub consumed_wheel: bool,
}

pub fn build_hud_layout(_state: &GameState) -> HudLayout {
let sw = screen_width();
let sh = screen_height();
let scale = ((sw / 1600.0).min(sh / 900.0)).clamp(0.85, 1.2);

    let bar_h = (220.0 * scale).clamp(180.0, 270.0);
    let bar_rect = Rect::new(0.0, (sh - bar_h).max(0.0), sw, bar_h);

    let top_strip_h = (40.0 * scale).clamp(32.0, 48.0);
    let top_strip_rect = Rect::new(bar_rect.x, bar_rect.y, bar_rect.w, top_strip_h);

    let content_y = top_strip_rect.y + top_strip_rect.h;
    let content_h = (bar_rect.h - top_strip_rect.h).max(1.0);

    let pawn_w = (sw * 0.26).clamp(260.0, 420.0);
    let minimap_w = (sw * 0.22).clamp(240.0, 360.0);
    let info_w = (sw * 0.26).clamp(300.0, 470.0);
    let build_w = (sw - pawn_w - minimap_w - info_w).clamp(340.0, 560.0);

    let pawn_panel = Rect::new(bar_rect.x, content_y, pawn_w, content_h);
    let build_panel = Rect::new(pawn_panel.x + pawn_panel.w, content_y, build_w, content_h);
    let info_panel = Rect::new(build_panel.x + build_panel.w, content_y, info_w, content_h);
    let minimap_panel = Rect::new(info_panel.x + info_panel.w, content_y, minimap_w, content_h);

    HudLayout {
        bar_rect,
        top_strip_rect,
        pawn_panel,
        build_panel,
        info_panel,
        minimap_panel,
    }
}

pub fn process_hud_input(
state: &mut GameState,
layout: &HudLayout,
mouse: Vec2,
left_click: bool,
right_click: bool,
wheel_y: f32,
time_now: f32,
) -> HudInputResult {
let mut out = HudInputResult::default();

    let over_bar = point_in_rect(mouse, layout.bar_rect);
    out.mouse_over_ui = over_bar;

    if over_bar && wheel_y.abs() > f32::EPSILON {
        out.consumed_wheel = true;
    }

    if left_click {
        if point_in_rect(mouse, layout.top_strip_rect) {
            if process_top_strip_input(state, layout.top_strip_rect, mouse) {
                out.consumed_click = true;
                return out;
            }
        }
        if point_in_rect(mouse, layout.pawn_panel) {
            if process_pawn_panel_input(state, layout.pawn_panel, mouse, time_now) {
                out.consumed_click = true;
                return out;
            }
        }
        if point_in_rect(mouse, layout.build_panel) {
            if process_build_panel_input(state, layout.build_panel, mouse) {
                out.consumed_click = true;
                return out;
            }
        }
        if point_in_rect(mouse, layout.info_panel) {
            if process_info_panel_input(state, layout.info_panel, mouse) {
                out.consumed_click = true;
                return out;
            }
        }
        if point_in_rect(mouse, layout.minimap_panel) {
            if process_minimap_panel_input(state, layout.minimap_panel, mouse) {
                out.consumed_click = true;
                return out;
            }
        }
    }

    if right_click {
        if point_in_rect(mouse, layout.minimap_panel) {
            out.consumed_click = true;
            return out;
        }
        if point_in_rect(mouse, layout.build_panel)
            || point_in_rect(mouse, layout.pawn_panel)
            || point_in_rect(mouse, layout.info_panel)
            || point_in_rect(mouse, layout.top_strip_rect)
        {
            out.consumed_click = true;
            return out;
        }
    }

    out
}

pub fn draw_hud(
state: &GameState,
layout: &HudLayout,
mouse: Vec2,
map_view: Rect,
world_camera: &Camera2D,
time: f32,
) {
draw_bar_background(layout.bar_rect);

    draw_top_strip(state, layout.top_strip_rect, mouse);

    draw_pawn_panel(state, layout.pawn_panel, mouse, time);
    draw_build_panel(state, layout.build_panel, mouse);
    draw_info_panel(state, layout.info_panel, mouse);
    draw_minimap_panel(state, layout.minimap_panel, mouse, map_view, world_camera);

    if let Some(menu) = &state.pawn_ui.context_menu {
        ui_pawns::draw_pawn_context_menu(state, menu, mouse, time);
    }
}

fn draw_bar_background(bar: Rect) {
let bg0 = rgba(18, 26, 34, 240);
let bg1 = rgba(10, 14, 20, 250);
draw_rectangle(bar.x, bar.y, bar.w, bar.h, bg0);
draw_rectangle(bar.x, bar.y + bar.h * 0.55, bar.w, bar.h * 0.45, bg1);
draw_rectangle_lines(bar.x, bar.y, bar.w, bar.h, 2.0, rgba(80, 120, 160, 220));
}

fn ui_col_border() -> Color {
rgba(78, 118, 150, 210)
}

fn ui_col_border_hi() -> Color {
rgba(160, 210, 250, 235)
}

fn ui_col_accent() -> Color {
rgba(252, 208, 138, 248)
}

fn ui_shadow_offset(fs: f32) -> Vec2 {
vec2((fs * 0.06).clamp(1.0, 2.0), (fs * 0.08).clamp(1.0, 2.0))
}

fn ui_text_and_shadow_for_bg(bg: Color) -> (Color, Color) {
let luma = 0.2126 * bg.r + 0.7152 * bg.g + 0.0722 * bg.b;
if luma > 0.55 {
(rgba(10, 12, 14, 248), rgba(0, 0, 0, 110))
} else {
(rgba(235, 245, 255, 248), rgba(0, 0, 0, 160))
}
}

fn draw_text_shadowed(text: &str, x: f32, y: f32, fs: f32, fill: Color, shadow: Color, off: Vec2) {
draw_text(text, x + off.x, y + off.y, fs, shadow);
draw_text(text, x, y, fs, fill);
}

fn draw_panel_frame(rect: Rect, title: &str, mouse: Vec2) {
let hovered = point_in_rect(mouse, rect);
let bg = if hovered {
rgba(26, 34, 46, 246)
} else {
rgba(22, 30, 40, 242)
};
draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, ui_col_border());

    let header_h = 24.0;
    let header = Rect::new(rect.x, rect.y, rect.w, header_h);
    draw_rectangle(header.x, header.y, header.w, header.h, rgba(18, 24, 32, 250));
    draw_rectangle_lines(header.x, header.y, header.w, header.h, 1.0, rgba(110, 170, 220, 150));

    let fs = 16.0;
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);
    draw_text_shadowed(
        title,
        rect.x + 10.0,
        rect.y + 17.5,
        fs,
        fill,
        shadow,
        ui_shadow_offset(fs),
    );
}

fn draw_top_strip(state: &GameState, rect: Rect, mouse: Vec2) {
let bg = rgba(16, 22, 30, 252);
draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, rgba(90, 140, 190, 210));

    let pad = 10.0;
    let y = rect.y + rect.h * 0.72;
    let fs = (rect.h * 0.46).clamp(14.0, 20.0);
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);

    let time_label = format!("Heure {}  J{}", state.sim.clock.format_hhmm(), state.sim.clock.day_index() + 1);
    draw_text_shadowed(
        &time_label,
        rect.x + pad,
        y,
        fs,
        fill,
        shadow,
        ui_shadow_offset(fs),
    );

    let cash = state.sim.cash();
    let sold = state.sim.sold_total();
    let cadence = state.sim.throughput_per_hour();
    let otif = state.sim.otif();
    let profit = state.sim.profit_total();

    let mut x = rect.x + rect.w * 0.34;
    let pill_h = (rect.h * 0.78).clamp(26.0, 38.0);
    let pill_y = rect.y + rect.h * 0.12;

    x = draw_stat_pill(
        Rect::new(x, pill_y, 210.0, pill_h),
        "Tresorerie",
        &format_money(cash),
        ui_col_accent(),
        mouse,
        true,
    ) + 10.0;

    x = draw_stat_pill(
        Rect::new(x, pill_y, 170.0, pill_h),
        "Ventes",
        &format!("{}", sold),
        rgba(98, 152, 188, 240),
        mouse,
        false,
    ) + 10.0;

    x = draw_stat_pill(
        Rect::new(x, pill_y, 170.0, pill_h),
        "Cadence",
        &format!("{:.1}/h", cadence),
        rgba(120, 180, 130, 230),
        mouse,
        false,
    ) + 10.0;

    x = draw_stat_pill(
        Rect::new(x, pill_y, 160.0, pill_h),
        "Service",
        &format!("{:.0}%", (otif * 100.0).clamp(0.0, 999.0)),
        rgba(180, 200, 120, 230),
        mouse,
        false,
    ) + 10.0;

    let profit_col = if profit >= 0.0 {
        rgba(110, 210, 130, 235)
    } else {
        rgba(210, 110, 110, 235)
    };
    draw_stat_pill(
        Rect::new(x, pill_y, 190.0, pill_h),
        "Resultat",
        &format_money(profit),
        profit_col,
        mouse,
        true,
    );

    let btn_w = (36.0).max(rect.h * 0.82);
    let btn_h = (rect.h * 0.78).clamp(26.0, 38.0);
    let mut bx = rect.x + rect.w - pad - btn_w * 4.0 - 6.0 * 3.0;
    let by = rect.y + rect.h * 0.12;

    for speed in [SimSpeed::Pause, SimSpeed::X1, SimSpeed::X2, SimSpeed::X4] {
        let brect = Rect::new(bx, by, btn_w, btn_h);
        let hovered = point_in_rect(mouse, brect);
        let active = state.hud_ui.sim_speed == speed;
        draw_small_button(brect, speed.label(), hovered, active);
        bx += btn_w + 6.0;
    }
}

fn process_top_strip_input(state: &mut GameState, rect: Rect, mouse: Vec2) -> bool {
let pad = 10.0;
let btn_w = (36.0).max(rect.h * 0.82);
let btn_h = (rect.h * 0.78).clamp(26.0, 38.0);
let mut bx = rect.x + rect.w - pad - btn_w * 4.0 - 6.0 * 3.0;
let by = rect.y + rect.h * 0.12;

    for speed in [SimSpeed::Pause, SimSpeed::X1, SimSpeed::X2, SimSpeed::X4] {
        let brect = Rect::new(bx, by, btn_w, btn_h);
        if point_in_rect(mouse, brect) {
            state.hud_ui.sim_speed = speed;
            return true;
        }
        bx += btn_w + 6.0;
    }
    false
}

fn draw_stat_pill(rect: Rect, label: &str, value: &str, accent: Color, mouse: Vec2, euro: bool) -> f32 {
let hovered = point_in_rect(mouse, rect);
let bg = if hovered {
rgba(30, 40, 52, 245)
} else {
rgba(24, 34, 44, 242)
};
draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, ui_col_border());

    draw_rectangle(rect.x, rect.y, 6.0, rect.h, accent);

    let fs = (rect.h * 0.44).clamp(12.0, 18.0);
    let value_fs = (rect.h * 0.54).clamp(13.0, 19.0);
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);

    draw_text_shadowed(
        label,
        rect.x + 10.0,
        rect.y + rect.h * 0.63,
        fs,
        fill,
        shadow,
        ui_shadow_offset(fs),
    );

    let val_w = measure_text(value, None, value_fs as u16, 1.0).width;
    let val_x = rect.x + rect.w - pad - val_w;
    let val_y = rect.y + rect.h * 0.72;
    if euro {
        let icon_h = (value_fs * 0.92).clamp(10.0, rect.h * 0.92);
        let icon_w = icon_h * 0.65;
        let gap = 6.0;
        let icon_x = (val_x - gap - icon_w).max(rect.x + pad + 60.0);
        draw_euro_icon_shadowed(
            icon_x,
            val_y,
            icon_h,
            with_alpha(accent, 0.92),
            shadow,
            ui_shadow_offset(value_fs),
        );
    }
    draw_text_shadowed(
        value,
        val_x,
        val_y,
        value_fs,
        fill,
        shadow,
        ui_shadow_offset(value_fs),
    );

    rect.x + rect.w
}

fn draw_euro_icon_shadowed(x: f32, baseline_y: f32, h: f32, color: Color, shadow: Color, shadow_off: Vec2) {
draw_euro_icon(x + shadow_off.x, baseline_y + shadow_off.y, h, shadow);
draw_euro_icon(x, baseline_y, h, color);
}

fn draw_euro_icon(x: f32, baseline_y: f32, h: f32, color: Color) {
let w = h * 0.65;
let cx = x + w * 0.55;
let cy = baseline_y - h * 0.42;
let r = h * 0.38;
let thickness = (h * 0.10).clamp(1.0, 3.2);
let a0 = std::f32::consts::PI * 0.35;
let a1 = std::f32::consts::PI * 1.65;
let steps = 16;
let mut prev: Option<Vec2> = None;
for i in 0..=steps {
let t = i as f32 / steps as f32;
let a = a0 + (a1 - a0) * t;
let p = vec2(cx + r * a.cos(), cy + r * a.sin());
if let Some(pp) = prev {
draw_line(pp.x, pp.y, p.x, p.y, thickness, color);
}
prev = Some(p);
}
let bar_len = w * 0.82;
let bx0 = x + w * 0.08;
let by1 = cy - h * 0.10;
let by2 = cy + h * 0.10;
draw_line(bx0, by1, bx0 + bar_len, by1, thickness, color);
draw_line(bx0, by2, bx0 + bar_len, by2, thickness, color);
}

fn draw_small_button(rect: Rect, label: &str, hovered: bool, active: bool) {
let base = if active {
ui_col_accent()
} else if hovered {
rgba(98, 152, 188, 240)
} else {
rgba(68, 100, 128, 236)
};
let border = if active {
rgba(252, 208, 138, 252)
} else if hovered {
ui_col_border_hi()
} else {
rgba(120, 171, 199, 224)
};
draw_rectangle(rect.x, rect.y, rect.w, rect.h, base);
draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, border);

    let fs = (rect.h * 0.72).clamp(12.0, 18.0);
    let dims = measure_text(label, None, fs as u16, 1.0);
    let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
    let ty = rect.y + rect.h * 0.5 + dims.height * 0.34;
    let (fill, shadow) = ui_text_and_shadow_for_bg(base);
    draw_text_shadowed(label, tx, ty, fs, fill, shadow, ui_shadow_offset(fs));
}


fn process_pawn_panel_input(
state: &mut GameState,
panel: Rect,
mouse: Vec2,
time_now: f32,
) -> bool {
let slots = pawn_slot_layout(state, panel);
for slot in &slots {
if point_in_rect(mouse, slot.follow_rect) {
state.pawn_ui.selected = Some(slot.key);
state.pawn_ui.follow = if state.pawn_ui.follow == Some(slot.key) {
None
} else {
Some(slot.key)
};
if state.pawn_ui.follow == Some(slot.key) {
if let Some(pos) = ui_pawns::pawn_world_pos(state, slot.key) {
state.camera_center = pos;
}
}
return true;
}
}

    for slot in &slots {
        if point_in_rect(mouse, slot.rect) {
            state.pawn_ui.selected = Some(slot.key);
            if let Some(pos) = ui_pawns::pawn_world_pos(state, slot.key) {
                state.camera_center = pos;
            }

            let within = (time_now - state.pawn_ui.last_click_time) <= 0.35;
            let same = state.pawn_ui.last_click_pawn == Some(slot.key);
            if within && same {
                state.pawn_ui.follow = if state.pawn_ui.follow == Some(slot.key) {
                    None
                } else {
                    Some(slot.key)
                };
            }
            state.pawn_ui.last_click_time = time_now;
            state.pawn_ui.last_click_pawn = Some(slot.key);
            return true;
        }
    }
    false
}

fn draw_pawn_panel(state: &GameState, panel: Rect, mouse: Vec2, time: f32) {
draw_panel_frame(panel, "Equipe", mouse);

    let slots = pawn_slot_layout(state, panel);
    for slot in &slots {
        draw_pawn_slot(state, slot, mouse, time);
    }
}

#[derive(Clone)]
struct PawnSlot {
key: PawnKey,
rect: Rect,
follow_rect: Rect,
}

fn pawn_slot_layout(state: &GameState, panel: Rect) -> Vec<PawnSlot> {
let scale = ((screen_width() / 1600.0).min(screen_height() / 900.0)).clamp(0.85, 1.15);
let pad = 10.0;
let header_h = 24.0;
let inner = Rect::new(
panel.x + pad,
panel.y + header_h + 10.0,
(panel.w - pad * 2.0).max(1.0),
(panel.h - header_h - 18.0).max(1.0),
);

    let card_h = (64.0 * scale).clamp(52.0, 74.0);
    let card_w = (160.0 * scale).clamp(132.0, 180.0);
    let gap = (10.0 * scale).clamp(8.0, 14.0);

    let mut slots = Vec::with_capacity(state.pawns.len());
    let mut x = inner.x;
    let y = inner.y;
    for pawn in &state.pawns {
        let rect = Rect::new(x, y, card_w, card_h);
        let btn = (20.0 * scale).clamp(16.0, 24.0);
        let follow_rect = Rect::new(rect.x + rect.w - btn - 6.0, rect.y + 6.0, btn, btn);
        slots.push(PawnSlot {
            key: pawn.key,
            rect,
            follow_rect,
        });
        x += card_w + gap;
        if x + card_w > inner.x + inner.w {
            break;
        }
    }
    slots
}

fn draw_pawn_slot(state: &GameState, slot: &PawnSlot, mouse: Vec2, time: f32) {
let selected = state.pawn_ui.selected == Some(slot.key);
let following = state.pawn_ui.follow == Some(slot.key);
let hovered = point_in_rect(mouse, slot.rect);

    let base = if following {
        rgba(210, 150, 82, 235)
    } else if selected {
        rgba(98, 152, 188, 236)
    } else {
        rgba(54, 74, 96, 230)
    };
    let border = if hovered || selected || following {
        ui_col_border_hi()
    } else {
        ui_col_border()
    };
    draw_rectangle(slot.rect.x, slot.rect.y, slot.rect.w, slot.rect.h, base);
    draw_rectangle_lines(slot.rect.x, slot.rect.y, slot.rect.w, slot.rect.h, 2.0, border);

    let portrait_center = vec2(slot.rect.x + 24.0, slot.rect.y + slot.rect.h * 0.56);
    if let Some(record) = ui_pawns::pawn_visual_record(state, slot.key) {
        draw_character(
            record,
            CharacterRenderParams {
                center: portrait_center,
                scale: 0.72,
                facing: CharacterFacing::Front,
                facing_left: false,
                is_walking: false,
                walk_cycle: time * 2.0,
                gesture: CharacterGesture::None,
                time,
                debug: false,
            },
        );
    }

    let pawn = state.pawns.iter().find(|p| p.key == slot.key);
    let name = pawn
        .map(|p| p.name.as_str())
        .unwrap_or(slot.key.short_label());
    let bg = base;
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);
    draw_text_shadowed(
        name,
        slot.rect.x + 44.0,
        slot.rect.y + 22.0,
        16.0,
        fill,
        shadow,
        ui_shadow_offset(16.0),
    );

    let follow_hover = point_in_rect(mouse, slot.follow_rect);
    let follow_active = following;
    draw_small_button(slot.follow_rect, "F", follow_hover, follow_active);

    if let Some(pawn) = pawn {
        let bar_w = slot.rect.w - 54.0;
        let bar_x = slot.rect.x + 44.0;
        let bar_y = slot.rect.y + slot.rect.h - 14.0;
        let hp = pawn.health.clamp(0.0, 1.0);
        draw_meter(bar_x, bar_y, bar_w, 7.0, hp, rgba(120, 210, 140, 240));
    }
}

fn draw_meter(x: f32, y: f32, w: f32, h: f32, t: f32, col: Color) {
draw_rectangle(x, y, w, h, rgba(0, 0, 0, 140));
draw_rectangle(x, y, (w * t.clamp(0.0, 1.0)).max(0.0), h, col);
draw_rectangle_lines(x, y, w, h, 1.0, rgba(160, 210, 250, 110));
}

fn process_build_panel_input(state: &mut GameState, panel: Rect, mouse: Vec2) -> bool {
let tab_rects = build_tab_rects(panel);
for (tab, rect) in tab_rects {
if point_in_rect(mouse, rect) {
state.hud_ui.build_tab = tab;
return true;
}
}

    match state.hud_ui.build_tab {
        HudBuildTab::Blocs => process_build_blocks_input(state, panel, mouse),
        HudBuildTab::Zones => process_build_zones_input(state, panel, mouse),
        HudBuildTab::Outils => process_build_tools_input(state, panel, mouse),
    }
}

fn draw_build_panel(state: &GameState, panel: Rect, mouse: Vec2) {
draw_panel_frame(panel, "Construction", mouse);

    let tab_rects = build_tab_rects(panel);
    for (tab, rect) in tab_rects {
        let active = state.hud_ui.build_tab == tab;
        let hovered = point_in_rect(mouse, rect);
        draw_small_button(rect, tab.label(), hovered, active);
    }

    match state.hud_ui.build_tab {
        HudBuildTab::Blocs => draw_build_blocks(state, panel, mouse),
        HudBuildTab::Zones => draw_build_zones(state, panel, mouse),
        HudBuildTab::Outils => draw_build_tools(state, panel, mouse),
    }
}

fn build_tab_rects(panel: Rect) -> Vec<(HudBuildTab, Rect)> {
let pad = 10.0;
let header_h = 24.0;
let tabs_y = panel.y + header_h + 6.0;
let tab_h = 24.0;
let tab_w = 90.0;
let gap = 8.0;

    let mut x = panel.x + pad;
    let mut v = Vec::new();
    for tab in [HudBuildTab::Blocs, HudBuildTab::Zones, HudBuildTab::Outils] {
        let r = Rect::new(x, tabs_y, tab_w, tab_h);
        v.push((tab, r));
        x += tab_w + gap;
    }
    v
}

fn build_inner_rect(panel: Rect) -> Rect {
let pad = 10.0;
let header_h = 24.0;
let tabs_h = 24.0;
let inner_y = panel.y + header_h + 6.0 + tabs_h + 10.0;
let inner_h = (panel.h - (inner_y - panel.y) - 10.0).max(1.0);
Rect::new(panel.x + pad, inner_y, (panel.w - pad * 2.0).max(1.0), inner_h)
}

fn build_item_button_rects(inner: Rect, cols: usize, rows: usize) -> Vec<Rect> {
let gap = 8.0;
let bw = ((inner.w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
let bh = ((inner.h - gap * (rows as f32 - 1.0)) / rows as f32).max(1.0);

    let mut rects = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            let x = inner.x + c as f32 * (bw + gap);
            let y = inner.y + r as f32 * (bh + gap);
            rects.push(Rect::new(x, y, bw, bh));
        }
    }
    rects
}

fn process_build_blocks_input(state: &mut GameState, panel: Rect, mouse: Vec2) -> bool {
let inner = build_inner_rect(panel);
let rects = build_item_button_rects(inner, 2, 2);

    let options: [BlockKind; 4] = [BlockKind::Storage, BlockKind::MachineA, BlockKind::MachineB, BlockKind::Buffer];

    for (i, kind) in options.iter().enumerate() {
        if i >= rects.len() {
            break;
        }
        if point_in_rect(mouse, rects[i]) {
            state.sim.set_block_brush(*kind);
            state.sim.toggle_build_mode();
            if !state.sim.build_mode_enabled() {
                state.sim.toggle_build_mode();
            }
            state.hud_ui.build_tab = HudBuildTab::Blocs;
            return true;
        }
    }
    false
}

fn draw_build_blocks(state: &GameState, panel: Rect, mouse: Vec2) {
let inner = build_inner_rect(panel);
let rects = build_item_button_rects(inner, 2, 2);

    let options: [BlockKind; 4] = [BlockKind::Storage, BlockKind::MachineA, BlockKind::MachineB, BlockKind::Buffer];

    for (i, kind) in options.iter().enumerate() {
        if i >= rects.len() {
            break;
        }
        let r = rects[i];
        let hovered = point_in_rect(mouse, r);
        let active = state.sim.block_brush() == *kind;
        draw_build_item_tile(r, kind.label(), hovered, active);
    }

    draw_build_footer(panel, state, mouse);
}

fn process_build_zones_input(state: &mut GameState, panel: Rect, mouse: Vec2) -> bool {
let inner = build_inner_rect(panel);
let rects = build_item_button_rects(inner, 2, 2);

    let options: [ZoneKind; 4] = [ZoneKind::Receiving, ZoneKind::Processing, ZoneKind::Shipping, ZoneKind::Support];

    for (i, kind) in options.iter().enumerate() {
        if i >= rects.len() {
            break;
        }
        if point_in_rect(mouse, rects[i]) {
            state.sim.set_zone_brush(*kind);
            state.sim.set_zone_paint_mode(true);
            state.sim.toggle_build_mode();
            if !state.sim.build_mode_enabled() {
                state.sim.toggle_build_mode();
            }
            state.hud_ui.build_tab = HudBuildTab::Zones;
            return true;
        }
    }
    false
}

fn draw_build_zones(state: &GameState, panel: Rect, mouse: Vec2) {
let inner = build_inner_rect(panel);
let rects = build_item_button_rects(inner, 2, 2);

    let options: [ZoneKind; 4] = [ZoneKind::Receiving, ZoneKind::Processing, ZoneKind::Shipping, ZoneKind::Support];

    for (i, kind) in options.iter().enumerate() {
        if i >= rects.len() {
            break;
        }
        let r = rects[i];
        let hovered = point_in_rect(mouse, r);
        let active = state.sim.zone_brush() == *kind;
        draw_build_item_tile(r, kind.label(), hovered, active);
    }

    draw_build_footer(panel, state, mouse);
}

fn process_build_tools_input(state: &mut GameState, panel: Rect, mouse: Vec2) -> bool {
let inner = build_inner_rect(panel);
let rects = build_item_button_rects(inner, 2, 2);

    if rects.len() >= 1 && point_in_rect(mouse, rects[0]) {
        state.sim.toggle_build_mode();
        return true;
    }
    if rects.len() >= 2 && point_in_rect(mouse, rects[1]) {
        state.sim.toggle_zone_overlay();
        return true;
    }
    if rects.len() >= 3 && point_in_rect(mouse, rects[2]) {
        state.sim.set_zone_paint_mode(!state.sim.zone_paint_mode_enabled());
        return true;
    }
    if rects.len() >= 4 && point_in_rect(mouse, rects[3]) {
        if state.sim.pending_move_block().is_some() {
            state.sim.clear_pending_move_block();
        } else {
            state.sim.build_status = "Move: appuie M sur la map pour choisir la source".to_string();
        }
        return true;
    }
    false
}

fn draw_build_tools(state: &GameState, panel: Rect, mouse: Vec2) {
let inner = build_inner_rect(panel);
let rects = build_item_button_rects(inner, 2, 2);

    if rects.len() >= 1 {
        let active = state.sim.build_mode_enabled();
        let hovered = point_in_rect(mouse, rects[0]);
        draw_build_item_tile(rects[0], "Mode build", hovered, active);
    }
    if rects.len() >= 2 {
        let active = state.sim.zone_overlay_enabled();
        let hovered = point_in_rect(mouse, rects[1]);
        draw_build_item_tile(rects[1], "Overlay zones", hovered, active);
    }
    if rects.len() >= 3 {
        let active = state.sim.zone_paint_mode_enabled();
        let hovered = point_in_rect(mouse, rects[2]);
        draw_build_item_tile(rects[2], "Peinture zones", hovered, active);
    }
    if rects.len() >= 4 {
        let active = state.sim.pending_move_block().is_some();
        let hovered = point_in_rect(mouse, rects[3]);
        draw_build_item_tile(rects[3], "Move", hovered, active);
    }

    draw_build_footer(panel, state, mouse);
}

fn draw_build_item_tile(rect: Rect, label: &str, hovered: bool, active: bool) {
let base = if active {
rgba(252, 208, 138, 220)
} else if hovered {
rgba(98, 152, 188, 225)
} else {
rgba(46, 62, 82, 230)
};
draw_rectangle(rect.x, rect.y, rect.w, rect.h, base);
draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, ui_col_border_hi());

    let fs = (rect.h * 0.20).clamp(12.0, 18.0);
    let dims = measure_text(label, None, fs as u16, 1.0);
    let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
    let ty = rect.y + rect.h - 10.0;
    let (fill, shadow) = ui_text_and_shadow_for_bg(base);
    draw_text_shadowed(label, tx, ty, fs, fill, shadow, ui_shadow_offset(fs));
}

fn draw_build_footer(panel: Rect, state: &GameState, mouse: Vec2) {
let footer_h = 22.0;
let r = Rect::new(panel.x + 10.0, panel.y + panel.h - footer_h - 8.0, panel.w - 20.0, footer_h);
let bg = rgba(14, 18, 24, 230);
draw_rectangle(r.x, r.y, r.w, r.h, bg);
draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, rgba(120, 171, 199, 140));

    let fs = 14.0;
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);
    let text = state.sim.status_line();
    draw_text_shadowed(
        text,
        r.x + 8.0,
        r.y + r.h * 0.72,
        fs,
        fill,
        shadow,
        ui_shadow_offset(fs),
    );

    let hint = "Astuce: clic droit sur un perso -> menu social";
    let dims = measure_text(hint, None, 12, 1.0);
    if dims.width + 12.0 < r.w {
        draw_text_shadowed(
            hint,
            r.x + r.w - dims.width - 8.0,
            r.y + r.h * 0.72,
            12.0,
            rgba(200, 224, 236, 240),
            shadow,
            ui_shadow_offset(12.0),
        );
    }

    let _ = mouse;
}

fn process_info_panel_input(state: &mut GameState, panel: Rect, mouse: Vec2) -> bool {
let tab_rects = info_tab_rects(panel);
for (tab, rect) in tab_rects {
if point_in_rect(mouse, rect) {
state.hud_ui.info_tab = tab;
return true;
}
}
false
}

fn draw_info_panel(state: &GameState, panel: Rect, mouse: Vec2) {
draw_panel_frame(panel, "Personnage", mouse);

    let tab_rects = info_tab_rects(panel);
    for (tab, rect) in tab_rects {
        let active = state.hud_ui.info_tab == tab;
        let hovered = point_in_rect(mouse, rect);
        draw_small_button(rect, tab.label(), hovered, active);
    }

    match state.hud_ui.info_tab {
        HudInfoTab::Fiche => draw_info_sheet(state, panel, mouse),
        HudInfoTab::Historique => draw_info_history(state, panel, mouse),
    }
}

fn info_tab_rects(panel: Rect) -> Vec<(HudInfoTab, Rect)> {
let pad = 10.0;
let header_h = 24.0;
let tabs_y = panel.y + header_h + 6.0;
let tab_h = 24.0;
let tab_w = 110.0;
let gap = 8.0;

    let mut x = panel.x + pad;
    let mut v = Vec::new();
    for tab in [HudInfoTab::Fiche, HudInfoTab::Historique] {
        let r = Rect::new(x, tabs_y, tab_w, tab_h);
        v.push((tab, r));
        x += tab_w + gap;
    }
    v
}

fn info_inner_rect(panel: Rect) -> Rect {
let pad = 10.0;
let header_h = 24.0;
let tabs_h = 24.0;
let inner_y = panel.y + header_h + 6.0 + tabs_h + 10.0;
let inner_h = (panel.h - (inner_y - panel.y) - 10.0).max(1.0);
Rect::new(panel.x + pad, inner_y, (panel.w - pad * 2.0).max(1.0), inner_h)
}

fn selected_pawn_record<'a>(state: &'a GameState) -> Option<&'a PawnCard> {
state
.pawn_ui
.selected
.and_then(|k| state.pawns.iter().find(|p| p.key == k))
}

fn draw_info_sheet(state: &GameState, panel: Rect, mouse: Vec2) {
let inner = info_inner_rect(panel);
let bg = rgba(16, 22, 30, 220);
draw_rectangle(inner.x, inner.y, inner.w, inner.h, bg);
draw_rectangle_lines(inner.x, inner.y, inner.w, inner.h, 1.5, rgba(120, 171, 199, 140));

    let Some(pawn) = selected_pawn_record(state) else {
        let fs = 18.0;
        let dims = measure_text("Aucun personnage selectionne.", None, fs as u16, 1.0);
        let x = inner.x + inner.w * 0.5 - dims.width * 0.5;
        let y = inner.y + inner.h * 0.5;
        let (fill, shadow) = ui_text_and_shadow_for_bg(bg);
        draw_text_shadowed(
            "Aucun personnage selectionne.",
            x,
            y,
            fs,
            fill,
            shadow,
            ui_shadow_offset(fs),
        );
        return;
    };

    let title_fs = 20.0;
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);
    let role = match pawn.key {
        PawnKey::Player => "Patron",
        PawnKey::Npc => "Visiteur",
        PawnKey::SimWorker => "Employe",
    };
    let title = format!("{} - {}", pawn.name, role);
    draw_text_shadowed(
        &title,
        inner.x + 10.0,
        inner.y + 24.0,
        title_fs,
        fill,
        shadow,
        ui_shadow_offset(title_fs),
    );

    let mut y = inner.y + 42.0;
    let bar_w = inner.w - 20.0;

    draw_text_shadowed(
        "Besoins",
        inner.x + 10.0,
        y + 20.0,
        16.0,
        rgba(210, 225, 236, 240),
        shadow,
        ui_shadow_offset(16.0),
    );
    y += 26.0;

    let needs = &pawn.metrics.needs;
    let entries: [(&str, f32, Color); 6] = [
        ("Manger", needs.hunger, rgba(140, 200, 150, 230)),
        ("Boire", needs.thirst, rgba(120, 170, 210, 230)),
        ("Energie", needs.energy, rgba(200, 200, 120, 230)),
        ("Toilettes", needs.toilet, rgba(180, 140, 210, 230)),
        ("Hygiene", needs.hygiene, rgba(130, 210, 200, 230)),
        ("Social", needs.social, rgba(210, 180, 120, 230)),
    ];

    for (name, v, col) in entries {
        draw_labeled_bar(inner.x + 10.0, y, bar_w, 12.0, name, v, col, bg);
        y += 18.0;
    }

    y += 8.0;
    draw_text_shadowed(
        "Etats",
        inner.x + 10.0,
        y + 20.0,
        16.0,
        rgba(210, 225, 236, 240),
        shadow,
        ui_shadow_offset(16.0),
    );
    y += 26.0;

    let stats: [(&str, f32, Color); 3] = [
        ("Sante", pawn.metrics.health, rgba(120, 210, 140, 240)),
        ("Fatigue", pawn.metrics.fatigue, rgba(210, 180, 120, 240)),
        ("Moral", pawn.metrics.morale, rgba(120, 170, 230, 240)),
    ];
    for (name, v, col) in stats {
        draw_labeled_bar(inner.x + 10.0, y, bar_w, 12.0, name, v, col, bg);
        y += 18.0;
    }

    y += 10.0;
    if pawn.key == PawnKey::SimWorker {
        let fs = 14.0;
        let activity = state.sim.primary_agent_activity_label();
        let t = format!("Activite: {}", activity);
        draw_text_shadowed(
            &t,
            inner.x + 10.0,
            (inner.y + inner.h - 12.0).min(y + 24.0),
            fs,
            rgba(220, 235, 242, 240),
            shadow,
            ui_shadow_offset(fs),
        );
    }

    let _ = mouse;
}

fn draw_labeled_bar(
x: f32,
y: f32,
w: f32,
h: f32,
label: &str,
v: f32,
col: Color,
bg: Color,
) {
let fs = 12.0;
let (fill, shadow) = ui_text_and_shadow_for_bg(bg);
draw_text_shadowed(label, x, y + h, fs, fill, shadow, ui_shadow_offset(fs));
let bar_x = x + 84.0;
let bar_w = (w - 84.0).max(1.0);
draw_meter(bar_x, y + 2.0, bar_w, h - 4.0, v, col);
}

fn draw_info_history(state: &GameState, panel: Rect, mouse: Vec2) {
let inner = info_inner_rect(panel);
let bg = rgba(12, 18, 26, 228);
draw_rectangle(inner.x, inner.y, inner.w, inner.h, bg);
draw_rectangle_lines(inner.x, inner.y, inner.w, inner.h, 1.5, rgba(120, 171, 199, 140));

    let Some(pawn) = selected_pawn_record(state) else {
        return;
    };

    let title_fs = 16.0;
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);
    draw_text_shadowed(
        "Evenements recents",
        inner.x + 10.0,
        inner.y + 20.0,
        title_fs,
        fill,
        shadow,
        ui_shadow_offset(title_fs),
    );

    let mut y = inner.y + 34.0;
    let line_fs = 14.0;
    let max_lines = 10usize;

    let mut shown = 0usize;
    let mut last_sig: Option<(crate::historique::LogCategorie, &str)> = None;
    let mut last_time: f32 = -999999.0;

    for entry in pawn.history.iter().rev() {
        match entry.cat {
            crate::historique::LogCategorie::Deplacement => continue,
            crate::historique::LogCategorie::Debug => continue,
            _ => {}
        }

        let sig = (entry.cat, entry.msg.as_str());
        let dt = (last_time - entry.t_sim_s).abs();
        if let Some((last_cat, last_msg)) = last_sig {
            if last_cat == sig.0 && last_msg == sig.1 && dt <= 6.0 {
                continue;
            }
        }

        let t = format!("[{}] {}", format_clock_hhmm(entry.t_sim_s), entry.msg);
        draw_text_shadowed(
            &t,
            inner.x + 10.0,
            y + line_fs,
            line_fs,
            rgba(230, 240, 250, 242),
            shadow,
            ui_shadow_offset(line_fs),
        );
        y += 18.0;
        shown += 1;
        last_sig = Some(sig);
        last_time = entry.t_sim_s;

        if shown >= max_lines || y + 20.0 > inner.y + inner.h {
            break;
        }
    }

    if point_in_rect(mouse, inner) {
        draw_rectangle_lines(inner.x + 2.0, inner.y + 2.0, inner.w - 4.0, inner.h - 4.0, 1.0, rgba(252, 208, 138, 140));
    }
}

fn format_clock_hhmm(t_sim_s: f32) -> String {
let total = t_sim_s.max(0.0) as i32;
let h = (total / 3600) % 24;
let m = (total / 60) % 60;
format!("{:02}:{:02}", h, m)
}

fn process_minimap_panel_input(state: &mut GameState, panel: Rect, mouse: Vec2) -> bool {
let inner = minimap_inner_rect(panel);
if !point_in_rect(mouse, inner) {
return false;
}

    let world_w = state.world.w as f32 * TILE_SIZE;
    let world_h = state.world.h as f32 * TILE_SIZE;
    if world_w <= 1.0 || world_h <= 1.0 {
        return true;
    }

    let nx = ((mouse.x - inner.x) / inner.w).clamp(0.0, 1.0);
    let ny = ((mouse.y - inner.y) / inner.h).clamp(0.0, 1.0);
    let wx = nx * world_w;
    let wy = ny * world_h;
    state.camera_center = vec2(wx, wy);
    state.pawn_ui.follow = None;
    true
}

fn draw_minimap_panel(
state: &GameState,
panel: Rect,
mouse: Vec2,
map_view: Rect,
world_camera: &Camera2D,
) {
draw_panel_frame(panel, "Minimap", mouse);

    let inner = minimap_inner_rect(panel);
    let bg = rgba(10, 14, 18, 240);
    draw_rectangle(inner.x, inner.y, inner.w, inner.h, bg);
    draw_rectangle_lines(inner.x, inner.y, inner.w, inner.h, 1.5, rgba(120, 171, 199, 140));

    let world_w = state.world.w as f32 * TILE_SIZE;
    let world_h = state.world.h as f32 * TILE_SIZE;
    if world_w <= 1.0 || world_h <= 1.0 {
        return;
    }

    let stride = 2;
    for ty in (0..state.world.h).step_by(stride) {
        for tx in (0..state.world.w).step_by(stride) {
            let kind = state.world.tile_kind(tx, ty);
            let col = match kind {
                TileKind::Floor => rgba(60, 80, 100, 110),
                TileKind::Wall => rgba(120, 150, 180, 180),
                TileKind::Water => rgba(40, 90, 140, 160),
                TileKind::Dirt => rgba(80, 70, 60, 110),
            };
            let x = inner.x + (tx as f32 / state.world.w as f32) * inner.w;
            let y = inner.y + (ty as f32 / state.world.h as f32) * inner.h;
            let w = inner.w / state.world.w as f32 * stride as f32;
            let h = inner.h / state.world.h as f32 * stride as f32;
            draw_rectangle(x, y, w + 0.5, h + 0.5, col);
        }
    }

    for b in state.sim.blocks() {
        let (tx, ty) = b.origin_tile;
        let x = inner.x + (tx as f32 / state.world.w as f32) * inner.w;
        let y = inner.y + (ty as f32 / state.world.h as f32) * inner.h;
        draw_rectangle(x, y, 4.0, 4.0, rgba(252, 208, 138, 220));
    }

    let pawn_points: [(PawnKey, Vec2, Color); 3] = [
        (PawnKey::Player, state.player.pos, rgba(120, 220, 160, 240)),
        (PawnKey::Npc, state.npc.pos, rgba(220, 170, 120, 240)),
        (
            PawnKey::SimWorker,
            tile_center(state.sim.primary_agent_tile()),
            rgba(150, 200, 250, 240),
        ),
    ];
    for (_k, pos, col) in pawn_points {
        let nx = (pos.x / world_w).clamp(0.0, 1.0);
        let ny = (pos.y / world_h).clamp(0.0, 1.0);
        let px = inner.x + nx * inner.w;
        let py = inner.y + ny * inner.h;
        draw_circle(px, py, 3.0, col);
        draw_circle_lines(px, py, 3.2, 1.0, rgba(0, 0, 0, 120));
    }

    ensure_default_material();
    let a = world_camera.screen_to_world(vec2(map_view.x, map_view.y));
    let b = world_camera.screen_to_world(vec2(map_view.x + map_view.w, map_view.y));
    let c = world_camera.screen_to_world(vec2(map_view.x + map_view.w, map_view.y + map_view.h));
    let d = world_camera.screen_to_world(vec2(map_view.x, map_view.y + map_view.h));

    let min_x = a.x.min(b.x).min(c.x).min(d.x).clamp(0.0, world_w);
    let max_x = a.x.max(b.x).max(c.x).max(d.x).clamp(0.0, world_w);
    let min_y = a.y.min(b.y).min(c.y).min(d.y).clamp(0.0, world_h);
    let max_y = a.y.max(b.y).max(c.y).max(d.y).clamp(0.0, world_h);

    let rx = inner.x + (min_x / world_w) * inner.w;
    let ry = inner.y + (min_y / world_h) * inner.h;
    let rw = ((max_x - min_x) / world_w) * inner.w;
    let rh = ((max_y - min_y) / world_h) * inner.h;

    draw_rectangle_lines(rx, ry, rw, rh, 2.0, rgba(252, 208, 138, 240));
}

fn minimap_inner_rect(panel: Rect) -> Rect {
let pad = 10.0;
let header_h = 24.0;
Rect::new(
panel.x + pad,
panel.y + header_h + 10.0,
(panel.w - pad * 2.0).max(1.0),
(panel.h - header_h - 20.0).max(1.0),
)
}

fn format_money(amount: f64) -> String {
let rounded = amount.round() as i64;
format_int_fr(rounded)
}

fn format_int_fr(v: i64) -> String {
let sign = if v < 0 { "-" } else { "" };
let mut n = v.abs() as u64;
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

Patch src/main.rs

A) Dans la liste des mod ... (en haut), ajoute :

mod ui_hud;

B) Dans les use ... (en haut), ajoute :

use ui_hud::*;

C) Dans pub struct GameState, ajoute le champ :

hud_ui: HudUiState,

Place-le juste après pawn_ui: PawnsUiState, (comme ça c’est logique : UI pawns + HUD).

Patch src/edition.rs

Dans GameState { ... } (dans new_game_state), ajoute :

hud_ui: HudUiState::default(),

Juste après pawn_ui: PawnsUiState::default(),.

Patch src/historique.rs

Change la dérive de LogCategorie :

Avant :

#[derive(Clone, Copy, Debug)]
pub enum LogCategorie {

Après :

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogCategorie {

Patch src/ui_pawns.rs

Rends la fonction de menu contextuel accessible :

Avant :

fn draw_pawn_context_menu(

Après :

pub fn draw_pawn_context_menu(

Patch src/sim.rs

A) Remplace ces 5 fonctions (dans impl FactorySim) par ces versions

    pub fn toggle_zone_overlay(&mut self) {
        self.show_zone_overlay = !self.show_zone_overlay;
        self.build_status = if self.show_zone_overlay {
            "Overlay zones : ON".to_string()
        } else {
            "Overlay zones : OFF".to_string()
        };
    }

    pub fn toggle_build_mode(&mut self) {
        self.build_mode = !self.build_mode;
        if !self.build_mode {
            self.pending_move_block = None;
        }
        self.build_status = if self.build_mode {
            "Mode construction : ON".to_string()
        } else {
            "Mode construction : OFF".to_string()
        };
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
            "Peinture zones : ON".to_string()
        } else {
            "Peinture zones : OFF".to_string()
        };
    }

B) Ajoute ce bloc de méthodes JUSTE APRÈS toggle_zone_paint_mode et JUSTE AVANT select_move_source

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
            "Peinture zones : ON".to_string()
        } else {
            "Peinture zones : OFF".to_string()
        };
    }

    pub fn pending_move_block(&self) -> Option<BlockId> {
        self.pending_move_block
    }

    pub fn clear_pending_move_block(&mut self) {
        self.pending_move_block = None;
        self.build_status = "Deplacement annule".to_string();
    }

Patch src/modes.rs

Remplace ENTIEREMENT la fonction pub(crate) fn run_play_frame(...) par celle-ci (c’est le plus sûr, sinon tu vas courir après des petits détails) :

pub(crate) fn run_play_frame(
state: &mut GameState,
frame_dt: f32,
accumulator: &mut f32,
) -> PlayAction {
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

    let time_now = get_time() as f32;
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let wheel_y = mouse_wheel().1;
    let left_click = is_mouse_button_pressed(MouseButton::Left);
    let right_click = is_mouse_button_pressed(MouseButton::Right);

    ui_pawns::sync_dynamic_pawn_metrics(state);

    let now_sim_s = state.sim.clock.seconds();
    let mut context_menu_consumed =
        ui_pawns::process_pawn_context_menu_input(state, mouse, left_click, right_click, now_sim_s);

    let hud_layout = ui_hud::build_hud_layout(state);
    let hud_input = ui_hud::process_hud_input(
        state,
        &hud_layout,
        mouse,
        left_click && !context_menu_consumed,
        right_click && !context_menu_consumed,
        wheel_y,
        time_now,
    );

    let wheel_units = normalize_wheel_units(wheel_y);
    if !hud_input.consumed_wheel && !hud_input.mouse_over_ui && wheel_units.abs() > f32::EPSILON {
        let zoom_factor = (1.0 + PLAY_CAMERA_ZOOM_STEP).powf(wheel_units);
        state.camera_zoom =
            (state.camera_zoom * zoom_factor).clamp(PLAY_CAMERA_ZOOM_MIN, PLAY_CAMERA_ZOOM_MAX);
    }

    if is_key_pressed(KeyCode::C) {
        state.camera_center = state.player.pos;
        state.pawn_ui.follow = None;
    }

    if let Some(follow) = state.pawn_ui.follow
        && let Some(pos) = ui_pawns::pawn_world_pos(state, follow)
    {
        state.camera_center = pos;
    }

    let pan = read_camera_pan_input();
    if pan.length_squared() > 0.0 {
        state.pawn_ui.follow = None;
        let speed = PLAY_CAMERA_PAN_SPEED / state.camera_zoom.max(0.01);
        state.camera_center += pan * speed * frame_dt;
    }

    let sw = screen_width();
    let margin = PLAY_CAMERA_MARGIN;
    let view_rect = Rect::new(
        margin,
        margin,
        (sw - margin * 2.0).max(1.0),
        (hud_layout.bar_rect.y - margin * 2.0).max(1.0),
    );

    let (world_camera, clamped_center, clamped_zoom) = build_world_camera_for_viewport(
        &state.world,
        state.camera_center,
        state.camera_zoom,
        view_rect,
        PLAY_CAMERA_ZOOM_MIN,
        PLAY_CAMERA_ZOOM_MAX,
    );
    state.camera_center = clamped_center;
    state.camera_zoom = clamped_zoom;
    let map_view_rect = view_rect;

    let mouse_in_map = point_in_rect(mouse, map_view_rect) && !hud_input.mouse_over_ui;
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
    if !context_menu_consumed
        && right_click
        && mouse_in_map
        && !hud_input.mouse_over_ui
        && let Some(world_pos) = mouse_world
    {
        if let Some(target) = ui_pawns::hit_test_pawn_world(state, world_pos) {
            ui_pawns::open_pawn_context_menu(state, target, mouse);
            context_menu_consumed = true;
        } else {
            state.pawn_ui.context_menu = None;
        }
    }

    let mut clicked_pawn: Option<PawnKey> = None;
    if !context_menu_consumed
        && left_click
        && mouse_in_map
        && !hud_input.mouse_over_ui
        && !hud_input.consumed_click
        && !state.sim.build_mode_enabled()
        && let Some(world_pos) = mouse_world
    {
        clicked_pawn = ui_pawns::hit_test_pawn_world(state, world_pos);
        if let Some(hit) = clicked_pawn {
            state.pawn_ui.selected = Some(hit);
            state.pawn_ui.context_menu = None;
            if let Some(pos) = ui_pawns::pawn_world_pos(state, hit) {
                state.camera_center = pos;
            }
            context_menu_consumed = true;
        }
    }

    if is_key_pressed(KeyCode::M)
        && let Some(tile) = mouse_tile
    {
        state.sim.select_move_source(tile);
    }

    if state.sim.build_mode_enabled() {
        if left_click
            && mouse_in_map
            && !context_menu_consumed
            && let Some(tile) = mouse_tile
        {
            state.sim.apply_build_click(tile, false);
        }
        if right_click
            && mouse_in_map
            && !context_menu_consumed
            && let Some(tile) = mouse_tile
        {
            state.sim.apply_build_click(tile, true);
        }
    }

    let click_tile = if left_click
        && mouse_in_map
        && !state.sim.build_mode_enabled()
        && !context_menu_consumed
        && clicked_pawn.is_none()
    {
        mouse_tile
    } else {
        None
    };

    if let Some(tile) = click_tile
        && let Some(player_card) = state.pawns.iter_mut().find(|p| p.key == PawnKey::Player)
    {
        player_card.history.push(
            now_sim_s,
            crate::historique::LogCategorie::Deplacement,
            format!("Ordre de deplacement vers ({}, {}).", tile.0, tile.1),
        );
    }

    state.last_input = read_input_dir();
    apply_control_inputs(
        &mut state.player,
        &state.world,
        state.last_input,
        click_tile,
    );

    let sim_factor = state.hud_ui.sim_speed.factor();
    if sim_factor <= 0.0 {
        *accumulator = 0.0;
    } else {
        *accumulator = (*accumulator + frame_dt * sim_factor)
            .min(FIXED_DT * MAX_SIM_STEPS_PER_FRAME as f32);
    }
    let mut sim_steps = 0usize;
    while *accumulator >= FIXED_DT && sim_steps < MAX_SIM_STEPS_PER_FRAME {
        state.sim.step(FIXED_DT);
        let sim_now_s = state.sim.clock.seconds();
        state.social_state.tick(
            FIXED_DT,
            sim_now_s,
            social::SocialTickContext {
                world: &state.world,
                sim: &state.sim,
            },
            social::SocialTickActors {
                player: &mut state.player,
                npc: &mut state.npc,
                pawns: &mut state.pawns,
            },
        );
        update_player(&mut state.player, &state.world, state.last_input, FIXED_DT);
        update_npc_wanderer(&mut state.npc, &state.world, FIXED_DT);
        *accumulator -= FIXED_DT;
        sim_steps += 1;
    }
    if sim_steps == MAX_SIM_STEPS_PER_FRAME && *accumulator >= FIXED_DT {
        *accumulator = 0.0;
    }

    ui_pawns::sync_dynamic_pawn_metrics(state);

    let time = time_now;
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

    let worker_pos = tile_center(state.sim.primary_agent_tile());
    let mut draw_order: [(f32, PawnKey); 3] = [
        (state.player.pos.y, PawnKey::Player),
        (state.npc.pos.y, PawnKey::Npc),
        (worker_pos.y, PawnKey::SimWorker),
    ];
    draw_order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    for (_, key) in draw_order {
        let hint = state.social_state.anim_hint(key);
        let gesture = hint.gesture.map(gesture_from_social).unwrap_or(CharacterGesture::None);

        match key {
            PawnKey::Player => {
                let mut facing = state.player.facing;
                let mut facing_left = state.player.facing_left;
                let mut is_walking = state.player.is_walking;

                if hint.force_face_partner
                    && let Some(partner) = hint.partner
                    && let Some(target_pos) = ui_pawns::pawn_world_pos(state, partner)
                {
                    let dir = target_pos - state.player.pos;
                    facing = select_character_facing(dir, facing);
                    facing_left = dir.x < 0.0;
                }
                if hint.force_idle {
                    is_walking = false;
                }

                draw_character(
                    &state.player_character,
                    CharacterRenderParams {
                        center: state.player.pos,
                        scale: 1.0,
                        facing,
                        facing_left,
                        is_walking,
                        walk_cycle: state.player.walk_cycle,
                        gesture,
                        time,
                        debug: false,
                    },
                );

                if state.debug {
                    draw_rectangle_lines(
                        state.player.pos.x - state.player.half.x,
                        state.player.pos.y - state.player.half.y,
                        state.player.half.x * 2.0,
                        state.player.half.y * 2.0,
                        1.3,
                        YELLOW,
                    );
                }
            }
            PawnKey::Npc => {
                let mut facing = CharacterFacing::Front;
                let mut facing_left = false;
                let mut is_walking = state.npc.is_walking;

                if hint.force_face_partner
                    && let Some(partner) = hint.partner
                    && let Some(target_pos) = ui_pawns::pawn_world_pos(state, partner)
                {
                    let dir = target_pos - state.npc.pos;
                    facing = select_character_facing(dir, facing);
                    facing_left = dir.x < 0.0;
                }
                if hint.force_idle {
                    is_walking = false;
                }

                draw_character(
                    &state.npc_character,
                    CharacterRenderParams {
                        center: state.npc.pos,
                        scale: 0.96,
                        facing,
                        facing_left,
                        is_walking,
                        walk_cycle: state.npc.walk_cycle,
                        gesture,
                        time,
                        debug: false,
                    },
                );

                if state.debug {
                    draw_rectangle_lines(
                        state.npc.pos.x - state.npc.half.x,
                        state.npc.pos.y - state.npc.half.y,
                        state.npc.half.x * 2.0,
                        state.npc.half.y * 2.0,
                        1.3,
                        ORANGE,
                    );
                }
            }
            PawnKey::SimWorker => {
                let mut facing = CharacterFacing::Front;
                let mut facing_left = false;
                if hint.force_face_partner
                    && let Some(partner) = hint.partner
                    && let Some(target_pos) = ui_pawns::pawn_world_pos(state, partner)
                {
                    let dir = target_pos - worker_pos;
                    facing = select_character_facing(dir, facing);
                    facing_left = dir.x < 0.0;
                }

                draw_character(
                    &state.sim_worker_character,
                    CharacterRenderParams {
                        center: worker_pos,
                        scale: 0.94,
                        facing,
                        facing_left,
                        is_walking: false,
                        walk_cycle: time * 2.0,
                        gesture,
                        time,
                        debug: false,
                    },
                );
            }
        }
    }

    ui_pawns::draw_selected_world_indicator(state);
    ui_pawns::draw_social_emotes(state, time);

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

    if state.debug || state.sim.build_mode_enabled() {
        draw_sim_agent_overlay(&state.sim, state.debug || state.sim.build_mode_enabled());
    }
    draw_lighting_region(&state.props, &state.palette, time, visible_bounds);
    begin_ui_pass();

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

    if state.show_character_inspector && state.pawn_ui.sheet_open.is_none() {
        draw_character_inspector_panel(state, time);
    }

    let hud_y0 = 18.0;

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
        let npc_hint = state.social_state.anim_hint(PawnKey::Npc);
        let npc_social = npc_hint.kind.map(|kind| kind.ui_label()).unwrap_or("idle");
        let info = format!(
            "Mode Jeu | Esc=menu | F10=editeur | F11=plein ecran\nF1: debug on/off | F2: inspector | F3: regenerate\nbar perso: clic=select/jump | double-clic ou bouton F=follow | bouton Comp=fiche\ncamera: ZQSD/WASD pan | molette zoom | C recentrer\nmouse: click-to-move sur la map | fleches: override manuel\nplayer pos(px)=({:.1},{:.1}) tile=({},{})\nmode={} walking={} frame={} facing={} facing_left={} walk_cycle={:.2}\ninput=({:.2},{:.2}) camera=({:.1},{:.1}) zoom={:.2} fps={}\nplayer_path_nodes={} next_wp={} target_tile={}\nnpc pos=({:.1},{:.1}) walking={} hold={:.2}s social={} npc_path_nodes={} npc_target={}\nwall_mask@tile={:04b}\nmutation_permille={} visual={}\n{}",
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
            state.npc.hold_timer,
            npc_social,
            state.npc.auto.path_world.len(),
            npc_target_tile,
            mask,
            state.character_catalog.mutation_permille(),
            player_visual,
            state.sim.debug_hud(),
        );
        draw_text(&info, 12.0, hud_y0, 18.0, WHITE);
    } else {
        draw_text(
            "Mode Jeu | Esc=menu | F10=editeur | F11=plein ecran",
            12.0,
            hud_y0 + 4.0,
            22.0,
            Color::from_rgba(220, 235, 242, 255),
        );
        draw_text(
            "HUD bas: persos/build/fiche/minimap | clic minimap: bouger camera | F7 build | F3 debug",
            12.0,
            hud_y0 + 28.0,
            18.0,
            Color::from_rgba(200, 224, 236, 255),
        );
    }

    ui_hud::draw_hud(state, &hud_layout, mouse, map_view_rect, &world_camera, time);

    PlayAction::None
}

Ce que tu obtiens en jeu 🎮
• En bas à gauche : cartes persos (clic = sélection + jump caméra, double-clic = follow, bouton “F” = follow toggle).
• Centre gauche : build trié (Blocs / Zones / Outils) + statut clair (et la barre capte les clics).
• Centre droit : fiche perso (besoins/états + activité du worker) + historique “digest” (filtré, non-spam).
• Tout à droite : minimap + rectangle de caméra + clic pour déplacer la caméra.
• Bandeau en haut du HUD : temps sim + “Tresorerie / Ventes / Cadence / Service / Resultat” + vitesses (pause/1x/2x/4x). Le symbole € est dessiné, donc fiable même si la font ne l’a pas.

Si tu veux ensuite que je pousse encore le “feeling RimWorld” (notifications/alertes, tooltips, raccourcis build par hotbar, bandeau d’ordres, etc.), dis-moi ce que tu veux prioriser et je te propose une version 2 bien “gameplay-first” 🙂