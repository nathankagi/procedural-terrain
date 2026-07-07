#[derive(Copy, Clone)]
pub struct Layer {
    thickness: f32,
    material_id: u16,
}

#[derive(Clone)]
pub struct Cell {
    layers: Vec<Layer>,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SurfaceCell {
    pub height: f32,
    pub material_id: u16,
}

pub struct Terrain {
    pub cells: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

impl Terrain {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::new(); width * height],
            width,
            height,
        }
    }

    #[inline]
    pub fn cell(&self, x: usize, z: usize) -> &Cell {
        &self.cells[z * self.width + x]
    }

    #[inline]
    pub fn cell_mut(&mut self, x: usize, z: usize) -> &mut Cell {
        &mut self.cells[z * self.width + x]
    }

    pub fn extract_heights(&self) -> Vec<f32> {
        self.cells.iter().map(|c| c.total_height()).collect()
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

    pub fn surface_material(&self) -> Option<u16> {
        self.surface_layer().map(|l| l.material_id)
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    pub fn deposit(&mut self, thickness: f32, material_id: u16) {
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
            material_id: 0u16,
        }
    }
}
