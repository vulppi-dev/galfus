use super::resolve_target_size;
use crate::core::realm::{AutoLink, RealmId, SurfaceKind, SurfaceState};
use crate::core::state::EngineState;
use crate::core::target::{TargetId, TargetKind, TargetState};
use glam::UVec2;

#[test]
fn resolve_target_size_prefers_target_surface_over_declared_size() {
    let mut engine = EngineState::new();
    let target_id = TargetId(700);
    engine.universal_state.targets.targets.entries.insert(
        target_id,
        TargetState {
            kind: TargetKind::Texture,
            window_id: None,
            size: Some(UVec2::new(300, 200)),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
    );

    let surface_id = engine
        .universal_state
        .composition
        .surfaces
        .alloc(SurfaceState {
            kind: SurfaceKind::Offscreen,
            size: UVec2::new(1280, 720),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
    engine.universal_state.targets.auto_links.insert(
        (RealmId(1).0, target_id),
        AutoLink {
            surface_id,
            connector_id: None,
            present_id: None,
        },
    );

    let size = resolve_target_size(&engine.universal_state, None, None, Some(target_id));
    assert_eq!(size, Some(UVec2::new(1280, 720)));
}

#[test]
fn resolve_target_size_falls_back_to_declared_without_runtime_surface() {
    let mut engine = EngineState::new();
    let target_id = TargetId(701);
    engine.universal_state.targets.targets.entries.insert(
        target_id,
        TargetState {
            kind: TargetKind::Texture,
            window_id: None,
            size: Some(UVec2::new(640, 360)),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
    );

    let size = resolve_target_size(&engine.universal_state, None, None, Some(target_id));
    assert_eq!(size, Some(UVec2::new(640, 360)));
}
