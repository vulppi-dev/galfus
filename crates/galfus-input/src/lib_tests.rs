use super::*;
use galfus_realm_core::ConnectorState;
use galfus_types::ConnectorId;

#[test]
fn modifiers_default_to_all_false() {
    let modifiers = ModifiersState::default();
    assert!(!modifiers.shift);
    assert!(!modifiers.ctrl);
    assert!(!modifiers.alt);
    assert!(!modifiers.meta);
}

#[test]
fn pointer_trace_config_defaults_to_full_sampling() {
    let config = PointerTraceConfig::default();
    assert_eq!(config.level, PointerTraceLevel::Full);
    assert_eq!(config.sampling_percent, 100);
}

#[test]
fn keyboard_event_round_trips_through_messagepack() {
    let event = KeyboardEvent::OnInput {
        window_id: 1,
        key_code: 42,
        state: ElementState::Pressed,
        location: 0,
        repeat: false,
        text: Some("a".into()),
        modifiers: ModifiersState {
            shift: true,
            ctrl: false,
            alt: false,
            meta: false,
        },
    };

    let bytes = rmp_serde::to_vec_named(&event).expect("keyboard event should encode");
    let decoded: KeyboardEvent =
        rmp_serde::from_slice(&bytes).expect("keyboard event should decode");

    match decoded {
        KeyboardEvent::OnInput {
            window_id,
            key_code,
            state,
            ..
        } => {
            assert_eq!(window_id, 1);
            assert_eq!(key_code, 42);
            assert_eq!(state, ElementState::Pressed);
        }
        _ => panic!("decoded wrong keyboard event variant"),
    }
}

#[test]
fn input_target_listener_store_lists_sorted_snapshots() {
    let mut store = InputTargetListenerStore::default();
    store.upsert(InputTargetListenerConfig {
        listener_id: 20,
        target_id: 7,
        enabled: true,
        events: vec!["pointer-move".into()],
        sample_percent: 100,
    });
    store.upsert(InputTargetListenerConfig {
        listener_id: 10,
        target_id: 7,
        enabled: false,
        events: Vec::new(),
        sample_percent: 0,
    });

    let listeners = store.list(Some(7));
    assert_eq!(listeners.len(), 2);
    assert_eq!(listeners[0].listener_id, 10);
    assert_eq!(listeners[1].listener_id, 20);
}

#[test]
fn input_target_listener_store_disposes_target_group() {
    let mut store = InputTargetListenerStore::default();
    store.upsert(InputTargetListenerConfig {
        listener_id: 1,
        target_id: 10,
        enabled: true,
        events: Vec::new(),
        sample_percent: 100,
    });
    store.upsert(InputTargetListenerConfig {
        listener_id: 2,
        target_id: 10,
        enabled: true,
        events: Vec::new(),
        sample_percent: 100,
    });
    store.upsert(InputTargetListenerConfig {
        listener_id: 3,
        target_id: 11,
        enabled: true,
        events: Vec::new(),
        sample_percent: 100,
    });

    assert_eq!(store.dispose_target(10), 2);
    assert_eq!(store.list(None).len(), 1);
    assert_eq!(store.listeners_for_target(10).len(), 0);
    assert_eq!(store.listeners_for_target(11).len(), 1);
}

#[test]
fn select_trace_payload_basic_strips_detailed_fields() {
    let full = PointerEventTrace {
        window_id: 1,
        realm_id: 2,
        target_id: Some(3),
        connector_id: Some(4),
        source_realm_id: Some(5),
        uv: Some(Vec2::new(0.25, 0.75)),
        hops: vec![PointerTraceHop {
            stage: PointerTraceStage::ConnectorHit,
            realm_id: Some(2),
            target_id: Some(3),
            layer_realm_id: Some(2),
            connector_id: Some(4),
            camera_id: Some(7),
            uv: Some(Vec2::new(0.25, 0.75)),
        }],
    };

    let trace = select_trace_payload(
        PointerTraceConfig {
            level: PointerTraceLevel::Basic,
            sampling_percent: 100,
        },
        0,
        1,
        Some(9),
        full,
    )
    .expect("basic trace should be present");

    assert_eq!(trace.window_id, 1);
    assert_eq!(trace.realm_id, 2);
    assert_eq!(trace.target_id, Some(3));
    assert!(trace.connector_id.is_none());
    assert!(trace.source_realm_id.is_none());
    assert!(trace.uv.is_none());
    assert!(trace.hops.is_empty());
}

