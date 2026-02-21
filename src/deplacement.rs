use super::*;

pub(crate) fn read_input_dir() -> Vec2 {
    let mut dir = Vec2::ZERO;

    if is_key_down(KeyCode::Up) {
        dir.y -= 1.0;
    }
    if is_key_down(KeyCode::Down) {
        dir.y += 1.0;
    }
    if is_key_down(KeyCode::Left) {
        dir.x -= 1.0;
    }
    if is_key_down(KeyCode::Right) {
        dir.x += 1.0;
    }

    if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec2::ZERO
    }
}

pub(crate) fn read_camera_pan_input() -> Vec2 {
    let mut dir = Vec2::ZERO;

    if is_key_down(KeyCode::W) || is_key_down(KeyCode::Z) {
        dir.y -= 1.0;
    }
    if is_key_down(KeyCode::S) {
        dir.y += 1.0;
    }
    if is_key_down(KeyCode::A) || is_key_down(KeyCode::Q) {
        dir.x -= 1.0;
    }
    if is_key_down(KeyCode::D) {
        dir.x += 1.0;
    }

    if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec2::ZERO
    }
}

pub(crate) fn read_editor_pan_input(space_held: bool) -> Vec2 {
    let mut dir = Vec2::ZERO;

    if is_key_down(KeyCode::Up) {
        dir.y -= 1.0;
    }
    if is_key_down(KeyCode::Down) {
        dir.y += 1.0;
    }
    if is_key_down(KeyCode::Left) {
        dir.x -= 1.0;
    }
    if is_key_down(KeyCode::Right) {
        dir.x += 1.0;
    }
    if space_held {
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Z) {
            dir.y -= 1.0;
        }
        if is_key_down(KeyCode::S) {
            dir.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Q) {
            dir.x -= 1.0;
        }
        if is_key_down(KeyCode::D) {
            dir.x += 1.0;
        }
    }

    if dir.length_squared() > 0.0 {
        dir.normalize()
    } else {
        Vec2::ZERO
    }
}

pub(crate) fn select_character_facing(input: Vec2, current: CharacterFacing) -> CharacterFacing {
    if input.length_squared() <= 0.0001 {
        return current;
    }

    let ax = input.x.abs();
    let ay = input.y.abs();

    if ay > ax + DIRECTION_HYSTERESIS {
        if input.y < 0.0 {
            CharacterFacing::Back
        } else {
            CharacterFacing::Front
        }
    } else if ax > ay + DIRECTION_HYSTERESIS {
        CharacterFacing::Side
    } else {
        match current {
            CharacterFacing::Side => CharacterFacing::Side,
            CharacterFacing::Back if input.y <= 0.0 => CharacterFacing::Back,
            CharacterFacing::Front if input.y >= 0.0 => CharacterFacing::Front,
            _ => {
                if input.y < 0.0 {
                    CharacterFacing::Back
                } else {
                    CharacterFacing::Front
                }
            }
        }
    }
}

pub(crate) fn facing_label(facing: CharacterFacing) -> &'static str {
    match facing {
        CharacterFacing::Front => "front",
        CharacterFacing::Side => "side",
        CharacterFacing::Back => "back",
    }
}

pub(crate) fn control_mode_label(mode: ControlMode) -> &'static str {
    match mode {
        ControlMode::Manual => "manual",
        ControlMode::AutoMove => "auto_move",
    }
}

pub(crate) fn tile_from_world_clamped(world: &World, pos: Vec2) -> (i32, i32) {
    let tx = clamp_i32((pos.x / TILE_SIZE).floor() as i32, 0, world.w - 1);
    let ty = clamp_i32((pos.y / TILE_SIZE).floor() as i32, 0, world.h - 1);
    (tx, ty)
}

pub(crate) fn tile_center(tile: (i32, i32)) -> Vec2 {
    vec2(
        (tile.0 as f32 + 0.5) * TILE_SIZE,
        (tile.1 as f32 + 0.5) * TILE_SIZE,
    )
}

