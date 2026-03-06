use crate::core::cmd::EngineEvent;
use crate::core::input::events::{
    ElementState, KeyboardEvent, PointerEvent, ScrollDelta, TouchPhase,
};
use crate::core::realm::{RealmId, UniversalState};
use crate::core::state::EngineState;
use crate::core::ui::state::UiRealmState;
use crate::core::window::WindowEvent;
use std::collections::HashMap;
use std::time::Instant;

pub fn process_ui_input(engine: &mut EngineState) {
    let input_start = Instant::now();
    let realms_by_window = collect_ui_realms_by_window(engine);
    let mut scratch = std::mem::take(&mut engine.universal_state.ui.input_scratch);
    scratch.pointer_updates.clear();
    scratch.modifier_updates.clear();
    scratch.focus_updates.clear();
    scratch.pointer_pos_updates.clear();

    let pointer_updates = &mut scratch.pointer_updates;
    let modifier_updates = &mut scratch.modifier_updates;
    let focus_updates = &mut scratch.focus_updates;
    let pointer_pos_updates = &mut scratch.pointer_pos_updates;

    for event in engine.event_queue.iter() {
        match event {
            EngineEvent::Pointer(pointer_event) => {
                if let Some((realm_id, document_id, pos)) =
                    resolve_pointer_realm(engine, pointer_event)
                {
                    let modifiers = current_modifiers(engine, realm_id);
                    if matches!(pointer_event, PointerEvent::OnMove { .. }) {
                        let previous = engine
                            .universal_state
                            .ui
                            .realms
                            .get(&realm_id)
                            .and_then(|realm| realm.last_pointer_pos);
                        if let Some(previous) = previous {
                            let delta = pos - previous;
                            if delta != egui::Vec2::ZERO {
                                pointer_updates.push((realm_id, egui::Event::MouseMoved(delta)));
                            }
                        }
                    }
                    if let Some(pointer_event) = build_pointer_event(pointer_event, pos, modifiers)
                    {
                        pointer_updates.push((realm_id, pointer_event));
                    }
                    if matches!(
                        pointer_event,
                        PointerEvent::OnMove { .. }
                            | PointerEvent::OnEnter { .. }
                            | PointerEvent::OnButton { .. }
                            | PointerEvent::OnTouch { .. }
                    ) {
                        pointer_pos_updates.push((realm_id, Some(pos)));
                    }

                    if matches!(
                        pointer_event,
                        PointerEvent::OnButton {
                            state: ElementState::Pressed,
                            ..
                        } | PointerEvent::OnTouch {
                            phase: TouchPhase::Started,
                            ..
                        }
                    ) {
                        let window_id = pointer_window_id(pointer_event);
                        focus_updates.push((window_id, realm_id, document_id));
                        engine
                            .universal_state
                            .ui
                            .capture_by_window
                            .insert(window_id, (realm_id, document_id, 0));
                    } else if matches!(
                        pointer_event,
                        PointerEvent::OnButton {
                            state: ElementState::Released,
                            ..
                        } | PointerEvent::OnTouch {
                            phase: TouchPhase::Ended,
                            ..
                        } | PointerEvent::OnTouch {
                            phase: TouchPhase::Cancelled,
                            ..
                        }
                    ) {
                        let window_id = pointer_window_id(pointer_event);
                        engine
                            .universal_state
                            .ui
                            .capture_by_window
                            .remove(&window_id);
                    }
                }
                if matches!(pointer_event, PointerEvent::OnLeave { .. }) {
                    let window_id = pointer_window_id(pointer_event);
                    if let Some(realm_id) = engine
                        .universal_state
                        .ui
                        .focus_by_window
                        .get(&window_id)
                        .copied()
                    {
                        pointer_pos_updates.push((realm_id, None));
                    }
                }
            }
            EngineEvent::Keyboard(keyboard_event) => {
                if let Some((realm_id, modifiers, events)) =
                    build_keyboard_event(engine, keyboard_event, &realms_by_window)
                {
                    modifier_updates.push((realm_id, modifiers));
                    for event in events {
                        pointer_updates.push((realm_id, event));
                    }
                }
            }
            EngineEvent::Window(WindowEvent::OnFocus { window_id, focused }) => {
                if let Some(realms) = realms_by_window.get(window_id) {
                    for realm_id in realms {
                        pointer_updates.push((*realm_id, egui::Event::WindowFocused(*focused)));
                    }
                } else if let Some(realm_id) = engine
                    .universal_state
                    .ui
                    .focus_by_window
                    .get(window_id)
                    .copied()
                {
                    pointer_updates.push((realm_id, egui::Event::WindowFocused(*focused)));
                }
            }
            EngineEvent::Window(WindowEvent::OnScaleFactorChange {
                window_id,
                scale_factor,
                ..
            }) => {
                let next_ppp = (*scale_factor as f32).max(0.001);
                if let Some(realms) = realms_by_window.get(window_id) {
                    for realm_id in realms {
                        if let Some(realm) = ensure_realm(&mut engine.universal_state.ui, *realm_id)
                        {
                            realm.pixels_per_point = next_ppp;
                        }
                    }
                } else if let Some(realm_id) = engine
                    .universal_state
                    .ui
                    .focus_by_window
                    .get(window_id)
                    .copied()
                    && let Some(realm) = ensure_realm(&mut engine.universal_state.ui, realm_id)
                {
                    realm.pixels_per_point = next_ppp;
                }
            }
            _ => {}
        }
    }

    for (window_id, realm_id, document_id) in focus_updates.drain(..) {
        engine
            .universal_state
            .ui
            .focus_by_window
            .insert(window_id, realm_id);
        engine
            .universal_state
            .ui
            .focus_document_by_window
            .insert(window_id, document_id);
        engine
            .universal_state
            .ui
            .focus_node_by_window
            .insert(window_id, 0);
    }

    for (realm_id, modifiers) in modifier_updates.drain(..) {
        if let Some(realm) = ensure_realm(&mut engine.universal_state.ui, realm_id) {
            realm.modifiers = modifiers;
        }
    }

    for (realm_id, event) in pointer_updates.drain(..) {
        if let Some(realm) = ensure_realm(&mut engine.universal_state.ui, realm_id) {
            realm.push_event(event);
        }
    }
    for (realm_id, pos) in pointer_pos_updates.drain(..) {
        if let Some(realm) = ensure_realm(&mut engine.universal_state.ui, realm_id) {
            realm.last_pointer_pos = pos;
        }
    }

    let input_ms = input_start.elapsed().as_secs_f32() * 1000.0;
    for realm in engine.universal_state.ui.realms.values_mut() {
        realm.profile.input_routing_ms = input_ms;
    }

    engine.universal_state.ui.input_scratch = scratch;
}

