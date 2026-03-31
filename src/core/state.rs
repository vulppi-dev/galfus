use crate::core::audio::AudioProxy;
#[cfg(not(feature = "wasm"))]
use crate::core::audio::KiraAudioProxy;
#[cfg(feature = "wasm")]
use crate::core::audio::WebAudioProxy;
use crate::core::buffers::state::BufferStorage;
use crate::core::cmd::{EngineBatchCmds, EngineBatchEvents, EngineBatchResponses};
use crate::core::gamepad::state::GamepadState;
#[cfg(not(feature = "wasm"))]
use crate::core::input::InputState;
use crate::core::profiling::TickProfiling;
use crate::core::profiling::gpu::GpuProfiler;
use crate::core::realm::{AudioState, UniversalState};
use crate::core::render::RenderManager;
use crate::core::resources::{
    MATERIAL_FALLBACK_ID, MaterialStandardParams, MaterialStandardRecord, RenderTarget,
    TextureAsyncManager, TextureDecodeResult,
};
use crate::core::window::WindowManager;
use std::collections::HashMap;
pub use vulfram_runtime::{DeferredCommandKey, DeferredCommandMeta, RuntimeFrameState};

/// Main engine state holding all runtime data
pub struct EngineState {
    pub window: WindowManager,
    pub render: RenderManager,

    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub wgpu: wgpu::Instance,
    #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
    pub caps: Option<wgpu::SurfaceCapabilities>,
    pub rgba16f_msaa_supported_mask: u8,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,

    pub buffers: BufferStorage,
    pub texture_async: TextureAsyncManager,
    pub audio: Box<dyn AudioProxy>,
    pub audio_available: bool,
    pub audio_state: AudioState,
    pub universal_state: UniversalState,
    pub surface_targets: HashMap<crate::core::realm::SurfaceId, RenderTarget>,
    pub present_sizes_cache: HashMap<crate::core::realm::SurfaceId, glam::UVec2>,
    pub present_sizes_hash: u64,

    pub cmd_queue: EngineBatchCmds,
    pub deferred_cmd_queue: EngineBatchCmds,
    pub deferred_cmd_meta: HashMap<DeferredCommandKey, DeferredCommandMeta>,
    pub event_queue: EngineBatchEvents,
    pub response_queue: EngineBatchResponses,
    pub pending_texture_decode_results: Vec<TextureDecodeResult>,

    pub(crate) frame: RuntimeFrameState,

    #[cfg(not(feature = "wasm"))]
    pub input: InputState,
    pub(crate) gamepad: GamepadState,

    pub(crate) profiling: TickProfiling,
    pub(crate) gpu_profiler: Option<GpuProfiler>,
}

impl EngineState {
    pub fn new() -> Self {
        #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
        let wgpu_descriptor = if cfg!(target_arch = "wasm32") {
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::BROWSER_WEBGPU,
                backend_options: wgpu::BackendOptions::default(),
                flags: wgpu::InstanceFlags::empty(),
                memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            }
        } else {
            wgpu::InstanceDescriptor {
                backends: if cfg!(target_os = "ios") || cfg!(target_os = "macos") {
                    wgpu::Backends::METAL | wgpu::Backends::VULKAN
                } else {
                    wgpu::Backends::DX12 | wgpu::Backends::VULKAN
                },
                backend_options: wgpu::BackendOptions::default(),
                flags: wgpu::InstanceFlags::empty(),
                memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
            }
        };

        let mut universal_state = UniversalState::default();
        crate::core::render::graph::ensure_default_render_graphs(
            &mut universal_state.render_graphs,
            &mut universal_state.render_graph_plan_cache,
        );
        universal_state
            .universal_resources
            .materials_standard
            .insert(
                MATERIAL_FALLBACK_ID,
                MaterialStandardRecord::new(
                    Some("Fallback Material".into()),
                    MaterialStandardParams::default(),
                ),
            );

