use super::*;
use crate::historique::LogCategorie;
use crate::interactions::SocialActionKind;

const SOCIAL_TICK_DT: f32 = 0.35;
const SOCIAL_RANGE_PX: f32 = 32.0;
const ORDER_TIMEOUT_S: f32 = 18.0;
const SOCIAL_COOLDOWN_S: f32 = 5.5;
const BUBBLE_DURATION_S: f32 = 2.2;

#[derive(Clone, Copy, Debug)]
pub struct Relation {
    pub affinity: f32, // [-1..+1]
}

impl Relation {
    pub fn new(seed: u64) -> Self {
        // Light deterministic variation.
        let r = ((seed ^ 0xA3B1_7F92_6611_0055) % 1000) as f32 / 1000.0;
        Self {
            affinity: r * 0.6 - 0.3,
        }
    }
}

#[derive(Clone, Debug)]
struct PendingOrder {
    kind: SocialActionKind,
    target: PawnKey,
    started_at_s: f64,
}

#[derive(Clone, Debug)]
struct SocialRuntime {
    cooldown_s: f32,
    bubble_s: f32,
    order: Option<PendingOrder>,
    last_job_id: Option<u64>,
}

impl Default for SocialRuntime {
    fn default() -> Self {
        Self {
            cooldown_s: 0.0,
            bubble_s: 0.0,
            order: None,
            last_job_id: None,
        }
    }
}

pub struct SocialState {
    keys: Vec<PawnKey>,
    rel: Vec<Vec<Relation>>,
    runtime: Vec<SocialRuntime>,
    tick_accum: f32,
}

pub struct SocialTickContext<'a> {
    pub world: &'a World,
    pub sim: &'a sim::FactorySim,
}

pub struct SocialTickActors<'a> {
    pub player: &'a mut Player,
    pub npc: &'a mut NpcWanderer,
    pub pawns: &'a mut [PawnCard],
}

impl SocialState {
    pub fn new(pawns: &[PawnCard], seed: u64) -> Self {
        let keys: Vec<PawnKey> = pawns.iter().map(|p| p.key).collect();
        let n = keys.len().max(1);

        let mut rel = vec![vec![Relation { affinity: 0.0 }; n]; n];
        for (i, row) in rel.iter_mut().enumerate().take(n) {
            for (j, cell) in row.iter_mut().enumerate().take(n) {
                if i == j {
                    cell.affinity = 1.0;
                } else {
                    *cell = Relation::new(seed ^ ((i as u64) << 32) ^ (j as u64));
                }
            }
        }

        let runtime = vec![SocialRuntime::default(); n];
        Self {
            keys,
            rel,
            runtime,
            tick_accum: 0.0,
        }
    }

    fn idx_of(&self, key: PawnKey) -> Option<usize> {
        self.keys.iter().position(|k| *k == key)
    }

    pub fn bubble_timer(&self, key: PawnKey) -> f32 {
        self.idx_of(key)
            .and_then(|idx| self.runtime.get(idx))
            .map(|r| r.bubble_s)
            .unwrap_or(0.0)
    }

    pub fn queue_order(
        &mut self,
        now_sim_s: f64,
        pawns: &mut [PawnCard],
        actor: PawnKey,
        target: PawnKey,
        kind: SocialActionKind,
    ) {
        if actor == target {
            return;
        }
        let Some(actor_idx) = self.idx_of(actor) else {
            return;
        };
        self.runtime[actor_idx].order = Some(PendingOrder {
            kind,
            target,
            started_at_s: now_sim_s,
        });
        push_history(
            pawns,
            actor,
            now_sim_s,
            LogCategorie::Ordre,
            format!(
                "Ordre: {} -> {}.",
                kind.ui_label(),
                pawn_name(pawns, target)
            ),
        );
    }

    pub fn tick(
        &mut self,
        dt: f32,
        now_sim_s: f64,
        context: SocialTickContext<'_>,
        actors: SocialTickActors<'_>,
    ) {
        let SocialTickContext { world, sim } = context;
        let SocialTickActors { player, npc, pawns } = actors;
        self.tick_accum += dt;

        for runtime in &mut self.runtime {
            runtime.cooldown_s = (runtime.cooldown_s - dt).max(0.0);
            runtime.bubble_s = (runtime.bubble_s - dt).max(0.0);
        }

        self.tick_sim_worker_job_history(now_sim_s, sim, pawns);

        while self.tick_accum >= SOCIAL_TICK_DT {
            self.tick_accum -= SOCIAL_TICK_DT;
            self.process_orders(now_sim_s, world, sim, player, npc, pawns);
            self.auto_social(now_sim_s, world, sim, player, npc, pawns);
        }
    }

