use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::inventory::chest::ChestOpen;
use crate::inventory::crafting::CraftingTableOpen;
use crate::inventory::furnace::FurnaceOpen;
use crate::ui::death_screen::PlayerDead;
use crate::ui::inventory_screen::InventoryOpen;
use crate::ui::main_menu::InMainMenu;
use crate::ui::pause_menu::PauseState;

use super::{Player, PlayerYaw, PlayerPitch};

const MOUSE_SENSITIVITY: f32 = 0.003;

fn any_ui_open(
    in_menu: &InMainMenu,
    pause: &PauseState,
    inventory_open: &InventoryOpen,
    ct_open: &CraftingTableOpen,
    furnace_open: &FurnaceOpen,
    chest_open: &ChestOpen,
    dead: &PlayerDead,
) -> bool {
    in_menu.0 || pause.0 || inventory_open.0 || ct_open.0 || furnace_open.0.is_some() || chest_open.0.is_some() || dead.0
}

pub fn mouse_look(
    motion: Res<AccumulatedMouseMotion>,
    in_menu: Res<InMainMenu>,
    inventory_open: Res<InventoryOpen>,
    pause: Res<PauseState>,
    ct_open: Res<CraftingTableOpen>,
    furnace_open: Res<FurnaceOpen>,
    chest_open: Res<ChestOpen>,
    dead: Res<PlayerDead>,
    mut player_q: Query<(&mut PlayerYaw, &mut PlayerPitch, &Children), With<Player>>,
    mut camera_q: Query<&mut Transform, With<Camera3d>>,
) {
    if any_ui_open(&in_menu, &pause, &inventory_open, &ct_open, &furnace_open, &chest_open, &dead) {
        return;
    }

    let delta = motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    for (mut yaw, mut pitch, children) in &mut player_q {
        yaw.0 -= delta.x * MOUSE_SENSITIVITY;
        pitch.0 = (pitch.0 - delta.y * MOUSE_SENSITIVITY).clamp(
            -89.0_f32.to_radians(),
            89.0_f32.to_radians(),
        );

        for child in children.iter() {
            if let Ok(mut cam_tf) = camera_q.get_mut(child) {
                cam_tf.rotation = Quat::from_rotation_x(pitch.0);
            }
        }
    }
}

pub fn player_movement(
    input: Res<ButtonInput<KeyCode>>,
    in_menu: Res<InMainMenu>,
    pause: Res<PauseState>,
    inventory_open: Res<InventoryOpen>,
    ct_open: Res<CraftingTableOpen>,
    furnace_open: Res<FurnaceOpen>,
    chest_open: Res<ChestOpen>,
    dead: Res<PlayerDead>,
    mut query: Query<(&mut super::Velocity, &PlayerYaw, &super::Hunger, &mut super::Sneaking), With<Player>>,
) {
    if any_ui_open(&in_menu, &pause, &inventory_open, &ct_open, &furnace_open, &chest_open, &dead) {
        return;
    }

    for (mut velocity, yaw, hunger, mut sneaking) in &mut query {
        let mut dir = Vec3::ZERO;

        let forward = Vec3::new(-yaw.0.sin(), 0.0, -yaw.0.cos());
        let right = Vec3::new(-forward.z, 0.0, forward.x);

        if input.pressed(KeyCode::KeyW) {
            dir += forward;
        }
        if input.pressed(KeyCode::KeyS) {
            dir -= forward;
        }
        if input.pressed(KeyCode::KeyA) {
            dir -= right;
        }
        if input.pressed(KeyCode::KeyD) {
            dir += right;
        }

        let dir = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            dir
        };

        let is_sneaking = input.pressed(KeyCode::ShiftLeft);
        sneaking.0 = is_sneaking;

        // Sneak overrides sprint; can't sprint when food_level <= 6
        let speed = if is_sneaking {
            super::SNEAK_SPEED
        } else if input.pressed(KeyCode::ControlLeft) && hunger.food_level > 6.0 {
            super::SPRINT_SPEED
        } else {
            super::WALK_SPEED
        };

        velocity.0.x = dir.x * speed;
        velocity.0.z = dir.z * speed;
    }
}

pub fn cursor_grab(
    mouse: Res<ButtonInput<MouseButton>>,
    inventory_open: Res<InventoryOpen>,
    pause: Res<PauseState>,
    in_menu: Res<InMainMenu>,
    ct_open: Res<CraftingTableOpen>,
    furnace_open: Res<FurnaceOpen>,
    chest_open: Res<ChestOpen>,
    dead: Res<PlayerDead>,
    mut cursor_q: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    // Don't grab cursor when any UI screen is open
    if any_ui_open(&in_menu, &pause, &inventory_open, &ct_open, &furnace_open, &chest_open, &dead) {
        return;
    }

    let Ok(mut cursor) = cursor_q.single_mut() else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
}

pub fn apply_yaw_to_transform(
    mut query: Query<(&mut Transform, &PlayerYaw), With<Player>>,
) {
    for (mut tf, yaw) in &mut query {
        tf.rotation = Quat::from_rotation_y(yaw.0);
    }
}
