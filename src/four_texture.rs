use crate::sim;
use macroquad::prelude::*;
use std::cell::RefCell;

fn with_alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha.clamp(0.0, 1.0);
    color
}

fn orientation_axis(orientation: sim::BlockOrientation) -> Vec2 {
    match orientation {
        sim::BlockOrientation::North => vec2(0.0, -1.0),
        sim::BlockOrientation::East => vec2(1.0, 0.0),
        sim::BlockOrientation::South => vec2(0.0, 1.0),
        sim::BlockOrientation::West => vec2(-1.0, 0.0),
    }
}

fn rect_inset(rect: Rect, inset: f32) -> Rect {
    let x = rect.x + inset;
    let y = rect.y + inset;
    let w = (rect.w - inset * 2.0).max(1.0);
    let h = (rect.h - inset * 2.0).max(1.0);
    Rect::new(x, y, w, h)
}

fn rect_inset_xy(rect: Rect, ix: f32, iy: f32) -> Rect {
    let x = rect.x + ix;
    let y = rect.y + iy;
    let w = (rect.w - ix * 2.0).max(1.0);
    let h = (rect.h - iy * 2.0).max(1.0);
    Rect::new(x, y, w, h)
}

fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp01((x - edge0) / (edge1 - edge0));
    t * t * (3.0 - 2.0 * t)
}

fn u8_from_f32(v: f32) -> u8 {
    (v.clamp(0.0, 255.0) + 0.5) as u8
}

fn hash32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^= x >> 16;
    x
}

fn hash2(x: i32, y: i32, seed: u32) -> u32 {
    let v = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add((y as u32).wrapping_mul(668265263))
        .wrapping_add(seed.wrapping_mul(1442695041));
    hash32(v)
}

fn noise01(x: i32, y: i32, seed: u32) -> f32 {
    let h = hash2(x, y, seed);
    (h & 0x00ff_ffff) as f32 / 16_777_215.0
}

fn gen_brushed_metal_tex(w: u16, h: u16, horizontal_brush: bool, seed: u32) -> Texture2D {
    let mut img = Image::gen_image_color(w, h, Color::from_rgba(200, 205, 212, 255));
    let wf = (w.max(2) as f32) - 1.0;
    let hf = (h.max(2) as f32) - 1.0;

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let nx = x as f32 / wf;
            let ny = y as f32 / hf;

            let a = noise01(x, y, seed);
            let b = noise01(x * 3, y * 3, seed ^ 0x9e3779b9);
            let c = noise01(x * 7, y * 7, seed ^ 0x85ebca6b);

            let brush_coord = if horizontal_brush { ny } else { nx };
            let micro_coord = if horizontal_brush {
                ny * 1.0 + nx * 0.15
            } else {
                nx * 1.0 + ny * 0.15
            };

            let stripe1 = (brush_coord * 820.0 + a * 6.0).sin() * 0.5 + 0.5;
            let stripe2 = (brush_coord * 2200.0 + b * 8.0).sin() * 0.5 + 0.5;
            let micro = (micro_coord * 1200.0 + c * 10.0).sin() * 0.5 + 0.5;

            let grain = stripe1 * 0.55 + stripe2 * 0.35 + micro * 0.10;

            let vignette = 1.0
                - (smoothstep(0.0, 0.12, nx.min(1.0 - nx)) * 0.12
                    + smoothstep(0.0, 0.12, ny.min(1.0 - ny)) * 0.12);
            let base = 190.0 + (grain - 0.5) * 34.0 + (a - 0.5) * 10.0;
            let tone = base * vignette;

            let r = u8_from_f32(tone * 0.96);
            let g = u8_from_f32(tone * 0.99);
            let b = u8_from_f32(tone * 1.04);

            let scratch_pick = noise01(x / 3, y / 3, seed ^ 0x27d4eb2d);
            let scratch = if scratch_pick > 0.996 {
                18.0
            } else if scratch_pick < 0.004 {
                -14.0
            } else {
                0.0
            };

            let rr = u8_from_f32(r as f32 + scratch);
            let gg = u8_from_f32(g as f32 + scratch * 0.8);
            let bb = u8_from_f32(b as f32 + scratch * 0.6);

            img.set_pixel(x as u32, y as u32, Color::from_rgba(rr, gg, bb, 255));
        }
    }

    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

