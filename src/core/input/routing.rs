use std::cmp::Ordering;
use std::collections::HashMap;

use glam::{UVec2, Vec2};

use crate::core::cmd::EngineEvent;
use crate::core::input::events::{ElementState, PointerEvent, PointerEventTrace, TouchPhase};
use crate::core::realm::{ConnectorId, ConnectorState, InputCapture, RealmId, UniversalState};
use crate::core::state::EngineState;
use crate::core::target::TargetId;

const INPUT_FLAG_RAYCAST: u32 = 1 << 0;

pub fn route_pointer_events(engine_state: &mut EngineState) {
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

    let mut connectors_by_realm: HashMap<RealmId, Vec<ConnectorHit>> = HashMap::new();
    for (connector_id, entry) in engine_state.universal_state.connectors.entries.iter() {
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
                target_id,
                target_rank: rank,
            });
    }

    for connectors in connectors_by_realm.values_mut() {
        connectors.sort_by(|a, b| {
            let rank_cmp = b.target_rank.cmp(&a.target_rank);
            if rank_cmp == Ordering::Equal {
                b.state.z_index.cmp(&a.state.z_index)
            } else {
                rank_cmp
            }
        });
    }

    for event in engine_state.event_queue.iter_mut() {
        let EngineEvent::Pointer(pointer_event) = event else {
            continue;
        };

        let window_id = pointer_window_id(pointer_event);
        let Some((realm_id, _surface_id)) = realm_by_window.get(&window_id).copied() else {
            continue;
        };

        let window_size = engine_state
            .window
            .states
            .get(&window_id)
            .map(|state| state.inner_size);
        let pointer_id = pointer_id(pointer_event);
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

        if let (Some(pointer_id), Some(position)) = (pointer_id, position) {
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
                    capture
                        .target_id
                        .and_then(|target_id| {
                            resolve_connector_for_target(
                                connectors_by_realm.get(&realm_id),
                                target_id,
                            )
                        })
                };
            } else if let Some(hit) = resolve_hit_connector(
                &engine_state.universal_state,
                connectors_by_realm.get(&realm_id),
                position,
                window_size,
            ) {
                connector_id = Some(hit.connector_id);
                uv_override = hit.uv;
            }

            if connector_id.is_none() {
                if let Some(focused_target) = resolve_focus_target(&engine_state.universal_state, window_id)
                {
                    target_id = Some(focused_target);
                    connector_id = resolve_connector_for_target(
                        connectors_by_realm.get(&realm_id),
                        focused_target,
                    );
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
                    uv = uv_override
                        .or_else(|| resolve_connector_uv(&engine_state.universal_state, connector, position));
                }
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

        let trace = PointerEventTrace {
            window_id,
            realm_id: realm_id.0,
            target_id: target_id.map(|id| id.0),
            connector_id: connector_id.map(|id| id.0),
            source_realm_id: source_realm_id.map(|id| id.0),
            uv,
        };

        apply_trace(pointer_event, trace);
    }
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

fn apply_trace(event: &mut PointerEvent, trace: PointerEventTrace) {
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
            *slot = Some(trace);
        }
    }
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
    universal: &UniversalState,
    connectors: Option<&Vec<ConnectorHit>>,
    position: Vec2,
    window_size: Option<UVec2>,
) -> Option<HitResult> {
    let connectors = connectors?;
    for connector in connectors {
        if connector.state.input_flags & INPUT_FLAG_RAYCAST != 0 {
            if let Some(window_size) = window_size {
                if let Some(uv) = normalize_window_uv(position, window_size) {
                    return Some(HitResult {
                        connector_id: connector.id,
                        uv: Some(uv),
                    });
                }
            }
            continue;
        }
        let source_size = universal
            .surfaces
            .entries
            .get(&connector.state.source_surface)
            .map(|entry| entry.value.size);
        let Some(source_size) = source_size else {
            continue;
        };
        if hit_test_connector(
            position,
            connector.state.rect,
            connector.state.clip,
            source_size,
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
) -> Option<Vec2> {
    let source_size = universal
        .surfaces
        .entries
        .get(&connector.source_surface)
        .map(|entry| entry.value.size)?;
    let rect_height = connector.rect.w;
    if rect_height <= 0.0 {
        return None;
    }

    let source_height = source_size.y.max(1) as f32;
    let scale = rect_height / source_height;
    let draw_width = (source_size.x.max(1) as f32 * scale).max(1.0);

    let u = ((position.x - connector.rect.x) / draw_width).clamp(0.0, 1.0);
    let v = ((position.y - connector.rect.y) / rect_height).clamp(0.0, 1.0);
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
                universal
                    .input_routing
                    .captures
                    .insert(
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
            universal.input_routing.captures.remove(&(window_id, pointer_id));
        }
        PointerEvent::OnTouch { phase, .. } => match phase {
            TouchPhase::Started | TouchPhase::Moved => {
                if let Some(connector_id) = connector_id {
                    universal
                        .input_routing
                        .captures
                        .insert(
                            (window_id, pointer_id),
                            InputCapture {
                                connector_id,
                                target_id,
                            },
                        );
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                universal.input_routing.captures.remove(&(window_id, pointer_id));
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
    universal.input_routing.focus_targets.get(&window_id).copied()
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
    target_id: Option<TargetId>,
    target_rank: i32,
}

#[derive(Debug, Clone, Copy)]
struct HitResult {
    connector_id: ConnectorId,
    uv: Option<Vec2>,
}

fn normalize_window_uv(position: Vec2, window_size: UVec2) -> Option<Vec2> {
    let width = window_size.x.max(1) as f32;
    let height = window_size.y.max(1) as f32;
    if position.x < 0.0 || position.y < 0.0 || position.x > width || position.y > height {
        return None;
    }
    let u = (position.x / width).clamp(0.0, 1.0);
    let v = (position.y / height).clamp(0.0, 1.0);
    Some(Vec2::new(u, v))
}

fn hit_test_connector(
    position: Vec2,
    rect: glam::Vec4,
    clip: Option<glam::Vec4>,
    source_size: glam::UVec2,
) -> bool {
    let rect_height = rect.w;
    if rect_height <= 0.0 {
        return false;
    }

    let source_height = source_size.y.max(1) as f32;
    let scale = rect_height / source_height;
    let draw_width = (source_size.x.max(1) as f32 * scale).max(1.0);

    let viewport = glam::Vec4::new(rect.x, rect.y, draw_width, rect_height);
    let mut clip_rect = glam::Vec4::new(rect.x, rect.y, rect.z, rect.w);
    if let Some(clip) = clip {
        clip_rect = intersect_rect(clip_rect, clip);
    }

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

fn intersect_rect(a: glam::Vec4, b: glam::Vec4) -> glam::Vec4 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.z).min(b.x + b.z);
    let y2 = (a.y + a.w).min(b.y + b.w);
    glam::Vec4::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
}
