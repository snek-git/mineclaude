use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::player::{Player, PlayerYaw, PlayerPitch, Health, AirSupply, Hunger, ArmorSlots};

#[derive(Component)]
pub struct DebugText;

#[derive(Component)]
pub struct HeartIcon(pub usize);

pub fn spawn_crosshair(mut commands: Commands) {
    // Vertical bar
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(2.0),
            height: Val::Px(20.0),
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-1.0),
                top: Val::Px(-10.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ));
    // Horizontal bar
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(20.0),
            height: Val::Px(2.0),
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-10.0),
                top: Val::Px(-1.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::WHITE),
    ));
}

pub fn spawn_debug_text(mut commands: Commands) {
    commands.spawn((
        DebugText,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        },
        Text::new(""),
        TextFont::default().with_font_size(16.0),
        TextColor(Color::WHITE),
    ));
}

const HEART_SIZE: f32 = 16.0;
const HEART_GAP: f32 = 2.0;
const NUM_HEARTS: usize = 10;

pub fn spawn_health_bar(mut commands: Commands) {
    let half_hotbar = (9.0 * 40.0 + 8.0 * 2.0) / 2.0; // hotbar total width / 2

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(56.0),
            left: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(-half_hotbar),
                ..default()
            },
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(HEART_GAP),
            ..default()
        })
        .with_children(|parent| {
            for i in 0..NUM_HEARTS {
                parent.spawn((
                    HeartIcon(i),
                    Node {
                        width: Val::Px(HEART_SIZE),
                        height: Val::Px(HEART_SIZE),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.8, 0.1, 0.1)),
                ));
            }
        });
}

pub fn update_health_bar(
    player_q: Query<&Health, With<Player>>,
    mut heart_q: Query<(&HeartIcon, &mut BackgroundColor)>,
) {
    let Ok(health) = player_q.single() else {
        return;
    };

    for (heart, mut bg) in &mut heart_q {
        let threshold = (heart.0 as f32 + 1.0) * 2.0;
        if health.current >= threshold {
            *bg = BackgroundColor(Color::srgb(0.8, 0.1, 0.1));
        } else if health.current >= threshold - 1.0 {
            *bg = BackgroundColor(Color::srgb(0.6, 0.1, 0.1));
        } else {
            *bg = BackgroundColor(Color::srgb(0.2, 0.05, 0.05));
        }
    }
}

pub fn update_debug_text(
    diagnostics: Res<DiagnosticsStore>,
    player_q: Query<(&Transform, &PlayerYaw, &PlayerPitch), With<Player>>,
    mut text_q: Query<&mut Text, With<DebugText>>,
) {
    let Ok(mut text) = text_q.single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let (pos_str, facing_str) = if let Ok((tf, yaw, pitch)) = player_q.single() {
        let pos = tf.translation;
        let chunk_x = (pos.x.floor() as i32).div_euclid(16);
        let chunk_y = (pos.y.floor() as i32).div_euclid(16);
        let chunk_z = (pos.z.floor() as i32).div_euclid(16);

        let yaw_deg = yaw.0.to_degrees().rem_euclid(360.0);
        let facing = match yaw_deg as i32 {
            315..=360 | 0..=44 => "South",
            45..=134 => "West",
            135..=224 => "North",
            225..=314 => "East",
            _ => "?",
        };

        (
            format!(
                "XYZ: {:.1} / {:.1} / {:.1}\nChunk: {} {} {}",
                pos.x, pos.y, pos.z, chunk_x, chunk_y, chunk_z
            ),
            format!("Facing: {} ({:.1} / {:.1})", facing, yaw_deg, pitch.0.to_degrees()),
        )
    } else {
        (String::new(), String::new())
    };

    **text = format!("FPS: {:.0}\n{}\n{}", fps, pos_str, facing_str);
}

const BUBBLE_SIZE: f32 = 12.0;
const BUBBLE_GAP: f32 = 2.0;
const NUM_BUBBLES: usize = 10;

#[derive(Component)]
pub struct AirBubble(pub usize);

#[derive(Component)]
pub struct AirBar;

pub fn spawn_air_bar(mut commands: Commands) {
    let total_width = NUM_BUBBLES as f32 * BUBBLE_SIZE + (NUM_BUBBLES as f32 - 1.0) * BUBBLE_GAP;
    let half_hotbar = (9.0 * 40.0 + 8.0 * 2.0) / 2.0;

    commands
        .spawn((
            AirBar,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(76.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(half_hotbar - total_width),
                    ..default()
                },
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(BUBBLE_GAP),
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            for i in 0..NUM_BUBBLES {
                parent.spawn((
                    AirBubble(i),
                    Node {
                        width: Val::Px(BUBBLE_SIZE),
                        height: Val::Px(BUBBLE_SIZE),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.5, 0.9, 0.8)),
                    BorderColor::all(Color::srgba(0.1, 0.3, 0.7, 0.9)),
                ));
            }
        });
}

