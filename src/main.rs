use bevy::prelude::*;

mod block;
mod world;
mod player;
mod ui;
mod lighting;
mod audio;
mod inventory;
mod save;
mod entity;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin {
            default_sampler: bevy::image::ImageSamplerDescriptor {
                mag_filter: bevy::image::ImageFilterMode::Nearest,
                min_filter: bevy::image::ImageFilterMode::Nearest,
                mipmap_filter: bevy::image::ImageFilterMode::Nearest,
                ..default()
            },
        }))
        .init_state::<GameState>()
        .add_plugins((
            world::WorldPlugin,
            player::PlayerPlugin,
            ui::UiPlugin,
            lighting::LightingPlugin,
            audio::GameAudioPlugin,
            inventory::InventoryPlugin,
            save::SavePlugin,
            entity::EntityPlugin,
        ))
        .run();
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Playing,
    Paused,
    Inventory,
}
