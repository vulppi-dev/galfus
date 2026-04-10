use crate::core::platform::{Window, WindowId};
use crate::core::window::{CursorGrabMode, CursorIcon, EngineWindowState};
#[cfg(not(target_arch = "wasm32"))]
use glam::IVec2;
use glam::{UVec2, Vec2};
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use web_sys::Event;
#[cfg(target_arch = "wasm32")]
use web_sys::EventTarget;

#[cfg(not(target_arch = "wasm32"))]
use crate::core::input::InputCacheManager;
use crate::core::resources::RenderTarget;

#[cfg(not(target_arch = "wasm32"))]
use super::cache::WindowCacheManager;

/// Represents a window with its associated WGPU resources
#[cfg(target_arch = "wasm32")]
pub struct WebListenerRegistration {
    pub target: EventTarget,
    pub event_type: &'static str,
    pub callback: Closure<dyn FnMut(Event)>,
}

pub struct WindowState {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub surface_target: Option<RenderTarget>,
    #[cfg(not(target_arch = "wasm32"))]
    pub inner_position: IVec2,
    #[cfg(not(target_arch = "wasm32"))]
    pub outer_position: IVec2,
    pub inner_size: UVec2,
    pub outer_size: UVec2,
    pub(crate) is_dirty: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) last_present_instant: Option<Instant>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) last_present_ns: u64,
    pub(crate) last_frame_delta_ns: u64,
    pub(crate) fps_instant: f64,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) redraw_force_until_ms: u64,
    #[cfg(target_arch = "wasm32")]
    pub web_listener_registrations: Vec<WebListenerRegistration>,
}

impl WindowState {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_native(
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        inner_position: IVec2,
        outer_position: IVec2,
        inner_size: UVec2,
        outer_size: UVec2,
        surface_target: Option<RenderTarget>,
    ) -> Self {
        Self {
            window,
            surface,
            config,
            surface_target,
            inner_position,
            outer_position,
            inner_size,
            outer_size,
            is_dirty: true,
            last_present_instant: None,
            last_frame_delta_ns: 0,
            fps_instant: 0.0,
            redraw_force_until_ms: 0,
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new_web(
        window: Arc<Window>,
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        size: UVec2,
        surface_target: Option<RenderTarget>,
        web_listener_registrations: Vec<WebListenerRegistration>,
    ) -> Self {
        Self {
            window,
            surface,
            config,
            surface_target,
            inner_size: size,
            outer_size: size,
            is_dirty: true,
            last_present_ns: 0,
            last_frame_delta_ns: 0,
            fps_instant: 0.0,
            web_listener_registrations,
        }
    }
}

/// Aggregates window state, IDs and caches
pub struct WindowManager {
    pub states: HashMap<u32, WindowState>,
    pub window_id_map: HashMap<WindowId, u32>,
    pub cursor_positions: HashMap<u32, Vec2>,
    pub cursor_icon_override: HashMap<u32, CursorIcon>,
    pub cursor_grab_modes: HashMap<u32, CursorGrabMode>,
    pub pointer_capture_active: HashMap<u32, bool>,
    pub canvas_active: HashMap<u32, bool>,
    pub lifecycle_states: HashMap<u32, EngineWindowState>,
    #[cfg(not(target_arch = "wasm32"))]
    pub cache: WindowCacheManager,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            window_id_map: HashMap::new(),
            cursor_positions: HashMap::new(),
            cursor_icon_override: HashMap::new(),
            cursor_grab_modes: HashMap::new(),
            pointer_capture_active: HashMap::new(),
            canvas_active: HashMap::new(),
            lifecycle_states: HashMap::new(),
            #[cfg(not(target_arch = "wasm32"))]
            cache: WindowCacheManager::new(),
        }
    }

    pub fn map_window(&mut self, winit_id: WindowId, engine_id: u32) {
        self.window_id_map.insert(winit_id, engine_id);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn resolve_window_id(&self, winit_id: &WindowId) -> Option<u32> {
        self.window_id_map.get(winit_id).copied()
    }

    pub fn insert_state(&mut self, window_id: u32, state: WindowState) {
        self.states.insert(window_id, state);
    }

    pub fn initialize_window_defaults(&mut self, window_id: u32) {
        self.set_cursor_grab_mode(window_id, CursorGrabMode::None);
        self.set_pointer_capture_active(window_id, false);
        self.set_canvas_active(window_id, false);
        self.set_lifecycle_state(window_id, EngineWindowState::Windowed);
    }

    pub fn set_cursor_grab_mode(&mut self, window_id: u32, mode: CursorGrabMode) {
        self.cursor_grab_modes.insert(window_id, mode);
    }

    pub fn cursor_grab_mode(&self, window_id: u32) -> CursorGrabMode {
        self.cursor_grab_modes
            .get(&window_id)
            .copied()
            .unwrap_or(CursorGrabMode::None)
    }

    pub fn set_pointer_capture_active(&mut self, window_id: u32, active: bool) -> bool {
        let changed = self.pointer_capture_active.get(&window_id).copied() != Some(active);
        self.pointer_capture_active.insert(window_id, active);
        changed
    }

    pub fn set_canvas_active(&mut self, window_id: u32, active: bool) -> bool {
        let changed = self.canvas_active.get(&window_id).copied() != Some(active);
        self.canvas_active.insert(window_id, active);
        changed
    }

    #[cfg(target_arch = "wasm32")]
    pub fn canvas_active(&self, window_id: u32) -> bool {
        self.canvas_active.get(&window_id).copied().unwrap_or(false)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn pointer_capture_active(&self, window_id: u32) -> bool {
        self.pointer_capture_active
            .get(&window_id)
            .copied()
            .unwrap_or(false)
    }

    pub fn set_lifecycle_state(&mut self, window_id: u32, state: EngineWindowState) -> bool {
        let changed = self.lifecycle_states.get(&window_id).copied() != Some(state);
        self.lifecycle_states.insert(window_id, state);
        changed
    }

    #[cfg(target_arch = "wasm32")]
    pub fn cleanup_window(&mut self, window_id: u32) -> bool {
        if let Some(mut window_state) = self.states.remove(&window_id) {
            self.window_id_map.remove(&window_state.window.id());
            self.cursor_positions.remove(&window_id);
            self.cursor_icon_override.remove(&window_id);
            self.cursor_grab_modes.remove(&window_id);
            self.pointer_capture_active.remove(&window_id);
            self.canvas_active.remove(&window_id);
            self.lifecycle_states.remove(&window_id);
            for registration in window_state.web_listener_registrations.drain(..) {
                let _ = registration.target.remove_event_listener_with_callback(
                    registration.event_type,
                    registration.callback.as_ref().unchecked_ref(),
                );
            }
            window_state.surface_target = None;
            true
        } else {
            false
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn cleanup_window(&mut self, window_id: u32, input_cache: &mut InputCacheManager) -> bool {
        if let Some(window_state) = self.states.remove(&window_id) {
            self.window_id_map.remove(&window_state.window.id());
            self.cache.remove(window_id);
            input_cache.remove_pointer(window_id);
            self.cursor_positions.remove(&window_id);
            self.cursor_icon_override.remove(&window_id);
            self.cursor_grab_modes.remove(&window_id);
            self.pointer_capture_active.remove(&window_id);
            self.canvas_active.remove(&window_id);
            self.lifecycle_states.remove(&window_id);
            true
        } else {
            false
        }
    }
}
