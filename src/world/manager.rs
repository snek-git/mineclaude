use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use std::collections::{HashMap, HashSet};
use crate::block::BlockType;

use super::chunk::{Chunk, CHUNK_SIZE};
use super::coordinates::world_to_chunk_pos;
use super::generation::{generate_chunk, set_world_seed};
use super::material::{AtlasTileMaterial, ChunkMaterialType};
use super::meshing::{build_chunk_mesh, NeighborChunks};
use super::WorldSeed;

const RENDER_DISTANCE: i32 = 16;
const DESPAWN_DISTANCE: i32 = 18;
const MAX_LOADS_PER_FRAME: usize = 8;
const WORLD_HEIGHT_CHUNKS: i32 = 16;

/// Tracks which chunk positions have spawned entities.
#[derive(Resource, Default)]
pub struct ChunkManager {
    pub loaded: HashMap<IVec3, Entity>,
}

/// Stores generated chunk data for neighbor lookups and future use.
#[derive(Resource, Default)]
pub struct ChunkDataStore {
    pub chunks: HashMap<IVec3, Chunk>,
    pub modified: HashSet<IVec3>,
}

/// Shared material handle for all chunk meshes (extended with atlas tiling shader).
#[derive(Resource)]
pub struct ChunkMaterial(pub Handle<ChunkMaterialType>);

/// Marker: this entity needs its mesh built.
#[derive(Component)]
pub struct NeedsMesh;

/// Holds an in-flight async mesh task.
#[derive(Component)]
pub struct MeshTask(Task<Mesh>);

/// Marker for the chunk entity's position in chunk coordinates.
#[derive(Component)]
pub struct ChunkCoord(pub IVec3);

/// Minimum time (seconds) before a sapling can grow.
pub const SAPLING_GROW_MIN: f32 = 60.0;
/// Maximum time (seconds) before a sapling grows.
pub const SAPLING_GROW_MAX: f32 = 180.0;
/// Minimum air blocks above sapling needed for tree growth.
const TREE_SPACE_REQUIRED: i32 = 6;
/// Leaf canopy radius for trees grown from saplings.
const TREE_CANOPY_RADIUS: i32 = 2;

/// Tracks placed saplings and their growth timers.
#[derive(Resource, Default)]
pub struct SaplingTracker {
    /// Maps world position -> remaining time until growth attempt.
    pub saplings: HashMap<IVec3, f32>,
    /// Chunk positions already scanned for saplings (prevents re-scanning).
    pub scanned_chunks: HashSet<IVec3>,
}

/// Minimum time (seconds) before a crop stage advances.
pub const CROP_GROW_MIN: f32 = 20.0;
/// Maximum time (seconds) before a crop stage advances.
pub const CROP_GROW_MAX: f32 = 40.0;

/// Tracks planted crops and their growth timers.
#[derive(Resource, Default)]
pub struct CropTracker {
    /// Maps world position -> remaining time until next growth stage.
    pub crops: HashMap<IVec3, f32>,
    /// Chunk positions already scanned for crops (prevents re-scanning).
    pub scanned_chunks: HashSet<IVec3>,
}

/// One-time setup: camera, light, and shared material.
pub fn setup_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ChunkMaterialType>>,
    world_seed: Res<WorldSeed>,
) {
    // Initialize terrain noise with the world seed
    set_world_seed(world_seed.0);

    // Shared chunk material with atlas tiling shader for greedy meshing
    let texture: Handle<Image> = asset_server.load("textures/atlas.png");
    let material = materials.add(bevy::pbr::ExtendedMaterial {
        base: StandardMaterial {
            base_color_texture: Some(texture),
            perceptual_roughness: 1.0,
            reflectance: 0.1,
            alpha_mode: AlphaMode::Opaque,
            ..default()
        },
        extension: AtlasTileMaterial {},
    });
    commands.insert_resource(ChunkMaterial(material));
    commands.init_resource::<ChunkManager>();
    commands.init_resource::<ChunkDataStore>();
    commands.insert_resource(crate::save::persistence::load_saplings());
    commands.insert_resource(crate::save::persistence::load_crops());
}

