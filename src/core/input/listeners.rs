use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::core::cmd::CmdResultSimple;
use crate::core::cmd::EngineEvent;
use crate::core::input::events::{KeyboardEvent, PointerEvent};
use crate::core::state::EngineState;
use crate::core::system::SystemEvent;
use crate::core::target::TargetId;

fn default_true() -> bool {
    true
}

fn default_scope() -> TargetListenerScope {
    TargetListenerScope::Target
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TargetListenerScope {
    Target,
    TargetAndDescendants,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CmdInputTargetListenerUpsertArgs {
    pub listener_id: u64,
    pub target_id: u64,
    #[serde(default)]
    pub window_id: Option<u32>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default = "default_scope")]
    pub scope: TargetListenerScope,
    #[serde(default)]
    pub throttle_ms: u32,
    #[serde(default = "u8_100")]
    pub sample_percent: u8,
}

const fn u8_100() -> u8 {
    100
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
    pub window_id: Option<u32>,
    pub enabled: bool,
    pub events: Vec<String>,
    pub scope: TargetListenerScope,
    pub throttle_ms: u32,
    pub sample_percent: u8,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultInputTargetListenerList {
    pub success: bool,
    pub message: String,
    pub listeners: Vec<InputTargetListenerSnapshot>,
}

#[derive(Debug, Clone)]
pub struct InputTargetListenerConfig {
    pub listener_id: u64,
    pub target_id: TargetId,
    pub window_id: Option<u32>,
    pub enabled: bool,
    pub events: Vec<String>,
    pub scope: TargetListenerScope,
    pub throttle_ms: u32,
    pub sample_percent: u8,
}

#[derive(Debug, Default)]
pub struct InputTargetListenerStore {
    by_listener: HashMap<u64, InputTargetListenerConfig>,
    listeners_by_target: HashMap<TargetId, Vec<u64>>,
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

    pub fn dispose_target(&mut self, target_id: TargetId) -> usize {
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

    pub fn dispose_targets<I: IntoIterator<Item = TargetId>>(&mut self, target_ids: I) -> usize {
        target_ids
            .into_iter()
            .map(|target_id| self.dispose_target(target_id))
            .sum()
    }

    pub fn list(&self, target_id: Option<TargetId>) -> Vec<InputTargetListenerSnapshot> {
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

    pub fn listeners_for_target(&self, target_id: TargetId) -> Vec<InputTargetListenerConfig> {
        self.listeners_by_target
            .get(&target_id)
            .into_iter()
            .flat_map(|listener_ids| listener_ids.iter())
            .filter_map(|listener_id| self.by_listener.get(listener_id).cloned())
            .collect()
    }

    fn remove_from_target_index(&mut self, target_id: TargetId, listener_id: u64) {
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
        target_id: config.target_id.0,
        window_id: config.window_id,
        enabled: config.enabled,
        events: config.events.clone(),
        scope: config.scope,
        throttle_ms: config.throttle_ms,
        sample_percent: config.sample_percent,
    }
}

pub fn engine_cmd_input_target_listener_upsert(
    engine: &mut EngineState,
    args: &CmdInputTargetListenerUpsertArgs,
) -> CmdResultSimple {
    engine
        .universal_state
        .target_listeners
        .upsert(InputTargetListenerConfig {
            listener_id: args.listener_id,
            target_id: TargetId(args.target_id),
            window_id: args.window_id,
            enabled: args.enabled,
            events: args.events.clone(),
            scope: args.scope,
            throttle_ms: args.throttle_ms,
            sample_percent: args.sample_percent.min(100),
        });
    CmdResultSimple {
        success: true,
        message: "Input target listener upserted".into(),
    }
}

pub fn engine_cmd_input_target_listener_dispose(
    engine: &mut EngineState,
    args: &CmdInputTargetListenerDisposeArgs,
) -> CmdResultSimple {
    let removed = engine
        .universal_state
        .target_listeners
        .dispose(args.listener_id);
    CmdResultSimple {
        success: true,
        message: if removed {
            "Input target listener disposed".into()
        } else {
            "Input target listener not found (no-op)".into()
        },
    }
}

pub fn engine_cmd_input_target_listener_list(
    engine: &mut EngineState,
    args: &CmdInputTargetListenerListArgs,
) -> CmdResultInputTargetListenerList {
    let listeners = engine
        .universal_state
        .target_listeners
        .list(args.target_id.map(TargetId));
    CmdResultInputTargetListenerList {
        success: true,
        message: "Input target listeners listed".into(),
        listeners,
    }
}

pub fn emit_target_listener_events(engine: &mut EngineState) {
    let source_events = engine.event_queue.clone();
    for event in source_events {
        match event {
            EngineEvent::Pointer(pointer_event) => {
                emit_pointer_listener_events(engine, &pointer_event);
            }
            EngineEvent::Keyboard(keyboard_event) => {
                emit_keyboard_listener_events(engine, &keyboard_event);
            }
            _ => {}
        }
    }
}

fn emit_pointer_listener_events(engine: &mut EngineState, event: &PointerEvent) {
    let (event_type, target_id, window_id, pointer_id, position_global, position_target) =
        match event {
            PointerEvent::OnMove {
                trace,
                window_id,
                pointer_id,
                position,
                position_target,
                ..
            } => (
                "pointer-move",
                trace.as_ref().and_then(|trace| trace.target_id),
                Some(*window_id),
                Some(*pointer_id),
                Some(*position),
                *position_target,
            ),
            PointerEvent::OnButton {
                trace,
                window_id,
                pointer_id,
                position,
                position_target,
                ..
            } => (
                "pointer-button",
                trace.as_ref().and_then(|trace| trace.target_id),
                Some(*window_id),
                Some(*pointer_id),
                Some(*position),
                *position_target,
            ),
            PointerEvent::OnTouch {
                trace,
                window_id,
                pointer_id,
                position,
                position_target,
                ..
            } => (
                "pointer-touch",
                trace.as_ref().and_then(|trace| trace.target_id),
                Some(*window_id),
                Some(*pointer_id),
                Some(*position),
                *position_target,
            ),
            PointerEvent::OnScroll {
                trace, window_id, ..
            } => (
                "pointer-scroll",
                trace.as_ref().and_then(|trace| trace.target_id),
                Some(*window_id),
                None,
                None,
                None,
            ),
            PointerEvent::OnEnter {
                trace,
                window_id,
                pointer_id,
                ..
            } => (
                "pointer-enter",
                trace.as_ref().and_then(|trace| trace.target_id),
                Some(*window_id),
                Some(*pointer_id),
                None,
                None,
            ),
            PointerEvent::OnLeave {
                trace,
                window_id,
                pointer_id,
                ..
            } => (
                "pointer-leave",
                trace.as_ref().and_then(|trace| trace.target_id),
                Some(*window_id),
                Some(*pointer_id),
                None,
                None,
            ),
            PointerEvent::OnPinchGesture {
                trace, window_id, ..
            } => (
                "pointer-pinch",
                trace.as_ref().and_then(|trace| trace.target_id),
                Some(*window_id),
                None,
                None,
                None,
            ),
            PointerEvent::OnPanGesture {
                trace, window_id, ..
            } => (
                "pointer-pan",
                trace.as_ref().and_then(|trace| trace.target_id),
                Some(*window_id),
                None,
                None,
                None,
            ),
            PointerEvent::OnRotationGesture {
                trace, window_id, ..
            } => (
                "pointer-rotation",
                trace.as_ref().and_then(|trace| trace.target_id),
                Some(*window_id),
                None,
                None,
                None,
            ),
            PointerEvent::OnDoubleTapGesture {
                trace, window_id, ..
            } => (
                "pointer-double-tap",
                trace.as_ref().and_then(|trace| trace.target_id),
                Some(*window_id),
                None,
                None,
                None,
            ),
        };
    let Some(target_id) = target_id else {
        return;
    };

    let listeners = engine
        .universal_state
        .target_listeners
        .listeners_for_target(TargetId(target_id));
    for listener in listeners {
        if !listener_matches(&listener, event_type, engine.state_frame_index()) {
            continue;
        }
        engine
            .event_queue
            .push(EngineEvent::System(SystemEvent::InputTargetListenerEvent {
                listener_id: listener.listener_id,
                target_id,
                event_type: event_type.to_string(),
                window_id,
                pointer_id,
                position_global,
                position_target,
                key_code: None,
                key_state: None,
            }));
    }
}

fn emit_keyboard_listener_events(engine: &mut EngineState, event: &KeyboardEvent) {
    let (event_type, window_id, key_code, key_state) = match event {
        KeyboardEvent::OnInput {
            window_id,
            key_code,
            state,
            ..
        } => (
            "keyboard-input",
            Some(*window_id),
            Some(*key_code),
            Some(*state),
        ),
        KeyboardEvent::OnModifiersChange { window_id, .. } => {
            ("keyboard-modifiers", Some(*window_id), None, None)
        }
        KeyboardEvent::OnImeEnable { window_id } => {
            ("keyboard-ime-enable", Some(*window_id), None, None)
        }
        KeyboardEvent::OnImePreedit { window_id, .. } => {
            ("keyboard-ime-preedit", Some(*window_id), None, None)
        }
        KeyboardEvent::OnImeCommit { window_id, .. } => {
            ("keyboard-ime-commit", Some(*window_id), None, None)
        }
        KeyboardEvent::OnImeDisable { window_id } => {
            ("keyboard-ime-disable", Some(*window_id), None, None)
        }
    };

    let Some(window_id) = window_id else {
        return;
    };
    let Some(target_id) = engine
        .universal_state
        .input_routing
        .focus_targets
        .get(&window_id)
        .copied()
    else {
        return;
    };

    let listeners = engine
        .universal_state
        .target_listeners
        .listeners_for_target(target_id);
    for listener in listeners {
        if !listener_matches(&listener, event_type, engine.state_frame_index()) {
            continue;
        }
        engine
            .event_queue
            .push(EngineEvent::System(SystemEvent::InputTargetListenerEvent {
                listener_id: listener.listener_id,
                target_id: target_id.0,
                event_type: event_type.to_string(),
                window_id: Some(window_id),
                pointer_id: None,
                position_global: None,
                position_target: None,
                key_code,
                key_state,
            }));
    }
}

fn listener_matches(
    listener: &InputTargetListenerConfig,
    event_type: &str,
    frame_index: u64,
) -> bool {
    if !listener.enabled {
        return false;
    }
    if !listener.events.is_empty()
        && !listener
            .events
            .iter()
            .any(|configured| configured == event_type)
    {
        return false;
    }
    let sample = listener.sample_percent.min(100);
    if sample == 0 {
        return false;
    }
    if sample == 100 {
        return true;
    }
    let mut hasher = DefaultHasher::new();
    listener.listener_id.hash(&mut hasher);
    listener.target_id.hash(&mut hasher);
    event_type.hash(&mut hasher);
    frame_index.hash(&mut hasher);
    (hasher.finish() % 100) < sample as u64
}

trait EngineStateFrameIndex {
    fn state_frame_index(&self) -> u64;
}

impl EngineStateFrameIndex for EngineState {
    fn state_frame_index(&self) -> u64 {
        self.frame_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::cmd::EngineEvent;
    use crate::core::input::events::PointerEventTrace;

    #[test]
    fn listener_emits_system_event_for_matching_pointer_target() {
        let mut engine = EngineState::new();
        engine
            .universal_state
            .target_listeners
            .upsert(InputTargetListenerConfig {
                listener_id: 10,
                target_id: TargetId(99),
                window_id: Some(1),
                enabled: true,
                events: vec!["pointer-move".into()],
                scope: TargetListenerScope::Target,
                throttle_ms: 0,
                sample_percent: 100,
            });
        engine
            .event_queue
            .push(EngineEvent::Pointer(PointerEvent::OnMove {
                window_id: 1,
                pointer_type: 0,
                pointer_id: 1,
                position: glam::vec2(1.0, 2.0),
                position_target: Some(glam::vec2(3.0, 4.0)),
                trace: Some(PointerEventTrace {
                    window_id: 1,
                    realm_id: 0,
                    target_id: Some(99),
                    connector_id: None,
                    source_realm_id: None,
                    uv: None,
                    hops: Vec::new(),
                }),
            }));

        emit_target_listener_events(&mut engine);

        assert!(engine.event_queue.iter().any(|event| {
            matches!(
                event,
                EngineEvent::System(SystemEvent::InputTargetListenerEvent {
                    listener_id: 10,
                    target_id: 99,
                    ..
                })
            )
        }));
    }
}
