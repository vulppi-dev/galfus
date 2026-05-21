use super::*;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn pointer_state_cache_detects_position_changes() {
    let cache = PointerStateCache::new();
    assert!(cache.position_changed(Vec2::new(1.0, 0.0)));
    assert!(!cache.position_changed(Vec2::new(0.0, 0.0)));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn input_state_starts_with_default_modifiers() {
    let state = InputState::new();
    assert_eq!(state.modifiers, ModifiersState::default());
    assert!(state.cache.pointers.is_empty());
}
