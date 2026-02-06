use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::block::BlockType;
use crate::block::atlas::texture_index;
use crate::block::Face;
use crate::inventory::item::Item;
use super::UiAtlas;

/// Marker for the shared hover tooltip text entity.
#[derive(Component)]
pub struct SlotHoverTooltip;


pub const SLOT_SIZE: f32 = 40.0;
pub const SLOT_GAP: f32 = 2.0;
pub const BLOCK_INSET: f32 = 4.0;
pub const BORDER_WIDTH: f32 = 2.0;
pub const INNER_SIZE: f32 = SLOT_SIZE - BLOCK_INSET * 2.0 - BORDER_WIDTH * 2.0;
pub const EMPTY_SLOT_COLOR: Color = Color::srgba(0.15, 0.15, 0.15, 0.8);

pub type SlotData = Option<(Item, u8, u16)>;

/// Build an ImageNode for an item slot (block items get a texture atlas, non-blocks get none).
pub fn slot_image(atlas: &UiAtlas, data: &SlotData) -> ImageNode {
    let texture_atlas = data.as_ref().and_then(|(item, _, _)| {
        item.as_block().and_then(|bt| {
            if bt == BlockType::Air {
                None
            } else {
                Some(TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: texture_index(bt, Face::South) as usize,
                })
            }
        })
    });
    let color = if texture_atlas.is_some() { Color::WHITE } else { Color::NONE };
    ImageNode {
        image: atlas.image.clone(),
        texture_atlas,
        color,
        ..default()
    }
}

/// Background color for a slot: empty slots get dark bg, block items get NONE (texture shows), non-block items get colored placeholder.
pub fn slot_bg(data: &SlotData) -> BackgroundColor {
    match data {
        None => BackgroundColor(EMPTY_SLOT_COLOR),
        Some((item, _, _)) => {
            if item.is_block() {
                BackgroundColor(Color::NONE)
            } else {
                BackgroundColor(item_placeholder_color(*item))
            }
        }
    }
}

/// Count text for a slot (shows count if > 1, empty string otherwise).
pub fn count_text(data: &SlotData) -> String {
    match data {
        Some((_, count, _)) if *count > 1 => format!("{}", count),
        _ => String::new(),
    }
}

/// Color for non-block items (placeholder until textures exist).
pub fn item_placeholder_color(item: Item) -> Color {
    match item {
        Item::Stick => Color::srgb(0.6, 0.45, 0.2),
        Item::Coal => Color::srgb(0.15, 0.15, 0.15),
        Item::IronIngot => Color::srgb(0.8, 0.8, 0.8),
        Item::GoldIngot => Color::srgb(0.9, 0.8, 0.2),
        Item::Diamond => Color::srgb(0.3, 0.9, 0.9),
        Item::WoodenPickaxe | Item::WoodenAxe | Item::WoodenShovel | Item::WoodenSword => {
            Color::srgb(0.7, 0.55, 0.3)
        }
        Item::StonePickaxe | Item::StoneAxe | Item::StoneShovel | Item::StoneSword => {
            Color::srgb(0.5, 0.5, 0.5)
        }
        Item::IronPickaxe | Item::IronAxe | Item::IronShovel | Item::IronSword => {
            Color::srgb(0.8, 0.8, 0.85)
        }
        Item::DiamondPickaxe | Item::DiamondAxe | Item::DiamondShovel | Item::DiamondSword => {
            Color::srgb(0.3, 0.9, 0.85)
        }
        Item::Apple => Color::srgb(0.8, 0.2, 0.2),
        Item::Bread => Color::srgb(0.7, 0.55, 0.25),
        Item::CookedPorkchop => Color::srgb(0.7, 0.4, 0.2),
        Item::RawPorkchop => Color::srgb(0.85, 0.5, 0.5),
        Item::RawBeef => Color::srgb(0.7, 0.2, 0.2),
        Item::CookedBeef => Color::srgb(0.6, 0.35, 0.15),
        Item::Leather => Color::srgb(0.6, 0.45, 0.3),
        Item::RawMutton => Color::srgb(0.75, 0.35, 0.35),
        Item::CookedMutton => Color::srgb(0.55, 0.3, 0.15),
        Item::Wool => Color::srgb(0.9, 0.9, 0.9),
        Item::RottenFlesh => Color::srgb(0.5, 0.25, 0.2),
        Item::Bone => Color::srgb(0.9, 0.85, 0.75),
        Item::WoodenHoe => Color::srgb(0.7, 0.55, 0.3),
        Item::StoneHoe => Color::srgb(0.5, 0.5, 0.5),
        Item::IronHoe => Color::srgb(0.8, 0.8, 0.85),
        Item::DiamondHoe => Color::srgb(0.3, 0.9, 0.85),
        Item::Seeds => Color::srgb(0.3, 0.6, 0.15),
        Item::Wheat => Color::srgb(0.8, 0.75, 0.2),
        Item::LeatherHelmet | Item::LeatherChestplate | Item::LeatherLeggings | Item::LeatherBoots => {
            Color::srgb(0.6, 0.4, 0.25)
        }
        Item::IronHelmet | Item::IronChestplate | Item::IronLeggings | Item::IronBoots => {
            Color::srgb(0.8, 0.8, 0.85)
        }
        Item::DiamondHelmet | Item::DiamondChestplate | Item::DiamondLeggings | Item::DiamondBoots => {
            Color::srgb(0.3, 0.9, 0.85)
        }
        Item::Block(_) => Color::NONE,
    }
}

