use std::error::Error;

use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn fade(t: f64) -> f64 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    (a + t * (b - a)).into()
}

fn grad3d(hash: i32, x: f64, y: f64, z: f64) -> f64 {
    let input = hash & 0xF;
    match input {
        0x0 => return x + y,
        0x1 => return -x + y,
        0x2 => return x - y,
        0x3 => return -x - y,
        0x4 => return x + z,
        0x5 => return -x + z,
        0x6 => return x - z,
        0x7 => return -x - z,
        0x8 => return y + z,
        0x9 => return -y + z,
        0xA => return y - z,
        0xB => return -y - z,
        0xC => return y + x,
        0xD => return -y + z,
        0xE => return y - x,
        0xF => return -y - z,
        _ => return 0.0, // should never happen
    }
}

// pub fn grad3d(hash: i32, x: f64, y: f64, z: f64) -> f64 {
//     let h = hash & 15;
//     let u = if h < 8 { x } else { y };

//     let v = if h < 4 {
//         y
//     } else if h == 12 || h == 14 {
//         x
//     } else {
//         z
//     };

//     return if (h & 1) == 0 { u } else { -u } + if (h & 2) == 0 { v } else { -v };
// }

fn grad2d(hash: i32, x: f64, y: f64) -> f64 {
    let vec = vec![[0, 1], [0, -1], [1, 0], [-1, 0]];
    let index = hash % 4;
    let g = vec[index as usize];
    return (g[0] as f64 * x) + (g[1] as f64 * y);
}

fn perlin3d(x: f64, y: f64, z: f64, p: &Vec<i32>) -> f64 {
    let _x = x.floor() as usize & 255;
    let _y = y.floor() as usize & 255;
    let _z = z.floor() as usize & 255;

    let xf: f64 = x - _x as f64;
    let yf: f64 = y - _y as f64;
    let zf: f64 = z - _z as f64;

    let u = fade(xf);
    let v = fade(yf);
    let w = fade(zf);

    let aaa = p[p[p[_x] as usize + _y] as usize + _z];
    let aba = p[p[p[_x] as usize + _y + 1] as usize + _z];
    let aab = p[p[p[_x] as usize + _y] as usize + _z + 1];
    let abb = p[p[p[_x] as usize + _y + 1] as usize + _z + 1];
    let baa = p[p[p[_x + 1] as usize + _y] as usize + _z];
    let bba = p[p[p[_x + 1] as usize + _y + 1] as usize + _z];
    let bab = p[p[p[_x + 1] as usize + _y] as usize + _z + 1];
    let bbb = p[p[p[_x + 1] as usize + _y + 1] as usize + _z + 1];

    let x01 = lerp(
        grad3d(aaa, xf, yf, zf) as f64,
        grad3d(baa, xf - 1.0, yf, zf) as f64,
        u,
    );
    let x02 = lerp(
        grad3d(aba, xf, yf - 1.0, zf) as f64,
        grad3d(bba, xf - 1.0, yf - 1.0, zf) as f64,
        u,
    );
    let y1 = lerp(x01, x02, v);

    let x11 = lerp(
        grad3d(aab, xf, yf, zf - 1.0) as f64,
        grad3d(bab, xf - 1.0, yf, zf - 1.0) as f64,
        u,
    );
    let x12 = lerp(
        grad3d(abb, xf, yf - 1.0, zf - 1.0) as f64,
        grad3d(bbb, xf - 1.0, yf - 1.0, zf - 1.0) as f64,
        u,
    );
    let y2 = lerp(x11, x12, v);

    return lerp(y1, y2, w);

    // let a = p[_x] as usize + _y;
    // let aa = p[a] as usize + _z;
    // let ab = p[a + 1] as usize + _z;

    // let b = p[_x + 1] as usize + _y;
    // let ba = p[b] as usize + _z;
    // let bb = p[b + 1] as usize + _z;

    // return lerp(
    //     lerp(
    //         lerp(
    //             grad3d(p[aa], xf, yf, zf),
    //             grad3d(p[ba], xf - 1.0, yf, zf),
    //             u,
    //         ),
    //         lerp(
    //             grad3d(p[ab], xf, yf - 1.0, zf),
    //             grad3d(p[bb], xf - 1.0, yf - 1.0, zf),
    //             u,
    //         ),
    //         v,
    //     ),
    //     lerp(
    //         lerp(
    //             grad3d(p[aa + 1], xf, yf, zf - 1.0),
    //             grad3d(p[ba + 1], xf - 1.0, yf, zf - 1.0),
    //             u,
    //         ),
    //         lerp(
    //             grad3d(p[ab + 1], xf, yf - 1.0, zf - 1.0),
    //             grad3d(p[bb + 1], xf - 1.0, yf - 1.0, zf - 1.0),
    //             u,
    //         ),
    //         v,
    //     ),
    //     w,
    // );
}