fn ensure_realm(
    ui_state: &mut crate::core::ui::UiState,
    realm_id: RealmId,
) -> Option<&mut UiRealmState> {
    ui_state.ensure_realm(realm_id);
    ui_state.realm_mut(realm_id)
}

fn resolve_pointer_realm(
    engine: &EngineState,
    event: &PointerEvent,
) -> Option<(RealmId, u32, egui::Pos2)> {
    let trace = match event {
        PointerEvent::OnMove { trace, .. }
        | PointerEvent::OnEnter { trace, .. }
        | PointerEvent::OnLeave { trace, .. }
        | PointerEvent::OnButton { trace, .. }
        | PointerEvent::OnScroll { trace, .. }
        | PointerEvent::OnTouch { trace, .. }
        | PointerEvent::OnPinchGesture { trace, .. }
        | PointerEvent::OnPanGesture { trace, .. }
        | PointerEvent::OnRotationGesture { trace, .. }
        | PointerEvent::OnDoubleTapGesture { trace, .. } => trace.as_ref(),
    }?;

    let realm_id = trace
        .source_realm_id
        .map(RealmId)
        .unwrap_or(RealmId(trace.realm_id));

    let position = match event {
        PointerEvent::OnMove { position, .. }
        | PointerEvent::OnButton { position, .. }
        | PointerEvent::OnTouch { position, .. } => Some(*position),
        _ => engine
            .window
            .cursor_positions
            .get(&trace.window_id)
            .copied(),
    };

    let (pos, realm_size) = if let Some(uv) = trace.uv {
        let size = if trace.source_realm_id.is_some() {
            realm_output_size(&engine.universal_state, realm_id)
                .or_else(|| connector_source_size(&engine.universal_state, trace.connector_id))
        } else {
            connector_source_size(&engine.universal_state, trace.connector_id)
                .or_else(|| realm_output_size(&engine.universal_state, realm_id))
        }?;
        (egui::pos2(uv.x * size.x as f32, uv.y * size.y as f32), size)
    } else if let Some(position) = position {
        let size = realm_output_size(&engine.universal_state, realm_id)
            .or_else(|| connector_source_size(&engine.universal_state, trace.connector_id))
            .unwrap_or(glam::UVec2::new(1, 1));
        (egui::pos2(position.x, position.y), size)
    } else {
        return None;
    };

    let document_id = hit_test_ui_document(&engine.universal_state.ui, realm_id, pos, realm_size)?;
    Some((realm_id, document_id, pos))
}

