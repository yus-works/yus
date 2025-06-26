use bytemuck::Pod;

use super::surface_context::SurfaceContext;

pub struct VertexCtx<V: Pod> {
    pub verts: Vec<V>,     // CPU-side cache (keeps allocation)
    pub capacity: u32,     // how many vertices fit in `buf`
    pub count: u32,        // verts actually in use this frame
    pub buf: wgpu::Buffer, // GPU buffer
}

impl<V: Pod> VertexCtx<V> {
    /// Build with an initial capacity (in vertices, not bytes).
    pub fn new(sc: &SurfaceContext, initial_cap: u32) -> Self {
        let buf = Self::create_vbuf(sc, initial_cap);
        Self {
            verts: Vec::with_capacity(initial_cap as usize),
            capacity: initial_cap,
            count: 0,
            buf,
        }
    }

    #[inline]
    fn create_vbuf(sc: &SurfaceContext, cap: u32) -> wgpu::Buffer {
        let byte_cap = cap as u64 * std::mem::size_of::<V>() as u64;
        sc.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dynamic-vertex-buffer"),
            size: byte_cap,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Grow GPU buffer if `self.verts` no longer fits.
    fn ensure_capacity(&mut self, sc: &SurfaceContext) {
        let needed = self.verts.len() as u32;
        if needed > self.capacity {
            self.capacity = needed.next_power_of_two();
            self.buf = Self::create_vbuf(sc, self.capacity);
        }
        self.count = needed;
    }

    /// Rebuild verts **and** upload them in one shot.
    pub fn sync<F>(&mut self, sc: &SurfaceContext, rebuild: F)
    where
        F: FnOnce(&mut Vec<V>),
    {
        self.verts.clear();
        rebuild(&mut self.verts); // fill CPU vec
        self.ensure_capacity(sc); // maybe realloc GPU
        sc.queue
            .write_buffer(&self.buf, 0, bytemuck::cast_slice(&self.verts));
    }
}
