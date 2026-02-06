use bevy::prelude::*;
use bevy::ecs::message::{Message, MessageReader, MessageWriter};

/// Message fired when a block is broken.
#[derive(Message)]
pub struct BlockBreakAudio;

/// Message fired when a block is placed.
#[derive(Message)]
pub struct BlockPlaceAudio;

/// Message fired for a footstep sound.
#[derive(Message)]
pub struct FootstepAudio;

/// Message fired when the player takes damage.
#[derive(Message)]
pub struct PlayerHurtAudio;

/// Message fired when the player takes fall damage.
#[derive(Message)]
pub struct FallDamageAudio;

/// Message fired when a mob is hurt (carries mob type for sound selection).
#[derive(Message)]
pub struct MobHurtAudio {
    pub is_zombie: bool,
}

/// Message fired when a mob dies.
#[derive(Message)]
pub struct MobDeathAudio;

/// Message fired when the player swings a weapon/attack.
#[derive(Message)]
pub struct SwordSwingAudio;

/// Message fired when the player picks up a dropped item.
#[derive(Message)]
pub struct ItemPickupAudio;

/// Stores preloaded sound effect handles.
#[derive(Resource)]
struct SoundEffects {
    break_sound: Handle<AudioSource>,
    place_sound: Handle<AudioSource>,
    footstep_sound: Handle<AudioSource>,
    player_hurt: Handle<AudioSource>,
    fall_damage: Handle<AudioSource>,
    zombie_hurt: Handle<AudioSource>,
    skeleton_hurt: Handle<AudioSource>,
    mob_death: Handle<AudioSource>,
    sword_swing: Handle<AudioSource>,
    item_pickup: Handle<AudioSource>,
}

/// Timer to throttle footstep sounds.
#[derive(Resource)]
struct FootstepTimer(Timer);

pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BlockBreakAudio>()
            .add_message::<BlockPlaceAudio>()
            .add_message::<FootstepAudio>()
            .add_message::<PlayerHurtAudio>()
            .add_message::<FallDamageAudio>()
            .add_message::<MobHurtAudio>()
            .add_message::<MobDeathAudio>()
            .add_message::<SwordSwingAudio>()
            .add_message::<ItemPickupAudio>()
            .add_systems(Startup, load_sounds)
            .add_systems(
                Update,
                (
                    play_break_sound,
                    play_place_sound,
                    footstep_detector,
                    play_footstep_sound,
                    play_player_hurt_sound,
                    play_fall_damage_sound,
                    play_mob_hurt_sound,
                    play_mob_death_sound,
                    play_sword_swing_sound,
                    play_item_pickup_sound,
                ),
            );
    }
}

fn load_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(SoundEffects {
        break_sound: asset_server.load("sounds/break.ogg"),
        place_sound: asset_server.load("sounds/place.ogg"),
        footstep_sound: asset_server.load("sounds/footstep.ogg"),
        player_hurt: asset_server.load("sounds/player_hurt.ogg"),
        fall_damage: asset_server.load("sounds/fall_damage.ogg"),
        zombie_hurt: asset_server.load("sounds/zombie_hurt.ogg"),
        skeleton_hurt: asset_server.load("sounds/skeleton_hurt.ogg"),
        mob_death: asset_server.load("sounds/mob_death.ogg"),
        sword_swing: asset_server.load("sounds/sword_swing.ogg"),
        item_pickup: asset_server.load("sounds/place.ogg"), // reuse place sound for pickup pop
    });
    commands.insert_resource(FootstepTimer(Timer::from_seconds(0.4, TimerMode::Repeating)));
}

fn play_break_sound(
    mut commands: Commands,
    mut messages: MessageReader<BlockBreakAudio>,
    sounds: Res<SoundEffects>,
) {
    for _ in messages.read() {
        commands.spawn((
            AudioPlayer::new(sounds.break_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn play_place_sound(
    mut commands: Commands,
    mut messages: MessageReader<BlockPlaceAudio>,
    sounds: Res<SoundEffects>,
) {
    for _ in messages.read() {
        commands.spawn((
            AudioPlayer::new(sounds.place_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

/// Detects when the player is moving on the ground and fires FootstepAudio messages.
fn footstep_detector(
    time: Res<Time>,
    mut timer: ResMut<FootstepTimer>,
    query: Query<
        (&crate::player::Velocity, &crate::player::OnGround),
        With<crate::player::Player>,
    >,
    mut footstep_writer: MessageWriter<FootstepAudio>,
) {
    let Ok((vel, on_ground)) = query.single() else {
        return;
    };

    // Only play footsteps when on ground and moving horizontally
    let horizontal_speed = Vec2::new(vel.0.x, vel.0.z).length();
    if on_ground.0 && horizontal_speed > 0.5 {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            footstep_writer.write(FootstepAudio);
        }
    } else {
        timer.0.reset();
    }
}

fn play_footstep_sound(
    mut commands: Commands,
    mut messages: MessageReader<FootstepAudio>,
    sounds: Res<SoundEffects>,
) {
    for _ in messages.read() {
        commands.spawn((
            AudioPlayer::new(sounds.footstep_sound.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn play_player_hurt_sound(
    mut commands: Commands,
    mut messages: MessageReader<PlayerHurtAudio>,
    sounds: Res<SoundEffects>,
) {
    for _ in messages.read() {
        commands.spawn((
            AudioPlayer::new(sounds.player_hurt.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn play_fall_damage_sound(
    mut commands: Commands,
    mut messages: MessageReader<FallDamageAudio>,
    sounds: Res<SoundEffects>,
) {
    for _ in messages.read() {
        commands.spawn((
            AudioPlayer::new(sounds.fall_damage.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn play_mob_hurt_sound(
    mut commands: Commands,
    mut messages: MessageReader<MobHurtAudio>,
    sounds: Res<SoundEffects>,
) {
    for msg in messages.read() {
        let handle = if msg.is_zombie {
            sounds.zombie_hurt.clone()
        } else {
            sounds.skeleton_hurt.clone()
        };
        commands.spawn((
            AudioPlayer::new(handle),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn play_mob_death_sound(
    mut commands: Commands,
    mut messages: MessageReader<MobDeathAudio>,
    sounds: Res<SoundEffects>,
) {
    for _ in messages.read() {
        commands.spawn((
            AudioPlayer::new(sounds.mob_death.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn play_sword_swing_sound(
    mut commands: Commands,
    mut messages: MessageReader<SwordSwingAudio>,
    sounds: Res<SoundEffects>,
) {
    for _ in messages.read() {
        commands.spawn((
            AudioPlayer::new(sounds.sword_swing.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

fn play_item_pickup_sound(
    mut commands: Commands,
    mut messages: MessageReader<ItemPickupAudio>,
    sounds: Res<SoundEffects>,
) {
    for _ in messages.read() {
        commands.spawn((
            AudioPlayer::new(sounds.item_pickup.clone()),
            PlaybackSettings {
                speed: 1.5, // higher pitch pop for pickup
                ..PlaybackSettings::DESPAWN
            },
        ));
    }
}
