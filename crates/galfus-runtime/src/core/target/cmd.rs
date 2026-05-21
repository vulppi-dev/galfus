use glam::UVec2;
use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;
use crate::core::target::{
    SurfaceAlphaModeDto, SurfaceFormatDto, TargetId, TargetKind, TargetLayerLayout,
    TargetLayerState, TargetState, dispose_layer, dispose_target,
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

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTargetMeasurementArgs {
    pub target_id: u64,
    pub get_size: bool,
    pub get_window_size: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetMeasurement {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub size: Option<UVec2>,
    #[serde(default)]
    pub window_size: Option<UVec2>,
    #[serde(default)]
    pub source_kind: Option<String>,
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
    pub enabled_camera_ids: Vec<u32>,
    #[serde(default)]
    pub environment_id: Option<u32>,
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

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTargetGetArgs {
    pub target_id: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetGet {
    pub success: bool,
    pub message: String,
    pub target_id: u64,
    pub kind: Option<TargetKind>,
    pub window_id: Option<u32>,
    pub size: Option<UVec2>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTargetListArgs {
    pub window_id: Option<u32>,
    pub ids: Option<Vec<u64>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetListItem {
    pub target_id: u64,
    pub kind: TargetKind,
    pub window_id: Option<u32>,
    pub size: Option<UVec2>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetList {
    pub success: bool,
    pub message: String,
    pub items: Vec<TargetListItem>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTargetLayerGetArgs {
    pub realm_id: u32,
    pub target_id: u64,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetLayerGet {
    pub success: bool,
    pub message: String,
    pub realm_id: u32,
    pub target_id: u64,
    pub enabled_camera_ids: Vec<u32>,
    pub environment_id: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdTargetLayerListArgs {
    pub realm_id: Option<u32>,
    pub target_id: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetLayerListItem {
    pub realm_id: u32,
    pub target_id: u64,
    pub enabled_camera_ids: Vec<u32>,
    pub environment_id: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTargetLayerList {
    pub success: bool,
    pub message: String,
    pub items: Vec<TargetLayerListItem>,
}

pub fn engine_cmd_target_upsert(
    engine: &mut EngineState,
    args: &CmdTargetUpsertArgs,
) -> CmdResultTargetUpsert {
    if matches!(args.kind, TargetKind::Window) && args.window_id.is_none() {
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
    let size = args
        .size
        .map(|size| UVec2::new(size.x.max(1), size.y.max(1)));
    engine.universal_state.targets.targets.entries.insert(
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

pub fn engine_cmd_target_measurement(
    engine: &mut EngineState,
    args: &CmdTargetMeasurementArgs,
) -> CmdResultTargetMeasurement {
    let target_id = TargetId(args.target_id);
    let Some(target) = engine
        .universal_state
        .targets
        .targets
        .entries
        .get(&target_id)
    else {
        return CmdResultTargetMeasurement {
            success: true,
            message: format!(
                "Target {} not ready yet; returning empty measurement",
                args.target_id
            ),
            ..Default::default()
        };
    };

    let resolved = resolve_target_measurement_size(engine, target_id);
    let window_size = if args.get_window_size {
        target
            .window_id
            .and_then(|window_id| engine.window.states.get(&window_id))
            .map(|state| state.inner_size)
    } else {
        None
    };
    let size = if args.get_size || !args.get_window_size {
        resolved.map(|(size, _)| size)
    } else {
        None
    };
    let source_kind = if args.get_size || !args.get_window_size {
        resolved.map(|(_, source)| source.to_string())
    } else {
        None
    };

    CmdResultTargetMeasurement {
        success: true,
        message: "Target measurement command applied successfully".into(),
        size,
        window_size,
        source_kind,
    }
}

fn resolve_target_measurement_size(
    engine: &EngineState,
    target_id: TargetId,
) -> Option<(UVec2, &'static str)> {
    let target = engine
        .universal_state
        .targets
        .targets
        .entries
        .get(&target_id)?;

    let surface_id = engine
        .universal_state
        .targets
        .auto_links
        .iter()
        .filter_map(|((realm_id, layer_target_id), link)| {
            if *layer_target_id == target_id {
                Some((*realm_id, link.surface_id))
            } else {
                None
            }
        })
        .min_by_key(|(realm_id, _)| *realm_id)
        .map(|(_, surface_id)| surface_id);
    if let Some(surface_id) = surface_id
        && let Some(surface) = engine
            .universal_state
            .composition
            .surfaces
            .entries
            .get(&surface_id)
    {
        return Some((surface.value.size, "surface"));
    }

    if let Some(window_id) = target.window_id
        && let Some(window_state) = engine.window.states.get(&window_id)
    {
        return Some((
            UVec2::new(
                window_state.config.width.max(1),
                window_state.config.height.max(1),
            ),
            "window-surface",
        ));
    }

    target.size.map(|size| (size, "declared"))
}

pub fn engine_cmd_target_dispose(
    engine: &mut EngineState,
    args: &CmdTargetDisposeArgs,
) -> CmdResultTargetDispose {
    let target_id = TargetId(args.target_id);
    if !dispose_target(&mut engine.universal_state, target_id) {
        return CmdResultTargetDispose {
            success: false,
            message: format!("Target {} not found", args.target_id),
        };
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
    let target_id = TargetId(args.target_id);
    if args.layout.width.resolve(1.0, 8.0) <= 0.0 || args.layout.height.resolve(1.0, 8.0) <= 0.0 {
        return CmdResultTargetLayerUpsert {
            success: false,
            message: "TargetLayer layout width/height must be > 0".into(),
        };
    }
    engine.universal_state.targets.target_layers.entries.insert(
        (args.realm_id, target_id),
        TargetLayerState {
            realm_id: args.realm_id,
            target_id,
            layout: args.layout,
            enabled_camera_ids: args.enabled_camera_ids.clone(),
            environment_id: args.environment_id,
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
    if !dispose_layer(&mut engine.universal_state, args.realm_id, target_id) {
        return CmdResultTargetLayerDispose {
            success: false,
            message: format!(
                "Layer not found (realm_id={}, target_id={})",
                args.realm_id, args.target_id
            ),
        };
    }

    CmdResultTargetLayerDispose {
        success: true,
        message: "TargetLayer disposed".into(),
    }
}

pub fn engine_cmd_target_get(
    engine: &mut EngineState,
    args: &CmdTargetGetArgs,
) -> CmdResultTargetGet {
    let target_id = TargetId(args.target_id);
    let Some(target) = engine
        .universal_state
        .targets
        .targets
        .entries
        .get(&target_id)
    else {
        return CmdResultTargetGet {
            success: false,
            message: format!("Target {} not found", args.target_id),
            target_id: args.target_id,
            ..Default::default()
        };
    };
    CmdResultTargetGet {
        success: true,
        message: "Target found".into(),
        target_id: args.target_id,
        kind: Some(target.kind),
        window_id: target.window_id,
        size: target.size,
    }
}

pub fn engine_cmd_target_list(
    engine: &mut EngineState,
    args: &CmdTargetListArgs,
) -> CmdResultTargetList {
    let items = engine
        .universal_state
        .targets
        .targets
        .entries
        .iter()
        .filter(|(id, state)| {
            args.window_id
                .is_none_or(|window_id| state.window_id == Some(window_id))
                && args.ids.as_ref().is_none_or(|ids| ids.contains(&id.0))
        })
        .map(|(id, state)| TargetListItem {
            target_id: id.0,
            kind: state.kind,
            window_id: state.window_id,
            size: state.size,
        })
        .collect();

    CmdResultTargetList {
        success: true,
        message: "Targets listed".into(),
        items,
    }
}

pub fn engine_cmd_target_layer_get(
    engine: &mut EngineState,
    args: &CmdTargetLayerGetArgs,
) -> CmdResultTargetLayerGet {
    let Some(layer) = engine
        .universal_state
        .targets
        .target_layers
        .entries
        .get(&(args.realm_id, TargetId(args.target_id)))
    else {
        return CmdResultTargetLayerGet {
            success: false,
            message: "Target layer not found".into(),
            realm_id: args.realm_id,
            target_id: args.target_id,
            ..Default::default()
        };
    };
    CmdResultTargetLayerGet {
        success: true,
        message: "Target layer found".into(),
        realm_id: args.realm_id,
        target_id: args.target_id,
        enabled_camera_ids: layer.enabled_camera_ids.clone(),
        environment_id: layer.environment_id,
    }
}

pub fn engine_cmd_target_layer_list(
    engine: &mut EngineState,
    args: &CmdTargetLayerListArgs,
) -> CmdResultTargetLayerList {
    let items = engine
        .universal_state
        .targets
        .target_layers
        .entries
        .iter()
        .filter(|((realm_id, target_id), _)| {
            args.realm_id.is_none_or(|id| id == *realm_id)
                && args.target_id.is_none_or(|id| id == target_id.0)
        })
        .map(|((realm_id, target_id), layer)| TargetLayerListItem {
            realm_id: *realm_id,
            target_id: target_id.0,
            enabled_camera_ids: layer.enabled_camera_ids.clone(),
            environment_id: layer.environment_id,
        })
        .collect();
    CmdResultTargetLayerList {
        success: true,
        message: "Target layers listed".into(),
        items,
    }
}

#[cfg(test)]
#[path = "cmd_tests.rs"]
mod tests;
