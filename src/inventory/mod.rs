pub mod chest;
pub mod crafting;
pub mod furnace;
pub mod inventory;
pub mod item;

use bevy::prelude::*;

pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        let inv = load_saved_inventory();
        app.insert_resource(inv)
            .init_resource::<crafting::CraftingGrid>()
            .init_resource::<crafting::CraftingTableGrid>()
            .init_resource::<crafting::CraftingTableOpen>()
            .insert_resource(crate::save::persistence::load_furnaces())
            .init_resource::<furnace::FurnaceOpen>()
            .insert_resource(crate::save::persistence::load_chests())
            .init_resource::<chest::ChestOpen>()
            .add_systems(Update, (
                crafting::update_crafting_output,
                crafting::update_crafting_table_output,
                furnace::furnace_tick,
            ));
    }
}

fn load_saved_inventory() -> inventory::Inventory {
    if let Some(data) = crate::save::persistence::load_player() {
        if data.inventory.len() == inventory::INVENTORY_SLOTS {
            let mut slots: [inventory::Slot; inventory::INVENTORY_SLOTS] = [None; inventory::INVENTORY_SLOTS];
            for (i, slot) in data.inventory.into_iter().enumerate() {
                slots[i] = slot;
            }
            return inventory::Inventory { slots };
        }
    }
    inventory::Inventory::default()
}
