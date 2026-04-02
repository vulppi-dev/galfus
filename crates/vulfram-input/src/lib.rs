mod builders;
mod cache;
mod gamepad;
mod keycodes;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use glam::Vec2;
use serde::{Deserialize, Serialize};
use vulfram_realm_core::ConnectorState;
use vulfram_types::{ConnectorId, RealmId, SurfaceId};

pub use builders::{
    element_state_from_pressed, keyboard_ime_commit_event, keyboard_ime_disable_event,
    keyboard_ime_enable_event, keyboard_ime_preedit_event, keyboard_input_event,
    keyboard_modifiers_event, pointer_button_event, pointer_double_tap_gesture_event,
    pointer_enter_event, pointer_leave_event, pointer_move_event, pointer_pan_gesture_event,
    pointer_pinch_gesture_event, pointer_rotation_gesture_event, pointer_scroll_event,
    pointer_touch_event,
};
pub use cache::InputCacheManager;
#[cfg(not(target_arch = "wasm32"))]
pub use cache::InputState;
#[cfg(not(target_arch = "wasm32"))]
pub use cache::{KeyboardStateCache, PointerStateCache};
pub use gamepad::{
    GAMEPAD_AXIS_CHANGE_THRESHOLD, GAMEPAD_AXIS_DEAD_ZONE, GAMEPAD_BUTTON_CHANGE_THRESHOLD,
    GamepadCacheManager, GamepadEvent, GamepadState, GamepadStateCache,
};
pub use keycodes::{KEY_ESCAPE, KEY_UNIDENTIFIED, KEY_W, map_web_key_code};

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

fn default_true() -> bool {
    true
}

