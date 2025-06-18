use super::{surface_context::SurfaceContext, utils::{common_bind_group, create_ubos, load_texture, spatial_bind_group, texturing_bind_group}};

pub struct Group {
    pub group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
}

pub struct ResourceContext {
    pub camera_ubo: wgpu::Buffer,
    pub model_ubo: wgpu::Buffer,
    pub light_ubo: wgpu::Buffer,
    pub material_ubo: wgpu::Buffer,
    pub time_ubo: wgpu::Buffer,
    pub resolution_ubo: wgpu::Buffer,

    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    pub common_bind_group: Group,
    pub spatial_bind_group: Group,
    pub texturing_bind_group: Group,
}

impl ResourceContext {
    pub fn pipeline_layout(&self, device: &wgpu::Device) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[
                &self.common_bind_group.layout,
                &self.spatial_bind_group.layout,
                &self.texturing_bind_group.layout,
            ],
            push_constant_ranges: &[],
        })
    }

    pub async fn new_async(sc: &SurfaceContext) -> Self {
        let (camera_ubo, model_ubo, light_ubo, material_ubo, time_ubo, resolution_ubo) = create_ubos(&sc);
        let (texture_view, sampler) = load_texture(&sc);

        // let colours = [
        //     [1.0, 0.0, 0.0, 1.0],   // red cube
        //     [0.0, 1.0, 0.0, 1.0],   // green cube
        //     [0.0, 0.0, 1.0, 1.0],   // blue cube
        // ];

        let (common_layout, common_group) = common_bind_group(&sc.device, &time_ubo, &resolution_ubo);
        let (spatial_layout, spatial_group) = spatial_bind_group(&sc.device, &camera_ubo, &model_ubo, &light_ubo);
        let (texturing_layout, texturing_group) = texturing_bind_group(&sc.device, &material_ubo, &texture_view, &sampler);

        let common_bind_group = Group {
            group: common_group,
            layout: common_layout,
        };

        let spatial_bind_group = Group {
            group: spatial_group,
            layout: spatial_layout,
        };

        let texturing_bind_group = Group {
            group: texturing_group,
            layout: texturing_layout,
        };

        ResourceContext {
            camera_ubo,
            model_ubo,
            light_ubo,
            material_ubo,
            time_ubo,
            resolution_ubo,

            common_bind_group,
            spatial_bind_group,
            texturing_bind_group,

            sampler,
            texture_view,
        }
    }
}
