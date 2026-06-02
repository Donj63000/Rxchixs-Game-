use super::*;
use crate::gestion::{EmployeeRole, SimCommand};
use crate::rendu::theme::{feedback_theme, ui_panel_fill, ui_panel_header_fill, ui_theme};
use crate::sim::{BlockKind, BuildFloorKind, ZoneKind};
use std::cell::RefCell;

thread_local! {
    static INITIAL_RAW_MATERIAL_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
}

const MINIMAP_CACHE_STRIDE: i32 = 2;

pub(crate) fn set_initial_raw_material_texture(texture: Option<Texture2D>) {
    INITIAL_RAW_MATERIAL_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Linear);
        }
        *slot.borrow_mut() = prepared;
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildToolAction {
    ToggleBuildMode,
    ToggleZoneOverlay,
    ToggleZonePaint,
    ToggleSalesManager,
    CancelMoveSource,
    SaveLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildMenuSelection {
    Block(BlockKind),
    Zone(ZoneKind),
    Floor(BuildFloorKind),
    Tool(BuildToolAction),
}

#[derive(Clone, Copy, Debug)]
struct BuildMenuEntry {
    selection: BuildMenuSelection,
    label: &'static str,
    description: &'static str,
    hint: &'static str,
}

const BUILD_MENU_BLOCKS: [BuildMenuEntry; 14] = [
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::InputHopper),
        label: "Entree ligne",
        description: "Tremie 8x3 avec tapis stockeur bleu pour alimenter la ligne.",
        hint: "Ligne",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::Conveyor),
        label: "Convoyeur",
        description: "Module 1x1 orientable pour transferer le produit.",
        hint: "Ligne",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::FluidityTank),
        label: "Bac fluidite",
        description: "Bac 5x5 brasse a l'eau pour laver le produit.",
        hint: "Lavage",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::Cutter),
        label: "Coupeuse",
        description: "Bloc inox avec lames circulaires de coupe fine.",
        hint: "Coupe",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::DistributorBelt),
        label: "Tapis repartiteur",
        description: "Bras 7x1 oscillant pour repartition avant le four.",
        hint: "Four",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::DryerOven),
        label: "Four deshydratation",
        description: "Unite 20x10 avec tunnel thermique et tapis traversant.",
        hint: "Four",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::OvenExitConveyor),
        label: "Tapis sortie four",
        description: "Recuperation 7x1 en sortie de four vers broyage.",
        hint: "Flux",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::Flaker),
        label: "Floconneuse",
        description: "Cylindre de concassage des lanieres deshydratees.",
        hint: "Floc",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::SuctionPipe),
        label: "Tuyau aspiration",
        description: "Reseau d'aspiration modulaire adaptatif vers Sortex.",
        hint: "Pipe",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::Sortex),
        label: "Sortex",
        description: "Tri optique qui separe en flux bleu et rouge.",
        hint: "Tri",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::BlueBagChute),
        label: "Descente sac bleu",
        description: "Remplissage auto des sacs bleus (bon produit).",
        hint: "Sortie",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::RedBagChute),
        label: "Descente sac rouge",
        description: "Remplissage auto des sacs rouges (rejets).",
        hint: "Sortie",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::Buffer),
        label: "Rack palettes",
        description: "Rack vertical, niveaux RDC + N1..N5 pour palettes.",
        hint: "Stock",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Block(BlockKind::Seller),
        label: "Bureau vente",
        description: "Poste commercial requis dans la zone vente.",
        hint: "Vente",
    },
];

const BUILD_MENU_ZONES: [BuildMenuEntry; 4] = [
    BuildMenuEntry {
        selection: BuildMenuSelection::Zone(ZoneKind::Receiving),
        label: "Zone stockage",
        description: "Zone bleue de stockage (selection en rectangle).",
        hint: "Zone",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Zone(ZoneKind::Processing),
        label: "Zone de cassage",
        description: "Zone de production dediee au cassage.",
        hint: "Zone",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Zone(ZoneKind::Shipping),
        label: "Zone de dehy/finition",
        description: "Zone de production dediee a la dehy/finition.",
        hint: "Zone",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Zone(ZoneKind::Support),
        label: "Zone vente",
        description: "Zone vente activee avec bureau + responsable.",
        hint: "Zone",
    },
];

const BUILD_MENU_FLOORS: [BuildMenuEntry; 5] = [
    BuildMenuEntry {
        selection: BuildMenuSelection::Floor(BuildFloorKind::Standard),
        label: "Sol standard",
        description: "Sol usine polyvalent a cout reduit.",
        hint: "Sol",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Floor(BuildFloorKind::Metal),
        label: "Sol metal",
        description: "Sol industriel robuste, zone de trafic intense.",
        hint: "Sol",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Floor(BuildFloorKind::Bois),
        label: "Sol bois",
        description: "Sol de finition legere pour zones seches.",
        hint: "Sol",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Floor(BuildFloorKind::Mousse),
        label: "Sol mousse",
        description: "Sol technique amorti pour zones confort.",
        hint: "Sol",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Floor(BuildFloorKind::Sable),
        label: "Sol sable",
        description: "Sol brut economique pour zones exterieures.",
        hint: "Sol",
    },
];

const BUILD_MENU_TOOLS: [BuildMenuEntry; 6] = [
    BuildMenuEntry {
        selection: BuildMenuSelection::Tool(BuildToolAction::ToggleBuildMode),
        label: "Mode construction",
        description: "Active ou desactive les actions de construction.",
        hint: "F7",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Tool(BuildToolAction::ToggleZoneOverlay),
        label: "Surcouche zones",
        description: "Affiche visuellement les zones logiques.",
        hint: "F6",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Tool(BuildToolAction::ToggleZonePaint),
        label: "Peinture zones",
        description: "Definit des zones via rectangle (coin 1 -> coin 2).",
        hint: "V",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Tool(BuildToolAction::ToggleSalesManager),
        label: "Resp. ventes",
        description: "Assigner/retirer le responsable des ventes.",
        hint: "Etat",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Tool(BuildToolAction::CancelMoveSource),
        label: "Annuler deplacement",
        description: "Supprime la source de deplacement en attente.",
        hint: "M",
    },
    BuildMenuEntry {
        selection: BuildMenuSelection::Tool(BuildToolAction::SaveLayout),
        label: "Sauver layout",
        description: "Ecrit l'etat de l'usine dans le fichier layout.",
        hint: "F8",
    },
];
const PANEL_SCROLL_STEP: f32 = 34.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudBuildTab {
    Blocs,
    Zones,
    Sols,
    Outils,
}

impl HudBuildTab {
    fn label(self) -> &'static str {
        match self {
            HudBuildTab::Blocs => "Blocs",
            HudBuildTab::Zones => "Zones",
            HudBuildTab::Sols => "Sols",
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
            HudInfoTab::Fiche => "CARACTÉRISTIQUES",
            HudInfoTab::Historique => "HISTORIQUE",
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
    pub build_menu_open: bool,
    pub build_menu_page: usize,
    pub build_menu_selected: Option<BuildMenuSelection>,
    pub info_tab: HudInfoTab,
    pub sim_speed: SimSpeed,
    pub pawn_scroll_y: f32,
    pub info_scroll_y: f32,
    pub info_window_open: bool,
}

impl Default for HudUiState {
    fn default() -> Self {
        Self {
            build_tab: HudBuildTab::Blocs,
            build_menu_open: false,
            build_menu_page: 0,
            build_menu_selected: None,
            info_tab: HudInfoTab::Fiche,
            sim_speed: SimSpeed::X1,
            pawn_scroll_y: 0.0,
            info_scroll_y: 0.0,
            info_window_open: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HudLayout {
    pub bar_rect: Rect,
    pub top_strip_rect: Rect,
    pub footer_strip_rect: Rect,
    pub pawn_panel: Rect,
    pub build_panel: Rect,
    pub info_panel: Rect,
    pub telephone_panel: Rect,
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

    let top_strip_h = (58.0 * scale).clamp(52.0, 70.0);
    let top_strip_rect = Rect::new(0.0, 0.0, sw, top_strip_h);

    let bar_h = (220.0 * scale).clamp(185.0, 260.0);
    let bar_rect = Rect::new(0.0, (sh - bar_h).max(0.0), sw, bar_h);

    let outer_gap = (8.0 * scale).clamp(6.0, 10.0);
    let panel_gap = (8.0 * scale).clamp(6.0, 10.0);
    let footer_h = (50.0 * scale).clamp(42.0, 58.0);
    let footer_strip_rect = Rect::new(
        bar_rect.x + outer_gap,
        bar_rect.y + bar_rect.h - footer_h - outer_gap,
        (bar_rect.w - outer_gap * 2.0).max(1.0),
        footer_h,
    );

    let content_y = bar_rect.y + outer_gap;
    let content_h = (footer_strip_rect.y - content_y - outer_gap).max(1.0);

    let panels_w = (sw - outer_gap * 2.0 - panel_gap * 4.0).max(1.0);
    let (pawn_w, build_w, info_w, phone_w, minimap_w) = compute_bottom_panel_widths(panels_w);

    let mut x = bar_rect.x + outer_gap;
    let pawn_panel = Rect::new(x, content_y, pawn_w, content_h);
    x += pawn_panel.w + panel_gap;
    let build_panel = Rect::new(x, content_y, build_w, content_h);
    x += build_panel.w + panel_gap;
    let info_panel = Rect::new(x, content_y, info_w, content_h);
    x += info_panel.w + panel_gap;
    let telephone_panel = Rect::new(x, content_y, phone_w, content_h);
    x += telephone_panel.w + panel_gap;
    let minimap_panel = Rect::new(x, content_y, minimap_w, content_h);

    HudLayout {
        bar_rect,
        top_strip_rect,
        footer_strip_rect,
        pawn_panel,
        build_panel,
        info_panel,
        telephone_panel,
        minimap_panel,
    }
}

fn compute_bottom_panel_widths(sw: f32) -> (f32, f32, f32, f32, f32) {
    let sw = sw.max(1.0);
    let mut pawn_w = (sw * 0.22).clamp(72.0, 420.0);
    let mut phone_w = (sw * 0.11).clamp(54.0, 220.0);
    let mut minimap_w = (sw * 0.24).clamp(92.0, 430.0);
    let mut info_w = (sw * 0.24).clamp(92.0, 470.0);
    let min_build_w = (sw * 0.16).clamp(70.0, 340.0);

    let fixed_sum = pawn_w + info_w + phone_w + minimap_w;
    if fixed_sum + min_build_w > sw {
        let available_for_fixed = (sw - min_build_w).max(0.0);
        let scale = if fixed_sum > 0.0 {
            (available_for_fixed / fixed_sum).clamp(0.0, 1.0)
        } else {
            1.0
        };
        pawn_w *= scale;
        info_w *= scale;
        phone_w *= scale;
        minimap_w *= scale;
    }

    let build_w = (sw - pawn_w - info_w - phone_w - minimap_w).max(1.0);
    (pawn_w, build_w, info_w, phone_w, minimap_w)
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

    if state.hud_ui.build_menu_open {
        let menu_rect = build_menu_rect();
        let over_menu = point_in_rect(mouse, menu_rect);
        out.mouse_over_ui = true;

        if wheel_y.abs() > f32::EPSILON {
            out.consumed_wheel = true;
        }

        if left_click {
            let close_rect = build_menu_close_rect(menu_rect);
            if point_in_rect(mouse, close_rect) {
                state.hud_ui.build_menu_open = false;
                out.consumed_click = true;
                return out;
            }
            if over_menu {
                let _ = process_build_menu_input(state, mouse);
                out.consumed_click = true;
                return out;
            }
            state.hud_ui.build_menu_open = false;
            out.consumed_click = true;
            return out;
        }

        if right_click {
            state.hud_ui.build_menu_open = false;
            out.consumed_click = true;
            return out;
        }
    }

    let info_modal = if state.hud_ui.info_window_open {
        Some(info_window_rect())
    } else {
        None
    };

    if let Some(modal_rect) = info_modal {
        let over_modal = point_in_rect(mouse, modal_rect);
        out.mouse_over_ui = true;

        if wheel_y.abs() > f32::EPSILON && over_modal {
            let _ = process_info_panel_wheel(state, modal_rect, wheel_y);
            out.consumed_wheel = true;
        } else if wheel_y.abs() > f32::EPSILON {
            out.consumed_wheel = true;
        }

        if left_click {
            let close_rect = info_window_close_rect(modal_rect);
            if point_in_rect(mouse, close_rect) {
                state.hud_ui.info_window_open = false;
                out.consumed_click = true;
                return out;
            }
            if over_modal {
                let _ = process_info_panel_input(state, modal_rect, mouse);
                out.consumed_click = true;
                return out;
            }
            state.hud_ui.info_window_open = false;
            out.consumed_click = true;
            return out;
        }

        if right_click {
            if over_modal {
                state.hud_ui.info_window_open = false;
            }
            out.consumed_click = true;
            return out;
        }
    }

    let over_top = point_in_rect(mouse, layout.top_strip_rect);
    let over_bar = point_in_rect(mouse, layout.bar_rect);
    let over_phone =
        telephone::telephone_panel_contains_mouse(state, layout.telephone_panel, mouse);
    out.mouse_over_ui = out.mouse_over_ui || over_top;
    out.mouse_over_ui = out.mouse_over_ui || over_bar;
    out.mouse_over_ui = out.mouse_over_ui || over_phone;

    if wheel_y.abs() > f32::EPSILON {
        if point_in_rect(mouse, layout.pawn_panel) {
            let _ = process_pawn_panel_wheel(state, layout.pawn_panel, wheel_y);
            out.consumed_wheel = true;
        } else if over_bar {
            out.consumed_wheel = true;
        }
    }

    if left_click {
        if point_in_rect(mouse, layout.footer_strip_rect)
            && process_footer_strip_input(state, layout.footer_strip_rect, mouse)
        {
            out.consumed_click = true;
            return out;
        }
        if point_in_rect(mouse, layout.top_strip_rect) {
            out.consumed_click = true;
            return out;
        }
        if point_in_rect(mouse, layout.pawn_panel)
            && process_pawn_panel_input(state, layout.pawn_panel, mouse, time_now)
        {
            out.consumed_click = true;
            return out;
        }
        if point_in_rect(mouse, layout.build_panel)
            && process_build_panel_input(state, layout.build_panel, mouse)
        {
            out.consumed_click = true;
            return out;
        }
        if point_in_rect(mouse, layout.info_panel)
            && process_info_panel_quick_input(state, layout.info_panel, mouse)
        {
            out.consumed_click = true;
            return out;
        }
        if telephone::telephone_panel_contains_mouse(state, layout.telephone_panel, mouse)
            && telephone::process_telephone_panel_input(state, layout.telephone_panel, mouse)
        {
            out.consumed_click = true;
            return out;
        }
        if point_in_rect(mouse, layout.minimap_panel)
            && process_minimap_panel_input(state, layout.minimap_panel, mouse)
        {
            out.consumed_click = true;
            return out;
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
            || telephone::telephone_panel_contains_mouse(state, layout.telephone_panel, mouse)
            || point_in_rect(mouse, layout.top_strip_rect)
            || point_in_rect(mouse, layout.footer_strip_rect)
        {
            out.consumed_click = true;
            return out;
        }
    }

    out
}

pub fn draw_hud(
    state: &mut GameState,
    layout: &HudLayout,
    mouse: Vec2,
    map_view: Rect,
    world_camera: &Camera2D,
    time: f32,
) {
    begin_ui_pass();
    draw_top_strip(state, layout.top_strip_rect, mouse);
    draw_bar_background(layout.bar_rect, time);

    draw_pawn_panel(state, layout.pawn_panel, mouse, time);
    draw_build_panel(state, layout.build_panel, mouse);
    draw_info_panel(state, layout.info_panel, mouse);
    telephone::draw_telephone_panel(state, layout.telephone_panel, mouse, time);
    draw_minimap_panel(state, layout.minimap_panel, mouse, map_view, world_camera);
    draw_footer_strip(state, layout.footer_strip_rect, mouse);
    draw_info_window(state, mouse);
    draw_build_menu(state, mouse);

    if state.pawn_ui.context_menu.is_some() && !state.hud_ui.build_menu_open {
        ui_pawns::draw_pawn_context_menu(state, mouse);
    }
}

fn draw_bar_background(bar: Rect, _time: f32) {
    let base_top = rgba(42, 62, 86, 248);
    let base_bottom = rgba(16, 24, 36, 252);
    draw_vertical_gradient(bar, base_top, base_bottom, 34);

    draw_rectangle(
        bar.x,
        bar.y + bar.h * 0.62,
        bar.w,
        bar.h * 0.38,
        with_alpha(rgba(0, 0, 0, 255), 0.24),
    );

    draw_rectangle(
        bar.x,
        bar.y,
        bar.w,
        2.0,
        with_alpha(ui_col_border_hi(), 0.42),
    );
    draw_rectangle(
        bar.x,
        bar.y + 2.0,
        bar.w,
        1.0,
        with_alpha(ui_col_border_hi(), 0.24),
    );
    draw_rectangle(
        bar.x,
        bar.y + bar.h * 0.56,
        bar.w,
        1.0,
        with_alpha(ui_col_border(), 0.30),
    );
    draw_rectangle_lines(
        bar.x,
        bar.y,
        bar.w,
        bar.h,
        2.0,
        with_alpha(ui_col_border(), 0.88),
    );
    draw_rectangle_lines(
        bar.x + 1.0,
        bar.y + 1.0,
        bar.w - 2.0,
        bar.h - 2.0,
        1.0,
        rgba(22, 30, 40, 228),
    );
}

fn ui_col_border() -> Color {
    ui_theme().border
}

fn ui_col_border_hi() -> Color {
    ui_theme().border_hi
}

fn ui_col_accent() -> Color {
    ui_theme().accent_amber
}

fn ui_col_text_primary() -> Color {
    ui_theme().text_primary
}

fn ui_col_text_secondary() -> Color {
    ui_theme().text_secondary
}

fn ui_col_glow_cyan() -> Color {
    ui_theme().accent_cyan
}

fn ui_col_glow_teal() -> Color {
    ui_theme().accent_teal
}

fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

fn draw_vertical_gradient(rect: Rect, top: Color, bottom: Color, slices: usize) {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }
    let slices = slices.max(1);
    let slice_h = rect.h / slices as f32;
    let denom = (slices.saturating_sub(1)).max(1) as f32;
    for i in 0..slices {
        let t = i as f32 / denom;
        let y = rect.y + i as f32 * slice_h;
        let h = if i + 1 == slices {
            (rect.y + rect.h - y).max(0.0)
        } else {
            (slice_h + 0.5).max(0.0)
        };
        if h > 0.0 {
            draw_rectangle(rect.x, y, rect.w, h, mix_color(top, bottom, t));
        }
    }
}

fn draw_panel_drop_shadow(rect: Rect, alpha: f32) {
    draw_rectangle(
        rect.x + 2.0,
        rect.y + 3.0,
        rect.w,
        rect.h,
        with_alpha(rgba(0, 0, 0, 255), alpha.clamp(0.0, 1.0)),
    );
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
    let ox = off.x.max(0.75);
    let oy = off.y.max(0.75);
    let soft_shadow = with_alpha(shadow, (shadow.a * 0.65).clamp(0.0, 0.12));
    draw_text(text, x + ox * 0.55, y + oy * 0.75, fs, soft_shadow);
    draw_text(text, x, y, fs, fill);
}

fn draw_panel_frame(rect: Rect, title: &str, mouse: Vec2) {
    let hovered = point_in_rect(mouse, rect);
    draw_panel_drop_shadow(rect, if hovered { 0.32 } else { 0.24 });

    let (base_top, base_bottom) = ui_panel_fill(hovered);
    draw_vertical_gradient(rect, base_top, base_bottom, 18);
    draw_rectangle(
        rect.x + 1.0,
        rect.y + rect.h * 0.54,
        (rect.w - 2.0).max(0.0),
        (rect.h * 0.46).max(0.0),
        with_alpha(rgba(0, 0, 0, 255), 0.22),
    );

    let border_col = if hovered {
        ui_col_border_hi()
    } else {
        ui_col_border()
    };
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, border_col);
    draw_rectangle_lines(
        rect.x + 1.0,
        rect.y + 1.0,
        rect.w - 2.0,
        rect.h - 2.0,
        1.0,
        rgba(10, 20, 34, 220),
    );

    let header_h = 24.0;
    let header = Rect::new(
        rect.x + 1.0,
        rect.y + 1.0,
        (rect.w - 2.0).max(1.0),
        header_h - 1.0,
    );
    let (header_top, header_bottom) = ui_panel_header_fill(hovered);
    draw_vertical_gradient(header, header_top, header_bottom, 10);
    draw_rectangle(
        header.x,
        header.y + header.h * 0.62,
        header.w,
        header.h * 0.38,
        with_alpha(rgba(0, 0, 0, 255), 0.18),
    );
    draw_rectangle_lines(
        header.x,
        header.y,
        header.w,
        header.h,
        1.0,
        with_alpha(ui_col_border_hi(), if hovered { 0.78 } else { 0.54 }),
    );

    let fs = 16.0;
    let (fill, shadow) = ui_text_and_shadow_for_bg(header_bottom);
    draw_panel_title_icon(
        title,
        vec2(rect.x + 15.0, rect.y + 13.0),
        if hovered {
            ui_col_border_hi()
        } else {
            ui_col_glow_cyan()
        },
    );
    draw_text_shadowed(
        title,
        rect.x + 36.0,
        rect.y + 17.5,
        fs,
        if hovered { ui_col_text_primary() } else { fill },
        shadow,
        ui_shadow_offset(fs),
    );
}

fn draw_panel_title_icon(title: &str, center: Vec2, color: Color) {
    let c = with_alpha(color, 0.92);
    let dark = rgba(3, 8, 16, 170);
    match title {
        "ÉQUIPE" | "Equipe" => {
            draw_circle(center.x - 3.5, center.y - 2.5, 2.2, dark);
            draw_circle(center.x - 3.5, center.y - 2.5, 2.0, c);
            draw_line(
                center.x - 7.0,
                center.y + 4.0,
                center.x,
                center.y + 4.0,
                2.0,
                c,
            );
            draw_circle(center.x + 4.0, center.y + 1.0, 2.0, with_alpha(c, 0.75));
            draw_line(
                center.x + 1.0,
                center.y + 6.0,
                center.x + 8.0,
                center.y + 6.0,
                1.7,
                c,
            );
        }
        "CONSTRUCTION" | "Construction" => {
            draw_rectangle_lines(center.x - 7.0, center.y - 7.0, 6.0, 6.0, 1.5, c);
            draw_rectangle_lines(center.x + 1.0, center.y - 1.0, 6.0, 6.0, 1.5, c);
            draw_line(
                center.x - 1.0,
                center.y - 4.0,
                center.x + 3.0,
                center.y - 4.0,
                1.4,
                c,
            );
            draw_line(
                center.x + 3.0,
                center.y - 4.0,
                center.x + 3.0,
                center.y - 1.0,
                1.4,
                c,
            );
            draw_line(
                center.x - 5.0,
                center.y - 1.0,
                center.x - 5.0,
                center.y + 7.0,
                1.2,
                c,
            );
        }
        "PERSONNAGE" | "Personnage" => {
            draw_rectangle_lines(center.x - 6.0, center.y - 6.0, 12.0, 12.0, 1.3, c);
            draw_circle(center.x, center.y - 2.0, 2.2, c);
            draw_line(
                center.x - 4.0,
                center.y + 4.5,
                center.x + 4.0,
                center.y + 4.5,
                2.0,
                c,
            );
        }
        "MINI-CARTE" | "Mini-carte" => {
            draw_rectangle_lines(center.x - 7.0, center.y - 5.0, 14.0, 10.0, 1.3, c);
            draw_line(
                center.x - 2.5,
                center.y - 5.0,
                center.x - 2.5,
                center.y + 5.0,
                1.0,
                c,
            );
            draw_line(
                center.x + 3.0,
                center.y - 5.0,
                center.x + 3.0,
                center.y + 5.0,
                1.0,
                c,
            );
            draw_circle(center.x + 4.5, center.y + 1.0, 1.8, with_alpha(c, 0.8));
        }
        _ => {
            draw_circle_lines(center.x, center.y, 6.2, 1.5, c);
            draw_circle(center.x, center.y, 2.0, c);
        }
    }
}

fn draw_top_strip(state: &GameState, rect: Rect, mouse: Vec2) {
    let scale = (rect.h / 78.0).clamp(0.78, 1.2);
    let top = rgba(4, 18, 36, 252);
    let bottom = rgba(2, 10, 24, 255);
    draw_vertical_gradient(rect, top, bottom, 18);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h * 0.48, rgba(16, 56, 92, 82));
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        2.0,
        with_alpha(ui_col_border_hi(), 0.22),
    );
    draw_rectangle(
        rect.x,
        rect.y + rect.h - 2.0,
        rect.w,
        2.0,
        with_alpha(ui_col_border(), 0.86),
    );

