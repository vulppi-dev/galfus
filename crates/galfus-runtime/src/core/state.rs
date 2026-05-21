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
use crate::core::render::{RenderManager, ensure_runtime_render_defaults};
use crate::core::resources::{RenderTarget, TextureAsyncManager, TextureDecodeResult};
use crate::core::window::WindowManager;
use crate::core::{realm, target};
use galfus_input::GamepadState;
use galfus_log::LogLevel;
#[cfg(not(target_arch = "wasm32"))]
use galfus_platform::PlatformGamepadBackendState;
use std::collections::HashMap;
pub type EngineRuntimeState = galfus_runtime::RuntimeState<
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
    pub log_level: LogLevel,
    pub revision: u64,
    pub pending_texture_decode_results: Vec<TextureDecodeResult>,

    #[cfg(not(target_arch = "wasm32"))]
    pub input: InputState,
    pub(crate) gamepad: GamepadState,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) gamepad_backend: PlatformGamepadBackendState,

    pub(crate) profiling: TickProfiling,
    pub(crate) gpu_profiler: Option<GpuProfiler>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) debug_capture: DebugCaptureState,
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct DebugCaptureState {
    pub enabled: bool,
    pub capture_passes: bool,
    pub path_template: String,
    pub capture_every_frame: bool,
    pub captured_once: bool,
    pub downscale_factor: f32,
}

#[cfg(not(target_arch = "wasm32"))]
impl DebugCaptureState {
    fn from_env() -> Self {
        let path_template = std::env::var("GALFUS_DEBUG_CAPTURE_PATH")
            .ok()
            .filter(|path| !path.trim().is_empty())
            .unwrap_or_else(|| "build/debug/frame-{frame}.png".to_string());
        let enabled = std::env::var("GALFUS_DEBUG_CAPTURE")
            .map(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false);
        let capture_passes = std::env::var("GALFUS_DEBUG_CAPTURE_PASSES")
            .map(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
            .unwrap_or(false);
        let capture_every_frame = path_template.contains("{frame}");
        let downscale_factor = std::env::var("GALFUS_DEBUG_CAPTURE_SCALE")
            .ok()
            .and_then(|value| value.trim().parse::<f32>().ok())
            .map(|value| value.clamp(0.05, 1.0))
            .unwrap_or(1.0);
        Self {
            enabled,
            capture_passes,
            path_template,
            capture_every_frame,
            captured_once: false,
            downscale_factor,
        }
    }

    pub fn should_capture(&self) -> bool {
        self.enabled && (!self.captured_once || self.capture_every_frame)
    }

    pub fn resolve_path(&self, frame_index: u64, window_id: u32, surface_id: u32) -> String {
        self.path_template
            .replace("{frame}", &frame_index.to_string())
            .replace("{window}", &window_id.to_string())
            .replace("{surface}", &surface_id.to_string())
    }

    pub fn resolve_capture_size(&self, base_width: u32, base_height: u32) -> glam::UVec2 {
        let factor = self.downscale_factor.clamp(0.05, 1.0);
        if (factor - 1.0).abs() < f32::EPSILON {
            return glam::UVec2::new(base_width.max(1), base_height.max(1));
        }
        let width = ((base_width as f32) * factor).round().max(1.0) as u32;
        let height = ((base_height as f32) * factor).round().max(1.0) as u32;
        glam::UVec2::new(width, height)
    }
}

impl EngineState {
    pub fn new() -> Self {
        let mut universal_state = UniversalState::default();
        ensure_runtime_render_defaults(&mut universal_state);

        let mut state = Self {
            window: WindowManager::new(),
            render: RenderManager::default(),
            #[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
            wgpu: galfus_render::create_default_instance(),
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
            log_level: LogLevel::Info,
            revision: 0,
            pending_texture_decode_results: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            input: InputState::new(),
            gamepad: GamepadState::new(),
            #[cfg(not(target_arch = "wasm32"))]
            gamepad_backend: PlatformGamepadBackendState::new(),
            profiling: TickProfiling::default(),
            gpu_profiler: None,
            #[cfg(not(target_arch = "wasm32"))]
            debug_capture: DebugCaptureState::from_env(),
        };

        let _ = crate::core::resources::engine_cmd_material_definition_create(
            &mut state,
            &crate::core::resources::CmdMaterialDefinitionCreateArgs {
                definition_id: crate::core::resources::MATERIAL_DEFINITION_STANDARD_ID,
                slug: crate::core::resources::MATERIAL_DEFINITION_STANDARD_SLUG.to_string(),
                label: Some("builtin-standard".to_string()),
                preset: Some(crate::core::resources::ShaderMaterialPreset::Standard),
                shader_type: None,
                shader_source: None,
                shader_params_schema: None,
                capabilities: None,
            },
        );
        let _ = crate::core::resources::engine_cmd_material_definition_create(
            &mut state,
            &crate::core::resources::CmdMaterialDefinitionCreateArgs {
                definition_id: crate::core::resources::MATERIAL_DEFINITION_PBR_ID,
                slug: crate::core::resources::MATERIAL_DEFINITION_PBR_SLUG.to_string(),
                label: Some("builtin-pbr".to_string()),
                preset: Some(crate::core::resources::ShaderMaterialPreset::Pbr),
                shader_type: None,
                shader_source: None,
                shader_params_schema: None,
                capabilities: None,
            },
        );

        state
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::resources::{
        MATERIAL_DEFINITION_PBR_ID, MATERIAL_DEFINITION_STANDARD_ID, MaterialShaderType,
    };

    #[test]
    fn builtin_material_definitions_compile_without_parse_errors() {
        let state = EngineState::new();
        let defs = &state.universal_state.scene.material_definitions;

        let standard = defs
            .get(&MATERIAL_DEFINITION_STANDARD_ID)
            .expect("standard definition must exist");
        let pbr = defs
            .get(&MATERIAL_DEFINITION_PBR_ID)
            .expect("pbr definition must exist");

        for definition in [standard, pbr] {
            assert_eq!(definition.shader_type, MaterialShaderType::Model);
            assert!(definition.compile_error.is_none());
            let source = definition
                .compiled_shader_source
                .as_ref()
                .expect("compiled shader source must exist");
            assert!(source.contains("@vertex"));
            assert!(source.contains("@fragment"));
            assert!(source.contains("fn vs_main"));
            assert!(source.contains("fn fs_main"));
        }
    }
}
