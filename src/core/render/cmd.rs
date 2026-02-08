use serde::{Deserialize, Serialize};

use crate::core::realm::{RealmId, RealmKind};
use crate::core::render::graph::{RenderGraphApplyResult, RenderGraphDesc, RenderGraphState};
use crate::core::state::EngineState;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdRenderGraph3DSetArgs {
    pub realm_id: u32,
    pub graph: RenderGraphDesc,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultRenderGraph3DSet {
    pub success: bool,
    pub fallback_used: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdRenderGraph2DSetArgs {
    pub realm_id: u32,
    pub graph: RenderGraphDesc,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultRenderGraph2DSet {
    pub success: bool,
    pub fallback_used: bool,
    pub message: String,
}

pub fn engine_cmd_render_graph_3d_set(
    engine: &mut EngineState,
    args: &CmdRenderGraph3DSetArgs,
) -> CmdResultRenderGraph3DSet {
    let realm_id = RealmId(args.realm_id);
    let entry = match engine.universal_state.realms.get_mut(realm_id) {
        Some(entry) => entry,
        None => {
            return CmdResultRenderGraph3DSet {
                success: false,
                fallback_used: false,
                message: format!("Realm {} not found", args.realm_id),
            };
        }
    };

    if entry.value.kind != RealmKind::ThreeD {
        return CmdResultRenderGraph3DSet {
            success: false,
            fallback_used: false,
            message: format!("Realm {} is not 3D", args.realm_id),
        };
    }

    let graph_state = entry
        .value
        .render_graph
        .get_or_insert_with(RenderGraphState::new);

    match graph_state.apply_graph(args.graph.clone()) {
        Ok(RenderGraphApplyResult::Applied) => CmdResultRenderGraph3DSet {
            success: true,
            fallback_used: false,
            message: "Render graph applied".into(),
        },
        Ok(RenderGraphApplyResult::FallbackUsed(err)) => CmdResultRenderGraph3DSet {
            success: true,
            fallback_used: true,
            message: format!("Render graph invalid, fallback used: {}", err),
        },
        Err(err) => CmdResultRenderGraph3DSet {
            success: false,
            fallback_used: false,
            message: err,
        },
    }
}

pub fn engine_cmd_render_graph_2d_set(
    engine: &mut EngineState,
    args: &CmdRenderGraph2DSetArgs,
) -> CmdResultRenderGraph2DSet {
    let realm_id = RealmId(args.realm_id);
    let entry = match engine.universal_state.realms.get_mut(realm_id) {
        Some(entry) => entry,
        None => {
            return CmdResultRenderGraph2DSet {
                success: false,
                fallback_used: false,
                message: format!("Realm {} not found", args.realm_id),
            };
        }
    };

    if entry.value.kind != RealmKind::TwoD {
        return CmdResultRenderGraph2DSet {
            success: false,
            fallback_used: false,
            message: format!("Realm {} is not 2D", args.realm_id),
        };
    }

    let graph_state = entry
        .value
        .render_graph
        .get_or_insert_with(RenderGraphState::new);

    match graph_state.apply_graph(args.graph.clone()) {
        Ok(RenderGraphApplyResult::Applied) => CmdResultRenderGraph2DSet {
            success: true,
            fallback_used: false,
            message: "Render graph applied".into(),
        },
        Ok(RenderGraphApplyResult::FallbackUsed(err)) => CmdResultRenderGraph2DSet {
            success: true,
            fallback_used: true,
            message: format!("Render graph invalid, fallback used: {}", err),
        },
        Err(err) => CmdResultRenderGraph2DSet {
            success: false,
            fallback_used: false,
            message: err,
        },
    }
}
