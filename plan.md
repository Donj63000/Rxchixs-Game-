OK, je vais faire ça proprement et “RimWorld‑like” : on garde l’esprit simulation continue (aquarium), mais on rend les interactions sociales lisibles, vivantes, débuggables, et surtout sans systèmes parallèles qui se contredisent. 😊

Je commence par l’analyse de ce qu’on a actuellement, puis je te donne les axes d’amélioration, et ensuite je te donne tout le code à copier/coller (fichier complet si petit/moyen, sinon patch précis).
Je me base aussi sur tes exigences “IA débuggable / overlay raisons” et sur le fait que les traits/compétences doivent influencer les décisions .

Actuellement, comment c’est fait (et pourquoi ça ne “vit” pas assez)

SocialState est “instantané + texte”
Dans src/social.rs aujourd’hui, une interaction est essentiellement :

un ordre (queue_order) ou une auto-interaction,

un apply_social_action qui modifie affinité + besoins,

un bubble_timer (simple durée) qui fait apparaître un texte au-dessus des têtes via ui_pawns::draw_social_bubbles.

Résultat :

Les persos ne se regardent pas forcément.

Ils ne bougent pas les mains / bouche, donc même si tu vois “Discuter”, visuellement ça ressemble à rien.

La bulle est un texte statique, sans animation (pas de “ils parlent vraiment”).

NPC greeting = système parallèle (source de bugs)
En plus du SocialState, le NPC a son propre système dans NpcWanderer :

bubble_timer, bubble_cooldown,

une logique dans update_npc_wanderer qui déclenche un “Yo!” près du joueur,

et le rendu dans draw_npc dans rendu.rs.

Ça crée 2 problèmes :

deux systèmes de bulles (social vs greeting) → incohérences, “spam”, et code plus fragile,

ça ne s’intègre pas au reste (relations, besoins, cooldowns pairs, etc.).

Pas de “state machine” social (approche → échange → fin)
Sans phases, tu n’as pas :

un moment où un perso “approche” l’autre,

un moment où ils se mettent face à face et “parlent”,

une fin claire (et donc un feedback visuel propre + cooldowns).

Debuggabilité limitée
Tu veux pouvoir comprendre ce qu’ils font et pourquoi (overlay)

Jobs-–-Système-&-Priorités

.
Aujourd’hui, c’est surtout de l’historique texte + le timer.

Axes d’amélioration (ce que je mets en place)

Axe A — Unifier TOUTES les interactions sociales dans un seul moteur

Le greeting NPC devient une interaction sociale normale (DireBonjour) (plus de bubble_timer dans le NPC).

Ordres du joueur + interactions autonomes + greeting = même pipeline, mêmes cooldowns, mêmes logs.

Axe B — Social = machine à états (Approach → Active → Afterglow)

Approach : quelqu’un se déplace vers l’autre (ou l’inverse si le “moteur” ne peut pas bouger).

Active : ils se mettent immobiles + face à face + gestuelle + bulle animée.

Afterglow : une bulle qui reste un peu (fade), pour que ce soit lisible même si tu regardes ailleurs.

Axe C — Lisibilité visuelle : pictos + animation “ils parlent”

Bulles avec icônes (cœur, ?, colère, éclair…) + mini texte très court optionnel.

Alternance “speaker” (un parle, l’autre écoute) via des dots animés / légère pulsation.

Ajout d’une gestuelle dans draw_character (bras + bouche) via un champ gesture.

Axe D — Pas de bugs / pas de spam

Repath throttling (anti “recalcul path” en boucle).

Timeout d’ordre (si la rencontre n’arrive pas).

Cancel si ils s’éloignent trop pendant l’échange.

Cooldown perso + cooldown pair (et plus long pour “bonjour” / hostile).

NPC “hold_timer” pour empêcher le wander pendant une interaction (sinon il repart et casse tout).

Axe E — Hook debug (prêt pour overlay)

SocialState::anim_hint() et SocialState::emote_view() donnent des infos propres au rendu.

Tu peux facilement rajouter ensuite un overlay “raison du choix” (tu le veux à terme

Jobs-–-Système-&-Priorités

).

Maintenant le concret : code à copier/coller

Je te donne d’abord 2 fichiers complets (petits/moyens), puis des patches précis pour les gros fichiers.

===========================================================

Remplace ENTIEREMENT src/interactions.rs
===========================================================

// src/interactions.rs

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialActionKind {
DireBonjour,
SmallTalk,
Compliment,
DemanderAide,
Blague,
Ragot,
SExcuser,
Menacer,
Insulter,
SEngueuler,
}

/// Petit “langage visuel” au-dessus des têtes.
/// (On dessine des pictos simples côté rendu.)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialEmoteIcon {
TalkDots,
Heart,
Question,
Laugh,
Apology,
Exclamation,
Anger,
Scribble,
Lightning,
}

/// Intention de gestuelle (mappée côté rendu).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocialGesture {
None,
Talk,
Wave,
Explain,
Laugh,
Apologize,
Threaten,
Argue,
}

impl SocialActionKind {
pub const ALL: [SocialActionKind; 10] = [
SocialActionKind::DireBonjour,
SocialActionKind::SmallTalk,
SocialActionKind::Compliment,
SocialActionKind::DemanderAide,
SocialActionKind::Blague,
SocialActionKind::Ragot,
SocialActionKind::SExcuser,
SocialActionKind::Menacer,
SocialActionKind::Insulter,
SocialActionKind::SEngueuler,
];

