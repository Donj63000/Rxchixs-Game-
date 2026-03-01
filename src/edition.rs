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
        EditorTool::Select => "Selection",
        EditorTool::Brush => "Pinceau",
        EditorTool::Rect => "Rectangle",
        EditorTool::Line => "Ligne",
        EditorTool::Fill => "Remplissage",
        EditorTool::Paste => "Coller",
    }
}

pub(crate) fn zone_kind_label(kind: ZoneKind) -> &'static str {
    match kind {
        ZoneKind::Logistique => "logistique",
        ZoneKind::Propre => "propre",
        ZoneKind::Froide => "froide",
        ZoneKind::Production => "production",
        ZoneKind::Stockage => "stockage",
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub message: String,
    pub tile: Option<(i32, i32)>,
}

pub(crate) fn editor_capture_snapshot(map: &MapAsset) -> EditorSnapshot {
    EditorSnapshot {
        world: map.world.clone(),
        props: map.props.clone(),
        zones: map.zones.clone(),
        player_spawn: map.player_spawn,
        npc_spawn: map.npc_spawn,
    }
}

pub(crate) fn editor_apply_snapshot(map: &mut MapAsset, snapshot: EditorSnapshot) {
    map.world = snapshot.world;
    map.props = snapshot.props;
    map.zones = snapshot.zones;
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
        Ok(()) => {
            editor.ui.dirty = false;
            editor_set_status(editor, format!("Carte sauvegardee: {}", MAP_FILE_PATH));
        }
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
            editor.selected_tile = None;
            editor.selected_prop = None;
            editor.selection_rect = None;
            editor.clipboard = None;
            editor.ui.dirty = false;
            editor.camera_initialized = false;
            editor_set_status(editor, format!("Carte chargee: {}", MAP_FILE_PATH));
        }
        Err(err) => editor_set_status(editor, err),
    }
}

pub(crate) fn editor_set_player_spawn(
    editor: &mut EditorState,
    map: &mut MapAsset,
    tile: (i32, i32),
) -> Result<(), &'static str> {
    if !map.world.in_bounds(tile.0, tile.1) {
        return Err("tuile hors carte");
    }
    if map.world.is_solid(tile.0, tile.1) {
        return Err("tuile non marchable");
    }
    if map.player_spawn == tile {
        return Err("deja sur cette tuile");
    }
    editor_push_undo(editor, map);
    map.player_spawn = tile;
    editor.redo_stack.clear();
    editor.ui.dirty = true;
    Ok(())
}

pub(crate) fn editor_set_npc_spawn(
    editor: &mut EditorState,
    map: &mut MapAsset,
    tile: (i32, i32),
) -> Result<(), &'static str> {
    if !map.world.in_bounds(tile.0, tile.1) {
        return Err("tuile hors carte");
    }
    if map.world.is_solid(tile.0, tile.1) {
        return Err("tuile non marchable");
    }
    if map.npc_spawn == tile {
        return Err("deja sur cette tuile");
    }
    editor_push_undo(editor, map);
    map.npc_spawn = tile;
    editor.redo_stack.clear();
    editor.ui.dirty = true;
    Ok(())
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

#[allow(dead_code)]
pub(crate) fn set_prop_at_tile(map: &mut MapAsset, tile: (i32, i32), kind: PropKind) -> bool {
    set_prop_at_tile_with_rotation(map, tile, kind, 0)
}

