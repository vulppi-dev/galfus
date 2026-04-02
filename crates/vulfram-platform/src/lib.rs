pub mod bootstrap;
pub mod browser;
pub mod gamepad;
pub mod window;

use std::collections::HashSet;

pub use bootstrap::{
    PlatformRenderBootstrapTarget, PlatformRenderSurfaceKind, PlatformSurfaceAlphaMode,
    plan_native_render_bootstrap_target, plan_web_render_bootstrap_target,
};
pub use browser::{
    BrowserCursorCommandPlan, BrowserPointerCaptureUpdate, BrowserPointerMotionInput,
    BrowserSurfaceResizePlan, PlatformCursorGrabMode, PlatformWindowState,
    map_browser_pointer_type, normalize_browser_key_text, plan_browser_cursor_mode_change,
    plan_browser_surface_resize, resolve_browser_pointer_position, resolve_browser_window_state,
    resolve_canvas_surface_size, resolve_pointer_lock_change, resolve_pointer_lock_error,
};
pub use gamepad::{PlatformGamepadBackendState, map_gilrs_axis, map_gilrs_button};
#[cfg(not(target_arch = "wasm32"))]
pub use gamepad::{
    PlatformGamepadEvent, PlatformGamepadEventType, drain_gilrs_events, resolve_gilrs_gamepad_name,
};
pub use window::{
    PlatformFullscreenMode, PlatformWindowLifecycleState, resolve_platform_window_state,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformActivityEvent {
    Pointer { window_id: u32 },
    Keyboard { window_id: u32 },
    Window { window_id: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowRedrawPlan {
    pub window_id: u32,
    pub redraw_force_until_ms: u64,
    pub should_request_redraw: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowRedrawInput {
    pub window_id: u32,
    pub redraw_force_until_ms: u64,
    pub is_dirty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedrawContext {
    pub now_ms: u64,
    pub had_commands_this_frame: bool,
    pub has_ui_animations: bool,
    pub has_ui_repaint_request: bool,
    pub has_pending_texture_work: bool,
    pub has_unbound_ui_async_loading: bool,
}

pub const INPUT_REDRAW_GRACE_MS: u64 = 250;

pub fn active_windows_from_events(events: &[PlatformActivityEvent]) -> HashSet<u32> {
    events
        .iter()
        .map(|event| match event {
            PlatformActivityEvent::Pointer { window_id }
            | PlatformActivityEvent::Keyboard { window_id }
            | PlatformActivityEvent::Window { window_id } => *window_id,
        })
        .collect()
}

pub fn plan_window_redraws(
    events: &[PlatformActivityEvent],
    pending_ui_image_windows: &HashSet<u32>,
    windows: &[WindowRedrawInput],
    context: RedrawContext,
) -> Vec<WindowRedrawPlan> {
    let input_windows = active_windows_from_events(events);
    windows
        .iter()
        .map(|window| {
            let redraw_force_until_ms = if input_windows.contains(&window.window_id) {
                context.now_ms.saturating_add(INPUT_REDRAW_GRACE_MS)
            } else {
                window.redraw_force_until_ms
            };
            let has_recent_input = context.now_ms <= redraw_force_until_ms;
            let should_request_redraw = window.is_dirty
                || context.had_commands_this_frame
                || has_recent_input
                || context.has_ui_animations
                || context.has_ui_repaint_request
                || context.has_pending_texture_work
                || pending_ui_image_windows.contains(&window.window_id)
                || context.has_unbound_ui_async_loading;
            WindowRedrawPlan {
                window_id: window.window_id,
                redraw_force_until_ms,
                should_request_redraw,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        INPUT_REDRAW_GRACE_MS, PlatformActivityEvent, RedrawContext, WindowRedrawInput,
        active_windows_from_events, plan_window_redraws,
    };
    use std::collections::HashSet;

    #[test]
    fn active_windows_from_events_deduplicates_window_ids() {
        let windows = active_windows_from_events(&[
            PlatformActivityEvent::Pointer { window_id: 2 },
            PlatformActivityEvent::Keyboard { window_id: 2 },
            PlatformActivityEvent::Window { window_id: 3 },
        ]);
        assert_eq!(windows.len(), 2);
        assert!(windows.contains(&2));
        assert!(windows.contains(&3));
    }

    #[test]
    fn redraw_plan_extends_input_grace_and_requests_redraw() {
        let plans = plan_window_redraws(
            &[PlatformActivityEvent::Pointer { window_id: 7 }],
            &HashSet::new(),
            &[WindowRedrawInput {
                window_id: 7,
                redraw_force_until_ms: 0,
                is_dirty: false,
            }],
            RedrawContext {
                now_ms: 1000,
                had_commands_this_frame: false,
                has_ui_animations: false,
                has_ui_repaint_request: false,
                has_pending_texture_work: false,
                has_unbound_ui_async_loading: false,
            },
        );
        assert_eq!(plans[0].redraw_force_until_ms, 1000 + INPUT_REDRAW_GRACE_MS);
        assert!(plans[0].should_request_redraw);
    }

    #[test]
    fn redraw_plan_respects_pending_ui_windows_without_input() {
        let mut pending = HashSet::new();
        pending.insert(9);
        let plans = plan_window_redraws(
            &[],
            &pending,
            &[WindowRedrawInput {
                window_id: 9,
                redraw_force_until_ms: 0,
                is_dirty: false,
            }],
            RedrawContext {
                now_ms: 100,
                had_commands_this_frame: false,
                has_ui_animations: false,
                has_ui_repaint_request: false,
                has_pending_texture_work: false,
                has_unbound_ui_async_loading: false,
            },
        );
        assert!(plans[0].should_request_redraw);
    }
}
