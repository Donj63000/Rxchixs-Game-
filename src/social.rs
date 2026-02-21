use super::*;

use crate::historique::LogCategorie;
use crate::interactions::{SocialActionKind, SocialEmoteIcon, SocialGesture};
use std::collections::HashMap;

const SOCIAL_TICK_DT: f32 = 0.25;
const SOCIAL_RANGE_PX: f32 = 40.0;
const SOCIAL_MEET_ARRIVE_PX: f32 = 18.0;
const ORDER_TIMEOUT_S: f32 = 14.0;

const PERSONAL_COOLDOWN_S: f32 = 4.5;
const PAIR_COOLDOWN_S: f32 = 9.0;

const AFTERGLOW_DURATION_S: f32 = 2.2;
const SPEAKER_SWITCH_S: f32 = 0.55;
const MAX_ACTIVE_SEPARATION_PX: f32 = 72.0;
const REPATH_COOLDOWN_S: f32 = 0.8;

// Auto-social
const AUTO_SOCIAL_BASE_CHANCE: f32 = 0.11; // par social tick
const AUTO_SOCIAL_MAX_DIST_PX: f32 = 260.0;

pub struct SocialTickContext<'a> {
    pub world: &'a World,
    pub sim: &'a sim::FactorySim,
}

pub struct SocialTickActors<'a> {
    pub player: &'a mut Player,
    pub npc: &'a mut NpcWanderer,
    pub pawns: &'a mut [PawnCard],
}

#[derive(Clone, Copy, Debug)]
pub struct Relation {
    pub affinity: f32,
}