    let cash = state.sim.cash();
    let sold = state.sim.sold_total();
    let cadence = state.sim.throughput_per_hour();
    let otif = state.sim.otif();

    let metrics = [
        (
            "TRÉSORERIE",
            format_money(cash),
            HeaderIcon::Euro,
            ui_col_accent(),
        ),
        (
            "VENTES",
            sold.to_string(),
            HeaderIcon::Cart,
            rgba(84, 188, 242, 242),
        ),
        (
            "CADENCE",
            format!("{cadence:.1} / h"),
            HeaderIcon::Pulse,
            rgba(86, 188, 232, 242),
        ),
        (
            "FIABILITÉ",
            format!("{:.0}%", (otif * 100.0).clamp(0.0, 999.0)),
            HeaderIcon::Shield,
            rgba(84, 218, 112, 242),
        ),
        (
            "SERVICE",
            format!("{:.0}%", (otif * 100.0).clamp(0.0, 999.0)),
            HeaderIcon::Check,
            rgba(118, 218, 112, 242),
        ),
    ];

    let layout = compute_top_strip_layout(rect, scale, metrics.len());
    draw_brand_header(layout.brand, scale);
    draw_header_nav_group(layout.nav, scale);
    draw_simulation_card(state, layout.simulation, mouse, scale);