#[test]
fn update_focus_state_tracks_press_and_release() {
    let mut focus_targets = HashMap::new();
    let event = PointerEvent::OnButton {
        window_id: 1,
        window_width: None,
        window_height: None,
        pointer_type: 0,
        pointer_id: 10,
        button: 0,
        state: ElementState::Pressed,
        position: Vec2::new(0.0, 0.0),
        position_target: None,
        target_width: None,
        target_height: None,
        trace: None,
    };

    update_focus_state(&mut focus_targets, 1, Some(42), &event);
    assert_eq!(focus_targets.get(&1), Some(&42));

    let release = PointerEvent::OnButton {
        window_id: 1,
        window_width: None,
        window_height: None,
        pointer_type: 0,
        pointer_id: 10,
        button: 0,
        state: ElementState::Released,
        position: Vec2::new(0.0, 0.0),
        position_target: None,
        target_width: None,
        target_height: None,
        trace: None,
    };
    update_focus_state(&mut focus_targets, 1, Some(42), &release);
    assert!(focus_targets.is_empty());
}

#[test]
fn input_routing_state_defaults_to_empty_maps_and_full_trace() {
    let routing = InputRoutingState::default();

    assert!(routing.captures.is_empty());
    assert!(routing.focus_targets.is_empty());
    assert_eq!(routing.trace.level, PointerTraceLevel::Full);
    assert_eq!(routing.trace.sampling_percent, 100);
    assert_eq!(routing.cache.topology_hash, 0);
    assert!(routing.cache.connectors_by_realm.is_empty());
}

#[test]
fn resolve_hit_connector_returns_uv_for_raycast_connectors() {
    let connectors = vec![InputRoutingConnectorHit {
        id: ConnectorId(7),
        state: ConnectorState {
            source_surface: SurfaceId(1),
            target_realm: RealmId(2),
            rect: glam::Vec4::new(0.0, 0.0, 100.0, 100.0),
            clip: None,
            z_index: 0,
            blend_mode: 0,
            input_flags: 1,
        },
        source_size: glam::UVec2::new(100, 100),
        target_id: None,
        target_rank: 0,
    }];

    let hit = resolve_hit_connector(
        Some(&connectors),
        Vec2::new(50.0, 50.0),
        Some(glam::UVec2::new(100, 100)),
    )
    .expect("connector should hit");

    assert_eq!(hit.connector_id, ConnectorId(7));
    assert_eq!(hit.uv, Some(Vec2::new(0.5, 0.5)));
}

#[test]
fn resolve_captured_connector_maps_primitives_to_ids() {
    let mut captures = HashMap::new();
    captures.insert(
        (1, 2),
        InputCapture {
            connector_id: 9,
            target_id: Some(11),
        },
    );

    let resolved = resolve_captured_connector(&captures, 1, 2).expect("capture should resolve");

    assert_eq!(resolved.connector_id, ConnectorId(9));
    assert_eq!(resolved.target_id, Some(galfus_realm_core::TargetId(11)));
}

#[test]
fn resolve_connector_for_target_finds_matching_entry() {
    let connectors = vec![InputRoutingConnectorHit {
        id: ConnectorId(7),
        state: ConnectorState {
            source_surface: SurfaceId(1),
            target_realm: RealmId(2),
            rect: glam::Vec4::new(0.0, 0.0, 100.0, 100.0),
            clip: None,
            z_index: 0,
            blend_mode: 0,
            input_flags: 0,
        },
        source_size: glam::UVec2::new(100, 100),
        target_id: Some(galfus_realm_core::TargetId(42)),
        target_rank: 0,
    }];

    let connector_id =
        resolve_connector_for_target(Some(&connectors), galfus_realm_core::TargetId(42));

    assert_eq!(connector_id, Some(ConnectorId(7)));
}

