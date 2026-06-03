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
    world: &mut crate::World,
    sim: &mut crate::sim::FactorySim,
) -> Option<String> {
    if pnj.termine || pnj.blocage.is_some() {
        return None;
    }

    if !pnj.sol_prepare {
        match preparer_chantier(pnj, plan, world, sim) {
            Ok((sols, zones)) => {
                pnj.sol_prepare = true;

                if sols > 0 || zones > 0 {
                    pnj.derniere_action =
                        format!("Preparation usine test ({sols} sols, {zones} zones)");
                }
            }
            Err(err) => {
                let message = format!("Papa bloque: {err}");
                pnj.blocage = Some(message.clone());
                pnj.derniere_action = "Preparation chantier interrompue".to_string();
                return Some(message);
            }
        }
    }

    pnj.timer_etape_s += dt.max(0.0);

    if pnj.timer_etape_s < plan.delai_etape_s {
        return None;
    }

    pnj.timer_etape_s = 0.0;

    let Some(etape) = plan.blocs.get(pnj.prochaine_etape) else {
        pnj.termine = true;
        pnj.derniere_action = "Ligne Papa terminee".to_string();

        let setup = sim
            .bootstrap_functional_factory()
            .unwrap_or_else(|err| format!("Bootstrap usine incomplet: {err}"));

        return Some(format!(
            "Papa: ligne de production complete et fonctionnelle. {setup}"
        ));
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

                let setup = sim
                    .bootstrap_functional_factory()
                    .unwrap_or_else(|err| format!("Bootstrap usine incomplet: {err}"));

                return Some(format!(
                    "Papa: ligne de production complete et fonctionnelle. {setup}"
                ));
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

fn preparer_chantier(
    pnj: &PapaPnjEtat,
    plan: &PapaPlanAsset,
    world: &mut crate::World,
    sim: &mut crate::sim::FactorySim,
) -> Result<(usize, usize), String> {
    let sols = preparer_sols_chantier(pnj, plan, world)?;
    let zones = preparer_zones_chantier(pnj, plan, sim)?;

    Ok((sols, zones))
}

fn preparer_sols_chantier(
    pnj: &PapaPnjEtat,
    plan: &PapaPlanAsset,
    world: &mut crate::World,
) -> Result<usize, String> {
    let mut count = 0usize;

    for sol in &plan.sols {
        if sol.size.0 <= 0 || sol.size.1 <= 0 {
            return Err(format!(
                "dalle invalide taille {:?} pour offset {:?}",
                sol.size, sol.offset
            ));
        }

        for dy in 0..sol.size.1 {
            for dx in 0..sol.size.0 {
                let tile = (
                    pnj.ancre.0 + sol.offset.0 + dx,
                    pnj.ancre.1 + sol.offset.1 + dy,
                );

                if !world.in_bounds(tile.0, tile.1) {
                    return Err(format!("dalle hors carte en {:?}", tile));
                }

                if world.is_solid(tile.0, tile.1) {
                    return Err(format!("dalle bloquee par un mur en {:?}", tile));
                }
            }
        }
    }

    for sol in &plan.sols {
        let tile_kind = sol.kind.to_tile();

        for dy in 0..sol.size.1 {
            for dx in 0..sol.size.0 {
                let tile = (
                    pnj.ancre.0 + sol.offset.0 + dx,
                    pnj.ancre.1 + sol.offset.1 + dy,
                );

                world.set(tile.0, tile.1, tile_kind);
                count += 1;
            }
        }
    }

    Ok(count)
}

fn preparer_zones_chantier(
    pnj: &PapaPnjEtat,
    plan: &PapaPlanAsset,
    sim: &mut crate::sim::FactorySim,
) -> Result<usize, String> {
    let mut count = 0usize;

    for zone in &plan.zones {
        let origin = (pnj.ancre.0 + zone.offset.0, pnj.ancre.1 + zone.offset.1);

        count += sim.paint_zone_rect_script(origin, zone.size, zone.kind)?;
    }

    Ok(count)
}
