use glam::Vec2;

use crate::{
    ElementState, KeyboardEvent, ModifiersState, PointerEvent, PointerEventTrace, ScrollDelta,
    TouchPhase,
};

pub fn element_state_from_pressed(pressed: bool) -> ElementState {
    if pressed {
        ElementState::Pressed
    } else {
        ElementState::Released
    }
}

pub fn keyboard_input_event(
    window_id: u32,
    key_code: u32,
    state: ElementState,
    location: u32,
    repeat: bool,
    text: Option<String>,
    modifiers: ModifiersState,
) -> KeyboardEvent {
    KeyboardEvent::OnInput {
        window_id,
        key_code,
        state,
        location,
        repeat,
        text,
        modifiers,
    }
}

pub fn keyboard_modifiers_event(window_id: u32, modifiers: ModifiersState) -> KeyboardEvent {
    KeyboardEvent::OnModifiersChange {
        window_id,
        modifiers,
    }
}

pub fn keyboard_ime_enable_event(window_id: u32) -> KeyboardEvent {
    KeyboardEvent::OnImeEnable { window_id }
}

pub fn keyboard_ime_preedit_event(
    window_id: u32,
    text: String,
    cursor_range: Option<(usize, usize)>,
) -> KeyboardEvent {
    KeyboardEvent::OnImePreedit {
        window_id,
        text,
        cursor_range,
    }
}

pub fn keyboard_ime_commit_event(window_id: u32, text: String) -> KeyboardEvent {
    KeyboardEvent::OnImeCommit { window_id, text }
}

pub fn keyboard_ime_disable_event(window_id: u32) -> KeyboardEvent {
    KeyboardEvent::OnImeDisable { window_id }
}

pub fn pointer_move_event(
    window_id: u32,
    pointer_type: u32,
    pointer_id: u64,
    position: Vec2,
    trace: Option<PointerEventTrace>,
) -> PointerEvent {
    PointerEvent::OnMove {
        window_id,
        window_width: None,
        window_height: None,
        pointer_type,
        pointer_id,
        position,
        position_target: None,
        target_width: None,
        target_height: None,
        trace,
    }
}

pub fn pointer_enter_event(
    window_id: u32,
    pointer_type: u32,
    pointer_id: u64,
    trace: Option<PointerEventTrace>,
) -> PointerEvent {
    PointerEvent::OnEnter {
        window_id,
        window_width: None,
        window_height: None,
        pointer_type,
        pointer_id,
        target_width: None,
        target_height: None,
        trace,
    }
}

pub fn pointer_leave_event(
    window_id: u32,
    pointer_type: u32,
    pointer_id: u64,
    trace: Option<PointerEventTrace>,
) -> PointerEvent {
    PointerEvent::OnLeave {
        window_id,
        window_width: None,
        window_height: None,
        pointer_type,
        pointer_id,
        target_width: None,
        target_height: None,
        trace,
    }
}

pub fn pointer_button_event(
    window_id: u32,
    pointer_type: u32,
    pointer_id: u64,
    button: u32,
    state: ElementState,
    position: Vec2,
    trace: Option<PointerEventTrace>,
) -> PointerEvent {
    PointerEvent::OnButton {
        window_id,
        window_width: None,
        window_height: None,
        pointer_type,
        pointer_id,
        button,
        state,
        position,
        position_target: None,
        target_width: None,
        target_height: None,
        trace,
    }
}

pub fn pointer_scroll_event(
    window_id: u32,
    delta: ScrollDelta,
    phase: TouchPhase,
    trace: Option<PointerEventTrace>,
) -> PointerEvent {
    PointerEvent::OnScroll {
        window_id,
        window_width: None,
        window_height: None,
        delta,
        phase,
        target_width: None,
        target_height: None,
        trace,
    }
}

pub fn pointer_touch_event(
    window_id: u32,
    pointer_id: u64,
    phase: TouchPhase,
    position: Vec2,
    pressure: Option<f32>,
    trace: Option<PointerEventTrace>,
) -> PointerEvent {
    PointerEvent::OnTouch {
        window_id,
        window_width: None,
        window_height: None,
        pointer_id,
        phase,
        position,
        position_target: None,
        target_width: None,
        target_height: None,
        pressure,
        trace,
    }
}

pub fn pointer_pinch_gesture_event(
    window_id: u32,
    delta: f64,
    phase: TouchPhase,
    trace: Option<PointerEventTrace>,
) -> PointerEvent {
    PointerEvent::OnPinchGesture {
        window_id,
        window_width: None,
        window_height: None,
        delta,
        phase,
        target_width: None,
        target_height: None,
        trace,
    }
}

pub fn pointer_pan_gesture_event(
    window_id: u32,
    delta: Vec2,
    phase: TouchPhase,
    trace: Option<PointerEventTrace>,
) -> PointerEvent {
    PointerEvent::OnPanGesture {
        window_id,
        window_width: None,
        window_height: None,
        delta,
        phase,
        target_width: None,
        target_height: None,
        trace,
    }
}

pub fn pointer_rotation_gesture_event(
    window_id: u32,
    delta: f32,
    phase: TouchPhase,
    trace: Option<PointerEventTrace>,
) -> PointerEvent {
    PointerEvent::OnRotationGesture {
        window_id,
        window_width: None,
        window_height: None,
        delta,
        phase,
        target_width: None,
        target_height: None,
        trace,
    }
}

pub fn pointer_double_tap_gesture_event(
    window_id: u32,
    trace: Option<PointerEventTrace>,
) -> PointerEvent {
    PointerEvent::OnDoubleTapGesture {
        window_id,
        window_width: None,
        window_height: None,
        target_width: None,
        target_height: None,
        trace,
    }
}
