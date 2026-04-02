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