/// Load/unload chunks based on camera position.
pub fn update_chunk_loading(
    mut manager: ResMut<ChunkManager>,
    mut store: ResMut<ChunkDataStore>,
    mut tracker: ResMut<SaplingTracker>,
    mut crop_tracker: ResMut<CropTracker>,
    camera: Query<&Transform, With<Camera3d>>,
    mut commands: Commands,
) {
    let Ok(cam_transform) = camera.single() else {
        return;
    };
    let cam_chunk = world_to_chunk_pos(cam_transform.translation);

    // Despawn distant chunks
    let to_despawn: Vec<IVec3> = manager
        .loaded
        .keys()
        .filter(|pos| {
            let dx = (pos.x - cam_chunk.x).abs();
            let dz = (pos.z - cam_chunk.z).abs();
            dx > DESPAWN_DISTANCE || dz > DESPAWN_DISTANCE
        })
        .copied()
        .collect();

    for pos in to_despawn {
        if let Some(entity) = manager.loaded.remove(&pos) {
            commands.entity(entity).despawn();
        }
        if store.modified.contains(&pos) {
            if let Some(chunk) = store.chunks.get(&pos) {
                let _ = crate::save::persistence::save_chunk(pos, chunk);
            }
            store.modified.remove(&pos);
        }
        store.chunks.remove(&pos);
        // Keep sapling/crop tracker entries in memory — they're tiny (IVec3 + f32 each).
        // Only mark as unscanned so re-scan on reload doesn't create duplicates via or_insert.
        tracker.scanned_chunks.remove(&pos);
        crop_tracker.scanned_chunks.remove(&pos);
    }

    // Collect positions to load, sorted by distance (closest first)
    let mut to_load: Vec<(IVec3, i32)> = Vec::new();
    for x in (cam_chunk.x - RENDER_DISTANCE)..=(cam_chunk.x + RENDER_DISTANCE) {
        for z in (cam_chunk.z - RENDER_DISTANCE)..=(cam_chunk.z + RENDER_DISTANCE) {
            let dx = (x - cam_chunk.x).abs();
            let dz = (z - cam_chunk.z).abs();
            if dx > RENDER_DISTANCE || dz > RENDER_DISTANCE {
                continue;
            }
            for y in 0..WORLD_HEIGHT_CHUNKS {
                let pos = IVec3::new(x, y, z);
                if !manager.loaded.contains_key(&pos) {
                    let dist_sq = dx * dx + dz * dz;
                    to_load.push((pos, dist_sq));
                }
            }
        }
    }
    to_load.sort_by_key(|&(_, d)| d);

    // Spawn up to MAX_LOADS_PER_FRAME chunks
    let mut loaded = 0;
    for (pos, _) in to_load {
        if loaded >= MAX_LOADS_PER_FRAME {
            break;
        }
        if manager.loaded.contains_key(&pos) {
            continue;
        }

        // Try loading from disk first, otherwise generate
        let chunk = crate::save::persistence::load_chunk(pos)
            .unwrap_or_else(|| generate_chunk(pos));
        store.chunks.insert(pos, chunk);

        // Spawn entity with NeedsMesh marker
        let world_pos = Vec3::new(
            (pos.x * CHUNK_SIZE as i32) as f32,
            (pos.y * CHUNK_SIZE as i32) as f32,
            (pos.z * CHUNK_SIZE as i32) as f32,
        );
        let entity = commands
            .spawn((
                ChunkCoord(pos),
                NeedsMesh,
                Transform::from_translation(world_pos),
                Visibility::default(),
            ))
            .id();
        manager.loaded.insert(pos, entity);

        // Scan newly loaded chunk for saplings and crops
        if !tracker.scanned_chunks.contains(&pos) {
            tracker.scanned_chunks.insert(pos);
            if let Some(chunk) = store.chunks.get(&pos) {
                let size = CHUNK_SIZE as i32;
                let base_x = pos.x * size;
                let base_y = pos.y * size;
                let base_z = pos.z * size;
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        for x in 0..CHUNK_SIZE {
                            let block = chunk.get(x, y, z);
                            if block == BlockType::OakSapling || block == BlockType::BirchSapling {
                                let world_pos = IVec3::new(
                                    base_x + x as i32,
                                    base_y + y as i32,
                                    base_z + z as i32,
                                );
                                tracker.saplings.entry(world_pos).or_insert_with(|| {
                                    SAPLING_GROW_MIN + rand::random::<f32>() * (SAPLING_GROW_MAX - SAPLING_GROW_MIN)
                                });
                            }
                        }
                    }
                }
            }
        }
        if !crop_tracker.scanned_chunks.contains(&pos) {
            crop_tracker.scanned_chunks.insert(pos);
            if let Some(chunk) = store.chunks.get(&pos) {
                let size = CHUNK_SIZE as i32;
                let base_x = pos.x * size;
                let base_y = pos.y * size;
                let base_z = pos.z * size;
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        for x in 0..CHUNK_SIZE {
                            let block = chunk.get(x, y, z);
                            if matches!(block, BlockType::WheatStage0 | BlockType::WheatStage1 | BlockType::WheatStage2) {
                                let world_pos = IVec3::new(
                                    base_x + x as i32,
                                    base_y + y as i32,
                                    base_z + z as i32,
                                );
                                crop_tracker.crops.entry(world_pos).or_insert_with(|| {
                                    CROP_GROW_MIN + rand::random::<f32>() * (CROP_GROW_MAX - CROP_GROW_MIN)
                                });
                            }
                        }
                    }
                }
            }
        }

        loaded += 1;
    }
}

