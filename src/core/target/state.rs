use std::collections::HashMap;

use serde::{Deserialize, Serialize};
pub use vulfram_scene_core::{
    DimensionValue, TargetId, TargetKind, TargetLayerLayout, TargetLayerState,
};

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

#[cfg(test)]
mod tests {
    use super::DimensionValue;

    #[test]
    fn dimension_value_px_resolves_directly() {
        let value = DimensionValue::Px(24.0);
        assert_eq!(value.resolve(100.0, 8.0), 24.0);
    }

    #[test]
    fn dimension_value_percent_uses_reference_axis() {
        let value = DimensionValue::Percent(25.0);
        assert_eq!(value.resolve(400.0, 8.0), 100.0);
    }

    #[test]
    fn dimension_value_character_uses_char_width() {
        let value = DimensionValue::Character(10.0);
        assert_eq!(value.resolve(0.0, 7.5), 75.0);
    }

    #[test]
    fn dimension_value_display_uses_four_pixel_grid() {
        let value = DimensionValue::Display(6.0);
        assert_eq!(value.resolve(0.0, 8.0), 24.0);
    }

    #[test]
    fn dimension_value_deserializes_from_host_shape() {
        #[derive(serde::Serialize)]
        struct HostDimension {
            unit: &'static str,
            value: f32,
        }
        let bytes = rmp_serde::to_vec_named(&HostDimension {
            unit: "percent",
            value: 50.0,
        })
        .unwrap();
        let value: DimensionValue = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(value, DimensionValue::Percent(50.0));
    }
}
