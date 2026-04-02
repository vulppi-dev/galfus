use super::{KEY_ESCAPE, KEY_UNIDENTIFIED, KEY_W, map_web_key_code};

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
