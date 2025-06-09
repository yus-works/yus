use glam::{Mat4, Vec3};
use wgpu::{CommandEncoder, StoreOp, TextureView};

use crate::render::renderer::camera::Camera;

use super::{resource_context::ResourceContext, surface_context::SurfaceContext};

pub struct GpuState {
    pub surface_context: SurfaceContext,
    pub resource_context: ResourceContext,

    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,

    pub start_time: f64,

    pub camera: Camera,
    pub dragging: bool,
    pub last_mouse_pos: (f32, f32),

    pub depth_view: wgpu::TextureView,
}

impl GpuState {
    pub fn resolution(&self) -> (f32, f32) {
        (self.surface_context.config.width as f32, self.surface_context.config.height as f32)
    }

    fn render_pass(&self, encoder: &mut CommandEncoder, view: TextureView) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),

            occlusion_query_set: None,
            timestamp_writes: None,
        });
        rpass.set_pipeline(&self.pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));   // mesh verts
        rpass.set_vertex_buffer(1, self.instance_buffer.slice(..)); // per-instance models
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        rpass.set_bind_group(0, &self.resource_context.bind_group, &[]);

        rpass.draw_indexed(0..self.num_indices, 0, 0..self.instance_count);
    }

    pub fn render(&mut self) {
        // 1) state already ready

        // 2) acquire next frame
        let frame = self.surface_context.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());

        // 3) encode a render pass that clears green and draws the quad
        let mut encoder = self.surface_context.device.create_command_encoder(&Default::default());
        self.render_pass(&mut encoder, view);

        let yaw = self.camera.yaw;
        let pitch = self.camera.pitch;

        self.camera.eye = Vec3::new(
            self.camera.distance * yaw.cos() * pitch.cos(),
            self.camera.distance * pitch.sin(),
            self.camera.distance * yaw.sin() * pitch.cos(),
        ) + self.camera.target;

        let aspect = self.resolution().0 as f32 / self.resolution().1 as f32;
        let proj = Mat4::perspective_rh_gl(45f32.to_radians(), aspect, 0.1, 100.0);
        let view = Mat4::look_at_rh(self.camera.eye, self.camera.target, self.camera.up);

        let view_proj = proj * view;

        self.surface_context.queue.write_buffer(
            &self.resource_context.camera_ubo,
            0,
            bytemuck::cast_slice(&view_proj.to_cols_array_2d()),
        );

        // 4) submit + present
        self.surface_context.queue.submit(Some(encoder.finish()));
        frame.present();
    }
}

