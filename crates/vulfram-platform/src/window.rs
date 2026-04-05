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
#[path = "window_tests.rs"]
mod tests;