        Self {
            window: WindowManager::new(),
            render: RenderManager::default(),
            #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
            wgpu: wgpu::Instance::new(&wgpu_descriptor),
            #[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
            caps: None,
            rgba16f_msaa_supported_mask: crate::core::render::RenderState::MSAA_MASK_DEFAULT_SAFE,
            device: None,
            queue: None,
            buffers: BufferStorage::new(),
            texture_async: TextureAsyncManager::new(),
            #[cfg(not(feature = "wasm"))]
            audio: Box::new(KiraAudioProxy::default()),
            #[cfg(feature = "wasm")]
            audio: Box::new(WebAudioProxy::default()),
            audio_available: true,
            audio_state: AudioState::default(),
            universal_state,
            surface_targets: HashMap::new(),
            present_sizes_cache: HashMap::new(),
            present_sizes_hash: 0,
            cmd_queue: Vec::new(),
            deferred_cmd_queue: Vec::new(),
            deferred_cmd_meta: HashMap::new(),
            event_queue: Vec::new(),
            response_queue: Vec::new(),
            pending_texture_decode_results: Vec::new(),
            frame: RuntimeFrameState::default(),
            #[cfg(not(feature = "wasm"))]
            input: InputState::new(),
            gamepad: GamepadState::new(),
            profiling: TickProfiling::default(),
            gpu_profiler: None,
        }
    }

    pub fn cleanup_window(&mut self, window_id: u32) -> bool {
        #[cfg(feature = "wasm")]
        let cleaned = self.window.cleanup_window(window_id);

        #[cfg(not(feature = "wasm"))]
        let cleaned = self.window.cleanup_window(window_id, &mut self.input.cache);

        if cleaned {
            if let Some(mut render_state) = self.render.remove(window_id) {
                render_state.drop_all();
            }
            let targets_to_remove: std::collections::HashSet<_> = self
                .universal_state
                .targets
                .entries
                .iter()
                .filter_map(|(target_id, target)| {
                    if target.window_id == Some(window_id) {
                        Some(*target_id)
                    } else {
                        None
                    }
                })
                .collect();
            if !targets_to_remove.is_empty() {
                let layers_to_remove: Vec<_> = self
                    .universal_state
                    .target_layers
                    .entries
                    .keys()
                    .filter(|(_, layer_target)| targets_to_remove.contains(layer_target))
                    .copied()
                    .collect();
                for (layer_realm_id, layer_target_id) in layers_to_remove {
                    self.universal_state
                        .target_layers
                        .entries
                        .remove(&(layer_realm_id, layer_target_id));
                    crate::core::target::resolve::remove_auto_link_for_layer(
                        &mut self.universal_state,
                        layer_realm_id,
                        layer_target_id,
                    );
                }
                self.universal_state
                    .targets
                    .entries
                    .retain(|target_id, _| !targets_to_remove.contains(target_id));
                self.universal_state
                    .universal_resources
                    .target_texture_binds
                    .retain(|_, binding| !targets_to_remove.contains(&binding.target_id));
                self.universal_state
                    .ui
                    .external_textures
                    .retain(|target_id, _| {
                        !targets_to_remove.contains(&crate::core::target::TargetId(*target_id))
                    });
                self.universal_state
                    .ui
                    .target_size_requests
                    .retain(|target_id, _| {
                        !targets_to_remove.contains(&crate::core::target::TargetId(*target_id))
                    });
                self.universal_state
                    .target_ui_realm_index
                    .retain(|target_id, _| !targets_to_remove.contains(target_id));
                let _ = self
                    .universal_state
                    .target_listeners
                    .dispose_targets(targets_to_remove.iter().map(|target_id| target_id.0));
                self.universal_state
                    .input_routing
                    .focus_targets
                    .retain(|_, target_id| !targets_to_remove.contains(target_id));
            }

            let surfaces_to_remove: Vec<_> = self
                .universal_state
                .presents
                .entries
                .values()
                .filter(|present| present.value.window_id == window_id)
                .map(|present| present.value.surface)
                .collect();
            self.universal_state.presents.remove_by_window(window_id);
            self.universal_state
                .input_routing
                .captures
                .retain(|(capture_window, _), _| *capture_window != window_id);
            self.universal_state
                .input_routing
                .focus_targets
                .retain(|focus_window, _| *focus_window != window_id);
            if !surfaces_to_remove.is_empty() {
                let surface_set: std::collections::HashSet<_> =
                    surfaces_to_remove.iter().copied().collect();
                let mut realms_to_remove = Vec::new();
                for (realm_id, entry) in self.universal_state.realms.entries.iter() {
                    if entry
                        .value
                        .output_surface
                        .is_some_and(|surface| surface_set.contains(&surface))
                    {
                        realms_to_remove.push(*realm_id);
                    }
                }
                let realm_set: std::collections::HashSet<_> =
                    realms_to_remove.iter().copied().collect();
                for realm_id in realms_to_remove {
                    self.universal_state.realms.remove(realm_id);
                    self.universal_state.realm_entities.remove(&realm_id);
                    self.universal_state.ui.remove_realm(realm_id);
                    self.universal_state
                        .host_realm_index
                        .retain(|_, indexed_realm_id| *indexed_realm_id != realm_id);
                }
                for surface_id in &surfaces_to_remove {
                    self.universal_state.surfaces.remove(*surface_id);
                    self.surface_targets.remove(surface_id);
                }
                self.universal_state
                    .auto_links
                    .retain(|_, link| !surface_set.contains(&link.surface_id));
                let mut removed_connectors = Vec::new();
                self.universal_state
                    .connectors
                    .entries
                    .retain(|connector_id, entry| {
                        let remove = surface_set.contains(&entry.value.source_surface)
                            || realm_set.contains(&entry.value.target_realm);
                        if remove {
                            removed_connectors.push(*connector_id);
                        }
                        !remove
                    });
                if !removed_connectors.is_empty() {
                    let removed_set: std::collections::HashSet<_> =
                        removed_connectors.into_iter().collect();
                    self.universal_state
                        .input_routing
                        .captures
                        .retain(|_, capture| {
                            !removed_set
                                .contains(&crate::core::realm::ConnectorId(capture.connector_id))
                        });
                }
                self.universal_state
                    .surface_cache
                    .last_good
                    .retain(|_, source| !surface_set.contains(source));
                self.universal_state
                    .surface_cache
                    .fallback
                    .retain(|_, source| !surface_set.contains(source));
            }
            self.universal_state.target_graph_cache.prune_dead_entries(
                &self.universal_state.targets.entries,
                &self.universal_state.target_layers.entries,
                &self.universal_state.realms,
            );
        }
        cleaned
    }
}
