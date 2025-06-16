use crate::render::renderer::vertex::Vertex;

pub const QUAD_VERTS: &[Vertex] = &[
    Vertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 0.0] },
    Vertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [0.0, 1.0] },
    Vertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, 1.0], uv: [1.0, 0.0] },
];

pub const QUAD_INDICES: &[u16] = &[
    0, 1, 2,   // first  triangle  (TL-BL-BR)
    0, 2, 3,   // second triangle  (TL-BR-TR)
];
