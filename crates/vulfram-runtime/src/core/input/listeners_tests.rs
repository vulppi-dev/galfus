use super::{InputTargetListenerConfig, emit_target_listener_events};
use crate::core::cmd::EngineEvent;
use crate::core::input::events::{PointerEvent, PointerEventTrace};
use crate::core::state::EngineState;
use crate::core::system::SystemEvent;

#[test]
fn listener_emits_system_event_for_matching_pointer_target() {
    let mut engine = EngineState::new();
    engine
        .universal_state
        .interaction
        .target_listeners
        .upsert(InputTargetListenerConfig {
            listener_id: 10,
            target_id: 99,
            enabled: true,
            events: vec!["pointer-move".into()],
            sample_percent: 100,
        });
    engine
        .runtime
        .push_event(EngineEvent::Pointer(PointerEvent::OnMove {
            window_id: 1,
            window_width: Some(800),
            window_height: Some(600),
            pointer_type: 0,
            pointer_id: 1,
            position: glam::vec2(1.0, 2.0),
            position_target: Some(glam::vec2(3.0, 4.0)),
            target_width: Some(320),
            target_height: Some(240),
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

    assert!(engine.runtime.events().iter().any(|event| {
        matches!(
            event,
            EngineEvent::System(SystemEvent::InputTargetListenerEvent {
                listener_id: 10,
                target_id: 99,
                window_width: Some(800),
                window_height: Some(600),
                target_width: Some(320),
                target_height: Some(240),
                ..
            })
        )
    }));
}