/// For chunks with NeedsMesh, spawn async mesh tasks.
pub fn start_mesh_tasks(
    mut commands: Commands,
    chunks: Query<(Entity, &ChunkCoord), With<NeedsMesh>>,
    store: Res<ChunkDataStore>,
) {
    let task_pool = AsyncComputeTaskPool::get();

    for (entity, coord) in &chunks {
        let Some(chunk) = store.chunks.get(&coord.0) else {
            continue;
        };
        let chunk_clone = chunk.clone();

        // Look up the 6 neighbor chunks: [+X, -X, +Y, -Y, +Z, -Z]
        let offsets = [
            IVec3::X,   // +X (East)
            IVec3::NEG_X, // -X (West)
            IVec3::Y,   // +Y (Top)
            IVec3::NEG_Y, // -Y (Bottom)
            IVec3::Z,   // +Z (South)
            IVec3::NEG_Z, // -Z (North)
        ];
        let neighbor_chunks: [Option<Chunk>; 6] = std::array::from_fn(|i| {
            store.chunks.get(&(coord.0 + offsets[i])).cloned()
        });

        let task = task_pool.spawn(async move {
            let neighbors: NeighborChunks = std::array::from_fn(|i| {
                neighbor_chunks[i].as_ref()
            });
            build_chunk_mesh(&chunk_clone, &neighbors)
        });

        commands
            .entity(entity)
            .remove::<NeedsMesh>()
            .insert(MeshTask(task));
    }
}

/// Poll completed mesh tasks and insert mesh + material components.
pub fn apply_mesh_results(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    material: Res<ChunkMaterial>,
    mut tasks: Query<(Entity, &mut MeshTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mesh) = block_on(poll_once(&mut task.0)) {
            commands
                .entity(entity)
                .remove::<MeshTask>()
                .insert((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d::<ChunkMaterialType>(material.0.clone()),
                ));
        }
    }
}

/// Helper: read a block from the chunk data store at a world position.
fn get_block_at(store: &ChunkDataStore, pos: IVec3) -> BlockType {
    let size = CHUNK_SIZE as i32;
    let chunk_pos = IVec3::new(
        pos.x.div_euclid(size),
        pos.y.div_euclid(size),
        pos.z.div_euclid(size),
    );
    let Some(chunk) = store.chunks.get(&chunk_pos) else {
        return BlockType::Air;
    };
    let lx = pos.x.rem_euclid(size) as usize;
    let ly = pos.y.rem_euclid(size) as usize;
    let lz = pos.z.rem_euclid(size) as usize;
    chunk.get(lx, ly, lz)
}

