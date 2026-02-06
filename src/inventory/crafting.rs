use bevy::prelude::*;

use super::item::Item;
use crate::block::BlockType;

pub const CRAFTING_GRID_SIZE: usize = 2;
pub const CRAFTING_SLOTS: usize = CRAFTING_GRID_SIZE * CRAFTING_GRID_SIZE;

#[derive(Resource)]
pub struct CraftingGrid {
    pub slots: [Option<(Item, u8, u16)>; CRAFTING_SLOTS],
    pub output: Option<(Item, u8, u16)>,
}

impl Default for CraftingGrid {
    fn default() -> Self {
        Self {
            slots: [None; CRAFTING_SLOTS],
            output: None,
        }
    }
}

pub struct CraftingRecipe {
    /// 2x2 pattern - None means empty slot, Some means required item
    pub pattern: [[Option<Item>; CRAFTING_GRID_SIZE]; CRAFTING_GRID_SIZE],
    pub output: Item,
    pub output_count: u8,
}

/// Returns all known 2x2 crafting recipes.
fn recipes() -> Vec<CraftingRecipe> {
    vec![
        // 1 OakLog anywhere -> 4 Planks (shapeless, single item)
        // We represent this as 4 patterns covering each possible position
        CraftingRecipe {
            pattern: [
                [Some(Item::Block(BlockType::OakLog)), None],
                [None, None],
            ],
            output: Item::Block(BlockType::Planks),
            output_count: 4,
        },
        CraftingRecipe {
            pattern: [
                [None, Some(Item::Block(BlockType::OakLog))],
                [None, None],
            ],
            output: Item::Block(BlockType::Planks),
            output_count: 4,
        },
        CraftingRecipe {
            pattern: [
                [None, None],
                [Some(Item::Block(BlockType::OakLog)), None],
            ],
            output: Item::Block(BlockType::Planks),
            output_count: 4,
        },
        CraftingRecipe {
            pattern: [
                [None, None],
                [None, Some(Item::Block(BlockType::OakLog))],
            ],
            output: Item::Block(BlockType::Planks),
            output_count: 4,
        },
        // BirchLog -> 4 Planks (same patterns)
        CraftingRecipe {
            pattern: [
                [Some(Item::Block(BlockType::BirchLog)), None],
                [None, None],
            ],
            output: Item::Block(BlockType::Planks),
            output_count: 4,
        },
        CraftingRecipe {
            pattern: [
                [None, Some(Item::Block(BlockType::BirchLog))],
                [None, None],
            ],
            output: Item::Block(BlockType::Planks),
            output_count: 4,
        },
        CraftingRecipe {
            pattern: [
                [None, None],
                [Some(Item::Block(BlockType::BirchLog)), None],
            ],
            output: Item::Block(BlockType::Planks),
            output_count: 4,
        },
        CraftingRecipe {
            pattern: [
                [None, None],
                [None, Some(Item::Block(BlockType::BirchLog))],
            ],
            output: Item::Block(BlockType::Planks),
            output_count: 4,
        },
        // 2x2 Planks -> 1 CraftingTable
        CraftingRecipe {
            pattern: [
                [Some(Item::Block(BlockType::Planks)), Some(Item::Block(BlockType::Planks))],
                [Some(Item::Block(BlockType::Planks)), Some(Item::Block(BlockType::Planks))],
            ],
            output: Item::Block(BlockType::CraftingTable),
            output_count: 1,
        },
        // 2 Planks vertical (left column) -> 4 Sticks
        CraftingRecipe {
            pattern: [
                [Some(Item::Block(BlockType::Planks)), None],
                [Some(Item::Block(BlockType::Planks)), None],
            ],
            output: Item::Stick,
            output_count: 4,
        },
        // 2 Planks vertical (right column) -> 4 Sticks
        CraftingRecipe {
            pattern: [
                [None, Some(Item::Block(BlockType::Planks))],
                [None, Some(Item::Block(BlockType::Planks))],
            ],
            output: Item::Stick,
            output_count: 4,
        },
        // 2x2 Sand -> 1 Sandstone
        CraftingRecipe {
            pattern: [
                [Some(Item::Block(BlockType::Sand)), Some(Item::Block(BlockType::Sand))],
                [Some(Item::Block(BlockType::Sand)), Some(Item::Block(BlockType::Sand))],
            ],
            output: Item::Block(BlockType::Sandstone),
            output_count: 1,
        },
        // Torch: Coal over Stick, left column
        CraftingRecipe {
            pattern: [
                [Some(Item::Coal), None],
                [Some(Item::Stick), None],
            ],
            output: Item::Block(BlockType::Torch),
            output_count: 4,
        },
        // Torch: Coal over Stick, right column
        CraftingRecipe {
            pattern: [
                [None, Some(Item::Coal)],
                [None, Some(Item::Stick)],
            ],
            output: Item::Block(BlockType::Torch),
            output_count: 4,
        },
    ]
}

/// Format an item option for logging.
fn fmt_item(item: &Option<Item>) -> String {
    match item {
        Some(i) => i.display_name().to_string(),
        None => "_".to_string(),
    }
}

/// Format a slot with count for logging.
fn fmt_slot(slot: &Option<(Item, u8, u16)>) -> String {
    match slot {
        Some((item, count, dur)) => format!("{}x{} (dur={})", item.display_name(), count, dur),
        None => "_".to_string(),
    }
}

/// Check the crafting grid against known recipes and return the output if any match.
pub fn check_recipes(grid: &CraftingGrid) -> Option<(Item, u8, u16)> {
    let current_pattern: [[Option<Item>; CRAFTING_GRID_SIZE]; CRAFTING_GRID_SIZE] = [
        [
            grid.slots[0].map(|(it, _, _)| it),
            grid.slots[1].map(|(it, _, _)| it),
        ],
        [
            grid.slots[2].map(|(it, _, _)| it),
            grid.slots[3].map(|(it, _, _)| it),
        ],
    ];

    debug!(
        "[CRAFT] check_recipes 2x2: [{}, {}] / [{}, {}]",
        fmt_item(&current_pattern[0][0]),
        fmt_item(&current_pattern[0][1]),
        fmt_item(&current_pattern[1][0]),
        fmt_item(&current_pattern[1][1]),
    );

    for recipe in recipes() {
        if recipe.pattern == current_pattern {
            let dur = recipe.output.max_durability();
            debug!(
                "[CRAFT] 2x2 MATCH: {} x{} (dur={})",
                recipe.output.display_name(),
                recipe.output_count,
                dur,
            );
            return Some((recipe.output, recipe.output_count, dur));
        }
    }
    debug!("[CRAFT] 2x2 NO MATCH for grid: [{}, {}] / [{}, {}]",
        fmt_slot(&grid.slots[0]),
        fmt_slot(&grid.slots[1]),
        fmt_slot(&grid.slots[2]),
        fmt_slot(&grid.slots[3]),
    );
    None
}

/// Update the crafting grid output based on current inputs.
pub fn update_crafting_output(mut grid: ResMut<CraftingGrid>) {
    if !grid.is_changed() {
        return;
    }
    let old_output = grid.output;
    grid.output = check_recipes(&grid);
    if old_output != grid.output {
        debug!(
            "[CRAFT] 2x2 output changed: {:?} -> {:?}",
            old_output.map(|(i, c, _)| format!("{}x{}", i.display_name(), c)),
            grid.output.map(|(i, c, _)| format!("{}x{}", i.display_name(), c)),
        );
    }
}

