use bevy::prelude::*;
use rand::Rng;

use crate::block::BlockType;
use crate::lighting::day_night::DayNightCycle;
use crate::player::{Health, Player};
use crate::world::chunk::CHUNK_SIZE;
use crate::world::coordinates::{world_to_chunk_pos, world_to_local_pos};
use crate::world::manager::ChunkDataStore;

const MAX_MOBS: usize = 15;
const MAX_HOSTILE_MOBS: usize = 10;
const SPAWN_INTERVAL: f32 = 2.5;
const DESPAWN_DISTANCE: f32 = 120.0;
const GRAVITY: f32 = -20.0;
const MOB_SPEED: f32 = 2.0;
const HOSTILE_DETECT_RANGE: f32 = 16.0;
const HOSTILE_LOSE_RANGE: f32 = 24.0;
const ATTACK_RANGE: f32 = 2.0;
const ATTACK_COOLDOWN: f32 = 1.0;
const SUNBURN_DPS: f32 = 1.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobType {
    Sheep,
    Cow,
    Zombie,
    Skeleton,
}

impl MobType {
    pub fn is_hostile(self) -> bool {
        matches!(self, MobType::Zombie | MobType::Skeleton)
    }

    pub fn max_health(self) -> f32 {
        match self {
            MobType::Sheep => 8.0,
            MobType::Cow => 10.0,
            MobType::Zombie => 20.0,
            MobType::Skeleton => 20.0,
        }
    }

    pub fn attack_damage(self) -> f32 {
        match self {
            MobType::Zombie => 3.0,
            MobType::Skeleton => 2.0,
            _ => 0.0,
        }
    }

    /// Returns (width, height, depth) of the mob's hitbox cuboid
    pub fn hitbox_size(self) -> Vec3 {
        match self {
            MobType::Sheep => Vec3::new(0.8, 0.8, 0.8),
            MobType::Cow => Vec3::new(0.9, 0.9, 1.2),
            MobType::Zombie | MobType::Skeleton => Vec3::new(0.6, 1.8, 0.6),
        }
    }

    /// Returns item drops when this mob dies
    pub fn loot_drops(&self) -> Vec<(crate::inventory::item::Item, u8)> {
        use crate::inventory::item::Item;
        let mut rng = rand::rng();
        match self {
            MobType::Sheep => {
                let mut drops = vec![(Item::RawMutton, rng.random_range(1u8..=2))];
                drops.push((Item::Wool, 1));
                drops
            }
            MobType::Cow => {
                let mut drops = vec![(Item::RawBeef, rng.random_range(1u8..=3))];
                let leather_count = rng.random_range(0u8..=2);
                if leather_count > 0 {
                    drops.push((Item::Leather, leather_count));
                }
                drops
            }
            MobType::Zombie => {
                let count = rng.random_range(0u8..=2);
                if count > 0 { vec![(Item::RottenFlesh, count)] } else { vec![] }
            }
            MobType::Skeleton => {
                let count = rng.random_range(0u8..=2);
                if count > 0 { vec![(Item::Bone, count)] } else { vec![] }
            }
        }
    }
}

#[derive(Component)]
pub struct Mob {
    pub mob_type: MobType,
}

/// Marker component for hostile mobs
#[derive(Component)]
pub struct Hostile;

#[derive(Component)]
pub struct MobHealth {
    pub current: f32,
    pub max: f32,
}

impl MobHealth {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
}

/// Tracks attack cooldown for hostile mobs
#[derive(Component)]
pub struct MobAttackTimer {
    pub cooldown: f32,
}

