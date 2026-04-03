use serde::{Deserialize, Serialize};

use crate::core::realm::{
    RealmId, RealmKind, RealmState, detach_realm_runtime, dispose_realm_layers,
    dispose_surface_links, init_realm_runtime, remove_connectors_for_realms,
};
use crate::core::render::ensure_runtime_render_defaults;
use crate::core::render::graph::{DEFAULT_2D_RENDER_GRAPH_ID, DEFAULT_3D_RENDER_GRAPH_ID};
use crate::core::state::EngineState;
use crate::core::target::prune_target_graph_cache;

use super::RealmKindDto;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdRealmCreateArgs {
    pub kind: RealmKindDto,
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
    let kind = match args.kind {
        RealmKindDto::ThreeD => RealmKind::ThreeD,
        RealmKindDto::TwoD => RealmKind::TwoD,
    };
    ensure_runtime_render_defaults(&mut engine.universal_state);
    let render_graph_id = match kind {
        RealmKind::ThreeD => Some(DEFAULT_3D_RENDER_GRAPH_ID),
        RealmKind::TwoD => Some(DEFAULT_2D_RENDER_GRAPH_ID),
    };

    let realm_id = engine.universal_state.composition.realms.alloc(RealmState {
        kind,
        output_surface: None,
        render_graph_id,
        importance: args.importance.unwrap_or(1),
        cache_policy: args.cache_policy.unwrap_or(0),
        last_render_frame: 0,
    });
    init_realm_runtime(&mut engine.universal_state, realm_id, kind);

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
    let Some(entry) = engine.universal_state.composition.realms.remove(realm_id) else {
        return CmdResultRealmDispose {
            success: false,
            message: format!("Realm {} not found", args.realm_id),
        };
    };
    detach_realm_runtime(&mut engine.universal_state, realm_id, entry.value.kind);
    remove_connectors_for_realms(
        &mut engine.universal_state,
        &std::collections::HashSet::from([realm_id]),
        &std::collections::HashSet::new(),
    );
    dispose_realm_layers(&mut engine.universal_state, realm_id);

    if let Some(surface_id) = entry.value.output_surface {
        dispose_surface_links(&mut engine.universal_state, realm_id, surface_id);
        engine.surface_targets.remove(&surface_id);
        engine
            .universal_state
            .composition
            .surfaces
            .remove(surface_id);
    }
    prune_target_graph_cache(&mut engine.universal_state);

    CmdResultRealmDispose {
        success: true,
        message: "Realm disposed".into(),
    }
}
