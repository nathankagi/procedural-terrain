use bevy::prelude::*;
use procedural_terrain::heightmap::{HeightMap, Meshable};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, setup_lights, setup_ambient_light))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size = 1000;

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(
            -(size as f32) / 2.0 as f32,
            size as f32 / 2.0,
            -(size as f32) / 2.0,
        )
        .looking_at(
            Vec3::new((size as f32) / 2.0, 0.0, (size as f32) / 2.0),
            Vec3::Y,
        ),
        ..Default::default()
    });

    let mesh = HeightMap::new(size, size, size as f64, 8, 0.5);

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh.triangle_mesh()),
        material: materials.add(StandardMaterial {
            base_color: Color::GRAY,
            perceptual_roughness: 1.0,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}

fn setup_ambient_light(mut ambient_light: ResMut<AmbientLight>) {
    ambient_light.brightness = 100.0;
}

fn setup_lights(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 30_000_000_000.0,
            range: 10_000.0,
            radius: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(500.0, 500.0, 0.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
}