fn perlin2d(x: f64, y: f64, p: &Vec<i32>) -> f64 {
    let _x = x.floor() as usize & 255;
    let _y = y.floor() as usize & 255;

    let xf = x - _x as f64;
    let yf = y - _y as f64;

    let u = fade(xf);
    let v = fade(yf);

    let aa = grad2d(p[p[_x] as usize + _y], xf, yf);
    let ab = grad2d(p[p[_x] as usize + _y + 1], xf, yf - 1.0);
    let bb = grad2d(p[p[_x + 1] as usize + _y + 1], xf - 1.0, yf - 1.0);
    let ba = grad2d(p[p[_x + 1] as usize + _y], xf - 1.0, yf);

    let x1 = lerp(aa, ba, u);
    let x2 = lerp(ab, bb, u);
    return lerp(x1, x2, v);
}

fn octave_perlin3d(
    x: f64,
    y: f64,
    z: f64,
    octaves: i32,
    persistence: f64,
    permutation: &Vec<i32>,
) -> f64 {
    let mut value = 0.0;
    let mut max_value = 1.0;

    for o in 0..octaves {
        let f = 2.0f64.powi(o);
        let amplitude = persistence.powi(o);

        max_value += amplitude;
        value += perlin3d(x * f, y * f, z * f, permutation) * amplitude;
    }
    return value / max_value;
}

fn octave_perlin2d(x: f64, y: f64, octaves: i32, persistence: f64, permutation: &Vec<i32>) -> f64 {
    let mut value = 0.0;
    let mut max_value = 1.0;

    for o in 0..octaves {
        let f = 2.0f64.powi(o);
        let amplitude = persistence.powi(o);

        max_value += amplitude;
        value += perlin2d(x * f, y * f, permutation) * amplitude;
    }
    return value / max_value;
}

fn generate_permutation(seed: u32) -> Vec<i32> {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed.into());
    let mut permutation: Vec<i32> = (0..256).collect();
    permutation.shuffle(&mut rng);

    let mut p: Vec<i32> = vec![];
    for i in 0..512 {
        p.push(permutation[i % 256]);
    }
    return p;
}

fn create_mesh() -> Mesh {
    Mesh::new(PrimitiveTopology::LineList)
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 5.0],
                [0.0, 0.0, 5.0],
                [5.0, 0.0, 5.0],
                [5.0, 0.0, 5.0],
                [5.0, 0.0, 0.0],
                [5.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![
                [0.0, 1.0],
                [0.0, 1.0],
                [0.0, 1.0],
                [0.0, 1.0],
                [0.0, 1.0],
                [0.0, 1.0],
                [0.0, 1.0],
                [0.0, 1.0],
            ],
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
        )
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
    let width = 1000;
    let height = 1000;

    let octaves = 4;
    let persistence: f64 = 0.4;

    let mut rng = rand::thread_rng();
    let seed = rng.gen::<u32>();
    let permutation = generate_permutation(seed);

    let mut noise_map = vec![vec![0.0; width]; height];
    for i in 0..height {
        for j in 0..width {
            noise_map[i][j] = octave_perlin3d(
                i as f64 / height as f64,
                j as f64 / width as f64,
                0.00001,
                octaves,
                persistence,
                &permutation,
            ) as f64;
        }
    }

    save_output(noise_map)?;

    // App::new()
    //     .add_plugins(DefaultPlugins)
    //     .add_systems(Startup, setup)
    //     .add_systems(Update, draw_cursor)
    //     .run();

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
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(create_mesh()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },
        Ground,
    ));

    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
