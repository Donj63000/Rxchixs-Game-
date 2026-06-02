use super::*;

#[derive(Clone)]
pub(crate) struct EditorClipboard {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<(i32, i32, Tile)>,
    pub props: Vec<(i32, i32, PropKind, i8)>,
    pub zones: Vec<(i32, i32, ZoneKind)>,
}

#[derive(Copy, Clone)]
pub(crate) struct EditorToolInput {
    pub can_edit_map: bool,
    pub left_pressed: bool,
    pub left_down: bool,
    pub left_released: bool,
    pub right_pressed: bool,
    pub shift_down: bool,
    pub hover_tile: Option<(i32, i32)>,
    pub editing_zones: bool,
}

#[inline]
pub(crate) fn rect_bounds(a: (i32, i32), b: (i32, i32)) -> ((i32, i32), (i32, i32)) {
    ((a.0.min(b.0), a.1.min(b.1)), (a.0.max(b.0), a.1.max(b.1)))
}

pub(crate) fn line_tiles(start: (i32, i32), end: (i32, i32)) -> Vec<(i32, i32)> {
    let (mut x0, mut y0) = start;
    let (x1, y1) = end;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut out = Vec::new();

    loop {
        out.push((x0, y0));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = err * 2;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    out
}

fn brush_tiles(center: (i32, i32), brush_size: u8) -> Vec<(i32, i32)> {
    let side = brush_size.max(1) as i32;
    let half = side / 2;
    let start_x = center.0 - half;
    let start_y = center.1 - half;
    let mut out = Vec::with_capacity((side * side) as usize);
    for y in 0..side {
        for x in 0..side {
            out.push((start_x + x, start_y + y));
        }
    }
    out
}

fn flood_fill_tiles(map: &MapAsset, start: (i32, i32), target: Tile) -> Vec<(i32, i32)> {
    if !map.world.in_bounds(start.0, start.1) {
        return Vec::new();
    }
    let source = map.world.get(start.0, start.1);
    if source == target {
        return Vec::new();
    }
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut out = Vec::new();
    queue.push_back(start);
    visited.insert(start);

    while let Some((x, y)) = queue.pop_front() {
        if !map.world.in_bounds(x, y) || map.world.get(x, y) != source {
            continue;
        }
        out.push((x, y));
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if map.world.in_bounds(nx, ny) && visited.insert((nx, ny)) {
                queue.push_back((nx, ny));
            }
        }
    }
    out
}

fn flood_fill_zone_tiles(
    map: &MapAsset,
    start: (i32, i32),
    source: Option<ZoneKind>,
) -> Vec<(i32, i32)> {
    if !map.world.in_bounds(start.0, start.1) {
        return Vec::new();
    }
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut out = Vec::new();
    queue.push_back(start);
    visited.insert(start);

    while let Some((x, y)) = queue.pop_front() {
        if !map.world.in_bounds(x, y) {
            continue;
        }
        if zone_kind_at_tile(map, (x, y)) != source {
            continue;
        }
        out.push((x, y));
        for (nx, ny) in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if map.world.in_bounds(nx, ny) && visited.insert((nx, ny)) {
                queue.push_back((nx, ny));
            }
        }
    }
    out
}

fn apply_tiles_with_brush(
    editor: &EditorState,
    map: &mut MapAsset,
    tiles: impl IntoIterator<Item = (i32, i32)>,
    editing_zones: bool,
) -> bool {
    let mut changed = false;
    for tile in tiles {
        if editing_zones {
            changed |= set_zone_kind_at_tile(map, tile, Some(editor.zone_kind));
        } else {
            changed |=
                editor_apply_brush_with_rotation(map, editor.brush, tile, editor.prop_rotation);
        }
    }
    changed
}

fn capture_tile_or_rect(editor: &EditorState) -> Option<((i32, i32), (i32, i32))> {
    if let Some((a, b)) = editor.selection_rect {
        return Some(rect_bounds(a, b));
    }
    editor.selected_tile.map(|tile| (tile, tile))
}

