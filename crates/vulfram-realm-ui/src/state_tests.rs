use super::*;

#[test]
fn ui_anim_key_round_trips_through_json() {
    let key = UiAnimKey {
        document_id: 10,
        node_id: 20,
        property: UiAnimProperty::TranslateY,
    };

    let json = serde_json::to_string(&key).expect("serialize");
    let decoded: UiAnimKey = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(decoded, key);
}

#[test]
fn ui_debug_state_defaults_disabled() {
    let state = UiDebugState::default();

    assert!(!state.enabled);
    assert!(!state.show_bounds);
    assert!(!state.show_ids);
    assert!(!state.show_profile);
}