fn hit_test_ui_document(
    ui_state: &crate::core::ui::UiState,
    realm_id: RealmId,
    pos: egui::Pos2,
    realm_size: glam::UVec2,
) -> Option<u32> {
    let mut best: Option<(i32, u32)> = None;
    for document in ui_state.documents.values() {
        if document.realm_id != realm_id {
            continue;
        }
        let rect = resolve_document_rect(document.rect, realm_size);
        if !rect.contains(pos) {
            continue;
        }
        let z = document
            .root_children
            .iter()
            .filter_map(|node_id| {
                document
                    .nodes
                    .get(node_id)
                    .and_then(|entry| entry.node.z_index)
            })
            .max()
            .unwrap_or(0);
        let key = (z, document.document_id);
        match best {
            Some(current) if key <= current => {}
            _ => best = Some(key),
        }
    }
    best.map(|(_, document_id)| document_id)
}

fn resolve_document_rect(rect: glam::Vec4, realm_size: glam::UVec2) -> egui::Rect {
    let max_w = realm_size.x.max(1) as f32;
    let max_h = realm_size.y.max(1) as f32;
    let x = rect.x.max(0.0).min(max_w);
    let y = rect.y.max(0.0).min(max_h);
    let mut w = rect.z;
    let mut h = rect.w;
    if w <= 0.0 {
        w = (max_w - x).max(1.0);
    }
    if h <= 0.0 {
        h = (max_h - y).max(1.0);
    }
    let clamped_w = w.max(1.0).min((max_w - x).max(1.0));
    let clamped_h = h.max(1.0).min((max_h - y).max(1.0));
    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(clamped_w, clamped_h))
}

fn connector_source_size(
    universal: &UniversalState,
    connector_id: Option<u32>,
) -> Option<glam::UVec2> {
    let connector_id = connector_id.map(crate::core::realm::ConnectorId)?;
    let connector = universal.connectors.entries.get(&connector_id)?;
    universal
        .surfaces
        .entries
        .get(&connector.value.source_surface)
        .map(|entry| entry.value.size)
}

fn realm_output_size(universal: &UniversalState, realm_id: RealmId) -> Option<glam::UVec2> {
    let realm = universal.realms.get(realm_id)?;
    let surface_id = realm.value.output_surface?;
    universal
        .surfaces
        .get(surface_id)
        .map(|entry| entry.value.size)
}

fn build_pointer_event(
    event: &PointerEvent,
    pos: egui::Pos2,
    modifiers: egui::Modifiers,
) -> Option<egui::Event> {
    match event {
        PointerEvent::OnMove { .. } => Some(egui::Event::PointerMoved(pos)),
        PointerEvent::OnEnter { .. } => Some(egui::Event::PointerMoved(pos)),
        PointerEvent::OnLeave { .. } => Some(egui::Event::PointerGone),
        PointerEvent::OnButton { button, state, .. } => {
            let button = pointer_button(*button)?;
            Some(egui::Event::PointerButton {
                pos,
                button,
                pressed: *state == ElementState::Pressed,
                modifiers,
            })
        }
        PointerEvent::OnScroll { delta, .. } => Some(egui::Event::MouseWheel {
            unit: match delta {
                ScrollDelta::Line(_) => egui::MouseWheelUnit::Line,
                ScrollDelta::Pixel(_) => egui::MouseWheelUnit::Point,
            },
            delta: match delta {
                ScrollDelta::Line(value) => egui::vec2(value.x, value.y),
                ScrollDelta::Pixel(value) => egui::vec2(value.x, value.y),
            },
            modifiers,
        }),
        PointerEvent::OnTouch {
            pointer_id,
            phase,
            pressure,
            ..
        } => Some(egui::Event::Touch {
            device_id: egui::TouchDeviceId(0),
            id: egui::TouchId(*pointer_id),
            phase: match phase {
                TouchPhase::Started => egui::TouchPhase::Start,
                TouchPhase::Moved => egui::TouchPhase::Move,
                TouchPhase::Ended => egui::TouchPhase::End,
                TouchPhase::Cancelled => egui::TouchPhase::Cancel,
            },
            pos,
            force: *pressure,
        }),
        PointerEvent::OnPinchGesture { delta, .. } => Some(egui::Event::Zoom(*delta as f32)),
        PointerEvent::OnPanGesture { delta, .. } => Some(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Point,
            delta: egui::vec2(delta.x, delta.y),
            modifiers,
        }),
        PointerEvent::OnRotationGesture { delta, .. } => {
            Some(egui::Event::Zoom((1.0 + (*delta * 0.01)).clamp(0.5, 2.0)))
        }
        _ => None,
    }
}

