use super::effets::draw_background;
use super::*;

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
            Color::from_rgba(6, 11, 17, 92),
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
            Color::from_rgba(6, 11, 17, 74),
        );
    }

    let sw = screen_width();
    let sh = screen_height();
    let ui = palette.ui;
    let feedback = palette.feedback;
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let left_click = is_mouse_button_pressed(MouseButton::Left);
    let wheel_y = mouse_wheel().1;
    let has_save = !menu_state.saves.is_empty();

    draw_rectangle(0.0, 0.0, sw, sh, with_alpha(ui.panel_inset, 0.08));
    draw_rectangle(
        0.0,
        sh * 0.82,
        sw,
        sh * 0.18,
        with_alpha(ui.panel_inset, 0.16),
    );

    let title = "RXCHIXS";
    let subtitle = "Usine vivante • top-down industriel premium";
    let title_fs = (sh * 0.12).clamp(52.0, 92.0);
    let subtitle_fs = (sh * 0.03).clamp(18.0, 24.0);
    let title_dims = measure_text(title, None, title_fs as u16, 1.0);
    let subtitle_dims = measure_text(subtitle, None, subtitle_fs as u16, 1.0);
    let title_x = sw * 0.5 - title_dims.width * 0.5;
    let subtitle_x = sw * 0.5 - subtitle_dims.width * 0.5;
    draw_menu_text_line(title, title_x, sh * 0.22, title_fs, ui.text_primary);
    draw_menu_text_line(
        subtitle,
        subtitle_x,
        sh * 0.26,
        subtitle_fs,
        ui.text_secondary,
    );
    draw_rectangle(
        sw * 0.5 - 170.0,
        sh * 0.275,
        340.0,
        2.0,
        with_alpha(ui.accent_amber, 0.72),
    );

    let button_w = (sw * 0.28).clamp(300.0, 420.0);
    let button_h = (sh * 0.07).clamp(48.0, 62.0);
    let button_gap = (button_h * 0.22).clamp(10.0, 14.0);
    let button_count = 6.0;
    let stack_h = button_h * button_count + button_gap * (button_count - 1.0);
    let start_y = (sh * 0.54 - stack_h * 0.5).max(sh * 0.28);
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
            with_alpha(ui.panel_inset, 0.54),
        );
        let no_save_text = "Aucune sauvegarde";
        let no_save_dims = measure_text(no_save_text, None, 15, 1.0);
        draw_menu_text_line(
            no_save_text,
            continue_rect.x + continue_rect.w * 0.5 - no_save_dims.width * 0.5,
            continue_rect.y + continue_rect.h * 0.5 + 5.0,
            15.0,
            ui.text_secondary,
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
                ui.text_secondary,
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
                with_alpha(ui.panel_mid, 0.90),
            );
            draw_rectangle_lines(
                popup_rect.x + 0.5,
                popup_rect.y + 0.5,
                popup_rect.w - 1.0,
                popup_rect.h - 1.0,
                1.4,
                ui.border,
            );
            draw_rectangle(
                popup_rect.x + 1.0,
                popup_rect.y + 1.0,
                popup_rect.w - 2.0,
                30.0,
                with_alpha(ui.accent_steel, 0.16),
            );
            draw_menu_text_line(
                "Charger",
                popup_rect.x + 14.0,
                popup_rect.y + 24.0,
                24.0,
                ui.text_primary,
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
                with_alpha(ui.panel_inset, 0.84),
            );
            draw_rectangle_lines(
                list_rect.x + 0.5,
                list_rect.y + 0.5,
                list_rect.w - 1.0,
                list_rect.h - 1.0,
                1.2,
                with_alpha(ui.border, 0.92),
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
                    ui.text_secondary,
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
                        with_alpha(ui.accent_cyan, 0.36)
                    } else if hovered {
                        with_alpha(ui.accent_steel, 0.22)
                    } else {
                        with_alpha(ui.panel_mid, 0.54)
                    };
                    draw_rectangle(row_rect.x, row_rect.y, row_rect.w, row_rect.h, fill);
                    draw_rectangle_lines(
                        row_rect.x + 0.5,
                        row_rect.y + 0.5,
                        row_rect.w - 1.0,
                        row_rect.h - 1.0,
                        1.0,
                        if selected {
                            ui.border_hi
                        } else {
                            with_alpha(ui.border, 0.76)
                        },
                    );
                    draw_menu_text_line(
                        &slot.save_name,
                        row_rect.x + 8.0,
                        row_rect.y + 18.0,
                        15.0,
                        ui.text_primary,
                    );
                    draw_menu_text_line(
                        &slot.saved_at_label,
                        row_rect.x + 8.0,
                        row_rect.y + 33.0,
                        12.0,
                        ui.text_secondary,
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
                with_alpha(ui.panel_mid, 0.90),
            );
            draw_rectangle_lines(
                popup_rect.x + 0.5,
                popup_rect.y + 0.5,
                popup_rect.w - 1.0,
                popup_rect.h - 1.0,
                1.4,
                ui.border,
            );
            draw_menu_text_line(
                "Options",
                popup_rect.x + 14.0,
                popup_rect.y + 24.0,
                24.0,
                ui.text_primary,
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
        draw_menu_text_line(warn, sw * 0.18, sh - 18.0, 14.0, feedback.warning);
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
            feedback.warning,
        );
    }

    if menu_state.show_fps {
        draw_menu_text_line(
            &format!("FPS {}", get_fps()),
            sw - 112.0,
            24.0,
            16.0,
            ui.text_secondary,
        );
    }

    MainMenuAction::None
}