    for ((label, value, icon, accent), card) in metrics.iter().zip(layout.metrics.iter()) {
        draw_metric_card(*card, label, value, *icon, *accent, mouse, scale);
    }
}

#[derive(Clone, Debug)]
struct TopStripLayout {
    brand: Rect,
    nav: Rect,
    simulation: Rect,
    metrics: Vec<Rect>,
}

fn shrink_dimension(value: &mut f32, minimum: f32, overflow: &mut f32) {
    if *overflow <= 0.0 {
        return;
    }
    let shrink = (*value - minimum).max(0.0).min(*overflow);
    *value -= shrink;
    *overflow -= shrink;
}

fn compute_top_strip_layout(rect: Rect, scale: f32, metric_count: usize) -> TopStripLayout {
    let metric_count = metric_count.max(1);
    let pad = (12.0 * scale).clamp(9.0, 14.0);
    let gap = (14.0 * scale).clamp(9.0, 16.0);
    let metric_gap = (8.0 * scale).clamp(6.0, 10.0);
    let card_h = (rect.h - pad * 2.0).max(44.0);
    let card_y = rect.y + (rect.h - card_h) * 0.5;
    let content_w = (rect.w - pad * 2.0).max(1.0);

    let mut brand_w = (rect.w * 0.108).clamp(140.0, 220.0);
    let mut nav_w = (rect.w * 0.235).clamp(240.0, 440.0);
    let mut sim_w = (rect.w * 0.145).clamp(178.0, 280.0);
    let mut metric_w = (rect.w * 0.092).clamp(82.0, 176.0);

    let min_brand_w = (112.0 * scale).clamp(96.0, 132.0);
    let min_nav_w = (185.0 * scale).clamp(140.0, 230.0);
    let min_sim_w = (150.0 * scale).clamp(116.0, 186.0);
    let min_metric_w = (68.0 * scale).clamp(46.0, 82.0);
    let metric_gaps_w = metric_gap * (metric_count.saturating_sub(1) as f32);
    let section_gaps_w = gap * 3.0;

    let preferred_total =
        brand_w + nav_w + sim_w + metric_w * metric_count as f32 + metric_gaps_w + section_gaps_w;
    let mut overflow = (preferred_total - content_w).max(0.0);
    shrink_dimension(&mut nav_w, min_nav_w, &mut overflow);
    shrink_dimension(&mut sim_w, min_sim_w, &mut overflow);
    shrink_dimension(&mut brand_w, min_brand_w, &mut overflow);
    shrink_dimension(&mut metric_w, min_metric_w, &mut overflow);

    let fixed_w = brand_w + nav_w + sim_w + metric_gaps_w + section_gaps_w;
    let available_metric_w = (content_w - fixed_w).max(metric_count as f32);
    metric_w = (available_metric_w / metric_count as f32)
        .min(metric_w)
        .max(1.0);

    let left = rect.x + pad;
    let right = rect.x + rect.w - pad;
    let metrics_total = metric_w * metric_count as f32 + metric_gaps_w;
    let metrics_x = (right - metrics_total).max(left);
    let simulation_x = (metrics_x - gap - sim_w).max(left);
    let nav_x = (simulation_x - gap - nav_w).max(left);

    let brand = Rect::new(left, card_y, (nav_x - left - gap).max(1.0), card_h);
    let nav = Rect::new(nav_x, card_y, nav_w.max(1.0), card_h);
    let simulation = Rect::new(simulation_x, card_y, sim_w.max(1.0), card_h);
    let metrics = (0..metric_count)
        .map(|idx| {
            Rect::new(
                metrics_x + idx as f32 * (metric_w + metric_gap),
                card_y,
                metric_w,
                card_h,
            )
        })
        .collect();

    TopStripLayout {
        brand,
        nav,
        simulation,
        metrics,
    }
}

#[derive(Clone, Copy)]
enum HeaderIcon {
    Euro,
    Cart,
    Pulse,
    Shield,
    Check,
    Clock,
    Calendar,
    Result,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GestionQuickAction {
    HireLead,
    HireCariste,
    HireAdmin,
    BuyRaw500,
    BuyRaw1000,
    ToggleInterim,
}

fn draw_brand_header(rect: Rect, scale: f32) {
    let title_size = (27.0 * scale).clamp(22.0, 30.0);
    let sub_size = (14.0 * scale).clamp(12.0, 16.0);
    let shadow = rgba(0, 0, 0, 180);
    draw_text_shadowed(
        "RXCHIXS",
        rect.x + 8.0,
        rect.y + rect.h * 0.42,
        title_size,
        rgba(206, 232, 250, 255),
        shadow,
        ui_shadow_offset(title_size),
    );
    draw_text_shadowed(
        "PROTOTYPE VISUEL",
        rect.x + 8.0,
        rect.y + rect.h * 0.78,
        sub_size,
        rgba(82, 166, 232, 250),
        shadow,
        ui_shadow_offset(sub_size),
    );
    draw_circle(
        rect.x + rect.w - 14.0,
        rect.y + rect.h * 0.76,
        2.2,
        rgba(92, 210, 246, 230),
    );
}

fn draw_header_nav_group(rect: Rect, scale: f32) {
    draw_top_card_frame(rect, false);
    let cells = [
        ("MODE JEU", "ÉCHAP : PAUSE"),
        ("ÉDITEUR", "F10"),
        ("PLEIN ÉCRAN", "F11"),
    ];
    let cell_w = rect.w / cells.len() as f32;
    for (idx, (label, hint)) in cells.iter().enumerate() {
        let x = rect.x + idx as f32 * cell_w;
        if idx > 0 {
            draw_line(
                x,
                rect.y + 8.0,
                x,
                rect.y + rect.h - 8.0,
                1.0,
                rgba(74, 132, 184, 118),
            );
        }
        let fs = (14.0 * scale).clamp(11.0, 15.0);
        let hint_fs = (13.0 * scale).clamp(10.0, 14.0);
        let label_w = measure_text(label, None, fs as u16, 1.0).width;
        let hint_w = measure_text(hint, None, hint_fs as u16, 1.0).width;
        draw_text_shadowed(
            label,
            x + cell_w * 0.5 - label_w * 0.5,
            rect.y + rect.h * 0.38,
            fs,
            ui_col_text_primary(),
            rgba(0, 0, 0, 170),
            ui_shadow_offset(fs),
        );
        draw_text_shadowed(
            hint,
            x + cell_w * 0.5 - hint_w * 0.5,
            rect.y + rect.h * 0.72,
            hint_fs,
            ui_col_text_secondary(),
            rgba(0, 0, 0, 150),
            ui_shadow_offset(hint_fs),
        );
    }
}

fn draw_simulation_card(state: &GameState, rect: Rect, mouse: Vec2, scale: f32) {
    draw_top_card_frame(rect, point_in_rect(mouse, rect));
    let accent = ui_col_glow_cyan();
    let icon_rect = Rect::new(
        rect.x + 16.0 * scale,
        rect.y + rect.h * 0.25,
        rect.h * 0.44,
        rect.h * 0.50,
    );
    draw_rectangle(
        icon_rect.x,
        icon_rect.y,
        icon_rect.w,
        icon_rect.h,
        rgba(10, 44, 76, 210),
    );
    draw_rectangle_lines(
        icon_rect.x,
        icon_rect.y,
        icon_rect.w,
        icon_rect.h,
        1.2,
        rgba(94, 190, 246, 170),
    );
    draw_triangle(
        vec2(
            icon_rect.x + icon_rect.w * 0.38,
            icon_rect.y + icon_rect.h * 0.24,
        ),
        vec2(
            icon_rect.x + icon_rect.w * 0.38,
            icon_rect.y + icon_rect.h * 0.76,
        ),
        vec2(
            icon_rect.x + icon_rect.w * 0.76,
            icon_rect.y + icon_rect.h * 0.50,
        ),
        accent,
    );

    let label_fs = (13.0 * scale).clamp(10.0, 14.0);
    let value_fs = (28.0 * scale).clamp(22.0, 31.0);
    let tx = icon_rect.x + icon_rect.w + 12.0 * scale;
    draw_text_shadowed(
        "SIMULATION",
        tx,
        rect.y + rect.h * 0.33,
        label_fs,
        ui_col_text_secondary(),
        rgba(0, 0, 0, 160),
        ui_shadow_offset(label_fs),
    );
    draw_text_shadowed(
        &format_clock_hhmmss(state.sim.clock.seconds()),
        tx,
        rect.y + rect.h * 0.74,
        value_fs,
        ui_col_text_primary(),
        rgba(0, 0, 0, 190),
        ui_shadow_offset(value_fs),
    );
    draw_circuit_ticks(rect, accent);
}

fn draw_metric_card(
    rect: Rect,
    label: &str,
    value: &str,
    icon: HeaderIcon,
    accent: Color,
    mouse: Vec2,
    scale: f32,
) {
    let hovered = point_in_rect(mouse, rect);
    draw_top_card_frame(rect, hovered);
    let icon_center = vec2(rect.x + 26.0 * scale, rect.y + rect.h * 0.58);
    draw_header_icon(icon, icon_center, rect.h * 0.30, accent);

    let label_fs = (11.0 * scale).clamp(9.0, 12.0);
    let value_fs = (22.0 * scale).clamp(16.0, 24.0);
    let text_x = rect.x + (50.0 * scale).clamp(38.0, 56.0);
    draw_text_shadowed(
        label,
        text_x,
        rect.y + rect.h * 0.36,
        label_fs,
        ui_col_text_secondary(),
        rgba(0, 0, 0, 160),
        ui_shadow_offset(label_fs),
    );
    draw_text_shadowed(
        value,
        text_x,
        rect.y + rect.h * 0.76,
        value_fs,
        ui_col_text_primary(),
        rgba(0, 0, 0, 190),
        ui_shadow_offset(value_fs),
    );
}

fn draw_top_card_frame(rect: Rect, hovered: bool) {
    draw_panel_drop_shadow(rect, if hovered { 0.26 } else { 0.18 });
    let top = if hovered {
        rgba(13, 47, 78, 244)
    } else {
        rgba(7, 29, 54, 240)
    };
    let bottom = rgba(2, 11, 26, 246);
    draw_vertical_gradient(rect, top, bottom, 14);
    draw_rectangle(
        rect.x + 1.0,
        rect.y + 1.0,
        (rect.w - 2.0).max(0.0),
        rect.h * 0.43,
        rgba(40, 112, 166, if hovered { 70 } else { 48 }),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.7,
        if hovered {
            rgba(162, 226, 255, 222)
        } else {
            rgba(72, 144, 204, 176)
        },
    );
    draw_rectangle_lines(
        rect.x + 1.0,
        rect.y + 1.0,
        (rect.w - 2.0).max(0.0),
        (rect.h - 2.0).max(0.0),
        1.0,
        rgba(3, 10, 22, 210),
    );
}

fn draw_circuit_ticks(rect: Rect, color: Color) {
    let c = with_alpha(color, 0.32);
    let y = rect.y + rect.h - 14.0;
    let x0 = rect.x + rect.w - 54.0;
    draw_line(x0, y, x0 + 12.0, y, 1.0, c);
    draw_line(x0 + 20.0, y + 3.0, x0 + 35.0, y + 3.0, 1.0, c);
    draw_circle(x0 + 15.0, y, 1.3, c);
    draw_circle(x0 + 38.0, y + 3.0, 1.3, c);
}

fn draw_header_icon(icon: HeaderIcon, center: Vec2, size: f32, color: Color) {
    let col = with_alpha(color, 0.96);
    let s = size.max(8.0);
    match icon {
        HeaderIcon::Euro => draw_euro_icon(center.x - s * 0.32, center.y + s * 0.46, s, col),
        HeaderIcon::Cart => {
            draw_line(
                center.x - s * 0.48,
                center.y - s * 0.28,
                center.x - s * 0.30,
                center.y + s * 0.24,
                2.3,
                col,
            );
            draw_rectangle_lines(
                center.x - s * 0.24,
                center.y - s * 0.10,
                s * 0.62,
                s * 0.34,
                2.0,
                col,
            );
            draw_circle(center.x - s * 0.10, center.y + s * 0.36, s * 0.08, col);
            draw_circle(center.x + s * 0.34, center.y + s * 0.36, s * 0.08, col);
        }
        HeaderIcon::Pulse => {
            draw_line(
                center.x - s * 0.52,
                center.y,
                center.x - s * 0.28,
                center.y,
                2.0,
                col,
            );
            draw_line(
                center.x - s * 0.28,
                center.y,
                center.x - s * 0.16,
                center.y - s * 0.22,
                2.0,
                col,
            );
            draw_line(
                center.x - s * 0.16,
                center.y - s * 0.22,
                center.x + s * 0.02,
                center.y + s * 0.28,
                2.0,
                col,
            );
            draw_line(
                center.x + s * 0.02,
                center.y + s * 0.28,
                center.x + s * 0.18,
                center.y - s * 0.18,
                2.0,
                col,
            );
            draw_line(
                center.x + s * 0.18,
                center.y - s * 0.18,
                center.x + s * 0.34,
                center.y,
                2.0,
                col,
            );
            draw_line(
                center.x + s * 0.34,
                center.y,
                center.x + s * 0.52,
                center.y,
                2.0,
                col,
            );
        }
        HeaderIcon::Shield => {
            let pts = [
                vec2(center.x, center.y - s * 0.54),
                vec2(center.x + s * 0.42, center.y - s * 0.32),
                vec2(center.x + s * 0.34, center.y + s * 0.24),
                vec2(center.x, center.y + s * 0.54),
                vec2(center.x - s * 0.34, center.y + s * 0.24),
                vec2(center.x - s * 0.42, center.y - s * 0.32),
            ];
            for i in 0..pts.len() {
                let a = pts[i];
                let b = pts[(i + 1) % pts.len()];
                draw_line(a.x, a.y, b.x, b.y, 1.8, col);
            }
            draw_line(
                center.x - s * 0.16,
                center.y,
                center.x - s * 0.02,
                center.y + s * 0.16,
                2.0,
                col,
            );
            draw_line(
                center.x - s * 0.02,
                center.y + s * 0.16,
                center.x + s * 0.20,
                center.y - s * 0.14,
                2.0,
                col,
            );
        }
        HeaderIcon::Check => {
            draw_circle(center.x, center.y, s * 0.48, with_alpha(col, 0.22));
            draw_circle_lines(center.x, center.y, s * 0.48, 2.0, col);
            draw_line(
                center.x - s * 0.21,
                center.y,
                center.x - s * 0.04,
                center.y + s * 0.17,
                3.2,
                col,
            );
            draw_line(
                center.x - s * 0.04,
                center.y + s * 0.17,
                center.x + s * 0.25,
                center.y - s * 0.20,
                3.2,
                col,
            );
        }
        HeaderIcon::Clock => {
            draw_circle_lines(center.x, center.y, s * 0.44, 2.0, col);
            draw_line(center.x, center.y, center.x, center.y - s * 0.25, 2.0, col);
            draw_line(
                center.x,
                center.y,
                center.x + s * 0.22,
                center.y + s * 0.12,
                2.0,
                col,
            );
        }
        HeaderIcon::Calendar => {
            draw_rectangle_lines(
                center.x - s * 0.42,
                center.y - s * 0.34,
                s * 0.84,
                s * 0.68,
                2.0,
                col,
            );
            draw_line(
                center.x - s * 0.42,
                center.y - s * 0.12,
                center.x + s * 0.42,
                center.y - s * 0.12,
                2.0,
                col,
            );
            draw_line(
                center.x - s * 0.20,
                center.y - s * 0.46,
                center.x - s * 0.20,
                center.y - s * 0.24,
                2.0,
                col,
            );
            draw_line(
                center.x + s * 0.20,
                center.y - s * 0.46,
                center.x + s * 0.20,
                center.y - s * 0.24,
                2.0,
                col,
            );
        }
        HeaderIcon::Result => {
            draw_rectangle(
                center.x - s * 0.36,
                center.y + s * 0.12,
                s * 0.15,
                s * 0.28,
                col,
            );
            draw_rectangle(
                center.x - s * 0.10,
                center.y - s * 0.08,
                s * 0.15,
                s * 0.48,
                col,
            );
            draw_rectangle(
                center.x + s * 0.16,
                center.y - s * 0.30,
                s * 0.15,
                s * 0.70,
                col,
            );
        }
    }
}

fn draw_footer_strip(state: &GameState, rect: Rect, mouse: Vec2) {
    let scale = (rect.h / 50.0).clamp(0.82, 1.2);
    draw_top_card_frame(rect, point_in_rect(mouse, rect));
    let _revenue = state.sim.revenue_total();
    let _cost = state.sim.cost_total();

    let pad = (6.0 * scale).clamp(5.0, 8.0);
    let gap = (8.0 * scale).clamp(6.0, 10.0);
    let cell_h = (rect.h - pad * 2.0).max(28.0);
    let y = rect.y + pad;
    let speed_cluster_w = (rect.w * 0.18).clamp(220.0, 340.0);
    let right_edge = rect.x + rect.w - pad;
    let speed_rect = Rect::new(right_edge - speed_cluster_w, y, speed_cluster_w, cell_h);

    let result_w = (rect.w * 0.17).clamp(190.0, 310.0);
    let result_rect = Rect::new(speed_rect.x - gap - result_w, y, result_w, cell_h);

    let fixed_right_x = result_rect.x - gap;
    let cells = [
        (
            "HEURE",
            state.sim.clock.format_hhmm(),
            HeaderIcon::Clock,
            rgba(150, 210, 252, 240),
        ),
        (
            "JOUR",
            format!("JOUR {}", state.sim.clock.day_index() + 1),
            HeaderIcon::Calendar,
            rgba(150, 210, 252, 240),
        ),
        (
            "TRÉSORERIE",
            format_money(state.sim.cash()),
            HeaderIcon::Euro,
            ui_col_accent(),
        ),
        (
            "VENTES",
            state.sim.sold_total().to_string(),
            HeaderIcon::Cart,
            rgba(84, 188, 242, 242),
        ),
        (
            "CADENCE",
            format!("{:.1} / h", state.sim.throughput_per_hour()),
            HeaderIcon::Pulse,
            rgba(86, 188, 232, 242),
        ),
        (
            "FIABILITÉ",
            format!("{:.0}%", (state.sim.otif() * 100.0).clamp(0.0, 999.0)),
            HeaderIcon::Shield,
            rgba(84, 218, 112, 242),
        ),
    ];
    let available = (fixed_right_x - (rect.x + pad)).max(1.0);
    let cell_w = ((available - gap * (cells.len() as f32 - 1.0)) / cells.len() as f32).max(70.0);
    let mut x = rect.x + pad;
    for (label, value, icon, accent) in cells {
        if x + cell_w <= fixed_right_x + 0.5 {
            draw_footer_cell(
                Rect::new(x, y, cell_w, cell_h),
                label,
                &value,
                icon,
                accent,
                mouse,
                scale,
            );
        }
        x += cell_w + gap;
    }

    let profit = state.sim.profit_total();
    let profit_col = if profit >= 0.0 {
        feedback_theme().positive
    } else {
        feedback_theme().danger
    };
    draw_footer_cell(
        result_rect,
        "RÉSULTAT",
        &format_money(profit),
        HeaderIcon::Result,
        profit_col,
        mouse,
        scale,
    );
    draw_speed_cluster(state, speed_rect, mouse);
}

fn draw_footer_cell(
    rect: Rect,
    label: &str,
    value: &str,
    icon: HeaderIcon,
    accent: Color,
    mouse: Vec2,
    scale: f32,
) {
    let hovered = point_in_rect(mouse, rect);
    draw_top_card_frame(rect, hovered);
    draw_header_icon(
        icon,
        vec2(rect.x + 24.0 * scale, rect.y + rect.h * 0.54),
        rect.h * 0.48,
        accent,
    );
    let label_fs = (11.0 * scale).clamp(9.0, 12.0);
    let value_fs = (18.0 * scale).clamp(14.0, 20.0);
    let text_x = rect.x + (48.0 * scale).clamp(40.0, 54.0);
    draw_text_shadowed(
        label,
        text_x,
        rect.y + rect.h * 0.38,
        label_fs,
        ui_col_text_secondary(),
        rgba(0, 0, 0, 160),
        ui_shadow_offset(label_fs),
    );
    draw_text_shadowed(
        value,
        text_x,
        rect.y + rect.h * 0.76,
        value_fs,
        ui_col_text_primary(),
        rgba(0, 0, 0, 180),
        ui_shadow_offset(value_fs),
    );
}

fn draw_speed_cluster(state: &GameState, rect: Rect, mouse: Vec2) {
    draw_top_card_frame(rect, point_in_rect(mouse, rect));
    for (speed, brect) in footer_speed_button_rects(rect) {
        let hovered = point_in_rect(mouse, brect);
        let active = state.hud_ui.sim_speed == speed;
        draw_small_button(brect, speed.label(), hovered, active);
    }
}

fn footer_speed_button_rects(rect: Rect) -> [(SimSpeed, Rect); 4] {
    let gap = 8.0;
    let pad = 8.0;
    let btn_h = (rect.h - pad * 2.0).max(24.0);
    let btn_w = ((rect.w - pad * 2.0 - gap * 3.0) / 4.0).max(28.0);
    let y = rect.y + (rect.h - btn_h) * 0.5;
    [
        (SimSpeed::Pause, Rect::new(rect.x + pad, y, btn_w, btn_h)),
        (
            SimSpeed::X1,
            Rect::new(rect.x + pad + (btn_w + gap), y, btn_w, btn_h),
        ),
        (
            SimSpeed::X2,
            Rect::new(rect.x + pad + (btn_w + gap) * 2.0, y, btn_w, btn_h),
        ),
        (
            SimSpeed::X4,
            Rect::new(rect.x + pad + (btn_w + gap) * 3.0, y, btn_w, btn_h),
        ),
    ]
}

fn process_footer_strip_input(state: &mut GameState, rect: Rect, mouse: Vec2) -> bool {
    let scale = (rect.h / 50.0).clamp(0.82, 1.2);
    let pad = (6.0 * scale).clamp(5.0, 8.0);
    let cell_h = (rect.h - pad * 2.0).max(28.0);
    let speed_cluster_w = (rect.w * 0.18).clamp(220.0, 340.0);
    let speed_rect = Rect::new(
        rect.x + rect.w - pad - speed_cluster_w,
        rect.y + pad,
        speed_cluster_w,
        cell_h,
    );
    for (speed, brect) in footer_speed_button_rects(speed_rect) {
        if point_in_rect(mouse, brect) {
            state.hud_ui.sim_speed = speed;
            return true;
        }
    }
    false
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
        mix_color(ui_col_accent(), ui_col_glow_teal(), 0.18)
    } else if hovered {
        rgba(78, 166, 228, 244)
    } else {
        rgba(46, 96, 148, 236)
    };
    let bottom = mix_color(base, rgba(6, 12, 24, 255), if active { 0.34 } else { 0.46 });
    let border = if active {
        mix_color(ui_col_accent(), WHITE, 0.42)
    } else if hovered {
        ui_col_border_hi()
    } else {
        with_alpha(ui_col_border(), 0.90)
    };