/// Clear the crafting grid, returning items to inventory.
pub fn clear_crafting_grid(grid: &mut CraftingGrid, inventory: &mut crate::inventory::inventory::Inventory) {
    for (idx, slot) in grid.slots.iter_mut().enumerate() {
        if let Some((item, count, _)) = slot.take() {
            debug!(
                "[CRAFT] clear_crafting_grid: returning slot {} -> {}x{} to inventory",
                idx,
                item.display_name(),
                count,
            );
            for _ in 0..count {
                inventory.add_item(item);
            }
        }
    }
    grid.output = None;
    debug!("[CRAFT] clear_crafting_grid: done, output cleared");
}

// === 3x3 Crafting Table Grid ===

pub const CRAFTING_TABLE_SIZE: usize = 3;
pub const CRAFTING_TABLE_SLOTS: usize = CRAFTING_TABLE_SIZE * CRAFTING_TABLE_SIZE;

#[derive(Resource)]
pub struct CraftingTableGrid {
    pub slots: [Option<(Item, u8, u16)>; CRAFTING_TABLE_SLOTS],
    pub output: Option<(Item, u8, u16)>,
}

impl Default for CraftingTableGrid {
    fn default() -> Self {
        Self {
            slots: [None; CRAFTING_TABLE_SLOTS],
            output: None,
        }
    }
}

#[derive(Resource, Default)]
pub struct CraftingTableOpen(pub bool);

pub struct CraftingRecipe3x3 {
    pub pattern: [[Option<Item>; CRAFTING_TABLE_SIZE]; CRAFTING_TABLE_SIZE],
    pub output: Item,
    pub output_count: u8,
}