    /// Liste utilisée par le menu contextuel.
    pub const MENU_DEFAULT: [SocialActionKind; 10] = [
        SocialActionKind::DireBonjour,
        SocialActionKind::SmallTalk,
        SocialActionKind::Compliment,
        SocialActionKind::DemanderAide,
        SocialActionKind::Blague,
        SocialActionKind::Ragot,
        SocialActionKind::SExcuser,
        SocialActionKind::Insulter,
        SocialActionKind::SEngueuler,
        SocialActionKind::Menacer,
    ];

    pub fn ui_label(self) -> &'static str {
        match self {
            SocialActionKind::DireBonjour => "Dire bonjour",
            SocialActionKind::SmallTalk => "Discuter",
            SocialActionKind::Compliment => "Compliment",
            SocialActionKind::DemanderAide => "Demander aide",
            SocialActionKind::Blague => "Faire une blague",
            SocialActionKind::Ragot => "Ragot",
            SocialActionKind::SExcuser => "S'excuser",
            SocialActionKind::Menacer => "Menacer",
            SocialActionKind::Insulter => "Insulter",
            SocialActionKind::SEngueuler => "S'engueuler",
        }
    }

    pub fn is_positive(self) -> bool {
        matches!(
            self,
            SocialActionKind::DireBonjour
                | SocialActionKind::SmallTalk
                | SocialActionKind::Compliment
                | SocialActionKind::DemanderAide
                | SocialActionKind::Blague
                | SocialActionKind::SExcuser
        )
    }

    pub fn is_hostile(self) -> bool {
        matches!(
            self,
            SocialActionKind::Menacer | SocialActionKind::Insulter | SocialActionKind::SEngueuler
        )
    }

    /// Durée (en secondes) de l’animation/interaction.
    pub fn duration_s(self) -> f32 {
        match self {
            SocialActionKind::DireBonjour => 0.9,
            SocialActionKind::SmallTalk => 2.8,
            SocialActionKind::Compliment => 2.0,
            SocialActionKind::DemanderAide => 2.2,
            SocialActionKind::Blague => 2.4,
            SocialActionKind::Ragot => 2.6,
            SocialActionKind::SExcuser => 2.2,
            SocialActionKind::Menacer => 1.6,
            SocialActionKind::Insulter => 1.7,
            SocialActionKind::SEngueuler => 3.2,
        }
    }

    pub fn emote_icon(self) -> SocialEmoteIcon {
        match self {
            SocialActionKind::DireBonjour => SocialEmoteIcon::TalkDots,
            SocialActionKind::SmallTalk => SocialEmoteIcon::TalkDots,
            SocialActionKind::Compliment => SocialEmoteIcon::Heart,
            SocialActionKind::DemanderAide => SocialEmoteIcon::Question,
            SocialActionKind::Blague => SocialEmoteIcon::Laugh,
            SocialActionKind::Ragot => SocialEmoteIcon::Scribble,
            SocialActionKind::SExcuser => SocialEmoteIcon::Apology,
            SocialActionKind::Menacer => SocialEmoteIcon::Exclamation,
            SocialActionKind::Insulter => SocialEmoteIcon::Anger,
            SocialActionKind::SEngueuler => SocialEmoteIcon::Lightning,
        }
    }

    pub fn gesture(self) -> SocialGesture {
        match self {
            SocialActionKind::DireBonjour => SocialGesture::Wave,
            SocialActionKind::SmallTalk => SocialGesture::Talk,
            SocialActionKind::Compliment => SocialGesture::Talk,
            SocialActionKind::DemanderAide => SocialGesture::Explain,
            SocialActionKind::Blague => SocialGesture::Laugh,
            SocialActionKind::Ragot => SocialGesture::Talk,
            SocialActionKind::SExcuser => SocialGesture::Apologize,
            SocialActionKind::Menacer => SocialGesture::Threaten,
            SocialActionKind::Insulter => SocialGesture::Argue,
            SocialActionKind::SEngueuler => SocialGesture::Argue,
        }
    }
}
===========================================================
2) Remplace ENTIEREMENT src/social.rs
   // src/social.rs

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
        for i in 0..n {
            for j in 0..n {
                if i == j {
                    rel[i][j] = Relation { affinity: 1.0 };
                } else {
                    let s = lineage_seed ^ ((i as u64) << 32) ^ (j as u64);
                    rel[i][j] = Relation::new(s);
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

            self.tick_encounters(now_sim_s, world, sim, actors.player, actors.npc, actors.pawns);
            self.process_orders(now_sim_s, world, sim, actors.player, actors.npc, actors.pawns);
            self.auto_social(now_sim_s, world, sim, actors.player, actors.npc, actors.pawns);
        }
    }

    pub fn emote_view(&self, key: PawnKey) -> Option<SocialEmoteView> {
        let i = self.idx_of(key)?;

        if let Some(id) = self.runtime[i].encounter_id {
            if let Some(e) = self.encounters.iter().find(|x| x.id == id) {
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
        }

        if self.runtime[i].afterglow_s > 0.0 {
            if let Some(icon) = self.runtime[i].afterglow_icon {
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
        }

        None
    }

    pub fn anim_hint(&self, key: PawnKey) -> SocialAnimHint {
        let Some(i) = self.idx_of(key) else { return SocialAnimHint::default() };
        let Some(id) = self.runtime[i].encounter_id else { return SocialAnimHint::default() };
        let Some(e) = self.encounters.iter().find(|x| x.id == id) else { return SocialAnimHint::default() };

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
        let mut i = 0;
        while i < self.encounters.len() {
            let done = {
                let e = &mut self.encounters[i];
                match e.stage {
                    EncounterStage::Approach => {
                        if (now_sim_s - e.created_at_s) as f32 > ORDER_TIMEOUT_S {
                            e.cancelled = true;
                            self.log_encounter_cancel(pawns, e, "timeout");
                            true
                        } else {
                            self.tick_encounter_approach(now_sim_s, world, sim, player, npc, pawns, e)
                        }
                    }
                    EncounterStage::Active => self.tick_encounter_active(now_sim_s, sim, player, npc, pawns, e),
                }
            };

            if done {
                let removed = self.encounters.swap_remove(i);
                self.release_participant(removed.a);
                self.release_participant(removed.b);

                if removed.cancelled {
                    self.start_personal_cooldown_for(removed.a, 1.0);
                    self.start_personal_cooldown_for(removed.b, 1.0);
                    self.start_pair_cooldown(removed.a, removed.b, 1.5);
                } else {
                    self.start_afterglow_for(removed.a, removed.kind);
                    self.start_afterglow_for(removed.b, removed.kind);

                    let personal_cd = if removed.kind.is_hostile() { 6.0 } else { PERSONAL_COOLDOWN_S };
                    self.start_personal_cooldown_for(removed.a, personal_cd);
                    self.start_personal_cooldown_for(removed.b, personal_cd);

                    let pair_cd = if removed.kind == SocialActionKind::DireBonjour {
                        18.0
                    } else if removed.kind.is_hostile() {
                        14.0
                    } else {
                        PAIR_COOLDOWN_S
                    };
                    self.start_pair_cooldown(removed.a, removed.b, pair_cd);
                }

                self.hold_npc_if_involved(npc, removed.a, 0.35);
                self.hold_npc_if_involved(npc, removed.b, 0.35);

                continue;
            }

            i += 1;
        }
    }

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
            self.log_encounter_cancel(pawns, e, "interrompu (trop loin)");
            return true;
        }

        self.hold_npc_if_involved(npc, e.a, 0.8);
        self.hold_npc_if_involved(npc, e.b, 0.8);

        let elapsed = (now_sim_s - e.stage_started_at_s) as f32;
        if elapsed >= e.duration_s {
            push_history(
                pawns,
                LogCategorie::Social,
                format!(
                    "{} & {}: fin ({})",
                    pawn_name(pawns, e.a),
                    pawn_name(pawns, e.b),
                    e.kind.ui_label()
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
            let Some(ai) = self.idx_of(actor) else { continue };
            let Some(order) = self.runtime[ai].order.clone() else { continue };

            if (now_sim_s - order.issued_at_s) as f32 > ORDER_TIMEOUT_S {
                self.runtime[ai].order = None;
                push_history(
                    pawns,
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

        let Some((target, _)) = best_target else { return };

        let kind = self.choose_action_for_pair(pawns, initiator, target);

        let Some(ai) = self.idx_of(initiator) else { return };
        let Some(ti) = self.idx_of(target) else { return };
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

        let Some(ni) = self.idx_of(npc_key) else { return false };
        let Some(pi) = self.idx_of(player_key) else { return false };

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
                LogCategorie::Social,
                format!("{}: {}", pawn_name(pawns, npc_key), kind.ui_label()),
            );
            return true;
        }

        false
    }

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
        meet_tile = nearest_walkable_tile(world, meet_tile.0, meet_tile.1);

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

        if self.runtime[i].repath_cooldown_s > 0.0 {
            if self.runtime[i].last_move_tile == Some(target_tile) {
                return;
            }
        }

        self.runtime[i].repath_cooldown_s = REPATH_COOLDOWN_S;
        self.runtime[i].last_move_tile = Some(target_tile);

        issue_move_to_tile(world, player, npc, actor, target_tile);
        self.hold_npc_if_involved(npc, actor, 1.0);
    }

    fn stop_movement_for_social(&mut self, player: &mut Player, npc: &mut NpcWanderer, key: PawnKey) {
        match key {
            PawnKey::Player => {
                if player.control_mode == PlayerControlMode::AutoMove {
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
        let Some(i) = self.idx_of(key) else { return false };
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

    fn choose_action_for_pair(&mut self, pawns: &[PawnCard], a: PawnKey, b: PawnKey) -> SocialActionKind {
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
        let mut w_compl = 0.15 + like * 1.10 + empathy * 0.35;
        let mut w_help = 0.30 + lonely * 0.40 + empathy * 0.20;
        let mut w_joke = 0.12 + like * 0.55 + sociability * 0.75;
        let mut w_gossip = 0.10 + lonely * 0.25 + (1.0 - empathy) * 0.20;
        let mut w_sorry = if dislike > 0.20 { 0.08 + empathy * 0.90 } else { 0.02 };

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
        _now_sim_s: f64,
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

    fn log_encounter_cancel(&self, pawns: &mut [PawnCard], e: &SocialEncounter, reason: &str) {
        push_history(
            pawns,
            LogCategorie::Social,
            format!(
                "{} / {}: annulation {} ({})",
                pawn_name(pawns, e.a),
                pawn_name(pawns, e.b),
                e.kind.ui_label(),
                reason
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

    fn choose_weighted<const N: usize>(&mut self, choices: &[(SocialActionKind, f32); N]) -> SocialActionKind {
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

    fn tick_sim_worker_job_history(&mut self, now_sim_s: f64, pawns: &mut [PawnCard], sim: &sim::FactorySim) {
        let worker_key = PawnKey::SimWorker;
        let Some(wi) = self.idx_of(worker_key) else { return };
        let current_job = sim.primary_agent_current_job_id();
        if self.runtime[wi].last_job_id != current_job {
            self.runtime[wi].last_job_id = current_job;
            let job_text = match current_job {
                Some(id) => format!("Job #{}", id),
                None => "Idle".to_string(),
            };
            push_history(
                pawns,
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
.map(|p| p.display_name.clone())
.unwrap_or_else(|| format!("{:?}", key))
}

fn push_history(pawns: &mut [PawnCard], cat: LogCategorie, msg: String) {
for p in pawns {
p.metrics.history.push(cat, msg.clone());
}
}

fn pawn_pos(sim: &sim::FactorySim, player: &Player, npc: &NpcWanderer, key: PawnKey) -> Vec2 {
match key {
PawnKey::Player => player.pos,
PawnKey::Npc => npc.pos,
PawnKey::SimWorker => tile_center(sim.primary_agent_tile()),
}
}

fn issue_move_to_tile(world: &World, player: &mut Player, npc: &mut NpcWanderer, actor: PawnKey, target: (i32, i32)) {
match actor {
PawnKey::Player => issue_auto_move_command(world, player, target.0, target.1),
PawnKey::Npc => issue_npc_wander_command(world, npc, target.0, target.1),
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
===========================================================
3) Patch src/main.rs (NPC bubble supprimé + hold_timer)

A) En haut de main.rs, modifie l’import character
Cherche :

use character::{
CharacterCatalog, CharacterFacing, CharacterRecord, CharacterRenderParams, build_lineage_preview,
compact_visual_summary, draw_character, inspector_lines,
};

Remplace par :

use character::{
CharacterCatalog, CharacterFacing, CharacterGesture, CharacterRecord, CharacterRenderParams,
build_lineage_preview, compact_visual_summary, draw_character, inspector_lines,
};

B) Supprime les constantes greeting NPC (plus utilisées)
Supprime :

const NPC_GREETING_RADIUS: f32 = 26.0;
const NPC_GREETING_DURATION: f32 = 1.25;
const NPC_GREETING_COOLDOWN: f32 = 3.8;

C) Modifie la struct NpcWanderer (retirer bubble_timer/bubble_cooldown, ajouter hold_timer)
Dans main.rs, trouve :

struct NpcWanderer {
pos: Vec2,
half: Vec2,
speed: f32,
facing: CharacterFacing,
facing_left: bool,
velocity: Vec2,
is_walking: bool,
anim_frame: usize,
walk_cycle: f32,
auto: AutoMoveState,
idle_timer: f32,
bubble_timer: f32,
bubble_cooldown: f32,
rng_state: u64,
}

Remplace par :

struct NpcWanderer {
pos: Vec2,
half: Vec2,
speed: f32,
facing: CharacterFacing,
facing_left: bool,
velocity: Vec2,
is_walking: bool,
anim_frame: usize,
walk_cycle: f32,
auto: AutoMoveState,
idle_timer: f32,

    // Empêche le wander de relancer une destination pendant une interaction sociale
    hold_timer: f32,

    rng_state: u64,
}

D) Mets à jour NpcWanderer::new
Trouve :

bubble_timer: 0.0,
bubble_cooldown: 0.0,

Supprime-les et ajoute :

hold_timer: 0.0,

E) Mets à jour les tests (dans mod tests en bas)
Remplace complètement le test :

#[test]
fn npc_greeting_triggers_when_player_is_close() { ... }

par :

#[test]
fn social_proximity_greeting_creates_emote() {
let world = World::new_room(25, 15);
let sim = sim::FactorySim::new(sim::StarterSimConfig::default(), 25, 15);

    let mut player = Player::new(tile_center((6, 6)));
    let mut npc = NpcWanderer::new(tile_center((6, 6)), 99);

    let mut pawns = vec![
        PawnCard {
            key: PawnKey::Player,
            display_name: "Player".to_string(),
            metrics: PawnMetrics::seeded(1),
        },
        PawnCard {
            key: PawnKey::Npc,
            display_name: "NPC".to_string(),
            metrics: PawnMetrics::seeded(2),
        },
        PawnCard {
            key: PawnKey::SimWorker,
            display_name: "Worker".to_string(),
            metrics: PawnMetrics::seeded(3),
        },
    ];

    let mut social_state = social::SocialState::new(&pawns, 0x1234);

    // Un tick social suffit à déclencher la rencontre "bonjour" (Approach)
    social_state.tick(
        0.25,
        0.0,
        social::SocialTickContext { world: &world, sim: &sim },
        social::SocialTickActors { player: &mut player, npc: &mut npc, pawns: &mut pawns },
    );

    let view = social_state.emote_view(PawnKey::Npc);
    assert!(view.is_some());
    assert_eq!(view.unwrap().kind, Some(SocialActionKind::DireBonjour));
}

Et mets à jour les 2 autres tests qui appellent update_npc_wanderer : on change la signature (voir section deplacement ci-dessous).

===========================================================
4) Patch src/deplacement.rs (update_npc_wanderer)

A) Change la signature
Cherche :

pub(crate) fn update_npc_wanderer(npc: &mut NpcWanderer, world: &World, player: &Player, dt: f32) {

Remplace par :

pub(crate) fn update_npc_wanderer(npc: &mut NpcWanderer, world: &World, dt: f32) {

B) Au début de la fonction, retire le bloc greeting + timers bubble et ajoute hold_timer
Dans update_npc_wanderer, supprime tout ce bloc (il commence par) :

npc.bubble_timer = (npc.bubble_timer - dt).max(0.0);
npc.bubble_cooldown = (npc.bubble_cooldown - dt).max(0.0);
// greeting...

Et à la place, mets :

npc.hold_timer = (npc.hold_timer - dt).max(0.0);

C) Empêche le wander de repartir quand hold_timer > 0
Dans la partie :

if path_finished {
if npc.idle_timer > 0.0 {
npc.idle_timer -= dt;
} else {
// choose wander target ...

Modifie ainsi :

if path_finished {
// Si on est “hold” (social), on ne relance pas de wander target
if npc.hold_timer > 0.0 {
npc.idle_timer = npc.idle_timer.max(0.2);
} else if npc.idle_timer > 0.0 {
npc.idle_timer -= dt;
} else {
// choose wander target ...
===========================================================
5) Patch src/rendu.rs (retirer le rendu “Yo!” + gesture param)

A) Retire le bloc “Yo!”
Dans draw_npc, supprime :

// Draw greeting bubble
if npc.bubble_timer > 0.0 {
...
}

B) Ajoute gesture: CharacterGesture::None dans chaque CharacterRenderParams { ... }
Exemples à modifier :

Dans show_lineage_inspector (en haut de rendu.rs), ajoute :

gesture: CharacterGesture::None,

Dans draw_player et draw_npc, ajoute :

gesture: CharacterGesture::None,
===========================================================
6) Patch src/character.rs (gestuelle + bouche qui bouge)

A) Ajoute l’enum CharacterGesture
Après pub enum CharacterFacing { ... }, colle :

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterGesture {
None,
Talk,
Wave,
Explain,
Laugh,
Apologize,
Threaten,
Argue,
}

B) Ajoute le champ dans CharacterRenderParams
Trouve :

pub struct CharacterRenderParams {
pub center: Vec2,
pub half: Vec2,
pub time: f32,
pub scale: f32,
pub facing: CharacterFacing,
pub facing_left: bool,
pub is_walking: bool,
pub walk_cycle: f32,
}

Remplace par :

pub struct CharacterRenderParams {
pub center: Vec2,
pub half: Vec2,
pub time: f32,
pub scale: f32,
pub facing: CharacterFacing,
pub facing_left: bool,
pub is_walking: bool,
pub walk_cycle: f32,

    pub gesture: CharacterGesture,
}

C) Patch de draw_character (ajout animation bras + bouche)
Dans draw_character, juste après :

let stride = if is_walking {
(walk_cycle * 2.0 * std::f32::consts::PI).sin()
} else {
0.0
};
let idle_wave = if is_walking {
0.0
} else {
(params.time * 1.2 + seed_phase * 1.7).sin() * 0.35
};
let bob = stride * 0.24 + idle_wave;

Remplace le calcul de bob par celui-ci :

let stride = if is_walking {
(walk_cycle * 2.0 * std::f32::consts::PI).sin()
} else {
0.0
};

let idle_wave = if is_walking {
0.0
} else {
(params.time * 1.2 + seed_phase * 1.7).sin() * 0.35
};

let gesture = params.gesture;
let g_phase = (params.time * 7.5 + seed_phase * 3.1).sin();
let g_phase2 = (params.time * 6.2 + seed_phase * 2.3).cos();
let g_power = match gesture {
CharacterGesture::None => 0.0,
CharacterGesture::Talk => 0.7,
CharacterGesture::Explain => 0.9,
CharacterGesture::Laugh => 0.8,
CharacterGesture::Apologize => 0.6,
CharacterGesture::Wave => 1.5,
CharacterGesture::Threaten => 1.0,
CharacterGesture::Argue => 1.2,
};
let g_anim = if is_walking { 0.0 } else { g_phase * g_power };

let bob = stride * 0.24 + idle_wave + g_anim * 0.35;

Ensuite, dans le bloc CharacterFacing::Side | CharacterFacing::Back (là où tu as near_arm_y et far_arm_y), remplace :

let far_arm_y = arms_y - stride * 1.7 * stride_sign;
let near_arm_y = arms_y + stride * 2.2 * stride_sign;

par :

let (near_extra, far_extra) = if !is_walking {
match gesture {
CharacterGesture::Talk => (g_anim * 1.6, -g_anim * 0.6),
CharacterGesture::Explain => (g_anim.abs() * 1.2, g_anim * 0.8),
CharacterGesture::Wave => (g_anim.abs() * 4.0, 0.0),
CharacterGesture::Apologize => (-g_anim.abs() * 1.4, -g_anim.abs() * 0.5),
CharacterGesture::Threaten => (g_anim.abs() * 2.8, g_anim.abs() * 1.6),
CharacterGesture::Argue => (g_anim * 2.5, -g_anim * 2.0),
CharacterGesture::Laugh => (g_anim.abs() * 1.8, g_anim * 0.4),
CharacterGesture::None => (0.0, 0.0),
}
} else {
(0.0, 0.0)
};

// y vers le bas => soustraire = lever le bras
let far_arm_y = arms_y - stride * 1.7 * stride_sign - far_extra;
let near_arm_y = arms_y + stride * 2.2 * stride_sign - near_extra;

Dans le bloc CharacterFacing::Front (là où tu as let arms_y = ... et left_arm/right_arm), remplace la création des rectangles d’armes par une version avec offsets :

Cherche :

let arms_y = 14.0 + bob;
let left_arm = xform.rect(
16.0 - metrics.shoulder_w * 0.5 - 2.4,
arms_y,
2.4,
7.0,
);
let right_arm = xform.rect(
16.0 + metrics.shoulder_w * 0.5,
arms_y,
2.4,
7.0,
);

Remplace par :

let arms_y = 14.0 + bob;

let (left_extra, right_extra) = if !is_walking {
match gesture {
CharacterGesture::Talk => (-g_anim * 1.4, g_anim * 1.1),
CharacterGesture::Explain => (g_anim.abs() * 1.1, g_anim * 0.6),
CharacterGesture::Wave => (0.0, g_anim.abs() * 3.8),
CharacterGesture::Apologize => (-g_anim.abs() * 1.2, -g_anim.abs() * 0.8),
CharacterGesture::Threaten => (g_anim.abs() * 2.2, g_anim.abs() * 2.4),
CharacterGesture::Argue => (-g_anim * 2.0, g_anim * 2.0),
CharacterGesture::Laugh => (g_anim.abs() * 1.3, g_anim * 0.5),
CharacterGesture::None => (0.0, 0.0),
}
} else {
(0.0, 0.0)
};

let left_arm = xform.rect(
16.0 - metrics.shoulder_w * 0.5 - 2.4,
arms_y - left_extra,
2.4,
7.0,
);
let right_arm = xform.rect(
16.0 + metrics.shoulder_w * 0.5,
arms_y - right_extra,
2.4,
7.0,
);

D) Animation de bouche (Front)
Dans la partie :

draw_line(
head_center.x - 1.8 * scale,
head_center.y + 3.0 * scale,
head_center.x + 1.8 * scale,
head_center.y + 3.0 * scale,
1.0 * scale,
shade(skin, 0.62),
);

Remplace par :

let mouth_col = shade(skin, 0.62);
let talking = !is_walking
&& matches!(
gesture,
CharacterGesture::Talk
| CharacterGesture::Explain
| CharacterGesture::Laugh
| CharacterGesture::Apologize
| CharacterGesture::Threaten
| CharacterGesture::Argue
);

if talking {
let m = (params.time * 10.0 + seed_phase * 4.7).sin();
if matches!(gesture, CharacterGesture::Laugh) {
let amp = 0.9 + 0.8 * m.abs();
draw_line(
head_center.x - 1.6 * scale,
head_center.y + 3.0 * scale,
head_center.x,
head_center.y + (3.0 + amp) * scale,
1.0 * scale,
mouth_col,
);
draw_line(
head_center.x,
head_center.y + (3.0 + amp) * scale,
head_center.x + 1.6 * scale,
head_center.y + 3.0 * scale,
1.0 * scale,
mouth_col,
);
} else if m > 0.0 {
draw_circle(
head_center.x,
head_center.y + 3.2 * scale,
1.05 * scale,
mouth_col,
);
} else {
draw_line(
head_center.x - 1.8 * scale,
head_center.y + 3.0 * scale,
head_center.x + 1.8 * scale,
head_center.y + 3.0 * scale,
1.0 * scale,
mouth_col,
);
}
} else {
draw_line(
head_center.x - 1.8 * scale,
head_center.y + 3.0 * scale,
head_center.x + 1.8 * scale,
head_center.y + 3.0 * scale,
1.0 * scale,
mouth_col,
);
}
===========================================================
7) Patch src/ui_pawns.rs (bulles sociales animées + pictos)

A) Modifie imports en haut
Trouve :

use crate::interactions::SocialActionKind;
use crate::social::{SocialTickActors, SocialTickContext};

Remplace par :

use crate::interactions::{SocialActionKind, SocialEmoteIcon};
use crate::social::{SocialEmoteView, SocialTickActors, SocialTickContext, SocialVisualStage};

B) Remplace ENTIEREMENT draw_social_bubbles par draw_social_emotes
Dans ui_pawns.rs, trouve la fonction draw_social_bubbles et remplace-la par ce bloc complet :

pub fn draw_social_emotes(state: &GameState, time: f32) {
for pawn in &state.pawns {
let key = pawn.key;
let Some(view) = state.social_state.emote_view(key) else { continue };
let Some(pos) = pawn_world_pos(state, key) else { continue };

        draw_social_emote_bubble(pos, &view, time, state.debug);
    }
}

fn draw_social_emote_bubble(pos: Vec2, view: &SocialEmoteView, time: f32, debug: bool) {
let base = pos + vec2(0.0, -42.0);

    let wobble = 1.0 + 0.03 * (time * 4.0 + view.phase * 1.7).sin();
    let speaker_boost = if view.is_speaker && view.stage == SocialVisualStage::Active {
        1.06
    } else {
        1.0
    };
    let scale = wobble * speaker_boost;

    let w = 46.0 * scale;
    let h = 26.0 * scale;

    let x = base.x - w * 0.5;
    let y = base.y - h;

    let bg = Color::new(1.0, 1.0, 1.0, 0.85 * view.alpha);
    let border = Color::new(0.05, 0.05, 0.05, 0.70 * view.alpha);

    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 1.0, border);

    // petite “queue” vers la tête
    let tip = vec2(pos.x, pos.y - 16.0);
    let a = vec2(base.x - 6.0 * scale, y + h);
    let b = vec2(base.x + 6.0 * scale, y + h);
    draw_triangle(a, b, tip, bg);
    draw_triangle_lines(a, b, tip, 1.0, border);

    // icône au centre
    let icon_center = vec2(base.x, y + h * 0.52);
    draw_emote_icon(icon_center, view, scale, time);

    // label court (optionnel)
    if debug {
        if let Some(kind) = view.kind {
            draw_text_centered(kind.ui_label(), base.x, y - 4.0, 14.0, border);
        }
    } else if view.stage == SocialVisualStage::Active {
        if let Some(kind) = view.kind {
            let label = short_label(kind);
            if !label.is_empty() {
                draw_text_centered(label, base.x, y + h - 2.0, 14.0, Color::new(0.08, 0.08, 0.08, 0.75 * view.alpha));
            }
        }
    }
}

