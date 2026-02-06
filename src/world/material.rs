use bevy::prelude::*;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// Material extension that tiles atlas textures correctly for greedy-meshed quads.
/// Uses UV_1 to pass the tile origin so the fragment shader can wrap UVs within
/// the atlas tile boundaries.
#[derive(Asset, AsBindGroup, TypePath, Clone, Default)]
pub struct AtlasTileMaterial {}

impl MaterialExtension for AtlasTileMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/atlas_tile.wgsl".into()
    }
}

/// Type alias for the chunk material: StandardMaterial extended with atlas tiling.
pub type ChunkMaterialType = ExtendedMaterial<StandardMaterial, AtlasTileMaterial>;
