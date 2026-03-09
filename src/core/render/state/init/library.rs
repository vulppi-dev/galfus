mod library_common;
mod library_effects;

use super::super::RenderState;

pub(crate) struct Layouts {
    pub(crate) shared: wgpu::BindGroupLayout,
    pub(crate) object: wgpu::BindGroupLayout,
    pub(crate) object_standard: wgpu::BindGroupLayout,
    pub(crate) object_pbr: wgpu::BindGroupLayout,
    pub(crate) target: wgpu::BindGroupLayout,
    pub(crate) light_cull: wgpu::BindGroupLayout,
    pub(crate) ssao: wgpu::BindGroupLayout,
    pub(crate) ssao_blur: wgpu::BindGroupLayout,
    pub(crate) ssao_msaa: wgpu::BindGroupLayout,
    pub(crate) ssao_blur_msaa: wgpu::BindGroupLayout,
    pub(crate) bloom: wgpu::BindGroupLayout,
    pub(crate) skybox: wgpu::BindGroupLayout,
}

impl RenderState {
    pub(crate) fn init_layouts(&self, device: &wgpu::Device) -> Layouts {
        Layouts {
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