    draw_panel_drop_shadow(rect, if active { 0.24 } else { 0.18 });
    draw_vertical_gradient(rect, base, bottom, 10);
    draw_rectangle(
        rect.x + 1.0,
        rect.y + 1.0,
        (rect.w - 2.0).max(0.0),
        rect.h * 0.45,
        with_alpha(
            WHITE,
            if active {
                0.24
            } else if hovered {
                0.14
            } else {
                0.09
            },
        ),
    );
    draw_rectangle(
        rect.x,
        rect.y + rect.h * 0.68,
        rect.w,
        rect.h * 0.32,
        with_alpha(rgba(0, 0, 0, 255), 0.18),
    );
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, border);
    draw_rectangle_lines(
        rect.x + 1.0,
        rect.y + 1.0,
        rect.w - 2.0,
        rect.h - 2.0,
        1.0,
        rgba(8, 14, 24, 158),
    );

    let fs = (rect.h * 0.72).clamp(12.0, 18.0);
    let dims = measure_text(label, None, fs as u16, 1.0);
    let tx = rect.x + rect.w * 0.5 - dims.width * 0.5;
    let ty = rect.y + rect.h * 0.5 + dims.height * 0.34;
    let (fill, shadow) = ui_text_and_shadow_for_bg(bottom);
    let text_col = if active { rgba(18, 24, 30, 248) } else { fill };
    draw_text_shadowed(label, tx, ty, fs, text_col, shadow, ui_shadow_offset(fs));
}

fn rects_intersect(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y
}

fn rows_for_count(item_count: usize, cols: usize) -> usize {
    if item_count == 0 || cols == 0 {
        0
    } else {
        item_count.div_ceil(cols)
    }
}

fn apply_panel_wheel_scroll(scroll_y: &mut f32, wheel_y: f32, max_scroll: f32) -> bool {
    let max_scroll = max_scroll.max(0.0);
    let before = *scroll_y;
    *scroll_y = (*scroll_y - wheel_y * PANEL_SCROLL_STEP).clamp(0.0, max_scroll);
    (before - *scroll_y).abs() > 0.01
}

fn draw_vertical_scrollbar(view: Rect, content_h: f32, scroll_y: f32) {
    if content_h <= view.h + 1.0 || view.h <= 2.0 || view.w <= 2.0 {
        return;
    }
    let track_w = 6.0;
    let track_x = view.x + view.w - track_w - 2.0;
    let track = Rect::new(track_x, view.y + 2.0, track_w, (view.h - 4.0).max(1.0));
    draw_vertical_gradient(track, rgba(12, 24, 38, 194), rgba(8, 14, 24, 214), 8);
    draw_rectangle_lines(
        track.x,
        track.y,
        track.w,
        track.h,
        1.0,
        with_alpha(ui_col_border(), 0.60),
    );

    let max_scroll = (content_h - view.h).max(1.0);
    let thumb_h = (view.h / content_h * track.h).clamp(16.0, track.h);
    let travel = (track.h - thumb_h).max(0.0);
    let t = (scroll_y / max_scroll).clamp(0.0, 1.0);
    let thumb_y = track.y + travel * t;
    let thumb = Rect::new(track.x, thumb_y, track.w, thumb_h);
    draw_vertical_gradient(thumb, rgba(174, 230, 255, 232), rgba(112, 178, 222, 236), 8);
    draw_rectangle_lines(
        thumb.x,
        thumb.y,
        thumb.w,
        thumb.h,
        1.0,
        with_alpha(ui_col_border_hi(), 0.76),
    );
    draw_rectangle(
        thumb.x,
        thumb.y,
        thumb.w,
        2.0,
        with_alpha(ui_col_border_hi(), 0.55),
    );
}

fn pawn_inner_rect(panel: Rect) -> Rect {
    let pad = 10.0;
    let header_h = 24.0;
    Rect::new(
        panel.x + pad,
        panel.y + header_h + 10.0,
        (panel.w - pad * 2.0).max(1.0),
        (panel.h - header_h - 18.0).max(1.0),
    )
}

fn pawn_grid_layout(panel: Rect) -> (Rect, f32, f32, f32, usize) {
    let scale = ((screen_width() / 1600.0).min(screen_height() / 900.0)).clamp(0.85, 1.15);
    let inner = pawn_inner_rect(panel);
    let card_h = (64.0 * scale).clamp(52.0, 74.0);
    let card_w = (inner.w - 10.0).max(120.0);
    let gap = (10.0 * scale).clamp(8.0, 14.0);
    let cols = 1;
    (inner, card_w, card_h, gap, cols)
}

fn pawn_content_height(state: &GameState, panel: Rect) -> f32 {
    let (_inner, _card_w, card_h, gap, cols) = pawn_grid_layout(panel);
    let rows = rows_for_count(state.pawns.len(), cols);
    if rows == 0 {
        0.0
    } else {
        rows as f32 * card_h + (rows.saturating_sub(1)) as f32 * gap
    }
}

fn process_pawn_panel_wheel(state: &mut GameState, panel: Rect, wheel_y: f32) -> bool {
    let inner = pawn_inner_rect(panel);
    let content_h = pawn_content_height(state, panel);
    let max_scroll = (content_h - inner.h).max(0.0);
    apply_panel_wheel_scroll(&mut state.hud_ui.pawn_scroll_y, wheel_y, max_scroll)
}

