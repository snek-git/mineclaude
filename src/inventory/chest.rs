use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::item::Item;

pub const CHEST_SLOTS: usize = 27; // 3 rows x 9 cols

pub type ChestSlot = Option<(Item, u8, u16)>;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ChestData {
    pub slots: [ChestSlot; CHEST_SLOTS],
}

#[derive(Resource, Default)]
pub struct ChestStore {
    pub data: HashMap<IVec3, ChestData>,
}

#[derive(Resource, Default)]
pub struct ChestOpen(pub Option<IVec3>);
