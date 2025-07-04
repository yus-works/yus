use std::cell::RefCell;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::Result;

use web_sys::HtmlCanvasElement;
use crate::render::renderer::gpu::utils::*;
use crate::render::renderer::instance::InstanceRaw;
use crate::render::renderer::vertex::Vertex;
use crate::render::renderer::vertex;
use web_sys;

use super::renderer::gpu::gpu_state::GpuState;
use super::renderer::gpu::resource_context::ResourceContext;
use super::renderer::gpu::surface_context::SurfaceContext;

pub async fn reload_pipeline(
    state_rc: &Rc<RefCell<Option<GpuState>>>,
    vs_src: &str,
    fs_src: &str,
) -> anyhow::Result<wgpu::RenderPipeline> {
    use wgpu::ErrorFilter as F;

    let (layout, config, device) = {
        let guard = state_rc.borrow();
        let st = guard.as_ref()
            .ok_or_else(|| anyhow!("GpuState is None"))?;
        (
            st.resource_context.pipeline_layout(&st.surface_context.device),
            st.surface_context.config.clone(),
            st.surface_context.device.clone(),
        )
    };

    device.push_error_scope(F::Validation);

    let vs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("live-reload VS"),
        source: wgpu::ShaderSource::Wgsl(vs_src.into()),
    });
    let fs = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("live-reload FS"),
        source: wgpu::ShaderSource::Wgsl(fs_src.into()),
    });

    let promise = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("live-reload pipeline"),
        layout: Some(&layout),
        cache: None,
        vertex: wgpu::VertexState {
            module: &vs,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: Some(wgpu::DepthStencilState{
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: false,   // or true – doesn’t matter for flat-quad
            depth_compare: wgpu::CompareFunction::Always,
            stencil: Default::default(),
            bias: Default::default(),
        }),

        multisample: Default::default(),
        multiview: None,
    });

    let new_pipe = promise;
    match device.pop_error_scope().await {
        None => Ok(new_pipe),
        Some(err) => Err(anyhow!(err.to_string())),
    }
}

pub fn default_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    pipeline_layout: &wgpu::PipelineLayout,
    vs_shader: &VertexShader,
    fs_shader: &FragmentShader,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        cache: None,
        label: Some("Default Render Pipeline"),
        layout: Some(pipeline_layout),
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

pub async fn init_wgpu(canvas: &HtmlCanvasElement, ) -> Result<GpuState> {
    let sc = SurfaceContext::new_async(&canvas).await?;
    let rc = ResourceContext::new_async(&sc).await;

    let depth_view = create_depth_view(&sc.device, &sc.config);

    let t0 = web_sys::window().unwrap().performance().unwrap().now();

    Ok(GpuState {
        surface_context: sc,
        resource_context: rc,

        start_ms: t0,
        prev_ms: t0,
        frame_counter: 0,

        depth_view,
    })
}
