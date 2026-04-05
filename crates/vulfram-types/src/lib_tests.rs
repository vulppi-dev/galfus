use super::*;

#[test]
fn realm_id_round_trips_through_messagepack() {
    let encoded = rmp_serde::to_vec_named(&RealmId(42)).expect("realm id should encode");
    let decoded: RealmId = rmp_serde::from_slice(&encoded).expect("realm id should decode");
    assert_eq!(decoded, RealmId(42));
}

#[test]
fn realm_kind_uses_kebab_case_strings() {
    let encoded = serde_json::to_string(&RealmKind::ThreeD).expect("enum should encode");
    assert_eq!(encoded, "\"three-d\"");
}

#[test]
fn surface_kind_distinguishes_onscreen_and_offscreen() {
    assert_ne!(SurfaceKind::Onscreen, SurfaceKind::Offscreen);
}
