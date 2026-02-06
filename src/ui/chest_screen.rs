use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::chest::{ChestOpen, ChestStore, CHEST_SLOTS};
use crate::inventory::inventory::{Inventory, INVENTORY_COLS, INVENTORY_SLOTS};
use crate::ui::inventory_screen::CursorItem;
use super::UiAtlas;
use super::common::*;

const CHEST_COLS: usize = 9;
const CHEST_ROWS: usize = 3;

#[derive(Component)]
pub struct ChestUiRoot;

#[derive(Component)]
pub struct ChestSlot(pub(crate) usize);

#[derive(Component)]
pub struct ChestSlotCount(usize);

#[derive(Component)]
pub struct ChestInvSlot(pub(crate) usize);

#[derive(Component)]
pub struct ChestInvSlotCount(usize);

pub fn toggle_chest(
    keys: Res<ButtonInput<KeyCode>>,
    mut chest_open: ResMut<ChestOpen>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut cursor_item: ResMut<CursorItem>,
    mut inventory: ResMut<Inventory>,
) {
    if chest_open.0.is_none() {
        return;
    }

    let close = keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyE);
    if !close {
        return;
    }

    chest_open.0 = None;

    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;

    if let Some((item, count, _)) = cursor_item.0.take() {
        for _ in 0..count {
            inventory.add_item(item);
        }
    }
}

pub fn spawn_chest_ui(
    mut commands: Commands,
    chest_open: Res<ChestOpen>,
    chest_store: Res<ChestStore>,
    inventory: Res<Inventory>,
    atlas: Res<UiAtlas>,
    existing: Query<Entity, With<ChestUiRoot>>,
) {
    if !chest_open.is_changed() {
        return;
    }

    let Some(pos) = chest_open.0 else {
        return;
    };

    if !existing.is_empty() {
        return;
    }

    let chest_data = chest_store.data.get(&pos);
    let rows = INVENTORY_SLOTS / INVENTORY_COLS;

    commands
        .spawn((
            ChestUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ZIndex(50),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Chest"),
                TextColor(Color::WHITE),
                TextFont { font_size: 20.0, ..default() },
                Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
            ));

            // Chest slots: 3 rows x 9 cols
            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(SLOT_GAP),
                    ..default()
                })
                .with_children(|grid| {
                    for row in 0..CHEST_ROWS {
                        grid.spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(SLOT_GAP),
                            ..default()
                        })
                        .with_children(|row_node| {
                            for col in 0..CHEST_COLS {
                                let slot_idx = row * CHEST_COLS + col;
                                let data = chest_data
                                    .and_then(|d| d.slots[slot_idx]);

                                row_node
                                    .spawn((
                                        Node {
                                            width: Val::Px(SLOT_SIZE),
                                            height: Val::Px(SLOT_SIZE),
                                            border: UiRect::all(Val::Px(BORDER_WIDTH)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BorderColor::all(Color::srgba(0.6, 0.45, 0.2, 0.7)),
                                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
                                        Interaction::default(),
                                    ))
                                    .with_children(|sp| {
                                        sp.spawn((
                                            ChestSlot(slot_idx),
                                            Node {
                                                width: Val::Px(INNER_SIZE),
                                                height: Val::Px(INNER_SIZE),
                                                justify_content: JustifyContent::End,
                                                align_items: AlignItems::End,
                                                ..default()
                                            },
                                            slot_image(&atlas, &data),
                                            slot_bg(&data),
                                        ))
                                        .with_children(|bp| {
                                            bp.spawn((
                                                ChestSlotCount(slot_idx),
                                                Text::new(count_text(&data)),
                                                TextColor(Color::WHITE),
                                                TextFont { font_size: 11.0, ..default() },
                                            ));
                                        });
                                    });
                            }
                        });
                    }
                });

            // Separator
            parent.spawn((
                Text::new("Inventory"),
                TextColor(Color::WHITE),
                TextFont { font_size: 16.0, ..default() },
                Node { margin: UiRect::vertical(Val::Px(4.0)), ..default() },
            ));

            // Player inventory: 4 rows x 9 cols
            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(SLOT_GAP),
                    ..default()
                })
                .with_children(|grid| {
                    for row in 0..rows {
                        grid.spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(SLOT_GAP),
                            ..default()
                        })
                        .with_children(|row_node| {
                            for col in 0..INVENTORY_COLS {
                                let slot_idx = row * INVENTORY_COLS + col;
                                let data = inventory.slots[slot_idx];

                                let is_hotbar_row = row == rows - 1;
                                let border_color = if is_hotbar_row {
                                    Color::srgba(0.8, 0.8, 0.3, 0.7)
                                } else {
                                    Color::srgba(0.4, 0.4, 0.4, 0.5)
                                };

                                row_node
                                    .spawn((
                                        Node {
                                            width: Val::Px(SLOT_SIZE),
                                            height: Val::Px(SLOT_SIZE),
                                            border: UiRect::all(Val::Px(BORDER_WIDTH)),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BorderColor::all(border_color),
                                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
                                        Interaction::default(),
                                    ))
                                    .with_children(|sp| {
                                        sp.spawn((
                                            ChestInvSlot(slot_idx),
                                            Node {
                                                width: Val::Px(INNER_SIZE),
                                                height: Val::Px(INNER_SIZE),
                                                justify_content: JustifyContent::End,
                                                align_items: AlignItems::End,
                                                ..default()
                                            },
                                            slot_image(&atlas, &data),
                                            slot_bg(&data),
                                        ))
                                        .with_children(|bp| {
                                            bp.spawn((
                                                ChestInvSlotCount(slot_idx),
                                                Text::new(count_text(&data)),
                                                TextColor(Color::WHITE),
                                                TextFont { font_size: 11.0, ..default() },
                                            ));
                                        });
                                    });
                            }
                        });
                    }
                });
        });
}

