use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use glam::{UVec2, Vec2};

use crate::core::cmd::EngineEvent;
use crate::core::input::events::{
    ElementState, PointerEvent, PointerEventTrace, PointerTraceConfig, PointerTraceHop,
    PointerTraceLevel, PointerTraceStage, TouchPhase,
};
use crate::core::input::raycast::resolve_realm_plane_hit;
use crate::core::realm::{ConnectorId, ConnectorState, InputCapture, RealmId, UniversalState};
use crate::core::state::EngineState;
use crate::core::target::TargetId;

const INPUT_FLAG_RAYCAST: u32 = 1 << 0;
const MAX_ROUTE_STEPS: usize = 32;

pub fn route_pointer_events(engine_state: &mut EngineState) {
    let mut events = std::mem::take(&mut engine_state.event_queue);
    let trace_config = engine_state.universal_state.input_routing.trace;
    let frame_index = engine_state.frame_index;

    let mut realm_by_surface = HashMap::new();
    for (realm_id, entry) in engine_state.universal_state.realms.entries.iter() {
        if let Some(surface_id) = entry.value.output_surface {
            realm_by_surface.insert(surface_id, *realm_id);
        }
    }

    let mut realm_by_window = HashMap::new();
    for present in engine_state.universal_state.presents.entries.values() {
        if let Some(realm_id) = realm_by_surface.get(&present.value.surface) {
            realm_by_window.insert(present.value.window_id, (*realm_id, present.value.surface));
        }
    }

    let mut target_rank: HashMap<TargetId, i32> = HashMap::new();
    for (index, target_id) in engine_state
        .universal_state
        .target_graph_cache
        .last_plan
        .order
        .iter()
        .enumerate()
    {
        target_rank.insert(*target_id, index as i32);
    }

    let mut connector_targets: HashMap<ConnectorId, TargetId> = HashMap::new();
    for ((_, target_id), link) in engine_state.universal_state.auto_links.iter() {
        if let Some(connector_id) = link.connector_id {
            connector_targets.insert(connector_id, *target_id);
        }
    }
    let mut layer_camera_by_key: HashMap<(u32, TargetId), Option<u32>> = HashMap::new();
    for ((layer_realm_id, layer_target_id), layer) in
        engine_state.universal_state.target_layers.entries.iter()
    {
        layer_camera_by_key.insert((*layer_realm_id, *layer_target_id), layer.camera_id);
    }

    let mut connectors_by_realm: HashMap<RealmId, Vec<ConnectorHit>> = HashMap::new();
    for (connector_id, entry) in engine_state.universal_state.connectors.entries.iter() {
        let Some(source_size) = engine_state
            .universal_state
            .surfaces
            .entries
            .get(&entry.value.source_surface)
            .map(|surface| surface.value.size)
        else {
            continue;
        };
        let target_id = connector_targets.get(connector_id).copied();
        let rank = target_id
            .and_then(|id| target_rank.get(&id).copied())
            .unwrap_or(-1);
        connectors_by_realm
            .entry(entry.value.target_realm)
            .or_default()
            .push(ConnectorHit {
                id: *connector_id,
                state: entry.value.clone(),
                source_size,
                target_id,
                target_rank: rank,
            });
    }

    for connectors in connectors_by_realm.values_mut() {
        connectors.sort_by(|a, b| {
            let z_cmp = b.state.z_index.cmp(&a.state.z_index);
            if z_cmp == Ordering::Equal {
                let rank_cmp = b.target_rank.cmp(&a.target_rank);
                if rank_cmp == Ordering::Equal {
                    b.id.0.cmp(&a.id.0)
                } else {
                    rank_cmp
                }
            } else {
                z_cmp
            }
        });
    }

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
            if let Some(capture) =
                resolve_captured_connector(&engine_state.universal_state, window_id, pointer_id)
            {
                target_id = capture.target_id;
                connector_id = if engine_state
                    .universal_state
                    .connectors
                    .entries
                    .contains_key(&capture.connector_id)
                {
                    Some(capture.connector_id)
                } else {
                    capture.target_id.and_then(|target_id| {
                        resolve_connector_for_target(connectors_by_realm.get(&realm_id), target_id)
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
                if let Some(focused_target) =
                    resolve_focus_target(&engine_state.universal_state, window_id)
                {
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
                &mut engine_state.universal_state,
                window_id,
                pointer_id,
                connector_id,
                target_id,
                pointer_event,
            );
            update_focus_state(
                &mut engine_state.universal_state,
                window_id,
                target_id,
                pointer_event,
            );
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
        apply_trace(pointer_event, trace);
    }

    engine_state.event_queue = events;
}

fn pointer_window_id(event: &PointerEvent) -> u32 {
    match event {
        PointerEvent::OnMove { window_id, .. }
        | PointerEvent::OnEnter { window_id, .. }
        | PointerEvent::OnLeave { window_id, .. }
        | PointerEvent::OnButton { window_id, .. }
        | PointerEvent::OnScroll { window_id, .. }
        | PointerEvent::OnTouch { window_id, .. }
        | PointerEvent::OnPinchGesture { window_id, .. }
        | PointerEvent::OnPanGesture { window_id, .. }
        | PointerEvent::OnRotationGesture { window_id, .. }
        | PointerEvent::OnDoubleTapGesture { window_id, .. } => *window_id,
    }
}

fn pointer_id(event: &PointerEvent) -> Option<u64> {
    match event {
        PointerEvent::OnMove { pointer_id, .. }
        | PointerEvent::OnEnter { pointer_id, .. }
        | PointerEvent::OnLeave { pointer_id, .. }
        | PointerEvent::OnButton { pointer_id, .. }
        | PointerEvent::OnTouch { pointer_id, .. } => Some(*pointer_id),
        PointerEvent::OnScroll { .. }
        | PointerEvent::OnPinchGesture { .. }
        | PointerEvent::OnPanGesture { .. }
        | PointerEvent::OnRotationGesture { .. }
        | PointerEvent::OnDoubleTapGesture { .. } => None,
    }
}

fn pointer_position(event: &PointerEvent) -> Option<Vec2> {
    match event {
        PointerEvent::OnMove { position, .. }
        | PointerEvent::OnButton { position, .. }
        | PointerEvent::OnTouch { position, .. } => Some(*position),
        PointerEvent::OnEnter { .. }
        | PointerEvent::OnLeave { .. }
        | PointerEvent::OnScroll { .. }
        | PointerEvent::OnPinchGesture { .. }
        | PointerEvent::OnPanGesture { .. }
        | PointerEvent::OnRotationGesture { .. }
        | PointerEvent::OnDoubleTapGesture { .. } => None,
    }
}

fn apply_trace(event: &mut PointerEvent, trace: Option<PointerEventTrace>) {
    match event {
        PointerEvent::OnMove { trace: slot, .. }
        | PointerEvent::OnEnter { trace: slot, .. }
        | PointerEvent::OnLeave { trace: slot, .. }
        | PointerEvent::OnButton { trace: slot, .. }
        | PointerEvent::OnScroll { trace: slot, .. }
        | PointerEvent::OnTouch { trace: slot, .. }
        | PointerEvent::OnPinchGesture { trace: slot, .. }
        | PointerEvent::OnPanGesture { trace: slot, .. }
        | PointerEvent::OnRotationGesture { trace: slot, .. }
        | PointerEvent::OnDoubleTapGesture { trace: slot, .. } => {
            *slot = trace;
        }
    }
}

fn select_trace_payload(
    config: PointerTraceConfig,
    frame_index: u64,
    window_id: u32,
    pointer_id: Option<u64>,
    full: PointerEventTrace,
) -> Option<PointerEventTrace> {
    if !trace_is_sampled(config, frame_index, window_id, pointer_id) {
        return None;
    }
    match config.level {
        PointerTraceLevel::Off => None,
        PointerTraceLevel::Errors => trace_contains_error(&full).then_some(full),
        PointerTraceLevel::Basic => Some(PointerEventTrace {
            window_id: full.window_id,
            realm_id: full.realm_id,
            target_id: full.target_id,
            connector_id: None,
            source_realm_id: None,
            uv: None,
            hops: Vec::new(),
        }),
        PointerTraceLevel::Full => Some(full),
    }
}

fn trace_contains_error(trace: &PointerEventTrace) -> bool {
    trace.hops.iter().any(|hop| {
        matches!(
            hop.stage,
            PointerTraceStage::StopStepBudget | PointerTraceStage::StopCycle
        )
    })
}

fn trace_is_sampled(
    config: PointerTraceConfig,
    frame_index: u64,
    window_id: u32,
    pointer_id: Option<u64>,
) -> bool {
    let percent = config.sampling_percent.min(100);
    if percent == 0 {
        return false;
    }
    if percent == 100 {
        return true;
    }
    let seed = frame_index
        ^ window_id as u64
        ^ pointer_id
            .unwrap_or_default()
            .wrapping_mul(11400714819323198485);
    seed % 100 < percent as u64
}

fn resolve_captured_connector(
    universal: &UniversalState,
    window_id: u32,
    pointer_id: u64,
) -> Option<InputCapture> {
    universal
        .input_routing
        .captures
        .get(&(window_id, pointer_id))
        .copied()
}

fn resolve_hit_connector(
    connectors: Option<&Vec<ConnectorHit>>,
    position: Vec2,
    window_size: Option<UVec2>,
) -> Option<HitResult> {
    let connectors = connectors?;
    let target_size = window_size.unwrap_or_else(|| UVec2::new(1, 1));
    for connector in connectors {
        if connector.state.input_flags & INPUT_FLAG_RAYCAST != 0 {
            if hit_test_connector(
                position,
                connector.state.rect,
                connector.state.clip,
                connector.source_size,
                target_size,
            ) {
                let uv = resolve_connector_uv_from_sizes(
                    connector.state.rect,
                    connector.state.clip,
                    position,
                    connector.source_size,
                    target_size,
                );
                return Some(HitResult {
                    connector_id: connector.id,
                    uv,
                });
            }
            continue;
        }
        if hit_test_connector(
            position,
            connector.state.rect,
            connector.state.clip,
            connector.source_size,
            target_size,
        ) {
            return Some(HitResult {
                connector_id: connector.id,
                uv: None,
            });
        }
    }
    None
}

fn resolve_connector_uv(
    universal: &UniversalState,
    connector: &ConnectorState,
    position: Vec2,
    target_size: UVec2,
) -> Option<Vec2> {
    let source_size = universal
        .surfaces
        .entries
        .get(&connector.source_surface)
        .map(|entry| entry.value.size)?;
    resolve_connector_uv_from_sizes(
        connector.rect,
        connector.clip,
        position,
        source_size,
        target_size,
    )
}

fn resolve_connector_uv_from_sizes(
    rect: glam::Vec4,
    clip: Option<glam::Vec4>,
    position: Vec2,
    source_size: UVec2,
    target_size: UVec2,
) -> Option<Vec2> {
    let (viewport, _) = resolve_overlay_geometry(rect, clip, source_size, target_size)?;
    let u = ((position.x - viewport.x) / viewport.z.max(1.0)).clamp(0.0, 1.0);
    let v = ((position.y - viewport.y) / viewport.w.max(1.0)).clamp(0.0, 1.0);
    Some(Vec2::new(u, v))
}

fn update_capture_state(
    universal: &mut UniversalState,
    window_id: u32,
    pointer_id: u64,
    connector_id: Option<ConnectorId>,
    target_id: Option<TargetId>,
    event: &PointerEvent,
) {
    match event {
        PointerEvent::OnButton {
            state: ElementState::Pressed,
            ..
        } => {
            if let Some(connector_id) = connector_id {
                universal.input_routing.captures.insert(
                    (window_id, pointer_id),
                    InputCapture {
                        connector_id,
                        target_id,
                    },
                );
            }
        }
        PointerEvent::OnButton {
            state: ElementState::Released,
            ..
        } => {
            universal
                .input_routing
                .captures
                .remove(&(window_id, pointer_id));
        }
        PointerEvent::OnTouch { phase, .. } => match phase {
            TouchPhase::Started | TouchPhase::Moved => {
                if let Some(connector_id) = connector_id {
                    universal.input_routing.captures.insert(
                        (window_id, pointer_id),
                        InputCapture {
                            connector_id,
                            target_id,
                        },
                    );
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                universal
                    .input_routing
                    .captures
                    .remove(&(window_id, pointer_id));
            }
        },
        _ => {}
    }
}

fn update_focus_state(
    universal: &mut UniversalState,
    window_id: u32,
    target_id: Option<TargetId>,
    event: &PointerEvent,
) {
    match event {
        PointerEvent::OnButton {
            state: ElementState::Pressed,
            ..
        }
        | PointerEvent::OnTouch {
            phase: TouchPhase::Started,
            ..
        } => {
            if let Some(target_id) = target_id {
                universal
                    .input_routing
                    .focus_targets
                    .insert(window_id, target_id);
            }
        }
        PointerEvent::OnButton {
            state: ElementState::Released,
            ..
        }
        | PointerEvent::OnTouch {
            phase: TouchPhase::Ended | TouchPhase::Cancelled,
            ..
        } => {
            universal.input_routing.focus_targets.remove(&window_id);
        }
        _ => {}
    }
}

fn resolve_focus_target(universal: &UniversalState, window_id: u32) -> Option<TargetId> {
    universal
        .input_routing
        .focus_targets
        .get(&window_id)
        .copied()
}

fn resolve_connector_for_target(
    connectors: Option<&Vec<ConnectorHit>>,
    target_id: TargetId,
) -> Option<ConnectorId> {
    let connectors = connectors?;
    for connector in connectors {
        if connector.target_id == Some(target_id) {
            return Some(connector.id);
        }
    }
    None
}

#[derive(Debug, Clone)]
struct ConnectorHit {
    id: ConnectorId,
    state: ConnectorState,
    source_size: UVec2,
    target_id: Option<TargetId>,
    target_rank: i32,
}

#[derive(Debug, Clone, Copy)]
struct HitResult {
    connector_id: ConnectorId,
    uv: Option<Vec2>,
}

fn realm_surface_size(universal: &UniversalState, realm_id: RealmId) -> Option<UVec2> {
    let realm = universal.realms.entries.get(&realm_id)?;
    let surface_id = realm.value.output_surface?;
    let surface = universal.surfaces.entries.get(&surface_id)?;
    Some(surface.value.size)
}

fn quantize_uv(value: f32) -> u16 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 1024.0).round() as u16
}

fn hit_test_connector(
    position: Vec2,
    rect: glam::Vec4,
    clip: Option<glam::Vec4>,
    source_size: glam::UVec2,
    target_size: UVec2,
) -> bool {
    let Some((viewport, clip_rect)) =
        resolve_overlay_geometry(rect, clip, source_size, target_size)
    else {
        return false;
    };

    let inside_viewport = position.x >= viewport.x
        && position.y >= viewport.y
        && position.x <= viewport.x + viewport.z
        && position.y <= viewport.y + viewport.w;
    let inside_clip = position.x >= clip_rect.x
        && position.y >= clip_rect.y
        && position.x <= clip_rect.x + clip_rect.z
        && position.y <= clip_rect.y + clip_rect.w;
    inside_viewport && inside_clip
}

fn resolve_overlay_geometry(
    rect: glam::Vec4,
    clip: Option<glam::Vec4>,
    source_size: glam::UVec2,
    target_size: UVec2,
) -> Option<(glam::Vec4, glam::Vec4)> {
    if rect.z <= 0.0 || rect.w <= 0.0 {
        return None;
    }

    let source_width = source_size.x.max(1) as f32;
    let source_height = source_size.y.max(1) as f32;
    let scale = rect.w / source_height;
    let draw_width = (source_width * scale).max(1.0);

    let mut viewport_x = rect.x + (rect.z - draw_width) * 0.5;
    let mut viewport_y = rect.y;
    let mut viewport_width = draw_width;
    let mut viewport_height = rect.w.max(1.0);

    if viewport_x < 0.0 {
        viewport_width = (viewport_width + viewport_x).max(0.0);
        viewport_x = 0.0;
    }
    if viewport_y < 0.0 {
        viewport_height = (viewport_height + viewport_y).max(0.0);
        viewport_y = 0.0;
    }

    let max_width = target_size.x as f32 - viewport_x;
    let max_height = target_size.y as f32 - viewport_y;
    if max_width <= 0.0 || max_height <= 0.0 {
        return None;
    }
    viewport_width = viewport_width.min(max_width);
    viewport_height = viewport_height.min(max_height);
    if viewport_width <= 0.0 || viewport_height <= 0.0 {
        return None;
    }

    let viewport = glam::Vec4::new(viewport_x, viewport_y, viewport_width, viewport_height);
    let mut clip_rect = rect;
    if let Some(clip) = clip {
        clip_rect = intersect_rect(clip_rect, clip);
    }
    Some((viewport, clip_rect))
}

fn intersect_rect(a: glam::Vec4, b: glam::Vec4) -> glam::Vec4 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.z).min(b.x + b.z);
    let y2 = (a.y + a.w).min(b.y + b.w);
    glam::Vec4::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
}
