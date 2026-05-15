use super::theme::mix_color;
use super::*;

pub(crate) fn draw_background(palette: &Palette, time: f32) {
    let sw = screen_width();
    let sh = screen_height();
    let lines = sh.max(1.0) as i32;
    let world = palette.world;
    let pulse = (time * 0.18).sin() * 0.5 + 0.5;

    for y in 0..lines {
        let t = y as f32 / (lines - 1).max(1) as f32;
        let band = if t < 0.38 {
            mix_color(world.bg_top, world.bg_mid, t / 0.38)
        } else {
            mix_color(world.bg_mid, world.bg_bottom, (t - 0.38) / 0.62)
        };
        draw_line(0.0, y as f32, sw, y as f32, 1.0, band);
    }

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
