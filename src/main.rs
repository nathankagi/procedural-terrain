use bevy::prelude::*;
use procedural_terrain::heightmap::{HeightMap, Meshable};
use bevy::render::{
    mesh::{Indices, Mesh},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};
use procedural_terrain::noise;
use nalgebra::{Vector2, Vector3};
use rand::Rng;
use std::{error::Error, usize};

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
    let size = 5000;
    let octaves = 8;
    let persistence = 0.5;

    let width = size;
    let height = size;
    let scale = size as f32;

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(
            -(size as f32) / 2.0 as f32,
            size as f32,
            -(size as f32) / 2.0,
        )
        .looking_at(
            Vec3::new((size as f32) / 2.0, 0.0, (size as f32) / 2.0),
            Vec3::Y,
        ),
        ..Default::default()
    });

    // Terrain Mesh
    let mut heighmap = HeightMap::new(size, size);

    let mut rng = rand::thread_rng();
    let seed = rng.gen::<u32>();
    let permutation = noise::generate_permutation(seed);

    for i in 0..height {
        for j in 0..width {
            heighmap.map[i][j] = noise::octave_perlin3d(
                i as f32 / height as f32,
                j as f32 / width as f32,
                0.0,
                octaves,
                persistence,
                &permutation,
            ) as f32
                * scale;
        }
    }

    let meshed = heighmap.triangle_mesh();

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_indices(Indices::U32(meshed.indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, meshed.vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, meshed.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, meshed.uvs);

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
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
    ambient_light.brightness = 200.0;
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
        transform: Transform::from_translation(Vec3::new(500.0, 1000.0, 500.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
}
