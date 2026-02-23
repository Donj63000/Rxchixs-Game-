use super::*;

pub(crate) fn is_border_tile(world: &World, tile: (i32, i32)) -> bool {
    tile.0 <= 0 || tile.1 <= 0 || tile.0 >= world.w - 1 || tile.1 >= world.h - 1
}

pub(crate) fn prop_kind_label(kind: PropKind) -> &'static str {
    match kind {
        PropKind::Crate => "caisse",
        PropKind::Pipe => "tuyau",
        PropKind::Lamp => "lampe",
        PropKind::Banner => "banniere",
        PropKind::Plant => "pot de fleur",
        PropKind::Bench => "banc",
        PropKind::Crystal => "cristal",
        PropKind::BoxCartonVide => "box carton vide",
        PropKind::BoxSacBleu => "box sac bleu",
        PropKind::BoxSacRouge => "box sac rouge",
        PropKind::BoxSacVert => "box sac vert",
        PropKind::PaletteLogistique => "palette logistique",
        PropKind::BureauPcOn => "bureau PC ON",
        PropKind::BureauPcOff => "bureau PC OFF",
        PropKind::CaisseAilBrut => "caisse d'ail brut",
        PropKind::CaisseAilCasse => "caisse d'ail cassé",
        PropKind::Lavabo => "lavabo",
    }
}

pub(crate) fn editor_brush_label(brush: EditorBrush) -> &'static str {
    match brush {
        EditorBrush::Floor => "Sol",
        EditorBrush::FloorMetal => "Sol metal",
        EditorBrush::FloorWood => "Sol bois",
        EditorBrush::FloorMoss => "Sol mousse",
        EditorBrush::FloorSand => "Sol sable",
        EditorBrush::Wall => "Mur",
        EditorBrush::WallBrick => "Mur brique",
        EditorBrush::WallSteel => "Mur acier",
        EditorBrush::WallNeon => "Mur neon",
        EditorBrush::Crate => "Caisse",
        EditorBrush::Pipe => "Tuyau",
        EditorBrush::Lamp => "Lampe",
        EditorBrush::Banner => "Banniere",
        EditorBrush::Plant => "Pot de fleur",
        EditorBrush::Bench => "Banc",
        EditorBrush::Crystal => "Cristal",
        EditorBrush::BoxCartonVide => "Box carton vide",
        EditorBrush::BoxSacBleu => "Box sac bleu",
        EditorBrush::BoxSacRouge => "Box sac rouge",
        EditorBrush::BoxSacVert => "Box sac vert",
        EditorBrush::PaletteLogistique => "Palette logistique",
        EditorBrush::BureauPcOn => "Bureau PC ON",
        EditorBrush::BureauPcOff => "Bureau PC OFF",
        EditorBrush::CaisseAilBrut => "Caisse d'ail brut",
        EditorBrush::CaisseAilCasse => "Caisse d'ail cassé",
        EditorBrush::Lavabo => "Lavabo",
        EditorBrush::EraseProp => "Effacer objet",
    }
}

pub(crate) fn editor_tool_label(tool: EditorTool) -> &'static str {
    match tool {
        EditorTool::Brush => "Pinceau",
        EditorTool::Rect => "Rectangle",
    }
}

pub(crate) fn editor_capture_snapshot(map: &MapAsset) -> EditorSnapshot {
    EditorSnapshot {
        world: map.world.clone(),
        props: map.props.clone(),
        player_spawn: map.player_spawn,
        npc_spawn: map.npc_spawn,
    }
}

pub(crate) fn editor_apply_snapshot(map: &mut MapAsset, snapshot: EditorSnapshot) {
    map.world = snapshot.world;
    map.props = snapshot.props;
    map.player_spawn = snapshot.player_spawn;
    map.npc_spawn = snapshot.npc_spawn;
}

pub(crate) fn editor_push_undo(editor: &mut EditorState, map: &MapAsset) {
    editor.undo_stack.push(editor_capture_snapshot(map));
    if editor.undo_stack.len() > EDITOR_UNDO_LIMIT {
        let overflow = editor.undo_stack.len() - EDITOR_UNDO_LIMIT;
        editor.undo_stack.drain(0..overflow);
    }
}

pub(crate) fn editor_set_status(editor: &mut EditorState, message: impl Into<String>) {
    editor.status_text = message.into();
    editor.status_timer = 3.4;
}

pub(crate) fn editor_reset_stroke_state(editor: &mut EditorState) {
    editor.stroke_active = false;
    editor.stroke_changed = false;
    editor.drag_start = None;
}

