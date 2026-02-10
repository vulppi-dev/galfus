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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiLayoutDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
    Grid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiAlign {
    Start,
    Center,
    End,
    Stretch,
}

impl Default for UiAlign {
    fn default() -> Self {
        UiAlign::Start
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum UiLength {
    Auto,
    Fill,
    Px(f32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiSize {
    pub width: UiLength,
    pub height: UiLength,
}

impl Default for UiSize {
    fn default() -> Self {
        Self {
            width: UiLength::Auto,
            height: UiLength::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiLayout {
    pub direction: UiLayoutDirection,
    #[serde(default)]
    pub align: UiAlign,
    #[serde(default)]
    pub justify: UiAlign,
    #[serde(default)]
    pub gap: f32,
    #[serde(default)]
    pub columns: Option<u32>,
    #[serde(default)]
    pub wrap: bool,
    #[serde(default)]
    pub wrap_limit: Option<f32>,
}

impl Default for UiLayout {
    fn default() -> Self {
        Self {
            direction: UiLayoutDirection::Column,
            align: UiAlign::Start,
            justify: UiAlign::Start,
            gap: 0.0,
            columns: None,
            wrap: false,
            wrap_limit: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiPadding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum UiNodeProps {
    Container {
        #[serde(default)]
        layout: UiLayout,
        #[serde(default)]
        padding: Option<UiPadding>,
        #[serde(default)]
        size: Option<UiSize>,
        #[serde(default)]
        scroll_x: bool,
        #[serde(default)]
        scroll_y: bool,
    },
    Text {
        text: String,
        #[serde(default)]
        size: Option<f32>,
        #[serde(default)]
        color: Option<UiColor>,
    },
    Button {
        label: String,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Input {
        value: String,
        #[serde(default)]
        placeholder: Option<String>,
        #[serde(default)]
        enabled: Option<bool>,
    },
    Image {
        source: UiImageSource,
        #[serde(default)]
        size: Option<UiSize>,
    },
    Separator,
    Spacer {
        width: Option<f32>,
        height: Option<f32>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum UiImageSource {
    UiImage(UiImageId),
    Target(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiNode {
    pub id: UiNodeId,
    pub kind: UiNodeKind,
    pub props: UiNodeProps,
    #[serde(default)]
    pub anim: Option<UiAnim>,
    #[serde(default)]
    pub display: Option<bool>,
    #[serde(default)]
    pub visible: Option<bool>,
    #[serde(default)]
    pub opacity: Option<f32>,
    #[serde(default)]
    pub z_index: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiAnim {
    #[serde(default)]
    pub opacity: Option<UiAnimSpec>,
    #[serde(default)]
    pub translate_y: Option<UiAnimSpec>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UiAnimSpec {
    pub from: f32,
    pub to: f32,
    pub duration_ms: u32,
    #[serde(default)]
    pub easing: UiAnimEasing,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiAnimEasing {
    Linear,
    EaseInOut,
}

impl Default for UiAnimEasing {
    fn default() -> Self {
        UiAnimEasing::Linear
    }
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
