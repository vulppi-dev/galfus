pub use galfus_protocol::{CmdResultSystemBuildVersionGet, CmdSystemBuildVersionGetArgs};
use serde::{Deserialize, Serialize};

use crate::core::input::events::PointerTraceLevel;
use crate::core::profiling::state::ProfilingDetailLevel;
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdSystemDiagnosticsSetArgs {
    pub profiling_enabled: Option<bool>,
    pub profiling_detail: Option<ProfilingDetailLevel>,
    pub profiling_sampling_percent: Option<u8>,
    pub profiling_window_frames: Option<u8>,
    pub trace_level: Option<PointerTraceLevel>,
    pub trace_sampling_percent: Option<u8>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdResultSystemDiagnosticsSet {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_system_diagnostics_set(
    engine: &mut EngineState,
    args: &CmdSystemDiagnosticsSetArgs,
) -> CmdResultSystemDiagnosticsSet {
    if let Some(enabled) = args.profiling_enabled {
        engine.profiling.config.enabled = enabled;
    }
    if let Some(detail) = args.profiling_detail {
        engine.profiling.config.detail = detail;
    }
    if let Some(sampling_percent) = args.profiling_sampling_percent {
        engine.profiling.config.sampling_percent = sampling_percent.min(100);
    }
    if let Some(window_frames) = args.profiling_window_frames {
        engine.profiling.config.window_frames = window_frames.clamp(1, 120);
    }
    if let Some(level) = args.trace_level {
        engine.universal_state.interaction.input_routing.trace.level = level;
    }
    if let Some(sampling_percent) = args.trace_sampling_percent {
        engine
            .universal_state
            .interaction
            .input_routing
            .trace
            .sampling_percent = sampling_percent.min(100);
    }
    CmdResultSystemDiagnosticsSet {
        success: true,
        message: "System diagnostics updated".into(),
    }
}

pub fn engine_cmd_system_build_version_get(
    _engine: &mut EngineState,
    _args: &CmdSystemBuildVersionGetArgs,
) -> CmdResultSystemBuildVersionGet {
    CmdResultSystemBuildVersionGet {
        success: true,
        message: "Build version retrieved".into(),
        build_version: env!("CARGO_PKG_VERSION").into(),
    }
}

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
