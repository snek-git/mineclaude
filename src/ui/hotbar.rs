use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;

use crate::block::BlockType;
use crate::block::atlas::texture_index;
use crate::block::Face;
use crate::inventory::inventory::{Inventory, INVENTORY_COLS, INVENTORY_SLOTS};
use crate::inventory::item::Item;
use super::UiAtlas;
use super::common::{SLOT_SIZE, SLOT_GAP, BLOCK_INSET, BORDER_WIDTH, item_placeholder_color};

pub const HOTBAR_SLOTS: usize = 9;

#[derive(Resource)]
pub struct HotbarState {
    pub slots: [Item; HOTBAR_SLOTS],
    pub durabilities: [(u16, u16); HOTBAR_SLOTS], // (current, max)
    pub selected_slot: usize,
}

impl Default for HotbarState {
    fn default() -> Self {
        Self {
            slots: [Item::Block(BlockType::Air); HOTBAR_SLOTS],
            durabilities: [(0, 0); HOTBAR_SLOTS],
            selected_slot: 0,
        }
    }
}

#[derive(Component)]
pub(crate) struct HotbarSlot(usize);

#[derive(Component)]
pub(crate) struct HotbarSlotBorder(usize);

#[derive(Component)]
pub(crate) struct HotbarItemName;

#[derive(Component)]
pub(crate) struct DurabilityBar(usize);

pub fn spawn_hotbar(mut commands: Commands, atlas: Res<UiAtlas>, inventory: Res<Inventory>) {
    let total_width = HOTBAR_SLOTS as f32 * SLOT_SIZE + (HOTBAR_SLOTS as f32 - 1.0) * SLOT_GAP;
    let hotbar_start = INVENTORY_SLOTS - INVENTORY_COLS;

    // Container for hotbar + item name
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-total_width / 2.0),
                ..default()
            },
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            // Item name text above hotbar
            let initial_item = match inventory.slots[hotbar_start] {
                Some((item, _, _)) => item,
                None => Item::Block(BlockType::Air),
            };
            parent.spawn((
                HotbarItemName,
                Text::new(initial_item.display_name()),
                TextColor(Color::WHITE),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ));

            // Hotbar slots row
            parent
                .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(SLOT_GAP),
                    ..default()
                })
                .with_children(|row| {
                    for i in 0..HOTBAR_SLOTS {
                        let inv_slot = hotbar_start + i;
                        let item = match inventory.slots[inv_slot] {
                            Some((it, _, _)) => it,
                            None => Item::Block(BlockType::Air),
                        };

                        row.spawn((
                            HotbarSlotBorder(i),
                            Node {
                                width: Val::Px(SLOT_SIZE),
                                height: Val::Px(SLOT_SIZE),
                                border: UiRect::all(Val::Px(BORDER_WIDTH)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.6)),
                            BorderColor::all(if i == 0 {
                                Color::WHITE
                            } else {
                                Color::srgba(0.5, 0.5, 0.5, 0.5)
                            }),
                        ))
                        .with_children(|slot_parent| {
                            let inner_size = SLOT_SIZE - BLOCK_INSET * 2.0 - BORDER_WIDTH * 2.0;
                            if let Some(bt) = item.as_block() {
                                if bt != BlockType::Air {
                                    let tile = texture_index(bt, Face::South) as usize;
                                    slot_parent.spawn((
                                        HotbarSlot(i),
                                        Node {
                                            width: Val::Px(inner_size),
                                            height: Val::Px(inner_size),
                                            ..default()
                                        },
                                        ImageNode {
                                            image: atlas.image.clone(),
                                            texture_atlas: Some(TextureAtlas {
                                                layout: atlas.layout.clone(),
                                                index: tile,
                                            }),
                                            ..default()
                                        },
                                    ));
                                } else {
                                    slot_parent.spawn((
                                        HotbarSlot(i),
                                        Node {
                                            width: Val::Px(inner_size),
                                            height: Val::Px(inner_size),
                                            ..default()
                                        },
                                        BackgroundColor(Color::NONE),
                                    ));
                                }
                            } else {
                                // Non-block item: colored placeholder
                                slot_parent.spawn((
                                    HotbarSlot(i),
                                    Node {
                                        width: Val::Px(inner_size),
                                        height: Val::Px(inner_size),
                                        ..default()
                                    },
                                    BackgroundColor(item_placeholder_color(item)),
                                ));
                            }

                            // Durability bar at bottom of slot
                            slot_parent.spawn((
                                DurabilityBar(i),
                                Node {
                                    position_type: PositionType::Absolute,
                                    bottom: Val::Px(1.0),
                                    left: Val::Px(BLOCK_INSET - BORDER_WIDTH),
                                    width: Val::Px(inner_size),
                                    height: Val::Px(2.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.0, 1.0, 0.0)),
                                Visibility::Hidden,
                            ));
                        });
                    }
                });
        });
}

