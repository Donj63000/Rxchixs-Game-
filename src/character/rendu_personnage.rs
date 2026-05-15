use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
struct PresentationTuning {
    shadow_width: f32,
    shadow_height: f32,
    shadow_y: f32,
    shadow_alpha: f32,
    outline_size: f32,
    outline_alpha: f32,
    rim_alpha: f32,
    trim_factor: f32,
    accent_mix: f32,
    stride_scale: f32,
    bob_scale: f32,
    gesture_scale: f32,
    face_scale: f32,
    silhouette_boost: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct MotionProfile {
    stride: f32,
    bob: f32,
    gesture: f32,
    idle_wave: f32,
}

#[derive(Clone, Copy)]
struct OutfitColors {
    base: Color,
    trim: Color,
    accent: Color,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy)]
struct TorsoRenderContext<'a> {
    xform: &'a CharacterCanvas,
    torso: Rect,
    outfit: OutfitColors,
    bob: f32,
    facing: CharacterFacing,
    presentation: CharacterPresentation,
    outline: Color,
    tuning: PresentationTuning,
}

#[derive(Clone, Copy)]
struct HairRenderContext<'a> {
    xform: &'a CharacterCanvas,
    head_center: Vec2,
    head_r: f32,
    bob: f32,
    facing: CharacterFacing,
    grow: f32,
}

#[derive(Clone, Copy)]
struct AccessoryRenderContext<'a> {
    xform: &'a CharacterCanvas,
    head_center: Vec2,
    metrics: BodyMetrics,
    outfit: OutfitColors,
    bob: f32,
    facing: CharacterFacing,
    presentation: CharacterPresentation,
    outline: Color,
    tuning: PresentationTuning,
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

    fn rect_grown(self, local_x: f32, local_y: f32, width: f32, height: f32, grow: f32) -> Rect {
        let growth = grow.max(0.0);
        let extra_w = width * (growth - 1.0);
        let extra_h = height * (growth - 1.0);
        self.rect(
            local_x - extra_w * 0.5,
            local_y - extra_h * 0.5,
            width + extra_w,
            height + extra_h,
        )
    }
}