pub(crate) fn capture_selection_clipboard(
    editor: &EditorState,
    map: &MapAsset,
) -> Option<EditorClipboard> {
    let ((min_x, min_y), (max_x, max_y)) = capture_tile_or_rect(editor)?;
    let mut tiles = Vec::new();
    let mut props = Vec::new();
    let mut zones = Vec::new();

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if !map.world.in_bounds(x, y) {
                continue;
            }
            tiles.push((x - min_x, y - min_y, map.world.get(x, y)));
            if let Some(idx) = prop_index_at_tile(&map.props, (x, y)) {
                let prop = map.props[idx];
                props.push((x - min_x, y - min_y, prop.kind, prop.rotation_quarter));
            }
            if let Some(kind) = zone_kind_at_tile(map, (x, y)) {
                zones.push((x - min_x, y - min_y, kind));
            }
        }
    }

    Some(EditorClipboard {
        width: max_x - min_x + 1,
        height: max_y - min_y + 1,
        tiles,
        props,
        zones,
    })
}

pub(crate) fn paste_clipboard_at(
    map: &mut MapAsset,
    clip: &EditorClipboard,
    anchor: (i32, i32),
) -> bool {
    let mut changed = false;
    for &(dx, dy, tile) in &clip.tiles {
        let world_tile = (anchor.0 + dx, anchor.1 + dy);
        changed |= set_map_tile(map, world_tile, tile);
    }
    for &(dx, dy, kind, rot) in &clip.props {
        let world_tile = (anchor.0 + dx, anchor.1 + dy);
        changed |= set_prop_at_tile_with_rotation(map, world_tile, kind, rot);
    }
    for &(dx, dy, kind) in &clip.zones {
        let world_tile = (anchor.0 + dx, anchor.1 + dy);
        changed |= set_zone_kind_at_tile(map, world_tile, Some(kind));
    }
    changed
}

pub(crate) fn eyedropper_pick_brush(map: &MapAsset, tile: (i32, i32)) -> Option<EditorBrush> {
    if !map.world.in_bounds(tile.0, tile.1) {
        return None;
    }
    if let Some(idx) = prop_index_at_tile(&map.props, tile) {
        return Some(editor_brush_for_prop(map.props[idx].kind));
    }
    Some(editor_brush_for_tile(map.world.get(tile.0, tile.1)))
}

fn editor_brush_for_tile(tile: Tile) -> EditorBrush {
    match tile {
        Tile::Floor => EditorBrush::Floor,
        Tile::FloorMetal => EditorBrush::FloorMetal,
        Tile::FloorWood => EditorBrush::FloorWood,
        Tile::FloorMoss => EditorBrush::FloorMoss,
        Tile::FloorSand => EditorBrush::FloorSand,
        Tile::Wall => EditorBrush::Wall,
        Tile::WallBrick => EditorBrush::WallBrick,
        Tile::WallSteel => EditorBrush::WallSteel,
        Tile::WallNeon => EditorBrush::WallNeon,
    }
}

fn editor_brush_for_prop(kind: PropKind) -> EditorBrush {
    match kind {
        PropKind::Crate => EditorBrush::Crate,
        PropKind::Pipe => EditorBrush::Pipe,
        PropKind::Lamp => EditorBrush::Lamp,
        PropKind::Banner => EditorBrush::Banner,
        PropKind::Plant => EditorBrush::Plant,
        PropKind::Bench => EditorBrush::Bench,
        PropKind::Crystal => EditorBrush::Crystal,
        PropKind::BoxCartonVide => EditorBrush::BoxCartonVide,
        PropKind::BoxSacBleu => EditorBrush::BoxSacBleu,
        PropKind::BoxSacRouge => EditorBrush::BoxSacRouge,
        PropKind::BoxSacVert => EditorBrush::BoxSacVert,
        PropKind::PaletteLogistique => EditorBrush::PaletteLogistique,
        PropKind::BureauPcOn => EditorBrush::BureauPcOn,
        PropKind::BureauPcOff => EditorBrush::BureauPcOff,
        PropKind::CaisseAilBrut => EditorBrush::CaisseAilBrut,
        PropKind::CaisseAilCasse => EditorBrush::CaisseAilCasse,
        PropKind::Lavabo => EditorBrush::Lavabo,
    }
}

