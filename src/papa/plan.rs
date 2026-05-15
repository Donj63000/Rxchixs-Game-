use crate::sim;
use ron::de::from_str as ron_from_str;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;

pub const PAPA_PLAN_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct PapaPlanAsset {
    pub schema_version: u32,
    pub label: String,
    pub delai_etape_s: f32,
    pub blocs: Vec<PapaPlanBloc>,
}

impl Default for PapaPlanAsset {
    fn default() -> Self {
        Self {
            schema_version: PAPA_PLAN_SCHEMA_VERSION,
            label: "Ligne moderne Papa".to_string(),
            delai_etape_s: 0.42,
            blocs: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PapaPlanBloc {
    pub kind: sim::BlockKind,
    pub orientation: sim::BlockOrientation,
    pub offset: (i32, i32),
}

impl PapaPlanAsset {
    pub fn charger_depuis_fichier(path: &str) -> Result<Self, String> {
        let payload =
            fs::read_to_string(path).map_err(|err| format!("lecture plan Papa echouee: {err}"))?;
        let plan: PapaPlanAsset =
            ron_from_str(&payload).map_err(|err| format!("parse plan Papa echoue: {err}"))?;
        plan.valider()?;
        Ok(plan)
    }

    pub fn valider(&self) -> Result<(), String> {
        if self.schema_version != PAPA_PLAN_SCHEMA_VERSION {
            return Err(format!(
                "schema plan Papa invalide: attendu={} recu={}",
                PAPA_PLAN_SCHEMA_VERSION, self.schema_version
            ));
        }
        if self.blocs.is_empty() {
            return Err("plan Papa vide: aucun bloc a poser".to_string());
        }
        if self.delai_etape_s <= 0.0 || !self.delai_etape_s.is_finite() {
            return Err("delai_etape_s doit etre > 0".to_string());
        }

        let mut kinds = BTreeSet::new();
        for bloc in &self.blocs {
            if !bloc.kind.is_player_buyable() {
                return Err(format!(
                    "plan Papa contient un bloc non achetable: {}",
                    bloc.kind.label()
                ));
            }
            kinds.insert(bloc.kind);
        }

        let requis = [
            sim::BlockKind::InputHopper,
            sim::BlockKind::Conveyor,
            sim::BlockKind::FluidityTank,
            sim::BlockKind::Cutter,
            sim::BlockKind::DistributorBelt,
            sim::BlockKind::DryerOven,
            sim::BlockKind::OvenExitConveyor,
            sim::BlockKind::Flaker,
            sim::BlockKind::SuctionPipe,
            sim::BlockKind::Sortex,
            sim::BlockKind::BlueBagChute,
            sim::BlockKind::RedBagChute,
        ];
        for kind in requis {
            if !kinds.contains(&kind) {
                return Err(format!(
                    "plan Papa incomplet: bloc manquant {}",
                    kind.buyable_label()
                ));
            }
        }

        self.valider_connectivite_fonctionnelle()?;

        Ok(())
    }

    fn valider_connectivite_fonctionnelle(&self) -> Result<(), String> {
        let map_w = 220;
        let map_h = 220;
        let world = crate::World::new_room(map_w, map_h);
        let mut sim = sim::FactorySim::new(sim::StarterSimConfig::default(), map_w, map_h);

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        for bloc in &self.blocs {
            min_x = min_x.min(bloc.offset.0);
            min_y = min_y.min(bloc.offset.1);
        }
        // Evite les murs internes utilitaires de World::new_room (x=12, y=8)
        // pour que la validation de plan teste bien la connectivite metier.
        let anchor_x = 30 - min_x;
        let anchor_y = 30 - min_y;

        for bloc in &self.blocs {
            let tile = (anchor_x + bloc.offset.0, anchor_y + bloc.offset.1);
            sim.poser_bloc_script(&world, bloc.kind, tile, bloc.orientation, false)
                .map_err(|err| format!("plan Papa invalide: pose impossible ({err})"))?;
        }

        if !sim.modern_line_ready() {
            return Err(
                "plan Papa invalide: la sequence ne produit pas une ligne fonctionnelle"
                    .to_string(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_validation_rejects_missing_required_blocks() {
        let plan = PapaPlanAsset {
            schema_version: PAPA_PLAN_SCHEMA_VERSION,
            label: "test".to_string(),
            delai_etape_s: 0.2,
            blocs: vec![PapaPlanBloc {
                kind: sim::BlockKind::InputHopper,
                orientation: sim::BlockOrientation::East,
                offset: (0, 0),
            }],
        };

        let err = plan.valider().expect_err("plan should be invalid");
        assert!(err.contains("bloc manquant"));
    }

    #[test]
    fn plan_validation_rejects_disconnected_layout_even_with_required_kinds() {
        let plan = PapaPlanAsset {
            schema_version: PAPA_PLAN_SCHEMA_VERSION,
            label: "test-broken-connectivity".to_string(),
            delai_etape_s: 0.2,
            blocs: vec![
                PapaPlanBloc {
                    kind: sim::BlockKind::InputHopper,
                    orientation: sim::BlockOrientation::East,
                    offset: (0, 0),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::Conveyor,
                    orientation: sim::BlockOrientation::East,
                    offset: (8, 2),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::FluidityTank,
                    orientation: sim::BlockOrientation::East,
                    offset: (9, 0),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::Conveyor,
                    orientation: sim::BlockOrientation::East,
                    offset: (14, 2),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::Cutter,
                    orientation: sim::BlockOrientation::East,
                    offset: (15, 1),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::DistributorBelt,
                    orientation: sim::BlockOrientation::East,
                    offset: (18, 2),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::DryerOven,
                    orientation: sim::BlockOrientation::East,
                    offset: (25, -3),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::OvenExitConveyor,
                    orientation: sim::BlockOrientation::East,
                    offset: (110, 86),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::Flaker,
                    orientation: sim::BlockOrientation::East,
                    offset: (52, 1),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::SuctionPipe,
                    orientation: sim::BlockOrientation::East,
                    offset: (55, 2),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::SuctionPipe,
                    orientation: sim::BlockOrientation::East,
                    offset: (56, 2),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::Sortex,
                    orientation: sim::BlockOrientation::East,
                    offset: (57, 0),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::BlueBagChute,
                    orientation: sim::BlockOrientation::East,
                    offset: (61, 0),
                },
                PapaPlanBloc {
                    kind: sim::BlockKind::RedBagChute,
                    orientation: sim::BlockOrientation::East,
                    offset: (61, 3),
                },
            ],
        };

        let err = plan
            .valider()
            .expect_err("broken connectivity should be rejected");
        assert!(
            err.contains("ligne fonctionnelle"),
            "unexpected error: {err}"
        );
    }
}
