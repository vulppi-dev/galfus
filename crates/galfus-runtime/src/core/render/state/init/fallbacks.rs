use super::super::RenderState;
use galfus_render::FallbackTextures;

impl RenderState {
    pub(crate) fn init_fallback_textures(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> FallbackTextures {
        galfus_render::create_fallback_textures(device, queue)
    }
}
