use super::theme::mix_color;
use super::*;

#[derive(Copy, Clone)]
pub(crate) struct FloorTone {
    pub base_a: Color,
    pub base_b: Color,
    pub accent: Color,
}

#[derive(Copy, Clone)]
pub(crate) struct WallTone {
    pub top: Color,
    pub mid: Color,
    pub dark: Color,
    pub outline: Color,
    pub glow: Color,
}

pub(crate) fn floor_material_variant(x: i32, y: i32) -> u8 {
    (hash_with_salt(x, y, 0x51A9) % 7) as u8
}

pub(crate) fn floor_tones(tile: Tile, exterior_hint: bool, palette: &Palette) -> FloorTone {
    let world = &palette.world;
    match tile {
        Tile::Floor if exterior_hint => FloorTone {
            base_a: mix_color(world.exterior_grass, rgba(42, 92, 52, 255), 0.32),
            base_b: mix_color(world.concrete_moss, rgba(86, 138, 70, 255), 0.26),
            accent: mix_color(rgba(126, 112, 72, 255), world.exterior_grass, 0.42),
        },
        Tile::Floor => FloorTone {
            base_a: world.floor_a,
            base_b: world.floor_b,
            accent: mix_color(world.floor_c, world.steel_cool, 0.22),
        },
        Tile::FloorMetal => FloorTone {
            base_a: mix_color(world.floor_b, world.steel_cool, 0.42),
            base_b: mix_color(world.floor_c, world.steel_deep, 0.52),
            accent: mix_color(world.floor_marking, world.steel_cool, 0.34),
        },
        Tile::FloorWood => FloorTone {
            base_a: rgba(122, 78, 42, 255),
            base_b: rgba(92, 54, 28, 255),
            accent: rgba(166, 108, 58, 255),
        },
        Tile::FloorMoss => FloorTone {
            base_a: rgba(58, 112, 62, 255),
            base_b: rgba(44, 88, 52, 255),
            accent: rgba(118, 144, 78, 255),
        },
        Tile::FloorSand => FloorTone {
            base_a: mix_color(rgba(140, 126, 90, 255), world.exterior_grass, 0.22),
            base_b: mix_color(rgba(112, 102, 78, 255), world.floor_a, 0.18),
            accent: mix_color(world.floor_marking, rgba(170, 154, 104, 255), 0.34),
        },
        _ => FloorTone {
            base_a: world.floor_a,
            base_b: world.floor_b,
            accent: world.floor_c,
        },
    }
}

pub(crate) fn wall_tones(tile: Tile, palette: &Palette) -> WallTone {
    let world = &palette.world;
    match tile {
        Tile::WallBrick => WallTone {
            top: mix_color(world.prop_crate_light, world.wall_top, 0.32),
            mid: mix_color(world.prop_crate_dark, world.wall_mid, 0.38),
            dark: mix_color(world.prop_crate_dark, world.wall_dark, 0.56),
            outline: mix_color(world.wall_outline, world.prop_crate_dark, 0.26),
            glow: mix_color(world.floor_marking, world.prop_crate_light, 0.18),
        },
        Tile::WallSteel => WallTone {
            top: mix_color(world.wall_top, world.steel_cool, 0.46),
            mid: mix_color(world.wall_mid, world.prop_pipe, 0.40),
            dark: mix_color(world.wall_dark, world.steel_deep, 0.36),
            outline: mix_color(world.wall_outline, world.prop_pipe_highlight, 0.14),
            glow: mix_color(world.prop_pipe_highlight, world.steel_cool, 0.26),
        },
        Tile::WallNeon => WallTone {
            top: mix_color(rgba(124, 126, 170, 255), world.wall_top, 0.26),
            mid: mix_color(rgba(88, 94, 146, 255), world.wall_mid, 0.24),
            dark: mix_color(rgba(54, 60, 110, 255), world.wall_dark, 0.20),
            outline: mix_color(rgba(176, 234, 255, 255), world.wall_outline, 0.34),
            glow: mix_color(rgba(132, 252, 234, 255), world.lamp_hot, 0.24),
        },
        _ => WallTone {
            top: world.wall_top,
            mid: world.wall_mid,
            dark: world.wall_dark,
            outline: world.wall_outline,
            glow: world.steel_cool,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_material_variant_is_deterministic_and_varied() {
        assert_eq!(floor_material_variant(3, 7), floor_material_variant(3, 7));
        assert_ne!(floor_material_variant(3, 7), floor_material_variant(4, 7));
    }

    #[test]
    fn floor_tones_keep_metal_distinct_from_standard_concrete() {
        let palette = Palette::new();
        let standard = floor_tones(Tile::Floor, false, &palette);
        let metal = floor_tones(Tile::FloorMetal, false, &palette);
        assert_ne!(standard.base_a, metal.base_a);
        assert_ne!(standard.accent, metal.accent);
    }

    #[test]
    fn wall_tones_keep_neon_glow_distinct() {
        let palette = Palette::new();
        let steel = wall_tones(Tile::WallSteel, &palette);
        let neon = wall_tones(Tile::WallNeon, &palette);
        assert_ne!(steel.glow, neon.glow);
        assert_ne!(steel.outline, neon.outline);
    }
}