/// Color for an item (block items use block_color, non-blocks use placeholder).
pub fn item_display_color(item: Item) -> Color {
    match item {
        Item::Block(bt) => block_color(bt),
        _ => item_placeholder_color(item),
    }
}

pub fn block_color(block: BlockType) -> Color {
    match block {
        BlockType::Dirt => Color::srgb(0.55, 0.35, 0.2),
        BlockType::Stone => Color::srgb(0.5, 0.5, 0.5),
        BlockType::Grass => Color::srgb(0.3, 0.65, 0.2),
        BlockType::OakLog => Color::srgb(0.45, 0.3, 0.15),
        BlockType::Planks => Color::srgb(0.7, 0.55, 0.3),
        BlockType::Cobblestone => Color::srgb(0.4, 0.4, 0.4),
        BlockType::Sand => Color::srgb(0.85, 0.8, 0.55),
        BlockType::Glass => Color::srgba(0.7, 0.85, 0.95, 0.5),
        BlockType::OakLeaves => Color::srgb(0.2, 0.5, 0.15),
        BlockType::Gravel => Color::srgb(0.55, 0.5, 0.48),
        BlockType::CoalOre => Color::srgb(0.3, 0.3, 0.3),
        BlockType::IronOre => Color::srgb(0.6, 0.5, 0.45),
        BlockType::GoldOre => Color::srgb(0.8, 0.7, 0.2),
        BlockType::DiamondOre => Color::srgb(0.3, 0.8, 0.8),
        BlockType::Bedrock => Color::srgb(0.2, 0.2, 0.2),
        BlockType::Water => Color::srgba(0.2, 0.3, 0.8, 0.7),
        BlockType::Snow => Color::srgb(0.95, 0.95, 0.95),
        BlockType::Clay => Color::srgb(0.6, 0.6, 0.65),
        BlockType::Sandstone => Color::srgb(0.8, 0.75, 0.5),
        BlockType::BirchLog => Color::srgb(0.85, 0.8, 0.75),
        BlockType::BirchLeaves => Color::srgb(0.35, 0.6, 0.25),
        BlockType::CraftingTable => Color::srgb(0.6, 0.45, 0.25),
        BlockType::Furnace => Color::srgb(0.45, 0.45, 0.45),
        BlockType::Torch => Color::srgb(0.9, 0.7, 0.2),
        BlockType::TallGrass => Color::srgb(0.25, 0.55, 0.18),
        BlockType::Chest => Color::srgb(0.6, 0.45, 0.2),
        BlockType::Bed => Color::srgb(0.7, 0.2, 0.2),
        BlockType::DoorBottom | BlockType::DoorTop
        | BlockType::DoorBottomOpen | BlockType::DoorTopOpen => Color::srgb(0.6, 0.45, 0.25),
        BlockType::OakSapling => Color::srgb(0.2, 0.5, 0.15),
        BlockType::BirchSapling => Color::srgb(0.35, 0.6, 0.25),
        BlockType::Farmland => Color::srgb(0.4, 0.25, 0.1),
        BlockType::WheatStage0 => Color::srgb(0.3, 0.55, 0.15),
        BlockType::WheatStage1 => Color::srgb(0.4, 0.6, 0.15),
        BlockType::WheatStage2 => Color::srgb(0.6, 0.65, 0.15),
        BlockType::WheatStage3 => Color::srgb(0.8, 0.75, 0.2),
        BlockType::Air => Color::NONE,
    }
}

