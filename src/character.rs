use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;

pub const CHARACTER_BASE_SIZE: f32 = 32.0;

const DEFAULT_CATALOG_RON: &str = r#"
(
    mutation_permille: 22,
    body_types: [
        (id: slim, weight: 22),
        (id: standard, weight: 54),
        (id: broad, weight: 24),
    ],
    skin_tones: [
        (id: porcelain, weight: 18),
        (id: warm, weight: 30),
        (id: olive, weight: 27),
        (id: deep, weight: 25),
    ],
    hair_styles: [
        (id: buzz, weight: 16),
        (id: crew, weight: 26),
        (id: ponytail, weight: 19),
        (id: mohawk, weight: 10),
        (id: curly, weight: 16),
        (id: braids, weight: 13),
    ],
    hair_colors: [
        (id: black, weight: 32),
        (id: dark_brown, weight: 28),
        (id: chestnut, weight: 16),
        (id: blonde, weight: 12),
        (id: silver, weight: 7),
        (id: teal_dye, weight: 5),
    ],
    outfit_styles: [
        (id: worker, weight: 30),
        (id: engineer, weight: 28),
        (id: medic, weight: 22),
        (id: scout, weight: 20),
    ],
    outfit_palettes: [
        (id: rust, weight: 24),
        (id: slate, weight: 22),
        (id: moss, weight: 20),
        (id: sand, weight: 17),
        (id: cobalt, weight: 17),
    ],
    accessories: [
        (id: none, weight: 26),
        (id: goggles, weight: 16),
        (id: bandana, weight: 17),
        (id: backpack, weight: 17),
        (id: toolbelt, weight: 14),
        (id: shoulder_pad, weight: 10),
    ],
)
"#;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BodyType {
    Slim,
    Standard,
    Broad,
}

