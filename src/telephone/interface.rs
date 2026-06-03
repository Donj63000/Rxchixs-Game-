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

fn phone_alpha(mut c: Color, alpha: f32) -> Color {
    c.a = alpha.clamp(0.0, 1.0);
    c
}

fn phone_round_rect(rect: Rect, radius: f32, color: Color) {
    if rect.w <= 0.0 || rect.h <= 0.0 {
        return;
    }

    let r = radius.max(0.0).min(rect.w * 0.5).min(rect.h * 0.5);

    if r <= 0.5 {
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, color);
        return;
    }

    draw_rectangle(rect.x + r, rect.y, rect.w - r * 2.0, rect.h, color);
    draw_rectangle(rect.x, rect.y + r, rect.w, rect.h - r * 2.0, color);

    draw_circle(rect.x + r, rect.y + r, r, color);
    draw_circle(rect.x + rect.w - r, rect.y + r, r, color);
    draw_circle(rect.x + r, rect.y + rect.h - r, r, color);
    draw_circle(rect.x + rect.w - r, rect.y + rect.h - r, r, color);
}

fn phone_panel(rect: Rect, hovered: bool) {
    phone_round_rect(
        Rect::new(rect.x + 2.0, rect.y + 4.0, rect.w, rect.h),
        12.0,
        Color::from_rgba(0, 0, 0, if hovered { 86 } else { 62 }),
    );

    phone_round_rect(
        rect,
        12.0,
        Color::from_rgba(86, 78, 58, if hovered { 190 } else { 138 }),
    );

    phone_round_rect(
        Rect::new(
            rect.x + 1.0,
            rect.y + 1.0,
            (rect.w - 2.0).max(0.0),
            (rect.h - 2.0).max(0.0),
        ),
        11.0,
        Color::from_rgba(10, 11, 11, 236),
    );

    phone_round_rect(
        Rect::new(
            rect.x + 1.0,
            rect.y + 1.0,
            (rect.w - 2.0).max(0.0),
            rect.h * 0.44,
        ),
        11.0,
        Color::from_rgba(30, 29, 25, if hovered { 174 } else { 122 }),
    );
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
    phone_panel(panel, hovered);

    let accent = Color::from_rgba(232, 168, 62, 238);
    let text = Color::from_rgba(246, 240, 226, 250);
    let muted = Color::from_rgba(184, 176, 158, 228);
    let green = Color::from_rgba(112, 212, 126, 238);

    draw_line(
        panel.x + 15.0,
        panel.y + 14.0,
        panel.x + 21.0,
        panel.y + 20.0,
        2.0,
        accent,
    );
    draw_line(
        panel.x + 21.0,
        panel.y + 20.0,
        panel.x + 29.0,
        panel.y + 12.0,
        2.0,
        accent,
    );

    draw_phone_text(
        "COMMS",
        panel.x + 38.0,
        panel.y + 20.0,
        PANEL_TITLE_SIZE,
        text,
    );

    draw_rectangle(
        panel.x + 14.0,
        panel.y + 27.0,
        (panel.w - 28.0).max(0.0),
        1.0,
        phone_alpha(accent, 0.32),
    );

    let screen_h = (panel.h * 0.34).clamp(42.0, 68.0);
    let screen = Rect::new(panel.x + 12.0, panel.y + 38.0, panel.w - 24.0, screen_h);

    phone_round_rect(screen, 8.0, Color::from_rgba(0, 0, 0, 164));

    phone_round_rect(
        Rect::new(
            screen.x + 1.0,
            screen.y + 1.0,
            (screen.w - 2.0).max(0.0),
            (screen.h - 2.0).max(0.0),
        ),
        7.0,
        Color::from_rgba(13, 14, 13, 230),
    );

    let blink = (time * 2.4).sin().abs();
    draw_circle(
        screen.x + 10.0,
        screen.y + 13.0,
        2.0,
        Color::from_rgba(112, 212, 126, 90 + (blink * 110.0) as u8),
    );

    draw_phone_text(
        "Papa : ligne de prod",
        screen.x + 20.0,
        screen.y + 17.0,
        11.5,
        muted,
    );

    if let Some(statut) = state.telephone.dernier_statut.as_deref() {
        let clipped = truncate_safe_text(statut, 22);
        draw_phone_text(
            clipped,
            screen.x + 8.0,
            screen.y + screen.h - 10.0,
            12.5,
            Color::from_rgba(226, 238, 214, 230),
        );
    } else {
        draw_phone_text(
            "Aucun appel actif",
            screen.x + 8.0,
            screen.y + screen.h - 10.0,
            12.5,
            Color::from_rgba(170, 164, 150, 210),
        );
    }

    let bouton = bouton_telephone_rect(panel);
    let button_active = state.telephone.ouvert;
    let button_hovered = crate::point_in_rect(mouse, bouton);

    phone_round_rect(
        Rect::new(bouton.x + 1.5, bouton.y + 2.0, bouton.w, bouton.h),
        8.0,
        Color::from_rgba(0, 0, 0, 62),
    );

    phone_round_rect(
        bouton,
        8.0,
        if button_active {
            Color::from_rgba(232, 168, 62, 226)
        } else if button_hovered {
            Color::from_rgba(52, 50, 42, 242)
        } else {
            Color::from_rgba(30, 30, 27, 238)
        },
    );

    phone_round_rect(
        Rect::new(
            bouton.x + 1.0,
            bouton.y + 1.0,
            (bouton.w - 2.0).max(0.0),
            (bouton.h - 2.0).max(0.0),
        ),
        7.0,
        if button_active {
            Color::from_rgba(214, 146, 42, 228)
        } else {
            Color::from_rgba(17, 18, 18, 232)
        },
    );

    let icon = Rect::new(
        bouton.x + bouton.w * 0.32,
        bouton.y + bouton.h * 0.23,
        bouton.w * 0.36,
        bouton.h * 0.40,
    );

    let icon_col = if button_active {
        Color::from_rgba(26, 18, 8, 244)
    } else {
        green
    };

    draw_line(
        icon.x + icon.w * 0.16,
        icon.y + icon.h * 0.20,
        icon.x + icon.w * 0.84,
        icon.y + icon.h * 0.80,
        4.0,
        icon_col,
    );
    draw_circle(
        icon.x + icon.w * 0.16,
        icon.y + icon.h * 0.20,
        4.2,
        icon_col,
    );
    draw_circle(
        icon.x + icon.w * 0.84,
        icon.y + icon.h * 0.80,
        4.2,
        icon_col,
    );

    let label = if state.telephone.ouvert {
        "FERMER"
    } else {
        "CONTACTS"
    };

    let label_size = 10.5;
    let dims = measure_text(label, None, label_size as u16, 1.0);

    draw_phone_text(
        label,
        bouton.x + bouton.w * 0.5 - dims.width * 0.5,
        bouton.y + bouton.h - 7.0,
        label_size,
        if button_active {
            Color::from_rgba(26, 18, 8, 246)
        } else {
            text
        },
    );

    if state.telephone.ouvert {
        let contacts_panel = contacts_panel_rect(panel);
        phone_panel(contacts_panel, true);

        draw_phone_text(
            "Contacts",
            contacts_panel.x + 10.0,
            contacts_panel.y + 20.0,
            16.0,
            text,
        );

        draw_rectangle(
            contacts_panel.x + 10.0,
            contacts_panel.y + 27.0,
            (contacts_panel.w - 20.0).max(0.0),
            1.0,
            phone_alpha(accent, 0.35),
        );

        for (idx, contact) in contacts_disponibles().iter().enumerate() {
            let row = contact_row_rect(contacts_panel, idx);
            let row_hovered = crate::point_in_rect(mouse, row);

            phone_round_rect(
                row,
                7.0,
                if row_hovered {
                    Color::from_rgba(58, 50, 36, 242)
                } else {
                    Color::from_rgba(18, 19, 18, 236)
                },
            );

            draw_rectangle(
                row.x + 1.0,
                row.y + 1.0,
                3.0,
                (row.h - 2.0).max(0.0),
                if row_hovered {
                    accent
                } else {
                    phone_alpha(accent, 0.34)
                },
            );

            draw_phone_text(
                contact_label(*contact),
                row.x + 10.0,
                row.y + 20.0,
                CONTACT_TEXT_SIZE,
                text,
            );
        }
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
