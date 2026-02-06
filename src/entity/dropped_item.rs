use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use rand::Rng;

use crate::block::atlas::{texture_index, tile_uvs};
use crate::block::{BlockType, Face};
use crate::inventory::inventory::Inventory;
use crate::inventory::item::Item;
use crate::player::Player;

/// How long a dropped item lives before despawning (5 minutes like vanilla).
const DESPAWN_TIME: f32 = 300.0;

/// How close the player must be to pick up an item.
const PICKUP_RADIUS: f32 = 1.5;

/// Brief delay before item can be picked up (prevents instant grab on break).
const PICKUP_DELAY: f32 = 0.5;

/// Gravity for dropped items.
const ITEM_GRAVITY: f32 = -20.0;

/// Size of the dropped item cube.
const ITEM_SIZE: f32 = 0.25;

/// A dropped item entity in the world.
#[derive(Component)]
pub struct DroppedItem {
    pub item: Item,
    pub count: u8,
    pub age: f32,
    pub despawn_timer: f32,
}

/// Velocity for dropped items (simple gravity).
#[derive(Component, Default)]
pub struct ItemVelocity(pub Vec3);

/// Whether the item is on the ground.
#[derive(Component, Default)]
pub struct ItemOnGround(pub bool);

/// Stores the mesh/material handles for dropped items.
#[derive(Resource)]
pub struct DroppedItemAssets {
    pub mesh: Handle<Mesh>,
    pub atlas_material: Handle<StandardMaterial>,
    /// Fallback colored materials for non-block items
    pub stick_material: Handle<StandardMaterial>,
    pub coal_material: Handle<StandardMaterial>,
    pub iron_material: Handle<StandardMaterial>,
    pub gold_material: Handle<StandardMaterial>,
    pub diamond_material: Handle<StandardMaterial>,
    pub tool_material: Handle<StandardMaterial>,
    pub food_material: Handle<StandardMaterial>,
    pub seed_material: Handle<StandardMaterial>,
}

pub fn setup_dropped_item_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mesh = meshes.add(Cuboid::new(ITEM_SIZE, ITEM_SIZE, ITEM_SIZE));

    let atlas_tex: Handle<Image> = asset_server.load("textures/atlas.png");
    let atlas_material = materials.add(StandardMaterial {
        base_color_texture: Some(atlas_tex),
        perceptual_roughness: 1.0,
        ..default()
    });

    let stick_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.6, 0.4, 0.2),
        perceptual_roughness: 1.0,
        ..default()
    });
    let coal_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.15),
        perceptual_roughness: 1.0,
        ..default()
    });
    let iron_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        perceptual_roughness: 0.6,
        ..default()
    });
    let gold_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.85, 0.2),
        perceptual_roughness: 0.4,
        ..default()
    });
    let diamond_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.9, 0.9),
        perceptual_roughness: 0.3,
        ..default()
    });
    let tool_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.35, 0.2),
        perceptual_roughness: 0.8,
        ..default()
    });
    let food_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.3, 0.2),
        perceptual_roughness: 1.0,
        ..default()
    });
    let seed_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.5, 0.15),
        perceptual_roughness: 1.0,
        ..default()
    });

    commands.insert_resource(DroppedItemAssets {
        mesh,
        atlas_material,
        stick_material,
        coal_material,
        iron_material,
        gold_material,
        diamond_material,
        tool_material,
        food_material,
        seed_material,
    });
}

/// Returns a colored material handle for the given item type.
fn material_for_item(item: Item, assets: &DroppedItemAssets) -> Handle<StandardMaterial> {
    match item {
        Item::Block(_) => assets.atlas_material.clone(),
        Item::Stick => assets.stick_material.clone(),
        Item::Coal => assets.coal_material.clone(),
        Item::IronIngot => assets.iron_material.clone(),
        Item::GoldIngot => assets.gold_material.clone(),
        Item::Diamond => assets.diamond_material.clone(),
        Item::Apple | Item::Bread | Item::CookedPorkchop | Item::RawPorkchop
        | Item::RawBeef | Item::CookedBeef | Item::RawMutton | Item::CookedMutton
        | Item::RottenFlesh => {
            assets.food_material.clone()
        }
        Item::Bone | Item::Wool => assets.iron_material.clone(),
        Item::Leather => assets.stick_material.clone(),
        Item::Seeds => assets.seed_material.clone(),
        Item::Wheat => assets.gold_material.clone(),
        Item::LeatherHelmet | Item::LeatherChestplate | Item::LeatherLeggings | Item::LeatherBoots => {
            assets.stick_material.clone()
        }
        Item::IronHelmet | Item::IronChestplate | Item::IronLeggings | Item::IronBoots => {
            assets.iron_material.clone()
        }
        Item::DiamondHelmet | Item::DiamondChestplate | Item::DiamondLeggings | Item::DiamondBoots => {
            assets.diamond_material.clone()
        }
        _ => {
            if item.is_tool() {
                assets.tool_material.clone()
            } else {
                assets.stick_material.clone()
            }
        }
    }
}

