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
