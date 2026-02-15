use serde::{Deserialize, Serialize};

use crate::core::realm::{RealmId, RealmKind, RealmState, SurfaceId};
use crate::core::render::graph::RenderGraphState;
use crate::core::state::EngineState;
use crate::core::target::resolve::remove_auto_link_for_layer;

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

    let kind = match args.kind {
        RealmKindDto::ThreeD => RealmKind::ThreeD,
        RealmKindDto::TwoD => RealmKind::TwoD,
    };
    let render_graph = match kind {
        RealmKind::ThreeD => RenderGraphState::new(),
        RealmKind::TwoD => RenderGraphState::new_ui(),
    };

    let realm_id = engine.universal_state.realms.alloc(RealmState {
        kind,
        host_window_id: args.host_window_id,
        output_surface,
        render_graph: Some(render_graph),
        importance: args.importance.unwrap_or(1),
        cache_policy: args.cache_policy.unwrap_or(0),
        last_render_frame: 0,
    });
    if kind == RealmKind::TwoD {
        engine.universal_state.ui.ensure_realm(realm_id);
    }

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
    if entry.value.kind == RealmKind::TwoD {
        engine.universal_state.ui.remove_realm(realm_id);
    }

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
        let removed_set: std::collections::HashSet<_> = removed_connectors.into_iter().collect();
        engine
            .universal_state
            .input_routing
            .captures
            .retain(|_, capture| !removed_set.contains(&capture.connector_id));
    }

    let removed_layers: Vec<_> = engine
        .universal_state
        .target_layers
        .entries
        .keys()
        .filter(|(layer_realm, _)| *layer_realm == realm_id.0)
        .copied()
        .collect();
    for (layer_realm, layer_target) in removed_layers {
        engine
            .universal_state
            .target_layers
            .entries
            .remove(&(layer_realm, layer_target));
        remove_auto_link_for_layer(&mut engine.universal_state, layer_realm, layer_target);
    }

    if let Some(surface_id) = entry.value.output_surface {
        engine
            .universal_state
            .surface_cache
            .last_good
            .retain(|_, source| *source != surface_id);
        engine
            .universal_state
            .surface_cache
            .fallback
            .retain(|_, source| *source != surface_id);
    }

    CmdResultRealmDispose {
        success: true,
        message: "Realm disposed".into(),
    }
}
