use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ElementState {
    Released = 0,
    Pressed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TouchPhase {
    Started = 0,
    Moved,
    Ended,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifiersState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum KeyboardEvent {
    #[serde(rename_all = "camelCase")]
    OnInput {
        window_id: u32,
        key_code: u32,
        state: ElementState,
        location: u32,
        repeat: bool,
        text: Option<String>,
        modifiers: ModifiersState,
    },
    #[serde(rename_all = "camelCase")]
    OnModifiersChange {
        window_id: u32,
        modifiers: ModifiersState,
    },
    #[serde(rename_all = "camelCase")]
    OnImeEnable { window_id: u32 },
    #[serde(rename_all = "camelCase")]
    OnImePreedit {
        window_id: u32,
        text: String,
        cursor_range: Option<(usize, usize)>,
    },
    #[serde(rename_all = "camelCase")]
    OnImeCommit { window_id: u32, text: String },
    #[serde(rename_all = "camelCase")]
    OnImeDisable { window_id: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum ScrollDelta {
    Line(Vec2),
    Pixel(Vec2),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum PointerEvent {
    #[serde(rename_all = "camelCase")]
    OnMove {
        window_id: u32,
        #[serde(default)]
        window_width: Option<u32>,
        #[serde(default)]
        window_height: Option<u32>,
        pointer_type: u32,
        pointer_id: u64,
        position: Vec2,
        #[serde(default)]
        position_target: Option<Vec2>,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        trace: Option<PointerEventTrace>,
    },
    #[serde(rename_all = "camelCase")]
    OnEnter {
        window_id: u32,
        #[serde(default)]
        window_width: Option<u32>,
        #[serde(default)]
        window_height: Option<u32>,
        pointer_type: u32,
        pointer_id: u64,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        trace: Option<PointerEventTrace>,
    },
    #[serde(rename_all = "camelCase")]
    OnLeave {
        window_id: u32,
        #[serde(default)]
        window_width: Option<u32>,
        #[serde(default)]
        window_height: Option<u32>,
        pointer_type: u32,
        pointer_id: u64,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        trace: Option<PointerEventTrace>,
    },
    #[serde(rename_all = "camelCase")]
    OnButton {
        window_id: u32,
        #[serde(default)]
        window_width: Option<u32>,
        #[serde(default)]
        window_height: Option<u32>,
        pointer_type: u32,
        pointer_id: u64,
        button: u32,
        state: ElementState,
        position: Vec2,
        #[serde(default)]
        position_target: Option<Vec2>,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        trace: Option<PointerEventTrace>,
    },
    #[serde(rename_all = "camelCase")]
    OnScroll {
        window_id: u32,
        #[serde(default)]
        window_width: Option<u32>,
        #[serde(default)]
        window_height: Option<u32>,
        delta: ScrollDelta,
        phase: TouchPhase,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        trace: Option<PointerEventTrace>,
    },
    #[serde(rename_all = "camelCase")]
    OnTouch {
        window_id: u32,
        #[serde(default)]
        window_width: Option<u32>,
        #[serde(default)]
        window_height: Option<u32>,
        pointer_id: u64,
        phase: TouchPhase,
        position: Vec2,
        #[serde(default)]
        position_target: Option<Vec2>,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        pressure: Option<f32>,
        trace: Option<PointerEventTrace>,
    },
    #[serde(rename_all = "camelCase")]
    OnPinchGesture {
        window_id: u32,
        #[serde(default)]
        window_width: Option<u32>,
        #[serde(default)]
        window_height: Option<u32>,
        delta: f64,
        phase: TouchPhase,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        trace: Option<PointerEventTrace>,
    },
    #[serde(rename_all = "camelCase")]
    OnPanGesture {
        window_id: u32,
        #[serde(default)]
        window_width: Option<u32>,
        #[serde(default)]
        window_height: Option<u32>,
        delta: Vec2,
        phase: TouchPhase,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        trace: Option<PointerEventTrace>,
    },
    #[serde(rename_all = "camelCase")]
    OnRotationGesture {
        window_id: u32,
        #[serde(default)]
        window_width: Option<u32>,
        #[serde(default)]
        window_height: Option<u32>,
        delta: f32,
        phase: TouchPhase,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        trace: Option<PointerEventTrace>,
    },
    #[serde(rename_all = "camelCase")]
    OnDoubleTapGesture {
        window_id: u32,
        #[serde(default)]
        window_width: Option<u32>,
        #[serde(default)]
        window_height: Option<u32>,
        #[serde(default)]
        target_width: Option<u32>,
        #[serde(default)]
        target_height: Option<u32>,
        trace: Option<PointerEventTrace>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointerEventTrace {
    pub window_id: u32,
    pub realm_id: u32,
    pub target_id: Option<u64>,
    pub connector_id: Option<u32>,
    pub source_realm_id: Option<u32>,
    pub uv: Option<Vec2>,
    #[serde(default)]
    pub hops: Vec<PointerTraceHop>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointerTraceHop {
    pub stage: PointerTraceStage,
    #[serde(default)]
    pub realm_id: Option<u32>,
    #[serde(default)]
    pub target_id: Option<u64>,
    #[serde(default)]
    pub layer_realm_id: Option<u32>,
    #[serde(default)]
    pub connector_id: Option<u32>,
    #[serde(default)]
    pub surface_id: Option<u32>,
    #[serde(default)]
    pub camera_id: Option<u32>,
    #[serde(default)]
    pub uv: Option<Vec2>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PointerTraceStage {
    RootWindow,
    Capture,
    FocusFallback,
    ConnectorHit,
    RealmPlaneHit,
    HopForward,
    StopNoHit,
    StopCycle,
    StopStepBudget,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PointerTraceLevel {
    Off,
    Errors,
    Basic,
    Full,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointerTraceConfig {
    pub level: PointerTraceLevel,
    pub sampling_percent: u8,
}

impl Default for PointerTraceConfig {
    fn default() -> Self {
        Self {
            level: PointerTraceLevel::Full,
            sampling_percent: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_default_to_all_false() {
        let modifiers = ModifiersState::default();
        assert!(!modifiers.shift);
        assert!(!modifiers.ctrl);
        assert!(!modifiers.alt);
        assert!(!modifiers.meta);
    }

    #[test]
    fn pointer_trace_config_defaults_to_full_sampling() {
        let config = PointerTraceConfig::default();
        assert_eq!(config.level, PointerTraceLevel::Full);
        assert_eq!(config.sampling_percent, 100);
    }

    #[test]
    fn keyboard_event_round_trips_through_messagepack() {
        let event = KeyboardEvent::OnInput {
            window_id: 1,
            key_code: 42,
            state: ElementState::Pressed,
            location: 0,
            repeat: false,
            text: Some("a".into()),
            modifiers: ModifiersState {
                shift: true,
                ctrl: false,
                alt: false,
                meta: false,
            },
        };

        let bytes = rmp_serde::to_vec_named(&event).expect("keyboard event should encode");
        let decoded: KeyboardEvent =
            rmp_serde::from_slice(&bytes).expect("keyboard event should decode");

        match decoded {
            KeyboardEvent::OnInput {
                window_id,
                key_code,
                state,
                ..
            } => {
                assert_eq!(window_id, 1);
                assert_eq!(key_code, 42);
                assert_eq!(state, ElementState::Pressed);
            }
            _ => panic!("decoded wrong keyboard event variant"),
        }
    }
}
