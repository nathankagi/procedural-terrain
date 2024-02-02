use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;
use image::{DynamicImage, GenericImage, Rgba};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

fn fade(t: f64) -> f64 {
    return (6.0 * t.powi(5) - 15.0 * t.powi(4) + 1.0 * t.powi(3)).into();
}

fn lerp(t: f64, a: f64, b: f64) -> f64 {
    (a + t * (b - a)).into()
}

fn grad(hash: i32, x: f64, y: f64, z: f64) -> f64 {
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

fn perlin(x: f64, y: f64, z: f64, p: &Vec<i32>) -> f64 {
    let _x = (x.floor() as usize % p.len()) & 255;
    let _y = (y.floor() as usize % p.len()) & 255;
    let _z = (z.floor() as usize % p.len()) & 255;

    let xf: f64 = x - _x as f64;
    let yf: f64 = x - _y as f64;
    let zf: f64 = x - _z as f64;

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

    let x1 = lerp(
        grad(aaa, xf, yf, zf) as f64,
        grad(baa, xf - 1.0, yf, zf) as f64,
        u,
    );
    let x2 = lerp(
        grad(aba, xf, yf - 1.0, zf) as f64,
        grad(bba, xf - 1.0, yf - 1.0, zf) as f64,
        u,
    );
    let y1 = lerp(x1, x2, v);

    let x1 = lerp(
        grad(aab, xf, yf, zf - 1.0) as f64,
        grad(bab, xf - 1.0, yf, zf - 1.0) as f64,
        u,
    );
    let x2 = lerp(
        grad(abb, xf, yf - 1.0, zf - 1.0) as f64,
        grad(bbb, xf - 1.0, yf - 1.0, zf - 1.0) as f64,
        u,
    );
    let y2 = lerp(x1, x2, v);

    return (lerp(y1, y2, w) + 1.0) / 2.0;
}

fn octave_perlin(
    x: f64,
    y: f64,
    z: f64,
    octaves: i32,
    persistence: f64,
    scale: f64,
    permutation: &Vec<i32>,
) -> f64 {
    let mut value = 0.0;
    for o in 0..octaves {
        let frequency = 2.0f64.powi(o);
        let amplitude = persistence.powi(o);
        value += perlin(
            x * scale * frequency,
            y * scale * frequency,
            z * scale * frequency,
            &permutation,
        ) * amplitude;
    }
    return value;
}

fn generate_permutation(seed: u64) -> Vec<i32> {
    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
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

fn main() {
    let width = 512;
    let height = 512;

    let scale = 0.1;
    let octaves = 6;
    let persistence: f64 = 0.5;

    let mut rng = rand::thread_rng();
    let seed = rng.gen::<u64>();

    let permutation = generate_permutation(seed);

    let mut noise_map = vec![vec![0.0; height]; width];
    for i in 0..width {
        for j in 0..height {
            // let mut value = 0.0;
            // for o in 0..octaves {
            //     let frequency = 2.0f64.powi(o);
            //     let amplitude = persistence.powi(o);
            //     value += perlin(
            //         i as f64 * scale * frequency,
            //         // j as f64 * scale * frequency,
            //         0.0,
            //         0.0,
            //         &permutation,
            //     ) * amplitude;
            // }
            noise_map[i][j] = octave_perlin(
                i as f64 + rng.gen::<f64>(),
                j as f64 + rng.gen::<f64>(),
                0.0,
                octaves,
                persistence,
                scale,
                &permutation,
            );
        }
    }

    let heightmap: Vec<Vec<f64>> = noise_map;

    // Set the dimensions of your image
    let width = heightmap[0].len();
    let height = heightmap.len();

    // Create a new dynamic image
    let mut img = DynamicImage::new_rgb8(width as u32, height as u32);

    // Convert the heightmap to image pixels
    for y in 0..height {
        for x in 0..width {
            let height_value = heightmap[y][x];

            // Map the height value to a grayscale color
            let color_value = (height_value * 255.0) as u8;

            // Create an Rgba pixel with grayscale color
            let pixel = Rgba([color_value, color_value, color_value, 255]);

            // Put the pixel in the image at the specified position
            img.put_pixel(x as u32, y as u32, pixel);
        }
    }

    // Save the image to a file
    img.save("output.png").expect("Failed to save image");

    // App::new()
    //     .add_plugins(DefaultPlugins)
    //     .add_systems(Startup, setup)
    //     .add_systems(Update, draw_cursor)
    //     .run();
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
