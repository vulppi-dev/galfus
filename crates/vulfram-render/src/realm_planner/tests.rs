use super::{
    AutoGraphExistingLink, AutoGraphLinkPlan, AutoGraphLinkSyncOp, AutoGraphSurfaceSyncOp,
    ComposeBlendMode, ComposeConnectorCandidate, ComposeOverlayPlan, ComposeOverlayPlanEntry,
    EnvironmentLayerBinding, ExternalTextureRefreshPlan, ExternalTextureSource,
    RealmEnvironmentBindingPlan, ResolvedSurfaceTarget, SurfaceTargetRequest,
    TargetSizeUpdatePlanEntry, TargetSizeUpdateRequest, build_soft_cut_diagnostic,
    build_target_surface_map, collect_connectors_by_realm, collect_cut_connectors,
    collect_window_camera_target_sizes, map_realms_to_windows, plan_auto_graph_layer_sync,
    plan_compose_overlays, plan_external_texture_refresh, plan_realm_environment_bindings,
    plan_surface_targets, plan_target_size_updates, resolve_connector_surface,
    resolve_realm_surface, should_render_realm, update_present_size_cache, update_surface_cache,
};
use std::collections::{HashMap, HashSet};
use vulfram_realm_core::{
    AutoLink, ConnectorId, DimensionValue, FrameCutEdge, RealmGraphEdge, RealmGraphPlan, RealmId,
    RealmState, SurfaceCache, SurfaceId, TargetId, TargetKind, TargetLayerLayout, TargetLayerState,
};

#[test]
fn collects_cut_connectors_from_plan() {
    let plan = RealmGraphPlan {
        order: vec![RealmId(1), RealmId(2)],
        cut_edges: vec![
            RealmGraphEdge {
                from: RealmId(1),
                to: RealmId(2),
                connector_id: Some(ConnectorId(9)),
            },
            RealmGraphEdge {
                from: RealmId(2),
                to: RealmId(1),
                connector_id: None,
            },
        ],
    };

    let cut = collect_cut_connectors(&plan);
    assert!(cut.contains(&ConnectorId(9)));
    assert_eq!(cut.len(), 1);
}

#[test]
fn auto_graph_layer_sync_creates_link_when_missing() {
    let host_realm_index = HashMap::from([(9, RealmId(3))]);

    let plan = plan_auto_graph_layer_sync(
        TargetKind::Window,
        Some(9),
        RealmId(3),
        vulfram_realm_core::RealmKind::ThreeD,
        &host_realm_index,
        None,
        false,
        true,
        None,
    );

    assert_eq!(plan.surface_op, AutoGraphSurfaceSyncOp::Allocate);
    assert_eq!(plan.link_op, AutoGraphLinkSyncOp::Create);
    assert_eq!(
        plan.desired_link,
        AutoGraphLinkPlan::Present { window_id: 9 }
    );
}

#[test]
fn auto_graph_layer_sync_rebuilds_when_link_shape_changes() {
    let host_realm_index = HashMap::from([(9, RealmId(3))]);

    let plan = plan_auto_graph_layer_sync(
        TargetKind::Window,
        Some(9),
        RealmId(7),
        vulfram_realm_core::RealmKind::ThreeD,
        &host_realm_index,
        Some(SurfaceId(44)),
        true,
        true,
        Some(AutoGraphExistingLink {
            surface_id: SurfaceId(44),
            has_connector: false,
            has_present: true,
        }),
    );

    assert_eq!(plan.surface_op, AutoGraphSurfaceSyncOp::Keep);
    assert_eq!(plan.link_op, AutoGraphLinkSyncOp::Rebuild);
    assert!(matches!(
        plan.desired_link,
        AutoGraphLinkPlan::Connector { .. }
    ));
}

#[test]
fn auto_graph_layer_sync_updates_connector_layout_when_shape_is_stable() {
    let host_realm_index = HashMap::from([(9, RealmId(3))]);

    let plan = plan_auto_graph_layer_sync(
        TargetKind::WidgetRealmViewport,
        Some(9),
        RealmId(7),
        vulfram_realm_core::RealmKind::ThreeD,
        &host_realm_index,
        Some(SurfaceId(44)),
        true,
        true,
        Some(AutoGraphExistingLink {
            surface_id: SurfaceId(44),
            has_connector: true,
            has_present: false,
        }),
    );

    assert_eq!(plan.surface_op, AutoGraphSurfaceSyncOp::Keep);
    assert_eq!(plan.link_op, AutoGraphLinkSyncOp::UpdateConnectorLayout);
}