pub fn draw_character(record: &CharacterRecord, params: CharacterRenderParams) {
    let scale = params.scale.max(0.2);
    let xform = CharacterCanvas::new(params.center, scale, params.facing_left);
    let visual = record.visual;
    let tuning = presentation_tuning(params.presentation);
    let motion = motion_profile(params, record.seed, tuning);
    let skin = skin_color(visual.skin_tone);
    let hair = hair_color(visual.hair_color);
    let eyes = shade(hair, 0.26);
    let outfit = outfit_colors(
        visual.outfit_style,
        visual.outfit_palette,
        params.presentation,
    );
    let metrics = body_metrics(visual.body_type, params.presentation);
    let outline = with_alpha(Color::new(0.02, 0.04, 0.05, 1.0), tuning.outline_alpha);
    let rim = with_alpha(blend(outfit.accent, WHITE, 0.38), tuning.rim_alpha);
    let facing = params.facing;
    let g_anim = motion.gesture;

    draw_ground_shadow(params.center, scale, tuning, motion);

    if matches!(visual.accessory, Accessory::Backpack) || matches!(facing, CharacterFacing::Back) {
        let pack = xform.rect_grown(
            8.15,
            12.0 + motion.bob,
            4.35,
            11.0 + tuning.silhouette_boost * 0.55,
            1.0 + tuning.silhouette_boost * 0.18,
        );
        draw_rect_with_outline(pack, shade(outfit.base, 0.60), outline, tuning.outline_size);
        let flap = inset_rect(pack, 0.9 * scale);
        draw_rectangle(
            flap.x,
            flap.y,
            flap.w,
            (flap.h * 0.48).max(0.1),
            shade(outfit.trim, 0.92),
        );
    }

    match facing {
        CharacterFacing::Side => {
            let far_leg = xform.rect(
                13.6 - motion.stride * 0.30,
                20.1 - motion.stride * 0.24,
                metrics.leg_w - 0.15,
                8.6,
            );
            let near_leg = xform.rect(
                16.45 + motion.stride * 0.30,
                19.9 + motion.stride * 0.24,
                metrics.leg_w + 0.35,
                9.2,
            );
            draw_rect_with_outline(
                far_leg,
                shade(outfit.base, 0.66),
                outline,
                tuning.outline_size * 0.72,
            );
            draw_rect_with_outline(
                near_leg,
                shade(outfit.base, 0.90),
                outline,
                tuning.outline_size * 0.78,
            );
        }
        CharacterFacing::Front | CharacterFacing::Back => {
            let left_leg = xform.rect(
                11.8 + motion.stride * 0.86,
                19.9 + motion.stride.abs() * 0.24,
                metrics.leg_w,
                8.95,
            );
            let right_leg = xform.rect(
                16.1 - motion.stride * 0.86,
                19.9 + (1.0 - motion.stride.abs()) * 0.34,
                metrics.leg_w,
                8.95,
            );
            let left_fill = if matches!(facing, CharacterFacing::Back) {
                shade(outfit.base, 0.66)
            } else {
                shade(outfit.base, 0.80)
            };
            let right_fill = if matches!(facing, CharacterFacing::Back) {
                shade(outfit.base, 0.74)
            } else {
                shade(outfit.base, 0.90)
            };
            draw_rect_with_outline(left_leg, left_fill, outline, tuning.outline_size * 0.72);
            draw_rect_with_outline(right_leg, right_fill, outline, tuning.outline_size * 0.72);
        }
    }

    let torso = xform.rect(
        16.0 - metrics.torso_w * 0.5,
        12.3 + motion.bob,
        metrics.torso_w,
        metrics.torso_h,
    );
    let torso_fill = match facing {
        CharacterFacing::Back => shade(outfit.base, 0.80),
        _ => outfit.base,
    };
    draw_rect_with_outline(torso, torso_fill, outline, tuning.outline_size);
    let trim_rect = inset_rect(torso, 1.1 * scale);
    draw_rectangle(
        trim_rect.x,
        trim_rect.y,
        trim_rect.w,
        trim_rect.h,
        match facing {
            CharacterFacing::Back => shade(outfit.trim, 0.94),
            _ => shade(outfit.trim, 1.04),
        },
    );
    draw_rectangle(
        trim_rect.x + 0.5 * scale,
        trim_rect.y + 0.2 * scale,
        trim_rect.w.max(0.1),
        (1.35 * scale).min(trim_rect.h.max(0.1)),
        rim,
    );
    draw_torso_style(
        visual.outfit_style,
        TorsoRenderContext {
            xform: &xform,
            torso,
            outfit,
            bob: motion.bob,
            facing,
            presentation: params.presentation,
            outline,
            tuning,
        },
    );

    match facing {
        CharacterFacing::Side => {
            let (near_extra, far_extra) = if !params.is_walking {
                match params.gesture {
                    CharacterGesture::Talk => (g_anim * 1.6, -g_anim * 0.6),
                    CharacterGesture::Explain => (g_anim.abs() * 1.2, g_anim * 0.7),
                    CharacterGesture::Wave => (g_anim.abs() * 4.2, 0.0),
                    CharacterGesture::Apologize => (-g_anim.abs() * 1.3, -g_anim.abs() * 0.4),
                    CharacterGesture::Threaten => (g_anim.abs() * 2.9, g_anim.abs() * 1.5),
                    CharacterGesture::Argue => (g_anim * 2.6, -g_anim * 1.9),
                    CharacterGesture::Laugh => (g_anim.abs() * 1.9, g_anim * 0.4),
                    CharacterGesture::None => (0.0, 0.0),
                }
            } else {
                (0.0, 0.0)
            };

            let far_arm = xform.rect(
                13.95,
                13.6 + motion.bob - motion.stride * 0.26 - far_extra,
                2.3,
                7.2,
            );
            let near_arm = xform.rect(
                17.35,
                13.6 + motion.bob + motion.stride * 0.38 - near_extra,
                2.65,
                7.9,
            );
            draw_rect_with_outline(
                far_arm,
                shade(outfit.base, 0.64),
                outline,
                tuning.outline_size * 0.68,
            );
            draw_rect_with_outline(
                near_arm,
                shade(outfit.base, 0.90),
                outline,
                tuning.outline_size * 0.72,
            );
            let near_hand = xform.point(18.7, 21.6 + motion.bob + motion.stride * 0.28);
            draw_circle_with_outline(
                near_hand,
                1.32 * scale,
                shade(skin, 0.92),
                outline,
                tuning.outline_size * 0.58,
            );
        }
        CharacterFacing::Front | CharacterFacing::Back => {
            let (left_extra, right_extra) = if !params.is_walking {
                match params.gesture {
                    CharacterGesture::Talk => (-g_anim * 1.4, g_anim * 1.1),
                    CharacterGesture::Explain => (g_anim.abs() * 1.15, g_anim * 0.6),
                    CharacterGesture::Wave => (0.0, g_anim.abs() * 4.1),
                    CharacterGesture::Apologize => (-g_anim.abs() * 1.15, -g_anim.abs() * 0.85),
                    CharacterGesture::Threaten => (g_anim.abs() * 2.35, g_anim.abs() * 2.45),
                    CharacterGesture::Argue => (-g_anim * 2.1, g_anim * 2.1),
                    CharacterGesture::Laugh => (g_anim.abs() * 1.4, g_anim * 0.5),
                    CharacterGesture::None => (0.0, 0.0),
                }
            } else {
                (0.0, 0.0)
            };

            let left_arm = xform.rect(
                16.0 - metrics.shoulder_w * 0.5 - 2.25,
                13.45 + motion.bob + motion.stride * 0.34 - left_extra,
                2.5,
                7.7,
            );
            let right_arm = xform.rect(
                16.0 + metrics.shoulder_w * 0.5 - 0.25,
                13.45 + motion.bob - motion.stride * 0.34 - right_extra,
                2.5,
                7.7,
            );
            let left_fill = if matches!(facing, CharacterFacing::Back) {
                shade(outfit.base, 0.62)
            } else {
                shade(outfit.base, 0.78)
            };
            let right_fill = if matches!(facing, CharacterFacing::Back) {
                shade(outfit.base, 0.70)
            } else {
                shade(outfit.base, 0.86)
            };
            draw_rect_with_outline(left_arm, left_fill, outline, tuning.outline_size * 0.68);
            draw_rect_with_outline(right_arm, right_fill, outline, tuning.outline_size * 0.68);

            if matches!(facing, CharacterFacing::Front) {
                let left_hand =
                    xform.point(16.0 - metrics.shoulder_w * 0.5 - 1.1, 21.2 + motion.bob);
                let right_hand =
                    xform.point(16.0 + metrics.shoulder_w * 0.5 + 1.1, 21.2 + motion.bob);
                draw_circle_with_outline(
                    left_hand,
                    1.28 * scale,
                    shade(skin, 0.92),
                    outline,
                    tuning.outline_size * 0.56,
                );
                draw_circle_with_outline(
                    right_hand,
                    1.28 * scale,
                    shade(skin, 0.92),
                    outline,
                    tuning.outline_size * 0.56,
                );
            }
        }
    }

    let head_center = xform.point(16.0, 8.1 + motion.bob);
    draw_circle_with_outline(
        head_center,
        metrics.head_r * scale,
        skin,
        outline,
        tuning.outline_size * 0.95,
    );
    draw_circle(
        head_center.x,
        head_center.y - metrics.head_r * scale * 0.34,
        metrics.head_r * scale * 0.52,
        with_alpha(blend(skin, WHITE, 0.30), tuning.rim_alpha * 0.95),
    );
    match facing {
        CharacterFacing::Front => {
            let eye_r = 0.72 * scale * tuning.face_scale;
            draw_circle(
                head_center.x - 1.2 * scale,
                head_center.y + 1.0 * scale,
                eye_r,
                eyes,
            );
            draw_circle(
                head_center.x + 1.2 * scale,
                head_center.y + 1.0 * scale,
                eye_r,
                eyes,
            );
            if matches!(params.presentation, CharacterPresentation::Portrait) {
                draw_line(
                    head_center.x - 2.0 * scale,
                    head_center.y + 0.1 * scale,
                    head_center.x - 0.3 * scale,
                    head_center.y - 0.3 * scale,
                    0.9 * scale,
                    with_alpha(eyes, 0.68),
                );
                draw_line(
                    head_center.x + 0.3 * scale,
                    head_center.y - 0.3 * scale,
                    head_center.x + 2.0 * scale,
                    head_center.y + 0.1 * scale,
                    0.9 * scale,
                    with_alpha(eyes, 0.68),
                );
            }
            let mouth_col = shade(skin, 0.58);
            let talking = !params.is_walking
                && matches!(
                    params.gesture,
                    CharacterGesture::Talk
                        | CharacterGesture::Explain
                        | CharacterGesture::Laugh
                        | CharacterGesture::Apologize
                        | CharacterGesture::Threaten
                        | CharacterGesture::Argue
                );
            if talking {
                let m = (params.time * 10.0 + record.seed as f32 * 0.0014).sin();
                if matches!(params.gesture, CharacterGesture::Laugh) {
                    let amp = 0.95 + 0.8 * m.abs();
                    draw_line(
                        head_center.x - 1.6 * scale,
                        head_center.y + 3.0 * scale,
                        head_center.x,
                        head_center.y + (3.0 + amp) * scale,
                        1.0 * scale * tuning.face_scale.max(0.92),
                        mouth_col,
                    );
                    draw_line(
                        head_center.x,
                        head_center.y + (3.0 + amp) * scale,
                        head_center.x + 1.6 * scale,
                        head_center.y + 3.0 * scale,
                        1.0 * scale * tuning.face_scale.max(0.92),
                        mouth_col,
                    );
                } else if m > 0.0 {
                    draw_circle(
                        head_center.x,
                        head_center.y + 3.2 * scale,
                        1.0 * scale * tuning.face_scale.max(0.9),
                        mouth_col,
                    );
                } else {
                    draw_line(
                        head_center.x - 1.8 * scale,
                        head_center.y + 3.0 * scale,
                        head_center.x + 1.8 * scale,
                        head_center.y + 3.0 * scale,
                        1.0 * scale * tuning.face_scale.max(0.9),
                        mouth_col,
                    );
                }
            } else {
                draw_line(
                    head_center.x - 1.8 * scale,
                    head_center.y + 3.0 * scale,
                    head_center.x + 1.8 * scale,
                    head_center.y + 3.0 * scale,
                    1.0 * scale * tuning.face_scale.max(0.9),
                    mouth_col,
                );
                if matches!(params.presentation, CharacterPresentation::Portrait) {
                    draw_line(
                        head_center.x,
                        head_center.y + 1.5 * scale,
                        head_center.x + 0.2 * scale,
                        head_center.y + 2.2 * scale,
                        0.8 * scale,
                        with_alpha(shade(skin, 0.54), 0.78),
                    );
                }
            }
        }
        CharacterFacing::Side => {
            let dir = if params.facing_left { -1.0 } else { 1.0 };
            draw_circle(
                head_center.x + dir * 0.95 * scale,
                head_center.y + 1.05 * scale,
                0.72 * scale * tuning.face_scale,
                eyes,
            );
            if matches!(params.presentation, CharacterPresentation::Portrait) {
                draw_line(
                    head_center.x - dir * 0.3 * scale,
                    head_center.y + 0.2 * scale,
                    head_center.x + dir * 1.5 * scale,
                    head_center.y - 0.1 * scale,
                    0.8 * scale,
                    with_alpha(eyes, 0.68),
                );
            }
            draw_line(
                head_center.x + dir * 1.5 * scale,
                head_center.y + 1.9 * scale,
                head_center.x + dir * 2.5 * scale,
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

    let hair_outline = with_alpha(
        Color::new(0.02, 0.04, 0.05, 1.0),
        tuning.outline_alpha * 0.95,
    );
    let hair_ctx = HairRenderContext {
        xform: &xform,
        head_center,
        head_r: metrics.head_r * scale,
        bob: motion.bob,
        facing,
        grow: 1.0 + tuning.silhouette_boost * 0.22,
    };
    draw_hair(visual.hair_style, hair_outline, hair_ctx);
    draw_hair(
        visual.hair_style,
        hair,
        HairRenderContext {
            grow: 1.0,
            ..hair_ctx
        },
    );
    if matches!(params.presentation, CharacterPresentation::Portrait) {
        draw_circle(
            head_center.x,
            head_center.y - metrics.head_r * scale * 0.82,
            metrics.head_r * scale * 0.24,
            with_alpha(blend(hair, WHITE, 0.42), 0.18),
        );
    }
    draw_accessory(
        visual.accessory,
        AccessoryRenderContext {
            xform: &xform,
            head_center,
            metrics,
            outfit,
            bob: motion.bob,
            facing,
            presentation: params.presentation,
            outline,
            tuning,
        },
    );

    if params.debug {
        let box_rect = xform.rect(8.7, 2.8 + motion.bob, 14.6, 25.6);
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

fn presentation_tuning(presentation: CharacterPresentation) -> PresentationTuning {
    match presentation {
        CharacterPresentation::World => PresentationTuning {
            shadow_width: 15.4,
            shadow_height: 4.3,
            shadow_y: 10.5,
            shadow_alpha: 0.50,
            outline_size: 0.78,
            outline_alpha: 0.26,
            rim_alpha: 0.14,
            trim_factor: 0.66,
            accent_mix: 0.16,
            stride_scale: 1.22,
            bob_scale: 1.08,
            gesture_scale: 1.08,
            face_scale: 0.94,
            silhouette_boost: 0.38,
        },
        CharacterPresentation::Portrait => PresentationTuning {
            shadow_width: 13.8,
            shadow_height: 3.5,
            shadow_y: 10.2,
            shadow_alpha: 0.36,
            outline_size: 0.52,
            outline_alpha: 0.16,
            rim_alpha: 0.10,
            trim_factor: 0.76,
            accent_mix: 0.08,
            stride_scale: 1.00,
            bob_scale: 0.94,
            gesture_scale: 0.96,
            face_scale: 1.08,
            silhouette_boost: 0.08,
        },
    }
}

fn motion_profile(
    params: CharacterRenderParams,
    seed: u64,
    tuning: PresentationTuning,
) -> MotionProfile {
    let walk_phase = if params.is_walking {
        params.walk_cycle.sin()
    } else {
        0.0
    };
    let stride = walk_phase * (0.85 + params.walk_cycle.cos().abs() * 0.30) * tuning.stride_scale;
    let idle_wave = (params.time * 2.1 + seed as f32 * 0.0003).sin() * 0.22 * tuning.bob_scale;
    let g_phase = (params.time * 7.5 + seed as f32 * 0.0009).sin();
    let g_power = match params.gesture {
        CharacterGesture::None => 0.0,
        CharacterGesture::Talk => 0.7,
        CharacterGesture::Explain => 0.9,
        CharacterGesture::Laugh => 0.8,
        CharacterGesture::Apologize => 0.6,
        CharacterGesture::Wave => 1.5,
        CharacterGesture::Threaten => 1.0,
        CharacterGesture::Argue => 1.2,
    };
    let gesture = if params.is_walking {
        0.0
    } else {
        g_phase * g_power * tuning.gesture_scale
    };
    let bob = stride * 0.28 + idle_wave + gesture * 0.34;

    MotionProfile {
        stride,
        bob,
        gesture,
        idle_wave,
    }
}

fn body_metrics(body_type: BodyType, presentation: CharacterPresentation) -> BodyMetrics {
    let mut metrics = match body_type {
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
    };

    if matches!(presentation, CharacterPresentation::World) {
        metrics.torso_w += 0.45;
        metrics.torso_h += 0.22;
        metrics.shoulder_w += 0.52;
        metrics.leg_w += 0.18;
        metrics.head_r += 0.18;
    }

    metrics
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

fn outfit_colors(
    style: OutfitStyle,
    palette: OutfitPalette,
    presentation: CharacterPresentation,
) -> OutfitColors {
    let tuning = presentation_tuning(presentation);
    let base_raw = match palette {
        OutfitPalette::Rust => rgb(124, 82, 66),
        OutfitPalette::Slate => rgb(76, 92, 110),
        OutfitPalette::Moss => rgb(82, 106, 88),
        OutfitPalette::Sand => rgb(150, 132, 96),
        OutfitPalette::Cobalt => rgb(68, 96, 144),
    };
    let base = match presentation {
        CharacterPresentation::World => blend(base_raw, rgb(28, 34, 40), 0.10),
        CharacterPresentation::Portrait => blend(base_raw, rgb(244, 242, 236), 0.04),
    };
    let trim = shade(base, tuning.trim_factor);
    let accent_base = match style {
        OutfitStyle::Worker => rgb(226, 184, 88),
        OutfitStyle::Engineer => rgb(102, 188, 216),
        OutfitStyle::Medic => rgb(214, 100, 96),
        OutfitStyle::Scout => rgb(132, 194, 118),
    };
    let accent = blend(accent_base, rgb(244, 238, 224), tuning.accent_mix);

    OutfitColors { base, trim, accent }
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

fn blend(a: Color, b: Color, t: f32) -> Color {
    let clamped = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * clamped,
        a.g + (b.g - a.g) * clamped,
        a.b + (b.b - a.b) * clamped,
        a.a + (b.a - a.a) * clamped,
    )
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, alpha.clamp(0.0, 1.0))
}

#[cfg(test)]
fn luminance(color: Color) -> f32 {
    color.r * 0.2126 + color.g * 0.7152 + color.b * 0.0722
}

fn draw_ground_shadow(center: Vec2, scale: f32, tuning: PresentationTuning, motion: MotionProfile) {
    let stretch = 1.0 + motion.stride.abs() * 0.04;
    let width = tuning.shadow_width * scale * stretch;
    let height = tuning.shadow_height * scale;
    let rect = Rect::new(
        center.x - width * 0.5,
        center.y + tuning.shadow_y * scale - height * 0.5,
        width,
        height,
    );
    let fill = Color::new(0.01, 0.02, 0.03, tuning.shadow_alpha);
    draw_capsule(rect, fill);
    let inner = Rect::new(
        rect.x + 1.2 * scale,
        rect.y + 0.5 * scale,
        (rect.w - 2.4 * scale).max(0.1),
        (rect.h - 1.0 * scale).max(0.1),
    );
    draw_capsule(inner, with_alpha(fill, tuning.shadow_alpha * 0.38));
}

fn draw_capsule(rect: Rect, color: Color) {
    let radius = (rect.h * 0.5).min(rect.w * 0.5);
    let mid_w = (rect.w - radius * 2.0).max(0.0);
    if mid_w > 0.0 {
        draw_rectangle(rect.x + radius, rect.y, mid_w, rect.h, color);
    }
    draw_circle(rect.x + radius, rect.y + rect.h * 0.5, radius, color);
    draw_circle(
        rect.x + rect.w - radius,
        rect.y + rect.h * 0.5,
        radius,
        color,
    );
}

fn draw_rect_with_outline(rect: Rect, fill: Color, outline: Color, pad: f32) {
    let outer = expand_rect(rect, pad.max(0.0));
    draw_rectangle(outer.x, outer.y, outer.w, outer.h, outline);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
}

fn draw_circle_with_outline(center: Vec2, radius: f32, fill: Color, outline: Color, pad: f32) {
    draw_circle(center.x, center.y, radius + pad.max(0.0), outline);
    draw_circle(center.x, center.y, radius, fill);
}

fn expand_rect(rect: Rect, pad: f32) -> Rect {
    Rect::new(
        rect.x - pad,
        rect.y - pad,
        rect.w + pad * 2.0,
        rect.h + pad * 2.0,
    )
}

fn inset_rect(rect: Rect, pad: f32) -> Rect {
    Rect::new(
        rect.x + pad,
        rect.y + pad,
        (rect.w - pad * 2.0).max(0.1),
        (rect.h - pad * 2.0).max(0.1),
    )
}

fn draw_torso_style(style: OutfitStyle, ctx: TorsoRenderContext<'_>) {
    let xform = ctx.xform;
    let torso = ctx.torso;
    let outfit = ctx.outfit;
    let bob = ctx.bob;
    let facing = ctx.facing;
    let presentation = ctx.presentation;
    let outline = ctx.outline;
    let tuning = ctx.tuning;

    if matches!(facing, CharacterFacing::Back) {
        draw_line(
            torso.x + torso.w * 0.22,
            torso.y + torso.h * 0.22,
            torso.x + torso.w * 0.78,
            torso.y + torso.h * 0.22,
            1.0 * xform.scale,
            shade(outfit.trim, 0.74),
        );
        draw_line(
            torso.x + torso.w * 0.5,
            torso.y + torso.h * 0.22,
            torso.x + torso.w * 0.5,
            torso.y + torso.h * 0.84,
            1.0 * xform.scale,
            shade(outfit.trim, 0.64),
        );
        return;
    }

    let role_boost = if matches!(presentation, CharacterPresentation::World) {
        0.45
    } else {
        0.0
    };

    match style {
        OutfitStyle::Worker => {
            let pocket_l = xform.rect_grown(11.4, 16.1 + bob, 2.3, 2.2, 1.0 + role_boost * 0.18);
            let pocket_r = xform.rect_grown(18.2, 16.1 + bob, 2.3, 2.2, 1.0 + role_boost * 0.18);
            draw_rect_with_outline(
                pocket_l,
                shade(outfit.trim, 0.98),
                outline,
                tuning.outline_size * 0.42,
            );
            draw_rect_with_outline(
                pocket_r,
                shade(outfit.trim, 0.98),
                outline,
                tuning.outline_size * 0.42,
            );
            draw_line(
                torso.x + torso.w * 0.5,
                torso.y + 1.2 * xform.scale,
                torso.x + torso.w * 0.5,
                torso.y + torso.h - 1.2 * xform.scale,
                1.0 * xform.scale,
                shade(outfit.trim, 0.66),
            );
        }
        OutfitStyle::Engineer => {
            let stripe = xform.rect_grown(12.8, 13.8 + bob, 6.4, 1.6, 1.0 + role_boost * 0.22);
            draw_rect_with_outline(stripe, outfit.accent, outline, tuning.outline_size * 0.42);
            draw_line(
                torso.x + 1.2 * xform.scale,
                torso.y + torso.h - 1.9 * xform.scale,
                torso.x + torso.w - 1.2 * xform.scale,
                torso.y + torso.h - 1.9 * xform.scale,
                1.0 * xform.scale,
                shade(outfit.trim, 0.64),
            );
        }
        OutfitStyle::Medic => {
            let cross_v = xform.rect_grown(15.3, 14.2 + bob, 1.5, 4.6, 1.0 + role_boost * 0.20);
            let cross_h = xform.rect_grown(13.8, 15.7 + bob, 4.6, 1.5, 1.0 + role_boost * 0.20);
            draw_rect_with_outline(cross_v, outfit.accent, outline, tuning.outline_size * 0.42);
            draw_rect_with_outline(cross_h, outfit.accent, outline, tuning.outline_size * 0.42);
        }
        OutfitStyle::Scout => {
            let scarf = xform.rect_grown(12.0, 12.7 + bob, 8.0, 2.1, 1.0 + role_boost * 0.20);
            draw_rect_with_outline(scarf, outfit.accent, outline, tuning.outline_size * 0.40);
            draw_line(
                scarf.x + scarf.w * 0.8,
                scarf.y + scarf.h,
                scarf.x + scarf.w * 0.8,
                scarf.y + scarf.h + 3.2 * xform.scale,
                1.0 * xform.scale,
                shade(outfit.accent, 0.78),
            );
        }
    }
}

fn draw_hair(style: HairStyle, hair: Color, ctx: HairRenderContext<'_>) {
    let xform = ctx.xform;
    let head_center = ctx.head_center;
    let head_r = ctx.head_r;
    let bob = ctx.bob;
    let facing = ctx.facing;
    let grow = ctx.grow;

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
                head_r * 0.58 * back_boost * grow,
                hair,
            );
            let strip = xform.rect_grown(13.1, 4.9 + bob, 5.4, 1.6, grow);
            draw_rectangle(strip.x, strip.y, strip.w, strip.h, shade(hair, 0.8));
        }
        HairStyle::Crew => {
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.8,
                head_r * 0.78 * back_boost * grow,
                hair,
            );
            let strip = xform.rect_grown(12.4, 6.0 + bob, 7.2, 2.1, grow);
            draw_rectangle(strip.x, strip.y, strip.w, strip.h, shade(hair, 0.9));
        }
        HairStyle::Ponytail => {
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.83,
                head_r * 0.75 * back_boost * grow,
                hair,
            );
            let pony_x = if matches!(facing, CharacterFacing::Back) {
                15.0
            } else {
                21.4
            };
            let pony = xform.rect_grown(pony_x, 8.3 + bob, 2.6, 5.6, grow);
            draw_rectangle(pony.x, pony.y, pony.w, pony.h, shade(hair, 0.86));
        }
        HairStyle::Mohawk => {
            let strip = xform.rect_grown(15.1, 2.6 + bob, 1.8, 6.0, grow);
            draw_rectangle(strip.x, strip.y, strip.w, strip.h, hair);
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.55,
                head_r * 0.62 * back_boost * grow,
                shade(hair, 0.84),
            );
        }
        HairStyle::Curly => {
            draw_circle(
                head_center.x - head_r * 0.5 + side_shift,
                head_center.y - head_r * 0.8,
                head_r * 0.48 * grow,
                hair,
            );
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.95,
                head_r * 0.52 * back_boost * grow,
                hair,
            );
            draw_circle(
                head_center.x + head_r * 0.5 + side_shift,
                head_center.y - head_r * 0.8,
                head_r * 0.48 * grow,
                hair,
            );
        }
        HairStyle::Braids => {
            draw_circle(
                head_center.x + side_shift,
                head_center.y - head_r * 0.78,
                head_r * 0.68 * back_boost * grow,
                hair,
            );
            let braid_l = xform.rect_grown(10.4, 8.0 + bob, 1.5, 6.6, grow);
            let braid_r = xform.rect_grown(20.1, 8.0 + bob, 1.5, 6.6, grow);
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

fn draw_accessory(accessory: Accessory, ctx: AccessoryRenderContext<'_>) {
    let xform = ctx.xform;
    let head_center = ctx.head_center;
    let metrics = ctx.metrics;
    let outfit = ctx.outfit;
    let bob = ctx.bob;
    let facing = ctx.facing;
    let presentation = ctx.presentation;
    let outline = ctx.outline;
    let tuning = ctx.tuning;

    let boost = if matches!(presentation, CharacterPresentation::World) {
        0.35
    } else {
        0.0
    };
    match accessory {
        Accessory::None => {}
        Accessory::Goggles => {
            if matches!(facing, CharacterFacing::Back) {
                let strap = xform.rect_grown(11.7, 8.1 + bob, 8.6, 1.4, 1.0 + boost * 0.10);
                draw_rect_with_outline(
                    strap,
                    shade(outfit.trim, 0.78),
                    outline,
                    tuning.outline_size * 0.36,
                );
            } else if matches!(facing, CharacterFacing::Side) {
                let lens = xform.rect_grown(16.0, 8.7 + bob, 2.15, 1.6, 1.0 + boost * 0.14);
                draw_rect_with_outline(
                    lens,
                    shade(outfit.accent, 0.92),
                    outline,
                    tuning.outline_size * 0.38,
                );
            } else {
                let lens_l = xform.rect_grown(12.9, 8.7 + bob, 2.15, 1.6, 1.0 + boost * 0.14);
                let lens_r = xform.rect_grown(16.95, 8.7 + bob, 2.15, 1.6, 1.0 + boost * 0.14);
                draw_rect_with_outline(
                    lens_l,
                    shade(outfit.accent, 0.92),
                    outline,
                    tuning.outline_size * 0.38,
                );
                draw_rect_with_outline(
                    lens_r,
                    shade(outfit.accent, 0.92),
                    outline,
                    tuning.outline_size * 0.38,
                );
                draw_line(
                    lens_l.x + lens_l.w,
                    lens_l.y + lens_l.h * 0.5,
                    lens_r.x,
                    lens_r.y + lens_r.h * 0.5,
                    1.0 * xform.scale,
                    shade(outfit.trim, 0.72),
                );
            }
        }
        Accessory::Bandana => {
            let band = xform.rect_grown(11.6, 7.5 + bob, 8.8, 1.7, 1.0 + boost * 0.12);
            draw_rect_with_outline(band, outfit.accent, outline, tuning.outline_size * 0.36);
            let knot_x = if matches!(facing, CharacterFacing::Back) {
                15.9
            } else {
                20.5
            };
            let knot = xform.rect_grown(knot_x, 7.9 + bob, 1.7, 1.7, 1.0 + boost * 0.12);
            draw_rect_with_outline(
                knot,
                shade(outfit.accent, 0.86),
                outline,
                tuning.outline_size * 0.34,
            );
        }
        Accessory::Backpack => {
            let strap_l = xform.rect_grown(11.9, 12.8 + bob, 1.3, 7.3, 1.0 + boost * 0.08);
            let strap_r = xform.rect_grown(18.8, 12.8 + bob, 1.3, 7.3, 1.0 + boost * 0.08);
            draw_rect_with_outline(
                strap_l,
                shade(outfit.trim, 0.82),
                outline,
                tuning.outline_size * 0.32,
            );
            draw_rect_with_outline(
                strap_r,
                shade(outfit.trim, 0.82),
                outline,
                tuning.outline_size * 0.32,
            );
        }
        Accessory::Toolbelt => {
            if matches!(facing, CharacterFacing::Back) {
                return;
            }
            let belt = xform.rect_grown(
                16.0 - metrics.torso_w * 0.5,
                18.3 + bob,
                metrics.torso_w,
                1.75,
                1.0 + boost * 0.12,
            );
            draw_rect_with_outline(
                belt,
                shade(outfit.accent, 0.84),
                outline,
                tuning.outline_size * 0.34,
            );
            draw_circle_with_outline(
                vec2(belt.x + belt.w * 0.5, belt.y + belt.h * 0.5),
                0.82 * xform.scale,
                shade(outfit.trim, 0.62),
                outline,
                tuning.outline_size * 0.22,
            );
        }
        Accessory::ShoulderPad => {
            let side_x = if matches!(facing, CharacterFacing::Side) {
                1.7
            } else {
                4.4
            };
            draw_circle_with_outline(
                vec2(
                    head_center.x + side_x * xform.scale,
                    head_center.y + 7.2 * xform.scale,
                ),
                (2.0 + boost * 0.35) * xform.scale,
                shade(outfit.accent, 0.90),
                outline,
                tuning.outline_size * 0.30,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_tuning_strengthens_outline_shadow_and_stride() {
        let world = presentation_tuning(CharacterPresentation::World);
        let portrait = presentation_tuning(CharacterPresentation::Portrait);

        assert!(world.outline_alpha > portrait.outline_alpha);
        assert!(world.shadow_alpha > portrait.shadow_alpha);
        assert!(world.stride_scale > portrait.stride_scale);
    }

    #[test]
    fn body_metrics_keep_expected_hierarchy_in_both_presentations() {
        for presentation in [
            CharacterPresentation::World,
            CharacterPresentation::Portrait,
        ] {
            let slim = body_metrics(BodyType::Slim, presentation);
            let standard = body_metrics(BodyType::Standard, presentation);
            let broad = body_metrics(BodyType::Broad, presentation);

            assert!(slim.torso_w < standard.torso_w);
            assert!(standard.torso_w < broad.torso_w);
            assert!(slim.shoulder_w < standard.shoulder_w);
            assert!(standard.shoulder_w < broad.shoulder_w);
            assert!(slim.head_r <= standard.head_r);
            assert!(standard.head_r <= broad.head_r);
        }
    }

    #[test]
    fn outfit_world_mode_keeps_role_accents_distinct_and_more_contrasted() {
        let world_worker = outfit_colors(
            OutfitStyle::Worker,
            OutfitPalette::Rust,
            CharacterPresentation::World,
        );
        let world_engineer = outfit_colors(
            OutfitStyle::Engineer,
            OutfitPalette::Rust,
            CharacterPresentation::World,
        );
        let portrait_worker = outfit_colors(
            OutfitStyle::Worker,
            OutfitPalette::Rust,
            CharacterPresentation::Portrait,
        );

        assert_ne!(world_worker.accent, world_engineer.accent);

        let world_contrast = (luminance(world_worker.accent) - luminance(world_worker.base)).abs();
        let portrait_contrast =
            (luminance(portrait_worker.accent) - luminance(portrait_worker.base)).abs();
        assert!(world_contrast > portrait_contrast);
    }

    #[test]
    fn motion_profile_is_deterministic_and_bounded() {
        let walking = CharacterRenderParams {
            center: vec2(0.0, 0.0),
            scale: 1.0,
            presentation: CharacterPresentation::World,
            facing: CharacterFacing::Front,
            facing_left: false,
            is_walking: true,
            walk_cycle: 3.6,
            gesture: CharacterGesture::Wave,
            time: 1.25,
            debug: false,
        };
        let tuning = presentation_tuning(walking.presentation);
        let a = motion_profile(walking, 42, tuning);
        let b = motion_profile(walking, 42, tuning);

        assert_eq!(a, b);
        assert!(a.stride.abs() <= 1.5);
        assert!(a.bob.abs() <= 1.2);
        assert_eq!(a.gesture, 0.0);

        let idle = CharacterRenderParams {
            is_walking: false,
            ..walking
        };
        let idle_motion = motion_profile(idle, 42, tuning);
        assert!(idle_motion.gesture.abs() <= 2.0);
        assert!(idle_motion.bob.abs() <= 1.4);
    }
}
