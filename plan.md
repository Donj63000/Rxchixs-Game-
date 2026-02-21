1) AJOUTER un fichier : src/render_safety.rs

Crée ce fichier nouveau : src/render_safety.rs

use macroquad::prelude::*;

/// Begin a UI/text pass safely.
///
/// Why: in macroquad, text is rendered as textured quads.
/// If a custom Material is still bound (gl_use_material), the font atlas will be drawn
/// using that shader too -> you get solid blocks instead of glyphs.
///
/// Call this right before drawing any screen-space UI / text.
#[inline]
pub(crate) fn begin_ui_pass() {
// UI is screen-space
set_default_camera();

    // Absolutely critical: reset any custom shader/material that might still be active.
    gl_use_default_material();
}

/// Force-reset to default material (useful as a cheap "safety pin").
/// You can call it before draw_text/draw_texture in UI if you suspect state leaks.
#[inline]
#[allow(dead_code)]
pub(crate) fn ensure_default_material() {
gl_use_default_material();
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
gl_use_default_material();
out
}
2) MODIFIER src/main.rs : déclarer + importer le module

Dans src/main.rs, en haut avec les autres mod ...;, ajoute :

mod render_safety;

Et dans les use ...; (là où tu as déjà use utilitaires::*; etc.), ajoute :

use render_safety::*;

🎯 Objectif : que begin_ui_pass() soit visible partout via vos use super::*;.

3) PATCH CRITIQUE : src/modes.rs (et tous tes “modes” où tu dessines du texte)

Le point clé : juste après ton rendu monde (caméra monde), au moment où tu fais set_default_camera();, remplace par begin_ui_pass();.

A) Dans run_play_frame

Tu as actuellement un bloc comme ça (je te montre l’endroit typique) :

draw_lighting_region(&state.props, &state.palette, time, visible_bounds);
set_default_camera();

draw_rectangle_lines( ... );
...
draw_text(...);

➡️ Remplace uniquement set_default_camera(); par :

begin_ui_pass();

Donc ça donne :

draw_lighting_region(&state.props, &state.palette, time, visible_bounds);
begin_ui_pass();

draw_rectangle_lines(
map_view_rect.x + 0.5,
map_view_rect.y + 0.5,
map_view_rect.w - 1.0,
map_view_rect.h - 1.0,
2.0,
Color::from_rgba(170, 213, 237, 135),
);

// Tout ce qui suit = UI/HUD/texte en screen-space
draw_ambient_dust(&state.palette, time);
draw_vignette(&state.palette);
...
B) Dans run_editor_frame

Même principe : tu as un set_default_camera(); après le rendu monde.

Remplace-le par :

begin_ui_pass();
4) IMPORTANT : sécuriser tous les endroits où tu utilises gl_use_material

Même si le patch ci-dessus règle 99% des cas (car il “nettoie” avant l’UI), la vraie robustesse c’est : plus jamais de leak de shader.

Donc :

Cherche dans le projet : gl_use_material(

Chaque occurrence doit être soit :

suivie de gl_use_default_material();

soit remplacée par with_material(&material, || { ... })

Exemple de remplacement (recommandé)

Avant (fragile) :

gl_use_material(&my_material);
draw_texture(my_tex, x, y, WHITE);
// OUPS: si on return/continue/branch -> pas de reset -> texte cassé

Après (robuste) :

with_material(&my_material, || {
draw_texture(my_tex, x, y, WHITE);
});

✅ Même si tu fais des branches/returns dans le bloc : tu peux structurer pour éviter, mais déjà ça réduit énormément les oublis.

5) Si tu veux le “fix ceinture + bretelles” (ultra robuste)

Si tu as des fonctions UI qui peuvent être appelées depuis plusieurs endroits (pawn bar, context menu, fiche perso), tu peux ajouter au tout début de ces fonctions :

ensure_default_material();

Ça coûte quasi rien, et ça empêche qu’un appel UI “isolé” rende du texte avec un shader leaké.

Typiquement au début de :

draw_pawn_bar(...)

draw_context_menu(...)

draw_character_sheet(...)