/// Update a slot's visual (ImageNode + BackgroundColor) based on the slot data.
pub fn update_slot_visual(
    atlas: &UiAtlas,
    data: &SlotData,
    img: &mut ImageNode,
    bg: &mut BackgroundColor,
) {
    match data {
        Some((item, _, _)) => {
            if let Some(bt) = item.as_block() {
                if bt != BlockType::Air {
                    img.image = atlas.image.clone();
                    img.texture_atlas = Some(TextureAtlas {
                        layout: atlas.layout.clone(),
                        index: texture_index(bt, Face::South) as usize,
                    });
                    img.color = Color::WHITE;
                    *bg = BackgroundColor(Color::NONE);
                } else {
                    img.texture_atlas = None;
                    img.color = Color::NONE;
                    *bg = BackgroundColor(EMPTY_SLOT_COLOR);
                }
            } else {
                img.texture_atlas = None;
                img.color = Color::NONE;
                *bg = BackgroundColor(item_placeholder_color(*item));
            }
        }
        None => {
            img.texture_atlas = None;
            img.color = Color::NONE;
            *bg = BackgroundColor(EMPTY_SLOT_COLOR);
        }
    }
}

/// Swap slot logic: pick up / place / merge stacks between a slot and the cursor item.
pub fn swap_slot(slot: &mut SlotData, cursor: &mut SlotData) {
    let held = cursor.take();
    let existing = slot.take();

    if let Some(held_item) = held {
        if let Some(existing_item) = existing {
            if held_item.0 == existing_item.0 && !held_item.0.is_tool() {
                let max = held_item.0.max_stack() as u16;
                let total = held_item.1 as u16 + existing_item.1 as u16;
                if total <= max {
                    *slot = Some((held_item.0, total as u8, 0));
                    *cursor = None;
                } else {
                    *slot = Some((held_item.0, max as u8, 0));
                    *cursor = Some((held_item.0, (total - max) as u8, 0));
                }
            } else {
                *slot = Some(held_item);
                *cursor = Some(existing_item);
            }
        } else {
            *slot = Some(held_item);
        }
    } else if let Some(existing_item) = existing {
        *cursor = Some(existing_item);
    }
}

