use nalgebra::Vector3;

use crate::heightmaps::lib::HeightMap;
use crate::mesh::{Mesh, Meshable};

const MAX_LAYER_COUNT: usize = 100;

pub struct Terrain {
    width: usize,
    height: usize,
    cells: Vec<Vec<Cell>>,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Material {
    erosion: f32,
    cohesion: f32,
    saturation: f32,
    permeability: f32,
    mass: f32,
}

#[derive(Copy, Clone)]
pub struct Layer {
    height: f32,
    material: Material,
}

#[derive(Clone)]
pub struct Cell {
    layers: Vec<Layer>,
    layer_index: usize,
}

#[derive(Copy, Clone)]
pub struct Chunk {}

impl Terrain {
    pub fn new(width: usize, height: usize) -> Self {
        Terrain {
            width,
            height,
            cells: vec![vec![Cell::new(); width]; height],
        }
    }

    pub fn at(&mut self, x: usize, y: usize) -> &mut Cell {
        return &mut self.cells[x][y];
    }

    pub fn height(&self, x: usize, y: usize) -> f32 {
        return self.cells[x][y].height();
    }

    pub fn material(&self, x: usize, y: usize) -> Material {
        return self.cells[x][y].layers.last().unwrap().material;
    }

    pub fn normal(&self, x: usize, y: usize) -> Vector3<f32> {
        let height_l = if x > 0 {
            self.cells[x - 1][y].height()
        } else {
            self.cells[x][y].height()
        };

        let height_r = if x < self.width - 1 {
            self.cells[x + 1][y].height()
        } else {
            self.cells[x][y].height()
        };

        let height_d = if y > 0 {
            self.cells[x][y - 1].height()
        } else {
            self.cells[x][y].height()
        };

        let height_u = if y < self.height - 1 {
            self.cells[x][y + 1].height()
        } else {
            self.cells[x][y].height()
        };

        return Vector3::new(height_l - height_r, 2.0, height_d - height_u).normalize();
    }

    pub fn gradient(&self, x: usize, y: usize) -> Vector3<f32> {
        let norm: Vector3<f32> = self.normal(x, y);
        return Vector3::new(-norm.x, 0.0, -norm.z);
    }

    pub fn add(&mut self, x: usize, y: usize, layer: Layer) {
        self.cells[x][y].add(layer);
    }

    pub fn remove(&mut self, x: usize, y: usize, height: f32) -> Layer {
        return self.cells[x][y].remove(height);
    }

    pub fn add_heightmap(&mut self, map: HeightMap) {
        // add to terrain data from a heightmap
    }
}

impl Cell {
    pub fn new() -> Self {
        Cell {
            layers: Vec::new(),
            layer_index: 0,
        }
    }

    pub fn add(&mut self, layer: Layer) {
        if self.layers[self.layer_index].material == layer.material {
            self.layers[self.layer_index].height =
                self.layers[self.layer_index].height + layer.height;
        } else {
            self.layers.push(layer);
            self.layer_index = self.layers.len();
        }
    }

    pub fn remove(&mut self, height: f32) -> Layer {
        return if height < self.layers[self.layer_index].height {
            self.layers[self.layer_index].height = self.layers[self.layer_index].height - height;
            let mut o = self.layers[self.layer_index].clone();
            o.height = height;
            o
        } else {
            let o = self.layers[self.layer_index].clone();
            self.layers.remove(self.layer_index);
            self.layer_index = self.layers.len();
            o
        };
    }

    pub fn height(&self) -> f32 {
        let mut h: f32 = 0.0;
        for layer in self.layers.iter() {
            h = h + layer.height;
        }
        return h;
    }

    pub fn depth(&self) -> usize {
        return self.layers.len();
    }
}

impl Layer {
    pub fn new() -> Self {
        Layer {
            ..Default::default()
        }
    }
}

impl Layer {}

impl Default for Layer {
    fn default() -> Self {
        Layer {
            height: 0.0,
            material: Material {
                ..Default::default()
            },
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Material {
            erosion: 0.0,
            cohesion: 0.0,
            saturation: 0.0,
            permeability: 0.0,
            mass: 0.0,
        }
    }
}

impl Meshable for Terrain {
    fn mesh_triangles(&self) -> Mesh {
        let height = self.height;
        let width = self.width;

        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(width * height);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(width * height);
        let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(width * height);
        let mut indices: Vec<u32> = Vec::new();

        for y in 0..width {
            for x in 0..height {
                // UVs
                uvs.push([x as f32 / width as f32, y as f32 / height as f32]);

                // Normals
                let normal = self.normal(x, y);
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
                vertices.push([x as f32, self.height(x, y) as f32, y as f32]);
            }
        }

        Mesh {
            vertices: vertices,
            normals: normals,
            uvs: uvs,
            indices: indices,
        }
    }

    fn remesh_triangles(&mut self, mesh: &mut Mesh, modified: Vec<(u32, u32)>) {
        for (x, y) in modified {
            let idx = (y * self.width as u32 + x) as usize;

            let normal = self.normal(x as usize, y as usize);
            mesh.normals[idx] = [normal.x as f32, normal.y as f32, normal.z as f32];

            mesh.vertices[idx] = [x as f32, self.height(x as usize, y as usize), y as f32];
        }
    }
}
