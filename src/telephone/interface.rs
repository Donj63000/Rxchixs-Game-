use super::{contact_label, contacts_disponibles};
use macroquad::prelude::*;

const PANEL_TITLE_SIZE: f32 = 15.0;
const CONTACT_TEXT_SIZE: f32 = 14.0;
const CONTACTS_HEADER_H: f32 = 28.0;
const CONTACT_ROW_H: f32 = 30.0;
const CONTACT_ROW_STEP: f32 = 34.0;
const CONTACTS_PANEL_PAD: f32 = 8.0;

pub fn contacts_panel_rect(panel: Rect) -> Rect {
    let contacts_h =
        (CONTACTS_HEADER_H + contacts_disponibles().len() as f32 * CONTACT_ROW_STEP + 6.0)
            .max(62.0);
    let max_w = (panel.x + panel.w - 16.0).max(90.0);
    let w = (panel.w * 1.10).clamp(150.0, 220.0).min(max_w);
    let x = (panel.x + panel.w - w).max(8.0);
    Rect::new(x, (panel.y - contacts_h - 8.0).max(8.0), w, contacts_h)
}

fn bouton_telephone_rect(panel: Rect) -> Rect {
    let btn_h = (panel.h * 0.28).clamp(36.0, 52.0);
    let btn_w = (panel.w - 20.0).max(40.0);
    Rect::new(
        panel.x + (panel.w - btn_w) * 0.5,
        panel.y + panel.h - btn_h - 8.0,
        btn_w,
        btn_h,
    )
}

fn contact_row_rect(contacts_panel: Rect, idx: usize) -> Rect {
    let y = contacts_panel.y + CONTACTS_HEADER_H + idx as f32 * CONTACT_ROW_STEP;
    Rect::new(
        contacts_panel.x + CONTACTS_PANEL_PAD,
        y,
        contacts_panel.w - CONTACTS_PANEL_PAD * 2.0,
        CONTACT_ROW_H,
    )
}

fn truncate_safe_text(text: &str, max_chars: usize) -> &str {
    if max_chars == 0 {
        return "";
    }
    let mut byte_end = text.len();
    for (char_count, (byte_idx, _)) in text.char_indices().enumerate() {
        if char_count == max_chars {
            byte_end = byte_idx;
            break;
        }
    }
    &text[..byte_end]
}

fn draw_phone_text(text: &str, x: f32, y: f32, size: f32, color: Color) {
    crate::render_safety::begin_ui_pass();
    draw_text(text, x + 1.0, y + 1.0, size, Color::from_rgba(0, 0, 0, 110));
    draw_text(text, x, y, size, color);
}

pub fn telephone_panel_contains_mouse(state: &crate::GameState, panel: Rect, mouse: Vec2) -> bool {
    if crate::point_in_rect(mouse, panel) {
        return true;
    }
    if state.telephone.ouvert {
        return crate::point_in_rect(mouse, contacts_panel_rect(panel));
    }
    false
}

pub fn process_telephone_panel_input(
    state: &mut crate::GameState,
    panel: Rect,
    mouse: Vec2,
) -> bool {
    let bouton = bouton_telephone_rect(panel);
    if crate::point_in_rect(mouse, bouton) {
        state.telephone.basculer_ouverture();
        return true;
    }

    if !state.telephone.ouvert {
        return false;
    }

    let contacts_panel = contacts_panel_rect(panel);
    if !crate::point_in_rect(mouse, contacts_panel) {
        return false;
    }

    for (idx, contact) in contacts_disponibles().iter().enumerate() {
        let row = contact_row_rect(contacts_panel, idx);
        if crate::point_in_rect(mouse, row) {
            state.telephone.demander_appel(*contact);
            state
                .telephone
                .definir_statut(format!("Appel en cours: {}", contact_label(*contact)));
            return true;
        }
    }
    true
}