fn process_pawn_panel_input(
    state: &mut GameState,
    panel: Rect,
    mouse: Vec2,
    time_now: f32,
) -> bool {
    let inner = pawn_inner_rect(panel);
    if !point_in_rect(mouse, inner) {
        return false;
    }
    let max_scroll = (pawn_content_height(state, panel) - inner.h).max(0.0);
    let scroll_y = state.hud_ui.pawn_scroll_y.clamp(0.0, max_scroll);
    let slots = pawn_slot_layout(state, panel, scroll_y);
    for slot in &slots {
        if !rects_intersect(slot.rect, inner) {
            continue;
        }
        if point_in_rect(mouse, slot.follow_rect) {
            state.pawn_ui.selected = Some(slot.key);
            state.pawn_ui.follow = if state.pawn_ui.follow == Some(slot.key) {
                None
            } else {
                Some(slot.key)
            };
            if state.pawn_ui.follow == Some(slot.key)
                && let Some(pos) = ui_pawns::pawn_world_pos(state, slot.key)
            {
                state.camera_center = pos;
            }
            return true;
        }
    }

    for slot in &slots {
        if !rects_intersect(slot.rect, inner) {
            continue;
        }
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
    draw_panel_frame(panel, "ÉQUIPE", mouse);

    let inner = pawn_inner_rect(panel);
    let content_h = pawn_content_height(state, panel);
    let max_scroll = (content_h - inner.h).max(0.0);
    let scroll_y = state.hud_ui.pawn_scroll_y.clamp(0.0, max_scroll);
    let slots = pawn_slot_layout(state, panel, scroll_y);
    for slot in &slots {
        if !rects_intersect(slot.rect, inner) {
            continue;
        }
        draw_pawn_slot(state, slot, mouse, time);
    }
    draw_vertical_scrollbar(inner, content_h, scroll_y);
}

#[derive(Clone)]
struct PawnSlot {
    key: PawnKey,
    rect: Rect,
    follow_rect: Rect,
}

fn pawn_slot_layout(state: &GameState, panel: Rect, scroll_y: f32) -> Vec<PawnSlot> {
    let (inner, card_w, card_h, gap, cols) = pawn_grid_layout(panel);

    let mut slots = Vec::with_capacity(state.pawns.len());
    let btn = ((screen_height() / 900.0) * 20.0).clamp(16.0, 24.0);
    for (i, pawn) in state.pawns.iter().enumerate() {
        let row = i / cols;
        let col = i % cols;
        let x = inner.x + col as f32 * (card_w + gap);
        let y = inner.y + row as f32 * (card_h + gap) - scroll_y;
        let rect = Rect::new(x, y, card_w, card_h);
        let follow_rect = Rect::new(rect.x + rect.w - btn * 2.0 - 12.0, rect.y + 6.0, btn, btn);
        slots.push(PawnSlot {
            key: pawn.key,
            rect,
            follow_rect,
        });
    }
    slots
}

fn draw_pawn_slot(state: &GameState, slot: &PawnSlot, mouse: Vec2, time: f32) {
    let selected = state.pawn_ui.selected == Some(slot.key);
    let following = state.pawn_ui.follow == Some(slot.key);
    let hovered = point_in_rect(mouse, slot.rect);

    let base = if hovered {
        rgba(40, 58, 78, 236)
    } else {
        rgba(28, 42, 58, 232)
    };
    let border = if selected || following {
        ui_col_border_hi()
    } else if hovered {
        rgba(172, 220, 252, 220)
    } else {
        ui_col_border()
    };
    draw_rectangle(slot.rect.x, slot.rect.y, slot.rect.w, slot.rect.h, base);
    draw_rectangle(
        slot.rect.x,
        slot.rect.y + slot.rect.h * 0.52,
        slot.rect.w,
        slot.rect.h * 0.48,
        rgba(8, 12, 18, 78),
    );
    draw_rectangle_lines(
        slot.rect.x,
        slot.rect.y,
        slot.rect.w,
        slot.rect.h,
        2.0,
        border,
    );
    draw_rectangle_lines(
        slot.rect.x + 1.0,
        slot.rect.y + 1.0,
        slot.rect.w - 2.0,
        slot.rect.h - 2.0,
        1.0,
        rgba(16, 24, 34, 186),
    );

    let accent_col = if following {
        ui_col_accent()
    } else if selected {
        rgba(118, 186, 232, 240)
    } else {
        rgba(90, 136, 170, 150)
    };
    draw_rectangle(
        slot.rect.x + 2.0,
        slot.rect.y + 2.0,
        4.0,
        slot.rect.h - 4.0,
        accent_col,
    );

    let title_bg = Rect::new(
        slot.rect.x + 40.0,
        slot.rect.y + 6.0,
        (slot.follow_rect.x - slot.rect.x - 46.0).max(42.0),
        18.0,
    );
    draw_rectangle(
        title_bg.x,
        title_bg.y,
        title_bg.w.max(1.0),
        title_bg.h,
        rgba(8, 12, 18, 126),
    );

    let portrait_center = vec2(slot.rect.x + 24.0, slot.rect.y + slot.rect.h * 0.56);
    if let Some(record) = ui_pawns::pawn_visual_record(state, slot.key) {
        draw_character(
            record,
            CharacterRenderParams {
                center: portrait_center,
                scale: 0.72,
                presentation: crate::character::CharacterPresentation::Portrait,
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
    let (_fill, shadow) = ui_text_and_shadow_for_bg(base);
    draw_text_shadowed(
        name,
        slot.rect.x + 44.0,
        slot.rect.y + 21.0,
        16.0,
        ui_col_text_primary(),
        shadow,
        ui_shadow_offset(16.0),
    );

    let follow_hover = point_in_rect(mouse, slot.follow_rect);
    let follow_active = following;
    draw_small_button(slot.follow_rect, "+", follow_hover, follow_active);
    let menu_rect = Rect::new(
        slot.follow_rect.x + slot.follow_rect.w + 6.0,
        slot.follow_rect.y,
        slot.follow_rect.w,
        slot.follow_rect.h,
    );
    if menu_rect.x + menu_rect.w <= slot.rect.x + slot.rect.w - 6.0 {
        draw_small_button(menu_rect, "...", point_in_rect(mouse, menu_rect), false);
    }

    if let Some(pawn) = pawn {
        let bar_w = (slot.rect.w - 118.0).max(44.0);
        let bar_x = slot.rect.x + 44.0;
        let bar_y = slot.rect.y + slot.rect.h - 14.0;
        let hp = pawn.metrics.synth[SynthBar::Sante as usize] as f32 / 100.0;
        draw_meter(bar_x, bar_y, bar_w, 7.0, hp, rgba(124, 226, 156, 240));
        let pct = format!("{:.0}%", hp * 100.0);
        let fs = 12.0;
        draw_text_shadowed(
            &pct,
            bar_x + bar_w + 10.0,
            bar_y + 8.5,
            fs,
            ui_col_text_primary(),
            rgba(0, 0, 0, 150),
            ui_shadow_offset(fs),
        );
    }
}

fn draw_meter(x: f32, y: f32, w: f32, h: f32, t: f32, col: Color) {
    draw_rectangle(x, y, w, h, rgba(0, 0, 0, 170));
    draw_rectangle(x + 1.0, y + 1.0, w - 2.0, h - 2.0, rgba(18, 26, 34, 180));
    draw_rectangle(
        x + 1.0,
        y + 1.0,
        ((w - 2.0) * t.clamp(0.0, 1.0)).max(0.0),
        h - 2.0,
        col,
    );
    draw_rectangle_lines(x, y, w, h, 1.0, rgba(170, 216, 248, 140));
}

fn process_build_panel_input(state: &mut GameState, panel: Rect, mouse: Vec2) -> bool {
    let menu_rect = build_menu_open_button_rect(panel);
    if point_in_rect(mouse, menu_rect) {
        state.hud_ui.build_menu_open = !state.hud_ui.build_menu_open;
        if state.hud_ui.build_menu_open {
            state.hud_ui.info_window_open = false;
            state.hud_ui.build_menu_page = 0;
            if state.hud_ui.build_menu_selected.is_none() {
                state.hud_ui.build_menu_selected =
                    Some(default_build_menu_selection(state, state.hud_ui.build_tab));
            }
        }
        return true;
    }
    for (action, rect) in gestion_quick_button_rects(panel) {
        if point_in_rect(mouse, rect) {
            let result = match action {
                GestionQuickAction::HireLead => state.sim.apply_command(SimCommand::HireEmployee {
                    role: EmployeeRole::ChefEquipe,
                }),
                GestionQuickAction::HireCariste => {
                    state.sim.apply_command(SimCommand::HireEmployee {
                        role: EmployeeRole::Cariste,
                    })
                }
                GestionQuickAction::HireAdmin => {
                    state.sim.apply_command(SimCommand::HireEmployee {
                        role: EmployeeRole::AdministrateurVente,
                    })
                }
                GestionQuickAction::BuyRaw500 => state
                    .sim
                    .apply_command(SimCommand::BuyRawStock { qty: 500 }),
                GestionQuickAction::BuyRaw1000 => state
                    .sim
                    .apply_command(SimCommand::BuyRawStock { qty: 1000 }),
                GestionQuickAction::ToggleInterim => {
                    let (line_id, enabled) = {
                        let line = state.sim.main_production_line();
                        let enabled = line
                            .assigned_lead_id
                            .and_then(|lead_id| state.sim.personnel().employee(lead_id))
                            .and_then(|lead| lead.temp_policy.as_ref())
                            .is_none_or(|policy| !policy.enabled);
                        (line.id, enabled)
                    };
                    state.sim.apply_command(SimCommand::SetLineTempPolicy {
                        line_id,
                        enabled,
                        max_temps: 3,
                    })
                }
            };
            match result {
                Ok(msg) | Err(msg) => state.sim.set_status_line(msg),
            }
            return true;
        }
    }
    false
}

fn draw_build_panel(state: &GameState, panel: Rect, mouse: Vec2) {
    draw_panel_frame(panel, "CONSTRUCTION", mouse);
    let summary = build_panel_summary_rect(panel);
    let bg = rgba(12, 18, 26, 228);
    draw_rectangle(summary.x, summary.y, summary.w, summary.h, bg);
    draw_rectangle(
        summary.x,
        summary.y + summary.h * 0.46,
        summary.w,
        summary.h * 0.54,
        rgba(8, 12, 18, 70),
    );
    draw_rectangle_lines(
        summary.x,
        summary.y,
        summary.w,
        summary.h,
        1.5,
        rgba(140, 194, 228, 150),
    );
    draw_rectangle_lines(
        summary.x + 1.0,
        summary.y + 1.0,
        summary.w - 2.0,
        summary.h - 2.0,
        1.0,
        rgba(24, 34, 44, 200),
    );

    let menu_rect = build_menu_open_button_rect(panel);
    let menu_label = if state.hud_ui.build_menu_open {
        "FERMER MENU"
    } else {
        "MENU CONSTRUCTION"
    };
    draw_small_button(
        menu_rect,
        menu_label,
        point_in_rect(mouse, menu_rect),
        state.hud_ui.build_menu_open,
    );

    let mode_line = if state.sim.build_mode_enabled() {
        "Mode construction: actif"
    } else {
        "Mode construction: arret"
    };
    let brush_line = if state.sim.zone_paint_mode_enabled() {
        format!("Brosse active: zone {}", state.sim.zone_brush().label())
    } else if state.sim.floor_paint_mode_enabled() {
        format!("Brosse active: sol {}", state.sim.floor_brush().label())
    } else {
        format!(
            "Brosse active: bloc {}",
            state.sim.block_brush().buyable_label()
        )
    };
    let selected = state
        .hud_ui
        .build_menu_selected
        .unwrap_or_else(|| default_build_menu_selection(state, state.hud_ui.build_tab));
    let selected_line = format!("Selection menu: {}", build_menu_selection_title(selected));
    let (_fill, shadow) = ui_text_and_shadow_for_bg(bg);
    let mut y = menu_rect.y + menu_rect.h + 18.0;
    for line in [&selected_line, mode_line, &brush_line] {
        draw_text_shadowed(
            line,
            summary.x + 8.0,
            y,
            14.0,
            ui_col_text_secondary(),
            shadow,
            ui_shadow_offset(14.0),
        );
        y += 18.0;
    }

    draw_gestion_quick_actions(state, panel, mouse, bg, shadow);

    draw_build_footer(panel, state, mouse);
}

fn gestion_quick_button_rects(panel: Rect) -> [(GestionQuickAction, Rect); 6] {
    let summary = build_panel_summary_rect(panel);
    let pad = 8.0;
    let gap = 6.0;
    let button_h = 24.0;
    let col_w = ((summary.w - pad * 2.0 - gap) * 0.5).max(54.0);
    let x0 = summary.x + pad;
    let x1 = x0 + col_w + gap;
    let y0 = summary.y + summary.h - pad - button_h * 3.0 - gap * 2.0;
    [
        (
            GestionQuickAction::HireLead,
            Rect::new(x0, y0, col_w, button_h),
        ),
        (
            GestionQuickAction::HireCariste,
            Rect::new(x1, y0, col_w, button_h),
        ),
        (
            GestionQuickAction::HireAdmin,
            Rect::new(x0, y0 + button_h + gap, col_w, button_h),
        ),
        (
            GestionQuickAction::BuyRaw500,
            Rect::new(x1, y0 + button_h + gap, col_w, button_h),
        ),
        (
            GestionQuickAction::BuyRaw1000,
            Rect::new(x0, y0 + (button_h + gap) * 2.0, col_w, button_h),
        ),
        (
            GestionQuickAction::ToggleInterim,
            Rect::new(x1, y0 + (button_h + gap) * 2.0, col_w, button_h),
        ),
    ]
}

fn gestion_quick_action_label(action: GestionQuickAction) -> &'static str {
    match action {
        GestionQuickAction::HireLead => "Chef +",
        GestionQuickAction::HireCariste => "Cariste +",
        GestionQuickAction::HireAdmin => "Admin +",
        GestionQuickAction::BuyRaw500 => "Stock 500",
        GestionQuickAction::BuyRaw1000 => "Stock 1000",
        GestionQuickAction::ToggleInterim => "Interim",
    }
}

fn draw_gestion_quick_actions(
    state: &GameState,
    panel: Rect,
    mouse: Vec2,
    bg: Color,
    shadow: Color,
) {
    let buttons = gestion_quick_button_rects(panel);
    let title_y = buttons[0].1.y - 9.0;
    draw_text_shadowed(
        "Gestion entreprise",
        buttons[0].1.x,
        title_y,
        13.0,
        ui_col_text_secondary(),
        shadow,
        ui_shadow_offset(13.0),
    );
    for (action, rect) in buttons {
        draw_small_button(
            rect,
            gestion_quick_action_label(action),
            point_in_rect(mouse, rect),
            false,
        );
    }

    let stock = state.sim.stock();
    let line = state.sim.main_production_line();
    let kpi = format!(
        "Mat rec {} | entree {} | paie {}/h",
        stock.raw_receiving,
        stock.raw_line_input,
        format_money(state.sim.payroll_per_hour())
    );
    let kpi_y = buttons[4].1.y + buttons[4].1.h + 14.0;
    draw_text_shadowed(
        &kpi,
        buttons[4].1.x,
        kpi_y,
        12.0,
        ui_col_text_secondary(),
        ui_shadow_color_for_text(ui_col_text_secondary()),
        ui_shadow_offset(12.0),
    );
    let line_text = format!("Ligne: {}", line.block_reason);
    draw_text_shadowed(
        &line_text,
        buttons[4].1.x,
        kpi_y + 14.0,
        12.0,
        ui_col_text_secondary(),
        ui_shadow_color_for_text(ui_col_text_secondary()),
        ui_shadow_offset(12.0),
    );
    let sales = state.sim.sales_state();
    let sales_text = format!(
        "Vente cap {:.1}/h | revenu {}/h",
        state.sim.sales_capacity_per_hour(),
        format_money(sales.last_revenue_per_hour)
    );
    draw_text_shadowed(
        &sales_text,
        buttons[4].1.x,
        kpi_y + 28.0,
        12.0,
        ui_col_text_secondary(),
        ui_shadow_color_for_text(ui_col_text_secondary()),
        ui_shadow_offset(12.0),
    );
    let _ = bg;
}

fn build_footer_rect(panel: Rect) -> Rect {
    let footer_h = 22.0;
    Rect::new(
        panel.x + 10.0,
        panel.y + panel.h - footer_h - 8.0,
        panel.w - 20.0,
        footer_h,
    )
}

fn build_panel_inner_rect(panel: Rect) -> Rect {
    let pad = 10.0;
    let header_h = 24.0;
    Rect::new(
        panel.x + pad,
        panel.y + header_h + 10.0,
        (panel.w - pad * 2.0).max(1.0),
        (panel.h - header_h - 18.0).max(1.0),
    )
}

fn build_panel_summary_rect(panel: Rect) -> Rect {
    let mut inner = build_panel_inner_rect(panel);
    let footer = build_footer_rect(panel);
    inner.h = (footer.y - 6.0 - inner.y).max(1.0);
    inner
}

fn build_menu_open_button_rect(panel: Rect) -> Rect {
    let summary = build_panel_summary_rect(panel);
    Rect::new(
        summary.x + 8.0,
        summary.y + 8.0,
        (summary.w - 16.0).max(1.0),
        34.0,
    )
}

fn build_menu_rect() -> Rect {
    let sw = screen_width();
    let sh = screen_height();
    let w = (sw * 0.74).clamp(760.0, 1220.0);
    let h = (sh * 0.72).clamp(420.0, 760.0);
    Rect::new((sw - w) * 0.5, (sh - h) * 0.42, w, h)
}

fn build_menu_close_rect(panel: Rect) -> Rect {
    Rect::new(panel.x + panel.w - 30.0, panel.y + 6.0, 22.0, 20.0)
}

fn build_menu_tab_rects(panel: Rect) -> Vec<(HudBuildTab, Rect)> {
    let y = panel.y + 30.0;
    let mut x = panel.x + 12.0;
    let h = 26.0;
    let gap = 8.0;
    let mut out = Vec::with_capacity(4);
    for (tab, w) in [
        (HudBuildTab::Blocs, 110.0),
        (HudBuildTab::Zones, 110.0),
        (HudBuildTab::Sols, 110.0),
        (HudBuildTab::Outils, 110.0),
    ] {
        out.push((tab, Rect::new(x, y, w, h)));
        x += w + gap;
    }
    out
}

fn build_menu_content_rect(panel: Rect) -> Rect {
    Rect::new(
        panel.x + 12.0,
        panel.y + 62.0,
        (panel.w - 24.0).max(1.0),
        (panel.h - 74.0).max(1.0),
    )
}

fn build_menu_entries(tab: HudBuildTab) -> &'static [BuildMenuEntry] {
    match tab {
        HudBuildTab::Blocs => &BUILD_MENU_BLOCKS,
        HudBuildTab::Zones => &BUILD_MENU_ZONES,
        HudBuildTab::Sols => &BUILD_MENU_FLOORS,
        HudBuildTab::Outils => &BUILD_MENU_TOOLS,
    }
}

fn default_build_menu_selection(state: &GameState, tab: HudBuildTab) -> BuildMenuSelection {
    match tab {
        HudBuildTab::Blocs => BuildMenuSelection::Block(state.sim.block_brush()),
        HudBuildTab::Zones => BuildMenuSelection::Zone(state.sim.zone_brush()),
        HudBuildTab::Sols => BuildMenuSelection::Floor(state.sim.floor_brush()),
        HudBuildTab::Outils => BuildMenuSelection::Tool(BuildToolAction::ToggleBuildMode),
    }
}

fn build_menu_selection_title(selection: BuildMenuSelection) -> String {
    match selection {
        BuildMenuSelection::Block(kind) => format!("Bloc {}", kind.buyable_label()),
        BuildMenuSelection::Zone(kind) => format!("Zone {}", kind.label()),
        BuildMenuSelection::Floor(kind) => format!("Sol {}", kind.label()),
        BuildMenuSelection::Tool(tool) => match tool {
            BuildToolAction::ToggleBuildMode => "Basculer mode construction".to_string(),
            BuildToolAction::ToggleZoneOverlay => "Basculer surcouche des zones".to_string(),
            BuildToolAction::ToggleZonePaint => "Basculer peinture des zones".to_string(),
            BuildToolAction::ToggleSalesManager => "Assigner responsable ventes".to_string(),
            BuildToolAction::CancelMoveSource => "Annuler source de deplacement".to_string(),
            BuildToolAction::SaveLayout => "Sauvegarder le layout".to_string(),
        },
    }
}

fn build_menu_selection_cost(selection: BuildMenuSelection) -> Option<f64> {
    match selection {
        BuildMenuSelection::Block(kind) => Some(kind.capex_eur()),
        BuildMenuSelection::Zone(kind) => Some(kind.capex_par_tuile_eur()),
        BuildMenuSelection::Floor(kind) => Some(kind.capex_par_tuile_eur()),
        BuildMenuSelection::Tool(_) => None,
    }
}

fn build_menu_selection_is_active(state: &GameState, selection: BuildMenuSelection) -> bool {
    match selection {
        BuildMenuSelection::Block(kind) => {
            !state.sim.zone_paint_mode_enabled()
                && !state.sim.floor_paint_mode_enabled()
                && state.sim.block_brush() == kind
        }
        BuildMenuSelection::Zone(kind) => {
            state.sim.zone_paint_mode_enabled() && state.sim.zone_brush() == kind
        }
        BuildMenuSelection::Floor(kind) => {
            state.sim.floor_paint_mode_enabled() && state.sim.floor_brush() == kind
        }
        BuildMenuSelection::Tool(tool) => match tool {
            BuildToolAction::ToggleBuildMode => state.sim.build_mode_enabled(),
            BuildToolAction::ToggleZoneOverlay => state.sim.zone_overlay_enabled(),
            BuildToolAction::ToggleZonePaint => state.sim.zone_paint_mode_enabled(),
            BuildToolAction::ToggleSalesManager => state.sim.sales_manager_assigned(),
            BuildToolAction::CancelMoveSource => state.sim.pending_move_block().is_some(),
            BuildToolAction::SaveLayout => false,
        },
    }
}

fn ensure_build_mode_enabled(state: &mut GameState) {
    if !state.sim.build_mode_enabled() {
        state.sim.toggle_build_mode();
    }
}

fn apply_build_menu_selection(state: &mut GameState, selection: BuildMenuSelection) {
    match selection {
        BuildMenuSelection::Block(kind) => {
            state.sim.set_floor_paint_mode(false);
            state.sim.set_zone_paint_mode(false);
            state.sim.set_block_brush(kind);
            ensure_build_mode_enabled(state);
        }
        BuildMenuSelection::Zone(kind) => {
            state.sim.set_floor_paint_mode(false);
            state.sim.set_zone_brush(kind);
            state.sim.set_zone_paint_mode(true);
            ensure_build_mode_enabled(state);
        }
        BuildMenuSelection::Floor(kind) => {
            state.sim.set_zone_paint_mode(false);
            state.sim.set_floor_brush(kind);
            ensure_build_mode_enabled(state);
        }
        BuildMenuSelection::Tool(tool) => match tool {
            BuildToolAction::ToggleBuildMode => state.sim.toggle_build_mode(),
            BuildToolAction::ToggleZoneOverlay => state.sim.toggle_zone_overlay(),
            BuildToolAction::ToggleZonePaint => {
                state
                    .sim
                    .set_zone_paint_mode(!state.sim.zone_paint_mode_enabled());
            }
            BuildToolAction::ToggleSalesManager => {
                state.sim.toggle_sales_manager_assigned();
            }
            BuildToolAction::CancelMoveSource => {
                if state.sim.pending_move_block().is_some() {
                    state.sim.clear_pending_move_block();
                }
            }
            BuildToolAction::SaveLayout => {
                let _ = state.sim.save_layout();
            }
        },
    }
}

fn build_menu_page_count(item_count: usize, page_size: usize) -> usize {
    if item_count == 0 {
        return 1;
    }
    item_count.div_ceil(page_size.max(1))
}

fn build_menu_page_range(
    item_count: usize,
    page_size: usize,
    requested_page: usize,
) -> (usize, usize, usize, usize) {
    let page_count = build_menu_page_count(item_count, page_size);
    let page = requested_page.min(page_count.saturating_sub(1));
    let start = page.saturating_mul(page_size.max(1));
    let end = (start + page_size.max(1)).min(item_count);
    (page, page_count, start, end)
}

struct BuildMenuLayout {
    panel: Rect,
    close_rect: Rect,
    tab_rects: Vec<(HudBuildTab, Rect)>,
    details_rect: Rect,
    footer_rect: Rect,
    prev_rect: Rect,
    next_rect: Rect,
    apply_rect: Rect,
    visible_entries: Vec<(usize, Rect)>,
    page: usize,
    page_count: usize,
}

fn build_menu_layout(state: &GameState) -> BuildMenuLayout {
    let panel = build_menu_rect();
    let close_rect = build_menu_close_rect(panel);
    let tab_rects = build_menu_tab_rects(panel);
    let content = build_menu_content_rect(panel);

    let split_gap = 12.0;
    let details_w = (content.w * 0.30).clamp(240.0, 360.0);
    let grid_w = (content.w - details_w - split_gap).max(1.0);
    let grid_rect = Rect::new(content.x, content.y, grid_w, content.h);
    let details_rect = Rect::new(
        grid_rect.x + grid_rect.w + split_gap,
        content.y,
        details_w,
        content.h,
    );

    let footer_h = 36.0;
    let cards_rect = Rect::new(
        grid_rect.x,
        grid_rect.y,
        grid_rect.w,
        (grid_rect.h - footer_h - 6.0).max(1.0),
    );
    let footer_rect = Rect::new(
        grid_rect.x,
        cards_rect.y + cards_rect.h + 6.0,
        grid_rect.w,
        footer_h,
    );
    let prev_rect = Rect::new(footer_rect.x, footer_rect.y, 94.0, footer_rect.h);
    let next_rect = Rect::new(
        footer_rect.x + footer_rect.w - 94.0,
        footer_rect.y,
        94.0,
        footer_rect.h,
    );
    let apply_rect = Rect::new(
        details_rect.x + 10.0,
        details_rect.y + details_rect.h - 44.0,
        (details_rect.w - 20.0).max(1.0),
        34.0,
    );

    let entries = build_menu_entries(state.hud_ui.build_tab);
    let cols = if cards_rect.w >= 620.0 {
        3
    } else if cards_rect.w >= 390.0 {
        2
    } else {
        1
    };
    let gap = 10.0;
    let card_h = 88.0;
    let rows_fit = (((cards_rect.h + gap) / (card_h + gap)).floor() as usize).max(1);
    let page_size = (rows_fit * cols).max(1);
    let (page, page_count, start, end) =
        build_menu_page_range(entries.len(), page_size, state.hud_ui.build_menu_page);

    let card_w = ((cards_rect.w - gap * (cols as f32 - 1.0)) / cols as f32).max(1.0);
    let mut visible_entries = Vec::with_capacity(end.saturating_sub(start));
    for (slot, idx) in (start..end).enumerate() {
        let row = slot / cols;
        let col = slot % cols;
        let x = cards_rect.x + col as f32 * (card_w + gap);
        let y = cards_rect.y + row as f32 * (card_h + gap);
        visible_entries.push((idx, Rect::new(x, y, card_w, card_h)));
    }

    BuildMenuLayout {
        panel,
        close_rect,
        tab_rects,
        details_rect,
        footer_rect,
        prev_rect,
        next_rect,
        apply_rect,
        visible_entries,
        page,
        page_count,
    }
}

fn build_menu_entry_for_selection(
    selection: BuildMenuSelection,
) -> Option<&'static BuildMenuEntry> {
    match selection {
        BuildMenuSelection::Block(kind) => BUILD_MENU_BLOCKS
            .iter()
            .find(|entry| entry.selection == BuildMenuSelection::Block(kind)),
        BuildMenuSelection::Zone(kind) => BUILD_MENU_ZONES
            .iter()
            .find(|entry| entry.selection == BuildMenuSelection::Zone(kind)),
        BuildMenuSelection::Floor(kind) => BUILD_MENU_FLOORS
            .iter()
            .find(|entry| entry.selection == BuildMenuSelection::Floor(kind)),
        BuildMenuSelection::Tool(tool) => BUILD_MENU_TOOLS
            .iter()
            .find(|entry| entry.selection == BuildMenuSelection::Tool(tool)),
    }
}

fn process_build_menu_input(state: &mut GameState, mouse: Vec2) -> bool {
    let layout = build_menu_layout(state);

    for (tab, rect) in &layout.tab_rects {
        if point_in_rect(mouse, *rect) {
            if state.hud_ui.build_tab != *tab {
                state.hud_ui.build_tab = *tab;
                state.hud_ui.build_menu_page = 0;
                state.hud_ui.build_menu_selected = Some(default_build_menu_selection(state, *tab));
            }
            return true;
        }
    }

    if point_in_rect(mouse, layout.prev_rect) {
        state.hud_ui.build_menu_page = state.hud_ui.build_menu_page.saturating_sub(1);
        return true;
    }
    if point_in_rect(mouse, layout.next_rect) {
        state.hud_ui.build_menu_page =
            (state.hud_ui.build_menu_page + 1).min(layout.page_count.saturating_sub(1));
        return true;
    }

    let entries = build_menu_entries(state.hud_ui.build_tab);
    for (idx, rect) in &layout.visible_entries {
        if point_in_rect(mouse, *rect) {
            if let Some(entry) = entries.get(*idx) {
                state.hud_ui.build_menu_selected = Some(entry.selection);
            }
            return true;
        }
    }

    if point_in_rect(mouse, layout.apply_rect) {
        let selection = state
            .hud_ui
            .build_menu_selected
            .unwrap_or_else(|| default_build_menu_selection(state, state.hud_ui.build_tab));
        apply_build_menu_selection(state, selection);
        state.hud_ui.build_menu_selected = Some(selection);
        state.hud_ui.build_menu_open = false;
        return true;
    }

    false
}

fn draw_build_menu(state: &GameState, mouse: Vec2) {
    if !state.hud_ui.build_menu_open {
        return;
    }

    let layout = build_menu_layout(state);
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        rgba(4, 8, 12, 150),
    );
    draw_panel_frame(layout.panel, "Menu construction", mouse);
    draw_small_button(
        layout.close_rect,
        "X",
        point_in_rect(mouse, layout.close_rect),
        false,
    );

    for (tab, rect) in &layout.tab_rects {
        let active = state.hud_ui.build_tab == *tab;
        let hovered = point_in_rect(mouse, *rect);
        draw_small_button(*rect, tab.label(), hovered, active);
    }

    let entries = build_menu_entries(state.hud_ui.build_tab);
    for (idx, rect) in &layout.visible_entries {
        let Some(entry) = entries.get(*idx) else {
            continue;
        };
        let hovered = point_in_rect(mouse, *rect);
        let selected = state.hud_ui.build_menu_selected == Some(entry.selection);
        let active = build_menu_selection_is_active(state, entry.selection);
        draw_build_menu_entry_card(*rect, entry, hovered, selected, active);
    }

    draw_small_button(
        layout.prev_rect,
        "< Page",
        point_in_rect(mouse, layout.prev_rect),
        false,
    );
    draw_small_button(
        layout.next_rect,
        "Page >",
        point_in_rect(mouse, layout.next_rect),
        false,
    );
    let page_label = format!("{}/{}", layout.page + 1, layout.page_count.max(1));
    let fs = 15.0;
    let dims = measure_text(&page_label, None, fs as u16, 1.0);
    let tx = layout.footer_rect.x + layout.footer_rect.w * 0.5 - dims.width * 0.5;
    let ty = layout.footer_rect.y + layout.footer_rect.h * 0.68;
    draw_text_shadowed(
        &page_label,
        tx,
        ty,
        fs,
        rgba(232, 243, 252, 246),
        rgba(0, 0, 0, 140),
        ui_shadow_offset(fs),
    );

    draw_build_menu_details(state, &layout, mouse);
}