/// Helper: set a block in the chunk data store at a world position.
fn set_block_at(store: &mut ChunkDataStore, pos: IVec3, block: BlockType) {
    let size = CHUNK_SIZE as i32;
    let chunk_pos = IVec3::new(
        pos.x.div_euclid(size),
        pos.y.div_euclid(size),
        pos.z.div_euclid(size),
    );
    if let Some(chunk) = store.chunks.get_mut(&chunk_pos) {
        let lx = pos.x.rem_euclid(size) as usize;
        let ly = pos.y.rem_euclid(size) as usize;
        let lz = pos.z.rem_euclid(size) as usize;
        chunk.set(lx, ly, lz, block);
        store.modified.insert(chunk_pos);
    }
}

/// Helper: mark a world position's chunk (and boundary neighbors) for remeshing.
fn mark_remesh(pos: IVec3, manager: &ChunkManager, commands: &mut Commands) {
    let chunk_pos = world_to_chunk_pos(pos.as_vec3());
    if let Some(&entity) = manager.loaded.get(&chunk_pos) {
        commands.entity(entity).insert(NeedsMesh);
    }
}

/// System: tick sapling growth timers and attempt to grow mature saplings into trees.
/// Saplings are tracked event-driven: added on place (place_block) and chunk load (update_chunk_loading).
pub fn update_sapling_growth(
    time: Res<Time>,
    mut tracker: ResMut<SaplingTracker>,
    mut store: ResMut<ChunkDataStore>,
    manager: Res<ChunkManager>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    // Remove tracked saplings that are no longer saplings (broken/replaced)
    tracker.saplings.retain(|pos, _| {
        let block = get_block_at(&store, *pos);
        block == BlockType::OakSapling || block == BlockType::BirchSapling
    });

    // Tick timers and collect saplings ready to grow
    let mut ready: Vec<(IVec3, BlockType)> = Vec::new();
    for (pos, timer) in tracker.saplings.iter_mut() {
        *timer -= dt;
        if *timer <= 0.0 {
            let block = get_block_at(&store, *pos);
            if block == BlockType::OakSapling || block == BlockType::BirchSapling {
                ready.push((*pos, block));
            }
        }
    }

    // Attempt growth for each ready sapling
    let size = CHUNK_SIZE as i32;
    for (pos, sapling_type) in &ready {
        let (log_type, leaf_type, trunk_height) = match sapling_type {
            BlockType::OakSapling => {
                let h = 5 + (rand::random::<u32>() % 2) as i32;
                (BlockType::OakLog, BlockType::OakLeaves, h)
            }
            BlockType::BirchSapling => {
                let h = 5 + (rand::random::<u32>() % 3) as i32;
                (BlockType::BirchLog, BlockType::BirchLeaves, h)
            }
            _ => continue,
        };

        // Check all chunks the tree would touch are loaded
        let tree_min = IVec3::new(pos.x - TREE_CANOPY_RADIUS, pos.y, pos.z - TREE_CANOPY_RADIUS);
        let tree_max = IVec3::new(pos.x + TREE_CANOPY_RADIUS, pos.y + trunk_height + 1, pos.z + TREE_CANOPY_RADIUS);
        let chunk_min = IVec3::new(
            tree_min.x.div_euclid(size),
            tree_min.y.div_euclid(size),
            tree_min.z.div_euclid(size),
        );
        let chunk_max = IVec3::new(
            tree_max.x.div_euclid(size),
            tree_max.y.div_euclid(size),
            tree_max.z.div_euclid(size),
        );
        let mut all_loaded = true;
        'outer: for cy in chunk_min.y..=chunk_max.y {
            for cz in chunk_min.z..=chunk_max.z {
                for cx in chunk_min.x..=chunk_max.x {
                    if !store.chunks.contains_key(&IVec3::new(cx, cy, cz)) {
                        all_loaded = false;
                        break 'outer;
                    }
                }
            }
        }
        if !all_loaded {
            // Retry later when chunks may be loaded
            if let Some(timer) = tracker.saplings.get_mut(pos) {
                *timer = SAPLING_GROW_MIN + rand::random::<f32>() * (SAPLING_GROW_MAX - SAPLING_GROW_MIN);
            }
            continue;
        }

        // Check space: need TREE_SPACE_REQUIRED air blocks above
        let mut has_space = true;
        for dy in 1..=TREE_SPACE_REQUIRED {
            let check_pos = *pos + IVec3::new(0, dy, 0);
            let block = get_block_at(&store, check_pos);
            if block != BlockType::Air && block != BlockType::OakSapling && block != BlockType::BirchSapling {
                has_space = false;
                break;
            }
        }
        if !has_space {
            // Can't grow, reset timer for another attempt later
            if let Some(timer) = tracker.saplings.get_mut(pos) {
                *timer = SAPLING_GROW_MIN + rand::random::<f32>() * (SAPLING_GROW_MAX - SAPLING_GROW_MIN);
            }
            continue;
        }

        // Grow the tree: place trunk
        set_block_at(&mut store, *pos, log_type); // replace sapling with log base
        mark_remesh(*pos, &manager, &mut commands);
        for dy in 1..=trunk_height {
            let trunk_pos = *pos + IVec3::new(0, dy, 0);
            set_block_at(&mut store, trunk_pos, log_type);
            mark_remesh(trunk_pos, &manager, &mut commands);
        }

        // Place leaves (radius 2 for both oak and birch, narrower at top)
        let leaf_start = trunk_height - 2;
        for dy_offset in leaf_start..=trunk_height + 1 {
            let wy = pos.y + dy_offset;
            let radius: i32 = if dy_offset >= trunk_height {
                1
            } else {
                TREE_CANOPY_RADIUS
            };
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    // Skip corners for rounder shape
                    if dx.abs() == radius && dz.abs() == radius {
                        continue;
                    }
                    // Don't overwrite trunk
                    if dx == 0 && dz == 0 && dy_offset <= trunk_height {
                        continue;
                    }
                    let leaf_pos = IVec3::new(pos.x + dx, wy, pos.z + dz);
                    let existing = get_block_at(&store, leaf_pos);
                    if existing == BlockType::Air {
                        set_block_at(&mut store, leaf_pos, leaf_type);
                        mark_remesh(leaf_pos, &manager, &mut commands);
                    }
                }
            }
        }

        // Remove from tracker
        tracker.saplings.remove(pos);
    }
}

