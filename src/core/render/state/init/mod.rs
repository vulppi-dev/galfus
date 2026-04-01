#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
mod fallbacks;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
mod library;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
mod systems;

#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
use crate::core::render::state::{RenderState, ResourceLibrary};

#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
impl RenderState {
    pub(crate) fn init(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _surface_format: wgpu::TextureFormat,
    ) {
        // 1. Initialize core systems
        self.init_core_systems(device, queue);

        // 2. Initialize samplers
        let samplers = self.init_samplers(device);

        // 3. Initialize layouts
        let layouts = self.init_layouts(device);

        // 4. Initialize fallback textures
        let fallbacks = self.init_fallback_textures(device, queue);

        // 5. Initialize pipeline layouts
        let pipeline_layouts = vulfram_render::create_pipeline_layouts(device, &layouts);

        // 6. Initialize shaders
        let forward_standard_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../passes/forward/branches/forward_standard.wgsl"
        ));
        let forward_pbr_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../passes/forward/branches/forward_pbr.wgsl"
        ));
        let post_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/post/post.wgsl"));
        let compose_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/compose/compose.wgsl"));
        let light_cull_shader = device.create_shader_module(wgpu::include_wgsl!(
            "../../passes/light_cull/light_cull.wgsl"
        ));
        let shadow_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/shadow/shadow.wgsl"));
        let outline_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/outline/outline.wgsl"));
        let ssao_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/ssao/ssao.wgsl"));
        let ssao_blur_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/ssao/ssao_blur.wgsl"));
        let ssao_msaa_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/ssao/ssao_msaa.wgsl"));
        let ssao_blur_msaa_shader = device
            .create_shader_module(wgpu::include_wgsl!("../../passes/ssao/ssao_blur_msaa.wgsl"));
        let bloom_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/bloom/bloom.wgsl"));
        let skybox_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../passes/skybox/skybox.wgsl"));
        let gizmo_shader =
            device.create_shader_module(wgpu::include_wgsl!("../../gizmos/gizmo.wgsl"));

        let effect_buffers = vulfram_render::create_effect_buffers(
            device,
            crate::core::render::passes::skybox_uniform_buffer_size(),
        );

        // 7. Initialize library
        self.library = Some(ResourceLibrary {
            layout_shared: layouts.shared,
            layout_object: layouts.object,
            layout_object_standard: layouts.object_standard,
            layout_object_pbr: layouts.object_pbr,
            layout_target: layouts.target,
            layout_light_cull: layouts.light_cull,
            layout_ssao: layouts.ssao,
            layout_ssao_blur: layouts.ssao_blur,
            layout_ssao_msaa: layouts.ssao_msaa,
            layout_ssao_blur_msaa: layouts.ssao_blur_msaa,
            layout_bloom: layouts.bloom,
            layout_skybox: layouts.skybox,
            forward_standard_pipeline_layout: pipeline_layouts.forward_standard,
            forward_pbr_pipeline_layout: pipeline_layouts.forward_pbr,
            shadow_pipeline_layout: pipeline_layouts.shadow,
            outline_pipeline_layout: pipeline_layouts.outline,
            ssao_pipeline_layout: pipeline_layouts.ssao,
            ssao_blur_pipeline_layout: pipeline_layouts.ssao_blur,
            ssao_msaa_pipeline_layout: pipeline_layouts.ssao_msaa,
            ssao_blur_msaa_pipeline_layout: pipeline_layouts.ssao_blur_msaa,
            bloom_pipeline_layout: pipeline_layouts.bloom,
            skybox_pipeline_layout: pipeline_layouts.skybox,
            forward_standard_shader,
            forward_pbr_shader,
            post_shader,
            compose_shader,
            outline_shader,
            ssao_shader,
            ssao_blur_shader,
            ssao_msaa_shader,
            ssao_blur_msaa_shader,
            bloom_shader,
            skybox_shader,
            light_cull_shader,
            shadow_shader,
            gizmo_shader,
            light_cull_pipeline_layout: pipeline_layouts.light_cull,
            gizmo_pipeline_layout: pipeline_layouts.gizmo,
            samplers,
            _fallback_texture: fallbacks.texture,
            fallback_view: fallbacks.view,
            _fallback_forward_atlas_texture: fallbacks.atlas_texture,
            fallback_forward_atlas_view: fallbacks.atlas_view,
            _fallback_shadow_texture: fallbacks.shadow_texture,
            fallback_shadow_view: fallbacks.shadow_view,
        });

        self.post_uniform_buffer = Some(effect_buffers.post);
        self.compose_uniform_buffer = Some(effect_buffers.compose);
        self.ssao_uniform_buffer = Some(effect_buffers.ssao);
        self.ssao_blur_uniform_buffer = Some(effect_buffers.ssao_blur);
        self.bloom_uniform_buffer = Some(effect_buffers.bloom);
        self.skybox_uniform_buffer = Some(effect_buffers.skybox);
    }
}
