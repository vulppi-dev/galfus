use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::common::{ElementState, TouchPhase};

/// Mouse scroll delta type
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum ScrollDelta {
    /// Line-based scrolling (traditional mouse wheel)
    Line(Vec2),
    /// Pixel-based scrolling (touchpad)
    Pixel(Vec2),
}

/// Pointer (Mouse/Touch) events - unified for both input types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum PointerEvent {
    /// Pointer moved
    #[serde(rename_all = "camelCase")]
    OnMove {
        window_id: u32,
        pointer_type: u32,
        pointer_id: u64,
        /// Position relative to the window (global pointer space).
        position: Vec2,
        /// Optional position relative to the resolved target.
        #[serde(default)]
        position_target: Option<Vec2>,
        trace: Option<PointerEventTrace>,
    },

    /// Pointer entered window area
    #[serde(rename_all = "camelCase")]
    OnEnter {
        window_id: u32,
        pointer_type: u32,
        pointer_id: u64,
        trace: Option<PointerEventTrace>,
    },

    /// Pointer left window area
    #[serde(rename_all = "camelCase")]
    OnLeave {
        window_id: u32,
        pointer_type: u32,
        pointer_id: u64,
        trace: Option<PointerEventTrace>,
    },

    /// Pointer button pressed/released (mouse) or touch started/ended
    #[serde(rename_all = "camelCase")]
    OnButton {
        window_id: u32,
        pointer_type: u32,
        pointer_id: u64,
        button: u32,
        state: ElementState,
        /// Position relative to the window (global pointer space).
        position: Vec2,
        /// Optional position relative to the resolved target.
        #[serde(default)]
        position_target: Option<Vec2>,
        trace: Option<PointerEventTrace>,
    },

    /// Mouse wheel/touchpad scroll
    #[serde(rename_all = "camelCase")]
    OnScroll {
        window_id: u32,
        delta: ScrollDelta,
        phase: TouchPhase,
        trace: Option<PointerEventTrace>,
    },

    /// Touch event with pressure and additional info
    #[serde(rename_all = "camelCase")]
    OnTouch {
        window_id: u32,
        pointer_id: u64,
        phase: TouchPhase,
        /// Position relative to the window (global pointer space).
        position: Vec2,
        /// Optional position relative to the resolved target.
        #[serde(default)]
        position_target: Option<Vec2>,
        pressure: Option<f32>,
        trace: Option<PointerEventTrace>,
    },

    /// Pinch gesture (zoom)
    #[serde(rename_all = "camelCase")]
    OnPinchGesture {
        window_id: u32,
        delta: f64,
        phase: TouchPhase,
        trace: Option<PointerEventTrace>,
    },

    /// Pan gesture
    #[serde(rename_all = "camelCase")]
    OnPanGesture {
        window_id: u32,
        delta: Vec2,
        phase: TouchPhase,
        trace: Option<PointerEventTrace>,
    },

    /// Rotation gesture
    #[serde(rename_all = "camelCase")]
    OnRotationGesture {
        window_id: u32,
        delta: f32,
        phase: TouchPhase,
        trace: Option<PointerEventTrace>,
    },

    /// Double tap gesture
    #[serde(rename_all = "camelCase")]
    OnDoubleTapGesture {
        window_id: u32,
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
