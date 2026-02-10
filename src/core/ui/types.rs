use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub type UiThemeId = u32;
#[allow(dead_code)]
pub type UiFontId = u32;
#[allow(dead_code)]
pub type UiImageId = u32;
#[allow(dead_code)]
pub type UiDocumentId = u32;
#[allow(dead_code)]
pub type UiNodeId = u32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum UiThemeValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiNodeKind {
    Container,
    Text,
    Button,
    Input,
    Image,
    Separator,
    Spacer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum UiNodeProps {
    Container,
    Text { text: String },
    Button { label: String },
    Input { value: String },
    Image { image_id: UiImageId },
    Separator,
    Spacer { width: Option<f32>, height: Option<f32> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiNode {
    pub id: UiNodeId,
    pub kind: UiNodeKind,
    pub props: UiNodeProps,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum UiOp {
    Add {
        parent: Option<UiNodeId>,
        node: UiNode,
        index: Option<u32>,
    },
    Remove {
        node_id: UiNodeId,
    },
    Clear {
        parent: Option<UiNodeId>,
    },
    Set {
        node_id: UiNodeId,
        props: UiNodeProps,
    },
    Move {
        node_id: UiNodeId,
        new_parent: Option<UiNodeId>,
        index: Option<u32>,
    },
}
