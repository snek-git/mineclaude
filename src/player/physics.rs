use bevy::prelude::*;

use crate::world::chunk::CHUNK_SIZE;
use crate::world::manager::ChunkDataStore;

use super::{Player, Velocity, OnGround, Health, FallTracker, AirSupply, Hunger, PendingExhaustion, JustJumped, Sneaking, PLAYER_EYE_HEIGHT};

const GRAVITY: f32 = 20.0;
const JUMP_VELOCITY: f32 = 7.4;

pub fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &OnGround), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut vel, on_ground) in &mut query {
        if !on_ground.0 {
            vel.0.y -= GRAVITY * dt;
        }
    }
}

pub fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut tf, vel) in &mut query {
        tf.translation += vel.0 * dt;
    }
}

pub fn ground_collision(
    store: Res<ChunkDataStore>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut OnGround), With<Player>>,
) {
    for (mut tf, mut vel, mut on_ground) in &mut query {
        let feet_y = tf.translation.y;
        let check_y = (feet_y - 0.01).floor() as i32;

        // Check all blocks the player AABB overlaps on X/Z
        let min_bx = (tf.translation.x - PLAYER_HALF_WIDTH).floor() as i32;
        let max_bx = (tf.translation.x + PLAYER_HALF_WIDTH - 0.001).floor() as i32;
        let min_bz = (tf.translation.z - PLAYER_HALF_WIDTH).floor() as i32;
        let max_bz = (tf.translation.z + PLAYER_HALF_WIDTH - 0.001).floor() as i32;

        let mut solid_below = false;
        for bx in min_bx..=max_bx {
            for bz in min_bz..=max_bz {
                if is_block_solid(&store, bx, check_y, bz) {
                    solid_below = true;
                    break;
                }
            }
            if solid_below { break; }
        }

        if solid_below && vel.0.y <= 0.0 {
            let landing_y = (check_y + 1) as f32;
            if feet_y <= landing_y + 0.01 {
                tf.translation.y = landing_y;
                vel.0.y = 0.0;
                on_ground.0 = true;
            } else {
                on_ground.0 = false;
            }
        } else {
            on_ground.0 = false;
        }

        // Also check head collision (hitting ceiling while jumping)
        if vel.0.y > 0.0 {
            let head_y = tf.translation.y + PLAYER_HEIGHT_FULL;
            let head_check = head_y.floor() as i32;
            let mut hit_ceiling = false;
            for bx in min_bx..=max_bx {
                for bz in min_bz..=max_bz {
                    if is_block_solid(&store, bx, head_check, bz) {
                        hit_ceiling = true;
                        break;
                    }
                }
                if hit_ceiling { break; }
            }
            if hit_ceiling {
                tf.translation.y = head_check as f32 - PLAYER_HEIGHT_FULL - 0.001;
                vel.0.y = 0.0;
            }
        }

    }
}

pub fn jump(
    input: Res<ButtonInput<KeyCode>>,
    dead: Res<crate::ui::death_screen::PlayerDead>,
    mut query: Query<(&mut Velocity, &OnGround, &mut JustJumped), With<Player>>,
) {
    if dead.0 {
        return;
    }
    for (mut vel, on_ground, mut just_jumped) in &mut query {
        if input.just_pressed(KeyCode::Space) && on_ground.0 {
            vel.0.y = JUMP_VELOCITY;
            just_jumped.0 = true;
        }
    }
}

pub fn track_fall(
    mut query: Query<(&Transform, &OnGround, &mut FallTracker, &mut Health), With<Player>>,
    mut fall_audio: bevy::ecs::message::MessageWriter<crate::audio::FallDamageAudio>,
) {
    for (tf, on_ground, mut tracker, mut health) in &mut query {
        if !on_ground.0 {
            if tracker.fall_start_y.is_none() {
                tracker.fall_start_y = Some(tf.translation.y);
            }
        } else if let Some(start_y) = tracker.fall_start_y.take() {
            let distance = start_y - tf.translation.y;
            if distance > 3.0 {
                let damage = distance - 3.0;
                health.current = (health.current - damage).max(0.0);
                fall_audio.write(crate::audio::FallDamageAudio);
            }
        }
    }
}

/// Hunger constants
const EXHAUSTION_THRESHOLD: f32 = 4.0;
const HEALTH_REGEN_FOOD_THRESHOLD: f32 = 18.0;
const HEALTH_REGEN_INTERVAL: f32 = 4.0;
const HEALTH_REGEN_EXHAUSTION: f32 = 6.0;
const STARVATION_INTERVAL: f32 = 4.0;
const STARVATION_MIN_HEALTH: f32 = 1.0; // Normal mode: stop at half a heart

