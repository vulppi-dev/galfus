use glam::UVec2;
use serde::{Deserialize, Serialize};

use crate::core::realm::SurfaceId;
use crate::core::state::EngineState;

use super::{SurfaceAlphaModeDto, SurfaceFormatDto, SurfaceKindDto};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdSurfaceCreateArgs {
    pub kind: SurfaceKindDto,
    pub size: UVec2,
    #[serde(default)]
    pub format_policy: Option<SurfaceFormatDto>,
    #[serde(default)]
    pub alpha_policy: Option<SurfaceAlphaModeDto>,
    #[serde(default)]
    pub msaa_samples: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultSurfaceCreate {
    pub success: bool,
    pub message: String,
    pub surface_id: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdSurfaceDisposeArgs {
    pub surface_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultSurfaceDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_surface_create(
    engine: &mut EngineState,
    args: &CmdSurfaceCreateArgs,
) -> CmdResultSurfaceCreate {
    let size = UVec2::new(args.size.x.max(1), args.size.y.max(1));
    let surface_id = engine.universal_state.surfaces.alloc(crate::core::realm::SurfaceState {
        kind: args.kind.into(),
        size,
        format_policy: args.format_policy.map(SurfaceFormatDto::to_wgpu),
        alpha_policy: args.alpha_policy.map(SurfaceAlphaModeDto::to_wgpu),
        msaa_samples: args.msaa_samples,
    });

    CmdResultSurfaceCreate {
        success: true,
        message: "Surface created".into(),
        surface_id: Some(surface_id.0),
    }
}

pub fn engine_cmd_surface_dispose(
    engine: &mut EngineState,
    args: &CmdSurfaceDisposeArgs,
) -> CmdResultSurfaceDispose {
    let surface_id = SurfaceId(args.surface_id);
    if engine.universal_state.surfaces.remove(surface_id).is_none() {
        return CmdResultSurfaceDispose {
            success: false,
            message: format!("Surface {} not found", args.surface_id),
        };
    }

    engine.surface_targets.remove(&surface_id);

    let mut removed_connectors = Vec::new();
    engine
        .universal_state
        .connectors
        .entries
        .retain(|connector_id, entry| {
            let remove = entry.value.source_surface == surface_id;
            if remove {
                removed_connectors.push(*connector_id);
            }
            !remove
        });
    if !removed_connectors.is_empty() {
        let removed_set: std::collections::HashSet<_> =
            removed_connectors.into_iter().collect();
        engine
            .universal_state
            .input_routing
            .captures
            .retain(|_, capture| !removed_set.contains(&capture.connector_id));
    }

    engine
        .universal_state
        .presents
        .entries
        .retain(|_, entry| entry.value.surface != surface_id);

    for realm in engine.universal_state.realms.entries.values_mut() {
        if realm.value.output_surface == Some(surface_id) {
            realm.value.output_surface = None;
        }
    }

    engine
        .universal_state
        .surface_cache
        .last_good
        .retain(|target, source| *target != surface_id && *source != surface_id);
    engine
        .universal_state
        .surface_cache
        .fallback
        .retain(|target, source| *target != surface_id && *source != surface_id);

    CmdResultSurfaceDispose {
        success: true,
        message: "Surface disposed".into(),
    }
}
