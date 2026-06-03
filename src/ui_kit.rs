quse super::*;
use crate::rendu::theme::{mix_color, ui_panel_fill, ui_theme};

const TOOLTIP_DELAY_S: f64 = 0.5;

#[derive(Default)]
pub(crate) struct UiTooltipState {
    hover_id: Option<String>,
    hover_since_s: f64,
    current_text: Option<String>,
    current_anchor: Vec2,
}

impl UiTooltipState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub(crate) fn tooltip_begin_frame(state: &mut UiTooltipState) {
    state.current_text = None;
}

pub(crate) fn tooltip_register(
    state: &mut UiTooltipState,
    id: &str,
    text: &str,
    hovered: bool,
    mouse: Vec2,
) {
    if !hovered {
        if state.hover_id.as_deref() == Some(id) {
            state.hover_id = None;
            state.hover_since_s = 0.0;
        }
        return;
    }

    let now = get_time();
    if state.hover_id.as_deref() != Some(id) {
        state.hover_id = Some(id.to_string());
        state.hover_since_s = now;
        return;
    }

    if now - state.hover_since_s >= TOOLTIP_DELAY_S {
        state.current_text = Some(text.to_string());
        state.current_anchor = mouse;
    }
}

pub(crate) fn draw_tooltip(state: &UiTooltipState, ui_scale: f32) {
    let Some(text) = state.current_text.as_deref() else {
        return;
    };
    let font_size = 13.0 * ui_scale;
    let pad = 8.0 * ui_scale;
    let dims = measure_text(text, None, font_size as u16, 1.0);
    let w = dims.width + pad * 2.0;
    let h = dims.height + pad * 2.0;
    let mut x = state.current_anchor.x + 14.0 * ui_scale;
    let mut y = state.current_anchor.y + 18.0 * ui_scale;
    x = x.clamp(8.0, screen_width() - w - 8.0);
    y = y.clamp(8.0, screen_height() - h - 8.0);
    let (top, bg) = ui_panel_fill(true);
    draw_rectangle(x, y, w, h, bg);
    draw_rectangle(x, y, w, h * 0.42, with_alpha(top, 0.80));
    draw_rectangle_lines(
        x + 0.5,
        y + 0.5,
        w - 1.0,
        h - 1.0,
        1.0,
        with_alpha(ui_theme().border, 0.88),
    );
    draw_ui_text_tinted_on(
        bg,
        Color::from_rgba(230, 244, 252, 255),
        text,
        x + pad,
        y + h * 0.66,
        font_size,
    );
}

pub(crate) fn tooltip_for_rect(
    tooltip_state: &mut UiTooltipState,
    tooltip_id: &str,
    tooltip_text: &str,
    rect: Rect,
    mouse: Vec2,
) {
    tooltip_register(
        tooltip_state,
        tooltip_id,
        tooltip_text,
        point_in_rect(mouse, rect),
        mouse,
    );
}

pub(crate) fn draw_tab_row(
    rect: Rect,
    tabs: &[(&str, bool)],
    mouse: Vec2,
    left_click: bool,
    font_size: f32,
) -> Option<usize> {
    let count = tabs.len().max(1);
    let gap = 6.0;
    let tab_w = ((rect.w - gap * (count as f32 - 1.0)) / count as f32).max(10.0);
    let mut x = rect.x;
    let mut clicked = None;
    for (i, (label, active)) in tabs.iter().enumerate() {
        let r = Rect::new(x, rect.y, tab_w, rect.h);
        if draw_ui_button_sized(r, label, mouse, left_click, *active, font_size) {
            clicked = Some(i);
        }
        x += tab_w + gap;
    }
    clicked
}

pub(crate) fn draw_text_input(
    rect: Rect,
    value: &mut String,
    focused: &mut bool,
    mouse: Vec2,
    left_click: bool,
    font_size: f32,
    placeholder: &str,
) -> bool {
    let ui = ui_theme();
    let (panel_top, panel_bottom) = ui_panel_fill(*focused);
    let bg = if *focused {
        mix_color(panel_top, ui.accent_cyan, 0.18)
    } else {
        panel_bottom
    };
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h * 0.46,
        with_alpha(panel_top, if *focused { 0.74 } else { 0.56 }),
    );
    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.w - 1.0,
        rect.h - 1.0,
        1.4,
        with_alpha(ui.border, 0.84),
    );

    if left_click {
        *focused = point_in_rect(mouse, rect);
    }

    let pad = 8.0;
    let show = if value.is_empty() {
        placeholder
    } else {
        value.as_str()
    };
    let fill = if value.is_empty() {
        with_alpha(ui.text_secondary, 0.78)
    } else {
        ui.text_primary
    };
    draw_ui_text_tinted_on(
        bg,
        fill,
        show,
        rect.x + pad,
        rect.y + rect.h * 0.62,
        font_size,
    );

    if *focused {
        let mut changed = false;
        while let Some(ch) = get_char_pressed() {
            if ch.is_control() {
                continue;
            }
            value.push(ch);
            changed = true;
        }
        if is_key_pressed(KeyCode::Backspace) && !value.is_empty() {
            value.pop();
            changed = true;
        }
        if is_key_pressed(KeyCode::Escape) {
            *focused = false;
        }
        return changed;
    }

    false
}

pub(crate) fn clamp_scroll_offset(offset: usize, total_rows: usize, visible_rows: usize) -> usize {
    offset.min(total_rows.saturating_sub(visible_rows.max(1)))
}

pub(crate) fn update_scroll_with_wheel(
    offset: &mut usize,
    total_rows: usize,
    visible_rows: usize,
    wheel_y: f32,
    pointer_inside: bool,
) {
    *offset = clamp_scroll_offset(*offset, total_rows, visible_rows);
    if !pointer_inside || wheel_y.abs() <= f32::EPSILON {
        return;
    }
    let max_offset = total_rows.saturating_sub(visible_rows.max(1));
    if wheel_y > 0.0 {
        *offset = offset.saturating_sub(1);
    } else if wheel_y < 0.0 {
        *offset = (*offset + 1).min(max_offset);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_scroll_offset_respects_row_window() {
        assert_eq!(clamp_scroll_offset(0, 0, 5), 0);
        assert_eq!(clamp_scroll_offset(10, 4, 3), 1);
        assert_eq!(clamp_scroll_offset(2, 20, 8), 2);
    }

    #[test]
    fn update_scroll_moves_only_when_pointer_is_inside() {
        let mut offset = 4usize;
        update_scroll_with_wheel(&mut offset, 20, 5, 1.0, false);
        assert_eq!(offset, 4);
        update_scroll_with_wheel(&mut offset, 20, 5, 1.0, true);
        assert_eq!(offset, 3);
    }
}