/// Resource to track hunger tick timers.
#[derive(Resource)]
pub struct HungerTimers {
    pub regen_timer: f32,
    pub starve_timer: f32,
}

impl Default for HungerTimers {
    fn default() -> Self {
        Self {
            regen_timer: 0.0,
            starve_timer: 0.0,
        }
    }
}

/// Processes exhaustion, saturation drain, food level drain, health regen, and starvation.
pub fn hunger_system(
    time: Res<Time>,
    mut timers: ResMut<HungerTimers>,
    mut pending: ResMut<PendingExhaustion>,
    mut query: Query<(&mut Hunger, &mut Health, &Velocity, &OnGround, &mut JustJumped), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    dead: Res<crate::ui::death_screen::PlayerDead>,
) {
    if dead.0 {
        return;
    }
    let dt = time.delta_secs();

    for (mut hunger, mut health, velocity, on_ground, mut just_jumped) in &mut query {
        // Drain pending exhaustion from other systems (block break, attack)
        if pending.0 > 0.0 {
            hunger.exhaustion += pending.0;
            pending.0 = 0.0;
        }

        // Accumulate exhaustion from sprinting (walking costs 0 in vanilla)
        let horizontal_speed = Vec3::new(velocity.0.x, 0.0, velocity.0.z).length();
        if horizontal_speed > 0.1 && on_ground.0 {
            let is_sprinting = input.pressed(KeyCode::ControlLeft) && hunger.food_level > 6.0;
            if is_sprinting {
                hunger.exhaustion += 0.1 * horizontal_speed * dt;
            }
        }

        // Jump exhaustion (once per jump via JustJumped flag)
        if just_jumped.0 {
            just_jumped.0 = false;
            let is_sprinting = input.pressed(KeyCode::ControlLeft) && hunger.food_level > 6.0;
            hunger.exhaustion += if is_sprinting { 0.2 } else { 0.05 };
        }

        // Process exhaustion threshold
        while hunger.exhaustion >= EXHAUSTION_THRESHOLD {
            hunger.exhaustion -= EXHAUSTION_THRESHOLD;
            if hunger.saturation > 0.0 {
                hunger.saturation = (hunger.saturation - 1.0).max(0.0);
            } else {
                hunger.food_level = (hunger.food_level - 1.0).max(0.0);
            }
        }

        // Health regen when food_level >= 18 (9+ drumsticks)
        if hunger.food_level >= HEALTH_REGEN_FOOD_THRESHOLD && health.current < health.max {
            timers.regen_timer += dt;
            if timers.regen_timer >= HEALTH_REGEN_INTERVAL {
                timers.regen_timer -= HEALTH_REGEN_INTERVAL;
                health.current = (health.current + 1.0).min(health.max);
                hunger.exhaustion += HEALTH_REGEN_EXHAUSTION;
            }
        } else {
            timers.regen_timer = 0.0;
        }

        // Starvation when food_level <= 0 (Normal mode: damage down to 1 heart / 2HP)
        if hunger.food_level <= 0.0 {
            timers.starve_timer += dt;
            if timers.starve_timer >= STARVATION_INTERVAL {
                timers.starve_timer -= STARVATION_INTERVAL;
                if health.current > STARVATION_MIN_HEALTH {
                    health.current = (health.current - 1.0).max(STARVATION_MIN_HEALTH);
                }
            }
        } else {
            timers.starve_timer = 0.0;
        }
    }
}

pub fn handle_death(
    dead: Res<crate::ui::death_screen::PlayerDead>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    if !dead.0 {
        return;
    }
    // Freeze player velocity while dead
    for mut vel in &mut query {
        vel.0 = Vec3::ZERO;
    }
}

pub fn drowning(
    time: Res<Time>,
    store: Res<ChunkDataStore>,
    mut query: Query<(&Transform, &mut AirSupply, &mut Health), With<Player>>,
) {
    let dt = time.delta_secs();
    for (tf, mut air, mut health) in &mut query {
        let eye_y = tf.translation.y + PLAYER_EYE_HEIGHT;
        let block_x = tf.translation.x.floor() as i32;
        let block_y = eye_y.floor() as i32;
        let block_z = tf.translation.z.floor() as i32;

        let block = get_block_at(&store, block_x, block_y, block_z);
        if block.is_liquid() {
            air.current = (air.current - dt).max(0.0);
            if air.current <= 0.0 {
                health.current = (health.current - 2.0 * dt).max(0.0);
            }
        } else {
            air.current = (air.current + 3.0 * dt).min(air.max);
        }
    }
}

