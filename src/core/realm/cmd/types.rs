use serde::{Deserialize, Serialize};

use crate::core::realm::{RealmKind, SurfaceKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RealmKindDto {
    ThreeD,
    TwoD,
}

impl From<RealmKindDto> for RealmKind {
    fn from(value: RealmKindDto) -> Self {
        match value {
            RealmKindDto::ThreeD => RealmKind::ThreeD,
            RealmKindDto::TwoD => RealmKind::TwoD,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SurfaceKindDto {
    Onscreen,
    Offscreen,
}

impl From<SurfaceKindDto> for SurfaceKind {
    fn from(value: SurfaceKindDto) -> Self {
        match value {
            SurfaceKindDto::Onscreen => SurfaceKind::Onscreen,
            SurfaceKindDto::Offscreen => SurfaceKind::Offscreen,
        }
    }
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