fn draw_build_menu_entry_card(
    rect: Rect,
    entry: &BuildMenuEntry,
    hovered: bool,
    selected: bool,
    active: bool,
) {
    let base = if selected {
        rgba(86, 142, 184, 236)
    } else if active {
        rgba(252, 208, 138, 220)
    } else if hovered {
        rgba(98, 152, 188, 225)
    } else {
        rgba(34, 50, 68, 230)
    };
    let border = if selected || active {
        ui_col_border_hi()
    } else {
        ui_col_border()
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, base);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, border);

    let (fill, shadow) = ui_text_and_shadow_for_bg(base);
    draw_text_shadowed(
        entry.label,
        rect.x + 10.0,
        rect.y + 22.0,
        18.0,
        fill,
        shadow,
        ui_shadow_offset(18.0),
    );
    draw_text_shadowed(
        entry.description,
        rect.x + 10.0,
        rect.y + 44.0,
        13.0,
        fill,
        shadow,
        ui_shadow_offset(13.0),
    );

    if let Some(cost) = build_menu_selection_cost(entry.selection) {
        let cost_line = match entry.selection {
            BuildMenuSelection::Zone(_) | BuildMenuSelection::Floor(_) => {
                format!("Cout estime: {} EUR / tuile", format_money(cost))
            }
            _ => format!("Cout: {} EUR", format_money(cost)),
        };
        draw_text_shadowed(
            &cost_line,
            rect.x + 10.0,
            rect.y + rect.h - 10.0,
            13.0,
            fill,
            shadow,
            ui_shadow_offset(13.0),
        );
    } else {
        let hint_line = format!("Raccourci: {}", entry.hint);
        draw_text_shadowed(
            &hint_line,
            rect.x + 10.0,
            rect.y + rect.h - 10.0,
            13.0,
            fill,
            shadow,
            ui_shadow_offset(13.0),
        );
    }
}

fn draw_build_menu_details(state: &GameState, layout: &BuildMenuLayout, mouse: Vec2) {
    let panel = layout.details_rect;
    let bg = rgba(12, 18, 26, 228);
    draw_rectangle(panel.x, panel.y, panel.w, panel.h, bg);
    draw_rectangle_lines(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        1.5,
        rgba(120, 171, 199, 160),
    );

    let selection = state
        .hud_ui
        .build_menu_selected
        .unwrap_or_else(|| default_build_menu_selection(state, state.hud_ui.build_tab));
    let entry = build_menu_entry_for_selection(selection);
    let title = build_menu_selection_title(selection);
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);

    draw_text_shadowed(
        "Details",
        panel.x + 10.0,
        panel.y + 22.0,
        18.0,
        fill,
        shadow,
        ui_shadow_offset(18.0),
    );
    draw_text_shadowed(
        &title,
        panel.x + 10.0,
        panel.y + 46.0,
        16.0,
        rgba(230, 242, 252, 246),
        shadow,
        ui_shadow_offset(16.0),
    );

    if let Some(entry) = entry {
        draw_text_shadowed(
            entry.description,
            panel.x + 10.0,
            panel.y + 70.0,
            14.0,
            fill,
            shadow,
            ui_shadow_offset(14.0),
        );
        draw_text_shadowed(
            &format!("Type: {}", entry.hint),
            panel.x + 10.0,
            panel.y + 92.0,
            13.0,
            rgba(190, 214, 230, 235),
            shadow,
            ui_shadow_offset(13.0),
        );
    }

    let status_line = if build_menu_selection_is_active(state, selection) {
        "Etat: actif"
    } else {
        "Etat: inactif"
    };
    draw_text_shadowed(
        status_line,
        panel.x + 10.0,
        panel.y + 118.0,
        14.0,
        rgba(220, 236, 248, 240),
        shadow,
        ui_shadow_offset(14.0),
    );

    if let Some(cost) = build_menu_selection_cost(selection) {
        let cost_text = match selection {
            BuildMenuSelection::Zone(_) | BuildMenuSelection::Floor(_) => {
                format!("Cout estime: {} EUR / tuile", format_money(cost))
            }
            _ => format!("Cout de placement: {} EUR", format_money(cost)),
        };
        draw_text_shadowed(
            &cost_text,
            panel.x + 10.0,
            panel.y + 140.0,
            14.0,
            rgba(240, 214, 150, 240),
            shadow,
            ui_shadow_offset(14.0),
        );
    } else {
        draw_text_shadowed(
            "Aucun cout direct",
            panel.x + 10.0,
            panel.y + 140.0,
            14.0,
            rgba(190, 214, 230, 235),
            shadow,
            ui_shadow_offset(14.0),
        );
    }

    let vente_line = if state.sim.sales_operational() {
        "Vente: operationnelle".to_string()
    } else {
        format!("Vente: bloquee ({})", state.sim.sales_block_reason())
    };
    draw_text_shadowed(
        &vente_line,
        panel.x + 10.0,
        panel.y + 162.0,
        13.0,
        if state.sim.sales_operational() {
            rgba(152, 224, 168, 240)
        } else {
            rgba(236, 188, 132, 238)
        },
        shadow,
        ui_shadow_offset(13.0),
    );

    draw_text_shadowed(
        state.sim.status_line(),
        panel.x + 10.0,
        panel.y + panel.h - 62.0,
        13.0,
        rgba(190, 214, 230, 235),
        shadow,
        ui_shadow_offset(13.0),
    );

    draw_small_button(
        layout.apply_rect,
        "Appliquer selection",
        point_in_rect(mouse, layout.apply_rect),
        false,
    );
}

fn draw_build_footer(panel: Rect, state: &GameState, mouse: Vec2) {
    let r = build_footer_rect(panel);
    let bg = rgba(14, 18, 24, 230);
    draw_rectangle(r.x, r.y, r.w, r.h, bg);
    draw_rectangle(r.x, r.y + r.h * 0.5, r.w, r.h * 0.5, rgba(8, 12, 18, 80));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 1.0, rgba(140, 194, 228, 160));

    let fs = 14.0;
    let (_fill, shadow) = ui_text_and_shadow_for_bg(bg);
    let text = state.sim.status_line();
    draw_text_shadowed(
        text,
        r.x + 8.0,
        r.y + r.h * 0.72,
        fs,
        ui_col_text_primary(),
        shadow,
        ui_shadow_offset(fs),
    );

    let hint = "Bouton Menu construction: ouvrir le catalogue complet";
    let dims = measure_text(hint, None, 12, 1.0);
    if dims.width + 12.0 < r.w {
        draw_text_shadowed(
            hint,
            r.x + r.w - dims.width - 8.0,
            r.y + r.h * 0.72,
            12.0,
            ui_col_text_secondary(),
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
            if state.hud_ui.info_tab != tab {
                state.hud_ui.info_scroll_y = 0.0;
            }
            state.hud_ui.info_tab = tab;
            return true;
        }
    }
    false
}

fn process_info_panel_quick_input(state: &mut GameState, panel: Rect, mouse: Vec2) -> bool {
    for (tab, rect) in info_quick_button_rects(panel) {
        if point_in_rect(mouse, rect) {
            state.hud_ui.info_tab = tab;
            state.hud_ui.info_scroll_y = 0.0;
            if selected_pawn_record(state).is_some() {
                state.hud_ui.info_window_open = true;
            }
            return true;
        }
    }
    false
}

fn info_history_line_count(state: &GameState) -> usize {
    let Some(pawn) = selected_pawn_record(state) else {
        return 0;
    };
    pawn.history
        .iter()
        .filter(|entry| {
            !matches!(
                entry.cat,
                crate::historique::LogCategorie::Deplacement
                    | crate::historique::LogCategorie::Debug
            )
        })
        .count()
}

fn info_history_viewport(inner: Rect) -> Rect {
    Rect::new(
        inner.x + 6.0,
        inner.y + 28.0,
        (inner.w - 12.0).max(1.0),
        (inner.h - 34.0).max(1.0),
    )
}

fn info_history_max_scroll(state: &GameState, panel: Rect) -> f32 {
    let inner = info_inner_rect(panel);
    let viewport_h = info_history_viewport(inner).h;
    let content_h = info_history_line_count(state) as f32 * 18.0 + 6.0;
    (content_h - viewport_h).max(0.0)
}

const INFO_SHEET_START_Y: f32 = 42.0;
const INFO_SHEET_VIEWPORT_TOP_Y: f32 = 28.0;
const INFO_SHEET_SECTION_HEADER_ADVANCE: f32 = 30.0;
const INFO_SHEET_ROW_ADVANCE: f32 = 21.0;
const INFO_SHEET_SECTION_GAP: f32 = 10.0;
const INFO_SHEET_WORKER_ADVANCE: f32 = 34.0;
const INFO_SHEET_BOTTOM_PAD: f32 = 8.0;

fn info_sheet_section_height(rows: usize) -> f32 {
    INFO_SHEET_SECTION_HEADER_ADVANCE
        + rows as f32 * INFO_SHEET_ROW_ADVANCE
        + INFO_SHEET_SECTION_GAP
}

fn info_sheet_content_height(is_worker: bool) -> f32 {
    let mut y = INFO_SHEET_START_Y;
    y += info_sheet_section_height(NeedBar::COUNT);
    y += info_sheet_section_height(SynthBar::COUNT);
    y += info_sheet_section_height(SkillBar::COUNT);
    y += info_sheet_section_height(TraitBar::COUNT);
    if is_worker {
        y += INFO_SHEET_WORKER_ADVANCE;
    }
    (y - INFO_SHEET_VIEWPORT_TOP_Y + INFO_SHEET_BOTTOM_PAD).max(0.0)
}

