use std::collections::HashSet;

use glam::Vec2;

use crate::core::cmd::EngineEvent;
use crate::core::input::events::{PointerEventTrace, PointerTraceHop, PointerTraceStage};
use crate::core::input::raycast::resolve_realm_plane_hit;
use crate::core::realm::RealmId;
use crate::core::state::EngineState;

mod route_cache;
mod route_events;
mod route_hit;

use route_cache::{
    realm_surface_size, rebuild_input_routing_cache, resolve_captured_connector,
    resolve_connector_for_target, resolve_focus_target,
};
use route_events::{
    apply_target_position, apply_target_size, apply_trace, apply_window_size, pointer_id,
    pointer_position, pointer_window_id, quantize_uv, select_trace_payload, update_capture_state,
    update_focus_state,
};
use route_hit::{
    resolve_connector_uv, resolve_hit_connector, resolve_target_relative_position,
    resolve_target_size,
};

const MAX_ROUTE_STEPS: usize = 32;

pub fn route_pointer_events(engine_state: &mut EngineState) {
    let mut events = engine_state.runtime.take_events();
    let trace_config = engine_state.universal_state.interaction.input_routing.trace;
    let frame_index = engine_state.runtime.frame_index();
    rebuild_input_routing_cache(&mut engine_state.universal_state);
    let mut captures = std::mem::take(
        &mut engine_state
            .universal_state
            .interaction
            .input_routing
            .captures,
    );
    let mut focus_targets = std::mem::take(
        &mut engine_state
            .universal_state
            .interaction
            .input_routing
            .focus_targets,
    );

    {
        let routing_cache = &engine_state.universal_state.interaction.input_routing.cache;
        let realm_by_surface = &routing_cache.realm_by_surface;
        let realm_by_window = &routing_cache.realm_by_window;
        let connector_targets = &routing_cache.connector_targets;
        let layer_camera_by_key = &routing_cache.layer_camera_by_key;
        let connectors_by_realm = &routing_cache.connectors_by_realm;

        for event in events.iter_mut() {
            let EngineEvent::Pointer(pointer_event) = event else {
                continue;
            };

            let window_id = pointer_window_id(pointer_event);
            let Some((realm_id, root_surface_id)) = realm_by_window.get(&window_id).copied() else {
                continue;
            };

            let root_surface_size = engine_state
                .universal_state
                .composition
                .surfaces
                .entries
                .get(&root_surface_id)
                .map(|entry| entry.value.size)
                .or_else(|| {
                    engine_state
                        .window
                        .states
                        .get(&window_id)
                        .map(|state| state.inner_size)
                });
            let pointer_id_value = pointer_id(pointer_event);
            let position = pointer_position(pointer_event).or_else(|| {
                engine_state
                    .window
                    .cursor_positions
                    .get(&window_id)
                    .copied()
            });

            let mut connector_id = None;
            let mut target_id = None;
            let mut source_realm_id = None;
            let mut uv = None;
            let mut uv_override = None;
            let mut hops: Vec<PointerTraceHop> = Vec::new();
            hops.push(PointerTraceHop {
                stage: PointerTraceStage::RootWindow,
                realm_id: Some(realm_id.0),
                target_id: None,
                layer_realm_id: None,
                connector_id: None,
                surface_id: Some(root_surface_id.0),
                camera_id: None,
                uv: None,
            });

            if let (Some(pointer_id), Some(position)) = (pointer_id_value, position) {
                if let Some(capture) = resolve_captured_connector(&captures, window_id, pointer_id)
                {
                    target_id = capture.target_id;
                    connector_id = if engine_state
                        .universal_state
                        .composition
                        .connectors
                        .entries
                        .contains_key(&capture.connector_id)
                    {
                        Some(capture.connector_id)
                    } else {
                        capture.target_id.and_then(|target_id| {
                            resolve_connector_for_target(
                                connectors_by_realm.get(&realm_id),
                                target_id,
                            )
                        })
                    };
                    hops.push(PointerTraceHop {
                        stage: PointerTraceStage::Capture,
                        realm_id: Some(realm_id.0),
                        target_id: target_id.map(|id| id.0),
                        layer_realm_id: Some(realm_id.0),
                        connector_id: connector_id.map(|id| id.0),
                        surface_id: None,
                        camera_id: target_id.and_then(|id| {
                            layer_camera_by_key
                                .get(&(realm_id.0, id))
                                .copied()
                                .flatten()
                        }),
                        uv: None,
                    });
                } else if let Some(hit) = resolve_hit_connector(
                    connectors_by_realm.get(&realm_id),
                    position,
                    root_surface_size,
                ) {
                    connector_id = Some(hit.connector_id);
                    uv_override = hit.uv;
                    hops.push(PointerTraceHop {
                        stage: PointerTraceStage::ConnectorHit,
                        realm_id: Some(realm_id.0),
                        target_id: connector_targets.get(&hit.connector_id).map(|id| id.0),
                        layer_realm_id: Some(realm_id.0),
                        connector_id: Some(hit.connector_id.0),
                        surface_id: None,
                        camera_id: connector_targets.get(&hit.connector_id).and_then(|id| {
                            layer_camera_by_key
                                .get(&(realm_id.0, *id))
                                .copied()
                                .flatten()
                        }),
                        uv: hit.uv,
                    });
                }

                if connector_id.is_none() {
                    if let Some(focused_target) = resolve_focus_target(&focus_targets, window_id) {
                        target_id = Some(focused_target);
                        connector_id = resolve_connector_for_target(
                            connectors_by_realm.get(&realm_id),
                            focused_target,
                        );
                        hops.push(PointerTraceHop {
                            stage: PointerTraceStage::FocusFallback,
                            realm_id: Some(realm_id.0),
                            target_id: Some(focused_target.0),
                            layer_realm_id: Some(realm_id.0),
                            connector_id: connector_id.map(|id| id.0),
                            surface_id: None,
                            camera_id: layer_camera_by_key
                                .get(&(realm_id.0, focused_target))
                                .copied()
                                .flatten(),
                            uv: None,
                        });
                    }
                }

                if target_id.is_none() {
                    target_id = connector_id.and_then(|id| connector_targets.get(&id).copied());
                }

                if let Some(connector_id) = connector_id {
                    let connector = engine_state
                        .universal_state
                        .composition
                        .connectors
                        .entries
                        .get(&connector_id)
                        .map(|entry| &entry.value);
                    if let Some(connector) = connector {
                        source_realm_id = realm_by_surface.get(&connector.source_surface).copied();
                        uv = uv_override.or_else(|| {
                            resolve_connector_uv(
                                &engine_state.universal_state,
                                connector,
                                position,
                                root_surface_size.unwrap_or(glam::UVec2::new(1, 1)),
                            )
                        });
                        hops.push(PointerTraceHop {
                            stage: PointerTraceStage::HopForward,
                            realm_id: source_realm_id.map(|id| id.0),
                            target_id: target_id.map(|id| id.0),
                            layer_realm_id: Some(connector.target_realm.0),
                            connector_id: Some(connector_id.0),
                            surface_id: Some(connector.source_surface.0),
                            camera_id: target_id.and_then(|id| {
                                layer_camera_by_key
                                    .get(&(connector.target_realm.0, id))
                                    .copied()
                                    .flatten()
                            }),
                            uv,
                        });
                    }
                } else if source_realm_id.is_none() {
                    if let Some(realm_plane_hit) = resolve_realm_plane_hit(
                        engine_state,
                        window_id,
                        realm_id,
                        target_id.and_then(|id| {
                            layer_camera_by_key
                                .get(&(realm_id.0, id))
                                .copied()
                                .flatten()
                        }),
                        position,
                        root_surface_size.unwrap_or(glam::UVec2::new(1, 1)),
                    ) {
                        source_realm_id = Some(realm_plane_hit.source_realm_id);
                        target_id = Some(realm_plane_hit.target_id);
                        uv = Some(realm_plane_hit.uv);
                        hops.push(PointerTraceHop {
                            stage: PointerTraceStage::RealmPlaneHit,
                            realm_id: Some(realm_plane_hit.source_realm_id.0),
                            target_id: Some(realm_plane_hit.target_id.0),
                            layer_realm_id: Some(realm_id.0),
                            connector_id: None,
                            surface_id: None,
                            camera_id: layer_camera_by_key
                                .get(&(realm_id.0, realm_plane_hit.target_id))
                                .copied()
                                .flatten(),
                            uv: Some(realm_plane_hit.uv),
                        });
                    }
                }

                let mut visited: HashSet<(RealmId, u16, u16)> = HashSet::new();
                for _ in 0..MAX_ROUTE_STEPS {
                    let (current_realm, current_uv) = match (source_realm_id, uv) {
                        (Some(realm), Some(uv)) => (realm, uv),
                        _ => break,
                    };
                    let key = (
                        current_realm,
                        quantize_uv(current_uv.x),
                        quantize_uv(current_uv.y),
                    );
                    if !visited.insert(key) {
                        hops.push(PointerTraceHop {
                            stage: PointerTraceStage::StopCycle,
                            realm_id: Some(current_realm.0),
                            target_id: target_id.map(|id| id.0),
                            layer_realm_id: Some(current_realm.0),
                            connector_id: connector_id.map(|id| id.0),
                            surface_id: None,
                            camera_id: None,
                            uv: Some(current_uv),
                        });
                        break;
                    }
                    let Some(surface_size) =
                        realm_surface_size(&engine_state.universal_state, current_realm)
                    else {
                        break;
                    };
                    let current_position = Vec2::new(
                        current_uv.x * surface_size.x as f32,
                        current_uv.y * surface_size.y as f32,
                    );

                    if let Some(hit) = resolve_hit_connector(
                        connectors_by_realm.get(&current_realm),
                        current_position,
                        Some(surface_size),
                    ) {
                        connector_id = Some(hit.connector_id);
                        target_id = connector_targets
                            .get(&hit.connector_id)
                            .copied()
                            .or(target_id);
                        let connector = engine_state
                            .universal_state
                            .composition
                            .connectors
                            .entries
                            .get(&hit.connector_id)
                            .map(|entry| &entry.value);
                        let Some(connector) = connector else {
                            break;
                        };
                        let next_realm = realm_by_surface.get(&connector.source_surface).copied();
                        let next_uv = hit.uv.or_else(|| {
                            resolve_connector_uv(
                                &engine_state.universal_state,
                                connector,
                                current_position,
                                surface_size,
                            )
                        });
                        hops.push(PointerTraceHop {
                            stage: PointerTraceStage::HopForward,
                            realm_id: next_realm.map(|id| id.0),
                            target_id: target_id.map(|id| id.0),
                            layer_realm_id: Some(current_realm.0),
                            connector_id: Some(hit.connector_id.0),
                            surface_id: Some(connector.source_surface.0),
                            camera_id: target_id.and_then(|id| {
                                layer_camera_by_key
                                    .get(&(current_realm.0, id))
                                    .copied()
                                    .flatten()
                            }),
                            uv: next_uv,
                        });
                        match (next_realm, next_uv) {
                            (Some(next_realm), Some(next_uv)) => {
                                source_realm_id = Some(next_realm);
                                uv = Some(next_uv);
                                continue;
                            }
                            _ => break,
                        }
                    }

                    if let Some(realm_plane_hit) = resolve_realm_plane_hit(
                        engine_state,
                        window_id,
                        current_realm,
                        target_id.and_then(|id| {
                            layer_camera_by_key
                                .get(&(current_realm.0, id))
                                .copied()
                                .flatten()
                        }),
                        current_position,
                        surface_size,
                    ) {
                        source_realm_id = Some(realm_plane_hit.source_realm_id);
                        target_id = Some(realm_plane_hit.target_id);
                        uv = Some(realm_plane_hit.uv);
                        hops.push(PointerTraceHop {
                            stage: PointerTraceStage::RealmPlaneHit,
                            realm_id: Some(realm_plane_hit.source_realm_id.0),
                            target_id: Some(realm_plane_hit.target_id.0),
                            layer_realm_id: Some(current_realm.0),
                            connector_id: None,
                            surface_id: None,
                            camera_id: layer_camera_by_key
                                .get(&(current_realm.0, realm_plane_hit.target_id))
                                .copied()
                                .flatten(),
                            uv: Some(realm_plane_hit.uv),
                        });
                        continue;
                    }
                    hops.push(PointerTraceHop {
                        stage: PointerTraceStage::StopNoHit,
                        realm_id: Some(current_realm.0),
                        target_id: target_id.map(|id| id.0),
                        layer_realm_id: Some(current_realm.0),
                        connector_id: connector_id.map(|id| id.0),
                        surface_id: None,
                        camera_id: None,
                        uv: Some(current_uv),
                    });
                    break;
                }

                if visited.len() >= MAX_ROUTE_STEPS {
                    hops.push(PointerTraceHop {
                        stage: PointerTraceStage::StopStepBudget,
                        realm_id: source_realm_id.map(|id| id.0),
                        target_id: target_id.map(|id| id.0),
                        layer_realm_id: source_realm_id.map(|id| id.0),
                        connector_id: connector_id.map(|id| id.0),
                        surface_id: None,
                        camera_id: None,
                        uv,
                    });
                }

                update_capture_state(
                    &mut captures,
                    window_id,
                    pointer_id,
                    connector_id,
                    target_id,
                    pointer_event,
                );
                update_focus_state(&mut focus_targets, window_id, target_id, pointer_event);
            }

            let full_trace = PointerEventTrace {
                window_id,
                realm_id: realm_id.0,
                target_id: target_id.map(|id| id.0),
                connector_id: connector_id.map(|id| id.0),
                source_realm_id: source_realm_id.map(|id| id.0),
                uv,
                hops,
            };

            let trace = select_trace_payload(
                trace_config,
                frame_index,
                pointer_window_id(pointer_event),
                pointer_id_value,
                full_trace,
            );
            let position_target = resolve_target_relative_position(
                &engine_state.universal_state,
                source_realm_id,
                connector_id,
                uv,
            );
            let window_size = engine_state
                .window
                .states
                .get(&window_id)
                .map(|state| state.inner_size);
            let target_size = resolve_target_size(
                &engine_state.universal_state,
                source_realm_id,
                connector_id,
                target_id,
            );
            apply_target_position(pointer_event, position_target);
            apply_target_size(
                pointer_event,
                target_size.map(|size| size.x),
                target_size.map(|size| size.y),
            );
            apply_window_size(
                pointer_event,
                window_size.map(|size| size.x),
                window_size.map(|size| size.y),
            );
            apply_trace(pointer_event, trace);
        }
    }

    engine_state.runtime.replace_events(events);
    engine_state
        .universal_state
        .interaction
        .input_routing
        .captures = captures;
    engine_state
        .universal_state
        .interaction
        .input_routing
        .focus_targets = focus_targets;
}
