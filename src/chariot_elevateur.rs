use super::*;

const CHARIOT_VITESSE_AVANT_MAX: f32 = 132.0;
const CHARIOT_VITESSE_ARRIERE_MAX: f32 = 74.0;
const CHARIOT_ACCEL_AVANT: f32 = 240.0;
const CHARIOT_ACCEL_ARRIERE: f32 = 200.0;
const CHARIOT_FREINAGE: f32 = 420.0;
const CHARIOT_TRAINEE_ROULAGE: f32 = 165.0;
const CHARIOT_REPONSE_BRAQUAGE: f32 = 5.8;
const CHARIOT_BRAQUAGE_MAX_RAD: f32 = 0.64;
const CHARIOT_ENTRAXE: f32 = 25.0;
const CHARIOT_VITESSE_BAS_REGIME: f32 = 22.0;
const CHARIOT_BRAQUAGE_IMMOBILE: f32 = 0.28;
const CHARIOT_ANIM_ROULANT: f32 = 7.2;
const CHARIOT_ANIM_RALENTI: f32 = 1.3;
const CHARIOT_FOURCHE_HAUTEUR_BASSE: f32 = 0.06;
const CHARIOT_FOURCHE_HAUTEUR_HAUTE: f32 = 0.98;
const CHARIOT_FOURCHE_TAUX: f32 = 1.9;
const CHARIOT_BATTERIE_MAX: f32 = 100.0;
const CHARIOT_BATTERIE_MIN_ROULAGE: f32 = 2.0;
const CHARIOT_BATTERIE_CONSO_PILOTE: f32 = 0.35;
const CHARIOT_BATTERIE_CONSO_MOUVEMENT: f32 = 2.4;
const CHARIOT_BATTERIE_CHARGE_PAR_SEC: f32 = 11.0;
const CHARIOT_ETAT_USURE_PAR_SEC: f32 = 0.018;
const CHARIOT_BOARD_RADIUS: f32 = 64.0;
pub(crate) const CHARGEUR_INTERACTION_RADIUS: f32 = 82.0;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum OrientationChariot {
    Haut,
    Bas,
    Gauche,
    Droite,
}

impl OrientationChariot {
    pub(crate) fn depuis_input(input: Vec2, fallback: Self) -> Self {
        if input.length_squared() <= 0.0001 {
            return fallback;
        }
        if input.x.abs() >= input.y.abs() {
            if input.x >= 0.0 {
                Self::Droite
            } else {
                Self::Gauche
            }
        } else if input.y >= 0.0 {
            Self::Bas
        } else {
            Self::Haut
        }
    }

    pub(crate) fn delta_devant(self) -> (i32, i32) {
        match self {
            Self::Haut => (0, -1),
            Self::Bas => (0, 1),
            Self::Gauche => (-1, 0),
            Self::Droite => (1, 0),
        }
    }

    pub(crate) fn delta_lateral(self) -> (i32, i32) {
        match self {
            Self::Haut => (1, 0),
            Self::Bas => (-1, 0),
            Self::Gauche => (0, 1),
            Self::Droite => (0, -1),
        }
    }

    pub(crate) fn depuis_heading(heading_rad: f32, fallback: Self) -> Self {
        let input = vec2(heading_rad.cos(), heading_rad.sin());
        Self::depuis_input(input, fallback)
    }

    pub(crate) fn to_character_facing(self) -> CharacterFacing {
        match self {
            Self::Gauche | Self::Droite => CharacterFacing::Side,
            Self::Haut => CharacterFacing::Back,
            Self::Bas => CharacterFacing::Front,
        }
    }

