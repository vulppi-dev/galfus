use std::collections::HashMap;

use glam::Vec4;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum TargetKind {
    Window,
    WidgetRealmViewport,
    RealmPlane,
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
#[serde(tag = "unit", content = "value", rename_all = "kebab-case")]
pub enum DimensionValue {
    Px(f32),
    Percent(f32),
    Character(f32),
    Display(f32),
}

impl DimensionValue {
    pub fn resolve(self, reference: f32, char_width: f32) -> f32 {
        match self {
            Self::Px(value) => value,
            Self::Percent(value) => (value / 100.0) * reference,
            Self::Character(value) => value * char_width,
            Self::Display(value) => value * 4.0,
        }
    }
}

impl Default for DimensionValue {
    fn default() -> Self {
        Self::Px(0.0)
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TargetLayerLayout {
    pub left: DimensionValue,
    pub top: DimensionValue,
    pub width: DimensionValue,
    pub height: DimensionValue,
    pub z_index: i32,
    pub blend_mode: u32,
    pub clip: Option<Vec4>,
}

impl Default for TargetLayerLayout {
    fn default() -> Self {
        Self {
            left: DimensionValue::Px(0.0),
            top: DimensionValue::Px(0.0),
            width: DimensionValue::Percent(100.0),
            height: DimensionValue::Percent(100.0),
            z_index: 0,
            blend_mode: 0,
            clip: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetLayerState {
    pub realm_id: u32,
    pub target_id: TargetId,
    pub layout: TargetLayerLayout,
    pub camera_id: Option<u32>,
    pub environment_id: Option<u32>,
}

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