/// System: tick crop growth timers and advance wheat stages.
/// Crops are tracked event-driven: added on plant (plant_seeds) and chunk load (update_chunk_loading).
pub fn update_crop_growth(
    time: Res<Time>,
    mut crop_tracker: ResMut<CropTracker>,
    mut store: ResMut<ChunkDataStore>,
    manager: Res<ChunkManager>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();

    // Remove tracked crops that are no longer growing crops (broken/replaced/fully grown)
    crop_tracker.crops.retain(|pos, _| {
        let block = get_block_at(&store, *pos);
        matches!(block, BlockType::WheatStage0 | BlockType::WheatStage1 | BlockType::WheatStage2)
    });

    // Tick timers and collect crops ready to advance
    let mut ready: Vec<IVec3> = Vec::new();
    for (pos, timer) in crop_tracker.crops.iter_mut() {
        *timer -= dt;
        if *timer <= 0.0 {
            ready.push(*pos);
        }
    }

    // Advance growth stage for each ready crop
    for pos in &ready {
        let block = get_block_at(&store, *pos);

        // Check that the block above is air (light requirement)
        let above = get_block_at(&store, *pos + IVec3::Y);
        if above != BlockType::Air {
            // Can't grow, reset timer
            if let Some(timer) = crop_tracker.crops.get_mut(pos) {
                *timer = CROP_GROW_MIN + rand::random::<f32>() * (CROP_GROW_MAX - CROP_GROW_MIN);
            }
            continue;
        }

        let next_stage = match block {
            BlockType::WheatStage0 => BlockType::WheatStage1,
            BlockType::WheatStage1 => BlockType::WheatStage2,
            BlockType::WheatStage2 => BlockType::WheatStage3,
            _ => continue,
        };

        set_block_at(&mut store, *pos, next_stage);
        mark_remesh(*pos, &manager, &mut commands);

        // If not yet fully grown, reset timer for next stage
        if next_stage != BlockType::WheatStage3 {
            if let Some(timer) = crop_tracker.crops.get_mut(pos) {
                *timer = CROP_GROW_MIN + rand::random::<f32>() * (CROP_GROW_MAX - CROP_GROW_MIN);
            }
        } else {
            // Fully grown, remove from tracker
            crop_tracker.crops.remove(pos);
        }
    }
}
