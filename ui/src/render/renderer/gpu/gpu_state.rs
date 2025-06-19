use std::{cell::RefCell, rc::Rc};

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use glam::Mat4;
use wgpu::{CommandEncoder, StoreOp, TextureView};

use crate::{components::demos::utils::RenderPass, render::renderer::{camera_input::CameraInput, mesh::CpuMesh, vertex::Vertex}};

use super::{resource_context::ResourceContext, surface_context::SurfaceContext};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct TimeUBO {
    millis:     u32,
    secs:       u32,
    dt_ms:      u32,
    frame_id:   u32,
}

pub struct GpuState {
    pub surface_context: SurfaceContext,
    pub resource_context: ResourceContext,

    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub instance_buffer: wgpu::Buffer,
    pub instance_count: u32,

    pub start_ms: f64,
    pub prev_ms: f64, // since last frame
    pub frame_counter: u32,

    pub depth_view: wgpu::TextureView,
}

pub fn create_vert_buff(sc: &SurfaceContext, vertices: &[Vertex]) -> wgpu::Buffer {
    sc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

pub fn create_idx_buff(sc: &SurfaceContext, indices: &[u16]) -> wgpu::Buffer {
    sc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    })
}

pub enum Projection {
    /// Full-screen clip-space quad
    FlatQuad,

    /// Perspective camera pointed at a 3-D mesh
    Fulcrum,

    /// 2-D screen units -> clip space (top-left = (0,0))
    Ortho2D { width: f32, height: f32 },

    /// Caller supplies their own matrix
    Custom(Mat4),
}

pub struct FrameCtx {
    pub frame:          wgpu::SurfaceTexture,
    pub encoder:        wgpu::CommandEncoder,
    pub color_view:     wgpu::TextureView,
    pub depth_view:     wgpu::TextureView,
}

impl FrameCtx {
    /// Open a render-pass, hand it to the user closure, then drop it.
    pub fn pass<'a, F>(&'a mut self, desc: wgpu::RenderPassDescriptor<'a>, f: F)
    where
        F: FnOnce(&mut wgpu::RenderPass<'a>)
    {
        let mut rp = self.encoder.begin_render_pass(&desc);
        f(&mut rp);                 // user records whatever commands they want
        // rp dropped here → render-pass ends
    }

    pub fn with_default_descriptor<F>(&mut self, clear: wgpu::Color, f: F)
    where
        F: for<'a> FnOnce(&mut wgpu::RenderPass<'a>)
    {
        let view = self.color_view.clone();
        let depth = self.depth_view.clone();

        let desc = wgpu::RenderPassDescriptor {
            label: Some("user pass"),

            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        };

        self.pass(desc, f);
    }
}

impl GpuState {
    pub fn set_vertices(&mut self, vertices: &[Vertex]) {
        self.vertex_buffer = create_vert_buff(&self.surface_context, vertices);
    }

    pub fn set_indicies(&mut self, indices: &[u16]) {
        self.index_buffer = create_idx_buff(&self.surface_context, indices)
    }

    pub fn resolution(&self) -> (f32, f32) {
        (self.surface_context.config.width as f32, self.surface_context.config.height as f32)
    }

    /// Borrow-checked “begin frame” – returns a FrameCtx the caller can mutate.
    pub fn begin_frame(&mut self) -> FrameCtx {
        // 1) acquire swap-chain tex
        let frame = self.surface_context.surface.get_current_texture().expect("swap-chain error");
        let color_view  = frame.texture.create_view(&Default::default());

        // 2) create an encoder for the caller
        let encoder = self.surface_context.device.create_command_encoder(&Default::default());

        FrameCtx {
            frame,
            encoder,
            color_view,
            depth_view : self.depth_view.clone(),
        }
    }

    pub fn populate_common_buffers(
        &mut self,
        proj: &Projection,
        ci: &CameraInput,
        mesh: &CpuMesh,
    ) {
        let sc = &self.surface_context;
        self.vertex_buffer = create_vert_buff(sc, mesh.vertices);
        self.index_buffer = create_idx_buff(sc, mesh.indices);
        self.num_indices = mesh.index_count;

        let view_proj = match proj {
            &Projection::FlatQuad => Mat4::IDENTITY,
            &Projection::Custom(m) => m,
            &Projection::Ortho2D { width, height } => {
                Mat4::orthographic_rh_gl(
                    0.0, width,
                    height, 0.0,
                    -1.0, 1.0,
                )
            }
            &Projection::Fulcrum => {
                let aspect = self.resolution().0 as f32 / self.resolution().1 as f32;
                let proj = Mat4::perspective_rh_gl(45f32.to_radians(), aspect, 0.1, 100.0);
                let view = Mat4::look_at_rh(ci.camera.eye(), ci.camera.target, ci.camera.up);

                proj * view
            }
        };

        self.surface_context.queue.write_buffer(
            &self.resource_context.camera_ubo, 0,
            bytemuck::cast_slice(&view_proj.to_cols_array_2d()),
        );

        let (w, h) = self.resolution(); // canvas actual size
        let res = [w as f32, h as f32, 0 as f32, 0 as f32];
        self.surface_context.queue.write_buffer(
            &self.resource_context.resolution_ubo,
            0,
            bytemuck::cast_slice(&res),
        );

        // ── time maths ────────────────────────────────
        let now_ms = web_sys::window().unwrap().performance().unwrap().now();
        let dt_ms  = (now_ms - self.prev_ms) as u32;          // u32 fits 49 days
        let secs   = (now_ms / 1000.0) as u32;
        let millis = (now_ms as u32) % 1000;

        let payload = TimeUBO {
            millis,
            secs,
            dt_ms,
            frame_id : self.frame_counter,
        };

        self.surface_context.queue.write_buffer(
            &self.resource_context.time_ubo, 0,
            bytemuck::bytes_of(&payload)
        );

        self.frame_counter += 1;
        self.prev_ms = now_ms;
    }

    /// Finalise: submit & present.
    pub fn end_frame(&mut self, frame_ctx: FrameCtx) {
        self.surface_context.queue.submit(Some(frame_ctx.encoder.finish()));

        // present after encoder is dropped so borrow checker is happy ?
        drop(frame_ctx.color_view);      // no-op but clarifies intent

        // 4) submit + present
        frame_ctx.frame.present();
    }

    fn default_rpass(&self, encoder: &mut CommandEncoder, view: &TextureView) {
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

        rpass.set_bind_group(0, &self.resource_context.common_bind_group.group, &[]);
        rpass.set_bind_group(1, &self.resource_context.spatial_bind_group.group, &[]);
        rpass.set_bind_group(2, &self.resource_context.texturing_bind_group.group, &[]);

        rpass.draw_indexed(0..self.num_indices, 0, 0..self.instance_count);
    }
}

pub fn make_default_rpass(
    mesh: Rc<RefCell<CpuMesh<'static>>>,
    proj: Rc<RefCell<Projection>>,
) -> RenderPass {
    Rc::new(RefCell::new(move |st: &mut GpuState, cam: &CameraInput, ctx: &mut FrameCtx| {
        st.populate_common_buffers(&proj.borrow(), cam, &mesh.borrow());
        st.default_rpass(&mut ctx.encoder, &ctx.color_view);
    }))
}
