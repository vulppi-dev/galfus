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

pub fn should_process_browser_gamepad_snapshots(has_windows: bool) -> bool {
    has_windows
}

pub fn should_dispatch_browser_action(canvas_active: bool) -> bool {
    canvas_active
}

pub fn should_activate_canvas_from_pointer(
    pointer_inside_canvas: bool,
    canvas_active: bool,
) -> bool {
    pointer_inside_canvas && !canvas_active
}

pub fn should_deactivate_canvas_from_outside_pointer(
    pointer_inside_canvas: bool,
    canvas_active: bool,
) -> bool {
    canvas_active && !pointer_inside_canvas
}

pub fn should_prevent_browser_default_wheel(canvas_active: bool) -> bool {
    canvas_active
}

pub fn should_prevent_browser_default_touch(canvas_active: bool) -> bool {
    canvas_active
}

pub fn should_prevent_browser_default_key(code: &str, canvas_active: bool) -> bool {
    if !canvas_active {
        return false;
    }
    matches!(
        code,
        "ArrowUp"
            | "ArrowDown"
            | "ArrowLeft"
            | "ArrowRight"
            | "PageUp"
            | "PageDown"
            | "Home"
            | "End"
            | "Space"
    )
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

pub fn resolve_browser_canvas_surface_position(
    local_x: f64,
    local_y: f64,
    rect_width: f64,
    rect_height: f64,
    surface_width: u32,
    surface_height: u32,
) -> Vec2 {
    let safe_rect_width = rect_width.max(1.0) as f32;
    let safe_rect_height = rect_height.max(1.0) as f32;
    let scale_x = surface_width.max(1) as f32 / safe_rect_width;
    let scale_y = surface_height.max(1) as f32 / safe_rect_height;
    Vec2::new(local_x as f32 * scale_x, local_y as f32 * scale_y)
}

pub fn resolve_browser_canvas_surface_delta(
    delta_x: f64,
    delta_y: f64,
    rect_width: f64,
    rect_height: f64,
    surface_width: u32,
    surface_height: u32,
) -> Vec2 {
    let safe_rect_width = rect_width.max(1.0) as f32;
    let safe_rect_height = rect_height.max(1.0) as f32;
    let scale_x = surface_width.max(1) as f32 / safe_rect_width;
    let scale_y = surface_height.max(1) as f32 / safe_rect_height;
    Vec2::new(delta_x as f32 * scale_x, delta_y as f32 * scale_y)
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
#[path = "browser_tests.rs"]
mod tests;
