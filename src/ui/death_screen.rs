use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::chest::ChestOpen;
use crate::inventory::crafting::{CraftingGrid, CraftingTableGrid, CraftingTableOpen};
use crate::inventory::furnace::FurnaceOpen;
use crate::player::{Player, Health, Velocity, AirSupply, Hunger, SpawnPoint, FallTracker, ArmorSlots};
use crate::ui::inventory_screen::{CursorItem, InventoryOpen};

/// Resource tracking whether the player is currently dead.
#[derive(Resource, Default)]
pub struct PlayerDead(pub bool);

#[derive(Component)]
pub(crate) struct DeathScreenRoot;

#[derive(Component)]
pub(crate) struct RespawnButton;

const BUTTON_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 44.0;

/// When health reaches 0, mark player as dead and show cursor.
pub fn detect_death(
    mut dead: ResMut<PlayerDead>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    query: Query<&Health, With<Player>>,
) {
    if dead.0 {
        return;
    }

    for health in &query {
        if health.current <= 0.0 {
            dead.0 = true;
            if let Ok(mut cursor) = cursor_q.single_mut() {
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            }
        }
    }
}

/// Spawn the death screen overlay when PlayerDead becomes true.
pub fn spawn_death_screen(
    mut commands: Commands,
    dead: Res<PlayerDead>,
    existing: Query<Entity, With<DeathScreenRoot>>,
) {
    if !dead.is_changed() || !dead.0 {
        return;
    }

    if !existing.is_empty() {
        return;
    }

    commands
        .spawn((
            DeathScreenRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.5, 0.0, 0.0, 0.6)),
            ZIndex(150),
        ))
        .with_children(|parent| {
            // "You Died!" title
            parent.spawn((
                Text::new("You Died!"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(16.0)),
                    ..default()
                },
            ));

            // Respawn button
            spawn_respawn_button(parent);
        });
}

fn spawn_respawn_button(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            RespawnButton,
            Node {
                width: Val::Px(BUTTON_WIDTH),
                height: Val::Px(BUTTON_HEIGHT),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.6, 0.6, 0.6, 0.8)),
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            Interaction::default(),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("Respawn"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
            ));
        });
}

/// Despawn the death screen when PlayerDead becomes false.
pub fn despawn_death_screen(
    mut commands: Commands,
    dead: Res<PlayerDead>,
    query: Query<Entity, With<DeathScreenRoot>>,
) {
    if !dead.is_changed() || dead.0 {
        return;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handle the respawn button click.
pub fn respawn_button_interaction(
    mut dead: ResMut<PlayerDead>,
    spawn: Res<SpawnPoint>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    button_q: Query<(&Interaction, &RespawnButton), Changed<Interaction>>,
    mut player_q: Query<(&mut Transform, &mut Health, &mut Velocity, &mut AirSupply, &mut Hunger, &mut FallTracker, &mut ArmorSlots), With<Player>>,
    mut inventory_open: ResMut<InventoryOpen>,
    mut furnace_open: ResMut<FurnaceOpen>,
    mut ct_open: ResMut<CraftingTableOpen>,
    mut chest_open: ResMut<ChestOpen>,
    mut cursor_item: ResMut<CursorItem>,
    mut crafting_grid: ResMut<CraftingGrid>,
    mut crafting_table_grid: ResMut<CraftingTableGrid>,
) {
    for (interaction, _) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        // Reset player state
        if let Ok((mut tf, mut health, mut vel, mut air, mut hunger, mut fall, mut armor)) = player_q.single_mut() {
            tf.translation = spawn.0;
            health.current = health.max;
            vel.0 = Vec3::ZERO;
            air.current = air.max;
            *hunger = Hunger::default();
            fall.fall_start_y = None;
            *armor = ArmorSlots::default();
        }

        // Close all open UI screens to prevent soft-lock
        inventory_open.0 = false;
        furnace_open.0 = None;
        ct_open.0 = false;
        chest_open.0 = None;
        cursor_item.0 = None;
        *crafting_grid = CraftingGrid::default();
        *crafting_table_grid = CraftingTableGrid::default();

        dead.0 = false;

        if let Ok(mut cursor) = cursor_q.single_mut() {
            cursor.grab_mode = CursorGrabMode::Locked;
            cursor.visible = false;
        }
    }
}

/// Button hover effect for respawn button.
pub fn respawn_button_hover(
    mut button_q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<RespawnButton>),
    >,
) {
    for (interaction, mut bg) in &mut button_q {
        *bg = match interaction {
            Interaction::Hovered => BackgroundColor(Color::srgba(0.35, 0.35, 0.35, 0.9)),
            Interaction::Pressed => BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
            Interaction::None => BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
        };
    }
}
