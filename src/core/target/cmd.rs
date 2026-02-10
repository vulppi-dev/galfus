use glam::UVec2;
use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;
use crate::core::target::resolve::remove_auto_link_for_bind;
use crate::core::target::{
    SurfaceAlphaModeDto, SurfaceFormatDto, TargetBindLayout, TargetBindState, TargetId, TargetKind,
    TargetState,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTargetUpsertArgs {
    pub target_id: u64,
    pub kind: TargetKind,
    #[serde(default)]
    pub owner_window_id: Option<u32>,
    #[serde(default)]
    pub size_override: Option<UVec2>,
    #[serde(default)]
    pub format_policy: Option<SurfaceFormatDto>,
    #[serde(default)]
    pub alpha_policy: Option<SurfaceAlphaModeDto>,
    #[serde(default)]
    pub msaa_samples: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetUpsert {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTargetDisposeArgs {
    pub target_id: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetDispose {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTargetBindUpsertArgs {
    pub realm_id: u32,
    pub target_id: u64,
    pub layout: TargetBindLayout,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetBindUpsert {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTargetBindDisposeArgs {
    pub realm_id: u32,
    pub target_id: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetBindDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_target_upsert(
    engine: &mut EngineState,
    args: &CmdTargetUpsertArgs,
) -> CmdResultTargetUpsert {
    let size_override = args
        .size_override
        .map(|size| UVec2::new(size.x.max(1), size.y.max(1)));
    engine.universal_state.targets.entries.insert(
        TargetId(args.target_id),
        TargetState {
            kind: args.kind,
            owner_window_id: args.owner_window_id,
            size_override,
            format_policy: args.format_policy.map(SurfaceFormatDto::to_wgpu),
            alpha_policy: args.alpha_policy.map(SurfaceAlphaModeDto::to_wgpu),
            msaa_samples: args.msaa_samples,
        },
    );

    CmdResultTargetUpsert {
        success: true,
        message: "Target upserted".into(),
    }
}

pub fn engine_cmd_target_dispose(
    engine: &mut EngineState,
    args: &CmdTargetDisposeArgs,
) -> CmdResultTargetDispose {
    let target_id = TargetId(args.target_id);
    if engine
        .universal_state
        .targets
        .entries
        .remove(&target_id)
        .is_none()
    {
        return CmdResultTargetDispose {
            success: false,
            message: format!("Target {} not found", args.target_id),
        };
    }

    engine
        .universal_state
        .target_binds
        .entries
        .retain(|(_, bind_target), _| *bind_target != target_id);

    let remove_keys: Vec<_> = engine
        .universal_state
        .auto_links
        .keys()
        .filter(|(_, bind_target)| *bind_target == target_id)
        .copied()
        .collect();
    for (realm_id, bind_target) in remove_keys {
        remove_auto_link_for_bind(&mut engine.universal_state, realm_id, bind_target);
    }

    CmdResultTargetDispose {
        success: true,
        message: "Target disposed".into(),
    }
}

pub fn engine_cmd_target_bind_upsert(
    engine: &mut EngineState,
    args: &CmdTargetBindUpsertArgs,
) -> CmdResultTargetBindUpsert {
    let target_id = TargetId(args.target_id);
    engine.universal_state.target_binds.entries.insert(
        (args.realm_id, target_id),
        TargetBindState {
            realm_id: args.realm_id,
            target_id,
            layout: args.layout,
        },
    );

    CmdResultTargetBindUpsert {
        success: true,
        message: "TargetBind upserted".into(),
    }
}

pub fn engine_cmd_target_bind_dispose(
    engine: &mut EngineState,
    args: &CmdTargetBindDisposeArgs,
) -> CmdResultTargetBindDispose {
    let target_id = TargetId(args.target_id);
    if engine
        .universal_state
        .target_binds
        .entries
        .remove(&(args.realm_id, target_id))
        .is_none()
    {
        return CmdResultTargetBindDispose {
            success: false,
            message: format!(
                "Bind not found (realm_id={}, target_id={})",
                args.realm_id, args.target_id
            ),
        };
    }

    remove_auto_link_for_bind(&mut engine.universal_state, args.realm_id, target_id);

    CmdResultTargetBindDispose {
        success: true,
        message: "TargetBind disposed".into(),
    }
}