impl Default for MobAttackTimer {
    fn default() -> Self {
        Self { cooldown: 0.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MobState {
    Idle,
    Walking,
    Chasing,
}

#[derive(Component)]
pub struct MobAI {
    pub state: MobState,
    pub target: Option<Vec3>,
    pub idle_timer: f32,
}

impl Default for MobAI {
    fn default() -> Self {
        Self {
            state: MobState::Idle,
            target: None,
            idle_timer: 2.0,
        }
    }
}

#[derive(Component, Default)]
pub struct MobVelocity(pub Vec3);

/// Tracks whether a mob is standing on solid ground
#[derive(Component)]
pub struct MobOnGround(pub bool);

impl Default for MobOnGround {
    fn default() -> Self {
        Self(false)
    }
}

/// Check if a block at world coordinates is solid, using div_euclid/rem_euclid for negative coords
fn is_block_solid(store: &ChunkDataStore, x: i32, y: i32, z: i32) -> bool {
    let size = CHUNK_SIZE as i32;
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

#[derive(Resource)]
pub struct MobSpawnTimer(pub Timer);

impl Default for MobSpawnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SPAWN_INTERVAL, TimerMode::Repeating))
    }
}

#[derive(Resource)]
pub struct MobMaterials {
    pub sheep_material: Handle<StandardMaterial>,
    pub cow_material: Handle<StandardMaterial>,
    pub zombie_material: Handle<StandardMaterial>,
    pub skeleton_material: Handle<StandardMaterial>,
    pub sheep_mesh: Handle<Mesh>,
    pub cow_mesh: Handle<Mesh>,
    pub humanoid_mesh: Handle<Mesh>,
}

pub fn setup_mob_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let sheep_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.9),
        perceptual_roughness: 1.0,
        ..default()
    });
    let cow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.28, 0.15),
        perceptual_roughness: 1.0,
        ..default()
    });
    let zombie_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.5, 0.25),
        perceptual_roughness: 1.0,
        ..default()
    });
    let skeleton_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.78, 0.78, 0.72),
        perceptual_roughness: 1.0,
        ..default()
    });
    let sheep_mesh = meshes.add(Cuboid::new(0.8, 0.8, 0.8));
    let cow_mesh = meshes.add(Cuboid::new(0.9, 0.9, 1.2));
    let humanoid_mesh = meshes.add(Cuboid::new(0.6, 1.8, 0.6));

    commands.insert_resource(MobMaterials {
        sheep_material,
        cow_material,
        zombie_material,
        skeleton_material,
        sheep_mesh,
        cow_mesh,
        humanoid_mesh,
    });
    commands.init_resource::<MobSpawnTimer>();
}

fn get_block_at(store: &ChunkDataStore, world_x: i32, world_y: i32, world_z: i32) -> BlockType {
    let world_pos = IVec3::new(world_x, world_y, world_z);
    let chunk_pos = world_to_chunk_pos(Vec3::new(world_x as f32, world_y as f32, world_z as f32));
    let local = world_to_local_pos(world_pos);
    if let Some(chunk) = store.chunks.get(&chunk_pos) {
        chunk.get(local.x as usize, local.y as usize, local.z as usize)
    } else {
        BlockType::Air
    }
}

const SUNBURN_CHECK_INTERVAL: f32 = 1.0;
const SKY_SCAN_MAX: i32 = 32;

/// Check if a position has direct sky exposure (no solid blocks within 32 blocks above)
fn is_sky_exposed(store: &ChunkDataStore, pos: Vec3) -> bool {
    let bx = pos.x.floor() as i32;
    let bz = pos.z.floor() as i32;
    let by = pos.y.floor() as i32;
    let max_y = (by + 1 + SKY_SCAN_MAX).min(256);
    for y in (by + 1)..=max_y {
        let block = get_block_at(store, bx, y, bz);
        if block.is_solid() {
            return false;
        }
    }
    true
}