#[test]
fn updates_surface_cache_without_overwriting_fallback() {
    let mut cache = SurfaceCache::default();
    cache.fallback.insert(ConnectorId(4), SurfaceId(99));

    update_surface_cache(
        &mut cache,
        &[
            (ConnectorId(4), SurfaceId(10)),
            (ConnectorId(7), SurfaceId(20)),
        ],
    );

    assert_eq!(cache.last_good.get(&ConnectorId(4)), Some(&SurfaceId(10)));
    assert_eq!(cache.fallback.get(&ConnectorId(4)), Some(&SurfaceId(99)));
    assert_eq!(cache.fallback.get(&ConnectorId(7)), Some(&SurfaceId(20)));
}

#[test]
fn maps_realms_to_smallest_window_and_present_fallback() {
    let existing_realms = HashSet::from([RealmId(1), RealmId(2)]);
    let realm_output_surfaces = HashMap::from([
        (RealmId(1), Some(SurfaceId(50))),
        (RealmId(2), Some(SurfaceId(60))),
    ]);

    let map = map_realms_to_windows(
        &existing_realms,
        &[(RealmId(1), 5), (RealmId(1), 3)],
        &[(SurfaceId(60), 8)],
        &realm_output_surfaces,
    );

    assert_eq!(map.get(&RealmId(1)), Some(&3));
    assert_eq!(map.get(&RealmId(2)), Some(&8));
}

#[test]
fn updates_present_size_cache_and_reports_stability() {
    let presents = vec![(SurfaceId(1), 10), (SurfaceId(2), 11)];
    let window_sizes = HashMap::from([
        (10, glam::UVec2::new(640, 480)),
        (11, glam::UVec2::new(800, 600)),
    ]);
    let mut cache = HashMap::new();
    let mut cache_hash = 0;

    assert!(update_present_size_cache(
        &presents,
        &window_sizes,
        &mut cache,
        &mut cache_hash,
    ));
    assert!(!update_present_size_cache(
        &presents,
        &window_sizes,
        &mut cache,
        &mut cache_hash,
    ));
    assert_eq!(cache.get(&SurfaceId(1)), Some(&glam::UVec2::new(640, 480)));
}

#[test]
fn groups_connectors_and_resolves_surface() {
    let grouped = collect_connectors_by_realm(&[
        (ConnectorId(2), RealmId(9)),
        (ConnectorId(1), RealmId(9)),
        (ConnectorId(5), RealmId(3)),
    ]);
    assert_eq!(
        grouped.get(&RealmId(9)),
        Some(&vec![ConnectorId(1), ConnectorId(2)])
    );

    let realm_output_surfaces = HashMap::from([(RealmId(3), Some(SurfaceId(77)))]);
    assert_eq!(
        resolve_realm_surface(&realm_output_surfaces, RealmId(3)),
        Some(SurfaceId(77))
    );
}

#[test]
fn should_render_realm_tracks_interval_and_updates_frame() {
    let mut realm = RealmState {
        kind: vulfram_realm_core::RealmKind::ThreeD,
        output_surface: None,
        render_graph_id: None,
        importance: 2,
        cache_policy: 1,
        last_render_frame: 0,
    };

    assert!(!should_render_realm(&mut realm, 1));
    assert!(should_render_realm(&mut realm, 4));
    assert_eq!(realm.last_render_frame, 4);
}

#[test]
fn builds_target_surface_map_for_texture_targets() {
    let targets = HashMap::from([
        (TargetId(1), (TargetKind::Texture, None)),
        (TargetId(2), (TargetKind::Window, None)),
    ]);
    let auto_links = HashMap::from([
        (
            (8, TargetId(1)),
            AutoLink {
                surface_id: SurfaceId(40),
                connector_id: None,
                present_id: None,
            },
        ),
        (
            (3, TargetId(1)),
            AutoLink {
                surface_id: SurfaceId(30),
                connector_id: None,
                present_id: None,
            },
        ),
    ]);

    let map = build_target_surface_map(&targets, &auto_links);
    assert_eq!(map.get(&TargetId(1)), Some(&SurfaceId(30)));
    assert!(!map.contains_key(&TargetId(2)));
}

