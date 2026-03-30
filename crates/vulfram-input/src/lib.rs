use std::collections::HashMap;

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
}