/// Create a small cube mesh with UVs mapped to a specific atlas tile for a block type.
fn block_item_mesh(block: BlockType) -> Mesh {
    let s = ITEM_SIZE / 2.0;

    // 6 faces, 4 verts each = 24 verts
    let face_data: [(Face, [[f32; 3]; 4], [f32; 3]); 6] = [
        // Top (+Y)
        (Face::Top, [[-s, s, -s], [s, s, -s], [s, s, s], [-s, s, s]], [0.0, 1.0, 0.0]),
        // Bottom (-Y)
        (Face::Bottom, [[-s, -s, s], [s, -s, s], [s, -s, -s], [-s, -s, -s]], [0.0, -1.0, 0.0]),
        // North (+Z)
        (Face::North, [[-s, -s, s], [-s, s, s], [s, s, s], [s, -s, s]], [0.0, 0.0, 1.0]),
        // South (-Z)
        (Face::South, [[s, -s, -s], [s, s, -s], [-s, s, -s], [-s, -s, -s]], [0.0, 0.0, -1.0]),
        // East (+X)
        (Face::East, [[s, -s, s], [s, s, s], [s, s, -s], [s, -s, -s]], [1.0, 0.0, 0.0]),
        // West (-X)
        (Face::West, [[-s, -s, -s], [-s, s, -s], [-s, s, s], [-s, -s, s]], [-1.0, 0.0, 0.0]),
    ];

    let mut positions = Vec::with_capacity(24);
    let mut normals = Vec::with_capacity(24);
    let mut uvs = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (face, verts, normal) in &face_data {
        let idx = positions.len() as u32;
        let ti = texture_index(block, *face);
        let [u_min, v_min, u_max, v_max] = tile_uvs(ti);

        for v in verts {
            positions.push(*v);
            normals.push(*normal);
        }
        // UVs: bl, br, tr, tl for the quad
        uvs.push([u_min, v_max]);
        uvs.push([u_max, v_max]);
        uvs.push([u_max, v_min]);
        uvs.push([u_min, v_min]);

        indices.extend_from_slice(&[idx, idx + 1, idx + 2, idx, idx + 2, idx + 3]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices))
}

/// Spawn a dropped item entity at the given world position.
pub fn spawn_dropped_item(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &DroppedItemAssets,
    item: Item,
    count: u8,
    position: Vec3,
) {
    let mut rng = rand::rng();
    // Small random velocity so items scatter slightly
    let vx = rng.random_range(-1.5..1.5);
    let vy = rng.random_range(2.0..4.0);
    let vz = rng.random_range(-1.5..1.5);

    let material = material_for_item(item, assets);

    // Block items get a custom mesh with correct atlas UVs; non-block items use the plain cube
    let mesh_handle = match item {
        Item::Block(bt) => meshes.add(block_item_mesh(bt)),
        _ => assets.mesh.clone(),
    };

    commands.spawn((
        DroppedItem {
            item,
            count,
            age: 0.0,
            despawn_timer: DESPAWN_TIME,
        },
        ItemVelocity(Vec3::new(vx, vy, vz)),
        ItemOnGround::default(),
        Mesh3d(mesh_handle),
        MeshMaterial3d(material),
        Transform::from_translation(position),
        Visibility::default(),
    ));
}

