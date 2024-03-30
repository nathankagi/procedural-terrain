use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use std::error::Error;

use procedural_terrain::heightmap::{self, HeightMap, Meshable};

fn height_map_to_point_mesh(map: Vec<Vec<f64>>) -> Mesh {
    let mut positions: Vec<[f32; 3]> = vec![];

    for i in 0..map.len() {
        for j in 0..(map[0].len()) {
            positions.push([i as f32, map[i][j] as f32, j as f32]);
        }
    }

    Mesh::new(PrimitiveTopology::PointList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0, 1.0]; positions.len()])
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 0.0, 1.0]; positions.len()],
        )
}

fn height_map_to_triangle_mesh(map: Vec<Vec<f64>>) -> Mesh {
    let width = map.len();
    let depth = map[0].len();

    let triangle_count: usize = width * depth * 2 * 3;
    let vertex_count: usize = width * depth;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertex_count);

    // for (i, row) in map.iter().enumerate() {}

    // ===== old =====
    for d in 0..depth {
        for w in 0..width {
            let (w_f32, d_f32) = (w as f32, d as f32);
            positions.push([w_f32, map[w][d] as f32, d_f32]);

            normals.push([0.0, 1.0, 0.0]);
            uvs.push([w_f32 / (width as f32), d_f32 / (depth as f32)]);
        }
    }

    let mut triangles: Vec<u32> = Vec::with_capacity(triangle_count);

    for d in 0..depth {
        for w in 0..width {
            // first triangle
            triangles.push(((d * width) + w) as u32);
            triangles.push((((d + 1) * width) + w) as u32);
            triangles.push((((d + 1) * width) + (w + 1)) as u32);

            // second triangle
            triangles.push(((d * width) + w) as u32);
            triangles.push((((d + 1) * width) + (w + 1)) as u32);
            triangles.push(((d * width) + (w + 1)) as u32);
        }
    }

    // Mesh::new(PrimitiveTopology::TriangleStrip)
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(triangles)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    return mesh;
}

fn save_output(arr: Vec<Vec<f64>>) -> Result<(), Box<dyn Error>> {
    let file_path = "output.csv";

    // Create a CSV writer
    let mut writer = csv::Writer::from_path(file_path)?;

    // Iterate over each row in the data and write it to the CSV file
    for row in arr {
        writer.write_record(row.iter().map(|&f| f.to_string()))?;
    }

    writer.flush()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .run();

    Ok(())
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

#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // noise mesh
    let mesh = HeightMap::new(heightmap::create_height_map());
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh.triangle_mesh()),
            material: materials.add(Color::GRAY.into()),
            ..default()
        },
        Ground,
    ));

    // // light
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

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(250.0, 100.0, 250.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
