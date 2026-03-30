use serde::{Deserialize, Serialize};
use vulfram_input::{ElementState, PointerEvent, TouchPhase};
use vulfram_types::{RealmId, UiNodeId};

use crate::{UiDocumentId, UiTracedPointerDispatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiFocusUpdate {
    pub window_id: u32,
    pub realm_id: RealmId,
    pub document_id: UiDocumentId,
    pub node_id: UiNodeId,
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
}
