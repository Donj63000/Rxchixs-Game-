use super::etat::PapaPnjEtat;
use super::plan::PapaPlanAsset;
use macroquad::prelude::*;

fn bloc_center_world(tile: (i32, i32), footprint: (i32, i32)) -> Vec2 {
    vec2(
        (tile.0 as f32 + footprint.0 as f32 * 0.5) * crate::TILE_SIZE,
        (tile.1 as f32 + footprint.1 as f32 * 0.5) * crate::TILE_SIZE,
    )
}

pub fn tick_construction(
    pnj: &mut PapaPnjEtat,
    plan: &PapaPlanAsset,
    dt: f32,
    world: &crate::World,
    sim: &mut crate::sim::FactorySim,
) -> Option<String> {
    if pnj.termine || pnj.blocage.is_some() {
        return None;
    }

    pnj.timer_etape_s += dt.max(0.0);
    if pnj.timer_etape_s < plan.delai_etape_s {
        return None;
    }
    pnj.timer_etape_s = 0.0;

    let Some(etape) = plan.blocs.get(pnj.prochaine_etape) else {
        pnj.termine = true;
        pnj.derniere_action = "Ligne Papa terminee".to_string();
        return Some("Papa: ligne de production complete et fonctionnelle.".to_string());
    };

    let tile = (pnj.ancre.0 + etape.offset.0, pnj.ancre.1 + etape.offset.1);
    match sim.poser_bloc_script(world, etape.kind, tile, etape.orientation, false) {
        Ok(_id) => {
            let footprint = etape.kind.footprint_for_orientation(etape.orientation);
            pnj.pos = bloc_center_world(tile, footprint);
            pnj.walk_cycle = (pnj.walk_cycle + dt * 6.0).fract();
            pnj.derniere_action = format!("Pose {}", etape.kind.buyable_label());
            pnj.prochaine_etape = pnj.prochaine_etape.saturating_add(1);

            if let Some(suivante) = plan.blocs.get(pnj.prochaine_etape) {
                let next_tile = (
                    pnj.ancre.0 + suivante.offset.0,
                    pnj.ancre.1 + suivante.offset.1,
                );
                let dir = vec2((next_tile.0 - tile.0) as f32, (next_tile.1 - tile.1) as f32);
                pnj.facing = crate::select_character_facing(dir, pnj.facing);
                pnj.facing_left = dir.x < 0.0;
            }

            if pnj.prochaine_etape >= plan.blocs.len() {
                pnj.termine = true;
                pnj.derniere_action = "Ligne Papa terminee".to_string();
                return Some("Papa: ligne de production complete et fonctionnelle.".to_string());
            }
            None
        }
        Err(err) => {
            let message = format!("Papa bloque: {err}");
            pnj.blocage = Some(message.clone());
            pnj.derniere_action = "Construction interrompue".to_string();
            Some(message)
        }
    }
}