fn gen_grill_tex(w: u16, h: u16) -> Texture2D {
    let mut img = Image::gen_image_color(w, h, Color::from_rgba(70, 76, 84, 255));
    let period = 10i32;
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let u = x.rem_euclid(period);
            let v = y.rem_euclid(period);
            let slot = (2..=7).contains(&u) && (v == 3 || v == 4);
            let edge = (v == 2 || v == 5) && (2..=7).contains(&u);
            let rivet = (u == 1 || u == 8) && (v == 3 || v == 4);

            let mut r = 82.0;
            let mut g = 88.0;
            let mut b = 98.0;

            if slot {
                r = 22.0;
                g = 24.0;
                b = 28.0;
            } else if edge {
                r = 126.0;
                g = 132.0;
                b = 142.0;
            } else if rivet {
                r = 150.0;
                g = 156.0;
                b = 166.0;
            }

            img.set_pixel(
                x as u32,
                y as u32,
                Color::from_rgba(u8_from_f32(r), u8_from_f32(g), u8_from_f32(b), 255),
            );
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

fn gen_glow_tex(w: u16, h: u16) -> Texture2D {
    let mut img = Image::gen_image_color(w, h, Color::from_rgba(0, 0, 0, 0));
    let cx = (w as f32 - 1.0) * 0.5;
    let cy = (h as f32 - 1.0) * 0.5;
    let maxd = (cx * cx + cy * cy).sqrt().max(1.0);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt() / maxd;
            let v = (1.0 - d).max(0.0);
            let v = v * v * (1.2 - 0.2 * d);
            let a = u8_from_f32(v * 255.0);
            img.set_pixel(x as u32, y as u32, Color::from_rgba(255, 255, 255, a));
        }
    }
    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

fn gen_belt_tex(w: u16, h: u16, seed: u32) -> Texture2D {
    let mut img = Image::gen_image_color(w, h, Color::from_rgba(72, 80, 92, 255));
    let pw = 14i32;
    let ph = 14i32;

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let u = x.rem_euclid(pw);
            let v = y.rem_euclid(ph);

            let diag1 = ((u - v).abs() <= 1) || ((u - v).abs() == 2 && (u == 0 || v == 0));
            let diag2 = ((u + v - (pw - 1)).abs() <= 1)
                || ((u + v - (pw - 1)).abs() == 2 && (u == pw - 1 || v == 0));

            let rod = v == 6 || v == 7;
            let pin = (u == 2 || u == 11) && (v == 6 || v == 7);

            let n = noise01(x, y, seed);
            let mut base = 0.33 + (n - 0.5) * 0.06;

            if diag1 || diag2 {
                base += 0.22;
            }
            if rod {
                base += 0.10;
            }
            if pin {
                base += 0.18;
            }

            let edge_dark = 1.0
                - smoothstep(
                    0.0,
                    0.18,
                    (y as f32 / (h as f32 - 1.0)).min(1.0 - y as f32 / (h as f32 - 1.0)),
                );
            base -= edge_dark * 0.10;

            let tone = (base * 255.0).clamp(0.0, 255.0);
            let r = u8_from_f32(tone * 0.92);
            let g = u8_from_f32(tone * 0.98);
            let b = u8_from_f32(tone * 1.06);

            img.set_pixel(x as u32, y as u32, Color::from_rgba(r, g, b, 255));
        }
    }

    let tex = Texture2D::from_image(&img);
    tex.set_filter(FilterMode::Linear);
    tex
}

fn draw_texture_stretched(tex: &Texture2D, rect: Rect, tint: Color) {
    draw_texture_ex(
        tex,
        rect.x,
        rect.y,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(rect.w, rect.h)),
            ..Default::default()
        },
    );
}

fn draw_tiled_along_x(tex: &Texture2D, rect: Rect, scroll_world_px: f32, tint: Color) {
    let th = tex.height().max(1.0);
    let tw = tex.width().max(1.0);
    let scale = rect.h / th;
    let tile_w = (tw * scale).max(1.0);
    let offset = scroll_world_px.rem_euclid(tile_w);

    let mut x = rect.x - offset;
    let end = rect.x + rect.w;

    while x < end {
        let tile_left = x;
        let tile_right = x + tile_w;

        let draw_left = tile_left.max(rect.x);
        let draw_right = tile_right.min(end);
        let draw_w = (draw_right - draw_left).max(0.0);

        if draw_w > 0.01 {
            let src_x = ((draw_left - tile_left) / scale).clamp(0.0, tw);
            let src_w = (draw_w / scale).clamp(0.0, tw - src_x);

            draw_texture_ex(
                tex,
                draw_left,
                rect.y,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(draw_w, rect.h)),
                    source: Some(Rect::new(src_x, 0.0, src_w, th)),
                    ..Default::default()
                },
            );
        }

        x += tile_w;
    }
}