pub(crate) fn apply_editor_tool(
    editor: &mut EditorState,
    map: &mut MapAsset,
    input: EditorToolInput,
) -> bool {
    if input.right_pressed
        && input.can_edit_map
        && let Some(tile) = input.hover_tile
    {
        editor.selected_tile = Some(tile);
        editor.selection_rect = None;
        editor.selected_prop = prop_index_at_tile(&map.props, tile);
    }

    if !input.can_edit_map {
        if input.left_released {
            editor.stroke_active = false;
            editor.stroke_changed = false;
            editor.drag_start = None;
        }
        return false;
    }

    let mut changed = false;
    match editor.tool {
        EditorTool::Select => {
            if input.left_pressed {
                editor.drag_start = input.hover_tile;
            }
            if input.left_released {
                if let Some(start) = editor.drag_start
                    && let Some(end) = input.hover_tile
                {
                    if start == end {
                        editor.selected_tile = Some(end);
                        editor.selection_rect = None;
                        editor.selected_prop = prop_index_at_tile(&map.props, end);
                    } else {
                        let rect = if input.shift_down {
                            if let Some((a, b)) = editor.selection_rect {
                                let ((min_a_x, min_a_y), (max_a_x, max_a_y)) = rect_bounds(a, b);
                                let ((min_b_x, min_b_y), (max_b_x, max_b_y)) =
                                    rect_bounds(start, end);
                                (
                                    (min_a_x.min(min_b_x), min_a_y.min(min_b_y)),
                                    (max_a_x.max(max_b_x), max_a_y.max(max_b_y)),
                                )
                            } else {
                                rect_bounds(start, end)
                            }
                        } else {
                            rect_bounds(start, end)
                        };
                        editor.selection_rect = Some(rect);
                        editor.selected_tile = Some(start);
                        editor.selected_prop = None;
                    }
                }
                editor.drag_start = None;
            }
        }
        EditorTool::Brush => {
            if input.left_pressed && input.hover_tile.is_some() {
                editor_push_undo(editor, map);
                editor.stroke_active = true;
                editor.stroke_changed = false;
            }
            if editor.stroke_active
                && input.left_down
                && let Some(tile) = input.hover_tile
            {
                let tiled = brush_tiles(tile, editor.brush_size);
                let step_changed = apply_tiles_with_brush(editor, map, tiled, input.editing_zones);
                if step_changed {
                    editor.stroke_changed = true;
                    changed = true;
                    editor.redo_stack.clear();
                }
            }
            if input.left_released {
                if editor.stroke_active && !editor.stroke_changed {
                    let _ = editor.undo_stack.pop();
                }
                editor.stroke_active = false;
                editor.stroke_changed = false;
            }
        }
        EditorTool::Rect => {
            if input.left_pressed {
                editor.drag_start = input.hover_tile;
            }
            if input.left_released {
                if let Some(start) = editor.drag_start
                    && let Some(end) = input.hover_tile
                {
                    let before = editor.undo_stack.len();
                    editor_push_undo(editor, map);
                    let ((min_x, min_y), (max_x, max_y)) = rect_bounds(start, end);
                    let mut local = false;
                    for y in min_y..=max_y {
                        for x in min_x..=max_x {
                            let tiles = brush_tiles((x, y), editor.brush_size);
                            local |=
                                apply_tiles_with_brush(editor, map, tiles, input.editing_zones);
                        }
                    }
                    if local {
                        changed = true;
                        editor.redo_stack.clear();
                    } else {
                        editor.undo_stack.truncate(before);
                    }
                }
                editor.drag_start = None;
            }
        }
        EditorTool::Line => {
            if input.left_pressed {
                editor.drag_start = input.hover_tile;
            }
            if input.left_released {
                if let Some(start) = editor.drag_start
                    && let Some(end) = input.hover_tile
                {
                    let before = editor.undo_stack.len();
                    editor_push_undo(editor, map);
                    let mut local = false;
                    for tile in line_tiles(start, end) {
                        local |= apply_tiles_with_brush(
                            editor,
                            map,
                            brush_tiles(tile, editor.brush_size),
                            input.editing_zones,
                        );
                    }
                    if local {
                        changed = true;
                        editor.redo_stack.clear();
                    } else {
                        editor.undo_stack.truncate(before);
                    }
                }
                editor.drag_start = None;
            }
        }
        EditorTool::Fill => {
            if input.left_pressed
                && let Some(start) = input.hover_tile
            {
                let before = editor.undo_stack.len();
                editor_push_undo(editor, map);
                let local = if input.editing_zones {
                    let source = zone_kind_at_tile(map, start);
                    let fill = flood_fill_zone_tiles(map, start, source);
                    apply_tiles_with_brush(editor, map, fill, true)
                } else if let Some(fill_tile) = editor_brush_to_tile(editor.brush) {
                    let fill = flood_fill_tiles(map, start, fill_tile);
                    apply_tiles_with_brush(editor, map, fill, false)
                } else {
                    false
                };
                if local {
                    changed = true;
                    editor.redo_stack.clear();
                } else {
                    editor.undo_stack.truncate(before);
                }
            }
        }
        EditorTool::Paste => {
            if input.left_pressed
                && let Some(anchor) = input.hover_tile
            {
                let Some(clip) = editor.clipboard.clone() else {
                    return changed;
                };
                let before = editor.undo_stack.len();
                editor_push_undo(editor, map);
                if paste_clipboard_at(map, &clip, anchor) {
                    changed = true;
                    editor.redo_stack.clear();
                } else {
                    editor.undo_stack.truncate(before);
                }
            }
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_map(w: i32, h: i32) -> MapAsset {
        let mut world = World {
            w,
            h,
            tiles: vec![Tile::Floor; (w * h) as usize],
            revision: 0,
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
    fn line_tiles_include_endpoints_and_middle() {
        let tiles = line_tiles((2, 2), (5, 2));
        assert_eq!(tiles.first().copied(), Some((2, 2)));
        assert_eq!(tiles.last().copied(), Some((5, 2)));
        assert!(tiles.contains(&(3, 2)));
        assert!(tiles.contains(&(4, 2)));
    }

    #[test]
    fn flood_fill_changes_only_connected_region() {
        let mut map = test_map(12, 12);
        map.world.set(6, 6, Tile::Wall);
        map.world.set(6, 7, Tile::Wall);
        map.world.set(6, 8, Tile::Wall);
        let mut editor = EditorState::new();
        editor.tool = EditorTool::Fill;
        editor.brush = EditorBrush::FloorWood;

        let changed = apply_editor_tool(
            &mut editor,
            &mut map,
            EditorToolInput {
                can_edit_map: true,
                left_pressed: true,
                left_down: true,
                left_released: false,
                right_pressed: false,
                shift_down: false,
                hover_tile: Some((2, 2)),
                editing_zones: false,
            },
        );
        assert!(changed);
        assert_eq!(map.world.get(2, 2), Tile::FloorWood);
        assert_eq!(map.world.get(9, 9), Tile::FloorWood);
        assert_eq!(map.world.get(6, 7), Tile::Wall);
    }

    #[test]
    fn copy_paste_selection_transfers_tiles_and_props() {
        let mut map = test_map(14, 14);
        assert!(set_map_tile(&mut map, (3, 3), Tile::FloorSand));
        assert!(set_prop_at_tile_with_rotation(
            &mut map,
            (4, 3),
            PropKind::Lamp,
            2
        ));
        let mut editor = EditorState::new();
        editor.selection_rect = Some(((3, 3), (4, 3)));

        let clip = capture_selection_clipboard(&editor, &map).expect("clipboard attendu");
        assert_eq!(clip.width, 2);
        assert_eq!(clip.height, 1);

        let pasted = paste_clipboard_at(&mut map, &clip, (8, 8));
        assert!(pasted);
        assert_eq!(map.world.get(8, 8), Tile::FloorSand);
        let idx = prop_index_at_tile(&map.props, (9, 8)).expect("prop collee attendue");
        assert_eq!(map.props[idx].kind, PropKind::Lamp);
        assert_eq!(map.props[idx].rotation_quarter, 2);
    }

    #[test]
    fn eyedropper_prefers_prop_over_tile() {
        let mut map = test_map(10, 10);
        assert!(set_map_tile(&mut map, (4, 4), Tile::FloorMoss));
        assert!(set_prop_at_tile_with_rotation(
            &mut map,
            (4, 4),
            PropKind::Bench,
            0
        ));

        let brush = eyedropper_pick_brush(&map, (4, 4)).expect("brush attendu");
        assert_eq!(brush, EditorBrush::Bench);
    }
}