pub(crate) fn idx_to_tile(world: &World, idx: usize) -> (i32, i32) {
    let idx_i32 = idx as i32;
    (idx_i32 % world.w, idx_i32 / world.w)
}

pub(crate) fn manhattan(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

pub(crate) fn move_towards_vec2(current: Vec2, target: Vec2, max_delta: f32) -> Vec2 {
    if max_delta <= 0.0 {
        return current;
    }
    let delta = target - current;
    let dist = delta.length();
    if dist <= max_delta || dist <= f32::EPSILON {
        target
    } else {
        current + delta / dist * max_delta
    }
}

pub(crate) fn nearest_walkable_tile(world: &World, desired: (i32, i32)) -> Option<(i32, i32)> {
    let start = (
        clamp_i32(desired.0, 0, world.w - 1),
        clamp_i32(desired.1, 0, world.h - 1),
    );
    if !world.is_solid(start.0, start.1) {
        return Some(start);
    }

    let mut queue = VecDeque::new();
    let mut visited = vec![false; (world.w * world.h) as usize];
    let start_idx = world.idx(start.0, start.1);
    visited[start_idx] = true;
    queue.push_back(start);

    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in dirs {
            let nx = x + dx;
            let ny = y + dy;
            if !world.in_bounds(nx, ny) {
                continue;
            }
            let idx = world.idx(nx, ny);
            if visited[idx] {
                continue;
            }
            visited[idx] = true;
            if !world.is_solid(nx, ny) {
                return Some((nx, ny));
            }
            queue.push_back((nx, ny));
        }
    }

    None
}

pub(crate) fn a_star_path(
    world: &World,
    start: (i32, i32),
    goal: (i32, i32),
) -> Option<Vec<(i32, i32)>> {
    if !world.in_bounds(start.0, start.1) || !world.in_bounds(goal.0, goal.1) {
        return None;
    }
    if world.is_solid(goal.0, goal.1) {
        return None;
    }
    if start == goal {
        return Some(vec![start]);
    }

    let size = (world.w * world.h) as usize;
    let mut g_score = vec![i32::MAX; size];
    let mut came_from: Vec<Option<usize>> = vec![None; size];
    let mut closed = vec![false; size];
    let mut open = BinaryHeap::new();

    let start_idx = world.idx(start.0, start.1);
    let goal_idx = world.idx(goal.0, goal.1);
    g_score[start_idx] = 0;
    open.push(OpenNode {
        f: manhattan(start, goal),
        g: 0,
        idx: start_idx,
    });

    let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    while let Some(node) = open.pop() {
        if closed[node.idx] {
            continue;
        }
        if node.idx == goal_idx {
            let mut rev = vec![goal];
            let mut cur = node.idx;
            while cur != start_idx {
                let prev = came_from[cur]?;
                cur = prev;
                rev.push(idx_to_tile(world, cur));
            }
            rev.reverse();
            return Some(rev);
        }

        closed[node.idx] = true;
        let (cx, cy) = idx_to_tile(world, node.idx);
        for (dx, dy) in dirs {
            let nx = cx + dx;
            let ny = cy + dy;
            if !world.in_bounds(nx, ny) || world.is_solid(nx, ny) {
                continue;
            }

            let nidx = world.idx(nx, ny);
            if closed[nidx] {
                continue;
            }

            let tentative_g = node.g + 1;
            if tentative_g < g_score[nidx] {
                g_score[nidx] = tentative_g;
                came_from[nidx] = Some(node.idx);
                open.push(OpenNode {
                    f: tentative_g + manhattan((nx, ny), goal),
                    g: tentative_g,
                    idx: nidx,
                });
            }
        }
    }

    None
}