const PLAYER_HALF_WIDTH: f32 = 0.3;
const PLAYER_HEIGHT_FULL: f32 = 1.8;

pub fn horizontal_collision(
    store: Res<ChunkDataStore>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    for (mut tf, mut vel) in &mut query {
        let pos = tf.translation;
        let min_by = pos.y.floor() as i32;
        let max_by = (pos.y + PLAYER_HEIGHT_FULL - 0.01).floor() as i32;

        // Resolve X axis — check all Z blocks the AABB spans
        let min_bz = (pos.z - PLAYER_HALF_WIDTH).floor() as i32;
        let max_bz = (pos.z + PLAYER_HALF_WIDTH - 0.001).floor() as i32;

        // Check -X face
        let neg_x_block = (pos.x - PLAYER_HALF_WIDTH).floor() as i32;
        let center_bx = pos.x.floor() as i32;
        if neg_x_block < center_bx {
            'neg_x: for by in min_by..=max_by {
                for bz in min_bz..=max_bz {
                    if is_block_solid(&store, neg_x_block, by, bz) {
                        tf.translation.x = (neg_x_block + 1) as f32 + PLAYER_HALF_WIDTH + 0.001;
                        vel.0.x = 0.0;
                        break 'neg_x;
                    }
                }
            }
        }
        // Check +X face
        let pos_x_block = (tf.translation.x + PLAYER_HALF_WIDTH).floor() as i32;
        let center_bx_new = tf.translation.x.floor() as i32;
        if pos_x_block > center_bx_new {
            'pos_x: for by in min_by..=max_by {
                for bz in min_bz..=max_bz {
                    if is_block_solid(&store, pos_x_block, by, bz) {
                        tf.translation.x = pos_x_block as f32 - PLAYER_HALF_WIDTH - 0.001;
                        vel.0.x = 0.0;
                        break 'pos_x;
                    }
                }
            }
        }

        // Resolve Z axis (use updated X position)
        let updated_x = tf.translation.x;
        let min_bx = (updated_x - PLAYER_HALF_WIDTH).floor() as i32;
        let max_bx = (updated_x + PLAYER_HALF_WIDTH - 0.001).floor() as i32;

        // Check -Z face
        let neg_z_block = (tf.translation.z - PLAYER_HALF_WIDTH).floor() as i32;
        let center_bz = tf.translation.z.floor() as i32;
        if neg_z_block < center_bz {
            'neg_z: for by in min_by..=max_by {
                for bx in min_bx..=max_bx {
                    if is_block_solid(&store, bx, by, neg_z_block) {
                        tf.translation.z = (neg_z_block + 1) as f32 + PLAYER_HALF_WIDTH + 0.001;
                        vel.0.z = 0.0;
                        break 'neg_z;
                    }
                }
            }
        }
        // Check +Z face
        let pos_z_block = (tf.translation.z + PLAYER_HALF_WIDTH).floor() as i32;
        let center_bz_new = tf.translation.z.floor() as i32;
        if pos_z_block > center_bz_new {
            'pos_z: for by in min_by..=max_by {
                for bx in min_bx..=max_bx {
                    if is_block_solid(&store, bx, by, pos_z_block) {
                        tf.translation.z = pos_z_block as f32 - PLAYER_HALF_WIDTH - 0.001;
                        vel.0.z = 0.0;
                        break 'pos_z;
                    }
                }
            }
        }
    }
}

fn get_block_at(store: &ChunkDataStore, x: i32, y: i32, z: i32) -> crate::block::BlockType {
    let size = CHUNK_SIZE as i32;
    let cx = x.div_euclid(size);
    let cy = y.div_euclid(size);
    let cz = z.div_euclid(size);
    let chunk_pos = IVec3::new(cx, cy, cz);

    let Some(chunk) = store.chunks.get(&chunk_pos) else {
        return crate::block::BlockType::Air;
    };

    let lx = x.rem_euclid(size) as usize;
    let ly = y.rem_euclid(size) as usize;
    let lz = z.rem_euclid(size) as usize;

    chunk.get(lx, ly, lz)
}

