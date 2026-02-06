pub mod controller;
pub mod interaction;
pub mod physics;

use bevy::prelude::*;

const WALK_SPEED: f32 = 4.317;
const SPRINT_SPEED: f32 = WALK_SPEED * 1.3;
const SNEAK_SPEED: f32 = WALK_SPEED * 0.3;
const PLAYER_EYE_HEIGHT: f32 = 1.62;

/// The player's respawn location. Defaults to world origin.
#[derive(Resource)]
pub struct SpawnPoint(pub Vec3);

impl Default for SpawnPoint {
    fn default() -> Self {
        Self(Vec3::new(0.0, 80.0, 0.0))
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_player, interaction::setup_break_overlay, interaction::setup_block_highlight))
            .init_resource::<SpawnPoint>()
            .init_resource::<interaction::BreakingState>()
            .init_resource::<physics::HungerTimers>()
            .init_resource::<PendingExhaustion>()
            .add_systems(
                Update,
                (
                    controller::cursor_grab,
                    controller::mouse_look,
                    controller::player_movement,
                    physics::apply_gravity,
                    physics::jump,
                    physics::apply_velocity
                        .after(physics::apply_gravity)
                        .after(physics::jump)
                        .after(controller::player_movement),
                    physics::ground_collision.after(physics::apply_velocity),
                    physics::horizontal_collision.after(physics::apply_velocity),
                    physics::sneak_edge_protection.after(physics::horizontal_collision),
                    physics::track_fall.after(physics::ground_collision),
                    physics::drowning,
                    physics::hunger_system.after(controller::player_movement),
                    physics::void_damage,
                    crate::ui::death_screen::detect_death
                        .after(physics::track_fall)
                        .after(physics::drowning)
                        .after(physics::hunger_system)
                        .after(physics::void_damage),
                    physics::handle_death
                        .after(crate::ui::death_screen::detect_death),
                    controller::apply_yaw_to_transform.after(controller::mouse_look),
                ),
            )
            .init_resource::<interaction::RightClickConsumed>()
            .init_resource::<interaction::MobHitThisFrame>()
            .init_resource::<interaction::AttackCooldown>()
            .add_systems(
                Update,
                (
                    interaction::attack_mob,
                    interaction::break_block
                        .after(interaction::attack_mob),
                    interaction::block_interact
                        .in_set(interaction::BlockInteractSet),
                    interaction::eat_food
                        .after(interaction::BlockInteractSet),
                    interaction::hoe_interact
                        .after(interaction::BlockInteractSet)
                        .after(interaction::eat_food),
                    interaction::plant_seeds
                        .after(interaction::hoe_interact),
                    interaction::place_block
                        .after(interaction::BlockInteractSet)
                        .after(interaction::eat_food)
                        .after(interaction::plant_seeds),
                    interaction::update_block_highlight,
                ),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
pub struct Velocity(pub Vec3);

#[derive(Component, Default)]
pub struct OnGround(pub bool);

#[derive(Component, Default)]
pub struct PlayerYaw(pub f32);

#[derive(Component, Default)]
pub struct JustJumped(pub bool);

#[derive(Component, Default)]
pub struct Sneaking(pub bool);

/// Player's 4 armor slots: [helmet, chestplate, leggings, boots].
/// Each slot: Option<(Item, count=1, remaining_durability)>.
#[derive(Component)]
pub struct ArmorSlots {
    pub slots: [Option<(crate::inventory::item::Item, u8, u16)>; 4],
}

impl Default for ArmorSlots {
    fn default() -> Self {
        Self { slots: [None; 4] }
    }
}

impl ArmorSlots {
    /// Total armor defense points from all worn pieces.
    pub fn total_armor_points(&self) -> u8 {
        self.slots.iter().filter_map(|s| s.as_ref()).map(|(item, _, _)| item.armor_points()).sum()
    }

    /// Damage all worn armor pieces by 1 durability each. Returns true if any piece broke.
    pub fn damage_all_pieces(&mut self) -> bool {
        let mut any_broke = false;
        for slot in self.slots.iter_mut() {
            if let Some((_, _, dur)) = slot {
                if *dur > 0 {
                    *dur -= 1;
                    if *dur == 0 {
                        *slot = None;
                        any_broke = true;
                    }
                }
            }
        }
        any_broke
    }
}

#[derive(Component, Default)]
pub struct PlayerPitch(pub f32);

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Default for Health {
    fn default() -> Self {
        Self { current: 20.0, max: 20.0 }
    }
}

#[derive(Component, Default)]
pub struct FallTracker {
    pub fall_start_y: Option<f32>,
}

#[derive(Component)]
pub struct AirSupply {
    pub current: f32,
    pub max: f32,
}

impl Default for AirSupply {
    fn default() -> Self {
        Self { current: 10.0, max: 10.0 }
    }
}

/// Accumulates exhaustion from systems that can't easily query Hunger (e.g. break_block, attack_mob).
/// Drained each frame by hunger_system.
#[derive(Resource, Default)]
pub struct PendingExhaustion(pub f32);

/// Tracks player hunger. food_level 0-20, saturation 0-20 (hidden buffer), exhaustion accumulator.
#[derive(Component)]
pub struct Hunger {
    pub food_level: f32,
    pub saturation: f32,
    pub exhaustion: f32,
}

impl Default for Hunger {
    fn default() -> Self {
        Self {
            food_level: 20.0,
            saturation: 5.0,
            exhaustion: 0.0,
        }
    }
}

fn spawn_player(mut commands: Commands, mut spawn_point: ResMut<SpawnPoint>) {
    let save = crate::save::persistence::load_player();

    let (pos, yaw, pitch, health, air, food_level, saturation) = if let Some(ref data) = save {
        // Restore spawn point from save
        if let (Some(&sx), Some(&sy), Some(&sz)) = (data.spawn_x.as_ref(), data.spawn_y.as_ref(), data.spawn_z.as_ref()) {
            spawn_point.0 = Vec3::new(sx, sy, sz);
        }
        (
            Vec3::new(data.position[0], data.position[1], data.position[2]),
            data.yaw,
            data.pitch,
            data.health,
            data.air_supply,
            data.food_level.unwrap_or(20.0),
            data.saturation.unwrap_or(5.0),
        )
    } else {
        (Vec3::new(0.0, 80.0, 0.0), 0.0, 0.0, 20.0, 10.0, 20.0, 5.0)
    };

    // Load armor from save
    let armor = if let Some(ref data) = save {
        if let Some(ref armor_data) = data.armor_slots {
            let mut slots = [None; 4];
            for (i, slot) in armor_data.iter().enumerate().take(4) {
                slots[i] = *slot;
            }
            ArmorSlots { slots }
        } else {
            ArmorSlots::default()
        }
    } else {
        ArmorSlots::default()
    };

    commands
        .spawn((
            Player,
            Velocity::default(),
            OnGround(false),
            PlayerYaw(yaw),
            PlayerPitch(pitch),
            Health { current: health, max: 20.0 },
            FallTracker::default(),
            AirSupply { current: air, max: 10.0 },
            Hunger { food_level, saturation, exhaustion: 0.0 },
            JustJumped::default(),
            Sneaking::default(),
            armor,
            Transform::from_xyz(pos.x, pos.y, pos.z),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Projection::Perspective(PerspectiveProjection {
                    fov: 80.0_f32.to_radians(),
                    ..default()
                }),
                Transform::from_xyz(0.0, PLAYER_EYE_HEIGHT, 0.0),
            ));
        });
}