pub(crate) fn editor_save_current_map(editor: &mut EditorState, map: &mut MapAsset) {
    sanitize_map_asset(map);
    match save_map_asset(MAP_FILE_PATH, map) {
        Ok(()) => editor_set_status(editor, format!("Carte sauvegardee: {}", MAP_FILE_PATH)),
        Err(err) => editor_set_status(editor, err),
    }
}

pub(crate) fn editor_load_current_map(editor: &mut EditorState, map: &mut MapAsset) {
    match load_map_asset(MAP_FILE_PATH) {
        Ok(loaded) => {
            *map = loaded;
            editor.undo_stack.clear();
            editor.redo_stack.clear();
            editor_reset_stroke_state(editor);
            editor.camera_initialized = false;
            editor_set_status(editor, format!("Carte chargee: {}", MAP_FILE_PATH));
        }
        Err(err) => editor_set_status(editor, err),
    }
}

pub(crate) fn editor_undo(editor: &mut EditorState, map: &mut MapAsset) -> bool {
    let Some(snapshot) = editor.undo_stack.pop() else {
        editor_set_status(editor, "Annulation vide");
        return false;
    };
    editor.redo_stack.push(editor_capture_snapshot(map));
    editor_apply_snapshot(map, snapshot);
    editor_set_status(editor, "Annulation appliquee");
    true
}

pub(crate) fn editor_redo(editor: &mut EditorState, map: &mut MapAsset) -> bool {
    let Some(snapshot) = editor.redo_stack.pop() else {
        editor_set_status(editor, "Retablissement vide");
        return false;
    };
    editor.undo_stack.push(editor_capture_snapshot(map));
    editor_apply_snapshot(map, snapshot);
    editor_set_status(editor, "Retablissement applique");
    true
}

pub(crate) fn enforce_world_border(world: &mut World) {
    for x in 0..world.w {
        world.set(x, 0, Tile::Wall);
        world.set(x, world.h - 1, Tile::Wall);
    }
    for y in 0..world.h {
        world.set(0, y, Tile::Wall);
        world.set(world.w - 1, y, Tile::Wall);
    }
}

pub(crate) fn prop_index_at_tile(props: &[Prop], tile: (i32, i32)) -> Option<usize> {
    props
        .iter()
        .position(|prop| prop.tile_x == tile.0 && prop.tile_y == tile.1)
}

pub(crate) fn prop_phase_for_tile(tile: (i32, i32)) -> f32 {
    let h = tile_hash(tile.0, tile.1) & 0xFF;
    (h as f32 / 255.0) * std::f32::consts::TAU
}

pub(crate) fn remove_prop_at_tile(map: &mut MapAsset, tile: (i32, i32)) -> bool {
    let Some(idx) = prop_index_at_tile(&map.props, tile) else {
        return false;
    };
    map.props.swap_remove(idx);
    true
}

pub(crate) fn set_prop_at_tile(map: &mut MapAsset, tile: (i32, i32), kind: PropKind) -> bool {
    if !map.world.in_bounds(tile.0, tile.1) || map.world.is_solid(tile.0, tile.1) {
        return false;
    }
    if let Some(idx) = prop_index_at_tile(&map.props, tile) {
        if map.props[idx].kind == kind {
            return false;
        }
        map.props[idx].kind = kind;
        map.props[idx].phase = prop_phase_for_tile(tile);
        return true;
    }

    map.props.push(Prop {
        tile_x: tile.0,
        tile_y: tile.1,
        kind,
        phase: prop_phase_for_tile(tile),
    });
    true
}

pub(crate) fn set_map_tile(map: &mut MapAsset, tile: (i32, i32), tile_kind: Tile) -> bool {
    if !map.world.in_bounds(tile.0, tile.1) {
        return false;
    }
    if is_border_tile(&map.world, tile) && !tile_is_wall(tile_kind) {
        return false;
    }
    if map.world.get(tile.0, tile.1) == tile_kind {
        return false;
    }

    map.world.set(tile.0, tile.1, tile_kind);
    if tile_is_wall(tile_kind) {
        let _ = remove_prop_at_tile(map, tile);
    }
    true
}

