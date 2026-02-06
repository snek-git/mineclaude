use bevy::prelude::*;
use bevy::window::{CursorOptions, PrimaryWindow};

use crate::block::BlockType;
use crate::entity::mob::{Mob, MobHealth, MobVelocity};

/// System set label for block_interact (needed because it exceeds the IntoSystemSet param limit).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockInteractSet;

/// Bundled UI-open state resources to keep block_interact under the 16-param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct UiOpenState<'w> {
    pub inventory_open: Res<'w, InventoryOpen>,
    pub furnace_open: ResMut<'w, FurnaceOpen>,
    pub crafting_table_open: ResMut<'w, CraftingTableOpen>,
    pub chest_open: ResMut<'w, ChestOpen>,
}
use crate::inventory::chest::{ChestOpen, ChestStore};
use crate::inventory::crafting::CraftingTableOpen;
use crate::inventory::furnace::{FurnaceOpen, Furnaces};
use crate::inventory::inventory::{Inventory, INVENTORY_COLS, INVENTORY_SLOTS};
use crate::inventory::item::{Item, ToolKind, ToolTier};
use crate::ui::hotbar::HotbarState;
use crate::ui::inventory_screen::InventoryOpen;
use crate::world::chunk::CHUNK_SIZE;
use crate::world::coordinates::{world_to_chunk_pos, world_to_local_pos};
use crate::world::manager::{ChunkDataStore, ChunkManager, NeedsMesh};

use super::{Player, Hunger, PendingExhaustion};

/// Flag set by furnace_interact to prevent place_block from also firing on the same right-click.
#[derive(Resource, Default)]
pub struct RightClickConsumed(pub bool);

/// Flag set by attack_mob to prevent break_block from starting when a mob was hit this frame.
#[derive(Resource, Default)]
pub struct MobHitThisFrame(pub bool);

/// Tracks player attack cooldown (prevents spam clicking).
#[derive(Resource)]
pub struct AttackCooldown {
    pub remaining: f32,
}

impl Default for AttackCooldown {
    fn default() -> Self {
        Self { remaining: 0.0 }
    }
}

const PLAYER_ATTACK_COOLDOWN: f32 = 0.5;
const KNOCKBACK_STRENGTH: f32 = 8.0;

/// Tracks the block currently being broken by the player.
#[derive(Resource, Default)]
pub struct BreakingState {
    /// The block position being broken, if any.
    pub target: Option<IVec3>,
    /// The type of block being broken.
    pub block_type: BlockType,
    /// Breaking progress from 0.0 to 1.0.
    pub progress: f32,
    /// Entity for the crack overlay mesh.
    pub overlay_entity: Option<Entity>,
    /// Last crack stage rendered (0-9).
    pub last_stage: usize,
}

impl BreakingState {
    fn reset(&mut self) -> Option<Entity> {
        let entity = self.overlay_entity.take();
        self.target = None;
        self.progress = 0.0;
        self.last_stage = 0;
        entity
    }
}

/// Pre-loaded assets for the break overlay.
#[derive(Resource)]
pub struct BreakOverlayAssets {
    pub materials: [Handle<StandardMaterial>; 10],
    pub mesh: Handle<Mesh>,
}

/// Startup system to load break overlay assets.
pub fn setup_break_overlay(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let mesh = meshes.add(Cuboid::new(1.002, 1.002, 1.002));
    let mats: [Handle<StandardMaterial>; 10] = std::array::from_fn(|i| {
        let tex: Handle<Image> = asset_server.load(format!("textures/destroy_stage_{}.png", i));
        materials.add(StandardMaterial {
            base_color_texture: Some(tex),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        })
    });
    commands.insert_resource(BreakOverlayAssets { materials: mats, mesh });
}

const REACH_DISTANCE: f32 = 5.0;
const PLAYER_WIDTH: f32 = 0.6;
const PLAYER_HEIGHT: f32 = 1.8;

/// Returns the vanilla tool speed multiplier for the held item against a block type.
/// The effective break time is: base_break_time / multiplier.
/// Higher = faster. 1.0 = bare hand speed.
fn tool_speed_multiplier(held: Item, block: BlockType) -> f32 {
    let kind = held.tool_kind();
    let tier = held.tool_tier();

    // Check if this tool type is effective on this block
    let is_effective = match kind {
        Some(ToolKind::Pickaxe) => matches!(block,
            BlockType::Stone | BlockType::Cobblestone | BlockType::Sandstone
            | BlockType::CoalOre | BlockType::IronOre | BlockType::GoldOre
            | BlockType::DiamondOre | BlockType::Furnace),
        Some(ToolKind::Axe) => matches!(block,
            BlockType::OakLog | BlockType::BirchLog | BlockType::Planks
            | BlockType::CraftingTable | BlockType::DoorBottom | BlockType::DoorTop
            | BlockType::DoorBottomOpen | BlockType::DoorTopOpen),
        Some(ToolKind::Shovel) => matches!(block,
            BlockType::Dirt | BlockType::Grass | BlockType::Sand
            | BlockType::Gravel | BlockType::Clay | BlockType::Snow | BlockType::Farmland),
        _ => false,
    };

    if !is_effective {
        return 1.0;
    }

    // Vanilla tool speed multipliers (higher = faster, base time is divided by this)
    match tier {
        Some(ToolTier::Wooden) => 2.0,
        Some(ToolTier::Stone) => 4.0,
        Some(ToolTier::Iron) => 6.0,
        Some(ToolTier::Gold) => 12.0,
        Some(ToolTier::Diamond) => 8.0,
        None => 1.0,
    }
}

