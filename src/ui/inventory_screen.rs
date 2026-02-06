use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::crafting::{self, CraftingGrid, CRAFTING_GRID_SIZE};
use crate::inventory::inventory::{Inventory, INVENTORY_COLS, INVENTORY_SLOTS};
use crate::inventory::item::Item;
use super::UiAtlas;
use super::common::*;

#[derive(Resource, Default)]
pub struct InventoryOpen(pub bool);

/// Tracks a "held" item on the cursor for drag-and-drop style interaction.
#[derive(Resource, Default)]
pub struct CursorItem(pub Option<(Item, u8, u16)>);

#[derive(Component)]
pub(crate) struct InventoryUiRoot;

#[derive(Component)]
pub(crate) struct InventorySlot(usize);

#[derive(Component)]
pub(crate) struct InventorySlotCount(usize);

#[derive(Component)]
pub(crate) struct CraftingSlot(usize);

#[derive(Component)]
pub(crate) struct CraftingSlotCount(usize);

#[derive(Component)]
pub(crate) struct CraftingOutputSlot;

#[derive(Component)]
pub(crate) struct CraftingOutputCount;

#[derive(Component)]
pub(crate) struct CursorItemDisplay;

#[derive(Component)]
pub(crate) struct ArmorSlotUi(usize); // 0=helmet, 1=chest, 2=legs, 3=boots

#[derive(Component)]
pub(crate) struct ArmorSlotCount(usize);

pub fn toggle_inventory(
    keys: Res<ButtonInput<KeyCode>>,
    mut inventory_open: ResMut<InventoryOpen>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut crafting_grid: ResMut<CraftingGrid>,
    mut inventory: ResMut<Inventory>,
    mut cursor_item: ResMut<CursorItem>,
    furnace_open: Res<crate::inventory::furnace::FurnaceOpen>,
    ct_open: Res<crate::inventory::crafting::CraftingTableOpen>,
    chest_open: Res<crate::inventory::chest::ChestOpen>,
    dead: Res<crate::ui::death_screen::PlayerDead>,
) {
    if furnace_open.0.is_some() || ct_open.0 || chest_open.0.is_some() || dead.0 {
        return;
    }

    let toggle = keys.just_pressed(KeyCode::KeyE)
        || (inventory_open.0 && keys.just_pressed(KeyCode::Escape));

    if !toggle {
        return;
    }

    inventory_open.0 = !inventory_open.0;

    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };

    if inventory_open.0 {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    } else {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
        crafting::clear_crafting_grid(&mut crafting_grid, &mut inventory);
        if let Some((item, count, _)) = cursor_item.0.take() {
            for _ in 0..count {
                inventory.add_item(item);
            }
        }
    }
}