pub(crate) fn editor_apply_brush(map: &mut MapAsset, brush: EditorBrush, tile: (i32, i32)) -> bool {
    match brush {
        EditorBrush::Floor => set_map_tile(map, tile, Tile::Floor),
        EditorBrush::FloorMetal => set_map_tile(map, tile, Tile::FloorMetal),
        EditorBrush::FloorWood => set_map_tile(map, tile, Tile::FloorWood),
        EditorBrush::FloorMoss => set_map_tile(map, tile, Tile::FloorMoss),
        EditorBrush::FloorSand => set_map_tile(map, tile, Tile::FloorSand),
        EditorBrush::Wall => set_map_tile(map, tile, Tile::Wall),
        EditorBrush::WallBrick => set_map_tile(map, tile, Tile::WallBrick),
        EditorBrush::WallSteel => set_map_tile(map, tile, Tile::WallSteel),
        EditorBrush::WallNeon => set_map_tile(map, tile, Tile::WallNeon),
        EditorBrush::Crate => set_prop_at_tile(map, tile, PropKind::Crate),
        EditorBrush::Pipe => set_prop_at_tile(map, tile, PropKind::Pipe),
        EditorBrush::Lamp => set_prop_at_tile(map, tile, PropKind::Lamp),
        EditorBrush::Banner => set_prop_at_tile(map, tile, PropKind::Banner),
        EditorBrush::Plant => set_prop_at_tile(map, tile, PropKind::Plant),
        EditorBrush::Bench => set_prop_at_tile(map, tile, PropKind::Bench),
        EditorBrush::Crystal => set_prop_at_tile(map, tile, PropKind::Crystal),
        EditorBrush::BoxCartonVide => set_prop_at_tile(map, tile, PropKind::BoxCartonVide),
        EditorBrush::BoxSacBleu => set_prop_at_tile(map, tile, PropKind::BoxSacBleu),
        EditorBrush::BoxSacRouge => set_prop_at_tile(map, tile, PropKind::BoxSacRouge),
        EditorBrush::BoxSacVert => set_prop_at_tile(map, tile, PropKind::BoxSacVert),
        EditorBrush::PaletteLogistique => set_prop_at_tile(map, tile, PropKind::PaletteLogistique),
        EditorBrush::BureauPcOn => set_prop_at_tile(map, tile, PropKind::BureauPcOn),
        EditorBrush::BureauPcOff => set_prop_at_tile(map, tile, PropKind::BureauPcOff),
        EditorBrush::CaisseAilBrut => set_prop_at_tile(map, tile, PropKind::CaisseAilBrut),
        EditorBrush::CaisseAilCasse => set_prop_at_tile(map, tile, PropKind::CaisseAilCasse),
        EditorBrush::Lavabo => set_prop_at_tile(map, tile, PropKind::Lavabo),
        EditorBrush::EraseProp => remove_prop_at_tile(map, tile),
    }
}

pub(crate) fn editor_apply_brush_rect(
    map: &mut MapAsset,
    brush: EditorBrush,
    start: (i32, i32),
    end: (i32, i32),
) -> bool {
    let min_x = start.0.min(end.0);
    let max_x = start.0.max(end.0);
    let min_y = start.1.min(end.1);
    let max_y = start.1.max(end.1);
    let mut changed = false;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            changed |= editor_apply_brush(map, brush, (x, y));
        }
    }
    changed
}

pub(crate) fn sanitize_map_asset(map: &mut MapAsset) {
    let needs_material_upgrade = map.version < MAP_FILE_VERSION;
    let needs_layout_upgrade =
        map.version < MAP_FILE_VERSION && (map.world.w < MAP_W || map.world.h < MAP_H);
    if needs_layout_upgrade {
        *map = MapAsset::new_default();
        return;
    }
    if map.world.w < 4 || map.world.h < 4 {
        map.world = generate_starter_factory_world(MAP_W, MAP_H);
    }

    let required = (map.world.w * map.world.h) as usize;
    if map.world.tiles.len() != required {
        map.world.tiles = vec![Tile::Floor; required];
    }

    if needs_material_upgrade {
        apply_material_variation(&mut map.world);
    }
    map.version = MAP_FILE_VERSION;

    enforce_world_border(&mut map.world);

    let mut occupied = HashSet::new();
    map.props.retain(|prop| {
        if !map.world.in_bounds(prop.tile_x, prop.tile_y) {
            return false;
        }
        if map.world.is_solid(prop.tile_x, prop.tile_y) {
            return false;
        }
        occupied.insert((prop.tile_x, prop.tile_y))
    });

    map.player_spawn = nearest_walkable_tile(&map.world, map.player_spawn).unwrap_or((2, 2));
    map.npc_spawn = nearest_walkable_tile(&map.world, map.npc_spawn)
        .unwrap_or((map.world.w - 3, map.world.h / 2));
}

pub(crate) fn serialize_map_asset(map: &MapAsset) -> Result<String, String> {
    let pretty = PrettyConfig::new()
        .depth_limit(4)
        .enumerate_arrays(true)
        .separate_tuple_members(true);
    ron_to_string_pretty(map, pretty).map_err(|err| format!("echec serialisation carte: {err}"))
}

