use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::crafting::{self, CraftingTableGrid, CraftingTableOpen, CRAFTING_TABLE_SIZE};
use crate::inventory::inventory::{Inventory, INVENTORY_COLS, INVENTORY_SLOTS};
use crate::ui::inventory_screen::CursorItem;
use super::UiAtlas;
use super::common::*;

#[derive(Component)]
pub struct CraftingTableUiRoot;

#[derive(Component)]
pub struct CraftingTableSlot(pub(crate) usize);

#[derive(Component)]
pub struct CraftingTableSlotCount(usize);

#[derive(Component)]
pub struct CraftingTableOutputSlot;

#[derive(Component)]
pub struct CraftingTableOutputCount;

#[derive(Component)]
pub struct CraftingTableInvSlot(pub(crate) usize);

#[derive(Component)]
pub struct CraftingTableInvSlotCount(usize);

pub fn toggle_crafting_table(
    keys: Res<ButtonInput<KeyCode>>,
    mut ct_open: ResMut<CraftingTableOpen>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut ct_grid: ResMut<CraftingTableGrid>,
    mut inventory: ResMut<Inventory>,
    mut cursor_item: ResMut<CursorItem>,
    furnace_open: Res<crate::inventory::furnace::FurnaceOpen>,
) {
    if furnace_open.0.is_some() {
        return;
    }

    let toggle = (ct_open.0 && keys.just_pressed(KeyCode::Escape))
        || (ct_open.0 && keys.just_pressed(KeyCode::KeyE));

    if !toggle {
        return;
    }

    info!("[CRAFT] toggle_crafting_table: closing crafting table");
    ct_open.0 = false;

    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;

    crafting::clear_crafting_table_grid(&mut ct_grid, &mut inventory);
    if let Some((item, count, _)) = cursor_item.0.take() {
        info!(
            "[CRAFT] toggle_crafting_table: returning cursor item {}x{} to inventory",
            item.display_name(),
            count,
        );
        for _ in 0..count {
            inventory.add_item(item);
        }
    }
}

