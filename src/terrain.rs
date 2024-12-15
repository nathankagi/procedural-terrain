use nalgebra::Vector3;

const MAX_LAYER_COUNT: usize = 100;

pub struct Terrain {
    width: usize,
    height: usize,
    cells: Vec<Vec<Cell>>,
}

#[derive(Copy, Clone)]
pub struct Material {
    erosion: f32,
    cohesion: f32,
    saturation: f32,
    permeability: f32,
}

#[derive(Copy, Clone)]
pub struct Layer {
    height: f32,
    material: Material,
}

#[derive(Clone)]
pub struct Cell {
    // layers: Vec<Layer>,
    layers: [Layer; MAX_LAYER_COUNT],
    layer_index: i32,
}

#[derive(Copy, Clone)]
pub struct Chunk {}

impl Terrain {
    fn new(width: usize, height: usize) -> Self {
        Terrain {
            width,
            height,
            cells: vec![vec![Cell::new(); width]; height],
        }
    }

    fn at(&mut self, x: usize, y: usize) -> &mut Cell {
        return &mut self.cells[x][y];
    }

    fn height(&self, x: usize, y: usize) -> f32 {
        return self.cells[x][y].height();
    }

    fn material(&self, x: usize, y: usize) -> Material {
        // return self.cells[x][y].layers.last().unwrap().material.clone();
        return self.cells[x][y].layers.last().unwrap().material;
    }

    fn normal(&self, x: usize, y: usize) -> Vector3<f32> {
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

    fn gradient(&self, x: usize, y: usize) -> Vector3<f32> {
        let norm : Vector3<f32> = self.normal(x, y);
        return Vector3::new(norm.x, 0.0, norm.z);
    }

    fn add(&mut self, x: usize, y: usize, layer: Layer) -> () {
        return ();
    }

    // fn remove(&mut self, x: usize, y: usize, height: f32) -> Vec<Layer> {
    //     // while height > 0
    //     // subtract top layer height from height, add to return layers

    //     let h = self.cells[x][y].height();

    //     let layers: Vec<Layer> = if height < h {
    //         let h = Some(self.cells[x][y].layers.last().unwrap().height);
    //         // self.cells[x][y].layers.last().unwrap().height =
    //         //     self.cells[x][y].layers.last().unwrap().height - height;

    //         vec![Layer {
    //             height,
    //             material: self.cells[x][y].layers.last().unwrap().material,
    //         }]
    //     } else if height == h {
    //         vec![self.cells[x][y].layers.pop().unwrap()]
    //     } else if height > h {
    //         let l = self.cells[x][y].layers.pop().unwrap();

    //         // self.cells[x][y].layers.last().unwrap().height =
    //         //     self.cells[x][y].layers.last().unwrap().height - height;

    //         vec![
    //             l,
    //             Layer {
    //                 height,
    //                 material: self.cells[x][y].layers.last().unwrap().material,
    //             },
    //         ]
    //     } else {
    //         return vec![];
    //     };

    //     return layers;
    // }

    fn top(&self, x: usize, y: usize) -> Option<&Layer> {
        return self.cells[x][y].layers.last();
    }
}

impl Cell {
    fn new() -> Self {
        Cell {
            layers: [Layer::default(); MAX_LAYER_COUNT],
            layer_index: 0,
        }
    }

    fn height(&self) -> f32 {
        let mut h: f32 = 0.0;
        for layer in self.layers.iter() {
            h = h + layer.height;
        }
        return h;
    }

    // fn add(&self, layer: Layer) {

    // }

    // fn remove(&self, height : f32) -> Vec<Layer> {
    //     let mut layers : Vec<Layer> = Vec::new();

    //     return layers
    // }
}

impl Layer {
    fn new() -> Self {
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
        }
    }
}