/// Check if the held item meets tool requirements to get drops from a block.
fn can_harvest(held: Item, block: BlockType) -> bool {
    let Some(required_tier) = block.required_pickaxe_tier() else {
        return true; // no requirement
    };
    // Must be holding a pickaxe of at least the required tier
    if held.tool_kind() != Some(ToolKind::Pickaxe) {
        return false;
    }
    match held.tool_tier() {
        Some(tier) => tier >= required_tier,
        None => false,
    }
}

/// Result of a successful raycast against the voxel grid.
struct RaycastHit {
    /// The block position that was hit.
    block_pos: IVec3,
    /// The position on the adjacent face (for placing).
    adjacent_pos: IVec3,
}

/// DDA raycast through a voxel grid. Returns the first solid block hit within max_dist.
fn voxel_raycast(origin: Vec3, direction: Vec3, max_dist: f32, store: &ChunkDataStore) -> Option<RaycastHit> {
    let dir = direction.normalize();

    // Current voxel position
    let mut x = origin.x.floor() as i32;
    let mut y = origin.y.floor() as i32;
    let mut z = origin.z.floor() as i32;

    // Step direction (+1 or -1)
    let step_x = if dir.x >= 0.0 { 1_i32 } else { -1 };
    let step_y = if dir.y >= 0.0 { 1_i32 } else { -1 };
    let step_z = if dir.z >= 0.0 { 1_i32 } else { -1 };

    // Distance along ray to cross one voxel boundary in each axis
    let t_delta_x = if dir.x != 0.0 { (1.0 / dir.x).abs() } else { f32::MAX };
    let t_delta_y = if dir.y != 0.0 { (1.0 / dir.y).abs() } else { f32::MAX };
    let t_delta_z = if dir.z != 0.0 { (1.0 / dir.z).abs() } else { f32::MAX };

    // Distance along ray to the next voxel boundary for each axis
    let mut t_max_x = if dir.x != 0.0 {
        if dir.x > 0.0 {
            ((x as f32 + 1.0) - origin.x) / dir.x
        } else {
            (x as f32 - origin.x) / dir.x
        }
    } else {
        f32::MAX
    };
    let mut t_max_y = if dir.y != 0.0 {
        if dir.y > 0.0 {
            ((y as f32 + 1.0) - origin.y) / dir.y
        } else {
            (y as f32 - origin.y) / dir.y
        }
    } else {
        f32::MAX
    };
    let mut t_max_z = if dir.z != 0.0 {
        if dir.z > 0.0 {
            ((z as f32 + 1.0) - origin.z) / dir.z
        } else {
            (z as f32 - origin.z) / dir.z
        }
    } else {
        f32::MAX
    };

    let mut prev_x = x;
    let mut prev_y = y;
    let mut prev_z = z;

    // Step through voxels
    let mut t = 0.0_f32;
    while t < max_dist {
        // Check current voxel
        let block = get_block(store, x, y, z);
        if block.is_targetable() {
            return Some(RaycastHit {
                block_pos: IVec3::new(x, y, z),
                adjacent_pos: IVec3::new(prev_x, prev_y, prev_z),
            });
        }

        prev_x = x;
        prev_y = y;
        prev_z = z;

        // Advance to next voxel boundary
        if t_max_x < t_max_y {
            if t_max_x < t_max_z {
                t = t_max_x;
                x += step_x;
                t_max_x += t_delta_x;
            } else {
                t = t_max_z;
                z += step_z;
                t_max_z += t_delta_z;
            }
        } else if t_max_y < t_max_z {
            t = t_max_y;
            y += step_y;
            t_max_y += t_delta_y;
        } else {
            t = t_max_z;
            z += step_z;
            t_max_z += t_delta_z;
        }
    }

    None
}

fn get_block(store: &ChunkDataStore, x: i32, y: i32, z: i32) -> BlockType {
    let size = CHUNK_SIZE as i32;
    let cx = x.div_euclid(size);
    let cy = y.div_euclid(size);
    let cz = z.div_euclid(size);
    let chunk_pos = IVec3::new(cx, cy, cz);

    let Some(chunk) = store.chunks.get(&chunk_pos) else {
        return BlockType::Air;
    };

    let lx = x.rem_euclid(size) as usize;
    let ly = y.rem_euclid(size) as usize;
    let lz = z.rem_euclid(size) as usize;

    chunk.get(lx, ly, lz)
}

fn handle_furnace_break(
    block: BlockType,
    pos: &IVec3,
    furnaces: &mut Furnaces,
    furnace_open: &mut FurnaceOpen,
    inventory: &mut Inventory,
) {
    if block != BlockType::Furnace {
        return;
    }
    if let Some(data) = furnaces.data.remove(pos) {
        if let Some((item, count, _)) = data.input {
            for _ in 0..count {
                inventory.add_item(item);
            }
        }
        if let Some((item, count, _)) = data.fuel {
            for _ in 0..count {
                inventory.add_item(item);
            }
        }
        if let Some((item, count, _)) = data.output {
            for _ in 0..count {
                inventory.add_item(item);
            }
        }
    }
    if furnace_open.0 == Some(*pos) {
        furnace_open.0 = None;
    }
}

