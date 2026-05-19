use crate::core::cmd::EngineEvent;
use crate::core::state::EngineState;

pub use galfus_log::{LogEvent, LogEventSink, LogLevel};

impl LogEventSink for EngineState {
    fn emit_log_event(&mut self, event: LogEvent) {
        if self.log_level.allows(event.level) {
            self.runtime.push_event(EngineEvent::Log(event));
        }
    }
}