fn build_keyboard_event(
    engine: &EngineState,
    event: &KeyboardEvent,
    realms_by_window: &HashMap<u32, Vec<RealmId>>,
) -> Option<(RealmId, egui::Modifiers, Vec<egui::Event>)> {
    let (window_id, modifiers_state) = match event {
        KeyboardEvent::OnInput {
            window_id,
            modifiers,
            ..
        } => (*window_id, *modifiers),
        KeyboardEvent::OnModifiersChange {
            window_id,
            modifiers,
        } => (*window_id, *modifiers),
        KeyboardEvent::OnImeEnable { window_id }
        | KeyboardEvent::OnImePreedit { window_id, .. }
        | KeyboardEvent::OnImeCommit { window_id, .. }
        | KeyboardEvent::OnImeDisable { window_id } => (*window_id, Default::default()),
    };

    let realm_id = engine
        .universal_state
        .ui
        .focus_by_window
        .get(&window_id)
        .copied()
        .or_else(|| {
            realms_by_window
                .get(&window_id)
                .and_then(|realms| realms.first().copied())
        })?;

    let modifiers = modifiers_from_state(modifiers_state);
    let events = match event {
        KeyboardEvent::OnModifiersChange { .. } => Vec::new(),
        KeyboardEvent::OnInput {
            key_code,
            state,
            repeat,
            text,
            ..
        } => {
            let key = map_key_code(*key_code);
            let mut events = Vec::new();
            if *state == ElementState::Pressed {
                if let Some(key) = key {
                    if modifiers.command && key == egui::Key::C {
                        events.push(egui::Event::Copy);
                    } else if modifiers.command && key == egui::Key::X {
                        events.push(egui::Event::Cut);
                    }
                }
                if let Some(text) = text {
                    if !modifiers.command && !modifiers.ctrl && !modifiers.alt {
                        events.push(egui::Event::Text(text.clone()));
                    }
                }
            }
            if let Some(key) = key {
                events.push(egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: *state == ElementState::Pressed,
                    repeat: *repeat,
                    modifiers,
                });
            }
            events
        }
        KeyboardEvent::OnImeEnable { .. } => vec![egui::Event::Ime(egui::ImeEvent::Enabled)],
        KeyboardEvent::OnImePreedit { text, .. } => {
            vec![egui::Event::Ime(egui::ImeEvent::Preedit(text.clone()))]
        }
        KeyboardEvent::OnImeCommit { text, .. } => {
            vec![egui::Event::Ime(egui::ImeEvent::Commit(text.clone()))]
        }
        KeyboardEvent::OnImeDisable { .. } => vec![egui::Event::Ime(egui::ImeEvent::Disabled)],
    };

    Some((realm_id, modifiers, events))
}

fn collect_ui_realms_by_window(engine: &EngineState) -> HashMap<u32, Vec<RealmId>> {
    let mut map: HashMap<u32, Vec<RealmId>> = HashMap::new();
    for ((layer_realm_id, layer_target_id), _layer) in &engine.universal_state.target_layers.entries
    {
        let Some(target) = engine.universal_state.targets.entries.get(layer_target_id) else {
            continue;
        };
        let Some(window_id) = target.window_id else {
            continue;
        };
        let realm_id = RealmId(*layer_realm_id);
        let is_ui_realm = engine
            .universal_state
            .realms
            .entries
            .get(&realm_id)
            .map(|entry| entry.value.kind == crate::core::realm::RealmKind::TwoD)
            .unwrap_or(false);
        if !is_ui_realm {
            continue;
        }
        map.entry(window_id).or_default().push(realm_id);
    }

    for (window_id, realm_id) in &engine.universal_state.host_realm_index {
        let is_ui_realm = engine
            .universal_state
            .realms
            .entries
            .get(realm_id)
            .map(|entry| entry.value.kind == crate::core::realm::RealmKind::TwoD)
            .unwrap_or(false);
        if is_ui_realm {
            map.entry(*window_id).or_default().push(*realm_id);
        }
    }

    for realms in map.values_mut() {
        realms.sort_by_key(|id| id.0);
        realms.dedup();
    }
    map
}

