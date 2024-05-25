use crate::noise;
use bevy::render::{
    mesh::{Indices, Mesh},
    render_asset::RenderAssetUsages,
    render_resource::PrimitiveTopology,
};
use nalgebra::Vector3;
use rand::Rng;
use std::{error::Error, usize};

pub trait Meshable {
    fn triangle_mesh(&self) -> Mesh;
}

pub trait CSV {
    fn save(&self) -> Result<(), Box<dyn Error>>;
}

pub trait PNG {
    fn save(&self) -> Result<(), Box<dyn Error>>;
}

pub struct HeightMap {
    pub map: Vec<Vec<f64>>,
}

pub fn create_height_map() -> Vec<Vec<f64>> {
    let width = 20;
    let height = 20;
    let scale = 200.0;

    let octaves = 6;
    let persistence: f64 = 0.5;

    let mut rng = rand::thread_rng();
    let seed = rng.gen::<u32>();
    let permutation = noise::generate_permutation(seed);

    let mut noise_map = vec![vec![0.0; width]; height];
    for i in 0..height {
        for j in 0..width {
            noise_map[i][j] = noise::octave_perlin3d(
                i as f64 / height as f64,
                j as f64 / width as f64,
                0.0,
                octaves,
                persistence,
                &permutation,
            ) as f64
                * scale;
        }
    }

    return noise_map;
}

impl HeightMap {
    pub fn new(
        width: usize,
        height: usize,
        scale: f64,
        octaves: i32,
        persistence: f64,
    ) -> HeightMap {
        let mut rng = rand::thread_rng();
        let seed = rng.gen::<u32>();
        let permutation = noise::generate_permutation(seed);

        let mut noise_map = vec![vec![0.0; width]; height];
        for i in 0..height {
            for j in 0..width {
                noise_map[i][j] = noise::octave_perlin3d(
                    i as f64 / height as f64,
                    j as f64 / width as f64,
                    0.0,
                    octaves,
                    persistence,
                    &permutation,
                ) as f64
                    * scale;
            }
        }

        return HeightMap { map: noise_map };
    }

    pub fn vertices(&self) -> usize {
        return (self.length() + 1) * (self.width() + 1);
    }

    pub fn length(&self) -> usize {
        return self.map.len();
    }
    pub fn width(&self) -> usize {
        return self.map[0].len();
    }
}

impl Meshable for HeightMap {
    // triangle list mesh
    fn triangle_mesh(&self) -> Mesh {
        let height = self.length();
        let width = self.width();

        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(width * height);

        let normals = calculate_normals(&self.map);
        let indices = generate_indices(width, height);
        let vertices = generate_vertices(&self.map);

        // for (i, row) in map.iter().enumerate() {}
        for y in 0..width {
            for x in 0..height {
                uvs.push([x as f32 / width as f32, y as f32 / height as f32]);
            }
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

impl CSV for HeightMap {
    fn save(&self) -> Result<(), Box<dyn Error>> {
        let file_path = "output.csv";

        let mut writer = csv::Writer::from_path(file_path)?;

        for row in &self.map {
            writer.write_record(row.iter().map(|&f| f.to_string()))?;
        }

        writer.flush()?;

        Ok(())
    }
}

fn calculate_normals(height_map: &Vec<Vec<f64>>) -> Vec<[f32; 3]> {
    let width = height_map.len();
    let height = height_map[0].len();
    let mut normals: Vec<[f32; 3]> = Vec::new();

    for y in 0..width {
        for x in 0..height {
            let height_l = if x > 0 {
                height_map[x - 1][y]
            } else {
                height_map[x][y]
            };

            let height_r = if x < width - 1 {
                height_map[x + 1][y]
            } else {
                height_map[x][y]
            };

            let height_d = if y > 0 {
                height_map[x][y - 1]
            } else {
                height_map[x][y]
            };

            let height_u = if y < height - 1 {
                height_map[x][y + 1]
            } else {
                height_map[x][y]
            };

            let normal = Vector3::new(height_l - height_r, 2.0, height_d - height_u).normalize();
            normals.push([normal.x as f32, normal.y as f32, normal.z as f32]);
        }
    }

    normals
}

fn generate_indices(width: usize, height: usize) -> Vec<u32> {
    let mut indices: Vec<u32> = Vec::new();

    for y in 0..width - 1 {
        for x in 0..height - 1 {
            let top_left = (x + y * width) as u32;
            let top_right = ((x + 1) + y * width) as u32;
            let bottom_left = (x + (y + 1) * width) as u32;
            let bottom_right = ((x + 1) + (y + 1) * width) as u32;

            indices.push(top_left);
            indices.push(bottom_left);
            indices.push(top_right);

            indices.push(top_right);
            indices.push(bottom_left);
            indices.push(bottom_right);
        }
    }

    indices
}

fn generate_vertices(height_map: &Vec<Vec<f64>>) -> Vec<[f32; 3]> {
    let width = height_map.len();
    let height = height_map[0].len();
    let mut vertices: Vec<[f32; 3]> = Vec::new();

    for y in 0..width {
        for x in 0..height {
            vertices.push([x as f32, height_map[x][y] as f32, y as f32]);
        }
    }

    vertices
}