fn short_label(kind: SocialActionKind) -> &'static str {
match kind {
SocialActionKind::DireBonjour => "Yo",
SocialActionKind::SmallTalk => "...",
SocialActionKind::Compliment => "Top",
SocialActionKind::DemanderAide => "Aide?",
SocialActionKind::Blague => "Ha",
SocialActionKind::Ragot => "Psst",
SocialActionKind::SExcuser => "Pardon",
SocialActionKind::Menacer => "!",
SocialActionKind::Insulter => "Grr",
SocialActionKind::SEngueuler => "!!",
}
}

fn draw_emote_icon(center: Vec2, view: &SocialEmoteView, scale: f32, time: f32) {
let col = Color::new(0.1, 0.1, 0.1, 0.85 * view.alpha);
let s = 8.5 * scale;

    match view.icon {
        SocialEmoteIcon::TalkDots => {
            let speed = if view.is_speaker { 6.0 } else { 3.5 };
            let step = ((view.phase * speed).floor() as i32).rem_euclid(3);

            for i in 0..3 {
                let dx = (i as f32 - 1.0) * (s * 0.8);
                let a = if i as i32 <= step { 1.0 } else { 0.25 };
                draw_circle(
                    center.x + dx,
                    center.y,
                    1.4 * scale,
                    Color::new(col.r, col.g, col.b, col.a * a),
                );
            }
        }
        SocialEmoteIcon::Heart => {
            let pulse = 1.0 + 0.15 * (time * 8.0 + view.phase * 2.0).sin().abs();
            let r = s * 0.38 * pulse;
            let dx = r * 0.9;
            let top_y = center.y - r * 0.2;
            draw_circle(center.x - dx, top_y, r, col);
            draw_circle(center.x + dx, top_y, r, col);
            let p1 = vec2(center.x - dx - r, top_y);
            let p2 = vec2(center.x + dx + r, top_y);
            let p3 = vec2(center.x, center.y + r * 1.9);
            draw_triangle(p1, p2, p3, col);
        }
        SocialEmoteIcon::Question => {
            draw_line(center.x, center.y - s * 0.7, center.x, center.y + s * 0.2, 1.2, col);
            draw_circle(center.x, center.y + s * 0.55, 1.2 * scale, col);
            draw_line(center.x - s * 0.35, center.y - s * 0.7, center.x + s * 0.35, center.y - s * 0.7, 1.2, col);
        }
        SocialEmoteIcon::Laugh => {
            let t = (time * 10.0 + view.phase * 3.0).sin().abs();
            draw_line(center.x - s * 0.7, center.y + s * 0.15, center.x, center.y + s * (0.55 + 0.2 * t), 1.2, col);
            draw_line(center.x, center.y + s * (0.55 + 0.2 * t), center.x + s * 0.7, center.y + s * 0.15, 1.2, col);
            draw_circle(center.x - s * 0.5, center.y - s * 0.25, 1.0 * scale, col);
            draw_circle(center.x + s * 0.5, center.y - s * 0.25, 1.0 * scale, col);
        }
        SocialEmoteIcon::Apology => {
            draw_line(center.x - s * 0.6, center.y, center.x + s * 0.2, center.y, 1.2, col);
            draw_line(center.x + s * 0.2, center.y, center.x + s * 0.55, center.y - s * 0.25, 1.2, col);
            draw_line(center.x + s * 0.2, center.y, center.x + s * 0.55, center.y + s * 0.25, 1.2, col);
        }
        SocialEmoteIcon::Exclamation => {
            draw_line(center.x, center.y - s * 0.7, center.x, center.y + s * 0.2, 1.2, col);
            draw_circle(center.x, center.y + s * 0.55, 1.3 * scale, col);
        }
        SocialEmoteIcon::Anger => {
            draw_line(center.x - s * 0.7, center.y - s * 0.6, center.x + s * 0.7, center.y + s * 0.6, 1.2, col);
            draw_line(center.x + s * 0.7, center.y - s * 0.6, center.x - s * 0.7, center.y + s * 0.6, 1.2, col);
        }
        SocialEmoteIcon::Scribble => {
            let t = time * 9.0 + view.phase * 1.5;
            for i in 0..4 {
                let dx = (i as f32 - 1.5) * (s * 0.35);
                let dy = (t + i as f32).sin() * (s * 0.15);
                draw_line(
                    center.x + dx - s * 0.15,
                    center.y + dy,
                    center.x + dx + s * 0.15,
                    center.y - dy,
                    1.1,
                    col,
                );
            }
        }
        SocialEmoteIcon::Lightning => {
            let p1 = vec2(center.x - s * 0.2, center.y - s * 0.8);
            let p2 = vec2(center.x + s * 0.1, center.y - s * 0.1);
            let p3 = vec2(center.x - s * 0.1, center.y - s * 0.1);
            let p4 = vec2(center.x + s * 0.2, center.y + s * 0.8);
            draw_line(p1.x, p1.y, p2.x, p2.y, 1.3, col);
            draw_line(p2.x, p2.y, p3.x, p3.y, 1.3, col);
            draw_line(p3.x, p3.y, p4.x, p4.y, 1.3, col);
        }
    }
}

