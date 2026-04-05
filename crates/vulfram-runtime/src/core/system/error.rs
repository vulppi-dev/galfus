use crate::core::cmd::EngineEvent;
use crate::core::state::EngineState;

use super::SystemEvent;

pub fn push_error_event(
    engine: &mut EngineState,
    scope: &str,
    message: impl Into<String>,
    command_id: Option<u64>,
    command_type: Option<String>,
) {
    engine
        .runtime
        .push_event(EngineEvent::System(SystemEvent::Error {
            scope: scope.to_string(),
            message: message.into(),
            command_id,
            command_type,
        }));
}
