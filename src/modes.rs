use super::*;

pub(crate) enum PlayAction {
    None,
    BackToMenu,
    OpenEditor,
}

pub(crate) fn run_play_frame(
    state: &mut GameState,
    frame_dt: f32,
    accumulator: &mut f32,
) -> PlayAction {
    if is_key_pressed(KeyCode::Escape) {
        return PlayAction::BackToMenu;
    }
    if is_key_pressed(KeyCode::F10) {
        return PlayAction::OpenEditor;
    }

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
    if is_key_pressed(KeyCode::N) {
        state.sim.cycle_zone_brush();
    }
    if is_key_pressed(KeyCode::V) {
        state.sim.toggle_zone_paint_mode();
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

    // Keep pawn bars synced with the sim.
    ui_pawns::sync_dynamic_pawn_metrics(state);

    // Hit-test layout for this frame.
    let ui_layout_pre = ui_pawns::build_pawn_ui_layout(state);
    let ui_input = ui_pawns::process_pawn_ui_input(
        state,
        &ui_layout_pre,
        mouse,
        left_click,
        wheel_y,
        time_now,
    );

    // Wheel zoom only if UI didn't consume the wheel.
    if !ui_input.consumed_wheel && !ui_input.mouse_over_ui && wheel_y.abs() > f32::EPSILON {
        let zoom_factor = (1.0 + wheel_y * PLAY_CAMERA_ZOOM_STEP).max(0.2);
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

    let pan = read_camera_pan_input();
    if pan.length_squared() > 0.0 {
        // User intent: moving camera manually => stop following.
        state.pawn_ui.follow = None;
        let speed = PLAY_CAMERA_PAN_SPEED / state.camera_zoom.max(0.01);
        state.camera_center += pan * speed * frame_dt;
    }

    // --- Build world camera ---
    let (world_camera, map_view_rect) = if state.world.w <= 36 && state.world.h <= 24 {
        let (camera, rect) = fit_world_camera_to_screen(&state.world, PLAY_CAMERA_MARGIN);
        state.camera_center = vec2(
            state.world.w as f32 * TILE_SIZE * 0.5,
            state.world.h as f32 * TILE_SIZE * 0.5,
        );
        (camera, rect)
    } else {
        let (camera, rect, clamped_center) = build_pannable_world_camera(
            &state.world,
            state.camera_center,
            state.camera_zoom,
            PLAY_CAMERA_MARGIN,
        );
        state.camera_center = clamped_center;
        (camera, rect)
    };

    // Mouse -> world only if cursor is in the map AND not hovering UI.
    let mouse_in_map = point_in_rect(mouse, map_view_rect) && !ui_input.mouse_over_ui;
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

    if is_key_pressed(KeyCode::M)
        && let Some(tile) = mouse_tile
    {
        state.sim.select_move_source(tile);
    }

    // Build mode clicks only if click was not on UI.
    if state.sim.build_mode_enabled() {
        if left_click
            && mouse_in_map
            && let Some(tile) = mouse_tile
        {
            state.sim.apply_build_click(tile, false);
        }
        if right_click
            && mouse_in_map
            && let Some(tile) = mouse_tile
        {
            state.sim.apply_build_click(tile, true);
        }
    }

    let click_tile = if left_click && mouse_in_map && !state.sim.build_mode_enabled() {
        mouse_tile
    } else {
        None
    };

    state.last_input = read_input_dir();
    apply_control_inputs(
        &mut state.player,
        &state.world,
        state.last_input,
        click_tile,
    );

    // --- Fixed-step simulation ---
    *accumulator = (*accumulator + frame_dt).min(FIXED_DT * MAX_SIM_STEPS_PER_FRAME as f32);
    let mut sim_steps = 0usize;
    while *accumulator >= FIXED_DT && sim_steps < MAX_SIM_STEPS_PER_FRAME {
        state.sim.step(FIXED_DT);
        update_player(&mut state.player, &state.world, state.last_input, FIXED_DT);
        update_npc_wanderer(&mut state.npc, &state.world, &state.player, FIXED_DT);
        *accumulator -= FIXED_DT;
        sim_steps += 1;
    }
    if sim_steps == MAX_SIM_STEPS_PER_FRAME && *accumulator >= FIXED_DT {
        *accumulator = 0.0;
    }

    // Sync again after sim tick so UI reflects latest fatigue/stress.
    ui_pawns::sync_dynamic_pawn_metrics(state);

    // Rebuild layout for drawing (sheet open/scroll might have changed due to input).
    let ui_layout = ui_pawns::build_pawn_ui_layout(state);

    // --- Render ---
    let time = time_now;
    let visible_bounds = tile_bounds_from_camera(&state.world, &world_camera, map_view_rect, 2);

    clear_background(state.palette.bg_bottom);
    draw_background(&state.palette, time);
    set_camera(&world_camera);
    draw_floor_layer_region(&state.world, &state.palette, visible_bounds);
    draw_sim_zone_overlay_region(&state.sim, visible_bounds);
    draw_wall_cast_shadows_region(&state.world, &state.palette, visible_bounds);
    draw_wall_layer_region(&state.world, &state.palette, visible_bounds);
    draw_prop_shadows_region(&state.props, &state.palette, time, visible_bounds);
    draw_props_region(&state.props, &state.palette, time, visible_bounds);
    draw_sim_blocks_overlay(
        &state.sim,
        state.debug || state.sim.build_mode_enabled(),
        Some(visible_bounds),
    );

    // Draw all "pawns" in a stable Y-sorted order.
    let worker_pos = tile_center(state.sim.primary_agent_tile());
    let mut draw_order: [(f32, u8); 3] = [
        (state.player.pos.y, 0),
        (state.npc.pos.y, 1),
        (worker_pos.y, 2),
    ];
    draw_order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    for (_, kind) in draw_order {
        match kind {
            0 => {
                if let Some(player_character) = state.lineage.get(state.player_lineage_index) {
                    draw_player(&state.player, player_character, time, state.debug);
                }
            }
            1 => {
                draw_npc(&state.npc, &state.npc_character, time, state.debug);
            }
            2 => {
                // Sim worker (visual pawn) tied to the primary sim agent.
                draw_character(
                    &state.sim_worker_character,
                    CharacterRenderParams {
                        center: worker_pos,
                        scale: 0.94,
                        facing: CharacterFacing::Front,
                        facing_left: false,
                        is_walking: false,
                        walk_cycle: time * 2.0,
                        time,
                        debug: false,
                    },
                );
            }
            _ => {}
        }
    }

    // Selection ring in world space.
    ui_pawns::draw_selected_world_indicator(state);

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

    // Sim agent overlay is helpful in debug/build mode (otherwise we already have the worker pawn).
    if state.debug || state.sim.build_mode_enabled() {
        draw_sim_agent_overlay(&state.sim, state.debug || state.sim.build_mode_enabled());
    }
    draw_lighting_region(&state.props, &state.palette, time, visible_bounds);
    set_default_camera();

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

    // HUD text starts below the pawn bar.
    let hud_y0 = ui_layout.top_bar.y + ui_layout.top_bar.h + 18.0;

    if state.debug {
        let tx = (state.player.pos.x / TILE_SIZE).floor() as i32;
        let ty = (state.player.pos.y / TILE_SIZE).floor() as i32;

        let mask = wall_mask_4(&state.world, tx, ty);
        let player_visual = state
            .lineage
            .get(state.player_lineage_index)
            .map(compact_visual_summary)
            .unwrap_or_else(|| "no-character".to_string());
        let target_tile = state
            .player
            .auto
            .target_tile
            .map(|(x, y)| format!("({}, {})", x, y))
            .unwrap_or_else(|| "none".to_string());
        let npc_target_tile = state
            .npc
            .auto
            .target_tile
            .map(|(x, y)| format!("({}, {})", x, y))
            .unwrap_or_else(|| "none".to_string());
        let info = format!(
            "Mode Jeu | Esc=menu | F10=editeur | F11=plein ecran\nF1: debug on/off | F2: inspector | F3: regenerate\nbar perso: clic=select/jump | double-clic ou bouton F=follow | bouton Comp=fiche\ncamera: ZQSD/WASD pan | molette zoom | C recentrer\nmouse: click-to-move sur la map | fleches: override manuel\nplayer pos(px)=({:.1},{:.1}) tile=({},{})\nmode={} walking={} frame={} facing={} facing_left={} walk_cycle={:.2}\ninput=({:.2},{:.2}) camera=({:.1},{:.1}) zoom={:.2} fps={}\nplayer_path_nodes={} next_wp={} target_tile={}\nnpc pos=({:.1},{:.1}) walking={} bubble={:.2}s cooldown={:.2}s npc_path_nodes={} npc_target={}\nwall_mask@tile={:04b}\nmutation_permille={} visual={}\n{}",
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
            state.npc.pos.x,
            state.npc.pos.y,
            state.npc.is_walking,
            state.npc.bubble_timer,
            state.npc.bubble_cooldown,
            state.npc.auto.path_world.len(),
            npc_target_tile,
            mask,
            state.character_catalog.mutation_permille(),
            player_visual,
            state.sim.debug_hud(),
        );
        draw_text(&info, 12.0, hud_y0, 18.0, WHITE);
    } else {
        draw_text(
            "Mode Jeu | Esc=menu | F10=editeur | F11=plein ecran",
            12.0,
            hud_y0 + 4.0,
            22.0,
            Color::from_rgba(220, 235, 242, 255),
        );
        draw_text(
            "Bar perso: clic=select/jump | double-clic ou bouton F=follow | bouton Comp=fiche",
            12.0,
            hud_y0 + 28.0,
            18.0,
            Color::from_rgba(200, 224, 236, 255),
        );
        let hud = state.sim.short_hud();
        draw_text(
            &hud,
            12.0,
            hud_y0 + 52.0,
            20.0,
            Color::from_rgba(200, 224, 236, 255),
        );
        let build = state.sim.build_hint_line();
        draw_text(
            &build,
            12.0,
            hud_y0 + 74.0,
            18.0,
            Color::from_rgba(182, 210, 228, 255),
        );
        if !state.sim.status_line().is_empty() {
            draw_text(
                state.sim.status_line(),
                12.0,
                hud_y0 + 96.0,
                18.0,
                Color::from_rgba(252, 228, 182, 255),
            );
        }
    }

    // NEW: pawn bar + sheet drawn last (always on top).
    ui_pawns::draw_pawn_ui(state, &ui_layout, mouse, time);

    PlayAction::None
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
    draw_wall_cast_shadows_region(&map.world, palette, visible_bounds);
    draw_wall_layer_region(&map.world, palette, visible_bounds);
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
    draw_text(
        "P",
        player_pos.x - 4.5,
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
    draw_text(
        "N",
        npc_pos.x - 5.0,
        npc_pos.y + 5.0,
        18.0,
        Color::from_rgba(255, 160, 95, 240),
    );

    set_default_camera();

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
            "{} | {}x{} | props {}",
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
        "TOOLBOX",
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
        (EditorBrush::Plant, "T Plante"),
        (EditorBrush::Bench, "Y Banc"),
        (EditorBrush::Crystal, "U Cristal"),
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
            "Grille: ON (G)"
        } else {
            "Grille: OFF (G)"
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
        "INSPECTOR",
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
        "Centrer camera (Home)",
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
        "Raccourcis:\nCtrl+S/L sauver/charger\nCtrl+Z/Y undo/redo\nF11 plein ecran\nPan: fleches ou Space+ZQSD\nZoom: molette / PageUp/Down\nDrag camera: molette maintenue",
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
