use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LogEvent {
    pub level: LogLevel,
    pub tag: String,
    pub message: String,
}

pub trait LogEventSink {
    fn emit_log_event(&mut self, event: LogEvent);
}

pub fn emit_log<S: LogEventSink + ?Sized>(
    sink: &mut S,
    level: LogLevel,
    tag: impl Into<String>,
    message: impl Into<String>,
) {
    sink.emit_log_event(LogEvent {
        level,
        tag: tag.into(),
        message: message.into(),
    });
}

pub fn emit_trace<S: LogEventSink + ?Sized>(
    sink: &mut S,
    tag: impl Into<String>,
    message: impl Into<String>,
) {
    emit_log(sink, LogLevel::Trace, tag, message);
}

pub fn emit_debug<S: LogEventSink + ?Sized>(
    sink: &mut S,
    tag: impl Into<String>,
    message: impl Into<String>,
) {
    emit_log(sink, LogLevel::Debug, tag, message);
}

pub fn emit_info<S: LogEventSink + ?Sized>(
    sink: &mut S,
    tag: impl Into<String>,
    message: impl Into<String>,
) {
    emit_log(sink, LogLevel::Info, tag, message);
}

pub fn emit_warn<S: LogEventSink + ?Sized>(
    sink: &mut S,
    tag: impl Into<String>,
    message: impl Into<String>,
) {
    emit_log(sink, LogLevel::Warn, tag, message);
}

pub fn emit_error<S: LogEventSink + ?Sized>(
    sink: &mut S,
    tag: impl Into<String>,
    message: impl Into<String>,
) {
    emit_log(sink, LogLevel::Error, tag, message);
}

impl LogEventSink for Vec<LogEvent> {
    fn emit_log_event(&mut self, event: LogEvent) {
        self.push(event);
    }
}

#[macro_export]
macro_rules! galfus_log_event {
    ($sink:expr, $level:expr, $tag:expr, $($arg:tt)*) => {
        $crate::emit_log($sink, $level, $tag, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! galfus_log_trace {
    ($sink:expr, $tag:expr, $($arg:tt)*) => {
        $crate::emit_trace($sink, $tag, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! galfus_log_debug {
    ($sink:expr, $tag:expr, $($arg:tt)*) => {
        $crate::emit_debug($sink, $tag, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! galfus_log_info {
    ($sink:expr, $tag:expr, $($arg:tt)*) => {
        $crate::emit_info($sink, $tag, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! galfus_log_warn {
    ($sink:expr, $tag:expr, $($arg:tt)*) => {
        $crate::emit_warn($sink, $tag, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! galfus_log_error {
    ($sink:expr, $tag:expr, $($arg:tt)*) => {
        $crate::emit_error($sink, $tag, format!($($arg)*))
    };
}