pub(crate) fn set_prop_at_tile_with_rotation(
    map: &mut MapAsset,
    tile: (i32, i32),
    kind: PropKind,
    rotation_quarter: i8,
) -> bool {
    if !map.world.in_bounds(tile.0, tile.1) || map.world.is_solid(tile.0, tile.1) {
        return false;
    }
    let rotation = rotation_quarter.rem_euclid(4);
    if let Some(idx) = prop_index_at_tile(&map.props, tile) {
        if map.props[idx].kind == kind && map.props[idx].rotation_quarter == rotation {
            return false;
        }
        map.props[idx].kind = kind;
        map.props[idx].rotation_quarter = rotation;
        map.props[idx].phase = prop_phase_for_tile(tile);
        return true;
    }

    map.props.push(Prop {
        tile_x: tile.0,
        tile_y: tile.1,
        kind,
        phase: prop_phase_for_tile(tile),
        rotation_quarter: rotation,
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

#[allow(dead_code)]
pub(crate) fn editor_apply_brush(map: &mut MapAsset, brush: EditorBrush, tile: (i32, i32)) -> bool {
    editor_apply_brush_with_rotation(map, brush, tile, 0)
}

pub(crate) fn editor_brush_to_tile(brush: EditorBrush) -> Option<Tile> {
    match brush {
        EditorBrush::Floor => Some(Tile::Floor),
        EditorBrush::FloorMetal => Some(Tile::FloorMetal),
        EditorBrush::FloorWood => Some(Tile::FloorWood),
        EditorBrush::FloorMoss => Some(Tile::FloorMoss),
        EditorBrush::FloorSand => Some(Tile::FloorSand),
        EditorBrush::Wall => Some(Tile::Wall),
        EditorBrush::WallBrick => Some(Tile::WallBrick),
        EditorBrush::WallSteel => Some(Tile::WallSteel),
        EditorBrush::WallNeon => Some(Tile::WallNeon),
        _ => None,
    }
}

pub(crate) fn editor_apply_brush_with_rotation(
    map: &mut MapAsset,
    brush: EditorBrush,
    tile: (i32, i32),
    rotation_quarter: i8,
) -> bool {
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
        EditorBrush::Crate => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::Crate, rotation_quarter)
        }
        EditorBrush::Pipe => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::Pipe, rotation_quarter)
        }
        EditorBrush::Lamp => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::Lamp, rotation_quarter)
        }
        EditorBrush::Banner => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::Banner, rotation_quarter)
        }
        EditorBrush::Plant => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::Plant, rotation_quarter)
        }
        EditorBrush::Bench => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::Bench, rotation_quarter)
        }
        EditorBrush::Crystal => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::Crystal, rotation_quarter)
        }
        EditorBrush::BoxCartonVide => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::BoxCartonVide, rotation_quarter)
        }
        EditorBrush::BoxSacBleu => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::BoxSacBleu, rotation_quarter)
        }
        EditorBrush::BoxSacRouge => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::BoxSacRouge, rotation_quarter)
        }
        EditorBrush::BoxSacVert => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::BoxSacVert, rotation_quarter)
        }
        EditorBrush::PaletteLogistique => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::PaletteLogistique, rotation_quarter)
        }
        EditorBrush::BureauPcOn => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::BureauPcOn, rotation_quarter)
        }
        EditorBrush::BureauPcOff => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::BureauPcOff, rotation_quarter)
        }
        EditorBrush::CaisseAilBrut => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::CaisseAilBrut, rotation_quarter)
        }
        EditorBrush::CaisseAilCasse => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::CaisseAilCasse, rotation_quarter)
        }
        EditorBrush::Lavabo => {
            set_prop_at_tile_with_rotation(map, tile, PropKind::Lavabo, rotation_quarter)
        }
        EditorBrush::EraseProp => remove_prop_at_tile(map, tile),
    }
}

#[allow(dead_code)]
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

fn zone_label_for_kind(kind: ZoneKind) -> String {
    match kind {
        ZoneKind::Logistique => "Zone logistique".to_string(),
        ZoneKind::Propre => "Zone propre".to_string(),
        ZoneKind::Froide => "Zone froide".to_string(),
        ZoneKind::Production => "Zone production".to_string(),
        ZoneKind::Stockage => "Zone stockage".to_string(),
    }
}

fn zone_id_for_kind(kind: ZoneKind) -> u16 {
    match kind {
        ZoneKind::Logistique => 1,
        ZoneKind::Propre => 2,
        ZoneKind::Froide => 3,
        ZoneKind::Production => 4,
        ZoneKind::Stockage => 5,
    }
}

pub(crate) fn zone_kind_at_tile(map: &MapAsset, tile: (i32, i32)) -> Option<ZoneKind> {
    for zone in &map.zones {
        if zone.tiles.contains(&tile) {
            return Some(zone.kind);
        }
    }
    None
}

pub(crate) fn set_zone_kind_at_tile(
    map: &mut MapAsset,
    tile: (i32, i32),
    zone_kind: Option<ZoneKind>,
) -> bool {
    if !map.world.in_bounds(tile.0, tile.1) {
        return false;
    }
    if zone_kind_at_tile(map, tile) == zone_kind {
        return false;
    }
    let mut changed = false;
    for zone in &mut map.zones {
        let before = zone.tiles.len();
        zone.tiles.retain(|&t| t != tile);
        changed |= zone.tiles.len() != before;
    }
    map.zones.retain(|zone| !zone.tiles.is_empty());

    if let Some(kind) = zone_kind {
        let wanted_id = zone_id_for_kind(kind);
        if let Some(index) = map.zones.iter().position(|zone| zone.id == wanted_id) {
            if !map.zones[index].tiles.contains(&tile) {
                map.zones[index].tiles.push(tile);
                changed = true;
            }
        } else {
            map.zones.push(ZoneRegion {
                id: wanted_id,
                label: zone_label_for_kind(kind),
                kind,
                acces_restreint: false,
                tags: Vec::new(),
                tiles: vec![tile],
            });
            changed = true;
        }
    }
    changed
}

