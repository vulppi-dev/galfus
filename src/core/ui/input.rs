use crate::core::cmd::EngineEvent;
use crate::core::input::events::{ElementState, KeyboardEvent, PointerEvent, ScrollDelta, TouchPhase};
use crate::core::realm::{RealmId, UniversalState};
use crate::core::state::EngineState;
use crate::core::ui::state::UiRealmState;

pub fn process_ui_input(engine: &mut EngineState) {
    let mut pointer_updates: Vec<(RealmId, egui::Event)> = Vec::new();
    let mut modifier_updates: Vec<(RealmId, egui::Modifiers)> = Vec::new();
    let mut focus_updates: Vec<(u32, RealmId)> = Vec::new();

    for event in engine.event_queue.iter() {
        match event {
            EngineEvent::Pointer(pointer_event) => {
                if let Some((realm_id, pos)) = resolve_pointer_realm(engine, pointer_event) {
                    let modifiers = current_modifiers(engine, realm_id);
                    if let Some(pointer_event) =
                        build_pointer_event(pointer_event, pos, modifiers)
                    {
                        pointer_updates.push((realm_id, pointer_event));
                    }

                    if matches!(
                        pointer_event,
                        PointerEvent::OnButton {
                            state: ElementState::Pressed,
                            ..
                        }
                            | PointerEvent::OnTouch {
                                phase: TouchPhase::Started,
                                ..
                            }
                    ) {
                        focus_updates.push((pointer_window_id(pointer_event), realm_id));
                    }
                }
            }
            EngineEvent::Keyboard(keyboard_event) => {
                if let Some((realm_id, modifiers, events)) =
                    build_keyboard_event(engine, keyboard_event)
                {
                    modifier_updates.push((realm_id, modifiers));
                    for event in events {
                        pointer_updates.push((realm_id, event));
                    }
                }
            }
            _ => {}
        }
    }

    for (window_id, realm_id) in focus_updates {
        engine
            .universal_state
            .ui
            .focus_by_window
            .insert(window_id, realm_id);
    }

    for (realm_id, modifiers) in modifier_updates {
        if let Some(realm) = ensure_realm(&mut engine.universal_state.ui, realm_id) {
            realm.modifiers = modifiers;
        }
    }

    for (realm_id, event) in pointer_updates {
        if let Some(realm) = ensure_realm(&mut engine.universal_state.ui, realm_id) {
            realm.push_event(event);
        }
    }
}

fn ensure_realm(ui_state: &mut crate::core::ui::UiState, realm_id: RealmId) -> Option<&mut UiRealmState> {
    ui_state.ensure_realm(realm_id);
    ui_state.realm_mut(realm_id)
}

fn resolve_pointer_realm(
    engine: &EngineState,
    event: &PointerEvent,
) -> Option<(RealmId, egui::Pos2)> {
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

    let pos = if let Some(uv) = trace.uv {
        let size = connector_source_size(&engine.universal_state, trace.connector_id)
            .or_else(|| realm_output_size(&engine.universal_state, realm_id))?;
        egui::pos2(uv.x * size.x as f32, uv.y * size.y as f32)
    } else if let Some(position) = position {
        egui::pos2(position.x, position.y)
    } else {
        return None;
    };

    Some((realm_id, pos))
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
    universal.surfaces.get(surface_id).map(|entry| entry.value.size)
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
        PointerEvent::OnButton {
            button, state, ..
        } => {
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
        _ => None,
    }
}

fn build_keyboard_event(
    engine: &EngineState,
    event: &KeyboardEvent,
) -> Option<(RealmId, egui::Modifiers, Vec<egui::Event>)> {
    let (window_id, modifiers_state) = match event {
        KeyboardEvent::OnInput { window_id, modifiers, .. } => (*window_id, *modifiers),
        KeyboardEvent::OnModifiersChange { window_id, modifiers } => (*window_id, *modifiers),
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
        .copied()?;

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
            if let Some(text) = text {
                if *state == ElementState::Pressed {
                    events.push(egui::Event::Text(text.clone()));
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
