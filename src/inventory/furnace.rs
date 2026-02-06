use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::item::Item;
use crate::block::BlockType;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FurnaceData {
    pub input: Option<(Item, u8, u16)>,
    pub fuel: Option<(Item, u8, u16)>,
    pub output: Option<(Item, u8, u16)>,
    pub progress: f32,
    pub fuel_remaining: f32,
    pub fuel_max: f32,
}

impl Default for FurnaceData {
    fn default() -> Self {
        Self {
            input: None,
            fuel: None,
            output: None,
            progress: 0.0,
            fuel_remaining: 0.0,
            fuel_max: 0.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct Furnaces {
    pub data: HashMap<IVec3, FurnaceData>,
}

#[derive(Resource, Default)]
pub struct FurnaceOpen(pub Option<IVec3>);

const SMELT_TIME: f32 = 10.0;

pub fn smelting_result(input: Item) -> Option<Item> {
    match input {
        Item::Block(BlockType::Cobblestone) => Some(Item::Block(BlockType::Stone)),
        Item::Block(BlockType::Sand) => Some(Item::Block(BlockType::Glass)),
        Item::Block(BlockType::IronOre) => Some(Item::IronIngot),
        Item::Block(BlockType::GoldOre) => Some(Item::GoldIngot),
        Item::Block(BlockType::CoalOre) => Some(Item::Coal),
        Item::RawPorkchop => Some(Item::CookedPorkchop),
        Item::RawBeef => Some(Item::CookedBeef),
        Item::RawMutton => Some(Item::CookedMutton),
        _ => None,
    }
}

pub fn fuel_value(item: Item) -> f32 {
    match item {
        Item::Block(BlockType::OakLog) | Item::Block(BlockType::BirchLog) => 15.0,
        Item::Block(BlockType::Planks) => 15.0,
        Item::Block(BlockType::CraftingTable) => 15.0,
        Item::Stick => 5.0,
        Item::Coal => 80.0,
        _ => 0.0,
    }
}

pub fn furnace_tick(time: Res<Time>, mut furnaces: ResMut<Furnaces>) {
    let dt = time.delta_secs();

    for (_pos, data) in furnaces.data.iter_mut() {
        // Check if we have a valid input to smelt
        let can_smelt = if let Some((input_item, _, _)) = data.input {
            if let Some(result) = smelting_result(input_item) {
                // Check output slot can accept the result
                match &data.output {
                    None => true,
                    Some((out_item, count, _)) => *out_item == result && *count < 64,
                }
            } else {
                false
            }
        } else {
            false
        };

        if !can_smelt {
            // Reset progress if we can't smelt
            data.progress = 0.0;
            continue;
        }

        // Try to consume fuel if none is burning
        if data.fuel_remaining <= 0.0 {
            if let Some((fuel_item, fuel_count, _)) = &mut data.fuel {
                let fv = fuel_value(*fuel_item);
                if fv > 0.0 {
                    data.fuel_remaining = fv;
                    data.fuel_max = fv;
                    if *fuel_count > 1 {
                        *fuel_count -= 1;
                    } else {
                        data.fuel = None;
                    }
                } else {
                    data.progress = 0.0;
                    continue;
                }
            } else {
                data.progress = 0.0;
                continue;
            }
        }

        // Burn fuel
        data.fuel_remaining -= dt;

        // Advance smelting progress
        data.progress += dt / SMELT_TIME;

        if data.progress >= 1.0 {
            data.progress = 0.0;

            // Consume one input
            let input_item = data.input.as_ref().map(|(it, _, _)| *it).unwrap_or(Item::Block(BlockType::Air));
            let result = smelting_result(input_item);

            if let Some((_, count, _)) = &mut data.input {
                if *count > 1 {
                    *count -= 1;
                } else {
                    data.input = None;
                }
            }

            // Add to output
            if let Some(result) = result {
                if let Some((_, count, _)) = &mut data.output {
                    *count += 1;
                } else {
                    data.output = Some((result, 1, 0));
                }
            }
        }
    }
}
