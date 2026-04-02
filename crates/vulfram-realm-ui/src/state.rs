use serde::{Deserialize, Serialize};

use crate::{UiDocumentId, UiNodeId};

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UiDebugState {
    pub enabled: bool,
    pub show_bounds: bool,
    pub show_ids: bool,
    pub show_profile: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UiAnimProperty {
    Opacity,
    TranslateY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiAnimKey {
    pub document_id: UiDocumentId,
    pub node_id: UiNodeId,
    pub property: UiAnimProperty,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiAnimState {
    pub start_time: f64,
    pub from: f32,
    pub to: f32,
    pub duration: f32,
    pub finished: bool,
    pub last_value: f32,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UiSceneState {
    pub pan: glam::Vec2,
    pub zoom: f32,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UiFrameProfile {
    pub layout_ms: f32,
    pub tessellate_ms: f32,
    pub upload_ms: f32,
    pub draw_ms: f32,
    pub input_routing_ms: f32,
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
