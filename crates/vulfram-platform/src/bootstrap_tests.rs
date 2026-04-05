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
