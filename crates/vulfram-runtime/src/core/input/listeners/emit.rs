use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::core::cmd::EngineEvent;
use crate::core::input::events::{KeyboardEvent, PointerEvent};
use crate::core::state::EngineState;
use crate::core::system::SystemEvent;

use super::model::InputTargetListenerConfig;

pub fn emit_target_listener_events(engine: &mut EngineState) {
    let source_events = engine.runtime.cloned_events();
    for event in source_events {
        match event {
            EngineEvent::Pointer(pointer_event) => {
                emit_pointer_listener_events(engine, &pointer_event)
            }
            EngineEvent::Keyboard(keyboard_event) => {
                emit_keyboard_listener_events(engine, &keyboard_event)
            }
            _ => {}
        }
    }
}

fn emit_pointer_listener_events(engine: &mut EngineState, event: &PointerEvent) {
    let (
        event_type,
        target_id,
        window_id,
        window_width,
        window_height,
        pointer_id,
        position_global,
        position_target,
        target_width,
        target_height,
    ) = match event {
        PointerEvent::OnMove {
            trace,
            window_id,
            window_width,
            window_height,
            pointer_id,
            position,
            position_target,
            target_width,
            target_height,
            ..
        } => (
            "pointer-move",
            trace.as_ref().and_then(|trace| trace.target_id),
            Some(*window_id),
            *window_width,
            *window_height,
            Some(*pointer_id),
            Some(*position),
            *position_target,
            *target_width,
            *target_height,
        ),
        PointerEvent::OnButton {
            trace,
            window_id,
            window_width,
            window_height,
            pointer_id,
            position,
            position_target,
            target_width,
            target_height,
            ..
        } => (
            "pointer-button",
            trace.as_ref().and_then(|trace| trace.target_id),
            Some(*window_id),
            *window_width,
            *window_height,
            Some(*pointer_id),
            Some(*position),
            *position_target,
            *target_width,
            *target_height,
        ),
        PointerEvent::OnTouch {
            trace,
            window_id,
            window_width,
            window_height,
            pointer_id,
            position,
            position_target,
            target_width,
            target_height,
            ..
        } => (
            "pointer-touch",
            trace.as_ref().and_then(|trace| trace.target_id),
            Some(*window_id),
            *window_width,
            *window_height,
            Some(*pointer_id),
            Some(*position),
            *position_target,
            *target_width,
            *target_height,
        ),
        PointerEvent::OnScroll {
            trace,
            window_id,
            window_width,
            window_height,
            target_width,
            target_height,
            ..
        } => (
            "pointer-scroll",
            trace.as_ref().and_then(|trace| trace.target_id),
            Some(*window_id),
            *window_width,
            *window_height,
            None,
            None,
            None,
            *target_width,
            *target_height,
        ),
        PointerEvent::OnEnter {
            trace,
            window_id,
            window_width,
            window_height,
            pointer_id,
            target_width,
            target_height,
            ..
        } => (
            "pointer-enter",
            trace.as_ref().and_then(|trace| trace.target_id),
            Some(*window_id),
            *window_width,
            *window_height,
            Some(*pointer_id),
            None,
            None,
            *target_width,
            *target_height,
        ),
        PointerEvent::OnLeave {
            trace,
            window_id,
            window_width,
            window_height,
            pointer_id,
            target_width,
            target_height,
            ..
        } => (
            "pointer-leave",
            trace.as_ref().and_then(|trace| trace.target_id),
            Some(*window_id),
            *window_width,
            *window_height,
            Some(*pointer_id),
            None,
            None,
            *target_width,
            *target_height,
        ),
        PointerEvent::OnPinchGesture {
            trace,
            window_id,
            window_width,
            window_height,
            target_width,
            target_height,
            ..
        } => (
            "pointer-pinch",
            trace.as_ref().and_then(|trace| trace.target_id),
            Some(*window_id),
            *window_width,
            *window_height,
            None,
            None,
            None,
            *target_width,
            *target_height,
        ),
        PointerEvent::OnPanGesture {
            trace,
            window_id,
            window_width,
            window_height,
            target_width,
            target_height,
            ..
        } => (
            "pointer-pan",
            trace.as_ref().and_then(|trace| trace.target_id),
            Some(*window_id),
            *window_width,
            *window_height,
            None,
            None,
            None,
            *target_width,
            *target_height,
        ),
        PointerEvent::OnRotationGesture {
            trace,
            window_id,
            window_width,
            window_height,
            target_width,
            target_height,
            ..
        } => (
            "pointer-rotation",
            trace.as_ref().and_then(|trace| trace.target_id),
            Some(*window_id),
            *window_width,
            *window_height,
            None,
            None,
            None,
            *target_width,
            *target_height,
        ),
        PointerEvent::OnDoubleTapGesture {
            trace,
            window_id,
            window_width,
            window_height,
            target_width,
            target_height,
            ..
        } => (
            "pointer-double-tap",
            trace.as_ref().and_then(|trace| trace.target_id),
            Some(*window_id),
            *window_width,
            *window_height,
            None,
            None,
            None,
            *target_width,
            *target_height,
        ),
    };
    let Some(target_id) = target_id else {
        return;
    };

    let listeners = engine
        .universal_state
        .interaction
        .target_listeners
        .listeners_for_target(target_id);
    for listener in listeners {
        if !listener_matches(&listener, event_type, engine.runtime.frame_index()) {
            continue;
        }
        engine
            .runtime
            .push_event(EngineEvent::System(SystemEvent::InputTargetListenerEvent {
                listener_id: listener.listener_id,
                target_id,
                event_type: event_type.to_string(),
                window_id,
                window_width,
                window_height,
                pointer_id,
                position_global,
                position_target,
                target_width,
                target_height,
                key_code: None,
                key_state: None,
            }));
    }
}

