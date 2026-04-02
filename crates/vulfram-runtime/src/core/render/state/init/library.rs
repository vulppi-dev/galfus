use super::super::RenderState;

impl RenderState {
    pub(crate) fn init_layouts(&self, device: &wgpu::Device) -> vulfram_render::Layouts {
        let sizes = vulfram_render::RenderLayoutSizes {
            frame_uniform_min_size: std::mem::size_of::<crate::core::resources::FrameComponent>()
                as u64,
            camera_uniform_min_size: std::mem::size_of::<crate::core::resources::CameraComponent>()
                as u64,
            light_draw_uniform_min_size: std::mem::size_of::<
                crate::core::render::state::light::LightDrawParams,
            >() as u64,
            model_storage_min_size: std::mem::size_of::<crate::core::resources::ModelComponent>()
                as u64,
            material_standard_uniform_min_size: std::mem::size_of::<
                crate::core::resources::MaterialStandardParams,
            >() as u64,
            material_pbr_uniform_min_size: std::mem::size_of::<
                crate::core::resources::MaterialPbrParams,
            >() as u64,
            matrix_storage_min_size: std::mem::size_of::<glam::Mat4>() as u64,
        };
        vulfram_render::create_render_layouts(device, &sizes)
    }
}
