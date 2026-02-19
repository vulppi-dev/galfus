use glam::UVec2;

use crate::core::realm::{
    PresentId, PresentState, RealmId, RealmKind, RealmState, SurfaceId, SurfaceKind, SurfaceState,
};
use crate::core::render::graph::RenderGraphState;
use crate::core::state::EngineState;

pub struct WindowRealmBinding {
    pub realm_id: RealmId,
    pub surface_id: SurfaceId,
    pub present_id: PresentId,
}

pub fn register_window_realm(
    engine: &mut EngineState,
    window_id: u32,
    size: UVec2,
) -> WindowRealmBinding {
    let surface_id = engine.universal_state.surfaces.alloc(SurfaceState {
        kind: SurfaceKind::Onscreen,
        size,
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    });
    let realm_id = engine.universal_state.realms.alloc(RealmState {
        kind: RealmKind::ThreeD,
        host_window_id: Some(window_id),
        output_surface: Some(surface_id),
        render_graph: Some(RenderGraphState::new()),
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });
    let present_id = engine.universal_state.presents.alloc(PresentState {
        window_id,
        surface: surface_id,
    });

    WindowRealmBinding {
        realm_id,
        surface_id,
        present_id,
    }
}

#[cfg(any(not(feature = "wasm"), target_arch = "wasm32"))]
pub fn resolve_rgba16f_msaa_supported_mask(
    adapter: &wgpu::Adapter,
    adapter_specific_enabled: bool,
) -> u8 {
    use crate::core::render::RenderState;

    if !adapter_specific_enabled {
        return RenderState::MSAA_MASK_DEFAULT_SAFE;
    }

    let flags = adapter
        .get_texture_format_features(wgpu::TextureFormat::Rgba16Float)
        .flags;

    let mut mask = RenderState::MSAA_MASK_1;
    if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X2) {
        mask |= RenderState::MSAA_MASK_2;
    }
    if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X4) {
        mask |= RenderState::MSAA_MASK_4;
    }
    if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X8) {
        mask |= RenderState::MSAA_MASK_8;
    }
    if flags.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_X16) {
        mask |= RenderState::MSAA_MASK_16;
    }

    if (mask & RenderState::MSAA_MASK_4) == 0 {
        mask |= RenderState::MSAA_MASK_4;
    }
    mask
}
