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
        flags: 0,
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
