use macroquad::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;

mod rendu_personnage;

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
    pub presentation: CharacterPresentation,
    pub facing: CharacterFacing,
    pub facing_left: bool,
    pub is_walking: bool,
    pub walk_cycle: f32,
    pub gesture: CharacterGesture,
    pub time: f32,
    pub debug: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharacterPresentation {
    World,
    Portrait,
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
    rendu_personnage::draw_character(record, params);
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