fn info_sheet_max_scroll(state: &GameState, panel: Rect) -> f32 {
    let inner = info_inner_rect(panel);
    let viewport_h = info_history_viewport(inner).h;
    let is_worker = selected_pawn_record(state).is_some_and(|pawn| pawn.key == PawnKey::SimWorker);
    let content_h = info_sheet_content_height(is_worker);
    (content_h - viewport_h).max(0.0)
}

fn info_panel_max_scroll(state: &GameState, panel: Rect) -> f32 {
    match state.hud_ui.info_tab {
        HudInfoTab::Fiche => info_sheet_max_scroll(state, panel),
        HudInfoTab::Historique => info_history_max_scroll(state, panel),
    }
}

fn process_info_panel_wheel(state: &mut GameState, panel: Rect, wheel_y: f32) -> bool {
    let max_scroll = info_panel_max_scroll(state, panel);
    apply_panel_wheel_scroll(&mut state.hud_ui.info_scroll_y, wheel_y, max_scroll)
}

fn draw_info_panel(state: &GameState, panel: Rect, mouse: Vec2) {
    draw_panel_frame(panel, "PERSONNAGE", mouse);

    let inner = info_compact_inner_rect(panel);
    draw_rectangle(inner.x, inner.y, inner.w, inner.h, rgba(12, 18, 26, 228));
    draw_rectangle(
        inner.x,
        inner.y + inner.h * 0.5,
        inner.w,
        inner.h * 0.5,
        rgba(8, 12, 18, 78),
    );
    draw_rectangle_lines(
        inner.x,
        inner.y,
        inner.w,
        inner.h,
        1.5,
        rgba(140, 194, 228, 150),
    );
    draw_rectangle_lines(
        inner.x + 1.0,
        inner.y + 1.0,
        inner.w - 2.0,
        inner.h - 2.0,
        1.0,
        rgba(24, 34, 44, 200),
    );

    let Some(pawn) = selected_pawn_record(state) else {
        let fs = 17.0;
        let msg = "Selectionne un personnage";
        let dims = measure_text(msg, None, fs as u16, 1.0);
        let x = inner.x + inner.w * 0.5 - dims.width * 0.5;
        let y = inner.y + inner.h * 0.5;
        draw_text_shadowed(
            msg,
            x,
            y,
            fs,
            ui_col_text_primary(),
            rgba(0, 0, 0, 160),
            ui_shadow_offset(fs),
        );
        return;
    };

    let role = match pawn.key {
        PawnKey::Player => "PATRON",
        PawnKey::Npc => "VISITEUR",
        PawnKey::SimWorker => "EMPLOYÉ",
    };
    let portrait = Rect::new(inner.x + 10.0, inner.y + 10.0, 64.0, 64.0);
    draw_rectangle(
        portrait.x,
        portrait.y,
        portrait.w,
        portrait.h,
        rgba(20, 42, 66, 230),
    );
    draw_rectangle_lines(
        portrait.x,
        portrait.y,
        portrait.w,
        portrait.h,
        1.5,
        rgba(108, 178, 230, 180),
    );
    if let Some(record) = ui_pawns::pawn_visual_record(state, pawn.key) {
        draw_character(
            record,
            CharacterRenderParams {
                center: vec2(
                    portrait.x + portrait.w * 0.5,
                    portrait.y + portrait.h * 0.72,
                ),
                scale: 1.0,
                presentation: crate::character::CharacterPresentation::Portrait,
                facing: CharacterFacing::Front,
                facing_left: false,
                is_walking: false,
                walk_cycle: 0.0,
                gesture: CharacterGesture::None,
                time: 0.0,
                debug: false,
            },
        );
    }

    let title = format!("{} - {}", pawn.name.to_uppercase(), role);
    let text_x = portrait.x + portrait.w + 14.0;
    draw_text_shadowed(
        &title,
        text_x,
        inner.y + 28.0,
        16.0,
        ui_col_text_primary(),
        rgba(0, 0, 0, 160),
        ui_shadow_offset(16.0),
    );

    draw_text_shadowed(
        "Ouvrir une fenêtre détaillée.",
        text_x,
        inner.y + 52.0,
        13.0,
        ui_col_text_secondary(),
        rgba(0, 0, 0, 140),
        ui_shadow_offset(13.0),
    );

    for (tab, rect) in info_quick_button_rects(panel) {
        let active = state.hud_ui.info_window_open && state.hud_ui.info_tab == tab;
        let hovered = point_in_rect(mouse, rect);
        draw_small_button(rect, tab.label(), hovered, active);
    }

    let rows = [
        (
            "Endurance",
            pawn.metrics.synth[SynthBar::Sante as usize],
            HeaderIcon::Shield,
        ),
        (
            "Vitesse",
            pawn.metrics.skills[SkillBar::Dexterite as usize],
            HeaderIcon::Pulse,
        ),
        (
            "Efficacité",
            pawn.metrics.skills[SkillBar::Qualite as usize],
            HeaderIcon::Check,
        ),
    ];
    let tabs = info_quick_button_rects(panel);
    let mut y = tabs
        .first()
        .map(|(_, rect)| rect.y + rect.h + 18.0)
        .unwrap_or(inner.y + 104.0);
    let bar_w = (inner.w - 158.0).max(80.0);
    let bar_x = inner.x + inner.w - bar_w - 20.0;
    for (label, value, icon) in rows {
        draw_info_metric_row(inner.x + 18.0, y, bar_x, bar_w, label, value, icon);
        y += 25.0;
    }
}

fn info_compact_inner_rect(panel: Rect) -> Rect {
    let pad = 10.0;
    let header_h = 24.0;
    Rect::new(
        panel.x + pad,
        panel.y + header_h + 10.0,
        (panel.w - pad * 2.0).max(1.0),
        (panel.h - header_h - 20.0).max(1.0),
    )
}

fn draw_info_metric_row(
    label_x: f32,
    y: f32,
    bar_x: f32,
    bar_w: f32,
    label: &str,
    value: u8,
    icon: HeaderIcon,
) {
    draw_header_icon(
        icon,
        vec2(label_x + 6.0, y - 5.0),
        13.0,
        rgba(184, 218, 242, 230),
    );
    draw_text_shadowed(
        label,
        label_x + 24.0,
        y,
        13.0,
        ui_col_text_secondary(),
        rgba(0, 0, 0, 140),
        ui_shadow_offset(13.0),
    );
    draw_meter(
        bar_x,
        y - 9.0,
        bar_w,
        8.0,
        value as f32 / 100.0,
        rgba(118, 212, 116, 240),
    );
    let pct = format!("{value}%");
    let fs = 12.0;
    draw_text_shadowed(
        &pct,
        bar_x + bar_w + 8.0,
        y,
        fs,
        ui_col_text_primary(),
        rgba(0, 0, 0, 145),
        ui_shadow_offset(fs),
    );
}

fn info_quick_button_rects(panel: Rect) -> Vec<(HudInfoTab, Rect)> {
    let inner = info_compact_inner_rect(panel);
    let y = inner.y + 88.0;
    let gap = 8.0;
    let w = ((inner.w - gap) * 0.5).max(60.0);
    let h = 34.0;
    vec![
        (HudInfoTab::Fiche, Rect::new(inner.x, y, w, h)),
        (
            HudInfoTab::Historique,
            Rect::new(inner.x + w + gap, y, w, h),
        ),
    ]
}

fn info_window_rect() -> Rect {
    let sw = screen_width();
    let sh = screen_height();
    let w = (sw * 0.44).clamp(430.0, 720.0);
    let h = (sh * 0.62).clamp(360.0, 700.0);
    Rect::new((sw - w) * 0.5, (sh - h) * 0.46, w, h)
}

fn info_window_close_rect(panel: Rect) -> Rect {
    Rect::new(panel.x + panel.w - 28.0, panel.y + 6.0, 20.0, 20.0)
}

fn draw_info_window(state: &GameState, mouse: Vec2) {
    if !state.hud_ui.info_window_open {
        return;
    }
    let panel = info_window_rect();
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        rgba(4, 8, 12, 120),
    );
    draw_panel_frame(panel, "PERSONNAGE", mouse);

    let close_rect = info_window_close_rect(panel);
    draw_small_button(close_rect, "X", point_in_rect(mouse, close_rect), false);

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
    let gap = 8.0;

    let mut x = panel.x + pad;
    let mut v = Vec::new();
    for (tab, tab_w) in [(HudInfoTab::Fiche, 166.0), (HudInfoTab::Historique, 116.0)] {
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
    Rect::new(
        panel.x + pad,
        inner_y,
        (panel.w - pad * 2.0).max(1.0),
        inner_h,
    )
}

fn selected_pawn_record(state: &GameState) -> Option<&PawnCard> {
    state
        .pawn_ui
        .selected
        .and_then(|k| state.pawns.iter().find(|p| p.key == k))
}

fn need_color(need: NeedBar) -> Color {
    match need {
        NeedBar::Manger => rgba(140, 200, 150, 230),
        NeedBar::Boire => rgba(120, 170, 210, 230),
        NeedBar::Dormir => rgba(200, 200, 120, 230),
        NeedBar::Toilettes => rgba(180, 140, 210, 230),
        NeedBar::Hygiene => rgba(130, 210, 200, 230),
        NeedBar::Divertissement => rgba(220, 170, 120, 230),
        NeedBar::Social => rgba(210, 180, 120, 230),
        NeedBar::Confort => rgba(170, 210, 160, 230),
        NeedBar::Calme => rgba(130, 200, 235, 230),
        NeedBar::Douleur => rgba(220, 120, 120, 230),
    }
}

fn skill_color(skill: SkillBar) -> Color {
    match skill {
        SkillBar::Mecanique => rgba(240, 182, 102, 235),
        SkillBar::Electricite => rgba(238, 226, 120, 235),
        SkillBar::Dexterite => rgba(130, 210, 210, 235),
        SkillBar::Qualite => rgba(120, 196, 232, 235),
        SkillBar::Force => rgba(220, 140, 110, 235),
        SkillBar::Logistique => rgba(120, 190, 150, 235),
        SkillBar::Intelligence => rgba(180, 168, 236, 235),
        SkillBar::Planification => rgba(150, 182, 240, 235),
        SkillBar::Sociabilite => rgba(238, 176, 168, 235),
        SkillBar::Management => rgba(220, 198, 136, 235),
        SkillBar::Securite => rgba(176, 212, 138, 235),
        SkillBar::Nettoyage => rgba(130, 218, 196, 235),
        SkillBar::Diagnostic => rgba(182, 168, 248, 235),
    }
}

fn trait_color(trait_bar: TraitBar) -> Color {
    match trait_bar {
        TraitBar::Motivation => rgba(240, 196, 108, 235),
        TraitBar::Discipline => rgba(146, 196, 236, 235),
        TraitBar::Fiabilite => rgba(128, 214, 168, 235),
        TraitBar::Patience => rgba(170, 206, 170, 235),
        TraitBar::Courage => rgba(224, 142, 132, 235),
        TraitBar::Empathie => rgba(218, 176, 224, 235),
    }
}

fn draw_info_section_title(y: f32, viewport: Rect, x: f32, title: &str, shadow: Color) {
    if y + 28.0 >= viewport.y && y <= viewport.y + viewport.h {
        draw_text_shadowed(
            title,
            x,
            y + 22.0,
            18.0,
            rgba(210, 225, 236, 240),
            shadow,
            ui_shadow_offset(18.0),
        );
    }
}

fn draw_info_sheet(state: &GameState, panel: Rect, mouse: Vec2) {
    let inner = info_inner_rect(panel);
    let bg = rgba(16, 22, 30, 220);
    draw_rectangle(inner.x, inner.y, inner.w, inner.h, bg);
    draw_rectangle_lines(
        inner.x,
        inner.y,
        inner.w,
        inner.h,
        1.5,
        rgba(120, 171, 199, 140),
    );

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

    let title_fs = 22.0;
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

    let viewport = info_history_viewport(inner);
    let max_scroll = info_sheet_max_scroll(state, panel);
    let scroll_y = state.hud_ui.info_scroll_y.clamp(0.0, max_scroll);
    let mut y = inner.y + INFO_SHEET_START_Y - scroll_y;
    let bar_w = inner.w - 20.0;
    let label_x = inner.x + 10.0;

    draw_info_section_title(y, viewport, label_x, "Besoins", shadow);
    y += INFO_SHEET_SECTION_HEADER_ADVANCE;
    for need in NeedBar::ALL {
        let v = pawn.metrics.needs[need as usize] as f32 / 100.0;
        if y + 20.0 >= viewport.y && y <= viewport.y + viewport.h {
            draw_labeled_bar(
                label_x,
                y,
                bar_w,
                14.0,
                need.label(),
                v,
                need_color(need),
                bg,
            );
        }
        y += INFO_SHEET_ROW_ADVANCE;
    }
    y += INFO_SHEET_SECTION_GAP;

    draw_info_section_title(y, viewport, label_x, "Etat general", shadow);
    y += INFO_SHEET_SECTION_HEADER_ADVANCE;
    for synth in SynthBar::ALL {
        let v = pawn.metrics.synth[synth as usize] as f32 / 100.0;
        let col = match synth {
            SynthBar::Sante => rgba(120, 210, 140, 240),
            SynthBar::Fatigue => rgba(210, 180, 120, 240),
            SynthBar::Moral => rgba(120, 170, 230, 240),
        };
        if y + 20.0 >= viewport.y && y <= viewport.y + viewport.h {
            draw_labeled_bar(label_x, y, bar_w, 14.0, synth.label(), v, col, bg);
        }
        y += INFO_SHEET_ROW_ADVANCE;
    }
    y += INFO_SHEET_SECTION_GAP;

    draw_info_section_title(y, viewport, label_x, "Competences", shadow);
    y += INFO_SHEET_SECTION_HEADER_ADVANCE;
    for skill in SkillBar::ALL {
        let v = pawn.metrics.skills[skill as usize] as f32 / 100.0;
        if y + 20.0 >= viewport.y && y <= viewport.y + viewport.h {
            draw_labeled_bar(
                label_x,
                y,
                bar_w,
                14.0,
                skill.label(),
                v,
                skill_color(skill),
                bg,
            );
        }
        y += INFO_SHEET_ROW_ADVANCE;
    }
    y += INFO_SHEET_SECTION_GAP;

    draw_info_section_title(y, viewport, label_x, "Traits", shadow);
    y += INFO_SHEET_SECTION_HEADER_ADVANCE;
    for trait_bar in TraitBar::ALL {
        let v = pawn.metrics.traits[trait_bar as usize] as f32 / 100.0;
        if y + 20.0 >= viewport.y && y <= viewport.y + viewport.h {
            draw_labeled_bar(
                label_x,
                y,
                bar_w,
                14.0,
                trait_bar.label(),
                v,
                trait_color(trait_bar),
                bg,
            );
        }
        y += INFO_SHEET_ROW_ADVANCE;
    }
    y += INFO_SHEET_SECTION_GAP;

    if pawn.key == PawnKey::SimWorker {
        let fs = 16.0;
        let activity = state
            .sim
            .primary_agent_current_job_id()
            .and_then(|id| state.sim.job_brief(id))
            .unwrap_or_else(|| "Inactif".to_string());
        let t = format!("Activite: {}", activity);
        if y + 24.0 >= viewport.y && y <= viewport.y + viewport.h {
            draw_text_shadowed(
                &t,
                label_x,
                y + 26.0,
                fs,
                rgba(220, 235, 242, 240),
                shadow,
                ui_shadow_offset(fs),
            );
        }
    }

    draw_vertical_scrollbar(
        viewport,
        info_sheet_content_height(pawn.key == PawnKey::SimWorker),
        scroll_y,
    );

    let _ = mouse;
}

#[allow(clippy::too_many_arguments)]
fn draw_labeled_bar(x: f32, y: f32, w: f32, h: f32, label: &str, v: f32, col: Color, bg: Color) {
    let fs = 14.0;
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);
    draw_text_shadowed(label, x, y + h, fs, fill, shadow, ui_shadow_offset(fs));
    let bar_x = x + 96.0;
    let bar_w = (w - 96.0).max(1.0);
    draw_meter(bar_x, y + 3.0, bar_w, h - 6.0, v, col);
}

