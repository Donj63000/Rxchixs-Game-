use macroquad::prelude::*;

const UI_PASS_RESETS_CAMERA: bool = true;
const UI_PASS_RESETS_MATERIAL: bool = true;

#[inline]
fn ui_pass_resets_camera() -> bool {
    UI_PASS_RESETS_CAMERA
}

#[inline]
fn ui_pass_resets_material() -> bool {
    UI_PASS_RESETS_MATERIAL
}

/// Begin a UI/text pass safely.
///
/// Why: in macroquad, text is rendered as textured quads.
/// If a custom Material is still bound (gl_use_material), the font atlas will be drawn
/// using that shader too -> you get solid blocks instead of glyphs.
///
/// Call this right before drawing any screen-space UI / text.
#[inline]
pub(crate) fn begin_ui_pass() {
    // UI is screen-space.
    if ui_pass_resets_camera() {
        set_default_camera();
    }

    // Absolutely critical: reset any custom shader/material that might still be active.
    if ui_pass_resets_material() {
        gl_use_default_material();
    }
}

/// Force-reset to default material (useful as a cheap "safety pin").
/// You can call it before draw_text/draw_texture in UI if you suspect state leaks.
#[inline]
#[allow(dead_code)]
pub(crate) fn ensure_default_material() {
    if ui_pass_resets_material() {
        gl_use_default_material();
    }
}

/// Safe wrapper to use a custom material without leaking it to other draw calls.
///
/// Use this instead of calling gl_use_material(...) directly.
/// It guarantees the default material is restored even if you early-return.
///
/// NOTE: keep your materials alive (stored in your GameState/resources) and avoid nesting
/// different materials without restoring (this wrapper restores to default).
#[inline]
#[allow(dead_code)]
pub(crate) fn with_material<R>(material: &Material, f: impl FnOnce() -> R) -> R {
    gl_use_material(material);
    let out = f();
    if ui_pass_resets_material() {
        gl_use_default_material();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_pass_invariants_require_camera_and_material_reset() {
        assert!(ui_pass_resets_camera());
        assert!(ui_pass_resets_material());
    }
}
