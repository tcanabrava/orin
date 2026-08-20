// SPDX-License-Identifier: MIT

//! A real curved tie, drawn as a [`UiMaterial`] fragment shader
//! (`assets/shaders/music_score_tie.wgsl`) rather than the flat rectangle
//! plain `bevy_ui` `Node`/`BackgroundColor` primitives are limited to —
//! the same "custom shader for a shape a plain `Node` can't express"
//! pattern `gameplay::note_tail_2d::NoteTail2dMaterial` already
//! established for the falling-note comet tail, applied here to a small,
//! static shape instead of an animated one. One shared material instance
//! covers every tie in the panel: unlike the note tail (animated per-note,
//! so each note needs its own material instance to carry its own uniform
//! values), a tie's shape never varies, so [`TieMaterialHandle`] is
//! created once at startup and every tie glyph just clones its `Handle`.

use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui_render::prelude::{UiMaterial, UiMaterialPlugin};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TieMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
    /// x = arc depth (fraction of the node's own height the middle dips
    /// to), y = line thickness (fraction of the node's own height), z/w
    /// unused (padding — `AsBindGroup` uniforms round up to a `vec4`
    /// alignment regardless).
    #[uniform(1)]
    pub params: Vec4,
}

impl UiMaterial for TieMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/music_score_tie.wgsl".into()
    }
}

/// The one shared tie material every tie glyph in the panel points at —
/// see the module doc comment for why a single instance is enough.
#[derive(Resource, Clone)]
pub struct TieMaterialHandle(pub Handle<TieMaterial>);

pub(super) struct TieMaterialPlugin;

impl Plugin for TieMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<TieMaterial>::default())
            .add_systems(Startup, load_tie_material);
    }
}

fn load_tie_material(mut materials: ResMut<Assets<TieMaterial>>, mut commands: Commands) {
    commands.insert_resource(TieMaterialHandle(materials.add(TieMaterial {
        color: Color::srgba(0.95, 0.80, 0.35, 0.95).into(),
        params: Vec4::new(0.85, 0.16, 0.0, 0.0),
    })));
}
