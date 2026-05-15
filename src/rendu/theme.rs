use super::*;

#[derive(Copy, Clone)]
pub(crate) struct WorldTheme {
    pub bg_top: Color,
    pub bg_mid: Color,
    pub bg_bottom: Color,
    pub bg_haze: Color,
    pub floor_a: Color,
    pub floor_b: Color,
    pub floor_c: Color,
    pub floor_edge: Color,
    pub floor_grime: Color,
    pub floor_marking: Color,
    pub wall_top: Color,
    pub wall_mid: Color,
    pub wall_dark: Color,
    pub wall_outline: Color,
    pub shadow_soft: Color,
    pub shadow_hard: Color,
    pub vignette: Color,
    pub lamp_warm: Color,
    pub lamp_hot: Color,
    pub prop_crate_light: Color,
    pub prop_crate_dark: Color,
    pub prop_pipe: Color,
    pub prop_pipe_highlight: Color,
    pub steel_cool: Color,
    pub steel_deep: Color,
    pub safety_amber: Color,
    pub safety_red: Color,
    pub concrete_moss: Color,
    pub exterior_grass: Color,
    pub dust: Color,
}

#[derive(Copy, Clone)]
pub(crate) struct UiTheme {
    pub panel_top: Color,
    pub panel_mid: Color,
    pub panel_bottom: Color,
    pub panel_inset: Color,
    pub border: Color,
    pub border_hi: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent_amber: Color,
    pub accent_cyan: Color,
    pub accent_teal: Color,
    pub accent_green: Color,
    pub accent_red: Color,
    pub accent_steel: Color,
}

#[derive(Copy, Clone)]
pub(crate) struct FeedbackTheme {
    pub info: Color,
    pub positive: Color,
    pub warning: Color,
    pub danger: Color,
    pub money: Color,
    pub logistics: Color,
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) struct Palette {
    pub world: WorldTheme,
    pub ui: UiTheme,
    pub feedback: FeedbackTheme,
    pub bg_top: Color,
    pub bg_bottom: Color,
    pub floor_a: Color,
    pub floor_b: Color,
    pub floor_c: Color,
    pub floor_edge: Color,
    pub floor_grime: Color,
    pub wall_top: Color,
    pub wall_mid: Color,
    pub wall_dark: Color,
    pub wall_outline: Color,
    pub shadow_soft: Color,
    pub shadow_hard: Color,
    pub vignette: Color,
    pub lamp_warm: Color,
    pub lamp_hot: Color,
    pub prop_crate_light: Color,
    pub prop_crate_dark: Color,
    pub prop_pipe: Color,
    pub prop_pipe_highlight: Color,
    pub dust: Color,
}

impl Palette {
    pub(crate) fn new() -> Self {
        let world = world_theme();
        let ui = ui_theme();
        let feedback = feedback_theme();
        Self {
            world,
            ui,
            feedback,
            bg_top: world.bg_top,
            bg_bottom: world.bg_bottom,
            floor_a: world.floor_a,
            floor_b: world.floor_b,
            floor_c: world.floor_c,
            floor_edge: world.floor_edge,
            floor_grime: world.floor_grime,
            wall_top: world.wall_top,
            wall_mid: world.wall_mid,
            wall_dark: world.wall_dark,
            wall_outline: world.wall_outline,
            shadow_soft: world.shadow_soft,
            shadow_hard: world.shadow_hard,
            vignette: world.vignette,
            lamp_warm: world.lamp_warm,
            lamp_hot: world.lamp_hot,
            prop_crate_light: world.prop_crate_light,
            prop_crate_dark: world.prop_crate_dark,
            prop_pipe: world.prop_pipe,
            prop_pipe_highlight: world.prop_pipe_highlight,
            dust: world.dust,
        }
    }
}

pub(crate) fn world_theme() -> WorldTheme {
    WorldTheme {
        bg_top: rgba(5, 17, 31, 255),
        bg_mid: rgba(8, 24, 38, 255),
        bg_bottom: rgba(10, 20, 32, 255),
        bg_haze: rgba(72, 132, 154, 255),
        floor_a: rgba(78, 92, 94, 255),
        floor_b: rgba(96, 110, 112, 255),
        floor_c: rgba(58, 72, 76, 255),
        floor_edge: rgba(18, 42, 34, 220),
        floor_grime: rgba(6, 12, 10, 255),
        floor_marking: rgba(212, 170, 76, 255),
        wall_top: rgba(148, 162, 174, 255),
        wall_mid: rgba(96, 112, 128, 255),
        wall_dark: rgba(58, 71, 84, 255),
        wall_outline: rgba(14, 19, 25, 225),
        shadow_soft: rgba(7, 10, 16, 140),
        shadow_hard: rgba(4, 6, 10, 205),
        vignette: rgba(2, 4, 7, 255),
        lamp_warm: rgba(238, 184, 92, 255),
        lamp_hot: rgba(255, 228, 176, 255),
        prop_crate_light: rgba(164, 128, 92, 255),
        prop_crate_dark: rgba(112, 82, 58, 255),
        prop_pipe: rgba(88, 112, 132, 255),
        prop_pipe_highlight: rgba(172, 194, 212, 255),
        steel_cool: rgba(120, 142, 162, 255),
        steel_deep: rgba(52, 66, 82, 255),
        safety_amber: rgba(228, 184, 84, 255),
        safety_red: rgba(214, 98, 86, 255),
        concrete_moss: rgba(64, 118, 70, 255),
        exterior_grass: rgba(72, 128, 64, 255),
        dust: rgba(182, 202, 214, 255),
    }
}