    fn tick_sim_worker_job_history(
        &mut self,
        now_sim_s: f64,
        sim: &sim::FactorySim,
        pawns: &mut [PawnCard],
    ) {
        let Some(worker_idx) = self.idx_of(PawnKey::SimWorker) else {
            return;
        };
        let current = sim.primary_agent_current_job_id();
        if current != self.runtime[worker_idx].last_job_id {
            self.runtime[worker_idx].last_job_id = current;
            if let Some(job) = current {
                let label = sim
                    .job_brief(job)
                    .unwrap_or_else(|| "Tache en cours".to_string());
                push_history(
                    pawns,
                    PawnKey::SimWorker,
                    now_sim_s,
                    LogCategorie::Travail,
                    format!("Commence: {label}."),
                );
            } else {
                push_history(
                    pawns,
                    PawnKey::SimWorker,
                    now_sim_s,
                    LogCategorie::Travail,
                    "Termine sa tache.".to_string(),
                );
            }
        }
    }

    fn process_orders(
        &mut self,
        now_sim_s: f64,
        world: &World,
        sim: &sim::FactorySim,
        player: &mut Player,
        npc: &mut NpcWanderer,
        pawns: &mut [PawnCard],
    ) {
        for actor_idx in 0..self.keys.len() {
            let actor = self.keys[actor_idx];
            let Some(order) = self.runtime[actor_idx].order.clone() else {
                continue;
            };

            if (now_sim_s - order.started_at_s) as f32 > ORDER_TIMEOUT_S {
                self.runtime[actor_idx].order = None;
                push_history(
                    pawns,
                    actor,
                    now_sim_s,
                    LogCategorie::Ordre,
                    "Ordre expire (trop long).".to_string(),
                );
                continue;
            }

            let (Some(a_pos), Some(b_pos)) = (
                pawn_pos(sim, player, npc, actor),
                pawn_pos(sim, player, npc, order.target),
            ) else {
                continue;
            };

            if a_pos.distance(b_pos) > SOCIAL_RANGE_PX {
                let tile = tile_from_world_clamped(world, b_pos);
                issue_move_to_tile(world, actor, player, npc, tile);
                push_history(
                    pawns,
                    actor,
                    now_sim_s,
                    LogCategorie::Deplacement,
                    format!(
                        "Se rapproche de {} pour \"{}\".",
                        pawn_name(pawns, order.target),
                        order.kind.ui_label()
                    ),
                );
                continue;
            }

            self.runtime[actor_idx].order = None;
            self.runtime[actor_idx].cooldown_s = SOCIAL_COOLDOWN_S;
            self.apply_social_action(now_sim_s, pawns, actor, order.target, order.kind);
        }
    }

    fn auto_social(
        &mut self,
        now_sim_s: f64,
        world: &World,
        sim: &sim::FactorySim,
        player: &mut Player,
        npc: &mut NpcWanderer,
        pawns: &mut [PawnCard],
    ) {
        if self.keys.len() < 2 {
            return;
        }

        let a = self.keys[macroquad::rand::gen_range(0, self.keys.len() as i32) as usize];
        let b = self.keys[macroquad::rand::gen_range(0, self.keys.len() as i32) as usize];
        if a == b {
            return;
        }

        let Some(a_idx) = self.idx_of(a) else {
            return;
        };
        if self.runtime[a_idx].cooldown_s > 0.0 {
            return;
        }

        let (Some(a_pos), Some(b_pos)) =
            (pawn_pos(sim, player, npc, a), pawn_pos(sim, player, npc, b))
        else {
            return;
        };

        if a_pos.distance(b_pos) > 220.0 && macroquad::rand::gen_range(0, 100) < 80 {
            return;
        }

        let action = choose_action_for_pair(self, pawns, a, b);
        self.runtime[a_idx].cooldown_s = SOCIAL_COOLDOWN_S;

        if a_pos.distance(b_pos) > SOCIAL_RANGE_PX {
            if macroquad::rand::gen_range(0, 100) < 55 {
                let tile = tile_from_world_clamped(world, b_pos);
                issue_move_to_tile(world, a, player, npc, tile);
                push_history(
                    pawns,
                    a,
                    now_sim_s,
                    LogCategorie::Deplacement,
                    format!("Va discuter avec {}.", pawn_name(pawns, b)),
                );
            }
            return;
        }

        self.apply_social_action(now_sim_s, pawns, a, b, action);
    }

