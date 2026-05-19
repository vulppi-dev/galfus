use super::{graph_is_compatible, supports_render_pass};
use galfus_realm_core::{RENDER_PASS_FORWARD, RENDER_PASS_POST, RENDER_PASS_UI};

#[test]
fn twod_realm_accepts_only_ui_passes() {
    assert!(supports_render_pass(RENDER_PASS_UI));
    assert!(!supports_render_pass(RENDER_PASS_FORWARD));
    assert!(graph_is_compatible([RENDER_PASS_UI]));
    assert!(!graph_is_compatible([RENDER_PASS_UI, RENDER_PASS_POST]));
}