pub fn draw_telephone_panel(state: &crate::GameState, panel: Rect, mouse: Vec2, time: f32) {
    let hovered = crate::point_in_rect(mouse, panel);
    draw_rectangle(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        Color::from_rgba(7, 14, 24, 236),
    );
    draw_rectangle(
        panel.x + 1.0,
        panel.y + 1.0,
        (panel.w - 2.0).max(1.0),
        24.0,
        Color::from_rgba(18, 34, 50, 216),
    );
    draw_rectangle(
        panel.x,
        panel.y + panel.h * 0.54,
        panel.w,
        panel.h * 0.46,
        Color::from_rgba(0, 0, 0, 52),
    );
    draw_rectangle_lines(
        panel.x,
        panel.y,
        panel.w,
        panel.h,
        1.8,
        if hovered {
            Color::from_rgba(150, 196, 224, 214)
        } else {
            Color::from_rgba(72, 122, 166, 154)
        },
    );

    draw_line(
        panel.x + 16.0,
        panel.y + 13.0,
        panel.x + 23.0,
        panel.y + 20.0,
        2.2,
        Color::from_rgba(150, 190, 220, 220),
    );
    draw_line(
        panel.x + 23.0,
        panel.y + 20.0,
        panel.x + 31.0,
        panel.y + 13.0,
        2.2,
        Color::from_rgba(150, 190, 220, 220),
    );
    draw_phone_text(
        "TELEPHONE",
        panel.x + 42.0,
        panel.y + 22.0,
        PANEL_TITLE_SIZE,
        Color::from_rgba(224, 236, 244, 246),
    );

    let screen_h = (panel.h * 0.34).clamp(46.0, 76.0);
    let screen = Rect::new(panel.x + 18.0, panel.y + 44.0, panel.w - 36.0, screen_h);
    draw_rectangle(
        screen.x,
        screen.y,
        screen.w,
        screen.h,
        Color::from_rgba(4, 10, 18, 224),
    );
    draw_rectangle_lines(
        screen.x + 0.6,
        screen.y + 0.6,
        (screen.w - 1.2).max(1.0),
        (screen.h - 1.2).max(1.0),
        1.0,
        Color::from_rgba(72, 122, 166, 126),
    );
    let blink = (time * 2.4).sin().abs();
    draw_circle(
        screen.x + 11.0,
        screen.y + 14.0,
        1.4,
        Color::from_rgba(70, 180, 170, 72 + (blink * 70.0) as u8),
    );

    let bouton = bouton_telephone_rect(panel);
    draw_rectangle(
        bouton.x,
        bouton.y,
        bouton.w,
        bouton.h,
        Color::from_rgba(18, 52, 72, 232),
    );
    draw_rectangle(
        bouton.x,
        bouton.y,
        bouton.w,
        bouton.h * 0.44,
        Color::from_rgba(120, 160, 190, 34),
    );
    draw_rectangle_lines(
        bouton.x + 0.6,
        bouton.y + 0.6,
        (bouton.w - 1.2).max(1.0),
        (bouton.h - 1.2).max(1.0),
        1.2,
        Color::from_rgba(96, 156, 190, 198),
    );

    let pulse = if state.telephone.ouvert {
        0.45 + 0.55 * (time * 4.0).sin().abs()
    } else {
        0.35
    };
    let icon = Rect::new(
        bouton.x + bouton.w * 0.30,
        bouton.y + bouton.h * 0.24,
        bouton.w * 0.40,
        bouton.h * 0.52,
    );
    let icon_col = Color::from_rgba(88, 186, 206, 168 + (pulse * 38.0) as u8);
    draw_line(
        icon.x + icon.w * 0.18,
        icon.y + icon.h * 0.18,
        icon.x + icon.w * 0.82,
        icon.y + icon.h * 0.82,
        5.0,
        icon_col,
    );
    draw_circle(
        icon.x + icon.w * 0.18,
        icon.y + icon.h * 0.18,
        5.0,
        icon_col,
    );
    draw_circle(
        icon.x + icon.w * 0.82,
        icon.y + icon.h * 0.82,
        5.0,
        icon_col,
    );
    let label = if state.telephone.ouvert {
        "FERMER"
    } else {
        "CONTACTS"
    };
    let label_size = 12.0;
    let dims = measure_text(label, None, label_size as u16, 1.0);
    draw_phone_text(
        label,
        bouton.x + bouton.w * 0.5 - dims.width * 0.5,
        bouton.y + bouton.h - 8.0,
        label_size,
        Color::from_rgba(230, 240, 246, 238),
    );

    if state.telephone.ouvert {
        let contacts_panel = contacts_panel_rect(panel);
        draw_rectangle(
            contacts_panel.x,
            contacts_panel.y,
            contacts_panel.w,
            contacts_panel.h,
            Color::from_rgba(8, 14, 24, 242),
        );
        draw_rectangle_lines(
            contacts_panel.x,
            contacts_panel.y,
            contacts_panel.w,
            contacts_panel.h,
            1.6,
            Color::from_rgba(96, 154, 196, 204),
        );
        draw_phone_text(
            "Contacts",
            contacts_panel.x + 10.0,
            contacts_panel.y + 20.0,
            17.0,
            Color::from_rgba(226, 238, 246, 248),
        );

        for (idx, contact) in contacts_disponibles().iter().enumerate() {
            let row = contact_row_rect(contacts_panel, idx);
            let row_hovered = crate::point_in_rect(mouse, row);
            draw_rectangle(
                row.x,
                row.y,
                row.w,
                row.h,
                if row_hovered {
                    Color::from_rgba(34, 64, 92, 242)
                } else {
                    Color::from_rgba(18, 36, 56, 236)
                },
            );
            draw_rectangle_lines(
                row.x + 0.6,
                row.y + 0.6,
                (row.w - 1.2).max(1.0),
                (row.h - 1.2).max(1.0),
                1.0,
                Color::from_rgba(92, 146, 190, 162),
            );
            draw_phone_text(
                contact_label(*contact),
                row.x + 8.0,
                row.y + 20.0,
                CONTACT_TEXT_SIZE,
                Color::from_rgba(232, 242, 248, 246),
            );
        }
    }

    if let Some(statut) = state.telephone.dernier_statut.as_deref() {
        let clipped = truncate_safe_text(statut, 20);
        draw_phone_text(
            clipped,
            screen.x + 8.0,
            screen.y + screen.h - 12.0,
            14.0,
            Color::from_rgba(182, 224, 244, 220),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_safe_text_handles_utf8_without_panicking() {
        let s = "Erreur: fichier spécifié introuvable";
        let out = truncate_safe_text(s, 20);
        assert!(!out.is_empty());
        assert!(out.chars().count() <= 20);
    }

    #[test]
    fn contacts_panel_rect_stays_inside_left_margin() {
        let phone_panel = Rect::new(20.0, 400.0, 90.0, 120.0);
        let contacts = contacts_panel_rect(phone_panel);
        assert!(contacts.x >= 8.0);
        assert!(contacts.w <= 220.0);
        assert!(contacts.x + contacts.w <= phone_panel.x + phone_panel.w + 0.001);
    }

    #[test]
    fn sober_phone_button_keeps_click_target_inside_panel() {
        for panel in [
            Rect::new(0.0, 0.0, 90.0, 120.0),
            Rect::new(500.0, 600.0, 150.0, 150.0),
            Rect::new(1200.0, 740.0, 170.0, 170.0),
        ] {
            let button = bouton_telephone_rect(panel);
            assert!(button.x >= panel.x);
            assert!(button.y >= panel.y);
            assert!(button.x + button.w <= panel.x + panel.w + 0.001);
            assert!(button.y + button.h <= panel.y + panel.h + 0.001);
            assert!(button.h >= 36.0);
        }
    }
}
