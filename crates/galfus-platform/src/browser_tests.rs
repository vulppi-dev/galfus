use super::{
    BrowserCursorCommandPlan, BrowserPointerCaptureUpdate, BrowserPointerMotionInput,
    BrowserSurfaceResizePlan, PlatformCursorGrabMode, PlatformWindowState, browser_now_ns,
    map_browser_pointer_type, normalize_browser_key_text, plan_browser_cursor_mode_change,
    plan_browser_surface_resize, resolve_browser_canvas_surface_delta,
    resolve_browser_canvas_surface_position, resolve_browser_pointer_position,
    resolve_browser_window_state, resolve_canvas_surface_size, resolve_pointer_lock_change,
    resolve_pointer_lock_error, should_activate_canvas_from_pointer,
    should_deactivate_canvas_from_outside_pointer, should_dispatch_browser_action,
    should_poll_browser_gamepads, should_prevent_browser_default_key,
    should_prevent_browser_default_touch, should_prevent_browser_default_wheel,
    should_process_browser_gamepad_snapshots,
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
fn resolve_browser_canvas_surface_position_scales_css_to_surface_space() {
    let position = resolve_browser_canvas_surface_position(50.0, 25.0, 100.0, 50.0, 1000, 500);
    assert_eq!(position, vec2(500.0, 250.0));
}

#[test]
fn resolve_browser_canvas_surface_delta_scales_relative_motion_to_surface_space() {
    let delta = resolve_browser_canvas_surface_delta(10.0, -5.0, 100.0, 50.0, 1000, 500);
    assert_eq!(delta, vec2(100.0, -50.0));
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
fn should_process_browser_gamepad_snapshots_requires_windows_only() {
    assert!(!should_process_browser_gamepad_snapshots(false));
    assert!(should_process_browser_gamepad_snapshots(true));
}

#[test]
fn browser_action_dispatch_requires_active_canvas() {
    assert!(!should_dispatch_browser_action(false));
    assert!(should_dispatch_browser_action(true));
}

#[test]
fn canvas_activation_and_deactivation_policies_match_pointer_scope() {
    assert!(should_activate_canvas_from_pointer(true, false));
    assert!(!should_activate_canvas_from_pointer(true, true));
    assert!(should_deactivate_canvas_from_outside_pointer(false, true));
    assert!(!should_deactivate_canvas_from_outside_pointer(true, true));
}

#[test]
fn browser_default_prevention_only_applies_when_canvas_is_active() {
    assert!(should_prevent_browser_default_wheel(true));
    assert!(!should_prevent_browser_default_wheel(false));
    assert!(should_prevent_browser_default_touch(true));
    assert!(!should_prevent_browser_default_touch(false));
    assert!(should_prevent_browser_default_key("PageDown", true));
    assert!(should_prevent_browser_default_key("ArrowUp", true));
    assert!(!should_prevent_browser_default_key("Escape", true));
    assert!(!should_prevent_browser_default_key("PageDown", false));
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
