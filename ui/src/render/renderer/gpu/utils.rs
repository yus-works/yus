use std::{fs, num::{NonZeroU32, NonZeroU64}, ops::Deref};
use anyhow::{Context, Result};
use glam::{Mat4, Vec3};
use wgpu::{util::DeviceExt, SurfaceTarget};
use crate::web_sys::HtmlCanvasElement;

use crate::render::web_gpu::SurfaceContext;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialRaw {
    pub base_colour: [f32; 4],   // add more fields if you need them
}

pub fn create_material_buffer(sc: &SurfaceContext, colours: &[[f32; 4]]) -> wgpu::Buffer {
    sc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label:    Some("Material buffer"),
        contents: bytemuck::cast_slice(colours),
        usage:    wgpu::BufferUsages::STORAGE,   // storage == read-write in shader
    })
}

#[macro_export]
macro_rules! simple_ubo_layout_entry {
  ($b:expr, $vis:expr, $size:expr) => {
    wgpu::BindGroupLayoutEntry {
        binding: $b,
        visibility: $vis,
        ty: wgpu::BindingType::Buffer {
            ty:                wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size:  Some( NonZeroU64::new($size).unwrap()),
        },
        count: None
    }
  };
}

pub fn create_uniform_bind_group_layout(sc: &SurfaceContext) -> wgpu::BindGroupLayout {
    sc.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("UBO Bind Group Layout"),

        entries: &[
            simple_ubo_layout_entry!(0, wgpu::ShaderStages::VERTEX, 64), // binding 0 = Camera UBO (mat4x4)
            simple_ubo_layout_entry!(1, wgpu::ShaderStages::VERTEX, 64), // binding 1 = Model UBO (mat4x4)
            simple_ubo_layout_entry!(2, wgpu::ShaderStages::FRAGMENT, 32), // binding 2 = Light UBO (vec3 + padding)
            // binding=3: the texture view
            wgpu::BindGroupLayoutEntry {
                binding:    3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type:     wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension:  wgpu::TextureViewDimension::D2,
                    multisampled:    false,
                },
                count: None,
            },
            // binding=4: the sampler
            wgpu::BindGroupLayoutEntry {
                binding:    4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },

            wgpu::BindGroupLayoutEntry {
                binding:    5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty:                wgpu::BufferBindingType::Storage { read_only: (true) },
                    has_dynamic_offset: false,
                    min_binding_size:   None,
                },
                count: None,
            },
        ],
    })
}

pub async fn request_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue)> {
    adapter.request_device(&wgpu::DeviceDescriptor {
        label: None,
        required_features: wgpu::Features::empty(),
        required_limits: if cfg!(target_arch = "wasm32") {
            wgpu::Limits::downlevel_webgl2_defaults()
        } else {
            wgpu::Limits::default()
        },
        memory_hints: wgpu::MemoryHints::default(),
        trace: wgpu::Trace::Off,
    }).await.context("Failed to request device")
}

pub fn create_surface_static(
    instance: &wgpu::Instance,
    canvas: &HtmlCanvasElement,
) -> anyhow::Result<wgpu::Surface<'static>> {
    let target = SurfaceTarget::Canvas(canvas.clone());
    instance
        .create_surface(target)
        .context("webgpu surface init failed")
}

pub async fn request_adapter(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface<'static>,
) -> Result<wgpu::Adapter> {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .context("Failed to request a GPU adapter")?;

    Ok(adapter)
}

pub fn create_ubos(sc: &SurfaceContext) -> (wgpu::Buffer, wgpu::Buffer, wgpu::Buffer) {
    // 2.1 Camera UBO
    let aspect = sc.config.width as f32 / sc.config.height as f32;
    let proj   = Mat4::perspective_rh_gl(45f32.to_radians(), aspect, 0.1, 100.0);
    let view   = Mat4::look_at_rh(Vec3::new(3.,2.,4.), Vec3::ZERO, Vec3::Y);
    let view_proj: [[f32;4];4] = (proj * view).to_cols_array_2d();

    let camera_buffer = sc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera UBO"),
        contents: bytemuck::cast_slice(&view_proj),
        usage:  wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // 2.2 Model UBO
    let model_mat: [[f32;4];4] = Mat4::IDENTITY.to_cols_array_2d();
    let model_buffer = sc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Model UBO"),
        contents: bytemuck::cast_slice(&model_mat),
        usage:  wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // 2.3 Light UBO
    // direction + color, pad to 16 bytes
    let light_dir_color: [[f32;4];2] = [
        [ -0.8, -1.0, -1.0, 0.0 ],  // light direction
        [ 0.0,  1.0,  1.0, 0.0 ],  // light color
    ];
    let light_buffer = sc.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label:    Some("Light UBO"),
        contents: bytemuck::cast_slice(&light_dir_color),  // &[ [f32;4];2 ]
        usage:    wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    (camera_buffer, model_buffer, light_buffer)
}

pub fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    camera_buffer: &wgpu::Buffer,
    model_buffer: &wgpu::Buffer,
    light_buffer: &wgpu::Buffer,
    material_buffer: &wgpu::Buffer,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: model_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: light_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: material_buffer.as_entire_binding(),
            },
        ],
        label: Some("UBO Bind Group"),
    })
}

// TODO: change to not compile time baked in image
static IMAGE_BYTES: &[u8] = include_bytes!("../../../../texture.png");

pub fn load_texture(sc: &SurfaceContext) -> (wgpu::TextureView, wgpu::Sampler) {
    // 1. Load and flip Y so UV [0,0] is bottom-left
    let img = image::load_from_memory(IMAGE_BYTES)
        .expect("texture.png not found")
        .flipv()
        .into_rgba8();

    let (width, height) = img.dimensions();
    let size = wgpu::Extent3d {
        width, height, depth_or_array_layers: 1,
    };

    // 2. Create the GPU texture
    let texture = sc.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Cube Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // 3. Upload pixel data
    sc.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &img,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(NonZeroU32::new(4 * width).unwrap().into()),
            rows_per_image: Some(NonZeroU32::new(height).unwrap().into()),
        },
        size,
    );

    // 4. Create a view & sampler
    let texture_view = texture.create_view(&Default::default());
    let sampler = sc.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    (texture_view, sampler)
}

pub fn create_depth_view(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
    };
    let desc = wgpu::TextureDescriptor {
        label: Some("depth_texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let texture = device.create_texture(&desc);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub struct VertexShader(pub wgpu::ShaderModule);
pub struct FragmentShader(pub wgpu::ShaderModule);

impl Deref for VertexShader {
    type Target = wgpu::ShaderModule;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for FragmentShader {
    type Target = wgpu::ShaderModule;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: make this usable again somehow (file over http or smth)
#[allow(dead_code)]
pub fn load_shader(label: &str, path: &str, device: &wgpu::Device) -> wgpu::ShaderModule {
    let src = fs::read_to_string(path).expect("failed to read shader file");
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(src.into()),
    })
}