fn draw_info_history(state: &GameState, panel: Rect, mouse: Vec2) {
    let inner = info_inner_rect(panel);
    let bg = rgba(12, 18, 26, 228);
    draw_rectangle(inner.x, inner.y, inner.w, inner.h, bg);
    draw_rectangle_lines(
        inner.x,
        inner.y,
        inner.w,
        inner.h,
        1.5,
        rgba(120, 171, 199, 140),
    );

    let Some(pawn) = selected_pawn_record(state) else {
        return;
    };

    let title_fs = 16.0;
    let (fill, shadow) = ui_text_and_shadow_for_bg(bg);
    draw_text_shadowed(
        "Evenements principaux",
        inner.x + 10.0,
        inner.y + 20.0,
        title_fs,
        fill,
        shadow,
        ui_shadow_offset(title_fs),
    );

    let viewport = info_history_viewport(inner);
    let max_scroll = info_history_max_scroll(state, panel);
    let scroll_y = state.hud_ui.info_scroll_y.clamp(0.0, max_scroll);
    let line_fs = 14.0;
    let row_h = 18.0;
    let mut y = viewport.y + 4.0 - scroll_y;

    for entry in pawn.history.iter().rev().filter(|entry| {
        !matches!(
            entry.cat,
            crate::historique::LogCategorie::Deplacement | crate::historique::LogCategorie::Debug
        )
    }) {
        if y + row_h < viewport.y {
            y += row_h;
            continue;
        }
        if y > viewport.y + viewport.h {
            break;
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
        y += row_h;
    }

    let content_h = info_history_line_count(state) as f32 * row_h + 6.0;
    draw_vertical_scrollbar(viewport, content_h, scroll_y);

    if point_in_rect(mouse, inner) {
        draw_rectangle_lines(
            inner.x + 2.0,
            inner.y + 2.0,
            inner.w - 4.0,
            inner.h - 4.0,
            1.0,
            rgba(252, 208, 138, 140),
        );
    }
}

fn format_clock_hhmm(t_sim_s: f64) -> String {
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
    state: &mut GameState,
    panel: Rect,
    mouse: Vec2,
    map_view: Rect,
    world_camera: &Camera2D,
) {
    draw_panel_frame(panel, "MINI-CARTE", mouse);

    let inner = minimap_inner_rect(panel);
    let bg = rgba(10, 14, 18, 240);
    draw_rectangle(inner.x, inner.y, inner.w, inner.h, bg);
    draw_rectangle(
        inner.x,
        inner.y + inner.h * 0.5,
        inner.w,
        inner.h * 0.5,
        rgba(8, 12, 18, 90),
    );
    draw_rectangle_lines(
        inner.x,
        inner.y,
        inner.w,
        inner.h,
        1.5,
        rgba(140, 194, 228, 150),
    );
    draw_rectangle_lines(
        inner.x + 1.0,
        inner.y + 1.0,
        inner.w - 2.0,
        inner.h - 2.0,
        1.0,
        rgba(24, 34, 44, 200),
    );

    let world_w = state.world.w as f32 * TILE_SIZE;
    let world_h = state.world.h as f32 * TILE_SIZE;
    if world_w <= 1.0 || world_h <= 1.0 {
        return;
    }

    if let Some((cache_w, cache_h)) = minimap_cache_dimensions(&state.world) {
        let cache_invalid = state.minimap_cache.texture.is_none()
            || state.minimap_cache.dirty
            || state.minimap_cache.world_revision != state.world.revision
            || state.minimap_cache.width_px != cache_w
            || state.minimap_cache.height_px != cache_h;
        if cache_invalid {
            rebuild_minimap_texture_cache(&state.world, &mut state.minimap_cache, cache_w, cache_h);
        }
        if let Some(texture) = state.minimap_cache.texture.as_ref() {
            draw_texture_ex(
                texture,
                inner.x,
                inner.y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(inner.w, inner.h)),
                    ..Default::default()
                },
            );
        }
    }

    for b in state.sim.blocks() {
        let (tx, ty) = b.origin_tile;
        let x = inner.x + (tx as f32 / state.world.w as f32) * inner.w;
        let y = inner.y + (ty as f32 / state.world.h as f32) * inner.h;
        draw_rectangle(x, y, 4.0, 4.0, rgba(252, 208, 138, 220));
    }

    let pawn_points: [Vec2; 3] = [
        state.player.pos,
        state.npc.pos,
        tile_center(state.sim.primary_agent_tile()),
    ];
    let pawn_colors: [Color; 3] = [
        rgba(120, 220, 160, 240),
        rgba(220, 170, 120, 240),
        rgba(150, 200, 250, 240),
    ];
    for i in 0..pawn_points.len() {
        let pos = pawn_points[i];
        let col = pawn_colors[i];
        let nx = (pos.x / world_w).clamp(0.0, 1.0);
        let ny = (pos.y / world_h).clamp(0.0, 1.0);
        let px = inner.x + nx * inner.w;
        let py = inner.y + ny * inner.h;
        draw_circle(px, py, 3.0, col);
        draw_circle_lines(px, py, 3.2, 1.0, rgba(0, 0, 0, 120));
    }
    if let Some(papa) = state.papa.pnj() {
        let nx = (papa.pos.x / world_w).clamp(0.0, 1.0);
        let ny = (papa.pos.y / world_h).clamp(0.0, 1.0);
        let px = inner.x + nx * inner.w;
        let py = inner.y + ny * inner.h;
        draw_circle(px, py, 3.2, rgba(212, 246, 190, 244));
        draw_circle_lines(px, py, 3.4, 1.1, rgba(24, 44, 30, 180));
    }

    ensure_default_material();
    let a =
        camera_screen_to_world_in_view_rect(world_camera, vec2(map_view.x, map_view.y), map_view);
    let b = camera_screen_to_world_in_view_rect(
        world_camera,
        vec2(map_view.x + map_view.w, map_view.y),
        map_view,
    );
    let c = camera_screen_to_world_in_view_rect(
        world_camera,
        vec2(map_view.x + map_view.w, map_view.y + map_view.h),
        map_view,
    );
    let d = camera_screen_to_world_in_view_rect(
        world_camera,
        vec2(map_view.x, map_view.y + map_view.h),
        map_view,
    );

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

fn minimap_cache_dimensions(world: &World) -> Option<(u16, u16)> {
    if world.w <= 0 || world.h <= 0 {
        return None;
    }
    let width = ((world.w + MINIMAP_CACHE_STRIDE - 1) / MINIMAP_CACHE_STRIDE)
        .clamp(1, u16::MAX as i32) as u16;
    let height = ((world.h + MINIMAP_CACHE_STRIDE - 1) / MINIMAP_CACHE_STRIDE)
        .clamp(1, u16::MAX as i32) as u16;
    Some((width, height))
}

fn minimap_tile_color(kind: Tile) -> Color {
    if tile_is_wall(kind) {
        rgba(120, 150, 180, 180)
    } else if matches!(kind, Tile::FloorMetal) {
        rgba(86, 112, 128, 140)
    } else if matches!(kind, Tile::FloorWood) {
        rgba(128, 78, 42, 158)
    } else if matches!(kind, Tile::FloorMoss) {
        rgba(58, 120, 62, 154)
    } else if matches!(kind, Tile::FloorSand) {
        rgba(118, 98, 58, 142)
    } else {
        rgba(56, 106, 54, 138)
    }
}

fn rebuild_minimap_texture_cache(
    world: &World,
    cache: &mut MinimapTextureCache,
    width: u16,
    height: u16,
) {
    let mut image = Image::gen_image_color(width, height, rgba(10, 14, 18, 240));
    for py in 0..height {
        for px in 0..width {
            let tx = ((px as i32) * MINIMAP_CACHE_STRIDE).min(world.w - 1);
            let ty = ((py as i32) * MINIMAP_CACHE_STRIDE).min(world.h - 1);
            image.set_pixel(px as u32, py as u32, minimap_tile_color(world.get(tx, ty)));
        }
    }

    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);
    cache.texture = Some(texture);
    cache.dirty = false;
    cache.world_revision = world.revision;
    cache.width_px = width;
    cache.height_px = height;
}

fn format_money(amount: f64) -> String {
    let rounded = amount.round() as i64;
    format_int_fr(rounded)
}

fn format_clock_hhmmss(t_sim_s: f64) -> String {
    let total = if t_sim_s.is_finite() {
        t_sim_s.max(0.0).floor() as u64
    } else {
        0
    };
    let h = (total / 3600) % 24;
    let m = (total / 60) % 60;
    let s = total % 60;
    format!("{h:02}:{m:02}:{s:02}")
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

    fn assert_color_close(actual: Color, expected: Color) {
        assert!((actual.r - expected.r).abs() < 0.0001);
        assert!((actual.g - expected.g).abs() < 0.0001);
        assert!((actual.b - expected.b).abs() < 0.0001);
        assert!((actual.a - expected.a).abs() < 0.0001);
    }

    #[test]
    fn mix_color_clamps_factor_to_bounds() {
        let a = rgba(10, 20, 30, 40);
        let b = rgba(110, 120, 130, 140);

        let low = mix_color(a, b, -2.0);
        assert!((low.r - a.r).abs() < 0.0001);
        assert!((low.g - a.g).abs() < 0.0001);
        assert!((low.b - a.b).abs() < 0.0001);
        assert!((low.a - a.a).abs() < 0.0001);

        let high = mix_color(a, b, 2.0);
        assert!((high.r - b.r).abs() < 0.0001);
        assert!((high.g - b.g).abs() < 0.0001);
        assert!((high.b - b.b).abs() < 0.0001);
        assert!((high.a - b.a).abs() < 0.0001);
    }

    #[test]
    fn mix_color_blends_midpoint_linearly() {
        let a = rgba(0, 100, 200, 50);
        let b = rgba(200, 0, 100, 250);
        let mid = mix_color(a, b, 0.5);

        assert!((mid.r - (a.r + b.r) * 0.5).abs() < 0.0001);
        assert!((mid.g - (a.g + b.g) * 0.5).abs() < 0.0001);
        assert!((mid.b - (a.b + b.b) * 0.5).abs() < 0.0001);
        assert!((mid.a - (a.a + b.a) * 0.5).abs() < 0.0001);
    }

    #[test]
    fn minimap_cache_dimensions_follow_sampling_stride() {
        let world = World {
            w: 7,
            h: 5,
            tiles: vec![Tile::Floor; 35],
            revision: 0,
        };

        assert_eq!(minimap_cache_dimensions(&world), Some((4, 3)));
    }

    #[test]
    fn minimap_cache_invalidates_when_world_revision_changes() {
        let mut world = World {
            w: 6,
            h: 4,
            tiles: vec![Tile::Floor; 24],
            revision: 0,
        };
        let before = world.revision;

        world.set(2, 0, Tile::WallSteel);

        assert_ne!(world.revision, before);
    }

    #[test]
    fn minimap_tile_color_groups_wall_tiles() {
        assert_color_close(
            minimap_tile_color(Tile::WallSteel),
            rgba(120, 150, 180, 180),
        );
        assert_color_close(minimap_tile_color(Tile::FloorMoss), rgba(58, 120, 62, 154));
    }

    #[test]
    fn rows_for_count_handles_edge_cases() {
        assert_eq!(rows_for_count(0, 2), 0);
        assert_eq!(rows_for_count(1, 2), 1);
        assert_eq!(rows_for_count(2, 2), 1);
        assert_eq!(rows_for_count(3, 2), 2);
        assert_eq!(rows_for_count(5, 2), 3);
    }

    #[test]
    fn wheel_scroll_is_clamped_to_bounds() {
        let mut scroll = 0.0;
        let changed = apply_panel_wheel_scroll(&mut scroll, -3.0, 60.0);
        assert!(changed);
        assert!(scroll > 0.0);

        let _ = apply_panel_wheel_scroll(&mut scroll, -99.0, 60.0);
        assert_eq!(scroll, 60.0);

        let _ = apply_panel_wheel_scroll(&mut scroll, 99.0, 60.0);
        assert_eq!(scroll, 0.0);
    }

    #[test]
    fn info_sheet_content_height_increases_for_worker_activity_line() {
        let base = info_sheet_content_height(false);
        let worker = info_sheet_content_height(true);
        assert!(worker > base);
        assert!((worker - base) >= 20.0);
    }

    #[test]
    fn info_sheet_content_height_includes_all_characteristics() {
        let base = info_sheet_content_height(false);
        let expected_rows = NeedBar::COUNT + SynthBar::COUNT + SkillBar::COUNT + TraitBar::COUNT;
        let min_expected = INFO_SHEET_START_Y
            + 4.0 * INFO_SHEET_SECTION_HEADER_ADVANCE
            + expected_rows as f32 * INFO_SHEET_ROW_ADVANCE
            + 4.0 * INFO_SHEET_SECTION_GAP
            - INFO_SHEET_VIEWPORT_TOP_Y
            + INFO_SHEET_BOTTOM_PAD;
        assert!(base >= min_expected);
    }

    #[test]
    fn build_menu_page_count_handles_empty_and_non_empty() {
        assert_eq!(build_menu_page_count(0, 8), 1);
        assert_eq!(build_menu_page_count(1, 8), 1);
        assert_eq!(build_menu_page_count(8, 8), 1);
        assert_eq!(build_menu_page_count(9, 8), 2);
    }

    #[test]
    fn build_menu_page_range_clamps_requested_page() {
        let (page, pages, start, end) = build_menu_page_range(19, 6, 99);
        assert_eq!(pages, 4);
        assert_eq!(page, 3);
        assert_eq!(start, 18);
        assert_eq!(end, 19);
    }

    #[test]
    fn build_menu_catalog_is_present_for_all_categories() {
        assert!(!build_menu_entries(HudBuildTab::Blocs).is_empty());
        assert!(!build_menu_entries(HudBuildTab::Zones).is_empty());
        assert!(!build_menu_entries(HudBuildTab::Sols).is_empty());
        assert!(!build_menu_entries(HudBuildTab::Outils).is_empty());
    }

    #[test]
    fn bottom_panel_widths_always_fit_screen() {
        for sw in [120.0_f32, 240.0, 320.0, 640.0, 800.0, 1024.0, 1600.0] {
            let (pawn_w, build_w, info_w, phone_w, minimap_w) = compute_bottom_panel_widths(sw);
            let sum = pawn_w + build_w + info_w + phone_w + minimap_w;
            assert!(pawn_w > 0.0);
            assert!(build_w > 0.0);
            assert!(info_w > 0.0);
            assert!(phone_w > 0.0);
            assert!(minimap_w > 0.0);
            assert!(
                sum <= sw + 0.001,
                "sum({sum}) should fit sw({sw}); widths=({pawn_w},{build_w},{info_w},{phone_w},{minimap_w})"
            );
        }
    }

    #[test]
    fn top_strip_layout_keeps_all_cards_inside_viewport() {
        for sw in [640.0_f32, 800.0, 1024.0, 1228.0, 1536.0, 1920.0] {
            let viewport = Rect::new(0.0, 0.0, sw, 78.0);
            let layout = compute_top_strip_layout(viewport, 1.0, 5);
            let mut cards = vec![layout.brand, layout.nav, layout.simulation];
            cards.extend(layout.metrics.iter().copied());

            for card in &cards {
                assert!(card.w > 0.0, "card width should stay positive for sw={sw}");
                assert!(
                    card.x >= viewport.x - 0.001,
                    "card {card:?} should not overflow left for sw={sw}"
                );
                assert!(
                    card.x + card.w <= viewport.x + viewport.w + 0.001,
                    "card {card:?} should not overflow right for sw={sw}"
                );
            }

            assert!(layout.brand.x + layout.brand.w <= layout.nav.x + 0.001);
            assert!(layout.nav.x + layout.nav.w <= layout.simulation.x + 0.001);
            let first_metric = layout.metrics.first().expect("metric card");
            assert!(layout.simulation.x + layout.simulation.w <= first_metric.x + 0.001);
        }
    }
}