fn find_spawn_position(
    player_pos: Vec3,
    store: &ChunkDataStore,
    rng: &mut impl Rng,
    min_dist: f32,
    max_dist: f32,
    require_grass: bool,
) -> Option<Vec3> {
    let angle: f32 = rng.random::<f32>() * std::f32::consts::TAU;
    let dist: f32 = rng.random_range(min_dist..max_dist);
    let x = player_pos.x + angle.cos() * dist;
    let z = player_pos.z + angle.sin() * dist;
    let bx = x.floor() as i32;
    let bz = z.floor() as i32;

    let max_y = 128;
    let min_y = 1;
    for y in (min_y..=max_y).rev() {
        let block = get_block_at(store, bx, y, bz);
        let above = get_block_at(store, bx, y + 1, bz);
        let above2 = get_block_at(store, bx, y + 2, bz);
        let valid_ground = if require_grass {
            block == BlockType::Grass
        } else {
            block.is_solid()
        };
        if valid_ground && above.is_air() && above2.is_air() {
            return Some(Vec3::new(x, y as f32 + 1.0, z));
        }
    }
    None
}

fn is_night(cycle: &DayNightCycle) -> bool {
    let sun_height = (cycle.time_of_day * std::f32::consts::TAU).sin();
    sun_height < 0.0
}

pub fn spawn_mobs(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<MobSpawnTimer>,
    mob_materials: Res<MobMaterials>,
    store: Res<ChunkDataStore>,
    cycle: Res<DayNightCycle>,
    mobs: Query<&Mob>,
    player: Query<&Transform, With<Player>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    let mut rng = rand::rng();
    let night = is_night(&cycle);

    // Count passive and hostile mobs separately
    let mut passive_count = 0usize;
    let mut hostile_count = 0usize;
    for mob in &mobs {
        if mob.mob_type.is_hostile() {
            hostile_count += 1;
        } else {
            passive_count += 1;
        }
    }

    // Try to spawn a hostile mob at night
    let mut spawned = false;
    if night && hostile_count < MAX_HOSTILE_MOBS {
        if let Some(spawn_pos) = find_spawn_position(player_pos, &store, &mut rng, 24.0, 128.0, false) {
            let mob_type = if rng.random_bool(0.5) {
                MobType::Zombie
            } else {
                MobType::Skeleton
            };

            let (mesh, material) = match mob_type {
                MobType::Zombie => (mob_materials.humanoid_mesh.clone(), mob_materials.zombie_material.clone()),
                MobType::Skeleton => (mob_materials.humanoid_mesh.clone(), mob_materials.skeleton_material.clone()),
                _ => (mob_materials.humanoid_mesh.clone(), mob_materials.zombie_material.clone()),
            };

            commands.spawn((
                Mob { mob_type },
                Hostile,
                MobHealth::new(mob_type.max_health()),
                MobAttackTimer::default(),
                MobAI::default(),
                MobVelocity::default(),
                MobOnGround::default(),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(spawn_pos),
                Visibility::default(),
            ));
            spawned = true;
        }
    }

    if spawned {
        return;
    }

    // Spawn passive mobs (fallback when hostile spawn fails or it's daytime)
    if passive_count >= MAX_MOBS {
        return;
    }

    let Some(spawn_pos) = find_spawn_position(player_pos, &store, &mut rng, 10.0, 40.0, true) else {
        return;
    };

    let mob_type = if rng.random_bool(0.5) {
        MobType::Sheep
    } else {
        MobType::Cow
    };

    let (mesh, material) = match mob_type {
        MobType::Sheep => (mob_materials.sheep_mesh.clone(), mob_materials.sheep_material.clone()),
        MobType::Cow => (mob_materials.cow_mesh.clone(), mob_materials.cow_material.clone()),
        _ => return,
    };

    commands.spawn((
        Mob { mob_type },
        MobHealth::new(mob_type.max_health()),
        MobAI::default(),
        MobVelocity::default(),
        MobOnGround::default(),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_translation(spawn_pos),
        Visibility::default(),
    ));
}