pub(crate) fn simplify_tile_path(path: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if path.len() <= 2 {
        return path.to_vec();
    }

    let mut out = Vec::with_capacity(path.len());
    out.push(path[0]);
    let mut prev_dir = (path[1].0 - path[0].0, path[1].1 - path[0].1);

    for i in 1..path.len() - 1 {
        let dir = (path[i + 1].0 - path[i].0, path[i + 1].1 - path[i].1);
        if dir != prev_dir {
            out.push(path[i]);
            prev_dir = dir;
        }
    }

    if let Some(last) = path.last() {
        out.push(*last);
    }
    out
}

pub(crate) fn clear_auto_move_state(auto: &mut AutoMoveState) {
    auto.target_tile = None;
    auto.target_world = None;
    auto.path_tiles.clear();
    auto.path_world.clear();
    auto.next_waypoint = 0;
}

pub(crate) fn reset_auto_move(player: &mut Player) {
    player.control_mode = ControlMode::Manual;
    player.velocity = Vec2::ZERO;
    clear_auto_move_state(&mut player.auto);
}

pub(crate) fn reset_npc_auto_move(npc: &mut NpcWanderer) {
    npc.velocity = Vec2::ZERO;
    clear_auto_move_state(&mut npc.auto);
}

pub(crate) fn issue_auto_move_command(
    player: &mut Player,
    world: &World,
    requested_tile: (i32, i32),
) -> bool {
    let start_tile = tile_from_world_clamped(world, player.pos);
    let Some(goal_tile) = nearest_walkable_tile(world, requested_tile) else {
        reset_auto_move(player);
        return false;
    };

    player.auto.target_tile = Some(goal_tile);
    player.auto.target_world = Some(tile_center(goal_tile));

    if goal_tile == start_tile {
        reset_auto_move(player);
        player.auto.target_tile = Some(goal_tile);
        player.auto.target_world = Some(tile_center(goal_tile));
        return true;
    }

    let Some(raw_path) = a_star_path(world, start_tile, goal_tile) else {
        reset_auto_move(player);
        return false;
    };

    let path_tiles = simplify_tile_path(&raw_path);
    let path_world: Vec<Vec2> = path_tiles
        .iter()
        .skip(1)
        .map(|&tile| tile_center(tile))
        .collect();

    player.auto.path_tiles = path_tiles;
    player.auto.path_world = path_world;
    player.auto.next_waypoint = 0;
    player.control_mode = ControlMode::AutoMove;
    true
}

pub(crate) fn step_auto_move(player: &mut Player, dt: f32) -> Vec2 {
    if player.auto.path_world.is_empty() {
        reset_auto_move(player);
        return Vec2::ZERO;
    }

    while player.auto.next_waypoint < player.auto.path_world.len() {
        let waypoint = player.auto.path_world[player.auto.next_waypoint];
        if player.pos.distance(waypoint) <= AUTO_WAYPOINT_REACH {
            player.auto.next_waypoint += 1;
        } else {
            break;
        }
    }

    if player.auto.next_waypoint >= player.auto.path_world.len() {
        reset_auto_move(player);
        return Vec2::ZERO;
    }

    let waypoint = player.auto.path_world[player.auto.next_waypoint];
    let to_waypoint = waypoint - player.pos;
    let dist = to_waypoint.length();
    if dist <= f32::EPSILON {
        return Vec2::ZERO;
    }

    let dir = to_waypoint / dist;
    let final_target = player.auto.target_world.unwrap_or(waypoint);
    let final_dist = player.pos.distance(final_target);
    let slowdown = if final_dist < AUTO_ARRIVE_RADIUS {
        (final_dist / AUTO_ARRIVE_RADIUS).clamp(0.22, 1.0)
    } else {
        1.0
    };

    let desired_velocity = dir * (player.speed * slowdown);
    player.velocity = move_towards_vec2(player.velocity, desired_velocity, AUTO_ACCEL * dt);
    player.velocity * dt
}

pub(crate) fn apply_control_inputs(
    player: &mut Player,
    world: &World,
    keyboard_input: Vec2,
    click_tile: Option<(i32, i32)>,
) {
    if let Some(tile) = click_tile {
        let _ = issue_auto_move_command(player, world, tile);
    }

    if keyboard_input.length_squared() > 0.0 {
        reset_auto_move(player);
        player.control_mode = ControlMode::Manual;
    }
}

