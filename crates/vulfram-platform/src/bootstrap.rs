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

#[cfg(test)]
mod tests {
    use super::{
        PlatformRenderBootstrapTarget, PlatformRenderSurfaceKind, PlatformSurfaceAlphaMode,
    };

    #[test]
    fn platform_bootstrap_target_clamps_surface_size() {
        let target = PlatformRenderBootstrapTarget::new(
            7,
            glam::UVec2::new(0, 9),
            PlatformRenderSurfaceKind::NativeWindow,
            PlatformSurfaceAlphaMode::Opaque,
            true,
        );
        assert_eq!(target.size, glam::UVec2::new(1, 9));
    }
}
