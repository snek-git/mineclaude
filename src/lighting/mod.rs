pub mod day_night;
pub mod sky;

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::block::BlockType;
use crate::world::chunk::CHUNK_SIZE;
use crate::world::manager::{ChunkDataStore, ChunkManager};
use day_night::Sun;

/// Maximum number of torch point lights active at once (performance cap).
pub const MAX_TORCH_LIGHTS: usize = 64;

/// Torch light range in blocks (vanilla torch = ~14).
pub const TORCH_LIGHT_RANGE: f32 = 14.0;

/// Torch light intensity.
pub const TORCH_LIGHT_INTENSITY: f32 = 1000.0;

/// Tracks spawned point lights for torch blocks. Maps world position to light entity.
#[derive(Resource, Default)]
pub struct TorchLights {
    pub lights: HashMap<IVec3, Entity>,
    /// Chunk positions that were loaded last frame — used to detect newly loaded/unloaded chunks.
    prev_loaded: HashSet<IVec3>,
}

/// Marker component for torch point light entities.
#[derive(Component)]
pub struct TorchLight;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<day_night::DayNightCycle>()
            .init_resource::<TorchLights>()
            .add_systems(Startup, (setup_lighting, sky::setup_sky))
            .add_systems(
                Update,
                (
                    day_night::advance_time,
                    day_night::update_sun,
                    day_night::update_ambient,
                    sky::update_sky_color,
                    sky::update_sky_bodies,
                )
                    .chain(),
            )
            .add_systems(Update, update_torch_lights);
    }
}

fn setup_lighting(mut commands: Commands) {
    // Directional light (sun)
    commands.spawn((
        Sun,
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85),
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
    ));

    // Global ambient light
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.6, 0.7, 1.0),
        brightness: 200.0,
        ..default()
    });
}

/// Scan a single chunk for all torch block positions, returning their world coords.
fn scan_chunk_for_torches(chunk_pos: IVec3, store: &ChunkDataStore) -> Vec<IVec3> {
    let Some(chunk) = store.chunks.get(&chunk_pos) else {
        return Vec::new();
    };
    let base = chunk_pos * CHUNK_SIZE as i32;
    let mut torches = Vec::new();
    for y in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if chunk.get(x, y, z) == BlockType::Torch {
                    torches.push(IVec3::new(
                        base.x + x as i32,
                        base.y + y as i32,
                        base.z + z as i32,
                    ));
                }
            }
        }
    }
    torches
}

/// Spawn a torch PointLight entity at the given world position.
fn spawn_torch_light(commands: &mut Commands, pos: IVec3) -> Entity {
    let light_pos = Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.7, pos.z as f32 + 0.5);
    commands
        .spawn((
            TorchLight,
            PointLight {
                color: Color::srgb(1.0, 0.8, 0.5),
                intensity: TORCH_LIGHT_INTENSITY,
                range: TORCH_LIGHT_RANGE,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(light_pos),
        ))
        .id()
}

/// Event-driven torch light sync system.
///
/// Instead of scanning all chunks every frame, this system:
/// 1. Detects newly loaded chunks (diff against previous frame) and scans only those for torches
/// 2. Detects unloaded chunks and removes their lights
/// 3. Validates existing lights still have torch blocks (catches block break/place, O(n) on active lights)
/// 4. Enforces the MAX_TORCH_LIGHTS cap by distance
fn update_torch_lights(
    mut commands: Commands,
    store: Res<ChunkDataStore>,
    manager: Res<ChunkManager>,
    camera: Query<&Transform, With<Camera3d>>,
    mut torch_lights: ResMut<TorchLights>,
) {
    let Ok(cam_transform) = camera.single() else {
        return;
    };
    let cam_pos = cam_transform.translation;

    // Build current loaded set
    let current_loaded: HashSet<IVec3> = manager.loaded.keys().copied().collect();

    // --- Step 1: Handle unloaded chunks — remove all lights in chunks that are no longer loaded ---
    let unloaded: Vec<IVec3> = torch_lights
        .prev_loaded
        .difference(&current_loaded)
        .copied()
        .collect();

    if !unloaded.is_empty() {
        let unloaded_set: HashSet<IVec3> = unloaded.into_iter().collect();
        let size = CHUNK_SIZE as i32;
        let to_remove: Vec<IVec3> = torch_lights
            .lights
            .keys()
            .filter(|pos| {
                let chunk_pos = IVec3::new(
                    pos.x.div_euclid(size),
                    pos.y.div_euclid(size),
                    pos.z.div_euclid(size),
                );
                unloaded_set.contains(&chunk_pos)
            })
            .copied()
            .collect();
        for pos in to_remove {
            if let Some(entity) = torch_lights.lights.remove(&pos) {
                commands.entity(entity).despawn();
            }
        }
    }

    // --- Step 2: Scan newly loaded chunks for torches ---
    let newly_loaded: Vec<IVec3> = current_loaded
        .difference(&torch_lights.prev_loaded)
        .copied()
        .collect();

    for chunk_pos in &newly_loaded {
        let torch_positions = scan_chunk_for_torches(*chunk_pos, &store);
        for pos in torch_positions {
            if !torch_lights.lights.contains_key(&pos) {
                let entity = spawn_torch_light(&mut commands, pos);
                torch_lights.lights.insert(pos, entity);
            }
        }
    }

    // --- Step 3: Validate existing lights — check each tracked position still has a torch ---
    // This is O(n) where n = number of active torch lights (max 64), not O(blocks).
    let size = CHUNK_SIZE as i32;
    let stale: Vec<IVec3> = torch_lights
        .lights
        .keys()
        .filter(|pos| {
            let chunk_pos = IVec3::new(
                pos.x.div_euclid(size),
                pos.y.div_euclid(size),
                pos.z.div_euclid(size),
            );
            if let Some(chunk) = store.chunks.get(&chunk_pos) {
                let lx = pos.x.rem_euclid(size) as usize;
                let ly = pos.y.rem_euclid(size) as usize;
                let lz = pos.z.rem_euclid(size) as usize;
                chunk.get(lx, ly, lz) != BlockType::Torch
            } else {
                true
            }
        })
        .copied()
        .collect();

    for pos in stale {
        if let Some(entity) = torch_lights.lights.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }

    // --- Step 4: Enforce MAX_TORCH_LIGHTS cap — keep closest, remove furthest ---
    if torch_lights.lights.len() > MAX_TORCH_LIGHTS {
        let mut by_distance: Vec<(IVec3, f32)> = torch_lights
            .lights
            .keys()
            .map(|pos| (*pos, cam_pos.distance_squared(pos.as_vec3())))
            .collect();
        // Sort ascending by distance — keep the closest
        by_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        // Remove entries beyond the cap (the tail = furthest)
        for (pos, _) in by_distance.drain(MAX_TORCH_LIGHTS..) {
            if let Some(entity) = torch_lights.lights.remove(&pos) {
                commands.entity(entity).despawn();
            }
        }
    }

    // --- Update prev_loaded for next frame ---
    torch_lights.prev_loaded = current_loaded;
}