fn recipes_3x3() -> Vec<CraftingRecipe3x3> {
    let p = Some(Item::Block(BlockType::Planks));
    let s = Some(Item::Stick);
    let c = Some(Item::Block(BlockType::Cobblestone));
    let i = Some(Item::IronIngot);
    let d = Some(Item::Diamond);
    let l = Some(Item::Leather);
    let n: Option<Item> = None;

    vec![
        // Wooden Pickaxe: PPP / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[p, p, p], [n, s, n], [n, s, n]], output: Item::WoodenPickaxe, output_count: 1 },
        // Wooden Axe: PP_ / PS_ / _S_
        CraftingRecipe3x3 { pattern: [[p, p, n], [p, s, n], [n, s, n]], output: Item::WoodenAxe, output_count: 1 },
        // Wooden Axe mirrored: _PP / _SP / _S_
        CraftingRecipe3x3 { pattern: [[n, p, p], [n, s, p], [n, s, n]], output: Item::WoodenAxe, output_count: 1 },
        // Wooden Shovel: _P_ / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[n, p, n], [n, s, n], [n, s, n]], output: Item::WoodenShovel, output_count: 1 },
        // Wooden Sword: _P_ / _P_ / _S_
        CraftingRecipe3x3 { pattern: [[n, p, n], [n, p, n], [n, s, n]], output: Item::WoodenSword, output_count: 1 },
        // Stone Pickaxe: CCC / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[c, c, c], [n, s, n], [n, s, n]], output: Item::StonePickaxe, output_count: 1 },
        // Stone Axe: CC_ / CS_ / _S_
        CraftingRecipe3x3 { pattern: [[c, c, n], [c, s, n], [n, s, n]], output: Item::StoneAxe, output_count: 1 },
        // Stone Axe mirrored: _CC / _SC / _S_
        CraftingRecipe3x3 { pattern: [[n, c, c], [n, s, c], [n, s, n]], output: Item::StoneAxe, output_count: 1 },
        // Stone Shovel: _C_ / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[n, c, n], [n, s, n], [n, s, n]], output: Item::StoneShovel, output_count: 1 },
        // Stone Sword: _C_ / _C_ / _S_
        CraftingRecipe3x3 { pattern: [[n, c, n], [n, c, n], [n, s, n]], output: Item::StoneSword, output_count: 1 },
        // Iron Pickaxe: III / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[i, i, i], [n, s, n], [n, s, n]], output: Item::IronPickaxe, output_count: 1 },
        // Iron Axe: II_ / IS_ / _S_
        CraftingRecipe3x3 { pattern: [[i, i, n], [i, s, n], [n, s, n]], output: Item::IronAxe, output_count: 1 },
        // Iron Axe mirrored: _II / _SI / _S_
        CraftingRecipe3x3 { pattern: [[n, i, i], [n, s, i], [n, s, n]], output: Item::IronAxe, output_count: 1 },
        // Iron Shovel: _I_ / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[n, i, n], [n, s, n], [n, s, n]], output: Item::IronShovel, output_count: 1 },
        // Iron Sword: _I_ / _I_ / _S_
        CraftingRecipe3x3 { pattern: [[n, i, n], [n, i, n], [n, s, n]], output: Item::IronSword, output_count: 1 },
        // Diamond Pickaxe: DDD / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[d, d, d], [n, s, n], [n, s, n]], output: Item::DiamondPickaxe, output_count: 1 },
        // Diamond Axe: DD_ / DS_ / _S_
        CraftingRecipe3x3 { pattern: [[d, d, n], [d, s, n], [n, s, n]], output: Item::DiamondAxe, output_count: 1 },
        // Diamond Axe mirrored: _DD / _SD / _S_
        CraftingRecipe3x3 { pattern: [[n, d, d], [n, s, d], [n, s, n]], output: Item::DiamondAxe, output_count: 1 },
        // Diamond Shovel: _D_ / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[n, d, n], [n, s, n], [n, s, n]], output: Item::DiamondShovel, output_count: 1 },
        // Diamond Sword: _D_ / _D_ / _S_
        CraftingRecipe3x3 { pattern: [[n, d, n], [n, d, n], [n, s, n]], output: Item::DiamondSword, output_count: 1 },
        // Furnace: CCC / C_C / CCC
        CraftingRecipe3x3 { pattern: [[c, c, c], [c, n, c], [c, c, c]], output: Item::Block(BlockType::Furnace), output_count: 1 },
        // Chest: PPP / P_P / PPP
        CraftingRecipe3x3 { pattern: [[p, p, p], [p, n, p], [p, p, p]], output: Item::Block(BlockType::Chest), output_count: 1 },
        // Bed: WWW / PPP / ___ (top-aligned)
        CraftingRecipe3x3 { pattern: [[Some(Item::Wool), Some(Item::Wool), Some(Item::Wool)], [p, p, p], [n, n, n]], output: Item::Block(BlockType::Bed), output_count: 1 },
        // Bed: ___ / WWW / PPP (bottom-aligned)
        CraftingRecipe3x3 { pattern: [[n, n, n], [Some(Item::Wool), Some(Item::Wool), Some(Item::Wool)], [p, p, p]], output: Item::Block(BlockType::Bed), output_count: 1 },
        // Door: PP_ / PP_ / PP_ (left-aligned)
        CraftingRecipe3x3 { pattern: [[p, p, n], [p, p, n], [p, p, n]], output: Item::Block(BlockType::DoorBottom), output_count: 3 },
        // Door: _PP / _PP / _PP (right-aligned)
        CraftingRecipe3x3 { pattern: [[n, p, p], [n, p, p], [n, p, p]], output: Item::Block(BlockType::DoorBottom), output_count: 3 },
        // Torch: Coal over Stick — center column, bottom-aligned
        CraftingRecipe3x3 { pattern: [[n, n, n], [n, Some(Item::Coal), n], [n, s, n]], output: Item::Block(BlockType::Torch), output_count: 4 },
        // Torch: Coal over Stick — left column, bottom-aligned
        CraftingRecipe3x3 { pattern: [[n, n, n], [Some(Item::Coal), n, n], [s, n, n]], output: Item::Block(BlockType::Torch), output_count: 4 },
        // Torch: Coal over Stick — right column, bottom-aligned
        CraftingRecipe3x3 { pattern: [[n, n, n], [n, n, Some(Item::Coal)], [n, n, s]], output: Item::Block(BlockType::Torch), output_count: 4 },
        // Torch: Coal over Stick — center column, top-aligned
        CraftingRecipe3x3 { pattern: [[n, Some(Item::Coal), n], [n, s, n], [n, n, n]], output: Item::Block(BlockType::Torch), output_count: 4 },
        // Torch: Coal over Stick — left column, top-aligned
        CraftingRecipe3x3 { pattern: [[Some(Item::Coal), n, n], [s, n, n], [n, n, n]], output: Item::Block(BlockType::Torch), output_count: 4 },
        // Torch: Coal over Stick — right column, top-aligned
        CraftingRecipe3x3 { pattern: [[n, n, Some(Item::Coal)], [n, n, s], [n, n, n]], output: Item::Block(BlockType::Torch), output_count: 4 },
        // Wooden Hoe: PP_ / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[p, p, n], [n, s, n], [n, s, n]], output: Item::WoodenHoe, output_count: 1 },
        // Wooden Hoe mirrored: _PP / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[n, p, p], [n, s, n], [n, s, n]], output: Item::WoodenHoe, output_count: 1 },
        // Stone Hoe: CC_ / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[c, c, n], [n, s, n], [n, s, n]], output: Item::StoneHoe, output_count: 1 },
        // Stone Hoe mirrored: _CC / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[n, c, c], [n, s, n], [n, s, n]], output: Item::StoneHoe, output_count: 1 },
        // Iron Hoe: II_ / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[i, i, n], [n, s, n], [n, s, n]], output: Item::IronHoe, output_count: 1 },
        // Iron Hoe mirrored: _II / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[n, i, i], [n, s, n], [n, s, n]], output: Item::IronHoe, output_count: 1 },
        // Diamond Hoe: DD_ / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[d, d, n], [n, s, n], [n, s, n]], output: Item::DiamondHoe, output_count: 1 },
        // Diamond Hoe mirrored: _DD / _S_ / _S_
        CraftingRecipe3x3 { pattern: [[n, d, d], [n, s, n], [n, s, n]], output: Item::DiamondHoe, output_count: 1 },
        // === Leather Armor ===
        // Leather Helmet: ___ / LLL / L_L
        CraftingRecipe3x3 { pattern: [[n, n, n], [l, l, l], [l, n, l]], output: Item::LeatherHelmet, output_count: 1 },
        // Leather Chestplate: L_L / LLL / LLL
        CraftingRecipe3x3 { pattern: [[l, n, l], [l, l, l], [l, l, l]], output: Item::LeatherChestplate, output_count: 1 },
        // Leather Leggings: LLL / L_L / L_L
        CraftingRecipe3x3 { pattern: [[l, l, l], [l, n, l], [l, n, l]], output: Item::LeatherLeggings, output_count: 1 },
        // Leather Boots (top-aligned): L_L / L_L / ___
        CraftingRecipe3x3 { pattern: [[l, n, l], [l, n, l], [n, n, n]], output: Item::LeatherBoots, output_count: 1 },
        // Leather Boots (bottom-aligned): ___ / L_L / L_L
        CraftingRecipe3x3 { pattern: [[n, n, n], [l, n, l], [l, n, l]], output: Item::LeatherBoots, output_count: 1 },
        // === Iron Armor ===
        // Iron Helmet: ___ / III / I_I
        CraftingRecipe3x3 { pattern: [[n, n, n], [i, i, i], [i, n, i]], output: Item::IronHelmet, output_count: 1 },
        // Iron Chestplate: I_I / III / III
        CraftingRecipe3x3 { pattern: [[i, n, i], [i, i, i], [i, i, i]], output: Item::IronChestplate, output_count: 1 },
        // Iron Leggings: III / I_I / I_I
        CraftingRecipe3x3 { pattern: [[i, i, i], [i, n, i], [i, n, i]], output: Item::IronLeggings, output_count: 1 },
        // Iron Boots (top-aligned): I_I / I_I / ___
        CraftingRecipe3x3 { pattern: [[i, n, i], [i, n, i], [n, n, n]], output: Item::IronBoots, output_count: 1 },
        // Iron Boots (bottom-aligned): ___ / I_I / I_I
        CraftingRecipe3x3 { pattern: [[n, n, n], [i, n, i], [i, n, i]], output: Item::IronBoots, output_count: 1 },
        // === Diamond Armor ===
        // Diamond Helmet: ___ / DDD / D_D
        CraftingRecipe3x3 { pattern: [[n, n, n], [d, d, d], [d, n, d]], output: Item::DiamondHelmet, output_count: 1 },
        // Diamond Chestplate: D_D / DDD / DDD
        CraftingRecipe3x3 { pattern: [[d, n, d], [d, d, d], [d, d, d]], output: Item::DiamondChestplate, output_count: 1 },
        // Diamond Leggings: DDD / D_D / D_D
        CraftingRecipe3x3 { pattern: [[d, d, d], [d, n, d], [d, n, d]], output: Item::DiamondLeggings, output_count: 1 },
        // Diamond Boots (top-aligned): D_D / D_D / ___
        CraftingRecipe3x3 { pattern: [[d, n, d], [d, n, d], [n, n, n]], output: Item::DiamondBoots, output_count: 1 },
        // Diamond Boots (bottom-aligned): ___ / D_D / D_D
        CraftingRecipe3x3 { pattern: [[n, n, n], [d, n, d], [d, n, d]], output: Item::DiamondBoots, output_count: 1 },
        // Bread: WWW / ___ / ___ (top-aligned)
        CraftingRecipe3x3 { pattern: [[Some(Item::Wheat), Some(Item::Wheat), Some(Item::Wheat)], [n, n, n], [n, n, n]], output: Item::Bread, output_count: 1 },
        // Bread: ___ / WWW / ___ (middle-aligned)
        CraftingRecipe3x3 { pattern: [[n, n, n], [Some(Item::Wheat), Some(Item::Wheat), Some(Item::Wheat)], [n, n, n]], output: Item::Bread, output_count: 1 },
        // Bread: ___ / ___ / WWW (bottom-aligned)
        CraftingRecipe3x3 { pattern: [[n, n, n], [n, n, n], [Some(Item::Wheat), Some(Item::Wheat), Some(Item::Wheat)]], output: Item::Bread, output_count: 1 },
    ]
}

