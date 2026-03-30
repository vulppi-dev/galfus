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
mod tests {
    use super::*;

    #[test]
    fn ui_anim_key_round_trips_through_json() {
        let key = UiAnimKey {
            document_id: 10,
            node_id: 20,
            property: UiAnimProperty::TranslateY,
        };

        let json = serde_json::to_string(&key).expect("serialize");
        let decoded: UiAnimKey = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(decoded, key);
    }

    #[test]
    fn ui_debug_state_defaults_disabled() {
        let state = UiDebugState::default();

        assert!(!state.enabled);
        assert!(!state.show_bounds);
        assert!(!state.show_ids);
        assert!(!state.show_profile);
    }
}