fn handle_chest_break(
    block: BlockType,
    pos: &IVec3,
    chest_store: &mut ChestStore,
    chest_open: &mut ChestOpen,
    inventory: &mut Inventory,
) {
    if block != BlockType::Chest {
        return;
    }
    if let Some(data) = chest_store.data.remove(pos) {
        for slot in &data.slots {
            if let Some((item, count, _)) = slot {
                for _ in 0..*count {
                    inventory.add_item(*item);
                }
            }
        }
    }
    if chest_open.0 == Some(*pos) {
        chest_open.0 = None;
    }
}

/// Returns the attack damage for the held item.
fn weapon_damage(held: Item) -> f32 {
    match held.tool_kind() {
        Some(ToolKind::Sword) => {
            match held.tool_tier() {
                Some(ToolTier::Wooden) => 4.0,
                Some(ToolTier::Stone) => 5.0,
                Some(ToolTier::Iron) => 6.0,
                Some(ToolTier::Diamond) => 7.0,
                _ => 4.0,
            }
        }
        Some(_) => 2.0, // other tools used as weapons
        None => 1.0,     // bare hand / non-tool item
    }
}

/// Ray-AABB intersection test. Returns distance along ray to hit, or None.
fn ray_aabb(origin: Vec3, dir: Vec3, aabb_min: Vec3, aabb_max: Vec3) -> Option<f32> {
    let inv_dir = Vec3::new(
        if dir.x != 0.0 { 1.0 / dir.x } else { f32::MAX },
        if dir.y != 0.0 { 1.0 / dir.y } else { f32::MAX },
        if dir.z != 0.0 { 1.0 / dir.z } else { f32::MAX },
    );

    let t1 = (aabb_min.x - origin.x) * inv_dir.x;
    let t2 = (aabb_max.x - origin.x) * inv_dir.x;
    let t3 = (aabb_min.y - origin.y) * inv_dir.y;
    let t4 = (aabb_max.y - origin.y) * inv_dir.y;
    let t5 = (aabb_min.z - origin.z) * inv_dir.z;
    let t6 = (aabb_max.z - origin.z) * inv_dir.z;

    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    if tmax < 0.0 || tmin > tmax {
        return None;
    }
    // If tmin < 0, origin is inside the AABB, return 0
    Some(if tmin < 0.0 { 0.0 } else { tmin })
}

/// System: attack mobs on left click. Runs before break_block.
pub fn attack_mob(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryOpen>,
    cursor_q: Query<&CursorOptions, With<PrimaryWindow>>,
    camera_q: Query<&GlobalTransform, With<Camera3d>>,
    player_q: Query<&Transform, With<Player>>,
    store: Res<ChunkDataStore>,
    hotbar: Res<HotbarState>,
    mut cooldown: ResMut<AttackCooldown>,
    mut mob_hit: ResMut<MobHitThisFrame>,
    mut mobs: Query<(Entity, &Transform, &Mob, &mut MobHealth, &mut MobVelocity)>,
    mut inventory: ResMut<Inventory>,
    mut swing_audio: bevy::ecs::message::MessageWriter<crate::audio::SwordSwingAudio>,
    mut mob_hurt_audio: bevy::ecs::message::MessageWriter<crate::audio::MobHurtAudio>,
    mut pending_exhaustion: ResMut<PendingExhaustion>,
) {
    let dt = time.delta_secs();
    cooldown.remaining = (cooldown.remaining - dt).max(0.0);
    mob_hit.0 = false;

    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    if inventory_open.0 {
        return;
    }
    let Ok(cursor) = cursor_q.single() else { return };
    if cursor.visible {
        return;
    }
    if cooldown.remaining > 0.0 {
        return;
    }

    let Ok(cam_global) = camera_q.single() else { return };
    let origin = cam_global.translation();
    let forward = cam_global.forward().as_vec3();

    // Find the closest block hit distance for comparison
    let block_dist = voxel_raycast(origin, forward, REACH_DISTANCE, &store)
        .map(|hit| {
            let block_center = hit.block_pos.as_vec3() + Vec3::splat(0.5);
            origin.distance(block_center)
        });

    // Find the closest mob hit
    let mut closest_mob: Option<(Entity, f32)> = None;
    for (entity, mob_transform, mob, _health, _vel) in &mobs {
        let size = mob.mob_type.hitbox_size();
        let half = size * 0.5;
        let pos = mob_transform.translation;
        // Mob AABB centered on its position
        let aabb_min = Vec3::new(pos.x - half.x, pos.y - half.y, pos.z - half.z);
        let aabb_max = Vec3::new(pos.x + half.x, pos.y + half.y, pos.z + half.z);

        if let Some(t) = ray_aabb(origin, forward, aabb_min, aabb_max) {
            if t <= REACH_DISTANCE {
                let better = match closest_mob {
                    Some((_, best_t)) => t < best_t,
                    None => true,
                };
                if better {
                    closest_mob = Some((entity, t));
                }
            }
        }
    }

    let Some((mob_entity, mob_dist)) = closest_mob else {
        return;
    };

    // Only hit the mob if it's closer than (or equal to) the block
    if let Some(bd) = block_dist {
        if mob_dist > bd + 0.1 {
            return; // block is closer
        }
    }

    // Deal damage
    let held_item = hotbar.slots[hotbar.selected_slot];
    let damage = weapon_damage(held_item);

    let Ok(player_transform) = player_q.single() else { return };
    let player_pos = player_transform.translation;

    swing_audio.write(crate::audio::SwordSwingAudio);

    if let Ok((_entity, mob_transform, mob, mut health, mut velocity)) = mobs.get_mut(mob_entity) {
        health.current -= damage;
        mob_hurt_audio.write(crate::audio::MobHurtAudio {
            is_zombie: mob.mob_type == crate::entity::mob::MobType::Zombie,
        });

        // Knockback: push mob away from player
        let diff = mob_transform.translation - player_pos;
        let horizontal = Vec3::new(diff.x, 0.0, diff.z);
        if horizontal.length() > 0.01 {
            let kb_dir = horizontal.normalize();
            velocity.0.x += kb_dir.x * KNOCKBACK_STRENGTH;
            velocity.0.z += kb_dir.z * KNOCKBACK_STRENGTH;
            velocity.0.y += 4.0; // slight upward knock
        }
    }

    // Use tool durability on attack
    if held_item.is_tool() {
        let slot_idx = INVENTORY_SLOTS - INVENTORY_COLS + hotbar.selected_slot;
        inventory.use_tool(slot_idx);
    }

    // Attacking adds 0.1 exhaustion
    pending_exhaustion.0 += 0.1;

    cooldown.remaining = PLAYER_ATTACK_COOLDOWN;
    mob_hit.0 = true;
}

