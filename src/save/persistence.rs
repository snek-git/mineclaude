use bevy::prelude::*;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::inventory::chest::{ChestData, ChestStore};
use crate::inventory::furnace::{FurnaceData, Furnaces};
use crate::inventory::item::Item;
use crate::world::chunk::Chunk;
use crate::world::manager::{ChunkDataStore, CropTracker, SaplingTracker};

fn chunk_path(pos: IVec3) -> PathBuf {
    PathBuf::from(format!(
        "saves/world/chunk_{}_{}_{}.bin",
        pos.x, pos.y, pos.z
    ))
}

pub fn save_chunk(pos: IVec3, chunk: &Chunk) -> Result<(), Box<dyn std::error::Error>> {
    let path = chunk_path(pos);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let encoded = bincode::serialize(chunk)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&encoded)?;
    let compressed = encoder.finish()?;
    fs::write(path, compressed)?;
    Ok(())
}

pub fn load_chunk(pos: IVec3) -> Option<Chunk> {
    let path = chunk_path(pos);
    let data = fs::read(&path).ok()?;
    let mut decoder = GzDecoder::new(&data[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).ok()?;
    bincode::deserialize(&decompressed).ok()
}

pub fn save_modified_chunks(mut store: ResMut<ChunkDataStore>) {
    if store.modified.is_empty() {
        return;
    }

    let positions: Vec<IVec3> = store.modified.drain().collect();
    for pos in positions {
        if let Some(chunk) = store.chunks.get(&pos) {
            if let Err(e) = save_chunk(pos, chunk) {
                warn!("Failed to save chunk {:?}: {}", pos, e);
            }
        }
    }
    info!("World saved");
}

#[derive(Serialize, Deserialize)]
pub struct PlayerSaveData {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub air_supply: f32,
    pub inventory: Vec<Option<(Item, u8, u16)>>,
    #[serde(default)]
    pub spawn_x: Option<f32>,
    #[serde(default)]
    pub spawn_y: Option<f32>,
    #[serde(default)]
    pub spawn_z: Option<f32>,
    #[serde(default)]
    pub food_level: Option<f32>,
    #[serde(default)]
    pub saturation: Option<f32>,
    #[serde(default)]
    pub armor_slots: Option<Vec<Option<(Item, u8, u16)>>>,
}

const PLAYER_SAVE_PATH: &str = "saves/player.json";

pub fn save_player(data: &PlayerSaveData) -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from("saves");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    let json = serde_json::to_string_pretty(data)?;
    fs::write(PLAYER_SAVE_PATH, json)?;
    Ok(())
}

pub fn load_player() -> Option<PlayerSaveData> {
    let data = fs::read_to_string(PLAYER_SAVE_PATH).ok()?;
    serde_json::from_str(&data).ok()
}

// --- Chest persistence ---

const CHESTS_SAVE_PATH: &str = "saves/chests.json";

pub fn save_chests(store: &ChestStore) -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from("saves");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    let entries: Vec<([i32; 3], &ChestData)> = store
        .data
        .iter()
        .map(|(pos, data)| ([pos.x, pos.y, pos.z], data))
        .collect();
    let json = serde_json::to_string_pretty(&entries)?;
    fs::write(CHESTS_SAVE_PATH, json)?;
    Ok(())
}

pub fn load_chests() -> ChestStore {
    let data = match fs::read_to_string(CHESTS_SAVE_PATH) {
        Ok(d) => d,
        Err(_) => return ChestStore::default(),
    };
    let entries: Vec<([i32; 3], ChestData)> = match serde_json::from_str(&data) {
        Ok(e) => e,
        Err(e) => {
            warn!("Failed to load chest data: {}", e);
            return ChestStore::default();
        }
    };
    let mut map = HashMap::new();
    for ([x, y, z], chest) in entries {
        map.insert(IVec3::new(x, y, z), chest);
    }
    ChestStore { data: map }
}

// --- Furnace persistence ---

const FURNACES_SAVE_PATH: &str = "saves/furnaces.json";

pub fn save_furnaces(furnaces: &Furnaces) -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from("saves");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    let entries: Vec<([i32; 3], &FurnaceData)> = furnaces
        .data
        .iter()
        .map(|(pos, data)| ([pos.x, pos.y, pos.z], data))
        .collect();
    let json = serde_json::to_string_pretty(&entries)?;
    fs::write(FURNACES_SAVE_PATH, json)?;
    Ok(())
}

pub fn load_furnaces() -> Furnaces {
    let data = match fs::read_to_string(FURNACES_SAVE_PATH) {
        Ok(d) => d,
        Err(_) => return Furnaces::default(),
    };
    let entries: Vec<([i32; 3], FurnaceData)> = match serde_json::from_str(&data) {
        Ok(e) => e,
        Err(e) => {
            warn!("Failed to load furnace data: {}", e);
            return Furnaces::default();
        }
    };
    let mut map = HashMap::new();
    for ([x, y, z], furnace) in entries {
        map.insert(IVec3::new(x, y, z), furnace);
    }
    Furnaces { data: map }
}

// --- Sapling tracker persistence ---

const SAPLINGS_SAVE_PATH: &str = "saves/saplings.json";

pub fn save_saplings(tracker: &SaplingTracker) -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from("saves");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    let entries: Vec<([i32; 3], f32)> = tracker
        .saplings
        .iter()
        .map(|(pos, time)| ([pos.x, pos.y, pos.z], *time))
        .collect();
    let json = serde_json::to_string_pretty(&entries)?;
    fs::write(SAPLINGS_SAVE_PATH, json)?;
    Ok(())
}

pub fn load_saplings() -> SaplingTracker {
    let data = match fs::read_to_string(SAPLINGS_SAVE_PATH) {
        Ok(d) => d,
        Err(_) => return SaplingTracker::default(),
    };
    let entries: Vec<([i32; 3], f32)> = match serde_json::from_str(&data) {
        Ok(e) => e,
        Err(e) => {
            warn!("Failed to load sapling data: {}", e);
            return SaplingTracker::default();
        }
    };
    let mut saplings = HashMap::new();
    for ([x, y, z], time) in entries {
        saplings.insert(IVec3::new(x, y, z), time);
    }
    SaplingTracker {
        saplings,
        scanned_chunks: HashSet::new(),
    }
}

// --- Crop tracker persistence ---

const CROPS_SAVE_PATH: &str = "saves/crops.json";

pub fn save_crops(tracker: &CropTracker) -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from("saves");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    let entries: Vec<([i32; 3], f32)> = tracker
        .crops
        .iter()
        .map(|(pos, time)| ([pos.x, pos.y, pos.z], *time))
        .collect();
    let json = serde_json::to_string_pretty(&entries)?;
    fs::write(CROPS_SAVE_PATH, json)?;
    Ok(())
}

pub fn load_crops() -> CropTracker {
    let data = match fs::read_to_string(CROPS_SAVE_PATH) {
        Ok(d) => d,
        Err(_) => return CropTracker::default(),
    };
    let entries: Vec<([i32; 3], f32)> = match serde_json::from_str(&data) {
        Ok(e) => e,
        Err(e) => {
            warn!("Failed to load crop data: {}", e);
            return CropTracker::default();
        }
    };
    let mut crops = HashMap::new();
    for ([x, y, z], time) in entries {
        crops.insert(IVec3::new(x, y, z), time);
    }
    CropTracker {
        crops,
        scanned_chunks: HashSet::new(),
    }
}
