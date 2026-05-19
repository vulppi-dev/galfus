use crate::core::cmd::EngineEvent;
use crate::core::state::EngineState;

pub use galfus_log::{LogEvent, LogEventSink, LogLevel};

impl LogEventSink for EngineState {
    fn emit_log_event(&mut self, event: LogEvent) {
        self.runtime.push_event(EngineEvent::Log(event));
    }
}
