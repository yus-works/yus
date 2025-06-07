use anyhow::Result;

use wgpu::util::DeviceExt;

use web_sys::HtmlCanvasElement;
use wgpu::StoreOp;
use crate::render::renderer::camera::Camera;
use crate::render::renderer::gpu::utils::*;
use crate::render::renderer::instance::InstanceRaw;
use glam::Mat4;
use glam::Vec3;
use crate::render::renderer::vertex;
use web_sys;


pub fn create_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    vs_shader: &VertexShader,
    fs_shader: &FragmentShader,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[uniform_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        cache: None,
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            compilation_options: Default::default(),
            module: &vs_shader,
            entry_point: Some("vs_main"),
            buffers: &[vertex::Vertex::desc(), InstanceRaw::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            compilation_options: Default::default(),
            module: &fs_shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less, // passes if new depth < old
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview: None,
    })
}


#[allow(dead_code)]
pub struct GpuState {
    surface_context: SurfaceContext,
    resource_context: ResourceContext,

    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    instance_buffer: wgpu::Buffer,
    instance_count: u32,

    pub start_time: f64,

    pub camera: Camera,
    pub dragging: bool,
    pub last_mouse_pos: (f32, f32),

    depth_view: wgpu::TextureView,
}

impl GpuState {
    pub fn resolution(&self) -> (f32, f32) {
        (self.surface_context.config.width as f32, self.surface_context.config.height as f32)
    }

    pub fn render(&mut self, canvas: &Option<HtmlCanvasElement>) {
        // 1) state already ready

        // 2) acquire next frame
        let canvas = canvas.as_ref().unwrap();
        let frame = self.surface_context.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());

        // 3) encode a render pass that clears green and draws the quad
        let mut encoder = self.surface_context.device.create_command_encoder(&Default::default());
        {
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

pub struct SurfaceContext {
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl SurfaceContext {
    async fn new_async(canvas: &HtmlCanvasElement) -> Result<Self> {
        let instance = wgpu::Instance::default();
        let surface = create_surface_static(&instance, canvas)?;

        let adapter = request_adapter(&instance, &surface).await?;
        let (device, queue) = request_device(&adapter).await?;
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0]; // choose a supported format?

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: canvas.width(),
            height: canvas.height(),
            present_mode: wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Ok(SurfaceContext {
            surface,
            adapter,
            device,
            queue,
            config,
        })
    }
}

pub struct ResourceContext {
    pub camera_ubo: wgpu::Buffer,
    pub model_ubo: wgpu::Buffer,
    pub light_ubo: wgpu::Buffer,
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    pub material_buffer: wgpu::Buffer,
}

impl ResourceContext {
    async fn new_async(sc: &SurfaceContext) -> Self {
        let (camera_ubo, model_ubo, light_ubo) = create_ubos(&sc);
        let (texture_view, sampler) = load_texture(&sc);
        let bind_group_layout = create_uniform_bind_group_layout(&sc);

        let colours = [
            [1.0, 0.0, 0.0, 1.0],   // red cube
            [0.0, 1.0, 0.0, 1.0],   // green cube
            [0.0, 0.0, 1.0, 1.0],   // blue cube
        ];

        let material_buffer = create_material_buffer(sc, &colours);

        let bind_group = create_bind_group(
            &sc.device,
            &bind_group_layout,
            &camera_ubo,
            &model_ubo,
            &light_ubo,
            &material_buffer,
            &texture_view,
            &sampler
        );

        ResourceContext {
            camera_ubo,
            model_ubo,
            light_ubo,
            texture_view,
            sampler,
            bind_group_layout,
            bind_group,
            material_buffer,
        }
    }
}

static CUBE_VS: &str = include_str!("./renderer/shaders/cube.vert.wgsl");
static CUBE_FS: &str = include_str!("./renderer/shaders/cube.frag.wgsl");

pub async fn init_wgpu(canvas: &HtmlCanvasElement, ) -> Result<GpuState> {
    let sc = SurfaceContext::new_async(&canvas).await?;
    let rc = ResourceContext::new_async(&sc).await;

    let vs_module = VertexShader(
        sc.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Cube VS"),
                source: wgpu::ShaderSource::Wgsl(CUBE_VS.into()),
        })
    );
    let fs_module = FragmentShader(
        sc.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Cube FS"),
                source: wgpu::ShaderSource::Wgsl(CUBE_FS.into()),
        })
    );

    let pipeline = create_pipeline(&sc.device, &sc.config, &rc.bind_group_layout, &vs_module, &fs_module);
    let depth_view = create_depth_view(&sc.device, &sc.config);

    let vertex_buffer = sc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Cube Vertex Buffer"),
        contents: bytemuck::cast_slice(vertex::VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = sc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Cube Index Buffer"),
        contents: bytemuck::cast_slice(vertex::INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    let num_indices = vertex::INDICES.len() as u32;

    let translations = [
        Vec3::ZERO,
    ];
    let instances: Vec<_> = translations
        .iter()
        .map(|p| Mat4::from_translation(*p))
        .map(InstanceRaw::from_mat4)
        .collect();

    let instance_buffer = sc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Instance buffer"),
        contents: bytemuck::cast_slice(&instances),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let instance_count = instances.len() as u32;

    Ok(GpuState {
        surface_context: sc,
        resource_context: rc,
        pipeline,

        vertex_buffer,
        index_buffer,
        num_indices,
        instance_buffer,
        instance_count,

        start_time: web_sys::window().unwrap().performance().unwrap().now(),

        camera: Camera::default(),
        dragging: false,
        last_mouse_pos: (0.0, 0.0),

        depth_view,
    })
}