pub fn update_mob_ai(
    time: Res<Time>,
    mut mobs: Query<(&Transform, &mut MobAI, &Mob)>,
    player: Query<&Transform, With<Player>>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::rng();

    let player_pos = if let Ok(pt) = player.single() {
        Some(pt.translation)
    } else {
        None
    };

    for (transform, mut ai, mob) in &mut mobs {
        // Hostile mob chasing logic
        if mob.mob_type.is_hostile() {
            if let Some(pp) = player_pos {
                let dist = transform.translation.distance(pp);
                match ai.state {
                    MobState::Chasing => {
                        if dist > HOSTILE_LOSE_RANGE {
                            ai.state = MobState::Idle;
                            ai.target = None;
                            ai.idle_timer = rng.random_range(1.0..3.0);
                            continue;
                        }
                        ai.target = Some(pp);
                        continue;
                    }
                    MobState::Idle | MobState::Walking => {
                        if dist <= HOSTILE_DETECT_RANGE {
                            ai.state = MobState::Chasing;
                            ai.target = Some(pp);
                            continue;
                        }
                    }
                }
            }
        }

        // Standard passive AI (also used by hostile mobs not chasing)
        match ai.state {
            MobState::Idle => {
                ai.idle_timer -= dt;
                if ai.idle_timer <= 0.0 {
                    let angle: f32 = rng.random::<f32>() * std::f32::consts::TAU;
                    let dist: f32 = rng.random_range(3.0..8.0);
                    let target = Vec3::new(
                        transform.translation.x + angle.cos() * dist,
                        transform.translation.y,
                        transform.translation.z + angle.sin() * dist,
                    );
                    ai.target = Some(target);
                    ai.state = MobState::Walking;
                }
            }
            MobState::Walking => {
                if let Some(target) = ai.target {
                    let diff = target - transform.translation;
                    let horizontal = Vec3::new(diff.x, 0.0, diff.z);
                    if horizontal.length() < 0.5 {
                        ai.state = MobState::Idle;
                        ai.target = None;
                        ai.idle_timer = rng.random_range(2.0..5.0);
                    }
                } else {
                    ai.state = MobState::Idle;
                    ai.idle_timer = 2.0;
                }
            }
            MobState::Chasing => {
                // Non-hostile mobs shouldn't be in Chasing, reset to Idle
                ai.state = MobState::Idle;
                ai.idle_timer = 2.0;
            }
        }
    }
}

/// Hostile mobs attack the player when within range
pub fn hostile_attack_player(
    time: Res<Time>,
    mut mobs: Query<(&Transform, &Mob, &mut MobAttackTimer), With<Hostile>>,
    mut player: Query<(&Transform, &mut Health, &mut crate::player::ArmorSlots), With<Player>>,
    mut hurt_audio: bevy::ecs::message::MessageWriter<crate::audio::PlayerHurtAudio>,
) {
    let dt = time.delta_secs();
    let Ok((player_transform, mut player_health, mut armor)) = player.single_mut() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (transform, mob, mut attack_timer) in &mut mobs {
        attack_timer.cooldown -= dt;
        let dist = transform.translation.distance(player_pos);
        if dist <= ATTACK_RANGE && attack_timer.cooldown <= 0.0 {
            let raw_damage = mob.mob_type.attack_damage();
            if raw_damage > 0.0 {
                let total_armor = armor.total_armor_points() as f32;
                let damage = raw_damage * (1.0 - (total_armor.min(20.0) / 25.0));
                player_health.current = (player_health.current - damage).max(0.0);
                armor.damage_all_pieces();
                attack_timer.cooldown = ATTACK_COOLDOWN;
                hurt_audio.write(crate::audio::PlayerHurtAudio);
            }
        }
    }
}

/// Despawn mobs whose health has reached zero and drop loot as dropped item entities
pub fn despawn_dead_mobs(
    mut commands: Commands,
    mobs: Query<(Entity, &MobHealth, &Mob, &Transform)>,
    drop_assets: Res<super::dropped_item::DroppedItemAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut death_audio: bevy::ecs::message::MessageWriter<crate::audio::MobDeathAudio>,
) {
    for (entity, health, mob, transform) in &mobs {
        if health.current <= 0.0 {
            let drop_pos = transform.translation;
            for (item, count) in mob.mob_type.loot_drops() {
                super::dropped_item::spawn_dropped_item(&mut commands, &mut meshes, &drop_assets, item, count, drop_pos);
            }
            death_audio.write(crate::audio::MobDeathAudio);
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Resource)]
pub struct SunburnTimer(pub Timer);

impl Default for SunburnTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(SUNBURN_CHECK_INTERVAL, TimerMode::Repeating))
    }
}