fn draw_text_centered(text: &str, x: f32, y: f32, size: f32, color: Color) {
let m = measure_text(text, None, size as u16, 1.0);
draw_text(text, x - m.width * 0.5, y, size, color);
}

Important : tu dois aussi changer l’appel dans modes.rs (section suivante), parce que la fonction s’appelle désormais draw_social_emotes.

===========================================================
8) Patch src/modes.rs (dessin face à face + gestures + call)

A) Update NPC wander call (signature)
Trouve :

update_npc_wanderer(&mut state.npc, &state.world, &state.player, dt);

Remplace par :

update_npc_wanderer(&mut state.npc, &state.world, dt);

B) Remplace l’appel des bulles
Trouve :

ui_pawns::draw_social_bubbles(state);

Remplace par :

ui_pawns::draw_social_emotes(state, time);

C) Ajoute une fonction helper en bas de modes.rs (mapping gesture)
Ajoute quelque part (par ex tout en bas du fichier) :

fn gesture_from_social(g: crate::interactions::SocialGesture) -> CharacterGesture {
match g {
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

D) Dans la boucle de rendu des pawns, applique anim_hint (face + gesture)
Dans run_play_frame, repère la boucle :

for pawn_key in draw_order {
match pawn_key {
PawnKey::SimWorker => { ... }
PawnKey::Npc => { draw_npc(...) }
PawnKey::Player => { draw_player(...) }
}
}

L’idée : remplacer draw_npc/draw_player par draw_character direct, et injecter :

facing override vers partner,

gesture,

force_idle.

Je te donne le bloc complet à mettre à la place de la boucle (copie/colle) :

for pawn_key in draw_order {
let hint = state.social_state.anim_hint(pawn_key);
let gesture = gesture_from_social(hint.gesture);

    match pawn_key {
        PawnKey::SimWorker => {
            let pos = tile_center(state.sim.primary_agent_tile());
            let mut facing = CharacterFacing::Front;
            let mut facing_left = false;

            if hint.force_face_partner {
                if let Some(partner) = hint.partner {
                    if let Some(p2) = pawn_world_pos(state, partner) {
                        let dir = p2 - pos;
                        facing = select_character_facing(dir, facing);
                        facing_left = dir.x < 0.0;
                    }
                }
            }

            draw_character(
                &state.sim.primary_agent_lineage,
                CharacterRenderParams {
                    center: pos,
                    half: Vec2::new(12.0, 16.0),
                    time,
                    scale: 1.0,
                    facing,
                    facing_left,
                    is_walking: false,
                    walk_cycle: 0.0,
                    gesture,
                },
            );
        }

        PawnKey::Npc => {
            let npc = &state.npc;
            let mut facing = npc.facing;
            let mut facing_left = npc.facing_left;
            let mut is_walking = npc.is_walking;
            let walk_cycle = npc.walk_cycle;

            if hint.force_face_partner {
                if let Some(partner) = hint.partner {
                    if let Some(p2) = pawn_world_pos(state, partner) {
                        let dir = p2 - npc.pos;
                        facing = select_character_facing(dir, facing);
                        facing_left = dir.x < 0.0;
                    }
                }
            }
            if hint.force_idle {
                is_walking = false;
            }

            draw_character(
                &state.npc_lineage,
                CharacterRenderParams {
                    center: npc.pos,
                    half: npc.half,
                    time,
                    scale: 1.0,
                    facing,
                    facing_left,
                    is_walking,
                    walk_cycle,
                    gesture,
                },
            );

            if state.debug {
                draw_rectangle_lines(
                    npc.pos.x - npc.half.x,
                    npc.pos.y - npc.half.y,
                    npc.half.x * 2.0,
                    npc.half.y * 2.0,
                    1.0,
                    Color::new(0.0, 0.0, 0.0, 0.35),
                );
            }
        }

        PawnKey::Player => {
            let player = &state.player;
            let mut facing = player.facing;
            let mut facing_left = player.facing_left;
            let mut is_walking = player.is_walking;
            let walk_cycle = player.walk_cycle;

            if hint.force_face_partner {
                if let Some(partner) = hint.partner {
                    if let Some(p2) = pawn_world_pos(state, partner) {
                        let dir = p2 - player.pos;
                        facing = select_character_facing(dir, facing);
                        facing_left = dir.x < 0.0;
                    }
                }
            }
            if hint.force_idle {
                is_walking = false;
            }

            draw_character(
                &state.player_lineage,
                CharacterRenderParams {
                    center: player.pos,
                    half: player.half,
                    time,
                    scale: 1.0,
                    facing,
                    facing_left,
                    is_walking,
                    walk_cycle,
                    gesture,
                },
            );

            if state.debug {
                draw_rectangle_lines(
                    player.pos.x - player.half.x,
                    player.pos.y - player.half.y,
                    player.half.x * 2.0,
                    player.half.y * 2.0,
                    1.0,
                    Color::new(0.0, 0.0, 0.0, 0.35),
                );
            }
        }
    }
}

E) Debug HUD (enlever bubble_timer info, ajouter hold_timer + social)
Dans modes.rs, tu as un debug string qui mentionne bubble_timer. Remplace la partie NPC par quelque chose du genre :

