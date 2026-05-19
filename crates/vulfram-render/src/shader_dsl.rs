use std::collections::HashMap;

use crate::{LogicalId, RenderGraphValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RenderGraphShaderType {
    Screen,
    Draw,
    Compute,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGraphShaderSpec {
    #[serde(rename = "type")]
    pub shader_type: RenderGraphShaderType,
    pub source: String,
    #[serde(default)]
    pub params: HashMap<String, String>,
}

const FORBIDDEN_SHADER_TOKENS: [&str; 14] = [
    "@group",
    "@binding",
    "@vertex",
    "@fragment",
    "@compute",
    "@location",
    "@builtin",
    "var<uniform>",
    "var<storage>",
    "texture_2d",
    "texture_depth_2d",
    "sampler",
    "override ",
    "override\t",
];

pub fn validate_shader_spec(
    shader: &RenderGraphShaderSpec,
    inputs: &[LogicalId],
    outputs: &[LogicalId],
    values: &HashMap<String, RenderGraphValue>,
) -> Result<String, String> {
    if matches!(shader.shader_type, RenderGraphShaderType::Screen) && outputs.is_empty() {
        return Err("Screen shader requires at least one output resource".into());
    }
    validate_client_source(&shader.source)?;
    validate_entrypoint(shader.shader_type, &shader.source)?;
    validate_param_values(&shader.params, values)?;
    let wgsl = generate_physical_wgsl(shader, inputs, outputs);
    if let Err(err) = naga::front::wgsl::parse_str(&wgsl) {
        return Err(format!(
            "Generated WGSL is invalid: {}",
            err.emit_to_string(&wgsl)
        ));
    }
    Ok(wgsl)
}

fn validate_client_source(source: &str) -> Result<(), String> {
    for token in FORBIDDEN_SHADER_TOKENS {
        if source.contains(token) {
            return Err(format!(
                "Shader source contains forbidden token '{}'",
                token
            ));
        }
    }
    Ok(())
}

fn validate_entrypoint(shader_type: RenderGraphShaderType, source: &str) -> Result<(), String> {
    let has_vertex = source.contains("fn vertex(");
    let has_fragment = source.contains("fn fragment(");
    let has_compute = source.contains("fn compute(");
    match shader_type {
        RenderGraphShaderType::Screen => {
            if !has_fragment {
                return Err("Screen shader must define 'fn fragment(input: FragmentInput) -> FragmentOutput'".into());
            }
        }
        RenderGraphShaderType::Draw => {
            if !has_vertex || !has_fragment {
                return Err(
                    "Draw shader must define both 'fn vertex(...)' and 'fn fragment(...)'".into(),
                );
            }
        }
        RenderGraphShaderType::Compute => {
            if !has_compute {
                return Err(
                    "Compute shader must define 'fn compute(input: ComputeInput) -> ComputeOutput'"
                        .into(),
                );
            }
        }
    }
    Ok(())
}

fn validate_param_values(
    schema: &HashMap<String, String>,
    values: &HashMap<String, RenderGraphValue>,
) -> Result<(), String> {
    for key in values.keys() {
        if !schema.contains_key(key) {
            return Err(format!(
                "Param '{}' provided but not declared in shader schema",
                key
            ));
        }
    }
    for (param_name, param_ty) in schema {
        let Some(value) = values.get(param_name) else {
            continue;
        };
        let valid = match param_ty.as_str() {
            "f32" => matches!(value, RenderGraphValue::Float(_) | RenderGraphValue::Int(_)),
            "i32" => matches!(value, RenderGraphValue::Int(_)),
            "bool" => matches!(value, RenderGraphValue::Bool(_)),
            "string" => matches!(value, RenderGraphValue::String(_)),
            _ => {
                return Err(format!(
                    "Unsupported param type '{}' for '{}'",
                    param_ty, param_name
                ));
            }
        };
        if !valid {
            return Err(format!(
                "Param '{}' expects type '{}' but received incompatible value",
                param_name, param_ty
            ));
        }
    }
    Ok(())
}

fn generate_physical_wgsl(
    shader: &RenderGraphShaderSpec,
    inputs: &[LogicalId],
    outputs: &[LogicalId],
) -> String {
    let mut params_fields: Vec<(String, String)> = shader
        .params
        .iter()
        .map(|(name, ty)| (name.clone(), ty.clone()))
        .collect();
    params_fields.sort_by(|a, b| a.0.cmp(&b.0));

    let mut wgsl = String::new();
    wgsl.push_str("// Auto-generated physical WGSL from Vulfram shader DSL\n");
    if !params_fields.is_empty() {
        wgsl.push_str("struct PassParams {\n");
        for (name, ty) in &params_fields {
            wgsl.push_str(&format!("  {}: {},\n", sanitize_ident(name), ty));
        }
        wgsl.push_str("};\n");
        wgsl.push_str("@group(0) @binding(0) var<uniform> params: PassParams;\n");
    }

    wgsl.push_str(
        r#"
struct VsOut {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VsOut {
  var positions = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0)
  );
  let pos = positions[vertex_index];
  var out: VsOut;
  out.position = vec4<f32>(pos, 0.0, 1.0);
  out.uv = pos * 0.5 + vec2<f32>(0.5, 0.5);
  return out;
}

struct FragmentInput {
  uv: vec2<f32>,
  pixel: vec2<u32>,
};
struct FragmentOutput {
"#,
    );
    for output in outputs {
        wgsl.push_str(&format!("  {}: vec4<f32>,\n", resource_ident(output)));
    }
    wgsl.push_str("};\n");
    wgsl.push_str("struct PhysicalFragmentOutput {\n");
    for (index, output) in outputs.iter().enumerate() {
        wgsl.push_str(&format!(
            "  @location({}) {}: vec4<f32>,\n",
            index,
            resource_ident(output)
        ));
    }
    wgsl.push_str("};\n");

    for input in inputs {
        let name = resource_ident(input);
        wgsl.push_str(&format!(
            "fn sample_{}(_uv: vec2<f32>) -> vec4<f32> {{ return vec4<f32>(0.0); }}\n",
            name
        ));
        wgsl.push_str(&format!(
            "fn load_{}(_pixel: vec2<u32>) -> vec4<f32> {{ return vec4<f32>(0.0); }}\n",
            name
        ));
    }
    wgsl.push('\n');
    wgsl.push_str(shader.source.trim());
    wgsl.push_str(
        "\n@fragment\nfn fs_main(input: VsOut) -> PhysicalFragmentOutput {\n  let logical_in = FragmentInput(input.uv, vec2<u32>(0u, 0u));\n  let logical_out = fragment(logical_in);\n  var physical_out: PhysicalFragmentOutput;\n",
    );
    for output in outputs {
        let name = resource_ident(output);
        wgsl.push_str(&format!("  physical_out.{0} = logical_out.{0};\n", name));
    }
    wgsl.push_str("  return physical_out;\n}\n");
    wgsl
}

fn sanitize_ident(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn resource_ident(id: &LogicalId) -> String {
    sanitize_ident(&id.to_string())
}

#[cfg(test)]
#[path = "shader_dsl_tests.rs"]
mod tests;
