use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::save::persistence;
use crate::ui::inventory_screen::InventoryOpen;
use crate::ui::main_menu::InMainMenu;
use crate::world::manager::ChunkDataStore;

#[derive(Resource, Default)]
pub struct PauseState(pub bool);

#[derive(Component)]
pub(crate) struct PauseUiRoot;

#[derive(Component)]
pub(crate) enum PauseButton {
    Resume,
    Save,
    Quit,
}

const BUTTON_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 40.0;
const BUTTON_GAP: f32 = 12.0;

pub fn toggle_pause(
    keys: Res<ButtonInput<KeyCode>>,
    mut pause: ResMut<PauseState>,
    inventory_open: Res<InventoryOpen>,
    in_menu: Res<InMainMenu>,
    furnace_open: Res<crate::inventory::furnace::FurnaceOpen>,
    dead: Res<crate::ui::death_screen::PlayerDead>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    // Don't toggle pause when main menu, inventory, furnace, or death screen is open
    if in_menu.0 || inventory_open.0 || furnace_open.0.is_some() || dead.0 {
        return;
    }

    pause.0 = !pause.0;

    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };

    if pause.0 {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    } else {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

pub fn spawn_pause_ui(
    mut commands: Commands,
    pause: Res<PauseState>,
    existing: Query<Entity, With<PauseUiRoot>>,
) {
    if !pause.is_changed() || !pause.0 {
        return;
    }

    if !existing.is_empty() {
        return;
    }

    commands
        .spawn((
            PauseUiRoot,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            ZIndex(100),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Paused"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    ..default()
                },
            ));

            spawn_button(parent, "Resume", PauseButton::Resume);
            spawn_button(parent, "Save", PauseButton::Save);
            spawn_button(parent, "Quit", PauseButton::Quit);
        });
}

fn spawn_button(parent: &mut ChildSpawnerCommands, label: &str, button: PauseButton) {
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
            BorderColor::all(Color::srgba(0.6, 0.6, 0.6, 0.8)),
            BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.9)),
            Interaction::default(),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
            ));
        });
}

pub fn despawn_pause_ui(
    mut commands: Commands,
    pause: Res<PauseState>,
    query: Query<Entity, With<PauseUiRoot>>,
) {
    if !pause.is_changed() || pause.0 {
        return;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn pause_button_interaction(
    mut pause: ResMut<PauseState>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    button_q: Query<(&Interaction, &PauseButton), Changed<Interaction>>,
    store: ResMut<ChunkDataStore>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button) in &button_q {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match button {
            PauseButton::Resume => {
                pause.0 = false;
                if let Ok(mut cursor) = cursor_q.single_mut() {
                    cursor.grab_mode = CursorGrabMode::Locked;
                    cursor.visible = false;
                }
            }
            PauseButton::Save => {
                persistence::save_modified_chunks(store);
                return; // store is moved
            }
            PauseButton::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}

pub fn pause_button_hover(
    mut button_q: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PauseButton>),
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
