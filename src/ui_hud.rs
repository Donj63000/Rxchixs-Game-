use super::*;
use crate::sim::{BlockKind, BuildFloorKind, ZoneKind};
use std::cell::RefCell;

thread_local! {
    static INITIAL_RAW_MATERIAL_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
}

pub(crate) fn set_initial_raw_material_texture(texture: Option<Texture2D>) {
    INITIAL_RAW_MATERIAL_TEXTURE.with(|slot| {
        let prepared = texture;
        if let Some(tex) = prepared.as_ref() {
            tex.set_filter(FilterMode::Nearest);
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
            HudInfoTab::Fiche => "Caracteristiques",
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

    let bar_h = (220.0 * scale).clamp(180.0, 270.0);
    let bar_rect = Rect::new(0.0, (sh - bar_h).max(0.0), sw, bar_h);

    let top_strip_h = (40.0 * scale).clamp(32.0, 48.0);
    let top_strip_rect = Rect::new(bar_rect.x, bar_rect.y, bar_rect.w, top_strip_h);

    let content_y = top_strip_rect.y + top_strip_rect.h;
    let content_h = (bar_rect.h - top_strip_rect.h).max(1.0);

    let (pawn_w, build_w, info_w, phone_w, minimap_w) = compute_bottom_panel_widths(sw);

    let pawn_panel = Rect::new(bar_rect.x, content_y, pawn_w, content_h);
    let build_panel = Rect::new(pawn_panel.x + pawn_panel.w, content_y, build_w, content_h);
    let info_panel = Rect::new(build_panel.x + build_panel.w, content_y, info_w, content_h);
    let telephone_panel = Rect::new(info_panel.x + info_panel.w, content_y, phone_w, content_h);
    let minimap_panel = Rect::new(
        telephone_panel.x + telephone_panel.w,
        content_y,
        minimap_w,
        content_h,
    );

    HudLayout {
        bar_rect,
        top_strip_rect,
        pawn_panel,
        build_panel,
        info_panel,
        telephone_panel,
        minimap_panel,
    }
}

fn compute_bottom_panel_widths(sw: f32) -> (f32, f32, f32, f32, f32) {
    let sw = sw.max(1.0);
    let mut pawn_w = (sw * 0.26).clamp(52.0, 420.0);
    let mut phone_w = (sw * 0.08).clamp(28.0, 118.0);
    let mut minimap_w = (sw * 0.19).clamp(56.0, 340.0);
    let mut info_w = (sw * 0.26).clamp(70.0, 470.0);
    let min_build_w = (sw * 0.14).clamp(20.0, 260.0);

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

    let over_bar = point_in_rect(mouse, layout.bar_rect);
    let over_phone =
        telephone::telephone_panel_contains_mouse(state, layout.telephone_panel, mouse);
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
        if point_in_rect(mouse, layout.top_strip_rect)
            && process_top_strip_input(state, layout.top_strip_rect, mouse)
        {
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
    begin_ui_pass();
    draw_bar_background(layout.bar_rect, time);

    draw_top_strip(state, layout.top_strip_rect, mouse);

    draw_pawn_panel(state, layout.pawn_panel, mouse, time);
    draw_build_panel(state, layout.build_panel, mouse);
    draw_info_panel(state, layout.info_panel, mouse);
    telephone::draw_telephone_panel(state, layout.telephone_panel, mouse, time);
    draw_minimap_panel(state, layout.minimap_panel, mouse, map_view, world_camera);
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
    rgba(82, 182, 232, 220)
}

fn ui_col_border_hi() -> Color {
    rgba(196, 244, 255, 250)
}

fn ui_col_accent() -> Color {
    rgba(255, 196, 106, 248)
}

fn ui_col_surface() -> Color {
    rgba(14, 28, 48, 236)
}

fn ui_col_surface_hi() -> Color {
    rgba(26, 46, 72, 240)
}

fn ui_col_text_primary() -> Color {
    rgba(242, 250, 255, 248)
}

fn ui_col_text_secondary() -> Color {
    rgba(196, 220, 244, 244)
}

fn ui_col_glow_cyan() -> Color {
    rgba(84, 218, 255, 255)
}

fn ui_col_glow_teal() -> Color {
    rgba(112, 248, 206, 255)
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

    let base_top = if hovered {
        ui_col_surface_hi()
    } else {
        mix_color(ui_col_surface_hi(), ui_col_surface(), 0.55)
    };
    let base_bottom = if hovered {
        mix_color(ui_col_surface(), rgba(6, 12, 24, 255), 0.40)
    } else {
        mix_color(ui_col_surface(), rgba(4, 8, 18, 255), 0.44)
    };
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
    let header_top = if hovered {
        rgba(84, 118, 154, 248)
    } else {
        rgba(74, 106, 142, 246)
    };
    let header_bottom = if hovered {
        rgba(42, 64, 88, 248)
    } else {
        rgba(36, 58, 82, 246)
    };
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
    draw_text_shadowed(
        title,
        rect.x + 12.0,
        rect.y + 17.5,
        fs,
        if hovered { ui_col_text_primary() } else { fill },
        shadow,
        ui_shadow_offset(fs),
    );
}

fn draw_top_strip(state: &GameState, rect: Rect, mouse: Vec2) {
    let top = rgba(18, 44, 76, 252);
    let bottom = rgba(10, 24, 42, 252);
    draw_vertical_gradient(rect, top, bottom, 14);
    draw_rectangle(
        rect.x,
        rect.y + rect.h * 0.04,
        rect.w,
        rect.h * 0.28,
        with_alpha(ui_col_glow_cyan(), 0.12),
    );
    draw_rectangle(
        rect.x,
        rect.y + rect.h * 0.68,
        rect.w,
        rect.h * 0.32,
        with_alpha(rgba(0, 0, 0, 255), 0.24),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        2.0,
        with_alpha(ui_col_border(), 0.88),
    );
    draw_rectangle(
        rect.x + 1.0,
        rect.y + 1.0,
        rect.w - 2.0,
        2.0,
        with_alpha(ui_col_border_hi(), 0.44),
    );

    let pad = 10.0;
    let y = rect.y + rect.h * 0.72;
    let fs = (rect.h * 0.46).clamp(14.0, 20.0);
    let (fill, shadow) = ui_text_and_shadow_for_bg(bottom);

    let time_label = format!(
        "Heure {}  J{}",
        state.sim.clock.format_hhmm(),
        state.sim.clock.day_index() + 1
    );
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
    let _revenue = state.sim.revenue_total();
    let _cost = state.sim.cost_total();
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
        rgba(84, 188, 242, 242),
        mouse,
        false,
    ) + 10.0;

    x = draw_stat_pill(
        Rect::new(x, pill_y, 170.0, pill_h),
        "Cadence",
        &format!("{:.1}/h", cadence),
        rgba(96, 228, 178, 236),
        mouse,
        false,
    ) + 10.0;

    x = draw_stat_pill(
        Rect::new(x, pill_y, 160.0, pill_h),
        "Fiabilite",
        &format!("{:.0}%", (otif * 100.0).clamp(0.0, 999.0)),
        rgba(208, 230, 110, 236),
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

    let btn_w = (36.0_f32).max(rect.h * 0.82);
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
    let btn_w = (36.0_f32).max(rect.h * 0.82);
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

fn draw_stat_pill(
    rect: Rect,
    label: &str,
    value: &str,
    accent: Color,
    mouse: Vec2,
    euro: bool,
) -> f32 {
    let hovered = point_in_rect(mouse, rect);
    let pad = 10.0;
    draw_panel_drop_shadow(rect, if hovered { 0.28 } else { 0.20 });

    let tint = with_alpha(accent, if hovered { 0.42 } else { 0.30 });
    let top = if hovered {
        mix_color(ui_col_surface_hi(), tint, 0.54)
    } else {
        mix_color(ui_col_surface(), tint, 0.44)
    };
    let bottom = mix_color(top, rgba(4, 8, 20, 255), if hovered { 0.46 } else { 0.52 });
    draw_vertical_gradient(rect, top, bottom, 12);
    draw_rectangle(
        rect.x + 1.0,
        rect.y + rect.h * 0.52,
        (rect.w - 2.0).max(0.0),
        (rect.h * 0.48).max(0.0),
        with_alpha(rgba(0, 0, 0, 255), 0.24),
    );
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        1.8,
        if hovered {
            ui_col_border_hi()
        } else {
            with_alpha(ui_col_border(), 0.92)
        },
    );
    draw_rectangle_lines(
        rect.x + 1.0,
        rect.y + 1.0,
        rect.w - 2.0,
        rect.h - 2.0,
        1.0,
        rgba(9, 18, 30, 208),
    );

    let accent_bar_w = if hovered { 8.0 } else { 7.0 };
    draw_rectangle(
        rect.x,
        rect.y,
        accent_bar_w,
        rect.h,
        with_alpha(accent, 0.95),
    );
    draw_rectangle(
        rect.x + accent_bar_w,
        rect.y,
        (rect.w - accent_bar_w).max(0.0),
        2.0,
        with_alpha(ui_col_border_hi(), if hovered { 0.46 } else { 0.32 }),
    );
    draw_circle(
        rect.x + rect.w - 9.5,
        rect.y + 8.0,
        2.4,
        with_alpha(accent, if hovered { 0.92 } else { 0.72 }),
    );

    let fs = (rect.h * 0.44).clamp(12.0, 18.0);
    let value_fs = (rect.h * 0.54).clamp(13.0, 19.0);
    let (_fill, shadow) = ui_text_and_shadow_for_bg(bottom);

    draw_text_shadowed(
        label,
        rect.x + 10.0,
        rect.y + rect.h * 0.58,
        fs,
        ui_col_text_secondary(),
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
        ui_col_text_primary(),
        shadow,
        ui_shadow_offset(value_fs),
    );

    rect.x + rect.w
}

fn draw_euro_icon_shadowed(
    x: f32,
    baseline_y: f32,
    h: f32,
    color: Color,
    shadow: Color,
    shadow_off: Vec2,
) {
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
    let card_w = (160.0 * scale).clamp(132.0, 180.0);
    let gap = (10.0 * scale).clamp(8.0, 14.0);
    let cols = (((inner.w + gap) / (card_w + gap)).floor() as usize).max(1);
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
    draw_panel_frame(panel, "Equipe", mouse);

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
        let follow_rect = Rect::new(rect.x + rect.w - btn - 6.0, rect.y + 6.0, btn, btn);
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
        slot.rect.w - 72.0,
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
    draw_small_button(slot.follow_rect, "F", follow_hover, follow_active);

    if let Some(pawn) = pawn {
        let bar_w = slot.rect.w - 54.0;
        let bar_x = slot.rect.x + 44.0;
        let bar_y = slot.rect.y + slot.rect.h - 14.0;
        let hp = pawn.metrics.synth[SynthBar::Sante as usize] as f32 / 100.0;
        draw_meter(bar_x, bar_y, bar_w, 7.0, hp, rgba(124, 226, 156, 240));
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
    false
}

fn draw_build_panel(state: &GameState, panel: Rect, mouse: Vec2) {
    draw_panel_frame(panel, "Construction", mouse);
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
        "Fermer menu"
    } else {
        "Menu construction"
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

    draw_build_footer(panel, state, mouse);
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
    draw_panel_frame(panel, "Personnage", mouse);

    let inner = info_inner_rect(panel);
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
        PawnKey::Player => "Patron",
        PawnKey::Npc => "Visiteur",
        PawnKey::SimWorker => "Employe",
    };
    let title = format!("{} - {}", pawn.name, role);
    draw_text_shadowed(
        &title,
        inner.x + 10.0,
        inner.y + 22.0,
        16.0,
        ui_col_text_primary(),
        rgba(0, 0, 0, 160),
        ui_shadow_offset(16.0),
    );

    draw_text_shadowed(
        "Ouvrir une fenetre detaillee :",
        inner.x + 10.0,
        inner.y + 44.0,
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
}

fn info_quick_button_rects(panel: Rect) -> Vec<(HudInfoTab, Rect)> {
    let inner = info_inner_rect(panel);
    let y = inner.y + 56.0;
    let gap = 8.0;
    let w = ((inner.w - gap) * 0.5).max(60.0);
    let h = 28.0;
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
    draw_panel_frame(panel, "Personnage", mouse);

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
    state: &GameState,
    panel: Rect,
    mouse: Vec2,
    map_view: Rect,
    world_camera: &Camera2D,
) {
    draw_panel_frame(panel, "Mini-carte", mouse);

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

    let stride = 2;
    for ty in (0..state.world.h).step_by(stride) {
        for tx in (0..state.world.w).step_by(stride) {
            let kind = state.world.get(tx, ty);
            let col = if tile_is_wall(kind) {
                rgba(120, 150, 180, 180)
            } else if matches!(kind, Tile::FloorMetal) {
                rgba(86, 112, 128, 140)
            } else if matches!(kind, Tile::FloorWood) {
                rgba(120, 96, 78, 132)
            } else if matches!(kind, Tile::FloorMoss) {
                rgba(74, 118, 86, 132)
            } else if matches!(kind, Tile::FloorSand) {
                rgba(130, 112, 84, 132)
            } else {
                rgba(60, 80, 100, 110)
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
}
