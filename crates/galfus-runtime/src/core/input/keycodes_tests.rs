use super::{KEY_ESCAPE, KEY_UNIDENTIFIED, KEY_W, map_web_key_code};
#[cfg(not(target_arch = "wasm32"))]
use galfus_platform::map_winit_key_code;

#[test]
fn web_escape_maps_to_canonical_escape() {
    assert_eq!(map_web_key_code("Escape"), KEY_ESCAPE);
}

#[test]
fn web_w_maps_to_canonical_w() {
    assert_eq!(map_web_key_code("KeyW"), KEY_W);
}

#[test]
fn web_unknown_maps_to_unidentified() {
    assert_eq!(map_web_key_code("NotARealCode"), KEY_UNIDENTIFIED);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn desktop_browser_keycode_parity_subset() {
    use crate::core::platform::winit::keyboard::KeyCode as WinitKeyCode;

    let pairs = [
        (WinitKeyCode::KeyW, "KeyW"),
        (WinitKeyCode::Escape, "Escape"),
        (WinitKeyCode::ShiftLeft, "ShiftLeft"),
        (WinitKeyCode::Digit1, "Digit1"),
        (WinitKeyCode::ArrowUp, "ArrowUp"),
        (WinitKeyCode::Numpad0, "Numpad0"),
        (WinitKeyCode::F5, "F5"),
        (WinitKeyCode::BrowserBack, "BrowserBack"),
    ];

    for (winit_code, web_code) in pairs {
        assert_eq!(map_winit_key_code(winit_code), map_web_key_code(web_code));
    }
}