/// System: hold-to-break block on left click with crack overlay.
pub fn break_block(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryOpen>,
    mob_hit: Res<MobHitThisFrame>,
    cursor_q: Query<&CursorOptions, With<PrimaryWindow>>,
    camera_q: Query<&GlobalTransform, With<Camera3d>>,
    mut store: ResMut<ChunkDataStore>,
    mut inventory: ResMut<Inventory>,
    manager: Res<ChunkManager>,
    hotbar: Res<HotbarState>,
    mut commands: Commands,
    mut audio: bevy::ecs::message::MessageWriter<crate::audio::BlockBreakAudio>,
    (mut furnaces, mut furnace_open, mut chest_store, mut chest_open): (ResMut<Furnaces>, ResMut<FurnaceOpen>, ResMut<ChestStore>, ResMut<ChestOpen>),
    (mut breaking, mut pending_exhaustion, drop_assets): (ResMut<BreakingState>, ResMut<PendingExhaustion>, Res<crate::entity::dropped_item::DroppedItemAssets>),
    overlay_assets: Res<BreakOverlayAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let dt = time.delta_secs();

    // Check if we should be breaking
    let should_break = mouse.pressed(MouseButton::Left)
        && !inventory_open.0
        && !mob_hit.0
        && cursor_q.single().map_or(false, |c| !c.visible);

    if !should_break {
        if let Some(entity) = breaking.reset() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let Ok(cam_global) = camera_q.single() else {
        if let Some(entity) = breaking.reset() {
            commands.entity(entity).despawn();
        }
        return;
    };

    let origin = cam_global.translation();
    let forward = cam_global.forward().as_vec3();

    let Some(hit) = voxel_raycast(origin, forward, REACH_DISTANCE, &store) else {
        if let Some(entity) = breaking.reset() {
            commands.entity(entity).despawn();
        }
        return;
    };

    let block = get_block(&store, hit.block_pos.x, hit.block_pos.y, hit.block_pos.z);

    // Don't break bedrock or air
    if block == BlockType::Bedrock || block == BlockType::Air {
        if let Some(entity) = breaking.reset() {
            commands.entity(entity).despawn();
        }
        return;
    }

    // Get held item for tool mechanics
    let held_item = hotbar.slots[hotbar.selected_slot];
    let speed_mult = tool_speed_multiplier(held_item, block);
    let effective_break_time = block.break_time() / speed_mult;

    // Instant break for blocks with 0 break time
    if effective_break_time <= 0.0 {
        if let Some(entity) = breaking.reset() {
            commands.entity(entity).despawn();
        }

        handle_furnace_break(block, &hit.block_pos, &mut furnaces, &mut furnace_open, &mut inventory);
        handle_chest_break(block, &hit.block_pos, &mut chest_store, &mut chest_open, &mut inventory);
        handle_door_break(block, &hit.block_pos, &mut store, &manager, &mut commands);

        set_block(&mut store, hit.block_pos, BlockType::Air);
        if can_harvest(held_item, block) {
            let drop_pos = hit.block_pos.as_vec3() + Vec3::splat(0.5);
            if let Some(drop) = block.drop_item() {
                crate::entity::dropped_item::spawn_dropped_item(&mut commands, &mut meshes, &drop_assets, drop, 1, drop_pos);
            }
            for (bonus_item, bonus_count) in block.bonus_drops() {
                crate::entity::dropped_item::spawn_dropped_item(&mut commands, &mut meshes, &drop_assets, bonus_item, bonus_count, drop_pos);
            }
        }
        // Use tool durability
        if held_item.is_tool() {
            let slot_idx = INVENTORY_SLOTS - INVENTORY_COLS + hotbar.selected_slot;
            inventory.use_tool(slot_idx);
        }
        // Breaking blocks adds 0.005 exhaustion
        pending_exhaustion.0 += 0.005;
        audio.write(crate::audio::BlockBreakAudio);
        mark_needs_remesh(hit.block_pos, &manager, &mut commands);
        return;
    }

    // Check if target changed
    if breaking.target != Some(hit.block_pos) {
        if let Some(entity) = breaking.overlay_entity.take() {
            commands.entity(entity).despawn();
        }
        breaking.target = Some(hit.block_pos);
        breaking.block_type = block;
        breaking.progress = 0.0;
        breaking.last_stage = 0;

        let overlay_pos = hit.block_pos.as_vec3() + Vec3::splat(0.5);
        let entity = commands.spawn((
            Mesh3d(overlay_assets.mesh.clone()),
            MeshMaterial3d(overlay_assets.materials[0].clone()),
            Transform::from_translation(overlay_pos),
        )).id();
        breaking.overlay_entity = Some(entity);
    }

    // Increment progress using effective break time
    breaking.progress += dt / effective_break_time;

    // Update crack stage on overlay when it changes
    let stage = ((breaking.progress * 10.0).floor() as usize).min(9);
    if stage != breaking.last_stage {
        if let Some(entity) = breaking.overlay_entity {
            commands.entity(entity).insert(MeshMaterial3d(overlay_assets.materials[stage].clone()));
        }
        breaking.last_stage = stage;
    }

    // Check if breaking is complete
    if breaking.progress >= 1.0 {
        let target_pos = hit.block_pos;
        let target_block = breaking.block_type;

        if let Some(entity) = breaking.reset() {
            commands.entity(entity).despawn();
        }

        handle_furnace_break(target_block, &target_pos, &mut furnaces, &mut furnace_open, &mut inventory);
        handle_chest_break(target_block, &target_pos, &mut chest_store, &mut chest_open, &mut inventory);
        handle_door_break(target_block, &target_pos, &mut store, &manager, &mut commands);

        set_block(&mut store, target_pos, BlockType::Air);
        // Check tool requirements for drops
        if can_harvest(held_item, target_block) {
            let drop_pos = target_pos.as_vec3() + Vec3::splat(0.5);
            if let Some(drop) = target_block.drop_item() {
                crate::entity::dropped_item::spawn_dropped_item(&mut commands, &mut meshes, &drop_assets, drop, 1, drop_pos);
            }
            for (bonus_item, bonus_count) in target_block.bonus_drops() {
                crate::entity::dropped_item::spawn_dropped_item(&mut commands, &mut meshes, &drop_assets, bonus_item, bonus_count, drop_pos);
            }
        }
        // Use tool durability
        if held_item.is_tool() {
            let slot_idx = INVENTORY_SLOTS - INVENTORY_COLS + hotbar.selected_slot;
            inventory.use_tool(slot_idx);
        }
        // Breaking blocks adds 0.005 exhaustion
        pending_exhaustion.0 += 0.005;
        audio.write(crate::audio::BlockBreakAudio);
        mark_needs_remesh(target_pos, &manager, &mut commands);
    }
}

/// System: place block on right click.
pub fn place_block(
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryOpen>,
    consumed: Res<RightClickConsumed>,
    cursor_q: Query<&CursorOptions, With<PrimaryWindow>>,
    player_q: Query<&Transform, With<Player>>,
    camera_q: Query<&GlobalTransform, With<Camera3d>>,
    mut store: ResMut<ChunkDataStore>,
    mut inventory: ResMut<Inventory>,
    manager: Res<ChunkManager>,
    hotbar: Res<HotbarState>,
    mut commands: Commands,
    mut audio: bevy::ecs::message::MessageWriter<crate::audio::BlockPlaceAudio>,
    mut sapling_tracker: ResMut<crate::world::manager::SaplingTracker>,
    drop_assets: Res<crate::entity::dropped_item::DroppedItemAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    // Don't place if furnace interaction consumed the click
    if consumed.0 {
        return;
    }

    // Don't interact when inventory is open
    if inventory_open.0 {
        return;
    }

    let Ok(cursor) = cursor_q.single() else {
        return;
    };
    if cursor.visible {
        return;
    }

    let Ok(cam_global) = camera_q.single() else {
        return;
    };

    let origin = cam_global.translation();
    let forward = cam_global.forward().as_vec3();

    let Some(hit) = voxel_raycast(origin, forward, REACH_DISTANCE, &store) else {
        return;
    };

    let place_pos = hit.adjacent_pos;

    // Check that place position isn't occupied by a solid block
    let existing = get_block(&store, place_pos.x, place_pos.y, place_pos.z);
    if existing.is_solid() {
        return;
    }

    // If a non-cube block (torch, tallgrass, sapling, wheat) occupies the space,
    // break it first and drop its item
    if existing != BlockType::Air && existing.is_non_cube() {
        let drop_pos = place_pos.as_vec3() + Vec3::splat(0.5);
        if let Some(drop) = existing.drop_item() {
            crate::entity::dropped_item::spawn_dropped_item(&mut commands, &mut meshes, &drop_assets, drop, 1, drop_pos);
        }
        for (bonus_item, bonus_count) in existing.bonus_drops() {
            crate::entity::dropped_item::spawn_dropped_item(&mut commands, &mut meshes, &drop_assets, bonus_item, bonus_count, drop_pos);
        }
        set_block(&mut store, place_pos, BlockType::Air);
    }

    // Check overlap with player AABB
    let Ok(player_tf) = player_q.single() else {
        return;
    };
    if block_overlaps_player(place_pos, player_tf.translation) {
        return;
    }

    // Consume block from inventory (hotbar = bottom row of inventory)
    let slot_idx = INVENTORY_SLOTS - INVENTORY_COLS + hotbar.selected_slot;
    let Some(item) = inventory.remove_item(slot_idx) else {
        return;
    };
    let Some(block_type) = item.as_block() else {
        // Non-block items can't be placed; put it back
        inventory.add_item(item);
        return;
    };

    // Sapling placement: only on Air, on top of Dirt or Grass
    if block_type == BlockType::OakSapling || block_type == BlockType::BirchSapling {
        if existing != BlockType::Air {
            inventory.add_item(item);
            return;
        }
        let below = get_block(&store, place_pos.x, place_pos.y - 1, place_pos.z);
        if below != BlockType::Dirt && below != BlockType::Grass {
            inventory.add_item(item);
            return;
        }
    }

    // Door placement: need 2 blocks of space (bottom + top)
    if block_type == BlockType::DoorBottom {
        let top_pos = place_pos + IVec3::Y;
        let above = get_block(&store, top_pos.x, top_pos.y, top_pos.z);
        if above.is_solid() || block_overlaps_player(top_pos, player_tf.translation) {
            // Can't place door — not enough room; put item back
            inventory.add_item(item);
            return;
        }
        set_block(&mut store, place_pos, BlockType::DoorBottom);
        set_block(&mut store, top_pos, BlockType::DoorTop);
        audio.write(crate::audio::BlockPlaceAudio);
        mark_needs_remesh(place_pos, &manager, &mut commands);
        mark_needs_remesh(top_pos, &manager, &mut commands);
        return;
    }

    set_block(&mut store, place_pos, block_type);
    audio.write(crate::audio::BlockPlaceAudio);

    // Track placed saplings for growth
    if block_type == BlockType::OakSapling || block_type == BlockType::BirchSapling {
        use crate::world::manager::{SAPLING_GROW_MIN, SAPLING_GROW_MAX};
        let timer = SAPLING_GROW_MIN + rand::random::<f32>() * (SAPLING_GROW_MAX - SAPLING_GROW_MIN);
        sapling_tracker.saplings.insert(place_pos, timer);
    }

    mark_needs_remesh(place_pos, &manager, &mut commands);
}

fn set_block(store: &mut ChunkDataStore, world_pos: IVec3, block: BlockType) {
    let chunk_pos = world_to_chunk_pos(world_pos.as_vec3());
    let local = world_to_local_pos(world_pos);

    if let Some(chunk) = store.chunks.get_mut(&chunk_pos) {
        chunk.set(local.x as usize, local.y as usize, local.z as usize, block);
        store.modified.insert(chunk_pos);
    }
}

fn mark_needs_remesh(world_pos: IVec3, manager: &ChunkManager, commands: &mut Commands) {
    let chunk_pos = world_to_chunk_pos(world_pos.as_vec3());

    // Mark the primary chunk
    if let Some(&entity) = manager.loaded.get(&chunk_pos) {
        commands.entity(entity).insert(NeedsMesh);
    }

    // If block is on a chunk boundary, also remesh the neighbor
    let local = world_to_local_pos(world_pos);
    let size = CHUNK_SIZE as u32;
    let neighbors = [
        (local.x == 0, IVec3::new(-1, 0, 0)),
        (local.x == size - 1, IVec3::new(1, 0, 0)),
        (local.y == 0, IVec3::new(0, -1, 0)),
        (local.y == size - 1, IVec3::new(0, 1, 0)),
        (local.z == 0, IVec3::new(0, 0, -1)),
        (local.z == size - 1, IVec3::new(0, 0, 1)),
    ];
    for (on_edge, offset) in neighbors {
        if on_edge {
            let neighbor_chunk = chunk_pos + offset;
            if let Some(&entity) = manager.loaded.get(&neighbor_chunk) {
                commands.entity(entity).insert(NeedsMesh);
            }
        }
    }
}

/// When a door half is broken, also remove the other half.
fn handle_door_break(
    block: BlockType,
    pos: &IVec3,
    store: &mut ChunkDataStore,
    manager: &ChunkManager,
    commands: &mut Commands,
) {
    let other_pos = match block {
        BlockType::DoorBottom | BlockType::DoorBottomOpen => *pos + IVec3::Y,
        BlockType::DoorTop | BlockType::DoorTopOpen => *pos - IVec3::Y,
        _ => return,
    };
    let other_block = get_block(store, other_pos.x, other_pos.y, other_pos.z);
    match other_block {
        BlockType::DoorBottom | BlockType::DoorTop
        | BlockType::DoorBottomOpen | BlockType::DoorTopOpen => {
            set_block(store, other_pos, BlockType::Air);
            mark_needs_remesh(other_pos, manager, commands);
        }
        _ => {}
    }
}

/// Toggle a door between open and closed states, updating both halves.
fn toggle_door(
    block: BlockType,
    pos: IVec3,
    store: &mut ChunkDataStore,
    manager: &ChunkManager,
    commands: &mut Commands,
) {
    let (bottom_pos, top_pos) = match block {
        BlockType::DoorBottom | BlockType::DoorBottomOpen => {
            (pos, pos + IVec3::Y)
        }
        BlockType::DoorTop | BlockType::DoorTopOpen => {
            (pos - IVec3::Y, pos)
        }
        _ => return,
    };

    let bottom_block = get_block(store, bottom_pos.x, bottom_pos.y, bottom_pos.z);
    let top_block = get_block(store, top_pos.x, top_pos.y, top_pos.z);

    let (new_bottom, new_top) = match (bottom_block, top_block) {
        (BlockType::DoorBottom, BlockType::DoorTop) => {
            (BlockType::DoorBottomOpen, BlockType::DoorTopOpen)
        }
        (BlockType::DoorBottomOpen, BlockType::DoorTopOpen) => {
            (BlockType::DoorBottom, BlockType::DoorTop)
        }
        _ => return, // mismatched halves, don't toggle
    };

    set_block(store, bottom_pos, new_bottom);
    set_block(store, top_pos, new_top);
    mark_needs_remesh(bottom_pos, manager, commands);
    mark_needs_remesh(top_pos, manager, commands);
}

/// System: detect right-click on furnace or crafting table block and open the appropriate UI.
/// Runs before place_block so we can consume the click.
pub fn block_interact(
    mouse: Res<ButtonInput<MouseButton>>,
    mut ui_state: UiOpenState,
    dead: Res<crate::ui::death_screen::PlayerDead>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
    camera_q: Query<&GlobalTransform, With<Camera3d>>,
    mut store: ResMut<ChunkDataStore>,
    mut furnaces: ResMut<Furnaces>,
    mut chest_store: ResMut<ChestStore>,
    mut consumed: ResMut<RightClickConsumed>,
    mut cycle: ResMut<crate::lighting::day_night::DayNightCycle>,
    mut spawn_point: ResMut<super::SpawnPoint>,
    player_q: Query<&Transform, With<Player>>,
    manager: Res<ChunkManager>,
    mut commands: Commands,
) {
    consumed.0 = false;

    if dead.0 {
        return;
    }

    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    if ui_state.inventory_open.0 || ui_state.furnace_open.0.is_some() || ui_state.crafting_table_open.0 || ui_state.chest_open.0.is_some() {
        return;
    }

    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };
    if cursor.visible {
        return;
    }

    let Ok(cam_global) = camera_q.single() else {
        return;
    };

    let origin = cam_global.translation();
    let forward = cam_global.forward().as_vec3();

    let Some(hit) = voxel_raycast(origin, forward, REACH_DISTANCE, &store) else {
        return;
    };

    let block = get_block(&store, hit.block_pos.x, hit.block_pos.y, hit.block_pos.z);

    match block {
        BlockType::Furnace => {
            furnaces.data.entry(hit.block_pos).or_default();
            ui_state.furnace_open.0 = Some(hit.block_pos);
            consumed.0 = true;
            cursor.grab_mode = bevy::window::CursorGrabMode::None;
            cursor.visible = true;
        }
        BlockType::CraftingTable => {
            ui_state.crafting_table_open.0 = true;
            consumed.0 = true;
            cursor.grab_mode = bevy::window::CursorGrabMode::None;
            cursor.visible = true;
        }
        BlockType::Chest => {
            chest_store.data.entry(hit.block_pos).or_default();
            ui_state.chest_open.0 = Some(hit.block_pos);
            consumed.0 = true;
            cursor.grab_mode = bevy::window::CursorGrabMode::None;
            cursor.visible = true;
        }
        BlockType::Bed => {
            let sun = (cycle.time_of_day * std::f32::consts::TAU).sin();
            if sun < 0.0 {
                if let Ok(player_tf) = player_q.single() {
                    spawn_point.0 = player_tf.translation;
                }
                cycle.time_of_day = 0.0;
                consumed.0 = true;
            }
        }
        BlockType::DoorBottom | BlockType::DoorTop
        | BlockType::DoorBottomOpen | BlockType::DoorTopOpen => {
            toggle_door(block, hit.block_pos, &mut store, &manager, &mut commands);
            consumed.0 = true;
        }
        _ => {}
    }
}


