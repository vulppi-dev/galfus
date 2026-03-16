#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
use super::RenderScene;
use super::RenderState;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
use crate::core::render::cache::RenderCache;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
use crate::core::render::gizmos::GizmoSystem;
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
use crate::core::render::state::collector::DrawCollector;
use crate::core::resources::{
    MATERIAL_FALLBACK_ID, MaterialStandardParams, MaterialStandardRecord,
};
#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
use std::collections::HashMap;

impl RenderState {
    const VERTEX_COMPACT_FRAME_INTERVAL: u64 = 120;
    const VERTEX_COMPACT_THRESHOLD: f32 = 0.25;
    const VERTEX_COMPACT_SLACK_RATIO: f32 = 0.3;
    const VERTEX_COMPACT_MIN_DEAD_BYTES: u64 = 256 * 1024;

    /// Create a new RenderState with empty systems
    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub fn new(_surface_format: wgpu::TextureFormat) -> Self {
        let mut materials_standard = HashMap::new();
        materials_standard.insert(
            MATERIAL_FALLBACK_ID,
            MaterialStandardRecord::new(
                Some("Fallback Material".into()),
                MaterialStandardParams::default(),
            ),
        );

        Self {
            scene: RenderScene {
                cameras: HashMap::new(),
                models: HashMap::new(),
                lights: HashMap::new(),
                materials_standard,
                materials_pbr: HashMap::new(),
                textures: HashMap::new(),
                forward_atlas_entries: HashMap::new(),
            },
            detached_cameras: HashMap::new(),
            camera_order: Vec::new(),
            target_texture_binds: HashMap::new(),
            external_textures: HashMap::new(),
            external_texture_sources: HashMap::new(),
            bindings: None,
            library: None,
            vertex: None,
            light_system: None,
            gizmos: GizmoSystem::new(),
            shadow: None,
            cache: RenderCache::new(),
            post_uniform_buffer: None,
            compose_uniform_buffer: None,
            ssao_uniform_buffer: None,
            ssao_blur_uniform_buffer: None,
            bloom_uniform_buffer: None,
            skybox_uniform_buffer: None,
            collector: DrawCollector::default(),
            skinning: crate::core::render::state::SkinningSystem::default(),
            render_graph: crate::core::render::graph::RenderGraphState::new(),
            ui_renderers: HashMap::new(),
            environment: crate::core::resources::EnvironmentConfig::default(),
            environment_is_configured: false,
            camera_environment_overrides: HashMap::new(),
            compose_bind_cache: HashMap::new(),
            post_bind_cache: HashMap::new(),
            textures_sync_hash: 0,
            atlas_sync_hash: 0,
            target_binds_sync_hash: 0,
            light_prepare_sorted_ids: Vec::new(),
            light_prepare_lights: Vec::new(),
            light_prepare_frustums: Vec::new(),
            rgba16f_msaa_supported_mask: RenderState::MSAA_MASK_DEFAULT_SAFE,
        }
    }

    /// Explicitly drop all render state resources
    pub fn drop_all(&mut self) {
        self.scene.cameras.clear();
        self.detached_cameras.clear();
        self.camera_order.clear();
        self.scene.models.clear();
        self.scene.lights.clear();
        self.scene.materials_standard.clear();
        self.scene.materials_standard.insert(
            MATERIAL_FALLBACK_ID,
            MaterialStandardRecord::new(
                Some("Fallback Material".into()),
                MaterialStandardParams::default(),
            ),
        );
        self.scene.materials_pbr.clear();
        self.scene.textures.clear();
        self.scene.forward_atlas_entries.clear();
        self.target_texture_binds.clear();
        self.external_textures.clear();
        self.external_texture_sources.clear();
        self.bindings = None;
        self.library = None;
        self.vertex = None;
        self.light_system = None;
        self.shadow = None;
        self.cache.clear();
        self.post_uniform_buffer = None;
        self.compose_uniform_buffer = None;
        self.ssao_uniform_buffer = None;
        self.ssao_blur_uniform_buffer = None;
        self.bloom_uniform_buffer = None;
        self.skybox_uniform_buffer = None;
        self.skinning.clear();
        self.render_graph.reset_to_fallback();
        self.ui_renderers.clear();
        self.environment = crate::core::resources::EnvironmentConfig::default();
        self.environment_is_configured = false;
        self.camera_environment_overrides.clear();
        self.compose_bind_cache.clear();
        self.post_bind_cache.clear();
        self.textures_sync_hash = 0;
        self.atlas_sync_hash = 0;
        self.target_binds_sync_hash = 0;
        self.light_prepare_sorted_ids.clear();
        self.light_prepare_lights.clear();
        self.light_prepare_frustums.clear();
    }

    pub fn begin_frame(&mut self, frame_index: u64) {
        if let Some(vertex) = self.vertex.as_mut() {
            vertex.begin_frame(frame_index);
            if frame_index % Self::VERTEX_COMPACT_FRAME_INTERVAL == 0 {
                vertex.maybe_compact_all(
                    frame_index,
                    Self::VERTEX_COMPACT_THRESHOLD,
                    Self::VERTEX_COMPACT_SLACK_RATIO,
                    Self::VERTEX_COMPACT_MIN_DEAD_BYTES,
                );
            }
        }
        if let Some(bindings) = self.bindings.as_mut() {
            bindings.frame_pool.begin_frame(frame_index);
            bindings.camera_pool.begin_frame(frame_index);
            bindings.model_pool.begin_frame(frame_index);
            bindings.instance_pool.begin_frame(frame_index);
            bindings.outline_instance_pool.begin_frame(frame_index);
            bindings.shadow_instance_pool.begin_frame(frame_index);
            bindings.material_standard_pool.begin_frame(frame_index);
            bindings.material_standard_inputs.begin_frame(frame_index);
            bindings.material_pbr_pool.begin_frame(frame_index);
            bindings.material_pbr_inputs.begin_frame(frame_index);
            bindings.bones_pool.begin_frame(frame_index);
        }
        if let Some(light_system) = self.light_system.as_mut() {
            light_system.lights.begin_frame(frame_index);
            light_system.visible_indices.begin_frame(frame_index);
            light_system.visible_counts.begin_frame(frame_index);
            light_system.camera_frustums.begin_frame(frame_index);
            light_system.light_params.begin_frame(frame_index);
        }
        if let Some(shadow) = self.shadow.as_mut() {
            shadow.begin_frame(frame_index);
        }
        self.gizmos.clear();
        self.cache.gc(frame_index);
    }
}