pub fn spawn_inventory_ui(
    mut commands: Commands,
    inventory_open: Res<InventoryOpen>,
    inventory: Res<Inventory>,
    crafting_grid: Res<CraftingGrid>,
    atlas: Res<UiAtlas>,
    existing: Query<Entity, With<InventoryUiRoot>>,
    armor_q: Query<&crate::player::ArmorSlots, With<crate::player::Player>>,
) {
    if !inventory_open.is_changed() || !inventory_open.0 || !existing.is_empty() {
        return;
    }

    let rows = INVENTORY_SLOTS / INVENTORY_COLS;

    commands
        .spawn((
            InventoryUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            // === Armor + Crafting row ===
            let armor_slots = armor_q.single().ok();
            let armor_labels = ["Helmet", "Chest", "Legs", "Boots"];
            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(24.0),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|row| {
                    // Armor slots column
                    row.spawn(Node {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(SLOT_GAP),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|col| {
                        col.spawn((
                            Text::new("Armor"),
                            TextColor(Color::WHITE),
                            TextFont { font_size: 14.0, ..default() },
                            Node { margin: UiRect::bottom(Val::Px(4.0)), ..default() },
                        ));
                        for i in 0..4 {
                            let data = armor_slots.and_then(|a| a.slots[i]);
                            col.spawn((
                                Node {
                                    width: Val::Px(SLOT_SIZE),
                                    height: Val::Px(SLOT_SIZE),
                                    border: UiRect::all(Val::Px(BORDER_WIDTH)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BorderColor::all(Color::srgba(0.4, 0.6, 0.8, 0.7)),
                                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.7)),
                                Interaction::default(),
                            ))
                            .with_children(|sp| {
                                sp.spawn((
                                    ArmorSlotUi(i),
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
                                        ArmorSlotCount(i),
                                        Text::new(if data.is_some() { armor_labels[i].to_string() } else { armor_labels[i].to_string() }),
                                        TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
                                        TextFont { font_size: 8.0, ..default() },
                                    ));
                                });
                            });
                        }
                    });
                });

            // === Crafting area ===
            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(12.0),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|crafting_row| {
                    crafting_row.spawn((
                        Text::new("Crafting"),
                        TextColor(Color::WHITE),
                        TextFont { font_size: 16.0, ..default() },
                        Node { margin: UiRect::right(Val::Px(8.0)), ..default() },
                    ));

                    // 2x2 crafting grid
                    crafting_row
                        .spawn(Node {
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(SLOT_GAP),
                            ..default()
                        })
                        .with_children(|grid| {
                            for row in 0..CRAFTING_GRID_SIZE {
                                grid.spawn(Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Row,
                                    column_gap: Val::Px(SLOT_GAP),
                                    ..default()
                                })
                                .with_children(|row_node| {
                                    for col in 0..CRAFTING_GRID_SIZE {
                                        let slot_idx = row * CRAFTING_GRID_SIZE + col;
                                        let data = crafting_grid.slots[slot_idx];
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
                                                    CraftingSlot(slot_idx),
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
                                                        CraftingSlotCount(slot_idx),
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
                    let output = crafting_grid.output;
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
                                CraftingOutputSlot,
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
                                    CraftingOutputCount,
                                    Text::new(count_text(&output)),
                                    TextColor(Color::WHITE),
                                    TextFont { font_size: 11.0, ..default() },
                                ));
                            });
                        });
                });

            // === Inventory grid ===
            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(SLOT_GAP),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|grid| {
                    grid.spawn((
                        Text::new("Inventory"),
                        TextColor(Color::WHITE),
                        TextFont { font_size: 18.0, ..default() },
                        Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
                    ));

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
                                            InventorySlot(slot_idx),
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
                                                InventorySlotCount(slot_idx),
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

            // Cursor item display
            parent.spawn((
                CursorItemDisplay,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(SLOT_SIZE - 8.0),
                    height: Val::Px(SLOT_SIZE - 8.0),
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Visibility::Hidden,
            ));
        });
}

