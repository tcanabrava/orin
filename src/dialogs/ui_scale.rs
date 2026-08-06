/// UI scale never goes below the natural size, and caps out well before it
/// gets impractical. `pub` so the Options page's zoom slider
/// (`menu::pages::options`) can share the exact same bounds — the slider is
/// the only way to change the scale; an earlier Arrow Up/Down keyboard
/// shortcut was removed because it conflicted with Tab/arrow-key UI
/// navigation.
pub const MIN_SCALE: f32 = 1.0;
pub const MAX_SCALE: f32 = 8.0;