/// Spawn the shared hover tooltip entity (call at startup).
pub fn spawn_slot_hover_tooltip(mut commands: Commands) {
    commands.spawn((
        SlotHoverTooltip,
        Text::new(""),
        TextColor(Color::WHITE),
        TextFont { font_size: 12.0, ..default() },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            padding: UiRect::axes(Val::Px(4.0), Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ZIndex(300),
        Visibility::Hidden,
    ));
}

/// SystemParam bundle for all slot query types used by the tooltip system.
#[derive(bevy::ecs::system::SystemParam)]
pub struct SlotQueries<'w, 's> {
    pub inv_slots: Query<'w, 's, &'static super::inventory_screen::InventorySlot>,
    pub craft_slots: Query<'w, 's, &'static super::inventory_screen::CraftingSlot>,
    pub craft_output: Query<'w, 's, &'static super::inventory_screen::CraftingOutputSlot>,
    pub armor_slots: Query<'w, 's, &'static super::inventory_screen::ArmorSlotUi>,
    pub ct_slots: Query<'w, 's, &'static super::crafting_table_screen::CraftingTableSlot>,
    pub ct_output: Query<'w, 's, &'static super::crafting_table_screen::CraftingTableOutputSlot>,
    pub ct_inv_slots: Query<'w, 's, &'static super::crafting_table_screen::CraftingTableInvSlot>,
    pub furnace_input: Query<'w, 's, &'static super::furnace_screen::FurnaceInputSlot>,
    pub furnace_fuel: Query<'w, 's, &'static super::furnace_screen::FurnaceFuelSlot>,
    pub furnace_output: Query<'w, 's, &'static super::furnace_screen::FurnaceOutputSlot>,
    pub furnace_inv_slots: Query<'w, 's, &'static super::furnace_screen::FurnaceInvSlot>,
    pub chest_slots: Query<'w, 's, &'static super::chest_screen::ChestSlot>,
    pub chest_inv_slots: Query<'w, 's, &'static super::chest_screen::ChestInvSlot>,
}

/// SystemParam bundle for data resources used by the tooltip system.
#[derive(bevy::ecs::system::SystemParam)]
pub struct SlotDataSources<'w, 's> {
    pub inventory: Res<'w, crate::inventory::inventory::Inventory>,
    pub crafting_grid: Res<'w, crate::inventory::crafting::CraftingGrid>,
    pub ct_grid: Res<'w, crate::inventory::crafting::CraftingTableGrid>,
    pub furnaces: Res<'w, crate::inventory::furnace::Furnaces>,
    pub furnace_open: Res<'w, crate::inventory::furnace::FurnaceOpen>,
    pub chest_store: Res<'w, crate::inventory::chest::ChestStore>,
    pub chest_open: Res<'w, crate::inventory::chest::ChestOpen>,
    pub armor_q: Query<'w, 's, &'static crate::player::ArmorSlots, With<crate::player::Player>>,
}

/// Update the hover tooltip based on which slot is currently hovered.
/// This system checks all known slot component types across all screens.
pub fn update_slot_hover_tooltip(
    windows: Query<&Window, With<PrimaryWindow>>,
    interaction_q: Query<(&Interaction, &Children)>,
    slots: SlotQueries,
    data: SlotDataSources,
    mut tooltip_q: Query<(&mut Text, &mut Node, &mut Visibility), With<SlotHoverTooltip>>,
) {
    let Ok((mut text, mut node, mut vis)) = tooltip_q.single_mut() else { return };

    let mut found_name: Option<String> = None;

    for (interaction, children) in &interaction_q {
        if *interaction != Interaction::Hovered {
            continue;
        }
        for child in children.iter() {
            // Inventory slots
            if let Ok(slot) = slots.inv_slots.get(child) {
                if let Some((item, _, _)) = data.inventory.slots[slot.0] {
                    found_name = Some(item.display_name().to_string());
                }
            }
            // Crafting 2x2 slots
            if let Ok(slot) = slots.craft_slots.get(child) {
                if let Some((item, _, _)) = data.crafting_grid.slots[slot.0] {
                    found_name = Some(item.display_name().to_string());
                }
            }
            // Crafting 2x2 output
            if slots.craft_output.get(child).is_ok() {
                if let Some((item, _, _)) = data.crafting_grid.output {
                    found_name = Some(item.display_name().to_string());
                }
            }
            // Armor slots
            if let Ok(slot) = slots.armor_slots.get(child) {
                if let Ok(armor) = data.armor_q.single() {
                    if let Some((item, _, _)) = armor.slots[slot.0] {
                        found_name = Some(item.display_name().to_string());
                    }
                }
            }
            // Crafting table 3x3 slots
            if let Ok(slot) = slots.ct_slots.get(child) {
                if let Some((item, _, _)) = data.ct_grid.slots[slot.0] {
                    found_name = Some(item.display_name().to_string());
                }
            }
            // Crafting table output
            if slots.ct_output.get(child).is_ok() {
                if let Some((item, _, _)) = data.ct_grid.output {
                    found_name = Some(item.display_name().to_string());
                }
            }
            // Crafting table inventory slots
            if let Ok(slot) = slots.ct_inv_slots.get(child) {
                if let Some((item, _, _)) = data.inventory.slots[slot.0] {
                    found_name = Some(item.display_name().to_string());
                }
            }
            // Furnace input
            if slots.furnace_input.get(child).is_ok() {
                if let Some(pos) = data.furnace_open.0 {
                    if let Some(fdata) = data.furnaces.data.get(&pos) {
                        if let Some((item, _, _)) = fdata.input {
                            found_name = Some(item.display_name().to_string());
                        }
                    }
                }
            }
            // Furnace fuel
            if slots.furnace_fuel.get(child).is_ok() {
                if let Some(pos) = data.furnace_open.0 {
                    if let Some(fdata) = data.furnaces.data.get(&pos) {
                        if let Some((item, _, _)) = fdata.fuel {
                            found_name = Some(item.display_name().to_string());
                        }
                    }
                }
            }
            // Furnace output
            if slots.furnace_output.get(child).is_ok() {
                if let Some(pos) = data.furnace_open.0 {
                    if let Some(fdata) = data.furnaces.data.get(&pos) {
                        if let Some((item, _, _)) = fdata.output {
                            found_name = Some(item.display_name().to_string());
                        }
                    }
                }
            }
            // Furnace inventory slots
            if let Ok(slot) = slots.furnace_inv_slots.get(child) {
                if let Some((item, _, _)) = data.inventory.slots[slot.0] {
                    found_name = Some(item.display_name().to_string());
                }
            }
            // Chest slots
            if let Ok(slot) = slots.chest_slots.get(child) {
                if let Some(pos) = data.chest_open.0 {
                    if let Some(cdata) = data.chest_store.data.get(&pos) {
                        if let Some((item, _, _)) = cdata.slots[slot.0] {
                            found_name = Some(item.display_name().to_string());
                        }
                    }
                }
            }
            // Chest inventory slots
            if let Ok(slot) = slots.chest_inv_slots.get(child) {
                if let Some((item, _, _)) = data.inventory.slots[slot.0] {
                    found_name = Some(item.display_name().to_string());
                }
            }

            if found_name.is_some() {
                break;
            }
        }
        if found_name.is_some() {
            break;
        }
    }

    match found_name {
        Some(name) => {
            **text = name;
            *vis = Visibility::Visible;
            if let Ok(window) = windows.single() {
                if let Some(pos) = window.cursor_position() {
                    node.left = Val::Px(pos.x + 12.0);
                    node.top = Val::Px(pos.y - 24.0);
                }
            }
        }
        None => {
            *vis = Visibility::Hidden;
        }
    }
}

/// Right-click slot logic: pick up half, place 1, or do nothing on type mismatch.
pub fn swap_slot_right_click(slot: &mut SlotData, cursor: &mut SlotData) {
    let held = cursor.as_ref().cloned();
    let existing = slot.as_ref().cloned();

    match (held, existing) {
        // Empty cursor + occupied slot -> pick up half (rounded up), remainder stays
        (None, Some(existing_item)) => {
            let take = (existing_item.1 + 1) / 2; // rounded up
            let remain = existing_item.1 - take;
            *cursor = Some((existing_item.0, take, existing_item.2));
            if remain > 0 {
                *slot = Some((existing_item.0, remain, existing_item.2));
            } else {
                *slot = None;
            }
        }
        // Cursor item + empty slot -> place 1 item
        (Some(held_item), None) => {
            *slot = Some((held_item.0, 1, held_item.2));
            if held_item.1 > 1 {
                *cursor = Some((held_item.0, held_item.1 - 1, held_item.2));
            } else {
                *cursor = None;
            }
        }
        // Cursor item + same type slot -> place 1 item (if room)
        (Some(held_item), Some(existing_item)) if held_item.0 == existing_item.0 && !held_item.0.is_tool() => {
            let max = held_item.0.max_stack();
            if existing_item.1 < max {
                *slot = Some((existing_item.0, existing_item.1 + 1, existing_item.2));
                if held_item.1 > 1 {
                    *cursor = Some((held_item.0, held_item.1 - 1, held_item.2));
                } else {
                    *cursor = None;
                }
            }
            // else: slot full, do nothing
        }
        // Cursor item + different type slot -> do nothing
        (Some(_), Some(_)) => {}
        // Both empty -> nothing
        (None, None) => {}
    }
}