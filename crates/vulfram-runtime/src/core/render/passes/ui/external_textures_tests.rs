use super::resolve_external_target_surfaces;
use crate::core::realm::{AutoLink, RealmId, SurfaceId};
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
    let mut auto_links = HashMap::new();
    auto_links.insert(
        (1, target_id),
        AutoLink {
            surface_id: SurfaceId(10),
            connector_id: None,
            present_id: None,
        },
    );
    auto_links.insert(
        (7, target_id),
        AutoLink {
            surface_id: SurfaceId(20),
            connector_id: None,
            present_id: None,
        },
    );

    let resolved = resolve_external_target_surfaces(&auto_links, &targets, RealmId(7));
    assert_eq!(resolved.get(&target_id), Some(&(SurfaceId(20), 7)));
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
    let mut auto_links = HashMap::new();
    auto_links.insert(
        (3, window_target),
        AutoLink {
            surface_id: SurfaceId(11),
            connector_id: None,
            present_id: None,
        },
    );
    auto_links.insert(
        (3, texture_target),
        AutoLink {
            surface_id: SurfaceId(12),
            connector_id: None,
            present_id: None,
        },
    );

    let resolved = resolve_external_target_surfaces(&auto_links, &targets, RealmId(3));
    assert!(!resolved.contains_key(&window_target));
    assert_eq!(resolved.get(&texture_target), Some(&(SurfaceId(12), 3)));
}