fn draw_tiled_along_y(tex: &Texture2D, rect: Rect, scroll_world_px: f32, tint: Color) {
    let th = tex.height().max(1.0);
    let tw = tex.width().max(1.0);
    let scale = rect.w / tw;
    let tile_h = (th * scale).max(1.0);
    let offset = scroll_world_px.rem_euclid(tile_h);

    let mut y = rect.y - offset;
    let end = rect.y + rect.h;

    while y < end {
        let tile_top = y;
        let tile_bottom = y + tile_h;

        let draw_top = tile_top.max(rect.y);
        let draw_bottom = tile_bottom.min(end);
        let draw_h = (draw_bottom - draw_top).max(0.0);

        if draw_h > 0.01 {
            let src_y = ((draw_top - tile_top) / scale).clamp(0.0, th);
            let src_h = (draw_h / scale).clamp(0.0, th - src_y);

            draw_texture_ex(
                tex,
                rect.x,
                draw_top,
                tint,
                DrawTextureParams {
                    dest_size: Some(vec2(rect.w, draw_h)),
                    source: Some(Rect::new(0.0, src_y, tw, src_h)),
                    ..Default::default()
                },
            );
        }

        y += tile_h;
    }
}

pub struct OvenVisual {
    metal_h: Texture2D,
    metal_v: Texture2D,
    grill: Texture2D,
    glow: Texture2D,
    belt: Texture2D,
}

impl OvenVisual {
    pub fn new() -> Self {
        let metal_h = gen_brushed_metal_tex(256, 256, true, 0x1a2b3c4d);
        let metal_v = gen_brushed_metal_tex(256, 256, false, 0x5e6f7789);
        let grill = gen_grill_tex(96, 96);
        let glow = gen_glow_tex(96, 96);
        let belt = gen_belt_tex(140, 56, 0xabcddcba);

        Self {
            metal_h,
            metal_v,
            grill,
            glow,
            belt,
        }
    }

    pub fn draw(&self, rect: Rect, orientation: sim::BlockOrientation, time: f32) {
        let flow_horizontal = rect.w >= rect.h;
        let flow_orientation = if flow_horizontal {
            if matches!(
                orientation,
                sim::BlockOrientation::West | sim::BlockOrientation::North
            ) {
                sim::BlockOrientation::West
            } else {
                sim::BlockOrientation::East
            }
        } else if matches!(
            orientation,
            sim::BlockOrientation::North | sim::BlockOrientation::West
        ) {
            sim::BlockOrientation::North
        } else {
            sim::BlockOrientation::South
        };

        let axis = orientation_axis(flow_orientation);
        let normal = vec2(-axis.y, axis.x);
        let center = vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5);

        let short_side = rect.w.min(rect.h);
        let long_side = rect.w.max(rect.h);

        let wall = (short_side * 0.12).clamp(6.0, 18.0);
        let bevel = (short_side * 0.03).clamp(1.2, 3.6);

        let heat = (time * 2.2).sin() * 0.5 + 0.5;
        let pulse = (time * 0.9).sin() * 0.5 + 0.5;

        let metal_tex = if flow_horizontal {
            &self.metal_h
        } else {
            &self.metal_v
        };

        let shadow_off = vec2(short_side * 0.05, short_side * 0.055);
        draw_rectangle(
            rect.x + shadow_off.x,
            rect.y + shadow_off.y,
            rect.w,
            rect.h,
            with_alpha(Color::from_rgba(0, 0, 0, 255), 0.22),
        );

        draw_texture_stretched(metal_tex, rect, Color::from_rgba(255, 255, 255, 255));

