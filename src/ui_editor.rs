use super::*;
use crate::rendu::theme::{
    feedback_theme, mix_color, ui_panel_fill, ui_panel_header_fill, ui_theme,
};

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum EditorLeftTab {
    Placer,
    Zones,
    Outils,
    Fichiers,
}

#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum EditorRightTab {
    Survol,
    Selection,
    Carte,
    Validation,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum SplitDrag {
    Left,
    Right,
}

#[derive(Copy, Clone)]
pub(crate) struct EditorContextMenu {
    pub anchor_screen: Vec2,
    pub tile: (i32, i32),
}

#[derive(Serialize, Deserialize)]
struct EditorUiSavedState {
    ui_scale: f32,
    left_panel_ratio: f32,
    right_panel_ratio: f32,
    left_tab: EditorLeftTab,
    right_tab: EditorRightTab,
    high_contrast: bool,
}

impl Default for EditorUiSavedState {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            left_panel_ratio: 0.22,
            right_panel_ratio: 0.26,
            left_tab: EditorLeftTab::Placer,
            right_tab: EditorRightTab::Survol,
            high_contrast: false,
        }
    }
}

pub(crate) struct EditorUiState {
    pub left_tab: EditorLeftTab,
    pub right_tab: EditorRightTab,
    pub search: String,
    pub search_focused: bool,
    pub save_as_name: String,
    pub save_as_focused: bool,
    pub left_scroll: usize,
    pub files_scroll: usize,
    pub outliner_scroll: usize,
    pub ui_scale: f32,
    pub left_panel_ratio: f32,
    pub right_panel_ratio: f32,
    pub dragging_split: Option<SplitDrag>,
    pub dirty: bool,
    pub high_contrast: bool,
    pub selected_layout: Option<usize>,
    pub layout_entries: Vec<String>,
    pub show_unsaved_modal: bool,
    pub pending_action: Option<EditorAction>,
    pub settings_dirty: bool,
    pub show_recents: bool,
    pub show_favoris: bool,
    pub show_sols: bool,
    pub show_murs: bool,
    pub show_objets: bool,
    pub show_utilitaires: bool,
    pub favoris: Vec<EditorBrush>,
    pub recents: Vec<EditorBrush>,
    pub tooltip: ui_kit::UiTooltipState,
    pub context_menu: Option<EditorContextMenu>,
}

impl EditorUiState {
    pub fn new() -> Self {
        Self {
            left_tab: EditorLeftTab::Placer,
            right_tab: EditorRightTab::Survol,
            search: String::new(),
            search_focused: false,
            save_as_name: String::new(),
            save_as_focused: false,
            left_scroll: 0,
            files_scroll: 0,
            outliner_scroll: 0,
            ui_scale: 1.0,
            left_panel_ratio: 0.22,
            right_panel_ratio: 0.26,
            dragging_split: None,
            dirty: false,
            high_contrast: false,
            selected_layout: None,
            layout_entries: Vec::new(),
            show_unsaved_modal: false,
            pending_action: None,
            settings_dirty: false,
            show_recents: true,
            show_favoris: true,
            show_sols: true,
            show_murs: true,
            show_objets: true,
            show_utilitaires: true,
            favoris: vec![
                EditorBrush::Wall,
                EditorBrush::Floor,
                EditorBrush::Crate,
                EditorBrush::EraseProp,
            ],
            recents: Vec::new(),
            tooltip: ui_kit::UiTooltipState::new(),
            context_menu: None,
        }
    }
}

pub(crate) fn load_editor_ui_state(path: &str) -> EditorUiState {
    let mut state = EditorUiState::new();
    let Ok(raw) = fs::read_to_string(path) else {
        return state;
    };
    let Ok(saved) = ron_from_str::<EditorUiSavedState>(&raw) else {
        return state;
    };
    state.ui_scale = saved.ui_scale.clamp(1.0, 1.6);
    state.left_panel_ratio = saved.left_panel_ratio.clamp(0.12, 0.45);
    state.right_panel_ratio = saved.right_panel_ratio.clamp(0.12, 0.5);
    state.left_tab = saved.left_tab;
    state.right_tab = saved.right_tab;
    state.high_contrast = saved.high_contrast;
    state
}

pub(crate) fn save_editor_ui_state(path: &str, ui: &mut EditorUiState) -> Result<(), String> {
    let payload = EditorUiSavedState {
        ui_scale: ui.ui_scale.clamp(1.0, 1.6),
        left_panel_ratio: ui.left_panel_ratio.clamp(0.12, 0.45),
        right_panel_ratio: ui.right_panel_ratio.clamp(0.12, 0.5),
        left_tab: ui.left_tab,
        right_tab: ui.right_tab,
        high_contrast: ui.high_contrast,
    };
    let pretty = PrettyConfig::new()
        .depth_limit(4)
        .enumerate_arrays(true)
        .separate_tuple_members(true);
    let encoded =
        ron_to_string_pretty(&payload, pretty).map_err(|err| format!("ui ron encode: {err}"))?;
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|err| format!("ui dossier: {err}"))?;
    }
    fs::write(path, encoded).map_err(|err| format!("ui ecriture: {err}"))?;
    ui.settings_dirty = false;
    Ok(())
}

#[derive(Copy, Clone)]
pub(crate) struct EditorUiLayout {
    pub top_bar: Rect,
    pub left_panel: Rect,
    pub map_view: Rect,
    pub right_panel: Rect,
    pub bottom_bar: Rect,
    pub split_left_bar: Rect,
    pub split_right_bar: Rect,
}

pub(crate) fn editor_compute_layout(ui: &EditorUiState) -> EditorUiLayout {
    editor_compute_layout_for_size(screen_width(), screen_height(), ui)
}

fn editor_compute_layout_for_size(sw: f32, sh: f32, ui: &EditorUiState) -> EditorUiLayout {
    let margin = 10.0 * ui.ui_scale;
    let top_h = 64.0 * ui.ui_scale;
    let bottom_h = 30.0 * ui.ui_scale;

    let inner_w = (sw - margin * 2.0).max(1.0);
    let inner_h = (sh - margin * 2.0).max(1.0);

    let top_bar = Rect::new(margin, margin, inner_w, top_h);
    let bottom_bar = Rect::new(margin, margin + inner_h - bottom_h, inner_w, bottom_h);

    let content_y = top_bar.y + top_bar.h + margin;
    let content_h = (bottom_bar.y - margin - content_y).max(1.0);

    let mut left_w =
        (inner_w * ui.left_panel_ratio).clamp(220.0 * ui.ui_scale, 440.0 * ui.ui_scale);
    let mut right_w =
        (inner_w * ui.right_panel_ratio).clamp(260.0 * ui.ui_scale, 560.0 * ui.ui_scale);

    let min_map_w = 320.0 * ui.ui_scale;
    let mut map_w = inner_w - left_w - right_w - margin * 2.0;
    if map_w < min_map_w {
        let deficit = min_map_w - map_w;
        let shrink_r = deficit.min((right_w - 200.0 * ui.ui_scale).max(0.0));
        right_w -= shrink_r;
        let map_w2 = inner_w - left_w - right_w - margin * 2.0;
        if map_w2 < min_map_w {
            let deficit2 = min_map_w - map_w2;
            let shrink_l = deficit2.min((left_w - 180.0 * ui.ui_scale).max(0.0));
            left_w -= shrink_l;
        }
    }

    map_w = (inner_w - left_w - right_w - margin * 2.0).max(1.0);

    let left_panel = Rect::new(margin, content_y, left_w, content_h);
    let split_left_bar = Rect::new(left_panel.x + left_panel.w, content_y, margin, content_h);
    let map_view = Rect::new(
        split_left_bar.x + split_left_bar.w,
        content_y,
        map_w,
        content_h,
    );
    let split_right_bar = Rect::new(map_view.x + map_view.w, content_y, margin, content_h);
    let right_panel = Rect::new(
        split_right_bar.x + split_right_bar.w,
        content_y,
        right_w,
        content_h,
    );

    EditorUiLayout {
        top_bar,
        left_panel,
        map_view,
        right_panel,
        bottom_bar,
        split_left_bar,
        split_right_bar,
    }
}

pub(crate) fn editor_ui_handle_splits(
    ui: &mut EditorUiState,
    layout: &EditorUiLayout,
    mouse: Vec2,
) {
    let left_pressed = is_mouse_button_pressed(MouseButton::Left);
    let left_down = is_mouse_button_down(MouseButton::Left);
    let left_released = is_mouse_button_released(MouseButton::Left);

    let hover_left = point_in_rect(mouse, layout.split_left_bar);
    let hover_right = point_in_rect(mouse, layout.split_right_bar);

    if left_pressed && hover_left {
        ui.dragging_split = Some(SplitDrag::Left);
    } else if left_pressed && hover_right {
        ui.dragging_split = Some(SplitDrag::Right);
    }

    if left_released {
        ui.dragging_split = None;
    }

    if left_down && let Some(kind) = ui.dragging_split {
        let sw = screen_width();
        let margin = 10.0 * ui.ui_scale;
        let inner_w = (sw - margin * 2.0).max(1.0);
        let x = (mouse.x - margin).clamp(0.0, inner_w);
        if kind == SplitDrag::Left {
            ui.left_panel_ratio = (x / inner_w).clamp(0.12, 0.45);
        } else {
            let right_w = (inner_w - x).clamp(160.0 * ui.ui_scale, inner_w);
            ui.right_panel_ratio = (right_w / inner_w).clamp(0.12, 0.50);
        }
        ui.settings_dirty = true;
    }
}

