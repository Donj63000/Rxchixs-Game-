use super::theme::{feedback_theme, mix_color, world_theme};
use super::*;

pub(crate) fn sim_zone_overlay_color(zone: sim::ZoneKind) -> Option<Color> {
    let feedback = feedback_theme();
    match zone {
        sim::ZoneKind::Neutral => None,
        sim::ZoneKind::Receiving => Some(with_alpha(feedback.logistics, 0.24)),
        sim::ZoneKind::Processing => Some(with_alpha(feedback.warning, 0.25)),
        sim::ZoneKind::Shipping => Some(with_alpha(feedback.positive, 0.22)),
        sim::ZoneKind::Support => Some(with_alpha(feedback.money, 0.22)),
    }
}

pub(crate) fn sim_block_overlay_color(kind: sim::BlockKind) -> Color {
    let world = world_theme();
    match kind {
        sim::BlockKind::InputHopper => mix_color(world.steel_cool, world.prop_pipe_highlight, 0.24),
        sim::BlockKind::Conveyor => rgba(64, 150, 236, 255),
        sim::BlockKind::FluidityTank => rgba(76, 166, 208, 255),
        sim::BlockKind::Cutter => mix_color(world.steel_cool, world.wall_mid, 0.34),
        sim::BlockKind::DistributorBelt => rgba(74, 144, 228, 255),
        sim::BlockKind::DryerOven => mix_color(world.safety_amber, world.prop_crate_dark, 0.24),
        sim::BlockKind::OvenExitConveyor => rgba(96, 142, 214, 255),
        sim::BlockKind::Flaker => mix_color(world.safety_amber, world.steel_cool, 0.42),
        sim::BlockKind::SuctionPipe => mix_color(world.prop_pipe, world.prop_pipe_highlight, 0.26),
        sim::BlockKind::Sortex => rgba(108, 202, 156, 255),
        sim::BlockKind::BlueBagChute => rgba(102, 170, 244, 255),
        sim::BlockKind::RedBagChute => rgba(232, 120, 106, 255),
        sim::BlockKind::Storage => rgba(118, 176, 232, 255),
        sim::BlockKind::MachineA => mix_color(world.steel_cool, world.prop_pipe_highlight, 0.38),
        sim::BlockKind::MachineB => mix_color(world.wall_mid, world.prop_pipe_highlight, 0.34),
        sim::BlockKind::Buffer => mix_color(world.prop_crate_light, world.floor_marking, 0.20),
        sim::BlockKind::Seller => rgba(220, 190, 112, 255),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_overlay_colors_are_stable_and_distinct_for_key_steps() {
        assert_ne!(
            sim_block_overlay_color(sim::BlockKind::Conveyor),
            sim_block_overlay_color(sim::BlockKind::DryerOven)
        );
        assert_ne!(
            sim_block_overlay_color(sim::BlockKind::BlueBagChute),
            sim_block_overlay_color(sim::BlockKind::RedBagChute)
        );
    }

    #[test]
    fn zone_overlay_uses_explicit_visibility() {
        assert!(sim_zone_overlay_color(sim::ZoneKind::Neutral).is_none());
        assert!(sim_zone_overlay_color(sim::ZoneKind::Processing).is_some());
    }
}
