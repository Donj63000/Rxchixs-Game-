use super::plan::PapaPlanAsset;
use crate::sim;

fn rects_intersect(
    a_origin: (i32, i32),
    a_size: (i32, i32),
    b_origin: (i32, i32),
    b_size: (i32, i32),
) -> bool {
    let a_max = (a_origin.0 + a_size.0 - 1, a_origin.1 + a_size.1 - 1);
    let b_max = (b_origin.0 + b_size.0 - 1, b_origin.1 + b_size.1 - 1);
    a_origin.0 <= b_max.0 && a_max.0 >= b_origin.0 && a_origin.1 <= b_max.1 && a_max.1 >= b_origin.1
}

pub fn plan_peut_etre_pose(
    plan: &PapaPlanAsset,
    world: &crate::World,
    sim: &sim::FactorySim,
    ancre: (i32, i32),
) -> bool {
    let mut poses_locales: Vec<((i32, i32), (i32, i32))> = Vec::with_capacity(plan.blocs.len());

    for bloc in &plan.blocs {
        let tile = (ancre.0 + bloc.offset.0, ancre.1 + bloc.offset.1);
        let Ok(footprint) = sim.valider_pose_bloc_script(world, bloc.kind, tile, bloc.orientation)
        else {
            return false;
        };

        if poses_locales
            .iter()
            .any(|(origin, size)| rects_intersect(tile, footprint, *origin, *size))
        {
            return false;
        }
        poses_locales.push((tile, footprint));
    }

    true
}

pub fn trouver_ancre_libre(
    plan: &PapaPlanAsset,
    world: &crate::World,
    sim: &sim::FactorySim,
    centre_joueur: (i32, i32),
) -> Option<(i32, i32)> {
    // Bias "devant nous": a droite et legerement au-dessus du joueur.
    let base = (centre_joueur.0 + 8, centre_joueur.1 - 4);

    if plan_peut_etre_pose(plan, world, sim, base) {
        return Some(base);
    }

    let rayon_max = world.w.max(world.h).clamp(20, 220);
    for rayon in 1..=rayon_max {
        // Ring scan, deterministic order.
        let min = -rayon;
        let max = rayon;

        for dx in min..=max {
            let top = (base.0 + dx, base.1 + min);
            if plan_peut_etre_pose(plan, world, sim, top) {
                return Some(top);
            }
            let bottom = (base.0 + dx, base.1 + max);
            if plan_peut_etre_pose(plan, world, sim, bottom) {
                return Some(bottom);
            }
        }
        for dy in (min + 1)..=(max - 1) {
            let left = (base.0 + min, base.1 + dy);
            if plan_peut_etre_pose(plan, world, sim, left) {
                return Some(left);
            }
            let right = (base.0 + max, base.1 + dy);
            if plan_peut_etre_pose(plan, world, sim, right) {
                return Some(right);
            }
        }
    }

    None
}
