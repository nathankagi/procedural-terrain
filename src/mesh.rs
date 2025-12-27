pub trait Meshable {
    fn mesh_triangles(&self) -> Mesh;
    fn remesh_triangles(&mut self, mesh: &mut Mesh, modified: Vec<(u32, u32)>);
}

pub struct Mesh {
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}
