use bevy::prelude::*;
use bevy::render::settings::{Backends, RenderCreation, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::render::{
    mesh::{Indices, Mesh},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};
use rand::Rng;
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use procedural_terrain::heightmap::HeightMap;
use procedural_terrain::heightmap::{self};
use procedural_terrain::heightmaps;
use procedural_terrain::mesh::Meshable;
use procedural_terrain::{noise, terrain};

#[derive(Component)]
struct Terrain {
    size: usize,
    mesh_handle: Handle<Mesh>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                // backends: Some(Backends::VULKAN),
                ..default()
            }),
            ..default()
        }))
        // .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, setup_lights, setup_ambient_light))
        // .add_systems(Update, (update_terrain))
        .run();
}

// Setup functions
fn tests(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
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
            size as f32,
            -(size as f32) / 2.0,
        )
        .looking_at(
            Vec3::new((size as f32) / 2.0, 0.0, (size as f32) / 2.0),
            Vec3::Y,
        ),
        ..default()
    });

    // Terrain Mesh

    // let mut terrain = terrain::Terrain::new(size, size);

    let params = heightmaps::dla::DiffusionLimitedAggregationParams {
        height: 3,
        width: 3,
        spawns: vec![heightmaps::dla::Point::new(1, 1)],
        t: 0.7,
        particles: 5,
        layers: 8,
        density: 1.0,
        kernel: heightmaps::dla::Kernel {
            size: 3,
            value: 10.0,
            k_type: heightmaps::dla::KernelType::Gaussian,
        },
    };

    let _map = heightmaps::dla::generate(params);
    let dla_heightmap = heightmap::HeightMap { map: _map };

    let mut rng = rand::thread_rng();
    let seed = rng.gen::<u32>();
    let params = heightmap::FractalPerlinParams {
        height: dla_heightmap.height(),
        width: dla_heightmap.width(),
        scale: 100.0,
        octaves: 12,
        persistence: 0.5,
        seed: seed,
    };

    let p_heightmap = heightmap::generate(heightmap::Algorithms::FractalPerlin(params.clone()));
    let heightmap: HeightMap = dla_heightmap + p_heightmap;
    // let heightmap = dla_heightmap;

    let meshed = heightmap.mesh_triangles();

    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_indices(Indices::U32(meshed.indices))
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, meshed.vertices)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, meshed.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, meshed.uvs);

    let mesh_handle = meshes.add(mesh);

    commands.spawn(
        (Terrain {
            size: size,
            mesh_handle: mesh_handle.clone(),
        }),
    );

    commands.spawn(PbrBundle {
        // mesh: meshes.add(mesh_handle),
        mesh: mesh_handle,
        material: materials.add(StandardMaterial {
            base_color: Color::GRAY,
            perceptual_roughness: 1.0,
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });

    // commands.insert_resource(TerrainMeshHandle(mesh_handle));
}

fn setup_ambient_light(mut ambient_light: ResMut<AmbientLight>) {
    // ambient_light.brightness = 1000.0;
}

fn setup_lights(mut commands: Commands) {
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 30_000_000_000.0,
    //         range: 10_000.0,
    //         radius: 10_000.0,
    //         shadows_enabled: true,
    //         ..Default::default()
    //     },
    //     transform: Transform::from_translation(Vec3::new(500.0, 1000.0, 500.0))
    //         .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    //     ..Default::default()
    // });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: light_consts::lux::AMBIENT_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-500.0, 1000.0, 500.0))
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });
}

// Update functions
// fn update_terrain(
//     time: Res<Time>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut query: Query<&mut Terrain>,
// ) {
//     for terrain in query.iter_mut() {
//         if let Some(mesh) = meshes.get_mut(terrain.mesh_handle.clone()) {
//             let mut heightmap = HeightMap::new(terrain.size, terrain.size);
//             let z = time.elapsed_seconds() / 100.0;

//             let height = terrain.size;
//             let width = terrain.size;
//             let scale = terrain.size as f32;

//             heightmap
//                 .map
//                 .par_iter_mut()
//                 .enumerate()
//                 .for_each(|(i, row)| {
//                     row.iter_mut().enumerate().for_each(|(j, elem)| {
//                         *elem = noise::octave_perlin3d(
//                             i as f32 / height as f32,
//                             j as f32 / width as f32,
//                             z,
//                             terrain.octaves,
//                             terrain.persistence,
//                             &terrain.permutation,
//                         ) as f32
//                             * scale;
//                     });
//                 });

//             let meshed = heightmap.mesh_triangles();

//             mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, meshed.vertices);
//             mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, meshed.normals);
//         }
//     }
// }
