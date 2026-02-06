use bevy::prelude::*;

/// Tracks the current time of day as a value from 0.0 to 1.0.
/// 0.0 = sunrise, 0.25 = noon, 0.5 = sunset, 0.75 = midnight
#[derive(Resource)]
pub struct DayNightCycle {
    /// Current time of day, normalized 0.0..1.0
    pub time_of_day: f32,
    /// Duration of a full day cycle in seconds (default 120s for testing)
    pub cycle_duration: f32,
}

impl Default for DayNightCycle {
    fn default() -> Self {
        Self {
            time_of_day: 0.25, // Start at noon
            cycle_duration: 1200.0, // 20 minutes (similar to Minecraft)
        }
    }
}

/// Marker component for the sun directional light
#[derive(Component)]
pub struct Sun;

pub fn advance_time(time: Res<Time>, mut cycle: ResMut<DayNightCycle>) {
    let delta = time.delta_secs() / cycle.cycle_duration;
    cycle.time_of_day = (cycle.time_of_day + delta) % 1.0;
}

pub fn update_sun(
    cycle: Res<DayNightCycle>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
) {
    let Ok((mut transform, mut light)) = query.single_mut() else {
        return;
    };

    // Sun angle: full rotation over the day cycle
    // At time 0.0 (sunrise), sun is at the horizon (angle = 0)
    // At time 0.25 (noon), sun is overhead (angle = PI/2)
    // At time 0.5 (sunset), sun is at opposite horizon (angle = PI)
    // At time 0.75 (midnight), sun is below (angle = 3PI/2)
    let angle = cycle.time_of_day * std::f32::consts::TAU;

    // Rotate around the X axis to simulate sun arc across the sky
    // Add a slight Y rotation so shadows aren't perfectly aligned to one axis
    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        std::f32::consts::FRAC_PI_4 * 0.5, // slight Y offset
        -angle,
        0.0,
    );

    // Sun height determines intensity: sin of angle, positive = above horizon
    let sun_height = (angle).sin();

    // Illuminance: bright at noon, zero when below horizon
    light.illuminance = if sun_height > 0.0 {
        sun_height * 10000.0
    } else {
        0.0
    };

    // Color shifts: warm at sunrise/sunset, white at noon
    let warmth = 1.0 - sun_height.abs().min(1.0);
    let r = 1.0;
    let g = 0.95 - warmth * 0.15;
    let b = 0.85 - warmth * 0.3;
    light.color = Color::srgb(r, g, b);
}

pub fn update_ambient(
    cycle: Res<DayNightCycle>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    let angle = cycle.time_of_day * std::f32::consts::TAU;
    let sun_height = angle.sin();

    // Ambient is brighter during day, dimmer at night but never zero
    let day_brightness = if sun_height > 0.0 {
        100.0 + sun_height * 100.0
    } else {
        100.0 + sun_height * 60.0 // Dimmer at night, minimum ~40
    };

    ambient.brightness = day_brightness.max(40.0);

    // Slight blue tint at night, warmer during day
    if sun_height > 0.0 {
        ambient.color = Color::srgb(0.7, 0.75, 0.9);
    } else {
        ambient.color = Color::srgb(0.4, 0.5, 0.8);
    }
}