#[derive(Copy, Clone)]
struct BrushDef {
    brush: EditorBrush,
    label: &'static str,
    hotkey: &'static str,
    section: BrushSection,
    color: Color,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum BrushSection {
    Sols,
    Murs,
    Objets,
    Utilitaires,
}

const BRUSH_DEFS: &[BrushDef] = &[
    BrushDef {
        brush: EditorBrush::Floor,
        label: "Sol",
        hotkey: "1",
        section: BrushSection::Sols,
        color: Color::from_rgba(96, 176, 138, 255),
    },
    BrushDef {
        brush: EditorBrush::FloorMetal,
        label: "Sol metal",
        hotkey: "2",
        section: BrushSection::Sols,
        color: Color::from_rgba(74, 146, 178, 255),
    },
    BrushDef {
        brush: EditorBrush::FloorWood,
        label: "Sol bois",
        hotkey: "3",
        section: BrushSection::Sols,
        color: Color::from_rgba(142, 112, 78, 255),
    },
    BrushDef {
        brush: EditorBrush::FloorMoss,
        label: "Sol mousse",
        hotkey: "4",
        section: BrushSection::Sols,
        color: Color::from_rgba(84, 138, 86, 255),
    },
    BrushDef {
        brush: EditorBrush::FloorSand,
        label: "Sol sable",
        hotkey: "5",
        section: BrushSection::Sols,
        color: Color::from_rgba(192, 170, 102, 255),
    },
    BrushDef {
        brush: EditorBrush::Wall,
        label: "Mur",
        hotkey: "6",
        section: BrushSection::Murs,
        color: Color::from_rgba(138, 148, 162, 255),
    },
    BrushDef {
        brush: EditorBrush::WallBrick,
        label: "Mur brique",
        hotkey: "7",
        section: BrushSection::Murs,
        color: Color::from_rgba(172, 110, 96, 255),
    },
    BrushDef {
        brush: EditorBrush::WallSteel,
        label: "Mur acier",
        hotkey: "8",
        section: BrushSection::Murs,
        color: Color::from_rgba(124, 136, 154, 255),
    },
    BrushDef {
        brush: EditorBrush::WallNeon,
        label: "Mur neon",
        hotkey: "9",
        section: BrushSection::Murs,
        color: Color::from_rgba(86, 208, 220, 255),
    },
    BrushDef {
        brush: EditorBrush::Crate,
        label: "Caisse",
        hotkey: "0",
        section: BrushSection::Objets,
        color: Color::from_rgba(210, 150, 86, 255),
    },
    BrushDef {
        brush: EditorBrush::Pipe,
        label: "Tuyau",
        hotkey: "Q",
        section: BrushSection::Objets,
        color: Color::from_rgba(160, 174, 182, 255),
    },
    BrushDef {
        brush: EditorBrush::Lamp,
        label: "Lampe",
        hotkey: "W",
        section: BrushSection::Objets,
        color: Color::from_rgba(228, 210, 132, 255),
    },
    BrushDef {
        brush: EditorBrush::Banner,
        label: "Banniere",
        hotkey: "E",
        section: BrushSection::Objets,
        color: Color::from_rgba(188, 96, 132, 255),
    },
    BrushDef {
        brush: EditorBrush::Plant,
        label: "Pot de fleur",
        hotkey: "T",
        section: BrushSection::Objets,
        color: Color::from_rgba(90, 172, 108, 255),
    },
    BrushDef {
        brush: EditorBrush::Bench,
        label: "Banc",
        hotkey: "Y",
        section: BrushSection::Objets,
        color: Color::from_rgba(186, 128, 92, 255),
    },
    BrushDef {
        brush: EditorBrush::Crystal,
        label: "Cristal",
        hotkey: "U",
        section: BrushSection::Objets,
        color: Color::from_rgba(146, 214, 244, 255),
    },
    BrushDef {
        brush: EditorBrush::BoxCartonVide,
        label: "Box carton vide",
        hotkey: "-",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(206, 156, 108, 255),
    },
    BrushDef {
        brush: EditorBrush::BoxSacBleu,
        label: "Box sac bleu",
        hotkey: "-",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(112, 168, 224, 255),
    },
    BrushDef {
        brush: EditorBrush::BoxSacRouge,
        label: "Box sac rouge",
        hotkey: "-",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(224, 118, 108, 255),
    },
    BrushDef {
        brush: EditorBrush::BoxSacVert,
        label: "Box sac vert",
        hotkey: "-",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(124, 182, 112, 255),
    },
    BrushDef {
        brush: EditorBrush::PaletteLogistique,
        label: "Palette logistique",
        hotkey: "-",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(200, 166, 120, 255),
    },
    BrushDef {
        brush: EditorBrush::BureauPcOn,
        label: "Bureau PC ON",
        hotkey: "-",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(128, 212, 156, 255),
    },
    BrushDef {
        brush: EditorBrush::BureauPcOff,
        label: "Bureau PC OFF",
        hotkey: "-",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(138, 156, 172, 255),
    },
    BrushDef {
        brush: EditorBrush::CaisseAilBrut,
        label: "Caisse d'ail brut",
        hotkey: "-",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(214, 188, 136, 255),
    },
    BrushDef {
        brush: EditorBrush::CaisseAilCasse,
        label: "Caisse d'ail casse",
        hotkey: "-",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(216, 132, 114, 255),
    },
    BrushDef {
        brush: EditorBrush::Lavabo,
        label: "Lavabo",
        hotkey: "-",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(144, 198, 214, 255),
    },
    BrushDef {
        brush: EditorBrush::EraseProp,
        label: "Effacer objet",
        hotkey: "X",
        section: BrushSection::Utilitaires,
        color: Color::from_rgba(214, 118, 108, 255),
    },
];

pub(crate) struct EditorUiResult {
    pub action: EditorAction,
    pub center_camera_on: Option<(i32, i32)>,
    pub map_changed: bool,
}

impl Default for EditorUiResult {
    fn default() -> Self {
        Self {
            action: EditorAction::None,
            center_camera_on: None,
            map_changed: false,
        }
    }
}

fn push_recent(ui: &mut EditorUiState, brush: EditorBrush) {
    ui.recents.retain(|b| *b != brush);
    ui.recents.insert(0, brush);
    if ui.recents.len() > 8 {
        ui.recents.truncate(8);
    }
}

fn draw_library_row(
    row_rect: Rect,
    def: BrushDef,
    active_brush: EditorBrush,
    mouse: Vec2,
    left_click: bool,
    favoris: &mut Vec<EditorBrush>,
    font_size: f32,
) -> bool {
    let hovered = point_in_rect(mouse, row_rect);
    let active = active_brush == def.brush;
    let ui = ui_theme();
    let (row_top, row_bottom) = ui_panel_fill(hovered || active);
    let row_bg = if active {
        mix_color(row_top, ui.accent_cyan, 0.34)
    } else if hovered {
        mix_color(row_top, row_bottom, 0.28)
    } else {
        row_bottom
    };
    draw_rectangle(row_rect.x, row_rect.y, row_rect.w, row_rect.h, row_bg);
    draw_rectangle(
        row_rect.x,
        row_rect.y,
        row_rect.w,
        row_rect.h * 0.42,
        with_alpha(row_top, if active { 0.84 } else { 0.62 }),
    );
    draw_rectangle_lines(
        row_rect.x + 0.5,
        row_rect.y + 0.5,
        row_rect.w - 1.0,
        row_rect.h - 1.0,
        1.0,
        if active {
            ui.border_hi
        } else {
            with_alpha(ui.border, 0.84)
        },
    );

    let icon = Rect::new(
        row_rect.x + 8.0,
        row_rect.y + (row_rect.h - 18.0) * 0.5,
        18.0,
        18.0,
    );
    draw_rectangle(icon.x, icon.y, icon.w, icon.h, def.color);
    draw_rectangle_lines(
        icon.x + 0.5,
        icon.y + 0.5,
        icon.w - 1.0,
        icon.h - 1.0,
        1.0,
        BLACK,
    );

    draw_ui_text_tinted_on(
        row_bg,
        Color::from_rgba(232, 244, 252, 255),
        def.label,
        row_rect.x + 34.0,
        row_rect.y + row_rect.h * 0.61,
        font_size,
    );
    draw_ui_text_tinted_on(
        row_bg,
        Color::from_rgba(174, 212, 234, 255),
        def.hotkey,
        row_rect.x + row_rect.w - 22.0,
        row_rect.y + row_rect.h * 0.61,
        (font_size - 1.0).max(11.0),
    );

    let fav_rect = Rect::new(
        row_rect.x + row_rect.w - 48.0,
        row_rect.y + 4.0,
        20.0,
        row_rect.h - 8.0,
    );
    let is_fav = favoris.contains(&def.brush);
    if draw_ui_button_sized(
        fav_rect,
        if is_fav { "*" } else { "+" },
        mouse,
        left_click,
        is_fav,
        13.0,
    ) {
        if is_fav {
            favoris.retain(|b| *b != def.brush);
        } else {
            favoris.push(def.brush);
        }
    }

    left_click && hovered
}

fn zone_color(kind: ZoneKind, alpha: u8) -> Color {
    let feedback = feedback_theme();
    let alpha = alpha as f32 / 255.0;
    match kind {
        ZoneKind::Logistique => with_alpha(feedback.logistics, alpha),
        ZoneKind::Propre => with_alpha(feedback.positive, alpha),
        ZoneKind::Froide => with_alpha(feedback.info, alpha),
        ZoneKind::Production => with_alpha(feedback.warning, alpha),
        ZoneKind::Stockage => with_alpha(feedback.money, alpha),
    }
}

fn draw_zone_kind_button(
    rect: Rect,
    label: &str,
    kind: ZoneKind,
    active_kind: ZoneKind,
    mouse: Vec2,
    left_click: bool,
    font_size: f32,
) -> bool {
    let active = active_kind == kind;
    let clicked = draw_ui_button_sized(rect, label, mouse, left_click, active, font_size);
    let swatch = Rect::new(rect.x + 5.0, rect.y + 5.0, 14.0, rect.h - 10.0);
    draw_rectangle(
        swatch.x,
        swatch.y,
        swatch.w,
        swatch.h,
        zone_color(kind, 180),
    );
    draw_rectangle_lines(
        swatch.x + 0.5,
        swatch.y + 0.5,
        swatch.w - 1.0,
        swatch.h - 1.0,
        1.0,
        BLACK,
    );
    clicked
}

fn issue_severity_color(severity: ValidationSeverity) -> Color {
    let feedback = feedback_theme();
    match severity {
        ValidationSeverity::Error => feedback.danger,
        ValidationSeverity::Warning => feedback.warning,
        ValidationSeverity::Info => feedback.info,
    }
}

#[derive(Copy, Clone)]
enum EditorContextAction {
    Delete,
    Copy,
    Paste,
    SetPlayerSpawn,
    SetNpcSpawn,
}

fn draw_editor_context_menu(
    editor: &mut EditorState,
    map: &mut MapAsset,
    mouse: Vec2,
    left_click: bool,
    scale: f32,
    result: &mut EditorUiResult,
) {
    let Some(menu) = editor.ui.context_menu else {
        return;
    };
    let actions = [
        (EditorContextAction::Delete, "Supprimer contenu"),
        (EditorContextAction::Copy, "Copier"),
        (EditorContextAction::Paste, "Coller"),
        (EditorContextAction::SetPlayerSpawn, "Definir spawn joueur"),
        (EditorContextAction::SetNpcSpawn, "Definir spawn PNJ"),
    ];
    let row_h = 30.0 * scale;
    let menu_w = 230.0 * scale;
    let menu_h = actions.len() as f32 * row_h + 10.0 * scale;
    let mx = menu
        .anchor_screen
        .x
        .clamp(8.0, screen_width() - menu_w - 8.0);
    let my = menu
        .anchor_screen
        .y
        .clamp(8.0, screen_height() - menu_h - 8.0);
    let menu_rect = Rect::new(mx, my, menu_w, menu_h);
    let (menu_top, menu_bottom) = ui_panel_fill(true);
    draw_rectangle(
        menu_rect.x,
        menu_rect.y,
        menu_rect.w,
        menu_rect.h,
        menu_bottom,
    );
    draw_rectangle(
        menu_rect.x,
        menu_rect.y,
        menu_rect.w,
        menu_rect.h * 0.48,
        with_alpha(menu_top, 0.82),
    );
    draw_rectangle_lines(
        menu_rect.x + 0.5,
        menu_rect.y + 0.5,
        menu_rect.w - 1.0,
        menu_rect.h - 1.0,
        1.1,
        with_alpha(ui_theme().border, 0.92),
    );

    let mut chosen = None;
    let mut row_y = menu_rect.y + 5.0 * scale;
    for (index, (action, label)) in actions.iter().enumerate() {
        let row = Rect::new(
            menu_rect.x + 4.0,
            row_y,
            menu_rect.w - 8.0,
            row_h - if index + 1 == actions.len() { 6.0 } else { 2.0 },
        );
        if draw_ui_button_sized(row, label, mouse, left_click, false, 12.8 * scale) {
            chosen = Some(*action);
        }
        row_y += row_h;
    }

    if left_click && !point_in_rect(mouse, menu_rect) {
        editor.ui.context_menu = None;
        return;
    }

    let Some(action) = chosen else {
        return;
    };
    let tile = menu.tile;
    match action {
        EditorContextAction::Delete => {
            let before = editor.undo_stack.len();
            editor_push_undo(editor, map);
            let mut changed = false;
            changed |= editor_apply_brush_with_rotation(map, EditorBrush::EraseProp, tile, 0);
            changed |= set_zone_kind_at_tile(map, tile, None);
            changed |= set_map_tile(map, tile, Tile::Floor);
            if changed {
                editor.redo_stack.clear();
                editor.ui.dirty = true;
                result.map_changed = true;
                editor_set_status(editor, "Contenu supprime");
            } else {
                editor.undo_stack.truncate(before);
                editor_set_status(editor, "Rien a supprimer");
            }
        }
        EditorContextAction::Copy => {
            editor.selected_tile = Some(tile);
            editor.selection_rect = None;
            if let Some(clip) = capture_selection_clipboard(editor, map) {
                editor.clipboard = Some(clip);
                editor_set_status(editor, "Tuile copiee");
            } else {
                editor_set_status(editor, "Copie impossible");
            }
        }
        EditorContextAction::Paste => {
            if let Some(clip) = editor.clipboard.clone() {
                let before = editor.undo_stack.len();
                editor_push_undo(editor, map);
                if paste_clipboard_at(map, &clip, tile) {
                    editor.redo_stack.clear();
                    editor.ui.dirty = true;
                    result.map_changed = true;
                    editor_set_status(editor, "Clipboard colle");
                } else {
                    editor.undo_stack.truncate(before);
                    editor_set_status(editor, "Collage sans effet");
                }
            } else {
                editor_set_status(editor, "Clipboard vide");
            }
        }
        EditorContextAction::SetPlayerSpawn => match editor_set_player_spawn(editor, map, tile) {
            Ok(()) => {
                editor.ui.dirty = true;
                result.map_changed = true;
            }
            Err(reason) => editor_set_status(editor, format!("Spawn joueur invalide: {reason}")),
        },
        EditorContextAction::SetNpcSpawn => match editor_set_npc_spawn(editor, map, tile) {
            Ok(()) => {
                editor.ui.dirty = true;
                result.map_changed = true;
            }
            Err(reason) => editor_set_status(editor, format!("Spawn PNJ invalide: {reason}")),
        },
    }
    editor.ui.context_menu = None;
}

pub(crate) fn draw_editor_ui(
    editor: &mut EditorState,
    map: &mut MapAsset,
    layout: &EditorUiLayout,
    palette: &Palette,
    mouse: Vec2,
    left_click: bool,
    wheel_y: f32,
) -> EditorUiResult {
    ui_kit::tooltip_begin_frame(&mut editor.ui.tooltip);
    let scale = editor.ui.ui_scale;
    let ui = ui_theme();
    let panel_bg = if editor.ui.high_contrast {
        mix_color(ui.panel_bottom, BLACK, 0.58)
    } else {
        mix_color(ui.panel_bottom, ui.panel_mid, 0.26)
    };
    let (top_gloss, top_bg) = if editor.ui.high_contrast {
        let dark = mix_color(ui.panel_mid, BLACK, 0.44);
        (mix_color(ui.panel_top, ui.border_hi, 0.12), dark)
    } else {
        ui_panel_header_fill(true)
    };
    let mut result = EditorUiResult::default();

    draw_rectangle(
        layout.top_bar.x,
        layout.top_bar.y,
        layout.top_bar.w,
        layout.top_bar.h,
        top_bg,
    );
    draw_rectangle(
        layout.top_bar.x,
        layout.top_bar.y,
        layout.top_bar.w,
        layout.top_bar.h * 0.42,
        with_alpha(top_gloss, 0.74),
    );
    draw_rectangle_lines(
        layout.top_bar.x + 0.5,
        layout.top_bar.y + 0.5,
        layout.top_bar.w - 1.0,
        layout.top_bar.h - 1.0,
        1.8,
        with_alpha(ui.border, 0.94),
    );

    let title_fs = 30.0 * scale;
    draw_ui_text_tinted_on(
        top_bg,
        Color::from_rgba(230, 242, 250, 255),
        "EDITEUR USINE",
        layout.top_bar.x + 16.0 * scale,
        layout.top_bar.y + 26.0 * scale,
        title_fs,
    );

    let meta_fs = 18.0 * scale;
    let dirty = if editor.ui.dirty { " *" } else { "" };
    let meta = format!(
        "{}{} | {}x{} | objets {} | zones {} | schema {}",
        map.label,
        dirty,
        map.world.w,
        map.world.h,
        map.props.len(),
        map.zones.len(),
        map.schema_version
    );
    draw_ui_text_tinted_on(
        top_bg,
        Color::from_rgba(176, 206, 223, 255),
        &meta,
        layout.top_bar.x + 16.0 * scale,
        layout.top_bar.y + 48.0 * scale,
        meta_fs,
    );

    let btn_h = 34.0 * scale;
    let btn_w = 106.0 * scale;
    let gap = 8.0 * scale;
    let by = layout.top_bar.y + (layout.top_bar.h - btn_h) * 0.5;
    let mut rx = layout.top_bar.x + layout.top_bar.w - 12.0 * scale;

    rx -= btn_w;
    let load_r = Rect::new(rx, by, btn_w, btn_h);
    rx -= gap + btn_w;
    let save_r = Rect::new(rx, by, btn_w, btn_h);
    rx -= gap + btn_w;
    let menu_r = Rect::new(rx, by, btn_w, btn_h);
    rx -= gap + btn_w;
    let play_r = Rect::new(rx, by, btn_w, btn_h);

    let mut requested_action = None;
    if draw_ui_button_sized(play_r, "Jouer F5", mouse, left_click, false, 16.0 * scale)
        || is_key_pressed(KeyCode::F5)
    {
        requested_action = Some(EditorAction::StartPlay);
    }
    ui_kit::tooltip_for_rect(
        &mut editor.ui.tooltip,
        "top_play",
        "Lancer la simulation (F5).",
        play_r,
        mouse,
    );
    if draw_ui_button_sized(menu_r, "Menu Esc", mouse, left_click, false, 16.0 * scale)
        || is_key_pressed(KeyCode::Escape)
    {
        requested_action = Some(EditorAction::BackToMenu);
    }
    ui_kit::tooltip_for_rect(
        &mut editor.ui.tooltip,
        "top_menu",
        "Retour menu principal (Esc).",
        menu_r,
        mouse,
    );
    if draw_ui_button_sized(save_r, "Sauver", mouse, left_click, false, 16.0 * scale) {
        editor_save_current_map(editor, map);
    }
    ui_kit::tooltip_for_rect(
        &mut editor.ui.tooltip,
        "top_save",
        "Sauver la carte active dans maps/main_map.ron.",
        save_r,
        mouse,
    );
    if draw_ui_button_sized(load_r, "Charger", mouse, left_click, false, 16.0 * scale) {
        if editor.ui.dirty {
            editor.ui.pending_action = None;
            editor.ui.show_unsaved_modal = true;
        } else {
            editor_load_current_map(editor, map);
        }
    }
    ui_kit::tooltip_for_rect(
        &mut editor.ui.tooltip,
        "top_load",
        "Recharger la carte active depuis maps/main_map.ron.",
        load_r,
        mouse,
    );

    if let Some(action) = requested_action {
        if editor.ui.dirty {
            editor.ui.pending_action = Some(action);
            editor.ui.show_unsaved_modal = true;
        } else {
            if action == EditorAction::StartPlay {
                sanitize_map_asset(map);
            }
            result.action = action;
        }
    }

    draw_rectangle(
        layout.left_panel.x,
        layout.left_panel.y,
        layout.left_panel.w,
        layout.left_panel.h,
        panel_bg,
    );
    draw_rectangle(
        layout.left_panel.x,
        layout.left_panel.y,
        layout.left_panel.w,
        layout.left_panel.h * 0.18,
        with_alpha(ui.panel_top, 0.20),
    );
    draw_rectangle_lines(
        layout.left_panel.x + 0.5,
        layout.left_panel.y + 0.5,
        layout.left_panel.w - 1.0,
        layout.left_panel.h - 1.0,
        1.8,
        with_alpha(ui.border, 0.86),
    );
    draw_rectangle(
        layout.right_panel.x,
        layout.right_panel.y,
        layout.right_panel.w,
        layout.right_panel.h,
        panel_bg,
    );
    draw_rectangle(
        layout.right_panel.x,
        layout.right_panel.y,
        layout.right_panel.w,
        layout.right_panel.h * 0.18,
        with_alpha(ui.panel_top, 0.20),
    );
    draw_rectangle_lines(
        layout.right_panel.x + 0.5,
        layout.right_panel.y + 0.5,
        layout.right_panel.w - 1.0,
        layout.right_panel.h - 1.0,
        1.8,
        with_alpha(ui.border, 0.86),
    );

    let tab_h = 30.0 * scale;
    let pad = 12.0 * scale;
    let left_tabs_rect = Rect::new(
        layout.left_panel.x + pad,
        layout.left_panel.y + pad,
        layout.left_panel.w - pad * 2.0,
        tab_h,
    );
    let left_tabs = [
        ("Placer", editor.ui.left_tab == EditorLeftTab::Placer),
        ("Zones", editor.ui.left_tab == EditorLeftTab::Zones),
        ("Outils", editor.ui.left_tab == EditorLeftTab::Outils),
        ("Fichiers", editor.ui.left_tab == EditorLeftTab::Fichiers),
    ];
    if let Some(idx) =
        ui_kit::draw_tab_row(left_tabs_rect, &left_tabs, mouse, left_click, 14.0 * scale)
    {
        editor.ui.left_tab = match idx {
            0 => EditorLeftTab::Placer,
            1 => EditorLeftTab::Zones,
            2 => EditorLeftTab::Outils,
            _ => EditorLeftTab::Fichiers,
        };
    }

    let right_tabs_rect = Rect::new(
        layout.right_panel.x + pad,
        layout.right_panel.y + pad,
        layout.right_panel.w - pad * 2.0,
        tab_h,
    );
    let right_tabs = [
        ("Survol", editor.ui.right_tab == EditorRightTab::Survol),
        (
            "Selection",
            editor.ui.right_tab == EditorRightTab::Selection,
        ),
        ("Carte", editor.ui.right_tab == EditorRightTab::Carte),
        (
            "Validation",
            editor.ui.right_tab == EditorRightTab::Validation,
        ),
    ];
    if let Some(idx) = ui_kit::draw_tab_row(
        right_tabs_rect,
        &right_tabs,
        mouse,
        left_click,
        13.0 * scale,
    ) {
        editor.ui.right_tab = match idx {
            0 => EditorRightTab::Survol,
            1 => EditorRightTab::Selection,
            2 => EditorRightTab::Carte,
            _ => EditorRightTab::Validation,
        };
    }

    if editor.ui.left_tab == EditorLeftTab::Placer {
        let search_r = Rect::new(
            layout.left_panel.x + pad,
            left_tabs_rect.y + left_tabs_rect.h + 10.0 * scale,
            layout.left_panel.w - pad * 2.0,
            30.0 * scale,
        );
        if ui_kit::draw_text_input(
            search_r,
            &mut editor.ui.search,
            &mut editor.ui.search_focused,
            mouse,
            left_click,
            14.0 * scale,
            "Rechercher...",
        ) {
            editor.ui.left_scroll = 0;
        }
        let list_r = Rect::new(
            layout.left_panel.x + pad,
            search_r.y + search_r.h + 8.0 * scale,
            layout.left_panel.w - pad * 2.0,
            (layout.left_panel.y + layout.left_panel.h) - (search_r.y + search_r.h + 18.0 * scale),
        );
        draw_rectangle(
            list_r.x,
            list_r.y,
            list_r.w,
            list_r.h,
            Color::from_rgba(8, 13, 20, 132),
        );
        draw_rectangle_lines(
            list_r.x + 0.5,
            list_r.y + 0.5,
            list_r.w - 1.0,
            list_r.h - 1.0,
            1.0,
            Color::from_rgba(90, 130, 154, 142),
        );

        enum Row {
            Header(&'static str, bool),
            Brush(BrushDef),
        }
        let needle = editor.ui.search.to_lowercase();
        let mut rows = Vec::new();
        let push_section =
            |rows: &mut Vec<Row>, title: &'static str, open: bool, items: Vec<BrushDef>| {
                rows.push(Row::Header(title, open));
                if open {
                    for item in items {
                        rows.push(Row::Brush(item));
                    }
                }
            };
        let filter_items = |pred: fn(&BrushDef) -> bool| -> Vec<BrushDef> {
            BRUSH_DEFS
                .iter()
                .copied()
                .filter(&pred)
                .filter(|def| needle.is_empty() || def.label.to_lowercase().contains(&needle))
                .collect()
        };
        let recents_defs: Vec<BrushDef> = editor
            .ui
            .recents
            .iter()
            .filter_map(|brush| BRUSH_DEFS.iter().copied().find(|def| def.brush == *brush))
            .collect();
        let favoris_defs: Vec<BrushDef> = editor
            .ui
            .favoris
            .iter()
            .filter_map(|brush| BRUSH_DEFS.iter().copied().find(|def| def.brush == *brush))
            .collect();
        push_section(&mut rows, "Recents", editor.ui.show_recents, recents_defs);
        push_section(&mut rows, "Favoris", editor.ui.show_favoris, favoris_defs);
        push_section(
            &mut rows,
            "Sols",
            editor.ui.show_sols,
            filter_items(|def| def.section == BrushSection::Sols),
        );
        push_section(
            &mut rows,
            "Murs",
            editor.ui.show_murs,
            filter_items(|def| def.section == BrushSection::Murs),
        );
        push_section(
            &mut rows,
            "Objets",
            editor.ui.show_objets,
            filter_items(|def| def.section == BrushSection::Objets),
        );
        push_section(
            &mut rows,
            "Utilitaires",
            editor.ui.show_utilitaires,
            filter_items(|def| def.section == BrushSection::Utilitaires),
        );

        let row_h = (30.0 * scale).max(28.0);
        let visible_rows = ((list_r.h - 8.0) / row_h).floor().max(1.0) as usize;
        ui_kit::update_scroll_with_wheel(
            &mut editor.ui.left_scroll,
            rows.len(),
            visible_rows,
            wheel_y,
            point_in_rect(mouse, list_r),
        );
        let start = editor.ui.left_scroll;
        let end = (start + visible_rows).min(rows.len());
        let mut y = list_r.y + 4.0;
        for row in &rows[start..end] {
            let row_rect = Rect::new(list_r.x + 4.0, y, list_r.w - 8.0, row_h - 2.0);
            match row {
                Row::Header(label, open) => {
                    let text = if *open {
                        format!("v {label}")
                    } else {
                        format!("> {label}")
                    };
                    if draw_ui_button_sized(row_rect, &text, mouse, left_click, false, 13.0 * scale)
                    {
                        match *label {
                            "Recents" => editor.ui.show_recents = !editor.ui.show_recents,
                            "Favoris" => editor.ui.show_favoris = !editor.ui.show_favoris,
                            "Sols" => editor.ui.show_sols = !editor.ui.show_sols,
                            "Murs" => editor.ui.show_murs = !editor.ui.show_murs,
                            "Objets" => editor.ui.show_objets = !editor.ui.show_objets,
                            _ => editor.ui.show_utilitaires = !editor.ui.show_utilitaires,
                        }
                    }
                }
                Row::Brush(def) => {
                    if draw_library_row(
                        row_rect,
                        *def,
                        editor.brush,
                        mouse,
                        left_click,
                        &mut editor.ui.favoris,
                        13.5 * scale,
                    ) {
                        editor.brush = def.brush;
                        editor.tool = EditorTool::Brush;
                        push_recent(&mut editor.ui, def.brush);
                    }
                }
            }
            y += row_h;
        }
    } else if editor.ui.left_tab == EditorLeftTab::Zones {
        let base_y = left_tabs_rect.y + left_tabs_rect.h + 12.0 * scale;
        let bw = layout.left_panel.w - pad * 2.0;
        let bh = 30.0 * scale;
        if draw_zone_kind_button(
            Rect::new(layout.left_panel.x + pad, base_y, bw, bh),
            "Zone logistique",
            ZoneKind::Logistique,
            editor.zone_kind,
            mouse,
            left_click,
            14.0 * scale,
        ) {
            editor.zone_kind = ZoneKind::Logistique;
            editor.tool = EditorTool::Brush;
        }
        if draw_zone_kind_button(
            Rect::new(layout.left_panel.x + pad, base_y + 38.0 * scale, bw, bh),
            "Zone propre",
            ZoneKind::Propre,
            editor.zone_kind,
            mouse,
            left_click,
            14.0 * scale,
        ) {
            editor.zone_kind = ZoneKind::Propre;
            editor.tool = EditorTool::Brush;
        }
        if draw_zone_kind_button(
            Rect::new(layout.left_panel.x + pad, base_y + 76.0 * scale, bw, bh),
            "Zone froide",
            ZoneKind::Froide,
            editor.zone_kind,
            mouse,
            left_click,
            14.0 * scale,
        ) {
            editor.zone_kind = ZoneKind::Froide;
            editor.tool = EditorTool::Brush;
        }
        if draw_zone_kind_button(
            Rect::new(layout.left_panel.x + pad, base_y + 114.0 * scale, bw, bh),
            "Zone production",
            ZoneKind::Production,
            editor.zone_kind,
            mouse,
            left_click,
            14.0 * scale,
        ) {
            editor.zone_kind = ZoneKind::Production;
            editor.tool = EditorTool::Brush;
        }
        if draw_zone_kind_button(
            Rect::new(layout.left_panel.x + pad, base_y + 152.0 * scale, bw, bh),
            "Zone stockage",
            ZoneKind::Stockage,
            editor.zone_kind,
            mouse,
            left_click,
            14.0 * scale,
        ) {
            editor.zone_kind = ZoneKind::Stockage;
            editor.tool = EditorTool::Brush;
        }
        let clear_r = Rect::new(layout.left_panel.x + pad, base_y + 196.0 * scale, bw, bh);
        if draw_ui_button_sized(
            clear_r,
            "Effacer zone sur selection",
            mouse,
            left_click,
            false,
            13.0 * scale,
        ) && let Some(tile) = editor.selected_tile
            && set_zone_kind_at_tile(map, tile, None)
        {
            editor.ui.dirty = true;
            result.map_changed = true;
        }
        draw_ui_text_tinted_on(
            panel_bg,
            Color::from_rgba(184, 212, 226, 255),
            "Utilise les outils B/R/L/F avec ce type de zone.",
            layout.left_panel.x + pad,
            base_y + 248.0 * scale,
            13.5 * scale,
        );
    } else if editor.ui.left_tab == EditorLeftTab::Outils {
        let y0 = left_tabs_rect.y + left_tabs_rect.h + 12.0 * scale;
        let bw = layout.left_panel.w - pad * 2.0;
        let bh = 30.0 * scale;
        let tool_buttons = [
            (EditorTool::Select, "Selection (S)"),
            (EditorTool::Brush, "Pinceau (B)"),
            (EditorTool::Rect, "Rectangle (R)"),
            (EditorTool::Line, "Ligne (L)"),
            (EditorTool::Fill, "Remplissage (F)"),
            (EditorTool::Paste, "Coller (V)"),
        ];
        for (idx, (tool, label)) in tool_buttons.iter().enumerate() {
            let rect = Rect::new(
                layout.left_panel.x + pad,
                y0 + idx as f32 * 36.0 * scale,
                bw,
                bh,
            );
            if draw_ui_button_sized(
                rect,
                label,
                mouse,
                left_click,
                editor.tool == *tool,
                13.5 * scale,
            ) {
                editor.tool = *tool;
            }
        }
        let brush_y = y0 + 6.0 * 36.0 * scale + 8.0 * scale;
        draw_ui_text_tinted_on(
            panel_bg,
            Color::from_rgba(204, 226, 238, 255),
            &format!("Taille pinceau: {}", editor.brush_size),
            layout.left_panel.x + pad,
            brush_y,
            14.0 * scale,
        );
        let minus = Rect::new(
            layout.left_panel.x + pad,
            brush_y + 8.0 * scale,
            34.0 * scale,
            28.0 * scale,
        );
        let plus = Rect::new(
            layout.left_panel.x + pad + 40.0 * scale,
            brush_y + 8.0 * scale,
            34.0 * scale,
            28.0 * scale,
        );
        if draw_ui_button_sized(minus, "-", mouse, left_click, false, 14.0 * scale) {
            editor.brush_size = editor.brush_size.saturating_sub(1).max(1);
        }
        if draw_ui_button_sized(plus, "+", mouse, left_click, false, 14.0 * scale) {
            editor.brush_size = (editor.brush_size + 1).min(5);
        }
        let rot_y = brush_y + 56.0 * scale;
        draw_ui_text_tinted_on(
            panel_bg,
            Color::from_rgba(204, 226, 238, 255),
            &format!("Rotation objet: {}*90", editor.prop_rotation),
            layout.left_panel.x + pad,
            rot_y,
            14.0 * scale,
        );
        let rot_m = Rect::new(
            layout.left_panel.x + pad,
            rot_y + 8.0 * scale,
            34.0 * scale,
            28.0 * scale,
        );
        let rot_p = Rect::new(
            layout.left_panel.x + pad + 40.0 * scale,
            rot_y + 8.0 * scale,
            34.0 * scale,
            28.0 * scale,
        );
        if draw_ui_button_sized(rot_m, "-", mouse, left_click, false, 14.0 * scale) {
            editor.prop_rotation = (editor.prop_rotation - 1).rem_euclid(4);
        }
        if draw_ui_button_sized(rot_p, "+", mouse, left_click, false, 14.0 * scale) {
            editor.prop_rotation = (editor.prop_rotation + 1).rem_euclid(4);
        }
        let grid_r = Rect::new(
            layout.left_panel.x + pad,
            rot_y + 50.0 * scale,
            bw,
            30.0 * scale,
        );
        if draw_ui_button_sized(
            grid_r,
            if editor.show_grid {
                "Grille: ON (G)"
            } else {
                "Grille: OFF (G)"
            },
            mouse,
            left_click,
            editor.show_grid,
            13.0 * scale,
        ) {
            editor.show_grid = !editor.show_grid;
        }
        let scale_r = Rect::new(
            layout.left_panel.x + pad,
            rot_y + 90.0 * scale,
            bw,
            30.0 * scale,
        );
        if draw_ui_button_sized(scale_r, "UI +", mouse, left_click, false, 13.0 * scale) {
            editor.ui.ui_scale = (editor.ui.ui_scale + 0.1).clamp(1.0, 1.6);
            editor.ui.settings_dirty = true;
        }
        let scale2_r = Rect::new(
            layout.left_panel.x + pad,
            rot_y + 128.0 * scale,
            bw,
            30.0 * scale,
        );
        if draw_ui_button_sized(scale2_r, "UI -", mouse, left_click, false, 13.0 * scale) {
            editor.ui.ui_scale = (editor.ui.ui_scale - 0.1).clamp(1.0, 1.6);
            editor.ui.settings_dirty = true;
        }
        let contrast_r = Rect::new(
            layout.left_panel.x + pad,
            rot_y + 166.0 * scale,
            bw,
            30.0 * scale,
        );
        if draw_ui_button_sized(
            contrast_r,
            if editor.ui.high_contrast {
                "Contraste fort: ON"
            } else {
                "Contraste fort: OFF"
            },
            mouse,
            left_click,
            editor.ui.high_contrast,
            12.8 * scale,
        ) {
            editor.ui.high_contrast = !editor.ui.high_contrast;
            editor.ui.settings_dirty = true;
        }
    } else {
        let y0 = left_tabs_rect.y + left_tabs_rect.h + 12.0 * scale;
        let bw = layout.left_panel.w - pad * 2.0;
        let input_r = Rect::new(layout.left_panel.x + pad, y0, bw, 30.0 * scale);
        ui_kit::draw_text_input(
            input_r,
            &mut editor.ui.save_as_name,
            &mut editor.ui.save_as_focused,
            mouse,
            left_click,
            14.0 * scale,
            "Nom layout (sans .ron)",
        );
        let save_as = Rect::new(
            layout.left_panel.x + pad,
            y0 + 38.0 * scale,
            bw,
            30.0 * scale,
        );
        if draw_ui_button_sized(
            save_as,
            "Sauver sous",
            mouse,
            left_click,
            false,
            13.5 * scale,
        ) {
            let save_name = editor.ui.save_as_name.trim().to_string();
            match editor_save_map_as(editor, map, &save_name) {
                Ok(saved_label) => {
                    editor_set_status(editor, format!("Layout sauve: {}", saved_label));
                }
                Err(err) => editor_set_status(editor, err),
            }
            refresh_editor_layouts(editor);
        }
        ui_kit::tooltip_for_rect(
            &mut editor.ui.tooltip,
            "files_save_as",
            "Sauver une copie de la carte dans maps/layouts.",
            save_as,
            mouse,
        );
        let duplicate_r = Rect::new(
            layout.left_panel.x + pad,
            y0 + 76.0 * scale,
            bw,
            30.0 * scale,
        );
        if draw_ui_button_sized(
            duplicate_r,
            "Dupliquer selection",
            mouse,
            left_click,
            false,
            12.8 * scale,
        ) && let Some(idx) = editor.ui.selected_layout
            && let Some(name) = editor.ui.layout_entries.get(idx).cloned()
        {
            let requested = editor.ui.save_as_name.trim().to_string();
            match editor_duplicate_layout(
                editor,
                &name,
                if requested.is_empty() {
                    None
                } else {
                    Some(requested.as_str())
                },
            ) {
                Ok(created) => editor_set_status(editor, format!("Layout duplique: {created}")),
                Err(err) => editor_set_status(editor, err),
            }
            refresh_editor_layouts(editor);
        }
        ui_kit::tooltip_for_rect(
            &mut editor.ui.tooltip,
            "files_duplicate",
            "Duplique le layout selectionne.",
            duplicate_r,
            mouse,
        );
        let export_r = Rect::new(
            layout.left_panel.x + pad,
            y0 + 114.0 * scale,
            bw,
            30.0 * scale,
        );
        if draw_ui_button_sized(
            export_r,
            "Exporter blueprint",
            mouse,
            left_click,
            false,
            12.8 * scale,
        ) {
            let requested = editor.ui.save_as_name.trim();
            match editor_export_blueprint(
                editor,
                map,
                if requested.is_empty() {
                    None
                } else {
                    Some(requested)
                },
            ) {
                Ok(saved) => editor_set_status(editor, format!("Blueprint exporte: {saved}")),
                Err(err) => editor_set_status(editor, err),
            }
        }
        ui_kit::tooltip_for_rect(
            &mut editor.ui.tooltip,
            "files_export",
            "Exporte la selection (ou la carte) en blueprint.",
            export_r,
            mouse,
        );
        let refresh_r = Rect::new(
            layout.left_panel.x + pad,
            y0 + 152.0 * scale,
            bw,
            30.0 * scale,
        );
        if draw_ui_button_sized(
            refresh_r,
            "Rafraichir liste",
            mouse,
            left_click,
            false,
            13.0 * scale,
        ) {
            refresh_editor_layouts(editor);
        }
        ui_kit::tooltip_for_rect(
            &mut editor.ui.tooltip,
            "files_refresh",
            "Recharge la liste des layouts disponibles.",
            refresh_r,
            mouse,
        );
        let load_sel = Rect::new(
            layout.left_panel.x + pad,
            y0 + 190.0 * scale,
            bw,
            30.0 * scale,
        );
        if draw_ui_button_sized(
            load_sel,
            "Charger selection",
            mouse,
            left_click,
            false,
            13.0 * scale,
        ) && let Some(idx) = editor.ui.selected_layout
            && let Some(name) = editor.ui.layout_entries.get(idx).cloned()
        {
            if editor.ui.dirty {
                editor.ui.pending_action = None;
                editor.ui.show_unsaved_modal = true;
            } else {
                match editor_load_map_as(editor, map, &name) {
                    Ok(()) => editor_set_status(editor, format!("Layout charge: {name}")),
                    Err(err) => editor_set_status(editor, err),
                }
            }
        }
        ui_kit::tooltip_for_rect(
            &mut editor.ui.tooltip,
            "files_load_selection",
            "Charge le layout selectionne dans l'editeur.",
            load_sel,
            mouse,
        );
        let list_r = Rect::new(
            layout.left_panel.x + pad,
            y0 + 228.0 * scale,
            bw,
            (layout.left_panel.y + layout.left_panel.h) - (y0 + 240.0 * scale),
        );
        draw_rectangle(
            list_r.x,
            list_r.y,
            list_r.w,
            list_r.h,
            Color::from_rgba(8, 13, 20, 132),
        );
        draw_rectangle_lines(
            list_r.x + 0.5,
            list_r.y + 0.5,
            list_r.w - 1.0,
            list_r.h - 1.0,
            1.0,
            Color::from_rgba(90, 130, 154, 142),
        );
        let row_h = 30.0 * scale;
        let visible_rows = ((list_r.h - 8.0) / row_h).floor().max(1.0) as usize;
        ui_kit::update_scroll_with_wheel(
            &mut editor.ui.files_scroll,
            editor.ui.layout_entries.len(),
            visible_rows,
            wheel_y,
            point_in_rect(mouse, list_r),
        );
        let start = editor.ui.files_scroll;
        let end = (start + visible_rows).min(editor.ui.layout_entries.len());
        let mut y = list_r.y + 4.0;
        for index in start..end {
            let name = editor.ui.layout_entries[index].clone();
            let row = Rect::new(list_r.x + 4.0, y, list_r.w - 8.0, row_h - 2.0);
            if draw_ui_button_sized(
                row,
                &name,
                mouse,
                left_click,
                editor.ui.selected_layout == Some(index),
                12.8 * scale,
            ) {
                editor.ui.selected_layout = Some(index);
            }
            y += row_h;
        }
    }

    if editor.ui.right_tab == EditorRightTab::Survol {
        let y = right_tabs_rect.y + right_tabs_rect.h + 12.0 * scale;
        let hover_info = if let Some(tile) = editor.hover_tile {
            let tile_kind = map.world.get(tile.0, tile.1);
            let prop = prop_index_at_tile(&map.props, tile)
                .map(|idx| prop_kind_label(map.props[idx].kind));
            let zone = zone_kind_at_tile(map, tile)
                .map(zone_kind_label)
                .unwrap_or("aucune");
            format!(
                "Case: ({}, {})\nTuile: {}\nObjet: {}\nZone: {}\nActif: {} / {}",
                tile.0,
                tile.1,
                tile_label(tile_kind),
                prop.unwrap_or("aucun"),
                zone,
                editor_tool_label(editor.tool),
                editor_brush_label(editor.brush)
            )
        } else {
            "Case: aucune".to_string()
        };
        draw_ui_text_tinted_on(
            panel_bg,
            Color::from_rgba(192, 218, 232, 255),
            &hover_info,
            layout.right_panel.x + pad,
            y,
            16.0 * scale,
        );
    } else if editor.ui.right_tab == EditorRightTab::Selection {
        let y0 = right_tabs_rect.y + right_tabs_rect.h + 12.0 * scale;
        if let Some(tile) = editor.selected_tile {
            let zone = zone_kind_at_tile(map, tile)
                .map(zone_kind_label)
                .unwrap_or("aucune");
            let info = format!(
                "Selection: ({}, {})\nTuile: {}\nZone: {}",
                tile.0,
                tile.1,
                tile_label(map.world.get(tile.0, tile.1)),
                zone
            );
            draw_ui_text_tinted_on(
                panel_bg,
                Color::from_rgba(206, 228, 240, 255),
                &info,
                layout.right_panel.x + pad,
                y0,
                15.5 * scale,
            );
            let bw = layout.right_panel.w - pad * 2.0;
            let set_floor = Rect::new(
                layout.right_panel.x + pad,
                y0 + 66.0 * scale,
                bw,
                28.0 * scale,
            );
            if draw_ui_button_sized(
                set_floor,
                "Mettre Sol",
                mouse,
                left_click,
                false,
                13.0 * scale,
            ) && editor_apply_brush_with_rotation(map, EditorBrush::Floor, tile, 0)
            {
                editor.ui.dirty = true;
                result.map_changed = true;
            }
            let set_wall = Rect::new(
                layout.right_panel.x + pad,
                y0 + 100.0 * scale,
                bw,
                28.0 * scale,
            );
            if draw_ui_button_sized(
                set_wall,
                "Mettre Mur",
                mouse,
                left_click,
                false,
                13.0 * scale,
            ) && editor_apply_brush_with_rotation(map, EditorBrush::Wall, tile, 0)
            {
                editor.ui.dirty = true;
                result.map_changed = true;
            }
            let remove_prop = Rect::new(
                layout.right_panel.x + pad,
                y0 + 134.0 * scale,
                bw,
                28.0 * scale,
            );
            if draw_ui_button_sized(
                remove_prop,
                "Supprimer objet",
                mouse,
                left_click,
                false,
                13.0 * scale,
            ) && editor_apply_brush_with_rotation(map, EditorBrush::EraseProp, tile, 0)
            {
                editor.ui.dirty = true;
                result.map_changed = true;
            }
            let set_zone = Rect::new(
                layout.right_panel.x + pad,
                y0 + 168.0 * scale,
                bw,
                28.0 * scale,
            );
            if draw_ui_button_sized(
                set_zone,
                "Appliquer type de zone actif",
                mouse,
                left_click,
                false,
                12.0 * scale,
            ) && set_zone_kind_at_tile(map, tile, Some(editor.zone_kind))
            {
                editor.ui.dirty = true;
                result.map_changed = true;
            }
            let set_player = Rect::new(
                layout.right_panel.x + pad,
                y0 + 206.0 * scale,
                bw,
                28.0 * scale,
            );
            if draw_ui_button_sized(
                set_player,
                "Set spawn joueur",
                mouse,
                left_click,
                false,
                12.5 * scale,
            ) {
                match editor_set_player_spawn(editor, map, tile) {
                    Ok(()) => {
                        editor.ui.dirty = true;
                        result.map_changed = true;
                    }
                    Err(reason) => {
                        editor_set_status(editor, format!("Spawn joueur invalide: {reason}"))
                    }
                }
            }
            let set_npc = Rect::new(
                layout.right_panel.x + pad,
                y0 + 240.0 * scale,
                bw,
                28.0 * scale,
            );
            if draw_ui_button_sized(
                set_npc,
                "Set spawn PNJ",
                mouse,
                left_click,
                false,
                12.5 * scale,
            ) {
                match editor_set_npc_spawn(editor, map, tile) {
                    Ok(()) => {
                        editor.ui.dirty = true;
                        result.map_changed = true;
                    }
                    Err(reason) => {
                        editor_set_status(editor, format!("Spawn PNJ invalide: {reason}"))
                    }
                }
            }
        } else {
            draw_ui_text_tinted_on(
                panel_bg,
                Color::from_rgba(184, 206, 220, 255),
                "Aucune selection.\nUtilise l'outil Selection (S) ou clic droit dans la carte.",
                layout.right_panel.x + pad,
                y0,
                15.0 * scale,
            );
        }
    } else if editor.ui.right_tab == EditorRightTab::Carte {
        let y0 = right_tabs_rect.y + right_tabs_rect.h + 12.0 * scale;
        let info = format!(
            "Carte: {}\nDimensions: {} x {}\nSpawns: J({}, {}) | N({}, {})\nObjets: {}\nZones: {}\nCamera: x={:.0} y={:.0} z={:.2}",
            map.label,
            map.world.w,
            map.world.h,
            map.player_spawn.0,
            map.player_spawn.1,
            map.npc_spawn.0,
            map.npc_spawn.1,
            map.props.len(),
            map.zones.len(),
            editor.camera_center.x,
            editor.camera_center.y,
            editor.camera_zoom
        );
        draw_ui_text_tinted_on(
            panel_bg,
            Color::from_rgba(192, 218, 232, 255),
            &info,
            layout.right_panel.x + pad,
            y0,
            15.0 * scale,
        );
        let center = Rect::new(
            layout.right_panel.x + pad,
            y0 + 158.0 * scale,
            layout.right_panel.w - pad * 2.0,
            30.0 * scale,
        );
        if draw_ui_button_sized(
            center,
            "Centrer camera sur spawn joueur",
            mouse,
            left_click,
            false,
            12.5 * scale,
        ) {
            result.center_camera_on = Some(map.player_spawn);
        }
        ui_kit::tooltip_for_rect(
            &mut editor.ui.tooltip,
            "map_center_spawn",
            "Place la camera sur le spawn joueur.",
            center,
            mouse,
        );
        let outliner_r = Rect::new(
            layout.right_panel.x + pad,
            y0 + 196.0 * scale,
            layout.right_panel.w - pad * 2.0,
            layout.right_panel.h - (y0 - layout.right_panel.y) - 208.0 * scale,
        );
        draw_rectangle(
            outliner_r.x,
            outliner_r.y,
            outliner_r.w,
            outliner_r.h,
            Color::from_rgba(8, 13, 20, 132),
        );
        draw_rectangle_lines(
            outliner_r.x + 0.5,
            outliner_r.y + 0.5,
            outliner_r.w - 1.0,
            outliner_r.h - 1.0,
            1.0,
            Color::from_rgba(90, 130, 154, 142),
        );

        let mut outliner_entries: Vec<(String, Option<(i32, i32)>)> = Vec::new();
        outliner_entries.push(("Spawn joueur".to_string(), Some(map.player_spawn)));
        outliner_entries.push(("Spawn PNJ".to_string(), Some(map.npc_spawn)));
        for zone in &map.zones {
            let tile = zone.tiles.first().copied();
            outliner_entries.push((
                format!("Zone {} [{}]", zone.label, zone_kind_label(zone.kind)),
                tile,
            ));
        }
        const OUTLINER_PROP_LIMIT: usize = 220;
        for prop in map.props.iter().take(OUTLINER_PROP_LIMIT) {
            outliner_entries.push((
                format!(
                    "Objet {} ({}, {})",
                    prop_kind_label(prop.kind),
                    prop.tile_x,
                    prop.tile_y
                ),
                Some((prop.tile_x, prop.tile_y)),
            ));
        }
        if map.props.len() > OUTLINER_PROP_LIMIT {
            outliner_entries.push((
                format!(
                    "... {} objets supplementaires",
                    map.props.len() - OUTLINER_PROP_LIMIT
                ),
                None,
            ));
        }

        let row_h = 30.0 * scale;
        let visible_rows = ((outliner_r.h - 8.0) / row_h).floor().max(1.0) as usize;
        ui_kit::update_scroll_with_wheel(
            &mut editor.ui.outliner_scroll,
            outliner_entries.len(),
            visible_rows,
            wheel_y,
            point_in_rect(mouse, outliner_r),
        );
        let start = editor.ui.outliner_scroll;
        let end = (start + visible_rows).min(outliner_entries.len());
        let mut y = outliner_r.y + 4.0;
        for (index, (label, tile)) in outliner_entries[start..end].iter().enumerate() {
            let row = Rect::new(outliner_r.x + 4.0, y, outliner_r.w - 8.0, row_h - 2.0);
            let targetable = tile.is_some();
            if draw_ui_button_sized(row, label, mouse, left_click, false, 12.4 * scale)
                && targetable
                && let Some(hit) = tile
            {
                result.center_camera_on = Some(*hit);
                editor.ui.outliner_scroll = start + index;
            }
            y += row_h;
        }
    } else {
        let y0 = right_tabs_rect.y + right_tabs_rect.h + 12.0 * scale;
        let list_r = Rect::new(
            layout.right_panel.x + pad,
            y0,
            layout.right_panel.w - pad * 2.0,
            layout.right_panel.h - (y0 - layout.right_panel.y) - 12.0 * scale,
        );
        draw_rectangle(
            list_r.x,
            list_r.y,
            list_r.w,
            list_r.h,
            Color::from_rgba(8, 13, 20, 132),
        );
        draw_rectangle_lines(
            list_r.x + 0.5,
            list_r.y + 0.5,
            list_r.w - 1.0,
            list_r.h - 1.0,
            1.0,
            Color::from_rgba(90, 130, 154, 142),
        );
        let row_h = 38.0 * scale;
        let visible_rows = ((list_r.h - 8.0) / row_h).floor().max(1.0) as usize;
        ui_kit::update_scroll_with_wheel(
            &mut editor.validation_scroll,
            editor.validation_issues.len(),
            visible_rows,
            wheel_y,
            point_in_rect(mouse, list_r),
        );
        let start = editor.validation_scroll;
        let end = (start + visible_rows).min(editor.validation_issues.len());
        let mut y = list_r.y + 4.0;
        if editor.validation_issues.is_empty() {
            draw_ui_text_tinted_on(
                panel_bg,
                Color::from_rgba(148, 214, 164, 255),
                "Aucun probleme detecte.",
                list_r.x + 10.0,
                list_r.y + 24.0,
                14.0 * scale,
            );
        } else {
            for issue in &editor.validation_issues[start..end] {
                let row = Rect::new(list_r.x + 4.0, y, list_r.w - 8.0, row_h - 2.0);
                let col = issue_severity_color(issue.severity);
                let active = draw_ui_button_sized(
                    row,
                    &issue.message,
                    mouse,
                    left_click,
                    false,
                    12.5 * scale,
                );
                draw_rectangle(row.x + 2.0, row.y + row.h - 4.0, row.w - 4.0, 2.0, col);
                if active && let Some(tile) = issue.tile {
                    result.center_camera_on = Some(tile);
                }
                y += row_h;
            }
        }
    }

    draw_rectangle(
        layout.bottom_bar.x,
        layout.bottom_bar.y,
        layout.bottom_bar.w,
        layout.bottom_bar.h,
        Color::from_rgba(8, 12, 18, 210),
    );
    draw_rectangle_lines(
        layout.bottom_bar.x + 0.5,
        layout.bottom_bar.y + 0.5,
        layout.bottom_bar.w - 1.0,
        layout.bottom_bar.h - 1.0,
        1.0,
        Color::from_rgba(90, 126, 149, 170),
    );
    let status = if editor.status_timer > 0.0 {
        editor.status_text.as_str()
    } else {
        "Pret"
    };
    let hover = editor
        .hover_tile
        .map(|(x, y)| format!("x={x} y={y}"))
        .unwrap_or_else(|| "x=- y=-".to_string());
    let overlay = format!(
        "{} | outil={} brush={} taille={} zone={} | {} | zoom {:.2}",
        status,
        editor_tool_label(editor.tool),
        editor_brush_label(editor.brush),
        editor.brush_size,
        zone_kind_label(editor.zone_kind),
        hover,
        editor.camera_zoom
    );
    draw_ui_text_tinted_on(
        Color::from_rgba(8, 12, 18, 210),
        if editor.ui.high_contrast {
            Color::from_rgba(255, 244, 204, 255)
        } else {
            Color::from_rgba(252, 232, 188, 255)
        },
        &overlay,
        layout.bottom_bar.x + 12.0 * scale,
        layout.bottom_bar.y + layout.bottom_bar.h * 0.72,
        14.0 * scale,
    );

    if editor.ui.show_unsaved_modal {
        editor.ui.context_menu = None;
        let mw = (420.0 * scale).min(screen_width() - 40.0);
        let mh = 170.0 * scale;
        let mx = (screen_width() - mw) * 0.5;
        let my = (screen_height() - mh) * 0.5;
        let ui = palette.ui;
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            with_alpha(ui.panel_inset, 0.64),
        );
        draw_rectangle(mx, my, mw, mh, with_alpha(ui.panel_mid, 0.96));
        draw_rectangle(mx, my, mw, mh * 0.36, with_alpha(ui.panel_top, 0.70));
        draw_rectangle_lines(
            mx + 0.5,
            my + 0.5,
            mw - 1.0,
            mh - 1.0,
            1.4,
            with_alpha(ui.border, 0.92),
        );
        draw_ui_text_tinted_on(
            with_alpha(ui.panel_mid, 0.96),
            ui.text_primary,
            "Changements non sauvegardes.",
            mx + 14.0 * scale,
            my + 34.0 * scale,
            20.0 * scale,
        );
        draw_ui_text_tinted_on(
            with_alpha(ui.panel_mid, 0.96),
            ui.text_secondary,
            "Sauvegarder avant de quitter/changer ?",
            mx + 14.0 * scale,
            my + 62.0 * scale,
            15.0 * scale,
        );
        let btn_w = 118.0 * scale;
        let btn_h = 34.0 * scale;
        let gap = 10.0 * scale;
        let by = my + mh - btn_h - 14.0 * scale;
        let bx = mx + mw - (btn_w * 3.0 + gap * 2.0) - 14.0 * scale;
        let save_b = Rect::new(bx, by, btn_w, btn_h);
        let discard_b = Rect::new(bx + btn_w + gap, by, btn_w, btn_h);
        let cancel_b = Rect::new(bx + (btn_w + gap) * 2.0, by, btn_w, btn_h);

        if draw_ui_button_sized(save_b, "Sauver", mouse, left_click, false, 13.0 * scale) {
            editor_save_current_map(editor, map);
            if let Some(action) = editor.ui.pending_action.take() {
                result.action = action;
            }
            editor.ui.show_unsaved_modal = false;
        }
        if draw_ui_button_sized(discard_b, "Ignorer", mouse, left_click, false, 13.0 * scale) {
            editor.ui.dirty = false;
            if let Some(action) = editor.ui.pending_action.take() {
                result.action = action;
            }
            editor.ui.show_unsaved_modal = false;
        }
        if draw_ui_button_sized(cancel_b, "Annuler", mouse, left_click, false, 13.0 * scale) {
            editor.ui.pending_action = None;
            editor.ui.show_unsaved_modal = false;
        }
    } else {
        draw_editor_context_menu(editor, map, mouse, left_click, scale, &mut result);
    }

