use vulfram_realm_core::RENDER_PASS_UI;

pub const SUPPORTED_RENDER_PASSES: &[&str] = &[RENDER_PASS_UI];

pub fn supports_render_pass(pass_id: &str) -> bool {
    SUPPORTED_RENDER_PASSES.contains(&pass_id)
}

pub fn graph_is_compatible<'a>(pass_ids: impl IntoIterator<Item = &'a str>) -> bool {
    pass_ids.into_iter().all(supports_render_pass)
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