/// System: eat food on right-click when holding a food item.
/// Runs after block_interact (so we don't eat when interacting with furnace/chest).
pub fn eat_food(
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryOpen>,
    dead: Res<crate::ui::death_screen::PlayerDead>,
    mut consumed: ResMut<RightClickConsumed>,
    cursor_q: Query<&CursorOptions, With<PrimaryWindow>>,
    mut hunger_q: Query<&mut Hunger, With<Player>>,
    mut inventory: ResMut<Inventory>,
    hotbar: Res<HotbarState>,
) {
    if dead.0 {
        return;
    }
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    if consumed.0 || inventory_open.0 {
        return;
    }
    let Ok(cursor) = cursor_q.single() else { return };
    if cursor.visible {
        return;
    }

    let Ok(mut hunger) = hunger_q.single_mut() else { return };

    // Check if holding a food item
    let held_item = hotbar.slots[hotbar.selected_slot];
    let Some((food_restore, sat_restore)) = held_item.food_value() else {
        return;
    };

    // Only eat when not full
    if hunger.food_level >= 20.0 {
        return;
    }

    // Consume 1 item from the hotbar slot
    let slot_idx = INVENTORY_SLOTS - INVENTORY_COLS + hotbar.selected_slot;
    if inventory.remove_item(slot_idx).is_none() {
        return;
    }

    // Prevent place_block from also firing on this right-click
    consumed.0 = true;

    // Restore food and saturation
    hunger.food_level = (hunger.food_level + food_restore).min(20.0);
    hunger.saturation = (hunger.saturation + sat_restore).min(hunger.food_level);
}

