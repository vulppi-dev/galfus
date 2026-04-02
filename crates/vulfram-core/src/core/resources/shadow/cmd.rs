use super::ShadowConfig;
use crate::core::state::EngineState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdShadowConfigureArgs {
    pub window_id: u32,
    pub config: ShadowConfig,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultShadowConfigure {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_shadow_configure(
    engine: &mut EngineState,
    args: &CmdShadowConfigureArgs,
) -> CmdResultShadowConfigure {
    let device = match engine.device.as_ref() {
        Some(d) => d,
        None => {
            return CmdResultShadowConfigure {
                success: false,
                message: "GPU Device not initialized".into(),
            };
        }
    };

    if let Some(render_state) = engine.render.get_mut(&args.window_id) {
        let Some(shadow) = render_state.shadow.as_mut() else {
            return CmdResultShadowConfigure {
                success: false,
                message: format!(
                    "Window {} not found or shadow manager not initialized",
                    args.window_id
                ),
            };
        };
        shadow.configure(device, args.config);
        if let Some(bindings) = render_state.bindings.as_mut() {
            bindings.shared_group = None;
            bindings.shadow_model_bind_group = None;
        }
        if let Some(window_state) = engine.window.states.get_mut(&args.window_id) {
            window_state.is_dirty = true;
        }
        CmdResultShadowConfigure {
            success: true,
            message: "Shadow configuration updated successfully".into(),
        }
    } else {
        CmdResultShadowConfigure {
            success: false,
            message: format!(
                "Window {} not found or shadow manager not initialized",
                args.window_id
            ),
        }
    }
}
