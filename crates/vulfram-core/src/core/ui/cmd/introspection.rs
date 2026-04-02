use crate::core::realm::RealmId;
use crate::core::state::EngineState;
pub use vulfram_realm_ui::{
    CmdResultUiDocumentGetLayoutRects, CmdResultUiDocumentGetTree, CmdResultUiEventTraceSet,
    CmdResultUiFocusGet, CmdResultUiFocusSet, CmdUiDocumentGetLayoutRectsArgs,
    CmdUiDocumentGetTreeArgs, CmdUiEventTraceSetArgs, CmdUiFocusGetArgs, CmdUiFocusSetArgs,
    UiFocusEntry, UiNodeLayoutRect, build_tree_node,
};

pub fn engine_cmd_ui_document_get_tree(
    engine: &mut EngineState,
    args: &CmdUiDocumentGetTreeArgs,
) -> CmdResultUiDocumentGetTree {
    let Some(doc) = engine
        .universal_state
        .ui
        .documents
        .get_mut(&args.document_id)
    else {
        return CmdResultUiDocumentGetTree {
            success: false,
            message: format!("UiDocument {} not found", args.document_id),
            ..Default::default()
        };
    };
    doc.ensure_layout_cache();
    let root_nodes = doc
        .ordered_root
        .iter()
        .filter_map(|node_id| build_tree_node(doc, *node_id))
        .collect();
    CmdResultUiDocumentGetTree {
        success: true,
        message: "UI document tree returned".into(),
        document_id: Some(args.document_id),
        version: Some(doc.version),
        root_nodes,
    }
}

pub fn engine_cmd_ui_document_get_layout_rects(
    engine: &mut EngineState,
    args: &CmdUiDocumentGetLayoutRectsArgs,
) -> CmdResultUiDocumentGetLayoutRects {
    let Some(doc) = engine.universal_state.ui.documents.get(&args.document_id) else {
        return CmdResultUiDocumentGetLayoutRects {
            success: false,
            message: format!("UiDocument {} not found", args.document_id),
            ..Default::default()
        };
    };
    let mut rects: Vec<UiNodeLayoutRect> = engine
        .universal_state
        .ui
        .layout_rects
        .iter()
        .filter(|((document_id, _), _)| *document_id == args.document_id)
        .map(|((_, node_id), rect)| UiNodeLayoutRect {
            node_id: *node_id,
            rect: *rect,
        })
        .collect();
    rects.sort_by_key(|entry| entry.node_id);
    CmdResultUiDocumentGetLayoutRects {
        success: true,
        message: "UI layout rects returned".into(),
        document_id: Some(args.document_id),
        version: Some(doc.version),
        rects,
    }
}

pub fn engine_cmd_ui_focus_set(
    engine: &mut EngineState,
    args: &CmdUiFocusSetArgs,
) -> CmdResultUiFocusSet {
    let realm_id = RealmId(args.realm_id);
    if engine.universal_state.realms.get(realm_id).is_none() {
        return CmdResultUiFocusSet {
            success: false,
            message: format!("Realm {} not found", args.realm_id),
        };
    }
    let Some(document) = engine.universal_state.ui.documents.get(&args.document_id) else {
        return CmdResultUiFocusSet {
            success: false,
            message: format!("UiDocument {} not found", args.document_id),
        };
    };
    if document.realm_id != realm_id {
        return CmdResultUiFocusSet {
            success: false,
            message: format!(
                "UiDocument {} does not belong to realm {}",
                args.document_id, args.realm_id
            ),
        };
    }
    if let Some(node_id) = args.node_id {
        if node_id != 0 && !document.nodes.contains_key(&node_id) {
            return CmdResultUiFocusSet {
                success: false,
                message: format!(
                    "UiNode {} not found in document {}",
                    node_id, args.document_id
                ),
            };
        }
    }

    let node_id = args.node_id.unwrap_or(0);
    let ui_state = &mut engine.universal_state.ui;
    ui_state.focus.set_focus(vulfram_realm_ui::UiFocusUpdate {
        window_id: args.window_id,
        realm_id,
        document_id: args.document_id,
        node_id,
    });

    CmdResultUiFocusSet {
        success: true,
        message: "UI focus updated".into(),
    }
}

pub fn engine_cmd_ui_focus_get(
    engine: &mut EngineState,
    args: &CmdUiFocusGetArgs,
) -> CmdResultUiFocusGet {
    let ui_state = &engine.universal_state.ui;
    let mut entries = Vec::new();
    for (window_id, realm_id) in &ui_state.focus.realm_by_window {
        if args.window_id.is_some() && args.window_id != Some(*window_id) {
            continue;
        }
        let document_id = ui_state
            .focus
            .focus_document(*window_id)
            .unwrap_or_default();
        let node_id = ui_state.focus.focus_node(*window_id).unwrap_or_default();
        entries.push(UiFocusEntry {
            window_id: *window_id,
            realm_id: realm_id.0,
            document_id,
            node_id,
        });
    }
    entries.sort_by_key(|entry| entry.window_id);

    CmdResultUiFocusGet {
        success: true,
        message: "UI focus state returned".into(),
        entries,
    }
}

pub fn engine_cmd_ui_event_trace_set(
    engine: &mut EngineState,
    args: &CmdUiEventTraceSetArgs,
) -> CmdResultUiEventTraceSet {
    let trace = &mut engine.universal_state.input_routing.trace;
    if let Some(level) = args.level {
        trace.level = level;
    }
    if let Some(sampling_percent) = args.sampling_percent {
        trace.sampling_percent = sampling_percent.min(100);
    }
    CmdResultUiEventTraceSet {
        success: true,
        message: "UI event trace config updated".into(),
        level: Some(trace.level),
        sampling_percent: Some(trace.sampling_percent),
    }
}
