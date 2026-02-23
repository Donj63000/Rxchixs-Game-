use super::*;

pub(crate) enum PlayAction {
    None,
    OpenEditor,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum EscapeIntent {
    ClosePause,
    CloseBuildMenu,
    ExitBuildMode,
    OpenPause,
}

fn resolve_escape_intent(
    pause_menu_open: bool,
    build_menu_open: bool,
    build_mode_enabled: bool,
) -> EscapeIntent {
    if pause_menu_open {
        EscapeIntent::ClosePause
    } else if build_menu_open {
        EscapeIntent::CloseBuildMenu
    } else if build_mode_enabled {
        EscapeIntent::ExitBuildMode
    } else {
        EscapeIntent::OpenPause
    }
}

fn normalize_wheel_units(raw_delta: f32) -> f32 {
    if raw_delta.abs() <= f32::EPSILON {
        return 0.0;
    }
    // Some Windows/device backends report +/-120 per notch.
    let normalized = if raw_delta.abs() > 10.0 {
        raw_delta / 120.0
    } else {
        raw_delta
    };
    normalized.clamp(-8.0, 8.0)
}

fn draw_overlay_panel(rect: Rect) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(7, 12, 18, 198),
    );
    draw_rectangle(
        rect.x + 1.0,
        rect.y + 1.0,
        (rect.w - 2.0).max(1.0),
        (rect.h * 0.42).max(1.0),
        Color::from_rgba(48, 84, 112, 38),
    );
    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        (rect.w - 1.0).max(1.0),
        (rect.h - 1.0).max(1.0),
        1.5,
        Color::from_rgba(124, 172, 206, 176),
    );
    draw_rectangle_lines(
        rect.x + 1.5,
        rect.y + 1.5,
        (rect.w - 3.0).max(1.0),
        (rect.h - 3.0).max(1.0),
        1.0,
        Color::from_rgba(20, 34, 48, 210),
    );
}

fn draw_overlay_line(text: &str, x: f32, y: f32, font_size: f32, color: Color) {
    let shadow = with_alpha(BLACK, 0.88);
    draw_text_shadowed(
        text,
        x,
        y,
        font_size,
        color,
        shadow,
        ui_shadow_offset(font_size),
    );
}

fn draw_overlay_multiline(
    text: &str,
    x: f32,
    y: f32,
    font_size: f32,
    line_height: f32,
    color: Color,
) {
    for (i, line) in text.lines().enumerate() {
        draw_overlay_line(line, x, y + i as f32 * line_height, font_size, color);
    }
}

fn read_chariot_fork_input() -> f32 {
    let monte = is_key_down(KeyCode::E);
    let descend = is_key_down(KeyCode::A);
    match (monte, descend) {
        (true, false) => 1.0,
        (false, true) => -1.0,
        _ => 0.0,
    }
}

fn draw_clark_status_panel(state: &GameState) {
    if !state.chariot.pilote_a_bord {
        return;
    }

    let rect = Rect::new((screen_width() - 280.0).max(12.0), 14.0, 268.0, 188.0);
    draw_overlay_panel(rect);
    draw_overlay_line(
        "Clark C500 - Fiche conduite",
        rect.x + 12.0,
        rect.y + 24.0,
        18.0,
        Color::from_rgba(240, 248, 255, 255),
    );

    let battery = state.chariot.batterie_pct.clamp(0.0, 100.0);
    let battery_ratio = state.chariot.batterie_ratio();
    let bar_rect = Rect::new(rect.x + 12.0, rect.y + 36.0, rect.w - 24.0, 16.0);
    draw_rectangle(
        bar_rect.x,
        bar_rect.y,
        bar_rect.w,
        bar_rect.h,
        Color::from_rgba(18, 28, 40, 214),
    );
    let battery_col = if battery_ratio > 0.5 {
        Color::from_rgba(96, 214, 132, 236)
    } else if battery_ratio > 0.2 {
        Color::from_rgba(232, 198, 92, 236)
    } else {
        Color::from_rgba(226, 98, 88, 240)
    };
    draw_rectangle(
        bar_rect.x + 1.0,
        bar_rect.y + 1.0,
        (bar_rect.w - 2.0) * battery_ratio,
        (bar_rect.h - 2.0).max(1.0),
        battery_col,
    );
    draw_rectangle_lines(
        bar_rect.x + 0.5,
        bar_rect.y + 0.5,
        bar_rect.w - 1.0,
        bar_rect.h - 1.0,
        1.0,
        Color::from_rgba(170, 200, 220, 190),
    );

    draw_overlay_line(
        &format!("Batterie: {:.0}%", battery),
        rect.x + 14.0,
        rect.y + 66.0,
        15.0,
        Color::from_rgba(222, 236, 248, 255),
    );
    draw_overlay_line(
        &format!(
            "Etat Clark: {:.0}%",
            state.chariot.etat_pct.clamp(0.0, 100.0)
        ),
        rect.x + 14.0,
        rect.y + 84.0,
        15.0,
        Color::from_rgba(198, 222, 236, 250),
    );
    draw_overlay_line(
        &format!("Statut: {}", state.chariot.statut_label()),
        rect.x + 14.0,
        rect.y + 102.0,
        15.0,
        if state.chariot.est_en_charge {
            Color::from_rgba(144, 220, 154, 250)
        } else {
            Color::from_rgba(204, 224, 238, 250)
        },
    );

    let cable_state = if state.chargeur_clark.cable_branche {
        "branche"
    } else if state.chargeur_clark.cable_tenu {
        "en main"
    } else {
        "range"
    };
    draw_overlay_line(
        &format!("Cable chargeur: {}", cable_state),
        rect.x + 14.0,
        rect.y + 120.0,
        15.0,
        Color::from_rgba(196, 220, 234, 245),
    );
    draw_overlay_line(
        &format!(
            "Conduite: {} | Vitesse {:.1}",
            if state.chariot.est_en_charge {
                "verrouillee"
            } else {
                "active"
            },
            state.chariot.velocity.length()
        ),
        rect.x + 14.0,
        rect.y + 138.0,
        15.0,
        Color::from_rgba(188, 212, 226, 242),
    );
    let rack_niveau = sim::FactorySim::rack_niveau_depuis_fourche(state.chariot.fourche_hauteur);
    draw_overlay_line(
        &format!(
            "Niveau rack cible: {}",
            sim::FactorySim::rack_niveau_label(rack_niveau)
        ),
        rect.x + 14.0,
        rect.y + 156.0,
        15.0,
        Color::from_rgba(186, 228, 204, 242),
    );
    draw_overlay_line(
        "A/E mat bas/haut | R descendre | F caisses",
        rect.x + 14.0,
        rect.y + 174.0,
        14.0,
        Color::from_rgba(244, 214, 146, 255),
    );
}

fn push_player_history(
    state: &mut GameState,
    sim_time_s: f64,
    cat: crate::historique::LogCategorie,
    msg: impl Into<String>,
) {
    if let Some(player_card) = state.pawns.iter_mut().find(|p| p.key == PawnKey::Player) {
        player_card.history.push(sim_time_s, cat, msg.into());
    }
}

fn tick_pause_status(state: &mut GameState, frame_dt: f32) {
    if state.pause_status_timer <= 0.0 {
        return;
    }
    state.pause_status_timer = (state.pause_status_timer - frame_dt).max(0.0);
    if state.pause_status_timer <= f32::EPSILON {
        state.pause_status_text = None;
    }
}

fn set_pause_status(state: &mut GameState, message: impl Into<String>) {
    state.pause_status_text = Some(message.into());
    state.pause_status_timer = 3.2;
}

fn snapshot_map_from_state(state: &GameState) -> MapAsset {
    MapAsset {
        version: MAP_FILE_VERSION,
        label: "Partie en cours".to_string(),
        world: state.world.clone(),
        props: state.props.clone(),
        player_spawn: tile_from_world_clamped(&state.world, state.player.pos),
        npc_spawn: tile_from_world_clamped(&state.world, state.npc.pos),
    }
}

fn rebuild_state_from_map(state: &mut GameState, mut map: MapAsset, lineage_seed: u64) {
    sanitize_map_asset(&mut map);
    let catalog = state.character_catalog.clone();
    let mut rebuilt = build_game_state_from_map(&map, &catalog, lineage_seed);
    rebuilt.pause_menu_open = false;
    rebuilt.pause_panel = PausePanel::Aucun;
    rebuilt.pause_status_text = None;
    rebuilt.pause_status_timer = 0.0;
    rebuilt.pause_save_name = String::new();
    rebuilt.pause_sauvegardes.clear();
    rebuilt.pause_sauvegardes_warning = None;
    rebuilt.pause_sauvegardes_offset = 0;
    rebuilt.pause_selected_sauvegarde = None;
    *state = rebuilt;
}

fn refresh_pause_sauvegardes(state: &mut GameState) {
    match lister_sauvegardes() {
        Ok(listing) => {
            state.pause_sauvegardes = listing.slots;
            state.pause_sauvegardes_warning = if listing.warnings.is_empty() {
                None
            } else {
                Some(format!(
                    "{} sauvegarde(s) ignoree(s): {}",
                    listing.warnings.len(),
                    listing.warnings[0]
                ))
            };
            if state.pause_sauvegardes.is_empty() {
                state.pause_selected_sauvegarde = None;
                state.pause_sauvegardes_offset = 0;
            } else {
                if state.pause_selected_sauvegarde.is_none() {
                    state.pause_selected_sauvegarde = Some(0);
                }
                let max_index = state.pause_sauvegardes.len() - 1;
                if let Some(selected) = state.pause_selected_sauvegarde {
                    state.pause_selected_sauvegarde = Some(selected.min(max_index));
                }
                state.pause_sauvegardes_offset = state.pause_sauvegardes_offset.min(max_index);
            }
        }
        Err(err) => {
            state.pause_sauvegardes.clear();
            state.pause_selected_sauvegarde = None;
            state.pause_sauvegardes_offset = 0;
            state.pause_sauvegardes_warning = Some(err);
        }
    }
}

fn ouvrir_pause_panel(state: &mut GameState, panel: PausePanel) {
    state.pause_panel = panel;
    if matches!(panel, PausePanel::Sauvegarder | PausePanel::Charger) {
        refresh_pause_sauvegardes(state);
    }
    if panel == PausePanel::Sauvegarder && state.pause_save_name.trim().is_empty() {
        state.pause_save_name = proposer_nom_sauvegarde(now_unix_seconds());
    }
}

