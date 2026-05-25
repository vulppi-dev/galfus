use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MaterialKind {
    Shader,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum MaterialOptions {
    Schema(HashMap<String, Vec4>),
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
    pub realm_kind: crate::core::resources::MaterialRealmKind,
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
    pub realm_kind: Option<crate::core::resources::MaterialRealmKind>,
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
    pub realm_kind: crate::core::resources::MaterialRealmKind,
    #[serde(default)]
    pub preset: Option<crate::core::resources::ShaderMaterialPreset>,
    #[serde(default)]
    pub shader_type: Option<crate::core::resources::MaterialShaderType>,
    #[serde(default)]
    pub shader_source: Option<String>,
    #[serde(default)]
    pub shader_params_schema: Option<HashMap<String, String>>,
    #[serde(default)]
    pub capabilities: Option<MaterialShaderCapabilities>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdMaterialDefinitionUpdateArgs {
    pub definition_id: u32,
    pub slug: Option<String>,
    pub label: Option<String>,
    #[serde(default)]
    pub realm_kind: Option<crate::core::resources::MaterialRealmKind>,
    #[serde(default)]
    pub preset: Option<crate::core::resources::ShaderMaterialPreset>,
    #[serde(default)]
    pub shader_type: Option<crate::core::resources::MaterialShaderType>,
    #[serde(default)]
    pub shader_source: Option<String>,
    #[serde(default)]
    pub shader_params_schema: Option<HashMap<String, String>>,
    #[serde(default)]
    pub capabilities: Option<MaterialShaderCapabilities>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct MaterialShaderCapabilities {
    pub semantics: Vec<String>,
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
