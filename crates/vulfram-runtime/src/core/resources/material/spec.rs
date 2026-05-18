use bytemuck::{Pod, Zeroable};
use glam::Vec4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

fn preset_shader_source(preset: ShaderMaterialPreset) -> String {
    match preset {
        ShaderMaterialPreset::Standard => {
            include_str!("../../render/passes/forward/branches/forward_standard.wgsl").to_string()
        }
        ShaderMaterialPreset::Pbr => {
            include_str!("../../render/passes/forward/branches/forward_pbr.wgsl").to_string()
        }
    }
}

fn preset_shader_hash(source: &str, preset: ShaderMaterialPreset) -> u64 {
    let mut hasher = DefaultHasher::new();
    (preset as u32).hash(&mut hasher);
    source.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u32)]
pub enum SurfaceType {
    Opaque = 0,
    Masked = 1,
    Transparent = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u32)]
pub enum PrimitiveTopology {
    PointList = 0,
    LineList = 1,
    TriangleList = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u32)]
pub enum PolygonMode {
    Fill = 0,
    Line = 1,
    Point = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[repr(u32)]
pub enum RenderSide {
    Front = 0,
    Back = 1,
    DoubleSide = 2,
}

pub const MATERIAL_FALLBACK_ID: u32 = 0;
pub const SHADER_MATERIAL_INPUTS_PER_MATERIAL: u32 = 8;
pub const SHADER_MATERIAL_TEXTURE_SLOTS: usize = 8;
pub const SHADER_MATERIAL_INVALID_SLOT: u32 = u32::MAX;
pub const STANDARD_INPUTS_PER_MATERIAL: u32 = 8;
pub const STANDARD_TEXTURE_SLOTS: usize = 8;
pub const STANDARD_INVALID_SLOT: u32 = u32::MAX;
pub const PBR_INPUTS_PER_MATERIAL: u32 = 8;
pub const PBR_TEXTURE_SLOTS: usize = 8;
pub const PBR_INVALID_SLOT: u32 = u32::MAX;
pub const TEX_SOURCE_STANDALONE: u32 = 0;
pub const TEX_SOURCE_ATLAS: u32 = 1;
pub const TEX_SOURCE_INVALID: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShaderMaterialPreset {
    Standard,
    Pbr,
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct MaterialStandardParams {
    pub input_indices: glam::UVec4,
    pub inputs_offset_count: glam::UVec2,
    pub surface_flags: glam::UVec2,
    pub texture_slots: [glam::UVec4; 2],
    pub sampler_indices: [glam::UVec4; 2],
    pub tex_sources: [glam::UVec4; 2],
    pub atlas_layers: [glam::UVec4; 2],
    pub atlas_scale_bias: [glam::Vec4; STANDARD_TEXTURE_SLOTS],
}

impl Default for MaterialStandardParams {
    fn default() -> Self {
        Self {
            input_indices: glam::UVec4::new(0, 1, 2, 3),
            inputs_offset_count: glam::UVec2::new(0, STANDARD_INPUTS_PER_MATERIAL),
            surface_flags: glam::UVec2::new(SurfaceType::Opaque as u32, 0),
            texture_slots: [glam::UVec4::splat(STANDARD_INVALID_SLOT); 2],
            sampler_indices: [glam::UVec4::ZERO; 2],
            tex_sources: [glam::UVec4::splat(TEX_SOURCE_INVALID); 2],
            atlas_layers: [glam::UVec4::ZERO; 2],
            atlas_scale_bias: [glam::Vec4::new(1.0, 1.0, 0.0, 0.0); STANDARD_TEXTURE_SLOTS],
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct MaterialPbrParams {
    pub input_indices: glam::UVec4,
    pub inputs_offset_count: glam::UVec2,
    pub surface_flags: glam::UVec2,
    pub texture_slots: [glam::UVec4; 2],
    pub sampler_indices: [glam::UVec4; 2],
    pub tex_sources: [glam::UVec4; 2],
    pub atlas_layers: [glam::UVec4; 2],
    pub atlas_scale_bias: [glam::Vec4; PBR_TEXTURE_SLOTS],
}

impl Default for MaterialPbrParams {
    fn default() -> Self {
        Self {
            input_indices: glam::UVec4::new(0, 1, 2, 3),
            inputs_offset_count: glam::UVec2::new(0, PBR_INPUTS_PER_MATERIAL),
            surface_flags: glam::UVec2::new(SurfaceType::Opaque as u32, 0),
            texture_slots: [glam::UVec4::splat(PBR_INVALID_SLOT); 2],
            sampler_indices: [glam::UVec4::ZERO; 2],
            tex_sources: [glam::UVec4::splat(TEX_SOURCE_INVALID); 2],
            atlas_layers: [glam::UVec4::ZERO; 2],
            atlas_scale_bias: [glam::Vec4::new(1.0, 1.0, 0.0, 0.0); PBR_TEXTURE_SLOTS],
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaterialStandardRecord {
    pub label: Option<String>,
    pub data: MaterialStandardParams,
    pub inputs: Vec<Vec4>,
    pub texture_ids: [u32; STANDARD_TEXTURE_SLOTS],
    pub surface_type: SurfaceType,
    pub topology: PrimitiveTopology,
    pub polygon_mode: PolygonMode,
    pub render_side: RenderSide,
    pub is_dirty: bool,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl MaterialStandardRecord {
    pub fn new(label: Option<String>, data: MaterialStandardParams) -> Self {
        let mut inputs = vec![Vec4::ZERO; STANDARD_INPUTS_PER_MATERIAL as usize];
        inputs[0] = Vec4::ONE;
        inputs[1] = Vec4::ONE;
        inputs[2] = Vec4::new(32.0, 0.0, 0.0, 0.0);
        Self {
            label,
            data,
            inputs,
            texture_ids: [STANDARD_INVALID_SLOT; STANDARD_TEXTURE_SLOTS],
            surface_type: SurfaceType::Opaque,
            topology: PrimitiveTopology::TriangleList,
            polygon_mode: PolygonMode::Fill,
            render_side: RenderSide::Front,
            is_dirty: true,
            bind_group: None,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }
}

#[derive(Debug, Clone)]
pub struct MaterialPbrRecord {
    pub label: Option<String>,
    pub data: MaterialPbrParams,
    pub inputs: Vec<Vec4>,
    pub texture_ids: [u32; PBR_TEXTURE_SLOTS],
    pub surface_type: SurfaceType,
    pub topology: PrimitiveTopology,
    pub polygon_mode: PolygonMode,
    pub render_side: RenderSide,
    pub is_dirty: bool,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl MaterialPbrRecord {
    pub fn new(label: Option<String>, data: MaterialPbrParams) -> Self {
        let mut inputs = vec![Vec4::ZERO; PBR_INPUTS_PER_MATERIAL as usize];
        inputs[0] = Vec4::ONE;
        inputs[1] = Vec4::ZERO;
        inputs[2] = Vec4::new(0.0, 1.0, 1.0, 0.0);
        inputs[3] = Vec4::new(1.0, 0.0, 0.0, 0.0);
        Self {
            label,
            data,
            inputs,
            texture_ids: [PBR_INVALID_SLOT; PBR_TEXTURE_SLOTS],
            surface_type: SurfaceType::Opaque,
            topology: PrimitiveTopology::TriangleList,
            polygon_mode: PolygonMode::Fill,
            render_side: RenderSide::Front,
            is_dirty: true,
            bind_group: None,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }
}

#[derive(Debug, Clone)]
pub struct ShaderMaterialRecord {
    pub label: Option<String>,
    pub preset: ShaderMaterialPreset,
    pub base_preset: ShaderMaterialPreset,
    pub shader_source: Option<String>,
    pub shader_params_schema: HashMap<String, String>,
    pub compiled_shader_hash: u64,
    pub compiled_shader_source: Option<String>,
    pub compile_error: Option<String>,
    pub data_standard: MaterialStandardParams,
    pub data_pbr: MaterialPbrParams,
    pub inputs: Vec<Vec4>,
    pub texture_ids: [u32; SHADER_MATERIAL_TEXTURE_SLOTS],
    pub surface_type: SurfaceType,
    pub topology: PrimitiveTopology,
    pub polygon_mode: PolygonMode,
    pub render_side: RenderSide,
    pub is_dirty: bool,
    pub bind_group: Option<wgpu::BindGroup>,
}

impl ShaderMaterialRecord {
    pub fn new_standard(label: Option<String>) -> Self {
        let compiled_shader_source = preset_shader_source(ShaderMaterialPreset::Standard);
        let compiled_shader_hash =
            preset_shader_hash(&compiled_shader_source, ShaderMaterialPreset::Standard);
        let mut inputs = vec![Vec4::ZERO; SHADER_MATERIAL_INPUTS_PER_MATERIAL as usize];
        inputs[0] = Vec4::ONE;
        inputs[1] = Vec4::ONE;
        inputs[2] = Vec4::new(32.0, 0.0, 0.0, 0.0);
        Self {
            label,
            preset: ShaderMaterialPreset::Standard,
            base_preset: ShaderMaterialPreset::Standard,
            shader_source: None,
            shader_params_schema: HashMap::new(),
            compiled_shader_hash,
            compiled_shader_source: Some(compiled_shader_source),
            compile_error: None,
            data_standard: MaterialStandardParams::default(),
            data_pbr: MaterialPbrParams::default(),
            inputs,
            texture_ids: [SHADER_MATERIAL_INVALID_SLOT; SHADER_MATERIAL_TEXTURE_SLOTS],
            surface_type: SurfaceType::Opaque,
            topology: PrimitiveTopology::TriangleList,
            polygon_mode: PolygonMode::Fill,
            render_side: RenderSide::Front,
            is_dirty: true,
            bind_group: None,
        }
    }

    pub fn new_pbr(label: Option<String>) -> Self {
        let compiled_shader_source = preset_shader_source(ShaderMaterialPreset::Pbr);
        let compiled_shader_hash = preset_shader_hash(&compiled_shader_source, ShaderMaterialPreset::Pbr);
        let mut inputs = vec![Vec4::ZERO; SHADER_MATERIAL_INPUTS_PER_MATERIAL as usize];
        inputs[0] = Vec4::ONE;
        inputs[1] = Vec4::ZERO;
        inputs[2] = Vec4::new(0.0, 1.0, 1.0, 0.0);
        inputs[3] = Vec4::new(1.0, 0.0, 0.0, 0.0);
        Self {
            label,
            preset: ShaderMaterialPreset::Pbr,
            base_preset: ShaderMaterialPreset::Pbr,
            shader_source: None,
            shader_params_schema: HashMap::new(),
            compiled_shader_hash,
            compiled_shader_source: Some(compiled_shader_source),
            compile_error: None,
            data_standard: MaterialStandardParams::default(),
            data_pbr: MaterialPbrParams::default(),
            inputs,
            texture_ids: [SHADER_MATERIAL_INVALID_SLOT; SHADER_MATERIAL_TEXTURE_SLOTS],
            surface_type: SurfaceType::Opaque,
            topology: PrimitiveTopology::TriangleList,
            polygon_mode: PolygonMode::Fill,
            render_side: RenderSide::Front,
            is_dirty: true,
            bind_group: None,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }

    pub fn mark_structural_dirty(&mut self) {
        self.is_dirty = true;
        self.bind_group = None;
    }
}
