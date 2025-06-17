use anyhow::Result;
use web_sys::HtmlCanvasElement;

use super::utils::{create_surface_static, request_adapter, request_device};

pub struct SurfaceContext {
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl SurfaceContext {
    pub async fn new_async(canvas: &HtmlCanvasElement) -> Result<Self> {
        let instance_desc = wgpu::InstanceDescriptor {
                backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
                ..Default::default()
        };

        let instance = wgpu::Instance::new(&instance_desc);

        let surface = create_surface_static(&instance, canvas)?;

        let adapter = request_adapter(&instance, &surface).await?;
        let (device, queue) = request_device(&adapter).await?;
        let surface_caps = surface.get_capabilities(&adapter);

        // NOTE: explicitly pick a non sRGB format because on WebGL the default is sRGB while on
        // WebGPU the default is linear so this way they look the same
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

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
