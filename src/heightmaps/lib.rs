use nalgebra::Vector3;
use std::ops::Add;

use crate::{
    heightmaps,
    mesh::{Mesh, Meshable},
};

const NORMAL_Y_COMPONENT: f32 = 2.0;

pub struct HeightMap {
    pub map: Vec<Vec<f32>>,
}

pub enum Algorithms {
    FractalPerlin(heightmaps::perlin::FractalPerlinParams),
    GradientFractalPerlin(heightmaps::perlin::GradientFractalPerlinParams),
    DiffusionLimitedAggregation(heightmaps::dla::DiffusionLimitedAggregationParams),
}

impl HeightMap {
    pub fn new(width: usize, height: usize) -> HeightMap {
        return HeightMap {
            map: vec![vec![0.0; width]; height],
        };
    }

    pub fn generate(algorithm: Algorithms) -> HeightMap {
        match algorithm {
            Algorithms::FractalPerlin(fractal_perlin_params) => {
                generate_fractal_perlin(fractal_perlin_params)
            }
            Algorithms::GradientFractalPerlin(gradient_fractal_perlin_params) => {
                generate_gradient_frac_perlin(gradient_fractal_perlin_params)
            }
            Algorithms::DiffusionLimitedAggregation(diffusion_limited_aggregation_params) => {
                generate_diff_lim_agg(diffusion_limited_aggregation_params)
            }
        }
    }

    pub fn load(values: &[Vec<f32>]) -> HeightMap {
        HeightMap {
            map: values.to_vec(),
        }
    }

    pub fn width(&self) -> usize {
        return self.map[0].len();
    }

    pub fn height(&self) -> usize {
        return self.map.len();
    }
}

impl Add<HeightMap> for HeightMap {
    type Output = HeightMap;

    fn add(self, other: HeightMap) -> HeightMap {
        assert!(self.height() == other.height());
        assert!(self.width() == other.width());
        let mut _h = HeightMap::new(self.width(), self.height());

        for y in 0..self.height() {
            for x in 0..self.width() {
                _h.map[y][x] = self.map[y][x] + other.map[y][x];
            }
        }

        return _h;
    }
}

impl Meshable for HeightMap {
    fn mesh_triangles(&mut self) -> Mesh {
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
                    Vector3::new(height_l - height_r, NORMAL_Y_COMPONENT, height_d - height_u)
                        .normalize();
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
            vertices,
            normals,
            uvs,
            indices,
        }
    }

    fn remesh_triangles(&mut self, mesh: &mut Mesh, modified: Vec<(u32, u32)>) {
        self.mesh_triangles();
    }
}

fn generate_fractal_perlin(params: heightmaps::perlin::FractalPerlinParams) -> HeightMap {
    // let mut rng = rand::thread_rng();
    // let seed = rng.gen::<u32>();
    let permutation = heightmaps::perlin::generate_permutation(params.seed);

    let mut hmap = HeightMap::new(params.width, params.height);
    for i in 0..params.height {
        for j in 0..params.width {
            hmap.map[i][j] = heightmaps::perlin::octave_perlin3d(
                i as f32 / params.height as f32,
                j as f32 / params.width as f32,
                0.0,
                params.octaves,
                params.persistence,
                &permutation,
            ) as f32
                * params.scale;
        }
    }

    return hmap;
}

fn generate_gradient_frac_perlin(
    params: heightmaps::perlin::GradientFractalPerlinParams,
) -> HeightMap {
    HeightMap::new(10, 10)
}

fn generate_diff_lim_agg(params: heightmaps::dla::DiffusionLimitedAggregationParams) -> HeightMap {
    HeightMap::new(10, 10)
}
