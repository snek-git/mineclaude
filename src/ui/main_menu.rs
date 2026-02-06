use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::entity::mob::Mob;
use crate::inventory::chest::ChestStore;
use crate::inventory::crafting::{CraftingGrid, CraftingTableGrid};
use crate::inventory::furnace::Furnaces;
use crate::inventory::inventory::Inventory;
use crate::player::{Player, SpawnPoint, Health, AirSupply, Velocity, OnGround, FallTracker, PlayerYaw, PlayerPitch, ArmorSlots, Hunger};
use crate::ui::inventory_screen::CursorItem;
use crate::world::WorldSeed;
use crate::world::generation::set_world_seed;
use crate::world::manager::{ChunkManager, ChunkDataStore, ChunkCoord, SaplingTracker, CropTracker};

#[derive(Resource)]
pub struct InMainMenu(pub bool);

impl Default for InMainMenu {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Component)]
pub(crate) struct MainMenuRoot;

#[derive(Component)]
pub(crate) enum MainMenuButton {
    NewWorld,
    LoadWorld,
    Quit,
}

const BUTTON_WIDTH: f32 = 220.0;
const BUTTON_HEIGHT: f32 = 44.0;
const BUTTON_GAP: f32 = 14.0;

pub fn setup_main_menu(
    mut commands: Commands,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if let Ok(mut cursor) = cursor_q.single_mut() {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }

    let has_saves = std::path::Path::new("saves/world").exists();

    commands
        .spawn((
            MainMenuRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(BUTTON_GAP),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.95)),
            ZIndex(200),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("MineClaude"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 52.0,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            spawn_menu_button(parent, "New World", MainMenuButton::NewWorld);

            if has_saves {
                spawn_menu_button(parent, "Load World", MainMenuButton::LoadWorld);
            }

            spawn_menu_button(parent, "Quit", MainMenuButton::Quit);
        });
}

fn spawn_menu_button(parent: &mut ChildSpawnerCommands, label: &str, button: MainMenuButton) {
    parent
        .spawn((
            button,
            Node {
                width: Val::Px(BUTTON_WIDTH),
                height: Val::Px(BUTTON_HEIGHT),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.5, 0.5, 0.6, 0.8)),
            BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
            Interaction::default(),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
            ));
        });
}

pub fn main_menu_interaction(
    mut in_menu: ResMut<InMainMenu>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    button_q: Query<(&Interaction, &MainMenuButton), Changed<Interaction>>,
    mut exit: MessageWriter<AppExit>,
    mut world_seed: ResMut<WorldSeed>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut chunk_store: ResMut<ChunkDataStore>,
    mut commands: Commands,
    despawn_entities: Query<Entity, Or<(With<ChunkCoord>, With<Mob>)>>,
    mut player_q: Query<(&mut Transform, &mut Velocity, &mut OnGround, &mut FallTracker, &mut Health, &mut AirSupply, &mut PlayerYaw, &mut PlayerPitch, &mut ArmorSlots, &mut Hunger), With<Player>>,
    mut spawn_point: ResMut<SpawnPoint>,
    mut reset_resources: (
        ResMut<Inventory>,
        ResMut<Furnaces>,
        ResMut<ChestStore>,
        ResMut<CraftingGrid>,
        ResMut<CraftingTableGrid>,
        ResMut<CursorItem>,
    ),
    mut sapling_tracker: ResMut<SaplingTracker>,
    mut crop_tracker: ResMut<CropTracker>,
) {
    for (interaction, button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button {
            MainMenuButton::NewWorld => {
                // Delete existing saves for a fresh world
                let _ = std::fs::remove_dir_all("saves");

                // Generate a random seed from system time
                let new_seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u32)
                    .unwrap_or(12345);
                world_seed.0 = new_seed;
                set_world_seed(new_seed);
                info!("[WORLD] New world with seed {}", new_seed);

                // Despawn all chunk entities and mob entities
                for entity in &despawn_entities {
                    commands.entity(entity).despawn();
                }

                // Clear chunk data
                chunk_manager.loaded.clear();
                chunk_store.chunks.clear();
                chunk_store.modified.clear();

                // Reset player inventory to default starter items
                *reset_resources.0 = Inventory::default();

                // Clear furnace, chest, crafting, and cursor state
                reset_resources.1.data.clear();
                reset_resources.2.data.clear();
                *reset_resources.3 = CraftingGrid::default();
                *reset_resources.4 = CraftingTableGrid::default();
                reset_resources.5.0 = None;

                // Clear growth trackers
                *sapling_tracker = SaplingTracker::default();
                *crop_tracker = CropTracker::default();

                // Reset player to default spawn — use actual terrain height
                let terrain_y = crate::world::generation::sample_terrain_height(0, 0);
                *spawn_point = SpawnPoint(Vec3::new(0.0, (terrain_y + 1) as f32, 0.0));
                if let Ok((mut transform, mut vel, mut on_ground, mut fall, mut health, mut air, mut yaw, mut pitch, mut armor, mut hunger)) = player_q.single_mut() {
                    transform.translation = spawn_point.0;
                    *vel = Velocity::default();
                    on_ground.0 = false;
                    fall.fall_start_y = None;
                    *health = Health::default();
                    *air = AirSupply::default();
                    yaw.0 = 0.0;
                    pitch.0 = 0.0;
                    *armor = ArmorSlots::default();
                    *hunger = Hunger::default();
                }

                in_menu.0 = false;
                if let Ok(mut cursor) = cursor_q.single_mut() {
                    cursor.grab_mode = CursorGrabMode::Locked;
                    cursor.visible = false;
                }
            }
            MainMenuButton::LoadWorld => {
                in_menu.0 = false;
                if let Ok(mut cursor) = cursor_q.single_mut() {
                    cursor.grab_mode = CursorGrabMode::Locked;
                    cursor.visible = false;
                }
            }
            MainMenuButton::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

pub fn cleanup_main_menu(
    mut commands: Commands,
    in_menu: Res<InMainMenu>,
    query: Query<Entity, With<MainMenuRoot>>,
) {
    if !in_menu.is_changed() || in_menu.0 {
        return;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn main_menu_button_hover(
    mut button_q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MainMenuButton>),
    >,
) {
    for (interaction, mut bg) in &mut button_q {
        *bg = match interaction {
            Interaction::Hovered => BackgroundColor(Color::srgba(0.3, 0.3, 0.4, 0.9)),
            Interaction::Pressed => BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
            Interaction::None => BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
        };
    }
}
