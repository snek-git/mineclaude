pub mod persistence;

use bevy::prelude::*;

use crate::inventory::chest::ChestStore;
use crate::inventory::furnace::Furnaces;
use crate::inventory::inventory::Inventory;
use crate::player::{AirSupply, ArmorSlots, Health, Hunger, Player, PlayerPitch, PlayerYaw, SpawnPoint};
use crate::world::manager::{CropTracker, SaplingTracker};

pub struct SavePlugin;

#[derive(Resource)]
struct AutoSaveTimer(Timer);

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AutoSaveTimer(Timer::from_seconds(
            60.0,
            TimerMode::Repeating,
        )))
        .add_systems(Update, (auto_save_system, manual_save_system));
    }
}

fn save_all(
    store: ResMut<crate::world::manager::ChunkDataStore>,
    player_query: &Query<
        (&Transform, &PlayerYaw, &PlayerPitch, &Health, &AirSupply, &Hunger, &ArmorSlots),
        With<Player>,
    >,
    inventory: &Res<Inventory>,
    spawn_point: &Res<SpawnPoint>,
    chest_store: &Res<ChestStore>,
    furnaces: &Res<Furnaces>,
    sapling_tracker: &Res<SaplingTracker>,
    crop_tracker: &Res<CropTracker>,
) {
    persistence::save_modified_chunks(store);
    save_player_state(player_query, inventory, spawn_point);
    if let Err(e) = persistence::save_chests(chest_store) {
        warn!("Failed to save chests: {}", e);
    }
    if let Err(e) = persistence::save_furnaces(furnaces) {
        warn!("Failed to save furnaces: {}", e);
    }
    if let Err(e) = persistence::save_saplings(sapling_tracker) {
        warn!("Failed to save saplings: {}", e);
    }
    if let Err(e) = persistence::save_crops(crop_tracker) {
        warn!("Failed to save crops: {}", e);
    }
}

fn save_player_state(
    player_query: &Query<
        (&Transform, &PlayerYaw, &PlayerPitch, &Health, &AirSupply, &Hunger, &ArmorSlots),
        With<Player>,
    >,
    inventory: &Res<Inventory>,
    spawn_point: &Res<SpawnPoint>,
) {
    if let Ok((transform, yaw, pitch, health, air, hunger, armor)) = player_query.single() {
        let pos = transform.translation;
        let sp = spawn_point.0;
        let data = persistence::PlayerSaveData {
            position: [pos.x, pos.y, pos.z],
            yaw: yaw.0,
            pitch: pitch.0,
            health: health.current,
            air_supply: air.current,
            inventory: inventory.slots.to_vec(),
            spawn_x: Some(sp.x),
            spawn_y: Some(sp.y),
            spawn_z: Some(sp.z),
            food_level: Some(hunger.food_level),
            saturation: Some(hunger.saturation),
            armor_slots: Some(armor.slots.to_vec()),
        };
        if let Err(e) = persistence::save_player(&data) {
            warn!("Failed to save player: {}", e);
        }
    }
}

fn auto_save_system(
    time: Res<Time>,
    mut timer: ResMut<AutoSaveTimer>,
    store: ResMut<crate::world::manager::ChunkDataStore>,
    player_query: Query<
        (&Transform, &PlayerYaw, &PlayerPitch, &Health, &AirSupply, &Hunger, &ArmorSlots),
        With<Player>,
    >,
    inventory: Res<Inventory>,
    spawn_point: Res<SpawnPoint>,
    chest_store: Res<ChestStore>,
    furnaces: Res<Furnaces>,
    sapling_tracker: Res<SaplingTracker>,
    crop_tracker: Res<CropTracker>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        save_all(store, &player_query, &inventory, &spawn_point, &chest_store, &furnaces, &sapling_tracker, &crop_tracker);
    }
}

fn manual_save_system(
    keys: Res<ButtonInput<KeyCode>>,
    store: ResMut<crate::world::manager::ChunkDataStore>,
    player_query: Query<
        (&Transform, &PlayerYaw, &PlayerPitch, &Health, &AirSupply, &Hunger, &ArmorSlots),
        With<Player>,
    >,
    inventory: Res<Inventory>,
    spawn_point: Res<SpawnPoint>,
    chest_store: Res<ChestStore>,
    furnaces: Res<Furnaces>,
    sapling_tracker: Res<SaplingTracker>,
    crop_tracker: Res<CropTracker>,
) {
    if keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyS) {
        save_all(store, &player_query, &inventory, &spawn_point, &chest_store, &furnaces, &sapling_tracker, &crop_tracker);
    }
}
