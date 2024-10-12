const MAX_LAYER_COUNT: i32 = 100;

pub struct Terrain {
    width: usize,
    height: usize,
    map: Vec<Vec<Block>>,
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
pub struct Block {
    height: f32,
    layers: Vec<Layer>,
}

#[derive(Copy, Clone)]
pub struct Chunk {}

#[repr(u8)]
enum MaterialType {
    Stone = 0,
    Soil = 1,
    Sand = 2,
}

impl Terrain {
    fn new(width: usize, height: usize) -> Self {
        Terrain {
            width,
            height,
            map: vec![vec![Block::new(); width]; height],
        }
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
