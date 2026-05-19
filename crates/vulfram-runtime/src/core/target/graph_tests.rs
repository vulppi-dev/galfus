use std::collections::HashMap;

use crate::core::realm::{RealmKind, RealmState, RealmTable};
use crate::core::target::{
    TargetEdge, TargetGraphCache, TargetGraphPlanner, TargetId, TargetKind, TargetLayerLayout,
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
        enabled_camera_ids: Vec::new(),
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

#[test]
fn collect_render_invocations_uses_target_order_and_layout_size() {
    let targets = HashMap::from([
        (
            TargetId(1),
            TargetState {
                kind: TargetKind::Texture,
                window_id: None,
                size: Some(glam::UVec2::new(512, 512)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        ),
        (
            TargetId(2),
            TargetState {
                kind: TargetKind::Window,
                window_id: Some(9),
                size: None,
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        ),
    ]);
    let layers = HashMap::from([
        (
            (7, TargetId(1)),
            TargetLayerState {
                realm_id: 7,
                target_id: TargetId(1),
                layout: TargetLayerLayout::default(),
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        ),
        (
            (8, TargetId(2)),
            TargetLayerState {
                realm_id: 8,
                target_id: TargetId(2),
                layout: TargetLayerLayout::default(),
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        ),
    ]);
    let window_sizes = HashMap::from([(9, glam::UVec2::new(1920, 1080))]);
    let invocations = crate::core::target::collect_render_invocations(
        &[TargetId(1), TargetId(2)],
        &targets,
        &layers,
        &window_sizes,
        42,
    );

    assert_eq!(invocations.len(), 2);
    assert_eq!(invocations[0].target_id, TargetId(1));
    assert_eq!(invocations[0].render_size_px, glam::UVec2::new(512, 512));
    assert_eq!(invocations[1].target_id, TargetId(2));
    assert_eq!(invocations[1].render_size_px, glam::UVec2::new(1920, 1080));
    assert_eq!(invocations[0].frame_id, 42);
}

#[test]
fn collect_render_invocations_supports_same_realm_multiple_target_sizes() {
    let targets = HashMap::from([
        (
            TargetId(11),
            TargetState {
                kind: TargetKind::Window,
                window_id: Some(1),
                size: None,
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        ),
        (
            TargetId(12),
            TargetState {
                kind: TargetKind::Texture,
                window_id: None,
                size: Some(glam::UVec2::new(512, 512)),
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        ),
    ]);
    let layers = HashMap::from([
        (
            (7, TargetId(11)),
            TargetLayerState {
                realm_id: 7,
                target_id: TargetId(11),
                layout: TargetLayerLayout::default(),
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        ),
        (
            (7, TargetId(12)),
            TargetLayerState {
                realm_id: 7,
                target_id: TargetId(12),
                layout: TargetLayerLayout::default(),
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        ),
    ]);
    let window_sizes = HashMap::from([(1, glam::UVec2::new(1920, 1080))]);
    let invocations = crate::core::target::collect_render_invocations(
        &[TargetId(11), TargetId(12)],
        &targets,
        &layers,
        &window_sizes,
        9,
    );

    assert_eq!(invocations.len(), 2);
    assert_eq!(invocations[0].realm_id, 7);
    assert_eq!(invocations[1].realm_id, 7);
    assert_eq!(invocations[0].render_size_px, glam::UVec2::new(1920, 1080));
    assert_eq!(invocations[1].render_size_px, glam::UVec2::new(512, 512));
}

#[test]
fn collect_render_invocations_filters_disabled_and_sorts_by_layer_order() {
    let targets = HashMap::from([(
        TargetId(21),
        TargetState {
            kind: TargetKind::Texture,
            window_id: None,
            size: Some(glam::UVec2::new(800, 600)),
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
    )]);
    let layers = HashMap::from([
        (
            (1, TargetId(21)),
            TargetLayerState {
                realm_id: 1,
                target_id: TargetId(21),
                layout: TargetLayerLayout {
                    z_index: 20,
                    ..TargetLayerLayout::default()
                },
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        ),
        (
            (2, TargetId(21)),
            TargetLayerState {
                realm_id: 2,
                target_id: TargetId(21),
                layout: TargetLayerLayout {
                    enabled: false,
                    z_index: 10,
                    ..TargetLayerLayout::default()
                },
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        ),
        (
            (3, TargetId(21)),
            TargetLayerState {
                realm_id: 3,
                target_id: TargetId(21),
                layout: TargetLayerLayout {
                    opacity: 0.0,
                    z_index: 0,
                    ..TargetLayerLayout::default()
                },
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        ),
        (
            (4, TargetId(21)),
            TargetLayerState {
                realm_id: 4,
                target_id: TargetId(21),
                layout: TargetLayerLayout {
                    z_index: 5,
                    ..TargetLayerLayout::default()
                },
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        ),
    ]);

    let invocations = crate::core::target::collect_render_invocations(
        &[TargetId(21)],
        &targets,
        &layers,
        &HashMap::new(),
        1,
    );

    assert_eq!(invocations.len(), 2);
    assert_eq!(invocations[0].realm_id, 4);
    assert_eq!(invocations[1].realm_id, 1);
}
