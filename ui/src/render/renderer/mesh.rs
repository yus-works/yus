use wgpu::{Buffer, BufferUsages, Device, Queue, RenderPass};
use wgpu::util::DeviceExt;

pub struct Mesh {
    vertex_buf: Buffer,
    index_buf:  Buffer,
    index_count: u32,
}

impl Mesh {
    pub fn new<V: bytemuck::Pod>(
        device: &Device,
        queue:  &Queue,
        vertices: &[V],
        indices:  &[u16],
        usage: BufferUsages,            // e.g. VERTEX | COPY_DST (optional mutability)
    ) -> Self {
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh vertex buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: usage | BufferUsages::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh index buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
        });

        Self {
            vertex_buf,
            index_buf,
            index_count: indices.len() as u32,
        }
    }

    pub fn draw<'rp>(&'rp self, pass: &mut RenderPass<'rp>) {
        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.set_index_buffer (self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}
