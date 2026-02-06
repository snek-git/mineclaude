use bevy::prelude::*;

use super::item::Item;

pub const INVENTORY_SLOTS: usize = 36; // 4 rows x 9 columns
pub const INVENTORY_COLS: usize = 9;

/// Inventory slot: (item, count, durability). Durability is remaining uses for tools (0 = N/A).
pub type Slot = Option<(Item, u8, u16)>;

#[derive(Resource)]
pub struct Inventory {
    pub slots: [Slot; INVENTORY_SLOTS],
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: [None; INVENTORY_SLOTS],
        }
    }
}

impl Inventory {
    /// Add one item to the inventory. Returns false if inventory is full.
    pub fn add_item(&mut self, item: Item) -> bool {
        let max = item.max_stack();
        let dur = item.max_durability();
        // First, try to stack onto an existing slot with the same item type (non-tools only)
        if max > 1 {
            for slot in self.slots.iter_mut() {
                if let Some((it, count, _)) = slot {
                    if *it == item && *count < max {
                        *count += 1;
                        return true;
                    }
                }
            }
        }
        // Otherwise, find the first empty slot
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some((item, 1, dur));
                return true;
            }
        }
        false
    }

    /// Remove one item from a specific slot. Returns the item if successful.
    pub fn remove_item(&mut self, slot: usize) -> Option<Item> {
        if slot >= INVENTORY_SLOTS {
            return None;
        }
        if let Some((item, count, _)) = &mut self.slots[slot] {
            let it = *item;
            if *count > 1 {
                *count -= 1;
            } else {
                self.slots[slot] = None;
            }
            Some(it)
        } else {
            None
        }
    }

    /// Decrement durability of the tool in the given slot. Removes it if durability reaches 0.
    /// Returns true if the tool broke (was removed).
    pub fn use_tool(&mut self, slot: usize) -> bool {
        if slot >= INVENTORY_SLOTS {
            return false;
        }
        if let Some((item, _, dur)) = &mut self.slots[slot] {
            if item.is_tool() && *dur > 0 {
                *dur -= 1;
                if *dur == 0 {
                    self.slots[slot] = None;
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockType;

    fn empty_inventory() -> Inventory {
        Inventory {
            slots: [None; INVENTORY_SLOTS],
        }
    }

    #[test]
    fn add_item_to_empty_inventory() {
        let mut inv = empty_inventory();
        assert!(inv.add_item(Item::Block(BlockType::Stone)));
        assert_eq!(inv.slots[0], Some((Item::Block(BlockType::Stone), 1, 0)));
    }

    #[test]
    fn add_item_stacks_on_existing() {
        let mut inv = empty_inventory();
        inv.add_item(Item::Block(BlockType::Stone));
        inv.add_item(Item::Block(BlockType::Stone));
        assert_eq!(inv.slots[0], Some((Item::Block(BlockType::Stone), 2, 0)));
        assert_eq!(inv.slots[1], None);
    }

    #[test]
    fn add_item_different_types_use_different_slots() {
        let mut inv = empty_inventory();
        inv.add_item(Item::Block(BlockType::Stone));
        inv.add_item(Item::Block(BlockType::Dirt));
        assert_eq!(inv.slots[0], Some((Item::Block(BlockType::Stone), 1, 0)));
        assert_eq!(inv.slots[1], Some((Item::Block(BlockType::Dirt), 1, 0)));
    }

    #[test]
    fn add_item_respects_max_stack() {
        let mut inv = empty_inventory();
        inv.slots[0] = Some((Item::Block(BlockType::Stone), 64, 0));
        assert!(inv.add_item(Item::Block(BlockType::Stone)));
        assert_eq!(inv.slots[0], Some((Item::Block(BlockType::Stone), 64, 0)));
        assert_eq!(inv.slots[1], Some((Item::Block(BlockType::Stone), 1, 0)));
    }

    #[test]
    fn tools_do_not_stack() {
        let mut inv = empty_inventory();
        inv.add_item(Item::WoodenPickaxe);
        inv.add_item(Item::WoodenPickaxe);
        // Tools have max_stack=1, so they should use separate slots with durability
        assert_eq!(inv.slots[0], Some((Item::WoodenPickaxe, 1, 59)));
        assert_eq!(inv.slots[1], Some((Item::WoodenPickaxe, 1, 59)));
    }

    #[test]
    fn add_item_fails_when_full() {
        let mut inv = empty_inventory();
        for i in 0..INVENTORY_SLOTS {
            inv.slots[i] = Some((Item::Block(BlockType::Bedrock), 64, 0));
        }
        assert!(!inv.add_item(Item::Block(BlockType::Stone)));
    }

    #[test]
    fn remove_item_returns_item_and_decrements() {
        let mut inv = empty_inventory();
        inv.slots[0] = Some((Item::Block(BlockType::Dirt), 5, 0));
        let removed = inv.remove_item(0);
        assert_eq!(removed, Some(Item::Block(BlockType::Dirt)));
        assert_eq!(inv.slots[0], Some((Item::Block(BlockType::Dirt), 4, 0)));
    }

    #[test]
    fn remove_item_clears_slot_at_count_1() {
        let mut inv = empty_inventory();
        inv.slots[0] = Some((Item::Block(BlockType::Dirt), 1, 0));
        let removed = inv.remove_item(0);
        assert_eq!(removed, Some(Item::Block(BlockType::Dirt)));
        assert_eq!(inv.slots[0], None);
    }

    #[test]
    fn remove_item_from_empty_slot() {
        let mut inv = empty_inventory();
        assert_eq!(inv.remove_item(0), None);
    }

    #[test]
    fn remove_item_out_of_bounds() {
        let mut inv = empty_inventory();
        assert_eq!(inv.remove_item(INVENTORY_SLOTS), None);
        assert_eq!(inv.remove_item(100), None);
    }

    #[test]
    fn add_then_remove_roundtrip() {
        let mut inv = empty_inventory();
        inv.add_item(Item::Block(BlockType::Glass));
        inv.add_item(Item::Block(BlockType::Glass));
        inv.add_item(Item::Block(BlockType::Glass));
        assert_eq!(inv.slots[0], Some((Item::Block(BlockType::Glass), 3, 0)));
        inv.remove_item(0);
        inv.remove_item(0);
        inv.remove_item(0);
        assert_eq!(inv.slots[0], None);
    }

    #[test]
    fn use_tool_decrements_durability() {
        let mut inv = empty_inventory();
        inv.add_item(Item::WoodenPickaxe);
        assert_eq!(inv.slots[0], Some((Item::WoodenPickaxe, 1, 59)));
        assert!(!inv.use_tool(0)); // not broken yet
        assert_eq!(inv.slots[0], Some((Item::WoodenPickaxe, 1, 58)));
    }

    #[test]
    fn use_tool_breaks_at_zero_durability() {
        let mut inv = empty_inventory();
        inv.slots[0] = Some((Item::WoodenPickaxe, 1, 1));
        assert!(inv.use_tool(0)); // tool breaks
        assert_eq!(inv.slots[0], None);
    }

    #[test]
    fn use_tool_does_nothing_for_non_tools() {
        let mut inv = empty_inventory();
        inv.slots[0] = Some((Item::Block(BlockType::Dirt), 5, 0));
        assert!(!inv.use_tool(0));
        assert_eq!(inv.slots[0], Some((Item::Block(BlockType::Dirt), 5, 0)));
    }
}