/// When sneaking on ground, prevent the player from walking off block edges.
pub fn sneak_edge_protection(
    store: Res<ChunkDataStore>,
    mut query: Query<(&mut Transform, &Sneaking, &OnGround), With<Player>>,
) {
    for (mut tf, sneaking, on_ground) in &mut query {
        if !sneaking.0 || !on_ground.0 {
            continue;
        }

        let pos = tf.translation;
        let check_y = (pos.y - 0.01).floor() as i32 - 1;

        // Check if any AABB corner would be over air; if so, clamp that axis
        let corners = [
            (pos.x - PLAYER_HALF_WIDTH, pos.z - PLAYER_HALF_WIDTH),
            (pos.x - PLAYER_HALF_WIDTH, pos.z + PLAYER_HALF_WIDTH - 0.001),
            (pos.x + PLAYER_HALF_WIDTH - 0.001, pos.z - PLAYER_HALF_WIDTH),
            (pos.x + PLAYER_HALF_WIDTH - 0.001, pos.z + PLAYER_HALF_WIDTH - 0.001),
        ];

        // Check if there's any solid ground under the player at all at check_y
        let has_any_ground = corners.iter().any(|(cx, cz)| {
            is_block_solid(&store, cx.floor() as i32, check_y, cz.floor() as i32)
        });

        if !has_any_ground {
            // Already fully over air (shouldn't happen normally), don't clamp
            continue;
        }

        // Clamp X: check if moving in X caused corners to go over air
        let min_x_over_air = !is_block_solid(&store, (pos.x - PLAYER_HALF_WIDTH).floor() as i32, check_y, (pos.z - PLAYER_HALF_WIDTH).floor() as i32)
            || !is_block_solid(&store, (pos.x - PLAYER_HALF_WIDTH).floor() as i32, check_y, (pos.z + PLAYER_HALF_WIDTH - 0.001).floor() as i32);
        let max_x_over_air = !is_block_solid(&store, (pos.x + PLAYER_HALF_WIDTH - 0.001).floor() as i32, check_y, (pos.z - PLAYER_HALF_WIDTH).floor() as i32)
            || !is_block_solid(&store, (pos.x + PLAYER_HALF_WIDTH - 0.001).floor() as i32, check_y, (pos.z + PLAYER_HALF_WIDTH - 0.001).floor() as i32);

        if min_x_over_air {
            let edge = (pos.x - PLAYER_HALF_WIDTH).floor() + 1.0;
            tf.translation.x = tf.translation.x.max(edge + PLAYER_HALF_WIDTH);
        }
        if max_x_over_air {
            let edge = (pos.x + PLAYER_HALF_WIDTH - 0.001).floor() as f32;
            tf.translation.x = tf.translation.x.min(edge + 0.001 - PLAYER_HALF_WIDTH);
        }

        // Re-read position after X clamp for Z checks
        let pos = tf.translation;
        let min_z_over_air = !is_block_solid(&store, (pos.x - PLAYER_HALF_WIDTH).floor() as i32, check_y, (pos.z - PLAYER_HALF_WIDTH).floor() as i32)
            || !is_block_solid(&store, (pos.x + PLAYER_HALF_WIDTH - 0.001).floor() as i32, check_y, (pos.z - PLAYER_HALF_WIDTH).floor() as i32);
        let max_z_over_air = !is_block_solid(&store, (pos.x - PLAYER_HALF_WIDTH).floor() as i32, check_y, (pos.z + PLAYER_HALF_WIDTH - 0.001).floor() as i32)
            || !is_block_solid(&store, (pos.x + PLAYER_HALF_WIDTH - 0.001).floor() as i32, check_y, (pos.z + PLAYER_HALF_WIDTH - 0.001).floor() as i32);

        if min_z_over_air {
            let edge = (pos.z - PLAYER_HALF_WIDTH).floor() + 1.0;
            tf.translation.z = tf.translation.z.max(edge + PLAYER_HALF_WIDTH);
        }
        if max_z_over_air {
            let edge = (pos.z + PLAYER_HALF_WIDTH - 0.001).floor() as f32;
            tf.translation.z = tf.translation.z.min(edge + 0.001 - PLAYER_HALF_WIDTH);
        }
    }
}

/// Deal 4 damage every 0.5 seconds when the player falls below Y=-10 (into the void).
pub fn void_damage(
    time: Res<Time>,
    mut timer: Local<f32>,
    mut query: Query<(&Transform, &mut Health), With<Player>>,
) {
    let dt = time.delta_secs();
    for (tf, mut health) in &mut query {
        if tf.translation.y < -10.0 {
            *timer += dt;
            while *timer >= 0.5 {
                *timer -= 0.5;
                health.current = (health.current - 4.0).max(0.0);
            }
        } else {
            *timer = 0.0;
        }
    }
}

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
