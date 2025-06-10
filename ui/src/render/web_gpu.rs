use anyhow::Result;

use wgpu::util::DeviceExt;

use web_sys::HtmlCanvasElement;
use crate::render::renderer::camera::Camera;
use crate::render::renderer::gpu::utils::*;
use crate::render::renderer::instance::InstanceRaw;
use glam::Mat4;
use glam::Vec3;
use crate::render::renderer::vertex;
use web_sys;

use super::renderer::gpu::gpu_state::GpuState;
use super::renderer::gpu::resource_context::ResourceContext;
use super::renderer::gpu::surface_context::SurfaceContext;

pub fn reload_pipeline(state: &mut GpuState, vs_src: &str, fs_src: &str) {
    use std::borrow::Cow;

    let vs = state.surface_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("live VS shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(vs_src)),
    });

    let fs = state.surface_context.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("live FS shader"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(fs_src)),
    });

    state.pipeline = create_pipeline(
        &state.surface_context.device,
        &state.surface_context.config,
        &state.resource_context.bind_group_layout,
        &VertexShader(vs),
        &FragmentShader(fs)
    );
}


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
