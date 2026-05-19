use crate::core::realm::{AutoLink, RealmId, SurfaceId, SurfaceKind, SurfaceState};
use crate::core::state::EngineState;
use crate::core::target::{TargetId, TargetKind, TargetState};

pub fn test_engine() -> EngineState {
    EngineState::new()
}

pub fn insert_texture_target(engine: &mut EngineState, target_id: TargetId, size: glam::UVec2) {
    engine.universal_state.targets.targets.entries.insert(
        target_id,
        TargetState {
            kind: TargetKind::Texture,
            window_id: None,
            size: Some(size),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
    );
}

pub fn alloc_offscreen_surface(engine: &mut EngineState, size: glam::UVec2) -> SurfaceId {
    engine
        .universal_state
        .composition
        .surfaces
        .alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        })
}

pub fn link_target_surface(
    engine: &mut EngineState,
    realm_id: RealmId,
    target_id: TargetId,
    surface_id: SurfaceId,
) {
    engine.universal_state.targets.auto_links.insert(
        (realm_id.0, target_id),
        AutoLink {
            surface_id,
            connector_id: None,
            present_id: None,
        },
    );
}
