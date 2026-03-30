pub const SUPPORTED_RENDER_PASSES: &[&str] = &[
    "shadow",
    "light-cull",
    "skybox",
    "forward",
    "outline",
    "ssao",
    "ssao-blur",
    "bloom",
    "post",
    "compose",
    "ui",
];

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
    fn threed_realm_accepts_full_pipeline_passes() {
        assert!(supports_render_pass("shadow"));
        assert!(supports_render_pass("ui"));
        assert!(!supports_render_pass("unknown"));
        assert!(graph_is_compatible(["shadow", "forward", "compose", "ui"]));
        assert!(!graph_is_compatible(["shadow", "unknown"]));
    }
}
