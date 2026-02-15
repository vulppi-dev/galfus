use glam::UVec2;
use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;
use crate::core::target::resolve::remove_auto_link_for_layer;
use crate::core::target::{
    SurfaceAlphaModeDto, SurfaceFormatDto, TargetId, TargetKind, TargetLayerLayout,
    TargetLayerState, TargetState,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTargetUpsertArgs {
    pub target_id: u64,
    pub kind: TargetKind,
    #[serde(default)]
    pub window_id: Option<u32>,
    #[serde(default)]
    pub size: Option<UVec2>,
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
pub struct CmdTargetLayerUpsertArgs {
    pub realm_id: u32,
    pub target_id: u64,
    pub layout: TargetLayerLayout,
    #[serde(default)]
    pub camera_id: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetLayerUpsert {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdTargetLayerDisposeArgs {
    pub realm_id: u32,
    pub target_id: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetLayerDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_target_upsert(
    engine: &mut EngineState,
    args: &CmdTargetUpsertArgs,
) -> CmdResultTargetUpsert {
    if matches!(
        args.kind,
        TargetKind::Window | TargetKind::WidgetRealmViewport | TargetKind::RealmPlane
    ) && args.window_id.is_none()
    {
        return CmdResultTargetUpsert {
            success: false,
            message: format!("Target {:?} requires windowId", args.kind),
        };
    }
    if matches!(args.kind, TargetKind::Texture) && args.window_id.is_some() {
        return CmdResultTargetUpsert {
            success: false,
            message: "Target texture does not accept windowId".into(),
        };
    }
    if !matches!(args.kind, TargetKind::Texture) && args.size.is_some() {
        return CmdResultTargetUpsert {
            success: false,
            message: "Target size is only valid for kind=texture".into(),
        };
    }
    if let Some(window_id) = args.window_id {
        if !engine.window.states.contains_key(&window_id) {
            return CmdResultTargetUpsert {
                success: false,
                message: format!("Window {} not found", window_id),
            };
        }
    }

    let size = args
        .size
        .map(|size| UVec2::new(size.x.max(1), size.y.max(1)));
    engine.universal_state.targets.entries.insert(
        TargetId(args.target_id),
        TargetState {
            kind: args.kind,
            window_id: args.window_id,
            size,
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
        .target_layers
        .entries
        .retain(|(_, layer_target), _| *layer_target != target_id);

    let remove_keys: Vec<_> = engine
        .universal_state
        .auto_links
        .keys()
        .filter(|(_, layer_target)| *layer_target == target_id)
        .copied()
        .collect();
    for (realm_id, layer_target) in remove_keys {
        remove_auto_link_for_layer(&mut engine.universal_state, realm_id, layer_target);
    }

    CmdResultTargetDispose {
        success: true,
        message: "Target disposed".into(),
    }
}

pub fn engine_cmd_target_layer_upsert(
    engine: &mut EngineState,
    args: &CmdTargetLayerUpsertArgs,
) -> CmdResultTargetLayerUpsert {
    if engine
        .universal_state
        .realms
        .get(crate::core::realm::RealmId(args.realm_id))
        .is_none()
    {
        return CmdResultTargetLayerUpsert {
            success: false,
            message: format!("Realm {} not found", args.realm_id),
        };
    }
    let target_id = TargetId(args.target_id);
    if !engine
        .universal_state
        .targets
        .entries
        .contains_key(&target_id)
    {
        return CmdResultTargetLayerUpsert {
            success: false,
            message: format!("Target {} not found", args.target_id),
        };
    }
    engine.universal_state.target_layers.entries.insert(
        (args.realm_id, target_id),
        TargetLayerState {
            realm_id: args.realm_id,
            target_id,
            layout: args.layout,
            camera_id: args.camera_id,
        },
    );

    CmdResultTargetLayerUpsert {
        success: true,
        message: "TargetLayer upserted".into(),
    }
}

pub fn engine_cmd_target_layer_dispose(
    engine: &mut EngineState,
    args: &CmdTargetLayerDisposeArgs,
) -> CmdResultTargetLayerDispose {
    let target_id = TargetId(args.target_id);
    if engine
        .universal_state
        .target_layers
        .entries
        .remove(&(args.realm_id, target_id))
        .is_none()
    {
        return CmdResultTargetLayerDispose {
            success: false,
            message: format!(
                "Layer not found (realm_id={}, target_id={})",
                args.realm_id, args.target_id
            ),
        };
    }

    remove_auto_link_for_layer(&mut engine.universal_state, args.realm_id, target_id);

    CmdResultTargetLayerDispose {
        success: true,
        message: "TargetLayer disposed".into(),
    }
}
