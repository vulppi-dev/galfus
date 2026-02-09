use serde::{Deserialize, Serialize};

use crate::core::realm::{RealmId, RealmState, SurfaceId};
use crate::core::render::graph::RenderGraphState;
use crate::core::state::EngineState;

use super::RealmKindDto;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdRealmCreateArgs {
    pub kind: RealmKindDto,
    pub output_surface_id: Option<u32>,
    #[serde(default)]
    pub host_window_id: Option<u32>,
    #[serde(default)]
    pub importance: Option<u8>,
    #[serde(default)]
    pub cache_policy: Option<u8>,
    #[serde(default)]
    pub flags: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultRealmCreate {
    pub success: bool,
    pub message: String,
    pub realm_id: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdRealmDisposeArgs {
    pub realm_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultRealmDispose {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_realm_create(
    engine: &mut EngineState,
    args: &CmdRealmCreateArgs,
) -> CmdResultRealmCreate {
    if let Some(host_window_id) = args.host_window_id {
        if !engine.window.states.contains_key(&host_window_id) {
            return CmdResultRealmCreate {
                success: false,
                message: format!("Host window {} not found", host_window_id),
                realm_id: None,
            };
        }
    }

    let output_surface = args.output_surface_id.map(SurfaceId);
    if let Some(surface_id) = output_surface {
        if engine.universal_state.surfaces.get(surface_id).is_none() {
            return CmdResultRealmCreate {
                success: false,
                message: format!("Surface {} not found", surface_id.0),
                realm_id: None,
            };
        }
    }

    let realm_id = engine.universal_state.realms.alloc(RealmState {
        kind: args.kind.into(),
        host_window_id: args.host_window_id,
        output_surface,
        render_graph: Some(RenderGraphState::new()),
        flags: args.flags.unwrap_or(0),
        importance: args.importance.unwrap_or(1),
        cache_policy: args.cache_policy.unwrap_or(0),
        last_render_frame: 0,
    });

    CmdResultRealmCreate {
        success: true,
        message: "Realm created".into(),
        realm_id: Some(realm_id.0),
    }
}

pub fn engine_cmd_realm_dispose(
    engine: &mut EngineState,
    args: &CmdRealmDisposeArgs,
) -> CmdResultRealmDispose {
    let realm_id = RealmId(args.realm_id);
    let Some(entry) = engine.universal_state.realms.remove(realm_id) else {
        return CmdResultRealmDispose {
            success: false,
            message: format!("Realm {} not found", args.realm_id),
        };
    };

    let mut removed_connectors = Vec::new();
    engine
        .universal_state
        .connectors
        .entries
        .retain(|connector_id, entry| {
            let remove = entry.value.target_realm == realm_id;
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

    if let Some(surface_id) = entry.value.output_surface {
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
    }

    CmdResultRealmDispose {
        success: true,
        message: "Realm disposed".into(),
    }
}
