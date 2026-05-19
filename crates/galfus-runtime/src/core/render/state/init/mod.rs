#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
mod fallbacks;
#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
mod library;
#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
mod systems;

#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
use crate::core::render::state::RenderState;

#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
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
        let pipeline_layouts = galfus_render::create_pipeline_layouts(device, &layouts);

        // 6. Initialize shaders
        let shaders = galfus_render::create_shader_modules(device);

        let effect_buffers = galfus_render::create_effect_buffers(
            device,
            crate::core::render::passes::skybox_uniform_buffer_size(),
        );

        // 7. Initialize library
        self.library = Some(galfus_render::build_resource_library(
            layouts,
            pipeline_layouts,
            shaders,
            samplers,
            fallbacks,
        ));

        self.post_uniform_buffer = Some(effect_buffers.post);
        self.compose_uniform_buffer = Some(effect_buffers.compose);
        self.ssao_uniform_buffer = Some(effect_buffers.ssao);
        self.ssao_blur_uniform_buffer = Some(effect_buffers.ssao_blur);
        self.bloom_uniform_buffer = Some(effect_buffers.bloom);
        self.skybox_uniform_buffer = Some(effect_buffers.skybox);
    }
}
