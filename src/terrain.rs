use nalgebra::Vector3;

const MAX_LAYER_COUNT: i32 = 100;

pub struct Terrain {
    width: usize,
    height: usize,
    map: Vec<Vec<Block>>,
}

#[derive(Copy, Clone, Default)]
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
pub struct Block {
    height: f32,
    layers: Vec<Layer>,
}

#[derive(Copy, Clone)]
pub struct Chunk {}

impl Terrain {
    fn new(width: usize, height: usize) -> Self {
        Terrain {
            width,
            height,
            map: vec![vec![Block::new(); width]; height],
        }
    }

    fn height(&self, x: usize, y: usize) -> f32 {
        return self.map[x][y].height;
    }

    fn material(&self, x: usize, y: usize) -> Material {
        return self.map[x][y].layers.last().unwrap().material;
    }

    fn normal(&self, x: usize, y: usize) -> Vector3<f32> {
        return Vector3::new(0.0, 0.0, 0.0);
    }

    fn gradient(&self, x: usize, y: usize) -> Vector3<f32> {
        return Vector3::new(0.0, 0.0, 0.0);
    }

    fn add(&self, x: usize, y: usize, layer: Layer) -> () {
        if ()
    }

    fn remove(&self, x: usize, y: usize, height: f32) -> Vec<Layer> {
        let h = self.map[x][y].height;

        let layers: Vec<Layer> = if (height < h) {
            self.map[x][y].layers.last().unwrap().height = self.map[x][y].layers.last().unwrap().height - height;

            return vec![Layer {
                height,
                material: self.map[x][y].layers.last().unwrap().material,
            }];

        } else if height == h {
            return vec![self.map[x][y].layers.pop().unwrap()];

        } else if height > h {
            let l = self.map[x][y].layers.pop().unwrap();

            self.map[x][y].layers.last().unwrap().height = self.map[x][y].layers.last().unwrap().height - height;

            return vec![l, Layer {
                height,
                material: self.map[x][y].layers.last().unwrap().material,
            }];
        }
        else {
            return vec![];
        }

        return layers;
    }

    fn top(&self, x: usize, y: usize) -> &Layer {
        return &self.map[x][y].layers.last().unwrap();
    }
}

impl Block {
    fn new() -> Self {
        Block {
            height: 0.0,
            // layers: vec![Layer::new(), MAX_LAYER_COUNT],
            layers: Vec::new(),
        }
    }
}

// impl Layer {
//     fn new() -> Self {}
// }

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
