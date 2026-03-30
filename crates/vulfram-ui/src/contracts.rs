use serde::{Deserialize, Serialize};
use vulfram_input::PointerTraceLevel;

use std::collections::HashMap;

use crate::{UiDocument, UiDocumentId, UiNodeId, UiNodeKind, UiOp, UiThemeId, UiThemeValue};

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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiDocumentGetTreeArgs {
    pub document_id: UiDocumentId,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiDocumentGetTree {
    pub success: bool,
    pub message: String,
    pub document_id: Option<UiDocumentId>,
    pub version: Option<u64>,
    pub root_nodes: Vec<UiDocumentTreeNode>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDocumentTreeNode {
    pub node_id: UiNodeId,
    pub kind: UiNodeKind,
    pub z_index: i32,
    pub children: Vec<UiDocumentTreeNode>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiDocumentGetLayoutRectsArgs {
    pub document_id: UiDocumentId,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiDocumentGetLayoutRects {
    pub success: bool,
    pub message: String,
    pub document_id: Option<UiDocumentId>,
    pub version: Option<u64>,
    pub rects: Vec<UiNodeLayoutRect>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiNodeLayoutRect {
    pub node_id: UiNodeId,
    pub rect: glam::Vec4,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiFocusSetArgs {
    pub window_id: u32,
    pub realm_id: u32,
    pub document_id: UiDocumentId,
    #[serde(default)]
    pub node_id: Option<UiNodeId>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiFocusGetArgs {
    #[serde(default)]
    pub window_id: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiFocusSet {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiFocusGet {
    pub success: bool,
    pub message: String,
    pub entries: Vec<UiFocusEntry>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiFocusEntry {
    pub window_id: u32,
    pub realm_id: u32,
    pub document_id: UiDocumentId,
    pub node_id: UiNodeId,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiEventTraceSetArgs {
    #[serde(default)]
    pub level: Option<PointerTraceLevel>,
    #[serde(default)]
    pub sampling_percent: Option<u8>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiEventTraceSet {
    pub success: bool,
    pub message: String,
    pub level: Option<PointerTraceLevel>,
    pub sampling_percent: Option<u8>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiThemeDefineArgs {
    pub theme_id: UiThemeId,
    #[serde(default)]
    pub version: Option<u32>,
    #[serde(default)]
    pub data: HashMap<String, UiThemeValue>,
    #[serde(default)]
    pub font_data: HashMap<String, Vec<u8>>,
    #[serde(default)]
    pub font_families: HashMap<String, Vec<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiThemeDefine {
    pub success: bool,
    pub message: String,
    pub theme_id: Option<UiThemeId>,
    pub version: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiThemeDisposeArgs {
    pub theme_id: UiThemeId,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiThemeDispose {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdUiDebugSetArgs {
    pub enabled: bool,
    #[serde(default)]
    pub show_bounds: bool,
    #[serde(default)]
    pub show_ids: bool,
    #[serde(default)]
    pub show_profile: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultUiDebugSet {
    pub success: bool,
    pub message: String,
}

pub fn build_tree_node(doc: &UiDocument, node_id: UiNodeId) -> Option<UiDocumentTreeNode> {
    let entry = doc.nodes.get(&node_id)?;
    let mut children = Vec::new();
    let ordered_children = doc
        .ordered_children
        .get(&node_id)
        .cloned()
        .unwrap_or_else(|| entry.children.clone());
    for child_id in ordered_children {
        if let Some(child) = build_tree_node(doc, child_id) {
            children.push(child);
        }
    }
    Some(UiDocumentTreeNode {
        node_id,
        kind: entry.node.kind.clone(),
        z_index: entry.node.z_index.unwrap_or(0),
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{UiNode, UiNodeProps};
    use vulfram_types::RealmId;

    #[test]
    fn build_tree_node_uses_ordered_children_when_available() {
        let mut doc = UiDocument::new(1, RealmId(7), glam::vec4(0.0, 0.0, 100.0, 100.0));
        doc.add_node(None, text_node(10, "root", Some(0)), None)
            .expect("root should be added");
        doc.add_node(Some(10), text_node(11, "back", Some(-1)), None)
            .expect("child should be added");
        doc.add_node(Some(10), text_node(12, "front", Some(1)), None)
            .expect("child should be added");
        doc.ensure_layout_cache();

        let tree = build_tree_node(&doc, 10).expect("tree should exist");
        assert_eq!(
            tree.children
                .iter()
                .map(|child| child.node_id)
                .collect::<Vec<_>>(),
            vec![11, 12]
        );
    }

    fn text_node(node_id: UiNodeId, text: &str, z_index: Option<i32>) -> UiNode {
        UiNode {
            id: node_id,
            kind: UiNodeKind::Text,
            props: UiNodeProps::Text {
                text: text.into(),
                size: None,
                color: None,
            },
            tooltip: None,
            context_menu: None,
            anim: None,
            display: None,
            visible: None,
            opacity: None,
            z_index,
        }
    }
}