pub fn despawn_inventory_ui(
    mut commands: Commands,
    inventory_open: Res<InventoryOpen>,
    query: Query<Entity, With<InventoryUiRoot>>,
) {
    if !inventory_open.is_changed() || inventory_open.0 {
        return;
    }
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Handle clicking on inventory slots to pick up / place items.
pub fn inventory_slot_interaction(
    mut inventory: ResMut<Inventory>,
    mut cursor_item: ResMut<CursorItem>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    inv_slot_q: Query<(&InventorySlot, &ChildOf)>,
    interaction_q: Query<(&Interaction, &Children)>,
) {
    // Left-click: Interaction::Pressed fires on left mouse button
    for (interaction, children) in &interaction_q {
        if *interaction == Interaction::Pressed {
            for child in children.iter() {
                if let Ok((slot, _)) = inv_slot_q.get(child) {
                    swap_slot(&mut inventory.slots[slot.0], &mut cursor_item.0);
                    return;
                }
            }
        }
        // Right-click: detect via Hovered + MouseButton::Right just_pressed
        if *interaction == Interaction::Hovered && mouse_buttons.just_pressed(MouseButton::Right) {
            for child in children.iter() {
                if let Ok((slot, _)) = inv_slot_q.get(child) {
                    swap_slot_right_click(&mut inventory.slots[slot.0], &mut cursor_item.0);
                    return;
                }
            }
        }
    }
}

/// Handle clicking on crafting input slots.
pub fn crafting_slot_interaction(
    mut crafting_grid: ResMut<CraftingGrid>,
    mut cursor_item: ResMut<CursorItem>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    craft_slot_q: Query<(&CraftingSlot, &ChildOf)>,
    interaction_q: Query<(&Interaction, &Children)>,
) {
    for (interaction, children) in &interaction_q {
        if *interaction == Interaction::Pressed {
            for child in children.iter() {
                if let Ok((slot, _)) = craft_slot_q.get(child) {
                    swap_slot(&mut crafting_grid.slots[slot.0], &mut cursor_item.0);
                    return;
                }
            }
        }
        if *interaction == Interaction::Hovered && mouse_buttons.just_pressed(MouseButton::Right) {
            for child in children.iter() {
                if let Ok((slot, _)) = craft_slot_q.get(child) {
                    swap_slot_right_click(&mut crafting_grid.slots[slot.0], &mut cursor_item.0);
                    return;
                }
            }
        }
    }
}

/// Handle clicking the crafting output slot.
pub fn crafting_output_interaction(
    mut crafting_grid: ResMut<CraftingGrid>,
    mut cursor_item: ResMut<CursorItem>,
    output_slot_q: Query<&CraftingOutputSlot>,
    interaction_q: Query<(&Interaction, &Children), Changed<Interaction>>,
) {
    for (interaction, children) in &interaction_q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for child in children.iter() {
            if output_slot_q.get(child).is_ok() {
                let Some(output) = crafting_grid.output else {
                    info!("[CRAFT] crafting_output_interaction (2x2): clicked but no output");
                    return;
                };

                if let Some(ref held) = cursor_item.0 {
                    if held.0 != output.0 {
                        info!(
                            "[CRAFT] crafting_output_interaction (2x2): cursor {} doesn't match output {}",
                            held.0.display_name(), output.0.display_name(),
                        );
                        return;
                    }
                    let max = held.0.max_stack() as u16;
                    if held.1 as u16 + output.1 as u16 > max {
                        info!(
                            "[CRAFT] crafting_output_interaction (2x2): stack overflow {}+{}>{} max",
                            held.1, output.1, max,
                        );
                        return;
                    }
                }

                info!(
                    "[CRAFT] crafting_output_interaction (2x2): taking output {}x{}",
                    output.0.display_name(), output.1,
                );

                for slot in crafting_grid.slots.iter_mut() {
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

                crafting_grid.output = crafting::check_recipes(&crafting_grid);
                info!(
                    "[CRAFT] crafting_output_interaction (2x2): new output={:?}",
                    crafting_grid.output.map(|(i, c, _)| format!("{}x{}", i.display_name(), c)),
                );
                return;
            }
        }
    }
}

/// Handle clicking on armor slots to equip/unequip armor.
pub fn armor_slot_interaction(
    mut cursor_item: ResMut<CursorItem>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    armor_slot_q: Query<(&ArmorSlotUi, &ChildOf)>,
    interaction_q: Query<(&Interaction, &Children)>,
    mut armor_q: Query<&mut crate::player::ArmorSlots, With<crate::player::Player>>,
) {
    let Ok(mut armor) = armor_q.single_mut() else { return };

    for (interaction, children) in &interaction_q {
        let clicked = *interaction == Interaction::Pressed
            || (*interaction == Interaction::Hovered && mouse_buttons.just_pressed(MouseButton::Right));
        if !clicked {
            continue;
        }
        for child in children.iter() {
            if let Ok((slot, _)) = armor_slot_q.get(child) {
                let slot_idx = slot.0;
                match (&cursor_item.0, &armor.slots[slot_idx]) {
                    // Empty cursor + occupied slot -> pick up armor
                    (None, Some(_)) => {
                        cursor_item.0 = armor.slots[slot_idx].take();
                        return;
                    }
                    // Cursor with matching armor type -> swap (equip cursor, unequip slot)
                    (Some((item, _, _)), _) if item.armor_slot() == Some(slot_idx) => {
                        let old = armor.slots[slot_idx].take();
                        armor.slots[slot_idx] = cursor_item.0.take();
                        cursor_item.0 = old;
                        return;
                    }
                    // Cursor with non-matching item + any slot -> do nothing
                    _ => {}
                }
            }
        }
    }
}

/// Update visual state of inventory and crafting UI when data changes.
pub fn update_inventory_ui(
    inventory: Res<Inventory>,
    crafting_grid: Res<CraftingGrid>,
    cursor_item: Res<CursorItem>,
    atlas: Res<UiAtlas>,
    mut inv_slots: Query<(&InventorySlot, &mut ImageNode, &mut BackgroundColor), Without<CraftingSlot>>,
    mut inv_counts: Query<(&InventorySlotCount, &mut Text), (Without<CraftingSlotCount>, Without<CraftingOutputCount>)>,
    mut craft_slots: Query<(&CraftingSlot, &mut ImageNode, &mut BackgroundColor), Without<InventorySlot>>,
    mut craft_counts: Query<(&CraftingSlotCount, &mut Text), (Without<InventorySlotCount>, Without<CraftingOutputCount>)>,
    mut output_slot: Query<(&mut ImageNode, &mut BackgroundColor), (With<CraftingOutputSlot>, Without<InventorySlot>, Without<CraftingSlot>)>,
    mut output_count: Query<&mut Text, (With<CraftingOutputCount>, Without<InventorySlotCount>, Without<CraftingSlotCount>)>,
) {
    let inv_changed = inventory.is_changed();
    let craft_changed = crafting_grid.is_changed();

    if !inv_changed && !craft_changed && !cursor_item.is_changed() {
        return;
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

    if craft_changed {
        for (slot, mut img, mut bg) in &mut craft_slots {
            let data = crafting_grid.slots[slot.0];
            update_slot_visual(&atlas, &data, &mut img, &mut bg);
        }
        for (slot, mut text) in &mut craft_counts {
            **text = count_text(&crafting_grid.slots[slot.0]);
        }

        if let Ok((mut img, mut bg)) = output_slot.single_mut() {
            update_slot_visual(&atlas, &crafting_grid.output, &mut img, &mut bg);
        }
        if let Ok(mut text) = output_count.single_mut() {
            **text = count_text(&crafting_grid.output);
        }
    }
}

/// Update armor slot visuals in the inventory screen.
pub fn update_armor_slots_ui(
    armor_q: Query<&crate::player::ArmorSlots, With<crate::player::Player>>,
    atlas: Res<UiAtlas>,
    mut armor_slots: Query<(&ArmorSlotUi, &mut ImageNode, &mut BackgroundColor)>,
) {
    let Ok(armor) = armor_q.single() else { return };
    for (slot, mut img, mut bg) in &mut armor_slots {
        let data = armor.slots[slot.0];
        update_slot_visual(&atlas, &data, &mut img, &mut bg);
    }
}

/// Update the cursor item display to follow the mouse.
pub fn update_cursor_item_display(
    cursor_item: Res<CursorItem>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut display_q: Query<(&mut Node, &mut BackgroundColor, &mut Visibility), With<CursorItemDisplay>>,
) {
    let Ok((mut node, mut bg, mut vis)) = display_q.single_mut() else {
        return;
    };

    match &cursor_item.0 {
        Some((item, _, _)) => {
            *vis = Visibility::Visible;
            *bg = BackgroundColor(item_display_color(*item));
            if let Ok(window) = windows.single() {
                if let Some(pos) = window.cursor_position() {
                    node.left = Val::Px(pos.x - 16.0);
                    node.top = Val::Px(pos.y - 16.0);
                }
            }
        }
        None => {
            *vis = Visibility::Hidden;
            *bg = BackgroundColor(Color::NONE);
        }
    }
}
