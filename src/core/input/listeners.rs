mod cmd;
mod emit;
mod model;
mod store;

pub use cmd::{
    engine_cmd_input_target_listener_dispose, engine_cmd_input_target_listener_list,
    engine_cmd_input_target_listener_upsert,
};
pub use emit::emit_target_listener_events;
#[allow(unused_imports)]
pub use model::{
    CmdInputTargetListenerDisposeArgs, CmdInputTargetListenerListArgs,
    CmdInputTargetListenerUpsertArgs, CmdResultInputTargetListenerList, InputTargetListenerConfig,
    TargetListenerScope,
};
pub use store::InputTargetListenerStore;

#[cfg(test)]
mod tests {
    use crate::core::cmd::EngineEvent;
    use crate::core::input::events::{PointerEvent, PointerEventTrace};
    use crate::core::state::EngineState;
    use crate::core::system::SystemEvent;
    use crate::core::target::TargetId;

    use super::{InputTargetListenerConfig, TargetListenerScope, emit_target_listener_events};

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
