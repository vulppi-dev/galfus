#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformFullscreenMode {
    Exclusive,
    Borderless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformWindowLifecycleState {
    Windowed,
    Fullscreen,
    WindowedFullscreen,
    Maximized,
    Minimized,
}

pub fn resolve_platform_window_state(
    is_minimized: bool,
    is_maximized: bool,
    fullscreen: Option<PlatformFullscreenMode>,
) -> PlatformWindowLifecycleState {
    if is_minimized {
        PlatformWindowLifecycleState::Minimized
    } else if is_maximized {
        PlatformWindowLifecycleState::Maximized
    } else if let Some(fullscreen) = fullscreen {
        match fullscreen {
            PlatformFullscreenMode::Exclusive => PlatformWindowLifecycleState::Fullscreen,
            PlatformFullscreenMode::Borderless => PlatformWindowLifecycleState::WindowedFullscreen,
        }
    } else {
        PlatformWindowLifecycleState::Windowed
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PlatformFullscreenMode, PlatformWindowLifecycleState, resolve_platform_window_state,
    };

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
}
