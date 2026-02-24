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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_map(w: i32, h: i32) -> MapAsset {
        let mut world = World {
            w,
            h,
            tiles: vec![Tile::Floor; (w * h) as usize],
        };
        enforce_world_border(&mut world);
        MapAsset {
            version: MAP_FILE_VERSION,
            label: "Test".to_string(),
            world,
            props: Vec::new(),
            player_spawn: (1, 1),
            npc_spawn: (w - 2, h - 2),
        }
    }

    #[test]
    fn border_helpers_detect_and_enforce_walls() {
        let mut world = World {
            w: 6,
            h: 5,
            tiles: vec![Tile::Floor; 30],
        };
        enforce_world_border(&mut world);

        assert!(is_border_tile(&world, (0, 0)));
        assert!(is_border_tile(&world, (5, 4)));
        assert!(!is_border_tile(&world, (2, 2)));
        assert_eq!(world.get(0, 2), Tile::Wall);
        assert_eq!(world.get(5, 2), Tile::Wall);
        assert_eq!(world.get(2, 0), Tile::Wall);
        assert_eq!(world.get(2, 4), Tile::Wall);
    }

    #[test]
    fn set_prop_updates_existing_slot_and_rejects_invalid_tiles() {
        let mut map = test_map(8, 8);
        let tile = (3, 3);

        assert!(set_prop_at_tile(&mut map, tile, PropKind::Crate));
        let idx = prop_index_at_tile(&map.props, tile).expect("prop doit exister");
        assert_eq!(map.props[idx].phase, prop_phase_for_tile(tile));
        assert_eq!(map.props[idx].kind, PropKind::Crate);

        assert!(!set_prop_at_tile(&mut map, tile, PropKind::Crate));
        assert!(set_prop_at_tile(&mut map, tile, PropKind::Lamp));
        let idx = prop_index_at_tile(&map.props, tile).expect("prop doit exister");
        assert_eq!(map.props[idx].kind, PropKind::Lamp);

        assert!(!set_prop_at_tile(&mut map, (0, 0), PropKind::Pipe));
        assert!(!set_prop_at_tile(&mut map, (-1, 0), PropKind::Pipe));
    }

    #[test]
    fn set_map_tile_enforces_border_and_removes_props_on_wall() {
        let mut map = test_map(9, 9);
        let tile = (4, 4);
        assert!(set_prop_at_tile(&mut map, tile, PropKind::Crystal));
        assert!(prop_index_at_tile(&map.props, tile).is_some());

        assert!(!set_map_tile(&mut map, (0, 3), Tile::FloorWood));
        assert!(set_map_tile(&mut map, tile, Tile::WallSteel));
        assert_eq!(map.world.get(tile.0, tile.1), Tile::WallSteel);
        assert!(prop_index_at_tile(&map.props, tile).is_none());
    }

    #[test]
    fn brush_rect_is_inclusive_for_all_tiles() {
        let mut map = test_map(12, 12);
        assert!(editor_apply_brush_rect(
            &mut map,
            EditorBrush::FloorWood,
            (3, 4),
            (5, 6)
        ));

        for y in 4..=6 {
            for x in 3..=5 {
                assert_eq!(map.world.get(x, y), Tile::FloorWood);
            }
        }
    }

    #[test]
    fn undo_stack_is_capped_and_stroke_reset_clears_state() {
        let mut map = test_map(8, 8);
        let mut editor = EditorState::new();
        editor.stroke_active = true;
        editor.stroke_changed = true;
        editor.drag_start = Some((2, 2));

        for i in 0..(EDITOR_UNDO_LIMIT + 7) {
            let tile = if i % 2 == 0 {
                Tile::Floor
            } else {
                Tile::FloorMetal
            };
            map.world.set(2, 2, tile);
            editor_push_undo(&mut editor, &map);
        }

        assert_eq!(editor.undo_stack.len(), EDITOR_UNDO_LIMIT);
        editor_reset_stroke_state(&mut editor);
        assert!(!editor.stroke_active);
        assert!(!editor.stroke_changed);
        assert!(editor.drag_start.is_none());
    }

    #[test]
    fn deserialize_map_rejects_invalid_payload_and_sanitizes_border() {
        assert!(deserialize_map_asset("not a ron payload").is_err());

        let mut map = test_map(10, 10);
        map.world.set(0, 5, Tile::FloorMoss);
        let encoded = serialize_map_asset(&map).expect("serialisation attendue");
        let decoded = deserialize_map_asset(&encoded).expect("deserialisation attendue");
        assert_eq!(decoded.world.get(0, 5), Tile::Wall);
    }

    #[test]
    fn labels_for_brushes_tools_and_props_are_non_empty() {
        assert!(!prop_kind_label(PropKind::Lavabo).is_empty());
        assert!(!editor_brush_label(EditorBrush::CaisseAilBrut).is_empty());
        assert_eq!(editor_tool_label(EditorTool::Brush), "Pinceau");
        assert_eq!(editor_tool_label(EditorTool::Rect), "Rectangle");
    }
}
