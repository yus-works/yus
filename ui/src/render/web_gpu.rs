use std::cell::RefCell;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::Result;

use wgpu::util::DeviceExt;

use web_sys::HtmlCanvasElement;
use crate::render::renderer::gpu::utils::*;
use crate::render::renderer::instance::InstanceRaw;
use glam::Mat4;
use glam::Vec3;
use crate::render::renderer::vertex;
use web_sys;

use super::renderer::gpu::gpu_state::GpuState;
use super::renderer::gpu::resource_context::ResourceContext;
use super::renderer::gpu::surface_context::SurfaceContext;

pub async fn reload_pipeline(
    state: &Rc<RefCell<Option<GpuState>>>,
    vs_src: &str,
    fs_src: &str,
) -> anyhow::Result<()> {
    use std::borrow::Cow;

    if state.clone().borrow().is_none() {
        return Err(anyhow!("Gpu state is None"));
    }

    assert!(
        state.clone().borrow().is_some(),
        "GpuState is None and somehow passed the guard clause. This should not be possible."
    ); // now I can safely unwrap past this point

    if let Ok(mut guard) = state.try_borrow_mut() {
        let st = guard.as_mut().unwrap();

        let device  = &st.surface_context.device;
        let config  = &st.surface_context.config;
        let layout  = &st.resource_context.bind_group_layout;

        // ── 1️⃣  Start an error-scope that captures *validation* errors ─────────────
        device.push_error_scope(wgpu::ErrorFilter::Validation);

        // ── 2️⃣  Compile the two WGSL modules and rebuild the pipeline ─────────────
        let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("live VS shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(vs_src)),
        });

        let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("live FS shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(fs_src)),
        });

        let new_pipeline = create_pipeline(device, config, layout, &VertexShader(vs), &FragmentShader(fs));

        // ── 3️⃣  Pop the scope and check if anything went wrong ───────────────────
        return match device.pop_error_scope().await {
            None => {
                // ✅  All good – swap in the fresh pipeline
                st.pipeline = new_pipeline;
                Ok(())
            }
            Some(err) => {
                // ❌  WGSL or pipeline validation failed – keep the old pipeline
                Err(anyhow!(err.to_string()))
            }
        }
    } else {
        Err(anyhow!("oh no"))
    }
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

        depth_view,
    })
}