    fn apply_social_action(
        &mut self,
        now_sim_s: f64,
        pawns: &mut [PawnCard],
        a: PawnKey,
        b: PawnKey,
        kind: SocialActionKind,
    ) {
        let a_name = pawn_name(pawns, a);
        let b_name = pawn_name(pawns, b);

        let line = match kind {
            SocialActionKind::DireBonjour => format!("{a_name} salue {b_name}. \"Salut !\""),
            SocialActionKind::SmallTalk => {
                format!("{a_name} papote avec {b_name}. \"Alors, la prod ?\"")
            }
            SocialActionKind::Compliment => {
                format!("{a_name} complimente {b_name}. \"Bien joue.\"")
            }
            SocialActionKind::DemanderAide => {
                format!("{a_name} demande de l'aide a {b_name}. \"T'as 2 minutes ?\"")
            }
            SocialActionKind::Blague => format!("{a_name} lache une blague a {b_name}."),
            SocialActionKind::Ragot => format!("{a_name} raconte un ragot a {b_name}."),
            SocialActionKind::SExcuser => format!("{a_name} s'excuse aupres de {b_name}."),
            SocialActionKind::Menacer => format!("{a_name} menace {b_name}. \"Fais gaffe.\""),
            SocialActionKind::Insulter => format!("{a_name} insulte {b_name}."),
            SocialActionKind::SEngueuler => format!("{a_name} s'engueule avec {b_name} !"),
        };

        push_history(pawns, a, now_sim_s, LogCategorie::Social, line.clone());
        push_history(pawns, b, now_sim_s, LogCategorie::Social, line);

        let delta_aff = if kind.is_positive() {
            0.08
        } else if kind.is_hostile() {
            -0.14
        } else {
            0.02
        };
        self.bump_affinity(a, b, delta_aff);
        self.bump_affinity(b, a, delta_aff);

        add_need_social(pawns, a, 10);
        add_need_social(pawns, b, 10);

        if let Some(idx) = self.idx_of(a) {
            self.runtime[idx].bubble_s = BUBBLE_DURATION_S;
        }
        if let Some(idx) = self.idx_of(b) {
            self.runtime[idx].bubble_s = BUBBLE_DURATION_S;
        }
    }

    fn bump_affinity(&mut self, a: PawnKey, b: PawnKey, delta: f32) {
        let (Some(ai), Some(bi)) = (self.idx_of(a), self.idx_of(b)) else {
            return;
        };
        self.rel[ai][bi].affinity = (self.rel[ai][bi].affinity + delta).clamp(-1.0, 1.0);
    }
}

fn pawn_name(pawns: &[PawnCard], key: PawnKey) -> String {
    pawns
        .iter()
        .find(|pawn| pawn.key == key)
        .map(|pawn| pawn.name.clone())
        .unwrap_or_else(|| key.short_label().to_string())
}

fn push_history(
    pawns: &mut [PawnCard],
    key: PawnKey,
    now_sim_s: f64,
    cat: LogCategorie,
    msg: impl Into<String>,
) {
    if let Some(pawn) = pawns.iter_mut().find(|pawn| pawn.key == key) {
        pawn.history.push(now_sim_s, cat, msg);
    }
}

fn pawn_pos(
    sim: &sim::FactorySim,
    player: &Player,
    npc: &NpcWanderer,
    key: PawnKey,
) -> Option<Vec2> {
    match key {
        PawnKey::Player => Some(player.pos),
        PawnKey::Npc => Some(npc.pos),
        PawnKey::SimWorker => Some(tile_center(sim.primary_agent_tile())),
    }
}

fn issue_move_to_tile(
    world: &World,
    actor: PawnKey,
    player: &mut Player,
    npc: &mut NpcWanderer,
    tile: (i32, i32),
) {
    match actor {
        PawnKey::Player => {
            let _ = issue_auto_move_command(player, world, tile);
        }
        PawnKey::Npc => {
            let _ = issue_npc_wander_command(npc, world, tile);
        }
        PawnKey::SimWorker => {}
    }
}

fn add_need_social(pawns: &mut [PawnCard], key: PawnKey, delta: i32) {
    if let Some(pawn) = pawns.iter_mut().find(|pawn| pawn.key == key) {
        let idx = NeedBar::Social as usize;
        let value = pawn.metrics.needs[idx] as i32;
        pawn.metrics.needs[idx] = (value + delta).clamp(0, 100) as u8;
    }
}

fn choose_action_for_pair(
    state: &SocialState,
    pawns: &[PawnCard],
    a: PawnKey,
    b: PawnKey,
) -> SocialActionKind {
    let ai = state
        .idx_of(a)
        .expect("actor key should exist in social state");
    let bi = state
        .idx_of(b)
        .expect("target key should exist in social state");
    let affinity = state.rel[ai][bi].affinity;
    let social_need = pawns
        .iter()
        .find(|pawn| pawn.key == a)
        .map(|pawn| pawn.metrics.needs[NeedBar::Social as usize] as f32)
        .unwrap_or(50.0);

    if affinity < -0.45 {
        return if macroquad::rand::gen_range(0, 100) < 55 {
            SocialActionKind::SEngueuler
        } else {
            SocialActionKind::Insulter
        };
    }

    if social_need < 35.0 && affinity > 0.15 {
        return if macroquad::rand::gen_range(0, 100) < 50 {
            SocialActionKind::DemanderAide
        } else {
            SocialActionKind::SmallTalk
        };
    }

    if affinity > 0.55 {
        return if macroquad::rand::gen_range(0, 100) < 45 {
            SocialActionKind::Compliment
        } else {
            SocialActionKind::Blague
        };
    }

    if macroquad::rand::gen_range(0, 100) < 50 {
        SocialActionKind::DireBonjour
    } else {
        SocialActionKind::SmallTalk
    }
}
