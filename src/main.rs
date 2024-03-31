use bevy::prelude::*;
use std::error::Error;

use procedural_terrain::heightmap::{HeightMap, Meshable};

fn main() -> Result<(), Box<dyn Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, spawn_camera))
        .add_systems(Update, (update_system, draw_cursor))
        .run();

    Ok(())
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct OrbitalCamera;

// Startup
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ground mesh
    let ground_mesh = HeightMap::new(200, 200, 200.0, 6, 0.5);

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(ground_mesh.triangle_mesh()),
            material: materials.add(Color::GRAY.into()),
            ..default()
        },
        Ground,
    ));

    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(250.0, 50.0, 250.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            color: Color::WHITE,
            shadows_enabled: true,
            illuminance: 50000.0,
            ..default()
        },
        ..default()
    });
}

fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(0.0, 100.0, 300.0);

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        OrbitalCamera {},
    ));
}

// Update
fn update_system(
    mut camera_query: Query<&mut Transform, With<OrbitalCamera>>,
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let time_now = time.elapsed_seconds();
    let time_delta = time.delta_seconds();
    let mut camera_transform = camera_query.single_mut();

    let mut x_delta = 0.0;
    let mut z_delta = 0.0;
    let scaler = 10.0;

    if keys.pressed(KeyCode::W) {
        z_delta = z_delta - time_delta;
    }
    if keys.pressed(KeyCode::S) {
        z_delta = z_delta + time_delta;
    }
    if keys.pressed(KeyCode::A) {
        x_delta = x_delta - time_delta;
    }
    if keys.pressed(KeyCode::D) {
        x_delta = x_delta + time_delta;
    }

    camera_transform.translation += Vec3::new(x_delta * scaler, 0.0, z_delta * scaler);
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    ground_query: Query<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let ground = ground_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the ground plane.
    let Some(distance) = ray.intersect_plane(ground.translation(), ground.up()) else {
        return;
    };
    let point = ray.get_point(distance);

    // Draw a circle just above the ground plane at that position.
    gizmos.circle(point + ground.up() * 0.01, ground.up(), 0.2, Color::WHITE);
}
