use crate::render::renderer::vertex::Vertex;

// 24 vertices (4 per face) so each face can have its own normal and UV coords
pub const CUBE_VERTICES: &[Vertex] = &[
    // +X face
    Vertex { position: [ 1., -1., -1.], normal: [1., 0., 0.], uv: [0.0, 0.0] },
    Vertex { position: [ 1.,  1., -1.], normal: [1., 0., 0.], uv: [0.0, 1.0] },
    Vertex { position: [ 1.,  1.,  1.], normal: [1., 0., 0.], uv: [1.0, 1.0] },
    Vertex { position: [ 1., -1.,  1.], normal: [1., 0., 0.], uv: [1.0, 0.0] },

    // -X face
    Vertex { position: [-1., -1.,  1.], normal: [-1., 0., 0.], uv: [0.0, 0.0] },
    Vertex { position: [-1.,  1.,  1.], normal: [-1., 0., 0.], uv: [0.0, 1.0] },
    Vertex { position: [-1.,  1., -1.], normal: [-1., 0., 0.], uv: [1.0, 1.0] },
    Vertex { position: [-1., -1., -1.], normal: [-1., 0., 0.], uv: [1.0, 0.0] },

    // +Y face
    Vertex { position: [-1.,  1., -1.], normal: [0., 1., 0.], uv: [0.0, 0.0] },
    Vertex { position: [-1.,  1.,  1.], normal: [0., 1., 0.], uv: [0.0, 1.0] },
    Vertex { position: [ 1.,  1.,  1.], normal: [0., 1., 0.], uv: [1.0, 1.0] },
    Vertex { position: [ 1.,  1., -1.], normal: [0., 1., 0.], uv: [1.0, 0.0] },

    // -Y face
    Vertex { position: [-1., -1.,  1.], normal: [0., -1., 0.], uv: [0.0, 0.0] },
    Vertex { position: [-1., -1., -1.], normal: [0., -1., 0.], uv: [0.0, 1.0] },
    Vertex { position: [ 1., -1., -1.], normal: [0., -1., 0.], uv: [1.0, 1.0] },
    Vertex { position: [ 1., -1.,  1.], normal: [0., -1., 0.], uv: [1.0, 0.0] },

    // +Z face
    Vertex { position: [-1., -1.,  1.], normal: [0., 0., 1.], uv: [0.0, 0.0] },
    Vertex { position: [ 1., -1.,  1.], normal: [0., 0., 1.], uv: [1.0, 0.0] },
    Vertex { position: [ 1.,  1.,  1.], normal: [0., 0., 1.], uv: [1.0, 1.0] },
    Vertex { position: [-1.,  1.,  1.], normal: [0., 0., 1.], uv: [0.0, 1.0] },

    // -Z face
    Vertex { position: [ 1., -1., -1.], normal: [0., 0., -1.], uv: [0.0, 0.0] },
    Vertex { position: [-1., -1., -1.], normal: [0., 0., -1.], uv: [1.0, 0.0] },
    Vertex { position: [-1.,  1., -1.], normal: [0., 0., -1.], uv: [1.0, 1.0] },
    Vertex { position: [ 1.,  1., -1.], normal: [0., 0., -1.], uv: [0.0, 1.0] },
];

// 6 faces × 2 triangles × 3 indices = 36
pub const CUBE_INDICES: &[u16] = &[
     0,  1,  2,  0,  2,  3,   // +X
     4,  5,  6,  4,  6,  7,   // -X
     8,  9, 10,  8, 10, 11,   // +Y
    12, 13, 14, 12, 14, 15,   // -Y
    16, 17, 18, 16, 18, 19,   // +Z
    20, 21, 22, 20, 22, 23,   // -Z
];