#[test]
fn collects_window_camera_target_sizes_from_layout_or_explicit_size() {
    let layers = HashMap::from([
        (
            (77, TargetId(1)),
            TargetLayerState {
                realm_id: 77,
                target_id: TargetId(1),
                layout: TargetLayerLayout {
                    left: DimensionValue::Percent(0.0),
                    top: DimensionValue::Percent(0.0),
                    width: DimensionValue::Percent(50.0),
                    height: DimensionValue::Percent(25.0),
                    z_index: 0,
                    blend_mode: 0,
                    clip: None,
                },
                camera_id: Some(501),
                environment_id: None,
            },
        ),
        (
            (77, TargetId(2)),
            TargetLayerState {
                realm_id: 77,
                target_id: TargetId(2),
                layout: TargetLayerLayout {
                    left: DimensionValue::Percent(0.0),
                    top: DimensionValue::Percent(0.0),
                    width: DimensionValue::Percent(10.0),
                    height: DimensionValue::Percent(10.0),
                    z_index: 0,
                    blend_mode: 0,
                    clip: None,
                },
                camera_id: Some(777),
                environment_id: None,
            },
        ),
    ]);
    let targets = HashMap::from([
        (TargetId(1), (Some(9), None)),
        (TargetId(2), (Some(9), Some(glam::UVec2::new(333, 222)))),
    ]);

    let sizes = collect_window_camera_target_sizes(
        &layers,
        &targets,
        RealmId(77),
        9,
        glam::UVec2::new(1920, 1080),
    );
    assert_eq!(sizes.get(&501), Some(&glam::UVec2::new(960, 270)));
    assert_eq!(sizes.get(&777), Some(&glam::UVec2::new(333, 222)));
}

#[test]
fn plans_realm_environment_bindings_in_z_order() {
    let plan = plan_realm_environment_bindings(&[
        EnvironmentLayerBinding {
            target_id: TargetId(9),
            camera_id: Some(7),
            environment_id: Some(30),
            z_index: 5,
        },
        EnvironmentLayerBinding {
            target_id: TargetId(2),
            camera_id: None,
            environment_id: Some(11),
            z_index: 0,
        },
        EnvironmentLayerBinding {
            target_id: TargetId(3),
            camera_id: None,
            environment_id: Some(12),
            z_index: 10,
        },
    ]);

    assert_eq!(
        plan,
        RealmEnvironmentBindingPlan {
            realm_environment_id: Some(12),
            camera_environment_ids: HashMap::from([(7, 30)]),
        }
    );
}

#[test]
fn soft_cut_diagnostic_reports_connector_summary() {
    let diagnostic = build_soft_cut_diagnostic(
        &[FrameCutEdge {
            from: 1,
            to: 2,
            connector_id: Some(9),
        }],
        0,
        42,
    );

    assert_eq!(
        diagnostic.as_deref(),
        Some("frame=42 cut_edges=1 connectors=9")
    );
}

#[test]
fn resolves_connector_surface_with_cut_fallbacks() {
    let cut_connectors = HashSet::from([ConnectorId(4)]);
    let last_good = HashMap::from([(ConnectorId(4), SurfaceId(10))]);
    let fallback = HashMap::from([(ConnectorId(7), SurfaceId(20))]);

    assert_eq!(
        resolve_connector_surface(
            &cut_connectors,
            &last_good,
            &fallback,
            ConnectorId(4),
            SurfaceId(1),
        ),
        SurfaceId(10)
    );
    assert_eq!(
        resolve_connector_surface(
            &cut_connectors,
            &last_good,
            &fallback,
            ConnectorId(7),
            SurfaceId(2),
        ),
        SurfaceId(2)
    );
}

