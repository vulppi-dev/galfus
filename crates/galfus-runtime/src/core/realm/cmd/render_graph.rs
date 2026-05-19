use galfus_realm_core::RealmKind;
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
    pub graph_kind: String,
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
    galfus_render::is_reserved_render_graph_id(render_graph_id)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GraphRegistryKind {
    ThreeD,
    TwoD,
}

impl GraphRegistryKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::ThreeD => "3d",
            Self::TwoD => "2d",
        }
    }
}

fn classify_graph_registry(
    plan: &crate::core::render::graph::RenderGraphPlan,
) -> Option<GraphRegistryKind> {
    if galfus_render::graph_is_compatible_with_realm_kind(plan, RealmKind::TwoD) {
        return Some(GraphRegistryKind::TwoD);
    }
    if galfus_render::graph_is_compatible_with_realm_kind(plan, RealmKind::ThreeD) {
        return Some(GraphRegistryKind::ThreeD);
    }
    None
}

fn registry_for_realm_kind(realm_kind: RealmKind) -> GraphRegistryKind {
    match realm_kind {
        RealmKind::ThreeD => GraphRegistryKind::ThreeD,
        RealmKind::TwoD => GraphRegistryKind::TwoD,
    }
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
    let cached_2d = engine
        .universal_state
        .render_catalog
        .render_graph_plan_cache_2d
        .get(&desc_hash)
        .cloned();
    let cached_3d = engine
        .universal_state
        .render_catalog
        .render_graph_plan_cache_3d
        .get(&desc_hash)
        .cloned();

    let (graph_state, graph_kind, cache_hit) = if let Some(state) = cached_2d {
        (state, GraphRegistryKind::TwoD, true)
    } else if let Some(state) = cached_3d {
        (state, GraphRegistryKind::ThreeD, true)
    } else {
        let graph_state =
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
        let Some(graph_kind) = classify_graph_registry(graph_state.plan()) else {
            let result = emit_render_graph_error(
                engine,
                format!(
                    "Render graph {} is incompatible with both Graph3D and Graph2D registries",
                    args.render_graph_id
                ),
                "render-graph-upsert",
            );
            return CmdResultRenderGraphUpsert {
                success: result.success,
                message: result.message,
            };
        };
        (graph_state, graph_kind, false)
    };
    if cache_hit {
        engine
            .universal_state
            .render_catalog
            .render_graph_compile_cache_hits = engine
            .universal_state
            .render_catalog
            .render_graph_compile_cache_hits
            .saturating_add(1);
    } else {
        engine
            .universal_state
            .render_catalog
            .render_graph_compile_cache_misses = engine
            .universal_state
            .render_catalog
            .render_graph_compile_cache_misses
            .saturating_add(1);
    }

    if classify_graph_registry(graph_state.plan()).is_none() {
        let result = emit_render_graph_error(
            engine,
            format!(
                "Render graph {} is incompatible with both Graph3D and Graph2D registries",
                args.render_graph_id
            ),
            "render-graph-upsert",
        );
        return CmdResultRenderGraphUpsert {
            success: result.success,
            message: result.message,
        };
    }

    let exists_in_other_registry = match graph_kind {
        GraphRegistryKind::ThreeD => engine
            .universal_state
            .render_catalog
            .render_graphs_2d
            .contains_key(&args.render_graph_id),
        GraphRegistryKind::TwoD => engine
            .universal_state
            .render_catalog
            .render_graphs_3d
            .contains_key(&args.render_graph_id),
    };
    if exists_in_other_registry {
        let result = emit_render_graph_error(
            engine,
            format!(
                "Render graph id {} is already used by the other graph registry",
                args.render_graph_id
            ),
            "render-graph-upsert",
        );
        return CmdResultRenderGraphUpsert {
            success: result.success,
            message: result.message,
        };
    }

    let used_by = realms_using_graph(engine, args.render_graph_id);
    for realm_id in &used_by {
        let Some(bound_realm_kind) = engine
            .universal_state
            .composition
            .realms
            .entries
            .get(realm_id)
            .map(|entry| entry.value.kind)
        else {
            continue;
        };
        if registry_for_realm_kind(bound_realm_kind) != graph_kind {
            let result = emit_render_graph_error(
                engine,
                format!(
                    "Render graph {} is bound to realm {} ({:?}) and cannot change graph registry kind",
                    args.render_graph_id, realm_id.0, bound_realm_kind
                ),
                "render-graph-upsert",
            );
            return CmdResultRenderGraphUpsert {
                success: result.success,
                message: result.message,
            };
        }
    }

    let existed = {
        let (registry, cache) = match graph_kind {
            GraphRegistryKind::ThreeD => (
                &mut engine.universal_state.render_catalog.render_graphs_3d,
                &mut engine
                    .universal_state
                    .render_catalog
                    .render_graph_plan_cache_3d,
            ),
            GraphRegistryKind::TwoD => (
                &mut engine.universal_state.render_catalog.render_graphs_2d,
                &mut engine
                    .universal_state
                    .render_catalog
                    .render_graph_plan_cache_2d,
            ),
        };
        let existed = registry
            .insert(
                args.render_graph_id,
                crate::core::render::graph::RenderGraphRecord {
                    state: graph_state.clone(),
                    desc_hash,
                },
            )
            .is_some();
        cache.entry(desc_hash).or_insert(graph_state.clone());
        existed
    };
    for realm_id in used_by {
        mark_realm_windows_dirty(engine, realm_id.0);
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

    let removed = if let Some(record) = engine
        .universal_state
        .render_catalog
        .render_graphs_3d
        .remove(&args.render_graph_id)
    {
        Some((GraphRegistryKind::ThreeD, record))
    } else {
        engine
            .universal_state
            .render_catalog
            .render_graphs_2d
            .remove(&args.render_graph_id)
            .map(|record| (GraphRegistryKind::TwoD, record))
    };
    if let Some((graph_kind, removed_graph)) = removed {
        let keep_plan_cached = match graph_kind {
            GraphRegistryKind::ThreeD => engine
                .universal_state
                .render_catalog
                .render_graphs_3d
                .values()
                .any(|record| record.desc_hash == removed_graph.desc_hash),
            GraphRegistryKind::TwoD => engine
                .universal_state
                .render_catalog
                .render_graphs_2d
                .values()
                .any(|record| record.desc_hash == removed_graph.desc_hash),
        };
        if !keep_plan_cached {
            match graph_kind {
                GraphRegistryKind::ThreeD => {
                    engine
                        .universal_state
                        .render_catalog
                        .render_graph_plan_cache_3d
                        .remove(&removed_graph.desc_hash);
                }
                GraphRegistryKind::TwoD => {
                    engine
                        .universal_state
                        .render_catalog
                        .render_graph_plan_cache_2d
                        .remove(&removed_graph.desc_hash);
                }
            }
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
        .render_graphs_3d
        .keys()
        .copied()
        .collect();
    render_graph_ids.extend(
        engine
            .universal_state
            .render_catalog
            .render_graphs_2d
            .keys()
            .copied(),
    );
    render_graph_ids.sort_unstable();
    render_graph_ids.dedup();
    let mut render_graphs = Vec::with_capacity(render_graph_ids.len());
    for render_graph_id in render_graph_ids {
        let (graph_kind, graph) = if let Some(graph) = engine
            .universal_state
            .render_catalog
            .render_graphs_3d
            .get(&render_graph_id)
        {
            (GraphRegistryKind::ThreeD, graph)
        } else if let Some(graph) = engine
            .universal_state
            .render_catalog
            .render_graphs_2d
            .get(&render_graph_id)
        {
            (GraphRegistryKind::TwoD, graph)
        } else {
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
            graph_kind: graph_kind.as_str().into(),
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
    let target_registry = registry_for_realm_kind(realm_kind);
    let graph = match target_registry {
        GraphRegistryKind::ThreeD => engine
            .universal_state
            .render_catalog
            .render_graphs_3d
            .get(&args.render_graph_id),
        GraphRegistryKind::TwoD => engine
            .universal_state
            .render_catalog
            .render_graphs_2d
            .get(&args.render_graph_id),
    };
    let Some(graph) = graph else {
        let result = emit_render_graph_error(
            engine,
            format!(
                "Render graph {} not found in {} registry",
                args.render_graph_id,
                target_registry.as_str()
            ),
            "realm-render-graph-bind",
        );
        return CmdResultRealmRenderGraphBind {
            success: result.success,
            message: result.message,
        };
    };

    if !galfus_render::graph_is_compatible_with_realm_kind(graph.state.plan(), realm_kind) {
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
