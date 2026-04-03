use serde::{Deserialize, Serialize};

use crate::core::realm::RealmId;
use crate::core::resources::common::mark_realm_windows_dirty;
use crate::core::state::EngineState;
use crate::core::system::push_error_event;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdRenderGraphUpsertArgs {
    pub render_graph_id: u32,
    pub graph: crate::core::render::graph::RenderGraphDesc,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultRenderGraphUpsert {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdRenderGraphDisposeArgs {
    pub render_graph_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultRenderGraphDispose {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdRenderGraphListArgs {}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct RenderGraphEntry {
    pub render_graph_id: u32,
    pub desc_hash: u64,
    pub pass_count: usize,
    pub pass_ids: Vec<String>,
    pub bound_realm_ids: Vec<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultRenderGraphList {
    pub success: bool,
    pub message: String,
    pub render_graphs: Vec<RenderGraphEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdRealmRenderGraphBindArgs {
    pub realm_id: u32,
    pub render_graph_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultRealmRenderGraphBind {
    pub success: bool,
    pub message: String,
}

fn is_reserved_graph_id(render_graph_id: u32) -> bool {
    vulfram_render::is_reserved_render_graph_id(render_graph_id)
}

fn realms_using_graph(engine: &EngineState, render_graph_id: u32) -> Vec<RealmId> {
    let mut realms = Vec::new();
    for (realm_id, realm_entry) in &engine.universal_state.composition.realms.entries {
        if realm_entry.value.render_graph_id == Some(render_graph_id) {
            realms.push(*realm_id);
        }
    }
    realms
}

fn emit_render_graph_error(
    engine: &mut EngineState,
    message: String,
    command_type: &'static str,
) -> CmdResultSimple {
    push_error_event(
        engine,
        "render-graph",
        message.clone(),
        None,
        Some(command_type.into()),
    );
    CmdResultSimple {
        success: false,
        message,
    }
}

#[derive(Debug, Default)]
struct CmdResultSimple {
    success: bool,
    message: String,
}

pub fn engine_cmd_render_graph_upsert(
    engine: &mut EngineState,
    args: &CmdRenderGraphUpsertArgs,
) -> CmdResultRenderGraphUpsert {
    if is_reserved_graph_id(args.render_graph_id) {
        let result = emit_render_graph_error(
            engine,
            format!(
                "Render graph id {} is reserved for core defaults",
                args.render_graph_id
            ),
            "render-graph-upsert",
        );
        return CmdResultRenderGraphUpsert {
            success: result.success,
            message: result.message,
        };
    }

    let desc_hash = crate::core::render::graph::render_graph_desc_hash(&args.graph);
    let cached_graph_state = engine
        .universal_state
        .render_catalog
        .render_graph_plan_cache
        .get(&desc_hash)
        .cloned();
    let graph_state = if let Some(cached) = cached_graph_state.clone() {
        cached
    } else {
        let compiled =
            match crate::core::render::graph::RenderGraphState::from_desc(args.graph.clone()) {
                Ok(state) => state,
                Err(err) => {
                    let result = emit_render_graph_error(
                        engine,
                        format!("Invalid render graph {}: {}", args.render_graph_id, err),
                        "render-graph-upsert",
                    );
                    return CmdResultRenderGraphUpsert {
                        success: result.success,
                        message: result.message,
                    };
                }
            };
        compiled
    };

    let used_by = realms_using_graph(engine, args.render_graph_id);
    for realm_id in &used_by {
        let Some(realm_kind) = engine
            .universal_state
            .composition
            .realms
            .entries
            .get(realm_id)
            .map(|entry| entry.value.kind)
        else {
            continue;
        };
        if !vulfram_render::graph_is_compatible_with_realm_kind(graph_state.plan(), realm_kind) {
            let result = emit_render_graph_error(
                engine,
                format!(
                    "Render graph {} is bound to realm {} ({:?}) and cannot be updated with incompatible passes",
                    args.render_graph_id, realm_id.0, realm_kind
                ),
                "render-graph-upsert",
            );
            return CmdResultRenderGraphUpsert {
                success: result.success,
                message: result.message,
            };
        }
    }

    let existed = engine
        .universal_state
        .render_catalog
        .render_graphs
        .insert(
            args.render_graph_id,
            crate::core::render::graph::RenderGraphRecord {
                state: graph_state.clone(),
                desc_hash,
            },
        )
        .is_some();
    for realm_id in used_by {
        mark_realm_windows_dirty(engine, realm_id.0);
    }
    if cached_graph_state.is_none() {
        engine
            .universal_state
            .render_catalog
            .render_graph_plan_cache
            .insert(desc_hash, graph_state.clone());
    }

    CmdResultRenderGraphUpsert {
        success: true,
        message: if existed {
            "Render graph updated successfully".into()
        } else {
            "Render graph created successfully".into()
        },
    }
}

pub fn engine_cmd_render_graph_dispose(
    engine: &mut EngineState,
    args: &CmdRenderGraphDisposeArgs,
) -> CmdResultRenderGraphDispose {
    if is_reserved_graph_id(args.render_graph_id) {
        let result = emit_render_graph_error(
            engine,
            format!(
                "Render graph id {} is reserved and cannot be disposed",
                args.render_graph_id
            ),
            "render-graph-dispose",
        );
        return CmdResultRenderGraphDispose {
            success: result.success,
            message: result.message,
        };
    }

    let used_by = realms_using_graph(engine, args.render_graph_id);
    if !used_by.is_empty() {
        let used_ids: Vec<String> = used_by
            .iter()
            .map(|realm_id| realm_id.0.to_string())
            .collect();
        let result = emit_render_graph_error(
            engine,
            format!(
                "Render graph {} is bound to realms [{}]",
                args.render_graph_id,
                used_ids.join(", ")
            ),
            "render-graph-dispose",
        );
        return CmdResultRenderGraphDispose {
            success: result.success,
            message: result.message,
        };
    }

    if let Some(removed_graph) = engine
        .universal_state
        .render_catalog
        .render_graphs
        .remove(&args.render_graph_id)
    {
        let keep_plan_cached = engine
            .universal_state
            .render_catalog
            .render_graphs
            .values()
            .any(|record| record.desc_hash == removed_graph.desc_hash);
        if !keep_plan_cached {
            engine
                .universal_state
                .render_catalog
                .render_graph_plan_cache
                .remove(&removed_graph.desc_hash);
        }
        CmdResultRenderGraphDispose {
            success: true,
            message: "Render graph disposed successfully".into(),
        }
    } else {
        let result = emit_render_graph_error(
            engine,
            format!("Render graph {} not found", args.render_graph_id),
            "render-graph-dispose",
        );
        CmdResultRenderGraphDispose {
            success: result.success,
            message: result.message,
        }
    }
}

pub fn engine_cmd_render_graph_list(
    engine: &mut EngineState,
    _args: &CmdRenderGraphListArgs,
) -> CmdResultRenderGraphList {
    let mut render_graph_ids: Vec<u32> = engine
        .universal_state
        .render_catalog
        .render_graphs
        .keys()
        .copied()
        .collect();
    render_graph_ids.sort_unstable();
    let mut render_graphs = Vec::with_capacity(render_graph_ids.len());
    for render_graph_id in render_graph_ids {
        let Some(graph) = engine
            .universal_state
            .render_catalog
            .render_graphs
            .get(&render_graph_id)
        else {
            continue;
        };
        let plan = graph.state.plan();
        let pass_ids = plan.nodes.iter().map(|node| node.pass_id.clone()).collect();
        let mut bound_realm_ids: Vec<u32> = realms_using_graph(engine, render_graph_id)
            .into_iter()
            .map(|realm_id| realm_id.0)
            .collect();
        bound_realm_ids.sort_unstable();
        render_graphs.push(RenderGraphEntry {
            render_graph_id,
            desc_hash: graph.desc_hash,
            pass_count: plan.nodes.len(),
            pass_ids,
            bound_realm_ids,
        });
    }

    CmdResultRenderGraphList {
        success: true,
        message: "Render graph list fetched successfully".into(),
        render_graphs,
    }
}

pub fn engine_cmd_realm_render_graph_bind(
    engine: &mut EngineState,
    args: &CmdRealmRenderGraphBindArgs,
) -> CmdResultRealmRenderGraphBind {
    let Some(graph) = engine
        .universal_state
        .render_catalog
        .render_graphs
        .get(&args.render_graph_id)
    else {
        let result = emit_render_graph_error(
            engine,
            format!("Render graph {} not found", args.render_graph_id),
            "realm-render-graph-bind",
        );
        return CmdResultRealmRenderGraphBind {
            success: result.success,
            message: result.message,
        };
    };

    let realm_id = RealmId(args.realm_id);
    let Some(realm_kind) = engine
        .universal_state
        .composition
        .realms
        .entries
        .get(&realm_id)
        .map(|entry| entry.value.kind)
    else {
        let result = emit_render_graph_error(
            engine,
            format!("Realm {} not found", args.realm_id),
            "realm-render-graph-bind",
        );
        return CmdResultRealmRenderGraphBind {
            success: result.success,
            message: result.message,
        };
    };

    if !vulfram_render::graph_is_compatible_with_realm_kind(graph.state.plan(), realm_kind) {
        let result = emit_render_graph_error(
            engine,
            format!(
                "Realm {} ({:?}) cannot bind render graph {} due to pass incompatibility",
                args.realm_id, realm_kind, args.render_graph_id
            ),
            "realm-render-graph-bind",
        );
        return CmdResultRealmRenderGraphBind {
            success: result.success,
            message: result.message,
        };
    }

    let Some(realm) = engine
        .universal_state
        .composition
        .realms
        .entries
        .get_mut(&realm_id)
    else {
        let result = emit_render_graph_error(
            engine,
            format!("Realm {} not found", args.realm_id),
            "realm-render-graph-bind",
        );
        return CmdResultRealmRenderGraphBind {
            success: result.success,
            message: result.message,
        };
    };

    realm.value.render_graph_id = Some(args.render_graph_id);
    mark_realm_windows_dirty(engine, args.realm_id);

    CmdResultRealmRenderGraphBind {
        success: true,
        message: "Realm render graph binding updated successfully".into(),
    }
}