fn update_pause_save_name_input(state: &mut GameState) {
    const MAX_NAME_LEN: usize = 64;
    while let Some(ch) = get_char_pressed() {
        let keep = ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '_' | '-' | '.');
        if keep && state.pause_save_name.len() < MAX_NAME_LEN {
            state.pause_save_name.push(ch);
        }
    }
    if is_key_pressed(KeyCode::Backspace) {
        let _ = state.pause_save_name.pop();
    }
}

fn draw_pause_sauvegardes_list(
    state: &mut GameState,
    rect: Rect,
    mouse: Vec2,
    left_click: bool,
    wheel_y: f32,
) {
    draw_rectangle(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        Color::from_rgba(10, 18, 28, 220),
    );
    draw_rectangle_lines(
        rect.x + 0.5,
        rect.y + 0.5,
        rect.w - 1.0,
        rect.h - 1.0,
        1.2,
        Color::from_rgba(96, 138, 168, 210),
    );

    let row_h = 34.0;
    let visible_rows = ((rect.h - 8.0) / row_h).floor().max(1.0) as usize;
    let max_offset = state.pause_sauvegardes.len().saturating_sub(visible_rows);
    state.pause_sauvegardes_offset = state.pause_sauvegardes_offset.min(max_offset);

    if point_in_rect(mouse, rect) && wheel_y.abs() > f32::EPSILON {
        if wheel_y > 0.0 {
            state.pause_sauvegardes_offset = state.pause_sauvegardes_offset.saturating_sub(1);
        } else if wheel_y < 0.0 {
            state.pause_sauvegardes_offset = (state.pause_sauvegardes_offset + 1).min(max_offset);
        }
    }

    if state.pause_sauvegardes.is_empty() {
        draw_overlay_line(
            "Aucune sauvegarde dans le dossier saves/",
            rect.x + 10.0,
            rect.y + 24.0,
            15.0,
            Color::from_rgba(178, 206, 224, 255),
        );
        return;
    }

    let start = state.pause_sauvegardes_offset;
    let end = (start + visible_rows).min(state.pause_sauvegardes.len());
    let mut y = rect.y + 4.0;
    for index in start..end {
        let slot = &state.pause_sauvegardes[index];
        let row_rect = Rect::new(rect.x + 4.0, y, rect.w - 8.0, row_h - 2.0);
        let hovered = point_in_rect(mouse, row_rect);
        let selected = state.pause_selected_sauvegarde == Some(index);
        let row_color = if selected {
            Color::from_rgba(80, 124, 156, 228)
        } else if hovered {
            Color::from_rgba(56, 88, 114, 220)
        } else {
            Color::from_rgba(30, 48, 64, 210)
        };
        draw_rectangle(row_rect.x, row_rect.y, row_rect.w, row_rect.h, row_color);
        draw_rectangle_lines(
            row_rect.x + 0.5,
            row_rect.y + 0.5,
            row_rect.w - 1.0,
            row_rect.h - 1.0,
            1.0,
            if selected {
                Color::from_rgba(220, 238, 250, 240)
            } else {
                Color::from_rgba(116, 156, 182, 192)
            },
        );
        draw_overlay_line(
            &slot.save_name,
            row_rect.x + 8.0,
            row_rect.y + 16.0,
            14.0,
            Color::from_rgba(236, 246, 255, 255),
        );
        draw_overlay_line(
            &slot.saved_at_label,
            row_rect.x + 8.0,
            row_rect.y + 30.0,
            12.0,
            Color::from_rgba(192, 220, 238, 248),
        );
        if left_click && hovered {
            state.pause_selected_sauvegarde = Some(index);
        }
        y += row_h;
    }
}

fn draw_pause_panel_details(
    state: &mut GameState,
    rect: Rect,
    mouse: Vec2,
    left_click: bool,
    wheel_y: f32,
) -> PlayAction {
    draw_overlay_panel(rect);
    let title = match state.pause_panel {
        PausePanel::Aucun => "Infos",
        PausePanel::Aide => "Aide",
        PausePanel::Options => "Options",
        PausePanel::Sauvegarder => "Sauvegarder",
        PausePanel::Charger => "Charger",
    };
    draw_overlay_line(
        title,
        rect.x + 10.0,
        rect.y + 22.0,
        20.0,
        Color::from_rgba(236, 246, 255, 255),
    );

    match state.pause_panel {
        PausePanel::Aucun => {
            draw_overlay_multiline(
                "Utilise les boutons a gauche.\n\nSauvegarder:\n- saisir un nom\n- creer une sauvegarde horodatee\n\nCharger:\n- selectionner une sauvegarde\n- charger la selection",
                rect.x + 10.0,
                rect.y + 50.0,
                15.0,
                19.0,
                Color::from_rgba(204, 228, 242, 255),
            );
        }
        PausePanel::Aide => {
            draw_overlay_multiline(
                "Objectif: piloter l'usine en continu.\nLe menu pause stoppe toute simulation.\nSauvegarder cree des fichiers nommes + horodates dans saves/.\nCharger restaure la partie depuis la sauvegarde selectionnee.\nEditeur ouvre l'outil map en conservant l'etat courant.",
                rect.x + 10.0,
                rect.y + 50.0,
                14.0,
                18.0,
                Color::from_rgba(204, 228, 242, 255),
            );
        }
        PausePanel::Options => {
            let sim_speed_label = match state.hud_ui.sim_speed {
                SimSpeed::Pause => "Pause",
                SimSpeed::X1 => "1x",
                SimSpeed::X2 => "2x",
                SimSpeed::X4 => "4x",
            };
            let options_text = format!(
                "Simulation: {}\nZoom camera: {:.2}\nPlein ecran: F11\nDebug: {}\nInspecteur perso: {}\nDossier sauvegardes: {}/",
                sim_speed_label,
                state.camera_zoom,
                if state.debug { "ON" } else { "OFF" },
                if state.show_character_inspector {
                    "ON"
                } else {
                    "OFF"
                },
                SAVE_DIR_PATH
            );
            draw_overlay_multiline(
                &options_text,
                rect.x + 10.0,
                rect.y + 50.0,
                15.0,
                19.0,
                Color::from_rgba(204, 228, 242, 255),
            );
        }
        PausePanel::Sauvegarder => {
            update_pause_save_name_input(state);
            let now_s = now_unix_seconds();
            let input_rect = Rect::new(rect.x + 10.0, rect.y + 48.0, rect.w - 20.0, 36.0);
            draw_rectangle(
                input_rect.x,
                input_rect.y,
                input_rect.w,
                input_rect.h,
                Color::from_rgba(16, 28, 40, 232),
            );
            draw_rectangle_lines(
                input_rect.x + 0.5,
                input_rect.y + 0.5,
                input_rect.w - 1.0,
                input_rect.h - 1.0,
                1.2,
                Color::from_rgba(116, 166, 198, 236),
            );

            let mut shown_name = state.pause_save_name.clone();
            if (get_time() as i32) % 2 == 0 {
                shown_name.push('_');
            }
            draw_overlay_line(
                &format!("Nom: {}", shown_name),
                input_rect.x + 8.0,
                input_rect.y + 23.0,
                16.0,
                Color::from_rgba(236, 246, 255, 255),
            );

            draw_overlay_line(
                &format!("Horodate (UTC): {}", format_horodate_utc(now_s)),
                rect.x + 12.0,
                rect.y + 104.0,
                14.0,
                Color::from_rgba(192, 220, 238, 248),
            );

            let controls_gap = 8.0;
            let controls_w = ((rect.w - 20.0 - controls_gap * 2.0) / 3.0).max(120.0);
            let save_now_rect = Rect::new(rect.x + 10.0, rect.y + 116.0, controls_w, 32.0);
            let refresh_rect = Rect::new(
                save_now_rect.x + save_now_rect.w + controls_gap,
                rect.y + 116.0,
                controls_w,
                32.0,
            );
            let default_name_rect = Rect::new(
                refresh_rect.x + refresh_rect.w + controls_gap,
                rect.y + 116.0,
                controls_w,
                32.0,
            );
            let save_requested =
                draw_ui_button_sized(save_now_rect, "Enregistrer", mouse, left_click, false, 15.0)
                    || is_key_pressed(KeyCode::Enter);
            if save_requested {
                let snapshot = snapshot_map_from_state(state);
                match enregistrer_sauvegarde(&snapshot, &state.pause_save_name) {
                    Ok(slot) => {
                        set_pause_status(
                            state,
                            format!(
                                "Sauvegarde creee: {} ({})",
                                slot.save_name, slot.saved_at_label
                            ),
                        );
                        refresh_pause_sauvegardes(state);
                        if let Some(index) = state
                            .pause_sauvegardes
                            .iter()
                            .position(|it| it.file_name == slot.file_name)
                        {
                            state.pause_selected_sauvegarde = Some(index);
                        }
                    }
                    Err(err) => set_pause_status(state, format!("Sauvegarde echouee: {err}")),
                }
            }
            if draw_ui_button_sized(
                refresh_rect,
                "Rafraichir liste",
                mouse,
                left_click,
                false,
                15.0,
            ) {
                refresh_pause_sauvegardes(state);
            }
            if draw_ui_button_sized(
                default_name_rect,
                "Nom auto",
                mouse,
                left_click,
                false,
                15.0,
            ) {
                state.pause_save_name = proposer_nom_sauvegarde(now_s);
            }

            let list_rect = Rect::new(rect.x + 10.0, rect.y + 160.0, rect.w - 20.0, rect.h - 170.0);
            draw_pause_sauvegardes_list(state, list_rect, mouse, left_click, wheel_y);
        }
        PausePanel::Charger => {
            let controls_gap = 10.0;
            let controls_w = ((rect.w - 20.0 - controls_gap) * 0.5).max(140.0);
            let load_rect = Rect::new(rect.x + 10.0, rect.y + 48.0, controls_w, 34.0);
            let refresh_rect = Rect::new(
                load_rect.x + load_rect.w + controls_gap,
                rect.y + 48.0,
                controls_w,
                34.0,
            );
            if draw_ui_button_sized(
                load_rect,
                "Charger la selection",
                mouse,
                left_click,
                false,
                15.0,
            ) {
                if let Some(selected) = state.pause_selected_sauvegarde {
                    if let Some(slot) = state.pause_sauvegardes.get(selected) {
                        match charger_sauvegarde(&slot.file_name) {
                            Ok(map) => {
                                let seed = state.lineage_seed;
                                rebuild_state_from_map(state, map, seed);
                                return PlayAction::None;
                            }
                            Err(err) => {
                                set_pause_status(state, format!("Chargement echoue: {err}"))
                            }
                        }
                    }
                } else {
                    set_pause_status(state, "Selectionne une sauvegarde d'abord.");
                }
            }
            if draw_ui_button_sized(
                refresh_rect,
                "Rafraichir liste",
                mouse,
                left_click,
                false,
                15.0,
            ) {
                refresh_pause_sauvegardes(state);
            }

            let list_rect = Rect::new(rect.x + 10.0, rect.y + 92.0, rect.w - 20.0, rect.h - 102.0);
            draw_pause_sauvegardes_list(state, list_rect, mouse, left_click, wheel_y);
        }
    }

    if let Some(warn) = state.pause_sauvegardes_warning.as_deref() {
        draw_overlay_line(
            warn,
            rect.x + 12.0,
            rect.y + rect.h - 12.0,
            13.0,
            Color::from_rgba(244, 214, 146, 255),
        );
    }

    PlayAction::None
}

