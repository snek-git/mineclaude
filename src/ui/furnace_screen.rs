use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::furnace::{FurnaceOpen, Furnaces};
use crate::inventory::inventory::{Inventory, INVENTORY_COLS, INVENTORY_SLOTS};
use crate::inventory::item::Item;
use crate::ui::inventory_screen::CursorItem;
use super::UiAtlas;
use super::common::*;

const FURNACE_SLOT_SIZE: f32 = 48.0;
const FURNACE_INNER_SIZE: f32 = FURNACE_SLOT_SIZE - BLOCK_INSET * 2.0 - BORDER_WIDTH * 2.0;

#[derive(Component)]
pub struct FurnaceUiRoot;

#[derive(Component)]
pub struct FurnaceInputSlot;

#[derive(Component)]
pub struct FurnaceFuelSlot;

#[derive(Component)]
pub struct FurnaceOutputSlot;

#[derive(Component)]
pub struct FurnaceInputCount;

#[derive(Component)]
pub struct FurnaceFuelCount;

#[derive(Component)]
pub struct FurnaceOutputCount;

#[derive(Component)]
pub struct FurnaceProgressBar;

#[derive(Component)]
pub struct FurnaceFuelBar;

#[derive(Component)]
pub struct FurnaceInvSlot(usize);

#[derive(Component)]
pub struct FurnaceInvSlotCount(usize);

pub fn toggle_furnace(
    keys: Res<ButtonInput<KeyCode>>,
    mut furnace_open: ResMut<FurnaceOpen>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut cursor_item: ResMut<CursorItem>,
    mut inventory: ResMut<Inventory>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    if furnace_open.0.is_none() {
        return;
    }

    // Close furnace
    furnace_open.0 = None;

    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;

    // Return cursor item to inventory
    if let Some((item, count, _)) = cursor_item.0.take() {
        for _ in 0..count {
            inventory.add_item(item);
        }
    }
}

