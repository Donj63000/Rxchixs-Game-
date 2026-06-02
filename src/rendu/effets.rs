use super::theme::mix_color;
use super::*;
use std::cell::RefCell;

const BACKGROUND_GRADIENT_W: u16 = 4;
const BACKGROUND_GRADIENT_H: u16 = 256;

thread_local! {
    static BACKGROUND_GRADIENT_TEXTURE: RefCell<Option<Texture2D>> = const { RefCell::new(None) };
}

fn background_gradient_color_at(world: theme::WorldTheme, y: u16) -> Color {
    let denom = (BACKGROUND_GRADIENT_H - 1).max(1) as f32;
    let t = y as f32 / denom;
    if t < 0.38 {
        mix_color(world.bg_top, world.bg_mid, t / 0.38)
    } else {
        mix_color(world.bg_mid, world.bg_bottom, (t - 0.38) / 0.62)
    }
}

fn build_background_gradient_texture(world: theme::WorldTheme) -> Texture2D {
    let mut image = Image::gen_image_color(BACKGROUND_GRADIENT_W, BACKGROUND_GRADIENT_H, BLACK);
    for y in 0..BACKGROUND_GRADIENT_H {
        let color = background_gradient_color_at(world, y);
        for x in 0..BACKGROUND_GRADIENT_W {
            image.set_pixel(x as u32, y as u32, color);
        }
    }
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Linear);
    texture
}

pub(crate) fn draw_background(palette: &Palette, time: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let world = palette.world;
    let pulse = (time * 0.18).sin() * 0.5 + 0.5;

    BACKGROUND_GRADIENT_TEXTURE.with(|slot| {
        let mut texture = slot.borrow_mut();
        if texture.is_none() {
            *texture = Some(build_background_gradient_texture(world));
        }
        if let Some(texture) = texture.as_ref() {
            draw_texture_ex(
                texture,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(sw.max(1.0), sh.max(1.0))),
                    ..Default::default()
                },
            );
        }
    });

    let haze_alpha = 0.05 + pulse * 0.04;
    draw_circle(
        sw * 0.22,
        sh * 0.16,
        sh * 0.22,
        with_alpha(world.bg_haze, haze_alpha),
    );
    draw_circle(
        sw * 0.82,
        sh * 0.12,
        sh * 0.18,
        with_alpha(world.bg_haze, haze_alpha * 0.64),
    );
    draw_rectangle(
        0.0,
        sh * 0.74,
        sw,
        sh * 0.26,
        with_alpha(world.shadow_hard, 0.08),
    );
}

pub(crate) fn draw_ambient_dust(palette: &Palette, time: f32) {
    let sw = screen_width();
    let sh = screen_height();
    for i in 0..12 {
        let fx = ((i as f32 * 137.0) + time * (4.5 + i as f32 * 0.14)).rem_euclid(sw + 90.0) - 40.0;
        let fy = (((i * 97) % 11) as f32 * sh * 0.09 + time * (2.2 + i as f32 * 0.08))
            .rem_euclid(sh + 60.0)
            - 30.0;
        let alpha = 0.05 + ((time * 0.6 + i as f32 * 0.31).sin() * 0.5 + 0.5) * 0.08;
        draw_circle(
            fx,
            fy,
            1.4 + (i % 3) as f32 * 0.9,
            with_alpha(palette.dust, alpha),
        );
    }
}

pub(crate) fn draw_vignette(palette: &Palette) {
    let sw = screen_width();
    let sh = screen_height();
    let bands = 9;
    for i in 0..bands {
        let t = i as f32 / bands as f32;
        let inset_x = sw * t * 0.032;
        let inset_y = sh * t * 0.040;
        let alpha = 0.018 + t * 0.042;
        draw_rectangle_lines(
            inset_x,
            inset_y,
            (sw - inset_x * 2.0).max(1.0),
            (sh - inset_y * 2.0).max(1.0),
            10.0,
            with_alpha(palette.vignette, alpha),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn background_gradient_color_uses_top_mid_bottom_anchors() {
        let world = theme::world_theme();
        let top = background_gradient_color_at(world, 0);
        let middle =
            background_gradient_color_at(world, (BACKGROUND_GRADIENT_H as f32 * 0.38) as u16);
        let bottom = background_gradient_color_at(world, BACKGROUND_GRADIENT_H - 1);

        assert_eq!(top, world.bg_top);
        assert!((middle.r - world.bg_mid.r).abs() < 0.01);
        assert!((middle.g - world.bg_mid.g).abs() < 0.01);
        assert!((middle.b - world.bg_mid.b).abs() < 0.01);
        assert_eq!(bottom, world.bg_bottom);
    }

    #[test]
    fn background_gradient_texture_budget_is_constant() {
        assert_eq!(BACKGROUND_GRADIENT_W, 4);
        assert_eq!(BACKGROUND_GRADIENT_H, 256);
    }
}
