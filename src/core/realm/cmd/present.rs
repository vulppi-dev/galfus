use serde::{Deserialize, Serialize};

use crate::core::realm::{PresentId, PresentState, SurfaceId};
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdPresentCreateArgs {
    pub window_id: u32,
    pub surface_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultPresentCreate {
    pub success: bool,
    pub message: String,
    pub present_id: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdPresentDisposeArgs {
    pub present_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultPresentDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_present_create(
    engine: &mut EngineState,
    args: &CmdPresentCreateArgs,
) -> CmdResultPresentCreate {
    if !engine.window.states.contains_key(&args.window_id) {
        return CmdResultPresentCreate {
            success: false,
            message: format!("Window {} not found", args.window_id),
            present_id: None,
        };
    }

    let surface_id = SurfaceId(args.surface_id);
    if engine.universal_state.surfaces.get(surface_id).is_none() {
        return CmdResultPresentCreate {
            success: false,
            message: format!("Surface {} not found", args.surface_id),
            present_id: None,
        };
    }

    if engine
        .universal_state
        .presents
        .entries
        .values()
        .any(|present| present.value.window_id == args.window_id)
    {
        return CmdResultPresentCreate {
            success: false,
            message: format!("Window {} already has a present", args.window_id),
            present_id: None,
        };
    }

    let present_id = engine.universal_state.presents.alloc(PresentState {
        window_id: args.window_id,
        surface: surface_id,
    });

    CmdResultPresentCreate {
        success: true,
        message: "Present created".into(),
        present_id: Some(present_id.0),
    }
}

pub fn engine_cmd_present_dispose(
    engine: &mut EngineState,
    args: &CmdPresentDisposeArgs,
) -> CmdResultPresentDispose {
    let present_id = PresentId(args.present_id);
    if engine.universal_state.presents.remove(present_id).is_none() {
        return CmdResultPresentDispose {
            success: false,
            message: format!("Present {} not found", args.present_id),
        };
    }

    CmdResultPresentDispose {
        success: true,
        message: "Present disposed".into(),
    }
}