impl Relation {
    fn new(seed: u64) -> Self {
        // bruit initial léger, pour éviter une matrice trop plate
        let x = (seed ^ 0x9E37_79B9_7F4A_7C15).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        let n = ((x >> 33) as i32 % 11) as f32; // ~[-10..10]
        let affinity = (n / 100.0).clamp(-0.1, 0.1);
        Self { affinity }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialVisualStage {
    Approaching,
    Active,
    Afterglow,
}

#[derive(Clone, Copy, Debug)]
pub struct SocialEmoteView {
    pub icon: SocialEmoteIcon,
    pub kind: Option<SocialActionKind>,
    pub stage: SocialVisualStage,
    pub alpha: f32,
    pub phase: f32,
    pub is_speaker: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SocialAnimHint {
    pub partner: Option<PawnKey>,
    pub kind: Option<SocialActionKind>,
    pub gesture: SocialGesture,
    pub force_face_partner: bool,
    pub force_idle: bool,
}

impl Default for SocialAnimHint {
    fn default() -> Self {
        Self {
            partner: None,
            kind: None,
            gesture: SocialGesture::None,
            force_face_partner: false,
            force_idle: false,
        }
    }
}

#[derive(Clone, Debug)]
struct PendingOrder {
    kind: SocialActionKind,
    target: PawnKey,
    issued_at_s: f64,
}

#[derive(Clone, Debug)]
struct SocialRuntime {
    cooldown_s: f32,

    afterglow_s: f32,
    afterglow_icon: Option<SocialEmoteIcon>,
    afterglow_kind: Option<SocialActionKind>,

    order: Option<PendingOrder>,

    encounter_id: Option<u64>,

    repath_cooldown_s: f32,
    last_move_tile: Option<(i32, i32)>,

    last_job_id: Option<u64>,
}

impl Default for SocialRuntime {
    fn default() -> Self {
        Self {
            cooldown_s: 0.0,
            afterglow_s: 0.0,
            afterglow_icon: None,
            afterglow_kind: None,
            order: None,
            encounter_id: None,
            repath_cooldown_s: 0.0,
            last_move_tile: None,
            last_job_id: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EncounterStage {
    Approach,
    Active,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EncounterSource {
    Order,
    Auto,
    Proximity,
}

impl EncounterSource {
    fn label(self) -> &'static str {
        match self {
            EncounterSource::Order => "ordre",
            EncounterSource::Auto => "auto",
            EncounterSource::Proximity => "proximite",
        }
    }
}

#[derive(Clone, Debug)]
struct SocialEncounter {
    id: u64,
    a: PawnKey,
    b: PawnKey,

    initiator: PawnKey,
    mover: PawnKey,
    anchor: PawnKey,
    source: EncounterSource,

    kind: SocialActionKind,

    meet_tile: (i32, i32),

    stage: EncounterStage,
    created_at_s: f64,
    stage_started_at_s: f64,

    duration_s: f32,
    applied: bool,
    cancelled: bool,

    speaker: PawnKey,
    speaker_timer_s: f32,
    phase: f32,
}

pub struct SocialState {
    keys: Vec<PawnKey>,
    idx: HashMap<PawnKey, usize>,
    rel: Vec<Vec<Relation>>,
    runtime: Vec<SocialRuntime>,
    pair_cooldown: Vec<Vec<f32>>,

    encounters: Vec<SocialEncounter>,
    next_encounter_id: u64,

    tick_accum: f32,
    rng_state: u64,
}

impl SocialState {
    pub fn new(pawns: &[PawnCard], lineage_seed: u64) -> Self {
        let keys: Vec<PawnKey> = pawns.iter().map(|p| p.key).collect();
        let mut idx = HashMap::new();
        for (i, k) in keys.iter().copied().enumerate() {
            idx.insert(k, i);
        }

        let n = keys.len();
        let mut rel = vec![vec![Relation { affinity: 0.0 }; n]; n];
        for (i, row) in rel.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                if i == j {
                    *cell = Relation { affinity: 1.0 };
                } else {
                    let s = lineage_seed ^ ((i as u64) << 32) ^ (j as u64);
                    *cell = Relation::new(s);
                }
            }
        }

        let runtime = vec![SocialRuntime::default(); n];
        let pair_cooldown = vec![vec![0.0; n]; n];

        let rng_state = lineage_seed ^ 0xD1B5_4A32_D192_ED03;

        Self {
            keys,
            idx,
            rel,
            runtime,
            pair_cooldown,
            encounters: Vec::new(),
            next_encounter_id: 1,
            tick_accum: 0.0,
            rng_state,
        }
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
        let Some(ai) = self.idx_of(actor) else { return };

        self.runtime[ai].order = Some(PendingOrder {
            kind,
            target,
            issued_at_s: now_sim_s,
        });

        push_history(
            pawns,
            now_sim_s,
            LogCategorie::Social,
            format!(
                "{} ordonne: {} -> {}",
                pawn_name(pawns, actor),
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
        let world = context.world;
        let sim = context.sim;

        self.tick_accum += dt;

        for r in &mut self.runtime {
            r.cooldown_s = (r.cooldown_s - dt).max(0.0);
            r.repath_cooldown_s = (r.repath_cooldown_s - dt).max(0.0);

            if r.afterglow_s > 0.0 {
                r.afterglow_s = (r.afterglow_s - dt).max(0.0);
                if r.afterglow_s <= 0.0 {
                    r.afterglow_icon = None;
                    r.afterglow_kind = None;
                }
            }
        }

        for row in &mut self.pair_cooldown {
            for v in row {
                *v = (*v - dt).max(0.0);
            }
        }

        for e in &mut self.encounters {
            e.phase += dt;
            if e.stage == EncounterStage::Active {
                e.speaker_timer_s -= dt;
                if e.speaker_timer_s <= 0.0 {
                    e.speaker = if e.speaker == e.a { e.b } else { e.a };
                    e.speaker_timer_s += SPEAKER_SWITCH_S;
                }
            }
        }

        self.tick_sim_worker_job_history(now_sim_s, actors.pawns, sim);

        while self.tick_accum >= SOCIAL_TICK_DT {
            self.tick_accum -= SOCIAL_TICK_DT;

            self.tick_encounters(
                now_sim_s,
                world,
                sim,
                actors.player,
                actors.npc,
                actors.pawns,
            );
            self.process_orders(
                now_sim_s,
                world,
                sim,
                actors.player,
                actors.npc,
                actors.pawns,
            );
            self.auto_social(
                now_sim_s,
                world,
                sim,
                actors.player,
                actors.npc,
                actors.pawns,
            );
        }
    }

    pub fn emote_view(&self, key: PawnKey) -> Option<SocialEmoteView> {
        let i = self.idx_of(key)?;

        if let Some(id) = self.runtime[i].encounter_id
            && let Some(e) = self.encounters.iter().find(|x| x.id == id)
        {
            let (stage, icon, alpha, speaker) = match e.stage {
                EncounterStage::Approach => (
                    SocialVisualStage::Approaching,
                    SocialEmoteIcon::TalkDots,
                    1.0,
                    false,
                ),
                EncounterStage::Active => (
                    SocialVisualStage::Active,
                    e.kind.emote_icon(),
                    1.0,
                    e.speaker == key,
                ),
            };
            return Some(SocialEmoteView {
                icon,
                kind: Some(e.kind),
                stage,
                alpha,
                phase: e.phase,
                is_speaker: speaker,
            });
        }

        if self.runtime[i].afterglow_s > 0.0
            && let Some(icon) = self.runtime[i].afterglow_icon
        {
            let a = (self.runtime[i].afterglow_s / AFTERGLOW_DURATION_S).clamp(0.0, 1.0);
            return Some(SocialEmoteView {
                icon,
                kind: self.runtime[i].afterglow_kind,
                stage: SocialVisualStage::Afterglow,
                alpha: a,
                phase: 0.0,
                is_speaker: false,
            });
        }

        None
    }

    pub fn anim_hint(&self, key: PawnKey) -> SocialAnimHint {
        let Some(i) = self.idx_of(key) else {
            return SocialAnimHint::default();
        };
        let Some(id) = self.runtime[i].encounter_id else {
            return SocialAnimHint::default();
        };
        let Some(e) = self.encounters.iter().find(|x| x.id == id) else {
            return SocialAnimHint::default();
        };

        match e.stage {
            EncounterStage::Approach => {
                let partner = if key == e.a { e.b } else { e.a };
                SocialAnimHint {
                    partner: Some(partner),
                    kind: Some(e.kind),
                    gesture: SocialGesture::None,
                    force_face_partner: false,
                    force_idle: false,
                }
            }
            EncounterStage::Active => {
                let partner = if key == e.a { e.b } else { e.a };
                SocialAnimHint {
                    partner: Some(partner),
                    kind: Some(e.kind),
                    gesture: e.kind.gesture(),
                    force_face_partner: true,
                    force_idle: true,
                }
            }
        }
    }

    fn tick_encounters(
        &mut self,
        now_sim_s: f64,
        world: &World,
        sim: &sim::FactorySim,
        player: &mut Player,
        npc: &mut NpcWanderer,
        pawns: &mut [PawnCard],
    ) {
        let mut encounters = std::mem::take(&mut self.encounters);
        let mut remaining = Vec::with_capacity(encounters.len());

        for mut encounter in encounters.drain(..) {
            let done = match encounter.stage {
                EncounterStage::Approach => {
                    if (now_sim_s - encounter.created_at_s) as f32 > ORDER_TIMEOUT_S {
                        encounter.cancelled = true;
                        self.log_encounter_cancel(pawns, now_sim_s, &encounter, "timeout");
                        true
                    } else {
                        self.tick_encounter_approach(
                            now_sim_s,
                            world,
                            sim,
                            player,
                            npc,
                            pawns,
                            &mut encounter,
                        )
                    }
                }
                EncounterStage::Active => {
                    self.tick_encounter_active(now_sim_s, sim, player, npc, pawns, &mut encounter)
                }
            };

            if done {
                self.release_participant(encounter.a);
                self.release_participant(encounter.b);

                if encounter.cancelled {
                    self.start_personal_cooldown_for(encounter.a, 1.0);
                    self.start_personal_cooldown_for(encounter.b, 1.0);
                    self.start_pair_cooldown(encounter.a, encounter.b, 1.5);
                } else {
                    self.start_afterglow_for(encounter.a, encounter.kind);
                    self.start_afterglow_for(encounter.b, encounter.kind);

                    let personal_cd = if encounter.kind.is_hostile() {
                        6.0
                    } else {
                        PERSONAL_COOLDOWN_S
                    };
                    self.start_personal_cooldown_for(encounter.a, personal_cd);
                    self.start_personal_cooldown_for(encounter.b, personal_cd);

                    let pair_cd = if encounter.kind == SocialActionKind::DireBonjour {
                        18.0
                    } else if encounter.kind.is_hostile() {
                        14.0
                    } else {
                        PAIR_COOLDOWN_S
                    };
                    self.start_pair_cooldown(encounter.a, encounter.b, pair_cd);
                }

                self.hold_npc_if_involved(npc, encounter.a, 0.35);
                self.hold_npc_if_involved(npc, encounter.b, 0.35);
            } else {
                remaining.push(encounter);
            }
        }

        self.encounters = remaining;
    }

    #[allow(clippy::too_many_arguments)]
    fn tick_encounter_approach(
        &mut self,
        now_sim_s: f64,
        world: &World,
        sim: &sim::FactorySim,
        player: &mut Player,
        npc: &mut NpcWanderer,
        pawns: &mut [PawnCard],
        e: &mut SocialEncounter,
    ) -> bool {
        let a_pos = pawn_pos(sim, player, npc, e.a);
        let b_pos = pawn_pos(sim, player, npc, e.b);
        let dist_ab = a_pos.distance(b_pos);

        self.hold_npc_if_involved(npc, e.anchor, 1.0);

        if dist_ab <= SOCIAL_RANGE_PX {
            e.stage = EncounterStage::Active;
            e.stage_started_at_s = now_sim_s;
            e.duration_s = e.kind.duration_s();
            e.applied = false;
            e.speaker = e.a;
            e.speaker_timer_s = SPEAKER_SWITCH_S;

            self.stop_movement_for_social(player, npc, e.a);
            self.stop_movement_for_social(player, npc, e.b);

            if !e.applied {
                self.apply_social_action(now_sim_s, pawns, e.a, e.b, e.kind);
                e.applied = true;
            }

            return false;
        }

        let meet_world = tile_center(e.meet_tile);
        let mover_pos = pawn_pos(sim, player, npc, e.mover);
        let dist_to_meet = mover_pos.distance(meet_world);

        if dist_to_meet > SOCIAL_MEET_ARRIVE_PX {
            self.ensure_move_to_tile(world, player, npc, e.mover, e.meet_tile);
        } else {
            self.hold_npc_if_involved(npc, e.mover, 0.6);
        }

        false
    }

    fn tick_encounter_active(
        &mut self,
        now_sim_s: f64,
        sim: &sim::FactorySim,
        player: &mut Player,
        npc: &mut NpcWanderer,
        pawns: &mut [PawnCard],
        e: &mut SocialEncounter,
    ) -> bool {
        let a_pos = pawn_pos(sim, player, npc, e.a);
        let b_pos = pawn_pos(sim, player, npc, e.b);
        if a_pos.distance(b_pos) > MAX_ACTIVE_SEPARATION_PX {
            e.cancelled = true;
            self.log_encounter_cancel(pawns, now_sim_s, e, "interrompu (trop loin)");
            return true;
        }

        self.hold_npc_if_involved(npc, e.a, 0.8);
        self.hold_npc_if_involved(npc, e.b, 0.8);

        let elapsed = (now_sim_s - e.stage_started_at_s) as f32;
        if elapsed >= e.duration_s {
            push_history(
                pawns,
                now_sim_s,
                LogCategorie::Social,
                format!(
                    "{} & {}: fin ({}, source={}, initiateur={})",
                    pawn_name(pawns, e.a),
                    pawn_name(pawns, e.b),
                    e.kind.ui_label(),
                    e.source.label(),
                    pawn_name(pawns, e.initiator)
                ),
            );
            return true;
        }

        false
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
        for idx in 0..self.keys.len() {
            let actor = self.keys[idx];
            let Some(ai) = self.idx_of(actor) else {
                continue;
            };
            let Some(order) = self.runtime[ai].order.clone() else {
                continue;
            };

            if (now_sim_s - order.issued_at_s) as f32 > ORDER_TIMEOUT_S {
                self.runtime[ai].order = None;
                push_history(
                    pawns,
                    now_sim_s,
                    LogCategorie::Social,
                    format!(
                        "{}: ordre expiré ({} -> {})",
                        pawn_name(pawns, actor),
                        order.kind.ui_label(),
                        pawn_name(pawns, order.target)
                    ),
                );
                continue;
            }

            if self.runtime[ai].encounter_id.is_some() || self.runtime[ai].cooldown_s > 0.0 {
                continue;
            }

            let Some(ti) = self.idx_of(order.target) else {
                self.runtime[ai].order = None;
                continue;
            };
            if self.runtime[ti].encounter_id.is_some() {
                continue;
            }
            if self.pair_cooldown[ai][ti] > 0.0 {
                continue;
            }

            if let Some(enc) = self.build_encounter(
                now_sim_s,
                world,
                sim,
                player,
                npc,
                actor,
                order.target,
                order.kind,
                EncounterSource::Order,
            ) {
                self.runtime[ai].order = None;
                self.attach_encounter(enc);
                push_history(
                    pawns,
                    now_sim_s,
                    LogCategorie::Social,
                    format!(
                        "{} commence: {} avec {}",
                        pawn_name(pawns, actor),
                        order.kind.ui_label(),
                        pawn_name(pawns, order.target)
                    ),
                );
            }
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
        // Greeting NPC -> Player (unifié, déterministe + cooldown pair)
        if self.try_proximity_greeting(now_sim_s, world, sim, player, npc, pawns) {
            return;
        }

        if !self.roll(AUTO_SOCIAL_BASE_CHANCE) {
            return;
        }

        let mut candidates = Vec::new();
        for &k in &self.keys {
            if k == PawnKey::Player {
                continue;
            }
            if !self.pawn_available_for_social(sim, k) {
                continue;
            }
            candidates.push(k);
        }
        if candidates.is_empty() {
            return;
        }

        let initiator = candidates[self.rand_range_usize(0, candidates.len())];
        let initiator_pos = pawn_pos(sim, player, npc, initiator);

        let mut best_target: Option<(PawnKey, f32)> = None;
        for &t in &self.keys {
            if t == initiator || t == PawnKey::Player {
                continue;
            }
            if !self.pawn_available_for_social(sim, t) {
                continue;
            }

            let d = initiator_pos.distance(pawn_pos(sim, player, npc, t));
            if d > AUTO_SOCIAL_MAX_DIST_PX {
                continue;
            }

            let aff = self.affinity(initiator, t);
            let score = (1.0 + aff).max(0.05) * (1.0 / (d + 1.0));
            if best_target.map(|(_, s)| score > s).unwrap_or(true) {
                best_target = Some((t, score));
            }
        }

        let Some((target, _)) = best_target else {
            return;
        };

        let kind = self.choose_action_for_pair(pawns, initiator, target);

        let Some(ai) = self.idx_of(initiator) else {
            return;
        };
        let Some(ti) = self.idx_of(target) else {
            return;
        };
        if self.pair_cooldown[ai][ti] > 0.0 {
            return;
        }

        if let Some(enc) = self.build_encounter(
            now_sim_s,
            world,
            sim,
            player,
            npc,
            initiator,
            target,
            kind,
            EncounterSource::Auto,
        ) {
            self.attach_encounter(enc);
            push_history(
                pawns,
                now_sim_s,
                LogCategorie::Social,
                format!(
                    "Auto: {} -> {} ({})",
                    pawn_name(pawns, initiator),
                    pawn_name(pawns, target),
                    kind.ui_label()
                ),
            );
        }
    }

    fn try_proximity_greeting(
        &mut self,
        now_sim_s: f64,
        world: &World,
        sim: &sim::FactorySim,
        player: &mut Player,
        npc: &mut NpcWanderer,
        pawns: &mut [PawnCard],
    ) -> bool {
        let npc_key = PawnKey::Npc;
        let player_key = PawnKey::Player;

        let Some(ni) = self.idx_of(npc_key) else {
            return false;
        };
        let Some(pi) = self.idx_of(player_key) else {
            return false;
        };

        if self.runtime[ni].encounter_id.is_some() || self.runtime[ni].cooldown_s > 0.0 {
            return false;
        }
        if self.pair_cooldown[ni][pi] > 0.0 {
            return false;
        }

        let d = npc.pos.distance(player.pos);
        if d > SOCIAL_RANGE_PX {
            return false;
        }

        let kind = SocialActionKind::DireBonjour;
        if let Some(enc) = self.build_encounter(
            now_sim_s,
            world,
            sim,
            player,
            npc,
            npc_key,
            player_key,
            kind,
            EncounterSource::Proximity,
        ) {
            self.attach_encounter(enc);
            push_history(
                pawns,
                now_sim_s,
                LogCategorie::Social,
                format!("{}: {}", pawn_name(pawns, npc_key), kind.ui_label()),
            );
            return true;
        }

        false
    }

    #[allow(clippy::too_many_arguments)]
    fn build_encounter(
        &mut self,
        now_sim_s: f64,
        world: &World,
        sim: &sim::FactorySim,
        player: &mut Player,
        npc: &mut NpcWanderer,
        a: PawnKey,
        b: PawnKey,
        kind: SocialActionKind,
        source: EncounterSource,
    ) -> Option<SocialEncounter> {
        if a == b {
            return None;
        }
        let ai = self.idx_of(a)?;
        let bi = self.idx_of(b)?;

        if self.runtime[ai].encounter_id.is_some() || self.runtime[bi].encounter_id.is_some() {
            return None;
        }
        if self.runtime[ai].cooldown_s > 0.0 || self.runtime[bi].cooldown_s > 0.0 {
            return None;
        }
        if self.pair_cooldown[ai][bi] > 0.0 {
            return None;
        }

        let a_can_move = pawn_can_move(a);
        let b_can_move = pawn_can_move(b);

        let (mover, anchor) = if a_can_move {
            (a, b)
        } else if b_can_move {
            (b, a)
        } else {
            (a, b)
        };

        let anchor_pos = pawn_pos(sim, player, npc, anchor);
        let mut meet_tile = tile_from_world_clamped(world, anchor_pos);
        meet_tile = nearest_walkable_tile(world, meet_tile).unwrap_or(meet_tile);

        let id = self.next_encounter_id;
        self.next_encounter_id = self.next_encounter_id.wrapping_add(1);

        Some(SocialEncounter {
            id,
            a,
            b,
            initiator: a,
            mover,
            anchor,
            source,
            kind,
            meet_tile,
            stage: EncounterStage::Approach,
            created_at_s: now_sim_s,
            stage_started_at_s: now_sim_s,
            duration_s: kind.duration_s(),
            applied: false,
            cancelled: false,
            speaker: a,
            speaker_timer_s: SPEAKER_SWITCH_S,
            phase: 0.0,
        })
    }

    fn attach_encounter(&mut self, enc: SocialEncounter) {
        let id = enc.id;
        let a = enc.a;
        let b = enc.b;

        if let Some(ai) = self.idx_of(a) {
            self.runtime[ai].encounter_id = Some(id);
            self.runtime[ai].last_move_tile = None;
            self.runtime[ai].repath_cooldown_s = 0.0;
        }
        if let Some(bi) = self.idx_of(b) {
            self.runtime[bi].encounter_id = Some(id);
            self.runtime[bi].last_move_tile = None;
            self.runtime[bi].repath_cooldown_s = 0.0;
        }

        self.encounters.push(enc);
    }

    fn release_participant(&mut self, key: PawnKey) {
        if let Some(i) = self.idx_of(key) {
            self.runtime[i].encounter_id = None;
            self.runtime[i].last_move_tile = None;
        }
    }

    fn ensure_move_to_tile(
        &mut self,
        world: &World,
        player: &mut Player,
        npc: &mut NpcWanderer,
        actor: PawnKey,
        target_tile: (i32, i32),
    ) {
        let Some(i) = self.idx_of(actor) else { return };

        if self.runtime[i].repath_cooldown_s > 0.0
            && self.runtime[i].last_move_tile == Some(target_tile)
        {
            return;
        }

        self.runtime[i].repath_cooldown_s = REPATH_COOLDOWN_S;
        self.runtime[i].last_move_tile = Some(target_tile);

        issue_move_to_tile(world, player, npc, actor, target_tile);
        self.hold_npc_if_involved(npc, actor, 1.0);
    }

    fn stop_movement_for_social(
        &mut self,
        player: &mut Player,
        npc: &mut NpcWanderer,
        key: PawnKey,
    ) {
        match key {
            PawnKey::Player => {
                if player.control_mode == ControlMode::AutoMove {
                    reset_auto_move(player);
                }
                player.velocity = Vec2::ZERO;
            }
            PawnKey::Npc => {
                reset_npc_auto_move(npc);
                npc.velocity = Vec2::ZERO;
                npc.hold_timer = npc.hold_timer.max(0.8);
            }
            PawnKey::SimWorker => {}
        }
    }

    fn hold_npc_if_involved(&self, npc: &mut NpcWanderer, key: PawnKey, seconds: f32) {
        if key == PawnKey::Npc {
            npc.hold_timer = npc.hold_timer.max(seconds);
        }
    }

    fn start_personal_cooldown_for(&mut self, key: PawnKey, seconds: f32) {
        if let Some(i) = self.idx_of(key) {
            self.runtime[i].cooldown_s = self.runtime[i].cooldown_s.max(seconds);
        }
    }

    fn start_pair_cooldown(&mut self, a: PawnKey, b: PawnKey, seconds: f32) {
        let Some(ai) = self.idx_of(a) else { return };
        let Some(bi) = self.idx_of(b) else { return };
        self.pair_cooldown[ai][bi] = self.pair_cooldown[ai][bi].max(seconds);
        self.pair_cooldown[bi][ai] = self.pair_cooldown[bi][ai].max(seconds);
    }

    fn start_afterglow_for(&mut self, key: PawnKey, kind: SocialActionKind) {
        if let Some(i) = self.idx_of(key) {
            self.runtime[i].afterglow_s = AFTERGLOW_DURATION_S;
            self.runtime[i].afterglow_icon = Some(kind.emote_icon());
            self.runtime[i].afterglow_kind = Some(kind);
        }
    }

    fn pawn_available_for_social(&self, sim: &sim::FactorySim, key: PawnKey) -> bool {
        let Some(i) = self.idx_of(key) else {
            return false;
        };
        if self.runtime[i].cooldown_s > 0.0 {
            return false;
        }
        if self.runtime[i].encounter_id.is_some() {
            return false;
        }
        if key == PawnKey::SimWorker && sim.primary_agent_current_job_id().is_some() {
            return false;
        }
        true
    }

    fn choose_action_for_pair(
        &mut self,
        pawns: &[PawnCard],
        a: PawnKey,
        b: PawnKey,
    ) -> SocialActionKind {
        let aff = self.affinity(a, b);

        let social_need = get_need01(pawns, a, NeedBar::Social);
        let calm = get_need01(pawns, a, NeedBar::Calme);
        let empathy = get_trait01(pawns, a, TraitBar::Empathie);
        let patience = get_trait01(pawns, a, TraitBar::Patience);
        let sociability = get_skill01(pawns, a, SkillBar::Sociabilite);

        let lonely = (1.0 - social_need).clamp(0.0, 1.0);
        let stressed = (1.0 - calm).clamp(0.0, 1.0);
        let like = aff.clamp(-1.0, 1.0).max(0.0);
        let dislike = (-aff).clamp(0.0, 1.0);

        let mut w_hello = 0.55 + lonely * 0.45 + sociability * 0.25;
        let mut w_talk = 0.85 + lonely * 1.20 + sociability * 0.55;
        let w_compl = 0.15 + like * 1.10 + empathy * 0.35;
        let w_help = 0.30 + lonely * 0.40 + empathy * 0.20;
        let w_joke = 0.12 + like * 0.55 + sociability * 0.75;
        let mut w_gossip = 0.10 + lonely * 0.25 + (1.0 - empathy) * 0.20;
        let w_sorry = if dislike > 0.20 {
            0.08 + empathy * 0.90
        } else {
            0.02
        };

        let mut w_threat = if dislike > 0.55 {
            0.05 + (1.0 - empathy) * 0.60 + (1.0 - patience) * 0.30
        } else {
            0.0
        };
        let mut w_insult = if dislike > 0.45 {
            0.07 + (1.0 - empathy) * 0.65 + stressed * 0.35
        } else {
            0.0
        };
        let mut w_argue = if dislike > 0.30 {
            0.10 + stressed * 0.85 + (1.0 - patience) * 0.35
        } else {
            0.0
        };

        let host_dampen = (1.0 - empathy * 0.65) * (1.0 - sociability * 0.25);
        w_threat *= host_dampen;
        w_insult *= host_dampen;
        w_argue *= 1.0 - empathy * 0.35;

        if social_need > 0.75 && calm > 0.75 {
            w_talk *= 0.55;
            w_hello *= 0.65;
            w_gossip *= 0.55;
        }

        let choices: [(SocialActionKind, f32); 10] = [
            (SocialActionKind::DireBonjour, w_hello),
            (SocialActionKind::SmallTalk, w_talk),
            (SocialActionKind::Compliment, w_compl),
            (SocialActionKind::DemanderAide, w_help),
            (SocialActionKind::Blague, w_joke),
            (SocialActionKind::Ragot, w_gossip),
            (SocialActionKind::SExcuser, w_sorry),
            (SocialActionKind::Menacer, w_threat),
            (SocialActionKind::Insulter, w_insult),
            (SocialActionKind::SEngueuler, w_argue),
        ];

        self.choose_weighted(&choices)
    }

    fn apply_social_action(
        &mut self,
        now_sim_s: f64,
        pawns: &mut [PawnCard],
        a: PawnKey,
        b: PawnKey,
        kind: SocialActionKind,
    ) {
        if a == b {
            return;
        }
        let Some(ai) = self.idx_of(a) else { return };
        let Some(bi) = self.idx_of(b) else { return };

        let base_delta = match kind {
            SocialActionKind::DireBonjour => 0.04,
            SocialActionKind::SmallTalk => 0.06,
            SocialActionKind::Compliment => 0.10,
            SocialActionKind::DemanderAide => 0.05,
            SocialActionKind::Blague => 0.07,
            SocialActionKind::Ragot => -0.02,
            SocialActionKind::SExcuser => 0.09,
            SocialActionKind::Menacer => -0.15,
            SocialActionKind::Insulter => -0.18,
            SocialActionKind::SEngueuler => -0.22,
        };

        let soc = get_skill01(pawns, a, SkillBar::Sociabilite);
        let emp = get_trait01(pawns, a, TraitBar::Empathie);
        let calm = get_need01(pawns, a, NeedBar::Calme);

        let mut delta = base_delta;
        if delta >= 0.0 {
            let k = 0.80 + 0.55 * ((soc + emp) * 0.5);
            delta *= k;
        } else {
            let damp = (1.0 - emp * 0.55) * (1.0 - calm * 0.25);
            delta *= 0.80 + 0.80 * damp;
        }

        self.rel[ai][bi].affinity = (self.rel[ai][bi].affinity + delta).clamp(-1.0, 1.0);
        self.rel[bi][ai].affinity = (self.rel[bi][ai].affinity + delta).clamp(-1.0, 1.0);

        let (social_d, calm_d, moral_d) = match kind {
            SocialActionKind::DireBonjour => (8, 1, 1),
            SocialActionKind::SmallTalk => (10, 2, 1),
            SocialActionKind::Compliment => (14, 3, 2),
            SocialActionKind::DemanderAide => (8, 1, 0),
            SocialActionKind::Blague => (12, 2, 2),
            SocialActionKind::Ragot => (6, 0, -1),
            SocialActionKind::SExcuser => (10, 2, 1),
            SocialActionKind::Menacer => (-12, -8, -3),
            SocialActionKind::Insulter => (-14, -10, -4),
            SocialActionKind::SEngueuler => (-16, -12, -5),
        };

        add_need(pawns, a, NeedBar::Social, social_d);
        add_need(pawns, b, NeedBar::Social, social_d);
        add_need(pawns, a, NeedBar::Calme, calm_d);
        add_need(pawns, b, NeedBar::Calme, calm_d);
        add_synth(pawns, a, SynthBar::Moral, moral_d);
        add_synth(pawns, b, SynthBar::Moral, moral_d);

        push_history(
            pawns,
            now_sim_s,
            LogCategorie::Social,
            format!(
                "{} -> {}: {} (aff={:+.2})",
                pawn_name(pawns, a),
                pawn_name(pawns, b),
                kind.ui_label(),
                self.rel[ai][bi].affinity
            ),
        );
    }

    fn log_encounter_cancel(
        &self,
        pawns: &mut [PawnCard],
        now_sim_s: f64,
        e: &SocialEncounter,
        reason: &str,
    ) {
        push_history(
            pawns,
            now_sim_s,
            LogCategorie::Social,
            format!(
                "{} / {}: annulation {} ({}, source={}, initiateur={})",
                pawn_name(pawns, e.a),
                pawn_name(pawns, e.b),
                e.kind.ui_label(),
                reason,
                e.source.label(),
                pawn_name(pawns, e.initiator)
            ),
        );
    }

    fn affinity(&self, a: PawnKey, b: PawnKey) -> f32 {
        let Some(ai) = self.idx_of(a) else { return 0.0 };
        let Some(bi) = self.idx_of(b) else { return 0.0 };
        self.rel[ai][bi].affinity
    }

    fn idx_of(&self, key: PawnKey) -> Option<usize> {
        self.idx.get(&key).copied()
    }

    fn rng_next_u64(&mut self) -> u64 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        x
    }

    fn rand01(&mut self) -> f32 {
        let v = (self.rng_next_u64() >> 40) as u32;
        (v as f32) / (u32::MAX as f32 + 1.0)
    }

    fn roll(&mut self, chance: f32) -> bool {
        self.rand01() < chance
    }

    fn rand_range_usize(&mut self, min_incl: usize, max_excl: usize) -> usize {
        if max_excl <= min_incl + 1 {
            return min_incl;
        }
        let span = (max_excl - min_incl) as u32;
        let v = (self.rng_next_u64() as u32) % span;
        min_incl + v as usize
    }

    fn choose_weighted<const N: usize>(
        &mut self,
        choices: &[(SocialActionKind, f32); N],
    ) -> SocialActionKind {
        let mut total = 0.0;
        for (_, w) in choices {
            total += w.max(0.0);
        }
        if total <= 0.0001 {
            return SocialActionKind::SmallTalk;
        }
        let mut r = self.rand01() * total;
        for (k, w) in choices {
            let w = w.max(0.0);
            if r <= w {
                return *k;
            }
            r -= w;
        }
        choices[N - 1].0
    }

    fn tick_sim_worker_job_history(
        &mut self,
        now_sim_s: f64,
        pawns: &mut [PawnCard],
        sim: &sim::FactorySim,
    ) {
        let worker_key = PawnKey::SimWorker;
        let Some(wi) = self.idx_of(worker_key) else {
            return;
        };
        let current_job = sim.primary_agent_current_job_id();
        if self.runtime[wi].last_job_id != current_job {
            self.runtime[wi].last_job_id = current_job;
            let job_text = match current_job {
                Some(id) => sim.job_brief(id).unwrap_or_else(|| format!("Job #{}", id)),
                None => "Idle".to_string(),
            };
            push_history(
                pawns,
                now_sim_s,
                LogCategorie::Social,
                format!(
                    "{}: changement d'activité ({}) t={:.1}",
                    pawn_name(pawns, worker_key),
                    job_text,
                    now_sim_s
                ),
            );
        }
    }
}

// Helpers

fn pawn_can_move(key: PawnKey) -> bool {
    match key {
        PawnKey::Player => true,
        PawnKey::Npc => true,
        PawnKey::SimWorker => false,
    }
}

fn pawn_name(pawns: &[PawnCard], key: PawnKey) -> String {
    pawns
        .iter()
        .find(|p| p.key == key)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("{:?}", key))
}

fn push_history(pawns: &mut [PawnCard], now_sim_s: f64, cat: LogCategorie, msg: String) {
    for p in pawns {
        p.history.push(now_sim_s, cat, msg.clone());
    }
}

fn pawn_pos(sim: &sim::FactorySim, player: &Player, npc: &NpcWanderer, key: PawnKey) -> Vec2 {
    match key {
        PawnKey::Player => player.pos,
        PawnKey::Npc => npc.pos,
        PawnKey::SimWorker => tile_center(sim.primary_agent_tile()),
    }
}

fn issue_move_to_tile(
    world: &World,
    player: &mut Player,
    npc: &mut NpcWanderer,
    actor: PawnKey,
    target: (i32, i32),
) {
    match actor {
        PawnKey::Player => {
            let _ = issue_auto_move_command(player, world, target);
        }
        PawnKey::Npc => {
            let _ = issue_npc_wander_command(npc, world, target);
        }
        PawnKey::SimWorker => {}
    }
}

fn add_need(pawns: &mut [PawnCard], key: PawnKey, bar: NeedBar, delta: i32) {
    if let Some(p) = pawns.iter_mut().find(|p| p.key == key) {
        let i = bar as usize;
        let v = p.metrics.needs[i] as i32 + delta;
        p.metrics.needs[i] = v.clamp(0, 100) as u8;
    }
}

fn add_synth(pawns: &mut [PawnCard], key: PawnKey, bar: SynthBar, delta: i32) {
    if let Some(p) = pawns.iter_mut().find(|p| p.key == key) {
        let i = bar as usize;
        let v = p.metrics.synth[i] as i32 + delta;
        p.metrics.synth[i] = v.clamp(0, 100) as u8;
    }
}

fn get_need01(pawns: &[PawnCard], key: PawnKey, bar: NeedBar) -> f32 {
    pawns
        .iter()
        .find(|p| p.key == key)
        .map(|p| p.metrics.needs[bar as usize] as f32 / 100.0)
        .unwrap_or(0.5)
}

fn get_trait01(pawns: &[PawnCard], key: PawnKey, bar: TraitBar) -> f32 {
    pawns
        .iter()
        .find(|p| p.key == key)
        .map(|p| p.metrics.traits[bar as usize] as f32 / 100.0)
        .unwrap_or(0.5)
}

fn get_skill01(pawns: &[PawnCard], key: PawnKey, bar: SkillBar) -> f32 {
    pawns
        .iter()
        .find(|p| p.key == key)
        .map(|p| p.metrics.skills[bar as usize] as f32 / 100.0)
        .unwrap_or(0.5)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_pawns() -> Vec<PawnCard> {
        vec![
            PawnCard {
                key: PawnKey::Player,
                name: "Player".to_string(),
                role: "Manager".to_string(),
                metrics: PawnMetrics::seeded(10),
                history: crate::historique::HistoriqueLog::new(64),
            },
            PawnCard {
                key: PawnKey::Npc,
                name: "NPC".to_string(),
                role: "Visitor".to_string(),
                metrics: PawnMetrics::seeded(11),
                history: crate::historique::HistoriqueLog::new(64),
            },
            PawnCard {
                key: PawnKey::SimWorker,
                name: "Worker".to_string(),
                role: "Operator".to_string(),
                metrics: PawnMetrics::seeded(12),
                history: crate::historique::HistoriqueLog::new(64),
            },
        ]
    }

    fn social_fixture_same_tile() -> (
        World,
        sim::FactorySim,
        Player,
        NpcWanderer,
        Vec<PawnCard>,
        SocialState,
    ) {
        let world = World::new_room(25, 15);
        let sim = sim::FactorySim::new(sim::StarterSimConfig::default(), 25, 15);
        let player = Player::new(tile_center((6, 6)));
        let npc = NpcWanderer::new(tile_center((6, 6)), 0xABCD);
        let pawns = test_pawns();
        let social = SocialState::new(&pawns, 0x1234_5678);
        (world, sim, player, npc, pawns, social)
    }

    #[test]
    fn queue_order_ignores_self_target() {
        let (_, _, _, _, mut pawns, mut social) = social_fixture_same_tile();
        social.queue_order(
            0.0,
            &mut pawns,
            PawnKey::Player,
            PawnKey::Player,
            SocialActionKind::SmallTalk,
        );
        let pi = social
            .idx_of(PawnKey::Player)
            .expect("player key should exist");
        assert!(social.runtime[pi].order.is_none());
    }

    #[test]
    fn queued_order_expires_after_timeout() {
        let (world, sim, mut player, mut npc, mut pawns, mut social) = social_fixture_same_tile();
        social.queue_order(
            0.0,
            &mut pawns,
            PawnKey::Player,
            PawnKey::Npc,
            SocialActionKind::SmallTalk,
        );

        social.tick(
            SOCIAL_TICK_DT,
            ORDER_TIMEOUT_S as f64 + 0.5,
            SocialTickContext {
                world: &world,
                sim: &sim,
            },
            SocialTickActors {
                player: &mut player,
                npc: &mut npc,
                pawns: &mut pawns,
            },
        );

        let pi = social
            .idx_of(PawnKey::Player)
            .expect("player key should exist");
        assert!(social.runtime[pi].order.is_none());
    }

    #[test]
    fn proximity_greeting_reaches_active_then_afterglow() {
        let (world, sim, mut player, mut npc, mut pawns, mut social) = social_fixture_same_tile();

        social.tick(
            SOCIAL_TICK_DT,
            0.0,
            SocialTickContext {
                world: &world,
                sim: &sim,
            },
            SocialTickActors {
                player: &mut player,
                npc: &mut npc,
                pawns: &mut pawns,
            },
        );
        let first = social
            .emote_view(PawnKey::Npc)
            .expect("first social tick should create a greeting encounter");
        assert_eq!(first.kind, Some(SocialActionKind::DireBonjour));
        assert_eq!(first.stage, SocialVisualStage::Approaching);

        social.tick(
            SOCIAL_TICK_DT,
            SOCIAL_TICK_DT as f64,
            SocialTickContext {
                world: &world,
                sim: &sim,
            },
            SocialTickActors {
                player: &mut player,
                npc: &mut npc,
                pawns: &mut pawns,
            },
        );
        let hint = social.anim_hint(PawnKey::Npc);
        assert!(hint.force_face_partner);
        assert!(hint.force_idle);
        assert_eq!(hint.kind, Some(SocialActionKind::DireBonjour));
        assert_eq!(hint.gesture, SocialGesture::Wave);

        let mut now = SOCIAL_TICK_DT * 2.0;
        for _ in 0..6 {
            social.tick(
                SOCIAL_TICK_DT,
                now as f64,
                SocialTickContext {
                    world: &world,
                    sim: &sim,
                },
                SocialTickActors {
                    player: &mut player,
                    npc: &mut npc,
                    pawns: &mut pawns,
                },
            );
            now += SOCIAL_TICK_DT;
        }

        let final_view = social
            .emote_view(PawnKey::Npc)
            .expect("after completion the interaction should still be visible in afterglow");
        assert_eq!(final_view.stage, SocialVisualStage::Afterglow);

        let ni = social.idx_of(PawnKey::Npc).expect("npc key should exist");
        let pi = social
            .idx_of(PawnKey::Player)
            .expect("player key should exist");
        assert!(social.runtime[ni].encounter_id.is_none());
        assert!(social.pair_cooldown[ni][pi] > 0.0);
    }
}
