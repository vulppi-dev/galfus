use crate::core::resources::{PolygonMode, PrimitiveTopology, RenderSide, SurfaceType};
use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MaterialKind {
    Shader,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
#[repr(u32)]
pub enum MaterialSampler {
    PointClamp = 0,
    LinearClamp = 1,
    PointRepeat = 2,
    LinearRepeat = 3,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct StandardOptions {
    pub base_color: Option<Vec4>,
    pub surface_type: Option<SurfaceType>,
    pub topology: Option<PrimitiveTopology>,
    pub polygon_mode: Option<PolygonMode>,
    pub render_side: Option<RenderSide>,
    pub emissive_color: Option<Vec4>,
    pub spec_color: Option<Vec4>,
    pub spec_power: Option<f32>,
    pub base_tex_id: Option<u32>,
    pub base_sampler: Option<MaterialSampler>,
    pub spec_tex_id: Option<u32>,
    pub spec_sampler: Option<MaterialSampler>,
    pub normal_tex_id: Option<u32>,
    pub normal_sampler: Option<MaterialSampler>,
    pub toon_ramp_tex_id: Option<u32>,
    pub toon_ramp_sampler: Option<MaterialSampler>,
    pub emissive_tex_id: Option<u32>,
    pub emissive_sampler: Option<MaterialSampler>,
    pub flags: Option<u32>,
    pub toon_params: Option<Vec4>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PbrOptions {
    pub base_color: Option<Vec4>,
    pub surface_type: Option<SurfaceType>,
    pub topology: Option<PrimitiveTopology>,
    pub polygon_mode: Option<PolygonMode>,
    pub render_side: Option<RenderSide>,
    pub emissive_color: Option<Vec4>,
    pub metallic: Option<f32>,
    pub roughness: Option<f32>,
    pub ao: Option<f32>,
    pub normal_scale: Option<f32>,
    pub base_tex_id: Option<u32>,
    pub base_sampler: Option<MaterialSampler>,
    pub normal_tex_id: Option<u32>,
    pub normal_sampler: Option<MaterialSampler>,
    pub metallic_roughness_tex_id: Option<u32>,
    pub metallic_roughness_sampler: Option<MaterialSampler>,
    pub emissive_tex_id: Option<u32>,
    pub emissive_sampler: Option<MaterialSampler>,
    pub ao_tex_id: Option<u32>,
    pub ao_sampler: Option<MaterialSampler>,
    pub flags: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum MaterialOptions {
    Standard(StandardOptions),
    Pbr(PbrOptions),
}

// MARK: - Create Material

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialCreateArgs {
    pub material_id: u32,
    pub label: Option<String>,
    pub slug: String,
    pub kind: MaterialKind,
    #[serde(default)]
    pub options: Option<MaterialOptions>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialCreate {
    pub success: bool,
    pub message: String,
}

// MARK: - Update Material

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialUpdateArgs {
    pub material_id: u32,
    pub label: Option<String>,
    pub slug: Option<String>,
    pub kind: Option<MaterialKind>,
    #[serde(default)]
    pub options: Option<MaterialOptions>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialUpdate {
    pub success: bool,
    pub message: String,
}

// MARK: - Dispose Material

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialDisposeArgs {
    pub material_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialDispose {
    pub success: bool,
    pub message: String,
}

// MARK: - Material Definition

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialDefinitionCreateArgs {
    pub definition_id: u32,
    pub slug: String,
    pub label: Option<String>,
    pub preset: crate::core::resources::ShaderMaterialPreset,
    #[serde(default)]
    pub shader_type: Option<crate::core::resources::MaterialShaderType>,
    pub shader_source: String,
    #[serde(default)]
    pub shader_params_schema: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialDefinitionUpdateArgs {
    pub definition_id: u32,
    pub slug: Option<String>,
    pub label: Option<String>,
    pub preset: Option<crate::core::resources::ShaderMaterialPreset>,
    #[serde(default)]
    pub shader_type: Option<crate::core::resources::MaterialShaderType>,
    pub shader_source: String,
    #[serde(default)]
    pub shader_params_schema: Option<HashMap<String, String>>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialDefinitionDisposeArgs {
    pub definition_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialDefinition {
    pub success: bool,
    pub message: String,
}

// MARK: - Material Instance

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialInstanceCreateArgs {
    pub material_id: u32,
    pub slug: String,
    pub label: Option<String>,
    #[serde(default)]
    pub options: Option<MaterialOptions>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialInstanceUpdateArgs {
    pub material_id: u32,
    pub slug: Option<String>,
    pub label: Option<String>,
    #[serde(default)]
    pub options: Option<MaterialOptions>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdMaterialInstanceDisposeArgs {
    pub material_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultMaterialInstance {
    pub success: bool,
    pub message: String,
}