pub(crate) fn npc_rand_u32(npc: &mut NpcWanderer) -> u32 {
    let mut x = npc.rng_state;
    if x == 0 {
        x = 0x9E37_79B9_7F4A_7C15;
    }
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    npc.rng_state = x;
    (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 32) as u32
}

pub(crate) fn npc_rand_f32(npc: &mut NpcWanderer) -> f32 {
    npc_rand_u32(npc) as f32 / u32::MAX as f32
}

pub(crate) fn npc_rand_idle_duration(npc: &mut NpcWanderer) -> f32 {
    NPC_IDLE_MIN + (NPC_IDLE_MAX - NPC_IDLE_MIN) * npc_rand_f32(npc)
}

pub(crate) fn npc_choose_wander_target(npc: &mut NpcWanderer, world: &World) -> Option<(i32, i32)> {
    let current_tile = tile_from_world_clamped(world, npc.pos);
    let inner_w = (world.w - 2).max(1) as u32;
    let inner_h = (world.h - 2).max(1) as u32;

    for _ in 0..30 {
        let tx = 1 + (npc_rand_u32(npc) % inner_w) as i32;
        let ty = 1 + (npc_rand_u32(npc) % inner_h) as i32;
        if world.is_solid(tx, ty) {
            continue;
        }
        if manhattan((tx, ty), current_tile) < 4 {
            continue;
        }
        return Some((tx, ty));
    }

    None
}

pub(crate) fn issue_npc_wander_command(
    npc: &mut NpcWanderer,
    world: &World,
    requested_tile: (i32, i32),
) -> bool {
    let start_tile = tile_from_world_clamped(world, npc.pos);
    let Some(goal_tile) = nearest_walkable_tile(world, requested_tile) else {
        reset_npc_auto_move(npc);
        return false;
    };

    if goal_tile == start_tile {
        reset_npc_auto_move(npc);
        npc.auto.target_tile = Some(goal_tile);
        npc.auto.target_world = Some(tile_center(goal_tile));
        return true;
    }

    let Some(raw_path) = a_star_path(world, start_tile, goal_tile) else {
        reset_npc_auto_move(npc);
        return false;
    };

    let path_tiles = simplify_tile_path(&raw_path);
    let path_world: Vec<Vec2> = path_tiles
        .iter()
        .skip(1)
        .map(|&tile| tile_center(tile))
        .collect();

    npc.auto.target_tile = Some(goal_tile);
    npc.auto.target_world = Some(tile_center(goal_tile));
    npc.auto.path_tiles = path_tiles;
    npc.auto.path_world = path_world;
    npc.auto.next_waypoint = 0;
    true
}

pub(crate) fn step_npc_auto_move(npc: &mut NpcWanderer, dt: f32) -> Vec2 {
    if npc.auto.path_world.is_empty() {
        return Vec2::ZERO;
    }

    while npc.auto.next_waypoint < npc.auto.path_world.len() {
        let waypoint = npc.auto.path_world[npc.auto.next_waypoint];
        if npc.pos.distance(waypoint) <= AUTO_WAYPOINT_REACH {
            npc.auto.next_waypoint += 1;
        } else {
            break;
        }
    }

    if npc.auto.next_waypoint >= npc.auto.path_world.len() {
        return Vec2::ZERO;
    }

    let waypoint = npc.auto.path_world[npc.auto.next_waypoint];
    let to_waypoint = waypoint - npc.pos;
    let dist = to_waypoint.length();
    if dist <= f32::EPSILON {
        return Vec2::ZERO;
    }

    let dir = to_waypoint / dist;
    let final_target = npc.auto.target_world.unwrap_or(waypoint);
    let final_dist = npc.pos.distance(final_target);
    let slowdown = if final_dist < AUTO_ARRIVE_RADIUS {
        (final_dist / AUTO_ARRIVE_RADIUS).clamp(0.22, 1.0)
    } else {
        1.0
    };
    let desired_velocity = dir * (npc.speed * slowdown);
    npc.velocity = move_towards_vec2(npc.velocity, desired_velocity, AUTO_ACCEL * 0.75 * dt);
    npc.velocity * dt
}

