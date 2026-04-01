use vulfram_platform::{PlatformRenderBootstrapTarget, PlatformSurfaceAlphaMode};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderDeviceFeaturePlan {
    pub required_features: wgpu::Features,
    pub gpu_profiling_supported: bool,
    pub adapter_specific_format_features_supported: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderAdapterBootstrapInfo {
    pub feature_plan: RenderDeviceFeaturePlan,
    pub rgba16f_msaa_supported_mask: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderSurfaceConfigPlan {
    pub format: wgpu::TextureFormat,
    pub present_mode: wgpu::PresentMode,
    pub alpha_mode: wgpu::CompositeAlphaMode,
    pub width: u32,
    pub height: u32,
}

pub fn plan_device_features(adapter_features: wgpu::Features) -> RenderDeviceFeaturePlan {
    let mut required_features = wgpu::Features::empty();
    let gpu_profiling_supported = adapter_features.contains(
        wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS,
    );
    if gpu_profiling_supported {
        required_features |=
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
    }

    if adapter_features.contains(wgpu::Features::POLYGON_MODE_LINE) {
        required_features |= wgpu::Features::POLYGON_MODE_LINE;
    }
    if adapter_features.contains(wgpu::Features::POLYGON_MODE_POINT) {
        required_features |= wgpu::Features::POLYGON_MODE_POINT;
    }

    let adapter_specific_format_features_supported =
        adapter_features.contains(wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES);
    if adapter_specific_format_features_supported {
        required_features |= wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES;
    }

    RenderDeviceFeaturePlan {
        required_features,
        gpu_profiling_supported,
        adapter_specific_format_features_supported,
    }
}

pub fn analyze_adapter(adapter: &wgpu::Adapter) -> RenderAdapterBootstrapInfo {
    let feature_plan = plan_device_features(adapter.features());
    let rgba16f_msaa_supported_mask = resolve_rgba16f_msaa_supported_mask(
        adapter,
        feature_plan.adapter_specific_format_features_supported,
    );
    RenderAdapterBootstrapInfo {
        feature_plan,
        rgba16f_msaa_supported_mask,
    }
}

pub fn build_device_descriptor(
    feature_plan: RenderDeviceFeaturePlan,
) -> wgpu::DeviceDescriptor<'static> {
    wgpu::DeviceDescriptor {
        label: None,
        required_features: feature_plan.required_features,
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        ..Default::default()
    }
}

pub fn build_default_instance_descriptor() -> wgpu::InstanceDescriptor {
    if cfg!(target_arch = "wasm32") {
        wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            backend_options: wgpu::BackendOptions::default(),
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        }
    } else {
        wgpu::InstanceDescriptor {
            backends: if cfg!(target_os = "ios") || cfg!(target_os = "macos") {
                wgpu::Backends::METAL | wgpu::Backends::VULKAN
            } else {
                wgpu::Backends::DX12 | wgpu::Backends::VULKAN
            },
            backend_options: wgpu::BackendOptions::default(),
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        }
    }
}

pub fn create_default_instance() -> wgpu::Instance {
    wgpu::Instance::new(&build_default_instance_descriptor())
}

pub fn plan_surface_config(
    caps: &wgpu::SurfaceCapabilities,
    target: PlatformRenderBootstrapTarget,
) -> RenderSurfaceConfigPlan {
    let format = caps
        .formats
        .iter()
        .copied()
        .find(|format| format.is_srgb())
        .unwrap_or(caps.formats[0]);

    let present_mode = if target.prefer_low_latency_present
        && caps.present_modes.contains(&wgpu::PresentMode::Mailbox)
    {
        wgpu::PresentMode::Mailbox
    } else {
        wgpu::PresentMode::Fifo
    };

    let alpha_mode = match target.alpha_mode {
        PlatformSurfaceAlphaMode::Opaque => wgpu::CompositeAlphaMode::Opaque,
        PlatformSurfaceAlphaMode::Transparent => wgpu::CompositeAlphaMode::PreMultiplied,
    };

    RenderSurfaceConfigPlan {
        format,
        present_mode,
        alpha_mode,
        width: target.size.x.max(1),
        height: target.size.y.max(1),
    }
}

pub fn resolve_rgba16f_msaa_supported_mask(
    adapter: &wgpu::Adapter,
    adapter_specific_enabled: bool,
) -> u8 {
    if !adapter_specific_enabled {
        return core_defaults::msaa_mask_default_safe();
    }

    let flags = adapter
        .get_texture_format_features(wgpu::TextureFormat::Rgba16Float)
        .flags;

    let mut mask = core_defaults::msaa_mask_1();
    if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X2) {
        mask |= core_defaults::msaa_mask_2();
    }
    if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4) {
        mask |= core_defaults::msaa_mask_4();
    }
    if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8) {
        mask |= core_defaults::msaa_mask_8();
    }
    if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X16) {
        mask |= core_defaults::msaa_mask_16();
    }

    if (mask & core_defaults::msaa_mask_4()) == 0 {
        mask |= core_defaults::msaa_mask_4();
    }
    mask
}

mod core_defaults {
    pub const fn msaa_mask_default_safe() -> u8 {
        msaa_mask_1() | msaa_mask_4()
    }

    pub const fn msaa_mask_1() -> u8 {
        1 << 0
    }

    pub const fn msaa_mask_2() -> u8 {
        1 << 1
    }

    pub const fn msaa_mask_4() -> u8 {
        1 << 2
    }

    pub const fn msaa_mask_8() -> u8 {
        1 << 3
    }

    pub const fn msaa_mask_16() -> u8 {
        1 << 4
    }
}

#[cfg(test)]
mod tests {
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
}
