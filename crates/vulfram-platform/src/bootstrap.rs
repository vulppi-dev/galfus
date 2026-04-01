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
mod tests {
    use super::{
        PlatformRenderBootstrapTarget, PlatformRenderSurfaceKind, PlatformSurfaceAlphaMode,
        plan_native_render_bootstrap_target, plan_web_render_bootstrap_target,
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

    #[test]
    fn native_bootstrap_target_maps_transparency_and_latency_preference() {
        let target = plan_native_render_bootstrap_target(3, glam::UVec2::new(320, 240), true);
        assert_eq!(target.surface_kind, PlatformRenderSurfaceKind::NativeWindow);
        assert_eq!(target.alpha_mode, PlatformSurfaceAlphaMode::Transparent);
        assert!(target.prefer_low_latency_present);
    }

    #[test]
    fn web_bootstrap_target_uses_canvas_defaults() {
        let target = plan_web_render_bootstrap_target(5, glam::UVec2::new(640, 360));
        assert_eq!(target.surface_kind, PlatformRenderSurfaceKind::WebCanvas);
        assert_eq!(target.alpha_mode, PlatformSurfaceAlphaMode::Opaque);
        assert!(!target.prefer_low_latency_present);
    }
}