pub(crate) fn move_npc_axis(npc: &mut NpcWanderer, world: &World, delta: f32, is_x_axis: bool) {
    if delta.abs() <= f32::EPSILON {
        return;
    }

    if is_x_axis {
        npc.pos.x += delta;
    } else {
        npc.pos.y += delta;
    }

    let mut aabb = Aabb::from_center(npc.pos, npc.half);
    let (min_tx, max_tx, min_ty, max_ty) = tiles_overlapping_aabb(world, aabb);

    for ty in min_ty..=max_ty {
        for tx in min_tx..=max_tx {
            if !world.is_solid(tx, ty) {
                continue;
            }

            let tile = World::tile_rect(tx, ty);
            if !aabb.intersects_rect(tile) {
                continue;
            }

            if is_x_axis {
                if delta > 0.0 {
                    npc.pos.x = tile.x - npc.half.x;
                } else {
                    npc.pos.x = tile.x + tile.w + npc.half.x;
                }
            } else if delta > 0.0 {
                npc.pos.y = tile.y - npc.half.y;
            } else {
                npc.pos.y = tile.y + tile.h + npc.half.y;
            }

            aabb = Aabb::from_center(npc.pos, npc.half);
        }
    }
}

pub(crate) fn update_npc_wanderer(npc: &mut NpcWanderer, world: &World, dt: f32) {
    npc.hold_timer = (npc.hold_timer - dt).max(0.0);

    let had_active_path =
        !npc.auto.path_world.is_empty() && npc.auto.next_waypoint < npc.auto.path_world.len();
    let path_finished =
        npc.auto.path_world.is_empty() || npc.auto.next_waypoint >= npc.auto.path_world.len();
    if path_finished {
        if npc.hold_timer > 0.0 {
            npc.idle_timer = npc.idle_timer.max(0.2);
        } else if npc.idle_timer > 0.0 {
            npc.idle_timer = (npc.idle_timer - dt).max(0.0);
        } else if let Some(target_tile) = npc_choose_wander_target(npc, world) {
            if issue_npc_wander_command(npc, world, target_tile) {
                npc.idle_timer = 0.0;
            } else {
                npc.idle_timer = 0.35;
            }
        } else {
            npc.idle_timer = 0.35;
        }
    }

    let requested_delta = step_npc_auto_move(npc, dt);
    let before = npc.pos;
    move_npc_axis(npc, world, requested_delta.x, true);
    move_npc_axis(npc, world, requested_delta.y, false);
    let moved = npc.pos - before;

    let facing_source = if moved.length_squared() > 0.0001 {
        moved.normalize()
    } else if npc.velocity.length_squared() > 0.0001 {
        npc.velocity.normalize()
    } else {
        Vec2::ZERO
    };

    if facing_source.x < -0.05 {
        npc.facing_left = true;
    } else if facing_source.x > 0.05 {
        npc.facing_left = false;
    }

    npc.facing = select_character_facing(facing_source, npc.facing);
    npc.is_walking = moved.length_squared() > 0.0005;

    if npc.is_walking {
        let expected = (npc.speed * dt).max(0.001);
        let speed_scale = (moved.length() / expected).clamp(0.3, 1.2);
        npc.walk_cycle += dt * WALK_CYCLE_SPEED * speed_scale;
        if npc.walk_cycle > std::f32::consts::TAU {
            npc.walk_cycle -= std::f32::consts::TAU;
        }
        npc.anim_frame = ((npc.walk_cycle / std::f32::consts::PI) as usize) & 1;
    } else {
        npc.walk_cycle *= 0.82;
        npc.anim_frame = 0;
        npc.velocity = Vec2::ZERO;
    }

    let reached_end =
        !npc.auto.path_world.is_empty() && npc.auto.next_waypoint >= npc.auto.path_world.len();
    if reached_end {
        reset_npc_auto_move(npc);
    }

    if had_active_path && reached_end {
        npc.idle_timer = npc_rand_idle_duration(npc);
    }
}

