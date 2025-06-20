use super::vertex::Vertex;

pub struct CpuMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub index_count: u32,
}

impl CpuMesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices:  Vec<u16>,
    ) -> Self {

        let c = indices.len() as u32;
        Self {
            vertices,
            indices,
            index_count: c,
        }
    }
}
