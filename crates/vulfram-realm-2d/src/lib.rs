pub const SUPPORTED_RENDER_PASSES: &[&str] = &["ui"];

pub fn supports_render_pass(pass_id: &str) -> bool {
    SUPPORTED_RENDER_PASSES.contains(&pass_id)
}

pub fn graph_is_compatible<'a>(pass_ids: impl IntoIterator<Item = &'a str>) -> bool {
    pass_ids.into_iter().all(supports_render_pass)
}

#[cfg(test)]
mod tests {
    use super::{graph_is_compatible, supports_render_pass};

    #[test]
    fn twod_realm_accepts_only_ui_passes() {
        assert!(supports_render_pass("ui"));
        assert!(!supports_render_pass("forward"));
        assert!(graph_is_compatible(["ui"]));
        assert!(!graph_is_compatible(["ui", "post"]));
    }
}
