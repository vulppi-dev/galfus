use serde::{Deserialize, Serialize};

use crate::core::resources::EnvironmentConfig;
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdEnvironmentCreateArgs {
    pub environment_id: u32,
    #[serde(default)]
    pub config: EnvironmentConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdEnvironmentUpdateArgs {
    pub environment_id: u32,
    #[serde(default)]
    pub config: EnvironmentConfig,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdEnvironmentDisposeArgs {
    pub environment_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultEnvironment {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_environment_create(
    engine: &mut EngineState,
    args: &CmdEnvironmentCreateArgs,
) -> CmdResultEnvironment {
    engine
        .universal_state
        .scene
        .realm3d
        .environment_profiles
        .insert(args.environment_id, args.config.clone());
    engine.universal_state.scene.realm3d.default_environment_id = Some(args.environment_id);

    CmdResultEnvironment {
        success: true,
        message: "Environment upserted".into(),
    }
}

pub fn engine_cmd_environment_update(
    engine: &mut EngineState,
    args: &CmdEnvironmentUpdateArgs,
) -> CmdResultEnvironment {
    engine
        .universal_state
        .scene
        .realm3d
        .environment_profiles
        .insert(args.environment_id, args.config.clone());
    engine.universal_state.scene.realm3d.default_environment_id = Some(args.environment_id);

    CmdResultEnvironment {
        success: true,
        message: "Environment updated".into(),
    }
}

pub fn engine_cmd_environment_dispose(
    engine: &mut EngineState,
    args: &CmdEnvironmentDisposeArgs,
) -> CmdResultEnvironment {
    if engine
        .universal_state
        .scene
        .realm3d
        .environment_profiles
        .remove(&args.environment_id)
        .is_none()
    {
        return CmdResultEnvironment {
            success: false,
            message: format!("Environment {} not found", args.environment_id),
        };
    }

    if engine.universal_state.scene.realm3d.default_environment_id == Some(args.environment_id) {
        engine.universal_state.scene.realm3d.default_environment_id = engine
            .universal_state
            .scene
            .realm3d
            .environment_profiles
            .keys()
            .copied()
            .min();
    }

    for layer in engine
        .universal_state
        .targets
        .target_layers
        .entries
        .values_mut()
    {
        if layer.environment_id == Some(args.environment_id) {
            layer.environment_id = None;
        }
    }

    CmdResultEnvironment {
        success: true,
        message: "Environment disposed".into(),
    }
}

#[cfg(test)]
#[path = "cmd_tests.rs"]
mod tests;
