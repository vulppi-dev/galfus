mod state;

use galfus_realm_core::{
    RENDER_PASS_2D_BATCH, RENDER_PASS_2D_COMPOSE, RENDER_PASS_2D_DRAW, RENDER_PASS_2D_PREPARE,
    RENDER_PASS_UI,
};

pub const SUPPORTED_RENDER_PASSES: &[&str] = &[
    RENDER_PASS_UI,
    RENDER_PASS_2D_PREPARE,
    RENDER_PASS_2D_BATCH,
    RENDER_PASS_2D_DRAW,
    RENDER_PASS_2D_COMPOSE,
];

pub fn supports_render_pass(pass_id: &str) -> bool {
    SUPPORTED_RENDER_PASSES.contains(&pass_id)
}

pub fn graph_is_compatible<'a>(pass_ids: impl IntoIterator<Item = &'a str>) -> bool {
    pass_ids.into_iter().all(supports_render_pass)
}

pub use state::{Realm2dState, RealmEntities};

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