fn sanitize_layout_name(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        let keep = ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.');
        if keep {
            out.push(ch);
        } else if ch == ' ' {
            out.push('_');
        }
    }
    out.trim_matches('.').to_string()
}

fn editor_layout_path(name: &str) -> Result<String, String> {
    let cleaned = sanitize_layout_name(name);
    if cleaned.is_empty() {
        return Err("Nom de layout vide".to_string());
    }
    let file_name = if cleaned.ends_with(".ron") {
        cleaned
    } else {
        format!("{cleaned}.ron")
    };
    Ok(format!("{EDITOR_LAYOUTS_DIR_PATH}/{file_name}"))
}

fn editor_blueprint_path(name: &str) -> Result<String, String> {
    let cleaned = sanitize_layout_name(name);
    if cleaned.is_empty() {
        return Err("Nom de blueprint vide".to_string());
    }
    let file_name = if cleaned.ends_with(".ron") {
        cleaned
    } else {
        format!("{cleaned}.ron")
    };
    Ok(format!("{EDITOR_BLUEPRINTS_DIR_PATH}/{file_name}"))
}

fn next_available_file_name(dir: &Path, base_stem: &str) -> String {
    let cleaned = sanitize_layout_name(base_stem)
        .trim_end_matches(".ron")
        .to_string();
    let stem = if cleaned.is_empty() {
        "layout".to_string()
    } else {
        cleaned
    };
    let mut candidate = format!("{stem}.ron");
    if !dir.join(&candidate).exists() {
        return candidate;
    }
    for index in 2..=9999 {
        candidate = format!("{stem}_{index}.ron");
        if !dir.join(&candidate).exists() {
            return candidate;
        }
    }
    format!("{stem}_overflow.ron")
}

#[derive(Serialize, Deserialize)]
struct EditorBlueprintAsset {
    schema_version: u32,
    label: String,
    width: i32,
    height: i32,
    tiles: Vec<(i32, i32, Tile)>,
    props: Vec<(i32, i32, PropKind, i8)>,
    zones: Vec<(i32, i32, ZoneKind)>,
}

fn capture_full_map_clipboard(map: &MapAsset) -> EditorClipboard {
    let mut tiles = Vec::with_capacity((map.world.w * map.world.h) as usize);
    let mut props = Vec::with_capacity(map.props.len());
    let mut zones = Vec::new();

    for y in 0..map.world.h {
        for x in 0..map.world.w {
            tiles.push((x, y, map.world.get(x, y)));
        }
    }
    for prop in &map.props {
        props.push((prop.tile_x, prop.tile_y, prop.kind, prop.rotation_quarter));
    }
    for zone in &map.zones {
        for &(x, y) in &zone.tiles {
            zones.push((x, y, zone.kind));
        }
    }

    EditorClipboard {
        width: map.world.w,
        height: map.world.h,
        tiles,
        props,
        zones,
    }
}

pub(crate) fn editor_duplicate_layout(
    _editor: &mut EditorState,
    source_name: &str,
    requested_name: Option<&str>,
) -> Result<String, String> {
    let source_path = editor_layout_path(source_name)?;
    let mut source_map = load_map_asset(&source_path)?;
    let source_stem = Path::new(source_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("layout");
    let wanted_stem = requested_name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            if source_stem.ends_with("_copie") {
                source_stem
            } else {
                "layout_copie"
            }
        });
    let wanted_stem = if wanted_stem == "layout_copie" {
        format!("{source_stem}_copie")
    } else {
        wanted_stem.to_string()
    };

    let dir = Path::new(EDITOR_LAYOUTS_DIR_PATH);
    fs::create_dir_all(dir).map_err(|err| format!("echec creation dossier layouts: {err}"))?;
    let file_name = next_available_file_name(dir, &wanted_stem);
    let target_path = dir.join(&file_name);
    source_map.label = file_name.trim_end_matches(".ron").to_string();
    save_map_asset(
        target_path
            .to_str()
            .ok_or_else(|| "chemin layout invalide".to_string())?,
        &source_map,
    )?;
    Ok(file_name)
}

