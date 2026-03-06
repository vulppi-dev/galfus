use crate::core::platform::{Window, WindowId};
use crate::core::window::CursorIcon;
#[cfg(not(feature = "wasm"))]
use glam::IVec2;
use glam::{UVec2, Vec2};
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(not(feature = "wasm"))]
use std::time::Instant;

#[cfg(feature = "wasm")]
use wasm_bindgen::JsCast;
#[cfg(feature = "wasm")]
use wasm_bindgen::closure::Closure;
#[cfg(feature = "wasm")]
use web_sys::Event;
#[cfg(feature = "wasm")]
use web_sys::EventTarget;

#[cfg(not(feature = "wasm"))]
use crate::core::input::InputCacheManager;
use crate::core::resources::RenderTarget;

#[cfg(not(feature = "wasm"))]
use super::cache::WindowCacheManager;

/// Represents a window with its associated WGPU resources
#[cfg(feature = "wasm")]
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
    #[cfg(not(feature = "wasm"))]
    pub inner_position: IVec2,
    #[cfg(not(feature = "wasm"))]
    pub outer_position: IVec2,
    pub inner_size: UVec2,
    pub outer_size: UVec2,
    pub(crate) is_dirty: bool,
    #[cfg(not(feature = "wasm"))]
    pub(crate) last_present_instant: Option<Instant>,
    #[cfg(feature = "wasm")]
    pub(crate) last_present_ns: u64,
    pub(crate) last_frame_delta_ns: u64,
    pub(crate) fps_instant: f64,
    #[cfg(not(feature = "wasm"))]
    pub(crate) redraw_force_until_ms: u64,
    #[cfg(feature = "wasm")]
    pub web_listener_registrations: Vec<WebListenerRegistration>,
}

/// Aggregates window state, IDs and caches
pub struct WindowManager {
    pub states: HashMap<u32, WindowState>,
    pub window_id_map: HashMap<WindowId, u32>,
    pub cursor_positions: HashMap<u32, Vec2>,
    pub cursor_icon_override: HashMap<u32, CursorIcon>,
    #[cfg(not(feature = "wasm"))]
    pub cache: WindowCacheManager,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            window_id_map: HashMap::new(),
            cursor_positions: HashMap::new(),
            cursor_icon_override: HashMap::new(),
            #[cfg(not(feature = "wasm"))]
            cache: WindowCacheManager::new(),
        }
    }

    pub fn map_window(&mut self, winit_id: WindowId, engine_id: u32) {
        self.window_id_map.insert(winit_id, engine_id);
    }

    #[cfg(not(feature = "wasm"))]
    pub fn resolve_window_id(&self, winit_id: &WindowId) -> Option<u32> {
        self.window_id_map.get(winit_id).copied()
    }

    pub fn insert_state(&mut self, window_id: u32, state: WindowState) {
        self.states.insert(window_id, state);
    }

    #[cfg(feature = "wasm")]
    pub fn cleanup_window(&mut self, window_id: u32) -> bool {
        if let Some(mut window_state) = self.states.remove(&window_id) {
            self.window_id_map.remove(&window_state.window.id());
            self.cursor_positions.remove(&window_id);
            self.cursor_icon_override.remove(&window_id);
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

    #[cfg(not(feature = "wasm"))]
    pub fn cleanup_window(&mut self, window_id: u32, input_cache: &mut InputCacheManager) -> bool {
        if let Some(window_state) = self.states.remove(&window_id) {
            self.window_id_map.remove(&window_state.window.id());
            self.cache.remove(window_id);
            input_cache.remove_pointer(window_id);
            self.cursor_positions.remove(&window_id);
            self.cursor_icon_override.remove(&window_id);
            true
        } else {
            false
        }
    }
}
