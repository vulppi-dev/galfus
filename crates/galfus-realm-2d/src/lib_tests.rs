use super::{graph_is_compatible, supports_render_pass};
use galfus_realm_core::{
    RENDER_PASS_BATCH, RENDER_PASS_COMPOSE, RENDER_PASS_CUSTOM_POST_FORWARD,
    RENDER_PASS_CUSTOM_PRE_FORWARD, RENDER_PASS_FORWARD, RENDER_PASS_POST, RENDER_PASS_PREPARE,
};

#[test]
fn twod_realm_accepts_base_passes() {
    assert!(supports_render_pass(RENDER_PASS_CUSTOM_PRE_FORWARD));
    assert!(supports_render_pass(RENDER_PASS_CUSTOM_POST_FORWARD));
    assert!(supports_render_pass(RENDER_PASS_PREPARE));
    assert!(supports_render_pass(RENDER_PASS_BATCH));
    assert!(supports_render_pass(RENDER_PASS_FORWARD));
    assert!(supports_render_pass(RENDER_PASS_COMPOSE));
    assert!(graph_is_compatible([
        RENDER_PASS_PREPARE,
        RENDER_PASS_BATCH,
        RENDER_PASS_FORWARD,
        RENDER_PASS_COMPOSE
    ]));
    assert!(!graph_is_compatible([
        RENDER_PASS_CUSTOM_POST_FORWARD,
        RENDER_PASS_POST
    ]));
}