fn draw_pause_menu_overlay(state: &mut GameState, mouse: Vec2, left_click: bool) -> PlayAction {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        Color::from_rgba(4, 8, 14, 178),
    );

    let panel_w = (screen_width() * 0.86).clamp(780.0, 1180.0);
    let panel_h = (screen_height() * 0.86).clamp(560.0, 760.0);
    let panel = Rect::new(
        (screen_width() - panel_w) * 0.5,
        (screen_height() - panel_h) * 0.5,
        panel_w,
        panel_h,
    );
    draw_overlay_panel(panel);

    draw_overlay_line(
        "PAUSE",
        panel.x + 14.0,
        panel.y + 32.0,
        28.0,
        Color::from_rgba(245, 252, 255, 255),
    );
    draw_overlay_line(
        "Menu partie",
        panel.x + 16.0,
        panel.y + 52.0,
        15.0,
        Color::from_rgba(184, 214, 232, 255),
    );

    let menu_col_rect = Rect::new(panel.x + 14.0, panel.y + 68.0, 240.0, panel.h - 82.0);
    let details_rect = Rect::new(
        menu_col_rect.x + menu_col_rect.w + 14.0,
        panel.y + 68.0,
        panel.w - menu_col_rect.w - 42.0,
        panel.h - 82.0,
    );
    draw_overlay_panel(menu_col_rect);

    let button_w = menu_col_rect.w - 16.0;
    let button_h = 36.0;
    let button_gap = 8.0;
    let mut button_y = menu_col_rect.y + 8.0;
    let button_x = menu_col_rect.x + 8.0;

    let new_rect = Rect::new(button_x, button_y, button_w, button_h);
    button_y += button_h + button_gap;
    let continue_rect = Rect::new(button_x, button_y, button_w, button_h);
    button_y += button_h + button_gap;
    let save_rect = Rect::new(button_x, button_y, button_w, button_h);
    button_y += button_h + button_gap;
    let load_rect = Rect::new(button_x, button_y, button_w, button_h);
    button_y += button_h + button_gap;
    let help_rect = Rect::new(button_x, button_y, button_w, button_h);
    button_y += button_h + button_gap;
    let options_rect = Rect::new(button_x, button_y, button_w, button_h);
    button_y += button_h + button_gap;
    let editor_rect = Rect::new(button_x, button_y, button_w, button_h);

    if draw_ui_button_sized(new_rect, "Nouvelle partie", mouse, left_click, false, 16.0) {
        let next_seed = advance_seed(state.lineage_seed);
        rebuild_state_from_map(state, MapAsset::new_default(), next_seed);
        return PlayAction::None;
    }

    if draw_ui_button_sized(continue_rect, "Continuer", mouse, left_click, false, 16.0) {
        state.pause_menu_open = false;
        state.pause_panel = PausePanel::Aucun;
        return PlayAction::None;
    }

    if draw_ui_button_sized(
        save_rect,
        "Sauvegarder",
        mouse,
        left_click,
        state.pause_panel == PausePanel::Sauvegarder,
        16.0,
    ) {
        ouvrir_pause_panel(state, PausePanel::Sauvegarder);
    }

    if draw_ui_button_sized(
        load_rect,
        "Charger",
        mouse,
        left_click,
        state.pause_panel == PausePanel::Charger,
        16.0,
    ) {
        ouvrir_pause_panel(state, PausePanel::Charger);
    }

    if draw_ui_button_sized(
        help_rect,
        "Aide",
        mouse,
        left_click,
        state.pause_panel == PausePanel::Aide,
        16.0,
    ) {
        ouvrir_pause_panel(state, PausePanel::Aide);
    }

    if draw_ui_button_sized(
        options_rect,
        "Options",
        mouse,
        left_click,
        state.pause_panel == PausePanel::Options,
        16.0,
    ) {
        ouvrir_pause_panel(state, PausePanel::Options);
    }

    if draw_ui_button_sized(editor_rect, "Editeur", mouse, left_click, false, 16.0) {
        state.pause_menu_open = false;
        state.pause_panel = PausePanel::Aucun;
        return PlayAction::OpenEditor;
    }

    let wheel_y = mouse_wheel().1;
    if details_rect.h > 40.0 {
        let panel_action =
            draw_pause_panel_details(state, details_rect, mouse, left_click, wheel_y);
        match panel_action {
            PlayAction::None => {}
            _ => return panel_action,
        }
    }

    if let Some(status) = state.pause_status_text.as_deref()
        && state.pause_status_timer > 0.0
    {
        draw_overlay_line(
            status,
            panel.x + 16.0,
            panel.y + panel.h - 14.0,
            14.0,
            Color::from_rgba(244, 216, 144, 255),
        );
    }

    PlayAction::None
}

fn run_play_pause_frame(state: &mut GameState, frame_dt: f32, accumulator: &mut f32) -> PlayAction {
    tick_pause_status(state, frame_dt);
    *accumulator = 0.0;

    ui_pawns::sync_dynamic_pawn_metrics(state);
    let time = get_time() as f32;
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let left_click = is_mouse_button_pressed(MouseButton::Left);
    let hud_layout = ui_hud::build_hud_layout(state);

    let sw = screen_width();
    let margin = PLAY_CAMERA_MARGIN;
    let map_view_rect = Rect::new(
        margin,
        margin,
        (sw - margin * 2.0).max(1.0),
        (hud_layout.bar_rect.y - margin * 2.0).max(1.0),
    );
    let (world_camera, clamped_center, clamped_zoom) = build_world_camera_for_viewport(
        &state.world,
        state.camera_center,
        state.camera_zoom,
        map_view_rect,
        PLAY_CAMERA_ZOOM_MIN,
        PLAY_CAMERA_ZOOM_MAX,
    );
    state.camera_center = clamped_center;
    state.camera_zoom = clamped_zoom;
    let visible_bounds = tile_bounds_from_camera(&state.world, &world_camera, map_view_rect, 2);

    clear_background(state.palette.bg_bottom);
    draw_background(&state.palette, time);
    set_camera(&world_camera);
    draw_floor_layer_region(&state.world, &state.palette, visible_bounds);
    draw_exterior_ground_ambiance_region(&state.world, &state.palette, time, visible_bounds);
    draw_sim_zone_overlay_region(&state.sim, visible_bounds);
    draw_wall_cast_shadows_region(&state.world, &state.palette, visible_bounds);
    draw_wall_layer_region(&state.world, &state.palette, visible_bounds);
    draw_exterior_trees_region(&state.world, &state.palette, time, visible_bounds);
    draw_prop_shadows_region(&state.props, &state.palette, time, visible_bounds);
    draw_props_region(&state.props, &state.palette, time, visible_bounds);
    draw_chargeur_clark(
        &state.chargeur_clark,
        &state.chariot,
        state.player.pos,
        &state.palette,
        time,
        state.debug,
    );
    draw_sim_blocks_overlay(&state.sim, state.debug, Some(visible_bounds));

    let worker_pos = tile_center(state.sim.primary_agent_tile());
    if !state.chariot.pilote_a_bord
        && let Some(player_character) = state.lineage.get(state.player_lineage_index)
    {
        draw_character(
            player_character,
            CharacterRenderParams {
                center: state.player.pos,
                scale: 1.0,
                facing: state.player.facing,
                facing_left: state.player.facing_left,
                is_walking: state.player.is_walking,
                walk_cycle: state.player.walk_cycle,
                gesture: CharacterGesture::None,
                time,
                debug: false,
            },
        );
    }
    draw_character(
        &state.npc_character,
        CharacterRenderParams {
            center: state.npc.pos,
            scale: 0.96,
            facing: state.npc.facing,
            facing_left: state.npc.facing_left,
            is_walking: state.npc.is_walking,
            walk_cycle: state.npc.walk_cycle,
            gesture: CharacterGesture::None,
            time,
            debug: false,
        },
    );
    draw_character(
        &state.sim_worker_character,
        CharacterRenderParams {
            center: worker_pos,
            scale: 0.94,
            facing: CharacterFacing::Front,
            facing_left: false,
            is_walking: false,
            walk_cycle: time * 2.0,
            gesture: CharacterGesture::None,
            time,
            debug: false,
        },
    );

    let driver = if state.chariot.pilote_a_bord {
        state.lineage.get(state.player_lineage_index)
    } else {
        None
    };
    draw_chariot_elevateur(&state.chariot, &state.palette, time, driver, state.debug);

    draw_lighting_region(&state.props, &state.palette, time, visible_bounds);
    begin_ui_pass();
    draw_clark_status_panel(state);
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
    ui_hud::draw_hud(
        state,
        &hud_layout,
        mouse,
        map_view_rect,
        &world_camera,
        time,
    );

    draw_pause_menu_overlay(state, mouse, left_click)
}

