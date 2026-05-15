use std::collections::HashMap;

use crate::core::realm::{RealmKind, RealmState, RealmTable};
use crate::core::target::{
    TargetGraphCache, TargetGraphPlanner, TargetId, TargetKind, TargetLayerLayout,
    TargetLayerState, TargetState, TargetEdge,
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

fn realm_state() -> RealmState {
    RealmState {
        kind: RealmKind::TwoD,
        output_surface: None,
        render_graph_id: None,
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    }
}

#[test]
fn planner_keeps_targets_without_derived_realm_edges() {
    let mut targets = HashMap::new();
    targets.insert(TargetId(10), window_target(10));
    targets.insert(TargetId(5), window_target(5));
    targets.insert(TargetId(100), realm_target(TargetKind::Texture));

    let mut realms = RealmTable::default();
    let _r2 = realms.alloc(realm_state());
    let _r1 = realms.alloc(realm_state());

    let mut layers = HashMap::new();
    layers.insert((0, TargetId(10)), layer_state(0, TargetId(10)));
    layers.insert((1, TargetId(5)), layer_state(1, TargetId(5)));
    layers.insert((0, TargetId(100)), layer_state(0, TargetId(100)));
    layers.insert((1, TargetId(100)), layer_state(1, TargetId(100)));

    let plan = TargetGraphPlanner.build_plan(&targets, &[], &layers, &realms);
    assert!(plan.edges.is_empty());
    assert_eq!(plan.order, vec![TargetId(5), TargetId(10), TargetId(100)]);
}

#[test]
fn cache_update_reports_changes_and_skips_when_unchanged() {
    let mut targets = HashMap::new();
    targets.insert(TargetId(1), window_target(1));

    let mut realms = RealmTable::default();
    let _realm = realms.alloc(realm_state());

    let layers = HashMap::new();
    let mut cache = TargetGraphCache::default();

    let first = cache.update(&targets, &[], &layers, &realms);
    assert!(first.is_some());
    assert!(cache.last_plan.order.contains(&TargetId(1)));

    let second = cache.update(&targets, &[], &layers, &realms);
    assert!(second.is_none());

    targets.insert(TargetId(2), realm_target(TargetKind::Texture));
    let third = cache.update(&targets, &[], &layers, &realms);
    assert!(third.is_some());
    assert!(
        third
            .expect("diff should exist after change")
            .added_targets
            .contains(&TargetId(2))
    );
}

#[test]
fn planner_orders_targets_with_explicit_dependencies() {
    let targets = HashMap::from([
        (TargetId(1), realm_target(TargetKind::Texture)),
        (TargetId(2), realm_target(TargetKind::Texture)),
        (TargetId(3), window_target(3)),
    ]);
    let dependencies = vec![
        TargetEdge {
            parent: TargetId(1),
            child: TargetId(2),
        },
        TargetEdge {
            parent: TargetId(2),
            child: TargetId(3),
        },
    ];
    let plan = TargetGraphPlanner.build_plan(
        &targets,
        &dependencies,
        &HashMap::new(),
        &RealmTable::default(),
    );
    assert_eq!(plan.order, vec![TargetId(1), TargetId(2), TargetId(3)]);
    assert!(plan.cut_edges.is_empty());
}

#[test]
fn cache_recomputes_when_dependency_edges_change() {
    let targets = HashMap::from([
        (TargetId(1), realm_target(TargetKind::Texture)),
        (TargetId(2), window_target(2)),
    ]);
    let realms = RealmTable::default();
    let layers = HashMap::new();
    let mut cache = TargetGraphCache::default();

    let first = cache.update(
        &targets,
        &[TargetEdge {
            parent: TargetId(1),
            child: TargetId(2),
        }],
        &layers,
        &realms,
    );
    assert!(first.is_some());
    assert_eq!(cache.last_plan.order, vec![TargetId(1), TargetId(2)]);

    let second = cache.update(
        &targets,
        &[TargetEdge {
            parent: TargetId(2),
            child: TargetId(1),
        }],
        &layers,
        &realms,
    );
    assert!(second.is_some());
    assert_eq!(cache.last_plan.order, vec![TargetId(2), TargetId(1)]);
}
