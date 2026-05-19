use super::*;
use crate::core::cmd::EngineEvent;
use crate::core::test_support::test_engine;
use galfus_log::{LogEventSink, LogLevel};

#[test]
fn system_build_version_get_returns_pkg_version() {
    let mut engine = test_engine();
    let result = engine_cmd_system_build_version_get(&mut engine, &CmdSystemBuildVersionGetArgs {});
    assert!(result.success);
    assert_eq!(result.build_version, env!("CARGO_PKG_VERSION"));
}

#[test]
fn system_log_level_defaults_to_info() {
    let mut engine = test_engine();
    let result = engine_cmd_system_log_level_get(&mut engine, &CmdSystemLogLevelGetArgs {});
    assert!(result.success);
    assert_eq!(result.current_level, LogLevel::Info);
}

#[test]
fn system_log_level_set_updates_and_get_reflects_value() {
    let mut engine = test_engine();
    let set_result = engine_cmd_system_log_level_set(
        &mut engine,
        &CmdSystemLogLevelSetArgs {
            level: LogLevel::Error,
        },
    );
    assert!(set_result.success);
    assert_eq!(set_result.current_level, LogLevel::Error);

    let get_result = engine_cmd_system_log_level_get(&mut engine, &CmdSystemLogLevelGetArgs {});
    assert!(get_result.success);
    assert_eq!(get_result.current_level, LogLevel::Error);
}

#[test]
fn log_sink_filters_events_by_current_level() {
    let mut engine = test_engine();
    engine.emit_log_event(galfus_log::LogEvent {
        level: LogLevel::Debug,
        tag: "test".into(),
        message: "debug".into(),
    });
    engine.emit_log_event(galfus_log::LogEvent {
        level: LogLevel::Info,
        tag: "test".into(),
        message: "info".into(),
    });
    let events = engine.runtime.take_events();
    assert_eq!(events.len(), 1);
    match &events[0] {
        EngineEvent::Log(log) => assert_eq!(log.level, LogLevel::Info),
        other => panic!("unexpected event: {other:?}"),
    }

    let _ = engine_cmd_system_log_level_set(
        &mut engine,
        &CmdSystemLogLevelSetArgs {
            level: LogLevel::Trace,
        },
    );
    engine.emit_log_event(galfus_log::LogEvent {
        level: LogLevel::Debug,
        tag: "test".into(),
        message: "debug-visible".into(),
    });
    assert_eq!(engine.runtime.take_events().len(), 1);
}
