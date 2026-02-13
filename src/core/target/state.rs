use std::collections::HashMap;

use glam::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    Window,
    RealmViewport,
    UiPlane,
    Texture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceFormatDto {
    Rgba16Float,
    Rgba8Unorm,
    Bgra8Unorm,
    Depth32Float,
    Depth24Plus,
}

impl SurfaceFormatDto {
    pub fn to_wgpu(self) -> wgpu::TextureFormat {
        match self {
            SurfaceFormatDto::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            SurfaceFormatDto::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            SurfaceFormatDto::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
            SurfaceFormatDto::Depth32Float => wgpu::TextureFormat::Depth32Float,
            SurfaceFormatDto::Depth24Plus => wgpu::TextureFormat::Depth24Plus,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceAlphaModeDto {
    Auto,
    Opaque,
    PreMultiplied,
    PostMultiplied,
    Inherit,
}

impl SurfaceAlphaModeDto {
    pub fn to_wgpu(self) -> wgpu::CompositeAlphaMode {
        match self {
            SurfaceAlphaModeDto::Auto => wgpu::CompositeAlphaMode::Auto,
            SurfaceAlphaModeDto::Opaque => wgpu::CompositeAlphaMode::Opaque,
            SurfaceAlphaModeDto::PreMultiplied => wgpu::CompositeAlphaMode::PreMultiplied,
            SurfaceAlphaModeDto::PostMultiplied => wgpu::CompositeAlphaMode::PostMultiplied,
            SurfaceAlphaModeDto::Inherit => wgpu::CompositeAlphaMode::Inherit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetState {
    pub kind: TargetKind,
    pub window_id: Option<u32>,
    pub size: Option<glam::UVec2>,
    pub format_policy: Option<wgpu::TextureFormat>,
    pub alpha_policy: Option<wgpu::CompositeAlphaMode>,
    pub msaa_samples: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct TargetTable {
    pub entries: HashMap<TargetId, TargetState>,
}

impl TargetTable {}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TargetLayerLayout {
    pub rect: Vec4,
    pub z_index: i32,
    pub blend_mode: u32,
    pub clip: Option<Vec4>,
    pub input_flags: u32,
}

impl Default for TargetLayerLayout {
    fn default() -> Self {
        Self {
            rect: Vec4::ZERO,
            z_index: 0,
            blend_mode: 0,
            clip: None,
            input_flags: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetLayerState {
    pub realm_id: u32,
    pub target_id: TargetId,
    pub layout: TargetLayerLayout,
}

#[derive(Debug, Clone, Default)]
pub struct TargetLayerTable {
    pub entries: HashMap<(u32, TargetId), TargetLayerState>,
}

impl TargetLayerTable {}
