use std::collections::HashMap;

use crate::core::realm::{RealmKind, RealmState, RealmTable};
use crate::core::target::{
    TargetGraphCache, TargetGraphPlanner, TargetId, TargetKind, TargetLayerLayout,
    TargetLayerState, TargetState,
};

fn window_target(window_id: u32) -> TargetState {
    TargetState {
        kind: TargetKind::Window,
        window_id: Some(window_id),
        size: None,
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    }
}

fn realm_target(kind: TargetKind) -> TargetState {
    TargetState {
        kind,
        window_id: None,
        size: None,
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    }
}

fn layer_state(realm_id: u32, target_id: TargetId) -> TargetLayerState {
    TargetLayerState {
        realm_id,
        target_id,
        layout: TargetLayerLayout::default(),
        camera_id: None,
        environment_id: None,
    }
}

fn realm_state(host_window_id: Option<u32>) -> RealmState {
    RealmState {
        kind: RealmKind::TwoD,
        host_window_id,
        output_surface: None,
        render_graph: None,
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    }
}

#[test]
fn planner_links_realm_target_to_smallest_window_fallback() {
    let mut targets = HashMap::new();
    targets.insert(TargetId(10), window_target(10));
    targets.insert(TargetId(5), window_target(5));
    targets.insert(TargetId(100), realm_target(TargetKind::RealmPlane));

    let mut realms = RealmTable::default();
    let _r2 = realms.alloc(realm_state(Some(10)));
    let _r1 = realms.alloc(realm_state(Some(5)));

    let mut layers = HashMap::new();
    layers.insert((0, TargetId(100)), layer_state(0, TargetId(100)));
    layers.insert((1, TargetId(100)), layer_state(1, TargetId(100)));

    let plan = TargetGraphPlanner.build_plan(&targets, &layers, &realms);
    assert_eq!(
        plan.edges,
        vec![crate::core::target::TargetEdge {
            parent: TargetId(5),
            child: TargetId(100)
        }]
    );
    assert_eq!(plan.order, vec![TargetId(5), TargetId(10), TargetId(100)]);
}

#[test]
fn cache_update_reports_changes_and_skips_when_unchanged() {
    let mut targets = HashMap::new();
    targets.insert(TargetId(1), window_target(1));

    let mut realms = RealmTable::default();
    let _realm = realms.alloc(realm_state(Some(1)));

    let layers = HashMap::new();
    let mut cache = TargetGraphCache::default();

    let first = cache.update(&targets, &layers, &realms);
    assert!(first.is_some());
    assert!(cache.last_plan.order.contains(&TargetId(1)));

    let second = cache.update(&targets, &layers, &realms);
    assert!(second.is_none());

    targets.insert(TargetId(2), realm_target(TargetKind::Texture));
    let third = cache.update(&targets, &layers, &realms);
    assert!(third.is_some());
    assert!(
        third
            .expect("diff should exist after change")
            .added_targets
            .contains(&TargetId(2))
    );
}
