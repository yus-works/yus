use glam::Mat4;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
    /// column-major model matrix takes up 4 locations (3-6)
    pub const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        6 => Float32x4,
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceRaw>() as _,
            step_mode:    wgpu::VertexStepMode::Instance,
            attributes:   &Self::ATTRIBS,   // lives for the program lifetime ⇒ OK
        }
    }

    pub fn from_mat4(m: Mat4) -> Self {
        Self { model: m.to_cols_array_2d() }
    }
}