pub(crate) fn ui_theme() -> UiTheme {
    UiTheme {
        panel_top: rgba(13, 42, 70, 248),
        panel_mid: rgba(5, 24, 44, 246),
        panel_bottom: rgba(2, 12, 26, 250),
        panel_inset: rgba(2, 10, 22, 222),
        border: rgba(58, 146, 216, 226),
        border_hi: rgba(176, 232, 255, 250),
        text_primary: rgba(238, 247, 252, 255),
        text_secondary: rgba(178, 206, 224, 250),
        accent_amber: rgba(238, 188, 92, 252),
        accent_cyan: rgba(98, 206, 246, 252),
        accent_teal: rgba(108, 224, 192, 252),
        accent_green: rgba(118, 212, 136, 248),
        accent_red: rgba(222, 112, 100, 248),
        accent_steel: rgba(146, 172, 194, 248),
    }
}

pub(crate) fn feedback_theme() -> FeedbackTheme {
    let ui = ui_theme();
    FeedbackTheme {
        info: ui.accent_cyan,
        positive: ui.accent_green,
        warning: ui.accent_amber,
        danger: ui.accent_red,
        money: rgba(224, 198, 108, 252),
        logistics: rgba(126, 178, 236, 250),
    }
}

pub(crate) fn mix_color(a: Color, b: Color, t: f32) -> Color {
    color_lerp(a, b, t)
}

pub(crate) fn ui_panel_fill(hovered: bool) -> (Color, Color) {
    let ui = ui_theme();
    let top = if hovered {
        mix_color(ui.panel_top, ui.accent_cyan, 0.12)
    } else {
        mix_color(ui.panel_top, ui.panel_mid, 0.22)
    };
    let bottom = if hovered {
        mix_color(ui.panel_bottom, ui.panel_mid, 0.18)
    } else {
        ui.panel_bottom
    };
    (top, bottom)
}

pub(crate) fn ui_panel_header_fill(hovered: bool) -> (Color, Color) {
    let ui = ui_theme();
    let top = if hovered {
        mix_color(ui.panel_top, ui.accent_steel, 0.26)
    } else {
        mix_color(ui.panel_mid, ui.accent_steel, 0.14)
    };
    let bottom = if hovered {
        mix_color(ui.panel_mid, ui.panel_bottom, 0.24)
    } else {
        mix_color(ui.panel_bottom, ui.panel_mid, 0.18)
    };
    (top, bottom)
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub(crate) enum UiButtonKind {
    Neutral,
    Primary,
    Positive,
    Warning,
    Danger,
}

#[derive(Copy, Clone)]
pub(crate) struct ButtonVisual {
    pub top: Color,
    pub bottom: Color,
    pub border: Color,
    pub text: Color,
    pub shadow_alpha: f32,
}

pub(crate) fn ui_button_visual(hovered: bool, active: bool, kind: UiButtonKind) -> ButtonVisual {
    let ui = ui_theme();
    let accent = match kind {
        UiButtonKind::Neutral => ui.accent_steel,
        UiButtonKind::Primary => ui.accent_cyan,
        UiButtonKind::Positive => ui.accent_green,
        UiButtonKind::Warning => ui.accent_amber,
        UiButtonKind::Danger => ui.accent_red,
    };
    let base_top = if active {
        mix_color(accent, ui.panel_top, 0.18)
    } else if hovered {
        mix_color(ui.panel_top, accent, 0.32)
    } else {
        mix_color(ui.panel_mid, accent, 0.18)
    };
    let base_bottom = if active {
        mix_color(ui.panel_bottom, accent, 0.20)
    } else if hovered {
        mix_color(ui.panel_bottom, accent, 0.12)
    } else {
        ui.panel_bottom
    };
    let border = if active {
        mix_color(ui.border_hi, accent, 0.28)
    } else if hovered {
        mix_color(ui.border_hi, accent, 0.14)
    } else {
        mix_color(ui.border, accent, 0.18)
    };
    ButtonVisual {
        top: base_top,
        bottom: base_bottom,
        border,
        text: if active {
            ui_ensure_text_contrast(base_top, rgba(12, 18, 24, 255), 5.0)
        } else {
            ui_ensure_text_contrast(base_top, ui.text_primary, 4.5)
        },
        shadow_alpha: if active {
            0.24
        } else if hovered {
            0.21
        } else {
            0.16
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_keeps_legacy_world_aliases_in_sync() {
        let palette = Palette::new();
        assert_eq!(palette.bg_top, palette.world.bg_top);
        assert_eq!(palette.bg_bottom, palette.world.bg_bottom);
        assert_eq!(palette.wall_outline, palette.world.wall_outline);
        assert_eq!(palette.dust, palette.world.dust);
    }

    #[test]
    fn ui_button_visual_reacts_to_state_changes() {
        let idle = ui_button_visual(false, false, UiButtonKind::Neutral);
        let hovered = ui_button_visual(true, false, UiButtonKind::Neutral);
        let active = ui_button_visual(false, true, UiButtonKind::Primary);

        assert_ne!(idle.top, hovered.top);
        assert_ne!(idle.border, hovered.border);
        assert_ne!(hovered.text, active.text);
    }

    #[test]
    fn world_theme_uses_distinct_industrial_signal_colors() {
        let world = world_theme();
        assert_ne!(world.safety_amber, world.safety_red);
        assert_ne!(world.steel_cool, world.steel_deep);
        assert!(world.shadow_hard.a >= world.shadow_soft.a);
    }
}
