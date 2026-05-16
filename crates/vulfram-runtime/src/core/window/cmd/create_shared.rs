use glam::UVec2;

use crate::core::realm::{
    PresentState, RealmId, RealmKind, RealmState, SurfaceKind, SurfaceState,
};
use crate::core::render::RenderState;
use crate::core::render::ensure_runtime_render_defaults;
use crate::core::render::graph::DEFAULT_3D_RENDER_GRAPH_ID;
use crate::core::resources::RenderTarget;
use crate::core::state::EngineState;

pub struct WindowRealmBinding {
    pub realm_id: RealmId,
}

pub struct WindowRenderBootstrapArtifacts {
    pub config: wgpu::SurfaceConfiguration,
    pub render_state: RenderState,
    pub surface_target: Option<RenderTarget>,
}

pub fn register_window_realm(
    engine: &mut EngineState,
    window_id: u32,
    size: UVec2,
) -> WindowRealmBinding {
    ensure_runtime_render_defaults(&mut engine.universal_state);
    let surface_id = engine
        .universal_state
        .composition
        .surfaces
        .alloc(SurfaceState {
            kind: SurfaceKind::Onscreen,
            size,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
    let realm_id = engine.universal_state.composition.realms.alloc(RealmState {
        kind: RealmKind::ThreeD,
        output_surface: Some(surface_id),
        render_graph_id: Some(DEFAULT_3D_RENDER_GRAPH_ID),
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });
    engine
        .universal_state
        .scene
        .realm3d
        .entities
        .entry(realm_id)
        .or_default();
    let _present_id = engine
        .universal_state
        .composition
        .presents
        .alloc(PresentState {
            window_id,
            surface: surface_id,
        });

    WindowRealmBinding { realm_id }
}

pub fn build_window_render_state(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    surface_format: wgpu::TextureFormat,
    target_size: UVec2,
    rgba16f_msaa_supported_mask: u8,
) -> (RenderState, Option<RenderTarget>) {
    let mut render_state = RenderState::new(surface_format);
    render_state.rgba16f_msaa_supported_mask = rgba16f_msaa_supported_mask;
    render_state.init(device, queue, surface_format);
    render_state.on_resize(device, target_size.x, target_size.y);

    let mut surface_target = None;
    crate::core::resources::ensure_render_target(
        device,
        &mut surface_target,
        target_size.x,
        target_size.y,
        wgpu::TextureFormat::Rgba16Float,
    );

    (render_state, surface_target)
}

pub fn build_window_render_bootstrap_artifacts(
    surface: &wgpu::Surface<'static>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    caps: &wgpu::SurfaceCapabilities,
    bootstrap_target: vulfram_platform::PlatformRenderBootstrapTarget,
    rgba16f_msaa_supported_mask: u8,
) -> WindowRenderBootstrapArtifacts {
    let surface_plan = vulfram_render::plan_surface_config(caps, bootstrap_target);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        width: surface_plan.width,
        height: surface_plan.height,
        present_mode: surface_plan.present_mode,
        format: surface_plan.format,
        alpha_mode: surface_plan.alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(device, &config);

    let (render_state, surface_target) = build_window_render_state(
        device,
        queue,
        surface_plan.format,
        bootstrap_target.size,
        rgba16f_msaa_supported_mask,
    );

    WindowRenderBootstrapArtifacts {
        config,
        render_state,
        surface_target,
    }
}
