use nalgebra::Vector2;

pub struct Point {
    pub attached: Vec<&Point>,
    pub position: Vector2,
}

pub mod diffusion_functions {
    pub const inv: Fn(f32) -> f32 = inverse;
}

pub fn generate() -> Vec<Vec<f32>> {
    vec![vec![0.0; 10]; 10]
}

impl Point {
    pub fn new() -> Point {
        Point {}
    }

    pub fn attach(&mut self, point: &Point) {
        self.attached.push(point);
    }

    pub fn height(&self, func: Fn(f32) -> f32) -> f32 {
        // return 1 - (1 / (1 + self.attached.len())) + 1;
        return func(self.attached.len() + 1);
    }
}

fn inverse(x: f32) -> f32 {
    1 - (1 / (1 + x))
}
