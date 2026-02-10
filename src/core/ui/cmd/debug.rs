use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiDebugSetArgs {
    pub enabled: bool,
    #[serde(default)]
    pub show_bounds: bool,
    #[serde(default)]
    pub show_ids: bool,
    #[serde(default)]
    pub show_profile: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiDebugSet {
    pub success: bool,
    pub message: String,
}

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
