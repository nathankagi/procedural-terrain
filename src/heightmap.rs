use bevy::render::{mesh::Mesh, render_resource::PrimitiveTopology};
use rand::Rng;

use crate::noise;

pub trait Meshable {
    fn triangle_mesh(&self) -> Mesh;
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
    pub fn new(map: Vec<Vec<f64>>) -> HeightMap {
        return HeightMap { map };
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
        let depth = self.length();
        let width = self.width();

        let triangle_count: usize = width * depth * 2 * 3;
        let vertex_count: usize = width * depth;

        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertex_count);
        let mut triangles: Vec<u32> = Vec::with_capacity(triangle_count);

        // for (i, row) in map.iter().enumerate() {}

        for d in 0..depth {
            for w in 0..width {
                // Push vertex positions, normals, and UVs
                positions.push([w as f32, self.map[w][d] as f32, d as f32]);
                normals.push([0.0, 1.0, 0.0]);
                uvs.push([w as f32 / width as f32, d as f32 / depth as f32]);

                // Define triangle indices
                if d < depth - 1 && w < width - 1 {
                    let index = (d * width + w) as u32;
                    let next_row_index = ((d + 1) * width + w) as u32;
                    let next_column_index = (d * width + w + 1) as u32;
                    let next_row_column_index = ((d + 1) * width + w + 1) as u32;

                    // First triangle
                    triangles.push(index);
                    triangles.push(next_row_index);
                    triangles.push(next_row_column_index);

                    // Second triangle
                    triangles.push(index);
                    triangles.push(next_row_column_index);
                    triangles.push(next_column_index);
                }
            }
        }

        // let mut mesh = Mesh::new(PrimitiveTopology::PointList);
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(triangles)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

        return mesh;
    }
}