pub fn hotbar_input(
    keys: Res<ButtonInput<KeyCode>>,
    scroll: Res<AccumulatedMouseScroll>,
    mut hotbar: ResMut<HotbarState>,
) {
    let number_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];

    for (i, key) in number_keys.iter().enumerate() {
        if keys.just_pressed(*key) {
            hotbar.selected_slot = i;
            return;
        }
    }

    let scroll_y = scroll.delta.y;
    if scroll_y.abs() > 0.0 {
        let current = hotbar.selected_slot as i32;
        let next = (current - scroll_y.signum() as i32).rem_euclid(HOTBAR_SLOTS as i32);
        hotbar.selected_slot = next as usize;
    }
}

/// Sync hotbar item types from the inventory's bottom row.
pub fn sync_hotbar_from_inventory(
    inventory: Res<Inventory>,
    mut hotbar: ResMut<HotbarState>,
) {
    if !inventory.is_changed() {
        return;
    }
    let hotbar_start = INVENTORY_SLOTS - INVENTORY_COLS;
    for i in 0..HOTBAR_SLOTS {
        hotbar.slots[i] = match inventory.slots[hotbar_start + i] {
            Some((item, _, _)) => item,
            None => Item::Block(BlockType::Air),
        };
        hotbar.durabilities[i] = match inventory.slots[hotbar_start + i] {
            Some((item, _, dur)) => (dur, item.max_durability()),
            None => (0, 0),
        };
    }
}

pub fn update_hotbar_visuals(
    hotbar: Res<HotbarState>,
    atlas: Res<UiAtlas>,
    mut border_q: Query<(&HotbarSlotBorder, &mut BorderColor)>,
    mut slot_q: Query<(&HotbarSlot, &mut Node, Option<&mut ImageNode>, Option<&mut BackgroundColor>), Without<DurabilityBar>>,
    mut dur_q: Query<(&DurabilityBar, &mut Node, &mut BackgroundColor, &mut Visibility), Without<HotbarSlot>>,
) {
    if !hotbar.is_changed() {
        return;
    }

    for (border, mut color) in &mut border_q {
        *color = if border.0 == hotbar.selected_slot {
            BorderColor::all(Color::WHITE)
        } else {
            BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 0.5))
        };
    }

    for (slot, _node, img_node, bg_color) in &mut slot_q {
        let item = hotbar.slots[slot.0];
        if let Some(bt) = item.as_block() {
            if bt != BlockType::Air {
                let tile = texture_index(bt, Face::South) as usize;
                if let Some(mut img) = img_node {
                    img.image = atlas.image.clone();
                    img.texture_atlas = Some(TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: tile,
                    });
                }
                if let Some(mut bg) = bg_color {
                    *bg = BackgroundColor(Color::NONE);
                }
            } else {
                if let Some(mut img) = img_node {
                    img.texture_atlas = None;
                }
                if let Some(mut bg) = bg_color {
                    *bg = BackgroundColor(Color::NONE);
                }
            }
        } else {
            // Non-block item
            if let Some(mut img) = img_node {
                img.texture_atlas = None;
            }
            if let Some(mut bg) = bg_color {
                *bg = BackgroundColor(item_placeholder_color(item));
            }
        }
    }

    // Update durability bars
    let inner_size = SLOT_SIZE - BLOCK_INSET * 2.0 - BORDER_WIDTH * 2.0;
    for (bar, mut node, mut bg, mut vis) in &mut dur_q {
        let item = hotbar.slots[bar.0];
        let (current, max) = hotbar.durabilities[bar.0];
        if item.is_tool() && max > 0 && current < max {
            *vis = Visibility::Inherited;
            let ratio = current as f32 / max as f32;
            node.width = Val::Px(ratio * inner_size);
            *bg = BackgroundColor(if ratio > 0.5 {
                Color::srgb(0.0, 1.0, 0.0)
            } else if ratio > 0.25 {
                Color::srgb(1.0, 1.0, 0.0)
            } else {
                Color::srgb(1.0, 0.0, 0.0)
            });
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

/// Update the item name text above the hotbar.
pub fn update_item_name(
    hotbar: Res<HotbarState>,
    mut name_q: Query<&mut Text, With<HotbarItemName>>,
) {
    if !hotbar.is_changed() {
        return;
    }
    let item = hotbar.slots[hotbar.selected_slot];
    for mut text in &mut name_q {
        **text = item.display_name().to_string();
    }
}