pub(crate) fn run_play_frame(
    state: &mut GameState,
    frame_dt: f32,
    accumulator: &mut f32,
) -> PlayAction {
    if is_key_pressed(KeyCode::Escape) {
        match resolve_escape_intent(
            state.pause_menu_open,
            state.hud_ui.build_menu_open,
            state.sim.build_mode_enabled(),
        ) {
            EscapeIntent::ClosePause => {
                state.pause_menu_open = false;
                state.pause_panel = PausePanel::Aucun;
                state.pause_status_text = None;
                state.pause_status_timer = 0.0;
            }
            EscapeIntent::CloseBuildMenu => {
                state.hud_ui.build_menu_open = false;
                state.hud_ui.build_menu_selected = None;
                return PlayAction::None;
            }
            EscapeIntent::ExitBuildMode => {
                if state.sim.zone_paint_mode_enabled() {
                    state.sim.set_zone_paint_mode(false);
                }
                if state.sim.floor_paint_mode_enabled() {
                    state.sim.set_floor_paint_mode(false);
                }
                state.sim.toggle_build_mode();
                state.hud_ui.build_menu_open = false;
                return PlayAction::None;
            }
            EscapeIntent::OpenPause => {
                state.pause_menu_open = true;
                state.pause_panel = PausePanel::Aucun;
                state.pause_status_text = None;
                state.pause_status_timer = 0.0;
                state.hud_ui.build_menu_open = false;
                state.hud_ui.info_window_open = false;
                state.pawn_ui.context_menu = None;
                refresh_pause_sauvegardes(state);
                if state.pause_save_name.trim().is_empty() {
                    state.pause_save_name = proposer_nom_sauvegarde(now_unix_seconds());
                }
            }
        }
    }

    if state.pause_menu_open {
        return run_play_pause_frame(state, frame_dt, accumulator);
    }
    if is_key_pressed(KeyCode::F10) {
        return PlayAction::OpenEditor;
    }
    tick_pause_status(state, frame_dt);

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
    if is_key_pressed(KeyCode::T) {
        let next = state.sim.block_orientation().next();
        state.sim.set_block_orientation(next);
    }
    if is_key_pressed(KeyCode::N) {
        state.sim.cycle_zone_brush();
    }
    if is_key_pressed(KeyCode::V) {
        state.sim.toggle_zone_paint_mode();
    }
    if is_key_pressed(KeyCode::K) {
        state.sim.cycle_floor_brush();
    }
    if is_key_pressed(KeyCode::F8) {
        let _ = state.sim.save_layout();
    }

    // --- UI first: it can consume clicks & wheel, and may move the camera (jump/follow) ---
    let time_now = get_time() as f32;
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let wheel_y = mouse_wheel().1;
    let left_click = is_mouse_button_pressed(MouseButton::Left);
    let right_click = is_mouse_button_pressed(MouseButton::Right);
    let middle_click = is_mouse_button_pressed(MouseButton::Middle);

    // Keep pawn bars synced with the sim.
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
    let sw = screen_width();
    let margin = PLAY_CAMERA_MARGIN;
    let input_view_rect = Rect::new(
        margin,
        margin,
        (sw - margin * 2.0).max(1.0),
        (hud_layout.bar_rect.y - margin * 2.0).max(1.0),
    );
    let mouse_in_map_input = point_in_rect(mouse, input_view_rect) && !hud_input.mouse_over_ui;

    // Wheel: rotate build blocks with mouse in construction mode, otherwise zoom camera.
    let wheel_units = normalize_wheel_units(wheel_y);
    let wheel_rotates_blocks = state.sim.build_mode_enabled()
        && mouse_in_map_input
        && !hud_input.mouse_over_ui
        && !hud_input.consumed_wheel
        && !state.sim.zone_paint_mode_enabled()
        && !state.sim.floor_paint_mode_enabled()
        && wheel_units.abs() > f32::EPSILON;
    if wheel_rotates_blocks {
        let mut orientation = state.sim.block_orientation();
        let steps = wheel_units.abs().round().max(1.0) as i32;
        for _ in 0..steps {
            orientation = if wheel_units > 0.0 {
                orientation.next()
            } else {
                // reverse rotation without adding an extra orientation API
                orientation.next().next().next()
            };
        }
        state.sim.set_block_orientation(orientation);
    } else if !hud_input.consumed_wheel
        && !hud_input.mouse_over_ui
        && wheel_units.abs() > f32::EPSILON
    {
        // Exponential scaling gives finer, more uniform zoom steps.
        let zoom_factor = (1.0 + PLAY_CAMERA_ZOOM_STEP).powf(wheel_units);
        state.camera_zoom =
            (state.camera_zoom * zoom_factor).clamp(PLAY_CAMERA_ZOOM_MIN, PLAY_CAMERA_ZOOM_MAX);
    }

    // Manual recenter cancels follow.
    if is_key_pressed(KeyCode::C) {
        state.camera_center = state.player.pos;
        state.pawn_ui.follow = None;
    }

    // Follow camera has priority (unless user pans this frame).
    if let Some(follow) = state.pawn_ui.follow
        && let Some(pos) = ui_pawns::pawn_world_pos(state, follow)
    {
        state.camera_center = pos;
    }

    let mut pan = read_camera_pan_input();
    if state.chariot.pilote_a_bord && is_key_down(KeyCode::A) {
        // A is reserved for mast down while driving.
        pan.x = pan.x.max(0.0);
    }
    if pan.length_squared() > 0.0 {
        // User intent: moving camera manually => stop following.
        state.pawn_ui.follow = None;
        let speed = PLAY_CAMERA_PAN_SPEED / state.camera_zoom.max(0.01);
        state.camera_center += pan * speed * frame_dt;
    }

    // --- Build world camera ---
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

    // Mouse -> world only if cursor is in the map AND not hovering UI.
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

    if !state.chariot.pilote_a_bord && is_key_pressed(KeyCode::E) {
        let charger_result = interagir_chargeur_clark(
            &mut state.chariot,
            &mut state.chargeur_clark,
            state.player.pos,
        );
        match charger_result {
            Ok(ActionChargeurClark::Pris) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Travail,
                    "Chargeur Clark: cable pris a la base.",
                );
            }
            Ok(ActionChargeurClark::Range) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Travail,
                    "Chargeur Clark: cable range a la base.",
                );
            }
            Ok(ActionChargeurClark::Branche) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Travail,
                    "Chargeur Clark: cable branche, charge active.",
                );
            }
            Ok(ActionChargeurClark::Debranche) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Travail,
                    "Chargeur Clark: cable debranche du chariot.",
                );
            }
            Err(ErreurChargeurClark::AucuneInteractionPossible)
            | Err(ErreurChargeurClark::TropLoinBase) => {
                match basculer_conduite_chariot(&mut state.chariot, &mut state.player, &state.world)
                {
                    Ok(ActionConduiteChariot::Monte) => {
                        push_player_history(
                            state,
                            now_sim_s,
                            crate::historique::LogCategorie::Deplacement,
                            "Montee dans le Clark jaune.",
                        );
                    }
                    Ok(ActionConduiteChariot::Descend) => {
                        push_player_history(
                            state,
                            now_sim_s,
                            crate::historique::LogCategorie::Deplacement,
                            "Descente du Clark jaune.",
                        );
                    }
                    Err(ErreurConduiteChariot::TropLoin) => {
                        push_player_history(
                            state,
                            now_sim_s,
                            crate::historique::LogCategorie::Etat,
                            "Impossible de monter: Clark trop loin.",
                        );
                    }
                    Err(ErreurConduiteChariot::AucuneSortieValide) => {
                        push_player_history(
                            state,
                            now_sim_s,
                            crate::historique::LogCategorie::Etat,
                            "Impossible de descendre: aucune tuile libre autour du Clark.",
                        );
                    }
                    Err(ErreurConduiteChariot::EnCharge) => {
                        push_player_history(
                            state,
                            now_sim_s,
                            crate::historique::LogCategorie::Etat,
                            "Clark indisponible: charge en cours (debranchez d'abord le cable).",
                        );
                    }
                }
            }
            Err(ErreurChargeurClark::ClarkOccupe) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Etat,
                    "Action chargeur impossible: descendez du Clark d'abord.",
                );
            }
        }
    } else if state.chariot.pilote_a_bord && is_key_pressed(KeyCode::R) {
        match basculer_conduite_chariot(&mut state.chariot, &mut state.player, &state.world) {
            Ok(ActionConduiteChariot::Descend) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Deplacement,
                    "Descente du Clark jaune.",
                );
            }
            Err(ErreurConduiteChariot::AucuneSortieValide) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Etat,
                    "Impossible de descendre: aucune tuile libre autour du Clark.",
                );
            }
            Err(ErreurConduiteChariot::TropLoin | ErreurConduiteChariot::EnCharge) => {}
            Ok(ActionConduiteChariot::Monte) => {}
        }
    }

    if is_key_pressed(KeyCode::F) {
        match actionner_fourches_chariot(
            &mut state.chariot,
            &state.world,
            &mut state.props,
            &mut state.sim,
        ) {
            Ok(ActionCaisseChariot::Chargee { kind, from }) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Travail,
                    format!(
                        "Fourches: caisse chargee ({}) depuis ({}, {}).",
                        prop_kind_label(kind),
                        from.0,
                        from.1
                    ),
                );
            }
            Ok(ActionCaisseChariot::Deposee { kind, to }) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Travail,
                    format!(
                        "Fourches: caisse dechargee ({}) vers ({}, {}).",
                        prop_kind_label(kind),
                        to.0,
                        to.1
                    ),
                );
            }
            Ok(ActionCaisseChariot::ChargeeDepuisRack { niveau, from }) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Travail,
                    format!(
                        "Fourches: palette chargee depuis rack ({}, {}) niveau {}.",
                        from.0,
                        from.1,
                        sim::FactorySim::rack_niveau_label(niveau)
                    ),
                );
            }
            Ok(ActionCaisseChariot::DeposeeDansRack { niveau, to }) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Travail,
                    format!(
                        "Fourches: palette deposee en rack ({}, {}) niveau {}.",
                        to.0,
                        to.1,
                        sim::FactorySim::rack_niveau_label(niveau)
                    ),
                );
            }
            Err(ErreurCaisseChariot::HorsConduite) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Etat,
                    "Impossible d'utiliser les fourches: montez d'abord dans le Clark (E).",
                );
            }
            Err(ErreurCaisseChariot::AucuneCaisseProche) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Etat,
                    "Aucune caisse transportable proche des fourches.",
                );
            }
            Err(ErreurCaisseChariot::TuileDepotBloquee) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Etat,
                    "Depot impossible: tuile devant le Clark occupee ou bloquee.",
                );
            }
            Err(ErreurCaisseChariot::RackNiveauOccupe) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Etat,
                    "Depot rack impossible: niveau deja occupe.",
                );
            }
            Err(ErreurCaisseChariot::RackNiveauVide) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Etat,
                    "Prise rack impossible: niveau vide.",
                );
            }
            Err(ErreurCaisseChariot::RackSansPalette) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Etat,
                    "Seule une palette logistique peut etre deposee dans un rack.",
                );
            }
            Err(ErreurCaisseChariot::RackIntrouvable) => {
                push_player_history(
                    state,
                    now_sim_s,
                    crate::historique::LogCategorie::Etat,
                    "Rack introuvable sur la tuile cible.",
                );
            }
        }
    }

    // Build mode clicks only if click was not on UI.
    if state.sim.build_mode_enabled() {
        if middle_click
            && mouse_in_map
            && !hud_input.mouse_over_ui
            && !state.sim.zone_paint_mode_enabled()
            && !state.sim.floor_paint_mode_enabled()
        {
            let next = state.sim.block_orientation().next();
            state.sim.set_block_orientation(next);
        }
        if left_click
            && mouse_in_map
            && !context_menu_consumed
            && let Some(tile) = mouse_tile
        {
            state.sim.apply_build_click(&mut state.world, tile, false);
        }
        if right_click
            && mouse_in_map
            && !context_menu_consumed
            && let Some(tile) = mouse_tile
        {
            state.sim.apply_build_click(&mut state.world, tile, true);
        }
    }

    let click_tile = if left_click
        && mouse_in_map
        && !state.sim.build_mode_enabled()
        && !context_menu_consumed
        && clicked_pawn.is_none()
        && !state.chariot.pilote_a_bord
    {
        mouse_tile
    } else {
        None
    };

    if let Some(tile) = click_tile {
        push_player_history(
            state,
            now_sim_s,
            crate::historique::LogCategorie::Deplacement,
            format!("Ordre de deplacement vers ({}, {}).", tile.0, tile.1),
        );
    }

    state.last_input = read_input_dir();
    if !state.chariot.pilote_a_bord {
        apply_control_inputs(
            &mut state.player,
            &state.world,
            state.last_input,
            click_tile,
        );
    } else {
        state.player.control_mode = ControlMode::Manual;
    }

    // --- Fixed-step simulation ---
    let sim_factor = state.hud_ui.sim_speed.factor();
    if sim_factor <= 0.0 {
        *accumulator = 0.0;
    } else {
        *accumulator =
            (*accumulator + frame_dt * sim_factor).min(FIXED_DT * MAX_SIM_STEPS_PER_FRAME as f32);
    }
    let mut sim_steps = 0usize;
    let fork_input = if state.chariot.pilote_a_bord {
        read_chariot_fork_input()
    } else {
        0.0
    };
    while *accumulator >= FIXED_DT && sim_steps < MAX_SIM_STEPS_PER_FRAME {
        state.sim.step(FIXED_DT);
        let drive_input = if state.chariot.pilote_a_bord {
            state.last_input
        } else {
            Vec2::ZERO
        };
        mettre_a_jour_chariot(
            &mut state.chariot,
            &state.world,
            drive_input,
            fork_input,
            FIXED_DT,
        );
        if state.chariot.pilote_a_bord {
            state.player.pos = state.chariot.pos;
            state.player.control_mode = ControlMode::Manual;
            state.player.facing = state.chariot.orientation.to_character_facing();
            state.player.facing_left = state.chariot.orientation.is_left();
            state.player.velocity = state.chariot.velocity;
            state.player.is_walking = state.chariot.velocity.length_squared() > 0.25;
            if state.player.is_walking {
                state.player.walk_cycle += FIXED_DT * WALK_CYCLE_SPEED * 0.82;
                if state.player.walk_cycle > std::f32::consts::TAU {
                    state.player.walk_cycle -= std::f32::consts::TAU;
                }
            } else {
                state.player.walk_cycle *= 0.82;
            }
            state.player.anim_frame = 0;
        }
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
        if !state.chariot.pilote_a_bord {
            update_player(&mut state.player, &state.world, state.last_input, FIXED_DT);
        } else {
            state.player.pos = state.chariot.pos;
        }
        update_npc_wanderer(&mut state.npc, &state.world, FIXED_DT);
        *accumulator -= FIXED_DT;
        sim_steps += 1;
    }
    if sim_steps == MAX_SIM_STEPS_PER_FRAME && *accumulator >= FIXED_DT {
        *accumulator = 0.0;
    }

    // Sync again after sim tick so UI reflects latest fatigue/stress.
    ui_pawns::sync_dynamic_pawn_metrics(state);

    // --- Render ---
    let time = time_now;
    let visible_bounds = tile_bounds_from_camera(&state.world, &world_camera, map_view_rect, 2);

    clear_background(state.palette.bg_bottom);
    draw_background(&state.palette, time);
    set_camera(&world_camera);
    draw_floor_layer_region(&state.world, &state.palette, visible_bounds);
    draw_exterior_ground_ambiance_region(&state.world, &state.palette, time, visible_bounds);
    draw_sim_zone_overlay_region(&state.sim, visible_bounds);
    draw_wall_cast_shadows_region(&state.world, &state.palette, visible_bounds);
    draw_wall_layer_region(&state.world, &state.palette, visible_bounds);
    draw_exterior_trees_region(&state.world, &state.palette, time, visible_bounds);
    draw_prop_shadows_region(&state.props, &state.palette, time, visible_bounds);
    draw_props_region(&state.props, &state.palette, time, visible_bounds);
    draw_chargeur_clark(
        &state.chargeur_clark,
        &state.chariot,
        state.player.pos,
        &state.palette,
        time,
        state.debug,
    );
    draw_sim_blocks_overlay(&state.sim, state.debug, Some(visible_bounds));
    draw_build_block_preview(&state.sim, &state.world, mouse_tile);

    // Draw world actors in stable Y order (pawns + parked forklift).
    #[derive(Copy, Clone)]
    enum DrawEntity {
        Player,
        Npc,
        SimWorker,
        Chariot,
    }

    let worker_pos = tile_center(state.sim.primary_agent_tile());
    let mut draw_order: Vec<(f32, DrawEntity)> = vec![
        (state.player.pos.y, DrawEntity::Player),
        (state.npc.pos.y, DrawEntity::Npc),
        (worker_pos.y, DrawEntity::SimWorker),
    ];
    if !state.chariot.pilote_a_bord {
        draw_order.push((state.chariot.pos.y, DrawEntity::Chariot));
    }
    draw_order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    for (_, entity) in draw_order {
        match entity {
            DrawEntity::Player => {
                let hint = state.social_state.anim_hint(PawnKey::Player);
                let gesture = gesture_from_social(hint.gesture);
                if state.chariot.pilote_a_bord {
                    let driver_character = state.lineage.get(state.player_lineage_index);
                    draw_chariot_elevateur(
                        &state.chariot,
                        &state.palette,
                        time,
                        driver_character,
                        state.debug,
                    );
                    if state.debug {
                        draw_rectangle_lines(
                            state.chariot.pos.x - state.chariot.half.x,
                            state.chariot.pos.y - state.chariot.half.y,
                            state.chariot.half.x * 2.0,
                            state.chariot.half.y * 2.0,
                            1.5,
                            Color::from_rgba(250, 214, 120, 245),
                        );
                    }
                } else if let Some(player_character) = state.lineage.get(state.player_lineage_index)
                {
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
                        player_character,
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
                            1.5,
                            GREEN,
                        );
                    }
                }
            }
            DrawEntity::Npc => {
                let hint = state.social_state.anim_hint(PawnKey::Npc);
                let gesture = gesture_from_social(hint.gesture);
                let mut facing = state.npc.facing;
                let mut facing_left = state.npc.facing_left;
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
            DrawEntity::SimWorker => {
                let hint = state.social_state.anim_hint(PawnKey::SimWorker);
                let gesture = gesture_from_social(hint.gesture);
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
            DrawEntity::Chariot => {
                draw_chariot_elevateur(&state.chariot, &state.palette, time, None, state.debug);
                if state.debug {
                    draw_rectangle_lines(
                        state.chariot.pos.x - state.chariot.half.x,
                        state.chariot.pos.y - state.chariot.half.y,
                        state.chariot.half.x * 2.0,
                        state.chariot.half.y * 2.0,
                        1.1,
                        Color::from_rgba(250, 214, 120, 190),
                    );
                }
            }
        }
    }

    // Selection ring in world space.
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

    // Sim agent overlay is kept debug-only to avoid clutter during construction.
    if state.debug {
        draw_sim_agent_overlay(&state.sim, true);
    }
    draw_lighting_region(&state.props, &state.palette, time, visible_bounds);
    begin_ui_pass();

    draw_clark_status_panel(state);

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

    // Old inspector panel can coexist, but we hide it when the new sheet is open to avoid overlap.
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
            .unwrap_or_else(|| "aucun-personnage".to_string());
        let target_tile = state
            .player
            .auto
            .target_tile
            .map(|(x, y)| format!("({}, {})", x, y))
            .unwrap_or_else(|| "aucune".to_string());
        let npc_target_tile = state
            .npc
            .auto
            .target_tile
            .map(|(x, y)| format!("({}, {})", x, y))
            .unwrap_or_else(|| "aucune".to_string());
        let npc_hint = state.social_state.anim_hint(PawnKey::Npc);
        let npc_social = npc_hint
            .kind
            .map(|kind| kind.ui_label())
            .unwrap_or("inactif");
        let chariot_tile = tile_from_world_clamped(&state.world, state.chariot.pos);
        let chariot_charge = state
            .chariot
            .caisse_chargee
            .map(prop_kind_label)
            .unwrap_or("aucune");
        let chariot_speed = state.chariot.velocity.length();
        let chariot_v_long = state.chariot.vitesse_longitudinale;
        let chariot_cap_deg = state.chariot.heading_rad.to_degrees();
        let chariot_braquage = state.chariot.angle_braquage * 100.0;
        let chariot_fourche = state.chariot.fourche_hauteur;
        let info = format!(
            "Mode jeu | Echap: pause | F10: editeur | F11: plein ecran\nF1: debogage | F2: inspecteur | F3: regenerer les visuels\nBarre basse: equipe, construction, caracteristiques, historique, mini-carte\nCamera: ZQSD/WASD deplacement | molette zoom | C recentrer\nBuild: F7 mode | B blocs | N zones | V peinture zones | K sols\nCarte: clic gauche = ordre de deplacement | fleches = controle manuel\nClark: E interaction/monter | R descendre | F caisses | A/E mat bas/haut\nJoueur monde=({:.1}, {:.1}) tuile=({}, {}) mode={} marche={} image={} orientation={} regard_gauche={} cycle={:.2}\nEntree joueur=({:.2}, {:.2}) camera=({:.1}, {:.1}) zoom={:.2} ips={}\nTrajet joueur: noeuds={} prochain_wp={} cible={}\nClark monde=({:.1}, {:.1}) tuile=({}, {}) conduite={} orientation={} charge={} vitesse={:.1} v_long={:.1} cap={:.1}deg braquage={:.0}% fourche={:.2}\nPNJ monde=({:.1}, {:.1}) marche={} attente={:.2}s social={} trajet={} cible={}\nMasque mur tuile={:04b}\nMutation={}/1000 | visuel={}\n{}",
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
            state.chariot.pos.x,
            state.chariot.pos.y,
            chariot_tile.0,
            chariot_tile.1,
            state.chariot.pilote_a_bord,
            state.chariot.orientation.label(),
            chariot_charge,
            chariot_speed,
            chariot_v_long,
            chariot_cap_deg,
            chariot_braquage,
            chariot_fourche,
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
        let font_size = 16.0;
        let line_height = 19.0;
        let line_count = info.lines().count().max(1) as f32;
        let panel = Rect::new(
            8.0,
            hud_y0 - 12.0,
            (screen_width() * 0.84).clamp(620.0, 1320.0),
            14.0 + line_count * line_height,
        );
        draw_overlay_panel(panel);
        draw_overlay_multiline(
            &info,
            panel.x + 10.0,
            panel.y + 20.0,
            font_size,
            line_height,
            Color::from_rgba(236, 246, 255, 255),
        );
    } else {
        let panel = Rect::new(
            8.0,
            hud_y0 - 10.0,
            (screen_width() * 0.84).clamp(620.0, 1320.0),
            72.0,
        );
        draw_overlay_panel(panel);
        draw_overlay_line(
            "Mode jeu | Echap: pause | F10: editeur | F11: plein ecran",
            panel.x + 10.0,
            panel.y + 24.0,
            21.0,
            Color::from_rgba(224, 240, 250, 255),
        );
        draw_overlay_line(
            "Commandes: ZQSD/WASD camera, molette zoom, clic carte deplacement, E interaction/monter, R descendre, F charger/decharger caisse, A/E mât",
            panel.x + 10.0,
            panel.y + 44.0,
            16.0,
            Color::from_rgba(204, 228, 242, 255),
        );
        let hud = state.sim.short_hud();
        draw_overlay_line(
            &hud,
            panel.x + 10.0,
            panel.y + 63.0,
            16.0,
            Color::from_rgba(196, 224, 236, 255),
        );
    }

    ui_hud::draw_hud(
        state,
        &hud_layout,
        mouse,
        map_view_rect,
        &world_camera,
        time,
    );

    PlayAction::None
}