#[test]
fn plans_compose_overlays_and_reports_blockers() {
    let plan = plan_compose_overlays(
        &[
            ComposeConnectorCandidate {
                connector_id: ConnectorId(1),
                source_surface: SurfaceId(8),
                rect: glam::Vec4::ONE,
                clip: None,
                z_index: 5,
                blend_mode: 1,
                widget_view: false,
            },
            ComposeConnectorCandidate {
                connector_id: ConnectorId(2),
                source_surface: SurfaceId(9),
                rect: glam::Vec4::ZERO,
                clip: None,
                z_index: 1,
                blend_mode: 0,
                widget_view: false,
            },
            ComposeConnectorCandidate {
                connector_id: ConnectorId(3),
                source_surface: SurfaceId(3),
                rect: glam::Vec4::ZERO,
                clip: None,
                z_index: 0,
                blend_mode: 2,
                widget_view: false,
            },
        ],
        SurfaceId(3),
        &HashSet::from([ConnectorId(2)]),
        &HashMap::new(),
        &HashMap::from([(ConnectorId(2), SurfaceId(11))]),
        &HashSet::from([SurfaceId(8), SurfaceId(11)]),
        RealmId(77),
    );

    assert_eq!(
        plan,
        ComposeOverlayPlan {
            blocked_connectors: vec![],
            self_sampled_connectors: vec![ConnectorId(3)],
            no_progress_realms: vec![RealmId(77)],
            overlays: vec![
                ComposeOverlayPlanEntry {
                    connector_id: ConnectorId(2),
                    source_surface: SurfaceId(11),
                    rect: glam::Vec4::ZERO,
                    clip: None,
                    z_index: 1,
                    blend_mode: ComposeBlendMode::Alpha,
                },
                ComposeOverlayPlanEntry {
                    connector_id: ConnectorId(1),
                    source_surface: SurfaceId(8),
                    rect: glam::Vec4::ONE,
                    clip: None,
                    z_index: 5,
                    blend_mode: ComposeBlendMode::PremultipliedAlpha,
                },
            ],
        }
    );
}

#[test]
fn plans_surface_targets_with_present_override_for_onscreen() {
    let targets = plan_surface_targets(
        &[
            SurfaceTargetRequest {
                surface_id: SurfaceId(1),
                declared_size: glam::UVec2::new(100, 50),
                is_onscreen: true,
            },
            SurfaceTargetRequest {
                surface_id: SurfaceId(2),
                declared_size: glam::UVec2::new(64, 64),
                is_onscreen: false,
            },
        ],
        &HashMap::from([(SurfaceId(1), glam::UVec2::new(800, 600))]),
    );

    assert_eq!(
        targets,
        vec![
            ResolvedSurfaceTarget {
                surface_id: SurfaceId(1),
                target_size: glam::UVec2::new(800, 600),
            },
            ResolvedSurfaceTarget {
                surface_id: SurfaceId(2),
                target_size: glam::UVec2::new(64, 64),
            },
        ]
    );
}

#[test]
fn plans_target_size_updates_without_touching_window_targets() {
    let plan = plan_target_size_updates(&[
        TargetSizeUpdateRequest {
            target_id: TargetId(1),
            kind: TargetKind::Texture,
            current_size: Some(glam::UVec2::new(10, 10)),
            requested_size: glam::UVec2::new(20, 0),
            msaa_samples: None,
            window_id: Some(7),
        },
        TargetSizeUpdateRequest {
            target_id: TargetId(2),
            kind: TargetKind::Window,
            current_size: Some(glam::UVec2::new(30, 30)),
            requested_size: glam::UVec2::new(40, 40),
            msaa_samples: None,
            window_id: Some(8),
        },
    ]);

    assert_eq!(
        plan,
        vec![TargetSizeUpdatePlanEntry {
            target_id: TargetId(1),
            desired_size: glam::UVec2::new(20, 1),
            needs_size_update: true,
            needs_msaa_init: true,
            window_id: Some(7),
        }]
    );
}

#[test]
fn plans_external_texture_refresh_from_source_keys() {
    let plan = plan_external_texture_refresh(
        &HashMap::from([(1, 100_usize), (2, 200_usize)]),
        &[
            ExternalTextureSource {
                texture_id: 1,
                source_key: 100,
            },
            ExternalTextureSource {
                texture_id: 3,
                source_key: 300,
            },
        ],
    );

    assert_eq!(
        plan,
        ExternalTextureRefreshPlan {
            stale_ids: vec![2],
            replace_ids: vec![3],
        }
    );
}
