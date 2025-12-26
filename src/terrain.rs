use std::collections::HashMap;

pub const CHUNK_SIZE: usize = 64;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MaterialID(pub u16);

pub struct MaterialRegistry {
    materials: Vec<Material>,
}

pub struct Terrain {
    chunks: HashMap<(i32, i32), Chunk>,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Material {
    pub erosion: f32,
    pub cohesion: f32,
    pub saturation: f32,
    pub permeability: f32,
    pub mass: f32,
}

#[derive(Copy, Clone)]
pub struct Layer {
    thickness: f32,
    material_id: MaterialID,
}

#[derive(Clone)]
pub struct Cell {
    layers: Vec<Layer>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SurfaceCell {
    pub height: f32,
    pub material_id: MaterialID,
}

#[derive(Clone)]
pub struct Chunk {
    pub position: (i32, i32),
    pub cells: Vec<Cell>,
}

impl MaterialRegistry {
    pub fn get(&self, id: MaterialID) -> &Material {
        &self.materials[id.0 as usize]
    }
}

impl Terrain {
    pub fn cell_mut(&mut self, wx: i32, wy: i32) -> &mut Cell {
        let cx = wx.div_euclid(CHUNK_SIZE as i32);
        let cy = wy.div_euclid(CHUNK_SIZE as i32);

        let lx = wx.rem_euclid(CHUNK_SIZE as i32) as usize;
        let ly = wy.rem_euclid(CHUNK_SIZE as i32) as usize;

        let chunk = self
            .chunks
            .entry((cx, cy))
            .or_insert_with(|| Chunk::new((cx, cy)));

        chunk.cell_mut(lx, ly)
    }
}

impl Cell {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn total_height(&self) -> f32 {
        self.layers.iter().map(|l| l.thickness).sum()
    }

    pub fn surface_layer(&self) -> Option<&Layer> {
        self.layers.last()
    }

    pub fn surface_material(&self) -> Option<MaterialID> {
        self.surface_layer().map(|l| l.material_id)
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    pub fn deposit(&mut self, thickness: f32, material_id: MaterialID) {
        if let Some(top) = self.layers.last_mut() {
            if top.material_id == material_id {
                top.thickness += thickness;
                return;
            }
        }

        self.layers.push(Layer {
            thickness,
            material_id,
        });
    }

    pub fn erode(&mut self, mut amount: f32) -> Vec<Layer> {
        let mut removed = Vec::new();

        while amount > 0.0 {
            let Some(top) = self.layers.last_mut() else {
                break;
            };

            if top.thickness > amount {
                top.thickness -= amount;
                removed.push(Layer {
                    thickness: amount,
                    material_id: top.material_id,
                });
                break;
            } else {
                let layer = self.layers.pop().unwrap();
                amount -= layer.thickness;
                removed.push(layer);
            }
        }

        removed
    }
}

impl Chunk {
    pub fn new(position: (i32, i32)) -> Self {
        Self {
            position,
            cells: vec![Cell::new(); CHUNK_SIZE * CHUNK_SIZE],
        }
    }

    #[inline]
    fn index(x: usize, y: usize) -> usize {
        debug_assert!(x < CHUNK_SIZE && y < CHUNK_SIZE);
        y * CHUNK_SIZE + x
    }

    pub fn cell(&self, x: usize, y: usize) -> &Cell {
        &self.cells[Self::index(x, y)]
    }

    pub fn cell_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[Self::index(x, y)]
    }

    pub fn extract_surface(&self) -> Vec<SurfaceCell> {
        self.cells
            .iter()
            .map(|cell| {
                let height = cell.total_height();
                let id = cell.surface_material().map(|id| id.0).unwrap_or(0);

                SurfaceCell {
                    height,
                    material_id: MaterialID(id),
                }
            })
            .collect()
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
            thickness: 0.0,
            material_id: MaterialID(0),
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
