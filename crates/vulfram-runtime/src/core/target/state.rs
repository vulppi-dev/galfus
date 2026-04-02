use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use vulfram_realm_core::AutoLink;
pub use vulfram_realm_core::{
    DimensionValue, TargetId, TargetKind, TargetLayerLayout, TargetLayerState,
};

use crate::core::realm::RealmId;
use crate::core::target::TargetGraphCache;

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

#[derive(Debug, Clone, Default)]
pub struct TargetLayerTable {
    pub entries: HashMap<(u32, TargetId), TargetLayerState>,
}

impl TargetLayerTable {}

#[derive(Debug, Default)]
pub struct TargetRoutingState {
    pub targets: TargetTable,
    pub target_layers: TargetLayerTable,
    pub target_graph_cache: TargetGraphCache,
    pub auto_links: HashMap<(u32, TargetId), AutoLink>,
    pub host_realm_index: HashMap<u32, RealmId>,
    pub target_ui_realm_index: HashMap<TargetId, RealmId>,
    pub target_autolink_failures: Vec<crate::core::realm::TargetAutoLinkFailure>,
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