fn emit_keyboard_listener_events(engine: &mut EngineState, event: &KeyboardEvent) {
    let (event_type, window_id, key_code, key_state) = match event {
        KeyboardEvent::OnInput {
            window_id,
            key_code,
            state,
            ..
        } => (
            "keyboard-input",
            Some(*window_id),
            Some(*key_code),
            Some(*state),
        ),
        KeyboardEvent::OnModifiersChange { window_id, .. } => {
            ("keyboard-modifiers", Some(*window_id), None, None)
        }
        KeyboardEvent::OnImeEnable { window_id } => {
            ("keyboard-ime-enable", Some(*window_id), None, None)
        }
        KeyboardEvent::OnImePreedit { window_id, .. } => {
            ("keyboard-ime-preedit", Some(*window_id), None, None)
        }
        KeyboardEvent::OnImeCommit { window_id, .. } => {
            ("keyboard-ime-commit", Some(*window_id), None, None)
        }
        KeyboardEvent::OnImeDisable { window_id } => {
            ("keyboard-ime-disable", Some(*window_id), None, None)
        }
    };

    let Some(window_id) = window_id else {
        return;
    };
    let Some(target_id) = engine
        .universal_state
        .interaction
        .input_routing
        .focus_targets
        .get(&window_id)
        .copied()
    else {
        return;
    };

    let listeners = engine
        .universal_state
        .interaction
        .target_listeners
        .listeners_for_target(target_id.0);
    for listener in listeners {
        if !listener_matches(&listener, event_type, engine.runtime.frame_index()) {
            continue;
        }
        engine
            .runtime
            .push_event(EngineEvent::System(SystemEvent::InputTargetListenerEvent {
                listener_id: listener.listener_id,
                target_id: target_id.0,
                event_type: event_type.to_string(),
                window_id: Some(window_id),
                window_width: None,
                window_height: None,
                pointer_id: None,
                position_global: None,
                position_target: None,
                target_width: None,
                target_height: None,
                key_code,
                key_state,
            }));
    }
}

fn listener_matches(
    listener: &InputTargetListenerConfig,
    event_type: &str,
    frame_index: u64,
) -> bool {
    if !listener.enabled {
        return false;
    }
    if !listener.events.is_empty()
        && !listener
            .events
            .iter()
            .any(|configured| configured == event_type)
    {
        return false;
    }
    let sample = listener.sample_percent.min(100);
    if sample == 0 {
        return false;
    }
    if sample == 100 {
        return true;
    }
    let mut hasher = DefaultHasher::new();
    listener.listener_id.hash(&mut hasher);
    listener.target_id.hash(&mut hasher);
    event_type.hash(&mut hasher);
    frame_index.hash(&mut hasher);
    (hasher.finish() % 100) < sample as u64
}