    pub(crate) fn is_left(self) -> bool {
        matches!(self, Self::Gauche)
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Haut => "haut",
            Self::Bas => "bas",
            Self::Gauche => "gauche",
            Self::Droite => "droite",
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct ChariotElevateur {
    pub pos: Vec2,
    pub half: Vec2,
    pub velocity: Vec2,
    pub vitesse_longitudinale: f32,
    pub heading_rad: f32,
    pub angle_braquage: f32,
    pub orientation: OrientationChariot,
    pub pilote_a_bord: bool,
    pub caisse_chargee: Option<PropKind>,
    pub phase_anim: f32,
    pub fourche_hauteur: f32,
    pub batterie_pct: f32,
    pub etat_pct: f32,
    pub est_en_charge: bool,
}

impl ChariotElevateur {
    pub(crate) fn new(pos: Vec2) -> Self {
        Self {
            pos,
            half: vec2(14.5, 13.5),
            velocity: Vec2::ZERO,
            vitesse_longitudinale: 0.0,
            heading_rad: 0.0,
            angle_braquage: 0.0,
            orientation: OrientationChariot::Droite,
            pilote_a_bord: false,
            caisse_chargee: None,
            phase_anim: 0.0,
            fourche_hauteur: CHARIOT_FOURCHE_HAUTEUR_BASSE,
            batterie_pct: CHARIOT_BATTERIE_MAX * 0.82,
            etat_pct: 100.0,
            est_en_charge: false,
        }
    }

    pub(crate) fn forward(self) -> Vec2 {
        vec2(self.heading_rad.cos(), self.heading_rad.sin())
    }

    pub(crate) fn tuile_courante(self, world: &World) -> (i32, i32) {
        tile_from_world_clamped(world, self.pos)
    }

    pub(crate) fn tuile_devant(self, world: &World) -> (i32, i32) {
        let current = self.tuile_courante(world);
        let d = self.orientation.delta_devant();
        (
            clamp_i32(current.0 + d.0, 0, world.w - 1),
            clamp_i32(current.1 + d.1, 0, world.h - 1),
        )
    }

    pub(crate) fn peut_monter(self, player_pos: Vec2) -> bool {
        self.pos.distance(player_pos) <= CHARIOT_BOARD_RADIUS
    }

    pub(crate) fn batterie_ratio(self) -> f32 {
        (self.batterie_pct / CHARIOT_BATTERIE_MAX).clamp(0.0, 1.0)
    }

    pub(crate) fn statut_label(self) -> &'static str {
        if self.est_en_charge {
            "en charge"
        } else if self.batterie_pct <= CHARIOT_BATTERIE_MIN_ROULAGE + 0.5 {
            "batterie faible"
        } else if self.etat_pct < 45.0 {
            "maintenance recommandee"
        } else {
            "operationnel"
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct ChargeurClark {
    pub base_pos: Vec2,
    pub cable_tenu: bool,
    pub cable_branche: bool,
}

impl ChargeurClark {
    pub(crate) fn point_prise(self) -> Vec2 {
        self.base_pos + vec2(9.0, -9.5)
    }

    pub(crate) fn proche_base(self, pos: Vec2) -> bool {
        self.base_pos.distance(pos) <= CHARGEUR_INTERACTION_RADIUS
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ActionConduiteChariot {
    Monte,
    Descend,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ErreurConduiteChariot {
    TropLoin,
    AucuneSortieValide,
    EnCharge,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ActionCaisseChariot {
    Chargee { kind: PropKind, from: (i32, i32) },
    Deposee { kind: PropKind, to: (i32, i32) },
    ChargeeDepuisRack { niveau: u8, from: (i32, i32) },
    DeposeeDansRack { niveau: u8, to: (i32, i32) },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ErreurCaisseChariot {
    HorsConduite,
    AucuneCaisseProche,
    TuileDepotBloquee,
    RackNiveauOccupe,
    RackNiveauVide,
    RackSansPalette,
    RackIntrouvable,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ActionChargeurClark {
    Pris,
    Range,
    Branche,
    Debranche,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum ErreurChargeurClark {
    AucuneInteractionPossible,
    TropLoinBase,
    ClarkOccupe,
}

pub(crate) fn est_caisse_transportable(kind: PropKind) -> bool {
    matches!(
        kind,
        PropKind::Crate
            | PropKind::BoxCartonVide
            | PropKind::BoxSacBleu
            | PropKind::BoxSacRouge
            | PropKind::BoxSacVert
            | PropKind::CaisseAilBrut
            | PropKind::CaisseAilCasse
            | PropKind::PaletteLogistique
    )
}

pub(crate) fn spawn_chariot_pour_map(world: &World, player_spawn: (i32, i32)) -> ChariotElevateur {
    let desired = (player_spawn.0 + 3, player_spawn.1);
    let tile = nearest_walkable_tile(world, desired).unwrap_or(player_spawn);
    ChariotElevateur::new(tile_center(tile))
}

pub(crate) fn spawn_chargeur_pour_chariot(
    world: &World,
    chariot: &ChariotElevateur,
) -> ChargeurClark {
    let chariot_tile = tile_from_world_clamped(world, chariot.pos);
    // Try several nearby slots first so the charger is accessible right around the Clark.
    let offsets = [
        (-2, 1),
        (-2, -1),
        (2, 1),
        (2, -1),
        (-3, 0),
        (3, 0),
        (0, 2),
        (0, -2),
        (-4, 2),
        (4, 2),
    ];

    let mut base_tile = chariot_tile;
    let mut found = false;
    for (dx, dy) in offsets {
        let tx = clamp_i32(chariot_tile.0 + dx, 0, world.w - 1);
        let ty = clamp_i32(chariot_tile.1 + dy, 0, world.h - 1);
        if world.in_bounds(tx, ty) && !world.is_solid(tx, ty) {
            base_tile = (tx, ty);
            found = true;
            break;
        }
    }
    if !found {
        let fallback = (chariot_tile.0 - 2, chariot_tile.1 + 1);
        base_tile = nearest_walkable_tile(world, fallback).unwrap_or(chariot_tile);
    }

    ChargeurClark {
        base_pos: tile_center(base_tile),
        cable_tenu: false,
        cable_branche: false,
    }
}

pub(crate) fn interagir_chargeur_clark(
    chariot: &mut ChariotElevateur,
    chargeur: &mut ChargeurClark,
    player_pos: Vec2,
) -> Result<ActionChargeurClark, ErreurChargeurClark> {
    let proche_base = chargeur.proche_base(player_pos);
    let proche_clark = chariot.peut_monter(player_pos);

    if chargeur.cable_branche {
        if !proche_base && !proche_clark {
            return Err(ErreurChargeurClark::AucuneInteractionPossible);
        }
        if chariot.pilote_a_bord {
            return Err(ErreurChargeurClark::ClarkOccupe);
        }
        chargeur.cable_branche = false;
        chariot.est_en_charge = false;
        // Priority: when close to the Clark, "E" means unplug from the Clark first.
        if proche_clark {
            chargeur.cable_tenu = true;
            Ok(ActionChargeurClark::Debranche)
        } else {
            chargeur.cable_tenu = false;
            Ok(ActionChargeurClark::Range)
        }
    } else if chargeur.cable_tenu {
        if proche_clark {
            if chariot.pilote_a_bord {
                return Err(ErreurChargeurClark::ClarkOccupe);
            }
            chargeur.cable_tenu = false;
            chargeur.cable_branche = true;
            chariot.est_en_charge = true;
            chariot.velocity = Vec2::ZERO;
            chariot.vitesse_longitudinale = 0.0;
            chariot.angle_braquage = 0.0;
            Ok(ActionChargeurClark::Branche)
        } else if proche_base {
            chargeur.cable_tenu = false;
            Ok(ActionChargeurClark::Range)
        } else {
            Err(ErreurChargeurClark::AucuneInteractionPossible)
        }
    } else if proche_base {
        chargeur.cable_tenu = true;
        Ok(ActionChargeurClark::Pris)
    } else if proche_clark {
        Err(ErreurChargeurClark::TropLoinBase)
    } else {
        Err(ErreurChargeurClark::AucuneInteractionPossible)
    }
}

pub(crate) fn basculer_conduite_chariot(
    chariot: &mut ChariotElevateur,
    player: &mut Player,
    world: &World,
) -> Result<ActionConduiteChariot, ErreurConduiteChariot> {
    if !chariot.pilote_a_bord {
        if chariot.est_en_charge {
            return Err(ErreurConduiteChariot::EnCharge);
        }
        if !chariot.peut_monter(player.pos) {
            return Err(ErreurConduiteChariot::TropLoin);
        }
        chariot.pilote_a_bord = true;
        chariot.velocity = Vec2::ZERO;
        chariot.vitesse_longitudinale = 0.0;
        chariot.angle_braquage = 0.0;
        player.pos = chariot.pos;
        reset_auto_move(player);
        player.control_mode = ControlMode::Manual;
        return Ok(ActionConduiteChariot::Monte);
    }

    let current = chariot.tuile_courante(world);
    let fd = chariot.orientation.delta_devant();
    let sd = chariot.orientation.delta_lateral();
    let candidates = [
        (current.0 - fd.0, current.1 - fd.1),
        (current.0 + sd.0, current.1 + sd.1),
        (current.0 - sd.0, current.1 - sd.1),
        (current.0 - fd.0 + sd.0, current.1 - fd.1 + sd.1),
        (current.0 - fd.0 - sd.0, current.1 - fd.1 - sd.1),
        current,
    ];

    if let Some(tile) = candidates
        .into_iter()
        .find(|&(x, y)| world.in_bounds(x, y) && !world.is_solid(x, y))
    {
        chariot.pilote_a_bord = false;
        chariot.velocity = Vec2::ZERO;
        chariot.vitesse_longitudinale = 0.0;
        chariot.angle_braquage = 0.0;
        player.pos = tile_center(tile);
        reset_auto_move(player);
        player.control_mode = ControlMode::Manual;
        return Ok(ActionConduiteChariot::Descend);
    }

    Err(ErreurConduiteChariot::AucuneSortieValide)
}

pub(crate) fn actionner_fourches_chariot(
    chariot: &mut ChariotElevateur,
    world: &World,
    props: &mut Vec<Prop>,
    sim: &mut crate::sim::FactorySim,
) -> Result<ActionCaisseChariot, ErreurCaisseChariot> {
    if !chariot.pilote_a_bord {
        return Err(ErreurCaisseChariot::HorsConduite);
    }

    let niveau_rack = crate::sim::FactorySim::rack_niveau_depuis_fourche(chariot.fourche_hauteur);
    let target_rack_tile = chariot.tuile_devant(world);
    let target_is_rack =
        sim.block_kind_at_tile(target_rack_tile) == Some(crate::sim::BlockKind::Buffer);

    if let Some(kind) = chariot.caisse_chargee {
        if target_is_rack {
            if kind != PropKind::PaletteLogistique {
                return Err(ErreurCaisseChariot::RackSansPalette);
            }
            match sim.rack_store_palette(target_rack_tile, niveau_rack) {
                Ok(()) => {
                    chariot.caisse_chargee = None;
                    return Ok(ActionCaisseChariot::DeposeeDansRack {
                        niveau: niveau_rack,
                        to: target_rack_tile,
                    });
                }
                Err(err) if err.contains("deja occupe") => {
                    return Err(ErreurCaisseChariot::RackNiveauOccupe);
                }
                Err(_) => return Err(ErreurCaisseChariot::RackIntrouvable),
            }
        }

        let target = chariot.tuile_devant(world);
        if world.is_solid(target.0, target.1) || prop_index_at_tile(props, target).is_some() {
            return Err(ErreurCaisseChariot::TuileDepotBloquee);
        }
        props.push(Prop {
            tile_x: target.0,
            tile_y: target.1,
            kind,
            phase: prop_phase_for_tile(target),
            rotation_quarter: 0,
        });
        chariot.caisse_chargee = None;
        return Ok(ActionCaisseChariot::Deposee { kind, to: target });
    }

    if target_is_rack {
        match sim.rack_take_palette(target_rack_tile, niveau_rack) {
            Ok(()) => {
                chariot.caisse_chargee = Some(PropKind::PaletteLogistique);
                return Ok(ActionCaisseChariot::ChargeeDepuisRack {
                    niveau: niveau_rack,
                    from: target_rack_tile,
                });
            }
            Err(err) if err.contains("vide") => return Err(ErreurCaisseChariot::RackNiveauVide),
            Err(_) => return Err(ErreurCaisseChariot::RackIntrouvable),
        }
    }

    let base = chariot.tuile_courante(world);
    let fd = chariot.orientation.delta_devant();
    let sd = chariot.orientation.delta_lateral();
    let candidates = [
        (base.0 + fd.0, base.1 + fd.1),
        base,
        (base.0 + fd.0 + sd.0, base.1 + fd.1 + sd.1),
        (base.0 + fd.0 - sd.0, base.1 + fd.1 - sd.1),
        (base.0 - fd.0, base.1 - fd.1),
    ];

    for tile in candidates {
        let Some(idx) = prop_index_at_tile(props, tile) else {
            continue;
        };
        let kind = props[idx].kind;
        if !est_caisse_transportable(kind) {
            continue;
        }
        props.swap_remove(idx);
        chariot.caisse_chargee = Some(kind);
        return Ok(ActionCaisseChariot::Chargee { kind, from: tile });
    }

    Err(ErreurCaisseChariot::AucuneCaisseProche)
}

pub(crate) fn mettre_a_jour_chariot(
    chariot: &mut ChariotElevateur,
    world: &World,
    input: Vec2,
    commande_fourche: f32,
    dt: f32,
) {
    if dt <= 0.0 {
        return;
    }

    if chariot.est_en_charge {
        chariot.batterie_pct = (chariot.batterie_pct + CHARIOT_BATTERIE_CHARGE_PAR_SEC * dt)
            .clamp(0.0, CHARIOT_BATTERIE_MAX);
    } else if chariot.pilote_a_bord {
        let speed_factor =
            (chariot.vitesse_longitudinale.abs() / CHARIOT_VITESSE_AVANT_MAX).clamp(0.0, 1.0);
        let conso = CHARIOT_BATTERIE_CONSO_PILOTE + CHARIOT_BATTERIE_CONSO_MOUVEMENT * speed_factor;
        chariot.batterie_pct = (chariot.batterie_pct - conso * dt).clamp(0.0, CHARIOT_BATTERIE_MAX);
        chariot.etat_pct = (chariot.etat_pct
            - CHARIOT_ETAT_USURE_PAR_SEC * dt * (0.6 + speed_factor))
            .clamp(0.0, 100.0);
    }

    let (commande_accel, commande_braquage) = if chariot.pilote_a_bord {
        commande_conduite_depuis_input(input)
    } else {
        (0.0, 0.0)
    };

    let batterie_ok = chariot.batterie_pct > CHARIOT_BATTERIE_MIN_ROULAGE && !chariot.est_en_charge;
    let commande_accel = if batterie_ok { commande_accel } else { 0.0 };
    let commande_braquage = if chariot.est_en_charge {
        0.0
    } else {
        commande_braquage
    };

    chariot.angle_braquage = move_towards_scalar(
        chariot.angle_braquage,
        commande_braquage,
        CHARIOT_REPONSE_BRAQUAGE * dt,
    )
    .clamp(-1.0, 1.0);

    let vitesse_cible = if commande_accel >= 0.0 {
        commande_accel * CHARIOT_VITESSE_AVANT_MAX
    } else {
        commande_accel * CHARIOT_VITESSE_ARRIERE_MAX
    };
    let acceleration = if commande_accel.abs() <= 0.001 {
        CHARIOT_TRAINEE_ROULAGE
    } else if chariot.vitesse_longitudinale.signum() != vitesse_cible.signum()
        && chariot.vitesse_longitudinale.abs() > 0.5
    {
        CHARIOT_FREINAGE
    } else if vitesse_cible.abs() > chariot.vitesse_longitudinale.abs() {
        if vitesse_cible >= 0.0 {
            CHARIOT_ACCEL_AVANT
        } else {
            CHARIOT_ACCEL_ARRIERE
        }
    } else {
        CHARIOT_FREINAGE * 0.75
    };
    chariot.vitesse_longitudinale = move_towards_scalar(
        chariot.vitesse_longitudinale,
        vitesse_cible,
        acceleration * dt,
    );

    if chariot.vitesse_longitudinale.abs() > 0.001 {
        let angle_roues = chariot.angle_braquage * CHARIOT_BRAQUAGE_MAX_RAD;
        let regime =
            (chariot.vitesse_longitudinale.abs() / CHARIOT_VITESSE_BAS_REGIME).clamp(0.0, 1.0);
        let gain_direction = CHARIOT_BRAQUAGE_IMMOBILE + (1.0 - CHARIOT_BRAQUAGE_IMMOBILE) * regime;
        let vitesse_rotation =
            (chariot.vitesse_longitudinale / CHARIOT_ENTRAXE) * angle_roues.tan() * gain_direction;
        chariot.heading_rad = normalize_angle_pi(chariot.heading_rad + vitesse_rotation * dt);
    }

    let forward = chariot.forward();
    chariot.velocity = forward * chariot.vitesse_longitudinale;

    let requested_delta = chariot.velocity * dt;
    let pos_before = chariot.pos;
    deplacer_chariot_axis(chariot, world, requested_delta.x, true);
    deplacer_chariot_axis(chariot, world, requested_delta.y, false);
    let moved_delta = chariot.pos - pos_before;
    chariot.velocity = moved_delta / dt;
    chariot.vitesse_longitudinale = chariot.velocity.dot(forward);

    if chariot.velocity.length_squared() < 0.001 {
        chariot.velocity = Vec2::ZERO;
        chariot.vitesse_longitudinale = 0.0;
    }

    if chariot.vitesse_longitudinale.abs() > 0.05 || chariot.angle_braquage.abs() > 0.05 {
        chariot.orientation =
            OrientationChariot::depuis_heading(chariot.heading_rad, chariot.orientation);
    }

    if chariot.pilote_a_bord {
        let commande = commande_fourche.clamp(-1.0, 1.0);
        chariot.fourche_hauteur = (chariot.fourche_hauteur + commande * CHARIOT_FOURCHE_TAUX * dt)
            .clamp(CHARIOT_FOURCHE_HAUTEUR_BASSE, CHARIOT_FOURCHE_HAUTEUR_HAUTE);
    } else {
        chariot.fourche_hauteur = move_towards_scalar(
            chariot.fourche_hauteur,
            CHARIOT_FOURCHE_HAUTEUR_BASSE,
            CHARIOT_FOURCHE_TAUX * 0.35 * dt,
        );
    }

    let anim_speed = if chariot.velocity.length_squared() > 0.001 {
        CHARIOT_ANIM_ROULANT
    } else {
        CHARIOT_ANIM_RALENTI
    };
    chariot.phase_anim += dt * anim_speed;
    if chariot.phase_anim > std::f32::consts::TAU {
        chariot.phase_anim -= std::f32::consts::TAU;
    }
}

fn move_towards_scalar(current: f32, target: f32, max_delta: f32) -> f32 {
    if max_delta <= 0.0 {
        return current;
    }
    let delta = target - current;
    if delta.abs() <= max_delta {
        target
    } else {
        current + delta.signum() * max_delta
    }
}

fn normalize_angle_pi(mut angle: f32) -> f32 {
    while angle > std::f32::consts::PI {
        angle -= std::f32::consts::TAU;
    }
    while angle < -std::f32::consts::PI {
        angle += std::f32::consts::TAU;
    }
    angle
}

fn commande_conduite_depuis_input(input: Vec2) -> (f32, f32) {
    let accel = (-input.y).clamp(-1.0, 1.0);
    let braquage = input.x.clamp(-1.0, 1.0);
    (accel, braquage)
}

fn deplacer_chariot_axis(
    chariot: &mut ChariotElevateur,
    world: &World,
    delta: f32,
    is_x_axis: bool,
) {
    if delta.abs() <= f32::EPSILON {
        return;
    }

    if is_x_axis {
        chariot.pos.x += delta;
    } else {
        chariot.pos.y += delta;
    }

    let mut aabb = Aabb::from_center(chariot.pos, chariot.half);
    let min_tx = (aabb.min.x / TILE_SIZE).floor() as i32 - 1;
    let max_tx = (aabb.max.x / TILE_SIZE).floor() as i32 + 1;
    let min_ty = (aabb.min.y / TILE_SIZE).floor() as i32 - 1;
    let max_ty = (aabb.max.y / TILE_SIZE).floor() as i32 + 1;

    for ty in min_ty..=max_ty {
        for tx in min_tx..=max_tx {
            if !world.is_solid(tx, ty) {
                continue;
            }
            let tile = World::tile_rect(tx, ty);
            if aabb.intersects_rect(tile) {
                if is_x_axis {
                    if delta > 0.0 {
                        chariot.pos.x = tile.x - chariot.half.x;
                    } else {
                        chariot.pos.x = tile.x + tile.w + chariot.half.x;
                    }
                } else if delta > 0.0 {
                    chariot.pos.y = tile.y - chariot.half.y;
                } else {
                    chariot.pos.y = tile.y + tile.h + chariot.half.y;
                }
                aabb = Aabb::from_center(chariot.pos, chariot.half);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn est_caisse_transportable_filtre_types_attendus() {
        assert!(est_caisse_transportable(PropKind::Crate));
        assert!(est_caisse_transportable(PropKind::PaletteLogistique));
        assert!(!est_caisse_transportable(PropKind::Lamp));
        assert!(!est_caisse_transportable(PropKind::Lavabo));
    }

    #[test]
    fn actionner_fourches_charge_et_decharge_cycle() {
        let world = World::new_room(16, 12);
        let mut chariot = ChariotElevateur::new(tile_center((6, 6)));
        chariot.pilote_a_bord = true;
        chariot.orientation = OrientationChariot::Droite;
        let mut sim = crate::sim::FactorySim::new(crate::sim::StarterSimConfig::default(), 16, 12);
        let mut props = vec![Prop {
            tile_x: 7,
            tile_y: 6,
            kind: PropKind::BoxCartonVide,
            phase: 0.0,
            rotation_quarter: 0,
        }];

        let action_charge = actionner_fourches_chariot(&mut chariot, &world, &mut props, &mut sim)
            .expect("charge should succeed");
        assert_eq!(
            action_charge,
            ActionCaisseChariot::Chargee {
                kind: PropKind::BoxCartonVide,
                from: (7, 6)
            }
        );
        assert!(props.is_empty());
        assert_eq!(chariot.caisse_chargee, Some(PropKind::BoxCartonVide));

        let action_decharge =
            actionner_fourches_chariot(&mut chariot, &world, &mut props, &mut sim)
                .expect("drop should succeed");
        assert_eq!(
            action_decharge,
            ActionCaisseChariot::Deposee {
                kind: PropKind::BoxCartonVide,
                to: (7, 6)
            }
        );
        assert_eq!(chariot.caisse_chargee, None);
        assert_eq!(props.len(), 1);
        assert_eq!(props[0].tile_x, 7);
        assert_eq!(props[0].tile_y, 6);
        assert_eq!(props[0].kind, PropKind::BoxCartonVide);
    }

    #[test]
    fn conduite_accelere_tourne_et_freine_progressivement() {
        let world = World::new_room(42, 42);
        let mut chariot = ChariotElevateur::new(tile_center((20, 20)));
        chariot.pilote_a_bord = true;
        let dt = 1.0 / 60.0;

        for _ in 0..120 {
            mettre_a_jour_chariot(&mut chariot, &world, vec2(0.0, -1.0), 0.0, dt);
        }
        assert!(
            chariot.vitesse_longitudinale > 58.0,
            "expected forward acceleration, got {}",
            chariot.vitesse_longitudinale
        );

        let heading_initial = chariot.heading_rad;
        for _ in 0..90 {
            mettre_a_jour_chariot(&mut chariot, &world, vec2(1.0, -1.0).normalize(), 0.0, dt);
        }
        let heading_delta = normalize_angle_pi(chariot.heading_rad - heading_initial).abs();
        assert!(
            heading_delta > 0.12,
            "expected heading change while steering, got {}",
            heading_delta
        );
        assert!(
            chariot.angle_braquage > 0.25,
            "expected steering animation to react, got {}",
            chariot.angle_braquage
        );

        for _ in 0..210 {
            mettre_a_jour_chariot(&mut chariot, &world, Vec2::ZERO, 0.0, dt);
        }
        assert!(
            chariot.vitesse_longitudinale.abs() < 1.5,
            "expected near stop after releasing throttle, got {}",
            chariot.vitesse_longitudinale
        );
    }

    #[test]
    fn conduite_arriere_reste_limitee() {
        let world = World::new_room(40, 40);
        let mut chariot = ChariotElevateur::new(tile_center((20, 20)));
        chariot.pilote_a_bord = true;
        let dt = 1.0 / 60.0;

        for _ in 0..320 {
            mettre_a_jour_chariot(&mut chariot, &world, vec2(0.0, 1.0), 0.0, dt);
        }

        assert!(
            chariot.vitesse_longitudinale < -15.0,
            "expected meaningful reverse speed, got {}",
            chariot.vitesse_longitudinale
        );
        assert!(
            chariot.vitesse_longitudinale >= -(CHARIOT_VITESSE_ARRIERE_MAX + 0.6),
            "reverse speed exceeded cap: {}",
            chariot.vitesse_longitudinale
        );
    }

    #[test]
    fn fourches_repondent_a_la_commande_manuel() {
        let world = World::new_room(24, 24);
        let mut chariot = ChariotElevateur::new(tile_center((12, 12)));
        chariot.pilote_a_bord = true;
        let dt = 1.0 / 60.0;

        let start = chariot.fourche_hauteur;
        for _ in 0..100 {
            mettre_a_jour_chariot(&mut chariot, &world, Vec2::ZERO, 1.0, dt);
        }
        assert!(
            chariot.fourche_hauteur > start + 0.3,
            "fork should go up, got {} from {}",
            chariot.fourche_hauteur,
            start
        );
        assert!(chariot.fourche_hauteur <= CHARIOT_FOURCHE_HAUTEUR_HAUTE + 0.001);

        for _ in 0..140 {
            mettre_a_jour_chariot(&mut chariot, &world, Vec2::ZERO, -1.0, dt);
        }
        assert!(
            chariot.fourche_hauteur < 0.2,
            "fork should go down, got {}",
            chariot.fourche_hauteur
        );
        assert!(chariot.fourche_hauteur >= CHARIOT_FOURCHE_HAUTEUR_BASSE - 0.001);
    }

    #[test]
    fn actionner_fourches_rack_palette_par_niveau() {
        let world = World::new_room(16, 12);
        let mut sim = crate::sim::FactorySim::new(crate::sim::StarterSimConfig::default(), 16, 12);
        let rack_tile = sim
            .block_debug_views()
            .into_iter()
            .find(|block| block.kind == crate::sim::BlockKind::Buffer)
            .expect("default layout must include one rack")
            .tile;
        let candidates = [
            ((rack_tile.0 - 1, rack_tile.1), OrientationChariot::Droite),
            ((rack_tile.0 + 1, rack_tile.1), OrientationChariot::Gauche),
            ((rack_tile.0, rack_tile.1 - 1), OrientationChariot::Bas),
            ((rack_tile.0, rack_tile.1 + 1), OrientationChariot::Haut),
        ];
        let (driver_tile, orientation) = candidates
            .into_iter()
            .find(|(tile, _)| {
                world.in_bounds(tile.0, tile.1)
                    && !world.is_solid(tile.0, tile.1)
                    && sim.block_kind_at_tile(*tile).is_none()
            })
            .expect("must find a free adjacent tile next to rack");

        let mut chariot = ChariotElevateur::new(tile_center(driver_tile));
        chariot.pilote_a_bord = true;
        chariot.orientation = orientation;
        chariot.fourche_hauteur = 0.98;
        chariot.caisse_chargee = Some(PropKind::PaletteLogistique);
        let mut props = Vec::new();

        let action_depot = actionner_fourches_chariot(&mut chariot, &world, &mut props, &mut sim)
            .expect("rack drop should succeed");
        match action_depot {
            ActionCaisseChariot::DeposeeDansRack { niveau, to } => {
                assert_eq!(to, rack_tile);
                assert_eq!(
                    niveau,
                    crate::sim::FactorySim::rack_niveau_depuis_fourche(0.98)
                );
            }
            other => panic!("unexpected action: {:?}", other),
        }
        assert_eq!(chariot.caisse_chargee, None);

        let action_charge = actionner_fourches_chariot(&mut chariot, &world, &mut props, &mut sim)
            .expect("rack pickup should succeed");
        match action_charge {
            ActionCaisseChariot::ChargeeDepuisRack { niveau, from } => {
                assert_eq!(from, rack_tile);
                assert_eq!(
                    niveau,
                    crate::sim::FactorySim::rack_niveau_depuis_fourche(0.98)
                );
            }
            other => panic!("unexpected action: {:?}", other),
        }
        assert_eq!(chariot.caisse_chargee, Some(PropKind::PaletteLogistique));
    }

    #[test]
    fn chargeur_branche_bloque_montee_et_permet_debranchage() {
        let world = World::new_room(28, 28);
        let mut chariot = ChariotElevateur::new(tile_center((14, 14)));
        let mut chargeur = spawn_chargeur_pour_chariot(&world, &chariot);

        let base_pos = chargeur.base_pos;
        let chariot_pos = chariot.pos;
        let action_prise =
            interagir_chargeur_clark(&mut chariot, &mut chargeur, base_pos).expect("pick cable");
        assert_eq!(action_prise, ActionChargeurClark::Pris);
        assert!(chargeur.cable_tenu);

        let action_branche =
            interagir_chargeur_clark(&mut chariot, &mut chargeur, chariot_pos).expect("plug cable");
        assert_eq!(action_branche, ActionChargeurClark::Branche);
        assert!(chargeur.cable_branche);
        assert!(chariot.est_en_charge);

        let mut player = Player::new(chariot.pos);
        let err = basculer_conduite_chariot(&mut chariot, &mut player, &world)
            .expect_err("boarding should be blocked while charging");
        assert_eq!(err, ErreurConduiteChariot::EnCharge);

        let action_debranche =
            interagir_chargeur_clark(&mut chariot, &mut chargeur, chariot_pos).expect("unplug");
        assert_eq!(action_debranche, ActionChargeurClark::Debranche);
        assert!(!chariot.est_en_charge);
        assert!(!chargeur.cable_branche);
        assert!(chargeur.cable_tenu);
    }
}
