use std::collections::HashMap;

use crate::{
    LogicalId, RenderGraphShaderSpec, RenderGraphShaderType, RenderGraphValue, validate_shader_spec,
};

fn shader_spec(source: &str) -> RenderGraphShaderSpec {
    RenderGraphShaderSpec {
        shader_type: RenderGraphShaderType::Screen,
        source: source.into(),
        params: HashMap::from([("threshold".into(), "f32".into())]),
        capabilities: Default::default(),
    }
}

#[test]
fn accepts_minimal_screen_shader_and_generates_physical_wgsl() {
    let spec = shader_spec(
        r#"
fn fragment(input: FragmentInput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = sample_color(input.uv);
    return out;
}
"#,
    );
    let values = HashMap::from([("threshold".into(), RenderGraphValue::Float(1.0))]);
    let wgsl = validate_shader_spec(
        &spec,
        &[LogicalId::Str("color".into())],
        &[LogicalId::Str("color".into())],
        &values,
    )
    .expect("shader should be valid");
    assert!(wgsl.contains("@group(0) @binding(0) var<uniform> params: PassParams;"));
    assert!(wgsl.contains("fn sample_color"));
    assert!(wgsl.contains("fn fragment(input: FragmentInput) -> FragmentOutput"));
}

#[test]
fn rejects_forbidden_binding_tokens() {
    let spec = shader_spec(
        r#"
@group(0) @binding(0) var<uniform> x: vec4<f32>;
fn fragment(input: FragmentInput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = vec4<f32>(1.0);
    return out;
}
"#,
    );
    let err = validate_shader_spec(
        &spec,
        &[LogicalId::Str("color".into())],
        &[LogicalId::Str("color".into())],
        &HashMap::new(),
    )
    .expect_err("shader should be rejected");
    assert!(err.contains("forbidden token"));
}

#[test]
fn rejects_param_not_declared_in_schema() {
    let spec = shader_spec(
        r#"
fn fragment(input: FragmentInput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = sample_color(input.uv);
    return out;
}
"#,
    );
    let values = HashMap::from([("intensity".into(), RenderGraphValue::Float(0.5))]);
    let err = validate_shader_spec(
        &spec,
        &[LogicalId::Str("color".into())],
        &[LogicalId::Str("color".into())],
        &values,
    )
    .expect_err("undeclared param should fail");
    assert!(err.contains("provided but not declared"));
}