/// System: use hoe on dirt/grass to convert to farmland.
/// Runs after block_interact, before place_block.
pub fn hoe_interact(
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryOpen>,
    dead: Res<crate::ui::death_screen::PlayerDead>,
    mut consumed: ResMut<RightClickConsumed>,
    cursor_q: Query<&CursorOptions, With<PrimaryWindow>>,
    camera_q: Query<&GlobalTransform, With<Camera3d>>,
    mut store: ResMut<ChunkDataStore>,
    mut inventory: ResMut<Inventory>,
    manager: Res<ChunkManager>,
    hotbar: Res<HotbarState>,
    mut commands: Commands,
    mut audio: bevy::ecs::message::MessageWriter<crate::audio::BlockPlaceAudio>,
) {
    if dead.0 {
        return;
    }
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    if consumed.0 || inventory_open.0 {
        return;
    }
    let Ok(cursor) = cursor_q.single() else { return };
    if cursor.visible {
        return;
    }

    // Check if holding a hoe
    let held_item = hotbar.slots[hotbar.selected_slot];
    if held_item.tool_kind() != Some(ToolKind::Hoe) {
        return;
    }

    let Ok(cam_global) = camera_q.single() else { return };
    let origin = cam_global.translation();
    let forward = cam_global.forward().as_vec3();

    let Some(hit) = voxel_raycast(origin, forward, REACH_DISTANCE, &store) else {
        return;
    };

    let block = get_block(&store, hit.block_pos.x, hit.block_pos.y, hit.block_pos.z);
    if block != BlockType::Dirt && block != BlockType::Grass {
        return;
    }

    // Convert to farmland
    set_block(&mut store, hit.block_pos, BlockType::Farmland);
    consumed.0 = true;
    audio.write(crate::audio::BlockPlaceAudio);
    mark_needs_remesh(hit.block_pos, &manager, &mut commands);

    // Use hoe durability
    let slot_idx = INVENTORY_SLOTS - INVENTORY_COLS + hotbar.selected_slot;
    inventory.use_tool(slot_idx);
}

