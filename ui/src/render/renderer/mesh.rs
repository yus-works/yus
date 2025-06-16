use super::vertex::Vertex;

pub struct CpuMesh<'a> {
    pub vertices: &'a[Vertex],
    pub indices: &'a[u16],
    pub index_count: u32,
}

impl<'a> CpuMesh<'a> {
    pub fn new(
        vertices: &'a[Vertex],
        indices:  &'a[u16],
    ) -> Self {

        let c = indices.len() as u32;
        Self {
            vertices,
            indices,
            index_count: c,
        }
    }
}