pub fn spawn_furnace_ui(
    mut commands: Commands,
    furnace_open: Res<FurnaceOpen>,
    furnaces: Res<Furnaces>,
    inventory: Res<Inventory>,
    atlas: Res<UiAtlas>,
    existing: Query<Entity, With<FurnaceUiRoot>>,
) {
    if !furnace_open.is_changed() {
        return;
    }

    let Some(pos) = furnace_open.0 else {
        return;
    };

    if !existing.is_empty() {
        return;
    }

    let data = furnaces.data.get(&pos);

    let input_data = data.and_then(|d| d.input);
    let fuel_data = data.and_then(|d| d.fuel);
    let output_data = data.and_then(|d| d.output);
    let progress = data.map(|d| d.progress).unwrap_or(0.0);
    let fuel_frac = data
        .map(|d| {
            if d.fuel_max > 0.0 {
                d.fuel_remaining / d.fuel_max
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);

    let rows = INVENTORY_SLOTS / INVENTORY_COLS;

    commands
        .spawn((
            FurnaceUiRoot,
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
                Text::new("Furnace"),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Main furnace layout: input slot on left, arrow+fuel in center, output on right
            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|row| {
                    // Left column: input on top, fuel on bottom
                    row.spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|col| {
                        // Input slot
                        spawn_furnace_slot(col, input_data, FurnaceSlotKind::Input, &atlas);
                        // Fuel slot
                        spawn_furnace_slot(col, fuel_data, FurnaceSlotKind::Fuel, &atlas);
                    });

                    // Center: progress arrow and fuel indicator
                    row.spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|center| {
                        // Progress bar (horizontal)
                        center
                            .spawn((
                                Node {
                                    width: Val::Px(40.0),
                                    height: Val::Px(16.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
                            ))
                            .with_children(|bar_parent| {
                                bar_parent.spawn((
                                    FurnaceProgressBar,
                                    Node {
                                        width: Val::Percent(progress * 100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.8, 0.6, 0.1)),
                                ));
                            });

                        // Arrow text
                        center.spawn((
                            Text::new("=>"),
                            TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                        ));

                        // Fuel indicator bar
                        center
                            .spawn((
                                Node {
                                    width: Val::Px(16.0),
                                    height: Val::Px(40.0),
                                    flex_direction: FlexDirection::ColumnReverse,
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
                            ))
                            .with_children(|bar_parent| {
                                bar_parent.spawn((
                                    FurnaceFuelBar,
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(fuel_frac * 100.0),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.9, 0.4, 0.1)),
                                ));
                            });
                    });

                    // Output slot (larger)
                    row.spawn((
                        Node {
                            width: Val::Px(FURNACE_SLOT_SIZE + 12.0),
                            height: Val::Px(FURNACE_SLOT_SIZE + 12.0),
                            border: UiRect::all(Val::Px(BORDER_WIDTH + 1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor::all(Color::srgba(0.8, 0.6, 0.2, 0.8)),
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
                        Interaction::default(),
                    ))
                    .with_children(|slot_parent| {
                        slot_parent
                            .spawn((
                                FurnaceOutputSlot,
                                Node {
                                    width: Val::Px(FURNACE_SLOT_SIZE - BLOCK_INSET),
                                    height: Val::Px(FURNACE_SLOT_SIZE - BLOCK_INSET),
                                    justify_content: JustifyContent::End,
                                    align_items: AlignItems::End,
                                    ..default()
                                },
                                slot_image(&atlas, &output_data),
                                slot_bg(&output_data),
                            ))
                            .with_children(|bp| {
                                bp.spawn((
                                    FurnaceOutputCount,
                                    Text::new(count_text(&output_data)),
                                    TextColor(Color::WHITE),
                                    TextFont {
                                        font_size: 11.0,
                                        ..default()
                                    },
                                ));
                            });
                    });
                });

            // Separator
            parent.spawn((
                Text::new("Inventory"),
                TextColor(Color::WHITE),
                TextFont { font_size: 16.0, ..default() },
                Node { margin: UiRect::vertical(Val::Px(4.0)), ..default() },
            ));

            // Player inventory grid
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
                                            FurnaceInvSlot(slot_idx),
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
                                                FurnaceInvSlotCount(slot_idx),
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

enum FurnaceSlotKind {
    Input,
    Fuel,
}

fn spawn_furnace_slot(
    parent: &mut ChildSpawnerCommands,
    data: Option<(Item, u8, u16)>,
    kind: FurnaceSlotKind,
    atlas: &UiAtlas,
) {
    let border_color = match kind {
        FurnaceSlotKind::Input => Color::srgba(0.6, 0.4, 0.2, 0.7),
        FurnaceSlotKind::Fuel => Color::srgba(0.9, 0.4, 0.1, 0.7),
    };

    parent
        .spawn((
            Node {
                width: Val::Px(FURNACE_SLOT_SIZE),
                height: Val::Px(FURNACE_SLOT_SIZE),
                border: UiRect::all(Val::Px(BORDER_WIDTH)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor::all(border_color),
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
            Interaction::default(),
        ))
        .with_children(|slot_parent| {
            let (marker_input, marker_fuel): (Option<FurnaceInputSlot>, Option<FurnaceFuelSlot>) =
                match kind {
                    FurnaceSlotKind::Input => (Some(FurnaceInputSlot), None),
                    FurnaceSlotKind::Fuel => (None, Some(FurnaceFuelSlot)),
                };

            let ct = count_text(&data);

            let mut slot_cmd = slot_parent.spawn((
                Node {
                    width: Val::Px(FURNACE_INNER_SIZE),
                    height: Val::Px(FURNACE_INNER_SIZE),
                    justify_content: JustifyContent::End,
                    align_items: AlignItems::End,
                    ..default()
                },
                slot_image(atlas, &data),
                slot_bg(&data),
            ));

            if let Some(m) = marker_input {
                slot_cmd.insert(m);
            }
            if let Some(m) = marker_fuel {
                slot_cmd.insert(m);
            }

            slot_cmd.with_children(|bp| {
                let mut count_cmd = bp.spawn((
                    Text::new(ct),
                    TextColor(Color::WHITE),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                ));
                match kind {
                    FurnaceSlotKind::Input => {
                        count_cmd.insert(FurnaceInputCount);
                    }
                    FurnaceSlotKind::Fuel => {
                        count_cmd.insert(FurnaceFuelCount);
                    }
                }
            });
        });
}

pub fn despawn_furnace_ui(
    mut commands: Commands,
    furnace_open: Res<FurnaceOpen>,
    query: Query<Entity, With<FurnaceUiRoot>>,
) {
    if !furnace_open.is_changed() {
        return;
    }

    if furnace_open.0.is_some() {
        return;
    }

    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handle clicking on furnace input slot.
pub fn furnace_input_interaction(
    mut furnaces: ResMut<Furnaces>,
    furnace_open: Res<FurnaceOpen>,
    mut cursor_item: ResMut<CursorItem>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    input_q: Query<&FurnaceInputSlot>,
    interaction_q: Query<(&Interaction, &Children)>,
) {
    let Some(pos) = furnace_open.0 else { return };

    for (interaction, children) in &interaction_q {
        if *interaction == Interaction::Pressed {
            for child in children.iter() {
                if input_q.get(child).is_ok() {
                    let data = furnaces.data.entry(pos).or_default();
                    swap_slot(&mut data.input, &mut cursor_item.0);
                    return;
                }
            }
        }
        if *interaction == Interaction::Hovered && mouse_buttons.just_pressed(MouseButton::Right) {
            for child in children.iter() {
                if input_q.get(child).is_ok() {
                    let data = furnaces.data.entry(pos).or_default();
                    swap_slot_right_click(&mut data.input, &mut cursor_item.0);
                    return;
                }
            }
        }
    }
}

/// Handle clicking on furnace fuel slot.
pub fn furnace_fuel_interaction(
    mut furnaces: ResMut<Furnaces>,
    furnace_open: Res<FurnaceOpen>,
    mut cursor_item: ResMut<CursorItem>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    fuel_q: Query<&FurnaceFuelSlot>,
    interaction_q: Query<(&Interaction, &Children)>,
) {
    let Some(pos) = furnace_open.0 else { return };

    for (interaction, children) in &interaction_q {
        if *interaction == Interaction::Pressed {
            for child in children.iter() {
                if fuel_q.get(child).is_ok() {
                    let data = furnaces.data.entry(pos).or_default();
                    swap_slot(&mut data.fuel, &mut cursor_item.0);
                    return;
                }
            }
        }
        if *interaction == Interaction::Hovered && mouse_buttons.just_pressed(MouseButton::Right) {
            for child in children.iter() {
                if fuel_q.get(child).is_ok() {
                    let data = furnaces.data.entry(pos).or_default();
                    swap_slot_right_click(&mut data.fuel, &mut cursor_item.0);
                    return;
                }
            }
        }
    }
}

/// Handle clicking on furnace output slot (take only).
pub fn furnace_output_interaction(
    mut furnaces: ResMut<Furnaces>,
    furnace_open: Res<FurnaceOpen>,
    mut cursor_item: ResMut<CursorItem>,
    output_q: Query<&FurnaceOutputSlot>,
    interaction_q: Query<(&Interaction, &Children), Changed<Interaction>>,
) {
    let Some(pos) = furnace_open.0 else { return };

    for (interaction, children) in &interaction_q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for child in children.iter() {
            if output_q.get(child).is_ok() {
                let data = furnaces.data.entry(pos).or_default();
                let Some(output) = data.output.take() else {
                    return;
                };

                if let Some(ref held) = cursor_item.0 {
                    if held.0 != output.0 {
                        // Can't mix, put it back
                        data.output = Some(output);
                        return;
                    }
                    let max = held.0.max_stack() as u16;
                    if held.1 as u16 + output.1 as u16 > max {
                        data.output = Some(output);
                        return;
                    }
                }

                if let Some(ref mut held) = cursor_item.0 {
                    held.1 += output.1;
                } else {
                    cursor_item.0 = Some(output);
                }
                return;
            }
        }
    }
}

/// Handle clicking on player inventory slots in the furnace screen.
pub fn furnace_inv_slot_interaction(
    mut inventory: ResMut<Inventory>,
    mut cursor_item: ResMut<CursorItem>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    slot_q: Query<(&FurnaceInvSlot, &ChildOf)>,
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

/// Update furnace UI visuals from furnace data.
pub fn update_furnace_ui(
    furnaces: Res<Furnaces>,
    furnace_open: Res<FurnaceOpen>,
    inventory: Res<Inventory>,
    atlas: Res<UiAtlas>,
    mut input_slots: Query<(&mut ImageNode, &mut BackgroundColor), (With<FurnaceInputSlot>, Without<FurnaceFuelSlot>, Without<FurnaceOutputSlot>, Without<FurnaceInvSlot>)>,
    mut input_counts: Query<&mut Text, (With<FurnaceInputCount>, Without<FurnaceFuelCount>, Without<FurnaceOutputCount>, Without<FurnaceInvSlotCount>)>,
    mut fuel_slots: Query<(&mut ImageNode, &mut BackgroundColor), (With<FurnaceFuelSlot>, Without<FurnaceInputSlot>, Without<FurnaceOutputSlot>, Without<FurnaceInvSlot>)>,
    mut fuel_counts: Query<&mut Text, (With<FurnaceFuelCount>, Without<FurnaceInputCount>, Without<FurnaceOutputCount>, Without<FurnaceInvSlotCount>)>,
    mut output_slots: Query<(&mut ImageNode, &mut BackgroundColor), (With<FurnaceOutputSlot>, Without<FurnaceInputSlot>, Without<FurnaceFuelSlot>, Without<FurnaceInvSlot>)>,
    mut output_counts: Query<&mut Text, (With<FurnaceOutputCount>, Without<FurnaceInputCount>, Without<FurnaceFuelCount>, Without<FurnaceInvSlotCount>)>,
    mut progress_bars: Query<&mut Node, (With<FurnaceProgressBar>, Without<FurnaceFuelBar>)>,
    mut fuel_bars: Query<&mut Node, (With<FurnaceFuelBar>, Without<FurnaceProgressBar>)>,
    mut inv_slots: Query<(&FurnaceInvSlot, &mut ImageNode, &mut BackgroundColor), (Without<FurnaceInputSlot>, Without<FurnaceFuelSlot>, Without<FurnaceOutputSlot>)>,
    mut inv_counts: Query<(&FurnaceInvSlotCount, &mut Text), (Without<FurnaceInputCount>, Without<FurnaceFuelCount>, Without<FurnaceOutputCount>)>,
) {
    let Some(pos) = furnace_open.0 else { return };
    let Some(data) = furnaces.data.get(&pos) else { return };

    // Input slot
    if let Ok((mut img, mut bg)) = input_slots.single_mut() {
        update_slot_visual(&atlas, &data.input, &mut img, &mut bg);
    }
    if let Ok(mut text) = input_counts.single_mut() {
        **text = count_text(&data.input);
    }

    // Fuel slot
    if let Ok((mut img, mut bg)) = fuel_slots.single_mut() {
        update_slot_visual(&atlas, &data.fuel, &mut img, &mut bg);
    }
    if let Ok(mut text) = fuel_counts.single_mut() {
        **text = count_text(&data.fuel);
    }

    // Output slot
    if let Ok((mut img, mut bg)) = output_slots.single_mut() {
        update_slot_visual(&atlas, &data.output, &mut img, &mut bg);
    }
    if let Ok(mut text) = output_counts.single_mut() {
        **text = count_text(&data.output);
    }

    // Progress bar
    if let Ok(mut node) = progress_bars.single_mut() {
        node.width = Val::Percent(data.progress * 100.0);
    }

    // Fuel bar
    if let Ok(mut node) = fuel_bars.single_mut() {
        let frac = if data.fuel_max > 0.0 {
            data.fuel_remaining / data.fuel_max
        } else {
            0.0
        };
        node.height = Val::Percent(frac.clamp(0.0, 1.0) * 100.0);
    }

    // Player inventory slots
    if inventory.is_changed() {
        for (slot, mut img, mut bg) in &mut inv_slots {
            let slot_data = inventory.slots[slot.0];
            update_slot_visual(&atlas, &slot_data, &mut img, &mut bg);
        }
        for (slot, mut text) in &mut inv_counts {
            **text = count_text(&inventory.slots[slot.0]);
        }
    }
}
