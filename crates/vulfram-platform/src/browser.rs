use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformCursorGrabMode {
    None,
    Confined,
    Locked,
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

pub fn map_browser_pointer_type(pointer_type: &str) -> u32 {
    match pointer_type {
        "mouse" => 0,
        "touch" => 1,
        "pen" => 2,
        _ => 0,
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

#[cfg(test)]
mod tests {
    use super::{
        BrowserPointerMotionInput, PlatformCursorGrabMode, map_browser_pointer_type,
        normalize_browser_key_text, resolve_browser_pointer_position, resolve_canvas_surface_size,
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
}