pub fn despawn_chest_ui(
    mut commands: Commands,
    chest_open: Res<ChestOpen>,
    query: Query<Entity, With<ChestUiRoot>>,
) {
    if !chest_open.is_changed() {
        return;
    }

    if chest_open.0.is_some() {
        return;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handle clicking on chest storage slots.
pub fn chest_slot_interaction(
    mut chest_store: ResMut<ChestStore>,
    chest_open: Res<ChestOpen>,
    mut cursor_item: ResMut<CursorItem>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    slot_q: Query<(&ChestSlot, &ChildOf)>,
    interaction_q: Query<(&Interaction, &Children)>,
) {
    let Some(pos) = chest_open.0 else { return };

    for (interaction, children) in &interaction_q {
        if *interaction == Interaction::Pressed {
            for child in children.iter() {
                if let Ok((slot, _)) = slot_q.get(child) {
                    let slot_idx = slot.0;
                    if slot_idx >= CHEST_SLOTS {
                        continue;
                    }
                    let data = chest_store.data.entry(pos).or_default();
                    swap_slot(&mut data.slots[slot_idx], &mut cursor_item.0);
                    return;
                }
            }
        }
        if *interaction == Interaction::Hovered && mouse_buttons.just_pressed(MouseButton::Right) {
            for child in children.iter() {
                if let Ok((slot, _)) = slot_q.get(child) {
                    let slot_idx = slot.0;
                    if slot_idx >= CHEST_SLOTS {
                        continue;
                    }
                    let data = chest_store.data.entry(pos).or_default();
                    swap_slot_right_click(&mut data.slots[slot_idx], &mut cursor_item.0);
                    return;
                }
            }
        }
    }
}

/// Handle clicking on player inventory slots in the chest screen.
pub fn chest_inv_slot_interaction(
    mut inventory: ResMut<Inventory>,
    mut cursor_item: ResMut<CursorItem>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    slot_q: Query<(&ChestInvSlot, &ChildOf)>,
    interaction_q: Query<(&Interaction, &Children)>,
) {
    for (interaction, children) in &interaction_q {
        if *interaction == Interaction::Pressed {
            for child in children.iter() {
                if let Ok((slot, _)) = slot_q.get(child) {
                    swap_slot(&mut inventory.slots[slot.0], &mut cursor_item.0);
                    return;
                }
            }
        }
        if *interaction == Interaction::Hovered && mouse_buttons.just_pressed(MouseButton::Right) {
            for child in children.iter() {
                if let Ok((slot, _)) = slot_q.get(child) {
                    swap_slot_right_click(&mut inventory.slots[slot.0], &mut cursor_item.0);
                    return;
                }
            }
        }
    }
}

/// Update chest UI visuals when data changes.
pub fn update_chest_ui(
    chest_store: Res<ChestStore>,
    chest_open: Res<ChestOpen>,
    inventory: Res<Inventory>,
    atlas: Res<UiAtlas>,
    mut chest_slots: Query<(&ChestSlot, &mut ImageNode, &mut BackgroundColor), Without<ChestInvSlot>>,
    mut chest_counts: Query<(&ChestSlotCount, &mut Text), Without<ChestInvSlotCount>>,
    mut inv_slots: Query<(&ChestInvSlot, &mut ImageNode, &mut BackgroundColor), Without<ChestSlot>>,
    mut inv_counts: Query<(&ChestInvSlotCount, &mut Text), Without<ChestSlotCount>>,
) {
    let Some(pos) = chest_open.0 else { return };

    let chest_changed = chest_store.is_changed();
    let inv_changed = inventory.is_changed();

    if !chest_changed && !inv_changed {
        return;
    }

    if chest_changed {
        let chest_data = chest_store.data.get(&pos);
        for (slot, mut img, mut bg) in &mut chest_slots {
            let data = chest_data.and_then(|d| d.slots[slot.0]);
            update_slot_visual(&atlas, &data, &mut img, &mut bg);
        }
        for (slot, mut text) in &mut chest_counts {
            let data = chest_store.data.get(&pos).and_then(|d| d.slots[slot.0]);
            **text = count_text(&data);
        }
    }

    if inv_changed {
        for (slot, mut img, mut bg) in &mut inv_slots {
            let data = inventory.slots[slot.0];
            update_slot_visual(&atlas, &data, &mut img, &mut bg);
        }
        for (slot, mut text) in &mut inv_counts {
            **text = count_text(&inventory.slots[slot.0]);
        }
    }
}