/// Hostile mobs burn in sunlight (checked every 1 second, scans up to 32 blocks above)
pub fn hostile_sunburn(
    time: Res<Time>,
    cycle: Res<DayNightCycle>,
    store: Res<ChunkDataStore>,
    mut timer: ResMut<SunburnTimer>,
    mut mobs: Query<(&Transform, &mut MobHealth), With<Hostile>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    if is_night(&cycle) {
        return;
    }
    for (transform, mut health) in &mut mobs {
        if is_sky_exposed(&store, transform.translation) {
            health.current -= SUNBURN_DPS * SUNBURN_CHECK_INTERVAL;
        }
    }
}

/// Check if a mob's AABB at a given position collides with any solid block.
/// `pos` is the mob center, `half_w` is XZ half-width, `height` is full height.
/// The mob AABB spans [pos.x - half_w, pos.x + half_w] x [pos.y - height/2, pos.y + height/2] x [pos.z - half_w, pos.z + half_w].
fn mob_collides_horizontal(store: &ChunkDataStore, pos: Vec3, half_w: f32, half_h: f32) -> bool {
    let min_bx = (pos.x - half_w).floor() as i32;
    let max_bx = (pos.x + half_w - 0.001).floor() as i32;
    let min_by = (pos.y - half_h).floor() as i32;
    let max_by = (pos.y + half_h - 0.001).floor() as i32;
    let min_bz = (pos.z - half_w).floor() as i32;
    let max_bz = (pos.z + half_w - 0.001).floor() as i32;

    for by in min_by..=max_by {
        for bx in min_bx..=max_bx {
            for bz in min_bz..=max_bz {
                if is_block_solid(store, bx, by, bz) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn move_mobs(
    time: Res<Time>,
    store: Res<ChunkDataStore>,
    mut mobs: Query<(&mut Transform, &MobAI, &mut MobVelocity, &Mob, &mut MobOnGround)>,
) {
    let dt = time.delta_secs();

    for (mut transform, ai, mut velocity, mob, mut on_ground) in &mut mobs {
        let hitbox = mob.mob_type.hitbox_size();
        let half_height = hitbox.y / 2.0;
        let half_w = hitbox.x / 2.0;

        // Apply gravity
        velocity.0.y += GRAVITY * dt;

        // Horizontal movement toward target
        let moving = matches!(ai.state, MobState::Walking | MobState::Chasing);
        if moving {
            if let Some(target) = ai.target {
                let diff = target - transform.translation;
                let horizontal = Vec3::new(diff.x, 0.0, diff.z);
                if horizontal.length() > 0.1 {
                    let dir = horizontal.normalize();
                    let speed = if ai.state == MobState::Chasing {
                        MOB_SPEED * 1.2
                    } else {
                        MOB_SPEED
                    };
                    velocity.0.x = dir.x * speed;
                    velocity.0.z = dir.z * speed;

                    // Face movement direction
                    let look_target = transform.translation + dir;
                    transform.look_at(look_target, Vec3::Y);
                }
            }
        } else {
            velocity.0.x = 0.0;
            velocity.0.z = 0.0;
        }

        let pos = transform.translation;

        // --- Y axis collision (gravity/falling) ---
        let new_y = pos.y + velocity.0.y * dt;
        if velocity.0.y <= 0.0 {
            // Falling: check blocks below feet
            let feet_y = new_y - half_height;
            let min_bx = (pos.x - half_w).floor() as i32;
            let max_bx = (pos.x + half_w - 0.001).floor() as i32;
            let min_bz = (pos.z - half_w).floor() as i32;
            let max_bz = (pos.z + half_w - 0.001).floor() as i32;
            let check_by = (feet_y).floor() as i32;

            let mut landed = false;
            for bx in min_bx..=max_bx {
                for bz in min_bz..=max_bz {
                    if is_block_solid(&store, bx, check_by, bz) {
                        let landing_y = (check_by + 1) as f32 + half_height;
                        if new_y - half_height <= landing_y - half_height + 0.01 {
                            transform.translation.y = landing_y;
                            velocity.0.y = 0.0;
                            on_ground.0 = true;
                            landed = true;
                            break;
                        }
                    }
                }
                if landed {
                    break;
                }
            }
            if !landed {
                transform.translation.y = new_y;
                on_ground.0 = false;
            }
        } else {
            // Rising: check blocks above head
            let head_y = new_y + half_height;
            let min_bx = (pos.x - half_w).floor() as i32;
            let max_bx = (pos.x + half_w - 0.001).floor() as i32;
            let min_bz = (pos.z - half_w).floor() as i32;
            let max_bz = (pos.z + half_w - 0.001).floor() as i32;
            let check_by = head_y.floor() as i32;

            let mut hit_ceiling = false;
            for bx in min_bx..=max_bx {
                for bz in min_bz..=max_bz {
                    if is_block_solid(&store, bx, check_by, bz) {
                        transform.translation.y = check_by as f32 - half_height - 0.001;
                        velocity.0.y = 0.0;
                        hit_ceiling = true;
                        break;
                    }
                }
                if hit_ceiling {
                    break;
                }
            }
            if !hit_ceiling {
                transform.translation.y = new_y;
                on_ground.0 = false;
            }
        }

        // --- X axis collision ---
        let new_x = transform.translation.x + velocity.0.x * dt;
        let proposed_x = Vec3::new(new_x, transform.translation.y, transform.translation.z);
        let mut stepped_up = false;

        if mob_collides_horizontal(&store, proposed_x, half_w, half_height) {
            // Blocked on X: try step-up if on ground
            if on_ground.0 {
                let step_up_pos = Vec3::new(new_x, transform.translation.y + 1.0, transform.translation.z);
                if !mob_collides_horizontal(&store, step_up_pos, half_w, half_height) {
                    // Can step up - move up and forward
                    transform.translation.x = new_x;
                    transform.translation.y += 1.0;
                    stepped_up = true;
                } else {
                    // Can't step up either, stop X movement
                    velocity.0.x = 0.0;
                }
            } else {
                velocity.0.x = 0.0;
            }
        } else {
            transform.translation.x = new_x;
        }

        // --- Z axis collision ---
        let new_z = transform.translation.z + velocity.0.z * dt;
        let proposed_z = Vec3::new(transform.translation.x, transform.translation.y, new_z);

        if mob_collides_horizontal(&store, proposed_z, half_w, half_height) {
            // Blocked on Z: try step-up if on ground and haven't already stepped up this frame
            if on_ground.0 && !stepped_up {
                let step_up_pos = Vec3::new(transform.translation.x, transform.translation.y + 1.0, new_z);
                if !mob_collides_horizontal(&store, step_up_pos, half_w, half_height) {
                    transform.translation.z = new_z;
                    transform.translation.y += 1.0;
                } else {
                    velocity.0.z = 0.0;
                }
            } else {
                velocity.0.z = 0.0;
            }
        } else {
            transform.translation.z = new_z;
        }

    }
}

/// Despawn mobs that fall into the void (below Y=-10).
pub fn despawn_void_mobs(
    mut commands: Commands,
    mobs: Query<(Entity, &Transform), With<Mob>>,
) {
    for (entity, transform) in &mobs {
        if transform.translation.y < -10.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn despawn_distant_mobs(
    mut commands: Commands,
    mobs: Query<(Entity, &Transform), With<Mob>>,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let player_pos = player_transform.translation;

    for (entity, transform) in &mobs {
        let dist = transform.translation.distance(player_pos);
        if dist > DESPAWN_DISTANCE {
            commands.entity(entity).despawn();
        }
    }
}
