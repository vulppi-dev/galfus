mod library_common;
mod library_effects;

use super::super::RenderState;

impl RenderState {
    pub(crate) fn init_layouts(&self, device: &wgpu::Device) -> vulfram_render::Layouts {
        vulfram_render::Layouts {
            shared: library_common::create_layout_shared(device),
            object: library_common::create_layout_object(device),
            object_standard: library_common::create_layout_object_standard(device),
            object_pbr: library_common::create_layout_object_pbr(device),
            target: library_common::create_layout_target(device),
            light_cull: library_effects::create_layout_light_cull(device),
            ssao: library_effects::create_layout_ssao(device),
            ssao_blur: library_effects::create_layout_ssao_blur(device),
            ssao_msaa: library_effects::create_layout_ssao_msaa(device),
            ssao_blur_msaa: library_effects::create_layout_ssao_blur_msaa(device),
            bloom: library_effects::create_layout_bloom(device),
            skybox: library_effects::create_layout_skybox(device),
        }
    }
}