impl BodyType {
    pub fn ui_label(self) -> &'static str {
        match self {
            Self::Slim => "mince",
            Self::Standard => "standard",
            Self::Broad => "costaud",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkinTone {
    Porcelain,
    Warm,
    Olive,
    Deep,
}

impl SkinTone {
    pub fn ui_label(self) -> &'static str {
        match self {
            Self::Porcelain => "porcelaine",
            Self::Warm => "chaud",
            Self::Olive => "olive",
            Self::Deep => "fonce",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HairStyle {
    Buzz,
    Crew,
    Ponytail,
    Mohawk,
    Curly,
    Braids,
}

impl HairStyle {
    pub fn ui_label(self) -> &'static str {
        match self {
            Self::Buzz => "coupe rase",
            Self::Crew => "courte",
            Self::Ponytail => "queue de cheval",
            Self::Mohawk => "iroquoise",
            Self::Curly => "boucles",
            Self::Braids => "tresses",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HairColor {
    Black,
    DarkBrown,
    Chestnut,
    Blonde,
    Silver,
    TealDye,
}

impl HairColor {
    pub fn ui_label(self) -> &'static str {
        match self {
            Self::Black => "noir",
            Self::DarkBrown => "brun fonce",
            Self::Chestnut => "chatain",
            Self::Blonde => "blond",
            Self::Silver => "argent",
            Self::TealDye => "teinte turquoise",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutfitStyle {
    Worker,
    Engineer,
    Medic,
    Scout,
}

impl OutfitStyle {
    pub fn ui_label(self) -> &'static str {
        match self {
            Self::Worker => "ouvrier",
            Self::Engineer => "ingenieur",
            Self::Medic => "medecin",
            Self::Scout => "eclaireur",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutfitPalette {
    Rust,
    Slate,
    Moss,
    Sand,
    Cobalt,
}

impl OutfitPalette {
    pub fn ui_label(self) -> &'static str {
        match self {
            Self::Rust => "rouille",
            Self::Slate => "ardoise",
            Self::Moss => "mousse",
            Self::Sand => "sable",
            Self::Cobalt => "cobalt",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Accessory {
    None,
    Goggles,
    Bandana,
    Backpack,
    Toolbelt,
    ShoulderPad,
}

impl Accessory {
    pub fn ui_label(self) -> &'static str {
        match self {
            Self::None => "aucun",
            Self::Goggles => "lunettes",
            Self::Bandana => "bandana",
            Self::Backpack => "sac a dos",
            Self::Toolbelt => "ceinture outils",
            Self::ShoulderPad => "epauliere",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraitGene<T: Copy + Eq> {
    pub a: T,
    pub b: T,
}

impl<T: Copy + Eq> TraitGene<T> {
    fn new(a: T, b: T) -> Self {
        Self { a, b }
    }

    fn pick(self, rng: &mut SeedRng) -> T {
        if rng.next_u32() & 1 == 0 {
            self.a
        } else {
            self.b
        }
    }

    #[cfg(test)]
    fn contains(self, value: T) -> bool {
        self.a == value || self.b == value
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterDna {
    pub body_type: TraitGene<BodyType>,
    pub skin_tone: TraitGene<SkinTone>,
    pub hair_style: TraitGene<HairStyle>,
    pub hair_color: TraitGene<HairColor>,
    pub outfit_style: TraitGene<OutfitStyle>,
    pub outfit_palette: TraitGene<OutfitPalette>,
    pub accessory: TraitGene<Accessory>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharacterVisual {
    pub body_type: BodyType,
    pub skin_tone: SkinTone,
    pub hair_style: HairStyle,
    pub hair_color: HairColor,
    pub outfit_style: OutfitStyle,
    pub outfit_palette: OutfitPalette,
    pub accessory: Accessory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterRecord {
    pub label: String,
    pub seed: u64,
    pub generation: u8,
    pub dna: CharacterDna,
    pub visual: CharacterVisual,
}

#[derive(Clone, Copy, Debug)]
pub struct CharacterRenderParams {
    pub center: Vec2,
    pub scale: f32,
    pub facing: CharacterFacing,
    pub facing_left: bool,
    pub is_walking: bool,
    pub walk_cycle: f32,
    pub gesture: CharacterGesture,
    pub time: f32,
    pub debug: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterFacing {
    Front,
    Side,
    Back,
}

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

#[derive(Clone, Debug, Deserialize)]
struct CharacterCatalogRaw {
    mutation_permille: u16,
    body_types: Vec<WeightedEntry<BodyType>>,
    skin_tones: Vec<WeightedEntry<SkinTone>>,
    hair_styles: Vec<WeightedEntry<HairStyle>>,
    hair_colors: Vec<WeightedEntry<HairColor>>,
    outfit_styles: Vec<WeightedEntry<OutfitStyle>>,
    outfit_palettes: Vec<WeightedEntry<OutfitPalette>>,
    accessories: Vec<WeightedEntry<Accessory>>,
}

#[derive(Clone, Debug, Deserialize)]
struct WeightedEntry<T> {
    id: T,
    weight: u32,
}

#[derive(Clone, Debug)]
pub struct CharacterCatalog {
    mutation_permille: u16,
    body_types: Vec<(BodyType, u32)>,
    skin_tones: Vec<(SkinTone, u32)>,
    hair_styles: Vec<(HairStyle, u32)>,
    hair_colors: Vec<(HairColor, u32)>,
    outfit_styles: Vec<(OutfitStyle, u32)>,
    outfit_palettes: Vec<(OutfitPalette, u32)>,
    accessories: Vec<(Accessory, u32)>,
}

impl CharacterCatalog {
    pub fn load_default() -> Result<Self, String> {
        Self::from_ron(DEFAULT_CATALOG_RON)
    }

    pub fn from_ron(raw: &str) -> Result<Self, String> {
        let parsed: CharacterCatalogRaw =
            ron::from_str(raw).map_err(|e| format!("catalog parse error: {e}"))?;
        let catalog = Self {
            mutation_permille: parsed.mutation_permille.min(300),
            body_types: convert_weighted(parsed.body_types, "body_types")?,
            skin_tones: convert_weighted(parsed.skin_tones, "skin_tones")?,
            hair_styles: convert_weighted(parsed.hair_styles, "hair_styles")?,
            hair_colors: convert_weighted(parsed.hair_colors, "hair_colors")?,
            outfit_styles: convert_weighted(parsed.outfit_styles, "outfit_styles")?,
            outfit_palettes: convert_weighted(parsed.outfit_palettes, "outfit_palettes")?,
            accessories: convert_weighted(parsed.accessories, "accessories")?,
        };
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn mutation_permille(&self) -> u16 {
        self.mutation_permille
    }

    pub fn spawn_founder(&self, label: &str, seed: u64) -> CharacterRecord {
        let dna = self.generate_founder_dna(seed);
        let visual = self.express_visual(&dna, seed ^ 0xA5A5_A5A5_A5A5_A5A5);
        CharacterRecord {
            label: label.to_string(),
            seed,
            generation: 0,
            dna,
            visual,
        }
    }

    pub fn spawn_child(
        &self,
        label: &str,
        parent_a: &CharacterRecord,
        parent_b: &CharacterRecord,
        seed: u64,
    ) -> CharacterRecord {
        let dna = self.inherit_dna_with_mutation(
            &parent_a.dna,
            &parent_b.dna,
            seed,
            self.mutation_permille,
        );
        let visual = self.express_visual(&dna, seed ^ 0x3C3C_3C3C_3C3C_3C3C);
        CharacterRecord {
            label: label.to_string(),
            seed,
            generation: parent_a
                .generation
                .max(parent_b.generation)
                .saturating_add(1),
            dna,
            visual,
        }
    }

    #[cfg(test)]
    pub(crate) fn inherit_dna_for_test(
        &self,
        parent_a: &CharacterDna,
        parent_b: &CharacterDna,
        seed: u64,
        mutation_permille: u16,
    ) -> CharacterDna {
        self.inherit_dna_with_mutation(parent_a, parent_b, seed, mutation_permille)
    }

    fn validate(&self) -> Result<(), String> {
        validate_weighted(&self.body_types, "body_types")?;
        validate_weighted(&self.skin_tones, "skin_tones")?;
        validate_weighted(&self.hair_styles, "hair_styles")?;
        validate_weighted(&self.hair_colors, "hair_colors")?;
        validate_weighted(&self.outfit_styles, "outfit_styles")?;
        validate_weighted(&self.outfit_palettes, "outfit_palettes")?;
        validate_weighted(&self.accessories, "accessories")?;
        Ok(())
    }

    fn generate_founder_dna(&self, seed: u64) -> CharacterDna {
        let mut rng = SeedRng::new(seed);
        CharacterDna {
            body_type: TraitGene::new(self.roll_body_type(&mut rng), self.roll_body_type(&mut rng)),
            skin_tone: TraitGene::new(self.roll_skin_tone(&mut rng), self.roll_skin_tone(&mut rng)),
            hair_style: TraitGene::new(
                self.roll_hair_style(&mut rng),
                self.roll_hair_style(&mut rng),
            ),
            hair_color: TraitGene::new(
                self.roll_hair_color(&mut rng),
                self.roll_hair_color(&mut rng),
            ),
            outfit_style: TraitGene::new(
                self.roll_outfit_style(&mut rng),
                self.roll_outfit_style(&mut rng),
            ),
            outfit_palette: TraitGene::new(
                self.roll_outfit_palette(&mut rng),
                self.roll_outfit_palette(&mut rng),
            ),
            accessory: TraitGene::new(self.roll_accessory(&mut rng), self.roll_accessory(&mut rng)),
        }
    }

    fn express_visual(&self, dna: &CharacterDna, seed: u64) -> CharacterVisual {
        let mut rng = SeedRng::new(seed ^ 0xD0D0_5EED_1337_CAFE);
        CharacterVisual {
            body_type: dna.body_type.pick(&mut rng),
            skin_tone: dna.skin_tone.pick(&mut rng),
            hair_style: dna.hair_style.pick(&mut rng),
            hair_color: dna.hair_color.pick(&mut rng),
            outfit_style: dna.outfit_style.pick(&mut rng),
            outfit_palette: dna.outfit_palette.pick(&mut rng),
            accessory: dna.accessory.pick(&mut rng),
        }
    }

    fn inherit_dna_with_mutation(
        &self,
        parent_a: &CharacterDna,
        parent_b: &CharacterDna,
        seed: u64,
        mutation_permille: u16,
    ) -> CharacterDna {
        let mut rng = SeedRng::new(seed ^ 0xBADC_0FFE_F00D_A11A);
        let mutation = mutation_permille.min(300);

        let mut body_type = TraitGene::new(
            parent_a.body_type.pick(&mut rng),
            parent_b.body_type.pick(&mut rng),
        );
        maybe_mutate(&mut body_type.a, mutation, &mut rng, |r| {
            self.roll_body_type(r)
        });
        maybe_mutate(&mut body_type.b, mutation, &mut rng, |r| {
            self.roll_body_type(r)
        });

        let mut skin_tone = TraitGene::new(
            parent_a.skin_tone.pick(&mut rng),
            parent_b.skin_tone.pick(&mut rng),
        );
        maybe_mutate(&mut skin_tone.a, mutation, &mut rng, |r| {
            self.roll_skin_tone(r)
        });
        maybe_mutate(&mut skin_tone.b, mutation, &mut rng, |r| {
            self.roll_skin_tone(r)
        });

        let mut hair_style = TraitGene::new(
            parent_a.hair_style.pick(&mut rng),
            parent_b.hair_style.pick(&mut rng),
        );
        maybe_mutate(&mut hair_style.a, mutation, &mut rng, |r| {
            self.roll_hair_style(r)
        });
        maybe_mutate(&mut hair_style.b, mutation, &mut rng, |r| {
            self.roll_hair_style(r)
        });

        let mut hair_color = TraitGene::new(
            parent_a.hair_color.pick(&mut rng),
            parent_b.hair_color.pick(&mut rng),
        );
        maybe_mutate(&mut hair_color.a, mutation, &mut rng, |r| {
            self.roll_hair_color(r)
        });
        maybe_mutate(&mut hair_color.b, mutation, &mut rng, |r| {
            self.roll_hair_color(r)
        });

        let mut outfit_style = TraitGene::new(
            parent_a.outfit_style.pick(&mut rng),
            parent_b.outfit_style.pick(&mut rng),
        );
        maybe_mutate(&mut outfit_style.a, mutation, &mut rng, |r| {
            self.roll_outfit_style(r)
        });
        maybe_mutate(&mut outfit_style.b, mutation, &mut rng, |r| {
            self.roll_outfit_style(r)
        });

        let mut outfit_palette = TraitGene::new(
            parent_a.outfit_palette.pick(&mut rng),
            parent_b.outfit_palette.pick(&mut rng),
        );
        maybe_mutate(&mut outfit_palette.a, mutation, &mut rng, |r| {
            self.roll_outfit_palette(r)
        });
        maybe_mutate(&mut outfit_palette.b, mutation, &mut rng, |r| {
            self.roll_outfit_palette(r)
        });

        let mut accessory = TraitGene::new(
            parent_a.accessory.pick(&mut rng),
            parent_b.accessory.pick(&mut rng),
        );
        maybe_mutate(&mut accessory.a, mutation, &mut rng, |r| {
            self.roll_accessory(r)
        });
        maybe_mutate(&mut accessory.b, mutation, &mut rng, |r| {
            self.roll_accessory(r)
        });

        CharacterDna {
            body_type,
            skin_tone,
            hair_style,
            hair_color,
            outfit_style,
            outfit_palette,
            accessory,
        }
    }

    fn roll_body_type(&self, rng: &mut SeedRng) -> BodyType {
        pick_weighted(rng, &self.body_types)
    }

    fn roll_skin_tone(&self, rng: &mut SeedRng) -> SkinTone {
        pick_weighted(rng, &self.skin_tones)
    }

    fn roll_hair_style(&self, rng: &mut SeedRng) -> HairStyle {
        pick_weighted(rng, &self.hair_styles)
    }

    fn roll_hair_color(&self, rng: &mut SeedRng) -> HairColor {
        pick_weighted(rng, &self.hair_colors)
    }

    fn roll_outfit_style(&self, rng: &mut SeedRng) -> OutfitStyle {
        pick_weighted(rng, &self.outfit_styles)
    }

    fn roll_outfit_palette(&self, rng: &mut SeedRng) -> OutfitPalette {
        pick_weighted(rng, &self.outfit_palettes)
    }

    fn roll_accessory(&self, rng: &mut SeedRng) -> Accessory {
        pick_weighted(rng, &self.accessories)
    }
}

pub fn build_lineage_preview(catalog: &CharacterCatalog, root_seed: u64) -> Vec<CharacterRecord> {
    let founder_a = catalog.spawn_founder("Fondateur-A", mix_seed(root_seed, 1));
    let founder_b = catalog.spawn_founder("Fondateur-B", mix_seed(root_seed, 2));
    let player = catalog.spawn_child("Joueur", &founder_a, &founder_b, mix_seed(root_seed, 3));
    let gen2_a = catalog.spawn_child("Gen2-A", &player, &founder_b, mix_seed(root_seed, 4));
    let gen2_b = catalog.spawn_child("Gen2-B", &founder_a, &player, mix_seed(root_seed, 5));
    vec![founder_a, founder_b, player, gen2_a, gen2_b]
}

pub fn compact_visual_summary(record: &CharacterRecord) -> String {
    format!(
        "{} {} {} {}",
        record.visual.body_type.ui_label(),
        record.visual.hair_style.ui_label(),
        record.visual.outfit_style.ui_label(),
        record.visual.accessory.ui_label(),
    )
}

pub fn inspector_lines(record: &CharacterRecord) -> Vec<String> {
    vec![
        format!(
            "{} (g{}) seed={:#X}",
            record.label, record.generation, record.seed
        ),
        gene_line(
            "corps",
            record.dna.body_type,
            record.visual.body_type,
            BodyType::ui_label,
        ),
        gene_line(
            "peau",
            record.dna.skin_tone,
            record.visual.skin_tone,
            SkinTone::ui_label,
        ),
        gene_line(
            "coiffure",
            record.dna.hair_style,
            record.visual.hair_style,
            HairStyle::ui_label,
        ),
        gene_line(
            "couleur_cheveux",
            record.dna.hair_color,
            record.visual.hair_color,
            HairColor::ui_label,
        ),
        gene_line(
            "tenue",
            record.dna.outfit_style,
            record.visual.outfit_style,
            OutfitStyle::ui_label,
        ),
        gene_line(
            "palette",
            record.dna.outfit_palette,
            record.visual.outfit_palette,
            OutfitPalette::ui_label,
        ),
        gene_line(
            "accessoire",
            record.dna.accessory,
            record.visual.accessory,
            Accessory::ui_label,
        ),
    ]
}

pub fn draw_character(record: &CharacterRecord, params: CharacterRenderParams) {
    let visual = record.visual;
    let scale = params.scale.max(0.2);
    let xform = CharacterCanvas::new(params.center, scale, params.facing_left);
    let skin = skin_color(visual.skin_tone);
    let hair = hair_color(visual.hair_color);
    let eyes = shade(hair, 0.35);
    let outfit = outfit_colors(visual.outfit_style, visual.outfit_palette);
    let metrics = body_metrics(visual.body_type);
    let facing = params.facing;
    let walk_phase = if params.is_walking {
        params.walk_cycle.sin()
    } else {
        0.0
    };
    let stride = walk_phase * (0.8 + params.walk_cycle.cos().abs() * 0.25);
    let idle_wave = (params.time * 2.1 + record.seed as f32 * 0.0003).sin() * 0.22;
    let gesture = params.gesture;
    let g_phase = (params.time * 7.5 + record.seed as f32 * 0.0009).sin();
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
    let g_anim = if params.is_walking {
        0.0
    } else {
        g_phase * g_power
    };
    let bob = stride * 0.24 + idle_wave + g_anim * 0.35;

    draw_circle(
        params.center.x,
        params.center.y + 10.2 * scale,
        7.2 * scale,
        Color::new(0.02, 0.03, 0.05, 0.34),
    );

    if matches!(visual.accessory, Accessory::Backpack) || matches!(facing, CharacterFacing::Back) {
        let back = xform.rect(8.3, 12.2 + bob, 4.0, 10.8);
        draw_rectangle(back.x, back.y, back.w, back.h, shade(outfit.base, 0.7));
        draw_rectangle_lines(
            back.x + 0.2,
            back.y + 0.2,
            (back.w - 0.4).max(0.1),
            (back.h - 0.4).max(0.1),
            1.0 * scale,
            shade(outfit.trim, 0.7),
        );
    }

    match facing {
        CharacterFacing::Side => {
            let far_leg = xform.rect(
                13.7 - stride * 0.25,
                20.3 - stride * 0.22,
                metrics.leg_w - 0.2,
                8.4,
            );
            let near_leg = xform.rect(
                16.4 + stride * 0.25,
                20.0 + stride * 0.22,
                metrics.leg_w + 0.25,
                8.9,
            );
            draw_rectangle(
                far_leg.x,
                far_leg.y,
                far_leg.w,
                far_leg.h,
                shade(outfit.base, 0.72),
            );
            draw_rectangle(
                near_leg.x,
                near_leg.y,
                near_leg.w,
                near_leg.h,
                shade(outfit.base, 0.88),
            );
        }
        CharacterFacing::Front | CharacterFacing::Back => {
            let left_leg = xform.rect(
                12.0 + stride * 0.75,
                20.0 + stride.abs() * 0.20,
                metrics.leg_w,
                8.8,
            );
            let right_leg = xform.rect(
                16.0 - stride * 0.75,
                20.0 + (1.0 - stride.abs()) * 0.28,
                metrics.leg_w,
                8.8,
            );
            let left_leg_color = if matches!(facing, CharacterFacing::Back) {
                shade(outfit.base, 0.70)
            } else {
                shade(outfit.base, 0.82)
            };
            let right_leg_color = if matches!(facing, CharacterFacing::Back) {
                shade(outfit.base, 0.78)
            } else {
                shade(outfit.base, 0.88)
            };
            draw_rectangle(
                left_leg.x,
                left_leg.y,
                left_leg.w,
                left_leg.h,
                left_leg_color,
            );
            draw_rectangle(
                right_leg.x,
                right_leg.y,
                right_leg.w,
                right_leg.h,
                right_leg_color,
            );
        }
    }

    let torso = xform.rect(
        16.0 - metrics.torso_w * 0.5,
        12.5 + bob,
        metrics.torso_w,
        metrics.torso_h,
    );
    let torso_base = match facing {
        CharacterFacing::Back => shade(outfit.base, 0.82),
        _ => outfit.base,
    };
    draw_rectangle(torso.x, torso.y, torso.w, torso.h, torso_base);
    draw_rectangle(
        torso.x + 1.2 * scale,
        torso.y + 1.2 * scale,
        (torso.w - 2.4 * scale).max(0.1),
        (torso.h - 2.4 * scale).max(0.1),
        match facing {
            CharacterFacing::Back => shade(outfit.trim, 0.92),
            _ => shade(outfit.trim, 1.02),
        },
    );
    draw_torso_style(visual.outfit_style, &xform, torso, outfit, bob, facing);

    match facing {
        CharacterFacing::Side => {
            let (near_extra, far_extra) = if !params.is_walking {
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

            let far_arm = xform.rect(
                14.0,
                13.8 + bob - stride * 0.22 - far_extra,
                2.2,
                7.0 + stride.abs() * 0.4,
            );
            let near_arm = xform.rect(
                17.4,
                13.8 + bob + stride * 0.34 - near_extra,
                2.5,
                7.6 + stride.abs() * 0.4,
            );
            draw_rectangle(
                far_arm.x,
                far_arm.y,
                far_arm.w,
                far_arm.h,
                shade(outfit.base, 0.68),
            );
            draw_rectangle(
                near_arm.x,
                near_arm.y,
                near_arm.w,
                near_arm.h,
                shade(outfit.base, 0.88),
            );
            let near_hand = xform.point(18.6, 21.4 + bob + stride * 0.24);
            draw_circle(near_hand.x, near_hand.y, 1.35 * scale, shade(skin, 0.9));
        }
        CharacterFacing::Front | CharacterFacing::Back => {
            let (left_extra, right_extra) = if !params.is_walking {
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
                16.0 - metrics.shoulder_w * 0.5 - 2.2,
                13.6 + bob + stride * 0.28 - left_extra,
                2.4,
                7.5,
            );
            let right_arm = xform.rect(
                16.0 + metrics.shoulder_w * 0.5 - 0.2,
                13.6 + bob - stride * 0.28 - right_extra,
                2.4,
                7.5,
            );
            let left_arm_color = if matches!(facing, CharacterFacing::Back) {
                shade(outfit.base, 0.66)
            } else {
                shade(outfit.base, 0.8)
            };
            let right_arm_color = if matches!(facing, CharacterFacing::Back) {
                shade(outfit.base, 0.74)
            } else {
                shade(outfit.base, 0.85)
            };
            draw_rectangle(
                left_arm.x,
                left_arm.y,
                left_arm.w,
                left_arm.h,
                left_arm_color,
            );
            draw_rectangle(
                right_arm.x,
                right_arm.y,
                right_arm.w,
                right_arm.h,
                right_arm_color,
            );

            if matches!(facing, CharacterFacing::Front) {
                let left_hand = xform.point(16.0 - metrics.shoulder_w * 0.5 - 1.0, 21.0 + bob);
                let right_hand = xform.point(16.0 + metrics.shoulder_w * 0.5 + 1.0, 21.0 + bob);
                draw_circle(left_hand.x, left_hand.y, 1.3 * scale, shade(skin, 0.9));
                draw_circle(right_hand.x, right_hand.y, 1.3 * scale, shade(skin, 0.9));
            }
        }
    }

    let head_center = xform.point(16.0, 8.3 + bob);
    draw_circle(head_center.x, head_center.y, metrics.head_r * scale, skin);
    match facing {
        CharacterFacing::Front => {
            draw_circle(
                head_center.x - 1.2 * scale,
                head_center.y + 1.1 * scale,
                0.75 * scale,
                eyes,
            );
            draw_circle(
                head_center.x + 1.2 * scale,
                head_center.y + 1.1 * scale,
                0.75 * scale,
                eyes,
            );
            let mouth_col = shade(skin, 0.62);
            let talking = !params.is_walking
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
                let m = (params.time * 10.0 + record.seed as f32 * 0.0014).sin();
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
        }
        CharacterFacing::Side => {
            let dir = if params.facing_left { -1.0 } else { 1.0 };
            draw_circle(
                head_center.x + dir * 0.95 * scale,
                head_center.y + 1.1 * scale,
                0.72 * scale,
                eyes,
            );
            draw_line(
                head_center.x + dir * 1.6 * scale,
                head_center.y + 1.9 * scale,
                head_center.x + dir * 2.4 * scale,
                head_center.y + 2.2 * scale,
                1.0 * scale,
                shade(skin, 0.58),
            );
        }
        CharacterFacing::Back => {
            draw_circle(
                head_center.x,
                head_center.y + 1.3 * scale,
                0.9 * scale,
                shade(skin, 0.88),
            );
        }
    }

    draw_hair(
        visual.hair_style,
        &xform,
        head_center,
        metrics.head_r * scale,
        hair,
        bob,
        facing,
    );
    draw_accessory(
        visual.accessory,
        &xform,
        head_center,
        metrics,
        outfit,
        bob,
        facing,
    );

    if params.debug {
        let box_rect = xform.rect(9.0, 3.0 + bob, 14.0, 25.0);
        draw_rectangle_lines(
            box_rect.x,
            box_rect.y,
            box_rect.w,
            box_rect.h,
            1.0,
            Color::new(0.1, 1.0, 0.2, 0.7),
        );
    }
}

fn gene_line<T: Copy + Eq>(
    label: &str,
    gene: TraitGene<T>,
    shown: T,
    to_str: fn(T) -> &'static str,
) -> String {
    format!(
        "{label}: {}|{} -> {}",
        to_str(gene.a),
        to_str(gene.b),
        to_str(shown)
    )
}

fn convert_weighted<T: Copy + Eq + std::hash::Hash>(
    entries: Vec<WeightedEntry<T>>,
    field: &str,
) -> Result<Vec<(T, u32)>, String> {
    if entries.is_empty() {
        return Err(format!("{field} cannot be empty"));
    }
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        if entry.weight == 0 {
            return Err(format!("{field} has a zero weight entry"));
        }
        if !seen.insert(entry.id) {
            return Err(format!("{field} contains duplicated ids"));
        }
        out.push((entry.id, entry.weight));
    }
    Ok(out)
}

fn validate_weighted<T: Copy + Eq + std::hash::Hash>(
    entries: &[(T, u32)],
    field: &str,
) -> Result<(), String> {
    if entries.is_empty() {
        return Err(format!("{field} cannot be empty"));
    }
    let mut seen = HashSet::new();
    for (id, weight) in entries {
        if *weight == 0 {
            return Err(format!("{field} has a zero weight entry"));
        }
        if !seen.insert(*id) {
            return Err(format!("{field} contains duplicated ids"));
        }
    }
    Ok(())
}

fn pick_weighted<T: Copy>(rng: &mut SeedRng, entries: &[(T, u32)]) -> T {
    let total: u32 = entries.iter().map(|(_, weight)| *weight).sum();
    let mut roll = rng.roll_range(total);
    for (value, weight) in entries {
        if roll < *weight {
            return *value;
        }
        roll -= *weight;
    }
    entries[entries.len() - 1].0
}

fn maybe_mutate<T: Copy>(
    allele: &mut T,
    mutation_permille: u16,
    rng: &mut SeedRng,
    mut roll: impl FnMut(&mut SeedRng) -> T,
) {
    if mutation_permille > 0 && rng.chance_permille(mutation_permille) {
        *allele = roll(rng);
    }
}

fn mix_seed(seed: u64, salt: u64) -> u64 {
    let mut x = seed ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[derive(Clone, Copy, Debug)]
struct SeedRng {
    state: u64,
}

impl SeedRng {
    fn new(seed: u64) -> Self {
        let base = if seed == 0 {
            0xA409_3822_299F_31D0
        } else {
            seed
        };
        Self {
            state: base ^ 0x517C_C1B7_2722_0A95,
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn roll_range(&mut self, upper: u32) -> u32 {
        if upper <= 1 {
            0
        } else {
            (self.next_u64() % upper as u64) as u32
        }
    }

    fn chance_permille(&mut self, rate: u16) -> bool {
        self.roll_range(1000) < u32::from(rate)
    }
}

#[derive(Clone, Copy)]
struct OutfitColors {
    base: Color,
    trim: Color,
    accent: Color,
}

#[derive(Clone, Copy)]
struct BodyMetrics {
    torso_w: f32,
    torso_h: f32,
    shoulder_w: f32,
    leg_w: f32,
    head_r: f32,
}

#[derive(Clone, Copy)]
struct CharacterCanvas {
    center: Vec2,
    scale: f32,
    facing_left: bool,
}

impl CharacterCanvas {
    fn new(center: Vec2, scale: f32, facing_left: bool) -> Self {
        Self {
            center,
            scale,
            facing_left,
        }
    }

    fn x(self, local_x: f32, width: f32) -> f32 {
        let mirrored = if self.facing_left {
            CHARACTER_BASE_SIZE - local_x - width
        } else {
            local_x
        };
        self.center.x + (mirrored - CHARACTER_BASE_SIZE * 0.5) * self.scale
    }

    fn y(self, local_y: f32) -> f32 {
        self.center.y + (local_y - CHARACTER_BASE_SIZE * 0.5) * self.scale
    }

    fn point(self, local_x: f32, local_y: f32) -> Vec2 {
        let mirrored = if self.facing_left {
            CHARACTER_BASE_SIZE - local_x
        } else {
            local_x
        };
        vec2(
            self.center.x + (mirrored - CHARACTER_BASE_SIZE * 0.5) * self.scale,
            self.center.y + (local_y - CHARACTER_BASE_SIZE * 0.5) * self.scale,
        )
    }

    fn rect(self, local_x: f32, local_y: f32, width: f32, height: f32) -> Rect {
        Rect::new(
            self.x(local_x, width),
            self.y(local_y),
            width * self.scale,
            height * self.scale,
        )
    }
}

fn body_metrics(body_type: BodyType) -> BodyMetrics {
    match body_type {
        BodyType::Slim => BodyMetrics {
            torso_w: 7.6,
            torso_h: 8.7,
            shoulder_w: 8.8,
            leg_w: 2.5,
            head_r: 4.5,
        },
        BodyType::Standard => BodyMetrics {
            torso_w: 8.8,
            torso_h: 9.4,
            shoulder_w: 10.0,
            leg_w: 2.7,
            head_r: 4.7,
        },
        BodyType::Broad => BodyMetrics {
            torso_w: 10.0,
            torso_h: 9.6,
            shoulder_w: 11.2,
            leg_w: 2.9,
            head_r: 4.8,
        },
    }
}

fn skin_color(skin: SkinTone) -> Color {
    match skin {
        SkinTone::Porcelain => rgb(243, 212, 192),
        SkinTone::Warm => rgb(219, 177, 146),
        SkinTone::Olive => rgb(182, 142, 110),
        SkinTone::Deep => rgb(124, 92, 72),
    }
}

fn hair_color(color: HairColor) -> Color {
    match color {
        HairColor::Black => rgb(35, 37, 46),
        HairColor::DarkBrown => rgb(67, 50, 41),
        HairColor::Chestnut => rgb(116, 78, 56),
        HairColor::Blonde => rgb(213, 179, 96),
        HairColor::Silver => rgb(180, 184, 191),
        HairColor::TealDye => rgb(62, 158, 163),
    }
}

fn outfit_colors(style: OutfitStyle, palette: OutfitPalette) -> OutfitColors {
    let base = match palette {
        OutfitPalette::Rust => rgb(126, 83, 70),
        OutfitPalette::Slate => rgb(79, 93, 112),
        OutfitPalette::Moss => rgb(88, 109, 81),
        OutfitPalette::Sand => rgb(152, 132, 98),
        OutfitPalette::Cobalt => rgb(74, 101, 150),
    };
    let trim = shade(base, 0.82);
    let accent = match style {
        OutfitStyle::Worker => rgb(223, 182, 94),
        OutfitStyle::Engineer => rgb(110, 188, 206),
        OutfitStyle::Medic => rgb(210, 93, 96),
        OutfitStyle::Scout => rgb(150, 210, 121),
    };

    OutfitColors { base, trim, accent }
}

fn draw_torso_style(
    style: OutfitStyle,
    xform: &CharacterCanvas,
    torso: Rect,
    outfit: OutfitColors,
    bob: f32,
    facing: CharacterFacing,
) {
    if matches!(facing, CharacterFacing::Back) {
        draw_line(
            torso.x + torso.w * 0.22,
            torso.y + torso.h * 0.22,
            torso.x + torso.w * 0.78,
            torso.y + torso.h * 0.22,
            1.0 * xform.scale,
            shade(outfit.trim, 0.72),
        );
        draw_line(
            torso.x + torso.w * 0.5,
            torso.y + torso.h * 0.22,
            torso.x + torso.w * 0.5,
            torso.y + torso.h * 0.84,
            1.0 * xform.scale,
            shade(outfit.trim, 0.62),
        );
        return;
    }

    match style {
        OutfitStyle::Worker => {
            let pocket_l = xform.rect(11.5, 16.2 + bob, 2.2, 2.1);
            let pocket_r = xform.rect(18.3, 16.2 + bob, 2.2, 2.1);
            draw_rectangle(
                pocket_l.x,
                pocket_l.y,
                pocket_l.w,
                pocket_l.h,
                shade(outfit.trim, 0.95),
            );
            draw_rectangle(
                pocket_r.x,
                pocket_r.y,
                pocket_r.w,
                pocket_r.h,
                shade(outfit.trim, 0.95),
            );
            draw_line(
                torso.x + torso.w * 0.5,
                torso.y + 1.2 * xform.scale,
                torso.x + torso.w * 0.5,
                torso.y + torso.h - 1.2 * xform.scale,
                1.0 * xform.scale,
                shade(outfit.trim, 0.7),
            );
        }
        OutfitStyle::Engineer => {
            let stripe = xform.rect(13.2, 13.9 + bob, 5.8, 1.5);
            draw_rectangle(stripe.x, stripe.y, stripe.w, stripe.h, outfit.accent);
            draw_line(
                torso.x + 1.2 * xform.scale,
                torso.y + torso.h - 2.0 * xform.scale,
                torso.x + torso.w - 1.2 * xform.scale,
                torso.y + torso.h - 2.0 * xform.scale,
                1.0 * xform.scale,
                shade(outfit.trim, 0.66),
            );
        }
        OutfitStyle::Medic => {
            let cross_v = xform.rect(15.4, 14.4 + bob, 1.4, 4.2);
            let cross_h = xform.rect(14.0, 15.8 + bob, 4.2, 1.4);
            draw_rectangle(cross_v.x, cross_v.y, cross_v.w, cross_v.h, outfit.accent);
            draw_rectangle(cross_h.x, cross_h.y, cross_h.w, cross_h.h, outfit.accent);
        }
        OutfitStyle::Scout => {
            let scarf = xform.rect(12.2, 12.8 + bob, 7.6, 2.0);
            draw_rectangle(scarf.x, scarf.y, scarf.w, scarf.h, outfit.accent);
            draw_line(
                scarf.x + scarf.w * 0.8,
                scarf.y + scarf.h,
                scarf.x + scarf.w * 0.8,
                scarf.y + scarf.h + 3.0 * xform.scale,
                1.0 * xform.scale,
                shade(outfit.accent, 0.8),
            );
        }
    }
}

fn draw_hair(
    style: HairStyle,
    xform: &CharacterCanvas,
    head_center: Vec2,
    head_r: f32,
    hair: Color,
    bob: f32,
    facing: CharacterFacing,
) {
    let dir = if xform.facing_left { -1.0 } else { 1.0 };
    let side_shift = if matches!(facing, CharacterFacing::Side) {
        dir * head_r * 0.18
    } else {
        0.0
    };
    let back_boost = if matches!(facing, CharacterFacing::Back) {
        1.08
    } else {
        1.0
    };

    match style {
        HairStyle::Buzz => {
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.65,
                head_r * 0.58 * back_boost,
                hair,
            );
            draw_rectangle(
                head_center.x - head_r * 0.6 + side_shift,
                head_center.y - head_r * 0.74,
                head_r * 1.2,
                head_r * 0.35,
                shade(hair, 0.8),
            );
        }
        HairStyle::Crew => {
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.8,
                head_r * 0.78 * back_boost,
                hair,
            );
            draw_rectangle(
                head_center.x - head_r * 0.8 + side_shift,
                head_center.y - head_r * 0.4,
                head_r * 1.6,
                head_r * 0.45,
                shade(hair, 0.9),
            );
        }
        HairStyle::Ponytail => {
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.83,
                head_r * 0.75 * back_boost,
                hair,
            );
            let pony_x = if matches!(facing, CharacterFacing::Back) {
                15.2
            } else {
                21.6
            };
            let pony = xform.rect(pony_x, 8.4 + bob, 2.5, 5.4);
            draw_rectangle(pony.x, pony.y, pony.w, pony.h, shade(hair, 0.86));
        }
        HairStyle::Mohawk => {
            let strip = xform.rect(15.2, 2.6 + bob, 1.6, 6.0);
            draw_rectangle(strip.x, strip.y, strip.w, strip.h, hair);
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.55,
                head_r * 0.62 * back_boost,
                shade(hair, 0.84),
            );
        }
        HairStyle::Curly => {
            draw_circle(
                head_center.x - head_r * 0.5 + side_shift,
                head_center.y - head_r * 0.8,
                head_r * 0.48,
                hair,
            );
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.95,
                head_r * 0.52 * back_boost,
                hair,
            );
            draw_circle(
                head_center.x + head_r * 0.5 + side_shift,
                head_center.y - head_r * 0.8,
                head_r * 0.48,
                hair,
            );
        }
        HairStyle::Braids => {
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.78,
                head_r * 0.68 * back_boost,
                hair,
            );
            let braid_l = xform.rect(10.5, 8.0 + bob, 1.4, 6.5);
            let braid_r = xform.rect(20.1, 8.0 + bob, 1.4, 6.5);
            draw_rectangle(
                braid_l.x,
                braid_l.y,
                braid_l.w,
                braid_l.h,
                shade(hair, 0.88),
            );
            draw_rectangle(
                braid_r.x,
                braid_r.y,
                braid_r.w,
                braid_r.h,
                shade(hair, 0.88),
            );
        }
    }
}

fn draw_accessory(
    accessory: Accessory,
    xform: &CharacterCanvas,
    head_center: Vec2,
    metrics: BodyMetrics,
    outfit: OutfitColors,
    bob: f32,
    facing: CharacterFacing,
) {
    match accessory {
        Accessory::None => {}
        Accessory::Goggles => {
            if matches!(facing, CharacterFacing::Back) {
                let strap = xform.rect(11.9, 8.2 + bob, 8.2, 1.3);
                draw_rectangle(strap.x, strap.y, strap.w, strap.h, shade(outfit.trim, 0.75));
            } else if matches!(facing, CharacterFacing::Side) {
                let lens = xform.rect(16.1, 8.8 + bob, 2.0, 1.5);
                draw_rectangle(lens.x, lens.y, lens.w, lens.h, shade(outfit.accent, 0.9));
            } else {
                let lens_l = xform.rect(13.1, 8.8 + bob, 2.0, 1.5);
                let lens_r = xform.rect(16.9, 8.8 + bob, 2.0, 1.5);
                draw_rectangle(
                    lens_l.x,
                    lens_l.y,
                    lens_l.w,
                    lens_l.h,
                    shade(outfit.accent, 0.9),
                );
                draw_rectangle(
                    lens_r.x,
                    lens_r.y,
                    lens_r.w,
                    lens_r.h,
                    shade(outfit.accent, 0.9),
                );
                draw_line(
                    lens_l.x + lens_l.w,
                    lens_l.y + lens_l.h * 0.5,
                    lens_r.x,
                    lens_r.y + lens_r.h * 0.5,
                    1.0 * xform.scale,
                    shade(outfit.trim, 0.7),
                );
            }
        }
        Accessory::Bandana => {
            let band = xform.rect(11.8, 7.6 + bob, 8.4, 1.6);
            draw_rectangle(band.x, band.y, band.w, band.h, outfit.accent);
            let knot_x = if matches!(facing, CharacterFacing::Back) {
                16.0
            } else {
                20.6
            };
            let knot = xform.rect(knot_x, 8.0 + bob, 1.6, 1.6);
            draw_rectangle(knot.x, knot.y, knot.w, knot.h, shade(outfit.accent, 0.86));
        }
        Accessory::Backpack => {
            let strap_l = xform.rect(12.0, 12.8 + bob, 1.2, 7.2);
            let strap_r = xform.rect(18.8, 12.8 + bob, 1.2, 7.2);
            draw_rectangle(
                strap_l.x,
                strap_l.y,
                strap_l.w,
                strap_l.h,
                shade(outfit.trim, 0.8),
            );
            draw_rectangle(
                strap_r.x,
                strap_r.y,
                strap_r.w,
                strap_r.h,
                shade(outfit.trim, 0.8),
            );
        }
        Accessory::Toolbelt => {
            if matches!(facing, CharacterFacing::Back) {
                return;
            }
            let belt = xform.rect(
                16.0 - metrics.torso_w * 0.5,
                18.4 + bob,
                metrics.torso_w,
                1.6,
            );
            draw_rectangle(belt.x, belt.y, belt.w, belt.h, shade(outfit.accent, 0.82));
            draw_circle(
                belt.x + belt.w * 0.5,
                belt.y + belt.h * 0.5,
                0.8 * xform.scale,
                shade(outfit.trim, 0.6),
            );
        }
        Accessory::ShoulderPad => {
            let side_x = if matches!(facing, CharacterFacing::Side) {
                1.6
            } else {
                4.4
            };
            draw_circle(
                head_center.x + side_x * xform.scale,
                head_center.y + 7.2 * xform.scale,
                2.0 * xform.scale,
                shade(outfit.accent, 0.9),
            );
        }
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}

fn shade(color: Color, factor: f32) -> Color {
    let f = factor.max(0.0);
    Color::new(
        (color.r * f).clamp(0.0, 1.0),
        (color.g * f).clamp(0.0, 1.0),
        (color.b * f).clamp(0.0, 1.0),
        color.a,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_catalog_loads() {
        let catalog = CharacterCatalog::load_default().expect("default catalog should parse");
        assert!(catalog.mutation_permille() <= 300);
    }

    #[test]
    fn founder_generation_is_deterministic() {
        let catalog = CharacterCatalog::load_default().expect("default catalog should parse");
        let a = catalog.spawn_founder("A", 42);
        let b = catalog.spawn_founder("A", 42);
        assert_eq!(a.dna, b.dna);
        assert_eq!(a.visual, b.visual);
    }

    #[test]
    fn child_inherits_parent_alleles_when_mutation_disabled() {
        let catalog = CharacterCatalog::load_default().expect("default catalog should parse");
        let parent_a = catalog.spawn_founder("A", 123);
        let parent_b = catalog.spawn_founder("B", 456);
        let child = catalog.inherit_dna_for_test(&parent_a.dna, &parent_b.dna, 777, 0);

        assert!(parent_a.dna.body_type.contains(child.body_type.a));
        assert!(parent_b.dna.body_type.contains(child.body_type.b));

        assert!(parent_a.dna.skin_tone.contains(child.skin_tone.a));
        assert!(parent_b.dna.skin_tone.contains(child.skin_tone.b));

        assert!(parent_a.dna.hair_style.contains(child.hair_style.a));
        assert!(parent_b.dna.hair_style.contains(child.hair_style.b));

        assert!(parent_a.dna.hair_color.contains(child.hair_color.a));
        assert!(parent_b.dna.hair_color.contains(child.hair_color.b));

        assert!(parent_a.dna.outfit_style.contains(child.outfit_style.a));
        assert!(parent_b.dna.outfit_style.contains(child.outfit_style.b));

        assert!(parent_a.dna.outfit_palette.contains(child.outfit_palette.a));
        assert!(parent_b.dna.outfit_palette.contains(child.outfit_palette.b));

        assert!(parent_a.dna.accessory.contains(child.accessory.a));
        assert!(parent_b.dna.accessory.contains(child.accessory.b));
    }

    #[test]
    fn invalid_catalog_is_rejected() {
        let invalid = r#"
        (
            mutation_permille: 12,
            body_types: [(id: slim, weight: 1), (id: slim, weight: 1)],
            skin_tones: [(id: warm, weight: 1)],
            hair_styles: [(id: crew, weight: 1)],
            hair_colors: [(id: black, weight: 1)],
            outfit_styles: [(id: worker, weight: 1)],
            outfit_palettes: [(id: rust, weight: 1)],
            accessories: [(id: none, weight: 1)],
        )
        "#;

        let err = CharacterCatalog::from_ron(invalid);
        assert!(err.is_err());
    }
}
