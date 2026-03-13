pub mod binding;
pub mod collector;
pub mod init;
pub mod library;
pub mod lifecycle;
pub mod light;
pub mod prepare;
pub mod scene;
pub mod skinning;

use crate::core::realm::RealmId;
use crate::core::render::cache::RenderCache;
use crate::core::render::gizmos::GizmoSystem;
use crate::core::resources::EnvironmentConfig;
use crate::core::resources::VertexAllocatorSystem;
use crate::core::resources::shadow::ShadowManager;
use crate::core::ui::UiRenderer;

pub use self::binding::BindingSystem;
pub use self::collector::{DrawCollector, DrawItem};
pub use self::library::ResourceLibrary;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
pub use self::library::SamplerSet;
pub use self::light::{FrustumPlane, LightCullingSystem};
pub use self::scene::RenderScene;
pub use self::skinning::SkinningSystem;
pub use crate::core::render::graph::RenderGraphState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampledTargetBindKey {
    pub target_view_ptr: usize,
    pub outline_view_ptr: usize,
    pub ssao_view_ptr: usize,
    pub bloom_view_ptr: usize,
    pub uniform_buffer_ptr: usize,
}

pub struct RenderState {
    pub scene: RenderScene,
    pub detached_cameras: std::collections::HashMap<u32, crate::core::resources::CameraRecord>,
    pub camera_order: Vec<u32>,
    pub target_texture_binds:
        std::collections::HashMap<u32, crate::core::resources::TargetTextureBinding>,
    pub external_textures: std::collections::HashMap<u32, wgpu::TextureView>,
    pub external_texture_sources: std::collections::HashMap<u32, usize>,
    pub bindings: Option<BindingSystem>,
    pub library: Option<ResourceLibrary>,
    pub vertex: Option<VertexAllocatorSystem>,
    pub light_system: Option<LightCullingSystem>,
    pub gizmos: GizmoSystem,
    pub shadow: Option<ShadowManager>,
    pub cache: RenderCache,
    pub post_uniform_buffer: Option<wgpu::Buffer>,
    pub ssao_uniform_buffer: Option<wgpu::Buffer>,
    pub ssao_blur_uniform_buffer: Option<wgpu::Buffer>,
    pub bloom_uniform_buffer: Option<wgpu::Buffer>,
    pub skybox_uniform_buffer: Option<wgpu::Buffer>,
    pub environment: EnvironmentConfig,
    pub environment_is_configured: bool,
    pub camera_environment_overrides: std::collections::HashMap<u32, EnvironmentConfig>,
    pub compose_bind_cache: std::collections::HashMap<SampledTargetBindKey, wgpu::BindGroup>,
    pub post_bind_cache: std::collections::HashMap<SampledTargetBindKey, wgpu::BindGroup>,
    pub textures_sync_hash: u64,
    pub atlas_sync_hash: u64,
    pub target_binds_sync_hash: u64,
    pub light_prepare_sorted_ids: Vec<u32>,
    pub light_prepare_lights: Vec<crate::core::resources::LightComponent>,
    pub light_prepare_frustums: Vec<FrustumPlane>,
    pub rgba16f_msaa_supported_mask: u8,
    pub skinning: SkinningSystem,
    pub render_graph: RenderGraphState,
    pub ui_renderers: std::collections::HashMap<RealmId, UiRenderer>,

    /// Per-frame collector for draw calls, reused to avoid allocations.
    pub collector: DrawCollector,
}

impl RenderState {
    pub const MSAA_MASK_1: u8 = 1 << 0;
    pub const MSAA_MASK_2: u8 = 1 << 1;
    pub const MSAA_MASK_4: u8 = 1 << 2;
    pub const MSAA_MASK_8: u8 = 1 << 3;
    pub const MSAA_MASK_16: u8 = 1 << 4;
    pub const MSAA_MASK_DEFAULT_SAFE: u8 = Self::MSAA_MASK_1 | Self::MSAA_MASK_4;

    pub fn msaa_sample_count_for_format(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> u32 {
        self.msaa_sample_count_for_environment(&self.environment, device, format)
    }

    pub fn msaa_sample_count_for_environment(
        &self,
        environment: &EnvironmentConfig,
        _device: &wgpu::Device,
        _format: wgpu::TextureFormat,
    ) -> u32 {
        let requested = if environment.msaa.enabled && environment.msaa.sample_count >= 2 {
            environment.msaa.sample_count
        } else {
            1
        };

        if requested <= 1 {
            return 1;
        }

        let supported = if self.rgba16f_msaa_supported_mask == 0 {
            Self::MSAA_MASK_DEFAULT_SAFE
        } else {
            self.rgba16f_msaa_supported_mask
        };

        let mut best = 1_u32;
        if (supported & Self::MSAA_MASK_2) != 0 && requested >= 2 {
            best = 2;
        }
        if (supported & Self::MSAA_MASK_4) != 0 && requested >= 4 {
            best = 4;
        }
        if (supported & Self::MSAA_MASK_8) != 0 && requested >= 8 {
            best = 8;
        }
        if (supported & Self::MSAA_MASK_16) != 0 && requested >= 16 {
            best = 16;
        }
        best
    }

    pub fn environment_for_camera(&self, camera_id: u32) -> &EnvironmentConfig {
        self.camera_environment_overrides
            .get(&camera_id)
            .unwrap_or(&self.environment)
    }

    pub fn camera_record(&self, camera_id: u32) -> Option<&crate::core::resources::CameraRecord> {
        self.scene
            .cameras
            .get(&camera_id)
            .or_else(|| self.detached_cameras.get(&camera_id))
    }

    pub fn sync_camera_targets_and_projection(
        &mut self,
        device: &wgpu::Device,
        surface_size: glam::UVec2,
        camera_target_sizes: Option<&std::collections::HashMap<u32, glam::UVec2>>,
    ) -> bool {
        let mut any_camera_dirty = false;
        for (camera_id, record) in self.scene.cameras.iter_mut() {
            let requested_size = camera_target_sizes
                .and_then(|sizes| sizes.get(camera_id).copied())
                .unwrap_or_else(|| record.effective_target_size(surface_size));
            let projection_size =
                glam::UVec2::new(requested_size.x.max(1), requested_size.y.max(1));
            let target_width = projection_size.x;
            let target_height = projection_size.y;

            crate::core::resources::ensure_render_target(
                device,
                &mut record.render_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.emissive_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.post_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.outline_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.ssao_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.ssao_blur_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.bloom_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            for (level, target) in record.bloom_chain.iter_mut().enumerate() {
                let level_width = crate::core::render::bloom_chain_size(target_width, level);
                let level_height = crate::core::render::bloom_chain_size(target_height, level);
                crate::core::resources::ensure_render_target(
                    device,
                    target,
                    level_width,
                    level_height,
                    wgpu::TextureFormat::Rgba16Float,
                );
            }

            if record.last_projection_size != projection_size {
                record.data.update(
                    None,
                    None,
                    None,
                    None,
                    (target_width, target_height),
                    record.ortho_scale,
                );
                record.last_projection_size = projection_size;
                record.mark_dirty();
                any_camera_dirty = true;
            }
        }

        any_camera_dirty
    }

    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub fn on_resize(&mut self, _device: &wgpu::Device, _width: u32, _height: u32) {
        for record in self.scene.cameras.values_mut() {
            record.forward_depth_target = None;
            record.forward_msaa_target = None;
            record.forward_emissive_msaa_target = None;
        }
    }
}
