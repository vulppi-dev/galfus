pub const KEY_W: u32 = 41;
pub const KEY_ESCAPE: u32 = 106;
#[cfg(any(feature = "wasm", test))]
pub const KEY_UNIDENTIFIED: u32 = 255;

#[cfg(not(feature = "wasm"))]
pub fn map_winit_key_code(code: crate::core::platform::winit::keyboard::KeyCode) -> u32 {
    use crate::core::platform::winit::keyboard::KeyCode as WKeyCode;

    match code {
        // Writing System Keys (0-50)
        WKeyCode::Backquote => 0,
        WKeyCode::Backslash => 1,
        WKeyCode::BracketLeft => 2,
        WKeyCode::BracketRight => 3,
        WKeyCode::Comma => 4,
        WKeyCode::Digit0 => 5,
        WKeyCode::Digit1 => 6,
        WKeyCode::Digit2 => 7,
        WKeyCode::Digit3 => 8,
        WKeyCode::Digit4 => 9,
        WKeyCode::Digit5 => 10,
        WKeyCode::Digit6 => 11,
        WKeyCode::Digit7 => 12,
        WKeyCode::Digit8 => 13,
        WKeyCode::Digit9 => 14,
        WKeyCode::Equal => 15,
        WKeyCode::IntlBackslash => 16,
        WKeyCode::IntlRo => 17,
        WKeyCode::IntlYen => 18,
        WKeyCode::KeyA => 19,
        WKeyCode::KeyB => 20,
        WKeyCode::KeyC => 21,
        WKeyCode::KeyD => 22,
        WKeyCode::KeyE => 23,
        WKeyCode::KeyF => 24,
        WKeyCode::KeyG => 25,
        WKeyCode::KeyH => 26,
        WKeyCode::KeyI => 27,
        WKeyCode::KeyJ => 28,
        WKeyCode::KeyK => 29,
        WKeyCode::KeyL => 30,
        WKeyCode::KeyM => 31,
        WKeyCode::KeyN => 32,
        WKeyCode::KeyO => 33,
        WKeyCode::KeyP => 34,
        WKeyCode::KeyQ => 35,
        WKeyCode::KeyR => 36,
        WKeyCode::KeyS => 37,
        WKeyCode::KeyT => 38,
        WKeyCode::KeyU => 39,
        WKeyCode::KeyV => 40,
        WKeyCode::KeyW => 41,
        WKeyCode::KeyX => 42,
        WKeyCode::KeyY => 43,
        WKeyCode::KeyZ => 44,
        WKeyCode::Minus => 45,
        WKeyCode::Period => 46,
        WKeyCode::Quote => 47,
        WKeyCode::Semicolon => 48,
        WKeyCode::Slash => 49,

        // Functional Keys (50-63)
        WKeyCode::AltLeft => 50,
        WKeyCode::AltRight => 51,
        WKeyCode::Backspace => 52,
        WKeyCode::CapsLock => 53,
        WKeyCode::ContextMenu => 54,
        WKeyCode::ControlLeft => 55,
        WKeyCode::ControlRight => 56,
        WKeyCode::Enter => 57,
        WKeyCode::SuperLeft => 58,
        WKeyCode::SuperRight => 59,
        WKeyCode::ShiftLeft => 60,
        WKeyCode::ShiftRight => 61,
        WKeyCode::Space => 62,
        WKeyCode::Tab => 63,

        // Control Keys (64-70)
        WKeyCode::Delete => 64,
        WKeyCode::End => 65,
        WKeyCode::Help => 66,
        WKeyCode::Home => 67,
        WKeyCode::Insert => 68,
        WKeyCode::PageDown => 69,
        WKeyCode::PageUp => 70,

        // Arrow Keys (71-74)
        WKeyCode::ArrowDown => 71,
        WKeyCode::ArrowLeft => 72,
        WKeyCode::ArrowRight => 73,
        WKeyCode::ArrowUp => 74,

        // Numpad Keys (75-105)
        WKeyCode::NumLock => 75,
        WKeyCode::Numpad0 => 76,
        WKeyCode::Numpad1 => 77,
        WKeyCode::Numpad2 => 78,
        WKeyCode::Numpad3 => 79,
        WKeyCode::Numpad4 => 80,
        WKeyCode::Numpad5 => 81,
        WKeyCode::Numpad6 => 82,
        WKeyCode::Numpad7 => 83,
        WKeyCode::Numpad8 => 84,
        WKeyCode::Numpad9 => 85,
        WKeyCode::NumpadAdd => 86,
        WKeyCode::NumpadBackspace => 87,
        WKeyCode::NumpadClear => 88,
        WKeyCode::NumpadClearEntry => 89,
        WKeyCode::NumpadComma => 90,
        WKeyCode::NumpadDecimal => 91,
        WKeyCode::NumpadDivide => 92,
        WKeyCode::NumpadEnter => 93,
        WKeyCode::NumpadEqual => 94,
        WKeyCode::NumpadHash => 95,
        WKeyCode::NumpadMemoryAdd => 96,
        WKeyCode::NumpadMemoryClear => 97,
        WKeyCode::NumpadMemoryRecall => 98,
        WKeyCode::NumpadMemoryStore => 99,
        WKeyCode::NumpadMemorySubtract => 100,
        WKeyCode::NumpadMultiply => 101,
        WKeyCode::NumpadParenLeft => 102,
        WKeyCode::NumpadParenRight => 103,
        WKeyCode::NumpadStar => 104,
        WKeyCode::NumpadSubtract => 105,

        // Function Keys (106-130)
        WKeyCode::Escape => 106,
        WKeyCode::F1 => 107,
        WKeyCode::F2 => 108,
        WKeyCode::F3 => 109,
        WKeyCode::F4 => 110,
        WKeyCode::F5 => 111,
        WKeyCode::F6 => 112,
        WKeyCode::F7 => 113,
        WKeyCode::F8 => 114,
        WKeyCode::F9 => 115,
        WKeyCode::F10 => 116,
        WKeyCode::F11 => 117,
        WKeyCode::F12 => 118,
        WKeyCode::F13 => 119,
        WKeyCode::F14 => 120,
        WKeyCode::F15 => 121,
        WKeyCode::F16 => 122,
        WKeyCode::F17 => 123,
        WKeyCode::F18 => 124,
        WKeyCode::F19 => 125,
        WKeyCode::F20 => 126,
        WKeyCode::F21 => 127,
        WKeyCode::F22 => 128,
        WKeyCode::F23 => 129,
        WKeyCode::F24 => 130,

        // Lock / media / browser / system
        WKeyCode::ScrollLock => 131,
        WKeyCode::AudioVolumeDown => 132,
        WKeyCode::AudioVolumeMute => 133,
        WKeyCode::AudioVolumeUp => 134,
        WKeyCode::MediaPlayPause => 135,
        WKeyCode::MediaStop => 136,
        WKeyCode::MediaTrackNext => 137,
        WKeyCode::MediaTrackPrevious => 138,
        WKeyCode::BrowserBack => 139,
        WKeyCode::BrowserFavorites => 140,
        WKeyCode::BrowserForward => 141,
        WKeyCode::BrowserHome => 142,
        WKeyCode::BrowserRefresh => 143,
        WKeyCode::BrowserSearch => 144,
        WKeyCode::BrowserStop => 145,
        WKeyCode::PrintScreen => 146,
        WKeyCode::Pause => 147,
        _ => 255, // Unidentified
    }
}

