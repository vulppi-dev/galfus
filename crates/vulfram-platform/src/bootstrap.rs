#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformRenderSurfaceKind {
    NativeWindow,
    WebCanvas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformSurfaceAlphaMode {
    Opaque,
    Transparent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformRenderBootstrapTarget {
    pub window_id: u32,
    pub size: glam::UVec2,
    pub surface_kind: PlatformRenderSurfaceKind,
    pub alpha_mode: PlatformSurfaceAlphaMode,
    pub prefer_low_latency_present: bool,
}

impl PlatformRenderBootstrapTarget {
    pub fn new(
        window_id: u32,
        size: glam::UVec2,
        surface_kind: PlatformRenderSurfaceKind,
        alpha_mode: PlatformSurfaceAlphaMode,
        prefer_low_latency_present: bool,
    ) -> Self {
        Self {
            window_id,
            size: glam::UVec2::new(size.x.max(1), size.y.max(1)),
            surface_kind,
            alpha_mode,
            prefer_low_latency_present,
        }
    }
}

pub fn plan_native_render_bootstrap_target(
    window_id: u32,
    size: glam::UVec2,
    transparent: bool,
) -> PlatformRenderBootstrapTarget {
    PlatformRenderBootstrapTarget::new(
        window_id,
        size,
        PlatformRenderSurfaceKind::NativeWindow,
        if transparent {
            PlatformSurfaceAlphaMode::Transparent
        } else {
            PlatformSurfaceAlphaMode::Opaque
        },
        true,
    )
}

pub fn plan_web_render_bootstrap_target(
    window_id: u32,
    size: glam::UVec2,
) -> PlatformRenderBootstrapTarget {
    PlatformRenderBootstrapTarget::new(
        window_id,
        size,
        PlatformRenderSurfaceKind::WebCanvas,
        PlatformSurfaceAlphaMode::Opaque,
        false,
    )
}

#[cfg(test)]
#[path = "bootstrap_tests.rs"]
mod tests;
