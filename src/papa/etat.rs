use super::constructeur::tick_construction;
use super::placement::trouver_ancre_libre;
use super::plan::PapaPlanAsset;
use crate::character::CharacterFacing;
use macroquad::prelude::*;

#[derive(Clone, Debug)]
pub struct PapaPnjEtat {
    pub nom: String,
    pub pos: Vec2,
    pub facing: CharacterFacing,
    pub facing_left: bool,
    pub walk_cycle: f32,
    pub ancre: (i32, i32),
    pub prochaine_etape: usize,
    pub timer_etape_s: f32,
    pub termine: bool,
    pub blocage: Option<String>,
    pub derniere_action: String,
}

#[derive(Clone, Debug, Default)]
pub struct PapaEtat {
    plan: Option<PapaPlanAsset>,
    pub erreur_plan: Option<String>,
    pub pnj: Option<PapaPnjEtat>,
}

impl PapaEtat {
    pub fn charger_depuis_fichier(path: &str) -> Self {
        match PapaPlanAsset::charger_depuis_fichier(path) {
            Ok(plan) => Self {
                plan: Some(plan),
                erreur_plan: None,
                pnj: None,
            },
            Err(err) => Self {
                plan: None,
                erreur_plan: Some(err),
                pnj: None,
            },
        }
    }

    #[cfg(test)]
    pub fn depuis_plan(plan: PapaPlanAsset) -> Self {
        Self {
            plan: Some(plan),
            erreur_plan: None,
            pnj: None,
        }
    }

    pub fn declencher_appel(
        &mut self,
        world: &crate::World,
        sim: &crate::sim::FactorySim,
        player_pos: Vec2,
    ) -> Result<String, String> {
        if let Some(err) = &self.erreur_plan {
            return Err(format!("Appel impossible: plan Papa indisponible ({err})"));
        }
        let Some(plan) = self.plan.as_ref() else {
            return Err("Appel impossible: plan Papa absent".to_string());
        };

        if let Some(pnj) = &self.pnj
            && !pnj.termine
            && pnj.blocage.is_none()
        {
            return Err("Papa est deja en intervention".to_string());
        }

        let player_tile = crate::tile_from_world_clamped(world, player_pos);
        let spawn_tile =
            crate::nearest_walkable_tile(world, (player_tile.0 + 1, player_tile.1 + 1))
                .unwrap_or(player_tile);
        let Some(ancre) = trouver_ancre_libre(plan, world, sim, player_tile) else {
            return Err("Papa ne trouve aucun espace libre pour monter la ligne".to_string());
        };

        self.pnj = Some(PapaPnjEtat {
            nom: "Papa".to_string(),
            pos: crate::tile_center(spawn_tile),
            facing: CharacterFacing::Front,
            facing_left: false,
            walk_cycle: 0.0,
            ancre,
            prochaine_etape: 0,
            timer_etape_s: 0.0,
            termine: false,
            blocage: None,
            derniere_action: format!("Demarrage {} ({} etapes)", plan.label, plan.blocs.len()),
        });
        Ok("Papa arrive pour construire la ligne complete.".to_string())
    }

    pub fn tick(
        &mut self,
        dt: f32,
        world: &crate::World,
        sim: &mut crate::sim::FactorySim,
    ) -> Option<String> {
        let plan = self.plan.as_ref()?;
        let pnj = self.pnj.as_mut()?;
        tick_construction(pnj, plan, dt, world, sim)
    }

    pub fn pnj(&self) -> Option<&PapaPnjEtat> {
        self.pnj.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::super::plan::{PapaPlanAsset, PapaPlanBloc};
    use super::*;
    use crate::sim::{BlockKind, BlockOrientation, StarterSimConfig};

    fn plan_complet_test() -> PapaPlanAsset {
        PapaPlanAsset {
            schema_version: super::super::plan::PAPA_PLAN_SCHEMA_VERSION,
            label: "Plan test".to_string(),
            delai_etape_s: 0.01,
            blocs: vec![
                PapaPlanBloc {
                    kind: BlockKind::InputHopper,
                    orientation: BlockOrientation::East,
                    offset: (0, 0),
                },
                PapaPlanBloc {
                    kind: BlockKind::Conveyor,
                    orientation: BlockOrientation::East,
                    offset: (8, 2),
                },
                PapaPlanBloc {
                    kind: BlockKind::FluidityTank,
                    orientation: BlockOrientation::East,
                    offset: (9, 0),
                },
                PapaPlanBloc {
                    kind: BlockKind::Conveyor,
                    orientation: BlockOrientation::East,
                    offset: (14, 2),
                },
                PapaPlanBloc {
                    kind: BlockKind::Cutter,
                    orientation: BlockOrientation::East,
                    offset: (15, 1),
                },
                PapaPlanBloc {
                    kind: BlockKind::DistributorBelt,
                    orientation: BlockOrientation::East,
                    offset: (18, 2),
                },
                PapaPlanBloc {
                    kind: BlockKind::DryerOven,
                    orientation: BlockOrientation::East,
                    offset: (25, -3),
                },
                PapaPlanBloc {
                    kind: BlockKind::OvenExitConveyor,
                    orientation: BlockOrientation::East,
                    offset: (45, 2),
                },
                PapaPlanBloc {
                    kind: BlockKind::Flaker,
                    orientation: BlockOrientation::East,
                    offset: (52, 1),
                },
                PapaPlanBloc {
                    kind: BlockKind::SuctionPipe,
                    orientation: BlockOrientation::East,
                    offset: (55, 2),
                },
                PapaPlanBloc {
                    kind: BlockKind::SuctionPipe,
                    orientation: BlockOrientation::East,
                    offset: (56, 2),
                },
                PapaPlanBloc {
                    kind: BlockKind::Sortex,
                    orientation: BlockOrientation::East,
                    offset: (57, 0),
                },
                PapaPlanBloc {
                    kind: BlockKind::BlueBagChute,
                    orientation: BlockOrientation::East,
                    offset: (61, 0),
                },
                PapaPlanBloc {
                    kind: BlockKind::RedBagChute,
                    orientation: BlockOrientation::East,
                    offset: (61, 3),
                },
            ],
        }
    }

    #[test]
    fn call_spawns_papa_with_anchor() {
        let cfg = StarterSimConfig::default();
        let sim = crate::sim::FactorySim::new(cfg, 140, 90);
        let world = crate::World::new_room(140, 90);
        let mut etat = PapaEtat::depuis_plan(plan_complet_test());

        let msg = etat
            .declencher_appel(&world, &sim, crate::tile_center((20, 20)))
            .expect("call should succeed");
        assert!(msg.contains("Papa"));
        assert!(etat.pnj().is_some());
    }

    #[test]
    fn tick_builds_functional_modern_line() {
        let cfg = StarterSimConfig {
            starting_cash: 500_000.0,
            ..StarterSimConfig::default()
        };
        let mut sim = crate::sim::FactorySim::new(cfg, 140, 90);
        let world = crate::World::new_room(140, 90);
        let mut etat = PapaEtat::depuis_plan(plan_complet_test());

        etat.declencher_appel(&world, &sim, crate::tile_center((24, 24)))
            .expect("call should succeed");

        for _ in 0..3000 {
            let _ = etat.tick(1.0 / 60.0, &world, &mut sim);
            if etat.pnj().map(|p| p.termine).unwrap_or(false) {
                break;
            }
        }

        assert!(
            etat.pnj().map(|p| p.termine).unwrap_or(false),
            "Papa should finish the line"
        );
        assert!(sim.modern_line_ready(), "modern line should be ready");
    }
}
