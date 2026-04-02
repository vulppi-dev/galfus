use crate::core::audio::AudioProxy;
#[cfg(not(target_arch = "wasm32"))]
use crate::core::audio::KiraAudioProxy;
#[cfg(target_arch = "wasm32")]
use crate::core::audio::WebAudioProxy;
use crate::core::buffers::state::BufferStorage;
#[cfg(not(target_arch = "wasm32"))]
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
use crate::core::{realm, target};
use std::collections::HashMap;
use vulfram_input::GamepadState;
use vulfram_platform::PlatformGamepadBackendState;
pub type EngineRuntimeState = vulfram_runtime::RuntimeState<
    crate::core::cmd::EngineCmdEnvelope,
    crate::core::cmd::EngineEvent,
    crate::core::cmd::CommandResponseEnvelope,
>;

/// Main engine state holding all runtime data
pub struct EngineState {
    pub window: WindowManager,
    pub render: RenderManager,

    #[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
    pub wgpu: wgpu::Instance,
    #[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
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

    pub runtime: EngineRuntimeState,
    pub pending_texture_decode_results: Vec<TextureDecodeResult>,

    #[cfg(not(target_arch = "wasm32"))]
    pub input: InputState,
    pub(crate) gamepad: GamepadState,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) gamepad_backend: PlatformGamepadBackendState,

    pub(crate) profiling: TickProfiling,
    pub(crate) gpu_profiler: Option<GpuProfiler>,
}

impl EngineState {
    pub fn new() -> Self {
        let mut universal_state = UniversalState::default();
        crate::core::render::graph::ensure_default_render_graphs(
            &mut universal_state.scene.render_graphs,
            &mut universal_state.scene.render_graph_plan_cache,
        );
        universal_state.scene.realm3d.materials_standard.insert(
            MATERIAL_FALLBACK_ID,
            MaterialStandardRecord::new(
                Some("Fallback Material".into()),
                MaterialStandardParams::default(),
            ),
        );

        Self {
            window: WindowManager::new(),
            render: RenderManager::default(),
            #[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
            wgpu: vulfram_render::create_default_instance(),
            #[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
            caps: None,
            rgba16f_msaa_supported_mask: crate::core::render::RenderState::MSAA_MASK_DEFAULT_SAFE,
            device: None,
            queue: None,
            buffers: BufferStorage::new(),
            texture_async: TextureAsyncManager::new(),
            #[cfg(not(target_arch = "wasm32"))]
            audio: Box::new(KiraAudioProxy::default()),
            #[cfg(target_arch = "wasm32")]
            audio: Box::new(WebAudioProxy::default()),
            audio_available: true,
            audio_state: AudioState::default(),
            universal_state,
            surface_targets: HashMap::new(),
            present_sizes_cache: HashMap::new(),
            present_sizes_hash: 0,
            runtime: EngineRuntimeState::default(),
            pending_texture_decode_results: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            input: InputState::new(),
            gamepad: GamepadState::new(),
            #[cfg(not(target_arch = "wasm32"))]
            gamepad_backend: PlatformGamepadBackendState::new(),
            profiling: TickProfiling::default(),
            gpu_profiler: None,
        }
    }

    pub fn cleanup_window(&mut self, window_id: u32) -> bool {
        #[cfg(target_arch = "wasm32")]
        let cleaned = self.window.cleanup_window(window_id);

        #[cfg(not(target_arch = "wasm32"))]
        let cleaned = self.window.cleanup_window(window_id, &mut self.input.cache);

        if cleaned {
            if let Some(mut render_state) = self.render.remove(window_id) {
                render_state.drop_all();
            }
            target::dispose_window_targets(&mut self.universal_state, window_id);

            self.universal_state
                .interaction
                .input_routing
                .captures
                .retain(|(capture_window, _), _| *capture_window != window_id);
            self.universal_state
                .interaction
                .input_routing
                .focus_targets
                .retain(|focus_window, _| *focus_window != window_id);
            realm::dispose_surfaces_for_window(
                &mut self.universal_state,
                &mut self.surface_targets,
                window_id,
            );
            target::prune_target_graph_cache(&mut self.universal_state);
        }
        cleaned
    }
}
