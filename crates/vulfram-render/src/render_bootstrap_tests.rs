use super::{
    build_default_instance_descriptor, build_device_descriptor, plan_device_features,
    plan_surface_config,
};
use vulfram_platform::{
    PlatformRenderBootstrapTarget, PlatformRenderSurfaceKind, PlatformSurfaceAlphaMode,
};

#[test]
fn plans_device_features_from_adapter_capabilities() {
    let plan = plan_device_features(
        wgpu::Features::TIMESTAMP_QUERY
            | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS
            | wgpu::Features::POLYGON_MODE_LINE,
    );
    assert!(plan.gpu_profiling_supported);
    assert!(
        plan.required_features
            .contains(wgpu::Features::POLYGON_MODE_LINE)
    );
    assert!(
        plan.required_features
            .contains(wgpu::Features::TIMESTAMP_QUERY)
    );
}

#[test]
fn plans_surface_config_from_caps_and_target() {
    let caps = wgpu::SurfaceCapabilities {
        formats: vec![
            wgpu::TextureFormat::Bgra8Unorm,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        ],
        present_modes: vec![wgpu::PresentMode::Fifo, wgpu::PresentMode::Mailbox],
        alpha_modes: vec![wgpu::CompositeAlphaMode::Opaque],
        usages: wgpu::TextureUsages::RENDER_ATTACHMENT,
    };
    let target = PlatformRenderBootstrapTarget::new(
        1,
        glam::UVec2::new(1280, 720),
        PlatformRenderSurfaceKind::NativeWindow,
        PlatformSurfaceAlphaMode::Opaque,
        true,
    );

    let plan = plan_surface_config(&caps, target);
    assert_eq!(plan.format, wgpu::TextureFormat::Rgba8UnormSrgb);
    assert_eq!(plan.present_mode, wgpu::PresentMode::Mailbox);
    assert_eq!(plan.alpha_mode, wgpu::CompositeAlphaMode::Opaque);
}

#[test]
fn builds_device_descriptor_from_feature_plan() {
    let feature_plan = plan_device_features(wgpu::Features::POLYGON_MODE_LINE);
    let descriptor = build_device_descriptor(feature_plan);
    assert!(
        descriptor
            .required_features
            .contains(wgpu::Features::POLYGON_MODE_LINE)
    );
}

#[test]
fn default_instance_descriptor_uses_empty_flags() {
    let descriptor = build_default_instance_descriptor();
    assert!(descriptor.flags.is_empty());
}
