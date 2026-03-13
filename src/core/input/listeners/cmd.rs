use crate::core::cmd::CmdResultSimple;
use crate::core::state::EngineState;
use crate::core::target::TargetId;

use super::model::{
    CmdInputTargetListenerDisposeArgs, CmdInputTargetListenerListArgs,
    CmdInputTargetListenerUpsertArgs, CmdResultInputTargetListenerList, InputTargetListenerConfig,
};

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
            enabled: args.enabled,
            events: args.events.clone(),
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