pub fn update_air_bar(
    player_q: Query<&AirSupply, With<Player>>,
    mut bar_q: Query<&mut Visibility, With<AirBar>>,
    mut bubble_q: Query<(&AirBubble, &mut BackgroundColor)>,
) {
    let Ok(air) = player_q.single() else {
        return;
    };

    let ratio = air.current / air.max;

    // Hide bar when air is full
    for mut vis in &mut bar_q {
        *vis = if ratio >= 1.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }

    // Update individual bubbles
    for (bubble, mut bg) in &mut bubble_q {
        let bubble_threshold = (bubble.0 as f32 + 1.0) / NUM_BUBBLES as f32;
        if ratio >= bubble_threshold {
            *bg = BackgroundColor(Color::srgba(0.2, 0.5, 0.9, 0.8));
        } else if ratio >= (bubble.0 as f32) / NUM_BUBBLES as f32 {
            *bg = BackgroundColor(Color::srgba(0.2, 0.5, 0.9, 0.4));
        } else {
            *bg = BackgroundColor(Color::srgba(0.1, 0.2, 0.3, 0.3));
        }
    }
}

// Hunger bar constants
const DRUMSTICK_SIZE: f32 = 16.0;
const DRUMSTICK_GAP: f32 = 2.0;
const NUM_DRUMSTICKS: usize = 10;

const FULL_DRUMSTICK_COLOR: Color = Color::srgb(0.55, 0.35, 0.1);
const HALF_DRUMSTICK_COLOR: Color = Color::srgb(0.4, 0.25, 0.08);
const EMPTY_DRUMSTICK_COLOR: Color = Color::srgb(0.15, 0.1, 0.05);

#[derive(Component)]
pub struct DrumstickIcon(pub usize);

pub fn spawn_hunger_bar(mut commands: Commands) {
    let total_width = NUM_DRUMSTICKS as f32 * DRUMSTICK_SIZE + (NUM_DRUMSTICKS as f32 - 1.0) * DRUMSTICK_GAP;
    let half_hotbar = (9.0 * 40.0 + 8.0 * 2.0) / 2.0; // hotbar total width / 2

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(56.0),
            left: Val::Percent(50.0),
            margin: UiRect {
                left: Val::Px(half_hotbar - total_width),
                ..default()
            },
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(DRUMSTICK_GAP),
            ..default()
        })
        .with_children(|parent| {
            for i in 0..NUM_DRUMSTICKS {
                parent.spawn((
                    DrumstickIcon(i),
                    Node {
                        width: Val::Px(DRUMSTICK_SIZE),
                        height: Val::Px(DRUMSTICK_SIZE),
                        ..default()
                    },
                    BackgroundColor(FULL_DRUMSTICK_COLOR),
                ));
            }
        });
}

pub fn update_hunger_bar(
    player_q: Query<&Hunger, With<Player>>,
    mut drumstick_q: Query<(&DrumstickIcon, &mut BackgroundColor)>,
) {
    let Ok(hunger) = player_q.single() else {
        return;
    };

    // Each drumstick represents 2 food points
    for (drumstick, mut bg) in &mut drumstick_q {
        let threshold = (drumstick.0 as f32 + 1.0) * 2.0;
        if hunger.food_level >= threshold {
            *bg = BackgroundColor(FULL_DRUMSTICK_COLOR);
        } else if hunger.food_level >= threshold - 1.0 {
            *bg = BackgroundColor(HALF_DRUMSTICK_COLOR);
        } else {
            *bg = BackgroundColor(EMPTY_DRUMSTICK_COLOR);
        }
    }
}

// Armor bar constants
const SHIELD_SIZE: f32 = 16.0;
const SHIELD_GAP: f32 = 2.0;
const NUM_SHIELDS: usize = 10;

const FULL_SHIELD_COLOR: Color = Color::srgb(0.7, 0.7, 0.75);
const HALF_SHIELD_COLOR: Color = Color::srgb(0.45, 0.45, 0.5);
const EMPTY_SHIELD_COLOR: Color = Color::srgb(0.15, 0.15, 0.18);

#[derive(Component)]
pub struct ShieldIcon(pub usize);

#[derive(Component)]
pub struct ArmorBar;

pub fn spawn_armor_bar(mut commands: Commands) {
    let half_hotbar = (9.0 * 40.0 + 8.0 * 2.0) / 2.0;

    commands
        .spawn((
            ArmorBar,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(76.0),
                left: Val::Percent(50.0),
                margin: UiRect {
                    left: Val::Px(-half_hotbar),
                    ..default()
                },
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(SHIELD_GAP),
                ..default()
            },
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            for i in 0..NUM_SHIELDS {
                parent.spawn((
                    ShieldIcon(i),
                    Node {
                        width: Val::Px(SHIELD_SIZE),
                        height: Val::Px(SHIELD_SIZE),
                        ..default()
                    },
                    BackgroundColor(EMPTY_SHIELD_COLOR),
                ));
            }
        });
}

pub fn update_armor_bar(
    player_q: Query<&ArmorSlots, With<Player>>,
    mut bar_q: Query<&mut Visibility, With<ArmorBar>>,
    mut shield_q: Query<(&ShieldIcon, &mut BackgroundColor)>,
) {
    let Ok(armor) = player_q.single() else {
        return;
    };

    let total = armor.total_armor_points() as f32;

    // Hide bar when armor is 0
    for mut vis in &mut bar_q {
        *vis = if total > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    // Each shield icon represents 2 armor points
    for (shield, mut bg) in &mut shield_q {
        let threshold = (shield.0 as f32 + 1.0) * 2.0;
        if total >= threshold {
            *bg = BackgroundColor(FULL_SHIELD_COLOR);
        } else if total >= threshold - 1.0 {
            *bg = BackgroundColor(HALF_SHIELD_COLOR);
        } else {
            *bg = BackgroundColor(EMPTY_SHIELD_COLOR);
        }
    }
}
