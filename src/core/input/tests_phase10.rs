use crate::core::cmd::EngineEvent;
use crate::core::input::events::{PointerEvent, PointerTraceConfig, PointerTraceLevel};
use crate::core::input::route_pointer_events;
use crate::core::realm::{
    AutoLink, ConnectorState, PresentState, RealmKind, RealmState, SurfaceKind, SurfaceState,
};
use crate::core::state::EngineState;
use crate::core::target::{TargetGraphPlan, TargetId, TargetKind, TargetState};

#[test]
fn routing_prefers_highest_z_index_layer_in_overlap() {
    let mut engine = EngineState::new();
    engine.universal_state.input_routing.trace = PointerTraceConfig {
        level: PointerTraceLevel::Full,
        sampling_percent: 100,
    };

    let root_surface = engine.universal_state.surfaces.alloc(SurfaceState {
        kind: SurfaceKind::Onscreen,
        size: glam::uvec2(400, 300),
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    });
    let source_surface = engine.universal_state.surfaces.alloc(SurfaceState {
        kind: SurfaceKind::Offscreen,
        size: glam::uvec2(256, 256),
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    });
    let root_realm = engine.universal_state.realms.alloc(RealmState {
        kind: RealmKind::ThreeD,
        output_surface: Some(root_surface),
        render_graph: None,
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });
    let _target_realm = engine.universal_state.realms.alloc(RealmState {
        kind: RealmKind::TwoD,
        output_surface: Some(source_surface),
        render_graph: None,
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });
    let _ = engine.universal_state.presents.alloc(PresentState {
        window_id: 1,
        surface: root_surface,
    });

    let top_target = TargetId(10);
    let bottom_target = TargetId(20);
    engine.universal_state.targets.entries.insert(
        top_target,
        TargetState {
            kind: TargetKind::WidgetRealmViewport,
            window_id: Some(1),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
    );
    engine.universal_state.targets.entries.insert(
        bottom_target,
        TargetState {
            kind: TargetKind::WidgetRealmViewport,
            window_id: Some(1),
            size: None,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        },
    );
    engine.universal_state.target_graph_cache.last_plan = TargetGraphPlan {
        edges: Vec::new(),
        order: vec![top_target, bottom_target],
        cut_edges: Vec::new(),
    };

    let top_connector = engine.universal_state.connectors.alloc(ConnectorState {
        target_realm: root_realm,
        source_surface,
        rect: glam::vec4(0.0, 0.0, 300.0, 200.0),
        z_index: 10,
        blend_mode: 0,
        clip: None,
        input_flags: 0,
    });
    let bottom_connector = engine.universal_state.connectors.alloc(ConnectorState {
        target_realm: root_realm,
        source_surface,
        rect: glam::vec4(0.0, 0.0, 300.0, 200.0),
        z_index: 2,
        blend_mode: 0,
        clip: None,
        input_flags: 0,
    });
    engine.universal_state.auto_links.insert(
        (root_realm.0, top_target),
        AutoLink {
            surface_id: source_surface,
            connector_id: Some(top_connector),
            present_id: None,
        },
    );
    engine.universal_state.auto_links.insert(
        (root_realm.0, bottom_target),
        AutoLink {
            surface_id: source_surface,
            connector_id: Some(bottom_connector),
            present_id: None,
        },
    );

    engine
        .event_queue
        .push(EngineEvent::Pointer(PointerEvent::OnMove {
            window_id: 1,
            pointer_type: 0,
            pointer_id: 1,
            position: glam::vec2(120.0, 80.0),
            position_target: None,
            trace: None,
        }));

    route_pointer_events(&mut engine);

    let trace = match engine.event_queue.first().unwrap() {
        EngineEvent::Pointer(PointerEvent::OnMove { trace, .. }) => trace.clone().unwrap(),
        _ => panic!("pointer trace missing"),
    };
    assert_eq!(trace.connector_id, Some(top_connector.0));
    assert_eq!(trace.target_id, Some(top_target.0));
}

#[test]
fn routing_trace_errors_mode_only_keeps_error_paths() {
    let mut engine = EngineState::new();
    engine.universal_state.input_routing.trace = PointerTraceConfig {
        level: PointerTraceLevel::Errors,
        sampling_percent: 100,
    };

    let root_surface = engine.universal_state.surfaces.alloc(SurfaceState {
        kind: SurfaceKind::Onscreen,
        size: glam::uvec2(320, 240),
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    });
    let _ = engine.universal_state.realms.alloc(RealmState {
        kind: RealmKind::TwoD,
        output_surface: Some(root_surface),
        render_graph: None,
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });
    let _ = engine.universal_state.presents.alloc(PresentState {
        window_id: 1,
        surface: root_surface,
    });

    engine
        .event_queue
        .push(EngineEvent::Pointer(PointerEvent::OnMove {
            window_id: 1,
            pointer_type: 0,
            pointer_id: 9,
            position: glam::vec2(12.0, 10.0),
            position_target: None,
            trace: None,
        }));

    route_pointer_events(&mut engine);

    let trace = match engine.event_queue.first().unwrap() {
        EngineEvent::Pointer(PointerEvent::OnMove { trace, .. }) => trace.clone(),
        _ => None,
    };
    assert!(trace.is_none());
}

#[test]
fn routing_multiple_targets_no_cross_layer_bleed() {
    let mut engine = EngineState::new();
    engine.universal_state.input_routing.trace = PointerTraceConfig {
        level: PointerTraceLevel::Full,
        sampling_percent: 100,
    };

    let root_surface = engine.universal_state.surfaces.alloc(SurfaceState {
        kind: SurfaceKind::Onscreen,
        size: glam::uvec2(600, 240),
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    });
    let source_surface = engine.universal_state.surfaces.alloc(SurfaceState {
        kind: SurfaceKind::Offscreen,
        size: glam::uvec2(256, 256),
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    });
    let root_realm = engine.universal_state.realms.alloc(RealmState {
        kind: RealmKind::ThreeD,
        output_surface: Some(root_surface),
        render_graph: None,
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });
    let _ = engine.universal_state.realms.alloc(RealmState {
        kind: RealmKind::TwoD,
        output_surface: Some(source_surface),
        render_graph: None,
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });
    let _ = engine.universal_state.presents.alloc(PresentState {
        window_id: 1,
        surface: root_surface,
    });

    let left_target = TargetId(101);
    let right_target = TargetId(102);
    for target_id in [left_target, right_target] {
        engine.universal_state.targets.entries.insert(
            target_id,
            TargetState {
                kind: TargetKind::WidgetRealmViewport,
                window_id: Some(1),
                size: None,
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );
    }
    engine.universal_state.target_graph_cache.last_plan = TargetGraphPlan {
        edges: Vec::new(),
        order: vec![left_target, right_target],
        cut_edges: Vec::new(),
    };

    let left_connector = engine.universal_state.connectors.alloc(ConnectorState {
        target_realm: root_realm,
        source_surface,
        rect: glam::vec4(0.0, 0.0, 280.0, 200.0),
        z_index: 2,
        blend_mode: 0,
        clip: None,
        input_flags: 0,
    });
    let right_connector = engine.universal_state.connectors.alloc(ConnectorState {
        target_realm: root_realm,
        source_surface,
        rect: glam::vec4(320.0, 0.0, 280.0, 200.0),
        z_index: 2,
        blend_mode: 0,
        clip: None,
        input_flags: 0,
    });
    engine.universal_state.auto_links.insert(
        (root_realm.0, left_target),
        AutoLink {
            surface_id: source_surface,
            connector_id: Some(left_connector),
            present_id: None,
        },
    );
    engine.universal_state.auto_links.insert(
        (root_realm.0, right_target),
        AutoLink {
            surface_id: source_surface,
            connector_id: Some(right_connector),
            present_id: None,
        },
    );

    engine
        .event_queue
        .push(EngineEvent::Pointer(PointerEvent::OnMove {
            window_id: 1,
            pointer_type: 0,
            pointer_id: 1,
            position: glam::vec2(80.0, 80.0),
            position_target: None,
            trace: None,
        }));
    engine
        .event_queue
        .push(EngineEvent::Pointer(PointerEvent::OnMove {
            window_id: 1,
            pointer_type: 0,
            pointer_id: 2,
            position: glam::vec2(520.0, 80.0),
            position_target: None,
            trace: None,
        }));

    route_pointer_events(&mut engine);

    let first_target = match &engine.event_queue[0] {
        EngineEvent::Pointer(PointerEvent::OnMove { trace, .. }) => {
            trace.as_ref().and_then(|trace| trace.target_id)
        }
        _ => None,
    };
    let second_target = match &engine.event_queue[1] {
        EngineEvent::Pointer(PointerEvent::OnMove { trace, .. }) => {
            trace.as_ref().and_then(|trace| trace.target_id)
        }
        _ => None,
    };
    assert_eq!(first_target, Some(left_target.0));
    assert_eq!(second_target, Some(right_target.0));
}
