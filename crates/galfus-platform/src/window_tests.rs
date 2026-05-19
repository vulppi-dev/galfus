use super::{PlatformFullscreenMode, PlatformWindowLifecycleState, resolve_platform_window_state};

#[test]
fn resolve_platform_window_state_prioritizes_minimized_and_maximized() {
    assert_eq!(
        resolve_platform_window_state(true, true, Some(PlatformFullscreenMode::Exclusive)),
        PlatformWindowLifecycleState::Minimized
    );
    assert_eq!(
        resolve_platform_window_state(false, true, Some(PlatformFullscreenMode::Exclusive)),
        PlatformWindowLifecycleState::Maximized
    );
}

#[test]
fn resolve_platform_window_state_maps_fullscreen_modes() {
    assert_eq!(
        resolve_platform_window_state(false, false, Some(PlatformFullscreenMode::Exclusive)),
        PlatformWindowLifecycleState::Fullscreen
    );
    assert_eq!(
        resolve_platform_window_state(false, false, Some(PlatformFullscreenMode::Borderless)),
        PlatformWindowLifecycleState::WindowedFullscreen
    );
    assert_eq!(
        resolve_platform_window_state(false, false, None),
        PlatformWindowLifecycleState::Windowed
    );
}
