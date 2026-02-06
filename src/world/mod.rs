pub mod chunk;
pub mod coordinates;
pub mod generation;
pub mod manager;
pub mod material;
pub mod meshing;

use bevy::prelude::*;
use bevy::pbr::MaterialPlugin;

/// Resource holding the world seed. Changing this and clearing chunks triggers a new world.
#[derive(Resource)]
pub struct WorldSeed(pub u32);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(42)
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<material::ChunkMaterialType>::default())
            .init_resource::<WorldSeed>()
            .add_systems(Startup, manager::setup_world)
            .add_systems(
                Update,
                (
                    manager::update_chunk_loading,
                    manager::start_mesh_tasks.after(manager::update_chunk_loading),
                    manager::apply_mesh_results.after(manager::start_mesh_tasks),
                    manager::update_sapling_growth,
                    manager::update_crop_growth,
                ),
            );
    }
}
