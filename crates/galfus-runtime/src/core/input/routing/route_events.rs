use std::collections::HashMap;

use crate::core::input::events::PointerEvent;
use crate::core::realm::{ConnectorId, InputCapture};
use crate::core::target::TargetId;

pub(super) use galfus_input::{
    apply_target_position, apply_target_size, apply_trace, apply_window_size, pointer_id,
    pointer_position, pointer_window_id, quantize_uv, select_trace_payload,
};

pub(super) fn update_capture_state(
    captures: &mut HashMap<(u32, u64), InputCapture>,
    window_id: u32,
    pointer_id: u64,
    connector_id: Option<ConnectorId>,
    target_id: Option<TargetId>,
    event: &PointerEvent,
) {
    galfus_input::update_capture_state(
        captures,
        window_id,
        pointer_id,
        connector_id.map(|id| id.0),
        target_id.map(|id| id.0),
        event,
    );
}

pub(super) fn update_focus_state(
    focus_targets: &mut HashMap<u32, TargetId>,
    window_id: u32,
    target_id: Option<TargetId>,
    event: &PointerEvent,
) {
    let mut normalized_focus_targets = std::mem::take(focus_targets)
        .into_iter()
        .map(|(window_id, target_id)| (window_id, target_id.0))
        .collect::<HashMap<_, _>>();
    galfus_input::update_focus_state(
        &mut normalized_focus_targets,
        window_id,
        target_id.map(|id| id.0),
        event,
    );
    *focus_targets = normalized_focus_targets
        .into_iter()
        .map(|(window_id, target_id)| (window_id, TargetId(target_id)))
        .collect();
}
