use super::{graph_is_compatible, supports_render_pass};
use galfus_realm_core::{
    RENDER_PASS_2D_BATCH, RENDER_PASS_2D_COMPOSE, RENDER_PASS_2D_DRAW, RENDER_PASS_2D_PREPARE,
    RENDER_PASS_FORWARD, RENDER_PASS_POST, RENDER_PASS_UI,
};

#[test]
fn twod_realm_accepts_ui_and_base_2d_passes() {
    assert!(supports_render_pass(RENDER_PASS_UI));
    assert!(supports_render_pass(RENDER_PASS_2D_PREPARE));
    assert!(supports_render_pass(RENDER_PASS_2D_BATCH));
    assert!(supports_render_pass(RENDER_PASS_2D_DRAW));
    assert!(supports_render_pass(RENDER_PASS_2D_COMPOSE));
    assert!(!supports_render_pass(RENDER_PASS_FORWARD));
    assert!(graph_is_compatible([
        RENDER_PASS_2D_PREPARE,
        RENDER_PASS_2D_BATCH,
        RENDER_PASS_2D_DRAW,
        RENDER_PASS_2D_COMPOSE
    ]));
    assert!(!graph_is_compatible([RENDER_PASS_UI, RENDER_PASS_POST]));
}