        draw_rectangle(
            rect.x,
            rect.y,
            rect.w,
            rect.h * 0.18,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.10),
        );
        draw_rectangle(
            rect.x,
            rect.y,
            rect.w * 0.14,
            rect.h,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.07),
        );
        draw_rectangle(
            rect.x,
            rect.y + rect.h * 0.82,
            rect.w,
            rect.h * 0.18,
            with_alpha(Color::from_rgba(0, 0, 0, 255), 0.14),
        );
        draw_rectangle(
            rect.x + rect.w * 0.86,
            rect.y,
            rect.w * 0.14,
            rect.h,
            with_alpha(Color::from_rgba(0, 0, 0, 255), 0.12),
        );

        draw_rectangle_lines(
            rect.x + 0.5,
            rect.y + 0.5,
            (rect.w - 1.0).max(1.0),
            (rect.h - 1.0).max(1.0),
            bevel * 1.45,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.52),
        );
        draw_rectangle_lines(
            rect.x + bevel,
            rect.y + bevel,
            (rect.w - bevel * 2.0).max(1.0),
            (rect.h - bevel * 2.0).max(1.0),
            1.0,
            with_alpha(Color::from_rgba(0, 0, 0, 255), 0.18),
        );

        let panel_count = if flow_horizontal {
            (rect.w / (short_side * 0.72).max(28.0))
                .floor()
                .clamp(2.0, 6.0) as i32
        } else {
            (rect.h / (short_side * 0.72).max(28.0))
                .floor()
                .clamp(2.0, 6.0) as i32
        };

        for i in 1..panel_count {
            let t = i as f32 / panel_count as f32;
            if flow_horizontal {
                let x = rect.x + rect.w * t;
                draw_line(
                    x,
                    rect.y + bevel,
                    x,
                    rect.y + rect.h - bevel,
                    1.0,
                    with_alpha(Color::from_rgba(0, 0, 0, 255), 0.22),
                );
                draw_line(
                    x + 1.0,
                    rect.y + bevel,
                    x + 1.0,
                    rect.y + rect.h - bevel,
                    1.0,
                    with_alpha(Color::from_rgba(255, 255, 255, 255), 0.11),
                );
                let mut y = rect.y + wall * 0.5;
                while y <= rect.y + rect.h - wall * 0.5 {
                    draw_circle(
                        x,
                        y,
                        1.1,
                        with_alpha(Color::from_rgba(240, 244, 250, 255), 0.62),
                    );
                    y += (short_side * 0.28).clamp(8.0, 14.0);
                }
            } else {
                let y = rect.y + rect.h * t;
                draw_line(
                    rect.x + bevel,
                    y,
                    rect.x + rect.w - bevel,
                    y,
                    1.0,
                    with_alpha(Color::from_rgba(0, 0, 0, 255), 0.22),
                );
                draw_line(
                    rect.x + bevel,
                    y + 1.0,
                    rect.x + rect.w - bevel,
                    y + 1.0,
                    1.0,
                    with_alpha(Color::from_rgba(255, 255, 255, 255), 0.11),
                );
                let mut x = rect.x + wall * 0.5;
                while x <= rect.x + rect.w - wall * 0.5 {
                    draw_circle(
                        x,
                        y,
                        1.1,
                        with_alpha(Color::from_rgba(240, 244, 250, 255), 0.62),
                    );
                    x += (short_side * 0.28).clamp(8.0, 14.0);
                }
            }
        }

        let tunnel = if flow_horizontal {
            Rect::new(
                rect.x + wall * 1.15,
                rect.y + rect.h * 0.28,
                (rect.w - wall * 2.3).max(1.0),
                (rect.h * 0.44).max(2.0),
            )
        } else {
            Rect::new(
                rect.x + rect.w * 0.28,
                rect.y + wall * 1.15,
                (rect.w * 0.44).max(2.0),
                (rect.h - wall * 2.3).max(1.0),
            )
        };

        let tunnel_frame = rect_inset(tunnel, bevel * 0.5);
        draw_texture_stretched(
            metal_tex,
            tunnel_frame,
            with_alpha(Color::from_rgba(170, 176, 184, 255), 0.82),
        );
        draw_rectangle_lines(
            tunnel_frame.x + 0.4,
            tunnel_frame.y + 0.4,
            (tunnel_frame.w - 0.8).max(1.0),
            (tunnel_frame.h - 0.8).max(1.0),
            1.2,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.33),
        );

        let tunnel_in = rect_inset(tunnel_frame, (short_side * 0.04).clamp(2.2, 6.0));
        draw_rectangle(
            tunnel_in.x,
            tunnel_in.y,
            tunnel_in.w,
            tunnel_in.h,
            Color::from_rgba(16, 18, 20, 245),
        );

        let glow_alpha = 0.12 + heat * 0.22;
        draw_texture_stretched(
            &self.glow,
            tunnel_in,
            with_alpha(Color::from_rgba(255, 150, 70, 255), glow_alpha),
        );

        let belt_thickness = if flow_horizontal {
            (tunnel_in.h * 0.62).clamp(6.0, tunnel_in.h)
        } else {
            (tunnel_in.w * 0.62).clamp(6.0, tunnel_in.w)
        };

        let belt_rect = if flow_horizontal {
            Rect::new(
                tunnel_in.x,
                tunnel_in.y + tunnel_in.h * 0.5 - belt_thickness * 0.5,
                tunnel_in.w,
                belt_thickness,
            )
        } else {
            Rect::new(
                tunnel_in.x + tunnel_in.w * 0.5 - belt_thickness * 0.5,
                tunnel_in.y,
                belt_thickness,
                tunnel_in.h,
            )
        };

        let rail = (short_side * 0.035).clamp(1.6, 4.2);
        if flow_horizontal {
            draw_rectangle(
                tunnel_in.x,
                belt_rect.y - rail,
                tunnel_in.w,
                rail,
                with_alpha(Color::from_rgba(36, 40, 46, 255), 0.92),
            );
            draw_rectangle(
                tunnel_in.x,
                belt_rect.y + belt_rect.h,
                tunnel_in.w,
                rail,
                with_alpha(Color::from_rgba(36, 40, 46, 255), 0.92),
            );
            draw_line(
                tunnel_in.x,
                belt_rect.y - rail,
                tunnel_in.x + tunnel_in.w,
                belt_rect.y - rail,
                1.0,
                with_alpha(Color::from_rgba(255, 255, 255, 255), 0.16),
            );
            draw_line(
                tunnel_in.x,
                belt_rect.y + belt_rect.h + rail,
                tunnel_in.x + tunnel_in.w,
                belt_rect.y + belt_rect.h + rail,
                1.0,
                with_alpha(Color::from_rgba(0, 0, 0, 255), 0.18),
            );
        } else {
            draw_rectangle(
                belt_rect.x - rail,
                tunnel_in.y,
                rail,
                tunnel_in.h,
                with_alpha(Color::from_rgba(36, 40, 46, 255), 0.92),
            );
            draw_rectangle(
                belt_rect.x + belt_rect.w,
                tunnel_in.y,
                rail,
                tunnel_in.h,
                with_alpha(Color::from_rgba(36, 40, 46, 255), 0.92),
            );
            draw_line(
                belt_rect.x - rail,
                tunnel_in.y,
                belt_rect.x - rail,
                tunnel_in.y + tunnel_in.h,
                1.0,
                with_alpha(Color::from_rgba(255, 255, 255, 255), 0.16),
            );
            draw_line(
                belt_rect.x + belt_rect.w + rail,
                tunnel_in.y,
                belt_rect.x + belt_rect.w + rail,
                tunnel_in.y + tunnel_in.h,
                1.0,
                with_alpha(Color::from_rgba(0, 0, 0, 255), 0.18),
            );
        }

        draw_rectangle(
            belt_rect.x,
            belt_rect.y,
            belt_rect.w,
            belt_rect.h,
            with_alpha(Color::from_rgba(44, 50, 58, 255), 0.9),
        );
        let sign = if matches!(
            flow_orientation,
            sim::BlockOrientation::East | sim::BlockOrientation::South
        ) {
            1.0
        } else {
            -1.0
        };
        let speed = (38.0 + heat * 18.0) * sign;
        let scroll = time * speed;

        if flow_horizontal {
            draw_tiled_along_x(
                &self.belt,
                belt_rect,
                scroll,
                Color::from_rgba(255, 255, 255, 255),
            );
        } else {
            draw_tiled_along_y(
                &self.belt,
                belt_rect,
                scroll,
                Color::from_rgba(255, 255, 255, 255),
            );
        }

        draw_rectangle(
            belt_rect.x,
            belt_rect.y,
            belt_rect.w,
            belt_rect.h,
            with_alpha(Color::from_rgba(255, 180, 90, 255), 0.035 + heat * 0.04),
        );

        draw_rectangle_lines(
            belt_rect.x + 0.4,
            belt_rect.y + 0.4,
            (belt_rect.w - 0.8).max(1.0),
            (belt_rect.h - 0.8).max(1.0),
            1.0,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.22),
        );
        draw_rectangle_lines(
            belt_rect.x + 1.4,
            belt_rect.y + 1.4,
            (belt_rect.w - 2.8).max(1.0),
            (belt_rect.h - 2.8).max(1.0),
            1.0,
            with_alpha(Color::from_rgba(0, 0, 0, 255), 0.16),
        );

        let roller_r = (belt_thickness * 0.18).clamp(2.2, 7.0);
        if flow_horizontal {
            let cy = belt_rect.y + belt_rect.h * 0.5;
            let lx = belt_rect.x + roller_r * 0.9;
            let rx = belt_rect.x + belt_rect.w - roller_r * 0.9;
            draw_circle(
                lx,
                cy,
                roller_r,
                with_alpha(Color::from_rgba(92, 100, 112, 255), 0.95),
            );
            draw_circle(
                rx,
                cy,
                roller_r,
                with_alpha(Color::from_rgba(92, 100, 112, 255), 0.95),
            );
            draw_circle_lines(
                lx,
                cy,
                roller_r,
                1.0,
                with_alpha(Color::from_rgba(240, 244, 250, 255), 0.28),
            );
            draw_circle_lines(
                rx,
                cy,
                roller_r,
                1.0,
                with_alpha(Color::from_rgba(240, 244, 250, 255), 0.28),
            );
        } else {
            let cx = belt_rect.x + belt_rect.w * 0.5;
            let ty = belt_rect.y + roller_r * 0.9;
            let by = belt_rect.y + belt_rect.h - roller_r * 0.9;
            draw_circle(
                cx,
                ty,
                roller_r,
                with_alpha(Color::from_rgba(92, 100, 112, 255), 0.95),
            );
            draw_circle(
                cx,
                by,
                roller_r,
                with_alpha(Color::from_rgba(92, 100, 112, 255), 0.95),
            );
            draw_circle_lines(
                cx,
                ty,
                roller_r,
                1.0,
                with_alpha(Color::from_rgba(240, 244, 250, 255), 0.28),
            );
            draw_circle_lines(
                cx,
                by,
                roller_r,
                1.0,
                with_alpha(Color::from_rgba(240, 244, 250, 255), 0.28),
            );
        }

        let mouth_w = (short_side * 0.18).clamp(7.5, 14.0);
        let mouth_pad = (short_side * 0.06).clamp(1.6, 4.4);

        let inlet_rect;
        let outlet_rect;

        if flow_horizontal {
            let left = Rect::new(
                rect.x + mouth_pad,
                tunnel_frame.y + mouth_pad,
                mouth_w,
                (tunnel_frame.h - mouth_pad * 2.0).max(1.0),
            );
            let right = Rect::new(
                rect.x + rect.w - mouth_pad - mouth_w,
                tunnel_frame.y + mouth_pad,
                mouth_w,
                (tunnel_frame.h - mouth_pad * 2.0).max(1.0),
            );
            if matches!(flow_orientation, sim::BlockOrientation::East) {
                inlet_rect = left;
                outlet_rect = right;
            } else {
                inlet_rect = right;
                outlet_rect = left;
            }
        } else {
            let top = Rect::new(
                tunnel_frame.x + mouth_pad,
                rect.y + mouth_pad,
                (tunnel_frame.w - mouth_pad * 2.0).max(1.0),
                mouth_w,
            );
            let bottom = Rect::new(
                tunnel_frame.x + mouth_pad,
                rect.y + rect.h - mouth_pad - mouth_w,
                (tunnel_frame.w - mouth_pad * 2.0).max(1.0),
                mouth_w,
            );
            if matches!(flow_orientation, sim::BlockOrientation::South) {
                inlet_rect = top;
                outlet_rect = bottom;
            } else {
                inlet_rect = bottom;
                outlet_rect = top;
            }
        }

        for (mouth, is_outlet) in [(inlet_rect, false), (outlet_rect, true)] {
            draw_texture_stretched(
                metal_tex,
                mouth,
                with_alpha(Color::from_rgba(170, 176, 184, 255), 0.86),
            );
            draw_rectangle_lines(
                mouth.x + 0.4,
                mouth.y + 0.4,
                (mouth.w - 0.8).max(1.0),
                (mouth.h - 0.8).max(1.0),
                1.0,
                with_alpha(Color::from_rgba(255, 255, 255, 255), 0.28),
            );

            let mouth_in = rect_inset(mouth, 1.4);
            draw_rectangle(
                mouth_in.x,
                mouth_in.y,
                mouth_in.w,
                mouth_in.h,
                Color::from_rgba(14, 16, 18, 235),
            );

            let belt_mouth = if flow_horizontal {
                rect_inset_xy(mouth_in, 0.4, (mouth_in.h * 0.18).clamp(1.0, 4.0))
            } else {
                rect_inset_xy(mouth_in, (mouth_in.w * 0.18).clamp(1.0, 4.0), 0.4)
            };

            if flow_horizontal {
                draw_tiled_along_x(
                    &self.belt,
                    belt_mouth,
                    scroll,
                    Color::from_rgba(255, 255, 255, 255),
                );
            } else {
                draw_tiled_along_y(
                    &self.belt,
                    belt_mouth,
                    scroll,
                    Color::from_rgba(255, 255, 255, 255),
                );
            }

            let bolt_step = (short_side * 0.22).clamp(7.0, 12.0);
            if flow_horizontal {
                let mut y = mouth.y + 3.8;
                while y <= mouth.y + mouth.h - 3.8 {
                    draw_circle(
                        mouth.x + 2.7,
                        y,
                        1.0,
                        with_alpha(Color::from_rgba(240, 244, 250, 255), 0.6),
                    );
                    draw_circle(
                        mouth.x + mouth.w - 2.7,
                        y,
                        1.0,
                        with_alpha(Color::from_rgba(240, 244, 250, 255), 0.6),
                    );
                    y += bolt_step;
                }
            } else {
                let mut x = mouth.x + 3.8;
                while x <= mouth.x + mouth.w - 3.8 {
                    draw_circle(
                        x,
                        mouth.y + 2.7,
                        1.0,
                        with_alpha(Color::from_rgba(240, 244, 250, 255), 0.6),
                    );
                    draw_circle(
                        x,
                        mouth.y + mouth.h - 2.7,
                        1.0,
                        with_alpha(Color::from_rgba(240, 244, 250, 255), 0.6),
                    );
                    x += bolt_step;
                }
            }

            if is_outlet {
                let hot = 0.14 + heat * 0.22 + pulse * 0.08;
                draw_texture_stretched(
                    &self.glow,
                    mouth_in,
                    with_alpha(Color::from_rgba(255, 170, 80, 255), hot),
                );
                for i in 0..5 {
                    let phase = (time * 0.22 + i as f32 * 0.19).fract();
                    let drift = (time * 1.2 + i as f32 * 1.7).sin() * short_side * 0.04;
                    let pos = vec2(mouth_in.x + mouth_in.w * 0.5, mouth_in.y + mouth_in.h * 0.5)
                        + axis * (phase * short_side * 0.34)
                        + normal * drift;
                    draw_circle(
                        pos.x,
                        pos.y,
                        (short_side * (0.05 + phase * 0.05)).clamp(2.0, 7.0),
                        with_alpha(Color::from_rgba(210, 218, 230, 255), 0.12 * (1.0 - phase)),
                    );
                }
            }
        }

        let grill_w = if flow_horizontal {
            (rect.w * 0.18).clamp(18.0, rect.w * 0.3)
        } else {
            (rect.w * 0.26).clamp(18.0, rect.w * 0.6)
        };
        let grill_h = if flow_horizontal {
            (rect.h * 0.18).clamp(10.0, rect.h * 0.35)
        } else {
            (rect.h * 0.18).clamp(18.0, rect.h * 0.3)
        };

        let g1 = if flow_horizontal {
            Rect::new(
                rect.x + rect.w * 0.22,
                rect.y + rect.h * 0.10,
                grill_w,
                grill_h,
            )
        } else {
            Rect::new(
                rect.x + rect.w * 0.10,
                rect.y + rect.h * 0.22,
                grill_w,
                grill_h,
            )
        };
        let g2 = if flow_horizontal {
            Rect::new(
                rect.x + rect.w * 0.56,
                rect.y + rect.h * 0.10,
                grill_w,
                grill_h,
            )
        } else {
            Rect::new(
                rect.x + rect.w * 0.10,
                rect.y + rect.h * 0.56,
                grill_w,
                grill_h,
            )
        };

        draw_texture_stretched(
            &self.grill,
            g1,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.92),
        );
        draw_texture_stretched(
            &self.grill,
            g2,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.92),
        );
        draw_rectangle_lines(
            g1.x + 0.4,
            g1.y + 0.4,
            (g1.w - 0.8).max(1.0),
            (g1.h - 0.8).max(1.0),
            1.0,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.22),
        );
        draw_rectangle_lines(
            g2.x + 0.4,
            g2.y + 0.4,
            (g2.w - 0.8).max(1.0),
            (g2.h - 0.8).max(1.0),
            1.0,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.22),
        );

        let inlet_center = center - axis * (long_side * 0.5 - wall * 0.4);
        let panel_w = (short_side * 0.28).clamp(12.0, 26.0);
        let panel_h = (short_side * 0.22).clamp(10.0, 22.0);
        let panel_pos =
            inlet_center + normal * (short_side * 0.36) - vec2(panel_w * 0.5, panel_h * 0.5);
        let panel = Rect::new(panel_pos.x, panel_pos.y, panel_w, panel_h);

        draw_rectangle(
            panel.x + 1.2,
            panel.y + 1.2,
            panel.w,
            panel.h,
            with_alpha(Color::from_rgba(0, 0, 0, 255), 0.18),
        );
        draw_rectangle(
            panel.x,
            panel.y,
            panel.w,
            panel.h,
            Color::from_rgba(44, 48, 54, 238),
        );
        draw_rectangle_lines(
            panel.x + 0.4,
            panel.y + 0.4,
            (panel.w - 0.8).max(1.0),
            (panel.h - 0.8).max(1.0),
            1.0,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.22),
        );

        let screen = Rect::new(
            panel.x + panel.w * 0.10,
            panel.y + panel.h * 0.22,
            panel.w * 0.56,
            panel.h * 0.56,
        );
        draw_rectangle(
            screen.x,
            screen.y,
            screen.w,
            screen.h,
            Color::from_rgba(14, 18, 24, 250),
        );
        draw_rectangle(
            screen.x + 1.0,
            screen.y + 1.0,
            (screen.w - 2.0).max(1.0),
            (screen.h - 2.0).max(1.0),
            with_alpha(Color::from_rgba(60, 120, 190, 255), 0.22 + pulse * 0.08),
        );
        draw_rectangle_lines(
            screen.x + 0.4,
            screen.y + 0.4,
            (screen.w - 0.8).max(1.0),
            (screen.h - 0.8).max(1.0),
            1.0,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.18),
        );

        for i in 0..5 {
            let t = (i as f32 + 0.5) / 5.0;
            let bar =
                (0.25 + (time * 1.1 + i as f32 * 0.9).sin() * 0.18 + heat * 0.22).clamp(0.05, 0.95);
            let bx = screen.x + screen.w * (0.12 + t * 0.76);
            let by = screen.y + screen.h * (0.86 - bar * 0.72);
            draw_rectangle(
                bx - 1.2,
                by,
                2.4,
                screen.y + screen.h * 0.86 - by,
                with_alpha(Color::from_rgba(120, 220, 255, 255), 0.40),
            );
        }

        let led_r = (short_side * 0.03).clamp(1.5, 3.2);
        let led_x = panel.x + panel.w * 0.82;
        let led_y1 = panel.y + panel.h * 0.34;
        let led_y2 = panel.y + panel.h * 0.68;

        let ok_a = 0.35 + pulse * 0.35;
        let heat_a = 0.20 + heat * 0.55;

        draw_circle(
            led_x,
            led_y1,
            led_r,
            with_alpha(Color::from_rgba(80, 255, 140, 255), ok_a),
        );
        draw_circle(
            led_x,
            led_y2,
            led_r,
            with_alpha(Color::from_rgba(255, 190, 90, 255), heat_a),
        );
        draw_circle_lines(
            led_x,
            led_y1,
            led_r,
            1.0,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.18),
        );
        draw_circle_lines(
            led_x,
            led_y2,
            led_r,
            1.0,
            with_alpha(Color::from_rgba(255, 255, 255, 255), 0.18),
        );

        let stack_w = (short_side * 0.12).clamp(3.0, 10.0);
        let stack_h = (short_side * 0.34).clamp(8.0, 20.0);

        let stack_a = center - axis * (long_side * 0.20) + normal * (short_side * 0.28);
        let stack_b = center - axis * (long_side * 0.05) + normal * (short_side * 0.24);

        for (idx, base) in [stack_a, stack_b].into_iter().enumerate() {
            let stem = Rect::new(base.x - stack_w * 0.5, base.y - stack_h, stack_w, stack_h);
            draw_texture_stretched(
                metal_tex,
                stem,
                with_alpha(Color::from_rgba(180, 186, 194, 255), 0.84),
            );
            draw_rectangle_lines(
                stem.x + 0.4,
                stem.y + 0.4,
                (stem.w - 0.8).max(1.0),
                (stem.h - 0.8).max(1.0),
                1.0,
                with_alpha(Color::from_rgba(255, 255, 255, 255), 0.18),
            );
            draw_circle(
                base.x,
                base.y - stack_h,
                stack_w * 0.35,
                with_alpha(Color::from_rgba(240, 244, 250, 255), 0.38),
            );

            for puff in 0..4 {
                let phase = (time * 0.22 + puff as f32 * 0.21 + idx as f32 * 0.17).fract();
                let drift = (time * 1.0 + puff as f32 * 1.7).sin() * stack_w * 0.55;
                let p = vec2(base.x + drift, base.y - stack_h - phase * stack_h * 1.55);
                draw_circle(
                    p.x,
                    p.y,
                    stack_w * (0.22 + phase * 0.26),
                    with_alpha(Color::from_rgba(200, 210, 224, 255), 0.16 * (1.0 - phase)),
                );
            }
        }
    }
}

thread_local! {
    static OVEN_VISUAL: RefCell<Option<OvenVisual>> = const { RefCell::new(None) };
}

pub(crate) fn draw_dryer_oven_visual(rect: Rect, orientation: sim::BlockOrientation, time: f32) {
    OVEN_VISUAL.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_none() {
            *opt = Some(OvenVisual::new());
        }
        opt.as_ref().unwrap().draw(rect, orientation, time);
    });
}