/// Apply gravity and ground collision to dropped items.
pub fn dropped_item_physics(
    time: Res<Time>,
    store: Res<crate::world::manager::ChunkDataStore>,
    mut items: Query<(
        &mut Transform,
        &mut ItemVelocity,
        &mut ItemOnGround,
    ), With<DroppedItem>>,
) {
    let dt = time.delta_secs();
    let half = ITEM_SIZE / 2.0;

    for (mut transform, mut vel, mut on_ground) in &mut items {
        // Apply gravity
        vel.0.y += ITEM_GRAVITY * dt;

        // Horizontal drag
        vel.0.x *= 0.95_f32.powf(dt * 20.0);
        vel.0.z *= 0.95_f32.powf(dt * 20.0);

        // Y movement
        let new_y = transform.translation.y + vel.0.y * dt;
        let feet_y = new_y - half;
        let check_by = feet_y.floor() as i32;
        let bx = transform.translation.x.floor() as i32;
        let bz = transform.translation.z.floor() as i32;

        if vel.0.y <= 0.0 && is_block_solid(&store, bx, check_by, bz) {
            let landing_y = (check_by + 1) as f32 + half;
            transform.translation.y = landing_y;
            vel.0.y = 0.0;
            on_ground.0 = true;
        } else {
            transform.translation.y = new_y;
            on_ground.0 = false;
        }

        // X movement
        let new_x = transform.translation.x + vel.0.x * dt;
        let check_bx = if vel.0.x > 0.0 {
            (new_x + half).floor() as i32
        } else {
            (new_x - half).floor() as i32
        };
        let by = transform.translation.y.floor() as i32;
        if is_block_solid(&store, check_bx, by, bz) {
            vel.0.x = 0.0;
        } else {
            transform.translation.x = new_x;
        }

        // Z movement
        let new_z = transform.translation.z + vel.0.z * dt;
        let check_bz = if vel.0.z > 0.0 {
            (new_z + half).floor() as i32
        } else {
            (new_z - half).floor() as i32
        };
        let bx2 = transform.translation.x.floor() as i32;
        if is_block_solid(&store, bx2, by, check_bz) {
            vel.0.z = 0.0;
        } else {
            transform.translation.z = new_z;
        }

        // No void clamp — items that fall into void get despawned by dropped_item_despawn_void
    }
}

fn is_block_solid(store: &crate::world::manager::ChunkDataStore, x: i32, y: i32, z: i32) -> bool {
    let size = crate::world::chunk::CHUNK_SIZE as i32;
    let cx = x.div_euclid(size);
    let cy = y.div_euclid(size);
    let cz = z.div_euclid(size);
    let chunk_pos = IVec3::new(cx, cy, cz);

    let Some(chunk) = store.chunks.get(&chunk_pos) else {
        return false;
    };

    let lx = x.rem_euclid(size) as usize;
    let ly = y.rem_euclid(size) as usize;
    let lz = z.rem_euclid(size) as usize;

    chunk.get(lx, ly, lz).is_solid()
}

/// Rotate dropped items slowly for visual appeal.
pub fn dropped_item_bob(
    time: Res<Time>,
    mut items: Query<&mut Transform, With<DroppedItem>>,
) {
    let t = time.elapsed_secs();
    for mut transform in &mut items {
        // Slow spin
        transform.rotation = Quat::from_rotation_y(t * 1.5);
    }
}

/// Player picks up dropped items when within range.
pub fn pickup_dropped_items(
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    player_q: Query<&Transform, With<Player>>,
    items: Query<(Entity, &Transform, &DroppedItem)>,
    mut pickup_audio: bevy::ecs::message::MessageWriter<crate::audio::ItemPickupAudio>,
) {
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    let player_pos = player_tf.translation;

    for (entity, item_tf, dropped) in &items {
        // Don't pick up items that just spawned
        if dropped.age < PICKUP_DELAY {
            continue;
        }

        let dist = player_pos.distance(item_tf.translation);
        if dist <= PICKUP_RADIUS {
            // Try to add items to inventory
            let mut added = 0u8;
            for _ in 0..dropped.count {
                if inventory.add_item(dropped.item) {
                    added += 1;
                } else {
                    break;
                }
            }
            if added == dropped.count {
                // All items picked up, despawn entity
                commands.entity(entity).despawn();
                pickup_audio.write(crate::audio::ItemPickupAudio);
            }
            // If partial pickup, we'd need to mutate count — skip for simplicity,
            // player will pick up on next pass when inventory has space
        }
    }
}

/// Age dropped items and despawn expired ones.
pub fn dropped_item_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut items: Query<(Entity, &mut DroppedItem)>,
) {
    let dt = time.delta_secs();
    for (entity, mut dropped) in &mut items {
        dropped.age += dt;
        dropped.despawn_timer -= dt;
        if dropped.despawn_timer <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Despawn dropped items that fall into the void (below Y=-20).
pub fn dropped_item_despawn_void(
    mut commands: Commands,
    items: Query<(Entity, &Transform), With<DroppedItem>>,
) {
    for (entity, transform) in &items {
        if transform.translation.y < -20.0 {
            commands.entity(entity).despawn();
        }
    }
}
