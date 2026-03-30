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
}