let npc_hint = state.social_state.anim_hint(PawnKey::Npc);
let npc_social = npc_hint.kind.map(|k| k.ui_label()).unwrap_or("idle");
...
format!(
"NPC: hold={:.2} idle={:.2} social={}",
state.npc.hold_timer,
state.npc.idle_timer,
npc_social
)
===========================================================
9) Mise à jour des call sites CharacterRenderParams ailleurs

Tu dois ajouter gesture: CharacterGesture::None partout où tu construis un CharacterRenderParams.
Les endroits typiques dans ton repo :

src/rendu.rs (inspecteur + draw_player/draw_npc)

src/ui_pawns.rs (portrait / sheet)

src/modes.rs (déjà fait ci-dessus)

Dans ui_pawns.rs portrait (autour de ton draw_pawn_sheet_portrait), ajoute :

gesture: CharacterGesture::None,

Checklist rapide anti-bugs (important ✅)

Après avoir collé tout ça :

cargo build doit te forcer à corriger tous les endroits où CharacterRenderParams manque gesture (c’est voulu).

grep -R "bubble_timer" src/ doit retourner 0 résultat (car on a supprimé l’ancien greeting système).

Vérifie en jeu :

quand 2 persos parlent : ils se font face + bras/bouche bougent + bulle “vivante” (dots).

NPC près du player : il fait “Dire bonjour” (bulle) mais pas toutes les 2 secondes → cooldown pair.

Ce que tu gagnes immédiatement 🎯

Un seul moteur social = moins de bugs.

“Ça a l’air vivant” : face à face + gestuelle + bouche animée + bulles pictos.

Lecture claire en un coup d’œil (pictos + mini label).

Base saine pour ajouter ensuite :

un overlay debug “raison du choix” (tu le veux absolument

Jobs-–-Système-&-Priorités

),

des “topics” / mémoire sociale,

des incidents (engueulamde → baisse moral/calme etc) cohérents avec tes barres besoin/synth .