/// Canonical mapping from browser `KeyboardEvent.code` to Vulfram key ids.
#[cfg(any(feature = "wasm", test))]
pub fn map_web_key_code(code: &str) -> u32 {
    match code {
        // Writing system keys
        "Backquote" => 0,
        "Backslash" => 1,
        "BracketLeft" => 2,
        "BracketRight" => 3,
        "Comma" => 4,
        "Digit0" => 5,
        "Digit1" => 6,
        "Digit2" => 7,
        "Digit3" => 8,
        "Digit4" => 9,
        "Digit5" => 10,
        "Digit6" => 11,
        "Digit7" => 12,
        "Digit8" => 13,
        "Digit9" => 14,
        "Equal" => 15,
        "IntlBackslash" => 16,
        "IntlRo" => 17,
        "IntlYen" => 18,
        "KeyA" => 19,
        "KeyB" => 20,
        "KeyC" => 21,
        "KeyD" => 22,
        "KeyE" => 23,
        "KeyF" => 24,
        "KeyG" => 25,
        "KeyH" => 26,
        "KeyI" => 27,
        "KeyJ" => 28,
        "KeyK" => 29,
        "KeyL" => 30,
        "KeyM" => 31,
        "KeyN" => 32,
        "KeyO" => 33,
        "KeyP" => 34,
        "KeyQ" => 35,
        "KeyR" => 36,
        "KeyS" => 37,
        "KeyT" => 38,
        "KeyU" => 39,
        "KeyV" => 40,
        "KeyW" => 41,
        "KeyX" => 42,
        "KeyY" => 43,
        "KeyZ" => 44,
        "Minus" => 45,
        "Period" => 46,
        "Quote" => 47,
        "Semicolon" => 48,
        "Slash" => 49,

        // Functional keys
        "AltLeft" => 50,
        "AltRight" => 51,
        "Backspace" => 52,
        "CapsLock" => 53,
        "ContextMenu" => 54,
        "ControlLeft" => 55,
        "ControlRight" => 56,
        "Enter" => 57,
        "MetaLeft" => 58,
        "MetaRight" => 59,
        "ShiftLeft" => 60,
        "ShiftRight" => 61,
        "Space" => 62,
        "Tab" => 63,

        // Control keys
        "Delete" => 64,
        "End" => 65,
        "Help" => 66,
        "Home" => 67,
        "Insert" => 68,
        "PageDown" => 69,
        "PageUp" => 70,

        // Arrow keys
        "ArrowDown" => 71,
        "ArrowLeft" => 72,
        "ArrowRight" => 73,
        "ArrowUp" => 74,

        // Numpad keys
        "NumLock" => 75,
        "Numpad0" => 76,
        "Numpad1" => 77,
        "Numpad2" => 78,
        "Numpad3" => 79,
        "Numpad4" => 80,
        "Numpad5" => 81,
        "Numpad6" => 82,
        "Numpad7" => 83,
        "Numpad8" => 84,
        "Numpad9" => 85,
        "NumpadAdd" => 86,
        "NumpadBackspace" => 87,
        "NumpadClear" => 88,
        "NumpadClearEntry" => 89,
        "NumpadComma" => 90,
        "NumpadDecimal" => 91,
        "NumpadDivide" => 92,
        "NumpadEnter" => 93,
        "NumpadEqual" => 94,
        "NumpadHash" => 95,
        "NumpadMemoryAdd" => 96,
        "NumpadMemoryClear" => 97,
        "NumpadMemoryRecall" => 98,
        "NumpadMemoryStore" => 99,
        "NumpadMemorySubtract" => 100,
        "NumpadMultiply" => 101,
        "NumpadParenLeft" => 102,
        "NumpadParenRight" => 103,
        "NumpadStar" => 104,
        "NumpadSubtract" => 105,

        // Function keys
        "Escape" => 106,
        "F1" => 107,
        "F2" => 108,
        "F3" => 109,
        "F4" => 110,
        "F5" => 111,
        "F6" => 112,
        "F7" => 113,
        "F8" => 114,
        "F9" => 115,
        "F10" => 116,
        "F11" => 117,
        "F12" => 118,
        "F13" => 119,
        "F14" => 120,
        "F15" => 121,
        "F16" => 122,
        "F17" => 123,
        "F18" => 124,
        "F19" => 125,
        "F20" => 126,
        "F21" => 127,
        "F22" => 128,
        "F23" => 129,
        "F24" => 130,
        "ScrollLock" => 131,
        "AudioVolumeDown" => 132,
        "AudioVolumeMute" => 133,
        "AudioVolumeUp" => 134,
        "MediaPlayPause" => 135,
        "MediaStop" => 136,
        "MediaTrackNext" => 137,
        "MediaTrackPrevious" => 138,
        "BrowserBack" => 139,
        "BrowserFavorites" => 140,
        "BrowserForward" => 141,
        "BrowserHome" => 142,
        "BrowserRefresh" => 143,
        "BrowserSearch" => 144,
        "BrowserStop" => 145,
        "PrintScreen" => 146,
        "Pause" => 147,
        _ => KEY_UNIDENTIFIED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[cfg(not(feature = "wasm"))]
    #[test]
    fn desktop_browser_keycode_parity_subset() {
        use crate::core::platform::winit::keyboard::KeyCode as WKeyCode;

        let cases = [
            (WKeyCode::Escape, "Escape"),
            (WKeyCode::KeyW, "KeyW"),
            (WKeyCode::Enter, "Enter"),
            (WKeyCode::ArrowUp, "ArrowUp"),
            (WKeyCode::NumpadEnter, "NumpadEnter"),
            (WKeyCode::F12, "F12"),
        ];

        for (winit_code, web_code) in cases {
            assert_eq!(
                map_winit_key_code(winit_code),
                map_web_key_code(web_code),
                "parity mismatch for {:?} vs {}",
                winit_code,
                web_code
            );
        }
    }
}
