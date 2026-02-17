use serde::{Deserialize, Serialize};

use crate::core::state::EngineState;
use crate::core::ui::state::UiDocument;
use crate::core::ui::types::{UiDocumentId, UiOp, UiThemeId};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiDocumentCreateArgs {
    pub document_id: UiDocumentId,
    pub realm_id: u32,
    pub rect: glam::Vec4,
    #[serde(default)]
    pub theme_id: Option<UiThemeId>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiDocumentCreate {
    pub success: bool,
    pub message: String,
    pub document_id: Option<UiDocumentId>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiDocumentDisposeArgs {
    pub document_id: UiDocumentId,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiDocumentDispose {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiDocumentSetRectArgs {
    pub document_id: UiDocumentId,
    pub rect: glam::Vec4,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiDocumentSetRect {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiDocumentSetThemeArgs {
    pub document_id: UiDocumentId,
    pub theme_id: Option<UiThemeId>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiDocumentSetTheme {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiApplyOpsArgs {
    pub document_id: UiDocumentId,
    pub version: u64,
    pub ops: Vec<UiOp>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiApplyOps {
    pub success: bool,
    pub message: String,
    pub version: Option<u64>,
}

pub fn engine_cmd_ui_document_create(
    engine: &mut EngineState,
    args: &CmdUiDocumentCreateArgs,
) -> CmdResultUiDocumentCreate {
    let realm_id = crate::core::realm::RealmId(args.realm_id);
    if engine.universal_state.realms.get(realm_id).is_none() {
        return CmdResultUiDocumentCreate {
            success: false,
            message: format!("Realm {} not found", args.realm_id),
            document_id: None,
        };
    }
    let ui_state = &mut engine.universal_state.ui;
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
    let Some(doc) = ui_state.documents.get(&args.document_id) else {
        return CmdResultUiApplyOps {
            success: false,
            message: format!("UiDocument {} not found", args.document_id),
            version: None,
        };
    };
    if args.version <= doc.version {
        return CmdResultUiApplyOps {
            success: false,
            message: format!(
                "UiDocument {} version mismatch (current={}, incoming={})",
                args.document_id, doc.version, args.version
            ),
            version: Some(doc.version),
        };
    }

    let mut scratch = doc.clone();
    for op in &args.ops {
        let result = match op {
            UiOp::Add {
                parent,
                node,
                index,
            } => scratch.add_node(*parent, node.clone(), *index),
            UiOp::Remove { node_id } => scratch.remove_node(*node_id),
            UiOp::Clear { parent } => scratch.clear_children(*parent),
            UiOp::Set { node_id, props } => scratch.set_props(*node_id, props.clone()),
            UiOp::Move {
                node_id,
                new_parent,
                index,
            } => scratch.move_node(*node_id, *new_parent, *index),
        };
        if let Err(message) = result {
            return CmdResultUiApplyOps {
                success: false,
                message,
                version: Some(doc.version),
            };
        }
    }

    let alive_nodes: std::collections::HashSet<_> = scratch.nodes.keys().copied().collect();

    scratch.version = args.version;
    ui_state.documents.insert(args.document_id, scratch);
    ui_state.input_buffers.retain(|(document_id, node_id), _| {
        *document_id != args.document_id || alive_nodes.contains(node_id)
    });
    ui_state.bool_values.retain(|(document_id, node_id), _| {
        *document_id != args.document_id || alive_nodes.contains(node_id)
    });
    ui_state.number_values.retain(|(document_id, node_id), _| {
        *document_id != args.document_id || alive_nodes.contains(node_id)
    });
    ui_state
        .selection_values
        .retain(|(document_id, node_id), _| {
            *document_id != args.document_id || alive_nodes.contains(node_id)
        });
    ui_state
        .animations
        .retain(|key, _| key.document_id != args.document_id || alive_nodes.contains(&key.node_id));
    ui_state.split_ratios.retain(|(document_id, node_id), _| {
        *document_id != args.document_id || alive_nodes.contains(node_id)
    });
    ui_state
        .node_open_state
        .retain(|(document_id, node_id), _| {
            *document_id != args.document_id || alive_nodes.contains(node_id)
        });
    ui_state.area_positions.retain(|(document_id, node_id), _| {
        *document_id != args.document_id || alive_nodes.contains(node_id)
    });
    ui_state.scene_state.retain(|(document_id, node_id), _| {
        *document_id != args.document_id || alive_nodes.contains(node_id)
    });

    CmdResultUiApplyOps {
        success: true,
        message: "UI ops applied".into(),
        version: Some(args.version),
    }
}
