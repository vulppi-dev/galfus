#[cfg(not(target_arch = "wasm32"))]
use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};

#[derive(Debug, Default)]
pub struct PlatformGamepadBackendState {
    #[cfg(not(target_arch = "wasm32"))]
    pub gilrs: Option<Gilrs>,
}

impl PlatformGamepadBackendState {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let gilrs = match Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(error) => {
                log::warn!("Failed to initialize gamepad support: {:?}", error);
                None
            }
        };

        Self {
            #[cfg(not(target_arch = "wasm32"))]
            gilrs,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn drain_gilrs_events(state: &mut PlatformGamepadBackendState) -> Vec<Event> {
    let Some(gilrs) = &mut state.gilrs else {
        return Vec::new();
    };

    let mut events = Vec::new();
    while let Some(event) = gilrs.next_event() {
        events.push(event);
    }
    events
}

#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_gilrs_gamepad_name(
    state: &PlatformGamepadBackendState,
    gamepad_id: GamepadId,
) -> Option<String> {
    state
        .gilrs
        .as_ref()
        .map(|gilrs| gilrs.gamepad(gamepad_id).name().to_owned())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn map_gilrs_button(button: Button) -> u32 {
    match button {
        Button::South => 0,
        Button::East => 1,
        Button::West => 2,
        Button::North => 3,
        Button::LeftTrigger => 4,
        Button::RightTrigger => 5,
        Button::LeftTrigger2 => 6,
        Button::RightTrigger2 => 7,
        Button::Select => 8,
        Button::Start => 9,
        Button::Mode => 10,
        Button::LeftThumb => 11,
        Button::RightThumb => 12,
        Button::DPadUp => 13,
        Button::DPadDown => 14,
        Button::DPadLeft => 15,
        Button::DPadRight => 16,
        _ => 255,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn map_gilrs_axis(axis: Axis) -> u32 {
    match axis {
        Axis::LeftStickX => 0,
        Axis::LeftStickY => 1,
        Axis::RightStickX => 2,
        Axis::RightStickY => 3,
        Axis::LeftZ => 4,
        Axis::RightZ => 5,
        _ => 255,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub type PlatformGamepadEvent = Event;

#[cfg(not(target_arch = "wasm32"))]
pub type PlatformGamepadEventType = EventType;
