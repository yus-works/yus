use super::{surface_context::SurfaceContext, utils::{create_bind_group, create_material_buffer, create_ubos, create_uniform_bind_group_layout, load_texture}};

pub struct ResourceContext {
    pub camera_ubo: wgpu::Buffer,
    pub model_ubo: wgpu::Buffer,
    pub light_ubo: wgpu::Buffer,
    pub material_ubo: wgpu::Buffer,

    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl ResourceContext {
    pub async fn new_async(sc: &SurfaceContext) -> Self {
        let (camera_ubo, model_ubo, light_ubo, material_ubo) = create_ubos(&sc);
        let (texture_view, sampler) = load_texture(&sc);
        let bind_group_layout = create_uniform_bind_group_layout(&sc);

        let colours = [
            [1.0, 0.0, 0.0, 1.0],   // red cube
            [0.0, 1.0, 0.0, 1.0],   // green cube
            [0.0, 0.0, 1.0, 1.0],   // blue cube
        ];

        let bind_group = create_bind_group(
            &sc.device,
            &bind_group_layout,
            &camera_ubo,
            &model_ubo,
            &light_ubo,
            &material_ubo,
            &texture_view,
            &sampler
        );

        ResourceContext {
            camera_ubo,
            model_ubo,
            light_ubo,
            material_ubo,

            texture_view,
            sampler,
            bind_group_layout,
            bind_group,
        }
    }
}