pub(crate) fn editor_export_blueprint(
    editor: &EditorState,
    map: &MapAsset,
    requested_name: Option<&str>,
) -> Result<String, String> {
    let clip =
        capture_selection_clipboard(editor, map).unwrap_or_else(|| capture_full_map_clipboard(map));
    let base = requested_name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{}_blueprint", sanitize_layout_name(&map.label)));
    let _ = editor_blueprint_path(&base)?;

    let dir = Path::new(EDITOR_BLUEPRINTS_DIR_PATH);
    fs::create_dir_all(dir).map_err(|err| format!("echec creation dossier blueprints: {err}"))?;
    let file_name = next_available_file_name(dir, &base);
    let payload = EditorBlueprintAsset {
        schema_version: 1,
        label: file_name.trim_end_matches(".ron").to_string(),
        width: clip.width,
        height: clip.height,
        tiles: clip.tiles,
        props: clip.props,
        zones: clip.zones,
    };
    let pretty = PrettyConfig::new()
        .depth_limit(4)
        .enumerate_arrays(true)
        .separate_tuple_members(true);
    let encoded =
        ron_to_string_pretty(&payload, pretty).map_err(|err| format!("blueprint ron: {err}"))?;
    let path = dir.join(&file_name);
    fs::write(&path, encoded).map_err(|err| format!("echec ecriture blueprint: {err}"))?;
    Ok(file_name)
}

pub(crate) fn editor_autosave_map(
    _editor: &EditorState,
    map: &MapAsset,
    autosave_path: &str,
) -> Result<(), String> {
    let mut snapshot = map.clone();
    sanitize_map_asset(&mut snapshot);
    save_map_asset(autosave_path, &snapshot)
}

pub(crate) fn refresh_editor_layouts(editor: &mut EditorState) {
    let mut names = Vec::new();
    match fs::read_dir(EDITOR_LAYOUTS_DIR_PATH) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension()
                    && ext == "ron"
                    && let Some(name) = entry.path().file_name().and_then(|n| n.to_str())
                {
                    names.push(name.to_string());
                }
            }
            names.sort();
            editor.ui.layout_entries = names;
            if let Some(selected) = editor.ui.selected_layout {
                let max_index = editor.ui.layout_entries.len().saturating_sub(1);
                editor.ui.selected_layout = Some(selected.min(max_index));
            }
        }
        Err(_) => {
            editor.ui.layout_entries.clear();
            editor.ui.selected_layout = None;
        }
    }
}

pub(crate) fn editor_save_map_as(
    editor: &mut EditorState,
    map: &mut MapAsset,
    name: &str,
) -> Result<String, String> {
    let path = editor_layout_path(name)?;
    let saved_name = Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("layout.ron")
        .to_string();
    sanitize_map_asset(map);
    map.label = saved_name.replace(".ron", "");
    save_map_asset(&path, map)?;
    editor.ui.dirty = false;
    Ok(saved_name)
}

pub(crate) fn editor_load_map_as(
    editor: &mut EditorState,
    map: &mut MapAsset,
    name: &str,
) -> Result<(), String> {
    let path = editor_layout_path(name)?;
    let loaded = load_map_asset(&path)?;
    *map = loaded;
    editor.undo_stack.clear();
    editor.redo_stack.clear();
    editor_reset_stroke_state(editor);
    editor.selected_tile = None;
    editor.selected_prop = None;
    editor.selection_rect = None;
    editor.clipboard = None;
    editor.camera_initialized = false;
    editor.ui.dirty = false;
    Ok(())
}

fn compute_reachable_tiles(world: &World, start: (i32, i32)) -> HashSet<(i32, i32)> {
    if !world.in_bounds(start.0, start.1) || world.is_solid(start.0, start.1) {
        return HashSet::new();
    }
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    queue.push_back(start);
    visited.insert(start);
    while let Some((x, y)) = queue.pop_front() {
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if !world.in_bounds(nx, ny) || world.is_solid(nx, ny) {
                continue;
            }
            if visited.insert((nx, ny)) {
                queue.push_back((nx, ny));
            }
        }
    }
    visited
}