const fn u8_100() -> u8 {
    100
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmdInputTargetListenerUpsertArgs {
    pub listener_id: u64,
    pub target_id: u64,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default = "u8_100")]
    pub sample_percent: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmdInputTargetListenerDisposeArgs {
    pub listener_id: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmdInputTargetListenerListArgs {
    #[serde(default)]
    pub target_id: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputTargetListenerSnapshot {
    pub listener_id: u64,
    pub target_id: u64,
    pub enabled: bool,
    pub events: Vec<String>,
    pub sample_percent: u8,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputTargetListenerConfig {
    pub listener_id: u64,
    pub target_id: u64,
    pub enabled: bool,
    pub events: Vec<String>,
    pub sample_percent: u8,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputCapture {
    pub connector_id: u32,
    pub target_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedInputCapture {
    pub connector_id: ConnectorId,
    pub target_id: Option<vulfram_realm_core::TargetId>,
}

#[derive(Debug, Clone)]
pub struct InputRoutingConnectorHit {
    pub id: ConnectorId,
    pub state: ConnectorState,
    pub source_size: glam::UVec2,
    pub target_id: Option<vulfram_realm_core::TargetId>,
    pub target_rank: i32,
}

#[derive(Debug, Default)]
pub struct InputRoutingCache {
    pub topology_hash: u64,
    pub realm_by_surface: HashMap<SurfaceId, RealmId>,
    pub realm_by_window: HashMap<u32, (RealmId, SurfaceId)>,
    pub connector_targets: HashMap<ConnectorId, vulfram_realm_core::TargetId>,
    pub layer_camera_by_key: HashMap<(u32, vulfram_realm_core::TargetId), Option<u32>>,
    pub connectors_by_realm: HashMap<RealmId, Vec<InputRoutingConnectorHit>>,
}

#[derive(Debug, Default)]
pub struct InputRoutingState {
    pub captures: HashMap<(u32, u64), InputCapture>,
    pub focus_targets: HashMap<u32, vulfram_realm_core::TargetId>,
    pub trace: PointerTraceConfig,
    pub cache: InputRoutingCache,
}

#[derive(Debug, Clone, Copy)]
pub struct HitResult {
    pub connector_id: ConnectorId,
    pub uv: Option<Vec2>,
}

#[derive(Debug, Clone, Copy)]
pub struct InputRoutingRealmOutput {
    pub realm_id: RealmId,
    pub output_surface: Option<SurfaceId>,
}

#[derive(Debug, Clone, Copy)]
pub struct InputRoutingPresentBinding {
    pub window_id: u32,
    pub surface_id: SurfaceId,
}

#[derive(Debug, Clone, Copy)]
pub struct InputRoutingTargetRank {
    pub target_id: vulfram_realm_core::TargetId,
    pub rank: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct InputRoutingAutoLinkRecord {
    pub target_id: vulfram_realm_core::TargetId,
    pub connector_id: ConnectorId,
}

#[derive(Debug, Clone, Copy)]
pub struct InputRoutingLayerCameraRecord {
    pub realm_id: u32,
    pub target_id: vulfram_realm_core::TargetId,
    pub camera_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct InputRoutingConnectorRecord {
    pub connector_id: ConnectorId,
    pub state: ConnectorState,
    pub source_size: glam::UVec2,
}

#[derive(Debug, Clone, Copy)]
pub struct InputRoutingSurfaceSizeRecord {
    pub surface_id: SurfaceId,
    pub size: glam::UVec2,
}

#[derive(Debug, Default, Clone)]
pub struct InputRoutingTopologySnapshot {
    pub realms: Vec<InputRoutingRealmOutput>,
    pub presents: Vec<InputRoutingPresentBinding>,
    pub target_order: Vec<InputRoutingTargetRank>,
    pub auto_links: Vec<InputRoutingAutoLinkRecord>,
    pub layer_cameras: Vec<InputRoutingLayerCameraRecord>,
    pub connectors: Vec<InputRoutingConnectorRecord>,
    pub surfaces: Vec<InputRoutingSurfaceSizeRecord>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct InputTargetSizing {
    pub source_realm_size: Option<glam::UVec2>,
    pub connector_source_size: Option<glam::UVec2>,
    pub target_surface_size: Option<glam::UVec2>,
    pub target_declared_size: Option<glam::UVec2>,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultInputTargetListenerList {
    pub success: bool,
    pub message: String,
    pub listeners: Vec<InputTargetListenerSnapshot>,
}

#[derive(Debug, Default)]
pub struct InputTargetListenerStore {
    by_listener: HashMap<u64, InputTargetListenerConfig>,
    listeners_by_target: HashMap<u64, Vec<u64>>,
}

impl InputTargetListenerStore {
    pub fn upsert(&mut self, config: InputTargetListenerConfig) {
        if let Some(previous) = self.by_listener.insert(config.listener_id, config.clone()) {
            self.remove_from_target_index(previous.target_id, previous.listener_id);
        }
        self.listeners_by_target
            .entry(config.target_id)
            .or_default()
            .push(config.listener_id);
    }

    pub fn dispose(&mut self, listener_id: u64) -> bool {
        let Some(config) = self.by_listener.remove(&listener_id) else {
            return false;
        };
        self.remove_from_target_index(config.target_id, listener_id);
        true
    }

    pub fn dispose_target(&mut self, target_id: u64) -> usize {
        let Some(listener_ids) = self.listeners_by_target.remove(&target_id) else {
            return 0;
        };
        let mut removed = 0;
        for listener_id in listener_ids {
            if self.by_listener.remove(&listener_id).is_some() {
                removed += 1;
            }
        }
        removed
    }

    pub fn dispose_targets<I: IntoIterator<Item = u64>>(&mut self, target_ids: I) -> usize {
        target_ids
            .into_iter()
            .map(|target_id| self.dispose_target(target_id))
            .sum()
    }

    pub fn list(&self, target_id: Option<u64>) -> Vec<InputTargetListenerSnapshot> {
        let mut listeners = match target_id {
            Some(target_id) => self
                .listeners_by_target
                .get(&target_id)
                .into_iter()
                .flat_map(|listener_ids| listener_ids.iter())
                .filter_map(|listener_id| self.by_listener.get(listener_id))
                .map(to_snapshot)
                .collect::<Vec<_>>(),
            None => self
                .by_listener
                .values()
                .map(to_snapshot)
                .collect::<Vec<_>>(),
        };
        listeners.sort_by_key(|listener| listener.listener_id);
        listeners
    }

    pub fn listeners_for_target(&self, target_id: u64) -> Vec<InputTargetListenerConfig> {
        self.listeners_by_target
            .get(&target_id)
            .into_iter()
            .flat_map(|listener_ids| listener_ids.iter())
            .filter_map(|listener_id| self.by_listener.get(listener_id).cloned())
            .collect()
    }

    fn remove_from_target_index(&mut self, target_id: u64, listener_id: u64) {
        if let Some(listener_ids) = self.listeners_by_target.get_mut(&target_id) {
            listener_ids.retain(|id| *id != listener_id);
            if listener_ids.is_empty() {
                self.listeners_by_target.remove(&target_id);
            }
        }
    }
}

fn to_snapshot(config: &InputTargetListenerConfig) -> InputTargetListenerSnapshot {
    InputTargetListenerSnapshot {
        listener_id: config.listener_id,
        target_id: config.target_id,
        enabled: config.enabled,
        events: config.events.clone(),
        sample_percent: config.sample_percent,
    }
}

pub fn pointer_window_id(event: &PointerEvent) -> u32 {
    match event {
        PointerEvent::OnMove { window_id, .. }
        | PointerEvent::OnEnter { window_id, .. }
        | PointerEvent::OnLeave { window_id, .. }
        | PointerEvent::OnButton { window_id, .. }
        | PointerEvent::OnScroll { window_id, .. }
        | PointerEvent::OnTouch { window_id, .. }
        | PointerEvent::OnPinchGesture { window_id, .. }
        | PointerEvent::OnPanGesture { window_id, .. }
        | PointerEvent::OnRotationGesture { window_id, .. }
        | PointerEvent::OnDoubleTapGesture { window_id, .. } => *window_id,
    }
}

pub fn pointer_id(event: &PointerEvent) -> Option<u64> {
    match event {
        PointerEvent::OnMove { pointer_id, .. }
        | PointerEvent::OnEnter { pointer_id, .. }
        | PointerEvent::OnLeave { pointer_id, .. }
        | PointerEvent::OnButton { pointer_id, .. }
        | PointerEvent::OnTouch { pointer_id, .. } => Some(*pointer_id),
        PointerEvent::OnScroll { .. }
        | PointerEvent::OnPinchGesture { .. }
        | PointerEvent::OnPanGesture { .. }
        | PointerEvent::OnRotationGesture { .. }
        | PointerEvent::OnDoubleTapGesture { .. } => None,
    }
}

pub fn pointer_position(event: &PointerEvent) -> Option<Vec2> {
    match event {
        PointerEvent::OnMove { position, .. }
        | PointerEvent::OnButton { position, .. }
        | PointerEvent::OnTouch { position, .. } => Some(*position),
        PointerEvent::OnEnter { .. }
        | PointerEvent::OnLeave { .. }
        | PointerEvent::OnScroll { .. }
        | PointerEvent::OnPinchGesture { .. }
        | PointerEvent::OnPanGesture { .. }
        | PointerEvent::OnRotationGesture { .. }
        | PointerEvent::OnDoubleTapGesture { .. } => None,
    }
}

pub fn apply_trace(event: &mut PointerEvent, trace: Option<PointerEventTrace>) {
    match event {
        PointerEvent::OnMove { trace: slot, .. }
        | PointerEvent::OnEnter { trace: slot, .. }
        | PointerEvent::OnLeave { trace: slot, .. }
        | PointerEvent::OnButton { trace: slot, .. }
        | PointerEvent::OnScroll { trace: slot, .. }
        | PointerEvent::OnTouch { trace: slot, .. }
        | PointerEvent::OnPinchGesture { trace: slot, .. }
        | PointerEvent::OnPanGesture { trace: slot, .. }
        | PointerEvent::OnRotationGesture { trace: slot, .. }
        | PointerEvent::OnDoubleTapGesture { trace: slot, .. } => {
            *slot = trace;
        }
    }
}

pub fn apply_target_position(event: &mut PointerEvent, position_target: Option<Vec2>) {
    match event {
        PointerEvent::OnMove {
            position_target: slot,
            ..
        }
        | PointerEvent::OnButton {
            position_target: slot,
            ..
        }
        | PointerEvent::OnTouch {
            position_target: slot,
            ..
        } => {
            *slot = position_target;
        }
        PointerEvent::OnEnter { .. }
        | PointerEvent::OnLeave { .. }
        | PointerEvent::OnScroll { .. }
        | PointerEvent::OnPinchGesture { .. }
        | PointerEvent::OnPanGesture { .. }
        | PointerEvent::OnRotationGesture { .. }
        | PointerEvent::OnDoubleTapGesture { .. } => {}
    }
}

pub fn apply_target_size(
    event: &mut PointerEvent,
    target_width: Option<u32>,
    target_height: Option<u32>,
) {
    match event {
        PointerEvent::OnMove {
            target_width: width,
            target_height: height,
            ..
        }
        | PointerEvent::OnEnter {
            target_width: width,
            target_height: height,
            ..
        }
        | PointerEvent::OnLeave {
            target_width: width,
            target_height: height,
            ..
        }
        | PointerEvent::OnButton {
            target_width: width,
            target_height: height,
            ..
        }
        | PointerEvent::OnScroll {
            target_width: width,
            target_height: height,
            ..
        }
        | PointerEvent::OnTouch {
            target_width: width,
            target_height: height,
            ..
        }
        | PointerEvent::OnPinchGesture {
            target_width: width,
            target_height: height,
            ..
        }
        | PointerEvent::OnPanGesture {
            target_width: width,
            target_height: height,
            ..
        }
        | PointerEvent::OnRotationGesture {
            target_width: width,
            target_height: height,
            ..
        }
        | PointerEvent::OnDoubleTapGesture {
            target_width: width,
            target_height: height,
            ..
        } => {
            *width = target_width;
            *height = target_height;
        }
    }
}

pub fn apply_window_size(
    event: &mut PointerEvent,
    window_width: Option<u32>,
    window_height: Option<u32>,
) {
    match event {
        PointerEvent::OnMove {
            window_width: width,
            window_height: height,
            ..
        }
        | PointerEvent::OnEnter {
            window_width: width,
            window_height: height,
            ..
        }
        | PointerEvent::OnLeave {
            window_width: width,
            window_height: height,
            ..
        }
        | PointerEvent::OnButton {
            window_width: width,
            window_height: height,
            ..
        }
        | PointerEvent::OnScroll {
            window_width: width,
            window_height: height,
            ..
        }
        | PointerEvent::OnTouch {
            window_width: width,
            window_height: height,
            ..
        }
        | PointerEvent::OnPinchGesture {
            window_width: width,
            window_height: height,
            ..
        }
        | PointerEvent::OnPanGesture {
            window_width: width,
            window_height: height,
            ..
        }
        | PointerEvent::OnRotationGesture {
            window_width: width,
            window_height: height,
            ..
        }
        | PointerEvent::OnDoubleTapGesture {
            window_width: width,
            window_height: height,
            ..
        } => {
            *width = window_width;
            *height = window_height;
        }
    }
}

pub fn select_trace_payload(
    config: PointerTraceConfig,
    frame_index: u64,
    window_id: u32,
    pointer_id: Option<u64>,
    full: PointerEventTrace,
) -> Option<PointerEventTrace> {
    if !trace_is_sampled(config, frame_index, window_id, pointer_id) {
        return None;
    }
    match config.level {
        PointerTraceLevel::Off => None,
        PointerTraceLevel::Errors => trace_contains_error(&full).then_some(full),
        PointerTraceLevel::Basic => Some(PointerEventTrace {
            window_id: full.window_id,
            realm_id: full.realm_id,
            target_id: full.target_id,
            connector_id: None,
            source_realm_id: None,
            uv: None,
            hops: Vec::new(),
        }),
        PointerTraceLevel::Full => Some(full),
    }
}

pub fn update_capture_state(
    captures: &mut HashMap<(u32, u64), InputCapture>,
    window_id: u32,
    pointer_id: u64,
    connector_id: Option<u32>,
    target_id: Option<u64>,
    event: &PointerEvent,
) {
    match event {
        PointerEvent::OnButton {
            state: ElementState::Pressed,
            ..
        } => {
            if let Some(connector_id) = connector_id {
                captures.insert(
                    (window_id, pointer_id),
                    InputCapture {
                        connector_id,
                        target_id,
                    },
                );
            }
        }
        PointerEvent::OnButton {
            state: ElementState::Released,
            ..
        } => {
            captures.remove(&(window_id, pointer_id));
        }
        PointerEvent::OnTouch { phase, .. } => match phase {
            TouchPhase::Started | TouchPhase::Moved => {
                if let Some(connector_id) = connector_id {
                    captures.insert(
                        (window_id, pointer_id),
                        InputCapture {
                            connector_id,
                            target_id,
                        },
                    );
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                captures.remove(&(window_id, pointer_id));
            }
        },
        _ => {}
    }
}

pub fn update_focus_state(
    focus_targets: &mut HashMap<u32, u64>,
    window_id: u32,
    target_id: Option<u64>,
    event: &PointerEvent,
) {
    match event {
        PointerEvent::OnButton {
            state: ElementState::Pressed,
            ..
        }
        | PointerEvent::OnTouch {
            phase: TouchPhase::Started,
            ..
        } => {
            if let Some(target_id) = target_id {
                focus_targets.insert(window_id, target_id);
            }
        }
        PointerEvent::OnButton {
            state: ElementState::Released,
            ..
        }
        | PointerEvent::OnTouch {
            phase: TouchPhase::Ended | TouchPhase::Cancelled,
            ..
        } => {
            focus_targets.remove(&window_id);
        }
        _ => {}
    }
}

pub fn quantize_uv(value: f32) -> u16 {
    let clamped = value.clamp(0.0, 1.0);
    (clamped * 1024.0).round() as u16
}

pub fn resolve_hit_connector(
    connectors: Option<&Vec<InputRoutingConnectorHit>>,
    position: Vec2,
    window_size: Option<glam::UVec2>,
) -> Option<HitResult> {
    const INPUT_FLAG_RAYCAST: u32 = 1 << 0;

    let connectors = connectors?;
    let target_size = window_size.unwrap_or_else(|| glam::UVec2::new(1, 1));
    for connector in connectors {
        if connector.state.input_flags & INPUT_FLAG_RAYCAST != 0 {
            if hit_test_connector(
                position,
                connector.state.rect,
                connector.state.clip,
                connector.source_size,
                target_size,
            ) {
                let uv = resolve_connector_uv_from_sizes(
                    connector.state.rect,
                    connector.state.clip,
                    position,
                    connector.source_size,
                    target_size,
                );
                return Some(HitResult {
                    connector_id: connector.id,
                    uv,
                });
            }
            continue;
        }
        if hit_test_connector(
            position,
            connector.state.rect,
            connector.state.clip,
            connector.source_size,
            target_size,
        ) {
            return Some(HitResult {
                connector_id: connector.id,
                uv: None,
            });
        }
    }
    None
}

pub fn resolve_captured_connector(
    captures: &HashMap<(u32, u64), InputCapture>,
    window_id: u32,
    pointer_id: u64,
) -> Option<ResolvedInputCapture> {
    captures
        .get(&(window_id, pointer_id))
        .map(|capture| ResolvedInputCapture {
            connector_id: ConnectorId(capture.connector_id),
            target_id: capture.target_id.map(vulfram_realm_core::TargetId),
        })
}

pub fn resolve_focus_target(
    focus_targets: &HashMap<u32, vulfram_realm_core::TargetId>,
    window_id: u32,
) -> Option<vulfram_realm_core::TargetId> {
    focus_targets.get(&window_id).copied()
}

pub fn resolve_connector_for_target(
    connectors: Option<&Vec<InputRoutingConnectorHit>>,
    target_id: vulfram_realm_core::TargetId,
) -> Option<ConnectorId> {
    let connectors = connectors?;
    for connector in connectors {
        if connector.target_id == Some(target_id) {
            return Some(connector.id);
        }
    }
    None
}

pub fn resolve_target_relative_position(
    sizing: InputTargetSizing,
    uv: Option<Vec2>,
) -> Option<Vec2> {
    let uv = uv?;
    let size = sizing.source_realm_size.or(sizing.connector_source_size)?;
    Some(Vec2::new(
        uv.x.clamp(0.0, 1.0) * size.x.max(1) as f32,
        uv.y.clamp(0.0, 1.0) * size.y.max(1) as f32,
    ))
}

pub fn resolve_target_size(sizing: InputTargetSizing) -> Option<glam::UVec2> {
    sizing
        .source_realm_size
        .or(sizing.connector_source_size)
        .or(sizing.target_surface_size)
        .or(sizing.target_declared_size)
}

pub fn build_input_routing_cache(snapshot: &InputRoutingTopologySnapshot) -> InputRoutingCache {
    let realm_by_surface = snapshot
        .realms
        .iter()
        .filter_map(|realm| {
            realm
                .output_surface
                .map(|surface_id| (surface_id, realm.realm_id))
        })
        .collect::<HashMap<_, _>>();

    let mut realm_by_window = HashMap::new();
    for present in &snapshot.presents {
        if let Some(realm_id) = realm_by_surface.get(&present.surface_id).copied() {
            realm_by_window.insert(present.window_id, (realm_id, present.surface_id));
        }
    }

    let target_rank = snapshot
        .target_order
        .iter()
        .map(|entry| (entry.target_id, entry.rank))
        .collect::<HashMap<_, _>>();

    let connector_targets = snapshot
        .auto_links
        .iter()
        .map(|entry| (entry.connector_id, entry.target_id))
        .collect::<HashMap<_, _>>();

    let layer_camera_by_key = snapshot
        .layer_cameras
        .iter()
        .map(|entry| ((entry.realm_id, entry.target_id), entry.camera_id))
        .collect::<HashMap<_, _>>();

    let mut connectors_by_realm: HashMap<RealmId, Vec<InputRoutingConnectorHit>> = HashMap::new();
    for connector in &snapshot.connectors {
        let target_id = connector_targets.get(&connector.connector_id).copied();
        let rank = target_id
            .and_then(|id| target_rank.get(&id).copied())
            .unwrap_or(-1);
        connectors_by_realm
            .entry(connector.state.target_realm)
            .or_default()
            .push(InputRoutingConnectorHit {
                id: connector.connector_id,
                state: connector.state.clone(),
                source_size: connector.source_size,
                target_id,
                target_rank: rank,
            });
    }

    for connectors in connectors_by_realm.values_mut() {
        connectors.sort_by(|a, b| {
            let z_cmp = b.state.z_index.cmp(&a.state.z_index);
            if z_cmp == std::cmp::Ordering::Equal {
                let rank_cmp = b.target_rank.cmp(&a.target_rank);
                if rank_cmp == std::cmp::Ordering::Equal {
                    b.id.0.cmp(&a.id.0)
                } else {
                    rank_cmp
                }
            } else {
                z_cmp
            }
        });
    }

    InputRoutingCache {
        topology_hash: compute_input_topology_hash(snapshot),
        realm_by_surface,
        realm_by_window,
        connector_targets,
        layer_camera_by_key,
        connectors_by_realm,
    }
}

pub fn compute_input_topology_hash(snapshot: &InputRoutingTopologySnapshot) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    snapshot.realms.len().hash(&mut hasher);
    for realm in &snapshot.realms {
        realm.realm_id.hash(&mut hasher);
        realm.output_surface.hash(&mut hasher);
    }

    snapshot.presents.len().hash(&mut hasher);
    for present in &snapshot.presents {
        present.window_id.hash(&mut hasher);
        present.surface_id.hash(&mut hasher);
    }

    for target in &snapshot.target_order {
        target.target_id.hash(&mut hasher);
        target.rank.hash(&mut hasher);
    }

    for link in &snapshot.auto_links {
        link.target_id.hash(&mut hasher);
        link.connector_id.hash(&mut hasher);
    }

    for layer in &snapshot.layer_cameras {
        layer.realm_id.hash(&mut hasher);
        layer.target_id.hash(&mut hasher);
        layer.camera_id.hash(&mut hasher);
    }

    for connector in &snapshot.connectors {
        connector.connector_id.hash(&mut hasher);
        connector.state.target_realm.hash(&mut hasher);
        connector.state.source_surface.hash(&mut hasher);
        connector.state.z_index.hash(&mut hasher);
        connector.state.blend_mode.hash(&mut hasher);
        connector.state.input_flags.hash(&mut hasher);
        connector.state.rect.x.to_bits().hash(&mut hasher);
        connector.state.rect.y.to_bits().hash(&mut hasher);
        connector.state.rect.z.to_bits().hash(&mut hasher);
        connector.state.rect.w.to_bits().hash(&mut hasher);
        if let Some(clip) = connector.state.clip {
            clip.x.to_bits().hash(&mut hasher);
            clip.y.to_bits().hash(&mut hasher);
            clip.z.to_bits().hash(&mut hasher);
            clip.w.to_bits().hash(&mut hasher);
        }
    }

    for surface in &snapshot.surfaces {
        surface.surface_id.hash(&mut hasher);
        surface.size.x.hash(&mut hasher);
        surface.size.y.hash(&mut hasher);
    }

    hasher.finish()
}

pub fn resolve_connector_uv_from_sizes(
    rect: glam::Vec4,
    clip: Option<glam::Vec4>,
    position: Vec2,
    source_size: glam::UVec2,
    target_size: glam::UVec2,
) -> Option<Vec2> {
    let (viewport, _) = resolve_overlay_geometry(rect, clip, source_size, target_size)?;
    let u = ((position.x - viewport.x) / viewport.z.max(1.0)).clamp(0.0, 1.0);
    let v = ((position.y - viewport.y) / viewport.w.max(1.0)).clamp(0.0, 1.0);
    Some(Vec2::new(u, v))
}

fn trace_contains_error(trace: &PointerEventTrace) -> bool {
    trace.hops.iter().any(|hop| {
        matches!(
            hop.stage,
            PointerTraceStage::StopStepBudget | PointerTraceStage::StopCycle
        )
    })
}

fn trace_is_sampled(
    config: PointerTraceConfig,
    frame_index: u64,
    window_id: u32,
    pointer_id: Option<u64>,
) -> bool {
    let percent = config.sampling_percent.min(100);
    if percent == 0 {
        return false;
    }
    if percent == 100 {
        return true;
    }
    let seed = frame_index
        ^ window_id as u64
        ^ pointer_id
            .unwrap_or_default()
            .wrapping_mul(11400714819323198485);
    seed % 100 < percent as u64
}

fn hit_test_connector(
    position: Vec2,
    rect: glam::Vec4,
    clip: Option<glam::Vec4>,
    source_size: glam::UVec2,
    target_size: glam::UVec2,
) -> bool {
    let Some((viewport, clip_rect)) =
        resolve_overlay_geometry(rect, clip, source_size, target_size)
    else {
        return false;
    };

    let inside_viewport = position.x >= viewport.x
        && position.y >= viewport.y
        && position.x <= viewport.x + viewport.z
        && position.y <= viewport.y + viewport.w;
    let inside_clip = position.x >= clip_rect.x
        && position.y >= clip_rect.y
        && position.x <= clip_rect.x + clip_rect.z
        && position.y <= clip_rect.y + clip_rect.w;
    inside_viewport && inside_clip
}

fn resolve_overlay_geometry(
    rect: glam::Vec4,
    clip: Option<glam::Vec4>,
    source_size: glam::UVec2,
    target_size: glam::UVec2,
) -> Option<(glam::Vec4, glam::Vec4)> {
    if rect.z <= 0.0 || rect.w <= 0.0 {
        return None;
    }

    let source_width = source_size.x.max(1) as f32;
    let source_height = source_size.y.max(1) as f32;
    let scale = rect.w / source_height;
    let draw_width = (source_width * scale).max(1.0);

    let mut viewport_x = rect.x + (rect.z - draw_width) * 0.5;
    let mut viewport_y = rect.y;
    let mut viewport_width = draw_width;
    let mut viewport_height = rect.w.max(1.0);

    if viewport_x < 0.0 {
        viewport_width = (viewport_width + viewport_x).max(0.0);
        viewport_x = 0.0;
    }
    if viewport_y < 0.0 {
        viewport_height = (viewport_height + viewport_y).max(0.0);
        viewport_y = 0.0;
    }

    let max_width = target_size.x as f32 - viewport_x;
    let max_height = target_size.y as f32 - viewport_y;
    if max_width <= 0.0 || max_height <= 0.0 {
        return None;
    }
    viewport_width = viewport_width.min(max_width);
    viewport_height = viewport_height.min(max_height);
    if viewport_width <= 0.0 || viewport_height <= 0.0 {
        return None;
    }

    let viewport = glam::Vec4::new(viewport_x, viewport_y, viewport_width, viewport_height);
    let mut clip_rect = rect;
    if let Some(clip) = clip {
        clip_rect = intersect_rect(clip_rect, clip);
    }
    Some((viewport, clip_rect))
}

fn intersect_rect(a: glam::Vec4, b: glam::Vec4) -> glam::Vec4 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.z).min(b.x + b.z);
    let y2 = (a.y + a.w).min(b.y + b.w);
    glam::Vec4::new(x1, y1, (x2 - x1).max(0.0), (y2 - y1).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use vulfram_realm_core::ConnectorState;
    use vulfram_types::ConnectorId;

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

    #[test]
    fn input_target_listener_upsert_defaults_are_applied() {
        let decoded: CmdInputTargetListenerUpsertArgs =
            serde_json::from_str(r#"{ "listenerId": 1, "targetId": 2 }"#)
                .expect("listener args should decode");

        assert!(decoded.enabled);
        assert_eq!(decoded.sample_percent, 100);
        assert!(decoded.events.is_empty());
    }

    #[test]
    fn input_target_listener_store_lists_sorted_snapshots() {
        let mut store = InputTargetListenerStore::default();
        store.upsert(InputTargetListenerConfig {
            listener_id: 20,
            target_id: 7,
            enabled: true,
            events: vec!["pointer-move".into()],
            sample_percent: 100,
        });
        store.upsert(InputTargetListenerConfig {
            listener_id: 10,
            target_id: 7,
            enabled: false,
            events: Vec::new(),
            sample_percent: 0,
        });

        let listeners = store.list(Some(7));
        assert_eq!(listeners.len(), 2);
        assert_eq!(listeners[0].listener_id, 10);
        assert_eq!(listeners[1].listener_id, 20);
    }

    #[test]
    fn input_target_listener_store_disposes_target_group() {
        let mut store = InputTargetListenerStore::default();
        store.upsert(InputTargetListenerConfig {
            listener_id: 1,
            target_id: 10,
            enabled: true,
            events: Vec::new(),
            sample_percent: 100,
        });
        store.upsert(InputTargetListenerConfig {
            listener_id: 2,
            target_id: 10,
            enabled: true,
            events: Vec::new(),
            sample_percent: 100,
        });
        store.upsert(InputTargetListenerConfig {
            listener_id: 3,
            target_id: 11,
            enabled: true,
            events: Vec::new(),
            sample_percent: 100,
        });

        assert_eq!(store.dispose_target(10), 2);
        assert_eq!(store.list(None).len(), 1);
        assert_eq!(store.listeners_for_target(10).len(), 0);
        assert_eq!(store.listeners_for_target(11).len(), 1);
    }

    #[test]
    fn select_trace_payload_basic_strips_detailed_fields() {
        let full = PointerEventTrace {
            window_id: 1,
            realm_id: 2,
            target_id: Some(3),
            connector_id: Some(4),
            source_realm_id: Some(5),
            uv: Some(Vec2::new(0.25, 0.75)),
            hops: vec![PointerTraceHop {
                stage: PointerTraceStage::ConnectorHit,
                realm_id: Some(2),
                target_id: Some(3),
                layer_realm_id: Some(2),
                connector_id: Some(4),
                surface_id: Some(6),
                camera_id: Some(7),
                uv: Some(Vec2::new(0.25, 0.75)),
            }],
        };

        let trace = select_trace_payload(
            PointerTraceConfig {
                level: PointerTraceLevel::Basic,
                sampling_percent: 100,
            },
            0,
            1,
            Some(9),
            full,
        )
        .expect("basic trace should be present");

        assert_eq!(trace.window_id, 1);
        assert_eq!(trace.realm_id, 2);
        assert_eq!(trace.target_id, Some(3));
        assert!(trace.connector_id.is_none());
        assert!(trace.source_realm_id.is_none());
        assert!(trace.uv.is_none());
        assert!(trace.hops.is_empty());
    }

    #[test]
    fn update_focus_state_tracks_press_and_release() {
        let mut focus_targets = HashMap::new();
        let event = PointerEvent::OnButton {
            window_id: 1,
            window_width: None,
            window_height: None,
            pointer_type: 0,
            pointer_id: 10,
            button: 0,
            state: ElementState::Pressed,
            position: Vec2::new(0.0, 0.0),
            position_target: None,
            target_width: None,
            target_height: None,
            trace: None,
        };

        update_focus_state(&mut focus_targets, 1, Some(42), &event);
        assert_eq!(focus_targets.get(&1), Some(&42));

        let release = PointerEvent::OnButton {
            window_id: 1,
            window_width: None,
            window_height: None,
            pointer_type: 0,
            pointer_id: 10,
            button: 0,
            state: ElementState::Released,
            position: Vec2::new(0.0, 0.0),
            position_target: None,
            target_width: None,
            target_height: None,
            trace: None,
        };
        update_focus_state(&mut focus_targets, 1, Some(42), &release);
        assert!(focus_targets.is_empty());
    }

    #[test]
    fn input_routing_state_defaults_to_empty_maps_and_full_trace() {
        let routing = InputRoutingState::default();

        assert!(routing.captures.is_empty());
        assert!(routing.focus_targets.is_empty());
        assert_eq!(routing.trace.level, PointerTraceLevel::Full);
        assert_eq!(routing.trace.sampling_percent, 100);
        assert_eq!(routing.cache.topology_hash, 0);
        assert!(routing.cache.connectors_by_realm.is_empty());
    }

    #[test]
    fn resolve_hit_connector_returns_uv_for_raycast_connectors() {
        let connectors = vec![InputRoutingConnectorHit {
            id: ConnectorId(7),
            state: ConnectorState {
                source_surface: SurfaceId(1),
                target_realm: RealmId(2),
                rect: glam::Vec4::new(0.0, 0.0, 100.0, 100.0),
                clip: None,
                z_index: 0,
                blend_mode: 0,
                input_flags: 1,
            },
            source_size: glam::UVec2::new(100, 100),
            target_id: None,
            target_rank: 0,
        }];

        let hit = resolve_hit_connector(
            Some(&connectors),
            Vec2::new(50.0, 50.0),
            Some(glam::UVec2::new(100, 100)),
        )
        .expect("connector should hit");

        assert_eq!(hit.connector_id, ConnectorId(7));
        assert_eq!(hit.uv, Some(Vec2::new(0.5, 0.5)));
    }

    #[test]
    fn resolve_captured_connector_maps_primitives_to_ids() {
        let mut captures = HashMap::new();
        captures.insert(
            (1, 2),
            InputCapture {
                connector_id: 9,
                target_id: Some(11),
            },
        );

        let resolved = resolve_captured_connector(&captures, 1, 2).expect("capture should resolve");

        assert_eq!(resolved.connector_id, ConnectorId(9));
        assert_eq!(resolved.target_id, Some(vulfram_realm_core::TargetId(11)));
    }

    #[test]
    fn resolve_connector_for_target_finds_matching_entry() {
        let connectors = vec![InputRoutingConnectorHit {
            id: ConnectorId(7),
            state: ConnectorState {
                source_surface: SurfaceId(1),
                target_realm: RealmId(2),
                rect: glam::Vec4::new(0.0, 0.0, 100.0, 100.0),
                clip: None,
                z_index: 0,
                blend_mode: 0,
                input_flags: 0,
            },
            source_size: glam::UVec2::new(100, 100),
            target_id: Some(vulfram_realm_core::TargetId(42)),
            target_rank: 0,
        }];

        let connector_id =
            resolve_connector_for_target(Some(&connectors), vulfram_realm_core::TargetId(42));

        assert_eq!(connector_id, Some(ConnectorId(7)));
    }

    #[test]
    fn build_input_routing_cache_sorts_connectors_by_z_then_rank() {
        let snapshot = InputRoutingTopologySnapshot {
            realms: vec![InputRoutingRealmOutput {
                realm_id: RealmId(1),
                output_surface: Some(SurfaceId(5)),
            }],
            presents: vec![InputRoutingPresentBinding {
                window_id: 9,
                surface_id: SurfaceId(5),
            }],
            target_order: vec![
                InputRoutingTargetRank {
                    target_id: vulfram_realm_core::TargetId(100),
                    rank: 0,
                },
                InputRoutingTargetRank {
                    target_id: vulfram_realm_core::TargetId(200),
                    rank: 1,
                },
            ],
            auto_links: vec![
                InputRoutingAutoLinkRecord {
                    target_id: vulfram_realm_core::TargetId(100),
                    connector_id: ConnectorId(1),
                },
                InputRoutingAutoLinkRecord {
                    target_id: vulfram_realm_core::TargetId(200),
                    connector_id: ConnectorId(2),
                },
            ],
            layer_cameras: Vec::new(),
            connectors: vec![
                InputRoutingConnectorRecord {
                    connector_id: ConnectorId(1),
                    state: ConnectorState {
                        source_surface: SurfaceId(7),
                        target_realm: RealmId(1),
                        rect: glam::Vec4::new(0.0, 0.0, 10.0, 10.0),
                        z_index: 2,
                        blend_mode: 0,
                        clip: None,
                        input_flags: 0,
                    },
                    source_size: glam::UVec2::new(10, 10),
                },
                InputRoutingConnectorRecord {
                    connector_id: ConnectorId(2),
                    state: ConnectorState {
                        source_surface: SurfaceId(8),
                        target_realm: RealmId(1),
                        rect: glam::Vec4::new(0.0, 0.0, 10.0, 10.0),
                        z_index: 2,
                        blend_mode: 0,
                        clip: None,
                        input_flags: 0,
                    },
                    source_size: glam::UVec2::new(10, 10),
                },
            ],
            surfaces: vec![
                InputRoutingSurfaceSizeRecord {
                    surface_id: SurfaceId(5),
                    size: glam::UVec2::new(100, 100),
                },
                InputRoutingSurfaceSizeRecord {
                    surface_id: SurfaceId(7),
                    size: glam::UVec2::new(10, 10),
                },
                InputRoutingSurfaceSizeRecord {
                    surface_id: SurfaceId(8),
                    size: glam::UVec2::new(10, 10),
                },
            ],
        };

        let cache = build_input_routing_cache(&snapshot);
        let connectors = cache
            .connectors_by_realm
            .get(&RealmId(1))
            .expect("realm connectors should exist");

        assert_eq!(
            cache.realm_by_window.get(&9),
            Some(&(RealmId(1), SurfaceId(5)))
        );
        assert_eq!(connectors[0].id, ConnectorId(2));
        assert_eq!(connectors[1].id, ConnectorId(1));
    }

    #[test]
    fn resolve_target_relative_position_uses_first_runtime_size() {
        let position = resolve_target_relative_position(
            InputTargetSizing {
                source_realm_size: Some(glam::UVec2::new(200, 100)),
                connector_source_size: Some(glam::UVec2::new(10, 10)),
                target_surface_size: None,
                target_declared_size: None,
            },
            Some(Vec2::new(0.25, 0.5)),
        );

        assert_eq!(position, Some(Vec2::new(50.0, 50.0)));
    }

    #[test]
    fn resolve_target_size_falls_back_through_all_sources() {
        assert_eq!(
            resolve_target_size(InputTargetSizing {
                source_realm_size: None,
                connector_source_size: None,
                target_surface_size: Some(glam::UVec2::new(300, 200)),
                target_declared_size: Some(glam::UVec2::new(10, 10)),
            }),
            Some(glam::UVec2::new(300, 200))
        );

        assert_eq!(
            resolve_target_size(InputTargetSizing {
                source_realm_size: None,
                connector_source_size: None,
                target_surface_size: None,
                target_declared_size: Some(glam::UVec2::new(10, 10)),
            }),
            Some(glam::UVec2::new(10, 10))
        );
    }
}
