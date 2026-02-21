use super::*;

// ---------------------------------------------
// RimWorld-like pawn bar + skill/character sheet
// ---------------------------------------------
//
// Design goals:
// - Zero external dependencies (macroquad only).
// - Robust input handling: UI consumes clicks/wheel so gameplay doesn't mis-trigger.
// - Data model is future-proof: metrics are enums + fixed arrays (fast, cache-friendly).
// - Minimal per-frame allocations.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PawnKey {
    Player,
    Npc,
    SimWorker,
}

impl PawnKey {
    pub fn short_label(self) -> &'static str {
        match self {
            Self::Player => "Joueur",
            Self::Npc => "Visiteur",
            Self::SimWorker => "Employe",
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeedBar {
    Manger = 0,
    Boire = 1,
    Dormir = 2, // Energie
    Toilettes = 3,
    Hygiene = 4,
    Divertissement = 5,
    Social = 6,
    Confort = 7,
    Calme = 8,
    Douleur = 9,
}

impl NeedBar {
    pub const COUNT: usize = 10;
    pub const ALL: [NeedBar; NeedBar::COUNT] = [
        NeedBar::Manger,
        NeedBar::Boire,
        NeedBar::Dormir,
        NeedBar::Toilettes,
        NeedBar::Hygiene,
        NeedBar::Divertissement,
        NeedBar::Social,
        NeedBar::Confort,
        NeedBar::Calme,
        NeedBar::Douleur,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Manger => "Manger",
            Self::Boire => "Boire",
            Self::Dormir => "Energie",
            Self::Toilettes => "Toilettes",
            Self::Hygiene => "Hygiene",
            Self::Divertissement => "Divertissement",
            Self::Social => "Social",
            Self::Confort => "Confort",
            Self::Calme => "Calme",
            Self::Douleur => "Douleur",
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillBar {
    Mecanique = 0,
    Electricite = 1,
    Dexterite = 2,
    Qualite = 3,
    Force = 4,
    Logistique = 5,
    Intelligence = 6,
    Planification = 7,
    Sociabilite = 8,
    Management = 9,
    Securite = 10,
    Nettoyage = 11,
    Diagnostic = 12,
}

impl SkillBar {
    pub const COUNT: usize = 13;
    pub const ALL: [SkillBar; SkillBar::COUNT] = [
        SkillBar::Mecanique,
        SkillBar::Electricite,
        SkillBar::Dexterite,
        SkillBar::Qualite,
        SkillBar::Force,
        SkillBar::Logistique,
        SkillBar::Intelligence,
        SkillBar::Planification,
        SkillBar::Sociabilite,
        SkillBar::Management,
        SkillBar::Securite,
        SkillBar::Nettoyage,
        SkillBar::Diagnostic,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Mecanique => "Mecanique",
            Self::Electricite => "Electricite",
            Self::Dexterite => "Dexterite",
            Self::Qualite => "Qualite",
            Self::Force => "Force",
            Self::Logistique => "Logistique",
            Self::Intelligence => "Intelligence",
            Self::Planification => "Planification",
            Self::Sociabilite => "Sociabilite",
            Self::Management => "Management",
            Self::Securite => "Securite",
            Self::Nettoyage => "Nettoyage",
            Self::Diagnostic => "Diagnostic",
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraitBar {
    Motivation = 0,
    Discipline = 1,
    Fiabilite = 2,
    Patience = 3,
    Courage = 4,
    Empathie = 5,
}

impl TraitBar {
    pub const COUNT: usize = 6;
    pub const ALL: [TraitBar; TraitBar::COUNT] = [
        TraitBar::Motivation,
        TraitBar::Discipline,
        TraitBar::Fiabilite,
        TraitBar::Patience,
        TraitBar::Courage,
        TraitBar::Empathie,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Motivation => "Motivation",
            Self::Discipline => "Discipline",
            Self::Fiabilite => "Fiabilite",
            Self::Patience => "Patience",
            Self::Courage => "Courage",
            Self::Empathie => "Empathie",
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SynthBar {
    Sante = 0,
    Fatigue = 1,
    Moral = 2,
}

impl SynthBar {
    pub const COUNT: usize = 3;
    pub const ALL: [SynthBar; SynthBar::COUNT] =
        [SynthBar::Sante, SynthBar::Fatigue, SynthBar::Moral];

    pub fn label(self) -> &'static str {
        match self {
            Self::Sante => "Sante",
            Self::Fatigue => "Fatigue",
            Self::Moral => "Moral",
        }
    }
}

#[derive(Clone, Debug)]
pub struct PawnMetrics {
    pub needs: [u8; NeedBar::COUNT],
    pub skills: [u8; SkillBar::COUNT],
    pub traits: [u8; TraitBar::COUNT],
    pub synth: [u8; SynthBar::COUNT],
}

impl PawnMetrics {
    pub fn seeded(seed: u64) -> Self {
        // Deterministic, fast, no allocations.
        // Distributions are intentionally biased toward "competent but imperfect".
        let mut out = Self {
            needs: [80; NeedBar::COUNT],
            skills: [50; SkillBar::COUNT],
            traits: [55; TraitBar::COUNT],
            synth: [75; SynthBar::COUNT],
        };

        // Needs: mostly high.
        for (i, slot) in out.needs.iter_mut().enumerate() {
            *slot = rand_range_u8(seed ^ 0x00A1_1CEE_5501_u64, i as u32, 62, 96);
        }

        // Skills: wide range.
        for (i, slot) in out.skills.iter_mut().enumerate() {
            *slot = rand_range_u8(seed ^ 0x00BA_DA55_1502_u64, i as u32, 18, 92);
        }

        // Traits: centered.
        for (i, slot) in out.traits.iter_mut().enumerate() {
            *slot = rand_range_u8(seed ^ 0x00C0_DECA_FE03_u64, i as u32, 30, 88);
        }

        // Synthesis: stable.
        out.synth[SynthBar::Sante as usize] =
            rand_range_u8(seed ^ 0x00DE_ADBE_EF04_u64, 0, 70, 100);
        out.synth[SynthBar::Fatigue as usize] =
            rand_range_u8(seed ^ 0x00DE_ADBE_EF04_u64, 1, 55, 100);
        out.synth[SynthBar::Moral as usize] = rand_range_u8(seed ^ 0x00DE_ADBE_EF04_u64, 2, 55, 98);

        out
    }

    pub fn clamp_all(&mut self) {
        for v in &mut self.needs {
            *v = (*v).min(100);
        }
        for v in &mut self.skills {
            *v = (*v).min(100);
        }
        for v in &mut self.traits {
            *v = (*v).min(100);
        }
        for v in &mut self.synth {
            *v = (*v).min(100);
        }
    }
}

#[derive(Clone, Debug)]
pub struct PawnCard {
    pub key: PawnKey,
    pub name: String,
    pub role: String,
    pub metrics: PawnMetrics,
}

#[derive(Clone, Debug)]
pub struct PawnsUiState {
    pub selected: Option<PawnKey>,
    pub follow: Option<PawnKey>,
    pub sheet_open: Option<PawnKey>,
    pub bar_scroll_x: f32,
    pub last_click_time: f32,
    pub last_click_pawn: Option<PawnKey>,
}

impl Default for PawnsUiState {
    fn default() -> Self {
        Self {
            selected: None,
            follow: None,
            sheet_open: None,
            bar_scroll_x: 0.0,
            last_click_time: -999.0,
            last_click_pawn: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct PawnUiLayout {
    pub top_bar: Rect,
    pub slots: Vec<PawnSlotLayout>,
    pub sheet_rect: Option<Rect>,
    pub sheet_close: Option<Rect>,
}

#[derive(Clone, Debug)]
pub struct PawnSlotLayout {
    pub key: PawnKey,
    pub slot_rect: Rect,
    pub follow_rect: Rect,
    pub sheet_rect: Rect,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct PawnUiInputResult {
    pub mouse_over_ui: bool,
    pub consumed_click: bool,
    pub consumed_wheel: bool,
}

pub fn sync_dynamic_pawn_metrics(state: &mut GameState) {
    // Keep dynamic metrics in sync with simulation values.
    // Convention: in UI, higher value means "better".
    let fatigue = state.sim.primary_agent_fatigue().clamp(0.0, 100.0);
    let stress = state.sim.primary_agent_stress().clamp(0.0, 100.0);
    let energie = (100.0 - fatigue) as u8;
    let calme = (100.0 - stress) as u8;

    if let Some(worker) = state.pawns.iter_mut().find(|p| p.key == PawnKey::SimWorker) {
        worker.metrics.needs[NeedBar::Dormir as usize] = energie;
        worker.metrics.needs[NeedBar::Calme as usize] = calme;
        worker.metrics.synth[SynthBar::Fatigue as usize] = energie;
        // Moral slightly impacted by stress.
        worker.metrics.synth[SynthBar::Moral as usize] = ((calme as u32 + 40) / 2).min(100) as u8;
        worker.metrics.clamp_all();
    }
}

pub fn pawn_world_pos(state: &GameState, key: PawnKey) -> Option<Vec2> {
    match key {
        PawnKey::Player => Some(state.player.pos),
        PawnKey::Npc => Some(state.npc.pos),
        PawnKey::SimWorker => Some(tile_center(state.sim.primary_agent_tile())),
    }
}

pub fn pawn_visual_record(state: &GameState, key: PawnKey) -> Option<&CharacterRecord> {
    match key {
        PawnKey::Player => state.lineage.get(state.player_lineage_index),
        PawnKey::Npc => Some(&state.npc_character),
        PawnKey::SimWorker => Some(&state.sim_worker_character),
    }
}

pub fn build_pawn_ui_layout(state: &GameState) -> PawnUiLayout {
    let sw = screen_width();
    let sh = screen_height();

    let margin = 10.0;
    let bar_h = 74.0;
    let bar_w = (sw - margin * 2.0).max(240.0);
    let bar_rect = Rect::new(margin, margin, bar_w, bar_h);

    // Slot sizing: fixed-ish but scales down a bit on smaller windows.
    let scale = ((sw / 1600.0).min(sh / 900.0)).clamp(0.78, 1.10);
    let slot_h = (60.0 * scale).clamp(48.0, 64.0);
    let slot_w = (170.0 * scale).clamp(140.0, 190.0);
    let gap = (7.0 * scale).clamp(5.0, 9.0);

    let mut layout = PawnUiLayout {
        top_bar: bar_rect,
        slots: Vec::with_capacity(state.pawns.len()),
        sheet_rect: None,
        sheet_close: None,
    };

    let content_w =
        (state.pawns.len() as f32) * slot_w + ((state.pawns.len().saturating_sub(1)) as f32) * gap;
    let view_w = (bar_rect.w - 16.0).max(1.0);
    let max_scroll = (content_w - view_w).max(0.0);
    let scroll_x = state.pawn_ui.bar_scroll_x.clamp(0.0, max_scroll);

    let base_x = bar_rect.x + 8.0 - scroll_x;
    let base_y = bar_rect.y + (bar_rect.h - slot_h) * 0.5;

    for (i, pawn) in state.pawns.iter().enumerate() {
        let x = base_x + i as f32 * (slot_w + gap);
        let y = base_y;
        let slot_rect = Rect::new(x, y, slot_w, slot_h);

        // Small action buttons: top-right (follow) + bottom-right (sheet)
        let btn_size = (18.0 * scale).clamp(16.0, 22.0);
        let pad = (6.0 * scale).clamp(5.0, 8.0);
        let follow_rect = Rect::new(
            slot_rect.x + slot_rect.w - btn_size - pad,
            slot_rect.y + pad,
            btn_size,
            btn_size,
        );
        let sheet_rect = Rect::new(
            slot_rect.x + slot_rect.w - (btn_size * 2.7).max(46.0) - pad,
            slot_rect.y + slot_rect.h - btn_size - pad,
            (btn_size * 2.7).max(46.0),
            btn_size,
        );

        layout.slots.push(PawnSlotLayout {
            key: pawn.key,
            slot_rect,
            follow_rect,
            sheet_rect,
        });
    }

    // Sheet panel layout (right side), only if opened.
    if state.pawn_ui.sheet_open.is_some() {
        let panel_w = (sw * 0.32).clamp(420.0, 560.0);
        let panel_h = (sh - (bar_rect.y + bar_rect.h) - margin * 2.0).clamp(340.0, 780.0);
        let panel_x = sw - panel_w - margin;
        let panel_y = bar_rect.y + bar_rect.h + margin;
        let panel_rect = Rect::new(panel_x, panel_y, panel_w, panel_h);
        let close_rect = Rect::new(panel_x + panel_w - 30.0, panel_y + 10.0, 20.0, 20.0);
        layout.sheet_rect = Some(panel_rect);
        layout.sheet_close = Some(close_rect);
    }

    layout
}

pub fn process_pawn_ui_input(
    state: &mut GameState,
    layout: &PawnUiLayout,
    mouse: Vec2,
    left_pressed: bool,
    wheel_y: f32,
    time_now: f32,
) -> PawnUiInputResult {
    let mut out = PawnUiInputResult::default();

    let over_top = point_in_rect(mouse, layout.top_bar);
    let over_sheet = layout
        .sheet_rect
        .map(|r| point_in_rect(mouse, r))
        .unwrap_or(false);
    let over_close = layout
        .sheet_close
        .map(|r| point_in_rect(mouse, r))
        .unwrap_or(false);
    out.mouse_over_ui = over_top || over_sheet || over_close;

    // Wheel: scroll pawn bar when hovering it.
    if over_top && wheel_y.abs() > f32::EPSILON {
        let pawns = state.pawns.len();
        if pawns > 0 {
            let scale = ((screen_width() / 1600.0).min(screen_height() / 900.0)).clamp(0.78, 1.10);
            let slot_w = (170.0 * scale).clamp(140.0, 190.0);
            let gap = (7.0 * scale).clamp(5.0, 9.0);
            let content_w = pawns as f32 * slot_w + (pawns.saturating_sub(1) as f32) * gap;
            let view_w = (layout.top_bar.w - 16.0).max(1.0);
            let max_scroll = (content_w - view_w).max(0.0);
            state.pawn_ui.bar_scroll_x =
                (state.pawn_ui.bar_scroll_x - wheel_y * 42.0).clamp(0.0, max_scroll);
            out.consumed_wheel = true;
        }
    }

    if !left_pressed {
        return out;
    }

    // Modal-like behavior: clicking the close button or outside closes the sheet.
    if state.pawn_ui.sheet_open.is_some() {
        if over_close {
            state.pawn_ui.sheet_open = None;
            out.consumed_click = true;
            return out;
        }

        if let Some(panel) = layout.sheet_rect {
            let click_inside_panel = point_in_rect(mouse, panel);
            let click_inside_top = point_in_rect(mouse, layout.top_bar);
            if !click_inside_panel && !click_inside_top {
                state.pawn_ui.sheet_open = None;
                out.consumed_click = true;
                return out;
            }
        }
    }

    // Button precedence: follow -> sheet -> slot.
    for slot in &layout.slots {
        if point_in_rect(mouse, slot.follow_rect) {
            // Toggle follow for this pawn.
            state.pawn_ui.selected = Some(slot.key);
            state.pawn_ui.follow = if state.pawn_ui.follow == Some(slot.key) {
                None
            } else {
                Some(slot.key)
            };
            // Recenter immediately when enabling follow.
            if state.pawn_ui.follow == Some(slot.key)
                && let Some(pos) = pawn_world_pos(state, slot.key)
            {
                state.camera_center = pos;
            }
            out.consumed_click = true;
            return out;
        }
        if point_in_rect(mouse, slot.sheet_rect) {
            state.pawn_ui.selected = Some(slot.key);
            state.pawn_ui.sheet_open = if state.pawn_ui.sheet_open == Some(slot.key) {
                None
            } else {
                Some(slot.key)
            };
            out.consumed_click = true;
            return out;
        }
        if point_in_rect(mouse, slot.slot_rect) {
            // Select pawn and jump camera once.
            state.pawn_ui.selected = Some(slot.key);
            if let Some(pos) = pawn_world_pos(state, slot.key) {
                state.camera_center = pos;
            }

            // Double click toggles follow.
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

            out.consumed_click = true;
            return out;
        }
    }

    out
}

pub fn draw_selected_world_indicator(state: &GameState) {
    let Some(sel) = state.pawn_ui.selected else {
        return;
    };
    let Some(pos) = pawn_world_pos(state, sel) else {
        return;
    };

    // World-space ring, readable on any floor.
    let outer = Color::from_rgba(120, 220, 255, 245);
    let inner = Color::from_rgba(255, 255, 255, 180);
    draw_circle_lines(pos.x, pos.y, 16.0, 2.4, outer);
    draw_circle_lines(pos.x, pos.y, 13.0, 1.2, inner);
}

pub fn draw_pawn_ui(state: &GameState, layout: &PawnUiLayout, mouse: Vec2, time: f32) {
    // Top bar background.
    draw_rectangle(
        layout.top_bar.x,
        layout.top_bar.y,
        layout.top_bar.w,
        layout.top_bar.h,
        Color::from_rgba(10, 13, 18, 210),
    );
    draw_rectangle_lines(
        layout.top_bar.x + 0.5,
        layout.top_bar.y + 0.5,
        layout.top_bar.w - 1.0,
        layout.top_bar.h - 1.0,
        1.5,
        Color::from_rgba(95, 135, 160, 220),
    );

    // Subtle top highlight.
    draw_rectangle(
        layout.top_bar.x + 1.5,
        layout.top_bar.y + 1.5,
        (layout.top_bar.w - 3.0).max(1.0),
        (layout.top_bar.h * 0.42).max(1.0),
        Color::from_rgba(180, 220, 244, 26),
    );

    // Slots.
    for slot in &layout.slots {
        let selected = state.pawn_ui.selected == Some(slot.key);
        let following = state.pawn_ui.follow == Some(slot.key);
        let hovered = point_in_rect(mouse, slot.slot_rect);

        let base = if following {
            rgba(210, 150, 82, 235)
        } else if selected {
            rgba(98, 152, 188, 236)
        } else {
            rgba(54, 74, 96, 230)
        };
        let border = if hovered || selected || following {
            rgba(170, 220, 247, 240)
        } else {
            rgba(110, 150, 180, 220)
        };
        draw_rectangle(
            slot.slot_rect.x,
            slot.slot_rect.y,
            slot.slot_rect.w,
            slot.slot_rect.h,
            base,
        );
        draw_rectangle_lines(
            slot.slot_rect.x,
            slot.slot_rect.y,
            slot.slot_rect.w,
            slot.slot_rect.h,
            2.0,
            border,
        );
        if hovered || selected || following {
            draw_rectangle_lines(
                slot.slot_rect.x + 1.0,
                slot.slot_rect.y + 1.0,
                (slot.slot_rect.w - 2.0).max(1.0),
                (slot.slot_rect.h - 2.0).max(1.0),
                1.0,
                with_alpha(WHITE, 0.22),
            );
        }

        // Portrait.
        let portrait_center = vec2(
            slot.slot_rect.x + 26.0,
            slot.slot_rect.y + slot.slot_rect.h * 0.5,
        );
        if let Some(record) = pawn_visual_record(state, slot.key) {
            draw_character(
                record,
                CharacterRenderParams {
                    center: portrait_center,
                    scale: 0.70,
                    facing: CharacterFacing::Front,
                    facing_left: false,
                    is_walking: false,
                    walk_cycle: time * 2.0,
                    time,
                    debug: false,
                },
            );
        } else {
            draw_circle(
                portrait_center.x,
                portrait_center.y,
                12.0,
                Color::from_rgba(220, 240, 255, 160),
            );
        }

        // Name and small bars.
        let pawn_name = state
            .pawns
            .iter()
            .find(|p| p.key == slot.key)
            .map(|p| p.name.as_str())
            .unwrap_or(slot.key.short_label());

        let text_x = slot.slot_rect.x + 52.0;
        let text_y = slot.slot_rect.y + 18.0;
        draw_text_shadowed(
            pawn_name,
            text_x,
            text_y,
            18.0,
            Color::from_rgba(244, 252, 255, 255),
        );

        // Quick status bars: Energie + Calme.
        if let Some(pawn) = state.pawns.iter().find(|p| p.key == slot.key) {
            let energie = pawn.metrics.needs[NeedBar::Dormir as usize];
            let calme = pawn.metrics.needs[NeedBar::Calme as usize];
            let bar_w = (slot.slot_rect.w - 52.0 - 42.0).max(42.0);
            let bar_h = 6.0;
            let bar_x = text_x;
            let bar_y = slot.slot_rect.y + slot.slot_rect.h - 18.0;
            draw_tiny_bar(bar_x, bar_y, bar_w, bar_h, energie);
            draw_tiny_bar(bar_x, bar_y - 9.0, bar_w, bar_h, calme);
        }

        // Follow button.
        let follow_hover = point_in_rect(mouse, slot.follow_rect);
        draw_small_button(
            slot.follow_rect,
            if following { "F" } else { "f" },
            follow_hover,
            following,
        );

        // Skills/Sheet button.
        let sheet_active = state.pawn_ui.sheet_open == Some(slot.key);
        let sheet_hover = point_in_rect(mouse, slot.sheet_rect);
        draw_small_wide_button(slot.sheet_rect, "Comp", sheet_hover, sheet_active);
    }

    // Skills sheet.
    if let Some(open) = state.pawn_ui.sheet_open
        && let Some(panel) = layout.sheet_rect
    {
        draw_pawn_sheet(state, open, panel, layout.sheet_close, mouse, time);
    }
}

fn draw_pawn_sheet(
    state: &GameState,
    key: PawnKey,
    panel: Rect,
    close_rect: Option<Rect>,
    mouse: Vec2,
    time: f32,
) {
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::from_rgba(10, 13, 18, 232),
    );
    draw_rectangle_lines(
        panel.x + 0.5,
        panel.y + 0.5,
        panel.w - 1.0,
        panel.h - 1.0,
        2.0,
        Color::from_rgba(130, 175, 200, 225),
    );
    draw_rectangle(
        panel.x + 2.0,
        panel.y + 2.0,
        (panel.w - 4.0).max(1.0),
        (panel.h * 0.12).max(1.0),
        Color::from_rgba(180, 220, 244, 18),
    );

    if let Some(close) = close_rect {
        let hovered = point_in_rect(mouse, close);
        draw_small_button(close, "X", hovered, false);
    }

    let pawn = state.pawns.iter().find(|p| p.key == key);
    let name = pawn.map(|p| p.name.as_str()).unwrap_or(key.short_label());
    let role = pawn.map(|p| p.role.as_str()).unwrap_or("-");

    // Header portrait.
    let portrait_center = vec2(panel.x + 46.0, panel.y + 52.0);
    if let Some(record) = pawn_visual_record(state, key) {
        draw_character(
            record,
            CharacterRenderParams {
                center: portrait_center,
                scale: 1.05,
                facing: CharacterFacing::Front,
                facing_left: false,
                is_walking: false,
                walk_cycle: time * 2.0,
                time,
                debug: false,
            },
        );
    }

    draw_text_shadowed(
        name,
        panel.x + 90.0,
        panel.y + 40.0,
        26.0,
        Color::from_rgba(244, 252, 255, 255),
    );
    draw_text_shadowed(
        role,
        panel.x + 90.0,
        panel.y + 64.0,
        18.0,
        Color::from_rgba(182, 210, 228, 255),
    );

    draw_line(
        panel.x + 10.0,
        panel.y + 92.0,
        panel.x + panel.w - 10.0,
        panel.y + 92.0,
        1.0,
        Color::from_rgba(110, 150, 180, 120),
    );

    let Some(pawn) = pawn else {
        return;
    };

    // Layout: 2 columns
    let inner_x = panel.x + 14.0;
    let inner_y = panel.y + 104.0;
    let inner_w = panel.w - 28.0;
    let col_gap = 16.0;
    let col_w = (inner_w - col_gap) * 0.5;
    let left_x = inner_x;
    let right_x = inner_x + col_w + col_gap;
    let mut y_left = inner_y;
    let mut y_right = inner_y;

    // Left column: Besoins + Etats
    y_left = draw_group_title(left_x, y_left, col_w, "BESOINS");
    for bar in NeedBar::ALL {
        y_left = draw_labeled_bar(
            left_x,
            y_left,
            col_w,
            bar.label(),
            pawn.metrics.needs[bar as usize],
        );
    }
    y_left += 8.0;
    y_left = draw_group_title(left_x, y_left, col_w, "ETATS");
    for bar in SynthBar::ALL {
        y_left = draw_labeled_bar(
            left_x,
            y_left,
            col_w,
            bar.label(),
            pawn.metrics.synth[bar as usize],
        );
    }

    // Right column: Competences + Traits
    y_right = draw_group_title(right_x, y_right, col_w, "COMPETENCES");
    for bar in SkillBar::ALL {
        y_right = draw_labeled_bar(
            right_x,
            y_right,
            col_w,
            bar.label(),
            pawn.metrics.skills[bar as usize],
        );
    }
    y_right += 8.0;
    y_right = draw_group_title(right_x, y_right, col_w, "TRAITS");
    for bar in TraitBar::ALL {
        y_right = draw_labeled_bar(
            right_x,
            y_right,
            col_w,
            bar.label(),
            pawn.metrics.traits[bar as usize],
        );
    }
}

fn draw_group_title(x: f32, y: f32, w: f32, title: &str) -> f32 {
    let h = 22.0;
    draw_rectangle(x, y, w, h, Color::from_rgba(40, 60, 78, 200));
    draw_rectangle_lines(x, y, w, h, 1.0, Color::from_rgba(110, 150, 180, 200));
    draw_text_shadowed(
        title,
        x + 8.0,
        y + 16.0,
        16.0,
        Color::from_rgba(220, 240, 255, 255),
    );
    y + h + 8.0
}

fn draw_labeled_bar(x: f32, y: f32, w: f32, label: &str, value: u8) -> f32 {
    let row_h = 18.0;
    let label_w = (w * 0.44).clamp(110.0, 160.0);
    let bar_w = (w - label_w - 10.0).max(52.0);
    draw_text_shadowed(
        label,
        x,
        y + 13.0,
        14.0,
        Color::from_rgba(200, 224, 236, 255),
    );
    let bar_x = x + label_w;
    let bar_y = y + 4.0;
    draw_progress_bar(bar_x, bar_y, bar_w, 10.0, value);
    y + row_h
}

fn draw_progress_bar(x: f32, y: f32, w: f32, h: f32, value: u8) {
    let v = (value as f32 / 100.0).clamp(0.0, 1.0);
    let bg = Color::from_rgba(18, 24, 32, 220);
    draw_rectangle(x, y, w, h, bg);
    let fill_w = (w * v).clamp(0.0, w);
    let fill = bar_color(v);
    draw_rectangle(x, y, fill_w, h, fill);
    draw_rectangle_lines(x, y, w, h, 1.0, Color::from_rgba(120, 170, 200, 200));
}

fn draw_tiny_bar(x: f32, y: f32, w: f32, h: f32, value: u8) {
    let v = (value as f32 / 100.0).clamp(0.0, 1.0);
    draw_rectangle(x, y, w, h, Color::from_rgba(16, 22, 30, 200));
    draw_rectangle(x, y, w * v, h, bar_color(v));
    draw_rectangle_lines(x, y, w, h, 1.0, Color::from_rgba(130, 175, 200, 175));
}

fn bar_color(v01: f32) -> Color {
    // Smooth red->yellow->green.
    let t = v01.clamp(0.0, 1.0);
    let red = Color::from_rgba(236, 92, 72, 245);
    let yellow = Color::from_rgba(244, 204, 96, 245);
    let green = Color::from_rgba(86, 210, 132, 245);
    if t < 0.5 {
        color_lerp(red, yellow, t / 0.5)
    } else {
        color_lerp(yellow, green, (t - 0.5) / 0.5)
    }
}

fn draw_text_shadowed(text: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text(text, x + 1.0, y + 1.0, size, with_alpha(BLACK, 0.82));
    draw_text(text, x, y, size, color);
}

fn draw_small_button(rect: Rect, label: &str, hovered: bool, active: bool) {
    let base = if active {
        rgba(210, 150, 82, 242)
    } else if hovered {
        rgba(98, 152, 188, 240)
    } else {
        rgba(68, 100, 128, 236)
    };
    let border = if active {
        rgba(252, 208, 138, 252)
    } else if hovered {
        rgba(170, 220, 247, 240)
    } else {
        rgba(120, 171, 199, 224)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, base);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, border);

    let font_size = (rect.h * 0.82).clamp(10.0, 18.0);
    let dims = measure_text(label, None, font_size as u16, 1.0);
    let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
    let ty = rect.y + rect.h * 0.5 + dims.height * 0.34;
    draw_text_shadowed(
        label,
        tx,
        ty,
        font_size,
        Color::from_rgba(244, 252, 255, 255),
    );
}

fn draw_small_wide_button(rect: Rect, label: &str, hovered: bool, active: bool) {
    let base = if active {
        rgba(210, 150, 82, 242)
    } else if hovered {
        rgba(98, 152, 188, 240)
    } else {
        rgba(68, 100, 128, 236)
    };
    let border = if active {
        rgba(252, 208, 138, 252)
    } else if hovered {
        rgba(170, 220, 247, 240)
    } else {
        rgba(120, 171, 199, 224)
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, base);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, border);
    let font_size = (rect.h * 0.70).clamp(10.0, 16.0);
    let dims = measure_text(label, None, font_size as u16, 1.0);
    let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
    let ty = rect.y + rect.h * 0.5 + dims.height * 0.34;
    draw_text_shadowed(
        label,
        tx,
        ty,
        font_size,
        Color::from_rgba(244, 252, 255, 255),
    );
}

fn rand_range_u8(seed: u64, idx: u32, lo: u8, hi: u8) -> u8 {
    if lo >= hi {
        return lo;
    }
    let mut x = seed ^ ((idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    let span = (hi as u32).saturating_sub(lo as u32);
    let v = (x as u32) % span;
    (lo as u32 + v).min(100) as u8
}
