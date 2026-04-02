use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use vulfram_types::{ConnectorId, PresentId, RealmId, RealmKind, SurfaceId};

#[derive(Debug, Clone)]
pub struct RealmState {
    pub kind: RealmKind,
    pub output_surface: Option<SurfaceId>,
    pub render_graph_id: Option<u32>,
    pub importance: u8,
    pub cache_policy: u8,
    pub last_render_frame: u64,
}

#[derive(Debug, Clone)]
pub struct ConnectorState {
    pub target_realm: RealmId,
    pub source_surface: SurfaceId,
    pub rect: glam::Vec4,
    pub z_index: i32,
    pub blend_mode: u32,
    pub clip: Option<glam::Vec4>,
    pub input_flags: u32,
}

#[derive(Debug, Clone)]
pub struct PresentState {
    pub window_id: u32,
    pub surface: SurfaceId,
}

#[derive(Debug, Clone)]
pub struct AutoLink {
    pub surface_id: SurfaceId,
    pub connector_id: Option<ConnectorId>,
    pub present_id: Option<PresentId>,
}

#[derive(Debug, Default)]
pub struct SurfaceCache {
    pub last_good: HashMap<ConnectorId, SurfaceId>,
    pub fallback: HashMap<ConnectorId, SurfaceId>,
}

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
    pub clip: Option<glam::Vec4>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TargetEdge {
    pub parent: TargetId,
    pub child: TargetId,
}

#[derive(Debug, Clone, Default)]
pub struct TargetGraphPlan {
    pub edges: Vec<TargetEdge>,
    pub order: Vec<TargetId>,
    pub cut_edges: Vec<TargetEdge>,
}

#[derive(Debug, Clone, Default)]
pub struct TargetGraphDiff {
    pub added_targets: Vec<TargetId>,
    pub removed_targets: Vec<TargetId>,
    pub updated_targets: Vec<TargetId>,
    pub added_layers: Vec<(u32, TargetId)>,
    pub removed_layers: Vec<(u32, TargetId)>,
    pub updated_layers: Vec<(u32, TargetId)>,
    pub dirty_targets: Vec<TargetId>,
    pub plan_dirty: bool,
}

#[derive(Debug, Default)]
pub struct TargetGraphPlanner;