pub(crate) fn deserialize_map_asset(raw: &str) -> Result<MapAsset, String> {
    let mut map: MapAsset =
        ron_from_str(raw).map_err(|err| format!("echec lecture carte: {err}"))?;
    sanitize_map_asset(&mut map);
    Ok(map)
}

pub(crate) fn save_map_asset(path: &str, map: &MapAsset) -> Result<(), String> {
    let payload = serialize_map_asset(map)?;
    if let Some(parent) = Path::new(path).parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|err| format!("echec creation dossier carte: {err}"))?;
    }
    fs::write(path, payload).map_err(|err| format!("echec ecriture carte: {err}"))
}

pub(crate) fn load_map_asset(path: &str) -> Result<MapAsset, String> {
    let raw =
        fs::read_to_string(path).map_err(|err| format!("echec lecture fichier carte: {err}"))?;
    deserialize_map_asset(&raw)
}

pub(crate) fn build_game_state_from_map(
    map: &MapAsset,
    character_catalog: &CharacterCatalog,
    lineage_seed: u64,
) -> GameState {
    let mut map_copy = map.clone();
    sanitize_map_asset(&mut map_copy);
    let player = Player::new(tile_center(map_copy.player_spawn));
    let chariot = spawn_chariot_pour_map(&map_copy.world, map_copy.player_spawn);
    let chargeur_clark = spawn_chargeur_pour_chariot(&map_copy.world, &chariot);
    let npc = NpcWanderer::new(tile_center(map_copy.npc_spawn), 0x9922_11AA_77CC_44DD);
    let palette = Palette::new();
    let lineage = build_lineage_preview(character_catalog, lineage_seed);
    let npc_character =
        character_catalog.spawn_founder("Promeneur", lineage_seed ^ 0x55AA_7788_1133_2244);
    let sim_worker_character =
        character_catalog.spawn_founder("Employe-01", lineage_seed ^ 0xCC11_22DD_33EE_44FF);

    let sim = sim::FactorySim::load_or_default(SIM_CONFIG_PATH, map_copy.world.w, map_copy.world.h);

    let mut pawns = vec![
        PawnCard {
            key: PawnKey::Player,
            name: "Patron".to_string(),
            role: "Gestion".to_string(),
            metrics: PawnMetrics::seeded(lineage_seed ^ 0x1111_2222_3333_4444),
            history: crate::historique::HistoriqueLog::new(600),
        },
        PawnCard {
            key: PawnKey::Npc,
            name: npc_character.label.clone(),
            role: "Visiteur".to_string(),
            metrics: PawnMetrics::seeded(lineage_seed ^ 0x9999_AAAA_BBBB_CCCC),
            history: crate::historique::HistoriqueLog::new(600),
        },
        PawnCard {
            key: PawnKey::SimWorker,
            name: "Employe 01".to_string(),
            role: "Operateur".to_string(),
            metrics: PawnMetrics::seeded(lineage_seed ^ 0x0F0F_55AA_00FF_7788),
            history: crate::historique::HistoriqueLog::new(600),
        },
    ];

    for pawn in &mut pawns {
        pawn.history.push(
            0.0,
            crate::historique::LogCategorie::Systeme,
            "Arrive sur le site.".to_string(),
        );
    }
    let social_state = social::SocialState::new(&pawns, lineage_seed);

    let pawn_ui = PawnsUiState {
        selected: Some(PawnKey::Player),
        ..PawnsUiState::default()
    };

    GameState {
        world: map_copy.world,
        player,
        chariot,
        chargeur_clark,
        npc,
        camera_center: tile_center(map_copy.player_spawn),
        camera_zoom: 1.15,
        palette,
        sim,
        props: map_copy.props,
        character_catalog: character_catalog.clone(),
        lineage_seed,
        lineage,
        player_lineage_index: 2,
        npc_character,
        sim_worker_character,
        pawns,
        social_state,
        pawn_ui,
        hud_ui: HudUiState::default(),
        pause_menu_open: false,
        pause_panel: PausePanel::Aucun,
        pause_status_text: None,
        pause_status_timer: 0.0,
        pause_save_name: String::new(),
        pause_sauvegardes: Vec::new(),
        pause_sauvegardes_warning: None,
        pause_sauvegardes_offset: 0,
        pause_selected_sauvegarde: None,
        show_character_inspector: false,
        debug: false,
        last_input: Vec2::ZERO,
    }
}
