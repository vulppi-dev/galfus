use super::{
    GamepadCacheManager, GamepadEvent, GamepadStateCache, connect_gamepad, disconnect_gamepad,
};
use crate::ElementState;

#[test]
fn update_axis_and_get_returns_current_adjusted_value() {
    let mut cache = GamepadStateCache::new();
    let value = cache.update_axis_and_get(0, 0.5);
    assert_eq!(value, cache.get_axis_value(0));
}

#[test]
fn update_axis_and_get_applies_dead_zone() {
    let mut cache = GamepadStateCache::new();
    let value = cache.update_axis_and_get(0, 0.01);
    assert_eq!(value, 0.0);
}

#[test]
fn update_button_event_emits_pressed_state() {
    let mut cache = GamepadStateCache::new();
    let event = cache.update_button_event(7, 2, 1.0);
    assert!(matches!(
        event,
        Some(GamepadEvent::OnButton {
            gamepad_id: 7,
            button: 2,
            state: ElementState::Pressed,
            value,
        }) if value == 1.0
    ));
}

#[test]
fn update_axis_event_skips_small_changes() {
    let mut cache = GamepadStateCache::new();
    assert!(cache.update_axis_event(3, 1, 0.0).is_none());
}

#[test]
fn connect_and_disconnect_gamepad_emit_lifecycle_events_once() {
    let mut manager = GamepadCacheManager::new();
    assert!(matches!(
        connect_gamepad(&mut manager, 4, "Pad"),
        Some(GamepadEvent::OnConnect { gamepad_id: 4, .. })
    ));
    assert!(connect_gamepad(&mut manager, 4, "Pad").is_none());
    assert!(matches!(
        disconnect_gamepad(&mut manager, 4),
        Some(GamepadEvent::OnDisconnect { gamepad_id: 4 })
    ));
    assert!(disconnect_gamepad(&mut manager, 4).is_none());
}
