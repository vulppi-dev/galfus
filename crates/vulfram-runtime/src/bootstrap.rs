use vulfram_platform::PlatformRenderBootstrapTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBootstrapDeviceStrategy {
    CreateSharedDevice,
    ReuseSharedDevice,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeRenderBootstrapPlan {
    pub target: PlatformRenderBootstrapTarget,
    pub device_strategy: RenderBootstrapDeviceStrategy,
}

pub fn plan_render_bootstrap(
    has_shared_device: bool,
    target: PlatformRenderBootstrapTarget,
) -> RuntimeRenderBootstrapPlan {
    RuntimeRenderBootstrapPlan {
        target,
        device_strategy: if has_shared_device {
            RenderBootstrapDeviceStrategy::ReuseSharedDevice
        } else {
            RenderBootstrapDeviceStrategy::CreateSharedDevice
        },
    }
}
