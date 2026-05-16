use super::resolve_external_target_surfaces;
use crate::core::realm::SurfaceId;
use crate::core::target::{TargetId, TargetKind, TargetState, TargetTable};
use std::collections::HashMap;

fn target(kind: TargetKind) -> TargetState {
    TargetState {
        kind,
        window_id: None,
        size: None,
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    }
}

#[test]
fn prefers_link_from_current_realm_for_same_target() {
    let target_id = TargetId(42);
    let mut targets = TargetTable::default();
    targets
        .entries
        .insert(target_id, target(TargetKind::Texture));
    let target_surface_map = HashMap::from([(target_id, SurfaceId(20))]);

    let resolved = resolve_external_target_surfaces(&targets, &target_surface_map);
    assert_eq!(resolved.get(&target_id), Some(&SurfaceId(20)));
}

#[test]
fn ignores_non_external_target_kinds() {
    let mut targets = TargetTable::default();
    let window_target = TargetId(1);
    let texture_target = TargetId(2);
    targets
        .entries
        .insert(window_target, target(TargetKind::Window));
    targets
        .entries
        .insert(texture_target, target(TargetKind::Texture));
    let target_surface_map = HashMap::from([
        (window_target, SurfaceId(11)),
        (texture_target, SurfaceId(12)),
    ]);

    let resolved = resolve_external_target_surfaces(&targets, &target_surface_map);
    assert!(!resolved.contains_key(&window_target));
    assert_eq!(resolved.get(&texture_target), Some(&SurfaceId(12)));
}
