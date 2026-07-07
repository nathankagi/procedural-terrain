pub struct MaterialRegistry {
    materials: Vec<MaterialProperties>,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct MaterialProperties {
    pub erosion: f32,
    pub cohesion: f32,
    pub saturation: f32,
    pub permeability: f32,
    pub mass: f32,
}

impl Default for MaterialRegistry {
    fn default() -> Self {
        MaterialRegistry {
            materials: Vec::new(),
        }
    }
}

impl MaterialRegistry {
    pub fn get(&self, id: u16) -> &MaterialProperties {
        &self.materials[id as usize]
    }
}

impl Default for MaterialProperties {
    fn default() -> Self {
        MaterialProperties {
            erosion: 0.0,
            cohesion: 0.0,
            saturation: 0.0,
            permeability: 0.0,
            mass: 0.0,
        }
    }
}
