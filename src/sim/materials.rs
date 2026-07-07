pub struct MaterialRegistry {
    materials: Vec<Material>,
}

#[derive(Copy, Clone, PartialEq)]
pub struct Material {
    pub erosion: f32,
    pub cohesion: f32,
    pub saturation: f32,
    pub permeability: f32,
    pub mass: f32,
}
