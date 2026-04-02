use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformCursorGrabMode {
    None,
    Confined,
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformWindowState {
    Windowed,
    Fullscreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserPointerCaptureUpdate {
    pub active: bool,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserSurfaceResizePlan {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowserCursorCommandPlan {
    pub active: bool,
    pub reason: &'static str,
    pub message: &'static str,
    pub request_pointer_lock: bool,
    pub exit_pointer_lock: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BrowserPointerMotionInput {
    pub cursor_grab_mode: PlatformCursorGrabMode,
    pub pointer_capture_active: bool,
    pub absolute_position: Vec2,
    pub movement: Vec2,
    pub last_position: Option<Vec2>,
    pub window_size: Option<Vec2>,
}

pub fn browser_now_ns(now_ms: f64) -> u64 {
    (now_ms * 1_000_000.0) as u64
}

pub fn should_poll_browser_gamepads(has_windows: bool, document_has_focus: bool) -> bool {
    has_windows && document_has_focus
}

pub fn map_browser_pointer_type(pointer_type: &str) -> u32 {
    match pointer_type {
        "mouse" => 0,
        "touch" => 1,
        "pen" => 2,
        _ => 0,
    }
}

pub fn resolve_browser_window_state(is_fullscreen: bool) -> PlatformWindowState {
    if is_fullscreen {
        PlatformWindowState::Fullscreen
    } else {
        PlatformWindowState::Windowed
    }
}

pub fn normalize_browser_key_text(key: &str, is_composing: bool) -> Option<String> {
    if is_composing {
        return None;
    }
    let mut chars = key.chars();
    let _first = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some(key.to_owned())
}

pub fn resolve_canvas_surface_size(
    rect_width: f64,
    rect_height: f64,
    device_pixel_ratio: f64,
) -> (u32, u32) {
    let safe_dpr = device_pixel_ratio.max(1.0);
    let width = (rect_width * safe_dpr).round().max(1.0) as u32;
    let height = (rect_height * safe_dpr).round().max(1.0) as u32;
    (width, height)
}

pub fn plan_browser_surface_resize(
    current_width: u32,
    current_height: u32,
    next_width: u32,
    next_height: u32,
) -> Option<BrowserSurfaceResizePlan> {
    let width = next_width.max(1);
    let height = next_height.max(1);
    if current_width.max(1) == width && current_height.max(1) == height {
        return None;
    }
    Some(BrowserSurfaceResizePlan { width, height })
}

pub fn resolve_browser_pointer_position(input: BrowserPointerMotionInput) -> Vec2 {
    let use_relative = match input.cursor_grab_mode {
        PlatformCursorGrabMode::None => false,
        PlatformCursorGrabMode::Confined => true,
        PlatformCursorGrabMode::Locked => input.pointer_capture_active,
    };
    if !use_relative {
        return input.absolute_position;
    }
    let fallback = input
        .window_size
        .map(|size| Vec2::new(size.x * 0.5, size.y * 0.5))
        .unwrap_or(input.absolute_position);
    let base = input.last_position.unwrap_or(fallback);
    base + input.movement
}

pub fn resolve_pointer_lock_change(
    cursor_grab_mode: PlatformCursorGrabMode,
    canvas_has_pointer_lock: bool,
) -> Option<BrowserPointerCaptureUpdate> {
    if cursor_grab_mode != PlatformCursorGrabMode::Locked {
        return None;
    }
    Some(BrowserPointerCaptureUpdate {
        active: canvas_has_pointer_lock,
        reason: "pointer-lock-change",
    })
}

pub fn resolve_pointer_lock_error(
    cursor_grab_mode: PlatformCursorGrabMode,
) -> Option<BrowserPointerCaptureUpdate> {
    if cursor_grab_mode != PlatformCursorGrabMode::Locked {
        return None;
    }
    Some(BrowserPointerCaptureUpdate {
        active: false,
        reason: "pointer-lock-error",
    })
}

pub fn plan_browser_cursor_mode_change(mode: PlatformCursorGrabMode) -> BrowserCursorCommandPlan {
    match mode {
        PlatformCursorGrabMode::None => BrowserCursorCommandPlan {
            active: false,
            reason: "command",
            message: "Pointer capture disabled",
            request_pointer_lock: false,
            exit_pointer_lock: true,
        },
        PlatformCursorGrabMode::Confined => BrowserCursorCommandPlan {
            active: true,
            reason: "command-polyfill",
            message: "Pointer confined mode enabled (polyfill)",
            request_pointer_lock: false,
            exit_pointer_lock: false,
        },
        PlatformCursorGrabMode::Locked => BrowserCursorCommandPlan {
            active: false,
            reason: "command-requested",
            message: "Pointer lock requested",
            request_pointer_lock: true,
            exit_pointer_lock: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BrowserCursorCommandPlan, BrowserPointerCaptureUpdate, BrowserPointerMotionInput,
        BrowserSurfaceResizePlan, PlatformCursorGrabMode, PlatformWindowState, browser_now_ns,
        map_browser_pointer_type, normalize_browser_key_text, plan_browser_cursor_mode_change,
        plan_browser_surface_resize, resolve_browser_pointer_position,
        resolve_browser_window_state, resolve_canvas_surface_size, resolve_pointer_lock_change,
        resolve_pointer_lock_error, should_poll_browser_gamepads,
    };
    use glam::vec2;

    #[test]
    fn normalize_browser_key_text_accepts_single_character_only() {
        assert_eq!(normalize_browser_key_text("a", false), Some("a".to_owned()));
        assert_eq!(normalize_browser_key_text("Enter", false), None);
        assert_eq!(normalize_browser_key_text("x", true), None);
    }

    #[test]
    fn resolve_canvas_surface_size_uses_safe_dpr_and_minimum_one() {
        assert_eq!(resolve_canvas_surface_size(0.2, 0.3, 0.0), (1, 1));
        assert_eq!(resolve_canvas_surface_size(10.0, 5.0, 2.0), (20, 10));
    }

    #[test]
    fn resolve_browser_pointer_position_uses_relative_motion_when_capture_is_active() {
        let position = resolve_browser_pointer_position(BrowserPointerMotionInput {
            cursor_grab_mode: PlatformCursorGrabMode::Locked,
            pointer_capture_active: true,
            absolute_position: vec2(10.0, 20.0),
            movement: vec2(4.0, -2.0),
            last_position: Some(vec2(30.0, 40.0)),
            window_size: Some(vec2(200.0, 100.0)),
        });
        assert_eq!(position, vec2(34.0, 38.0));
    }

    #[test]
    fn resolve_browser_pointer_position_falls_back_to_absolute_without_capture() {
        let position = resolve_browser_pointer_position(BrowserPointerMotionInput {
            cursor_grab_mode: PlatformCursorGrabMode::Locked,
            pointer_capture_active: false,
            absolute_position: vec2(10.0, 20.0),
            movement: vec2(4.0, -2.0),
            last_position: Some(vec2(30.0, 40.0)),
            window_size: Some(vec2(200.0, 100.0)),
        });
        assert_eq!(position, vec2(10.0, 20.0));
    }

    #[test]
    fn map_browser_pointer_type_matches_known_inputs() {
        assert_eq!(map_browser_pointer_type("mouse"), 0);
        assert_eq!(map_browser_pointer_type("touch"), 1);
        assert_eq!(map_browser_pointer_type("pen"), 2);
        assert_eq!(map_browser_pointer_type("unknown"), 0);
    }

    #[test]
    fn resolve_browser_window_state_maps_fullscreen_flag() {
        assert_eq!(
            resolve_browser_window_state(false),
            PlatformWindowState::Windowed
        );
        assert_eq!(
            resolve_browser_window_state(true),
            PlatformWindowState::Fullscreen
        );
    }

    #[test]
    fn resolve_pointer_lock_change_only_emits_for_locked_mode() {
        assert_eq!(
            resolve_pointer_lock_change(PlatformCursorGrabMode::None, true),
            None
        );
        assert_eq!(
            resolve_pointer_lock_change(PlatformCursorGrabMode::Locked, true),
            Some(BrowserPointerCaptureUpdate {
                active: true,
                reason: "pointer-lock-change",
            })
        );
    }

    #[test]
    fn resolve_pointer_lock_error_only_emits_for_locked_mode() {
        assert_eq!(
            resolve_pointer_lock_error(PlatformCursorGrabMode::Confined),
            None
        );
        assert_eq!(
            resolve_pointer_lock_error(PlatformCursorGrabMode::Locked),
            Some(BrowserPointerCaptureUpdate {
                active: false,
                reason: "pointer-lock-error",
            })
        );
    }

    #[test]
    fn browser_now_ns_scales_milliseconds() {
        assert_eq!(browser_now_ns(12.5), 12_500_000);
    }

    #[test]
    fn should_poll_browser_gamepads_requires_windows_and_focus() {
        assert!(!should_poll_browser_gamepads(false, true));
        assert!(!should_poll_browser_gamepads(true, false));
        assert!(should_poll_browser_gamepads(true, true));
    }

    #[test]
    fn plan_browser_surface_resize_skips_unchanged_values() {
        assert_eq!(plan_browser_surface_resize(10, 20, 10, 20), None);
        assert_eq!(
            plan_browser_surface_resize(10, 20, 0, 30),
            Some(BrowserSurfaceResizePlan {
                width: 1,
                height: 30,
            })
        );
    }

    #[test]
    fn plan_browser_cursor_mode_change_matches_expected_behavior() {
        assert_eq!(
            plan_browser_cursor_mode_change(PlatformCursorGrabMode::Locked),
            BrowserCursorCommandPlan {
                active: false,
                reason: "command-requested",
                message: "Pointer lock requested",
                request_pointer_lock: true,
                exit_pointer_lock: false,
            }
        );
        assert_eq!(
            plan_browser_cursor_mode_change(PlatformCursorGrabMode::None),
            BrowserCursorCommandPlan {
                active: false,
                reason: "command",
                message: "Pointer capture disabled",
                request_pointer_lock: false,
                exit_pointer_lock: true,
            }
        );
    }
}