fn modifiers_from_state(state: crate::core::input::events::ModifiersState) -> egui::Modifiers {
    egui::Modifiers {
        alt: state.alt,
        ctrl: state.ctrl,
        shift: state.shift,
        mac_cmd: state.meta,
        command: state.ctrl || state.meta,
    }
}

fn current_modifiers(engine: &EngineState, realm_id: RealmId) -> egui::Modifiers {
    engine
        .universal_state
        .ui
        .realms
        .get(&realm_id)
        .map(|realm| realm.modifiers)
        .unwrap_or_default()
}

fn pointer_button(button: u32) -> Option<egui::PointerButton> {
    match button {
        0 => Some(egui::PointerButton::Primary),
        1 => Some(egui::PointerButton::Secondary),
        2 => Some(egui::PointerButton::Middle),
        3 => Some(egui::PointerButton::Extra1),
        4 => Some(egui::PointerButton::Extra2),
        _ => None,
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

fn map_key_code(code: u32) -> Option<egui::Key> {
    match code {
        0 => Some(egui::Key::Backtick),
        1 => Some(egui::Key::Backslash),
        2 => Some(egui::Key::OpenBracket),
        3 => Some(egui::Key::CloseBracket),
        4 => Some(egui::Key::Comma),
        5 => Some(egui::Key::Num0),
        6 => Some(egui::Key::Num1),
        7 => Some(egui::Key::Num2),
        8 => Some(egui::Key::Num3),
        9 => Some(egui::Key::Num4),
        10 => Some(egui::Key::Num5),
        11 => Some(egui::Key::Num6),
        12 => Some(egui::Key::Num7),
        13 => Some(egui::Key::Num8),
        14 => Some(egui::Key::Num9),
        15 => Some(egui::Key::Equals),
        45 => Some(egui::Key::Minus),
        46 => Some(egui::Key::Period),
        47 => Some(egui::Key::Quote),
        48 => Some(egui::Key::Semicolon),
        49 => Some(egui::Key::Slash),
        52 => Some(egui::Key::Backspace),
        57 => Some(egui::Key::Enter),
        62 => Some(egui::Key::Space),
        63 => Some(egui::Key::Tab),
        64 => Some(egui::Key::Delete),
        65 => Some(egui::Key::End),
        67 => Some(egui::Key::Home),
        69 => Some(egui::Key::PageDown),
        70 => Some(egui::Key::PageUp),
        71 => Some(egui::Key::ArrowDown),
        72 => Some(egui::Key::ArrowLeft),
        73 => Some(egui::Key::ArrowRight),
        74 => Some(egui::Key::ArrowUp),
        106 => Some(egui::Key::Escape),
        19 => Some(egui::Key::A),
        20 => Some(egui::Key::B),
        21 => Some(egui::Key::C),
        22 => Some(egui::Key::D),
        23 => Some(egui::Key::E),
        24 => Some(egui::Key::F),
        25 => Some(egui::Key::G),
        26 => Some(egui::Key::H),
        27 => Some(egui::Key::I),
        28 => Some(egui::Key::J),
        29 => Some(egui::Key::K),
        30 => Some(egui::Key::L),
        31 => Some(egui::Key::M),
        32 => Some(egui::Key::N),
        33 => Some(egui::Key::O),
        34 => Some(egui::Key::P),
        35 => Some(egui::Key::Q),
        36 => Some(egui::Key::R),
        37 => Some(egui::Key::S),
        38 => Some(egui::Key::T),
        39 => Some(egui::Key::U),
        40 => Some(egui::Key::V),
        41 => Some(egui::Key::W),
        42 => Some(egui::Key::X),
        43 => Some(egui::Key::Y),
        44 => Some(egui::Key::Z),
        _ => None,
    }
}
