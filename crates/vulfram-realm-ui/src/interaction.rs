use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use vulfram_input::{ElementState, PointerEvent, TouchPhase};
use vulfram_types::{RealmId, UiNodeId};

use crate::{UiDocument, UiDocumentId, UiTracedPointerDispatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiFocusUpdate {
    pub window_id: u32,
    pub realm_id: RealmId,
    pub document_id: UiDocumentId,
    pub node_id: UiNodeId,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UiFocusState {
    pub realm_by_window: HashMap<u32, RealmId>,
    pub document_by_window: HashMap<u32, UiDocumentId>,
    pub node_by_window: HashMap<u32, UiNodeId>,
    pub capture_by_window: HashMap<u32, UiCaptureEntry>,
}

impl UiFocusState {
    pub fn set_focus(&mut self, focus: UiFocusUpdate) {
        self.realm_by_window.insert(focus.window_id, focus.realm_id);
        self.document_by_window
            .insert(focus.window_id, focus.document_id);
        self.node_by_window.insert(focus.window_id, focus.node_id);
    }

    pub fn focus_realm(&self, window_id: u32) -> Option<RealmId> {
        self.realm_by_window.get(&window_id).copied()
    }

    pub fn focus_document(&self, window_id: u32) -> Option<UiDocumentId> {
        self.document_by_window.get(&window_id).copied()
    }

    pub fn focus_node(&self, window_id: u32) -> Option<UiNodeId> {
        self.node_by_window.get(&window_id).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiCaptureEntry {
    pub realm_id: RealmId,
    pub document_id: UiDocumentId,
    pub node_id: UiNodeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "action", content = "data", rename_all = "kebab-case")]
pub enum UiCaptureUpdate {
    None,
    Set {
        window_id: u32,
        capture: UiCaptureEntry,
    },
    Clear {
        window_id: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPointerPositionUpdate {
    pub realm_id: RealmId,
    pub pos: Option<glam::Vec2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiTracedPointerPumpPlan {
    pub focus_update: Option<UiFocusUpdate>,
    pub capture_update: UiCaptureUpdate,
    pub pointer_pos_update: Option<UiPointerPositionUpdate>,
}

pub fn plan_traced_pointer_pump(
    event: &PointerEvent,
    dispatch: Option<UiTracedPointerDispatch>,
    focused_realm_id: Option<RealmId>,
) -> UiTracedPointerPumpPlan {
    let window_id = pointer_event_window_id(event);
    let pointer_pos_update = match event {
        PointerEvent::OnMove { .. }
        | PointerEvent::OnEnter { .. }
        | PointerEvent::OnButton { .. }
        | PointerEvent::OnTouch { .. } => dispatch.map(|dispatch| UiPointerPositionUpdate {
            realm_id: dispatch.realm_id,
            pos: Some(dispatch.pos),
        }),
        PointerEvent::OnLeave { .. } => focused_realm_id.map(|realm_id| UiPointerPositionUpdate {
            realm_id,
            pos: None,
        }),
        _ => None,
    };

    let focus_update = match event {
        PointerEvent::OnButton {
            state: ElementState::Pressed,
            ..
        }
        | PointerEvent::OnTouch {
            phase: TouchPhase::Started,
            ..
        } => dispatch.map(|dispatch| UiFocusUpdate {
            window_id,
            realm_id: dispatch.realm_id,
            document_id: dispatch.document_id,
            node_id: 0,
        }),
        _ => None,
    };

    let capture_update = match event {
        PointerEvent::OnButton {
            state: ElementState::Pressed,
            ..
        }
        | PointerEvent::OnTouch {
            phase: TouchPhase::Started,
            ..
        } => dispatch.map_or(UiCaptureUpdate::None, |dispatch| UiCaptureUpdate::Set {
            window_id,
            capture: UiCaptureEntry {
                realm_id: dispatch.realm_id,
                document_id: dispatch.document_id,
                node_id: 0,
            },
        }),
        PointerEvent::OnButton {
            state: ElementState::Released,
            ..
        }
        | PointerEvent::OnTouch {
            phase: TouchPhase::Ended,
            ..
        }
        | PointerEvent::OnTouch {
            phase: TouchPhase::Cancelled,
            ..
        } => UiCaptureUpdate::Clear { window_id },
        _ => UiCaptureUpdate::None,
    };

    UiTracedPointerPumpPlan {
        focus_update,
        capture_update,
        pointer_pos_update,
    }
}

pub fn pointer_event_window_id(event: &PointerEvent) -> u32 {
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

pub fn prune_document_focus_links(focus_state: &mut UiFocusState, document_id: UiDocumentId) {
    focus_state
        .document_by_window
        .retain(|_, focus_document_id| *focus_document_id != document_id);
    focus_state
        .node_by_window
        .retain(|window_id, _| focus_state.document_by_window.contains_key(window_id));
    focus_state
        .capture_by_window
        .retain(|_, capture| capture.document_id != document_id);
}

pub fn prune_realm_focus_links(focus_state: &mut UiFocusState, realm_id: RealmId) {
    focus_state
        .realm_by_window
        .retain(|_, focus_realm_id| *focus_realm_id != realm_id);
    focus_state
        .capture_by_window
        .retain(|_, capture| capture.realm_id != realm_id);
}

pub fn retain_valid_focus_nodes(
    focus_state: &mut UiFocusState,
    documents: &HashMap<UiDocumentId, UiDocument>,
) {
    focus_state.node_by_window.retain(|window_id, node_id| {
        let Some(document_id) = focus_state.document_by_window.get(window_id) else {
            return false;
        };
        documents
            .get(document_id)
            .map(|document| document.nodes.contains_key(node_id))
            .unwrap_or(false)
    });
}

pub fn retain_valid_capture_entries(
    focus_state: &mut UiFocusState,
    documents: &HashMap<UiDocumentId, UiDocument>,
) {
    focus_state.capture_by_window.retain(|_, capture| {
        documents
            .get(&capture.document_id)
            .map(|document| document.nodes.contains_key(&capture.node_id))
            .unwrap_or(false)
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traced_pointer_pump_sets_focus_and_capture_on_press() {
        let plan = plan_traced_pointer_pump(
            &PointerEvent::OnButton {
                window_id: 7,
                window_width: None,
                window_height: None,
                pointer_type: 0,
                pointer_id: 1,
                button: 0,
                state: ElementState::Pressed,
                position: glam::vec2(10.0, 20.0),
                position_target: None,
                target_width: None,
                target_height: None,
                trace: None,
            },
            Some(UiTracedPointerDispatch {
                realm_id: RealmId(3),
                document_id: 9,
                pos: glam::vec2(10.0, 20.0),
                realm_size: glam::uvec2(100, 100),
            }),
            None,
        );

        assert_eq!(
            plan.focus_update,
            Some(UiFocusUpdate {
                window_id: 7,
                realm_id: RealmId(3),
                document_id: 9,
                node_id: 0,
            })
        );
        assert_eq!(
            plan.capture_update,
            UiCaptureUpdate::Set {
                window_id: 7,
                capture: UiCaptureEntry {
                    realm_id: RealmId(3),
                    document_id: 9,
                    node_id: 0,
                },
            }
        );
        assert_eq!(
            plan.pointer_pos_update,
            Some(UiPointerPositionUpdate {
                realm_id: RealmId(3),
                pos: Some(glam::vec2(10.0, 20.0)),
            })
        );
    }

    #[test]
    fn traced_pointer_pump_clears_pointer_position_on_leave() {
        let plan = plan_traced_pointer_pump(
            &PointerEvent::OnLeave {
                window_id: 5,
                window_width: None,
                window_height: None,
                pointer_type: 0,
                pointer_id: 1,
                target_width: None,
                target_height: None,
                trace: None,
            },
            None,
            Some(RealmId(11)),
        );

        assert_eq!(plan.focus_update, None);
        assert_eq!(plan.capture_update, UiCaptureUpdate::None);
        assert_eq!(
            plan.pointer_pos_update,
            Some(UiPointerPositionUpdate {
                realm_id: RealmId(11),
                pos: None,
            })
        );
    }

    #[test]
    fn prune_document_focus_links_removes_focus_and_capture_for_document() {
        let mut focus_state = UiFocusState {
            realm_by_window: HashMap::new(),
            document_by_window: HashMap::from([(1, 10), (2, 20)]),
            node_by_window: HashMap::from([(1, 100), (2, 200)]),
            capture_by_window: HashMap::from([
                (
                    1,
                    UiCaptureEntry {
                        realm_id: RealmId(7),
                        document_id: 10,
                        node_id: 100,
                    },
                ),
                (
                    2,
                    UiCaptureEntry {
                        realm_id: RealmId(8),
                        document_id: 20,
                        node_id: 200,
                    },
                ),
            ]),
        };

        prune_document_focus_links(&mut focus_state, 10);

        assert_eq!(focus_state.document_by_window, HashMap::from([(2, 20)]));
        assert_eq!(focus_state.node_by_window, HashMap::from([(2, 200)]));
        assert_eq!(
            focus_state.capture_by_window,
            HashMap::from([(
                2,
                UiCaptureEntry {
                    realm_id: RealmId(8),
                    document_id: 20,
                    node_id: 200,
                },
            )])
        );
    }
}
