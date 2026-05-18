use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaterialShaderBasePreset {
    Standard,
    Pbr,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialShaderCompileSpec {
    pub base_preset: MaterialShaderBasePreset,
    pub shader_source: Option<String>,
    #[serde(default)]
    pub shader_params_schema: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CompiledMaterialShader {
    pub source: String,
    pub hash: u64,
}

pub fn compile_material_shader_spec(
    spec: &MaterialShaderCompileSpec,
) -> Result<CompiledMaterialShader, String> {
    let source = match spec.shader_source.as_ref() {
        Some(custom) if !custom.trim().is_empty() => custom.clone(),
        _ => match spec.base_preset {
            MaterialShaderBasePreset::Standard => include_str!(
                "../../vulfram-runtime/src/core/render/passes/forward/branches/forward_standard.wgsl"
            )
            .to_string(),
            MaterialShaderBasePreset::Pbr => include_str!(
                "../../vulfram-runtime/src/core/render/passes/forward/branches/forward_pbr.wgsl"
            )
            .to_string(),
        },
    };

    if let Err(err) = naga::front::wgsl::parse_str(&source) {
        return Err(format!("Material WGSL is invalid: {}", err.emit_to_string(&source)));
    }

    let mut hasher = DefaultHasher::new();
    spec.base_preset.hash(&mut hasher);
    source.hash(&mut hasher);
    let mut params: Vec<_> = spec.shader_params_schema.iter().collect();
    params.sort_by(|a, b| a.0.cmp(b.0));
    for (name, ty) in params {
        name.hash(&mut hasher);
        ty.hash(&mut hasher);
    }
    let hash = hasher.finish();

    Ok(CompiledMaterialShader { source, hash })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_standard_preset() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Standard,
            shader_source: None,
            shader_params_schema: HashMap::new(),
        };
        let compiled = compile_material_shader_spec(&spec).expect("standard should compile");
        assert!(!compiled.source.is_empty());
        assert_ne!(compiled.hash, 0);
    }

    #[test]
    fn compiles_pbr_preset() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Pbr,
            shader_source: None,
            shader_params_schema: HashMap::new(),
        };
        let compiled = compile_material_shader_spec(&spec).expect("pbr should compile");
        assert!(!compiled.source.is_empty());
        assert_ne!(compiled.hash, 0);
    }

    #[test]
    fn compiles_custom_source() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Standard,
            shader_source: Some(
                include_str!(
                    "../../vulfram-runtime/src/core/render/passes/forward/branches/forward_standard.wgsl"
                )
                .to_string(),
            ),
            shader_params_schema: HashMap::new(),
        };
        let compiled = compile_material_shader_spec(&spec).expect("custom should compile");
        assert_ne!(compiled.hash, 0);
    }

    #[test]
    fn rejects_invalid_custom_source() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Standard,
            shader_source: Some("fn broken(".to_string()),
            shader_params_schema: HashMap::new(),
        };
        let err = compile_material_shader_spec(&spec).expect_err("invalid wgsl should fail");
        assert!(err.contains("invalid"));
    }

    #[test]
    fn hash_is_stable() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Standard,
            shader_source: None,
            shader_params_schema: HashMap::from([(String::from("a"), String::from("f32"))]),
        };
        let a = compile_material_shader_spec(&spec).expect("first compile");
        let b = compile_material_shader_spec(&spec).expect("second compile");
        assert_eq!(a.hash, b.hash);
    }
}