    ui_kit::draw_tooltip(&editor.ui.tooltip, scale);

    if editor.ui.settings_dirty
        && let Err(err) = save_editor_ui_state(EDITOR_UI_SETTINGS_PATH, &mut editor.ui)
    {
        editor_set_status(editor, format!("Sauvegarde UI impossible: {err}"));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_layout_preserves_minimum_map_width_on_small_screen() {
        let ui = EditorUiState::new();
        let layout = editor_compute_layout_for_size(960.0, 540.0, &ui);
        assert!(layout.map_view.w >= 320.0 * ui.ui_scale);
        assert!(layout.left_panel.w >= 180.0 * ui.ui_scale);
        assert!(layout.right_panel.w >= 200.0 * ui.ui_scale);
    }

    #[test]
    fn editor_layout_expands_side_panels_on_large_screen_within_budget() {
        let mut ui = EditorUiState::new();
        ui.left_panel_ratio = 0.32;
        ui.right_panel_ratio = 0.34;
        ui.ui_scale = 1.2;
        let layout = editor_compute_layout_for_size(1920.0, 1080.0, &ui);
        assert!(layout.left_panel.w <= 440.0 * ui.ui_scale + 0.001);
        assert!(layout.right_panel.w <= 560.0 * ui.ui_scale + 0.001);
        assert!(layout.map_view.w > layout.left_panel.w * 0.6);
        assert!(layout.map_view.h > 0.0);
    }

    #[test]
    fn editor_feedback_colors_stay_distinct_by_semantics() {
        assert_ne!(
            zone_color(ZoneKind::Logistique, 200),
            zone_color(ZoneKind::Production, 200)
        );
        assert_ne!(
            issue_severity_color(ValidationSeverity::Error),
            issue_severity_color(ValidationSeverity::Info)
        );
    }
}