#[test]
fn build_input_routing_cache_sorts_connectors_by_z_then_rank() {
    let snapshot = InputRoutingTopologySnapshot {
        realms: vec![InputRoutingRealmOutput {
            realm_id: RealmId(1),
            output_surface: Some(SurfaceId(5)),
        }],
        presents: vec![InputRoutingPresentBinding {
            window_id: 9,
            output_id: SurfaceId(5),
        }],
        target_order: vec![
            InputRoutingTargetRank {
                target_id: galfus_realm_core::TargetId(100),
                rank: 0,
            },
            InputRoutingTargetRank {
                target_id: galfus_realm_core::TargetId(200),
                rank: 1,
            },
        ],
        auto_links: vec![
            InputRoutingAutoLinkRecord {
                target_id: galfus_realm_core::TargetId(100),
                connector_id: ConnectorId(1),
            },
            InputRoutingAutoLinkRecord {
                target_id: galfus_realm_core::TargetId(200),
                connector_id: ConnectorId(2),
            },
        ],
        layer_cameras: Vec::new(),
        connectors: vec![
            InputRoutingConnectorRecord {
                connector_id: ConnectorId(1),
                state: ConnectorState {
                    source_surface: SurfaceId(7),
                    target_realm: RealmId(1),
                    rect: glam::Vec4::new(0.0, 0.0, 10.0, 10.0),
                    z_index: 2,
                    blend_mode: 0,
                    clip: None,
                    input_flags: 0,
                },
                source_size: glam::UVec2::new(10, 10),
            },
            InputRoutingConnectorRecord {
                connector_id: ConnectorId(2),
                state: ConnectorState {
                    source_surface: SurfaceId(8),
                    target_realm: RealmId(1),
                    rect: glam::Vec4::new(0.0, 0.0, 10.0, 10.0),
                    z_index: 2,
                    blend_mode: 0,
                    clip: None,
                    input_flags: 0,
                },
                source_size: glam::UVec2::new(10, 10),
            },
        ],
        surfaces: vec![
            InputRoutingSurfaceSizeRecord {
                output_id: SurfaceId(5),
                size: glam::UVec2::new(100, 100),
            },
            InputRoutingSurfaceSizeRecord {
                output_id: SurfaceId(7),
                size: glam::UVec2::new(10, 10),
            },
            InputRoutingSurfaceSizeRecord {
                output_id: SurfaceId(8),
                size: glam::UVec2::new(10, 10),
            },
        ],
    };

    let cache = build_input_routing_cache(&snapshot);
    let connectors = cache
        .connectors_by_realm
        .get(&RealmId(1))
        .expect("realm connectors should exist");

    assert_eq!(
        cache.realm_by_window.get(&9),
        Some(&(RealmId(1), SurfaceId(5)))
    );
    assert_eq!(connectors[0].id, ConnectorId(2));
    assert_eq!(connectors[1].id, ConnectorId(1));
}

#[test]
fn resolve_target_relative_position_uses_first_runtime_size() {
    let position = resolve_target_relative_position(
        InputTargetSizing {
            source_realm_size: Some(glam::UVec2::new(200, 100)),
            connector_source_size: Some(glam::UVec2::new(10, 10)),
            target_output_size: None,
            target_declared_size: None,
        },
        Some(Vec2::new(0.25, 0.5)),
    );

    assert_eq!(position, Some(Vec2::new(50.0, 50.0)));
}

#[test]
fn resolve_target_size_falls_back_through_all_sources() {
    assert_eq!(
        resolve_target_size(InputTargetSizing {
            source_realm_size: None,
            connector_source_size: None,
            target_output_size: Some(glam::UVec2::new(300, 200)),
            target_declared_size: Some(glam::UVec2::new(10, 10)),
        }),
        Some(glam::UVec2::new(300, 200))
    );

    assert_eq!(
        resolve_target_size(InputTargetSizing {
            source_realm_size: None,
            connector_source_size: None,
            target_output_size: None,
            target_declared_size: Some(glam::UVec2::new(10, 10)),
        }),
        Some(glam::UVec2::new(10, 10))
    );
}