pub(crate) fn move_player_axis(player: &mut Player, world: &World, delta: f32, is_x_axis: bool) {
    if delta.abs() <= f32::EPSILON {
        return;
    }

    if is_x_axis {
        player.pos.x += delta;
    } else {
        player.pos.y += delta;
    }

    let mut aabb = Aabb::from_center(player.pos, player.half);
    let (min_tx, max_tx, min_ty, max_ty) = tiles_overlapping_aabb(world, aabb);

    for ty in min_ty..=max_ty {
        for tx in min_tx..=max_tx {
            if !world.is_solid(tx, ty) {
                continue;
            }

            let tile = World::tile_rect(tx, ty);
            if !aabb.intersects_rect(tile) {
                continue;
            }

            if is_x_axis {
                if delta > 0.0 {
                    player.pos.x = tile.x - player.half.x;
                } else {
                    player.pos.x = tile.x + tile.w + player.half.x;
                }
            } else if delta > 0.0 {
                player.pos.y = tile.y - player.half.y;
            } else {
                player.pos.y = tile.y + tile.h + player.half.y;
            }

            aabb = Aabb::from_center(player.pos, player.half);
        }
    }
}

pub(crate) fn update_player(player: &mut Player, world: &World, input: Vec2, dt: f32) {
    let requested_delta = match player.control_mode {
        ControlMode::Manual => {
            player.velocity = input * player.speed;
            player.velocity * dt
        }
        ControlMode::AutoMove => step_auto_move(player, dt),
    };

    let before = player.pos;
    move_player_axis(player, world, requested_delta.x, true);
    move_player_axis(player, world, requested_delta.y, false);
    let moved = player.pos - before;

    let facing_source = if moved.length_squared() > 0.0001 {
        moved.normalize()
    } else if player.velocity.length_squared() > 0.0001 {
        player.velocity.normalize()
    } else {
        Vec2::ZERO
    };

    if facing_source.x < -0.05 {
        player.facing_left = true;
    } else if facing_source.x > 0.05 {
        player.facing_left = false;
    }

    player.facing = select_character_facing(facing_source, player.facing);
    player.is_walking = moved.length_squared() > 0.0005;

    if player.is_walking {
        let expected = (player.speed * dt).max(0.001);
        let speed_scale = (moved.length() / expected).clamp(0.35, 1.35);
        player.walk_cycle += dt * WALK_CYCLE_SPEED * speed_scale;
        if player.walk_cycle > std::f32::consts::TAU {
            player.walk_cycle -= std::f32::consts::TAU;
        }
        player.anim_frame = ((player.walk_cycle / std::f32::consts::PI) as usize) & 1;
    } else {
        player.walk_cycle *= 0.82;
        player.anim_frame = 0;
        if player.control_mode == ControlMode::Manual {
            player.velocity = Vec2::ZERO;
        }
    }

    if player.control_mode == ControlMode::AutoMove
        && player.auto.next_waypoint >= player.auto.path_world.len()
    {
        reset_auto_move(player);
    }
}

pub(crate) fn wall_mask_4(world: &World, x: i32, y: i32) -> u8 {
    let mut mask = 0;
    if tile_is_wall(world.get(x, y - 1)) {
        mask |= MASK_N;
    }
    if tile_is_wall(world.get(x + 1, y)) {
        mask |= MASK_E;
    }
    if tile_is_wall(world.get(x, y + 1)) {
        mask |= MASK_S;
    }
    if tile_is_wall(world.get(x - 1, y)) {
        mask |= MASK_W;
    }
    mask
}