/// System: plant seeds on farmland.
/// Runs after hoe_interact, before place_block.
pub fn plant_seeds(
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryOpen>,
    dead: Res<crate::ui::death_screen::PlayerDead>,
    mut consumed: ResMut<RightClickConsumed>,
    cursor_q: Query<&CursorOptions, With<PrimaryWindow>>,
    camera_q: Query<&GlobalTransform, With<Camera3d>>,
    mut store: ResMut<ChunkDataStore>,
    mut inventory: ResMut<Inventory>,
    manager: Res<ChunkManager>,
    hotbar: Res<HotbarState>,
    mut commands: Commands,
    mut audio: bevy::ecs::message::MessageWriter<crate::audio::BlockPlaceAudio>,
    mut crop_tracker: ResMut<crate::world::manager::CropTracker>,
) {
    if dead.0 {
        return;
    }
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    if consumed.0 || inventory_open.0 {
        return;
    }
    let Ok(cursor) = cursor_q.single() else { return };
    if cursor.visible {
        return;
    }

    // Check if holding seeds
    let held_item = hotbar.slots[hotbar.selected_slot];
    if held_item != Item::Seeds {
        return;
    }

    let Ok(cam_global) = camera_q.single() else { return };
    let origin = cam_global.translation();
    let forward = cam_global.forward().as_vec3();

    let Some(hit) = voxel_raycast(origin, forward, REACH_DISTANCE, &store) else {
        return;
    };

    let block = get_block(&store, hit.block_pos.x, hit.block_pos.y, hit.block_pos.z);
    if block != BlockType::Farmland {
        return;
    }

    // Check that the block above the farmland is air
    let above_pos = hit.block_pos + IVec3::Y;
    let above = get_block(&store, above_pos.x, above_pos.y, above_pos.z);
    if above != BlockType::Air {
        return;
    }

    // Consume 1 seed from hotbar
    let slot_idx = INVENTORY_SLOTS - INVENTORY_COLS + hotbar.selected_slot;
    if inventory.remove_item(slot_idx).is_none() {
        return;
    }

    // Place WheatStage0 above the farmland
    set_block(&mut store, above_pos, BlockType::WheatStage0);
    consumed.0 = true;
    audio.write(crate::audio::BlockPlaceAudio);
    mark_needs_remesh(above_pos, &manager, &mut commands);

    // Track crop for growth
    use crate::world::manager::{CROP_GROW_MIN, CROP_GROW_MAX};
    let timer = CROP_GROW_MIN + rand::random::<f32>() * (CROP_GROW_MAX - CROP_GROW_MIN);
    crop_tracker.crops.insert(above_pos, timer);
}

fn block_overlaps_player(block_pos: IVec3, player_pos: Vec3) -> bool {
    let half_w = PLAYER_WIDTH / 2.0;
    let player_min = Vec3::new(
        player_pos.x - half_w,
        player_pos.y,
        player_pos.z - half_w,
    );
    let player_max = Vec3::new(
        player_pos.x + half_w,
        player_pos.y + PLAYER_HEIGHT,
        player_pos.z + half_w,
    );

    let block_min = block_pos.as_vec3();
    let block_max = block_min + Vec3::ONE;

    // AABB overlap test
    player_min.x < block_max.x
        && player_max.x > block_min.x
        && player_min.y < block_max.y
        && player_max.y > block_min.y
        && player_min.z < block_max.z
        && player_max.z > block_min.z
}