/// Check the 3x3 crafting table grid against known recipes.
pub fn check_recipes_3x3(grid: &CraftingTableGrid) -> Option<(Item, u8, u16)> {
    let current: [[Option<Item>; CRAFTING_TABLE_SIZE]; CRAFTING_TABLE_SIZE] = [
        [
            grid.slots[0].map(|(it, _, _)| it),
            grid.slots[1].map(|(it, _, _)| it),
            grid.slots[2].map(|(it, _, _)| it),
        ],
        [
            grid.slots[3].map(|(it, _, _)| it),
            grid.slots[4].map(|(it, _, _)| it),
            grid.slots[5].map(|(it, _, _)| it),
        ],
        [
            grid.slots[6].map(|(it, _, _)| it),
            grid.slots[7].map(|(it, _, _)| it),
            grid.slots[8].map(|(it, _, _)| it),
        ],
    ];

    debug!(
        "[CRAFT] check_recipes_3x3: [{}, {}, {}] / [{}, {}, {}] / [{}, {}, {}]",
        fmt_item(&current[0][0]), fmt_item(&current[0][1]), fmt_item(&current[0][2]),
        fmt_item(&current[1][0]), fmt_item(&current[1][1]), fmt_item(&current[1][2]),
        fmt_item(&current[2][0]), fmt_item(&current[2][1]), fmt_item(&current[2][2]),
    );

    // Check 3x3 recipes first
    for recipe in recipes_3x3() {
        if recipe.pattern == current {
            let dur = recipe.output.max_durability();
            debug!(
                "[CRAFT] 3x3 MATCH: {} x{} (dur={})",
                recipe.output.display_name(),
                recipe.output_count,
                dur,
            );
            return Some((recipe.output, recipe.output_count, dur));
        }
    }

    debug!("[CRAFT] No 3x3 recipe matched, trying 2x2 sub-grids...");

    // Also check 2x2 recipes in all four positions within the 3x3 grid
    for row_off in 0..2 {
        for col_off in 0..2 {
            // Check if all cells outside the 2x2 sub-grid are empty
            let mut outside_empty = true;
            for r in 0..3 {
                for c in 0..3 {
                    if (r < row_off || r >= row_off + 2 || c < col_off || c >= col_off + 2)
                        && current[r][c].is_some()
                    {
                        outside_empty = false;
                    }
                }
            }
            if !outside_empty {
                debug!(
                    "[CRAFT] 2x2 sub-grid at ({},{}) skipped: outside cells not empty",
                    row_off, col_off,
                );
                continue;
            }

            let sub: [[Option<Item>; CRAFTING_GRID_SIZE]; CRAFTING_GRID_SIZE] = [
                [current[row_off][col_off], current[row_off][col_off + 1]],
                [current[row_off + 1][col_off], current[row_off + 1][col_off + 1]],
            ];

            debug!(
                "[CRAFT] Trying 2x2 sub-grid at ({},{}): [{}, {}] / [{}, {}]",
                row_off, col_off,
                fmt_item(&sub[0][0]), fmt_item(&sub[0][1]),
                fmt_item(&sub[1][0]), fmt_item(&sub[1][1]),
            );

            for recipe in recipes() {
                if recipe.pattern == sub {
                    let dur = recipe.output.max_durability();
                    debug!(
                        "[CRAFT] 2x2-in-3x3 MATCH at ({},{}): {} x{} (dur={})",
                        row_off, col_off,
                        recipe.output.display_name(),
                        recipe.output_count,
                        dur,
                    );
                    return Some((recipe.output, recipe.output_count, dur));
                }
            }
        }
    }

    debug!(
        "[CRAFT] 3x3 NO MATCH. Full grid: [{}] [{}] [{}] [{}] [{}] [{}] [{}] [{}] [{}]",
        fmt_slot(&grid.slots[0]), fmt_slot(&grid.slots[1]), fmt_slot(&grid.slots[2]),
        fmt_slot(&grid.slots[3]), fmt_slot(&grid.slots[4]), fmt_slot(&grid.slots[5]),
        fmt_slot(&grid.slots[6]), fmt_slot(&grid.slots[7]), fmt_slot(&grid.slots[8]),
    );
    None
}

/// Update the 3x3 crafting table grid output.
pub fn update_crafting_table_output(mut grid: ResMut<CraftingTableGrid>) {
    if !grid.is_changed() {
        return;
    }
    let old_output = grid.output;
    grid.output = check_recipes_3x3(&grid);
    if old_output != grid.output {
        debug!(
            "[CRAFT] 3x3 output changed: {:?} -> {:?}",
            old_output.map(|(i, c, _)| format!("{}x{}", i.display_name(), c)),
            grid.output.map(|(i, c, _)| format!("{}x{}", i.display_name(), c)),
        );
    }
}

