use std::collections::HashSet;

use crate::core::state::EngineState;
use crate::core::ui::state::{UiDocument, UiState};
pub use vulfram_realm_ui::{
    CmdResultUiApplyOps, CmdResultUiDocumentCreate, CmdResultUiDocumentDispose,
    CmdResultUiDocumentSetRect, CmdResultUiDocumentSetTheme, CmdUiApplyOpsArgs,
    CmdUiDocumentCreateArgs, CmdUiDocumentDisposeArgs, CmdUiDocumentSetRectArgs,
    CmdUiDocumentSetThemeArgs, UiDocumentId, UiNodeId,
};

pub fn engine_cmd_ui_document_create(
    engine: &mut EngineState,
    args: &CmdUiDocumentCreateArgs,
) -> CmdResultUiDocumentCreate {
    let realm_id = crate::core::realm::RealmId(args.realm_id);
    let ui_state = &mut engine.universal_state.ui;
    ui_state.ensure_realm(realm_id);
    if ui_state.documents.contains_key(&args.document_id) {
        return CmdResultUiDocumentCreate {
            success: false,
            message: format!("UiDocument {} already exists", args.document_id),
            document_id: None,
        };
    }
    let mut doc = UiDocument::new(args.document_id, realm_id, args.rect);
    doc.theme_id = args.theme_id;
    ui_state.documents.insert(args.document_id, doc);
    CmdResultUiDocumentCreate {
        success: true,
        message: "UI document created".into(),
        document_id: Some(args.document_id),
    }
}

pub fn engine_cmd_ui_document_dispose(
    engine: &mut EngineState,
    args: &CmdUiDocumentDisposeArgs,
) -> CmdResultUiDocumentDispose {
    let ui_state = &mut engine.universal_state.ui;
    if !ui_state.remove_document(args.document_id) {
        return CmdResultUiDocumentDispose {
            success: false,
            message: format!("UiDocument {} not found", args.document_id),
        };
    }
    CmdResultUiDocumentDispose {
        success: true,
        message: "UI document disposed".into(),
    }
}

pub fn engine_cmd_ui_document_set_rect(
    engine: &mut EngineState,
    args: &CmdUiDocumentSetRectArgs,
) -> CmdResultUiDocumentSetRect {
    let ui_state = &mut engine.universal_state.ui;
    let Some(doc) = ui_state.documents.get_mut(&args.document_id) else {
        return CmdResultUiDocumentSetRect {
            success: false,
            message: format!("UiDocument {} not found", args.document_id),
        };
    };
    doc.rect = args.rect;
    CmdResultUiDocumentSetRect {
        success: true,
        message: "UI document rect updated".into(),
    }
}

pub fn engine_cmd_ui_document_set_theme(
    engine: &mut EngineState,
    args: &CmdUiDocumentSetThemeArgs,
) -> CmdResultUiDocumentSetTheme {
    let ui_state = &mut engine.universal_state.ui;
    let Some(doc) = ui_state.documents.get_mut(&args.document_id) else {
        return CmdResultUiDocumentSetTheme {
            success: false,
            message: format!("UiDocument {} not found", args.document_id),
        };
    };
    doc.theme_id = args.theme_id;
    CmdResultUiDocumentSetTheme {
        success: true,
        message: "UI document theme updated".into(),
    }
}

pub fn engine_cmd_ui_apply_ops(
    engine: &mut EngineState,
    args: &CmdUiApplyOpsArgs,
) -> CmdResultUiApplyOps {
    let ui_state = &mut engine.universal_state.ui;
    let Some(current_doc) = ui_state.documents.get(&args.document_id) else {
        return CmdResultUiApplyOps {
            success: false,
            message: format!("UiDocument {} not found", args.document_id),
            version: None,
        };
    };
    if args.version <= current_doc.version {
        return CmdResultUiApplyOps {
            success: false,
            message: format!(
                "UiDocument {} version mismatch (current={}, incoming={})",
                args.document_id, current_doc.version, args.version
            ),
            version: Some(current_doc.version),
        };
    }
    let current_version = current_doc.version;
    let Some(doc) = ui_state.documents.get_mut(&args.document_id) else {
        return CmdResultUiApplyOps {
            success: false,
            message: format!("UiDocument {} not found", args.document_id),
            version: None,
        };
    };
    let removed_nodes = match doc.apply_ops(args.version, &args.ops) {
        Ok(result) => result.removed_nodes.into_iter().collect::<HashSet<_>>(),
        Err(result) => {
            return CmdResultUiApplyOps {
                success: false,
                message: result.message,
                version: result.version.or(Some(current_version)),
            };
        }
    };
    prune_removed_nodes(ui_state, args.document_id, &removed_nodes);

    CmdResultUiApplyOps {
        success: true,
        message: "UI ops applied".into(),
        version: Some(args.version),
    }
}

fn prune_removed_nodes(
    ui_state: &mut UiState,
    document_id: UiDocumentId,
    removed: &HashSet<UiNodeId>,
) {
    if removed.is_empty() {
        return;
    }
    ui_state
        .input_buffers
        .retain(|(entry_doc, node_id), _| *entry_doc != document_id || !removed.contains(node_id));
    ui_state
        .bool_values
        .retain(|(entry_doc, node_id), _| *entry_doc != document_id || !removed.contains(node_id));
    ui_state
        .number_values
        .retain(|(entry_doc, node_id), _| *entry_doc != document_id || !removed.contains(node_id));
    ui_state
        .selection_values
        .retain(|(entry_doc, node_id), _| *entry_doc != document_id || !removed.contains(node_id));
    ui_state
        .layout_rects
        .retain(|(entry_doc, node_id), _| *entry_doc != document_id || !removed.contains(node_id));
    ui_state
        .animations
        .retain(|key, _| key.document_id != document_id || !removed.contains(&key.node_id));
    ui_state
        .split_ratios
        .retain(|(entry_doc, node_id), _| *entry_doc != document_id || !removed.contains(node_id));
    ui_state
        .node_open_state
        .retain(|(entry_doc, node_id), _| *entry_doc != document_id || !removed.contains(node_id));
    ui_state
        .area_positions
        .retain(|(entry_doc, node_id), _| *entry_doc != document_id || !removed.contains(node_id));
    ui_state
        .scene_state
        .retain(|(entry_doc, node_id), _| *entry_doc != document_id || !removed.contains(node_id));
    ui_state.focus_node_by_window.retain(|window_id, node_id| {
        if *node_id == 0 || !removed.contains(node_id) {
            return true;
        }
        ui_state.focus_document_by_window.get(window_id) != Some(&document_id)
    });
    ui_state
        .capture_by_window
        .retain(|_, (_, capture_doc, node_id)| {
            *capture_doc != document_id || !removed.contains(node_id)
        });
}
