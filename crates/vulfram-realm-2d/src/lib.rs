use vulfram_realm_core::RENDER_PASS_UI;

pub const SUPPORTED_RENDER_PASSES: &[&str] = &[RENDER_PASS_UI];

pub fn supports_render_pass(pass_id: &str) -> bool {
    SUPPORTED_RENDER_PASSES.contains(&pass_id)
}

pub fn graph_is_compatible<'a>(pass_ids: impl IntoIterator<Item = &'a str>) -> bool {
    pass_ids.into_iter().all(supports_render_pass)
}

#[cfg(test)]
mod tests {
    use super::{graph_is_compatible, supports_render_pass};
    use vulfram_realm_core::{RENDER_PASS_FORWARD, RENDER_PASS_POST, RENDER_PASS_UI};

    #[test]
    fn twod_realm_accepts_only_ui_passes() {
        assert!(supports_render_pass(RENDER_PASS_UI));
        assert!(!supports_render_pass(RENDER_PASS_FORWARD));
        assert!(graph_is_compatible([RENDER_PASS_UI]));
        assert!(!graph_is_compatible([RENDER_PASS_UI, RENDER_PASS_POST]));
    }
}