/// Clear the 3x3 crafting table grid, returning items to inventory.
pub fn clear_crafting_table_grid(grid: &mut CraftingTableGrid, inventory: &mut crate::inventory::inventory::Inventory) {
    for (idx, slot) in grid.slots.iter_mut().enumerate() {
        if let Some((item, count, _)) = slot.take() {
            debug!(
                "[CRAFT] clear_crafting_table_grid: returning slot {} -> {}x{} to inventory",
                idx,
                item.display_name(),
                count,
            );
            for _ in 0..count {
                inventory.add_item(item);
            }
        }
    }
    grid.output = None;
    debug!("[CRAFT] clear_crafting_table_grid: done, output cleared");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_with(slots: [Option<(Item, u8, u16)>; CRAFTING_SLOTS]) -> CraftingGrid {
        CraftingGrid {
            slots,
            output: None,
        }
    }

    fn grid3x3_with(slots: [Option<(Item, u8, u16)>; CRAFTING_TABLE_SLOTS]) -> CraftingTableGrid {
        CraftingTableGrid {
            slots,
            output: None,
        }
    }

    fn s(item: Item) -> Option<(Item, u8, u16)> {
        Some((item, 1, 0))
    }

    fn s_count(item: Item, count: u8) -> Option<(Item, u8, u16)> {
        Some((item, count, 0))
    }

    fn s_dur(item: Item, dur: u16) -> Option<(Item, u8, u16)> {
        Some((item, 1, dur))
    }

    // Shorthand constants
    const N: Option<(Item, u8, u16)> = None;

    fn planks() -> Option<(Item, u8, u16)> { s(Item::Block(BlockType::Planks)) }
    fn stick() -> Option<(Item, u8, u16)> { s(Item::Stick) }
    fn cobble() -> Option<(Item, u8, u16)> { s(Item::Block(BlockType::Cobblestone)) }
    fn iron() -> Option<(Item, u8, u16)> { s(Item::IronIngot) }
    fn diamond() -> Option<(Item, u8, u16)> { s(Item::Diamond) }
    fn coal() -> Option<(Item, u8, u16)> { s(Item::Coal) }
    fn dirt() -> Option<(Item, u8, u16)> { s(Item::Block(BlockType::Dirt)) }
    fn sand() -> Option<(Item, u8, u16)> { s(Item::Block(BlockType::Sand)) }
    fn oak_log() -> Option<(Item, u8, u16)> { s(Item::Block(BlockType::OakLog)) }
    fn birch_log() -> Option<(Item, u8, u16)> { s(Item::Block(BlockType::BirchLog)) }
    fn wool() -> Option<(Item, u8, u16)> { s(Item::Wool) }

    // ===========================
    // 2x2 Recipe Tests (existing)
    // ===========================

    #[test]
    fn oak_log_produces_4_planks_top_left() {
        let grid = grid_with([oak_log(), N, N, N]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Planks), 4, 0)));
    }

    #[test]
    fn oak_log_produces_4_planks_top_right() {
        let grid = grid_with([N, oak_log(), N, N]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Planks), 4, 0)));
    }

    #[test]
    fn oak_log_produces_4_planks_bottom_left() {
        let grid = grid_with([N, N, oak_log(), N]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Planks), 4, 0)));
    }

    #[test]
    fn oak_log_produces_4_planks_bottom_right() {
        let grid = grid_with([N, N, N, oak_log()]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Planks), 4, 0)));
    }

    #[test]
    fn birch_log_produces_planks_all_positions() {
        for pos in 0..4 {
            let mut slots = [N; CRAFTING_SLOTS];
            slots[pos] = birch_log();
            let grid = grid_with(slots);
            let result = check_recipes(&grid);
            assert_eq!(result, Some((Item::Block(BlockType::Planks), 4, 0)),
                "BirchLog in position {} should produce 4 planks", pos);
        }
    }

    #[test]
    fn four_planks_produce_crafting_table() {
        let grid = grid_with([planks(), planks(), planks(), planks()]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::CraftingTable), 1, 0)));
    }

    #[test]
    fn sticks_left_column() {
        let grid = grid_with([planks(), N, planks(), N]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Stick, 4, 0)));
    }

    #[test]
    fn sticks_right_column() {
        let grid = grid_with([N, planks(), N, planks()]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Stick, 4, 0)));
    }

    #[test]
    fn sandstone_from_4_sand() {
        let grid = grid_with([sand(), sand(), sand(), sand()]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Sandstone), 1, 0)));
    }

    #[test]
    fn torch_2x2_left_column() {
        let grid = grid_with([coal(), N, stick(), N]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Torch), 4, 0)));
    }

    #[test]
    fn torch_2x2_right_column() {
        let grid = grid_with([N, coal(), N, stick()]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Torch), 4, 0)));
    }

    #[test]
    fn empty_grid_produces_nothing() {
        let grid = grid_with([N, N, N, N]);
        assert_eq!(check_recipes(&grid), None);
    }

    #[test]
    fn wrong_pattern_produces_nothing() {
        let grid = grid_with([oak_log(), oak_log(), N, N]);
        assert_eq!(check_recipes(&grid), None);
    }

    #[test]
    fn recipe_ignores_stack_count() {
        let grid = grid_with([s_count(Item::Block(BlockType::OakLog), 5), N, N, N]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Planks), 4, 0)));
    }

    #[test]
    fn recipe_ignores_stack_count_64() {
        let grid = grid_with([s_count(Item::Block(BlockType::OakLog), 64), N, N, N]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Planks), 4, 0)));
    }

    #[test]
    fn recipe_ignores_durability() {
        // A durability value on input should not affect matching
        let grid = grid_with([
            Some((Item::Block(BlockType::OakLog), 1, 999)),
            N,
            N,
            N,
        ]);
        let result = check_recipes(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Planks), 4, 0)));
    }

    // ===========================
    // 3x3 Recipe Tests
    // ===========================

    // --- Wooden Tools ---

    #[test]
    fn wooden_pickaxe() {
        // PPP / _S_ / _S_
        let grid = grid3x3_with([
            planks(), planks(), planks(),
            N,        stick(), N,
            N,        stick(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::WoodenPickaxe, 1, 59)));
    }

    #[test]
    fn wooden_axe_left() {
        // PP_ / PS_ / _S_
        let grid = grid3x3_with([
            planks(), planks(), N,
            planks(), stick(), N,
            N,        stick(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::WoodenAxe, 1, 59)));
    }

    #[test]
    fn wooden_axe_mirrored() {
        // _PP / _SP / _S_
        let grid = grid3x3_with([
            N, planks(), planks(),
            N, stick(),  planks(),
            N, stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::WoodenAxe, 1, 59)));
    }

    #[test]
    fn wooden_shovel() {
        // _P_ / _S_ / _S_
        let grid = grid3x3_with([
            N, planks(), N,
            N, stick(),  N,
            N, stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::WoodenShovel, 1, 59)));
    }

    #[test]
    fn wooden_sword() {
        // _P_ / _P_ / _S_
        let grid = grid3x3_with([
            N, planks(), N,
            N, planks(), N,
            N, stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::WoodenSword, 1, 59)));
    }

    // --- Stone Tools ---

    #[test]
    fn stone_pickaxe() {
        let grid = grid3x3_with([
            cobble(), cobble(), cobble(),
            N,        stick(),  N,
            N,        stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::StonePickaxe, 1, 131)));
    }

    #[test]
    fn stone_axe_left() {
        let grid = grid3x3_with([
            cobble(), cobble(), N,
            cobble(), stick(),  N,
            N,        stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::StoneAxe, 1, 131)));
    }

    #[test]
    fn stone_axe_mirrored() {
        let grid = grid3x3_with([
            N, cobble(), cobble(),
            N, stick(),  cobble(),
            N, stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::StoneAxe, 1, 131)));
    }

    #[test]
    fn stone_shovel() {
        let grid = grid3x3_with([
            N, cobble(), N,
            N, stick(),  N,
            N, stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::StoneShovel, 1, 131)));
    }

    #[test]
    fn stone_sword() {
        let grid = grid3x3_with([
            N, cobble(), N,
            N, cobble(), N,
            N, stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::StoneSword, 1, 131)));
    }

    // --- Iron Tools ---

    #[test]
    fn iron_pickaxe() {
        let grid = grid3x3_with([
            iron(), iron(), iron(),
            N,      stick(), N,
            N,      stick(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::IronPickaxe, 1, 250)));
    }

    #[test]
    fn iron_axe_left() {
        let grid = grid3x3_with([
            iron(), iron(), N,
            iron(), stick(), N,
            N,      stick(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::IronAxe, 1, 250)));
    }

    #[test]
    fn iron_axe_mirrored() {
        let grid = grid3x3_with([
            N, iron(), iron(),
            N, stick(), iron(),
            N, stick(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::IronAxe, 1, 250)));
    }

    #[test]
    fn iron_shovel() {
        let grid = grid3x3_with([
            N, iron(), N,
            N, stick(), N,
            N, stick(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::IronShovel, 1, 250)));
    }

    #[test]
    fn iron_sword() {
        let grid = grid3x3_with([
            N, iron(), N,
            N, iron(), N,
            N, stick(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::IronSword, 1, 250)));
    }

    // --- Diamond Tools ---

    #[test]
    fn diamond_pickaxe() {
        let grid = grid3x3_with([
            diamond(), diamond(), diamond(),
            N,         stick(),   N,
            N,         stick(),   N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::DiamondPickaxe, 1, 1561)));
    }

    #[test]
    fn diamond_axe_left() {
        let grid = grid3x3_with([
            diamond(), diamond(), N,
            diamond(), stick(),   N,
            N,         stick(),   N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::DiamondAxe, 1, 1561)));
    }

    #[test]
    fn diamond_axe_mirrored() {
        let grid = grid3x3_with([
            N, diamond(), diamond(),
            N, stick(),   diamond(),
            N, stick(),   N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::DiamondAxe, 1, 1561)));
    }

    #[test]
    fn diamond_shovel() {
        let grid = grid3x3_with([
            N, diamond(), N,
            N, stick(),   N,
            N, stick(),   N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::DiamondShovel, 1, 1561)));
    }

    #[test]
    fn diamond_sword() {
        let grid = grid3x3_with([
            N, diamond(), N,
            N, diamond(), N,
            N, stick(),   N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::DiamondSword, 1, 1561)));
    }

    // --- Furnace, Chest, Bed ---

    #[test]
    fn furnace_recipe() {
        // CCC / C_C / CCC
        let grid = grid3x3_with([
            cobble(), cobble(), cobble(),
            cobble(), N,        cobble(),
            cobble(), cobble(), cobble(),
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Furnace), 1, 0)));
    }

    #[test]
    fn chest_recipe() {
        // PPP / P_P / PPP
        let grid = grid3x3_with([
            planks(), planks(), planks(),
            planks(), N,        planks(),
            planks(), planks(), planks(),
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Chest), 1, 0)));
    }

    #[test]
    fn bed_top_aligned() {
        // WWW / PPP / ___
        let grid = grid3x3_with([
            wool(), wool(), wool(),
            planks(), planks(), planks(),
            N, N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Bed), 1, 0)));
    }

    #[test]
    fn bed_bottom_aligned() {
        // ___ / WWW / PPP
        let grid = grid3x3_with([
            N, N, N,
            wool(), wool(), wool(),
            planks(), planks(), planks(),
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Bed), 1, 0)));
    }

    // --- Torch 3x3 variants (6 total) ---

    #[test]
    fn torch_3x3_center_bottom_aligned() {
        // ___ / _C_ / _S_
        let grid = grid3x3_with([
            N, N, N,
            N, coal(), N,
            N, stick(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Torch), 4, 0)));
    }

    #[test]
    fn torch_3x3_left_bottom_aligned() {
        // ___ / C__ / S__
        let grid = grid3x3_with([
            N, N, N,
            coal(), N, N,
            stick(), N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Torch), 4, 0)));
    }

    #[test]
    fn torch_3x3_right_bottom_aligned() {
        // ___ / __C / __S
        let grid = grid3x3_with([
            N, N, N,
            N, N, coal(),
            N, N, stick(),
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Torch), 4, 0)));
    }

    #[test]
    fn torch_3x3_center_top_aligned() {
        // _C_ / _S_ / ___
        let grid = grid3x3_with([
            N, coal(), N,
            N, stick(), N,
            N, N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Torch), 4, 0)));
    }

    #[test]
    fn torch_3x3_left_top_aligned() {
        // C__ / S__ / ___
        let grid = grid3x3_with([
            coal(), N, N,
            stick(), N, N,
            N, N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Torch), 4, 0)));
    }

    #[test]
    fn torch_3x3_right_top_aligned() {
        // __C / __S / ___
        let grid = grid3x3_with([
            N, N, coal(),
            N, N, stick(),
            N, N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::Torch), 4, 0)));
    }

    // ===========================
    // 2x2 in 3x3 Grid Tests
    // ===========================

    #[test]
    fn oak_log_in_3x3_all_9_positions() {
        // A single oak log in any position of the 3x3 grid should produce planks
        // because 2x2 sub-grid matching should find it
        for pos in 0..9 {
            let mut slots = [N; CRAFTING_TABLE_SLOTS];
            slots[pos] = oak_log();
            let grid = grid3x3_with(slots);
            let result = check_recipes_3x3(&grid);
            assert_eq!(
                result,
                Some((Item::Block(BlockType::Planks), 4, 0)),
                "OakLog in 3x3 position {} should produce 4 planks via 2x2 sub-grid",
                pos
            );
        }
    }

    #[test]
    fn birch_log_in_3x3_all_9_positions() {
        for pos in 0..9 {
            let mut slots = [N; CRAFTING_TABLE_SLOTS];
            slots[pos] = birch_log();
            let grid = grid3x3_with(slots);
            let result = check_recipes_3x3(&grid);
            assert_eq!(
                result,
                Some((Item::Block(BlockType::Planks), 4, 0)),
                "BirchLog in 3x3 position {} should produce 4 planks via 2x2 sub-grid",
                pos
            );
        }
    }

    #[test]
    fn crafting_table_in_3x3_top_left() {
        // 2x2 planks in top-left of 3x3
        let grid = grid3x3_with([
            planks(), planks(), N,
            planks(), planks(), N,
            N, N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::CraftingTable), 1, 0)));
    }

    #[test]
    fn crafting_table_in_3x3_top_right() {
        let grid = grid3x3_with([
            N, planks(), planks(),
            N, planks(), planks(),
            N, N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::CraftingTable), 1, 0)));
    }

    #[test]
    fn crafting_table_in_3x3_bottom_left() {
        let grid = grid3x3_with([
            N, N, N,
            planks(), planks(), N,
            planks(), planks(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::CraftingTable), 1, 0)));
    }

    #[test]
    fn crafting_table_in_3x3_bottom_right() {
        let grid = grid3x3_with([
            N, N, N,
            N, planks(), planks(),
            N, planks(), planks(),
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Block(BlockType::CraftingTable), 1, 0)));
    }

    #[test]
    fn sticks_in_3x3_all_four_positions() {
        // Sticks need planks vertical in left or right column of 2x2
        // Position: top-left (cols 0-1, rows 0-1), left column
        let grid = grid3x3_with([
            planks(), N, N,
            planks(), N, N,
            N, N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Stick, 4, 0)), "Sticks top-left-left-col");

        // Position: top-right (cols 1-2, rows 0-1), left column of sub-grid
        let grid = grid3x3_with([
            N, planks(), N,
            N, planks(), N,
            N, N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Stick, 4, 0)), "Sticks top-right-left-col");

        // Position: bottom-left (cols 0-1, rows 1-2), right column of sub-grid
        let grid = grid3x3_with([
            N, N, N,
            N, planks(), N,
            N, planks(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Stick, 4, 0)), "Sticks bottom-left-right-col");

        // Position: bottom-right (cols 1-2, rows 1-2), right column
        let grid = grid3x3_with([
            N, N, N,
            N, N, planks(),
            N, N, planks(),
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::Stick, 4, 0)), "Sticks bottom-right-right-col");
    }

    #[test]
    fn extra_items_prevent_2x2_recipe_in_3x3() {
        // A single oak log in top-left position, but with extra dirt elsewhere
        // The outside cells are not empty, so 2x2 sub-grid match should fail
        let grid = grid3x3_with([
            oak_log(), N, N,
            N, N, N,
            N, N, dirt(),
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, None, "Extra items outside 2x2 sub-grid should prevent match");
    }

    #[test]
    fn extra_item_in_third_column_blocks_2x2() {
        // Crafting table pattern in top-left 2x2, but extra item in col 2
        let grid = grid3x3_with([
            planks(), planks(), dirt(),
            planks(), planks(), N,
            N, N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, None, "Extra item in col 2 should block 2x2 matching");
    }

    #[test]
    fn extra_item_in_third_row_blocks_2x2() {
        let grid = grid3x3_with([
            planks(), planks(), N,
            planks(), planks(), N,
            dirt(), N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, None, "Extra item in row 2 should block 2x2 matching");
    }

    // ===========================
    // Edge Cases
    // ===========================

    #[test]
    fn empty_3x3_grid_produces_nothing() {
        let grid = grid3x3_with([N; CRAFTING_TABLE_SLOTS]);
        assert_eq!(check_recipes_3x3(&grid), None);
    }

    #[test]
    fn partial_pickaxe_is_hoe() {
        // PP_ / _S_ / _S_ — this is now a valid Wooden Hoe recipe
        let grid = grid3x3_with([
            planks(), planks(), N,
            N,        stick(),  N,
            N,        stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::WoodenHoe, 1, 59)), "PP_/_S_/_S_ should match wooden hoe");
    }

    #[test]
    fn partial_furnace_missing_one_cobble() {
        // CCC / C_C / CC_ (missing bottom-right cobble)
        let grid = grid3x3_with([
            cobble(), cobble(), cobble(),
            cobble(), N,        cobble(),
            cobble(), cobble(), N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, None, "Incomplete furnace should not match");
    }

    #[test]
    fn single_item_wrong_position_no_match() {
        // A stick alone should not match anything
        let grid = grid3x3_with([
            N, N, N,
            N, stick(), N,
            N, N, N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, None, "Single stick should not match any recipe");
    }

    #[test]
    fn full_grid_same_item_no_recipe() {
        // 9 sticks should not match anything
        let grid = grid3x3_with([stick(); CRAFTING_TABLE_SLOTS]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, None, "9 sticks should not match any recipe");
    }

    #[test]
    fn full_grid_dirt_no_recipe() {
        let grid = grid3x3_with([dirt(); CRAFTING_TABLE_SLOTS]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, None, "9 dirt should not match any recipe");
    }

    #[test]
    fn stack_count_does_not_affect_3x3_matching() {
        // Wooden pickaxe with stack count of 5 on each plank and 10 on each stick
        let grid = grid3x3_with([
            s_count(Item::Block(BlockType::Planks), 5),
            s_count(Item::Block(BlockType::Planks), 5),
            s_count(Item::Block(BlockType::Planks), 5),
            N,
            s_count(Item::Stick, 10),
            N,
            N,
            s_count(Item::Stick, 10),
            N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::WoodenPickaxe, 1, 59)),
            "Stack count should not affect recipe matching");
    }

    #[test]
    fn durability_does_not_affect_3x3_matching() {
        // Items with durability values should still match
        let grid = grid3x3_with([
            s_dur(Item::Block(BlockType::Planks), 100),
            s_dur(Item::Block(BlockType::Planks), 200),
            s_dur(Item::Block(BlockType::Planks), 300),
            N,
            s_dur(Item::Stick, 50),
            N,
            N,
            s_dur(Item::Stick, 50),
            N,
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result, Some((Item::WoodenPickaxe, 1, 59)),
            "Durability on input items should not affect recipe matching");
    }

    // ===========================
    // Output Validation
    // ===========================

    #[test]
    fn output_count_4_planks_from_log() {
        let grid = grid_with([oak_log(), N, N, N]);
        let result = check_recipes(&grid);
        assert_eq!(result.unwrap().1, 4, "Should produce exactly 4 planks from 1 log");
    }

    #[test]
    fn output_count_4_sticks_from_2_planks() {
        let grid = grid_with([planks(), N, planks(), N]);
        let result = check_recipes(&grid);
        assert_eq!(result.unwrap().1, 4, "Should produce exactly 4 sticks from 2 planks");
    }

    #[test]
    fn output_count_4_torches() {
        let grid = grid_with([coal(), N, stick(), N]);
        let result = check_recipes(&grid);
        assert_eq!(result.unwrap().1, 4, "Should produce exactly 4 torches");
    }

    #[test]
    fn output_count_1_crafting_table() {
        let grid = grid_with([planks(), planks(), planks(), planks()]);
        let result = check_recipes(&grid);
        assert_eq!(result.unwrap().1, 1, "Should produce exactly 1 crafting table");
    }

    #[test]
    fn output_count_1_furnace() {
        let grid = grid3x3_with([
            cobble(), cobble(), cobble(),
            cobble(), N,        cobble(),
            cobble(), cobble(), cobble(),
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result.unwrap().1, 1, "Should produce exactly 1 furnace");
    }

    #[test]
    fn output_count_1_chest() {
        let grid = grid3x3_with([
            planks(), planks(), planks(),
            planks(), N,        planks(),
            planks(), planks(), planks(),
        ]);
        let result = check_recipes_3x3(&grid);
        assert_eq!(result.unwrap().1, 1, "Should produce exactly 1 chest");
    }

    #[test]
    fn tool_output_has_correct_durability() {
        // Wooden tools: durability = 59
        let grid = grid3x3_with([
            planks(), planks(), planks(),
            N,        stick(),  N,
            N,        stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid).unwrap();
        assert_eq!(result.0, Item::WoodenPickaxe);
        assert_eq!(result.2, 59, "Wooden pickaxe should have durability 59");

        // Stone tools: durability = 131
        let grid = grid3x3_with([
            cobble(), cobble(), cobble(),
            N,        stick(),  N,
            N,        stick(),  N,
        ]);
        let result = check_recipes_3x3(&grid).unwrap();
        assert_eq!(result.0, Item::StonePickaxe);
        assert_eq!(result.2, 131, "Stone pickaxe should have durability 131");

        // Iron tools: durability = 250
        let grid = grid3x3_with([
            iron(), iron(), iron(),
            N,      stick(), N,
            N,      stick(), N,
        ]);
        let result = check_recipes_3x3(&grid).unwrap();
        assert_eq!(result.0, Item::IronPickaxe);
        assert_eq!(result.2, 250, "Iron pickaxe should have durability 250");

        // Diamond tools: durability = 1561
        let grid = grid3x3_with([
            diamond(), diamond(), diamond(),
            N,         stick(),   N,
            N,         stick(),   N,
        ]);
        let result = check_recipes_3x3(&grid).unwrap();
        assert_eq!(result.0, Item::DiamondPickaxe);
        assert_eq!(result.2, 1561, "Diamond pickaxe should have durability 1561");
    }

    #[test]
    fn block_output_has_zero_durability() {
        // Furnace output should have 0 durability
        let grid = grid3x3_with([
            cobble(), cobble(), cobble(),
            cobble(), N,        cobble(),
            cobble(), cobble(), cobble(),
        ]);
        let result = check_recipes_3x3(&grid).unwrap();
        assert_eq!(result.2, 0, "Block output (Furnace) should have 0 durability");

        // Chest output should have 0 durability
        let grid = grid3x3_with([
            planks(), planks(), planks(),
            planks(), N,        planks(),
            planks(), planks(), planks(),
        ]);
        let result = check_recipes_3x3(&grid).unwrap();
        assert_eq!(result.2, 0, "Block output (Chest) should have 0 durability");

        // Planks output should have 0 durability
        let grid = grid_with([oak_log(), N, N, N]);
        let result = check_recipes(&grid).unwrap();
        assert_eq!(result.2, 0, "Block output (Planks) should have 0 durability");
    }

    #[test]
    fn sandstone_2x2_in_3x3_all_positions() {
        // Test sandstone (2x2 sand) in all four positions of 3x3 grid
        // Top-left
        let grid = grid3x3_with([
            sand(), sand(), N,
            sand(), sand(), N,
            N, N, N,
        ]);
        assert_eq!(check_recipes_3x3(&grid), Some((Item::Block(BlockType::Sandstone), 1, 0)),
            "Sandstone top-left");

        // Top-right
        let grid = grid3x3_with([
            N, sand(), sand(),
            N, sand(), sand(),
            N, N, N,
        ]);
        assert_eq!(check_recipes_3x3(&grid), Some((Item::Block(BlockType::Sandstone), 1, 0)),
            "Sandstone top-right");

        // Bottom-left
        let grid = grid3x3_with([
            N, N, N,
            sand(), sand(), N,
            sand(), sand(), N,
        ]);
        assert_eq!(check_recipes_3x3(&grid), Some((Item::Block(BlockType::Sandstone), 1, 0)),
            "Sandstone bottom-left");

        // Bottom-right
        let grid = grid3x3_with([
            N, N, N,
            N, sand(), sand(),
            N, sand(), sand(),
        ]);
        assert_eq!(check_recipes_3x3(&grid), Some((Item::Block(BlockType::Sandstone), 1, 0)),
            "Sandstone bottom-right");
    }

    #[test]
    fn torch_2x2_in_3x3_all_positions() {
        // Torch (coal/stick left column of 2x2) in all four positions of 3x3
        // Top-left
        let grid = grid3x3_with([
            coal(), N, N,
            stick(), N, N,
            N, N, N,
        ]);
        assert_eq!(check_recipes_3x3(&grid), Some((Item::Block(BlockType::Torch), 4, 0)),
            "Torch 2x2 top-left");

        // Top-right: coal/stick in right column of 2x2 sub-grid at (0,1)
        let grid = grid3x3_with([
            N, coal(), N,
            N, stick(), N,
            N, N, N,
        ]);
        // This is left-column of 2x2 sub-grid (0,1), which has coal/stick in its col 0
        assert_eq!(check_recipes_3x3(&grid), Some((Item::Block(BlockType::Torch), 4, 0)),
            "Torch 2x2 top-center");

        // Bottom-left
        let grid = grid3x3_with([
            N, N, N,
            coal(), N, N,
            stick(), N, N,
        ]);
        assert_eq!(check_recipes_3x3(&grid), Some((Item::Block(BlockType::Torch), 4, 0)),
            "Torch 2x2 bottom-left");

        // Bottom-right
        let grid = grid3x3_with([
            N, N, N,
            N, N, coal(),
            N, N, stick(),
        ]);
        assert_eq!(check_recipes_3x3(&grid), Some((Item::Block(BlockType::Torch), 4, 0)),
            "Torch 2x2 bottom-right-col");
    }

    // ===========================
    // Sword/Axe/Shovel for all tiers to ensure completeness
    // ===========================

    #[test]
    fn all_swords_produce_correct_output() {
        let tiers: Vec<(Option<(Item, u8, u16)>, Item, u16)> = vec![
            (planks(),  Item::WoodenSword,  59),
            (cobble(),  Item::StoneSword,   131),
            (iron(),    Item::IronSword,    250),
            (diamond(), Item::DiamondSword, 1561),
        ];
        for (mat, expected_item, expected_dur) in tiers {
            let grid = grid3x3_with([
                N, mat, N,
                N, mat, N,
                N, stick(), N,
            ]);
            let result = check_recipes_3x3(&grid);
            assert_eq!(result, Some((expected_item, 1, expected_dur)),
                "Sword recipe for {:?} should produce {:?}", mat, expected_item);
        }
    }

    #[test]
    fn all_shovels_produce_correct_output() {
        let tiers: Vec<(Option<(Item, u8, u16)>, Item, u16)> = vec![
            (planks(),  Item::WoodenShovel,  59),
            (cobble(),  Item::StoneShovel,   131),
            (iron(),    Item::IronShovel,    250),
            (diamond(), Item::DiamondShovel, 1561),
        ];
        for (mat, expected_item, expected_dur) in tiers {
            let grid = grid3x3_with([
                N, mat, N,
                N, stick(), N,
                N, stick(), N,
            ]);
            let result = check_recipes_3x3(&grid);
            assert_eq!(result, Some((expected_item, 1, expected_dur)),
                "Shovel recipe for {:?} should produce {:?}", mat, expected_item);
        }
    }

    #[test]
    fn all_pickaxes_produce_correct_output() {
        let tiers: Vec<(Option<(Item, u8, u16)>, Item, u16)> = vec![
            (planks(),  Item::WoodenPickaxe,  59),
            (cobble(),  Item::StonePickaxe,   131),
            (iron(),    Item::IronPickaxe,    250),
            (diamond(), Item::DiamondPickaxe, 1561),
        ];
        for (mat, expected_item, expected_dur) in tiers {
            let grid = grid3x3_with([
                mat, mat, mat,
                N, stick(), N,
                N, stick(), N,
            ]);
            let result = check_recipes_3x3(&grid);
            assert_eq!(result, Some((expected_item, 1, expected_dur)),
                "Pickaxe recipe for {:?} should produce {:?}", mat, expected_item);
        }
    }

    #[test]
    fn all_axes_left_produce_correct_output() {
        let tiers: Vec<(Option<(Item, u8, u16)>, Item, u16)> = vec![
            (planks(),  Item::WoodenAxe,  59),
            (cobble(),  Item::StoneAxe,   131),
            (iron(),    Item::IronAxe,    250),
            (diamond(), Item::DiamondAxe, 1561),
        ];
        for (mat, expected_item, expected_dur) in tiers {
            let grid = grid3x3_with([
                mat, mat, N,
                mat, stick(), N,
                N, stick(), N,
            ]);
            let result = check_recipes_3x3(&grid);
            assert_eq!(result, Some((expected_item, 1, expected_dur)),
                "Axe (left) recipe for {:?} should produce {:?}", mat, expected_item);
        }
    }

    #[test]
    fn all_axes_mirrored_produce_correct_output() {
        let tiers: Vec<(Option<(Item, u8, u16)>, Item, u16)> = vec![
            (planks(),  Item::WoodenAxe,  59),
            (cobble(),  Item::StoneAxe,   131),
            (iron(),    Item::IronAxe,    250),
            (diamond(), Item::DiamondAxe, 1561),
        ];
        for (mat, expected_item, expected_dur) in tiers {
            let grid = grid3x3_with([
                N, mat, mat,
                N, stick(), mat,
                N, stick(), N,
            ]);
            let result = check_recipes_3x3(&grid);
            assert_eq!(result, Some((expected_item, 1, expected_dur)),
                "Axe (mirrored) recipe for {:?} should produce {:?}", mat, expected_item);
        }
    }

    // ===========================
    // Regression: wrong material combos should not match
    // ===========================

    #[test]
    fn mixed_materials_no_match() {
        // Pickaxe with mixed materials (planks + cobblestone) should not match
        let grid = grid3x3_with([
            planks(), cobble(), planks(),
            N,        stick(),  N,
            N,        stick(),  N,
        ]);
        assert_eq!(check_recipes_3x3(&grid), None,
            "Mixed materials should not match any pickaxe recipe");
    }

    #[test]
    fn sword_with_wrong_handle() {
        // Sword shape but with planks handle instead of stick
        let grid = grid3x3_with([
            N, cobble(), N,
            N, cobble(), N,
            N, planks(), N,
        ]);
        assert_eq!(check_recipes_3x3(&grid), None,
            "Sword with planks handle should not match");
    }

    #[test]
    fn pickaxe_upside_down_no_match() {
        // Pickaxe pattern inverted vertically should not match
        let grid = grid3x3_with([
            N,        stick(),  N,
            N,        stick(),  N,
            planks(), planks(), planks(),
        ]);
        assert_eq!(check_recipes_3x3(&grid), None,
            "Upside-down pickaxe should not match");
    }
}