pub(crate) fn validate_map_asset(map: &MapAsset) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if !map.world.in_bounds(map.player_spawn.0, map.player_spawn.1)
        || map.world.is_solid(map.player_spawn.0, map.player_spawn.1)
    {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            message: "Spawn joueur invalide (hors carte ou mur)".to_string(),
            tile: Some(map.player_spawn),
        });
    }
    if !map.world.in_bounds(map.npc_spawn.0, map.npc_spawn.1)
        || map.world.is_solid(map.npc_spawn.0, map.npc_spawn.1)
    {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Error,
            message: "Spawn PNJ invalide (hors carte ou mur)".to_string(),
            tile: Some(map.npc_spawn),
        });
    }

    for prop in &map.props {
        let tile = (prop.tile_x, prop.tile_y);
        if !map.world.in_bounds(tile.0, tile.1) {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                message: format!("Objet hors carte: {}", prop_kind_label(prop.kind)),
                tile: Some(tile),
            });
        } else if map.world.is_solid(tile.0, tile.1) {
            issues.push(ValidationIssue {
                severity: ValidationSeverity::Error,
                message: format!("Objet sur mur: {}", prop_kind_label(prop.kind)),
                tile: Some(tile),
            });
        }
    }

    let reachable = compute_reachable_tiles(&map.world, map.player_spawn);
    if !reachable.is_empty() {
        for zone in &map.zones {
            let has_access = zone.tiles.iter().any(|tile| reachable.contains(tile));
            if !has_access && !zone.tiles.is_empty() {
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    message: format!("Zone '{}' enclavee", zone.label),
                    tile: zone.tiles.first().copied(),
                });
            }
        }
    }

    if a_star_path(&map.world, map.player_spawn, map.npc_spawn).is_none() {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Warning,
            message: "Couloir bloque entre spawns joueur/PNJ".to_string(),
            tile: Some(map.player_spawn),
        });
    } else {
        issues.push(ValidationIssue {
            severity: ValidationSeverity::Info,
            message: "Chemin joueur->PNJ valide".to_string(),
            tile: None,
        });
    }

    issues
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
    map.schema_version = MAP_SCHEMA_VERSION;

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
    for prop in &mut map.props {
        prop.rotation_quarter = prop.rotation_quarter.rem_euclid(4);
    }

    let mut seen_zone_tiles = HashSet::new();
    for zone in &mut map.zones {
        zone.id = zone_id_for_kind(zone.kind);
        if zone.label.trim().is_empty() {
            zone.label = zone_label_for_kind(zone.kind);
        }
        zone.tiles.retain(|tile| {
            map.world.in_bounds(tile.0, tile.1)
                && !map.world.is_solid(tile.0, tile.1)
                && seen_zone_tiles.insert(*tile)
        });
    }
    map.zones.retain(|zone| !zone.tiles.is_empty());

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
    let papa_character = character_catalog.spawn_founder("Papa", lineage_seed ^ 0xA114_5A2A);

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
        papa_character,
        pawns,
        social_state,
        pawn_ui,
        hud_ui: HudUiState::default(),
        telephone: crate::telephone::TelephoneEtat::default(),
        papa: crate::papa::PapaEtat::charger_depuis_fichier(PAPA_PLAN_PATH),
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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_map(w: i32, h: i32) -> MapAsset {
        let mut world = World {
            w,
            h,
            tiles: vec![Tile::Floor; (w * h) as usize],
        };
        enforce_world_border(&mut world);
        MapAsset {
            schema_version: MAP_SCHEMA_VERSION,
            version: MAP_FILE_VERSION,
            label: "Test".to_string(),
            world,
            props: Vec::new(),
            zones: Vec::new(),
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

    #[test]
    fn set_player_spawn_tracks_undo_and_clears_redo() {
        let mut map = test_map(10, 10);
        let mut editor = EditorState::new();
        editor.redo_stack.push(editor_capture_snapshot(&map));
        let before_undo = editor.undo_stack.len();

        let target = (3, 3);
        assert!(editor_set_player_spawn(&mut editor, &mut map, target).is_ok());
        assert_eq!(map.player_spawn, target);
        assert_eq!(editor.undo_stack.len(), before_undo + 1);
        assert!(editor.redo_stack.is_empty());
    }

    #[test]
    fn set_npc_spawn_rejects_invalid_tiles_without_side_effect() {
        let mut map = test_map(10, 10);
        let mut editor = EditorState::new();
        let before_npc = map.npc_spawn;
        let before_undo = editor.undo_stack.len();

        assert!(editor_set_npc_spawn(&mut editor, &mut map, (0, 0)).is_err());
        assert_eq!(map.npc_spawn, before_npc);
        assert_eq!(editor.undo_stack.len(), before_undo);
        assert!(editor.redo_stack.is_empty());
    }

    #[test]
    fn deserialize_legacy_payload_adds_schema_zones_and_prop_rotation_defaults() {
        let mut map = test_map(8, 8);
        assert!(set_prop_at_tile(&mut map, (3, 3), PropKind::Crate));
        let encoded = serialize_map_asset(&map).expect("serialisation attendue");
        let legacy = encoded
            .replace("schema_version: 2,\n", "")
            .replace("zones: [],\n", "")
            .replace("rotation_quarter: 0,\n", "");
        let decoded = deserialize_map_asset(&legacy).expect("deserialisation legacy attendue");

        assert_eq!(decoded.schema_version, MAP_SCHEMA_VERSION);
        assert!(decoded.zones.is_empty());
        let idx = prop_index_at_tile(&decoded.props, (3, 3)).expect("prop legacy");
        assert_eq!(decoded.props[idx].rotation_quarter, 0);
    }

    #[test]
    fn zone_assignment_and_validation_detects_enclaved_zone() {
        let mut map = test_map(12, 12);
        assert!(set_zone_kind_at_tile(
            &mut map,
            (8, 8),
            Some(ZoneKind::Production)
        ));
        assert_eq!(zone_kind_at_tile(&map, (8, 8)), Some(ZoneKind::Production));

        for tile in [(7, 8), (9, 8), (8, 7), (8, 9)] {
            assert!(set_map_tile(&mut map, tile, Tile::WallSteel));
        }
        let issues = validate_map_asset(&map);
        assert!(
            issues
                .iter()
                .any(|issue| issue.message.contains("enclavee"))
        );
    }

    #[test]
    fn prop_rotation_is_persisted_by_brush_application() {
        let mut map = test_map(10, 10);
        assert!(editor_apply_brush_with_rotation(
            &mut map,
            EditorBrush::Lamp,
            (4, 4),
            3
        ));
        let idx = prop_index_at_tile(&map.props, (4, 4)).expect("prop attendu");
        assert_eq!(map.props[idx].rotation_quarter, 3);
    }

    #[test]
    fn next_available_file_name_adds_increment_when_name_exists() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("temps systeme")
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("rxchixs_layout_test_{stamp}"));
        fs::create_dir_all(&temp_dir).expect("dossier temp");
        fs::write(temp_dir.join("layout.ron"), "x").expect("fichier 1");
        fs::write(temp_dir.join("layout_2.ron"), "x").expect("fichier 2");
        let picked = next_available_file_name(&temp_dir, "layout");
        assert_eq!(picked, "layout_3.ron");
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn capture_full_map_clipboard_contains_tiles_props_and_zones() {
        let mut map = test_map(9, 8);
        assert!(set_map_tile(&mut map, (3, 3), Tile::FloorSand));
        assert!(set_prop_at_tile_with_rotation(
            &mut map,
            (4, 3),
            PropKind::Lamp,
            2
        ));
        assert!(set_zone_kind_at_tile(
            &mut map,
            (5, 5),
            Some(ZoneKind::Production)
        ));

        let clip = capture_full_map_clipboard(&map);
        assert_eq!(clip.width, map.world.w);
        assert_eq!(clip.height, map.world.h);
        assert!(
            clip.tiles
                .iter()
                .any(|(x, y, tile)| *x == 3 && *y == 3 && *tile == Tile::FloorSand)
        );
        assert!(clip.props.iter().any(|(x, y, kind, rot)| {
            *x == 4 && *y == 3 && *kind == PropKind::Lamp && *rot == 2
        }));
        assert!(
            clip.zones
                .iter()
                .any(|(x, y, kind)| *x == 5 && *y == 5 && *kind == ZoneKind::Production)
        );
    }

    #[test]
    fn editor_autosave_map_writes_loadable_payload() {
        let map = test_map(10, 10);
        let editor = EditorState::new();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("temps systeme")
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("rxchixs_autosave_test_{stamp}"));
        fs::create_dir_all(&temp_dir).expect("dossier temp");
        let file_path = temp_dir.join("autosave.ron");
        let path = file_path.to_string_lossy().to_string();

        editor_autosave_map(&editor, &map, &path).expect("autosave");
        let loaded = load_map_asset(&path).expect("chargement autosave");
        assert_eq!(loaded.world.w, map.world.w);
        assert_eq!(loaded.world.h, map.world.h);
        assert_eq!(loaded.schema_version, MAP_SCHEMA_VERSION);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
