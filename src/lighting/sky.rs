use bevy::prelude::*;

use super::day_night::DayNightCycle;

/// Updates the ClearColor resource to simulate sky color changes throughout the day.
pub fn update_sky_color(cycle: Res<DayNightCycle>, mut clear_color: ResMut<ClearColor>) {
    let angle = cycle.time_of_day * std::f32::consts::TAU;
    // sun_height > 0 means daytime, < 0 means nighttime
    let sun_height = angle.sin();

    // Define key sky colors
    let day_color = Vec3::new(0.45, 0.65, 1.0); // bright blue
    let sunset_color = Vec3::new(0.9, 0.5, 0.2); // warm orange
    let night_color = Vec3::new(0.02, 0.02, 0.08); // very dark blue

    let sky = if sun_height > 0.15 {
        // Full daytime - blend from sunset toward blue as sun gets higher
        let t = ((sun_height - 0.15) / 0.85).clamp(0.0, 1.0);
        sunset_color.lerp(day_color, t)
    } else if sun_height > -0.1 {
        // Sunrise/sunset transition zone (-0.1 to 0.15)
        let t = ((sun_height + 0.1) / 0.25).clamp(0.0, 1.0);
        night_color.lerp(sunset_color, t)
    } else {
        // Night
        let t = ((sun_height + 0.1) / -0.9).clamp(0.0, 1.0);
        night_color.lerp(Vec3::new(0.01, 0.01, 0.04), t)
    };

    clear_color.0 = Color::srgb(sky.x, sky.y, sky.z);
}

/// Marker for the sun disc entity
#[derive(Component)]
pub struct SunDisc;

/// Marker for the moon disc entity
#[derive(Component)]
pub struct MoonDisc;

pub fn setup_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Sun disc - a bright emissive sphere placed far away
    commands.spawn((
        SunDisc,
        Mesh3d(meshes.add(Sphere::new(20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.95, 0.4),
            emissive: LinearRgba::new(10.0, 9.0, 3.0, 1.0),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, 500.0, 0.0)),
    ));

    // Moon disc - a dimmer sphere on the opposite side
    commands.spawn((
        MoonDisc,
        Mesh3d(meshes.add(Sphere::new(15.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.85, 0.85, 0.95),
            emissive: LinearRgba::new(1.5, 1.5, 2.0, 1.0),
            unlit: true,
            ..default()
        })),
        Transform::from_translation(Vec3::new(0.0, -500.0, 0.0)),
    ));
}

/// Moves the sun and moon discs to follow the day/night cycle, anchored to the camera.
pub fn update_sky_bodies(
    cycle: Res<DayNightCycle>,
    camera_query: Query<&Transform, With<Camera3d>>,
    mut sun_query: Query<&mut Transform, (With<SunDisc>, Without<Camera3d>, Without<MoonDisc>)>,
    mut moon_query: Query<&mut Transform, (With<MoonDisc>, Without<Camera3d>, Without<SunDisc>)>,
) {
    let Ok(cam_transform) = camera_query.single() else {
        return;
    };
    let cam_pos = cam_transform.translation;

    let angle = cycle.time_of_day * std::f32::consts::TAU;
    let sky_radius = 500.0;

    // Sun position: follows the same rotation as the directional light
    // Y-axis offset matches update_sun in day_night.rs
    let y_offset = std::f32::consts::FRAC_PI_4 * 0.5;
    let sun_dir = Vec3::new(
        -angle.cos() * y_offset.sin(),
        angle.sin(),
        -angle.cos() * y_offset.cos(),
    )
    .normalize();

    if let Ok(mut sun_tf) = sun_query.single_mut() {
        sun_tf.translation = cam_pos + sun_dir * sky_radius;
    }

    // Moon is opposite the sun
    if let Ok(mut moon_tf) = moon_query.single_mut() {
        moon_tf.translation = cam_pos - sun_dir * sky_radius;
    }
}
