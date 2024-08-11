use crate::noise;
use nalgebra::{Vector2, Vector3};
use rand::Rng;
use rayon::prelude::*;
use std::{error::Error, usize};

pub trait Meshable {
    fn triangle_mesh(&self) -> Mesh;
}

pub trait CSV {
    fn save(&self) -> Result<(), Box<dyn Error>>;
    fn load() -> Result<HeightMap, Box<dyn Error>>;
}

pub trait PNG {
    fn save(&self) -> Result<(), Box<dyn Error>>;
    fn load() -> Result<HeightMap, Box<dyn Error>>;
}

pub struct HeightMap {
    pub map: Vec<Vec<f32>>,
}

pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub values: Vec<f32>,
}

pub fn create_height_map() -> Vec<Vec<f32>> {
    let width = 20;
    let height = 20;
    let scale = 200.0;

    let octaves = 6;
    let persistence: f32 = 0.5;

    let mut rng = rand::thread_rng();
    let seed = rng.gen::<u32>();
    let permutation = noise::generate_permutation(seed);

    let mut noise_map = vec![vec![0.0; width]; height];
    for i in 0..height {
        for j in 0..width {
            noise_map[i][j] = noise::octave_perlin3d(
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

    return noise_map;
}

impl HeightMap {
    pub fn new(width: usize, height: usize) -> HeightMap {
        return HeightMap {
            map: vec![vec![0.0; width]; height],
        };
    }

    pub fn load(values: &[Vec<f32>]) -> HeightMap {
        HeightMap {
            map: values.to_vec(),
        }
    }

    pub fn width(&self) -> usize {
        return self.map.len();
    }
    pub fn height(&self) -> usize {
        return self.map[0].len();
    }
}

impl Meshable for HeightMap {
    fn triangle_mesh(&self) -> Mesh {
        let height = self.height();
        let width = self.width();

        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(width * height);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(width * height);
        let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(width * height);
        let mut indices: Vec<u32> = Vec::new();

        for y in 0..width {
            for x in 0..height {
                // UVs
                uvs.push([x as f32 / width as f32, y as f32 / height as f32]);

                // Normals
                let height_l = if x > 0 {
                    self.map[x - 1][y]
                } else {
                    self.map[x][y]
                };

                let height_r = if x < width - 1 {
                    self.map[x + 1][y]
                } else {
                    self.map[x][y]
                };

                let height_d = if y > 0 {
                    self.map[x][y - 1]
                } else {
                    self.map[x][y]
                };

                let height_u = if y < height - 1 {
                    self.map[x][y + 1]
                } else {
                    self.map[x][y]
                };

                let normal =
                    Vector3::new(height_l - height_r, 2.0, height_d - height_u).normalize();
                normals.push([normal.x as f32, normal.y as f32, normal.z as f32]);

                // Indices
                if x < (height - 1) && y < (width - 1) {
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

                // Vertices
                vertices.push([x as f32, self.map[x][y] as f32, y as f32]);
            }
        }

        Mesh {
            vertices: vertices,
            normals: normals,
            uvs: uvs,
            indices: indices,
            values: vec![0.0; width * height],
        }
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

    fn load() -> Result<HeightMap, Box<dyn Error>> {
        return Ok(HeightMap::new(0, 0));
    }
}

fn calculate_normals(height_map: &Vec<Vec<f32>>) -> Vec<[f32; 3]> {
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

fn generate_vertices(height_map: &Vec<Vec<f32>>) -> Vec<[f32; 3]> {
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
