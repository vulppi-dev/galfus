use std::collections::{HashMap, HashSet};

use super::*;
use galfus_types::{ConnectorId, SurfaceId};

#[test]
fn realm_table_allocates_monotonic_ids() {
    let mut table = RealmTable::default();
    let first = table.alloc(RealmState {
        kind: RealmKind::ThreeD,
        output_surface: None,
        render_graph_id: None,
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });
    let second = table.alloc(RealmState {
        kind: RealmKind::TwoD,
        output_surface: Some(SurfaceId(3)),
        render_graph_id: Some(9),
        importance: 2,
        cache_policy: 1,
        last_render_frame: 7,
    });

    assert_eq!(first, RealmId(0));
    assert_eq!(second, RealmId(1));
}

#[test]
fn present_table_remove_by_window_prunes_matching_entries() {
    let mut table = PresentTable::default();
    let keep = table.alloc(PresentState {
        window_id: 1,
        surface: SurfaceId(10),
    });
    let _drop = table.alloc(PresentState {
        window_id: 2,
        surface: SurfaceId(20),
    });

    table.remove_by_window(2);

    assert!(table.entries.contains_key(&keep));
    assert_eq!(table.entries.len(), 1);
}

#[test]
fn planner_orders_linear_dependency() {
    let realms = HashMap::from([
        (RealmId(0), Some(SurfaceId(10))),
        (RealmId(1), Some(SurfaceId(11))),
    ]);
    let presents = vec![(1, SurfaceId(11))];
    let connectors = vec![(ConnectorId(2), SurfaceId(10), RealmId(1))];

    let plan = RealmGraphPlanner.build_plan(&realms, &presents, &connectors);
    assert_eq!(plan.order, vec![RealmId(0), RealmId(1)]);
    assert!(plan.cut_edges.is_empty());
}

#[test]
fn planner_cuts_cycles_deterministically() {
    let realms = HashMap::from([
        (RealmId(0), Some(SurfaceId(10))),
        (RealmId(1), Some(SurfaceId(11))),
    ]);
    let presents = Vec::new();
    let connectors = vec![
        (ConnectorId(2), SurfaceId(10), RealmId(1)),
        (ConnectorId(3), SurfaceId(11), RealmId(0)),
    ];

    let plan = RealmGraphPlanner.build_plan(&realms, &presents, &connectors);
    assert_eq!(plan.order, vec![RealmId(0), RealmId(1)]);
    assert_eq!(plan.cut_edges.len(), 2);
}

#[test]
fn frame_report_serializes_realm_order_edges_and_cache() {
    let plan = RealmGraphPlan {
        order: vec![RealmId(3), RealmId(4)],
        cut_edges: vec![RealmGraphEdge {
            from: RealmId(3),
            to: RealmId(4),
            connector_id: Some(ConnectorId(9)),
        }],
    };
    let mut cache = SurfaceCache::default();
    cache.last_good.insert(ConnectorId(2), SurfaceId(5));
    cache.fallback.insert(ConnectorId(3), SurfaceId(6));

    let report = FrameReport::from_plan(&plan, &cache);
    assert_eq!(report.order, vec![3, 4]);
    assert_eq!(report.cut_edges.len(), 1);
    assert_eq!(report.cache_last_good.len(), 1);
    assert_eq!(report.cache_fallback.len(), 1);
}

#[test]
fn dimension_value_percent_uses_reference_axis() {
    let value = DimensionValue::Percent(25.0);
    assert_eq!(value.resolve(400.0, 8.0), 100.0);
}

#[test]
fn target_layer_layout_defaults_to_full_percent_size() {
    let layout = TargetLayerLayout::default();
    assert_eq!(layout.width, DimensionValue::Percent(100.0));
    assert_eq!(layout.height, DimensionValue::Percent(100.0));
    assert!(layout.enabled);
    assert_eq!(layout.opacity, 1.0);
}

#[test]
fn target_graph_planner_uses_targets_without_derived_edges() {
    let targets = HashMap::from([
        (TargetId(1), (TargetKind::Window, Some(7))),
        (TargetId(2), (TargetKind::Texture, None)),
    ]);
    let layers = HashMap::from([
        (
            (3, TargetId(2)),
            TargetLayerState {
                realm_id: 3,
                target_id: TargetId(2),
                layout: TargetLayerLayout::default(),
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        ),
        (
            (3, TargetId(1)),
            TargetLayerState {
                realm_id: 3,
                target_id: TargetId(1),
                layout: TargetLayerLayout::default(),
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        ),
    ]);
    let realms = HashSet::from([RealmId(3)]);

    let plan = TargetGraphPlanner.build_plan(&targets, &[], &layers, &realms);
    assert!(plan.edges.is_empty());
    assert_eq!(plan.order, vec![TargetId(1), TargetId(2)]);
}

#[test]
fn target_graph_planner_orders_targets_from_dependencies() {
    let targets = HashMap::from([
        (TargetId(1), (TargetKind::Texture, None)),
        (TargetId(2), (TargetKind::Texture, None)),
        (TargetId(3), (TargetKind::Window, Some(7))),
    ]);
    let layers = HashMap::new();
    let realms = HashSet::new();
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

    let plan = TargetGraphPlanner.build_plan(&targets, &dependencies, &layers, &realms);
    assert_eq!(plan.order, vec![TargetId(1), TargetId(2), TargetId(3)]);
    assert!(plan.cut_edges.is_empty());
}

#[test]
fn target_graph_planner_cuts_cycles_deterministically() {
    let targets = HashMap::from([
        (TargetId(10), (TargetKind::Texture, None)),
        (TargetId(20), (TargetKind::Texture, None)),
    ]);
    let dependencies = vec![
        TargetEdge {
            parent: TargetId(10),
            child: TargetId(20),
        },
        TargetEdge {
            parent: TargetId(20),
            child: TargetId(10),
        },
    ];

    let plan =
        TargetGraphPlanner.build_plan(&targets, &dependencies, &HashMap::new(), &HashSet::new());
    assert_eq!(plan.order, vec![TargetId(10), TargetId(20)]);
    assert_eq!(plan.cut_edges.len(), 2);
}
