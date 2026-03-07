use super::common::TouchPhase;

use crate::core::input::keycodes::map_winit_key_code;
use crate::core::platform::winit;

/// Convert winit TouchPhase to our TouchPhase
pub fn convert_touch_phase(phase: winit::event::TouchPhase) -> TouchPhase {
    match phase {
        winit::event::TouchPhase::Started => TouchPhase::Started,
        winit::event::TouchPhase::Moved => TouchPhase::Moved,
        winit::event::TouchPhase::Ended => TouchPhase::Ended,
        winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

/// Convert winit MouseButton to u32
pub fn convert_mouse_button(button: winit::event::MouseButton) -> u32 {
    match button {
        winit::event::MouseButton::Left => 0,
        winit::event::MouseButton::Right => 1,
        winit::event::MouseButton::Middle => 2,
        winit::event::MouseButton::Back => 3,
        winit::event::MouseButton::Forward => 4,
        winit::event::MouseButton::Other(id) => id as u32,
    }
}

/// Convert winit KeyLocation to u32
pub fn convert_key_location(location: winit::keyboard::KeyLocation) -> u32 {
    match location {
        winit::keyboard::KeyLocation::Standard => 0,
        winit::keyboard::KeyLocation::Left => 1,
        winit::keyboard::KeyLocation::Right => 2,
        winit::keyboard::KeyLocation::Numpad => 3,
    }
}

/// Convert winit PhysicalKey to u32
pub fn convert_key_code(physical_key: &winit::keyboard::PhysicalKey) -> u32 {
    use winit::keyboard::PhysicalKey;

    match physical_key {
        PhysicalKey::Code(code) => map_winit_key_code(*code),
        PhysicalKey::Unidentified(_) => 255,
    }
}
