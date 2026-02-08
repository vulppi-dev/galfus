use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCloseArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowClose {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_close(
    engine: &mut EngineState,
    args: &CmdWindowCloseArgs,
) -> CmdResultWindowClose {
    // Check if window exists
    if !engine.window.states.contains_key(&args.window_id) {
        return CmdResultWindowClose {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        };
    }

    // Cleanup window and all associated resources
    if engine.cleanup_window(args.window_id) {
        CmdResultWindowClose {
            success: true,
            message: "Window closed successfully".into(),
        }
    } else {
        CmdResultWindowClose {
            success: false,
            message: "Failed to close window".into(),
        }
    }
}