pub fn spawn_crafting_table_ui(
    mut commands: Commands,
    ct_open: Res<CraftingTableOpen>,
    ct_grid: Res<CraftingTableGrid>,
    inventory: Res<Inventory>,
    atlas: Res<UiAtlas>,
    existing: Query<Entity, With<CraftingTableUiRoot>>,
) {
    if !ct_open.is_changed() || !ct_open.0 || !existing.is_empty() {
        return;
    }

    let rows = INVENTORY_SLOTS / INVENTORY_COLS;

    commands
        .spawn((
            CraftingTableUiRoot,
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
                Text::new("Crafting Table"),
                TextColor(Color::WHITE),
                TextFont { font_size: 20.0, ..default() },
                Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
            ));

            // Crafting area: 3x3 grid + arrow + output
            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|crafting_row| {
                    // 3x3 crafting grid
                    crafting_row
                        .spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(SLOT_GAP),
                            ..default()
                        })
                        .with_children(|grid| {
                            for row in 0..CRAFTING_TABLE_SIZE {
                                grid.spawn(Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(SLOT_GAP),
                                    ..default()
                                })
                                .with_children(|row_node| {
                                    for col in 0..CRAFTING_TABLE_SIZE {
                                        let slot_idx = row * CRAFTING_TABLE_SIZE + col;
                                        let data = ct_grid.slots[slot_idx];
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
                                                BorderColor::all(Color::srgba(0.6, 0.4, 0.2, 0.7)),
                                                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
                                                Interaction::default(),
                                            ))
                                            .with_children(|sp| {
                                                sp.spawn((
                                                    CraftingTableSlot(slot_idx),
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
                                                        CraftingTableSlotCount(slot_idx),
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

                    // Arrow
                    crafting_row.spawn((
                        Text::new("=>"),
                        TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                        TextFont { font_size: 20.0, ..default() },
                        Node { margin: UiRect::horizontal(Val::Px(8.0)), ..default() },
                    ));

                    // Output slot
                    let output = ct_grid.output;
                    crafting_row
                        .spawn((
                            Node {
                                width: Val::Px(SLOT_SIZE + 8.0),
                                height: Val::Px(SLOT_SIZE + 8.0),
                                border: UiRect::all(Val::Px(BORDER_WIDTH + 1.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BorderColor::all(Color::srgba(0.8, 0.6, 0.2, 0.8)),
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
                            Interaction::default(),
                        ))
                        .with_children(|sp| {
                            sp.spawn((
                                CraftingTableOutputSlot,
                                Node {
                                    width: Val::Px(SLOT_SIZE - BLOCK_INSET),
                                    height: Val::Px(SLOT_SIZE - BLOCK_INSET),
                                    justify_content: JustifyContent::End,
                                    align_items: AlignItems::End,
                                    ..default()
                                },
                                slot_image(&atlas, &output),
                                slot_bg(&output),
                            ))
                            .with_children(|bp| {
                                bp.spawn((
                                    CraftingTableOutputCount,
                                    Text::new(count_text(&output)),
                                    TextColor(Color::WHITE),
                                    TextFont { font_size: 11.0, ..default() },
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
                                            CraftingTableInvSlot(slot_idx),
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
                                                CraftingTableInvSlotCount(slot_idx),
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

pub fn despawn_crafting_table_ui(
    mut commands: Commands,
    ct_open: Res<CraftingTableOpen>,
    query: Query<Entity, With<CraftingTableUiRoot>>,
) {
    if !ct_open.is_changed() || ct_open.0 {
        return;
    }
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handle clicking on crafting table input slots.
pub fn crafting_table_slot_interaction(
    mut ct_grid: ResMut<CraftingTableGrid>,
    mut cursor_item: ResMut<CursorItem>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    slot_q: Query<(&CraftingTableSlot, &ChildOf)>,
    interaction_q: Query<(&Interaction, &Children)>,
) {
    for (interaction, children) in &interaction_q {
        if *interaction == Interaction::Pressed {
            for child in children.iter() {
                if let Ok((slot, _)) = slot_q.get(child) {
                    swap_slot(&mut ct_grid.slots[slot.0], &mut cursor_item.0);
                    return;
                }
            }
        }
        if *interaction == Interaction::Hovered && mouse_buttons.just_pressed(MouseButton::Right) {
            for child in children.iter() {
                if let Ok((slot, _)) = slot_q.get(child) {
                    swap_slot_right_click(&mut ct_grid.slots[slot.0], &mut cursor_item.0);
                    return;
                }
            }
        }
    }
}

/// Handle clicking the crafting table output slot.
pub fn crafting_table_output_interaction(
    mut ct_grid: ResMut<CraftingTableGrid>,
    mut cursor_item: ResMut<CursorItem>,
    output_slot_q: Query<&CraftingTableOutputSlot>,
    interaction_q: Query<(&Interaction, &Children), Changed<Interaction>>,
) {
    for (interaction, children) in &interaction_q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for child in children.iter() {
            if output_slot_q.get(child).is_ok() {
                let Some(output) = ct_grid.output else {
                    info!("[CRAFT] crafting_table_output_interaction: clicked output but no output available");
                    return;
                };

                if let Some(ref held) = cursor_item.0 {
                    if held.0 != output.0 {
                        info!(
                            "[CRAFT] crafting_table_output_interaction: cursor item {} doesn't match output {}",
                            held.0.display_name(),
                            output.0.display_name(),
                        );
                        return;
                    }
                    let max = held.0.max_stack() as u16;
                    if held.1 as u16 + output.1 as u16 > max {
                        info!(
                            "[CRAFT] crafting_table_output_interaction: can't stack, cursor {}+output {}>{} max",
                            held.1, output.1, max,
                        );
                        return;
                    }
                }

                info!(
                    "[CRAFT] crafting_table_output_interaction: taking output {}x{}, consuming 1 of each input",
                    output.0.display_name(),
                    output.1,
                );

                for slot in ct_grid.slots.iter_mut() {
                    if let Some((_, count, _)) = slot {
                        if *count > 1 {
                            *count -= 1;
                        } else {
                            *slot = None;
                        }
                    }
                }

                if let Some(ref mut held) = cursor_item.0 {
                    held.1 += output.1;
                } else {
                    cursor_item.0 = Some(output);
                }

                ct_grid.output = crafting::check_recipes_3x3(&ct_grid);
                info!(
                    "[CRAFT] crafting_table_output_interaction: new output after consumption: {:?}",
                    ct_grid.output.map(|(i, c, _)| format!("{}x{}", i.display_name(), c)),
                );
                return;
            }
        }
    }
}

/// Handle clicking on player inventory slots in the crafting table screen.
pub fn crafting_table_inv_slot_interaction(
    mut inventory: ResMut<Inventory>,
    mut cursor_item: ResMut<CursorItem>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    slot_q: Query<(&CraftingTableInvSlot, &ChildOf)>,
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

/// Update visual state of crafting table UI when data changes.
pub fn update_crafting_table_ui(
    ct_grid: Res<CraftingTableGrid>,
    inventory: Res<Inventory>,
    atlas: Res<UiAtlas>,
    mut slots: Query<(&CraftingTableSlot, &mut ImageNode, &mut BackgroundColor), (Without<CraftingTableOutputSlot>, Without<CraftingTableInvSlot>)>,
    mut counts: Query<(&CraftingTableSlotCount, &mut Text), (Without<CraftingTableOutputCount>, Without<CraftingTableInvSlotCount>)>,
    mut output_slot: Query<(&mut ImageNode, &mut BackgroundColor), (With<CraftingTableOutputSlot>, Without<CraftingTableSlot>, Without<CraftingTableInvSlot>)>,
    mut output_count: Query<&mut Text, (With<CraftingTableOutputCount>, Without<CraftingTableSlotCount>, Without<CraftingTableInvSlotCount>)>,
    mut inv_slots: Query<(&CraftingTableInvSlot, &mut ImageNode, &mut BackgroundColor), (Without<CraftingTableSlot>, Without<CraftingTableOutputSlot>)>,
    mut inv_counts: Query<(&CraftingTableInvSlotCount, &mut Text), (Without<CraftingTableSlotCount>, Without<CraftingTableOutputCount>)>,
) {
    let ct_changed = ct_grid.is_changed();
    let inv_changed = inventory.is_changed();

    if !ct_changed && !inv_changed {
        return;
    }

    debug!(
        "[CRAFT] update_crafting_table_ui: refreshing (grid_changed={}, inv_changed={})",
        ct_changed, inv_changed,
    );

    if ct_changed {
        for (slot, mut img, mut bg) in &mut slots {
            let data = ct_grid.slots[slot.0];
            update_slot_visual(&atlas, &data, &mut img, &mut bg);
        }
        for (slot, mut text) in &mut counts {
            **text = count_text(&ct_grid.slots[slot.0]);
        }

        if let Ok((mut img, mut bg)) = output_slot.single_mut() {
            update_slot_visual(&atlas, &ct_grid.output, &mut img, &mut bg);
        }
        if let Ok(mut text) = output_count.single_mut() {
            **text = count_text(&ct_grid.output);
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
