use crate::core::state::EngineState;
pub use vulfram_realm_ui::{CmdResultUiDebugSet, CmdUiDebugSetArgs};

pub fn engine_cmd_ui_debug_set(
    engine: &mut EngineState,
    args: &CmdUiDebugSetArgs,
) -> CmdResultUiDebugSet {
    let debug = &mut engine.universal_state.ui.debug;
    debug.enabled = args.enabled;
    debug.show_bounds = args.show_bounds;
    debug.show_ids = args.show_ids;
    debug.show_profile = args.show_profile;

    CmdResultUiDebugSet {
        success: true,
        message: "UI debug updated".into(),
    }
}