fn gesture_from_social(gesture: crate::interactions::SocialGesture) -> CharacterGesture {
    match gesture {
        crate::interactions::SocialGesture::None => CharacterGesture::None,
        crate::interactions::SocialGesture::Talk => CharacterGesture::Talk,
        crate::interactions::SocialGesture::Wave => CharacterGesture::Wave,
        crate::interactions::SocialGesture::Explain => CharacterGesture::Explain,
        crate::interactions::SocialGesture::Laugh => CharacterGesture::Laugh,
        crate::interactions::SocialGesture::Apologize => CharacterGesture::Apologize,
        crate::interactions::SocialGesture::Threaten => CharacterGesture::Threaten,
        crate::interactions::SocialGesture::Argue => CharacterGesture::Argue,
    }
}

pub(crate) fn run_editor_frame(
    editor: &mut EditorState,
    map: &mut MapAsset,
    palette: &Palette,
    time: f32,
) -> EditorAction {
    if editor.status_timer > 0.0 {
        editor.status_timer = (editor.status_timer - get_frame_time()).max(0.0);
    }

    let frame_dt = get_frame_time().min(0.05);
    let mouse = vec2(mouse_position().0, mouse_position().1);
    let left_pressed = is_mouse_button_pressed(MouseButton::Left);
    let left_down = is_mouse_button_down(MouseButton::Left);
    let left_released = is_mouse_button_released(MouseButton::Left);
    let middle_down = is_mouse_button_down(MouseButton::Middle);
    let ctrl_down = is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl);
    let space_down = is_key_down(KeyCode::Space);

    let sw = screen_width();
    let sh = screen_height();
    let margin = 10.0;
    let top_bar_h = 64.0;
    let panel_top = margin + top_bar_h + margin;
    let panel_h = (sh - panel_top - margin).max(180.0);

    let mut left_panel_w = (sw * 0.18).clamp(220.0, 300.0);
    let mut right_panel_w = (sw * 0.24).clamp(280.0, 390.0);
    let min_map_w = 420.0;
    let mut map_w = sw - left_panel_w - right_panel_w - margin * 4.0;
    if map_w < min_map_w {
        let mut deficit = min_map_w - map_w;
        let shrink_right = deficit.min((right_panel_w - 240.0).max(0.0));
        right_panel_w -= shrink_right;
        deficit -= shrink_right;
        let shrink_left = deficit.min((left_panel_w - 190.0).max(0.0));
        left_panel_w -= shrink_left;
        map_w = sw - left_panel_w - right_panel_w - margin * 4.0;
    }
    map_w = map_w.max(180.0);

    let top_bar_rect = Rect::new(margin, margin, (sw - margin * 2.0).max(240.0), top_bar_h);
    let left_panel_rect = Rect::new(margin, panel_top, left_panel_w, panel_h);
    let map_slot_rect = Rect::new(
        left_panel_rect.x + left_panel_rect.w + margin,
        panel_top,
        map_w,
        panel_h,
    );
    let right_panel_rect = Rect::new(
        map_slot_rect.x + map_slot_rect.w + margin,
        panel_top,
        right_panel_w,
        panel_h,
    );
    let map_view_rect = map_slot_rect;

    if !editor.camera_initialized {
        editor.camera_center = tile_center(map.player_spawn);
        editor.camera_zoom = 1.0;
        editor.camera_initialized = true;
    }

    let mouse_over_map = point_in_rect(mouse, map_view_rect);
    let wheel = mouse_wheel().1;
    if mouse_over_map && wheel.abs() > f32::EPSILON {
        let zoom_factor = (1.0 + wheel * EDITOR_CAMERA_ZOOM_STEP).max(0.2);
        editor.camera_zoom = (editor.camera_zoom * zoom_factor)
            .clamp(EDITOR_CAMERA_ZOOM_MIN, EDITOR_CAMERA_ZOOM_MAX);
    }
    if is_key_pressed(KeyCode::PageUp) {
        editor.camera_zoom =
            (editor.camera_zoom * 1.15).clamp(EDITOR_CAMERA_ZOOM_MIN, EDITOR_CAMERA_ZOOM_MAX);
    }
    if is_key_pressed(KeyCode::PageDown) {
        editor.camera_zoom =
            (editor.camera_zoom / 1.15).clamp(EDITOR_CAMERA_ZOOM_MIN, EDITOR_CAMERA_ZOOM_MAX);
    }
    if is_key_pressed(KeyCode::Home) {
        editor.camera_center = tile_center(map.player_spawn);
    }

    let pan_input = read_editor_pan_input(space_down);
    if pan_input.length_squared() > 0.0 {
        let pan_speed = EDITOR_CAMERA_PAN_SPEED / editor.camera_zoom.max(0.01);
        editor.camera_center += pan_input * pan_speed * frame_dt;
    }
    if middle_down && mouse_over_map {
        let delta = mouse_delta_position();
        editor.camera_center -= delta / editor.camera_zoom.max(0.01);
    }

    let (world_camera, clamped_center, clamped_zoom) = build_world_camera_for_viewport(
        &map.world,
        editor.camera_center,
        editor.camera_zoom,
        map_view_rect,
        EDITOR_CAMERA_ZOOM_MIN,
        EDITOR_CAMERA_ZOOM_MAX,
    );
    editor.camera_center = clamped_center;
    editor.camera_zoom = clamped_zoom;

    let world_size_px = vec2(
        map.world.w as f32 * TILE_SIZE,
        map.world.h as f32 * TILE_SIZE,
    );
    let world_mouse = if mouse_over_map {
        let mut pos = world_camera.screen_to_world(mouse);
        pos.x = pos.x.clamp(0.0, (world_size_px.x - 0.001).max(0.0));
        pos.y = pos.y.clamp(0.0, (world_size_px.y - 0.001).max(0.0));
        Some(pos)
    } else {
        None
    };
    editor.hover_tile = world_mouse.map(|pos| tile_from_world_clamped(&map.world, pos));
    let visible_bounds = tile_bounds_from_camera(&map.world, &world_camera, map_view_rect, 2);

    if is_key_pressed(KeyCode::Key1) {
        editor.brush = EditorBrush::Floor;
    }
    if is_key_pressed(KeyCode::Key2) {
        editor.brush = EditorBrush::FloorMetal;
    }
    if is_key_pressed(KeyCode::Key3) {
        editor.brush = EditorBrush::FloorWood;
    }
    if is_key_pressed(KeyCode::Key4) {
        editor.brush = EditorBrush::FloorMoss;
    }
    if is_key_pressed(KeyCode::Key5) {
        editor.brush = EditorBrush::FloorSand;
    }
    if is_key_pressed(KeyCode::Key6) {
        editor.brush = EditorBrush::Wall;
    }
    if is_key_pressed(KeyCode::Key7) {
        editor.brush = EditorBrush::WallBrick;
    }
    if is_key_pressed(KeyCode::Key8) {
        editor.brush = EditorBrush::WallSteel;
    }
    if is_key_pressed(KeyCode::Key9) {
        editor.brush = EditorBrush::WallNeon;
    }
    if is_key_pressed(KeyCode::Key0) {
        editor.brush = EditorBrush::Crate;
    }
    if !space_down && is_key_pressed(KeyCode::Q) {
        editor.brush = EditorBrush::Pipe;
    }
    if !space_down && is_key_pressed(KeyCode::W) {
        editor.brush = EditorBrush::Lamp;
    }
    if is_key_pressed(KeyCode::E) {
        editor.brush = EditorBrush::Banner;
    }
    if is_key_pressed(KeyCode::T) {
        editor.brush = EditorBrush::Plant;
    }
    if is_key_pressed(KeyCode::Y) {
        editor.brush = EditorBrush::Bench;
    }
    if is_key_pressed(KeyCode::U) {
        editor.brush = EditorBrush::Crystal;
    }
    if is_key_pressed(KeyCode::X) {
        editor.brush = EditorBrush::EraseProp;
    }
    if is_key_pressed(KeyCode::B) {
        editor.tool = EditorTool::Brush;
    }
    if is_key_pressed(KeyCode::R) {
        editor.tool = EditorTool::Rect;
    }
    if is_key_pressed(KeyCode::G) {
        editor.show_grid = !editor.show_grid;
    }

    if ctrl_down && is_key_pressed(KeyCode::Z) {
        let _ = editor_undo(editor, map);
    }
    if ctrl_down && is_key_pressed(KeyCode::Y) {
        let _ = editor_redo(editor, map);
    }
    if ctrl_down && is_key_pressed(KeyCode::S) {
        editor_save_current_map(editor, map);
    }
    if ctrl_down && is_key_pressed(KeyCode::L) {
        editor_load_current_map(editor, map);
    }

    if is_key_pressed(KeyCode::P)
        && let Some(tile) = editor.hover_tile
        && !map.world.is_solid(tile.0, tile.1)
    {
        map.player_spawn = tile;
        editor_set_status(editor, format!("Point joueur -> ({}, {})", tile.0, tile.1));
    }
    if is_key_pressed(KeyCode::N)
        && let Some(tile) = editor.hover_tile
        && !map.world.is_solid(tile.0, tile.1)
    {
        map.npc_spawn = tile;
        editor_set_status(editor, format!("Point PNJ -> ({}, {})", tile.0, tile.1));
    }

    let over_ui = point_in_rect(mouse, top_bar_rect)
        || point_in_rect(mouse, left_panel_rect)
        || point_in_rect(mouse, right_panel_rect);
    let can_edit_map = mouse_over_map && !over_ui;

    if editor.tool == EditorTool::Brush {
        if left_pressed && can_edit_map && editor.hover_tile.is_some() {
            editor_push_undo(editor, map);
            editor.stroke_active = true;
            editor.stroke_changed = false;
        }
        if editor.stroke_active
            && left_down
            && let Some(tile) = editor.hover_tile
            && editor_apply_brush(map, editor.brush, tile)
        {
            editor.stroke_changed = true;
            editor.redo_stack.clear();
        }
        if left_released {
            if editor.stroke_active {
                if !editor.stroke_changed {
                    let _ = editor.undo_stack.pop();
                } else {
                    sanitize_map_asset(map);
                }
            }
            editor.stroke_active = false;
            editor.stroke_changed = false;
        }
    } else {
        if left_pressed && can_edit_map {
            editor.drag_start = editor.hover_tile;
        }
        if left_released {
            if let Some(start) = editor.drag_start {
                let end = editor.hover_tile.unwrap_or(start);
                let before = editor.undo_stack.len();
                editor_push_undo(editor, map);
                let changed = editor_apply_brush_rect(map, editor.brush, start, end);
                if changed {
                    editor.redo_stack.clear();
                    sanitize_map_asset(map);
                } else {
                    editor.undo_stack.truncate(before);
                }
            }
            editor.drag_start = None;
        }
    }

    clear_background(palette.bg_bottom);
    draw_background(palette, time);

    draw_rectangle(
        map_slot_rect.x,
        map_slot_rect.y,
        map_slot_rect.w,
        map_slot_rect.h,
        Color::from_rgba(8, 12, 18, 150),
    );

    set_camera(&world_camera);

    draw_floor_layer_region(&map.world, palette, visible_bounds);
    draw_exterior_ground_ambiance_region(&map.world, palette, time, visible_bounds);
    draw_wall_cast_shadows_region(&map.world, palette, visible_bounds);
    draw_wall_layer_region(&map.world, palette, visible_bounds);
    draw_exterior_trees_region(&map.world, palette, time, visible_bounds);
    draw_prop_shadows_region(&map.props, palette, time, visible_bounds);
    draw_props_region(&map.props, palette, time, visible_bounds);
    draw_lighting_region(&map.props, palette, time, visible_bounds);

    if editor.show_grid {
        draw_editor_grid_region(visible_bounds);
    }

    if let Some(tile) = editor.hover_tile {
        let rect = World::tile_rect(tile.0, tile.1);
        draw_rectangle_lines(
            rect.x + 1.0,
            rect.y + 1.0,
            rect.w - 2.0,
            rect.h - 2.0,
            2.2,
            Color::from_rgba(255, 212, 120, 240),
        );
    }

    if editor.tool == EditorTool::Rect
        && let Some(start) = editor.drag_start
    {
        let end = editor.hover_tile.unwrap_or(start);
        let min_x = start.0.min(end.0);
        let max_x = start.0.max(end.0);
        let min_y = start.1.min(end.1);
        let max_y = start.1.max(end.1);
        let rect = Rect::new(
            min_x as f32 * TILE_SIZE + 1.5,
            min_y as f32 * TILE_SIZE + 1.5,
            (max_x - min_x + 1) as f32 * TILE_SIZE - 3.0,
            (max_y - min_y + 1) as f32 * TILE_SIZE - 3.0,
        );
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            2.4,
            Color::from_rgba(90, 240, 210, 235),
        );
    }

    let player_pos = tile_center(map.player_spawn);
    draw_circle_lines(
        player_pos.x,
        player_pos.y,
        10.0,
        2.2,
        Color::from_rgba(95, 230, 120, 240),
    );
    draw_circle(
        player_pos.x,
        player_pos.y,
        7.0,
        Color::from_rgba(10, 26, 12, 176),
    );
    draw_overlay_line(
        "J",
        player_pos.x - 4.0,
        player_pos.y + 5.0,
        18.0,
        Color::from_rgba(95, 230, 120, 240),
    );

    let npc_pos = tile_center(map.npc_spawn);
    draw_circle_lines(
        npc_pos.x,
        npc_pos.y,
        10.0,
        2.2,
        Color::from_rgba(255, 160, 95, 240),
    );
    draw_circle(npc_pos.x, npc_pos.y, 7.0, Color::from_rgba(32, 18, 10, 176));
    draw_overlay_line(
        "N",
        npc_pos.x - 5.0,
        npc_pos.y + 5.0,
        18.0,
        Color::from_rgba(255, 160, 95, 240),
    );

    begin_ui_pass();

    draw_rectangle_lines(
        map_slot_rect.x + 0.5,
        map_slot_rect.y + 0.5,
        map_slot_rect.w - 1.0,
        map_slot_rect.h - 1.0,
        1.8,
        Color::from_rgba(90, 126, 149, 170),
    );
    draw_rectangle_lines(
        map_view_rect.x + 0.5,
        map_view_rect.y + 0.5,
        map_view_rect.w - 1.0,
        map_view_rect.h - 1.0,
        2.2,
        Color::from_rgba(170, 213, 237, 220),
    );

    draw_ambient_dust(palette, time);
    draw_vignette(palette);
    let top_bar_bg = Color::from_rgba(10, 18, 26, 228);
    let panel_bg = Color::from_rgba(9, 15, 22, 222);

    draw_rectangle(
        top_bar_rect.x,
        top_bar_rect.y,
        top_bar_rect.w,
        top_bar_rect.h,
        top_bar_bg,
    );
    draw_rectangle_lines(
        top_bar_rect.x + 0.5,
        top_bar_rect.y + 0.5,
        top_bar_rect.w - 1.0,
        top_bar_rect.h - 1.0,
        1.8,
        Color::from_rgba(92, 133, 162, 238),
    );
    draw_ui_text_tinted_on(
        top_bar_bg,
        Color::from_rgba(230, 242, 250, 255),
        "EDITEUR USINE",
        top_bar_rect.x + 16.0,
        top_bar_rect.y + 26.0,
        30.0,
    );
    draw_ui_text_tinted_on(
        top_bar_bg,
        Color::from_rgba(176, 206, 223, 255),
        &format!(
            "{} | {}x{} | objets {}",
            map.label,
            map.world.w,
            map.world.h,
            map.props.len()
        ),
        top_bar_rect.x + 16.0,
        top_bar_rect.y + 48.0,
        18.0,
    );

    let top_button_h = 34.0;
    let top_button_w = 102.0;
    let top_button_gap = 8.0;
    let top_button_y = top_bar_rect.y + (top_bar_rect.h - top_button_h) * 0.5;
    let mut right_cursor = top_bar_rect.x + top_bar_rect.w - 12.0;
    right_cursor -= top_button_w;
    let load_rect = Rect::new(right_cursor, top_button_y, top_button_w, top_button_h);
    right_cursor -= top_button_gap + top_button_w;
    let save_rect = Rect::new(right_cursor, top_button_y, top_button_w, top_button_h);
    right_cursor -= top_button_gap + top_button_w;
    let menu_rect = Rect::new(right_cursor, top_button_y, top_button_w, top_button_h);
    right_cursor -= top_button_gap + top_button_w;
    let play_rect = Rect::new(right_cursor, top_button_y, top_button_w, top_button_h);

    let mut action = EditorAction::None;
    if draw_ui_button(play_rect, "Jouer F5", mouse, left_pressed, false)
        || is_key_pressed(KeyCode::F5)
    {
        sanitize_map_asset(map);
        action = EditorAction::StartPlay;
    }
    if draw_ui_button(menu_rect, "Menu Esc", mouse, left_pressed, false)
        || is_key_pressed(KeyCode::Escape)
    {
        action = EditorAction::BackToMenu;
    }
    if draw_ui_button(save_rect, "Sauver", mouse, left_pressed, false) {
        editor_save_current_map(editor, map);
    }
    if draw_ui_button(load_rect, "Charger", mouse, left_pressed, false) {
        editor_load_current_map(editor, map);
    }

    draw_rectangle(
        left_panel_rect.x,
        left_panel_rect.y,
        left_panel_rect.w,
        left_panel_rect.h,
        panel_bg,
    );
    draw_rectangle_lines(
        left_panel_rect.x + 0.5,
        left_panel_rect.y + 0.5,
        left_panel_rect.w - 1.0,
        left_panel_rect.h - 1.0,
        1.8,
        Color::from_rgba(88, 124, 146, 232),
    );

    draw_rectangle(
        right_panel_rect.x,
        right_panel_rect.y,
        right_panel_rect.w,
        right_panel_rect.h,
        panel_bg,
    );
    draw_rectangle_lines(
        right_panel_rect.x + 0.5,
        right_panel_rect.y + 0.5,
        right_panel_rect.w - 1.0,
        right_panel_rect.h - 1.0,
        1.8,
        Color::from_rgba(88, 124, 146, 232),
    );

    draw_ui_text_tinted_on(
        panel_bg,
        Color::from_rgba(214, 232, 244, 255),
        "BOITE A OUTILS",
        left_panel_rect.x + 14.0,
        left_panel_rect.y + 24.0,
        24.0,
    );

    let left_pad = 14.0;
    let left_content_w = left_panel_rect.w - left_pad * 2.0;
    let undo_w = ((left_content_w - 8.0) * 0.5).max(80.0);
    let undo_rect = Rect::new(
        left_panel_rect.x + left_pad,
        left_panel_rect.y + 34.0,
        undo_w,
        30.0,
    );
    let redo_rect = Rect::new(
        undo_rect.x + undo_rect.w + 8.0,
        left_panel_rect.y + 34.0,
        undo_w,
        30.0,
    );
    if draw_ui_button_sized(undo_rect, "Annuler", mouse, left_pressed, false, 14.0) {
        let _ = editor_undo(editor, map);
    }
    if draw_ui_button_sized(redo_rect, "Retablir", mouse, left_pressed, false, 14.0) {
        let _ = editor_redo(editor, map);
    }

    let tool_label_y = left_panel_rect.y + 86.0;
    draw_ui_text_tinted_on(
        panel_bg,
        Color::from_rgba(190, 216, 231, 255),
        "Outil",
        left_panel_rect.x + 14.0,
        tool_label_y,
        20.0,
    );
    let tool_brush_rect = Rect::new(
        left_panel_rect.x + left_pad,
        tool_label_y + 8.0,
        left_content_w,
        30.0,
    );
    let tool_rect_rect = Rect::new(
        left_panel_rect.x + left_pad,
        tool_label_y + 44.0,
        left_content_w,
        30.0,
    );
    if draw_ui_button_sized(
        tool_brush_rect,
        "Pinceau (B)",
        mouse,
        left_pressed,
        editor.tool == EditorTool::Brush,
        14.0,
    ) {
        editor.tool = EditorTool::Brush;
    }
    if draw_ui_button_sized(
        tool_rect_rect,
        "Rectangle (R)",
        mouse,
        left_pressed,
        editor.tool == EditorTool::Rect,
        14.0,
    ) {
        editor.tool = EditorTool::Rect;
    }

    let brushes = [
        (EditorBrush::Floor, "1 Sol"),
        (EditorBrush::FloorMetal, "2 Sol metal"),
        (EditorBrush::FloorWood, "3 Sol bois"),
        (EditorBrush::FloorMoss, "4 Sol mousse"),
        (EditorBrush::FloorSand, "5 Sol sable"),
        (EditorBrush::Wall, "6 Mur"),
        (EditorBrush::WallBrick, "7 Mur brique"),
        (EditorBrush::WallSteel, "8 Mur acier"),
        (EditorBrush::WallNeon, "9 Mur neon"),
        (EditorBrush::Crate, "0 Caisse"),
        (EditorBrush::Pipe, "Q Tuyau"),
        (EditorBrush::Lamp, "W Lampe"),
        (EditorBrush::Banner, "E Banniere"),
        (EditorBrush::Plant, "T Pot de fleur"),
        (EditorBrush::Bench, "Y Banc"),
        (EditorBrush::Crystal, "U Cristal"),
        (EditorBrush::BoxCartonVide, "Box carton vide"),
        (EditorBrush::BoxSacBleu, "Box sac bleu"),
        (EditorBrush::BoxSacRouge, "Box sac rouge"),
        (EditorBrush::BoxSacVert, "Box sac vert"),
        (EditorBrush::PaletteLogistique, "Palette logistique"),
        (EditorBrush::BureauPcOn, "Bureau PC ON"),
        (EditorBrush::BureauPcOff, "Bureau PC OFF"),
        (EditorBrush::CaisseAilBrut, "Caisse d'ail brut"),
        (EditorBrush::CaisseAilCasse, "Caisse d'ail cassé"),
        (EditorBrush::Lavabo, "Lavabo"),
        (EditorBrush::EraseProp, "X Effacer"),
    ];

    let brush_title_y = tool_label_y + 90.0;
    draw_ui_text_tinted_on(
        panel_bg,
        Color::from_rgba(190, 216, 231, 255),
        "Pinceaux",
        left_panel_rect.x + 14.0,
        brush_title_y,
        20.0,
    );
    let brush_columns = if left_panel_rect.w > 250.0 { 2 } else { 1 };
    let brush_gap = 8.0;
    let brush_button_w = ((left_content_w - brush_gap * (brush_columns as f32 - 1.0))
        / brush_columns as f32)
        .max(78.0);
    let brush_button_h = 22.0;
    let brush_step_y = 25.0;
    for (i, (brush, label)) in brushes.iter().enumerate() {
        let row = i / brush_columns;
        let col = i % brush_columns;
        let rect = Rect::new(
            left_panel_rect.x + left_pad + col as f32 * (brush_button_w + brush_gap),
            brush_title_y + 8.0 + row as f32 * brush_step_y,
            brush_button_w,
            brush_button_h,
        );
        if draw_ui_button_sized(
            rect,
            label,
            mouse,
            left_pressed,
            editor.brush == *brush,
            12.0,
        ) {
            editor.brush = *brush;
        }
    }
    let brush_rows = brushes.len().div_ceil(brush_columns);
    let left_help_y = brush_title_y + 8.0 + brush_rows as f32 * brush_step_y + 10.0;

    let toggle_grid_rect = Rect::new(
        left_panel_rect.x + left_pad,
        left_help_y,
        left_content_w,
        28.0,
    );
    if draw_ui_button_sized(
        toggle_grid_rect,
        if editor.show_grid {
            "Grille: activee (G)"
        } else {
            "Grille: desactivee (G)"
        },
        mouse,
        left_pressed,
        editor.show_grid,
        13.0,
    ) {
        editor.show_grid = !editor.show_grid;
    }

    draw_ui_text_tinted_on(
        panel_bg,
        Color::from_rgba(214, 232, 244, 255),
        "INSPECTEUR",
        right_panel_rect.x + 14.0,
        right_panel_rect.y + 24.0,
        24.0,
    );
    let inspector_text = format!(
        "Actif: {} / {}\nCamera: x={:.0} y={:.0} zoom={:.2}\nViewport tuiles: x[{}..{}] y[{}..{}]\nSpawn joueur: ({}, {})\nSpawn PNJ: ({}, {})",
        editor_tool_label(editor.tool),
        editor_brush_label(editor.brush),
        editor.camera_center.x,
        editor.camera_center.y,
        editor.camera_zoom,
        visible_bounds.0,
        visible_bounds.1,
        visible_bounds.2,
        visible_bounds.3,
        map.player_spawn.0,
        map.player_spawn.1,
        map.npc_spawn.0,
        map.npc_spawn.1,
    );
    draw_ui_text_tinted_on(
        panel_bg,
        Color::from_rgba(186, 209, 224, 255),
        &inspector_text,
        right_panel_rect.x + 14.0,
        right_panel_rect.y + 50.0,
        17.0,
    );

    if let Some(tile) = editor.hover_tile {
        let tile_kind = map.world.get(tile.0, tile.1);
        let prop_at =
            prop_index_at_tile(&map.props, tile).map(|idx| prop_kind_label(map.props[idx].kind));
        draw_ui_text_tinted_on(
            panel_bg,
            Color::from_rgba(214, 232, 244, 255),
            &format!(
                "Case survolee: ({}, {})\nTuile: {}\nObjet: {}",
                tile.0,
                tile.1,
                tile_label(tile_kind),
                prop_at.unwrap_or("aucun")
            ),
            right_panel_rect.x + 14.0,
            right_panel_rect.y + 160.0,
            18.0,
        );
    } else {
        draw_ui_text_tinted_on(
            panel_bg,
            Color::from_rgba(166, 188, 204, 255),
            "Case survolee: aucune",
            right_panel_rect.x + 14.0,
            right_panel_rect.y + 160.0,
            18.0,
        );
    }

    let center_cam_rect = Rect::new(
        right_panel_rect.x + 14.0,
        right_panel_rect.y + 232.0,
        right_panel_rect.w - 28.0,
        30.0,
    );
    if draw_ui_button_sized(
        center_cam_rect,
        "Centrer camera (Origine)",
        mouse,
        left_pressed,
        false,
        13.0,
    ) {
        editor.camera_center = tile_center(map.player_spawn);
    }

    let set_player_rect = Rect::new(
        right_panel_rect.x + 14.0,
        right_panel_rect.y + 270.0,
        right_panel_rect.w - 28.0,
        30.0,
    );
    if draw_ui_button_sized(
        set_player_rect,
        "Definir spawn joueur (P)",
        mouse,
        left_pressed,
        false,
        13.0,
    ) && let Some(tile) = editor.hover_tile
        && !map.world.is_solid(tile.0, tile.1)
    {
        map.player_spawn = tile;
        editor_set_status(editor, format!("Point joueur -> ({}, {})", tile.0, tile.1));
    }

    let set_npc_rect = Rect::new(
        right_panel_rect.x + 14.0,
        right_panel_rect.y + 308.0,
        right_panel_rect.w - 28.0,
        30.0,
    );
    if draw_ui_button_sized(
        set_npc_rect,
        "Definir spawn PNJ (N)",
        mouse,
        left_pressed,
        false,
        13.0,
    ) && let Some(tile) = editor.hover_tile
        && !map.world.is_solid(tile.0, tile.1)
    {
        map.npc_spawn = tile;
        editor_set_status(editor, format!("Point PNJ -> ({}, {})", tile.0, tile.1));
    }

    draw_ui_text_tinted_on(
        panel_bg,
        Color::from_rgba(160, 186, 202, 255),
        "Raccourcis:\nCtrl+S/L sauver/charger\nCtrl+Z/Y annuler/retablir\nF11 plein ecran\nPan: fleches ou Espace+ZQSD\nZoom: molette / PagePrec/PageSuiv\nGlisser camera: molette maintenue",
        right_panel_rect.x + 14.0,
        right_panel_rect.y + right_panel_rect.h - 126.0,
        15.0,
    );

    let status_text = if editor.status_timer > 0.0 {
        editor.status_text.as_str()
    } else {
        "Pret"
    };
    draw_ui_text_tinted_on(
        top_bar_bg,
        Color::from_rgba(252, 232, 188, 255),
        status_text,
        top_bar_rect.x + 16.0,
        top_bar_rect.y + top_bar_rect.h - 6.0,
        16.0,
    );

    action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_priority_is_pause_then_build_menu_then_build_mode_then_pause_open() {
        assert_eq!(
            resolve_escape_intent(true, true, true),
            EscapeIntent::ClosePause
        );
        assert_eq!(
            resolve_escape_intent(false, true, true),
            EscapeIntent::CloseBuildMenu
        );
        assert_eq!(
            resolve_escape_intent(false, false, true),
            EscapeIntent::ExitBuildMode
        );
        assert_eq!(
            resolve_escape_intent(false, false, false),
            EscapeIntent::OpenPause
        );
    }

    #[test]
    fn escape_sequence_matches_expected_user_flow() {
        let first = resolve_escape_intent(false, true, true);
        assert_eq!(first, EscapeIntent::CloseBuildMenu);
        let second = resolve_escape_intent(false, false, true);
        assert_eq!(second, EscapeIntent::ExitBuildMode);
        let third = resolve_escape_intent(false, false, false);
        assert_eq!(third, EscapeIntent::OpenPause);
    }
}
